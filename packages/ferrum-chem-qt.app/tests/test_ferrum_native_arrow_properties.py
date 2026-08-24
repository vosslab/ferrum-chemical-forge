"""Semantic Arrow property editing through the Ferrum tab."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

ferrum_chem = pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.ferrum.arrow_properties
import ferrum_qt.ferrum.document_tab


#============================================
@pytest.fixture(scope="module")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return one offscreen application without importing the legacy host."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _select_arrow(tab: object) -> None:
	"""Select the one visible Arrow through the public graphics scene."""
	selectable = tuple(
		item for item in tab.view.scene().items()
		if item.flags() & PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable
	)
	assert len(selectable) == 1
	selectable[0].setSelected(True)


#============================================
def test_native_arrow_edit_updates_rust_and_retains_durable_selection(
		qapp: object,
		) -> None:
	"""A visible normal-Arrow edit commits once and retains its durable selection."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal" start="no" end="yes" '
		'width="1" color="#000"><point x="0" y="0"/>'
		'<point x="40" y="0"/></arrow></cdml>',
		"arrow.cdml",
	)
	try:
		_select_arrow(tab)
		assert tab.has_one_selected_arrow()
		arrow = tab.selected_arrow_projection()
		model = (
			ferrum_qt.ferrum.arrow_properties.
			dialog_model_from_projection(arrow)
		)
		assert (model.start_head, model.end_head, model.line_width, model.color) == (
			False, True, 1.0, "#000000",
		)
		closed = ferrum_qt.ferrum.arrow_properties.property_changes_from_dialog(
			(
				("start_head", True),
				("end_head", False),
				("line_width", 2.5),
				("color", "#123456"),
			),
		)
		result = tab.apply_selected_arrow_properties(closed)
		assert result.observation.snapshot.revision == 1
		assert tab.has_one_selected_arrow()
		updated = tab.selected_arrow_projection()
		assert updated.kind.kind == "normal"
		assert (updated.kind.start_head, updated.kind.end_head) == (True, False)
		assert (updated.stroke.width, updated.stroke.color) == (2.5, "#123456")
		assert len(tab.view.scene().selectedItems()) == 1
	finally:
		tab.dispose()


#============================================
def test_native_arrow_dialog_rejects_unrendered_or_unrepresentable_facts(
		qapp: object,
		) -> None:
	"""The form never rounds width or enables unsupported spline mutation."""
	del qapp
	session = ferrum_chem.DocumentSession.load(
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal" width="0.2"><point x="0" y="0"/>'
		'<point x="40" y="0"/></arrow></cdml>',
	)
	arrow = session.observe(0).projection.presentation_stack.roots[0].arrow
	with pytest.raises(ValueError, match="not representable"):
		ferrum_qt.ferrum.arrow_properties.dialog_model_from_projection(arrow)
	with pytest.raises(ValueError, match="spline rendering"):
		ferrum_qt.ferrum.arrow_properties.property_changes_from_dialog(
			(("spline", True),),
		)
	assert session.observe(0).snapshot.revision == 0
