//! Public semantic endpoint values for native direct-bond authoring.

use crate::{DirectBondPoint2V1, PersistentId, SessionOperationResultV1};

/// Explicitly resolved input for native, noninteractive direct-bond mutation.
///
/// Pointer probing, viewport transforms, render plans, and admission proofs are
/// deliberately outside this semantic input boundary.
#[derive(Clone, Debug, PartialEq)]
pub enum DirectBondEndpointIntent {
    ExistingAtom { atom: crate::DocumentObjectIdV1 },
    NewAtomAt { raw_point: DirectBondPoint2V1 },
}

/// Durable outcome of one accepted direct-bond transaction.
#[derive(Clone, Debug, PartialEq)]
pub struct CommittedDirectBondGestureV2 {
    bond: PersistentId,
    end_atom: PersistentId,
    second_created_atom: Option<PersistentId>,
    created_new_atom: bool,
    created_new_molecule: bool,
    result: SessionOperationResultV1,
}

impl CommittedDirectBondGestureV2 {
    pub(crate) fn new(
        bond: PersistentId,
        end_atom: PersistentId,
        second_created_atom: Option<PersistentId>,
        created_new_atom: bool,
        created_new_molecule: bool,
        result: SessionOperationResultV1,
    ) -> Self {
        Self {
            bond,
            end_atom,
            second_created_atom,
            created_new_atom,
            created_new_molecule,
            result,
        }
    }

    #[must_use]
    pub fn bond(&self) -> &PersistentId {
        &self.bond
    }
    #[must_use]
    pub fn end_atom(&self) -> &PersistentId {
        &self.end_atom
    }
    #[must_use]
    pub fn second_created_atom(&self) -> Option<&PersistentId> {
        self.second_created_atom.as_ref()
    }
    #[must_use]
    pub const fn created_new_atom(&self) -> bool {
        self.created_new_atom
    }
    #[must_use]
    pub const fn created_new_molecule(&self) -> bool {
        self.created_new_molecule
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
}
