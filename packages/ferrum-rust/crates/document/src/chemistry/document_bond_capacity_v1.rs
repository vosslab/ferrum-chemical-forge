//! Authenticated read-only neutral bond-capacity receipts.

use std::collections::HashMap;

use crate::{DocumentObjectIdV1, SessionDocumentObservationV1, TypedDocument};
use ferrum_core::{BondOrder, Molecule, VertexRef};
use ferrum_domain::{
    NeutralBondCapacityAtomOutcomeV1, NeutralBondCapacityAtomRecordV1, NeutralBondCapacityAtomV1,
    NeutralBondCapacityBondV1, NeutralBondCapacityExplicitHydrogensFactV1,
    NeutralBondCapacityFormalChargeFactV1, evaluate_neutral_bond_capacity_v1,
};
use thiserror::Error;

use super::document_molecule_inspection_v1::{
    DocumentMoleculeInspectionErrorV1, direct_projection_molecule_v1,
    verify_molecule_observation_v1,
};
use crate::document_direct_root_index_v1::document_direct_root_paint_orders_v1;

/// Stable schema identifier for neutral bond-capacity receipts.
pub const DOCUMENT_BOND_CAPACITY_SCHEMA_V1: &str = "ferrum-document-bond-capacity-v1";

/// Exact observation and selected durable direct roots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentBondCapacityRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_ids: Vec<DocumentObjectIdV1>,
}

impl DocumentBondCapacityRequestV1 {
    /// Construct a nonempty nonduplicated exact-root request.
    pub fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_ids: Vec<DocumentObjectIdV1>,
    ) -> Result<Self, DocumentBondCapacityRequestErrorV1> {
        if molecule_ids.is_empty() {
            return Err(DocumentBondCapacityRequestErrorV1::EmptySelection);
        }
        for (index, id) in molecule_ids.iter().enumerate() {
            if molecule_ids[..index].contains(id) {
                return Err(DocumentBondCapacityRequestErrorV1::DuplicateMolecule);
            }
        }
        Ok(Self {
            expected_revision,
            expected_digest,
            molecule_ids,
        })
    }
}

/// Authenticated root facts needed to display one receipt without reinterpreting CDML.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentBondCapacitySourceV1 {
    molecule_id: DocumentObjectIdV1,
    projection_key: String,
    source_id: String,
    document_paint_order: u32,
    authored_name: Option<String>,
}
impl DocumentBondCapacitySourceV1 {
    #[must_use]
    pub fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub fn projection_key(&self) -> &str {
        &self.projection_key
    }
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    #[must_use]
    pub const fn document_paint_order(&self) -> u32 {
        self.document_paint_order
    }
    #[must_use]
    pub fn authored_name(&self) -> Option<&str> {
        self.authored_name.as_deref()
    }
}

/// Complete-root outcome; excluded facts never produce partial atom outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentBondCapacityOutcomeV1 {
    /// Every supported atom was at or below the finite neutral table.
    WithinCapacity {
        atoms: Vec<NeutralBondCapacityAtomRecordV1>,
    },
    /// At least one supported atom exceeded the finite neutral table.
    ExceedsCapacity {
        atoms: Vec<NeutralBondCapacityAtomRecordV1>,
    },
    /// This complete root needs a broader representation model.
    NotChecked {
        reason: DocumentBondCapacityNotCheckedReasonV1,
    },
}

/// Stable closed-profile refusal categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentBondCapacityNotCheckedReasonV1 {
    NonAtomVertex,
    NonNeutralCharge,
    AuthoredAtomCapacityFact,
    UnsupportedElement,
    UnsupportedBondEndpoint,
    UnsupportedBondOrder,
    AromaticFact,
}

/// One ordered selected-root receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentBondCapacityRecordV1 {
    source: DocumentBondCapacitySourceV1,
    outcome: DocumentBondCapacityOutcomeV1,
}
impl DocumentBondCapacityRecordV1 {
    #[must_use]
    pub const fn source(&self) -> &DocumentBondCapacitySourceV1 {
        &self.source
    }
    #[must_use]
    pub const fn outcome(&self) -> &DocumentBondCapacityOutcomeV1 {
        &self.outcome
    }
}

