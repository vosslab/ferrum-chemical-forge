"""Semantic Arrow property editing through the Ferrum tab."""

# PIP3 modules
import PySide6.QtWidgets
import pytest

ferrum_chem = pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.ferrum.arrow_properties
import ferrum_qt.ferrum.document_tab


def _select_arrow(tab: object) -> None:
	"""Select the one visible Arrow through the graphics scene."""
	selectable = tuple(
		item for item in tab.view.scene().items()
		if item.flags() & PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable
	)
	assert len(selectable) == 1
	selectable[0].setSelected(True)


def test_native_arrow_edit_updates_the_document_and_retains_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A normal Arrow property edit changes durable style without losing selection."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal" start="no" end="yes" '
		'width="1" color="#000"><point x="0" y="0"/>'
		'<point x="40" y="0"/></arrow></cdml>',
		"arrow.cdml",
	)
	try:
		_select_arrow(tab)
		changes = ferrum_qt.ferrum.arrow_properties.property_changes_from_dialog(
			(("start_head", True), ("end_head", False), ("line_width", 2.5), ("color", "#123456")),
		)
		tab.apply_selected_arrow_properties(changes)
		assert 'width="2.5"' in tab.current_snapshot.cdml and 'color="#123456"' in tab.current_snapshot.cdml
		assert tab.has_one_selected_arrow()
	finally:
		tab.dispose()


def test_native_arrow_dialog_rejects_unrepresentable_facts(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The editor refuses facts that it cannot render or mutate faithfully."""
	session = ferrum_chem.DocumentSession.load(
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal" width="0.2"><point x="0" y="0"/>'
		'<point x="40" y="0"/></arrow></cdml>',
	)
	arrow = session.observe(0).projection.presentation_stack.roots[0].arrow
	with pytest.raises(ValueError, match="not representable"):
		ferrum_qt.ferrum.arrow_properties.dialog_model_from_projection(arrow)
	with pytest.raises(ValueError, match="spline rendering"):
		ferrum_qt.ferrum.arrow_properties.property_changes_from_dialog((("spline", True),))
