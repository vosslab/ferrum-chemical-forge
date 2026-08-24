use super::{
    CompactGroupMaterializationErrorV1, commit_compact_group_materialization_v1,
    commit_compact_group_placement_v1, prepare_compact_group_materialization_v1,
    prepare_compact_group_placement_v1,
};
use ferrum_document::{
    CompactGroupCatalogKeyV1, CompactGroupMaterializationRefusalV1,
    CompactGroupMaterializationRequestV1, CompactGroupPlacementModeV1,
    CompactGroupPlacementRequestV1, DocumentFenceV1, DocumentSession, MoleculeInsertionAtomV1,
    MoleculeInsertionV1, Point3V1,
};

fn point(x: f64, y: f64) -> Point3V1 {
    Point3V1::new(x, y, 0.0).expect("finite point")
}

fn fence(session: &DocumentSession) -> DocumentFenceV1 {
    let snapshot = session.snapshot().expect("snapshot");
    DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
}

fn session_with_anchor() -> DocumentSession {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let insertion = MoleculeInsertionV1::new(
        vec![MoleculeInsertionAtomV1::new("C", point(0.0, 0.0), None, None, None).expect("atom")],
        Vec::new(),
    )
    .expect("molecule");
    let revision = session.snapshot().expect("initial snapshot").revision();
    let mut pending = session
        .prepare_admitted_molecule_insertion_v1(revision, &insertion)
        .expect("candidate");
    session
        .commit_admitted_molecule_insertion_v1(revision, &mut pending)
        .expect("commit molecule");
    session
}

fn place_attached_at(
    session: &mut DocumentSession,
    key: CompactGroupCatalogKeyV1,
    anchor: Point3V1,
) {
    let observation = session
        .observe(session.snapshot().expect("snapshot").revision())
        .expect("observation");
    let molecule = &observation.projection().molecules()[0];
    let request = CompactGroupPlacementRequestV1::new(
        fence(session),
        key,
        anchor,
        CompactGroupPlacementModeV1::Attached {
            molecule_id: molecule.id().expect("molecule identity").clone(),
            anchor_atom_id: molecule.atoms()[0].id().expect("atom identity").clone(),
        },
    );
    let mut prepared =
        prepare_compact_group_placement_v1(session, &request).expect("renderable placement");
    commit_compact_group_placement_v1(session, &mut prepared).expect("placement commit");
}

fn place_attached(session: &mut DocumentSession, key: CompactGroupCatalogKeyV1) {
    place_attached_at(session, key, point(24.0, 0.0));
}

fn place_free(session: &mut DocumentSession, key: CompactGroupCatalogKeyV1) {
    let request = CompactGroupPlacementRequestV1::new(
        fence(session),
        key,
        point(24.0, 0.0),
        CompactGroupPlacementModeV1::Free,
    );
    let mut prepared =
        prepare_compact_group_placement_v1(session, &request).expect("renderable free placement");
    commit_compact_group_placement_v1(session, &mut prepared).expect("free placement commit");
}

fn materialization_request(session: &DocumentSession) -> CompactGroupMaterializationRequestV1 {
    let observation = session
        .observe(session.snapshot().expect("snapshot").revision())
        .expect("observation");
    let molecule = &observation.projection().molecules()[0];
    CompactGroupMaterializationRequestV1::new(
        fence(session),
        molecule.id().expect("molecule identity").clone(),
        molecule.compact_groups()[0].id().clone(),
    )
}

#[test]
fn attached_methyl_replaces_the_group_with_a_renderer_admitted_atom() {
    let mut session = session_with_anchor();
    place_attached(&mut session, CompactGroupCatalogKeyV1::Methyl);
    let before = session.snapshot().expect("before preparation");
    let request = materialization_request(&session);
    let mut prepared = prepare_compact_group_materialization_v1(&mut session, &request)
        .expect("renderer-admitted methyl candidate");
    assert_eq!(
        session.snapshot().expect("preflight preserves source"),
        before
    );
    let committed = commit_compact_group_materialization_v1(&mut session, &mut prepared)
        .expect("one materialization transition");
    assert_eq!(committed.materialization().created_atom_count(), 1);
    assert!(
        committed
            .operation_result()
            .observation()
            .projection()
            .molecules()[0]
            .compact_groups()
            .is_empty()
    );
}

