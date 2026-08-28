//! Projection from typed CDML persistence facts into the chemistry-independent core.

use std::collections::HashMap;

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
    /// A first-class compact-group record could not satisfy its closed V1 contract.
    #[error(transparent)]
    CompactGroup(#[from] crate::ProjectionError),
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
    /// A Ferrum structural record did not carry one unique nonblank source ID.
    #[error("{context}: structural source ID error: {source}")]
    StructuralSourceId {
        /// Typed record path and class.
        context: String,
        /// Exact source-ID admission failure.
        #[source]
        source: StructuralSourceIdError,
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

/// One source-ID admission failure at the typed structural projection boundary.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum StructuralSourceIdError {
    /// The canonical structural record had no `id` attribute.
    #[error("id attribute is required")]
    Missing,
    /// The canonical structural record supplied only whitespace for `id`.
    #[error("id attribute is blank")]
    Blank,
    /// Another Ferrum structural record already owns the exact source ID.
    #[error("id {source_id:?} duplicates structural record at {first_context}")]
    Duplicate {
        /// Exact duplicated source spelling.
        source_id: String,
        /// Typed path and class of the first structural owner.
        first_context: String,
    },
}

impl TypedDocument {
    /// Project typed molecule records into validated, owned `ferrum-core` values.
    pub fn core_projection(&self) -> Result<CoreProjection, CoreProjectionError> {
        let document_version = self.root().attribute("version").map(str::to_owned);
        validate_structural_source_ids(self.root())?;
        let mut molecules = Vec::new();
        for molecule_record in self.root().children_of(TypedClass::Molecule) {
            let molecule = load_molecule(molecule_record, document_version.as_deref())?;
            molecules.push(molecule);
        }
        Ok(CoreProjection {
            document_version,
            molecules,
        })
    }

    /// Project one durable typed molecule without requiring unrelated molecules.
    ///
    /// This targeted form is intended for bounded chemistry operations. It still
    /// admits the document-wide structural source-ID contract before projecting
    /// the selected molecule, while unrelated molecule semantics remain local.
    pub fn core_molecule(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Result<Option<Molecule>, CoreProjectionError> {
        let Some(record) = self.resolve_document_object_id(object_id)? else {
            return Ok(None);
        };
        if record.class() != TypedClass::Molecule {
            return Ok(None);
        }
        validate_structural_source_ids(self.root())?;
        load_molecule(record, self.root().attribute("version")).map(Some)
    }
}

struct RawBond {
    source_id: Identifier,
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
) -> Result<Molecule, CoreProjectionError> {
    let context = record_context(record);
    let source_id = required_source_identifier(record)?;
    let name = record.attribute("name").map(str::to_owned);
    let vertices = load_vertices(record)?;
    let raw_bonds = read_raw_bonds(record)?;
    let mut bonds = Vec::new();
    for raw_bond in &raw_bonds {
        let bond = build_bond(raw_bond, document_version, &vertices.references)?;
        bonds.push(bond);
    }
    let molecule = Molecule::new(
        source_id,
        name,
        vertices.atoms,
        vertices.groups,
        vertices.texts,
        vertices.queries,
        bonds,
    )
    .map_err(|source| CoreProjectionError::Model { context, source })?;
    Ok(molecule)
}

fn load_vertices(molecule: &TypedRecord) -> Result<MoleculeVertices, CoreProjectionError> {
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
                let atom = load_atom(record)?;
                register_reference(
                    &mut vertices.references,
                    atom.source_id(),
                    VertexRef::Atom(atom.identity().clone()),
                );
                vertices.atoms.push(atom);
            }
            TypedClass::CompactGroup => {
                // Validate the compact record in its document-owned projection lane before
                // retaining only its durable non-atom identity in the core topology.
                crate::compact_group_projection_v1::compact_group(child)?;
                let kind = RecordKind::Group;
                let vertex = load_non_atom_vertex(record, kind)?;
                register_reference(
                    &mut vertices.references,
                    vertex.source_id(),
                    vertex_reference(kind, vertex.identity().clone()),
                );
                vertices.groups.push(vertex);
            }
            TypedClass::MoleculeText | TypedClass::Query => {
                let kind = vertex_kind(record.class());
                let vertex = load_non_atom_vertex(record, kind)?;
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

fn load_atom(record: &TypedRecord) -> Result<Atom, CoreProjectionError> {
    let context = record_context(record);
    let source_id = required_source_identifier(record)?;
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
    let atom = Atom::new(
        source_id,
        element,
        position,
        formal_charge,
        isotope,
        explicit_hydrogens,
        valence,
        multiplicity,
        free_sites,
    )
    .map_err(|source| CoreProjectionError::Model { context, source })?;
    Ok(atom)
}

fn load_non_atom_vertex(
    record: &TypedRecord,
    kind: RecordKind,
) -> Result<NonAtomVertex, CoreProjectionError> {
    let context = record_context(record);
    let source_id = required_source_identifier(record)?;
    let vertex = NonAtomVertex::new(kind, source_id)
        .map_err(|source| CoreProjectionError::Model { context, source })?;
    Ok(vertex)
}

fn read_raw_bonds(molecule: &TypedRecord) -> Result<Vec<RawBond>, CoreProjectionError> {
    molecule
        .children_of(TypedClass::Bond)
        .map(|record| {
            let context = record_context(record);
            Ok(RawBond {
                source_id: required_source_identifier(record)?,
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
) -> Result<Bond, CoreProjectionError> {
    let start = resolve_vertex(references, &raw_bond.start, "start", &raw_bond.context)?;
    let end = resolve_vertex(references, &raw_bond.end, "end", &raw_bond.context)?;
    let (order, style) = raw_bond
        .source_type
        .as_deref()
        .map(|source_type| bond_semantics(document_version, source_type))
        .unwrap_or((None, None));
    let aromatic = order.map(|value| value == BondOrder::Aromatic);
    let bond = Bond::new(
        raw_bond.source_id.clone(),
        start,
        end,
        raw_bond.source_type.clone(),
        order,
        style,
        aromatic,
    )
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

fn validate_structural_source_ids(root: &TypedRecord) -> Result<(), CoreProjectionError> {
    let mut identifiers = StructuralSourceIdRegistry::default();
    for molecule in root.children_of(TypedClass::Molecule) {
        identifiers.register(molecule)?;
        for child in molecule.typed_children() {
            if matches!(
                child.record().class(),
                TypedClass::Atom
                    | TypedClass::CompactGroup
                    | TypedClass::MoleculeText
                    | TypedClass::Query
                    | TypedClass::Bond
            ) {
                identifiers.register(child.record())?;
            }
        }
    }
    Ok(())
}

#[derive(Default)]
struct StructuralSourceIdRegistry {
    contexts: HashMap<Identifier, String>,
}

impl StructuralSourceIdRegistry {
    fn register(&mut self, record: &TypedRecord) -> Result<(), CoreProjectionError> {
        let source_id = required_source_identifier(record)?;
        let context = record_context(record);
        if let Some(first_context) = self.contexts.get(&source_id) {
            return Err(CoreProjectionError::StructuralSourceId {
                context,
                source: StructuralSourceIdError::Duplicate {
                    source_id: source_id.as_str().to_owned(),
                    first_context: first_context.clone(),
                },
            });
        }
        self.contexts.insert(source_id, context);
        Ok(())
    }
}

fn required_source_identifier(record: &TypedRecord) -> Result<Identifier, CoreProjectionError> {
    let context = record_context(record);
    let value = record
        .attribute("id")
        .ok_or_else(|| CoreProjectionError::StructuralSourceId {
            context: context.clone(),
            source: StructuralSourceIdError::Missing,
        })?;
    Identifier::new(value).map_err(|_| CoreProjectionError::StructuralSourceId {
        context,
        source: StructuralSourceIdError::Blank,
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
    source_id: &Identifier,
    reference: VertexRef,
) {
    let previous = references.insert(source_id.as_str().to_owned(), reference);
    debug_assert!(
        previous.is_none(),
        "structural source-ID admission rejects duplicate declarations"
    );
}

fn vertex_kind(class: TypedClass) -> RecordKind {
    match class {
        TypedClass::CompactGroup => RecordKind::Group,
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
    crate::project_source_bond_semantics(source_type)
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
    fn current_source_projection_preserves_non_authorable_semantics() {
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
        assert_eq!(
            bond_semantics(Some("26.07"), "n0"),
            (Some(BondOrder::Other(0)), Some(BondStyle::Normal))
        );
    }
}
