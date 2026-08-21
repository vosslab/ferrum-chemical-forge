//! Streaming parser mechanics for the closed CML profile.

use std::collections::{BTreeMap, BTreeSet};

use xmlparser::{ElementEnd, Reference, Stream, Token, Tokenizer};

use super::*;

const CML1_NAMESPACE: &str = "http://www.xml-cml.org/schema";
const CML2_NAMESPACE: &str = "http://www.xml-cml.org/schema/cml2/core";
const MAX_INPUT_BYTES: usize = 1_048_576;
const MAX_TEXT_BYTES: usize = 1_048_576;
const MAX_DECLARATION_BYTES: usize = 256;
const MAX_COMMENT_BYTES: usize = 65_536;
const MAX_PI_BYTES: usize = 8_192;
const MAX_ELEMENTS: usize = 50_000;
const MAX_DEPTH: usize = 8;
const MAX_ATTRIBUTES: usize = 8;
const MAX_ATTRIBUTE_VALUE_BYTES: usize = 256;
const MAX_RECORDS: usize = 1_024;
const MAX_ATOMS_PER_RECORD: usize = 10_000;
const MAX_ATOMS: usize = 100_000;
const MAX_BONDS_PER_RECORD: usize = 20_000;
const MAX_BONDS: usize = 200_000;
const MAX_SOURCE_IDS: usize = 101_024;
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_COORDINATE: f64 = 100_000.0;

#[derive(Clone, Copy, Eq, PartialEq)]
enum Profile {
    Cml1,
    Cml2,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Frame {
    Root(Profile),
    Molecule,
    AtomArray,
    BondArray,
    Atom,
    Bond,
    Builtin(&'static str),
}

struct Pending {
    name: String,
    attributes: Vec<(String, String)>,
}
struct Builtin {
    name: &'static str,
    value: String,
    has_text: bool,
}
struct RecordBuilder {
    source_molecule_id: Option<String>,
    atoms: Vec<CmlSourceAtomV1>,
    bonds: Vec<CmlSourceBondV1>,
    atom_indexes: BTreeMap<String, usize>,
    atom_array_seen: bool,
    bond_array_seen: bool,
    current_atom: BTreeMap<&'static str, String>,
    current_bond: BTreeMap<&'static str, String>,
}

impl RecordBuilder {
    fn new(source_molecule_id: Option<String>) -> Self {
        Self {
            source_molecule_id,
            atoms: Vec::new(),
            bonds: Vec::new(),
            atom_indexes: BTreeMap::new(),
            atom_array_seen: false,
            bond_array_seen: false,
            current_atom: BTreeMap::new(),
            current_bond: BTreeMap::new(),
        }
    }
    fn add_atom(
        &mut self,
        fields: BTreeMap<&'static str, String>,
        total_atoms: &mut usize,
        source_ids: &mut usize,
    ) -> Result<()> {
        for field in ["id", "elementType", "x2", "y2"] {
            if !fields.contains_key(field) {
                return refused(CmlRefusalReasonV1::InvalidScalar);
            }
        }
        if self.atoms.len() >= MAX_ATOMS_PER_RECORD {
            return refused(CmlRefusalReasonV1::AtomsPerRecordLimit);
        }
        *total_atoms = total_atoms.checked_add(1).ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::AtomLimit,
        })?;
        if *total_atoms > MAX_ATOMS {
            return refused(CmlRefusalReasonV1::AtomLimit);
        }
        let id = fields["id"].clone();
        validate_id(&id)?;
        if self.atom_indexes.contains_key(&id) {
            return refused(CmlRefusalReasonV1::DuplicateAtomId);
        }
        *source_ids = add_budget(
            *source_ids,
            1,
            MAX_SOURCE_IDS,
            CmlRefusalReasonV1::SourceIdMapLimit,
        )?;
        let element =
            AtomicNumber::from_symbol(&fields["elementType"]).map_err(|_| CmlDecoderErrorV1 {
                reason: CmlRefusalReasonV1::InvalidScalar,
            })?;
        let x2 = coordinate(&fields["x2"])?;
        let y2 = coordinate(&fields["y2"])?;
        let formal_charge = fields
            .get("formalCharge")
            .map(|value| signed(value, -8, 8))
            .transpose()?;
        let isotope = fields
            .get("isotopeNumber")
            .map(|value| unsigned(value, 1, 400))
            .transpose()?
            .map(|value| value as u16);
        let index = self.atoms.len();
        self.atom_indexes.insert(id.clone(), index);
        self.atoms.push(CmlSourceAtomV1 {
            source_id: id,
            element,
            formal_charge,
            isotope,
            x2,
            y2,
        });
        Ok(())
    }
    fn add_bond(
        &mut self,
        fields: BTreeMap<&'static str, String>,
        total_bonds: &mut usize,
    ) -> Result<()> {
        for field in ["atomRefs2", "order"] {
            if !fields.contains_key(field) {
                return refused(CmlRefusalReasonV1::InvalidScalar);
            }
        }
        if self.bonds.len() >= MAX_BONDS_PER_RECORD {
            return refused(CmlRefusalReasonV1::BondsPerRecordLimit);
        }
        *total_bonds = total_bonds.checked_add(1).ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::BondLimit,
        })?;
        if *total_bonds > MAX_BONDS {
            return refused(CmlRefusalReasonV1::BondLimit);
        }
        let refs = fields["atomRefs2"]
            .split_ascii_whitespace()
            .collect::<Vec<_>>();
        if refs.len() != 2 {
            return refused(CmlRefusalReasonV1::InvalidScalar);
        }
        validate_id(refs[0])?;
        validate_id(refs[1])?;
        let start = *self.atom_indexes.get(refs[0]).ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::DanglingBond,
        })?;
        let end = *self.atom_indexes.get(refs[1]).ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::DanglingBond,
        })?;
        if start == end {
            return refused(CmlRefusalReasonV1::SelfBond);
        }
        if self.bonds.iter().any(|bond| {
            (bond.start.min(bond.end), bond.start.max(bond.end)) == (start.min(end), start.max(end))
        }) {
            return refused(CmlRefusalReasonV1::DuplicateBond);
        }
        let order = match fields["order"].as_str() {
            "1" | "S" => BondOrder::Single,
            "2" | "D" => BondOrder::Double,
            "3" | "T" => BondOrder::Triple,
            _ => return refused(CmlRefusalReasonV1::InvalidScalar),
        };
        self.bonds.push(CmlSourceBondV1 { start, end, order });
        Ok(())
    }
    fn finish(self) -> Result<CmlDecodedRecordV1> {
        if !self.atom_array_seen || self.atoms.is_empty() {
            return refused(CmlRefusalReasonV1::InvalidGraph);
        }
        Ok(CmlDecodedRecordV1 {
            source_molecule_id: self.source_molecule_id,
            atoms: self.atoms,
            bonds: self.bonds,
        })
    }
}

