use super::*;
use crate::{
    AttachedCompactGroupReleaseV1, DocumentCompactGroupMaterializationRequestV1, SessionOperation,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};

pub(super) const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

fn fence(session: &DocumentSession) -> DocumentFenceV1 {
    DocumentFenceV1::new(session.current_revision_v1(), session.current_digest_v1())
}

fn anchor(session: &DocumentSession) -> DocumentObjectIdV1 {
    session
        .document_observation()
        .expect("observation")
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule")
        .atoms()
        .iter()
        .find(|atom| atom.source_id() == Some("a"))
        .expect("seeded anchor")
        .document_object_id()
        .clone()
}

fn molecule(session: &DocumentSession) -> DocumentObjectIdV1 {
    session
        .document_observation()
        .expect("observation")
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule")
        .document_object_id()
        .clone()
}

fn target_for_anchor(
    session: &DocumentSession,
    anchor_atom_id: DocumentObjectIdV1,
) -> AttachedCompactGroupTargetV1 {
    AttachedCompactGroupTargetV1::new(molecule(session), anchor_atom_id)
}

fn target(session: &DocumentSession) -> AttachedCompactGroupTargetV1 {
    target_for_anchor(session, anchor(session))
}

fn attached_request(catalog_key: CompactGroupCatalogKeyV1) -> AttachCompactGroupV1 {
    AttachCompactGroupV1::new(
        catalog_key,
        AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
    )
}

pub(super) fn commit_attachment_for(
    session: &mut DocumentSession,
    catalog_key: CompactGroupCatalogKeyV1,
) -> AttachedCompactGroupCommitResultV1 {
    let mut pending = session
        .prepare_attach_compact_group_v1(
            fence(session),
            target(session),
            attached_request(catalog_key),
        )
        .expect("prepare");
    session
        .commit_attach_compact_group_v1(&mut pending)
        .expect("commit")
}

fn commit_attachment(session: &mut DocumentSession) -> AttachedCompactGroupCommitResultV1 {
    commit_attachment_for(session, CompactGroupCatalogKeyV1::Methyl)
}

fn assert_materialized_attached_ethyl(session: &DocumentSession) {
    let observation = session.document_observation().expect("observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule");
    assert!(molecule.compact_groups().is_empty());
    let anchor = molecule
        .atoms()
        .iter()
        .find(|atom| atom.source_id() == Some("a"))
        .expect("seeded carbon anchor");
    let first_carbon = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.order() == Some(ferrum_core::BondOrder::Single)
                && bond.style() == Some(&ferrum_core::BondStyle::Normal)
                && [bond.start(), bond.end()]
                    .into_iter()
                    .any(|endpoint| endpoint.source_id() == anchor.source_id())
        })
        .and_then(|bond| {
            [bond.start(), bond.end()].into_iter().find_map(|endpoint| {
                (endpoint.source_id() != anchor.source_id()).then_some(endpoint.source_id())
            })
        })
        .and_then(|source_id| {
            molecule
                .atoms()
                .iter()
                .find(|atom| atom.source_id() == source_id)
        })
        .expect("ethyl has a normal single bond from the anchor");
    assert_eq!(
        (first_carbon.element(), first_carbon.formal_charge()),
        (Some("C"), None)
    );
    assert!(molecule.bonds().iter().any(|bond| {
        bond.order() == Some(ferrum_core::BondOrder::Single)
            && bond.style() == Some(&ferrum_core::BondStyle::Normal)
            && [bond.start(), bond.end()].into_iter().any(|endpoint| {
                endpoint.source_id() == first_carbon.source_id()
                    && endpoint.source_id() != anchor.source_id()
            })
    }));
}

