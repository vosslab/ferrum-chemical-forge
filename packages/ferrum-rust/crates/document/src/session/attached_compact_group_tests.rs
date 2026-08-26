use super::*;
use crate::{
    AttachedCompactGroupReleaseV1, DocumentCompactGroupMaterializationRequestV1, SessionOperation,
    SessionOperationOutcomeV1, SessionOperationTransitionRequestV1, SessionOperationV1,
    TransitionAuthorizationV1,
};

const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

fn fence(session: &DocumentSession) -> DocumentFenceV1 {
    DocumentFenceV1::new(session.current_revision_v1(), session.current_digest_v1())
}

fn anchor(session: &DocumentSession) -> DocumentObjectIdV1 {
    session
        .document_observation()
        .expect("observation")
        .projection()
        .molecules()[0]
        .atoms()[0]
        .id()
        .expect("direct atom selector")
        .clone()
}

fn attached_request(catalog_key: CompactGroupCatalogKeyV1) -> AttachCompactGroupV1 {
    AttachCompactGroupV1::new(
        catalog_key,
        AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
    )
}

fn commit_attachment_for(
    session: &mut DocumentSession,
    catalog_key: CompactGroupCatalogKeyV1,
) -> AttachedCompactGroupCommitResultV1 {
    let mut pending = session
        .prepare_attach_compact_group_v1(
            fence(session),
            anchor(session),
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
    let molecule = &observation.projection().molecules()[0];
    assert!(molecule.compact_groups().is_empty());
    assert_eq!(molecule.atoms().len(), 3);
    assert!(
        molecule
            .atoms()
            .iter()
            .all(|atom| atom.element() == Some("C") && atom.formal_charge().is_none())
    );
    assert_eq!(molecule.bonds().len(), 2);
    assert!(molecule.bonds().iter().all(|bond| {
        bond.order() == Some(ferrum_core::BondOrder::Single)
            && bond.style() == Some(&ferrum_core::BondStyle::Normal)
    }));
}

fn assert_materialized_attached_methoxy(session: &DocumentSession) {
    let observation = session.document_observation().expect("observation");
    let molecule = &observation.projection().molecules()[0];
    assert!(molecule.compact_groups().is_empty());
    assert_eq!(molecule.atoms().len(), 3);
    assert_eq!(
        molecule
            .atoms()
            .iter()
            .filter(|atom| atom.element() == Some("O") && atom.formal_charge().is_none())
            .count(),
        1
    );
    assert_eq!(
        molecule
            .atoms()
            .iter()
            .filter(|atom| atom.element() == Some("C") && atom.formal_charge().is_none())
            .count(),
        2
    );
    assert_eq!(molecule.bonds().len(), 2);
    assert!(molecule.bonds().iter().all(|bond| {
        bond.order() == Some(ferrum_core::BondOrder::Single)
            && bond.style() == Some(&ferrum_core::BondStyle::Normal)
    }));
}

fn assert_materialized_attached_hydroxymethyl(
    session: &DocumentSession,
    focus_atom_id: &DocumentObjectIdV1,
    exterior_bond_id: &DocumentObjectIdV1,
) {
    let observation = session.document_observation().expect("observation");
    let molecule = &observation.projection().molecules()[0];
    assert!(molecule.compact_groups().is_empty());
    let attachment_carbon = molecule
        .atoms()
        .iter()
        .find(|atom| atom.id() == Some(focus_atom_id))
        .expect("materialization focus is the attachment carbon");
    assert_eq!(
        (
            attachment_carbon.element(),
            attachment_carbon.formal_charge()
        ),
        (Some("C"), None)
    );
    let exterior = molecule
        .bonds()
        .iter()
        .find(|bond| bond.id() == Some(exterior_bond_id))
        .expect("exterior bond retains its durable identity");
    let internal = molecule
        .bonds()
        .iter()
        .find(|bond| bond.id() != Some(exterior_bond_id))
        .expect("hydroxymethyl internal bond");
    let hydroxyl_oxygen = molecule
        .atoms()
        .iter()
        .find(|atom| atom.source_id() == internal.end().source_id())
        .expect("hydroxymethyl oxygen");
    assert_eq!(
        (
            internal.order(),
            internal.style(),
            internal.start().source_id()
        ),
        (
            Some(ferrum_core::BondOrder::Single),
            Some(&ferrum_core::BondStyle::Normal),
            attachment_carbon.source_id(),
        )
    );
    assert_eq!(
        (hydroxyl_oxygen.element(), hydroxyl_oxygen.formal_charge()),
        (Some("O"), None)
    );
    assert!(
        [exterior.start(), exterior.end()]
            .into_iter()
            .any(|endpoint| endpoint.source_id() == attachment_carbon.source_id()),
        "retained exterior bond targets the returned attachment-carbon focus"
    );
    assert_eq!(
        (exterior.order(), exterior.style()),
        (
            Some(ferrum_core::BondOrder::Single),
            Some(&ferrum_core::BondStyle::Normal),
        )
    );
}

fn assert_materialized_attached_carboxyl(
    session: &DocumentSession,
    focus_atom_id: &DocumentObjectIdV1,
    exterior_bond_id: &DocumentObjectIdV1,
) {
    let observation = session.document_observation().expect("observation");
    let molecule = &observation.projection().molecules()[0];
    assert!(molecule.compact_groups().is_empty());
    let attachment_carbon = molecule
        .atoms()
        .iter()
        .find(|atom| atom.id() == Some(focus_atom_id))
        .expect("materialization focus is the carboxyl attachment carbon");
    assert_eq!(
        (
            attachment_carbon.element(),
            attachment_carbon.formal_charge()
        ),
        (Some("C"), None)
    );
    let exterior = molecule
        .bonds()
        .iter()
        .find(|bond| bond.id() == Some(exterior_bond_id))
        .expect("exterior bond retains its durable identity");
    let carbonyl = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.id() != Some(exterior_bond_id)
                && bond.order() == Some(ferrum_core::BondOrder::Double)
                && bond.start().source_id() == attachment_carbon.source_id()
        })
        .expect("carboxyl has a carbonyl bond from the attachment carbon");
    let hydroxyl = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.id() != Some(exterior_bond_id)
                && bond.order() == Some(ferrum_core::BondOrder::Single)
                && bond.start().source_id() == attachment_carbon.source_id()
        })
        .expect("carboxyl has a hydroxyl bond from the attachment carbon");
    for bond in [carbonyl, hydroxyl] {
        assert_eq!(bond.style(), Some(&ferrum_core::BondStyle::Normal));
        let oxygen = molecule
            .atoms()
            .iter()
            .find(|atom| atom.source_id() == bond.end().source_id())
            .expect("carboxyl internal bond ends at oxygen");
        assert_eq!(
            (oxygen.element(), oxygen.formal_charge()),
            (Some("O"), None)
        );
    }
    assert!(
        [exterior.start(), exterior.end()]
            .into_iter()
            .any(|endpoint| endpoint.source_id() == attachment_carbon.source_id()),
        "retained exterior bond targets the returned attachment-carbon focus"
    );
    assert_eq!(
        (exterior.order(), exterior.style()),
        (
            Some(ferrum_core::BondOrder::Single),
            Some(&ferrum_core::BondStyle::Normal),
        )
    );
}