/// Decode one UTF-8 CML1 or CML2 document into bounded owned source graphs.
///
/// The input is bytes so non-UTF-8 input can receive the closed `invalid_utf8`
/// refusal before XML tokenization or retained graph allocation.
pub fn decode_cml_bytes_v1(input: &[u8]) -> Result<CmlDecodedDocumentV1> {
    let input = input.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(input);
    if input.len() > MAX_INPUT_BYTES {
        return refused(CmlRefusalReasonV1::InputBytesLimit);
    }
    let source = std::str::from_utf8(input).map_err(|_| CmlDecoderErrorV1 {
        reason: CmlRefusalReasonV1::InvalidUtf8,
    })?;
    let mut parser = Parser::new();
    for token in Tokenizer::from(source) {
        parser.token(token.map_err(|_| CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InvalidXml,
        })?)?;
    }
    parser.finish()
}

struct Parser {
    stack: Vec<Frame>,
    pending: Option<Pending>,
    builtin: Option<Builtin>,
    profile: Option<Profile>,
    records: Vec<CmlDecodedRecordV1>,
    record: Option<RecordBuilder>,
    molecule_ids: BTreeSet<String>,
    declaration_seen: bool,
    first_token: bool,
    semantic_seen: bool,
    root_closed: bool,
    text_bytes: usize,
    comment_bytes: usize,
    pi_bytes: usize,
    elements: usize,
    total_atoms: usize,
    total_bonds: usize,
    source_ids: usize,
}