fn assert_materialized_attached_methoxy(session: &DocumentSession) {
    let observation = session.document_observation().expect("observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule");
    assert!(molecule.compact_groups().is_empty());
    let anchor = molecule
        .atoms()
        .iter()
        .find(|atom| atom.source_id() == Some("a"))
        .expect("seeded carbon anchor");
    let oxygen = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.order() == Some(ferrum_core::BondOrder::Single)
                && bond.style() == Some(&ferrum_core::BondStyle::Normal)
                && [bond.start(), bond.end()]
                    .into_iter()
                    .any(|endpoint| endpoint.source_id() == anchor.source_id())
        })
        .and_then(|bond| {
            [bond.start(), bond.end()].into_iter().find_map(|endpoint| {
                (endpoint.source_id() != anchor.source_id()).then_some(endpoint.source_id())
            })
        })
        .and_then(|source_id| {
            molecule
                .atoms()
                .iter()
                .find(|atom| atom.source_id() == source_id)
        })
        .expect("methoxy has a normal single bond from the anchor");
    assert_eq!(
        (oxygen.element(), oxygen.formal_charge()),
        (Some("O"), None)
    );
    assert!(molecule.bonds().iter().any(|bond| {
        bond.order() == Some(ferrum_core::BondOrder::Single)
            && bond.style() == Some(&ferrum_core::BondStyle::Normal)
            && [bond.start(), bond.end()].into_iter().any(|endpoint| {
                endpoint.source_id() == oxygen.source_id()
                    && endpoint.source_id() != anchor.source_id()
            })
    }));
}

#[test]
fn prepare_commit_cancel_and_replay_preserve_the_closed_transaction_contract() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let before = session.snapshot().expect("before");
    let mut pending = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            target(&session),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        )
        .expect("prepare");
    assert_eq!(session.snapshot().expect("prepare is pure"), before);
    session
        .cancel_attach_compact_group_v1(&mut pending)
        .expect("cancel");
    assert_eq!(session.snapshot().expect("cancel is pure"), before);
    assert_eq!(
        session.commit_attach_compact_group_v1(&mut pending),
        Err(AttachedCompactGroupSessionErrorV1::Consumed)
    );
    let result = commit_attachment(&mut session);
    let after = result.observation().snapshot();
    assert_eq!(result.focus_object_id(), &anchor(&session));
    assert!(!result.compact_group_object_id().as_str().is_empty());
    assert_ne!(result.compact_group_object_id(), result.focus_object_id());
    assert_eq!(after.revision(), before.revision() + 1);
    let authored_molecule = result
        .observation()
        .projection()
        .molecules()
        .iter()
        .find(|molecule| {
            molecule
                .compact_groups()
                .iter()
                .any(|group| group.id() == result.compact_group_object_id())
        })
        .expect("returned compact-group identity remains selectable");
    let materialization = DocumentCompactGroupMaterializationRequestV1::new(
        after.revision(),
        *after.digest(),
        authored_molecule.document_object_id().clone(),
        result.compact_group_object_id().clone(),
    );
    let mut materialized = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            after.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(
                materialization,
            )),
            TransitionAuthorizationV1::None,
        ))
        .expect("existing materialization prepares");
    session
        .commit_session_operation_transition_v1(&mut materialized)
        .expect("existing materialization commits");
    assert_ne!(session.snapshot().expect("materialized snapshot"), *after);
}

#[test]
fn foreign_and_stale_pending_attachments_preserve_the_next_accepted_observation() {
    let mut owner = DocumentSession::load(SOURCE).expect("owner source");
    let mut other = DocumentSession::load(SOURCE).expect("other source");
    let mut foreign = owner
        .prepare_attach_compact_group_v1(
            fence(&owner),
            target(&owner),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        )
        .expect("owner prepare");
    assert_eq!(
        other.commit_attach_compact_group_v1(&mut foreign),
        Err(AttachedCompactGroupSessionErrorV1::ForeignSession)
    );
    let committed = owner
        .commit_attach_compact_group_v1(&mut foreign)
        .expect("owner commit");
    let mut fresh_owner = DocumentSession::load(SOURCE).expect("fresh owner source");
    assert_ne!(
        committed.compact_group_object_id(),
        commit_attachment(&mut fresh_owner).compact_group_object_id(),
    );

    let mut session = DocumentSession::load(SOURCE).expect("source");
    let mut first = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            target(&session),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        )
        .expect("first prepare");
    let mut stale = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            target(&session),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        )
        .expect("stale prepare");
    let first_committed = session
        .commit_attach_compact_group_v1(&mut first)
        .expect("first commit");
    assert_eq!(
        session.commit_attach_compact_group_v1(&mut stale),
        Err(AttachedCompactGroupSessionErrorV1::StaleRevision)
    );
    let mut fresh_session = DocumentSession::load(SOURCE).expect("fresh source");
    assert_ne!(
        first_committed.compact_group_object_id(),
        commit_attachment(&mut fresh_session).compact_group_object_id(),
    );
}

