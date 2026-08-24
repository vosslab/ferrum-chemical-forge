//! Renderer-gated session transaction for closed compact-group replacement.

use thiserror::Error;

use super::{
    AdmittedSessionTransitionRefusalV1, AuthoringCapabilityIssuerV1, DocumentFenceV1,
    DocumentObjectIdV1, DocumentSession, PersistentId, PreparedSessionTransitionV1, RevisionState,
    SessionOperationResultV1,
};
use crate::chemistry::{
    DocumentOrdinaryAttachmentAvailabilityV1, DocumentOrdinaryAttachmentReasonV1,
    OrdinaryAttachmentCandidateWitnessV1, admit_candidate_ordinary_attachment_capacity_v1,
};
use crate::{TypedClass, TypedDocument};
use ferrum_chemistry::OrdinaryAttachmentProfileV1;

/// Fenced durable target for the internal compact-group materialization experiment.
#[derive(Clone, Debug, PartialEq)]
pub struct CompactGroupMaterializationRequestV1 {
    fence: DocumentFenceV1,
    molecule_id: DocumentObjectIdV1,
    compact_group_id: DocumentObjectIdV1,
}

impl CompactGroupMaterializationRequestV1 {
    #[must_use]
    pub const fn new(
        fence: DocumentFenceV1,
        molecule_id: DocumentObjectIdV1,
        compact_group_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            fence,
            molecule_id,
            compact_group_id,
        }
    }
}

/// Closed preparation and redemption refusals for the internal experiment.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CompactGroupMaterializationRefusalV1 {
    #[error("compact-group materialization source is stale")]
    StaleObservation,
    #[error("compact-group materialization receipt belongs to another session")]
    ForeignSession,
    #[error("compact-group materialization source digest differs")]
    DigestMismatch,
    #[error("selected molecule is not a direct molecular root")]
    UnknownDirectMolecule,
    #[error("selected compact group is not a direct child of the selected root")]
    UnknownCompactGroup,
    #[error("compact-group materialization needs a supported retained group profile")]
    UnsupportedDocument,
    #[error("compact-group catalog key is not admitted by this experiment")]
    NotYetSupported,
    #[error("compact-group materialization exceeds a supported resource limit")]
    ResourceLimit,
    #[error("compact-group materialization was refused by renderer admission")]
    RendererAdmission,
}

/// Typed internal outcome with one durable replacement focus address.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactGroupMaterializationResultV1 {
    created_atom_count: usize,
    created_internal_bond_count: usize,
    replacement_focus_target: DocumentObjectIdV1,
}

impl CompactGroupMaterializationResultV1 {
    #[must_use]
    pub const fn created_atom_count(&self) -> usize {
        self.created_atom_count
    }
    #[must_use]
    pub const fn created_internal_bond_count(&self) -> usize {
        self.created_internal_bond_count
    }
    /// Return the recipe-defined exterior attachment atom after replacement.
    #[must_use]
    pub const fn replacement_focus_target(&self) -> &DocumentObjectIdV1 {
        &self.replacement_focus_target
    }
}

/// Opaque one-use candidate retaining exact session-owned IDs until commit.
#[derive(Debug)]
pub struct PendingCompactGroupMaterializationV1 {
    issuer: AuthoringCapabilityIssuerV1,
    transition: PreparedSessionTransitionV1,
    result: CompactGroupMaterializationResultV1,
}

impl PendingCompactGroupMaterializationV1 {
    #[must_use]
    pub fn is_consumed_v1(&self) -> bool {
        self.transition.is_consumed_v1()
    }

    #[must_use]
    pub const fn result(&self) -> &CompactGroupMaterializationResultV1 {
        &self.result
    }
}

