"""Semantic Plus property editing through the Ferrum tab."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

ferrum_chem = pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.plus_properties


#============================================
@pytest.fixture(scope="module")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return one offscreen application without importing the legacy host."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _select_plus(tab: object) -> None:
	"""Select the one rendered Plus through its actual scene item."""
	items = tuple(
		item for item in tab.view.scene().items()
		if type(item) is ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem
	)
	assert len(items) == 1
	items[0].setSelected(True)


#============================================
def test_native_plus_edit_updates_rust_and_retains_durable_selection(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A visible two-field edit commits once and installs its new rendered Plus."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		'<cdml><plus id="p" font_size="14" color="#000">'
		'<point x="10" y="20"/></plus></cdml>',
		"plus.cdml",
	)
	try:
		_select_plus(tab)
		assert tab.has_one_selected_plus()
		plus = tab.selected_plus_projection()
		model = ferrum_qt.ferrum.plus_properties.dialog_model_from_projection(
			plus,
		)
		assert (model.font_size, model.color) == (14, "#000000")
		changes = (
			("font_size", 18),
			("color", "#123456"),
		)
		closed = (
			ferrum_qt.ferrum.plus_properties.
			property_changes_from_dialog(changes)
		)
		result = tab.apply_selected_plus_properties(closed)
		assert result.observation.snapshot.revision == 1
		assert tab.has_one_selected_plus()
		updated = tab.selected_plus_projection()
		assert (updated.font.size, updated.font.color) == (18.0, "#123456")
		item = next(
			item for item in tab.view.scene().items()
			if type(item) is ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem
		)
		assert item.isSelected() and item.foreground_color.name() == "#123456"
	finally:
		tab.dispose()


#============================================
def test_native_plus_dialog_rejects_unrepresentable_source_without_mutation(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The integer form never rounds a valid fractional Rust source fact."""
	del qapp
	session = ferrum_chem.DocumentSession.load(
		'<cdml><plus id="p" font_size="14.5"><point x="1" y="2"/></plus></cdml>',
	)
	plus = session.observe(0).projection.presentation_stack.roots[0].plus
	with pytest.raises(ValueError, match="not representable"):
		ferrum_qt.ferrum.plus_properties.dialog_model_from_projection(plus)
	assert session.observe(0).snapshot.revision == 0