#[test]
fn materialization_preserves_recipe_hydrogen_facts_and_returns_its_attachment_focus() {
    let mut session = session_with_anchor();
    place_attached(&mut session, CompactGroupCatalogKeyV1::Methyl);
    let request = materialization_request(&session);
    let mut prepared =
        prepare_compact_group_materialization_v1(&mut session, &request).expect("candidate");
    let prepared_focus = prepared
        .materialization()
        .replacement_focus_target()
        .clone();
    let committed =
        commit_compact_group_materialization_v1(&mut session, &mut prepared).expect("commit");
    assert_eq!(
        committed.materialization().replacement_focus_target(),
        &prepared_focus
    );
    let molecule = &committed
        .operation_result()
        .observation()
        .projection()
        .molecules()[0];
    let methyl = molecule
        .atoms()
        .iter()
        .find(|atom| atom.id() == Some(&prepared_focus))
        .expect("materialization result names the replacement attachment atom");
    assert_eq!(methyl.element(), Some("C"));
    assert_eq!(methyl.explicit_hydrogens(), Some(3));

    let mut nitro = session_with_anchor();
    place_attached(&mut nitro, CompactGroupCatalogKeyV1::Nitro);
    let request = materialization_request(&nitro);
    let mut prepared =
        prepare_compact_group_materialization_v1(&mut nitro, &request).expect("candidate");
    let committed =
        commit_compact_group_materialization_v1(&mut nitro, &mut prepared).expect("commit");
    let atoms = committed
        .operation_result()
        .observation()
        .projection()
        .molecules()[0]
        .atoms();
    let recipe_atoms: Vec<_> = atoms
        .iter()
        .filter(|atom| atom.element() != Some("C"))
        .collect();
    assert_eq!(recipe_atoms.len(), 3);
    assert!(
        recipe_atoms
            .iter()
            .all(|atom| atom.explicit_hydrogens() == Some(0))
    );
    assert!(
        atoms
            .iter()
            .any(|atom| atom.formal_charge() == Some(1) && atom.element() == Some("N"))
    );
}

#[test]
fn attached_materialization_preserves_the_exterior_bond_identity() {
    let mut session = session_with_anchor();
    place_attached(&mut session, CompactGroupCatalogKeyV1::Methyl);
    let before = session
        .observe(session.snapshot().expect("snapshot").revision())
        .expect("observation");
    let exterior = before.projection().molecules()[0].bonds()[0]
        .id()
        .expect("bond identity")
        .clone();
    let request = materialization_request(&session);
    let mut prepared =
        prepare_compact_group_materialization_v1(&mut session, &request).expect("candidate");
    let committed =
        commit_compact_group_materialization_v1(&mut session, &mut prepared).expect("commit");
    let retained = committed
        .operation_result()
        .observation()
        .projection()
        .molecules()[0]
        .bonds()[0]
        .id()
        .expect("retained exterior bond identity");
    assert_eq!(retained, &exterior);
}

