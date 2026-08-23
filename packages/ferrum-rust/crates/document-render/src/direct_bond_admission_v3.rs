//! Stateful renderer admission for resolved direct-bond pointer gestures.

use ferrum_document::{
    DirectBondGestureErrorV1, DirectBondSnapPolicyV1, DocumentBondPresentationV1, DocumentFenceV1,
    DocumentSession,
};

use crate::direct_bond_pointer_v3::{
    CommittedDirectBondGestureV3, DirectBondAdmissionErrorV3, DirectBondAdmissionRefusalV3,
    DirectBondAdmissionV3, DirectBondGestureV3, DirectBondOverlayV3, DirectBondPointerProbeErrorV3,
    DirectBondPointerProbeV3,
};
use crate::direct_bond_probe_resolution_v3::resolve_probe;
use crate::direct_bond_v3_lifecycle::{
    DirectBondCommitError, admit_direct_bond_candidate, begin_direct_bond_v3_lifecycle,
    commit_direct_bond_admission,
};

pub fn begin_direct_bond_gesture_v3(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    start: DirectBondPointerProbeV3,
    presentation: DocumentBondPresentationV1,
    new_atom_element: String,
    snap: DirectBondSnapPolicyV1,
) -> Result<DirectBondGestureV3, DirectBondAdmissionErrorV3> {
    let intent = resolve_probe(session, fence, &start)?;
    begin_direct_bond_v3_lifecycle(session, fence, intent, presentation, new_atom_element, snap)
        .map(|gesture| DirectBondGestureV3 { gesture, fence })
        .map_err(begin_direct_bond_error_v3)
}

fn begin_direct_bond_error_v3(error: DirectBondGestureErrorV1) -> DirectBondAdmissionErrorV3 {
    match error {
        DirectBondGestureErrorV1::StaleRevision => {
            DirectBondPointerProbeErrorV3::StaleRevision.into()
        }
        DirectBondGestureErrorV1::StaleDigest => DirectBondPointerProbeErrorV3::StaleDigest.into(),
        DirectBondGestureErrorV1::ForeignSession => {
            DirectBondAdmissionRefusalV3::ForeignSession.into()
        }
        DirectBondGestureErrorV1::ReplayedGesture => {
            DirectBondAdmissionRefusalV3::ReplayedGesture.into()
        }
        DirectBondGestureErrorV1::UnknownStartAtom => {
            DirectBondAdmissionRefusalV3::UnknownStartAtom.into()
        }
        DirectBondGestureErrorV1::UnknownEndAtom => {
            DirectBondAdmissionRefusalV3::UnknownEndAtom.into()
        }
        DirectBondGestureErrorV1::UnsupportedPresentation => {
            DirectBondAdmissionRefusalV3::UnsupportedPresentation.into()
        }
        DirectBondGestureErrorV1::SelfLoop => DirectBondAdmissionRefusalV3::SelfLoop.into(),
        DirectBondGestureErrorV1::CrossMolecule => {
            DirectBondAdmissionRefusalV3::CrossMolecule.into()
        }
        DirectBondGestureErrorV1::DuplicateBond => {
            DirectBondAdmissionRefusalV3::DuplicateBond.into()
        }
        DirectBondGestureErrorV1::NonFinitePoint | DirectBondGestureErrorV1::InvalidSnapPolicy => {
            DirectBondAdmissionRefusalV3::InvalidEndpointInput.into()
        }
        DirectBondGestureErrorV1::CollapsedEndpoint => {
            DirectBondAdmissionRefusalV3::CollapsedEndpoint.into()
        }
        DirectBondGestureErrorV1::UnrenderableCandidate => {
            DirectBondAdmissionRefusalV3::UnrenderableCandidate.into()
        }
        DirectBondGestureErrorV1::ExceedsChemistryCapacity => {
            DirectBondAdmissionRefusalV3::ExceedsChemistryCapacity.into()
        }
        DirectBondGestureErrorV1::UnsupportedChemistryAdmission => {
            DirectBondAdmissionRefusalV3::UnsupportedChemistryAdmission.into()
        }
        DirectBondGestureErrorV1::SessionConflict => {
            DirectBondAdmissionErrorV3::DocumentGesture(DirectBondGestureErrorV1::SessionConflict)
        }
    }
}

pub fn admit_direct_bond_candidate_v3(
    session: &DocumentSession,
    gesture: &DirectBondGestureV3,
    end: DirectBondPointerProbeV3,
) -> Result<DirectBondAdmissionV3, DirectBondAdmissionErrorV3> {
    super::direct_bond_v3_lifecycle::require_available_direct_bond_gesture(
        session,
        &gesture.gesture,
    )
    .map_err(DirectBondAdmissionRefusalV3::from)?;
    let intent = resolve_probe(session, gesture.fence, &end)?;
    Ok(
        admit_direct_bond_candidate(session, &gesture.gesture, intent)
            .map(|admission| DirectBondAdmissionV3 {
                overlay: DirectBondOverlayV3 {
                    overlay: admission.overlay().clone(),
                },
                admission,
            })
            .map_err(DirectBondAdmissionRefusalV3::from)?,
    )
}

