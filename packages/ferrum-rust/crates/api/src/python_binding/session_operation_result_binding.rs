use ferrum_document::{
    AtomCreatedOutcomeV1, BondCreatedOutcomeV1, CreatedPresentationRootKindV1,
    CreatedPresentationRootOutcomeV1, DirectBondOperationOutcomeV1,
    DocumentCompactGroupMaterializationResultV1, DocumentMoleculeHydrogenMaterializationResultV1,
    InterchangeRecordBatchInsertedOutcomeV1, MoleculeInsertedOutcomeV1, ReactionCreatedOutcomeV1,
    ReactionDefinitionDeletedOutcomeV1, ReactionMembershipReplacedOutcomeV1,
    SessionOperationOutcomeV1, SessionOperationResultV1,
};
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use super::projection_binding::PySessionDocumentObservationV1;

/// Immutable result of one accepted document mutation or history transition.
///
/// `observation` owns the one authoritative post-operation snapshot and projection.
#[pyclass(frozen, name = "SessionOperationResultV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PySessionOperationResultV1 {
    #[pyo3(get)]
    observation: PySessionDocumentObservationV1,
    #[pyo3(get)]
    outcome: PySessionOperationOutcomeV1,
}

impl From<SessionOperationResultV1> for PySessionOperationResultV1 {
    fn from(result: SessionOperationResultV1) -> Self {
        Self {
            observation: result.observation().clone().into(),
            outcome: result.outcome().into(),
        }
    }
}

/// Closed generic operation outcome with optional operation-specific facts.
#[pyclass(frozen, name = "SessionOperationOutcomeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PySessionOperationOutcomeV1 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    direct_bond: Option<PyDirectBondOperationOutcomeV1>,
    #[pyo3(get)]
    atom_created: Option<PyAtomCreatedOutcomeV1>,
    #[pyo3(get)]
    bond_created: Option<PyBondCreatedOutcomeV1>,
    #[pyo3(get)]
    molecule_hydrogens_materialized: Option<PyMoleculeHydrogensMaterializedOutcomeV1>,
    #[pyo3(get)]
    compact_group_materialized: Option<PyCompactGroupMaterializedOutcomeV1>,
    #[pyo3(get)]
    molecule_inserted: Option<PyMoleculeInsertedOutcomeV1>,
    #[pyo3(get)]
    interchange_record_batch_inserted: Option<PyInterchangeRecordBatchInsertedOutcomeV1>,
    #[pyo3(get)]
    reaction_created: Option<PyReactionCreatedOutcomeV1>,
    #[pyo3(get)]
    reaction_membership_replaced: Option<PyReactionMembershipReplacedOutcomeV1>,
    #[pyo3(get)]
    reaction_definition_deleted: Option<PyReactionDefinitionDeletedOutcomeV1>,
    #[pyo3(get)]
    created_presentation_root: Option<PyCreatedPresentationRootOutcomeV1>,
}