#[test]
fn free_methyl_and_attached_nitro_reopen_as_ordinary_document_atoms() {
    let mut free = DocumentSession::create_empty_document_v1().expect("empty session");
    place_free(&mut free, CompactGroupCatalogKeyV1::Methyl);
    let request = materialization_request(&free);
    let mut prepared =
        prepare_compact_group_materialization_v1(&mut free, &request).expect("candidate");
    let committed =
        commit_compact_group_materialization_v1(&mut free, &mut prepared).expect("commit");
    let reopened =
        DocumentSession::load(committed.operation_result().observation().snapshot().cdml())
            .expect("materialized free methyl reopens");
    let reopened_observation = reopened.observe(0).expect("reopened projection");
    let molecule = &reopened_observation.projection().molecules()[0];
    assert!(molecule.compact_groups().is_empty());
    assert_eq!(molecule.atoms().len(), 1);
    assert_eq!(molecule.atoms()[0].explicit_hydrogens(), Some(3));

    let mut session = session_with_anchor();
    place_attached(&mut session, CompactGroupCatalogKeyV1::Nitro);
    let request = materialization_request(&session);
    let mut prepared = prepare_compact_group_materialization_v1(&mut session, &request)
        .expect("renderer-admitted nitro candidate");
    let committed = commit_compact_group_materialization_v1(&mut session, &mut prepared)
        .expect("nitro transition");
    assert_eq!(committed.materialization().created_internal_bond_count(), 2);
    let focus = committed
        .materialization()
        .replacement_focus_target()
        .clone();
    let reopened =
        DocumentSession::load(committed.operation_result().observation().snapshot().cdml())
            .expect("materialized nitro reopens");
    let reopened_observation = reopened.observe(0).expect("reopened projection");
    let molecule = &reopened_observation.projection().molecules()[0];
    assert!(molecule.compact_groups().is_empty());
    assert_eq!(molecule.atoms().len(), 4);
    assert_eq!(molecule.bonds().len(), 3);
    assert!(
        molecule
            .atoms()
            .iter()
            .any(|atom| atom.id() == Some(&focus))
    );
}

#[test]
fn materialization_is_undoable_and_redoable_as_one_transition() {
    let mut session = session_with_anchor();
    place_attached(&mut session, CompactGroupCatalogKeyV1::Methyl);
    let before = session.snapshot().expect("before");
    let request = materialization_request(&session);
    let mut prepared =
        prepare_compact_group_materialization_v1(&mut session, &request).expect("candidate");
    let committed =
        commit_compact_group_materialization_v1(&mut session, &mut prepared).expect("commit");
    let after = committed
        .operation_result()
        .observation()
        .snapshot()
        .clone();
    let undone = session.undo(after.revision()).expect("undo");
    assert_eq!(undone.observation().snapshot().cdml(), before.cdml());
    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("redo");
    assert_eq!(redone.observation().snapshot().cdml(), after.cdml());
}

#[test]
fn unsupported_catalog_key_refuses_without_mutating_the_document() {
    let mut session = session_with_anchor();
    place_attached(&mut session, CompactGroupCatalogKeyV1::Ethyl);
    let before = session.snapshot().expect("before");
    let request = materialization_request(&session);
    assert!(matches!(
        prepare_compact_group_materialization_v1(&mut session, &request),
        Err(CompactGroupMaterializationErrorV1::Refusal(
            CompactGroupMaterializationRefusalV1::NotYetSupported
        ))
    ));
    assert_eq!(session.snapshot().expect("after refusal"), before);
}