/// Immutable multi-root receipt tied to one exact document observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentBondCapacityV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    records: Vec<DocumentBondCapacityRecordV1>,
}
impl DocumentBondCapacityV1 {
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }
    #[must_use]
    pub fn records(&self) -> &[DocumentBondCapacityRecordV1] {
        &self.records
    }
}

/// Authenticate, extract, and calculate all selected roots without mutation.
pub fn inspect_document_bond_capacity_v1(
    observation: &SessionDocumentObservationV1,
    request: &DocumentBondCapacityRequestV1,
) -> Result<DocumentBondCapacityV1, DocumentBondCapacityErrorV1> {
    verify_molecule_observation_v1(
        observation,
        request.expected_revision,
        &request.expected_digest,
    )?;
    let snapshot = observation.snapshot();
    let projection = observation.projection();
    let document = TypedDocument::parse(snapshot.cdml())?;
    let document_paint_orders = document_direct_root_paint_orders_v1(projection)
        .map_err(|_| DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
    let mut records = Vec::new();
    records
        .try_reserve_exact(request.molecule_ids.len())
        .map_err(|_| DocumentBondCapacityErrorV1::ResourceAllocation)?;
    for molecule_id in &request.molecule_ids {
        let root = direct_projection_molecule_v1(projection, molecule_id)?;
        let source_id = root
            .source_id()
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
        let molecule = document
            .core_molecule(molecule_id)?
            .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?;
        if molecule.source_id().as_str() != source_id {
            return Err(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch.into());
        }
        let source = DocumentBondCapacitySourceV1 {
            molecule_id: copied_id(molecule_id)?,
            projection_key: copied(root.projection_key().as_str())?,
            source_id: copied(source_id)?,
            document_paint_order: *document_paint_orders
                .get(molecule_id)
                .ok_or(DocumentMoleculeInspectionErrorV1::ProjectionRootMismatch)?,
            authored_name: root.name().map(copied).transpose()?,
        };
        records.push(DocumentBondCapacityRecordV1 {
            source,
            outcome: evaluate_document_molecule_neutral_capacity_v1(&molecule)?,
        });
    }
    records.sort_by_key(|record| record.source.document_paint_order);
    Ok(DocumentBondCapacityV1 {
        schema: DOCUMENT_BOND_CAPACITY_SCHEMA_V1,
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        records,
    })
}

pub(crate) fn evaluate_document_molecule_neutral_capacity_v1(
    molecule: &Molecule,
) -> Result<DocumentBondCapacityOutcomeV1, DocumentBondCapacityErrorV1> {
    if molecule.atoms().is_empty()
        || !molecule.groups().is_empty()
        || !molecule.texts().is_empty()
        || !molecule.queries().is_empty()
    {
        return Ok(not_checked(
            DocumentBondCapacityNotCheckedReasonV1::NonAtomVertex,
        ));
    }
    let mut indices = HashMap::new();
    indices
        .try_reserve(molecule.atoms().len())
        .map_err(|_| DocumentBondCapacityErrorV1::ResourceAllocation)?;
    let mut atoms = Vec::new();
    atoms
        .try_reserve_exact(molecule.atoms().len())
        .map_err(|_| DocumentBondCapacityErrorV1::ResourceAllocation)?;
    for atom in molecule.atoms() {
        if atom.formal_charge().is_some_and(|charge| charge != 0) {
            return Ok(not_checked(
                DocumentBondCapacityNotCheckedReasonV1::NonNeutralCharge,
            ));
        }
        if atom.valence().is_some() || atom.multiplicity().is_some() || atom.free_sites().is_some()
        {
            return Ok(not_checked(
                DocumentBondCapacityNotCheckedReasonV1::AuthoredAtomCapacityFact,
            ));
        }
        let Some(element) = atom.element() else {
            return Ok(not_checked(
                DocumentBondCapacityNotCheckedReasonV1::UnsupportedElement,
            ));
        };
        if !matches!(
            element,
            "H" | "B" | "C" | "N" | "O" | "F" | "Cl" | "Br" | "I"
        ) {
            return Ok(not_checked(
                DocumentBondCapacityNotCheckedReasonV1::UnsupportedElement,
            ));
        }
        let index = atoms.len();
        if indices.insert(atom.identity().clone(), index).is_some() {
            return Ok(not_checked(
                DocumentBondCapacityNotCheckedReasonV1::UnsupportedBondEndpoint,
            ));
        }
        atoms.push(NeutralBondCapacityAtomV1 {
            source_id: Some(copied(atom.source_id().as_str())?),
            element: copied(element)?,
            explicit_hydrogens: NeutralBondCapacityExplicitHydrogensFactV1 {
                was_authored: atom.explicit_hydrogens().is_some(),
                value_or_zero: atom.explicit_hydrogens().unwrap_or(0),
            },
            formal_charge: NeutralBondCapacityFormalChargeFactV1 {
                was_authored: atom.formal_charge().is_some(),
                value_or_zero: atom.formal_charge().unwrap_or(0),
            },
        });
    }
    let mut bonds = Vec::new();
    bonds
        .try_reserve_exact(molecule.bonds().len())
        .map_err(|_| DocumentBondCapacityErrorV1::ResourceAllocation)?;
    for bond in molecule.bonds() {
        if bond.aromatic() == Some(true) || bond.order() == Some(BondOrder::Aromatic) {
            return Ok(not_checked(
                DocumentBondCapacityNotCheckedReasonV1::AromaticFact,
            ));
        }
        let order = match bond.order() {
            Some(BondOrder::Single) => 1,
            Some(BondOrder::Double) => 2,
            Some(BondOrder::Triple) => 3,
            _ => {
                return Ok(not_checked(
                    DocumentBondCapacityNotCheckedReasonV1::UnsupportedBondOrder,
                ));
            }
        };
        let (VertexRef::Atom(start), VertexRef::Atom(end)) = (bond.start(), bond.end()) else {
            return Ok(not_checked(
                DocumentBondCapacityNotCheckedReasonV1::UnsupportedBondEndpoint,
            ));
        };
        let (Some(start), Some(end)) = (indices.get(start), indices.get(end)) else {
            return Ok(not_checked(
                DocumentBondCapacityNotCheckedReasonV1::UnsupportedBondEndpoint,
            ));
        };
        bonds.push(NeutralBondCapacityBondV1 {
            start: *start,
            end: *end,
            order,
        });
    }
    let atoms = evaluate_neutral_bond_capacity_v1(&atoms, &bonds)
        .map_err(DocumentBondCapacityErrorV1::Domain)?;
    let outcome = if atoms.iter().any(|atom| {
        matches!(
            atom.outcome,
            NeutralBondCapacityAtomOutcomeV1::ExceedsCapacity { .. }
        )
    }) {
        DocumentBondCapacityOutcomeV1::ExceedsCapacity { atoms }
    } else {
        DocumentBondCapacityOutcomeV1::WithinCapacity { atoms }
    };
    Ok(outcome)
}

fn not_checked(reason: DocumentBondCapacityNotCheckedReasonV1) -> DocumentBondCapacityOutcomeV1 {
    DocumentBondCapacityOutcomeV1::NotChecked { reason }
}
fn copied(value: &str) -> Result<String, DocumentBondCapacityErrorV1> {
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentBondCapacityErrorV1::ResourceAllocation)?;
    result.push_str(value);
    Ok(result)
}
fn copied_id(
    value: &DocumentObjectIdV1,
) -> Result<DocumentObjectIdV1, DocumentBondCapacityErrorV1> {
    DocumentObjectIdV1::parse(copied(value.as_str())?)
        .map_err(|_| DocumentBondCapacityErrorV1::OpaqueIdInvariant)
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentBondCapacityRequestErrorV1 {
    #[error("bond capacity requires at least one selected molecule")]
    EmptySelection,
    #[error("bond capacity selection repeats a durable molecule")]
    DuplicateMolecule,
}
#[derive(Debug, Error)]
pub enum DocumentBondCapacityErrorV1 {
    #[error(transparent)]
    Inspection(#[from] DocumentMoleculeInspectionErrorV1),
    #[error(transparent)]
    Document(#[from] crate::TypedDocumentError),
    #[error(transparent)]
    CoreProjection(#[from] crate::CoreProjectionError),
    #[error(transparent)]
    Domain(#[from] ferrum_domain::NeutralBondCapacityErrorV1),
    #[error("bond capacity could not reserve owned receipt storage")]
    ResourceAllocation,
    #[error("validated opaque molecule selector could not be reconstructed")]
    OpaqueIdInvariant,
}
