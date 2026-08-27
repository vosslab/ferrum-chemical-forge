//! Streaming closed-grammar CDXML-C1 parser.

use std::collections::{BTreeMap, BTreeSet};

use xmlparser::{ElementEnd, ExternalId, Token};

use super::values::*;
use super::*;

const VENDOR_DTD: &str = "https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd";
const MAX_ELEMENTS: usize = 50_000;
const MAX_RECORDS: usize = 1_024;
const MAX_ATOMS_PER_RECORD: usize = 10_000;
const MAX_BONDS_PER_RECORD: usize = 20_000;

#[derive(Clone, Copy, Eq, PartialEq)]
enum DocumentPhase {
    Prolog,
    Root,
    Closed,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Frame {
    Root,
    ColorTable,
    Color,
    FontTable,
    Font,
    Page,
    Fragment,
    Node,
    Text,
    Span,
    TemplateGrid,
    Bond,
}
struct Pending {
    name: String,
    attributes: Vec<(String, String)>,
}
struct Atom {
    id: String,
    numeric: Option<AtomicNumber>,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    point: Point2,
    label: Option<String>,
}
struct Bond {
    start: String,
    end: String,
    order: BondOrder,
    direction: Option<BondDirection>,
}
struct Fragment {
    id: String,
    atoms: Vec<Atom>,
    bonds: Vec<Bond>,
    atom_indexes: BTreeMap<String, usize>,
    pairs: BTreeSet<(usize, usize)>,
}

impl Fragment {
    fn new(id: String) -> Self {
        Self {
            id,
            atoms: Vec::new(),
            bonds: Vec::new(),
            atom_indexes: BTreeMap::new(),
            pairs: BTreeSet::new(),
        }
    }
    fn add_atom(&mut self, atom: Atom) -> Result<()> {
        if self.atoms.len() >= MAX_ATOMS_PER_RECORD {
            return refused(CdxmlRefusalReasonV1::AtomsPerRecordLimit);
        }
        validate_id(&atom.id)?;
        if self.atom_indexes.contains_key(&atom.id) {
            return refused(CdxmlRefusalReasonV1::DuplicateAtomId);
        }
        let index = self.atoms.len();
        self.atom_indexes.insert(atom.id.clone(), index);
        self.atoms.push(atom);
        Ok(())
    }
    fn add_bond(&mut self, bond: Bond) -> Result<()> {
        if self.bonds.len() >= MAX_BONDS_PER_RECORD {
            return refused(CdxmlRefusalReasonV1::BondsPerRecordLimit);
        }
        let start = *self
            .atom_indexes
            .get(&bond.start)
            .ok_or(CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::DanglingBond,
            })?;
        let end = *self
            .atom_indexes
            .get(&bond.end)
            .ok_or(CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::DanglingBond,
            })?;
        if start == end {
            return refused(CdxmlRefusalReasonV1::SelfBond);
        }
        if !self.pairs.insert((start.min(end), start.max(end))) {
            return refused(CdxmlRefusalReasonV1::DuplicateBond);
        }
        if bond.direction.is_some() && bond.order != BondOrder::Single {
            return refused(CdxmlRefusalReasonV1::InvalidScalar);
        }
        self.bonds.push(bond);
        Ok(())
    }
    fn finish(self) -> Result<CdxmlDecodedRecordV1> {
        if self.atoms.is_empty() {
            return refused(CdxmlRefusalReasonV1::InvalidGraph);
        }
        let mut atoms = Vec::with_capacity(self.atoms.len());
        let mut points = Vec::with_capacity(self.atoms.len());
        for atom in &self.atoms {
            let label = atom.label.as_deref().map(element_symbol).transpose()?;
            let element = match (atom.numeric, label) {
                (Some(a), Some(b)) if a != b => {
                    return refused(CdxmlRefusalReasonV1::InvalidScalar);
                }
                (Some(a), _) => a,
                (None, Some(b)) => b,
                (None, None) => AtomicNumber::try_from(6).expect("carbon"),
            };
            atoms.push(
                MolAtom::new(element, atom.formal_charge, atom.isotope, None, false).map_err(
                    |_| CdxmlDecoderErrorV1 {
                        reason: CdxmlRefusalReasonV1::InvalidGraph,
                    },
                )?,
            );
            points.push(atom.point.clone());
        }
        let mut bonds = Vec::with_capacity(self.bonds.len());
        for bond in self.bonds {
            let start = *self
                .atom_indexes
                .get(&bond.start)
                .ok_or(CdxmlDecoderErrorV1 {
                    reason: CdxmlRefusalReasonV1::DanglingBond,
                })?;
            let end = *self
                .atom_indexes
                .get(&bond.end)
                .ok_or(CdxmlDecoderErrorV1 {
                    reason: CdxmlRefusalReasonV1::DanglingBond,
                })?;
            bonds.push(match bond.direction {
                Some(direction) => MolBond::directed(start, end, bond.order, false, direction)
                    .map_err(|_| CdxmlDecoderErrorV1 {
                        reason: CdxmlRefusalReasonV1::InvalidGraph,
                    })?,
                None => MolBond::new(start, end, bond.order, false),
            });
        }
        let graph = MolGraph::new(atoms, bonds, Some(Coordinates::new(points))).map_err(|_| {
            CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::InvalidGraph,
            }
        })?;
        Ok(CdxmlDecodedRecordV1 {
            source_fragment_id: self.id,
            record: InterchangeRecordV1::new(graph, None, Vec::new()),
        })
    }
}

