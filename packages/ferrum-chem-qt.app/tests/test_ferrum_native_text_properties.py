"""Semantic direct-root Text editing through the Rust-native tab."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

ferrum_chem = pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.canvas.items.ferrum_text_item
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_text_properties


#============================================
@pytest.fixture(scope="module")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return one offscreen application without importing the legacy host."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _select_text(tab: object) -> None:
	"""Select the one rendered Text through its actual scene item."""
	items = tuple(
		item for item in tab.view.scene().items()
		if type(item) is ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem
	)
	assert len(items) == 1
	items[0].setSelected(True)


#============================================
def test_native_text_edit_updates_rust_and_retains_durable_selection(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""One representable run/font edit commits and installs its replacement item."""
	del qapp
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		'<cdml><text id="t"><point x="10" y="20"/>'
		'<font size="12" color="#000"/><ftext>old</ftext></text></cdml>',
		"text.cdml",
	)
	try:
		_select_text(tab)
		assert tab.has_one_selected_text()
		model = (
			ferrum_qt.native.ferrum_native_text_properties.dialog_model_from_projection(
				tab.selected_text_projection(),
			)
		)
		changes = (
			ferrum_qt.native.ferrum_native_text_properties.property_changes_from_dialog(
				model,
				(("H", ()), ("2", ("sub",)), ("O", ())),
				18,
				"#123456",
			)
		)
		result = tab.apply_selected_text_properties(changes)
		assert result.observation.snapshot.revision == 1
		assert tab.has_one_selected_text()
		updated = tab.selected_text_projection()
		assert (updated.font.size, updated.font.color) == (18.0, "#123456")
		assert [(run.text, run.styles) for run in updated.runs] == [
			("H", ()), ("2", ("subscript",)), ("O", ()),
		]
		item = next(
			item for item in tab.view.scene().items()
			if type(item) is ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem
		)
		assert item.isSelected()
	finally:
		tab.dispose()


#============================================
def test_native_text_adapter_rejects_unrenderable_facts_without_mutation() -> None:
	"""Unsupported face intent never reaches a native document operation."""
	session = ferrum_chem.DocumentSession.load(
		'<cdml><text id="t"><point x="0" y="0"/><font family="Arial"/>'
		'<ftext>text</ftext></text></cdml>',
	)
	text = session.observe(0).projection.presentation_stack.roots[0].text
	with pytest.raises(ValueError, match="cannot preserve"):
		ferrum_qt.native.ferrum_native_text_properties.dialog_model_from_projection(text)
	assert session.snapshot().revision == 0

	supported_session = ferrum_chem.DocumentSession.load(
		'<cdml><text id="t"><point x="0" y="0"/><ftext>text</ftext></text></cdml>',
	)
	supported = supported_session.observe(0).projection.presentation_stack.roots[0].text
	model = ferrum_qt.native.ferrum_native_text_properties.dialog_model_from_projection(supported)
	with pytest.raises(ValueError, match="baseline, subscript, and superscript"):
		ferrum_qt.native.ferrum_native_text_properties.property_changes_from_dialog(
			model, (("text", ("b",)),), 12, "#000000",
		)
	assert supported_session.snapshot().revision == 0
