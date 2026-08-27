use super::tests::{SOURCE, commit_attachment_for};
use super::*;
use crate::{
    DocumentCompactGroupMaterializationRequestV1, SessionOperation, SessionOperationOutcomeV1,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};

fn assert_materialized_attached_nitro(
    session: &DocumentSession,
    focus_atom_id: &DocumentObjectIdV1,
    exterior_bond_id: &DocumentObjectIdV1,
) {
    let observation = session.document_observation().expect("observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule");
    let attachment_nitrogen = molecule
        .atoms()
        .iter()
        .find(|atom| atom.document_object_id() == focus_atom_id)
        .expect("materialization focus is the nitro attachment nitrogen");
    assert_eq!(
        (
            attachment_nitrogen.element(),
            attachment_nitrogen.formal_charge(),
        ),
        (Some("N"), Some(1))
    );
    for (order, charge) in [
        (ferrum_core::BondOrder::Double, None),
        (ferrum_core::BondOrder::Single, Some(-1)),
    ] {
        let oxygen_bond = molecule
            .bonds()
            .iter()
            .find(|bond| {
                bond.document_object_id() != exterior_bond_id
                    && bond.order() == Some(order)
                    && bond.style() == Some(&ferrum_core::BondStyle::Normal)
                    && [bond.start(), bond.end()]
                        .into_iter()
                        .any(|endpoint| endpoint.source_id() == attachment_nitrogen.source_id())
            })
            .expect("nitro nitrogen has the expected oxygen bond");
        let oxygen = molecule
            .atoms()
            .iter()
            .find(|atom| {
                atom.source_id()
                    == if oxygen_bond.start().source_id() == attachment_nitrogen.source_id() {
                        oxygen_bond.end().source_id()
                    } else {
                        oxygen_bond.start().source_id()
                    }
            })
            .expect("nitro bond terminates at oxygen");
        assert_eq!(
            (oxygen.element(), oxygen.formal_charge()),
            (Some("O"), charge)
        );
    }
}

fn assert_materialized_attached_hydroxymethyl(
    session: &DocumentSession,
    focus_atom_id: &DocumentObjectIdV1,
    exterior_bond_id: &DocumentObjectIdV1,
) {
    let observation = session.document_observation().expect("observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule");
    assert!(molecule.compact_groups().is_empty());
    let attachment_carbon = molecule
        .atoms()
        .iter()
        .find(|atom| atom.document_object_id() == focus_atom_id)
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
        .find(|bond| bond.document_object_id() == exterior_bond_id)
        .expect("exterior bond retains its durable identity");
    let internal = molecule
        .bonds()
        .iter()
        .find(|bond| bond.document_object_id() != exterior_bond_id)
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
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule");
    assert!(molecule.compact_groups().is_empty());
    let attachment_carbon = molecule
        .atoms()
        .iter()
        .find(|atom| atom.document_object_id() == focus_atom_id)
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
        .find(|bond| bond.document_object_id() == exterior_bond_id)
        .expect("exterior bond retains its durable identity");
    let carbonyl = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.document_object_id() != exterior_bond_id
                && bond.order() == Some(ferrum_core::BondOrder::Double)
                && bond.start().source_id() == attachment_carbon.source_id()
        })
        .expect("carboxyl has a carbonyl bond from the attachment carbon");
    let hydroxyl = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.document_object_id() != exterior_bond_id
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
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("seeded molecule");
    assert!(molecule.compact_groups().is_empty());
    let attachment_carbon = molecule
        .atoms()
        .iter()
        .find(|atom| atom.document_object_id() == focus_atom_id)
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
            bond.document_object_id() != exterior_bond_id
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
        .find(|bond| bond.document_object_id() == exterior_bond_id)
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
}

fn assert_reopened_cyano_structure(session: &DocumentSession) {
    let observation = session
        .document_observation()
        .expect("reopened observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("reopened seeded molecule");
    let anchor = molecule
        .atoms()
        .iter()
        .find(|atom| atom.source_id() == Some("a"))
        .expect("reopened anchor");
    let attachment_carbon = molecule
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
        .expect("reopened cyano attachment carbon");
    let terminal_nitrogen = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.order() == Some(ferrum_core::BondOrder::Triple)
                && bond.style() == Some(&ferrum_core::BondStyle::Normal)
                && [bond.start(), bond.end()]
                    .into_iter()
                    .any(|endpoint| endpoint.source_id() == attachment_carbon.source_id())
        })
        .and_then(|bond| {
            [bond.start(), bond.end()].into_iter().find_map(|endpoint| {
                (endpoint.source_id() != attachment_carbon.source_id())
                    .then_some(endpoint.source_id())
            })
        })
        .and_then(|source_id| {
            molecule
                .atoms()
                .iter()
                .find(|atom| atom.source_id() == source_id)
        })
        .expect("reopened cyano terminal nitrogen");
    assert_eq!(
        (attachment_carbon.element(), terminal_nitrogen.element()),
        (Some("C"), Some("N"))
    );
}

#[test]
fn attached_nitro_materialization_retains_charged_nitrogen_focus() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let attached = commit_attachment_for(&mut session, CompactGroupCatalogKeyV1::Nitro);
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
        .expect("attached Nitro molecule");
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
        .expect("attached Nitro exterior bond connects the anchor and compact group normally")
        .document_object_id()
        .clone();
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
        .expect("Nitro materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("Nitro materialization commits");
    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = materialized.outcome()
    else {
        panic!("Nitro materialization returns a focused outcome");
    };
    assert_materialized_attached_nitro(&session, outcome.focus_atom_id(), &exterior_bond_id);
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
        .expect(
            "attached Hydroxymethyl exterior bond connects the anchor and compact group normally",
        )
        .document_object_id()
        .clone();
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
        .expect("attached Carboxyl exterior bond connects the anchor and compact group normally")
        .document_object_id()
        .clone();
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
        .find(|atom| atom.document_object_id() == attached.focus_object_id())
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
        .document_object_id()
        .clone();
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
    let reopened = DocumentSession::load(materialized.observation().snapshot().cdml())
        .expect("materialized Cyano reopens");
    assert_reopened_cyano_structure(&reopened);
}
