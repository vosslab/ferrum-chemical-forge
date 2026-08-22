"""Focused Qt behavior for the Rust-authoritative reaction role composer."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine


_REACTION_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'>
<molecule id='left'><atom id='left-a' name='C'><point x='0' y='0'/></atom></molecule>
<molecule id='right'><atom id='right-a' name='O'><point x='160' y='0'/></atom></molecule>
<arrow id='arrow'><point x='40' y='0'/><point x='120' y='0'/></arrow>
</cdml>"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one offscreen application for modeless composer widgets."""
	app = PySide6.QtWidgets.QApplication.instance()
	return app if app is not None else PySide6.QtWidgets.QApplication([])


#============================================
def _check(panel: PySide6.QtWidgets.QWidget, role: str, identifier: str) -> None:
	"""Choose one visible reaction-member row through its Qt checkbox."""
	list_widget = panel.findChild(
		PySide6.QtWidgets.QListWidget, f"reaction-composer-{role}",
	)
	assert list_widget is not None
	for index in range(list_widget.count()):
		item = list_widget.item(index)
		if item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == identifier:
			item.setCheckState(PySide6.QtCore.Qt.CheckState.Checked)
			return
	raise AssertionError(f"missing reaction composer row {identifier!r}")


#============================================
def test_live_window_commits_roles_only_through_the_rust_reaction_bridge(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The ordinary ribbon route reprojects an accepted native reaction commit."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_REACTION_CDML, "reaction-input.cdml",
	)
	main_window._register_native_tab(tab, activate=True)
	observation = tab.observe_direct_root_interaction()
	selection = None
	for identifier in ("left", "right", "arrow"):
		modifier = (
			ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
			if selection is None else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle
		)
		selection = tab.select_direct_roots(
			observation, selection,
			ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(identifier, modifier),
		)
	main_window._replace_render_interaction_selection(selection, tab)
	main_window._create_reaction_action.trigger()
	qapp.processEvents()
	panel = main_window._reaction_composer._panel
	assert panel is not None, main_window.statusBar().currentMessage()
	try:
		_check(panel, "reactants", "left")
		_check(panel, "products", "right")
		_check(panel, "arrow", "arrow")
		panel.submitted.emit()
		qapp.processEvents()
		assert "<reaction id=\"rxn-1\"" in tab.current_snapshot.cdml
	finally:
		main_window._reaction_composer.close()
		tab_widget = main_window.centralWidget()
		assert isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
		tab_widget.removeTab(tab_widget.indexOf(tab))
		tab.dispose()
