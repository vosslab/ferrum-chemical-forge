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
        .molecules()
        .iter()
        .flat_map(|molecule| molecule.atoms().iter())
        .find(|atom| atom.source_id() == Some("a") && atom.element() == Some("C"))
        .expect("source anchor atom")
        .document_object_id()
        .clone()
}

fn target(session: &DocumentSession) -> AttachedCompactGroupTargetV1 {
    let molecule_id = session
        .document_observation()
        .expect("observation")
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("source molecule")
        .document_object_id()
        .clone();
    AttachedCompactGroupTargetV1::new(molecule_id, anchor(session))
}

#[test]
fn attached_acyl_chloride_materialization_retains_carbon_focus_and_exterior_identity() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let mut attachment = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            target(&session),
            AttachCompactGroupV1::new(
                CompactGroupCatalogKeyV1::AcylChloride,
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            ),
        )
        .expect("AcylChloride attachment prepares");
    let attached = session
        .commit_attach_compact_group_v1(&mut attachment)
        .expect("AcylChloride attachment commits");
    let attached_compact_group_id = attached.compact_group_object_id().clone();
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
                .any(|group| group.id() == &attached_compact_group_id)
        })
        .expect("attached AcylChloride molecule");
    let original_anchor_source_id = molecule
        .atoms()
        .iter()
        .find(|atom| atom.document_object_id() == attached.focus_object_id())
        .expect("attached focus retains the original source anchor")
        .source_id()
        .expect("original source anchor has a durable source ID")
        .to_owned();
    let exterior = molecule
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
            "attached AcylChloride exterior bond connects the anchor and compact group normally",
        );
    let exterior_bond_id = exterior.document_object_id().clone();
    let exterior_start_source_id = exterior
        .start()
        .source_id()
        .expect("attached exterior bond start has a durable source ID")
        .to_owned();
    let exterior_end_source_id = exterior
        .end()
        .source_id()
        .expect("attached exterior bond end has a durable source ID")
        .to_owned();
    let exterior_order = exterior.order();
    let exterior_style = exterior.style().cloned();
    let original_anchor_is_exterior_start = match (
        exterior_start_source_id.as_str() == original_anchor_source_id,
        exterior_end_source_id.as_str() == original_anchor_source_id,
        exterior.start().object_id() == Some(attached.compact_group_object_id()),
        exterior.end().object_id() == Some(attached.compact_group_object_id()),
    ) {
        (true, false, false, true) => true,
        (false, true, true, false) => false,
        endpoint_sides => panic!(
            "attached exterior bond must have the original anchor and compact-group placeholder on opposite directed sides: {endpoint_sides:?}"
        ),
    };
    let request = DocumentCompactGroupMaterializationRequestV1::new(
        attached_snapshot.revision(),
        *attached_snapshot.digest(),
        molecule.document_object_id().clone(),
        attached_compact_group_id.clone(),
    );
    let mut pending = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            attached_snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .expect("AcylChloride materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("AcylChloride materialization commits");
    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = materialized.outcome()
    else {
        panic!("AcylChloride materialization returns a focused outcome");
    };

    let observation = session.document_observation().expect("observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| {
            molecule
                .bonds()
                .iter()
                .any(|bond| bond.document_object_id() == &exterior_bond_id)
        })
        .expect("materialized molecule retains the exterior bond");
    assert!(
        !molecule
            .compact_groups()
            .iter()
            .any(|group| group.id() == &attached_compact_group_id),
        "materialization removes the authored compact-group identity while leaving unrelated groups unconstrained"
    );
    let attachment_carbon = molecule
        .atoms()
        .iter()
        .find(|atom| atom.document_object_id() == outcome.focus_atom_id())
        .expect("materialization focus is the acyl chloride attachment carbon");
    let attachment_carbon_source_id = attachment_carbon
        .source_id()
        .expect("attachment carbon has a durable source ID")
        .to_owned();
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
        .find(|bond| bond.document_object_id() == &exterior_bond_id)
        .expect("exterior bond retains its durable identity");
    assert_eq!(exterior.document_object_id(), &exterior_bond_id);
    assert_eq!(
        (exterior.order(), exterior.style()),
        (exterior_order, exterior_style.as_ref())
    );
    let carbonyl_oxygen = molecule
        .atoms()
        .iter()
        .find(|atom| {
            atom.element() == Some("O")
                && atom.formal_charge().is_none()
                && molecule.bonds().iter().any(|bond| {
                    bond.order() == Some(ferrum_core::BondOrder::Double)
                        && bond.style() == Some(&ferrum_core::BondStyle::Normal)
                        && [bond.start().source_id(), bond.end().source_id()]
                            .into_iter()
                            .all(|source_id| {
                                source_id == Some(attachment_carbon_source_id.as_str())
                                    || source_id == atom.source_id()
                            })
                })
        })
        .expect("acyl chloride has a neutral oxygen at the carbonyl endpoint");
    let chlorine = molecule
        .atoms()
        .iter()
        .find(|atom| {
            atom.element() == Some("Cl")
                && atom.formal_charge().is_none()
                && molecule.bonds().iter().any(|bond| {
                    bond.order() == Some(ferrum_core::BondOrder::Single)
                        && bond.style() == Some(&ferrum_core::BondStyle::Normal)
                        && [bond.start().source_id(), bond.end().source_id()]
                            .into_iter()
                            .all(|source_id| {
                                source_id == Some(attachment_carbon_source_id.as_str())
                                    || source_id == atom.source_id()
                            })
                })
        })
        .expect("acyl chloride has a neutral chlorine at the C-Cl endpoint");
    assert_eq!(
        (carbonyl_oxygen.element(), carbonyl_oxygen.formal_charge()),
        (Some("O"), None)
    );
    assert_eq!(
        (chlorine.element(), chlorine.formal_charge()),
        (Some("Cl"), None)
    );
    if original_anchor_is_exterior_start {
        assert_eq!(
            exterior.start().source_id(),
            Some(original_anchor_source_id.as_str()),
            "materialization preserves the original anchor on the exterior bond start side"
        );
        assert_eq!(
            exterior.end().source_id(),
            Some(attachment_carbon_source_id.as_str()),
            "materialization rewires only the exterior bond end placeholder to the returned carbon focus"
        );
    } else {
        assert_eq!(
            exterior.end().source_id(),
            Some(original_anchor_source_id.as_str()),
            "materialization preserves the original anchor on the exterior bond end side"
        );
        assert_eq!(
            exterior.start().source_id(),
            Some(attachment_carbon_source_id.as_str()),
            "materialization rewires only the exterior bond start placeholder to the returned carbon focus"
        );
    }
}
