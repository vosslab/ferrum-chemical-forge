"""Installed-wheel contract for Ferrum-Chem's revisioned document boundary."""

from __future__ import annotations

from pathlib import Path

import pytest

import ferrum_chem


SOURCE = (
    "<cdml xmlns='urn:ferrum:cdml'><molecule id=\"m\"><atom id=\"a\" name=\"C\">"
    "<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
)

BOND_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml" version="26.08"><molecule id="m">'
    '<atom id="a" name="C"><point x="1" y="2"/></atom>'
    '<atom id="b" name="O"><point x="3" y="2"/></atom>'
    '</molecule></cdml>'
)

COORDINATE_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml" version="26.08"><molecule id="m">'
    '<atom id="a" name="C"><point x="10" y="20"/></atom>'
    '<atom id="b" name="C"><point x="50" y="20"/></atom>'
    '<atom id="c" name="O"><point x="50" y="60"/></atom>'
    '<bond id="ab" start="a" end="b" type="n1"/>'
    '<bond id="bc" start="b" end="c" type="n1"/>'
    '</molecule></cdml>'
)

ATOM_PROPERTIES_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><molecule id="m">'
    '<atom id="a" name="C" charge="2" valency="4" isotope="13" '
    'multiplicity="3" show="no" hydrogens="off" vendor_keep="yes">'
    '<point x="1" y="2"/><font family="Courier" size="11" '
    'vendor_keep="yes"/><ftext>keep</ftext><v:keep/></atom>'
    '</molecule><v:opaque id="retained"/></cdml>'
)

BOND_PROPERTIES_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><molecule id="m">'
    '<atom id="a" name="C"><point x="1" y="2"/></atom>'
    '<atom id="b" name="O"><point x="20" y="2"/></atom>'
    '<bond id="ab" start="a" end="b" type="n1" line_width="1.5" '
    'bond_width="-2" wedge_width="3" color="#A0B1C2" vendor_keep="yes">'
    '<v:keep/></bond></molecule><v:opaque id="retained"/></cdml>'
)

HAWORTH_POSITION_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C"><point x="0" y="0"/></atom>'
    '<atom id="b" name="O"><point x="1" y="0"/></atom>'
    '<bond start="a" end="b" haworth_position="front"/>'
    '<bond start="b" end="a" haworth_position="back"/>'
    '<bond start="a" end="b" haworth_position="side"/></molecule></cdml>'
)


def set_atom(element: str) -> ferrum_chem.DocumentOperationV1:
    return ferrum_chem.DocumentOperationV1.set_atom_element("a", element)


def test_haworth_position_projection_preserves_closed_depth_and_reports_malformed() -> None:
    projection = ferrum_chem.DocumentSession.load(HAWORTH_POSITION_SOURCE).observe(0).projection
    bonds = projection.molecules[0].bonds

    assert (bonds[0].haworth_position, bonds[1].haworth_position) == (
        ferrum_chem.DocumentHaworthPositionV1.front,
        ferrum_chem.DocumentHaworthPositionV1.back,
    )
    assert bonds[2].haworth_position is None
    assert [issue.code for issue in projection.issues].count("invalid_presentation_fact") == 1


def test_bond_properties_reject_hostile_python_intent_without_mutation() -> None:
    class TupleSubclass(tuple):
        pass

    change_type = ferrum_chem.DocumentBondPropertyChangeV1
    order = change_type.order(ferrum_chem.DocumentBondOrderV1.double)
    session = ferrum_chem.DocumentSession.load(BOND_PROPERTIES_SOURCE)
    before = session.snapshot()

    with pytest.raises(TypeError):
        ferrum_chem.DocumentOperationV1.set_bond_properties("ab", [order])
    with pytest.raises(TypeError):
        ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (object(),))
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.set_bond_properties(
            "ab", TupleSubclass((order,)),
        )
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order,) * 8)
    with pytest.raises(ferrum_chem.OperationValidationError):
        ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order, order))
    for invalid in (0.0, float("nan"), float("inf")):
        with pytest.raises(ferrum_chem.OperationValidationError):
            change_type.bond_width(invalid)
    with pytest.raises(ferrum_chem.OperationValidationError):
        change_type.line_width(-1.0)
    with pytest.raises(ferrum_chem.OperationValidationError):
        change_type.line_width(True)
    with pytest.raises(ferrum_chem.OperationValidationError):
        change_type.color("not-a-color")
    with pytest.raises(AttributeError):
        order.value = 2
    with pytest.raises(ferrum_chem.RevisionConflictError):
        session.submit(
            1, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order,)),
        )
    with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
        session.submit(
            0, ferrum_chem.DocumentOperationV1.set_bond_properties("missing", (order,)),
        )
    unsupported = ferrum_chem.DocumentSession.load(
        BOND_PROPERTIES_SOURCE.replace('type="n1"', 'type="l1"'),
    )
    unsupported_before = unsupported.snapshot()
    with pytest.raises(ferrum_chem.OperationValidationError):
        unsupported.submit(
            0, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order,)),
        )
    unsupported_after = unsupported.snapshot()
    assert (unsupported_after.revision, unsupported_after.digest) == (
        unsupported_before.revision, unsupported_before.digest,
    )
    after = session.snapshot()
    assert (after.revision, after.digest) == (before.revision, before.digest)

    no_change = session.submit(
        0, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", ()),
    ).observation.snapshot
    assert no_change.revision == 0 and no_change.is_dirty is False