#[test]
fn refusal_categories_leave_the_next_accepted_observation_unchanged() {
    let mut selector_session = DocumentSession::load(SOURCE).expect("source");
    let before = selector_session.snapshot().expect("before");
    let missing = DocumentObjectIdV1::from_entropy_bytes([0; 16]);
    assert!(matches!(
        selector_session.prepare_attach_compact_group_v1(
            fence(&selector_session),
            target_for_anchor(&selector_session, missing),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        ),
        Err(AttachedCompactGroupSessionErrorV1::UnknownAnchor)
    ));
    assert_eq!(selector_session.snapshot().expect("unchanged"), before);
    let mut fresh_selector = DocumentSession::load(SOURCE).expect("fresh source");
    assert_ne!(
        commit_attachment(&mut selector_session).compact_group_object_id(),
        commit_attachment(&mut fresh_selector).compact_group_object_id(),
    );

    let mut pose_session = DocumentSession::load(SOURCE).expect("source");
    let before = pose_session.snapshot().expect("before");
    assert!(matches!(
        pose_session.prepare_attach_compact_group_v1(
            fence(&pose_session),
            target(&pose_session),
            AttachCompactGroupV1::new(
                CompactGroupCatalogKeyV1::Methyl,
                AttachedCompactGroupReleaseV1::new(0.0, 0.0).expect("release"),
            ),
        ),
        Err(AttachedCompactGroupSessionErrorV1::RendererAdmission)
    ));
    assert_eq!(pose_session.snapshot().expect("unchanged"), before);
    let mut fresh_pose = DocumentSession::load(SOURCE).expect("fresh source");
    assert_ne!(
        commit_attachment(&mut pose_session).compact_group_object_id(),
        commit_attachment(&mut fresh_pose).compact_group_object_id(),
    );

    let capacity_source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"h1\" name=\"H\"><point x=\"1\" y=\"0\"/></atom>",
        "<atom id=\"h2\" name=\"H\"><point x=\"-1\" y=\"0\"/></atom>",
        "<atom id=\"h3\" name=\"H\"><point x=\"0\" y=\"1\"/></atom>",
        "<atom id=\"h4\" name=\"H\"><point x=\"0\" y=\"-1\"/></atom>",
        "<bond id=\"b1\" start=\"a\" end=\"h1\" type=\"n1\"/>",
        "<bond id=\"b2\" start=\"a\" end=\"h2\" type=\"n1\"/>",
        "<bond id=\"b3\" start=\"a\" end=\"h3\" type=\"n1\"/>",
        "<bond id=\"b4\" start=\"a\" end=\"h4\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    let mut capacity_session = DocumentSession::load(capacity_source).expect("capacity source");
    let before = capacity_session.snapshot().expect("before");
    assert!(matches!(
        capacity_session.prepare_attach_compact_group_v1(
            fence(&capacity_session),
            target(&capacity_session),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        ),
        Err(AttachedCompactGroupSessionErrorV1::CandidateAdmission)
    ));
    assert_eq!(capacity_session.snapshot().expect("unchanged"), before);

    assert!(
        CompactGroupCatalogKeyV1::parse("invalid_attached_compact_group").is_none(),
        "an unknown persisted compact-group key is refused before session preparation"
    );
}

