//! Projection from typed CDML persistence facts into the chemistry-independent core.

use std::collections::{HashMap, HashSet};

use ferrum_core::{
    Atom, Bond, BondOrder, BondStyle, Identifier, ModelError, Molecule, NonAtomVertex, Position,
    RecordId, RecordKind, VertexRef,
};
use thiserror::Error;

use super::{DocumentObjectIdV1, TypedClass, TypedDocument, TypedRecord};

const POINTS_PER_CENTIMETRE: f64 = 72.0 / 2.54;

/// A core-model view of all typed molecule records in one CDML document.
#[derive(Clone, Debug, PartialEq)]
pub struct CoreProjection {
    document_version: Option<String>,
    molecules: Vec<Molecule>,
}

impl CoreProjection {
    /// Return the exact root version spelling, if authored.
    #[must_use]
    pub fn document_version(&self) -> Option<&str> {
        self.document_version.as_deref()
    }

    /// Return every molecule in document source order.
    #[must_use]
    pub fn molecules(&self) -> &[Molecule] {
        &self.molecules
    }
}

/// A typed document could not supply a valid `ferrum-core` graph.
#[derive(Debug, Error)]
pub enum CoreProjectionError {
    /// A field required by the core projection is absent.
    #[error("{context}: required field {field} is absent")]
    MissingField {
        /// Typed record path and class.
        context: String,
        /// Required field or child name.
        field: &'static str,
    },
    /// A lexical scalar cannot be represented by the core field.
    #[error("{context}: {field} value {value:?} is invalid")]
    InvalidValue {
        /// Typed record path and class.
        context: String,
        /// Field being converted.
        field: &'static str,
        /// Exact source spelling.
        value: String,
    },
    /// A bond endpoint does not name a carried local vertex.
    #[error("{context}: {field} names unknown molecule-local vertex {identifier:?}")]
    UnknownVertex {
        /// Typed record path and class.
        context: String,
        /// `start` or `end`.
        field: &'static str,
        /// Unresolved exact IDREF spelling.
        identifier: String,
    },
    /// A core-model invariant rejected the typed source facts.
    #[error("{context}: {source}")]
    Model {
        /// Typed record path and class.
        context: String,
        /// Core-model validation failure.
        #[source]
        source: ModelError,
    },
}

impl TypedDocument {
    /// Project typed molecule records into validated, owned `ferrum-core` values.
    pub fn core_projection(&self) -> Result<CoreProjection, CoreProjectionError> {
        let document_version = self.root().attribute("version").map(str::to_owned);
        let mut used_identities = HashSet::new();
        let mut molecules = Vec::new();
        for molecule_record in self.root().children_of(TypedClass::Molecule) {
            let molecule = load_molecule(
                molecule_record,
                document_version.as_deref(),
                &mut used_identities,
            )?;
            used_identities.insert(molecule.identity().clone());
            molecules.push(molecule);
        }
        Ok(CoreProjection {
            document_version,
            molecules,
        })
    }