impl Parser {
    fn new() -> Self {
        Self {
            stack: Vec::new(),
            pending: None,
            builtin: None,
            profile: None,
            records: Vec::new(),
            record: None,
            molecule_ids: BTreeSet::new(),
            declaration_seen: false,
            first_token: true,
            semantic_seen: false,
            root_closed: false,
            text_bytes: 0,
            comment_bytes: 0,
            pi_bytes: 0,
            elements: 0,
            total_atoms: 0,
            total_bonds: 0,
            source_ids: 0,
        }
    }
    fn token(&mut self, token: Token<'_>) -> Result<()> {
        let first_token = self.first_token;
        self.first_token = false;
        match token {
            Token::Declaration {
                version,
                encoding,
                standalone,
                span,
            } => {
                if !first_token
                    || self.declaration_seen
                    || self.semantic_seen
                    || !self.stack.is_empty()
                    || span.as_str().len() > MAX_DECLARATION_BYTES
                {
                    return refused(CmlRefusalReasonV1::InvalidXmlDeclaration);
                }
                if version.as_str() != "1.0"
                    || encoding.map(|value| value.as_str()) != Some("UTF-8")
                    || standalone.is_some()
                {
                    return refused(CmlRefusalReasonV1::InvalidXmlDeclaration);
                }
                self.declaration_seen = true;
            }
            Token::DtdStart { external_id, .. } | Token::EmptyDtd { external_id, .. } => {
                return refused(if external_id.is_some() {
                    CmlRefusalReasonV1::ExternalResourceForbidden
                } else {
                    CmlRefusalReasonV1::DtdForbidden
                });
            }
            Token::EntityDeclaration { .. } | Token::DtdEnd { .. } => {
                return refused(CmlRefusalReasonV1::DtdForbidden);
            }
            Token::ElementStart { prefix, local, .. } => {
                self.semantic_seen = true;
                if self.pending.is_some() || self.root_closed {
                    return refused(CmlRefusalReasonV1::InvalidXml);
                }
                if !prefix.as_str().is_empty() {
                    return refused(if prefix.as_str() == "xi" {
                        CmlRefusalReasonV1::XincludeForbidden
                    } else {
                        CmlRefusalReasonV1::NamespaceUnsupported
                    });
                }
                self.elements = self.elements.checked_add(1).ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::XmlElementLimit,
                })?;
                if self.elements > MAX_ELEMENTS {
                    return refused(CmlRefusalReasonV1::XmlElementLimit);
                }
                if self.stack.len() >= MAX_DEPTH {
                    return refused(CmlRefusalReasonV1::XmlDepthLimit);
                }
                self.pending = Some(Pending {
                    name: local.as_str().to_owned(),
                    attributes: Vec::new(),
                });
            }
            Token::Attribute {
                prefix,
                local,
                value,
                ..
            } => {
                let pending = self.pending.as_mut().ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::InvalidXml,
                })?;
                if pending.attributes.len() >= MAX_ATTRIBUTES {
                    return refused(CmlRefusalReasonV1::XmlAttributeLimit);
                }
                if !prefix.as_str().is_empty() {
                    return refused(CmlRefusalReasonV1::NamespaceUnsupported);
                }
                let decoded = decode_entities(
                    value.as_str(),
                    MAX_ATTRIBUTE_VALUE_BYTES,
                    CmlRefusalReasonV1::AttributeValueLimit,
                )?;
                let name = local.as_str().to_owned();
                if pending
                    .attributes
                    .iter()
                    .any(|(existing, _)| existing == &name)
                {
                    return refused(CmlRefusalReasonV1::AttributeUnsupported);
                }
                pending.attributes.push((name, decoded));
            }
            Token::ElementEnd { end, .. } => match end {
                ElementEnd::Open => self.open()?,
                ElementEnd::Empty => {
                    self.open()?;
                    self.close()?;
                }
                ElementEnd::Close(prefix, local) => {
                    if !prefix.as_str().is_empty() {
                        return refused(CmlRefusalReasonV1::NamespaceUnsupported);
                    }
                    if self
                        .stack
                        .last()
                        .is_none_or(|frame| frame_name(*frame) != local.as_str())
                    {
                        return refused(CmlRefusalReasonV1::InvalidXml);
                    }
                    self.close()?;
                }
            },
            Token::Text { text } => {
                self.semantic_seen = true;
                let remaining =
                    MAX_TEXT_BYTES
                        .checked_sub(self.text_bytes)
                        .ok_or(CmlDecoderErrorV1 {
                            reason: CmlRefusalReasonV1::XmlTextBytesLimit,
                        })?;
                let decoded = decode_entities(
                    text.as_str(),
                    remaining,
                    CmlRefusalReasonV1::XmlTextBytesLimit,
                )?;
                self.text_bytes += decoded.len();
                if let Some(builtin) = self.builtin.as_mut() {
                    if builtin.has_text {
                        return refused(CmlRefusalReasonV1::UnexpectedXmlText);
                    }
                    builtin.value = decoded;
                    builtin.has_text = true;
                } else if !decoded.chars().all(char::is_whitespace) {
                    return refused(CmlRefusalReasonV1::UnexpectedXmlText);
                }
            }
            Token::Cdata { .. } => return refused(CmlRefusalReasonV1::UnexpectedXmlNode),
            Token::Comment { span, .. } => {
                self.comment_bytes = add_budget(
                    self.comment_bytes,
                    span.as_str().len(),
                    MAX_COMMENT_BYTES,
                    CmlRefusalReasonV1::CommentBytesLimit,
                )?;
                self.leaf_node_forbidden()?;
            }
            Token::ProcessingInstruction { target, span, .. } => {
                if target.as_str() == "xml-stylesheet" {
                    return refused(CmlRefusalReasonV1::StylesheetForbidden);
                }
                self.pi_bytes = add_budget(
                    self.pi_bytes,
                    span.as_str().len(),
                    MAX_PI_BYTES,
                    CmlRefusalReasonV1::PiBytesLimit,
                )?;
                self.leaf_node_forbidden()?;
            }
        }
        Ok(())
    }
    fn leaf_node_forbidden(&self) -> Result<()> {
        if matches!(
            self.stack.last(),
            Some(Frame::Atom | Frame::Bond | Frame::Builtin(_))
        ) {
            refused(CmlRefusalReasonV1::UnexpectedXmlNode)
        } else {
            Ok(())
        }
    }
    fn open(&mut self) -> Result<()> {
        let pending = self.pending.take().ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InvalidXml,
        })?;
        let parent = self.stack.last().copied();
        let profile = self.profile;
        let frame = match parent {
            None => self.open_root(pending)?,
            Some(Frame::Root(_)) => {
                self.require_name(&pending, "molecule")?;
                self.open_molecule(pending)?;
                Frame::Molecule
            }
            Some(Frame::Molecule) => self.open_molecule_child(pending)?,
            Some(Frame::AtomArray) => {
                self.require_name(&pending, "atom")?;
                self.open_atom(
                    pending,
                    profile.ok_or(CmlDecoderErrorV1 {
                        reason: CmlRefusalReasonV1::InternalFailure,
                    })?,
                )?
            }
            Some(Frame::BondArray) => {
                self.require_name(&pending, "bond")?;
                self.open_bond(
                    pending,
                    profile.ok_or(CmlDecoderErrorV1 {
                        reason: CmlRefusalReasonV1::InternalFailure,
                    })?,
                )?
            }
            Some(Frame::Atom | Frame::Bond) if profile == Some(Profile::Cml2) => {
                return refused(CmlRefusalReasonV1::UnrepresentedSemanticFact);
            }
            Some(Frame::Atom) => self.open_builtin(pending, "atom")?,
            Some(Frame::Bond) => self.open_builtin(pending, "bond")?,
            Some(Frame::Builtin(_)) => return refused(CmlRefusalReasonV1::UnexpectedXmlNode),
        };
        self.stack.push(frame);
        Ok(())
    }
    fn open_root(&mut self, pending: Pending) -> Result<Frame> {
        if pending.name != "cml" {
            return refused(CmlRefusalReasonV1::RootUnsupported);
        }
        let namespace = only_default_namespace(&pending)?;
        let profile = match namespace {
            CML1_NAMESPACE => Profile::Cml1,
            CML2_NAMESPACE => Profile::Cml2,
            _ => return refused(CmlRefusalReasonV1::NamespaceUnsupported),
        };
        self.profile = Some(profile);
        Ok(Frame::Root(profile))
    }
    fn open_molecule(&mut self, pending: Pending) -> Result<()> {
        let id = optional_id(&pending)?;
        if let Some(value) = &id {
            validate_id(value)?;
            if !self.molecule_ids.insert(value.clone()) {
                return refused(CmlRefusalReasonV1::DuplicateSourceId);
            }
            self.source_ids = add_budget(
                self.source_ids,
                1,
                MAX_SOURCE_IDS,
                CmlRefusalReasonV1::SourceIdMapLimit,
            )?;
        }
        if self.records.len() >= MAX_RECORDS {
            return refused(CmlRefusalReasonV1::RecordLimit);
        }
        self.record = Some(RecordBuilder::new(id));
        Ok(())
    }
    fn open_molecule_child(&mut self, pending: Pending) -> Result<Frame> {
        require_no_attributes(&pending)?;
        let record = self.record.as_mut().ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InternalFailure,
        })?;
        match pending.name.as_str() {
            "atomArray" if !record.atom_array_seen => {
                record.atom_array_seen = true;
                Ok(Frame::AtomArray)
            }
            "bondArray" if record.atom_array_seen && !record.bond_array_seen => {
                record.bond_array_seen = true;
                Ok(Frame::BondArray)
            }
            _ => refused(CmlRefusalReasonV1::UnrepresentedSemanticFact),
        }
    }
    fn open_atom(&mut self, pending: Pending, profile: Profile) -> Result<Frame> {
        let record = self.record.as_mut().ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InternalFailure,
        })?;
        match profile {
            Profile::Cml1 => {
                require_no_attributes(&pending)?;
                record.current_atom.clear();
                Ok(Frame::Atom)
            }
            Profile::Cml2 => {
                let fields = fields(
                    &pending,
                    &[
                        "id",
                        "elementType",
                        "x2",
                        "y2",
                        "formalCharge",
                        "isotopeNumber",
                    ],
                )?;
                record.add_atom(fields, &mut self.total_atoms, &mut self.source_ids)?;
                Ok(Frame::Atom)
            }
        }
    }
    fn open_bond(&mut self, pending: Pending, profile: Profile) -> Result<Frame> {
        let record = self.record.as_mut().ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InternalFailure,
        })?;
        match profile {
            Profile::Cml1 => {
                require_no_attributes(&pending)?;
                record.current_bond.clear();
                Ok(Frame::Bond)
            }
            Profile::Cml2 => {
                let fields = fields(&pending, &["atomRefs2", "order"])?;
                record.add_bond(fields, &mut self.total_bonds)?;
                Ok(Frame::Bond)
            }
        }
    }
    fn open_builtin(&mut self, pending: Pending, target: &str) -> Result<Frame> {
        self.require_name(&pending, "builtin")?;
        let name = fields(&pending, &["builtin"])?
            .remove("builtin")
            .ok_or(CmlDecoderErrorV1 {
                reason: CmlRefusalReasonV1::InvalidScalar,
            })?;
        let allowed = match target {
            "atom" => [
                "atomId",
                "elementType",
                "x2",
                "y2",
                "formalCharge",
                "isotopeNumber",
            ]
            .as_slice(),
            _ => ["atomRef", "order"].as_slice(),
        };
        if !allowed.contains(&name.as_str()) {
            return refused(CmlRefusalReasonV1::UnrepresentedSemanticFact);
        }
        let static_name = match name.as_str() {
            "atomId" => "atomId",
            "elementType" => "elementType",
            "x2" => "x2",
            "y2" => "y2",
            "formalCharge" => "formalCharge",
            "isotopeNumber" => "isotopeNumber",
            "atomRef" => "atomRef",
            "order" => "order",
            _ => return refused(CmlRefusalReasonV1::UnrepresentedSemanticFact),
        };
        let present = match target {
            "atom" => {
                &self
                    .record
                    .as_ref()
                    .ok_or(CmlDecoderErrorV1 {
                        reason: CmlRefusalReasonV1::InternalFailure,
                    })?
                    .current_atom
            }
            _ => {
                &self
                    .record
                    .as_ref()
                    .ok_or(CmlDecoderErrorV1 {
                        reason: CmlRefusalReasonV1::InternalFailure,
                    })?
                    .current_bond
            }
        };
        let ordered = if target == "atom" {
            !present.contains_key(static_name)
                && matches!(
                    (static_name, present.len()),
                    ("atomId", 0)
                        | ("elementType", 1)
                        | ("x2", 2)
                        | ("y2", 3)
                        | ("formalCharge", 4)
                        | ("isotopeNumber", 4 | 5)
                )
        } else {
            matches!(
                (static_name, present.len()),
                ("atomRef", 0 | 1) | ("order", 2)
            )
        };
        if !ordered {
            return refused(CmlRefusalReasonV1::ProfileMismatch);
        }
        self.builtin = Some(Builtin {
            name: static_name,
            value: String::new(),
            has_text: false,
        });
        Ok(Frame::Builtin(static_name))
    }
    fn require_name(&self, pending: &Pending, expected: &str) -> Result<()> {
        if pending.name == expected {
            Ok(())
        } else {
            refused(CmlRefusalReasonV1::UnrepresentedSemanticFact)
        }
    }
    fn close(&mut self) -> Result<()> {
        let frame = self.stack.pop().ok_or(CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InvalidXml,
        })?;
        match frame {
            Frame::Builtin(name) => {
                let builtin = self.builtin.take().ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::InternalFailure,
                })?;
                if !builtin.has_text || builtin.value.is_empty() || builtin.name != name {
                    return refused(CmlRefusalReasonV1::InvalidScalar);
                }
                let record = self.record.as_mut().ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::InternalFailure,
                })?;
                let target = self.stack.last().copied().ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::InvalidXml,
                })?;
                match target {
                    Frame::Atom => {
                        record.current_atom.insert(name, builtin.value);
                    }
                    Frame::Bond => {
                        if name == "atomRef" {
                            let count = record.current_bond.len();
                            record.current_bond.insert(
                                if count == 0 { "atomRef1" } else { "atomRef2" },
                                builtin.value,
                            );
                        } else {
                            record.current_bond.insert(name, builtin.value);
                        }
                    }
                    _ => return refused(CmlRefusalReasonV1::InvalidXml),
                }
            }
            Frame::Atom if self.profile == Some(Profile::Cml1) => {
                let record = self.record.as_mut().ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::InternalFailure,
                })?;
                let mut fields = std::mem::take(&mut record.current_atom);
                if let Some(id) = fields.remove("atomId") {
                    fields.insert("id", id);
                }
                record.add_atom(fields, &mut self.total_atoms, &mut self.source_ids)?;
            }
            Frame::Bond if self.profile == Some(Profile::Cml1) => {
                let record = self.record.as_mut().ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::InternalFailure,
                })?;
                let mut fields = std::mem::take(&mut record.current_bond);
                let first = fields.remove("atomRef1");
                let second = fields.remove("atomRef2");
                match (first, second) {
                    (Some(first), Some(second)) => {
                        fields.insert("atomRefs2", format!("{first} {second}"));
                    }
                    _ => return refused(CmlRefusalReasonV1::InvalidScalar),
                };
                record.add_bond(fields, &mut self.total_bonds)?;
            }
            Frame::Molecule => {
                let record = self
                    .record
                    .take()
                    .ok_or(CmlDecoderErrorV1 {
                        reason: CmlRefusalReasonV1::InternalFailure,
                    })?
                    .finish()?;
                self.records.push(record);
            }
            Frame::Root(_) => self.root_closed = true,
            Frame::AtomArray | Frame::BondArray | Frame::Atom | Frame::Bond => {}
        }
        Ok(())
    }
    fn finish(self) -> Result<CmlDecodedDocumentV1> {
        if !self.stack.is_empty() || self.pending.is_some() || !self.root_closed {
            return refused(CmlRefusalReasonV1::InvalidXml);
        }
        if self.records.is_empty() {
            return refused(CmlRefusalReasonV1::EmptyDocument);
        }
        Ok(CmlDecodedDocumentV1 {
            records: self.records,
        })
    }
}

