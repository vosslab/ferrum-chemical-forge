"""Semantic native presentation ordering through the Rust session."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab


#============================================
@pytest.fixture(scope="module")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return one offscreen application without the legacy host."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _root_target(root: object) -> object:
	"""Return the exact payload target for the fixture's closed root kinds."""
	return getattr(root, root.kind).target


#============================================
def _source_order(tab: object) -> tuple[str, ...]:
	"""Return durable presentation source IDs in backend-issued order."""
	return tuple(
		_root_target(root).source_id
		for root in tab._document_observation.projection.presentation_stack.roots
	)


#============================================
def test_selected_roots_reorder_through_rust_and_retain_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A stack edit changes semantic order and history without a Qt-local fallback."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		'<cdml xmlns:v="urn:vendor"><arrow id="a"><point x="0" y="0"/>'
		'<point x="1" y="1"/></arrow><v:opaque retained="yes"/>'
		'<text id="t"><point x="2" y="2"/><ftext>note</ftext></text>'
		'<plus id="p"><point x="3" y="3"/></plus></cdml>',
		"stack.cdml",
	)
	try:
		roots = tab._document_observation.projection.presentation_stack.roots
		selected = tuple((root.kind, _root_target(root).id) for root in roots[:2])
		tab._controller.projection.select_durable(selected)
		assert tab.has_selected_presentation_stack_roots(2)
		result = tab.reorder_selected_presentation_roots(
			ferrum_chem.DocumentPresentationStackOrderV1.bring_to_front,
		)
		assert result.observation.snapshot.revision == 1
		assert _source_order(tab) == ("p", "a", "t")
		assert {
			(target.kind, target.identifier)
			for target in tab._controller.projection.selected_durable_targets()
		} == set(selected)
		assert 'retained="yes"' in result.observation.snapshot.cdml
		tab.undo()
		assert _source_order(tab) == ("a", "t", "p")
	finally:
		tab.dispose()


#============================================
def test_partial_bracket_selection_is_not_exposed_for_stack_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The native route cannot separate one member of an authoritative bracket pair."""
	del qapp
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		'<cdml><polyline id="left" bracket_pair="left" bracket_side="left" spline="no">'
		'<point x="0" y="0"/><point x="1" y="1"/><point x="1" y="2"/>'
		'<point x="0" y="3"/></polyline><polyline id="right" bracket_pair="left" '
		'bracket_side="right" spline="no"><point x="4" y="0"/>'
		'<point x="3" y="1"/><point x="3" y="2"/><point x="4" y="3"/>'
		'</polyline></cdml>',
		"bracket.cdml",
	)
	try:
		roots = tab._document_observation.projection.presentation_stack.roots
		left = _root_target(roots[0])
		tab._controller.projection.select_durable(((roots[0].kind, left.id),))
		assert not tab.has_selected_presentation_stack_roots()
		both = tuple((root.kind, _root_target(root).id) for root in roots)
		tab._controller.projection.select_durable(both)
		assert tab.has_selected_presentation_stack_roots(2)
		assert tab.current_snapshot.revision == 0
	finally:
		tab.dispose()