    /// Project one durable typed molecule without requiring unrelated molecules.
    ///
    /// This targeted form is intended for bounded chemistry operations. Invalid
    /// facts in another retained molecule must not prevent a caller from preparing
    /// an operation for the selected molecule.
    pub fn core_molecule(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Result<Option<Molecule>, CoreProjectionError> {
        let Some(record) = self.resolve_document_object_id(object_id) else {
            return Ok(None);
        };
        if record.class() != TypedClass::Molecule {
            return Ok(None);
        }
        let mut used_identities = HashSet::new();
        load_molecule(
            record,
            self.root().attribute("version"),
            &mut used_identities,
        )
        .map(Some)
    }
}

struct RawBond {
    source_id: Option<Identifier>,
    start: String,
    end: String,
    source_type: Option<String>,
    context: String,
}

struct MoleculeVertices {
    atoms: Vec<Atom>,
    groups: Vec<NonAtomVertex>,
    texts: Vec<NonAtomVertex>,
    queries: Vec<NonAtomVertex>,
    references: HashMap<String, VertexRef>,
}

fn load_molecule(
    record: &TypedRecord,
    document_version: Option<&str>,
    used_identities: &mut HashSet<RecordId>,
) -> Result<Molecule, CoreProjectionError> {
    let context = record_context(record);
    let source_id = optional_identifier(record, "id");
    let name = record.attribute("name").map(str::to_owned);
    let vertices = load_vertices(record, used_identities)?;
    let raw_bonds = read_raw_bonds(record)?;
    let mut bonds = Vec::new();
    for raw_bond in &raw_bonds {
        let bond = build_bond(
            raw_bond,
            document_version,
            &vertices.references,
            used_identities,
        )?;
        used_identities.insert(bond.identity().clone());
        bonds.push(bond);
    }
    let molecule = build_record(used_identities, Molecule::identity, |occurrence| {
        Molecule::new(
            source_id.clone(),
            name.clone(),
            vertices.atoms.clone(),
            vertices.groups.clone(),
            vertices.texts.clone(),
            vertices.queries.clone(),
            bonds.clone(),
            occurrence,
        )
    })
    .map_err(|source| CoreProjectionError::Model { context, source })?;
    Ok(molecule)
}

fn load_vertices(
    molecule: &TypedRecord,
    used_identities: &mut HashSet<RecordId>,
) -> Result<MoleculeVertices, CoreProjectionError> {
    let mut vertices = MoleculeVertices {
        atoms: Vec::new(),
        groups: Vec::new(),
        texts: Vec::new(),
        queries: Vec::new(),
        references: HashMap::new(),
    };
    for child in molecule.typed_children() {
        let record = child.record();
        match record.class() {
            TypedClass::Atom => {
                let atom = load_atom(record, used_identities)?;
                used_identities.insert(atom.identity().clone());
                register_reference(
                    &mut vertices.references,
                    atom.source_id(),
                    VertexRef::Atom(atom.identity().clone()),
                );
                vertices.atoms.push(atom);
            }
            TypedClass::Group | TypedClass::MoleculeText | TypedClass::Query => {
                let kind = vertex_kind(record.class());
                let vertex = load_non_atom_vertex(record, kind, used_identities)?;
                used_identities.insert(vertex.identity().clone());
                register_reference(
                    &mut vertices.references,
                    vertex.source_id(),
                    vertex_reference(kind, vertex.identity().clone()),
                );
                match kind {
                    RecordKind::Group => vertices.groups.push(vertex),
                    RecordKind::Text => vertices.texts.push(vertex),
                    RecordKind::Query => vertices.queries.push(vertex),
                    _ => unreachable!("the context table supplies only non-atom vertex kinds"),
                }
            }
            _ => {}
        }
    }
    Ok(vertices)
}

fn load_atom(
    record: &TypedRecord,
    used_identities: &HashSet<RecordId>,
) -> Result<Atom, CoreProjectionError> {
    let context = record_context(record);
    let source_id = optional_identifier(record, "id");
    let element = record.attribute("name").map(str::to_owned);
    let point = record
        .children_of(TypedClass::Point)
        .next()
        .ok_or_else(|| CoreProjectionError::MissingField {
            context: context.clone(),
            field: "point",
        })?;
    let position = load_position(point)?;
    let formal_charge = optional_scalar(record, "charge")?;
    let isotope = optional_scalar(record, "isotope")?;
    let explicit_hydrogens = optional_scalar(record, "explicit_hydrogens")?;
    let valence = optional_scalar(record, "valency")?;
    let multiplicity = optional_scalar(record, "multiplicity")?;
    let free_sites = optional_scalar(record, "free_sites")?;
    let atom = build_record(used_identities, Atom::identity, |occurrence| {
        Atom::new(
            source_id.clone(),
            element.clone(),
            position,
            formal_charge,
            isotope,
            explicit_hydrogens,
            valence,
            multiplicity,
            free_sites,
            occurrence,
        )
    })
    .map_err(|source| CoreProjectionError::Model { context, source })?;
    Ok(atom)
}

fn load_non_atom_vertex(
    record: &TypedRecord,
    kind: RecordKind,
    used_identities: &HashSet<RecordId>,
) -> Result<NonAtomVertex, CoreProjectionError> {
    let context = record_context(record);
    let source_id = optional_identifier(record, "id");
    let vertex = build_record(used_identities, NonAtomVertex::identity, |occurrence| {
        NonAtomVertex::new(kind, source_id.clone(), occurrence)
    })
    .map_err(|source| CoreProjectionError::Model { context, source })?;
    Ok(vertex)
}

fn read_raw_bonds(molecule: &TypedRecord) -> Result<Vec<RawBond>, CoreProjectionError> {
    molecule
        .children_of(TypedClass::Bond)
        .map(|record| {
            let context = record_context(record);
            Ok(RawBond {
                source_id: optional_identifier(record, "id"),
                start: required_attribute(record, "start")?.to_owned(),
                end: required_attribute(record, "end")?.to_owned(),
                source_type: record.attribute("type").map(str::to_owned),
                context,
            })
        })
        .collect()
}

fn build_bond(
    raw_bond: &RawBond,
    document_version: Option<&str>,
    references: &HashMap<String, VertexRef>,
    used_identities: &HashSet<RecordId>,
) -> Result<Bond, CoreProjectionError> {
    let start = resolve_vertex(references, &raw_bond.start, "start", &raw_bond.context)?;
    let end = resolve_vertex(references, &raw_bond.end, "end", &raw_bond.context)?;
    let (order, style) = raw_bond
        .source_type
        .as_deref()
        .map(|source_type| bond_semantics(document_version, source_type))
        .unwrap_or((None, None));
    let aromatic = order.map(|value| value == BondOrder::Aromatic);
    let bond = build_record(used_identities, Bond::identity, |occurrence| {
        Bond::new(
            raw_bond.source_id.clone(),
            start.clone(),
            end.clone(),
            raw_bond.source_type.clone(),
            order,
            style.clone(),
            aromatic,
            occurrence,
        )
    })
    .map_err(|source| CoreProjectionError::Model {
        context: raw_bond.context.clone(),
        source,
    })?;
    Ok(bond)
}

fn resolve_vertex(
    references: &HashMap<String, VertexRef>,
    identifier: &str,
    field: &'static str,
    context: &str,
) -> Result<VertexRef, CoreProjectionError> {
    references
        .get(identifier)
        .cloned()
        .ok_or_else(|| CoreProjectionError::UnknownVertex {
            context: context.to_owned(),
            field,
            identifier: identifier.to_owned(),
        })
}

fn load_position(record: &TypedRecord) -> Result<Position, CoreProjectionError> {
    let context = record_context(record);
    let x = coordinate(required_attribute(record, "x")?, record, "x")?;
    let y = coordinate(required_attribute(record, "y")?, record, "y")?;
    let z = record
        .attribute("z")
        .map(|value| coordinate(value, record, "z"))
        .transpose()?
        .unwrap_or(0.0);
    Position::new(x, y, z).map_err(|source| CoreProjectionError::Model { context, source })
}

fn coordinate(
    value: &str,
    record: &TypedRecord,
    field: &'static str,
) -> Result<f64, CoreProjectionError> {
    let (number, scale) = value
        .strip_suffix("cm")
        .map(|number| (number, POINTS_PER_CENTIMETRE))
        .unwrap_or((value, 1.0));
    number
        .parse::<f64>()
        .map(|parsed| parsed * scale)
        .map_err(|_| CoreProjectionError::InvalidValue {
            context: record_context(record),
            field,
            value: value.to_owned(),
        })
}

fn optional_identifier(record: &TypedRecord, field: &'static str) -> Option<Identifier> {
    record.attribute(field).map(|value| {
        Identifier::new(value)
            .expect("the document identity index validates every projected declaration ID")
    })
}

fn optional_scalar<T>(
    record: &TypedRecord,
    field: &'static str,
) -> Result<Option<T>, CoreProjectionError>
where
    T: std::str::FromStr,
{
    record
        .attribute(field)
        .map(|value| {
            value
                .parse()
                .map_err(|_| CoreProjectionError::InvalidValue {
                    context: record_context(record),
                    field,
                    value: value.to_owned(),
                })
        })
        .transpose()
}

fn required_attribute<'a>(
    record: &'a TypedRecord,
    field: &'static str,
) -> Result<&'a str, CoreProjectionError> {
    record
        .attribute(field)
        .ok_or_else(|| CoreProjectionError::MissingField {
            context: record_context(record),
            field,
        })
}