impl DocumentSession {
    /// Prepare one closed compact replacement without changing history or IDs.
    pub fn prepare_compact_group_materialization_v1(
        &mut self,
        request: &CompactGroupMaterializationRequestV1,
    ) -> Result<PendingCompactGroupMaterializationV1, CompactGroupMaterializationRefusalV1> {
        require_fence(self, request.fence)?;
        let source = self.current_document_v1();
        let molecule = resolve_direct_molecule(source, &request.molecule_id)?;
        let group = resolve_direct_group(source, &request.compact_group_id, &request.molecule_id)?;
        let source_facts = source
            .compact_group_materialization_source_v1(&molecule, &group)
            .map_err(|_| CompactGroupMaterializationRefusalV1::RendererAdmission)?;
        let definition =
            crate::compact_group_v1::materialization_definition_v1(source_facts.catalog_key)
                .ok_or(CompactGroupMaterializationRefusalV1::NotYetSupported)?;
        admit_existing_attachment(source, &request.molecule_id, &group, &source_facts)?;
        let ((atom_ids, bond_ids), effects) = self
            .reserve_generated_ids_for_transition_v1(|mut sequences, indexed| {
                let mut atom_ids = Vec::with_capacity(definition.atoms.len());
                let mut bond_ids = Vec::with_capacity(definition.bonds.len());
                for _ in definition.atoms {
                    let (id, after) = sequences.reserve_atom(indexed)?;
                    atom_ids.push(id);
                    sequences = after;
                }
                for _ in definition.bonds {
                    let (id, after) = sequences.reserve_bond(indexed)?;
                    bond_ids.push(id);
                    sequences = after;
                }
                Ok(((atom_ids, bond_ids), sequences))
            })
            .map_err(|_| CompactGroupMaterializationRefusalV1::ResourceLimit)?;
        let candidate = source
            .with_materialized_compact_group_v1(&molecule, &group, definition, &atom_ids, &bond_ids)
            .map_err(|_| CompactGroupMaterializationRefusalV1::RendererAdmission)?;
        let revision = self
            .next_revision_v1()
            .ok_or(CompactGroupMaterializationRefusalV1::UnsupportedDocument)?;
        let state = RevisionState::from_document(revision, candidate)
            .map_err(|_| CompactGroupMaterializationRefusalV1::UnsupportedDocument)?;
        let replacement_focus_target = DocumentObjectIdV1::from_class_source(
            "molecule/atom",
            atom_ids
                .first()
                .ok_or(CompactGroupMaterializationRefusalV1::UnsupportedDocument)?
                .as_str(),
        )
        .map_err(|_| CompactGroupMaterializationRefusalV1::UnsupportedDocument)?;
        let transition = self
            .prepare_changed_session_transition_v1(
                request.fence.revision(),
                request.fence.digest(),
                state,
                effects,
            )
            .map_err(map_prepare_error)?;
        Ok(PendingCompactGroupMaterializationV1 {
            issuer: self.authoring_capability_issuer_v1(),
            transition,
            result: CompactGroupMaterializationResultV1 {
                created_atom_count: atom_ids.len(),
                created_internal_bond_count: bond_ids.len(),
                replacement_focus_target,
            },
        })
    }

    /// Install one renderer-admitted replacement as a single history transition.
    pub fn commit_compact_group_materialization_v1(
        &mut self,
        pending: &mut PendingCompactGroupMaterializationV1,
    ) -> Result<
        (
            CompactGroupMaterializationResultV1,
            SessionOperationResultV1,
        ),
        CompactGroupMaterializationRefusalV1,
    > {
        if !pending
            .issuer
            .same_issuer(&self.authoring_capability_issuer_v1())
        {
            return Err(CompactGroupMaterializationRefusalV1::ForeignSession);
        }
        let operation = self
            .commit_session_operation_transition_v1(&mut pending.transition)
            .map_err(map_commit_error)?;
        Ok((pending.result.clone(), operation))
    }
}

fn map_prepare_error(error: super::DocumentSessionError) -> CompactGroupMaterializationRefusalV1 {
    match error {
        super::DocumentSessionError::RendererAdmission => {
            CompactGroupMaterializationRefusalV1::RendererAdmission
        }
        _ => CompactGroupMaterializationRefusalV1::UnsupportedDocument,
    }
}

fn map_commit_error(
    error: AdmittedSessionTransitionRefusalV1,
) -> CompactGroupMaterializationRefusalV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            CompactGroupMaterializationRefusalV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Replayed
        | AdmittedSessionTransitionRefusalV1::StaleSnapshot => {
            CompactGroupMaterializationRefusalV1::StaleObservation
        }
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            CompactGroupMaterializationRefusalV1::RendererAdmission
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            CompactGroupMaterializationRefusalV1::UnsupportedDocument
        }
        AdmittedSessionTransitionRefusalV1::HistoryCapacity => {
            CompactGroupMaterializationRefusalV1::ResourceLimit
        }
    }
}