#[test]
fn materialization_receipts_refuse_stale_foreign_and_replayed_redemption_atomically() {
    let mut stale = session_with_anchor();
    place_attached(&mut stale, CompactGroupCatalogKeyV1::Methyl);
    let request = materialization_request(&stale);
    let mut prepared =
        prepare_compact_group_materialization_v1(&mut stale, &request).expect("candidate");
    let current_revision = stale.snapshot().expect("source snapshot").revision();
    stale
        .undo(current_revision)
        .expect("independent transition");
    let after_transition = stale.snapshot().expect("transitioned snapshot");
    assert!(matches!(
        commit_compact_group_materialization_v1(&mut stale, &mut prepared),
        Err(CompactGroupMaterializationErrorV1::Refusal(
            CompactGroupMaterializationRefusalV1::StaleObservation
        ))
    ));
    assert_eq!(
        stale.snapshot().expect("stale refusal preserves state"),
        after_transition
    );

    let mut owner = session_with_anchor();
    place_attached(&mut owner, CompactGroupCatalogKeyV1::Methyl);
    let request = materialization_request(&owner);
    let mut prepared =
        prepare_compact_group_materialization_v1(&mut owner, &request).expect("candidate");
    let mut foreign = DocumentSession::load(owner.snapshot().expect("owner source").cdml())
        .expect("foreign session");
    let foreign_before = foreign.snapshot().expect("foreign snapshot");
    assert!(matches!(
        commit_compact_group_materialization_v1(&mut foreign, &mut prepared),
        Err(CompactGroupMaterializationErrorV1::Refusal(
            CompactGroupMaterializationRefusalV1::ForeignSession
        ))
    ));
    assert_eq!(
        foreign.snapshot().expect("foreign refusal preserves state"),
        foreign_before
    );
    let committed = commit_compact_group_materialization_v1(&mut owner, &mut prepared)
        .expect("owner remains able to commit");
    let owner_after = owner.snapshot().expect("owner committed");
    assert!(matches!(
        commit_compact_group_materialization_v1(&mut owner, &mut prepared),
        Err(CompactGroupMaterializationErrorV1::Replayed)
    ));
    assert_eq!(
        owner.snapshot().expect("replay refusal preserves state"),
        owner_after
    );
    assert_eq!(committed.materialization().created_atom_count(), 1);
}

#[test]
fn non_axis_nitro_materialization_preserves_attachment_orientation_without_reflection() {
    let mut session = session_with_anchor();
    place_attached_at(
        &mut session,
        CompactGroupCatalogKeyV1::Nitro,
        point(24.0, 24.0),
    );
    let request = materialization_request(&session);
    let mut prepared =
        prepare_compact_group_materialization_v1(&mut session, &request).expect("candidate");
    let committed =
        commit_compact_group_materialization_v1(&mut session, &mut prepared).expect("commit");
    let molecule = &committed
        .operation_result()
        .observation()
        .projection()
        .molecules()[0];
    let nitrogen = molecule
        .atoms()
        .iter()
        .find(|atom| atom.id() == Some(committed.materialization().replacement_focus_target()))
        .expect("focus names recipe attachment nitrogen");
    let exterior = molecule
        .atoms()
        .iter()
        .find(|atom| atom.id() != nitrogen.id())
        .expect("retained exterior attachment atom");
    let axis_x = nitrogen.position().x() - exterior.position().x();
    let axis_y = nitrogen.position().y() - exterior.position().y();
    assert!(
        axis_x > 0.0 && axis_y > 0.0,
        "non-axis attachment keeps its authored direction"
    );
    let neutral_oxygen = molecule
        .atoms()
        .iter()
        .find(|atom| atom.element() == Some("O") && atom.formal_charge() == Some(0))
        .expect("neutral recipe oxygen");
    let anionic_oxygen = molecule
        .atoms()
        .iter()
        .find(|atom| atom.element() == Some("O") && atom.formal_charge() == Some(-1))
        .expect("anionic recipe oxygen");
    let neutral_dx = neutral_oxygen.position().x() - nitrogen.position().x();
    let neutral_dy = neutral_oxygen.position().y() - nitrogen.position().y();
    let neutral_forward = axis_x * neutral_dx + axis_y * neutral_dy;
    let neutral_side = axis_x * neutral_dy - axis_y * neutral_dx;
    let anionic_dx = anionic_oxygen.position().x() - nitrogen.position().x();
    let anionic_dy = anionic_oxygen.position().y() - nitrogen.position().y();
    let anionic_forward = axis_x * anionic_dx + axis_y * anionic_dy;
    let anionic_side = axis_x * anionic_dy - axis_y * anionic_dx;
    assert!(neutral_forward > 0.0 && anionic_forward > 0.0);
    assert!(neutral_side > 0.0 && anionic_side < 0.0);
}
