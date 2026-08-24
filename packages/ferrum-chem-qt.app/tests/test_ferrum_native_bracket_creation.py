"""Semantic bracket-pair creation through the Ferrum tab."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

ferrum_chem = pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.geometric_properties as native_geometric_properties
import ferrum_qt.canvas.ferrum_presentation_projection


#============================================
@pytest.fixture(scope="module")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return one offscreen application without importing the legacy host."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def test_native_rectangular_bracket_uses_pair_facts_selection_and_history(
		qapp: object,
		) -> None:
	"""Create one pair and select both durable sides after authoritative render."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		'<cdml xmlns="urn:ferrum:cdml"><standard line_width="2" line_color="#123456"/></cdml>',
		"bracket.cdml",
	)
	try:
		result = tab.create_rectangular_bracket(0.0, 10.0, 100.0, 210.0)
		stack = result.observation.projection.presentation_stack
		assert len(stack.bracket_pairs) == 1
		pair = stack.bracket_pairs[0]
		assert pair.style is ferrum_chem.DocumentBracketStyleV1.rectangular
		assert (pair.line_width, pair.line_color) == (2.0, "#123456")
		assert [len(root.polyline.path.points) for root in stack.roots] == [4, 4]
		selected = tab._controller.projection.selected_durable_targets()
		assert [(target.kind, target.identifier) for target in selected] == [
			("polyline", root.polyline.target.id) for root in stack.roots
		]
		_pair, model = tab.selected_bracket_pair_projection()
		changes = native_geometric_properties.bracket_property_changes_from_dialog((
			("line_width", 2.5),
			("line_color", "#445566"),
		))
		updated = tab.apply_selected_bracket_properties(
			model.pair_id, model.member_target_ids, changes,
		)
		updated_stack = updated.observation.projection.presentation_stack
		assert (
			updated_stack.bracket_pairs[0].line_width,
			updated_stack.bracket_pairs[0].line_color,
		) == (2.5, "#445566")
		assert [root.polyline.stroke.width for root in updated_stack.roots] == [2.5, 2.5]
		assert len(tab._controller.projection.selected_durable_targets()) == 2
		tab.undo()
		assert tab._document_observation.projection.presentation_stack.bracket_pairs[0].line_width == 2.0
		tab.redo()
		assert tab._document_observation.projection.presentation_stack.bracket_pairs[0].line_width == 2.5
	finally:
		tab.dispose()


#============================================
def test_native_round_pair_uses_rust_issued_cubic_paths_without_fallback(
		qapp: object,
		) -> None:
	"""Create a round pair and retain both renderer-plan targets as selection."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "round-bracket.cdml",
	)
	try:
		result = tab.create_round_bracket(0.0, 0.0, 20.0, 20.0)
		stack = result.observation.projection.presentation_stack
		assert len(stack.bracket_pairs) == 1
		assert stack.bracket_pairs[0].style is ferrum_chem.DocumentBracketStyleV1.round
		assert [root.kind for root in stack.roots] == [
			"round_bracket", "round_bracket",
		]
		assert stack.issues == []
		assert len(tab._controller.projection.selected_durable_targets()) == 2
	finally:
		tab.dispose()