fn frame_name(frame: Frame) -> &'static str {
    match frame {
        Frame::Root(_) => "cml",
        Frame::Molecule => "molecule",
        Frame::AtomArray => "atomArray",
        Frame::BondArray => "bondArray",
        Frame::Atom => "atom",
        Frame::Bond => "bond",
        Frame::Builtin(_) => "builtin",
    }
}
fn add_budget(
    current: usize,
    increment: usize,
    limit: usize,
    reason: CmlRefusalReasonV1,
) -> Result<usize> {
    let value = current
        .checked_add(increment)
        .ok_or(CmlDecoderErrorV1 { reason })?;
    if value > limit {
        refused(reason)
    } else {
        Ok(value)
    }
}
fn only_default_namespace(pending: &Pending) -> Result<&str> {
    if pending.attributes.len() != 1 || pending.attributes[0].0 != "xmlns" {
        return refused(CmlRefusalReasonV1::NamespaceUnsupported);
    }
    Ok(&pending.attributes[0].1)
}
fn require_no_attributes(pending: &Pending) -> Result<()> {
    if pending.attributes.is_empty() {
        Ok(())
    } else if pending.attributes[0].0 == "xmlns" {
        refused(CmlRefusalReasonV1::NamespaceUnsupported)
    } else {
        attribute_refusal(&pending.attributes[0].0)
    }
}
fn optional_id(pending: &Pending) -> Result<Option<String>> {
    if pending.attributes.iter().any(|(name, _)| name == "xmlns") {
        return refused(CmlRefusalReasonV1::NamespaceUnsupported);
    }
    if pending.attributes.is_empty() {
        return Ok(None);
    }
    if pending.attributes.len() == 1 && pending.attributes[0].0 == "id" {
        return Ok(Some(pending.attributes[0].1.clone()));
    }
    if [
        "atomID",
        "elementType",
        "x2",
        "y2",
        "formalCharge",
        "isotopeNumber",
        "atomRefs2",
        "bondID",
        "order",
    ]
    .contains(&pending.attributes[0].0.as_str())
    {
        refused(CmlRefusalReasonV1::ArrayAttributeUnsupported)
    } else {
        refused(CmlRefusalReasonV1::AttributeUnsupported)
    }
}
fn fields(pending: &Pending, allowed: &[&str]) -> Result<BTreeMap<&'static str, String>> {
    let mut result = BTreeMap::new();
    for (name, value) in &pending.attributes {
        if !allowed.contains(&name.as_str()) {
            attribute_refusal(name)?;
        }
        let key = match name.as_str() {
            "id" => "id",
            "elementType" => "elementType",
            "x2" => "x2",
            "y2" => "y2",
            "formalCharge" => "formalCharge",
            "isotopeNumber" => "isotopeNumber",
            "atomRefs2" => "atomRefs2",
            "order" => "order",
            "builtin" => "builtin",
            _ => return refused(CmlRefusalReasonV1::AttributeUnsupported),
        };
        if result.insert(key, value.clone()).is_some() {
            return refused(CmlRefusalReasonV1::AttributeUnsupported);
        }
    }
    Ok(result)
}
fn attribute_refusal(name: &str) -> Result<()> {
    if name == "xmlns" {
        refused(CmlRefusalReasonV1::NamespaceUnsupported)
    } else if [
        "atomID",
        "elementType",
        "x2",
        "y2",
        "formalCharge",
        "isotopeNumber",
        "atomRefs2",
        "bondID",
        "order",
    ]
    .contains(&name)
    {
        refused(CmlRefusalReasonV1::ArrayAttributeUnsupported)
    } else {
        refused(CmlRefusalReasonV1::AttributeUnsupported)
    }
}
fn validate_id(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || !value.is_ascii() {
        return refused(if value.len() > MAX_IDENTIFIER_BYTES {
            CmlRefusalReasonV1::IdentifierBytesLimit
        } else {
            CmlRefusalReasonV1::InvalidScalar
        });
    }
    let bytes = value.as_bytes();
    if !matches!(bytes[0], b'A'..=b'Z' | b'a'..=b'z' | b'_')
        || !bytes.iter().skip(1).all(
            |byte| matches!(*byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.'),
        )
    {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    Ok(())
}
fn coordinate(value: &str) -> Result<f64> {
    if !decimal(value) {
        return refused(CmlRefusalReasonV1::InvalidCoordinate);
    }
    let value = value.parse::<f64>().map_err(|_| CmlDecoderErrorV1 {
        reason: CmlRefusalReasonV1::InvalidCoordinate,
    })?;
    if !value.is_finite() {
        return refused(CmlRefusalReasonV1::CoordinateNotFinite);
    }
    if value.abs() > MAX_COORDINATE {
        return refused(CmlRefusalReasonV1::CoordinateOutOfRange);
    }
    Ok(value)
}
fn signed(value: &str, minimum: i32, maximum: i32) -> Result<i32> {
    if value.is_empty()
        || value.trim() != value
        || !value.as_bytes().iter().enumerate().all(|(index, byte)| {
            byte.is_ascii_digit() || (index == 0 && matches!(*byte, b'+' | b'-'))
        })
    {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    let parsed = value.parse::<i32>().map_err(|_| CmlDecoderErrorV1 {
        reason: CmlRefusalReasonV1::InvalidScalar,
    })?;
    if !(minimum..=maximum).contains(&parsed) {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    Ok(parsed)
}
fn unsigned(value: &str, minimum: u32, maximum: u32) -> Result<u32> {
    if value.is_empty() || !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    let parsed = value.parse::<u32>().map_err(|_| CmlDecoderErrorV1 {
        reason: CmlRefusalReasonV1::InvalidScalar,
    })?;
    if !(minimum..=maximum).contains(&parsed) {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    Ok(parsed)
}
fn decimal(value: &str) -> bool {
    let value = value.strip_prefix(['+', '-']).unwrap_or(value);
    !value.is_empty()
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || *byte == b'.')
        && value.bytes().filter(|byte| *byte == b'.').count() <= 1
        && value.bytes().any(|byte| byte.is_ascii_digit())
}
fn decode_entities(
    value: &str,
    mut remaining: usize,
    limit_reason: CmlRefusalReasonV1,
) -> Result<String> {
    let mut stream = Stream::from(value);
    let mut start = 0;
    let mut decoded = String::new();
    while !stream.at_end() {
        if stream.curr_byte().map_err(|_| CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InvalidXml,
        })? == b'&'
        {
            append_bounded(
                &mut decoded,
                &value[start..stream.pos()],
                &mut remaining,
                limit_reason,
            )?;
            match stream.consume_reference().map_err(|_| CmlDecoderErrorV1 {
                reason: CmlRefusalReasonV1::EntityForbidden,
            })? {
                Reference::Char(character) => {
                    append_character_bounded(
                        &mut decoded,
                        character,
                        &mut remaining,
                        limit_reason,
                    )?;
                }
                Reference::Entity(name)
                    if matches!(name, "amp" | "lt" | "gt" | "quot" | "apos") =>
                {
                    let character = match name {
                        "amp" => '&',
                        "lt" => '<',
                        "gt" => '>',
                        "quot" => '\"',
                        _ => '\'',
                    };
                    append_character_bounded(
                        &mut decoded,
                        character,
                        &mut remaining,
                        limit_reason,
                    )?;
                }
                Reference::Entity(_) => return refused(CmlRefusalReasonV1::EntityForbidden),
            }
            start = stream.pos();
        } else {
            let character = value[stream.pos()..]
                .chars()
                .next()
                .ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::InvalidXml,
                })?;
            stream.advance(character.len_utf8());
        }
    }
    append_bounded(&mut decoded, &value[start..], &mut remaining, limit_reason)?;
    Ok(decoded)
}

fn append_bounded(
    output: &mut String,
    value: &str,
    remaining: &mut usize,
    limit_reason: CmlRefusalReasonV1,
) -> Result<()> {
    if value.len() > *remaining {
        return refused(limit_reason);
    }
    output.push_str(value);
    *remaining -= value.len();
    Ok(())
}

fn append_character_bounded(
    output: &mut String,
    value: char,
    remaining: &mut usize,
    limit_reason: CmlRefusalReasonV1,
) -> Result<()> {
    let byte_count = value.len_utf8();
    if byte_count > *remaining {
        return refused(limit_reason);
    }
    output.push(value);
    *remaining -= byte_count;
    Ok(())
}
