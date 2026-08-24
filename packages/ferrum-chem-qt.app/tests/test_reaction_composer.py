"""Focused Qt behavior for the Rust-authoritative reaction role composer."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
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
def _click_role(panel: PySide6.QtWidgets.QWidget, role: str, identifier: str) -> None:
	"""Choose one visible reaction-member row through its rendered checkbox."""
	list_widget = panel.findChild(
		PySide6.QtWidgets.QListWidget, f"reaction-composer-{role}",
	)
	assert list_widget is not None
	for index in range(list_widget.count()):
		item = list_widget.item(index)
		if item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == identifier:
			option = PySide6.QtWidgets.QStyleOptionViewItem()
			option.initFrom(list_widget)
			option.rect = list_widget.visualItemRect(item)
			option.features |= PySide6.QtWidgets.QStyleOptionViewItem.ViewItemFeature.HasCheckIndicator
			option.checkState = item.checkState()
			checkbox_rect = list_widget.style().subElementRect(
				PySide6.QtWidgets.QStyle.SubElement.SE_ItemViewItemCheckIndicator,
				option, list_widget,
			)
			assert checkbox_rect.isValid()
			PySide6.QtTest.QTest.mouseClick(
				list_widget.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				checkbox_rect.center(),
			)
			assert item.checkState() == PySide6.QtCore.Qt.CheckState.Checked
			return
	raise AssertionError(f"missing reaction composer row {identifier!r}")


#============================================
def _create_reaction_button(panel: PySide6.QtWidgets.QWidget) -> PySide6.QtWidgets.QPushButton:
	"""Return the visible, enabled submit control by its accessible UI contract."""
	buttons = [
		button for button in panel.findChildren(PySide6.QtWidgets.QPushButton)
		if button.accessibleName() == "Create Reaction"
	]
	assert len(buttons) == 1
	button = buttons[0]
	assert button.isVisible() and button.isEnabled()
	return button


#============================================
def test_live_window_commits_roles_only_through_the_rust_reaction_bridge(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The ordinary ribbon route reprojects an accepted native reaction commit."""
	main_window.resize(1280, 800)
	main_window.show()
	qapp.processEvents()
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
		_click_role(panel, "reactants", "left")
		_click_role(panel, "products", "right")
		_click_role(panel, "arrow", "arrow")
		PySide6.QtTest.QTest.mouseClick(
			_create_reaction_button(panel), PySide6.QtCore.Qt.MouseButton.LeftButton,
		)
		qapp.processEvents()
		assert "<reaction id=\"rxn-1\"" in tab.current_snapshot.cdml
	finally:
		main_window._reaction_composer.close()
		tab_widget = main_window.centralWidget()
		assert isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
		tab_widget.removeTab(tab_widget.indexOf(tab))
		tab.dispose()