fn register_reference(
    references: &mut HashMap<String, VertexRef>,
    source_id: Option<&Identifier>,
    reference: VertexRef,
) {
    let Some(source_id) = source_id else {
        return;
    };
    let previous = references.insert(source_id.as_str().to_owned(), reference);
    debug_assert!(
        previous.is_none(),
        "the document identity index rejects duplicate declaration IDs"
    );
}

fn build_record<T, B, I>(
    used_identities: &HashSet<RecordId>,
    identity: I,
    build: B,
) -> Result<T, ModelError>
where
    B: Fn(Option<u32>) -> Result<T, ModelError>,
    I: Fn(&T) -> &RecordId,
{
    if let Ok(record) = build(None) {
        return Ok(record);
    }
    let mut occurrence = 0_u32;
    loop {
        let record = build(Some(occurrence))?;
        if !used_identities.contains(identity(&record)) {
            return Ok(record);
        }
        occurrence += 1;
    }
}

fn vertex_kind(class: TypedClass) -> RecordKind {
    match class {
        TypedClass::Group => RecordKind::Group,
        TypedClass::MoleculeText => RecordKind::Text,
        TypedClass::Query => RecordKind::Query,
        _ => unreachable!("only molecule-local non-atom classes are converted"),
    }
}

fn vertex_reference(kind: RecordKind, identity: RecordId) -> VertexRef {
    match kind {
        RecordKind::Group => VertexRef::Group(identity),
        RecordKind::Text => VertexRef::Text(identity),
        RecordKind::Query => VertexRef::Query(identity),
        _ => unreachable!("only molecule-local non-atom kinds are converted"),
    }
}

