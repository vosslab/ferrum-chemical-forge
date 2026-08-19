//! Exact-observation inspection of one durable direct-root molecule.

use crate::{
    CoreProjectionError, DocumentObjectIdV1, DocumentProjectionV1, MoleculeProjectionV1,
    SessionDocumentObservationV1, TypedDocument, TypedDocumentError,
};
use ferrum_chemistry::AtomicNumber;
use ferrum_core::Molecule;
use thiserror::Error;

/// Stable schema identifier for a durable molecule inspection receipt.
pub const DOCUMENT_MOLECULE_INSPECTION_SCHEMA_V1: &str = "ferrum-document-molecule-inspection-v1";

/// Immutable selector and provenance fence for one direct-root molecule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeInspectionRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
}

impl DocumentMoleculeInspectionRequestV1 {
    /// Construct one inspection request from a frozen observation address.
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
        }
    }

    /// Return the revision which must still identify the observation.
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }

    /// Return the digest which must still identify the observation.
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }

    /// Return the opaque durable direct-root selector.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
}

/// Read-only source facts for one inspected durable molecule.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentMoleculeInspectionV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    projection_key: String,
    source_id: String,
    document_root_order: u32,
    authored_name: Option<String>,
    atom_count: usize,
    bond_count: usize,
    element_inventory: Vec<DocumentMoleculeElementCountV1>,
    total_formal_charge: Option<i64>,
    bounds: Option<DocumentMoleculeBoundsV1>,
}

impl DocumentMoleculeInspectionV1 {
    /// Return this receipt's stable schema identifier.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    /// Return the frozen source revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }
    /// Return the frozen source digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }
    /// Return the opaque durable root selector.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    /// Return the projection-local corroboration key.
    #[must_use]
    pub fn projection_key(&self) -> &str {
        &self.projection_key
    }
    /// Return the literal source root ID.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    /// Return the molecule's root-child source order.
    #[must_use]
    pub const fn document_root_order(&self) -> u32 {
        self.document_root_order
    }
    /// Return the authored molecule name, when supplied.
    #[must_use]
    pub fn authored_name(&self) -> Option<&str> {
        self.authored_name.as_deref()
    }
    /// Return the source atom count.
    #[must_use]
    pub const fn atom_count(&self) -> usize {
        self.atom_count
    }
    /// Return the source bond count.
    #[must_use]
    pub const fn bond_count(&self) -> usize {
        self.bond_count
    }
    /// Return element counts in lexical symbol order.
    #[must_use]
    pub fn element_inventory(&self) -> &[DocumentMoleculeElementCountV1] {
        &self.element_inventory
    }
    /// Return the complete authored charge total, or `None` when any atom omitted it.
    #[must_use]
    pub const fn total_formal_charge(&self) -> Option<i64> {
        self.total_formal_charge
    }
    /// Return normalized finite atom-coordinate bounds.
    #[must_use]
    pub const fn bounds(&self) -> Option<DocumentMoleculeBoundsV1> {
        self.bounds
    }
}

/// One lexical element inventory entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMoleculeElementCountV1 {
    symbol: String,
    atom_count: usize,
}

impl DocumentMoleculeElementCountV1 {
    /// Return the canonical element symbol.
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    /// Return atoms carrying this element.
    #[must_use]
    pub const fn atom_count(&self) -> usize {
        self.atom_count
    }
}

/// Finite normalized x/y bounds of the accepted source atoms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DocumentMoleculeBoundsV1 {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl DocumentMoleculeBoundsV1 {
    /// Return the minimum normalized x coordinate.
    #[must_use]
    pub const fn min_x(&self) -> f64 {
        self.min_x
    }
    /// Return the minimum normalized y coordinate.
    #[must_use]
    pub const fn min_y(&self) -> f64 {
        self.min_y
    }
    /// Return the maximum normalized x coordinate.
    #[must_use]
    pub const fn max_x(&self) -> f64 {
        self.max_x
    }
    /// Return the maximum normalized y coordinate.
    #[must_use]
    pub const fn max_y(&self) -> f64 {
        self.max_y
    }
}

