"""Semantic Wavy appearance editing through the Ferrum tab."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

ferrum_chem = pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.canvas.ferrum_presentation_projection
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.wavy_properties as native_wavy_properties


#============================================
@pytest.fixture(scope="module")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return one offscreen application without importing the legacy host."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _select_wavy(tab: object) -> object:
	"""Select and return the one rendered Wavy path through its durable target."""
	items = tuple(
		item for item in tab.view.scene().items()
		if type(item) is ferrum_qt.canvas.ferrum_presentation_projection.PolylineProjectionItem
		and item.target.record_kind == "polyline"
	)
	assert len(items) == 1
	items[0].setSelected(True)
	return items[0]


#============================================
def test_native_wavy_edit_preserves_authored_path_and_durable_selection(
		qapp: object,
		) -> None:
	"""A visible edit commits once without regenerating the stored point path."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		'<cdml><polyline id="wave" style="wavy" color="#000" width="1">'
		'<point x="0" y="0"/><point x="3" y="2"/>'
		'<point x="6" y="0"/></polyline></cdml>',
		"wave.cdml",
	)
	try:
		item = _select_wavy(tab)
		assert tab.has_one_selected_wavy()
		assert not tab.has_one_selected_geometric()
		root = tab.selected_wavy_projection()
		model = native_wavy_properties.dialog_model_from_projection(root)
		changes = native_wavy_properties.property_changes_from_dialog(
			(("width", 2.5), ("line_color", "#123456")),
		)
		result = tab.apply_selected_wavy_properties(
			model.target_id, model.source_id, changes,
		)
		assert result.observation.snapshot.revision == 1
		assert tab.has_one_selected_wavy()
		updated = tab.selected_wavy_projection().polyline
		assert [(point.x, point.y) for point in updated.path.points] == [
			(0.0, 0.0), (3.0, 2.0), (6.0, 0.0),
		]
		assert (updated.stroke.width, updated.stroke.color) == (2.5, "#123456")
		item = _select_wavy(tab)
		assert item.isSelected()
		assert item.path().elementCount() == 3
		assert (item.pen().widthF(), item.pen().color().name()) == (2.5, "#123456")
	finally:
		tab.dispose()


#============================================
def test_wavy_form_rejects_coercion_and_spline_has_a_typed_issue(qapp: object) -> None:
	"""Unrepresentable form values and unsupported interpolation fail visibly."""
	del qapp
	session = ferrum_chem.DocumentSession.load(
		'<cdml><polyline id="wave" style="wavy" width="1.2345">'
		'<point x="0" y="0"/><point x="2" y="2"/></polyline>'
		'<polyline id="spline" style="wavy" spline="yes">'
		'<point x="0" y="0"/><point x="2" y="2"/></polyline></cdml>',
	)
	stack = session.observe(0).projection.presentation_stack
	root = next(root for root in stack.roots if root.kind == "wavy")
	with pytest.raises(ValueError, match="not representable"):
		native_wavy_properties.dialog_model_from_projection(root)
	issue = next(issue for issue in stack.issues if issue.target.source_id == "spline")
	assert issue.code == "unsupported_spline"
	assert session.observe(0).snapshot.revision == 0


#============================================
def test_native_wavy_creation_uses_rust_identity_geometry_and_history(qapp: object) -> None:
	"""Create, select, undo, and redo one backend-authored Wavy root."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml/>", "new-wave.cdml",
	)
	try:
		result = tab.create_wavy(0.0, 0.0, 48.0, 0.0)
		roots = result.observation.projection.presentation_stack.roots
		assert len(roots) == 1 and roots[0].kind == "wavy"
		polyline = roots[0].polyline
		assert polyline.target.source_id == "ferrum-presentation-v1-0"
		assert len(polyline.path.points) == 5
		selected = tab._controller.projection.selected_durable_targets()
		assert len(selected) == 1 and selected[0].kind == "polyline"
		tab.undo()
		assert not tab._document_observation.projection.presentation_stack.roots
		tab.redo()
		assert tab._document_observation.projection.presentation_stack.roots[0].kind == "wavy"
	finally:
		tab.dispose()