def test_atom_deletion_removes_incident_bonds_and_uses_one_history_entry() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
        '<atom id="a" name="C"><point x="1" y="2"/></atom>'
        '<atom id="b" name="O"><point x="3" y="2"/></atom>'
        '<bond id="ab" type="n1" start="a" end="b"/>'
        '</molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    deleted = session.submit(
        0, ferrum_chem.DocumentOperationV1.delete_atom("b"),
    ).observation

    assert tuple(atom.source_id for atom in deleted.projection.molecules[0].atoms) == ("a",)
    assert not deleted.projection.molecules[0].bonds
    restored = session.undo(1).observation.projection.molecules[0]
    assert len(restored.atoms) == 2 and len(restored.bonds) == 1
    redone = session.redo(2).observation.projection.molecules[0]
    assert len(redone.atoms) == 1 and not redone.bonds


def test_bond_deletion_preserves_atoms_and_uses_one_history_entry() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
        '<atom id="a" name="C"><point x="1" y="2"/></atom>'
        '<atom id="b" name="O"><point x="3" y="2"/></atom>'
        '<bond id="ab" type="n1" start="a" end="b"/>'
        '</molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    deleted = session.submit(
        0, ferrum_chem.DocumentOperationV1.delete_bond("ab"),
    ).observation

    assert len(deleted.projection.molecules[0].atoms) == 2
    assert not deleted.projection.molecules[0].bonds
    assert len(session.undo(1).observation.projection.molecules[0].bonds) == 1
    assert not session.redo(2).observation.projection.molecules[0].bonds
    with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
        session.submit(3, ferrum_chem.DocumentOperationV1.delete_bond("missing"))
    assert caught.value.object_id == "missing"
    assert session.snapshot().revision == 3


def test_bond_order_change_is_typed_noop_aware_and_undoable() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
        '<atom id="a" name="C"><point x="1" y="2"/></atom>'
        '<atom id="b" name="O"><point x="20" y="2"/></atom>'
        '<bond id="ab" type="n1" start="a" end="b" retained="yes"/>'
        '</molecule></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    no_change = session.submit(
        0,
        ferrum_chem.DocumentOperationV1.set_bond_order(
            "ab", ferrum_chem.DocumentBondOrderV1.single,
        ),
    ).observation
    assert no_change.snapshot.revision == 0

    changed = session.submit(
        0,
        ferrum_chem.DocumentOperationV1.set_bond_order(
            "ab", ferrum_chem.DocumentBondOrderV1.double,
        ),
    ).observation
    assert changed.snapshot.revision == 1
    assert changed.projection.molecules[0].bonds[0].source_type == "n2"
    assert 'retained="yes"' in changed.snapshot.cdml
    assert session.undo(1).observation.projection.molecules[0].bonds[0].source_type == "n1"
    assert session.redo(2).observation.projection.molecules[0].bonds[0].source_type == "n2"
    with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
        session.submit(
            3,
            ferrum_chem.DocumentOperationV1.set_bond_order(
                "missing", ferrum_chem.DocumentBondOrderV1.triple,
            ),
        )
    assert caught.value.object_id == "missing"
    assert session.snapshot().revision == 3


def test_operation_validation_errors_are_specific_and_structured() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)

    with pytest.raises(ferrum_chem.InvalidAtomElementError):
        session.submit(0, set_atom("2"))
    with pytest.raises(ferrum_chem.InvalidAtomElementError):
        session.submit(0, set_atom("Xx"))
    with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
        session.submit(
            0,
            ferrum_chem.DocumentOperationV1.set_atom_element("missing", "N"),
        )

    assert caught.value.object_id == "missing"
    assert session.snapshot().revision == 0