#[test]
fn pair_local_target_refusals_leave_document_state_unchanged() {
    const TWO_MOLECULES: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\">",
        "<molecule id=\"m1\"><atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"m2\"><atom id=\"a2\" name=\"C\"><point x=\"20\" y=\"0\"/></atom></molecule>",
        "</cdml>",
    );
    let mut session = DocumentSession::load(TWO_MOLECULES).expect("source");
    let before = session.snapshot().expect("before");
    let observation = session.document_observation().expect("observation");
    let first = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m1"))
        .expect("first molecule");
    let second = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m2"))
        .expect("second molecule");
    let first_anchor_id = first
        .atoms()
        .iter()
        .find(|atom| atom.source_id() == Some("a1"))
        .expect("first anchor")
        .document_object_id()
        .clone();
    let second_molecule_id = second.document_object_id().clone();
    let unknown_molecule_id = DocumentObjectIdV1::from_entropy_bytes([1; 16]);

    assert!(matches!(
        session.prepare_attach_compact_group_v1(
            fence(&session),
            AttachedCompactGroupTargetV1::new(unknown_molecule_id, first_anchor_id.clone()),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        ),
        Err(AttachedCompactGroupSessionErrorV1::UnknownMolecule)
    ));
    assert_eq!(session.snapshot().expect("unknown root is pure"), before);
    assert!(matches!(
        session.prepare_attach_compact_group_v1(
            fence(&session),
            AttachedCompactGroupTargetV1::new(second_molecule_id, first_anchor_id),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        ),
        Err(AttachedCompactGroupSessionErrorV1::ForeignTarget)
    ));
    assert_eq!(session.snapshot().expect("foreign pair is pure"), before);
}

#[test]
fn availability_is_advisory_for_eligible_and_unavailable_anchors() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let before = session.snapshot().expect("before");
    let available = session.observe_attach_compact_group_availability_v1(
        fence(&session),
        target(&session),
        CompactGroupCatalogKeyV1::Methyl,
    );
    assert!(available.is_available());
    assert_eq!(
        available.category(),
        AttachedCompactGroupAvailabilityCategoryV1::Available
    );
    assert_eq!(session.snapshot().expect("availability is pure"), before);

    let methoxy = session.observe_attach_compact_group_availability_v1(
        fence(&session),
        target(&session),
        CompactGroupCatalogKeyV1::Methoxy,
    );
    assert!(methoxy.is_available());
    let mut methoxy_pending = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            target(&session),
            attached_request(CompactGroupCatalogKeyV1::Methoxy),
        )
        .expect("available Methoxy prepares through the renderer-owned pose");
    session
        .cancel_attach_compact_group_v1(&mut methoxy_pending)
        .expect("retiring the availability agreement probe is pure");
    assert_eq!(
        session.snapshot().expect("availability agreement is pure"),
        before
    );

    let missing = DocumentObjectIdV1::from_entropy_bytes([0; 16]);
    let unavailable = session.observe_attach_compact_group_availability_v1(
        fence(&session),
        target_for_anchor(&session, missing),
        CompactGroupCatalogKeyV1::Methyl,
    );
    assert!(!unavailable.is_available());
    assert_eq!(
        unavailable.category(),
        AttachedCompactGroupAvailabilityCategoryV1::UnknownAnchor
    );
    assert_eq!(session.snapshot().expect("availability is pure"), before);
}

#[test]
fn attached_ethyl_materialization_survives_history_and_reopen() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let attached = commit_attachment_for(&mut session, CompactGroupCatalogKeyV1::Ethyl);
    let attached_snapshot = attached.observation().snapshot();
    let molecule = attached
        .observation()
        .projection()
        .molecules()
        .iter()
        .find(|molecule| {
            molecule
                .compact_groups()
                .iter()
                .any(|group| group.id() == attached.compact_group_object_id())
        })
        .expect("attached Ethyl molecule");
    let request = DocumentCompactGroupMaterializationRequestV1::new(
        attached_snapshot.revision(),
        *attached_snapshot.digest(),
        molecule.document_object_id().clone(),
        attached.compact_group_object_id().clone(),
    );
    let mut pending = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            attached_snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .expect("Ethyl materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("Ethyl materialization commits");
    assert_materialized_attached_ethyl(&session);

    let undone = session
        .undo(materialized.observation().snapshot().revision())
        .expect("Ethyl materialization is undoable");
    let undone_molecule = undone
        .observation()
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule after undo");
    assert!(
        undone_molecule
            .compact_groups()
            .iter()
            .any(|group| group.catalog_key() == CompactGroupCatalogKeyV1::Ethyl)
    );

    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("Ethyl materialization is redoable");
    assert_materialized_attached_ethyl(&session);
    let reopened = DocumentSession::load(redone.observation().snapshot().cdml())
        .expect("materialized Ethyl reopens");
    assert_materialized_attached_ethyl(&reopened);
}

