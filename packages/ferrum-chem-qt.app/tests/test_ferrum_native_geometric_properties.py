"""Semantic geometric appearance editing through the Ferrum tab."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import pytest

ferrum_chem = pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.canvas.ferrum_presentation_projection
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.geometric_properties as native_geometric_properties


#============================================
@pytest.fixture(scope="module")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return one offscreen application without importing the legacy host."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _select_rectangle(tab: object) -> object:
	"""Select and return the one rendered rectangle through its scene item."""
	items = tuple(
		item for item in tab.view.scene().items()
		if type(item) is ferrum_qt.canvas.ferrum_presentation_projection.ShapeProjectionItem
		and item.target.record_kind == "rectangle"
	)
	assert len(items) == 1
	items[0].setSelected(True)
	return items[0]


#============================================
def test_native_shape_edit_updates_rust_and_retains_durable_selection(
		qapp: object,
		) -> None:
	"""A visible shape edit commits once and installs the new vector appearance."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		'<cdml xmlns="urn:ferrum:cdml"><rect id="shape" x1="1" y1="2" x2="30" y2="40" '
		'line_color="#000" area_color="#abcdef" width="1"/></cdml>',
		"shape.cdml",
	)
	try:
		_select_rectangle(tab)
		assert tab.has_one_selected_geometric()
		root = tab.selected_geometric_projection()
		model = native_geometric_properties.dialog_model_from_projection(root)
		assert (model.kind, model.line_width, model.line_color, model.area_color) == (
			"rectangle", 1.0, "#000000", "#abcdef",
		)
		closed = native_geometric_properties.property_changes_from_dialog(
			root,
			(
				("line_width", 2.5),
				("line_color", "#123456"),
				("area_color", None),
			),
		)
		result = tab.apply_selected_geometric_properties(
			model.target_id, model.source_id, closed,
		)
		assert result.observation.snapshot.revision == 1
		assert tab.has_one_selected_geometric()
		updated = tab.selected_geometric_projection().shape
		assert (updated.stroke.width, updated.stroke.color) == (2.5, "#123456")
		assert updated.fill.color is None
		item = _select_rectangle(tab)
		assert item.isSelected()
		assert (item.pen().widthF(), item.pen().color().name()) == (2.5, "#123456")
		assert item.brush().style() == PySide6.QtCore.Qt.BrushStyle.NoBrush
	finally:
		tab.dispose()


#============================================
def test_native_form_rejects_rounding_fill_leakage_and_wavy_fallback(
		qapp: object,
		) -> None:
	"""The form never rounds source width or treats Wavy as an ordinary line."""
	del qapp
	session = ferrum_chem.DocumentSession.load(
		'<cdml xmlns="urn:ferrum:cdml"><rect id="shape" x1="0" y1="0" x2="2" y2="2" '
		'width="1.2345"/><polyline id="line"><point x="0" y="0"/>'
		'<point x="2" y="2"/></polyline><polyline id="wave" style="wavy">'
		'<point x="0" y="0"/><point x="2" y="2"/></polyline></cdml>',
	)
	stack = session.observe(0).projection.presentation_stack
	shape_root = next(root for root in stack.roots if root.kind == "rectangle")
	line_root = next(root for root in stack.roots if root.kind == "polyline")
	with pytest.raises(ValueError, match="not representable"):
		native_geometric_properties.dialog_model_from_projection(shape_root)
	with pytest.raises(ValueError, match="unsupported property"):
		native_geometric_properties.property_changes_from_dialog(
			line_root, (("area_color", "#112233"),),
		)
	assert tuple(root.kind for root in stack.roots) == ("rectangle", "polyline", "wavy")
	wavy_root = next(root for root in stack.roots if root.kind == "wavy")
	with pytest.raises(ValueError, match="not editable geometry"):
		native_geometric_properties.dialog_model_from_projection(wavy_root)
	assert session.observe(0).snapshot.revision == 0