def test_prepared_atom_insertion_is_revision_bound_and_one_use() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    molecule_object_id = session.observe(0).projection.molecules[0].id
    prepared = session.prepare_create_atom_v1(
        0, molecule_object_id, "O", 3.0, 4.0, 0.0,
    )

    assert prepared.identifier.startswith("ferrum-atom-v1-")
    committed = session.commit_create_atom(0, prepared).observation.snapshot
    assert committed.revision == 1
    assert f'id="{prepared.identifier}"' in committed.cdml

    with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
        session.commit_create_atom(1, prepared)


def test_prepared_bond_insertion_preserves_directed_presentation_and_is_one_use() -> None:
    session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
    projection = session.observe(0).projection
    start, end = (atom.id for atom in projection.molecules[0].atoms)
    prepared = session.prepare_create_bond_v2(
        0, start, end, ferrum_chem.DocumentBondPresentationV1.solid_wedge,
    )

    assert prepared.identifier == "ferrum-bond-v1-0"
    committed = session.commit_create_bond(0, prepared).observation
    assert committed.snapshot.revision == 1
    bond = committed.projection.molecules[0].bonds[0]
    assert bond.source_type == "w1"
    assert (bond.start.object_id, bond.end.object_id) == (start, end)
    with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
        session.commit_create_bond(1, prepared)


def test_bond_insertion_rejects_self_and_duplicate_edges_without_state_change() -> None:
    session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
    start, end = (atom.id for atom in session.observe(0).projection.molecules[0].atoms)
    with pytest.raises(ferrum_chem.OperationValidationError):
        session.prepare_create_bond_v2(
            0, start, start, ferrum_chem.DocumentBondPresentationV1.hashed_wedge,
        )
    assert session.snapshot().revision == 0

    prepared = session.prepare_create_bond_v2(
        0, start, end, ferrum_chem.DocumentBondPresentationV1.hashed_wedge,
    )
    session.commit_create_bond(0, prepared)
    with pytest.raises(ferrum_chem.OperationValidationError):
        session.prepare_create_bond_v2(
            1, end, start, ferrum_chem.DocumentBondPresentationV1.normal_double,
        )
    assert session.snapshot().revision == 1


def test_bonded_atom_insertion_is_one_frozen_rust_operation_and_one_undo() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    start = session.observe(0).projection.molecules[0].atoms[0].id
    prepared = session.prepare_create_bonded_atom_v2(
        0, start, "O", 8.0, 9.0, 0.0,
        ferrum_chem.DocumentBondPresentationV1.hashed_wedge,
    )

    assert prepared.__class__.__module__ == "ferrum_chem"
    assert prepared.atom_identifier == "ferrum-atom-v1-0"
    assert prepared.bond_identifier == "ferrum-bond-v1-0"
    assert session.snapshot().revision == 0
    committed = session.commit_create_bonded_atom(0, prepared).observation
    molecule = committed.projection.molecules[0]
    assert committed.snapshot.revision == 1
    assert len(molecule.atoms) == 2 and len(molecule.bonds) == 1
    assert molecule.atoms[1].source_id == prepared.atom_identifier
    assert (molecule.atoms[1].position.x, molecule.atoms[1].position.y) == (8.0, 9.0)
    assert molecule.bonds[0].source_type == "h1"
    assert molecule.bonds[0].end.source_id == prepared.atom_identifier
    undone = session.undo(1).observation.projection.molecules[0]
    assert len(undone.atoms) == 1 and not undone.bonds
    with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
        session.commit_create_bonded_atom(2, prepared)


def test_prepared_atom_requires_finite_explicit_coordinates() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    molecule_object_id = session.observe(0).projection.molecules[0].id

    with pytest.raises(ferrum_chem.ProjectionError) as caught:
        session.prepare_create_atom_v1(
            0, molecule_object_id, "O", float("nan"), 0.0, 0.0,
        )

    assert "not finite" in caught.value.reason
    assert session.snapshot().revision == 0