fn assert_materialized_attached_cyano(
    session: &DocumentSession,
    focus_atom_id: &DocumentObjectIdV1,
    exterior_neighbor_source_id: &str,
    exterior_bond_id: &DocumentObjectIdV1,
) {
    let observation = session.document_observation().expect("observation");
    let molecule = &observation.projection().molecules()[0];
    assert!(molecule.compact_groups().is_empty());
    let attachment_carbon = molecule
        .atoms()
        .iter()
        .find(|atom| atom.id() == Some(focus_atom_id))
        .expect("materialization focus is the cyano attachment carbon");
    assert_eq!(
        (
            attachment_carbon.element(),
            attachment_carbon.formal_charge()
        ),
        (Some("C"), None)
    );
    let carbon_nitrogen = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.id() != Some(exterior_bond_id)
                && bond.order() == Some(ferrum_core::BondOrder::Triple)
                && bond.style() == Some(&ferrum_core::BondStyle::Normal)
                && bond.start().source_id() == attachment_carbon.source_id()
        })
        .expect("cyano has a normal triple bond from the attachment carbon");
    let terminal_nitrogen = molecule
        .atoms()
        .iter()
        .find(|atom| atom.source_id() == carbon_nitrogen.end().source_id())
        .expect("cyano triple bond ends at nitrogen");
    assert_eq!(
        (
            terminal_nitrogen.element(),
            terminal_nitrogen.formal_charge()
        ),
        (Some("N"), None)
    );
    let exterior = molecule
        .bonds()
        .iter()
        .find(|bond| bond.id() == Some(exterior_bond_id))
        .expect("exterior bond retains its durable identity");
    assert!(
        (exterior.start().source_id() == Some(exterior_neighbor_source_id)
            && exterior.end().source_id() == attachment_carbon.source_id())
            || (exterior.end().source_id() == Some(exterior_neighbor_source_id)
                && exterior.start().source_id() == attachment_carbon.source_id()),
        "retained exterior bond rewires the original exterior neighbor to the returned attachment-carbon focus"
    );
    assert_eq!(
        (exterior.order(), exterior.style()),
        (
            Some(ferrum_core::BondOrder::Single),
            Some(&ferrum_core::BondStyle::Normal),
        )
    );
    assert!(
        observation.snapshot().cdml().contains("type=\"n3\""),
        "ordinary normal triple materializes through the typed CDML n3 writer"
    );
}

