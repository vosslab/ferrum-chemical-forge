//! One closed shared-anchor cyclohexane mutation candidate.

use crate::{
    DocumentBondOrderV1, PersistentId, TypedDocument, TypedDocumentError,
    attached_cyclohexane_v1::{AttachedCyclohexaneCandidateV1, AttachedCyclohexaneVertexV1},
};

impl TypedDocument {
    /// Build the complete attached-C6 candidate without exposing a generic attachment seam.
    pub(crate) fn with_attach_cyclohexane_v1(
        &self,
        molecule_id: &PersistentId,
        anchor_atom_id: &PersistentId,
        added_atom_ids: &[PersistentId; 5],
        bond_ids: &[PersistentId; 6],
        candidate: &AttachedCyclohexaneCandidateV1,
    ) -> Result<Self, TypedDocumentError> {
        let added_atoms = candidate.added_atoms();
        let mut document =
            self.with_insert_atom(molecule_id, &added_atom_ids[0], "C", added_atoms[0])?;
        for index in 1..added_atom_ids.len() {
            document = document.with_insert_atom(
                molecule_id,
                &added_atom_ids[index],
                "C",
                added_atoms[index],
            )?;
        }
        for (bond_id, bond) in bond_ids.iter().zip(candidate.bonds()) {
            document = document.with_insert_bond(
                molecule_id,
                bond_id,
                attached_atom_id(anchor_atom_id, added_atom_ids, bond.start()),
                attached_atom_id(anchor_atom_id, added_atom_ids, bond.end()),
                crate::DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            )?;
        }
        Ok(document)
    }
}

fn attached_atom_id<'a>(
    anchor_atom_id: &'a PersistentId,
    added_atom_ids: &'a [PersistentId; 5],
    vertex: AttachedCyclohexaneVertexV1,
) -> &'a PersistentId {
    match vertex {
        AttachedCyclohexaneVertexV1::Anchor => anchor_atom_id,
        AttachedCyclohexaneVertexV1::Added(index) => &added_atom_ids[usize::from(index)],
    }
}