def test_molecule_coordinate_preparation_is_frozen_revision_bound_and_undoable() -> None:
    session = ferrum_chem.DocumentSession.load(COORDINATE_SOURCE)
    observation = session.observe(0)
    molecule_id = observation.projection.molecules[0].id
    prepared = ferrum_chem.prepare_molecule_coordinates_v1(observation, molecule_id)

    assert type(prepared) is ferrum_chem.PreparedMoleculeCoordinatesV1
    assert prepared.__class__.__module__ == "ferrum_chem"
    assert prepared.source_revision == observation.snapshot.revision
    assert prepared.source_digest == observation.snapshot.digest
    assert prepared.molecule_id == molecule_id and prepared.atom_count == 3
    with pytest.raises(AttributeError):
        prepared.atom_count = 4

    changed = session.apply_molecule_coordinates_v1(0, prepared).observation
    assert changed.snapshot.revision == 1 and changed.snapshot.is_dirty
    assert session.undo(1).observation.snapshot.cdml == observation.snapshot.cdml
    assert session.redo(2).observation.snapshot.digest == changed.snapshot.digest


def test_stale_molecule_coordinate_result_does_not_change_current_state() -> None:
    session = ferrum_chem.DocumentSession.load(COORDINATE_SOURCE)
    observation = session.observe(0)
    molecule_id = observation.projection.molecules[0].id
    prepared = ferrum_chem.prepare_molecule_coordinates_v1(observation, molecule_id)
    changed = session.submit(
        0, ferrum_chem.DocumentOperationV1.set_atom_element("a", "N"),
    ).observation.snapshot

    with pytest.raises(ferrum_chem.OperationValidationError):
        session.apply_molecule_coordinates_v1(changed.revision, prepared)
    assert session.snapshot().digest == changed.digest


def test_foreign_session_rejection_keeps_prepared_atom_retryable_by_its_owner() -> None:
    owner = ferrum_chem.DocumentSession.load(SOURCE)
    foreign = ferrum_chem.DocumentSession.load(SOURCE)
    molecule_object_id = owner.observe(0).projection.molecules[0].id
    prepared = owner.prepare_create_atom_v1(
        0, molecule_object_id, "O", 3.0, 4.0, 0.0,
    )

    with pytest.raises(ferrum_chem.PreparedOperationForeignSessionError):
        foreign.commit_create_atom(0, prepared)

    accepted = owner.commit_create_atom(0, prepared).observation.snapshot
    assert accepted.revision == 1
    assert f'id="{prepared.identifier}"' in accepted.cdml


def test_create_atom_requires_a_durable_typed_molecule_selector() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    atom_object_id = session.observe(0).projection.molecules[0].atoms[0].id

    with pytest.raises(ferrum_chem.InvalidDocumentObjectIdError):
        session.prepare_create_atom_v1(0, "m", "O", 3.0, 4.0, 0.0)
    with pytest.raises(ferrum_chem.OperationValidationError):
        session.prepare_create_atom_v1(0, atom_object_id, "O", 3.0, 4.0, 0.0)


def test_confirmed_save_or_unconfirmed_outcome_preserves_exact_contract(
    tmp_path: Path,
) -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    changed = session.submit(0, set_atom("N")).observation.snapshot
    published = session.save_atomic(tmp_path / "saved.cdml", changed.revision)

    revisions = (published.published_snapshot.revision, published.snapshot.revision)
    assert ((tmp_path / "saved.cdml").read_text(), revisions) == (
        published.published_snapshot.cdml, (changed.revision, changed.revision),
    )
    assert published.snapshot.is_dirty is (not published.outcome.is_confirmed)


def test_recovery_export_never_changes_the_session_state(tmp_path: Path) -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    changed = session.submit(0, set_atom("N")).observation.snapshot
    exported = session.recovery_export(tmp_path / "recovery.cdml", changed.revision)
    current = session.snapshot()

    assert (exported.snapshot.revision, exported.snapshot.is_dirty) == (changed.revision, True)
    assert (current.revision, current.digest, current.is_dirty) == (
        changed.revision, changed.digest, True,
    )


def test_invalid_destination_keeps_its_structured_public_fields(tmp_path: Path) -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)

    with pytest.raises(ferrum_chem.InvalidDestinationError) as caught:
        session.save_atomic(tmp_path, 0)

    assert caught.value.path == str(tmp_path)
    assert caught.value.reason == "destination exists but is not a regular file"


def test_publication_errors_share_the_documented_shape(tmp_path: Path) -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    target = tmp_path / "missing-parent" / "saved.cdml"

    with pytest.raises(ferrum_chem.PublicationNotStartedError) as caught:
        session.save_atomic(target, 0)

    assert (caught.value.path, bool(caught.value.reason)) == (str(target), True)