#[test]
fn attached_methoxy_materialization_survives_history_and_reopen() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let mut pending = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            target(&session),
            attached_request(CompactGroupCatalogKeyV1::Methoxy),
        )
        .expect("Methoxy prepares through renderer-issued pose admission");
    let overlay = pending
        .precommit_overlay_v1()
        .expect("Methoxy has an exterior-bond precommit overlay");
    assert!(
        overlay
            .primitives()
            .iter()
            .any(|primitive| { matches!(primitive.operation(), ferrum_render::RenderOp::Line(_)) })
    );
    let attached = session
        .commit_attach_compact_group_v1(&mut pending)
        .expect("Methoxy commits");
    let attached_snapshot = attached.observation().snapshot();
    let molecule = attached
        .observation()
        .projection()
        .molecules()
        .iter()
        .find(|molecule| {
            molecule
                .compact_groups()
                .iter()
                .any(|group| group.id() == attached.compact_group_object_id())
        })
        .expect("attached Methoxy molecule");
    let resolved_anchor = molecule
        .compact_groups()
        .iter()
        .find(|group| group.id() == attached.compact_group_object_id())
        .expect("renderer-admitted Methoxy compact group")
        .anchor();
    let request = DocumentCompactGroupMaterializationRequestV1::new(
        attached_snapshot.revision(),
        *attached_snapshot.digest(),
        molecule.document_object_id().clone(),
        attached.compact_group_object_id().clone(),
    );
    let mut pending = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            attached_snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .expect("Methoxy materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("Methoxy materialization commits");
    assert_materialized_attached_methoxy(&session);

    let undone = session
        .undo(materialized.observation().snapshot().revision())
        .expect("Methoxy materialization is undoable");
    assert!(
        undone
            .observation()
            .projection()
            .molecules()
            .iter()
            .find(|molecule| molecule.source_id() == Some("m"))
            .expect("seeded molecule after undo")
            .compact_groups()
            .iter()
            .any(|group| {
                group.catalog_key() == CompactGroupCatalogKeyV1::Methoxy
                    && group.anchor() == resolved_anchor
            })
    );

    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("Methoxy materialization is redoable");
    assert_materialized_attached_methoxy(&session);
    let reopened = DocumentSession::load(redone.observation().snapshot().cdml())
        .expect("materialized Methoxy reopens");
    assert_materialized_attached_methoxy(&reopened);
}

#[test]
fn every_authorable_compact_group_previews_an_exterior_bond_before_commit() {
    for catalog_key in ferrum_document_model::attached_compact_group_authoring_keys_v1() {
        let mut session = DocumentSession::load(SOURCE).expect("source");
        let request = AttachCompactGroupV1::new(
            *catalog_key,
            AttachedCompactGroupReleaseV1::new(1.0, 0.0).expect("short same-ray release is finite"),
        );
        let mut pending = session
            .prepare_attach_compact_group_v1(fence(&session), target(&session), request)
            .expect("renderer admits every authorable compact-group family");
        let overlay = pending
            .precommit_overlay_v1()
            .expect("renderer-admitted compact group has an exterior-bond overlay");
        assert!(
            overlay
                .primitives()
                .iter()
                .any(|primitive| matches!(primitive.operation(), ferrum_render::RenderOp::Line(_))),
            "{} preview contains its normal exterior bond",
            catalog_key.as_str(),
        );
        let attached = session
            .commit_attach_compact_group_v1(&mut pending)
            .expect("renderer-admitted compact group commits");
        let committed = attached
            .observation()
            .projection()
            .molecules()
            .iter()
            .find(|molecule| molecule.source_id() == Some("m"))
            .expect("seeded molecule remains after commit");
        assert!(
            committed
                .compact_groups()
                .iter()
                .any(|group| group.catalog_key() == *catalog_key),
            "{} preview and commit resolve the same authorable family",
            catalog_key.as_str(),
        );
    }
}
