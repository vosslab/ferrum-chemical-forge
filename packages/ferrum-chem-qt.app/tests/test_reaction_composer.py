"""Focused Qt behavior for the Rust-authoritative reaction role composer."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine


_REACTION_CDML = """<cdml xmlns="urn:ferrum:cdml" version='26.08'>
<molecule id='left'><atom id='left-a' name='C'><point x='0' y='0'/></atom></molecule>
<molecule id='right'><atom id='right-a' name='O'><point x='160' y='0'/></atom></molecule>
<arrow id='arrow'><point x='40' y='0'/><point x='120' y='0'/></arrow>
</cdml>"""


#============================================
def _click_role(panel: PySide6.QtWidgets.QWidget, role: str, identifier: str) -> None:
	"""Choose one visible reaction-member row through its rendered checkbox."""
	list_widget = next(
		(
			candidate for candidate in panel.findChildren(PySide6.QtWidgets.QListWidget)
			if role in candidate.accessibleName().casefold() and candidate.isVisible()
		),
		None,
	)
	assert list_widget is not None, f"missing visible {role} reaction-role control"
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
	for button in panel.findChildren(PySide6.QtWidgets.QPushButton):
		if button.accessibleName() == "Create Reaction":
			assert button.isVisible() and button.isEnabled()
			return button
	raise AssertionError("missing Create Reaction control")


#============================================
def _reaction_role_members(reaction: object) -> dict[str, set[str]]:
	"""Return durable reaction members grouped by their Rust-observed role."""
	members_by_role: dict[str, set[str]] = {}
	for member in reaction.members:
		members_by_role.setdefault(member.role, set()).add(member.document_object_id)
	return members_by_role


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
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	main_window._register_native_tab(tab, activate=True)
	observation = tab.observe_direct_root_interaction()
	root_ids = tuple(
		root.document_object_id
		for root in sorted(observation.roots, key=lambda root: root.paint_order)
	)
	left_document_object_id, right_document_object_id, arrow_document_object_id = root_ids
	selection = None
	for document_object_id in root_ids:
		modifier = (
			ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
			if selection is None else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle
		)
		selection = tab.select_direct_roots(
			observation, selection,
			ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(
				document_object_id, modifier,
			),
		)
	main_window._replace_render_interaction_selection(selection, tab)
	main_window._create_reaction_action.trigger()
	qapp.processEvents()
	panel = main_window._reaction_composer._panel
	assert panel is not None, main_window.statusBar().currentMessage()
	try:
		_click_role(panel, "reactants", left_document_object_id)
		_click_role(panel, "products", right_document_object_id)
		_click_role(panel, "arrow", arrow_document_object_id)
		PySide6.QtTest.QTest.mouseClick(
			_create_reaction_button(panel), PySide6.QtCore.Qt.MouseButton.LeftButton,
		)
		qapp.processEvents()
		snapshot = tab.current_snapshot
		reaction_list = tab._session.observe_reaction_list_v1(
			snapshot.revision, snapshot.digest,
		)
		assert len(reaction_list.reactions) == 1
		members_by_role = _reaction_role_members(reaction_list.reactions[0])
		assert members_by_role == {
			"reactant": {left_document_object_id},
			"product": {right_document_object_id},
			"arrow": {arrow_document_object_id},
		}
	finally:
		main_window._reaction_composer.close()
		tab_widget = main_window.centralWidget()
		assert isinstance(tab_widget, PySide6.QtWidgets.QTabWidget)
		index = tab_widget.indexOf(tab)
		assert index >= 0
		before = tab_widget.count()
		result = main_window._close_native_tab_at(
			index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
		assert tab_widget.count() == before - 1
		assert tab_widget.indexOf(tab) == -1
		assert main_window._operation_leases.active_for_tab(tab) == ()