def test_render_interaction_binding_uses_render_plan_authority() -> None:
    session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
    initial = session.snapshot()
    observation = session.observe_render_interaction_v1(
        initial.revision, initial.digest,
    )
    assert [root.identifier for root in observation.roots] == ["m"]
    selection = session.select_render_interaction_roots_v1(
        observation, None,
        ferrum_chem.RenderInteractionQueryV1.point(
            1.0, 2.0, ferrum_chem.RenderInteractionModifierV1.replace,
        ),
    )
    gesture = session.begin_render_interaction_translation_v1(
        selection, 1.0, 2.0, ferrum_chem.RenderInteractionSnapV1.free(),
    )
    preview = session.preview_render_interaction_translation_v1(gesture, 6.0, 0.0)
    committed = session.commit_render_interaction_translation_v1(gesture, preview)
    assert committed.changed is True
    assert committed.result.observation.snapshot.revision == 1
    assert session.undo(1).observation.snapshot.revision == 2

    unsupported = ferrum_chem.DocumentSession.load(
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="blocked"><atom id="a" name="C">'
        '<point x="1" y="2"/><ftext><b>rich</b></ftext>'
        '</atom></molecule></cdml>'
    )
    blocked_snapshot = unsupported.snapshot()
    blocked = unsupported.observe_render_interaction_v1(
        blocked_snapshot.revision, blocked_snapshot.digest,
    )
    assert blocked.roots == ()
    assert [(entry.identifier, entry.reason) for entry in blocked.exclusions] == [
        ("blocked", ferrum_chem.RenderInteractionExclusionReasonV1.unrenderable_depiction),
    ]
    with pytest.raises(ferrum_chem.RenderInteractionError) as blocked_error:
        unsupported.select_render_interaction_roots_v1(
            blocked, None,
            ferrum_chem.RenderInteractionQueryV1.root("blocked"),
        )
    assert blocked_error.value.category == ferrum_chem.RenderInteractionCategoryV1.unrenderable_depiction
    display_only = ferrum_chem.DocumentSession.load(
        '<cdml xmlns="urn:ferrum:cdml"><plus><point x="4" y="5"/></plus></cdml>',
    )
    display_snapshot = display_only.snapshot()
    display_observation = display_only.observe_render_interaction_v1(
        display_snapshot.revision, display_snapshot.digest,
    )
    assert display_observation.roots == ()
    assert display_observation.exclusions[0].reason == (
        ferrum_chem.RenderInteractionExclusionReasonV1.display_only
    )
    with pytest.raises(ferrum_chem.RenderInteractionError) as display_error:
        display_only.select_render_interaction_roots_v1(
            display_observation, None,
            ferrum_chem.RenderInteractionQueryV1.root(
                display_observation.exclusions[0].identifier,
            ),
        )
    assert display_error.value.category == ferrum_chem.RenderInteractionCategoryV1.display_only
    fragment_reference = ferrum_chem.DocumentSession.load(
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
        '<point x="0" y="0"/></atom><fragment><bond id="m"/>'
        '</fragment></molecule></cdml>',
    )
    fragment_snapshot = fragment_reference.snapshot()
    fragment_observation = fragment_reference.observe_render_interaction_v1(
        fragment_snapshot.revision, fragment_snapshot.digest,
    )
    assert [root.identifier for root in fragment_observation.roots] == ["m"]
    fragment_selection = fragment_reference.select_render_interaction_roots_v1(
        fragment_observation, None,
        ferrum_chem.RenderInteractionQueryV1.root("m"),
    )
    fragment_gesture = fragment_reference.begin_render_interaction_translation_v1(
        fragment_selection, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
    )
    fragment_preview = fragment_reference.preview_render_interaction_translation_v1(
        fragment_gesture, 3.0, 0.0,
    )
    assert fragment_reference.commit_render_interaction_translation_v1(
        fragment_gesture, fragment_preview,
    ).result.observation.snapshot.revision == 1
    foreign = ferrum_chem.DocumentSession.load(BOND_SOURCE)
    with pytest.raises(ferrum_chem.RenderInteractionError) as foreign_error:
        foreign.select_render_interaction_roots_v1(
            observation, None, ferrum_chem.RenderInteractionQueryV1.clear(),
        )
    assert foreign_error.value.category == ferrum_chem.RenderInteractionCategoryV1.foreign_session
    with pytest.raises(ferrum_chem.RevisionConflictError):
        session.select_render_interaction_roots_v1(
            observation, None, ferrum_chem.RenderInteractionQueryV1.clear(),
        )


