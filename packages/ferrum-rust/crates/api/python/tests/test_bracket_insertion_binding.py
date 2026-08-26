"""Installed-extension behavior for Rust-owned bracket-pair insertion."""

# PIP3 modules
import ferrum_chem
import pytest


def test_rectangular_bracket_exposes_pair_identity_geometry_and_history() -> None:
	"""Commit one backend-styled pair and retain its durable relationship."""
	session = ferrum_chem.DocumentSession.load(
		'<cdml xmlns="urn:ferrum:cdml"><standard line_width="2" line_color="#123"/></cdml>',
	)
	bounds = ferrum_chem.DocumentBracketBoundsV1(0.0, 10.0, 100.0, 210.0)
	prepared = session.prepare_create_bracket_v1(
		0, ferrum_chem.DocumentBracketStyleV1.rectangular, bounds,
	)
	assert prepared.pair_identifier
	assert prepared.left_identifier
	assert prepared.right_identifier
	result = session.commit_create_bracket(0, prepared)
	stack = result.observation.projection.presentation_stack
	pair, = stack.bracket_pairs
	assert tuple(pair.members) == tuple(
		root.polyline.target.document_object_id for root in stack.entries
	)
	assert pair.style is ferrum_chem.DocumentBracketStyleV1.rectangular
	undone_stack = session.undo(1).observation.projection.presentation_stack
	assert not undone_stack.bracket_pairs
	redone_stack = session.redo(2).observation.projection.presentation_stack
	assert redone_stack.bracket_pairs[0].members == pair.members


def test_round_projects_exact_spline_sides_and_bad_intent_is_atomic() -> None:
	"""Reject malformed bounds and project only the valid paired spline family."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	before = session.observe(0).snapshot
	for bounds in (
		(0.0, 0.0, float("inf"), 1.0),
		(1.0, 0.0, 1.0, 1.0),
		(2.0, 0.0, 1.0, 1.0),
	):
		with pytest.raises(ferrum_chem.OperationValidationError):
			ferrum_chem.DocumentBracketBoundsV1(*bounds)
	assert session.observe(0).snapshot.digest == before.digest
	bounds = ferrum_chem.DocumentBracketBoundsV1(0.0, 0.0, 80.0, 120.0)
	prepared = session.prepare_create_bracket_v1(
		0, ferrum_chem.DocumentBracketStyleV1.round, bounds,
	)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.commit_create_bracket(1, prepared)
	result = session.commit_create_bracket(0, prepared)
	stack = result.observation.projection.presentation_stack
	assert stack.bracket_pairs[0].style is ferrum_chem.DocumentBracketStyleV1.round
	assert [root.kind for root in stack.entries] == ["round_bracket", "round_bracket"]
	assert [len(root.polyline.path.points) for root in stack.entries] == [4, 4]
	assert stack.issues == []


def test_pair_properties_are_closed_atomic_and_update_both_members() -> None:
	"""Apply one common pair patch and reject malformed Python intent first."""
	class TupleSubclass(tuple):
		pass

	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	prepared = session.prepare_create_bracket_v1(
		0,
		ferrum_chem.DocumentBracketStyleV1.rectangular,
		ferrum_chem.DocumentBracketBoundsV1(0.0, 0.0, 20.0, 30.0),
	)
	session.commit_create_bracket(0, prepared)
	members = tuple(session.observe(1).projection.presentation_stack.bracket_pairs[0].members)
	before = session.observe(1).snapshot
	change = ferrum_chem.DocumentBracketPropertyChangeV1.line_color("#123456")
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_bracket_properties(
			members, TupleSubclass((change,)),
		)
	assert session.observe(1).snapshot.digest == before.digest

	operation = ferrum_chem.DocumentOperationV1.set_bracket_properties(
		members,
		(
			ferrum_chem.DocumentBracketPropertyChangeV1.line_width(2.5),
			change,
		),
	)
	result = session.apply_document_operation_v1(1, operation)
	pair = result.observation.projection.presentation_stack.bracket_pairs[0]
	assert (pair.line_width, pair.line_color) == (2.5, "#123456")
	assert [root.polyline.stroke.width for root in
			result.observation.projection.presentation_stack.entries] == [2.5, 2.5]
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.apply_document_operation_v1(1, operation)
	assert session.observe(2).snapshot.digest == result.observation.snapshot.digest


def test_live_pair_properties_require_current_durable_members_and_fence() -> None:
	"""Apply bracket properties through the fenced durable live-session adapter."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	prepared = session.prepare_create_bracket_v1(
		0,
		ferrum_chem.DocumentBracketStyleV1.rectangular,
		ferrum_chem.DocumentBracketBoundsV1(0.0, 0.0, 20.0, 30.0),
	)
	created = session.commit_create_bracket(0, prepared)
	members = tuple(
		root.polyline.target.document_object_id
		for root in created.observation.projection.presentation_stack.entries
	)
	assert len(members) == 2
	before = session.observe(1).snapshot
	change = ferrum_chem.DocumentBracketPropertyChangeV1.line_color("#123456")
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.set_bracket_pair_properties_v1(
			1, before.digest, (members[1], members[0]), (change,),
		)
	assert session.observe(1).snapshot.digest == before.digest
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.set_bracket_pair_properties_v1(
			0, before.digest, members, (change,),
		)
	assert session.observe(1).snapshot.digest == before.digest
	result = session.set_bracket_pair_properties_v1(
		1, before.digest, members, (change,),
	)
	assert result.observation.projection.presentation_stack.bracket_pairs[0].line_color == "#123456"