impl From<&SessionOperationOutcomeV1> for PySessionOperationOutcomeV1 {
    fn from(outcome: &SessionOperationOutcomeV1) -> Self {
        match outcome {
            SessionOperationOutcomeV1::Standard => Self {
                kind: "standard".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::DirectBondV1(outcome) => Self {
                kind: "direct_bond_v1".to_owned(),
                direct_bond: Some(outcome.into()),
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::AtomCreatedV1(outcome) => Self {
                kind: "atom_created_v1".to_owned(),
                direct_bond: None,
                atom_created: Some(outcome.into()),
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::BondCreatedV1(outcome) => Self {
                kind: "bond_created_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: Some(outcome.into()),
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::MoleculeHydrogensMaterializedV1(outcome) => Self {
                kind: "molecule_hydrogens_materialized_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: Some(outcome.into()),
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) => Self {
                kind: "compact_group_materialized_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: Some(outcome.into()),
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::CatalogMoleculePlacementV1(_) => Self {
                kind: "catalog_molecule_placement_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::CreatedPresentationRootV1(outcome) => Self {
                kind: "created_presentation_root_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: Some(outcome.into()),
            },
            SessionOperationOutcomeV1::ReactionCreatedV1(outcome) => Self {
                kind: "reaction_created_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: Some(outcome.into()),
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::ReactionMembershipReplacedV1(outcome) => Self {
                kind: "reaction_membership_replaced_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: Some(outcome.into()),
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::ReactionDefinitionDeletedV1(outcome) => Self {
                kind: "reaction_definition_deleted_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: Some(outcome.into()),
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::MoleculeInsertedV1(outcome) => Self {
                kind: "molecule_inserted_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: Some(outcome.into()),
                interchange_record_batch_inserted: None,
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
            SessionOperationOutcomeV1::InterchangeRecordBatchInsertedV1(outcome) => Self {
                kind: "interchange_record_batch_inserted_v1".to_owned(),
                direct_bond: None,
                atom_created: None,
                bond_created: None,
                molecule_hydrogens_materialized: None,
                compact_group_materialized: None,
                molecule_inserted: None,
                interchange_record_batch_inserted: Some(outcome.into()),
                reaction_created: None,
                reaction_membership_replaced: None,
                reaction_definition_deleted: None,
                created_presentation_root: None,
            },
        }
    }
}

/// Durable atom identity issued only after generic transition redemption.
#[pyclass(frozen, name = "AtomCreatedOutcomeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyAtomCreatedOutcomeV1 {
    #[pyo3(get)]
    atom_identifier: String,
}

impl From<&AtomCreatedOutcomeV1> for PyAtomCreatedOutcomeV1 {
    fn from(outcome: &AtomCreatedOutcomeV1) -> Self {
        Self {
            atom_identifier: outcome.atom_identifier().as_str().to_owned(),
        }
    }
}

/// Durable bond identity issued only after generic transition redemption.
#[pyclass(frozen, name = "BondCreatedOutcomeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyBondCreatedOutcomeV1 {
    #[pyo3(get)]
    bond_identifier: String,
}

impl From<&BondCreatedOutcomeV1> for PyBondCreatedOutcomeV1 {
    fn from(outcome: &BondCreatedOutcomeV1) -> Self {
        Self {
            bond_identifier: outcome.bond_identifier().as_str().to_owned(),
        }
    }
}

/// Durable facts from one committed generic explicit-hydrogen materialization.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "MoleculeHydrogensMaterializedOutcomeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyMoleculeHydrogensMaterializedOutcomeV1 {
    #[pyo3(get)]
    anchor_atom_identifier: String,
    #[pyo3(get)]
    added_hydrogen_count: usize,
    #[pyo3(get)]
    changed: bool,
}

impl From<&DocumentMoleculeHydrogenMaterializationResultV1>
    for PyMoleculeHydrogensMaterializedOutcomeV1
{
    fn from(outcome: &DocumentMoleculeHydrogenMaterializationResultV1) -> Self {
        Self {
            anchor_atom_identifier: outcome.anchor_atom_id().as_str().to_owned(),
            added_hydrogen_count: outcome.added_hydrogen_count(),
            changed: outcome.changed(),
        }
    }
}

/// Durable identities from one committed compact-group materialization.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "CompactGroupMaterializedOutcomeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyCompactGroupMaterializedOutcomeV1 {
    #[pyo3(get)]
    molecule_identifier: String,
    #[pyo3(get)]
    compact_group_identifier: String,
    #[pyo3(get)]
    replacement_focus_atom_identifier: String,
}

impl From<&DocumentCompactGroupMaterializationResultV1> for PyCompactGroupMaterializedOutcomeV1 {
    fn from(outcome: &DocumentCompactGroupMaterializationResultV1) -> Self {
        Self {
            molecule_identifier: outcome.molecule_id().as_str().to_owned(),
            compact_group_identifier: outcome.compact_group_id().as_str().to_owned(),
            replacement_focus_atom_identifier: outcome.focus_atom_id().as_str().to_owned(),
        }
    }
}

/// Closed semantic class of a committed presentation root.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "CreatedPresentationRootKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyCreatedPresentationRootKindV1 {
    StraightNormalArrow,
    StraightEquilibriumArrow,
    Plus,
    CurvedTerminalArrow,
    CurvedEquilibriumArrow,
    Path,
    Vector,
}

impl From<CreatedPresentationRootKindV1> for PyCreatedPresentationRootKindV1 {
    fn from(kind: CreatedPresentationRootKindV1) -> Self {
        match kind {
            CreatedPresentationRootKindV1::StraightNormalArrow => Self::StraightNormalArrow,
            CreatedPresentationRootKindV1::StraightEquilibriumArrow => {
                Self::StraightEquilibriumArrow
            }
            CreatedPresentationRootKindV1::Plus => Self::Plus,
            CreatedPresentationRootKindV1::CurvedTerminalArrow => Self::CurvedTerminalArrow,
            CreatedPresentationRootKindV1::CurvedEquilibriumArrow => Self::CurvedEquilibriumArrow,
            CreatedPresentationRootKindV1::Path => Self::Path,
            CreatedPresentationRootKindV1::Vector => Self::Vector,
        }
    }
}

/// Durable root facts issued by one committed generic visual operation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "CreatedPresentationRootOutcomeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyCreatedPresentationRootOutcomeV1 {
    #[pyo3(get)]
    document_object_id: String,
    #[pyo3(get)]
    kind: PyCreatedPresentationRootKindV1,
}

impl From<&CreatedPresentationRootOutcomeV1> for PyCreatedPresentationRootOutcomeV1 {
    fn from(outcome: &CreatedPresentationRootOutcomeV1) -> Self {
        Self {
            document_object_id: outcome.root().document_object_id().as_str().to_owned(),
            kind: outcome.kind().into(),
        }
    }
}

/// Durable IDs created only by one committed complete molecule insertion.
#[pyclass(frozen, name = "MoleculeInsertedOutcomeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMoleculeInsertedOutcomeV1 {
    molecule_identifier: String,
    atom_identifiers: Vec<String>,
    bond_identifiers: Vec<String>,
}

impl From<&MoleculeInsertedOutcomeV1> for PyMoleculeInsertedOutcomeV1 {
    fn from(outcome: &MoleculeInsertedOutcomeV1) -> Self {
        Self {
            molecule_identifier: outcome.molecule_identifier().as_str().to_owned(),
            atom_identifiers: outcome
                .atom_identifiers()
                .iter()
                .map(|identifier| identifier.as_str().to_owned())
                .collect(),
            bond_identifiers: outcome
                .bond_identifiers()
                .iter()
                .map(|identifier| identifier.as_str().to_owned())
                .collect(),
        }
    }
}

#[pymethods]
impl PyMoleculeInsertedOutcomeV1 {
    #[getter]
    fn molecule_identifier(&self) -> String {
        self.molecule_identifier.clone()
    }

    #[getter]
    fn atom_identifiers(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, &self.atom_identifiers)?.unbind())
    }