def test_render_interaction_binding_moves_molecule_and_plus_atomically() -> None:
    session = ferrum_chem.DocumentSession.load(
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
        '<point x="0" y="0"/></atom></molecule><plus id="p">'
        '<point x="40" y="0"/></plus></cdml>',
    )
    initial = session.snapshot()
    observation = session.observe_render_interaction_v1(
        initial.revision, initial.digest,
    )
    assert [root.identifier for root in observation.roots] == ["m", "p"]
    molecule = session.select_render_interaction_roots_v1(
        observation, None,
        ferrum_chem.RenderInteractionQueryV1.point(
            0.0, 0.0, ferrum_chem.RenderInteractionModifierV1.replace,
        ),
    )
    mixed = session.select_render_interaction_roots_v1(
        observation, molecule,
        ferrum_chem.RenderInteractionQueryV1.point(
            40.0, 0.0, ferrum_chem.RenderInteractionModifierV1.toggle,
        ),
    )
    assert [root.identifier for root in mixed.roots] == ["m", "p"]
    gesture = session.begin_render_interaction_translation_v1(
        mixed, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
    )
    preview = session.preview_render_interaction_translation_v1(gesture, 7.0, 4.0)
    committed = session.commit_render_interaction_translation_v1(gesture, preview)
    assert (committed.changed, committed.result.observation.snapshot.revision) == (True, 1)
    assert session.undo(1).observation.snapshot.revision == 2


def test_render_interaction_binding_returns_toggled_roots_in_source_order() -> None:
    session = ferrum_chem.DocumentSession.load(
        '<cdml xmlns="urn:ferrum:cdml" version="26.08"><molecule id="m1"><atom id="a1" name="C">'
        '<point x="0" y="0"/></atom></molecule><molecule id="m2"><atom id="a2" '
        'name="O"><point x="40" y="0"/></atom></molecule></cdml>',
    )
    snapshot = session.snapshot()
    observation = session.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
    later = session.select_render_interaction_roots_v1(
        observation, None,
        ferrum_chem.RenderInteractionQueryV1.point(
            40.0, 0.0, ferrum_chem.RenderInteractionModifierV1.replace,
        ),
    )
    selection = session.select_render_interaction_roots_v1(
        observation, later,
        ferrum_chem.RenderInteractionQueryV1.point(
            0.0, 0.0, ferrum_chem.RenderInteractionModifierV1.toggle,
        ),
    )

    assert [root.identifier for root in selection.roots] == ["m1", "m2"]


def test_render_interaction_binding_captures_raw_or_view_hex_grid_snap() -> None:
    session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
    initial = session.snapshot()
    observation = session.observe_render_interaction_v1(initial.revision, initial.digest)
    selection = session.select_render_interaction_roots_v1(
        observation, None, ferrum_chem.RenderInteractionQueryV1.root("m"),
    )
    raw = session.begin_render_interaction_translation_v1(
        selection, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
    )
    grid = session.begin_render_interaction_translation_v1(
        selection, 0.0, 0.0,
        ferrum_chem.RenderInteractionSnapV1.with_grid_policy(
            ferrum_chem.RenderInteractionAxisV1.free,
            ferrum_chem.RenderInteractionGridSnapPolicyV1.view_hex_grid,
        ),
    )
    raw_preview = session.preview_render_interaction_translation_v1(raw, 38.0, 18.0)
    grid_preview = session.preview_render_interaction_translation_v1(grid, 38.0, 18.0)
    assert (raw_preview.dx, raw_preview.dy) == (38.0, 18.0)
    assert (grid_preview.dx, grid_preview.dy) != (raw_preview.dx, raw_preview.dy)


