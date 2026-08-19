"""Semantic durable presentation deletion through the Ferrum tab."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.canvas.items.ferrum_text_item
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
def test_selected_text_deletion_updates_rust_scene_and_history(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The generic operation deletes one exact root and remains normally undoable."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		'<cdml xmlns:v="urn:vendor"><text id="t"><point x="1" y="2"/>'
		'<ftext>label</ftext></text><v:opaque retained-id="t"/>'
		'<plus id="p"><point x="3" y="4"/></plus></cdml>',
		"presentation.cdml",
	)
	try:
		item = next(
			item for item in tab.view.scene().items()
			if type(item) is ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem
		)
		item.setSelected(True)
		assert tab.has_one_selected_presentation_root()
		deleted = tab.delete_selected_presentation_root()
		assert deleted.observation.snapshot.revision == 1
		assert not any(
			type(item) is ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem
			for item in tab.view.scene().items()
		)
		assert '<v:opaque retained-id="t"' in deleted.observation.snapshot.cdml
		tab.undo()
		assert any(
			type(item) is ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem
			for item in tab.view.scene().items()
		)
	finally:
		tab.dispose()


#============================================
def test_bracket_deletion_requires_and_accepts_the_complete_pair(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The Ferrum action cannot leave half of an authoritative bracket behind."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		'<cdml><polyline id="left" bracket_pair="left" bracket_side="left" spline="yes">'
		'<point x="0" y="0"/><point x="1" y="1"/><point x="1" y="2"/>'
		'<point x="0" y="3"/></polyline><polyline id="right" bracket_pair="left" '
		'bracket_side="right" spline="yes"><point x="4" y="0"/>'
		'<point x="3" y="1"/><point x="3" y="2"/><point x="4" y="3"/>'
		'</polyline></cdml>',
		"bracket.cdml",
	)
	try:
		left = tab._document_observation.projection.presentation_stack.roots[0]
		right = tab._document_observation.projection.presentation_stack.roots[1]
		tab._controller.projection.select_durable((("polyline", left.polyline.target.id),))
		assert not tab.has_selected_presentation_roots_for_deletion()
		tab._controller.projection.select_durable((
			("polyline", left.polyline.target.id),
			("polyline", right.polyline.target.id),
		))
		deleted = tab.delete_selected_presentation_roots()
		assert not deleted.observation.projection.presentation_stack.roots
	finally:
		tab.dispose()