/// Inspect source facts for one durable direct-root molecule without mutation.
pub fn inspect_document_molecule_v1(
    observation: &SessionDocumentObservationV1,
    request: &DocumentMoleculeInspectionRequestV1,
) -> Result<DocumentMoleculeInspectionV1, DocumentMoleculeInspectionErrorV1> {
    let snapshot = observation.snapshot();
    let projection = observation.projection();
    verify_molecule_observation_v1(
        observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    let root = direct_projection_molecule_v1(projection, request.molecule_id())?;
    let root_source_id = root
        .source_id()
        .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    let document = TypedDocument::parse(snapshot.cdml())?;
    let molecule = document
        .core_molecule(&request.molecule_id)?
        .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    if molecule.source_id().map(ferrum_core::Identifier::as_str) != Some(root_source_id) {
        return Err(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch);
    }
    build_document_molecule_inspection_v1(
        snapshot.revision(),
        snapshot.digest(),
        request.molecule_id(),
        root,
        &molecule,
    )
}

pub fn verify_molecule_observation_v1(
    observation: &SessionDocumentObservationV1,
    expected_revision: u64,
    expected_digest: &[u8; 32],
) -> Result<(), DocumentMoleculeInspectionErrorV1> {
    let snapshot = observation.snapshot();
    let projection = observation.projection();
    if snapshot.revision() != projection.revision() || snapshot.digest() != projection.digest() {
        return Err(DocumentMoleculeInspectionErrorV1::ObservationProvenanceMismatch);
    }
    if snapshot.revision() != expected_revision {
        return Err(DocumentMoleculeInspectionErrorV1::StaleObservation {
            expected_revision,
            actual_revision: snapshot.revision(),
        });
    }
    if snapshot.digest() != expected_digest {
        return Err(DocumentMoleculeInspectionErrorV1::DigestMismatch);
    }
    Ok(())
}

pub fn direct_projection_molecule_v1<'a>(
    projection: &'a DocumentProjectionV1,
    molecule_id: &DocumentObjectIdV1,
) -> Result<&'a MoleculeProjectionV1, DocumentMoleculeInspectionErrorV1> {
    let mut roots = projection
        .molecules()
        .iter()
        .filter(|root| root.id() == Some(molecule_id));
    let root = match roots.next() {
        Some(root) => root,
        None => return Err(unknown_direct_molecule(molecule_id)?),
    };
    if roots.next().is_some() {
        return Err(unknown_direct_molecule(molecule_id)?);
    }
    Ok(root)
}

pub fn build_document_molecule_inspection_v1(
    source_revision: u64,
    source_digest: &[u8; 32],
    molecule_id: &DocumentObjectIdV1,
    root: &MoleculeProjectionV1,
    molecule: &Molecule,
) -> Result<DocumentMoleculeInspectionV1, DocumentMoleculeInspectionErrorV1> {
    let root_source_id = root
        .source_id()
        .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    if molecule.atoms().is_empty() {
        return Err(DocumentMoleculeInspectionErrorV1::EmptyMolecule);
    }
    for (kind, count) in [
        ("group", molecule.groups().len()),
        ("molecule text", molecule.texts().len()),
        ("query", molecule.queries().len()),
    ] {
        if count != 0 {
            return Err(DocumentMoleculeInspectionErrorV1::UnsupportedVertex { kind, count });
        }
    }

    let mut inventory = Vec::new();
    inventory
        .try_reserve_exact(molecule.atoms().len())
        .map_err(|_| DocumentMoleculeInspectionErrorV1::ResourceAllocation)?;
    let mut charge = Some(0_i64);
    let first = molecule.atoms()[0].position();
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x(), first.y(), first.x(), first.y());
    for (atom_index, atom) in molecule.atoms().iter().enumerate() {
        let supplied = atom
            .element()
            .ok_or(DocumentMoleculeInspectionErrorV1::MissingElement { atom_index })?;
        let atomic_number = match AtomicNumber::from_symbol(supplied) {
            Ok(atomic_number) => atomic_number,
            Err(_) => {
                return Err(DocumentMoleculeInspectionErrorV1::InvalidElement {
                    atom_index,
                    element: copied(supplied)?,
                });
            }
        };
        let symbol = atomic_number.symbol();
        if let Some((_, count)) = inventory.iter_mut().find(|(present, _)| *present == symbol) {
            *count += 1;
        } else {
            inventory.push((symbol, 1_usize));
        }
        charge = match (charge, atom.formal_charge()) {
            (Some(total), Some(value)) => Some(
                total
                    .checked_add(i64::from(value))
                    .ok_or(DocumentMoleculeInspectionErrorV1::FormalChargeOverflow)?,
            ),
            _ => None,
        };
        let position = atom.position();
        min_x = min_x.min(position.x());
        min_y = min_y.min(position.y());
        max_x = max_x.max(position.x());
        max_y = max_y.max(position.y());
    }
    inventory.sort_unstable_by_key(|(symbol, _)| *symbol);
    let mut element_inventory = Vec::new();
    element_inventory
        .try_reserve_exact(inventory.len())
        .map_err(|_| DocumentMoleculeInspectionErrorV1::ResourceAllocation)?;
    for (symbol, atom_count) in inventory {
        element_inventory.push(DocumentMoleculeElementCountV1 {
            symbol: copied(symbol)?,
            atom_count,
        });
    }
    Ok(DocumentMoleculeInspectionV1 {
        schema: DOCUMENT_MOLECULE_INSPECTION_SCHEMA_V1,
        source_revision,
        source_digest: *source_digest,
        molecule_id: copied_object_id(molecule_id)?,
        projection_key: copied(root.projection_key().as_str())?,
        source_id: copied(root_source_id)?,
        document_root_order: root.source_order(),
        authored_name: root.name().map(copied).transpose()?,
        atom_count: molecule.atoms().len(),
        bond_count: molecule.bonds().len(),
        element_inventory,
        total_formal_charge: charge,
        bounds: Some(DocumentMoleculeBoundsV1 {
            min_x,
            min_y,
            max_x,
            max_y,
        }),
    })
}