pub(super) struct Parser {
    stack: Vec<Frame>,
    pending: Option<Pending>,
    fragment: Option<Fragment>,
    current_atom: Option<Atom>,
    text: Option<String>,
    records: Vec<CdxmlDecodedRecordV1>,
    fragment_ids: BTreeSet<String>,
    losses: BTreeSet<CdxmlLossCategoryV1>,
    first: bool,
    declaration: bool,
    dtd: bool,
    phase: DocumentPhase,
    elements: usize,
    color_table_seen: bool,
    font_table_seen: bool,
    page_seen: bool,
    template_grid_seen: bool,
}

impl Parser {
    pub(super) fn new() -> Self {
        Self {
            stack: Vec::new(),
            pending: None,
            fragment: None,
            current_atom: None,
            text: None,
            records: Vec::new(),
            fragment_ids: BTreeSet::new(),
            losses: BTreeSet::new(),
            first: true,
            declaration: false,
            dtd: false,
            phase: DocumentPhase::Prolog,
            elements: 0,
            color_table_seen: false,
            font_table_seen: false,
            page_seen: false,
            template_grid_seen: false,
        }
    }
    pub(super) fn token(&mut self, token: Token<'_>) -> Result<()> {
        let first = self.first;
        self.first = false;
        match token {
            Token::Declaration {
                version,
                encoding,
                standalone,
                ..
            } => {
                if !first
                    || self.declaration
                    || self.phase != DocumentPhase::Prolog
                    || version.as_str() != "1.0"
                    || encoding.map(|value| value.as_str()) != Some("UTF-8")
                    || standalone.is_some()
                {
                    return refused(CdxmlRefusalReasonV1::InvalidXmlDeclaration);
                }
                self.declaration = true;
                self.losses.insert(CdxmlLossCategoryV1::LexicalSyntax);
            }
            Token::EmptyDtd {
                name, external_id, ..
            } => {
                if self.dtd
                    || self.phase != DocumentPhase::Prolog
                    || name.as_str() != "CDXML"
                    || !matches!(external_id, Some(ExternalId::System(value)) if value.as_str() == VENDOR_DTD)
                {
                    return refused(CdxmlRefusalReasonV1::DtdForbidden);
                }
                self.dtd = true;
                self.losses.insert(CdxmlLossCategoryV1::LexicalSyntax);
            }
            Token::DtdStart { .. } | Token::DtdEnd { .. } | Token::EntityDeclaration { .. } => {
                return refused(CdxmlRefusalReasonV1::DtdForbidden);
            }
            Token::ElementStart { prefix, local, .. } => {
                if !prefix.as_str().is_empty() {
                    return refused(CdxmlRefusalReasonV1::NamespaceUnsupported);
                }
                if self.pending.is_some() || self.phase == DocumentPhase::Closed {
                    return refused(CdxmlRefusalReasonV1::InvalidXml);
                }
                self.elements = add(
                    self.elements,
                    1,
                    MAX_ELEMENTS,
                    CdxmlRefusalReasonV1::XmlElementLimit,
                )?;
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
                if !prefix.as_str().is_empty() || local.as_str() == "xmlns" {
                    return refused(CdxmlRefusalReasonV1::NamespaceUnsupported);
                }
                let value = value.as_str();
                if value.len() > MAX_ATTRIBUTE_VALUE_BYTES {
                    return refused(CdxmlRefusalReasonV1::AttributeValueLimit);
                }
                if has_entity_reference(value) {
                    return refused(CdxmlRefusalReasonV1::EntityForbidden);
                }
                let pending = self.pending.as_mut().ok_or(CdxmlDecoderErrorV1 {
                    reason: CdxmlRefusalReasonV1::InvalidXml,
                })?;
                if pending
                    .attributes
                    .iter()
                    .any(|(name, _)| name == local.as_str())
                {
                    return refused(CdxmlRefusalReasonV1::AttributeUnsupported);
                }
                pending
                    .attributes
                    .push((local.as_str().to_owned(), value.to_owned()));
            }
            Token::ElementEnd { end, .. } => match end {
                ElementEnd::Open => self.open()?,
                ElementEnd::Empty => {
                    self.open()?;
                    self.close()?;
                }
                ElementEnd::Close(prefix, local) => {
                    if !prefix.as_str().is_empty() || self.stack.last().is_none() {
                        return refused(CdxmlRefusalReasonV1::InvalidXml);
                    }
                    let expected = self.name(*self.stack.last().expect("checked"));
                    if local.as_str() != expected {
                        return refused(CdxmlRefusalReasonV1::InvalidXml);
                    }
                    self.close()?;
                }
            },
            Token::Text { text } => {
                let value = text.as_str();
                if has_entity_reference(value) {
                    return refused(CdxmlRefusalReasonV1::EntityForbidden);
                }
                if self.stack.last() == Some(&Frame::Span) {
                    let current = self.text.as_mut().ok_or(CdxmlDecoderErrorV1 {
                        reason: CdxmlRefusalReasonV1::InternalFailure,
                    })?;
                    current.push_str(value);
                } else if !value.trim().is_empty() {
                    return refused(CdxmlRefusalReasonV1::UnexpectedXmlText);
                }
            }
            Token::Cdata { .. } => return refused(CdxmlRefusalReasonV1::UnexpectedXmlNode),
            Token::Comment { .. } => {
                self.losses.insert(CdxmlLossCategoryV1::LexicalSyntax);
                self.forbid_leaf_node()?;
            }
            Token::ProcessingInstruction { target, .. } => {
                if target.as_str() == "xml-stylesheet" {
                    return refused(CdxmlRefusalReasonV1::UnrepresentedSemanticFact);
                }
                self.losses.insert(CdxmlLossCategoryV1::LexicalSyntax);
                self.forbid_leaf_node()?;
            }
        };
        Ok(())
    }
    fn name(&self, frame: Frame) -> &'static str {
        match frame {
            Frame::Root => "CDXML",
            Frame::ColorTable => "colortable",
            Frame::Color => "color",
            Frame::FontTable => "fonttable",
            Frame::Font => "font",
            Frame::Page => "page",
            Frame::Fragment => "fragment",
            Frame::Node => "n",
            Frame::Text => "t",
            Frame::Span => "s",
            Frame::TemplateGrid => "templategrid",
            Frame::Bond => "b",
        }
    }
    fn forbid_leaf_node(&self) -> Result<()> {
        if matches!(
            self.stack.last(),
            Some(Frame::Node | Frame::Text | Frame::Span | Frame::Bond)
        ) {
            refused(CdxmlRefusalReasonV1::UnexpectedXmlNode)
        } else {
            Ok(())
        }
    }
    fn open(&mut self) -> Result<()> {
        let pending = self.pending.take().ok_or(CdxmlDecoderErrorV1 {
            reason: CdxmlRefusalReasonV1::InvalidXml,
        })?;
        let frame = match self.stack.last().copied() {
            None => {
                if self.phase != DocumentPhase::Prolog || pending.name != "CDXML" {
                    return refused(CdxmlRefusalReasonV1::RootUnsupported);
                }
                self.metadata(&pending, allowed_root_attribute)?;
                self.phase = DocumentPhase::Root;
                Frame::Root
            }
            Some(Frame::Root) => match pending.name.as_str() {
                "colortable"
                    if !self.color_table_seen
                        && !self.font_table_seen
                        && !self.page_seen
                        && !self.template_grid_seen =>
                {
                    self.require_no_attributes(&pending)?;
                    self.color_table_seen = true;
                    self.losses
                        .insert(CdxmlLossCategoryV1::DocumentViewMetadata);
                    Frame::ColorTable
                }
                "fonttable"
                    if !self.font_table_seen && !self.page_seen && !self.template_grid_seen =>
                {
                    self.require_no_attributes(&pending)?;
                    self.font_table_seen = true;
                    self.losses
                        .insert(CdxmlLossCategoryV1::DocumentViewMetadata);
                    Frame::FontTable
                }
                "page" => {
                    if self.template_grid_seen {
                        return refused(CdxmlRefusalReasonV1::UnrepresentedSemanticFact);
                    }
                    self.page_seen = true;
                    self.metadata(&pending, allowed_page_attribute)?;
                    Frame::Page
                }
                "templategrid" if self.page_seen && !self.template_grid_seen => {
                    self.require_no_attributes(&pending)?;
                    self.template_grid_seen = true;
                    self.losses
                        .insert(CdxmlLossCategoryV1::DocumentViewMetadata);
                    Frame::TemplateGrid
                }
                _ => return refused(CdxmlRefusalReasonV1::UnrepresentedSemanticFact),
            },
            Some(Frame::ColorTable) => {
                self.require_name(&pending, "color")?;
                self.metadata(&pending, allowed_color_attribute)?;
                Frame::Color
            }
            Some(Frame::FontTable) => {
                self.require_name(&pending, "font")?;
                self.metadata(&pending, allowed_font_attribute)?;
                Frame::Font
            }
            Some(Frame::Page) => {
                self.require_name(&pending, "fragment")?;
                self.open_fragment(&pending)?;
                Frame::Fragment
            }
            Some(Frame::Fragment) => match pending.name.as_str() {
                "n" => {
                    self.open_node(&pending)?;
                    Frame::Node
                }
                "b" => {
                    self.open_bond(&pending)?;
                    Frame::Bond
                }
                _ => return refused(CdxmlRefusalReasonV1::UnrepresentedSemanticFact),
            },
            Some(Frame::Node) => {
                self.require_name(&pending, "t")?;
                self.metadata(&pending, allowed_text_attribute)?;
                Frame::Text
            }
            Some(Frame::Text) => {
                self.require_name(&pending, "s")?;
                self.metadata(&pending, allowed_span_attribute)?;
                self.text = Some(String::new());
                Frame::Span
            }
            _ => return refused(CdxmlRefusalReasonV1::UnexpectedXmlNode),
        };
        self.stack.push(frame);
        Ok(())
    }
    fn require_name(&self, pending: &Pending, name: &str) -> Result<()> {
        if pending.name == name {
            Ok(())
        } else {
            refused(CdxmlRefusalReasonV1::UnrepresentedSemanticFact)
        }
    }
    fn require_no_attributes(&self, pending: &Pending) -> Result<()> {
        if pending.attributes.is_empty() {
            Ok(())
        } else {
            refused(CdxmlRefusalReasonV1::AttributeUnsupported)
        }
    }
    fn metadata(&mut self, pending: &Pending, allowed: fn(&str) -> bool) -> Result<()> {
        for (name, _) in &pending.attributes {
            if !allowed(name) {
                return refused(CdxmlRefusalReasonV1::AttributeUnsupported);
            }
        }
        if !pending.attributes.is_empty() {
            self.losses
                .insert(CdxmlLossCategoryV1::DocumentViewMetadata);
        }
        Ok(())
    }
    fn field<'a>(pending: &'a Pending, name: &str) -> Option<&'a str> {
        pending
            .attributes
            .iter()
            .find_map(|(key, value)| (key == name).then_some(value.as_str()))
    }
    fn only_fields(&self, pending: &Pending, allowed: &[&str]) -> Result<()> {
        if pending
            .attributes
            .iter()
            .all(|(name, _)| allowed.contains(&name.as_str()))
        {
            Ok(())
        } else {
            refused(CdxmlRefusalReasonV1::AttributeUnsupported)
        }
    }
    fn open_fragment(&mut self, pending: &Pending) -> Result<()> {
        self.only_fields(pending, &["id"])?;
        let id = Self::field(pending, "id")
            .ok_or(CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::InvalidScalar,
            })?
            .to_owned();
        validate_id(&id)?;
        if !self.fragment_ids.insert(id.clone()) {
            return refused(CdxmlRefusalReasonV1::DuplicateSourceId);
        }
        if self.records.len() >= MAX_RECORDS {
            return refused(CdxmlRefusalReasonV1::RecordLimit);
        }
        self.fragment = Some(Fragment::new(id));
        Ok(())
    }
    fn open_node(&mut self, pending: &Pending) -> Result<()> {
        self.only_fields(pending, &["id", "p", "Element", "Charge", "Isotope"])?;
        let id = Self::field(pending, "id")
            .ok_or(CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::InvalidScalar,
            })?
            .to_owned();
        let point = coordinate_pair(Self::field(pending, "p").ok_or(CdxmlDecoderErrorV1 {
            reason: CdxmlRefusalReasonV1::InvalidCoordinate,
        })?)?;
        let numeric = Self::field(pending, "Element")
            .map(element_number)
            .transpose()?;
        let formal_charge = Self::field(pending, "Charge")
            .map(formal_charge)
            .transpose()?
            .flatten();
        let isotope = Self::field(pending, "Isotope")
            .map(isotope)
            .transpose()?
            .flatten();
        self.current_atom = Some(Atom {
            id,
            numeric,
            formal_charge,
            isotope,
            point: Point2::new(point.0, point.1).map_err(|_| CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::CoordinateNotFinite,
            })?,
            label: None,
        });
        Ok(())
    }
    fn open_bond(&mut self, pending: &Pending) -> Result<()> {
        self.only_fields(pending, &["id", "B", "E", "Order", "Display"])?;
        if let Some(id) = Self::field(pending, "id") {
            validate_id(id)?;
        }
        let start = Self::field(pending, "B")
            .ok_or(CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::InvalidScalar,
            })?
            .to_owned();
        let end = Self::field(pending, "E")
            .ok_or(CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::InvalidScalar,
            })?
            .to_owned();
        validate_id(&start)?;
        validate_id(&end)?;
        let order = match Self::field(pending, "Order").unwrap_or("1") {
            "1" => BondOrder::Single,
            "2" => BondOrder::Double,
            "3" => BondOrder::Triple,
            _ => return refused(CdxmlRefusalReasonV1::UnrepresentedSemanticFact),
        };
        let direction = match Self::field(pending, "Display") {
            None | Some("Solid") => None,
            Some("WedgeBegin") => Some(BondDirection::BeginWedge),
            Some("WedgedHashBegin") => Some(BondDirection::BeginDash),
            Some("WedgeEnd") | Some("WedgedHashEnd") => {
                return refused(CdxmlRefusalReasonV1::UnrepresentedSemanticFact);
            }
            Some(_) => return refused(CdxmlRefusalReasonV1::UnrepresentedSemanticFact),
        };
        self.fragment
            .as_mut()
            .ok_or(CdxmlDecoderErrorV1 {
                reason: CdxmlRefusalReasonV1::InternalFailure,
            })?
            .add_bond(Bond {
                start,
                end,
                order,
                direction,
            })
    }
    fn close(&mut self) -> Result<()> {
        let frame = self.stack.pop().ok_or(CdxmlDecoderErrorV1 {
            reason: CdxmlRefusalReasonV1::InvalidXml,
        })?;
        match frame {
            Frame::Span => {
                let value = self.text.take().ok_or(CdxmlDecoderErrorV1 {
                    reason: CdxmlRefusalReasonV1::InternalFailure,
                })?;
                if value.is_empty() {
                    return refused(CdxmlRefusalReasonV1::InvalidScalar);
                }
                let atom = self.current_atom.as_mut().ok_or(CdxmlDecoderErrorV1 {
                    reason: CdxmlRefusalReasonV1::InternalFailure,
                })?;
                if atom.label.is_some() {
                    return refused(CdxmlRefusalReasonV1::InvalidScalar);
                }
                atom.label = Some(value);
            }
            Frame::Text => {}
            Frame::Node => {
                let atom = self.current_atom.take().ok_or(CdxmlDecoderErrorV1 {
                    reason: CdxmlRefusalReasonV1::InternalFailure,
                })?;
                self.fragment
                    .as_mut()
                    .ok_or(CdxmlDecoderErrorV1 {
                        reason: CdxmlRefusalReasonV1::InternalFailure,
                    })?
                    .add_atom(atom)?;
            }
            Frame::Fragment => {
                let record = self
                    .fragment
                    .take()
                    .ok_or(CdxmlDecoderErrorV1 {
                        reason: CdxmlRefusalReasonV1::InternalFailure,
                    })?
                    .finish()?;
                self.records.push(record);
            }
            Frame::Root => self.phase = DocumentPhase::Closed,
            Frame::ColorTable
            | Frame::Color
            | Frame::FontTable
            | Frame::Font
            | Frame::Page
            | Frame::TemplateGrid
            | Frame::Bond => {}
        };
        Ok(())
    }
    pub(super) fn finish(self) -> Result<CdxmlDecodedDocumentV1> {
        if !self.stack.is_empty() || self.pending.is_some() || self.phase != DocumentPhase::Closed {
            return refused(CdxmlRefusalReasonV1::InvalidXml);
        }
        if self.records.is_empty() {
            return refused(CdxmlRefusalReasonV1::EmptyDocument);
        }
        Ok(CdxmlDecodedDocumentV1 {
            records: self.records,
            declared_losses: self.losses.into_iter().collect(),
        })
    }
}

