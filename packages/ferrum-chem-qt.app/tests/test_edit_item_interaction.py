"""Focused behavior for direct Edit-mode item editor dispatch."""

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.actions.object_actions
import ferrum_qt.canvas.items.text_item
import ferrum_qt.modes.edit_item_interaction


#============================================
def test_text_double_click_replaces_scene_selection_then_uses_public_action(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Direct Text editing re-resolves one selected durable root publicly."""
	scene = PySide6.QtWidgets.QGraphicsScene()
	previous = PySide6.QtWidgets.QGraphicsSimpleTextItem("previous")
	text = ferrum_qt.canvas.items.text_item.TextItem("annotation")
	scene.addItem(previous)
	scene.addItem(text)
	previous.setSelected(True)
	window = object()
	observed = []
	monkeypatch.setattr(
		ferrum_qt.actions.object_actions, "edit_selected_text",
		lambda received: observed.append(received),
	)

	ferrum_qt.modes.edit_item_interaction.open_item_editor(
		text, lambda _item: None, lambda _item: None, scene, window,
	)

	assert scene.selectedItems() == [text] and observed == [window]