fn copied(value: &str) -> Result<String, DocumentMoleculeInspectionErrorV1> {
    let mut owned = String::new();
    owned
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentMoleculeInspectionErrorV1::ResourceAllocation)?;
    owned.push_str(value);
    Ok(owned)
}

pub(crate) fn copied_object_id(
    value: &DocumentObjectIdV1,
) -> Result<DocumentObjectIdV1, DocumentMoleculeInspectionErrorV1> {
    let value = copied(value.as_str())?;
    DocumentObjectIdV1::parse(value)
        .map_err(|_| DocumentMoleculeInspectionErrorV1::OpaqueIdInvariant)
}

fn unknown_direct_molecule(
    object_id: &DocumentObjectIdV1,
) -> Result<DocumentMoleculeInspectionErrorV1, DocumentMoleculeInspectionErrorV1> {
    Ok(DocumentMoleculeInspectionErrorV1::UnknownDirectMolecule {
        object_id: copied(object_id.as_str())?,
    })
}

/// Failure while inspecting retained source facts without mutation.
#[derive(Debug, Error)]
pub enum DocumentMoleculeInspectionErrorV1 {
    /// Snapshot and projection did not originate from the same source state.
    #[error("observation snapshot and projection provenance disagree")]
    ObservationProvenanceMismatch,
    /// The caller supplied a different expected revision.
    #[error(
        "document changed from revision {expected_revision} to {actual_revision}; inspect again"
    )]
    StaleObservation {
        expected_revision: u64,
        actual_revision: u64,
    },
    /// The caller supplied a different expected digest.
    #[error("document digest changed; inspect again")]
    DigestMismatch,
    /// The selector was not exactly one durable direct-root projection molecule.
    #[error("document object is not one durable direct-root molecule: {object_id}")]
    UnknownDirectMolecule { object_id: String },
    /// Projection and typed root identities could not corroborate each other.
    #[error("direct-root projection does not match the retained typed molecule")]
    ProjectionRootMismatch,
    /// An already validated opaque ID could not be reconstructed after copying.
    #[error("validated opaque molecule selector could not be reconstructed")]
    OpaqueIdInvariant,
    /// The retained snapshot could not be parsed.
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
    /// The selected retained molecule could not form a valid core graph.
    #[error(transparent)]
    CoreProjection(#[from] CoreProjectionError),
    /// A graph with no atoms has no inspectable v1 molecule facts.
    #[error("molecule must contain at least one atom")]
    EmptyMolecule,
    /// A non-atom vertex falls outside the closed source-fact receipt.
    #[error("molecule contains unsupported {count} {kind} vertices")]
    UnsupportedVertex { kind: &'static str, count: usize },
    /// An atom omitted its element fact.
    #[error("atom {atom_index} has no authored element")]
    MissingElement { atom_index: usize },
    /// An atom's element spelling is outside the native element vocabulary.
    #[error("atom {atom_index} has invalid element {element:?}")]
    InvalidElement { atom_index: usize, element: String },
    /// Explicit charges exceeded the receipt's signed total.
    #[error("complete authored formal charge cannot be represented")]
    FormalChargeOverflow,
    /// New receipt storage could not be allocated.
    #[error("molecule inspection could not reserve result storage")]
    ResourceAllocation,
}
