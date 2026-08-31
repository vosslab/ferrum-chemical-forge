"""Installed-extension checks for private document drawing-standard editing."""

import pytest

import ferrum_chem


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><standard line_width="1" '
	'font_size="12" font_family="Atkinson Hyperlegible Next" line_color="#000" '
	'area_color="" v:keep="yes"><bond width="6" wedge-width="5" '
	'double-ratio="0.75"><v:keep/></bond><atom show_hydrogens="0"/>'
	'</standard><molecule id="m"><atom id="a" name="C">'
	'<point x="0" y="0"/></atom></molecule></cdml>'
)


def _rendered_changes() -> tuple[object, ...]:
	"""Return the exact defaults consumed by the current renderer."""
	change = ferrum_chem.DocumentDrawingStandardPropertyChangeV1
	return (
		change.line_width(2.5),
		change.font_size(18),
		change.line_color("#AbC"),
		change.area_color("#123456"),
		change.bond_width(7.5),
		change.wedge_width(8.5),
		change.show_hydrogens(True),
	)


def test_private_standard_binding_commits_history_and_reopens_exact_facts() -> None:
	"""Renderer-consumed defaults remain Rust-owned across mutation and history."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.observe(0)
	operation = ferrum_chem.DocumentOperationV1.set_drawing_standard(_rendered_changes())
	changed = session.apply_document_operation_v1(0, operation)
	standard = changed.observation.projection.drawing_standard

	assert before.projection.drawing_standard.line_width == 1.0
	assert standard.line_width == 2.5
	assert standard.font_size == 18.0
	assert standard.font_family == "Atkinson Hyperlegible Next"
	assert standard.line_color == "#aabbcc"
	assert standard.area_color == "#123456"
	assert standard.bond_width == 7.5
	assert standard.wedge_width == 8.5
	assert standard.double_ratio == 0.75
	assert standard.show_hydrogens is True
	assert 'v:keep="yes"' in changed.observation.snapshot.cdml
	assert "<v:keep/>" in changed.observation.snapshot.cdml
	assert session.undo(1).observation.projection.drawing_standard.line_width == 1.0
	reopened_source = session.redo(2).observation.snapshot.cdml
	reopened = ferrum_chem.DocumentSession.load(reopened_source).observe(0)
	assert reopened.projection.drawing_standard.double_ratio == 0.75


def test_private_standard_binding_contains_invalid_python_values() -> None:
	"""Exact types, ranges, duplicate fields, and UTF-8 failures stay actionable."""
	change = ferrum_chem.DocumentDrawingStandardPropertyChangeV1
	invalid_factories = (
		(lambda: change.line_width(True), "plain number"),
		(lambda: change.font_size(3), "from 4 to 144"),
		(lambda: change.line_color("red"), "#rgb"),
		(lambda: change.line_color(chr(0xD800)), "valid UTF-8"),
	)
	for factory, message in invalid_factories:
		with pytest.raises(ferrum_chem.OperationValidationError, match=message):
			factory()

	one = change.line_width(1.0)
	with pytest.raises(ferrum_chem.OperationValidationError, match="repeated"):
		ferrum_chem.DocumentOperationV1.set_drawing_standard((one, one))
	with pytest.raises(TypeError, match="tuple"):
		ferrum_chem.DocumentOperationV1.set_drawing_standard([one])
