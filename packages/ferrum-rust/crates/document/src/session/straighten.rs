//! Revision-bound whole-depiction straightening preparation.

use std::collections::HashSet;

use super::{
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, PersistentId,
    PreparedStraightenDepictionsV1, SessionOperationError, TypedClass,
};

impl DocumentSession {
    /// Prepare complete whole-depiction layouts from one current authoritative state.
    ///
    /// Each selector must name one direct-root typed molecule. The returned value
    /// is immutable and carries the exact revision and digest required for apply.
    pub fn prepare_straighten_depictions_v1(
        &self,
        expected_revision: u64,
        molecule_ids: Vec<DocumentObjectIdV1>,
        minimize_rotation: bool,
    ) -> Result<PreparedStraightenDepictionsV1, DocumentSessionError> {
        self.require_current(expected_revision)?;
        if molecule_ids.is_empty() {
            return Err(SessionOperationError::InvalidStraightenDepiction(
                "whole-depiction straightening requires at least one molecule".to_owned(),
            )
            .into());
        }
        let direct_molecule_count = self
            .current_document_v1()
            .root()
            .children_of(TypedClass::Molecule)
            .count();
        if molecule_ids.len() > direct_molecule_count {
            return Err(SessionOperationError::InvalidStraightenDepiction(
                "whole-depiction straightening cannot select more molecules than the current document contains"
                    .to_owned(),
            )
            .into());
        }
        let mut resolved = Vec::with_capacity(molecule_ids.len());
        let mut unique = HashSet::with_capacity(molecule_ids.len());
        for object_id in molecule_ids {
            if !unique.insert(object_id.clone()) {
                return Err(SessionOperationError::InvalidStraightenDepiction(
                    "whole-depiction straightening molecule targets must be unique".to_owned(),
                )
                .into());
            }
            let object_key = object_id.as_str().to_owned();
            let record = self
                .current_document_v1()
                .resolve_document_object_id(&object_id)?
                .ok_or_else(|| SessionOperationError::UnknownDocumentObject(object_key.clone()))?;
            if record.class() != TypedClass::Molecule {
                return Err(
                    SessionOperationError::InvalidMoleculeCoordinateTarget(object_key).into(),
                );
            }
            let source_id = record.attribute("id").ok_or_else(|| {
                SessionOperationError::InvalidMoleculeCoordinateTarget(
                    object_id.as_str().to_owned(),
                )
            })?;
            let source_id = PersistentId::new(source_id.to_owned()).map_err(|_| {
                SessionOperationError::InvalidMoleculeCoordinateTarget(
                    object_id.as_str().to_owned(),
                )
            })?;
            resolved.push((source_id, object_id));
        }
        let molecules = resolved
            .into_iter()
            .map(|(source_id, object_id)| {
                super::super::straighten_depiction_update_v1::prepare_molecule(
                    self.current_document_v1(),
                    &source_id,
                    object_id,
                    minimize_rotation,
                )
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(SessionOperationError::Candidate)?;
        PreparedStraightenDepictionsV1::new(
            self.current_revision_v1(),
            self.current_digest_v1(),
            molecules,
        )
        .map_err(|error| {
            SessionOperationError::InvalidStraightenDepiction(error.to_string()).into()
        })
    }
}