def test_presentation_creation_gesture_binding_owns_preview_and_canonical_arrow() -> None:
    session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    snapshot = session.snapshot()
    gesture = session.begin_presentation_creation_gesture_v1(
        snapshot.revision, snapshot.digest,
        ferrum_chem.PresentationGestureKindV1.straight_normal_arrow,
        0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
        ferrum_chem.PresentationGestureSnapPolicyV1(
            angle_increment_degrees=45, fixed_length_pt=20,
        ),
    )
    preview = session.preview_presentation_creation_gesture_v1(gesture, 8.0, 9.0)
    assert type(preview.plan) is ferrum_chem.PresentationRenderPlanV1
    assert len(preview.plan.roots) == 1
    assert len(preview.plan.roots[0].vector_operations) == 2
    assert preview.plan.roots[0].bounds.right > 14.0
    assert session.snapshot().revision == 0
    commit = session.commit_presentation_creation_gesture_v1(gesture, preview)
    assert commit.root.kind == ferrum_chem.PresentationGestureRootKindV1.arrow
    assert commit.root.identifier.startswith("ferrum-presentation-v1-")
    assert commit.result.observation.snapshot.revision == 1
    cdml = commit.result.observation.snapshot.cdml
    assert 'width="1.0"' in cdml
    assert 'color="#000000"' in cdml
    assert "cm" in cdml
    with pytest.raises(ferrum_chem.PresentationGestureError) as replayed:
        session.commit_presentation_creation_gesture_v1(gesture, preview)
    assert replayed.value.category == ferrum_chem.PresentationGestureCategoryV1.replayed_gesture
    assert replayed.value.recovery == ferrum_chem.PresentationGestureRecoveryV1.refresh_and_restart
    assert session.snapshot().cdml == cdml


def test_equilibrium_creation_binding_requires_kind_owned_style() -> None:
    session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    snapshot = session.snapshot()
    gesture = session.begin_presentation_creation_gesture_v1(
        snapshot.revision, snapshot.digest,
        ferrum_chem.PresentationGestureKindV1.straight_equilibrium_arrow,
        0.0, 0.0, None, ferrum_chem.PresentationGestureSnapPolicyV1(),
    )
    preview = session.preview_presentation_creation_gesture_v1(gesture, 40.0, 0.0)
    assert type(preview.plan) is ferrum_chem.PresentationRenderPlanV1
    assert len(preview.plan.roots[0].vector_operations) == 3
    with pytest.raises(ferrum_chem.PresentationGestureError) as invalid:
        session.begin_presentation_creation_gesture_v1(
            snapshot.revision, snapshot.digest,
            ferrum_chem.PresentationGestureKindV1.straight_equilibrium_arrow,
            0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
            ferrum_chem.PresentationGestureSnapPolicyV1(),
        )
    assert invalid.value.category == ferrum_chem.PresentationGestureCategoryV1.invalid_gesture_style
    assert session.snapshot().revision == 0


def test_presentation_creation_gesture_binding_rejects_bad_handles_and_geometry() -> None:
    first = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    second = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    first_snapshot = first.snapshot()
    second_snapshot = second.snapshot()
    first_gesture = first.begin_presentation_creation_gesture_v1(
        first_snapshot.revision, first_snapshot.digest,
        ferrum_chem.PresentationGestureKindV1.straight_normal_arrow,
        0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
        ferrum_chem.PresentationGestureSnapPolicyV1(),
    )
    second_gesture = second.begin_presentation_creation_gesture_v1(
        second_snapshot.revision, second_snapshot.digest,
        ferrum_chem.PresentationGestureKindV1.straight_normal_arrow,
        0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
        ferrum_chem.PresentationGestureSnapPolicyV1(),
    )
    preview = first.preview_presentation_creation_gesture_v1(first_gesture, 10.0, 0.0)
    with pytest.raises(ferrum_chem.PresentationGestureError) as foreign:
        second.preview_presentation_creation_gesture_v1(first_gesture, 10.0, 0.0)
    assert foreign.value.category == ferrum_chem.PresentationGestureCategoryV1.foreign_session
    with pytest.raises(ferrum_chem.PresentationGestureError) as mixed:
        second.commit_presentation_creation_gesture_v1(second_gesture, preview)
    assert mixed.value.category == ferrum_chem.PresentationGestureCategoryV1.foreign_session
    assert second.snapshot().revision == 0
    with pytest.raises(ferrum_chem.PresentationGestureError) as short:
        first.preview_presentation_creation_gesture_v1(first_gesture, 1.0, 0.0)
    assert short.value.category == ferrum_chem.PresentationGestureCategoryV1.below_minimum_length
    with pytest.raises(ferrum_chem.PresentationGestureError) as long:
        first.preview_presentation_creation_gesture_v1(first_gesture, 20_001.0, 0.0)
    assert long.value.category == ferrum_chem.PresentationGestureCategoryV1.exceeds_geometry_limit
    assert first.snapshot().revision == 0


