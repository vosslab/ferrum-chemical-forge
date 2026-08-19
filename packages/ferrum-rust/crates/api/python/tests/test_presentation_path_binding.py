"""Installed-extension contract for Rust-owned presentation path lowering."""

# PIP3 modules
import ferrum_chem
import pytest


def _round_root() -> object:
	"""Return one accepted direct-root round bracket from a real observation."""
	session = ferrum_chem.DocumentSession.load("<cdml/>")
	prepared = session.prepare_create_bracket_v1(
		0, ferrum_chem.DocumentBracketStyleV1.round,
		ferrum_chem.DocumentBracketBoundsV1(0.0, 0.0, 20.0, 40.0),
	)
	result = session.commit_create_bracket(0, prepared)
	return result.observation.projection.presentation_stack.roots[0]


def test_round_bracket_lowering_emits_only_frozen_replay_commands() -> None:
	"""The PyO3 seam returns an immutable MoveTo/CubicTo path, not Qt policy."""
	path = ferrum_chem.lower_round_bracket_presentation_path_v1(_round_root())
	assert path.kind == "authored_spline"
	assert tuple(command.kind for command in path.commands) == ("move_to", "cubic_to")
	assert path.commands[0].point is not None
	assert path.commands[1].control_1 is not None
	assert path.commands[1].control_2 is not None
	assert path.commands[1].point is not None
	with pytest.raises(AttributeError):
		path.kind = "polyline"


def test_path_lowering_refuses_an_ordinary_document_polyline() -> None:
	"""Normal spline admission stays owned by document projection, not this seam."""
	session = ferrum_chem.DocumentSession.load(
		"<cdml><polyline><point x=\"0\" y=\"0\"/>"
		"<point x=\"1\" y=\"1\"/></polyline></cdml>",
	)
	root = session.observe(0).projection.presentation_stack.roots[0]
	with pytest.raises(ferrum_chem.PresentationPathError):
		ferrum_chem.lower_round_bracket_presentation_path_v1(root)