    #[getter]
    fn bond_identifiers(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, &self.bond_identifiers)?.unbind())
    }
}

/// Source-ordered committed identity facts for one interchange batch.
#[pyclass(
    frozen,
    name = "InterchangeRecordBatchInsertedOutcomeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyInterchangeRecordBatchInsertedOutcomeV1 {
    records: Vec<PyMoleculeInsertedOutcomeV1>,
}

impl From<&InterchangeRecordBatchInsertedOutcomeV1> for PyInterchangeRecordBatchInsertedOutcomeV1 {
    fn from(outcome: &InterchangeRecordBatchInsertedOutcomeV1) -> Self {
        Self {
            records: outcome.records().iter().map(Into::into).collect(),
        }
    }
}

#[pymethods]
impl PyInterchangeRecordBatchInsertedOutcomeV1 {
    #[getter]
    fn records(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, self.records.iter().cloned())?.unbind())
    }
}

/// Durable reaction identifier issued only after generic transition redemption.
#[pyclass(frozen, name = "ReactionCreatedOutcomeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyReactionCreatedOutcomeV1 {
    #[pyo3(get)]
    reaction_document_object_id: String,
}

impl From<&ReactionCreatedOutcomeV1> for PyReactionCreatedOutcomeV1 {
    fn from(outcome: &ReactionCreatedOutcomeV1) -> Self {
        Self {
            reaction_document_object_id: outcome.reaction_document_object_id().as_str().to_owned(),
        }
    }
}

/// Durable reaction identifier after generic membership replacement.
#[pyclass(
    frozen,
    name = "ReactionMembershipReplacedOutcomeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyReactionMembershipReplacedOutcomeV1 {
    #[pyo3(get)]
    reaction_document_object_id: String,
}

impl From<&ReactionMembershipReplacedOutcomeV1> for PyReactionMembershipReplacedOutcomeV1 {
    fn from(outcome: &ReactionMembershipReplacedOutcomeV1) -> Self {
        Self {
            reaction_document_object_id: outcome.reaction_document_object_id().as_str().to_owned(),
        }
    }
}

/// Durable reaction identifier after generic definition deletion.
#[pyclass(
    frozen,
    name = "ReactionDefinitionDeletedOutcomeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyReactionDefinitionDeletedOutcomeV1 {
    #[pyo3(get)]
    reaction_document_object_id: String,
}

impl From<&ReactionDefinitionDeletedOutcomeV1> for PyReactionDefinitionDeletedOutcomeV1 {
    fn from(outcome: &ReactionDefinitionDeletedOutcomeV1) -> Self {
        Self {
            reaction_document_object_id: outcome.reaction_document_object_id().as_str().to_owned(),
        }
    }
}

/// Authoritative post-commit facts for a generic direct-bond outcome.
#[pyclass(frozen, name = "DirectBondOperationOutcomeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDirectBondOperationOutcomeV1 {
    #[pyo3(get)]
    bond_document_object_id: String,
    #[pyo3(get)]
    end_atom_document_object_id: String,
    #[pyo3(get)]
    second_created_atom_document_object_id: Option<String>,
    #[pyo3(get)]
    created_new_atom: bool,
    #[pyo3(get)]
    created_new_molecule: bool,
}

impl From<&DirectBondOperationOutcomeV1> for PyDirectBondOperationOutcomeV1 {
    fn from(outcome: &DirectBondOperationOutcomeV1) -> Self {
        Self {
            bond_document_object_id: outcome.bond_document_object_id().as_str().to_owned(),
            end_atom_document_object_id: outcome.end_atom_document_object_id().as_str().to_owned(),
            second_created_atom_document_object_id: outcome
                .second_created_atom_document_object_id()
                .map(|identifier| identifier.as_str().to_owned()),
            created_new_atom: outcome.created_new_atom(),
            created_new_molecule: outcome.created_new_molecule(),
        }
    }
}