def test_dedicated_plus_placement_facade_commits_standard_plus() -> None:
    """Only the renderer-backed Plus facade persists one canonical root."""
    session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'><standard font_size='18'/></cdml>")
    snapshot = session.snapshot()
    assert not hasattr(ferrum_chem.PresentationGestureKindV1, "plus")
    gesture = session.begin_plus_placement_gesture_v1(
        snapshot.revision, snapshot.digest, 72.0, 36.0,
    )
    preview = session.preview_plus_placement_gesture_v1(gesture)
    assert preview.overlay.text == "+"
    assert preview.overlay.font_size == 18.0
    assert session.snapshot().revision == 0
    commit = session.commit_plus_placement_gesture_v1(gesture, preview)
    assert commit.identifier.startswith("ferrum-presentation-v1-")
    assert '<plus' in commit.result.observation.snapshot.cdml
    plus = commit.result.observation.snapshot.cdml.split('<plus', 1)[1].split('</plus>', 1)[0]
    assert 'font_size' not in plus


def test_dedicated_plus_preview_equals_the_committed_renderer() -> None:
    """The dedicated facade publishes only exact current renderer facts."""
    session = ferrum_chem.DocumentSession.load(
        "<cdml xmlns='urn:ferrum:cdml'><standard font_size='18' line_color='#123456'/></cdml>",
    )
    snapshot = session.snapshot()
    gesture = session.begin_plus_placement_gesture_v1(
        snapshot.revision, snapshot.digest, 72.0, 36.0,
    )
    preview = session.preview_plus_placement_gesture_v1(gesture)
    assert preview.overlay.color == '123456'
    commit = session.commit_plus_placement_gesture_v1(gesture, preview)
    plus = commit.result.observation.projection.presentation_stack.roots[0].plus
    rendered = session.observe_render(1).plus_renders[0]
    assert (plus.font.size, plus.font.color) == (18.0, '#123456')
    assert preview.overlay.color == rendered.operation.paint
    assert (preview.overlay.left - 72.0, preview.overlay.top - 36.0) == pytest.approx((
        rendered.bounds.left, rendered.bounds.top,
    ))
    assert (preview.overlay.right - 72.0, preview.overlay.bottom - 36.0) == pytest.approx((
        rendered.bounds.right, rendered.bounds.bottom,
    ))


def test_dedicated_plus_facade_rejects_foreign_and_replayed_handles() -> None:
    """The Plus facade keeps both opaque halves bound to one session snapshot."""
    first = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    second = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    first_snapshot = first.snapshot()
    gesture = first.begin_plus_placement_gesture_v1(
        first_snapshot.revision, first_snapshot.digest, 72.0, 36.0,
    )
    preview = first.preview_plus_placement_gesture_v1(gesture)
    with pytest.raises(ferrum_chem.PresentationGestureError) as foreign:
        second.commit_plus_placement_gesture_v1(gesture, preview)
    assert foreign.value.category == ferrum_chem.PresentationGestureCategoryV1.foreign_session
    first.commit_plus_placement_gesture_v1(gesture, preview)
    with pytest.raises(ferrum_chem.PresentationGestureError) as replay:
        first.commit_plus_placement_gesture_v1(gesture, preview)
    assert replay.value.category == ferrum_chem.PresentationGestureCategoryV1.replayed_gesture


def test_presentation_creation_gesture_binding_rejects_bool_and_replay_without_mutation() -> None:
    for kwargs in ({"angle_increment_degrees": True}, {"fixed_length_pt": True}):
        with pytest.raises(ferrum_chem.PresentationGestureError) as invalid:
            ferrum_chem.PresentationGestureSnapPolicyV1(**kwargs)
        assert invalid.value.category == ferrum_chem.PresentationGestureCategoryV1.invalid_snap_policy
    session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    snapshot = session.snapshot()
    gesture = session.begin_presentation_creation_gesture_v1(
        snapshot.revision, snapshot.digest,
        ferrum_chem.PresentationGestureKindV1.straight_normal_arrow,
        0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
        ferrum_chem.PresentationGestureSnapPolicyV1(),
    )
    preview = session.preview_presentation_creation_gesture_v1(gesture, 10.0, 0.0)
    session.commit_presentation_creation_gesture_v1(gesture, preview)
    after = session.snapshot()
    with pytest.raises(ferrum_chem.PresentationGestureError) as replay:
        session.commit_presentation_creation_gesture_v1(gesture, preview)
    assert replay.value.category == ferrum_chem.PresentationGestureCategoryV1.replayed_gesture
    assert session.snapshot().revision == after.revision
    assert session.snapshot().cdml == after.cdml
