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
		'<bond id="front" start="a" end="b" haworth_position="front"/>'
		'<bond id="back" start="b" end="a" haworth_position="back"/>'
		'<bond id="malformed" start="a" end="b" haworth_position="side"/></molecule></cdml>'
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
		session.apply_document_operation_v1(
			1, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order,)),
		)
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
		session.apply_document_operation_v1(
			0, ferrum_chem.DocumentOperationV1.set_bond_properties("missing", (order,)),
		)
	unsupported = ferrum_chem.DocumentSession.load(
		BOND_PROPERTIES_SOURCE.replace('type="n1"', 'type="l1"'),
	)
	unsupported_before = unsupported.snapshot()
	with pytest.raises(ferrum_chem.OperationValidationError):
		unsupported.apply_document_operation_v1(
			0, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order,)),
		)
	unsupported_after = unsupported.snapshot()
	assert (unsupported_after.revision, unsupported_after.digest) == (
		unsupported_before.revision, unsupported_before.digest,
	)
	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)

	no_change = session.apply_document_operation_v1(
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
	expected_remaining = session.observe(0).projection.molecules[0].atoms[0].document_object_id
	deleted = session.apply_document_operation_v1(
		0, ferrum_chem.DocumentOperationV1.delete_atom("b"),
	).observation

	assert tuple(
		atom.document_object_id for atom in deleted.projection.molecules[0].atoms
	) == (expected_remaining,)
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
	deleted = session.apply_document_operation_v1(
		0, ferrum_chem.DocumentOperationV1.delete_bond("ab"),
	).observation

	assert len(deleted.projection.molecules[0].atoms) == 2
	assert not deleted.projection.molecules[0].bonds
	assert len(session.undo(1).observation.projection.molecules[0].bonds) == 1
	assert not session.redo(2).observation.projection.molecules[0].bonds
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
		session.apply_document_operation_v1(3, ferrum_chem.DocumentOperationV1.delete_bond("missing"))
	assert caught.value.category == "unknown_document_object"
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
	no_change = session.apply_document_operation_v1(
		0,
		ferrum_chem.DocumentOperationV1.set_bond_order(
			"ab", ferrum_chem.DocumentBondOrderV1.single,
		),
	).observation
	assert no_change.snapshot.revision == 0

	changed = session.apply_document_operation_v1(
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
		session.apply_document_operation_v1(
			3,
			ferrum_chem.DocumentOperationV1.set_bond_order(
				"missing", ferrum_chem.DocumentBondOrderV1.triple,
			),
		)
	assert caught.value.category == "unknown_document_object"
	assert session.snapshot().revision == 3


def test_operation_validation_errors_are_specific_and_structured() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)

	with pytest.raises(ferrum_chem.InvalidAtomElementError):
		session.apply_document_operation_v1(0, set_atom("2"))
	with pytest.raises(ferrum_chem.InvalidAtomElementError):
		session.apply_document_operation_v1(0, set_atom("Xx"))
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
		session.apply_document_operation_v1(
			0,
			ferrum_chem.DocumentOperationV1.set_atom_element("missing", "N"),
		)

	assert caught.value.category == "unknown_document_object"
	assert session.snapshot().revision == 0


def test_generic_atom_and_bond_authoring_expose_committed_typed_outcomes() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	molecule_object_id = session.observe(0).projection.molecules[0].document_object_id
	atom_operation = ferrum_chem.DocumentOperationV1.create_atom_v1(
		molecule_object_id, "O", 3.0, 4.0, 0.0,
	)
	atom_result = session.apply_document_operation_v1(0, atom_operation)
	assert atom_result.outcome.kind == "atom_created_v1"
	assert atom_result.outcome.atom_created is not None
	assert atom_result.observation.snapshot.revision == 1
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.apply_document_operation_v1(0, atom_operation)
	assert session.snapshot().revision == 1

	bonds = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	start, end = (
		atom.document_object_id for atom in bonds.observe(0).projection.molecules[0].atoms
	)
	bond_operation = ferrum_chem.DocumentOperationV1.create_bond_v1(
		start, end, ferrum_chem.DocumentBondPresentationV1.solid_wedge,
	)
	bond_result = bonds.apply_document_operation_v1(0, bond_operation)
	assert bond_result.outcome.kind == "bond_created_v1"
	assert bond_result.outcome.bond_created is not None
	assert bond_result.observation.snapshot.revision == 1


def test_live_bond_authoring_requires_durable_endpoints_and_commits_one_transition() -> None:
	"""The live authoring boundary owns durable target and revision fencing."""
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.resolve_create_bond_v1(
			0, "a", "b", ferrum_chem.DocumentBondPresentationV1.normal_single,
		)
	start, end = (
		atom.document_object_id for atom in session.observe(0).projection.molecules[0].atoms
	)
	request = session.resolve_create_bond_v1(
		0, start, end, ferrum_chem.DocumentBondPresentationV1.normal_single,
	)
	prepared = session.prepare_session_operation_transition_v1(request)
	result = session.commit_session_operation_transition_v1(prepared)

	assert result.outcome.kind == "bond_created_v1"
	assert result.observation.snapshot.revision == 1


def test_confirmed_save_or_unconfirmed_outcome_preserves_exact_contract(
	tmp_path: Path,
) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.apply_document_operation_v1(0, set_atom("N")).observation.snapshot
	published = session.save_atomic(tmp_path / "saved.cdml", changed.revision)

	revisions = (published.published_snapshot.revision, published.snapshot.revision)
	assert ((tmp_path / "saved.cdml").read_text(), revisions) == (
		published.published_snapshot.cdml, (changed.revision, changed.revision),
	)
	assert published.snapshot.is_dirty is (not published.outcome.is_confirmed)


def test_recovery_export_never_changes_the_session_state(tmp_path: Path) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.apply_document_operation_v1(0, set_atom("N")).observation.snapshot
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
	root_ids = [root.document_object_id for root in observation.roots]
	assert len(root_ids) == 1
	selection = session.select_render_interaction_roots_v1(
		observation, None,
		ferrum_chem.RenderInteractionQueryV1.point(
			1.0, 2.0, ferrum_chem.RenderInteractionModifierV1.replace,
		),
	)
	gesture = session.begin_render_interaction_translation_v1(
		selection, 1.0, 2.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	committed = session.commit_render_interaction_translation_v1(gesture, 11.0, 7.0)
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
	assert len(blocked.exclusions) == 1
	assert blocked.exclusions[0].reason == (
		ferrum_chem.RenderInteractionExclusionReasonV1.unrenderable_depiction
	)
	with pytest.raises(ferrum_chem.RenderInteractionError) as blocked_error:
		unsupported.select_render_interaction_roots_v1(
			blocked, None,
			ferrum_chem.RenderInteractionQueryV1.root(
				blocked.exclusions[0].document_object_id,
			),
		)
	assert blocked_error.value.category == ferrum_chem.RenderInteractionCategoryV1.unrenderable_depiction
	display_only = ferrum_chem.DocumentSession.load(
		'<cdml xmlns="urn:ferrum:cdml"><polyline id="short">'
		'<point x="4" y="5"/></polyline></cdml>',
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
				display_observation.exclusions[0].document_object_id,
			),
		)
	assert display_error.value.category == ferrum_chem.RenderInteractionCategoryV1.display_only
	fragment_reference = ferrum_chem.DocumentSession.load(
		'<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
		'<point x="0" y="0"/></atom><fragment id="f"><bond id="m"/>'
		'</fragment></molecule></cdml>',
	)
	fragment_snapshot = fragment_reference.snapshot()
	fragment_observation = fragment_reference.observe_render_interaction_v1(
		fragment_snapshot.revision, fragment_snapshot.digest,
	)
	assert len(fragment_observation.roots) == 1
	fragment_selection = fragment_reference.select_render_interaction_roots_v1(
		fragment_observation, None,
		ferrum_chem.RenderInteractionQueryV1.root(
			fragment_observation.roots[0].document_object_id,
		),
	)
	fragment_gesture = fragment_reference.begin_render_interaction_translation_v1(
		fragment_selection, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	assert fragment_reference.commit_render_interaction_translation_v1(
		fragment_gesture, 3.0, 0.0,
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


def test_render_interaction_translation_rejects_stale_gesture_without_history_mutation() -> None:
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	initial = session.snapshot()
	observation = session.observe_render_interaction_v1(
		initial.revision, initial.digest,
	)
	selection = session.select_render_interaction_roots_v1(
		observation,
		None,
		ferrum_chem.RenderInteractionQueryV1.root(observation.roots[0].document_object_id),
	)
	gesture = session.begin_render_interaction_translation_v1(
		selection, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	intervening = session.apply_document_operation_v1(0, set_atom("N")).observation.snapshot

	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.commit_render_interaction_translation_v1(gesture, 6.0, 0.0)

	after_rejection = session.snapshot()
	assert (after_rejection.revision, after_rejection.digest) == (
		intervening.revision, intervening.digest,
	)
	undone = session.undo(intervening.revision).observation.snapshot
	assert (undone.revision, undone.digest) == (intervening.revision + 1, initial.digest)


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
	assert len(observation.roots) == 2
	initial_projection = session.observe(initial.revision).projection
	initial_position = initial_projection.molecules[0].atoms[0].position
	initial_plus = initial_projection.presentation_stack.entries[0].plus.anchor
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
	assert [root.document_object_id for root in mixed.roots] == [
		root.document_object_id for root in observation.roots
	]
	press = (10.0, 20.0)
	preview_point = (13.0, 22.0)
	release = (17.0, 24.0)
	gesture = session.begin_render_interaction_translation_v1(
		mixed, *press, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	preview = session.preview_render_interaction_translation_v1(gesture, *preview_point)
	assert (preview.dx, preview.dy) == (3.0, 2.0)
	committed = session.commit_render_interaction_translation_v1(gesture, *release)
	assert (committed.changed, committed.result.observation.snapshot.revision) == (True, 1)
	projection = committed.result.observation.projection
	position = projection.molecules[0].atoms[0].position
	plus = projection.presentation_stack.entries[0].plus.anchor
	displacement = (release[0] - press[0], release[1] - press[1])
	assert (position.x, position.y) == pytest.approx((
		initial_position.x + displacement[0], initial_position.y + displacement[1],
	))
	assert (plus.x, plus.y) == pytest.approx((
		initial_plus.x + displacement[0], initial_plus.y + displacement[1],
	))
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

	assert [root.document_object_id for root in selection.roots] == [
		root.document_object_id for root in observation.roots
	]


def test_render_interaction_binding_captures_raw_or_view_hex_grid_snap() -> None:
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	initial = session.snapshot()
	observation = session.observe_render_interaction_v1(initial.revision, initial.digest)
	selection = session.select_render_interaction_roots_v1(
		observation,
		None,
		ferrum_chem.RenderInteractionQueryV1.root(observation.roots[0].document_object_id),
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


def test_presentation_creation_gesture_binding_resolves_normal_arrow_generically() -> None:
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
	assert type(preview.plan) is ferrum_chem.PresentationPreviewRenderPlanV1
	assert preview.plan.schema == "ferrum-presentation-preview-render-plan-v1"
	assert len(preview.plan.roots) == 1
	assert len(preview.plan.roots[0].vector_operations) == 2
	assert preview.plan.roots[0].bounds.right > 14.0
	assert session.snapshot().revision == 0
	request = session.resolve_presentation_creation_gesture_v1(gesture, preview)
	prepared = session.prepare_session_operation_transition_v1(request)
	result = session.commit_session_operation_transition_v1(prepared)
	assert result.outcome.kind == "created_presentation_root_v1"
	assert result.outcome.created_presentation_root.kind == (
		ferrum_chem.CreatedPresentationRootKindV1.straight_normal_arrow
	)
	assert result.outcome.created_presentation_root.document_object_id.startswith(
		"ferrum-document-object-v1/",
	)
	assert result.observation.snapshot.revision == 1
	cdml = result.observation.snapshot.cdml
	assert 'width="1.0"' in cdml
	assert 'color="#000000"' in cdml


def test_equilibrium_creation_binding_requires_kind_owned_style() -> None:
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_creation_gesture_v1(
		snapshot.revision, snapshot.digest,
		ferrum_chem.PresentationGestureKindV1.straight_equilibrium_arrow,
		0.0, 0.0, None, ferrum_chem.PresentationGestureSnapPolicyV1(),
	)
	preview = session.preview_presentation_creation_gesture_v1(gesture, 40.0, 0.0)
	assert type(preview.plan) is ferrum_chem.PresentationPreviewRenderPlanV1
	assert preview.plan.schema == "ferrum-presentation-preview-render-plan-v1"
	assert len(preview.plan.roots[0].vector_operations) == 3
	request = session.resolve_presentation_creation_gesture_v1(gesture, preview)
	prepared = session.prepare_session_operation_transition_v1(request)
	result = session.commit_session_operation_transition_v1(prepared)
	assert result.outcome.created_presentation_root.kind == (
		ferrum_chem.CreatedPresentationRootKindV1.straight_equilibrium_arrow
	)
	assert result.observation.snapshot.revision == 1


def test_presentation_creation_gesture_binding_resolves_plus_generically() -> None:
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'><standard font_size='18'/></cdml>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_creation_gesture_v1(
		snapshot.revision, snapshot.digest, ferrum_chem.PresentationGestureKindV1.plus,
		72.0, 36.0, None, ferrum_chem.PresentationGestureSnapPolicyV1(),
	)
	preview = session.preview_presentation_creation_gesture_v1(gesture, 72.0, 36.0)
	assert type(preview.plan) is ferrum_chem.PresentationPreviewRenderPlanV1
	assert preview.plan.schema == "ferrum-presentation-preview-render-plan-v1"
	assert session.snapshot().revision == 0
	request = session.resolve_presentation_creation_gesture_v1(gesture, preview)
	prepared = session.prepare_session_operation_transition_v1(request)
	result = session.commit_session_operation_transition_v1(prepared)
	assert result.outcome.created_presentation_root.kind == ferrum_chem.CreatedPresentationRootKindV1.plus
	assert '<plus' in result.observation.snapshot.cdml


def test_presentation_creation_gesture_binding_rejects_boolean_snap_values() -> None:
	for kwargs in ({"angle_increment_degrees": True}, {"fixed_length_pt": True}):
		with pytest.raises(ferrum_chem.PresentationGestureError) as invalid:
			ferrum_chem.PresentationGestureSnapPolicyV1(**kwargs)
		assert invalid.value.category == ferrum_chem.PresentationGestureCategoryV1.invalid_snap_policy