#[test]
fn prepare_commit_cancel_and_replay_preserve_the_closed_transaction_contract() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let before = session.snapshot().expect("before");
    let mut pending = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            anchor(&session),
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
        authored_molecule
            .id()
            .cloned()
            .expect("authored molecule remains durable-addressable"),
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
            anchor(&owner),
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
            anchor(&session),
            attached_request(CompactGroupCatalogKeyV1::Methyl),
        )
        .expect("first prepare");
    let mut stale = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            anchor(&session),
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
            missing,
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
            anchor(&pose_session),
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
            anchor(&capacity_session),
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
fn availability_is_advisory_for_eligible_and_unavailable_anchors() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let before = session.snapshot().expect("before");
    let available = session.observe_attach_compact_group_availability_v1(
        fence(&session),
        anchor(&session),
        CompactGroupCatalogKeyV1::Methyl,
    );
    assert!(available.is_available());
    assert_eq!(
        available.category(),
        AttachedCompactGroupAvailabilityCategoryV1::Available
    );
    assert_eq!(available.catalog_key(), CompactGroupCatalogKeyV1::Methyl);
    assert_eq!(available.revision(), before.revision());
    assert_eq!(available.digest(), before.digest());
    assert_eq!(session.snapshot().expect("availability is pure"), before);

    let nitro = session.observe_attach_compact_group_availability_v1(
        fence(&session),
        anchor(&session),
        CompactGroupCatalogKeyV1::Nitro,
    );
    assert!(nitro.is_available());
    assert_eq!(nitro.catalog_key(), CompactGroupCatalogKeyV1::Nitro);

    let methoxy = session.observe_attach_compact_group_availability_v1(
        fence(&session),
        anchor(&session),
        CompactGroupCatalogKeyV1::Methoxy,
    );
    assert!(methoxy.is_available());
    assert_eq!(methoxy.catalog_key(), CompactGroupCatalogKeyV1::Methoxy);
    let mut methoxy_pending = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            anchor(&session),
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
        missing,
        CompactGroupCatalogKeyV1::Methyl,
    );
    assert!(!unavailable.is_available());
    assert_eq!(
        unavailable.category(),
        AttachedCompactGroupAvailabilityCategoryV1::UnknownAnchor
    );
    let unsupported = session.observe_attach_compact_group_availability_v1(
        fence(&session),
        anchor(&session),
        CompactGroupCatalogKeyV1::Phenyl,
    );
    assert_eq!(
        unsupported.category(),
        AttachedCompactGroupAvailabilityCategoryV1::CandidateAdmission,
    );
    assert_eq!(
        session
            .snapshot()
            .expect("unavailable availability is pure"),
        before
    );
}