fn formal_charge(value: &str) -> Result<Option<i32>> {
    if value == "0" {
        return Ok(None);
    }
    let digits = value.strip_prefix('-').unwrap_or(value).as_bytes();
    if digits.is_empty()
        || !matches!(digits[0], b'1'..=b'9')
        || !digits.iter().all(u8::is_ascii_digit)
    {
        return refused(CdxmlRefusalReasonV1::InvalidScalar);
    }
    let charge = value.parse::<i32>().map_err(|_| CdxmlDecoderErrorV1 {
        reason: CdxmlRefusalReasonV1::InvalidScalar,
    })?;
    if !(-128..=127).contains(&charge) {
        return refused(CdxmlRefusalReasonV1::InvalidScalar);
    }
    Ok(Some(charge))
}

fn isotope(value: &str) -> Result<Option<u16>> {
    if value == "0" {
        return Ok(None);
    }
    if !matches!(value.as_bytes().first(), Some(b'1'..=b'9'))
        || !value.as_bytes().iter().all(u8::is_ascii_digit)
    {
        return refused(CdxmlRefusalReasonV1::InvalidScalar);
    }
    let isotope = value.parse::<u16>().map_err(|_| CdxmlDecoderErrorV1 {
        reason: CdxmlRefusalReasonV1::InvalidScalar,
    })?;
    if isotope > 32_767 {
        return refused(CdxmlRefusalReasonV1::InvalidScalar);
    }
    Ok(Some(isotope))
}

fn add(value: usize, extra: usize, limit: usize, reason: CdxmlRefusalReasonV1) -> Result<usize> {
    let value = value
        .checked_add(extra)
        .ok_or(CdxmlDecoderErrorV1 { reason })?;
    if value > limit {
        refused(reason)
    } else {
        Ok(value)
    }
}