pub fn commit_direct_bond_admission_v3(
    session: &mut DocumentSession,
    admission: &mut DirectBondAdmissionV3,
) -> Result<CommittedDirectBondGestureV3, DirectBondCommitError> {
    commit_direct_bond_admission(session, &mut admission.admission)
        .map(|committed| CommittedDirectBondGestureV3 { committed })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::direct_bond_pointer_v3::{DirectBondPointerHitStateV3, DirectBondViewportToSceneV3};
    use ferrum_document::DocumentBondOrderV1;

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"m\"><atom id=\"atom-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"atom-c\" name=\"C\"><point x=\"40\" y=\"0\"/></atom></molecule></cdml>";

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    fn frame() -> DirectBondViewportToSceneV3 {
        DirectBondViewportToSceneV3::new(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).expect("identity frame")
    }

    fn no_hit(x: f64, y: f64) -> DirectBondPointerProbeV3 {
        DirectBondPointerProbeV3::new(x, y, frame(), DirectBondPointerHitStateV3::None, None)
            .expect("finite empty probe")
    }

    fn direct_atom(_session: &DocumentSession, source_id: &str) -> DirectBondPointerProbeV3 {
        DirectBondPointerProbeV3::new(
            0.0,
            0.0,
            frame(),
            DirectBondPointerHitStateV3::UniqueAtom,
            Some(source_id.to_owned()),
        )
        .expect("direct atom probe")
    }

    #[test]
    fn pointer_probe_v3_preserves_every_endpoint_form() {
        for (name, start_existing, end_existing) in [
            ("existing_existing", true, true),
            ("existing_new", true, false),
            ("new_existing", false, true),
            ("new_new", false, false),
        ] {
            let mut session = DocumentSession::load(SOURCE).expect("session");
            let start = if start_existing {
                direct_atom(&session, "atom-a")
            } else {
                no_hit(-40.0, 0.0)
            };
            let end = if end_existing {
                direct_atom(&session, "atom-c")
            } else {
                no_hit(80.0, 0.0)
            };
            let gesture = begin_direct_bond_gesture_v3(
                &session,
                fence(&session),
                start,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .unwrap_or_else(|error| panic!("{name} begins: {error}"));
            let mut admission = admit_direct_bond_candidate_v3(&session, &gesture, end)
                .unwrap_or_else(|error| panic!("{name} admits: {error}"));
            assert!(
                !admission.overlay().operations().is_empty(),
                "{name} retains operations"
            );
            let committed = commit_direct_bond_admission_v3(&mut session, &mut admission)
                .unwrap_or_else(|error| panic!("{name} commits: {error}"));
            assert!(
                committed.result().observation().snapshot().revision() > 0,
                "{name} commits once"
            );
        }
    }

    #[test]
    fn pointer_probe_v3_preserves_same_atom_admission_refusals() {
        for presentation in [
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            DocumentBondPresentationV1::SolidWedge,
            DocumentBondPresentationV1::HashedWedge,
        ] {
            let session = DocumentSession::load(SOURCE).expect("session");
            let before = session.snapshot().expect("before snapshot");
            let gesture = begin_direct_bond_gesture_v3(
                &session,
                fence(&session),
                direct_atom(&session, "atom-a"),
                presentation,
                "C".to_owned(),
                DirectBondSnapPolicyV1::free(),
            )
            .expect("gesture begins");
            let refusal =
                admit_direct_bond_candidate_v3(&session, &gesture, direct_atom(&session, "atom-a"))
                    .expect_err("same atom must be refused");
            assert!(matches!(
                refusal,
                DirectBondAdmissionErrorV3::Refusal(DirectBondAdmissionRefusalV3::SelfLoop)
            ));
            let after = session.snapshot().expect("after snapshot");
            assert_eq!(after.revision(), before.revision());
            assert_eq!(after.digest(), before.digest());
        }
    }

    #[test]
    fn pointer_probe_v3_replay_precedes_stale_endpoint_preflight() {
        let mut session = DocumentSession::load(SOURCE).expect("session");
        let gesture = begin_direct_bond_gesture_v3(
            &session,
            fence(&session),
            direct_atom(&session, "atom-a"),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("gesture begins");
        let mut admission =
            admit_direct_bond_candidate_v3(&session, &gesture, direct_atom(&session, "atom-c"))
                .expect("candidate preflights");
        commit_direct_bond_admission_v3(&mut session, &mut admission).expect("admission commits");
        let committed = session.snapshot().expect("committed snapshot");

        let replay =
            admit_direct_bond_candidate_v3(&session, &gesture, direct_atom(&session, "atom-c"))
                .expect_err("consumed gesture refuses before stale endpoint preflight");
        assert!(matches!(
            replay,
            DirectBondAdmissionErrorV3::Refusal(DirectBondAdmissionRefusalV3::ReplayedGesture)
        ));
        let after_replay = session.snapshot().expect("snapshot after replay refusal");
        assert_eq!(after_replay.revision(), committed.revision());
        assert_eq!(after_replay.digest(), committed.digest());
    }

    #[test]
    fn pointer_probe_v3_uses_native_nearest_tie_transform_and_grid_policy() {
        let session = DocumentSession::load(SOURCE).expect("session");
        let near = no_hit(43.0, 0.0);
        let gesture = begin_direct_bond_gesture_v3(
            &session,
            fence(&session),
            near,
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("near point resolves to atom");
        let admission = admit_direct_bond_candidate_v3(&session, &gesture, no_hit(80.0, 0.0))
            .expect("candidate");
        assert_eq!(admission.overlay().start_x(), 40.0);

        let tied = DirectBondPointerProbeV3::new(
            20.0,
            0.0,
            frame(),
            DirectBondPointerHitStateV3::AmbiguousAtom,
            None,
        )
        .expect("ambiguous evidence");
        assert!(matches!(
            begin_direct_bond_gesture_v3(
                &session,
                fence(&session),
                tied,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free()
            ),
            Err(DirectBondAdmissionErrorV3::PointerProbe(
                DirectBondPointerProbeErrorV3::AmbiguousAtom
            ))
        ));

        let zoom =
            DirectBondViewportToSceneV3::new(0.5, 0.0, 0.0, 0.5, 0.0, 0.0).expect("two x zoom");
        let zoom_probe =
            DirectBondPointerProbeV3::new(80.0, 0.0, zoom, DirectBondPointerHitStateV3::None, None)
                .expect("zoom probe");
        let identity_probe = no_hit(80.0, 0.0);
        let zoom_gesture = begin_direct_bond_gesture_v3(
            &session,
            fence(&session),
            direct_atom(&session, "atom-a"),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("zoom begin");
        let identity_gesture = begin_direct_bond_gesture_v3(
            &session,
            fence(&session),
            direct_atom(&session, "atom-a"),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::free(),
        )
        .expect("identity begin");
        let zoom_admission = admit_direct_bond_candidate_v3(&session, &zoom_gesture, zoom_probe)
            .expect("zoom admission");
        let identity_admission =
            admit_direct_bond_candidate_v3(&session, &identity_gesture, identity_probe)
                .expect("identity admission");
        assert_eq!(
            (
                zoom_admission.overlay().end_x(),
                zoom_admission.overlay().end_y()
            ),
            (
                identity_admission.overlay().end_x(),
                identity_admission.overlay().end_y()
            )
        );

        let mut grid_session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"/>")
                .expect("empty session");
        let grid_gesture = begin_direct_bond_gesture_v3(
            &grid_session,
            fence(&grid_session),
            no_hit(0.0, 0.0),
            DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
            "C".to_owned(),
            DirectBondSnapPolicyV1::new(true, None, None).expect("grid policy"),
        )
        .expect("grid begin");
        let mut grid_admission =
            admit_direct_bond_candidate_v3(&grid_session, &grid_gesture, no_hit(14.0, 6.0))
                .expect("grid admission");
        assert_eq!(
            (
                grid_admission.overlay().end_x(),
                grid_admission.overlay().end_y()
            ),
            (10.0, 10.0)
        );
        commit_direct_bond_admission_v3(&mut grid_session, &mut grid_admission)
            .expect("grid commit");
    }

    #[test]
    fn pointer_probe_v3_refuses_malformed_unknown_and_stale_inputs() {
        assert_eq!(
            DirectBondViewportToSceneV3::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            Err(DirectBondPointerProbeErrorV3::MalformedTransform)
        );
        let session = DocumentSession::load(SOURCE).expect("session");
        let unknown = DirectBondPointerProbeV3::new(
            0.0,
            0.0,
            frame(),
            DirectBondPointerHitStateV3::UniqueAtom,
            Some("unknown-source-id".to_owned()),
        )
        .expect("probe");
        assert!(matches!(
            begin_direct_bond_gesture_v3(
                &session,
                fence(&session),
                unknown,
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free()
            ),
            Err(DirectBondAdmissionErrorV3::PointerProbe(
                DirectBondPointerProbeErrorV3::UnknownDirectAtom
            ))
        ));
        let stale = DocumentFenceV1::new(1, fence(&session).digest());
        assert!(matches!(
            begin_direct_bond_gesture_v3(
                &session,
                stale,
                no_hit(80.0, 0.0),
                DocumentBondPresentationV1::Normal(DocumentBondOrderV1::Single),
                "C".to_owned(),
                DirectBondSnapPolicyV1::free()
            ),
            Err(DirectBondAdmissionErrorV3::PointerProbe(
                DirectBondPointerProbeErrorV3::StaleRevision
            ))
        ));
    }
}
