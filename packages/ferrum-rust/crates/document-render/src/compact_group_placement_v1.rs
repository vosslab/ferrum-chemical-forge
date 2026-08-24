//! Renderer-preflight bridge for compact-group placement candidates.

use ferrum_document::{
    CompactGroupPlacementRefusalV1, CompactGroupPlacementRequestV1, DocumentSession,
    DocumentSessionError, PendingCompactGroupPlacementV1, SessionOperationResultV1,
    TypedDocumentError,
};
use thiserror::Error;

/// Closed renderer bridge outcome for compact-group placement.
#[derive(Debug, Error)]
pub enum CompactGroupPlacementErrorV1 {
    #[error("compact-group placement was refused: {0}")]
    Refusal(#[from] CompactGroupPlacementRefusalV1),
    #[error("compact-group candidate could not complete the normal document render plan")]
    RenderPreparation,
    #[error("compact-group placement receipt was already consumed")]
    Replayed,
}

/// Opaque receipt proving an exact compact-group candidate passed renderer admission.
#[derive(Debug)]
pub struct PreparedCompactGroupPlacementV1 {
    pending: PendingCompactGroupPlacementV1,
}

pub fn prepare_compact_group_placement_v1(
    session: &mut DocumentSession,
    request: &CompactGroupPlacementRequestV1,
) -> Result<PreparedCompactGroupPlacementV1, CompactGroupPlacementErrorV1> {
    let pending = session
        .prepare_compact_group_placement_v1(request)
        .map_err(|error| match error {
            CompactGroupPlacementRefusalV1::RendererAdmission => {
                CompactGroupPlacementErrorV1::RenderPreparation
            }
            other => CompactGroupPlacementErrorV1::Refusal(other),
        })?;
    Ok(PreparedCompactGroupPlacementV1 { pending })
}

pub fn commit_compact_group_placement_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedCompactGroupPlacementV1,
) -> Result<SessionOperationResultV1, CompactGroupPlacementErrorV1> {
    if prepared.pending.is_consumed_v1() {
        return Err(CompactGroupPlacementErrorV1::Replayed);
    }
    match session.commit_compact_group_placement_v1(&mut prepared.pending) {
        Ok(result) => Ok(result),
        Err(CompactGroupPlacementRefusalV1::RendererAdmission) => {
            Err(CompactGroupPlacementErrorV1::RenderPreparation)
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{
        CompactGroupCatalogKeyV1, CompactGroupPlacementModeV1, DocumentFenceV1,
        MoleculeInsertionAtomV1, MoleculeInsertionBondV1, MoleculeInsertionV1, Point3V1,
    };

    fn point(x: f64, y: f64) -> Point3V1 {
        Point3V1::new(x, y, 0.0).expect("finite point")
    }

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn session_with_molecule(
        atom_count: usize,
        bonds: Vec<MoleculeInsertionBondV1>,
    ) -> DocumentSession {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let atoms = (0..atom_count)
            .map(|index| {
                MoleculeInsertionAtomV1::new("C", point(index as f64 * 24.0, 0.0), None, None, None)
                    .expect("carbon")
            })
            .collect();
        let insertion = MoleculeInsertionV1::new(atoms, bonds).expect("molecule insertion");
        let revision = session.snapshot().expect("initial snapshot").revision();
        let mut pending = session
            .prepare_admitted_molecule_insertion_v1(revision, &insertion)
            .expect("candidate");
        session
            .commit_admitted_molecule_insertion_v1(revision, &mut pending)
            .expect("commit molecule");
        session
    }

    fn attached_request(session: &DocumentSession) -> CompactGroupPlacementRequestV1 {
        let revision = session.snapshot().expect("snapshot").revision();
        let observation = session.observe(revision).expect("observation");
        let molecule = &observation.projection().molecules()[0];
        CompactGroupPlacementRequestV1::new(
            fence(session),
            CompactGroupCatalogKeyV1::Methyl,
            point(36.0, 0.0),
            CompactGroupPlacementModeV1::Attached {
                molecule_id: molecule.id().expect("durable molecule").clone(),
                anchor_atom_id: molecule.atoms()[0].id().expect("durable atom").clone(),
            },
        )
    }

    #[test]
    fn free_typed_group_is_renderer_admitted_and_observable() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let request = CompactGroupPlacementRequestV1::new(
            fence(&session),
            CompactGroupCatalogKeyV1::Nitro,
            point(20.0, 30.0),
            CompactGroupPlacementModeV1::Free,
        );
        let mut prepared = prepare_compact_group_placement_v1(&mut session, &request)
            .expect("renderer-admitted compact group");
        let committed =
            commit_compact_group_placement_v1(&mut session, &mut prepared).expect("one transition");
        let molecule = &committed.observation().projection().molecules()[0];
        assert_eq!(molecule.compact_groups()[0].label(), "NO2");
        assert_eq!(molecule.bonds().len(), 0);
    }

    #[test]
    fn attached_group_is_undoable_and_redoable_as_one_transition() {
        let mut session = session_with_molecule(1, Vec::new());
        let before = session.snapshot().expect("before placement");
        let request = attached_request(&session);
        let mut prepared =
            prepare_compact_group_placement_v1(&mut session, &request).expect("attached placement");
        let committed = commit_compact_group_placement_v1(&mut session, &mut prepared)
            .expect("commit attached group");
        let after = committed.observation().snapshot().clone();
        assert_eq!(
            committed.observation().projection().molecules()[0]
                .compact_groups()
                .len(),
            1
        );
        let undone = session
            .undo(after.revision())
            .expect("undo group placement");
        assert_eq!(undone.observation().snapshot().cdml(), before.cdml());
        let redone = session
            .redo(undone.observation().snapshot().revision())
            .expect("redo group placement");
        assert_eq!(redone.observation().snapshot().cdml(), after.cdml());
    }

    #[test]
    fn full_valence_anchor_refuses_before_renderer_or_history_change() {
        let bonds = (1..5)
            .map(|index| {
                MoleculeInsertionBondV1::new(0, index, ferrum_document::DocumentBondOrderV1::Single)
            })
            .collect();
        let mut session = session_with_molecule(5, bonds);
        let before = session.snapshot().expect("before refusal");
        let request = attached_request(&session);
        assert!(matches!(
            prepare_compact_group_placement_v1(&mut session, &request),
            Err(CompactGroupPlacementErrorV1::Refusal(
                CompactGroupPlacementRefusalV1::AttachmentUnavailable
            ))
        ));
        assert_eq!(session.snapshot().expect("after refusal"), before);
    }

    #[test]
    fn unsupported_text_face_is_refused_before_placement_session_exists() {
        let source = r#"<cdml xmlns="urn:ferrum:cdml"><plus id="bad"><point x="1" y="2"/><font family="Arial"/></plus></cdml>"#;
        assert!(matches!(
            DocumentSession::load(source),
            Err(DocumentSessionError::Load(TypedDocumentError::UnsupportedTextFace {
                root_id,
                family,
            })) if root_id == "bad" && family == "Arial"
        ));
    }

    #[test]
    fn source_ids_matching_former_provisional_names_do_not_affect_placement() {
        let source = r#"<cdml xmlns="urn:ferrum:cdml"><plus id="ferrum-compact-group-candidate-molecule"><point x="1" y="2"/></plus><plus id="ferrum-compact-group-candidate-group"><point x="3" y="4"/></plus><plus id="ferrum-compact-group-candidate-bond"><point x="5" y="6"/></plus></cdml>"#;
        let mut session = DocumentSession::load(source).expect("source session");
        let request = CompactGroupPlacementRequestV1::new(
            fence(&session),
            CompactGroupCatalogKeyV1::Methyl,
            point(12.0, 0.0),
            CompactGroupPlacementModeV1::Free,
        );
        let mut prepared = prepare_compact_group_placement_v1(&mut session, &request)
            .expect("exact durable candidate ignores ordinary source IDs");
        let committed = commit_compact_group_placement_v1(&mut session, &mut prepared)
            .expect("placement succeeds");
        assert_eq!(
            committed.observation().projection().molecules()[0]
                .compact_groups()
                .len(),
            1
        );
    }

    #[test]
    fn stale_fence_refuses_without_creating_a_candidate() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let stale = fence(&session);
        let insertion = MoleculeInsertionV1::new(
            vec![
                MoleculeInsertionAtomV1::new("C", point(0.0, 0.0), None, None, None)
                    .expect("carbon"),
            ],
            Vec::new(),
        )
        .expect("insertion");
        let mut molecule = session
            .prepare_admitted_molecule_insertion_v1(0, &insertion)
            .expect("candidate");
        session
            .commit_admitted_molecule_insertion_v1(0, &mut molecule)
            .expect("advance session");
        let before = session.snapshot().expect("advanced session");
        let request = CompactGroupPlacementRequestV1::new(
            stale,
            CompactGroupCatalogKeyV1::Methyl,
            point(1.0, 0.0),
            CompactGroupPlacementModeV1::Free,
        );
        assert!(matches!(
            prepare_compact_group_placement_v1(&mut session, &request),
            Err(CompactGroupPlacementErrorV1::Refusal(
                CompactGroupPlacementRefusalV1::StaleObservation
            ))
        ));
        assert_eq!(session.snapshot().expect("after stale refusal"), before);
    }

    #[test]
    fn compact_catalog_key_parser_refuses_labels_and_unknown_keys() {
        assert!(CompactGroupCatalogKeyV1::parse("Me").is_none());
        assert!(CompactGroupCatalogKeyV1::parse("legacy_group").is_none());
    }

    #[test]
    fn stale_prepared_receipt_remains_a_stale_refusal_without_mutation() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let request = CompactGroupPlacementRequestV1::new(
            fence(&session),
            CompactGroupCatalogKeyV1::Methyl,
            point(12.0, 0.0),
            CompactGroupPlacementModeV1::Free,
        );
        let mut prepared =
            prepare_compact_group_placement_v1(&mut session, &request).expect("first receipt");
        let advance = CompactGroupPlacementRequestV1::new(
            fence(&session),
            CompactGroupCatalogKeyV1::Nitro,
            point(24.0, 0.0),
            CompactGroupPlacementModeV1::Free,
        );
        let mut advancing =
            prepare_compact_group_placement_v1(&mut session, &advance).expect("advancing receipt");
        commit_compact_group_placement_v1(&mut session, &mut advancing).expect("advance session");
        let advanced = session.snapshot().expect("advanced snapshot");

        for attempt in 0..2 {
            assert!(
                matches!(
                    commit_compact_group_placement_v1(&mut session, &mut prepared),
                    Err(CompactGroupPlacementErrorV1::Refusal(
                        CompactGroupPlacementRefusalV1::StaleObservation
                    ))
                ),
                "stale attempt {attempt} must remain a typed refusal"
            );
            assert_eq!(session.snapshot().expect("after stale refusal"), advanced);
        }
    }

    #[test]
    fn foreign_session_rejects_receipt_without_consuming_owner_receipt() {
        let mut owner = DocumentSession::create_empty_document_v1().expect("owner session");
        let mut foreign = DocumentSession::create_empty_document_v1().expect("foreign session");
        let request = CompactGroupPlacementRequestV1::new(
            fence(&owner),
            CompactGroupCatalogKeyV1::Methyl,
            point(12.0, 0.0),
            CompactGroupPlacementModeV1::Free,
        );
        let mut prepared =
            prepare_compact_group_placement_v1(&mut owner, &request).expect("owner receipt");
        let foreign_before = foreign.snapshot().expect("foreign before");

        assert!(matches!(
            commit_compact_group_placement_v1(&mut foreign, &mut prepared),
            Err(CompactGroupPlacementErrorV1::Refusal(
                CompactGroupPlacementRefusalV1::ForeignSession
            ))
        ));
        assert_eq!(foreign.snapshot().expect("foreign after"), foreign_before);
        commit_compact_group_placement_v1(&mut owner, &mut prepared)
            .expect("owner receipt remains redeemable");
    }

    #[test]
    fn committed_receipt_refuses_replay_without_second_history_transition() {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
        let request = CompactGroupPlacementRequestV1::new(
            fence(&session),
            CompactGroupCatalogKeyV1::Methyl,
            point(12.0, 0.0),
            CompactGroupPlacementModeV1::Free,
        );
        let mut prepared =
            prepare_compact_group_placement_v1(&mut session, &request).expect("receipt");
        commit_compact_group_placement_v1(&mut session, &mut prepared).expect("first commit");
        let after_first = session.snapshot().expect("first committed snapshot");

        assert!(matches!(
            commit_compact_group_placement_v1(&mut session, &mut prepared),
            Err(CompactGroupPlacementErrorV1::Replayed)
        ));
        assert_eq!(
            session.snapshot().expect("after replay refusal"),
            after_first
        );
    }

    #[test]
    fn attached_pointer_orientation_persists_through_reopen() {
        let mut session = session_with_molecule(1, Vec::new());
        let request = CompactGroupPlacementRequestV1::new(
            fence(&session),
            CompactGroupCatalogKeyV1::Methyl,
            point(36.0, 24.0),
            {
                let observation = session
                    .observe(session.snapshot().expect("snapshot").revision())
                    .expect("observation");
                let molecule = &observation.projection().molecules()[0];
                CompactGroupPlacementModeV1::Attached {
                    molecule_id: molecule.id().expect("durable molecule").clone(),
                    anchor_atom_id: molecule.atoms()[0].id().expect("durable atom").clone(),
                }
            },
        );
        let mut prepared =
            prepare_compact_group_placement_v1(&mut session, &request).expect("attached receipt");
        let committed = commit_compact_group_placement_v1(&mut session, &mut prepared)
            .expect("attached commit");
        let expected = 24.0_f64.atan2(36.0).to_degrees();
        let observed = committed.observation().projection().molecules()[0].compact_groups()[0]
            .orientation_degrees();
        assert!((observed - expected).abs() < f64::EPSILON);

        let reopened = DocumentSession::load(committed.observation().snapshot().cdml())
            .expect("persisted compact group reopens");
        let reopened_observation = reopened
            .observe(reopened.snapshot().expect("reopened snapshot").revision())
            .expect("reopened observation");
        let reopened_orientation = reopened_observation.projection().molecules()[0]
            .compact_groups()[0]
            .orientation_degrees();
        assert!((reopened_orientation - expected).abs() < f64::EPSILON);
    }
}