#[test]
fn supported_choices_and_committed_recipes_remain_projectable_and_persistent() {
    let mut choices = crate::attached_compact_group_choices_v1();
    assert!(choices.clone().any(|choice| {
        choice.catalog_key() == CompactGroupCatalogKeyV1::Methyl && choice.label() == "Me"
    }));
    assert!(choices.clone().any(|choice| {
        choice.catalog_key() == CompactGroupCatalogKeyV1::Nitro && choice.label() == "NO2"
    }));
    assert!(choices.clone().any(|choice| {
        choice.catalog_key() == CompactGroupCatalogKeyV1::Ethyl && choice.label() == "Et"
    }));
    assert!(choices.any(|choice| {
        choice.catalog_key() == CompactGroupCatalogKeyV1::Methoxy && choice.label() == "OMe"
    }));
    assert!(crate::attached_compact_group_choices_v1().any(|choice| {
        choice.catalog_key() == CompactGroupCatalogKeyV1::Hydroxymethyl && choice.label() == "CH2OH"
    }));
    assert!(crate::attached_compact_group_choices_v1().any(|choice| {
        choice.catalog_key() == CompactGroupCatalogKeyV1::AcylChloride && choice.label() == "COCl"
    }));

    for catalog_key in [
        CompactGroupCatalogKeyV1::Methyl,
        CompactGroupCatalogKeyV1::Nitro,
        CompactGroupCatalogKeyV1::Ethyl,
        CompactGroupCatalogKeyV1::Methoxy,
        CompactGroupCatalogKeyV1::Hydroxymethyl,
        CompactGroupCatalogKeyV1::AcylChloride,
    ] {
        let mut session = DocumentSession::load(SOURCE).expect("source");
        let result = commit_attachment_for(&mut session, catalog_key);
        let snapshot = result.observation().snapshot().clone();
        let group = result
            .observation()
            .projection()
            .molecules()
            .iter()
            .flat_map(|molecule| molecule.compact_groups())
            .find(|group| group.id() == result.compact_group_object_id())
            .expect("committed compact group remains projected");
        assert_eq!(group.catalog_key(), catalog_key);
        let reopened =
            DocumentSession::load(snapshot.cdml()).expect("reopen serialized compact group");
        assert!(
            reopened
                .document_observation()
                .expect("reopened observation")
                .projection()
                .molecules()
                .iter()
                .flat_map(|molecule| molecule.compact_groups())
                .any(|group| group.catalog_key() == catalog_key)
        );
    }
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
        molecule.id().cloned().expect("durable molecule"),
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
    let undone_molecule = &undone.observation().projection().molecules()[0];
    assert_eq!(undone_molecule.atoms().len(), 1);
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
            anchor(&session),
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
        molecule.id().cloned().expect("durable molecule"),
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
        undone.observation().projection().molecules()[0]
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
fn attached_hydroxymethyl_materialization_retains_carbon_focus_and_exterior_identity() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let attached = commit_attachment_for(&mut session, CompactGroupCatalogKeyV1::Hydroxymethyl);
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
        .expect("attached Hydroxymethyl molecule");
    let exterior_bond_id = molecule.bonds()[0]
        .id()
        .cloned()
        .expect("attached exterior bond has a durable identity");
    let request = DocumentCompactGroupMaterializationRequestV1::new(
        attached_snapshot.revision(),
        *attached_snapshot.digest(),
        molecule.id().cloned().expect("durable molecule"),
        attached.compact_group_object_id().clone(),
    );
    let mut pending = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            attached_snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .expect("Hydroxymethyl materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("Hydroxymethyl materialization commits");
    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = materialized.outcome()
    else {
        panic!("Hydroxymethyl materialization returns a focused outcome");
    };
    assert_materialized_attached_hydroxymethyl(
        &session,
        outcome.focus_atom_id(),
        &exterior_bond_id,
    );
}

#[test]
fn attached_carboxyl_materialization_retains_carbon_focus_and_exterior_identity() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let attached = commit_attachment_for(&mut session, CompactGroupCatalogKeyV1::Carboxyl);
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
        .expect("attached Carboxyl molecule");
    let exterior_bond_id = molecule.bonds()[0]
        .id()
        .cloned()
        .expect("attached exterior bond has a durable identity");
    let request = DocumentCompactGroupMaterializationRequestV1::new(
        attached_snapshot.revision(),
        *attached_snapshot.digest(),
        molecule.id().cloned().expect("durable molecule"),
        attached.compact_group_object_id().clone(),
    );
    let mut pending = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            attached_snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .expect("Carboxyl materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("Carboxyl materialization commits");
    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = materialized.outcome()
    else {
        panic!("Carboxyl materialization returns a focused outcome");
    };
    assert_materialized_attached_carboxyl(&session, outcome.focus_atom_id(), &exterior_bond_id);
}

#[test]
fn attached_cyano_materialization_retains_carbon_focus_and_exterior_identity() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let attached = commit_attachment_for(&mut session, CompactGroupCatalogKeyV1::Cyano);
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
        .expect("attached Cyano molecule");
    let exterior_neighbor_source_id = molecule
        .atoms()
        .iter()
        .find(|atom| atom.id() == Some(attached.focus_object_id()))
        .expect("attached Cyano retains the original exterior anchor")
        .source_id()
        .expect("attached Cyano exterior anchor has a source ID")
        .to_owned();
    let exterior_bond_id = molecule
        .bonds()
        .iter()
        .find(|bond| {
            ((bond.start().object_id() == Some(attached.focus_object_id())
                && bond.end().object_id() == Some(attached.compact_group_object_id()))
                || (bond.end().object_id() == Some(attached.focus_object_id())
                    && bond.start().object_id() == Some(attached.compact_group_object_id())))
                && bond.order() == Some(ferrum_core::BondOrder::Single)
                && bond.style() == Some(&ferrum_core::BondStyle::Normal)
        })
        .expect("attached Cyano exterior bond connects the anchor and compact group normally")
        .id()
        .cloned()
        .expect("attached exterior bond has a durable identity");
    let request = DocumentCompactGroupMaterializationRequestV1::new(
        attached_snapshot.revision(),
        *attached_snapshot.digest(),
        molecule.id().cloned().expect("durable molecule"),
        attached.compact_group_object_id().clone(),
    );
    let mut pending = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            attached_snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .expect("Cyano materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("Cyano materialization commits");
    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = materialized.outcome()
    else {
        panic!("Cyano materialization returns a focused outcome");
    };
    assert_materialized_attached_cyano(
        &session,
        outcome.focus_atom_id(),
        &exterior_neighbor_source_id,
        &exterior_bond_id,
    );
}
