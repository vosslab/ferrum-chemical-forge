"""Installed-extension behavior for Rust-owned bracket-pair insertion."""

# PIP3 modules
import ferrum_chem
import pytest


def test_rectangular_bracket_exposes_pair_identity_geometry_and_history() -> None:
	"""Commit one backend-styled pair and retain its durable relationship."""
	session = ferrum_chem.DocumentSession.load(
		'<cdml><standard line_width="2" line_color="#123"/></cdml>',
	)
	bounds = ferrum_chem.DocumentBracketBoundsV1(0.0, 10.0, 100.0, 210.0)
	prepared = session.prepare_create_bracket_v1(
		0, ferrum_chem.DocumentBracketStyleV1.rectangular, bounds,
	)
	assert (
		prepared.pair_identifier,
		prepared.left_identifier,
		prepared.right_identifier,
	) == (
		"ferrum-presentation-v1-0",
		"ferrum-presentation-v1-0",
		"ferrum-presentation-v1-1",
	)
	result = session.commit_create_bracket(0, prepared)
	stack = result.observation.projection.presentation_stack
	assert len(stack.roots) == 2
	assert [root.kind for root in stack.roots] == ["polyline", "polyline"]
	assert [len(root.polyline.path.points) for root in stack.roots] == [4, 4]
	assert len(stack.bracket_pairs) == 1
	pair = stack.bracket_pairs[0]
	assert pair.pair_id == prepared.pair_identifier
	assert pair.member_ids == [prepared.left_identifier, prepared.right_identifier]
	assert pair.style is ferrum_chem.DocumentBracketStyleV1.rectangular
	assert (pair.line_width, pair.line_color) == (2.0, "#112233")
	assert session.undo(1).observation.projection.presentation_stack.bracket_pairs == []
	assert len(session.redo(2).observation.projection.presentation_stack.bracket_pairs) == 1


def test_round_projects_exact_spline_sides_and_bad_intent_is_atomic() -> None:
	"""Reject malformed bounds and project only the valid paired spline family."""
	session = ferrum_chem.DocumentSession.load("<cdml/>")
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
	assert [root.kind for root in stack.roots] == ["round_bracket", "round_bracket"]
	assert [len(root.polyline.path.points) for root in stack.roots] == [4, 4]
	assert stack.issues == []


def test_pair_properties_are_closed_atomic_and_update_both_members() -> None:
	"""Apply one common pair patch and reject malformed Python intent first."""
	class TupleSubclass(tuple):
		pass

	session = ferrum_chem.DocumentSession.load("<cdml/>")
	prepared = session.prepare_create_bracket_v1(
		0,
		ferrum_chem.DocumentBracketStyleV1.rectangular,
		ferrum_chem.DocumentBracketBoundsV1(0.0, 0.0, 20.0, 30.0),
	)
	session.commit_create_bracket(0, prepared)
	before = session.observe(1).snapshot
	change = ferrum_chem.DocumentBracketPropertyChangeV1.line_color("#123456")
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_bracket_properties(
			prepared.pair_identifier, TupleSubclass((change,)),
		)
	assert session.observe(1).snapshot.digest == before.digest

	operation = ferrum_chem.DocumentOperationV1.set_bracket_properties(
		prepared.pair_identifier,
		(
			ferrum_chem.DocumentBracketPropertyChangeV1.line_width(2.5),
			change,
		),
	)
	result = session.submit(1, operation)
	pair = result.observation.projection.presentation_stack.bracket_pairs[0]
	assert (pair.line_width, pair.line_color) == (2.5, "#123456")
	assert [root.polyline.stroke.width for root in
			result.observation.projection.presentation_stack.roots] == [2.5, 2.5]
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.submit(1, operation)
	assert session.observe(2).snapshot.digest == result.observation.snapshot.digest