fn admit_existing_attachment(
    source: &TypedDocument,
    molecule_id: &DocumentObjectIdV1,
    group_id: &PersistentId,
    facts: &crate::CompactGroupMaterializationSourceV1,
) -> Result<(), CompactGroupMaterializationRefusalV1> {
    let (Some(exterior_atom), Some(exterior_bond)) = (&facts.exterior_atom, &facts.exterior_bond)
    else {
        return Ok(());
    };
    let molecule = source
        .core_molecule(molecule_id)
        .map_err(|_| CompactGroupMaterializationRefusalV1::UnsupportedDocument)?
        .ok_or(CompactGroupMaterializationRefusalV1::UnknownDirectMolecule)?;
    let anchor = molecule
        .atoms()
        .iter()
        .find(|atom| {
            atom.source_id()
                .is_some_and(|id| id.as_str() == exterior_atom.as_str())
        })
        .ok_or(CompactGroupMaterializationRefusalV1::UnsupportedDocument)?;
    let group = molecule
        .groups()
        .iter()
        .find(|entry| {
            entry
                .source_id()
                .is_some_and(|id| id.as_str() == group_id.as_str())
        })
        .ok_or(CompactGroupMaterializationRefusalV1::UnsupportedDocument)?;
    let bond = molecule
        .bonds()
        .iter()
        .find(|entry| {
            entry
                .source_id()
                .is_some_and(|id| id.as_str() == exterior_bond.as_str())
        })
        .ok_or(CompactGroupMaterializationRefusalV1::UnsupportedDocument)?;
    let witness = OrdinaryAttachmentCandidateWitnessV1::new(
        anchor.identity().clone(),
        group.identity().clone(),
        bond.identity().clone(),
        OrdinaryAttachmentProfileV1::NormalSingle,
    );
    match admit_candidate_ordinary_attachment_capacity_v1(&molecule, &witness)
        .map_err(|_| CompactGroupMaterializationRefusalV1::UnsupportedDocument)?
    {
        DocumentOrdinaryAttachmentAvailabilityV1::Available => Ok(()),
        DocumentOrdinaryAttachmentAvailabilityV1::Unavailable { reason, .. } => match reason {
            DocumentOrdinaryAttachmentReasonV1::ResourceLimit => {
                Err(CompactGroupMaterializationRefusalV1::ResourceLimit)
            }
            _ => Err(CompactGroupMaterializationRefusalV1::UnsupportedDocument),
        },
    }
}

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), CompactGroupMaterializationRefusalV1> {
    let snapshot = session
        .snapshot()
        .map_err(|_| CompactGroupMaterializationRefusalV1::UnsupportedDocument)?;
    if snapshot.revision() != fence.revision() {
        return Err(CompactGroupMaterializationRefusalV1::StaleObservation);
    }
    if *snapshot.digest() != fence.digest() {
        return Err(CompactGroupMaterializationRefusalV1::DigestMismatch);
    }
    Ok(())
}

fn resolve_direct_molecule(
    document: &TypedDocument,
    id: &DocumentObjectIdV1,
) -> Result<PersistentId, CompactGroupMaterializationRefusalV1> {
    let record = document
        .resolve_document_object_id(id)
        .filter(|record| {
            record.class() == TypedClass::Molecule && record.path().components().len() == 1
        })
        .ok_or(CompactGroupMaterializationRefusalV1::UnknownDirectMolecule)?;
    PersistentId::new(
        record
            .attribute("id")
            .ok_or(CompactGroupMaterializationRefusalV1::UnsupportedDocument)?
            .to_owned(),
    )
    .map_err(|_| CompactGroupMaterializationRefusalV1::UnsupportedDocument)
}

fn resolve_direct_group(
    document: &TypedDocument,
    id: &DocumentObjectIdV1,
    molecule_id: &DocumentObjectIdV1,
) -> Result<PersistentId, CompactGroupMaterializationRefusalV1> {
    let molecule = document
        .resolve_document_object_id(molecule_id)
        .filter(|record| {
            record.class() == TypedClass::Molecule && record.path().components().len() == 1
        })
        .ok_or(CompactGroupMaterializationRefusalV1::UnknownDirectMolecule)?;
    let record = document
        .resolve_document_object_id(id)
        .filter(|record| {
            record.class() == TypedClass::CompactGroup && record.path().components().len() == 2
        })
        .ok_or(CompactGroupMaterializationRefusalV1::UnknownCompactGroup)?;
    if record.path().components().first() != molecule.path().components().first() {
        return Err(CompactGroupMaterializationRefusalV1::UnknownCompactGroup);
    }
    PersistentId::new(
        record
            .attribute("id")
            .ok_or(CompactGroupMaterializationRefusalV1::UnsupportedDocument)?
            .to_owned(),
    )
    .map_err(|_| CompactGroupMaterializationRefusalV1::UnsupportedDocument)
}