fn bond_semantics(
    document_version: Option<&str>,
    source_type: &str,
) -> (Option<BondOrder>, Option<BondStyle>) {
    if document_version == Some("0.8") {
        return match source_type {
            "s" => (Some(BondOrder::Single), Some(BondStyle::Normal)),
            "d" => (Some(BondOrder::Double), Some(BondStyle::Normal)),
            _ => current_bond_semantics(source_type),
        };
    }
    current_bond_semantics(source_type)
}

fn current_bond_semantics(source_type: &str) -> (Option<BondOrder>, Option<BondStyle>) {
    let Some(digits) = source_type.get(1..) else {
        return (None, None);
    };
    let order = if digits.is_empty() {
        None
    } else {
        let value: u8 = match digits.parse() {
            Ok(value) => value,
            Err(_) => return (None, None),
        };
        Some(match value {
            1 => BondOrder::Single,
            2 => BondOrder::Double,
            3 => BondOrder::Triple,
            4 => BondOrder::Aromatic,
            other => BondOrder::Other(other),
        })
    };
    (order, bond_style(source_type))
}

fn bond_style(source_type: &str) -> Option<BondStyle> {
    let character = source_type.chars().next()?;
    let style = match character {
        'n' => BondStyle::Normal,
        'w' => BondStyle::Wedge,
        'h' | 'l' | 'r' => BondStyle::Hashed,
        'a' => BondStyle::Adder,
        'b' => BondStyle::Bold,
        'd' => BondStyle::Dashed,
        'o' => BondStyle::Dotted,
        's' => BondStyle::Wavy,
        'q' => BondStyle::HaworthFront,
        other => BondStyle::Other(other.to_string()),
    };
    Some(style)
}

fn record_context(record: &TypedRecord) -> String {
    format!("{} at {}", record.class().name(), record.path())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_zero_eight_tokens_keep_their_historical_order_meaning() {
        assert_eq!(
            bond_semantics(Some("0.8"), "s"),
            (Some(BondOrder::Single), Some(BondStyle::Normal))
        );
        assert_eq!(
            bond_semantics(Some("0.8"), "d"),
            (Some(BondOrder::Double), Some(BondStyle::Normal))
        );
    }

    #[test]
    fn current_tokens_keep_order_and_depiction_separate() {
        assert_eq!(
            bond_semantics(Some("26.07"), "s1"),
            (Some(BondOrder::Single), Some(BondStyle::Wavy))
        );
        assert_eq!(
            bond_semantics(Some("26.07"), "q1"),
            (Some(BondOrder::Single), Some(BondStyle::HaworthFront))
        );
        assert_eq!(
            bond_semantics(Some("26.07"), "l2"),
            (Some(BondOrder::Double), Some(BondStyle::Hashed))
        );
    }
}
