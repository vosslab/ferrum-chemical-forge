"""Live-window coverage for Ferrum's shared frontend seam clients."""

# Standard Library
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.themes.document_display_palette
import ferrum_qt.themes.theme_loader
import ferrum_qt.actions.action_registry
import ferrum_qt.main_window
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.property_observation
import ferrum_qt.ferrum.window_shared_seams
import ferrum_qt.ferrum.window_mode_sync
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.controllers
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar


#============================================
def _add_atom_through_active_canvas(
		qapp: PySide6.QtWidgets.QApplication,
		window: object,
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> None:
	"""Create one real Rust dirty revision through the ordinary Add Atom tool."""
	atom = window._action_registry.get_qt_action("draw.atom_at_point")
	assert window._window_mode_sync.select_action(atom)
	point = PySide6.QtCore.QPointF(40.0, 50.0)
	local = tab.view.mapFromScene(point)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, local,
	)
	qapp.processEvents()
	assert tab.is_dirty


#============================================
def test_main_window_requires_a_theme_manager_before_building_its_shell(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> None:
	"""The application shell owns an applied display palette from construction."""
	del qapp
	with pytest.raises(TypeError, match="requires ThemeManager"):
		ferrum_qt.main_window.MainWindow(object())
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		assert isinstance(
			window._require_document_display_palette(),
			ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_later_registered_tab_receives_the_current_theme_change_palette(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> None:
	"""A tab registered after a switch receives the manager's stored palette."""
	del qapp
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	changes: list[object] = []
	theme_manager.theme_changed.connect(changes.append)
	theme_manager.apply_theme("dark")
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "later-theme-change.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	try:
		window._register_native_tab(tab, activate=True)
		change = changes[-1]
		assert change.name == "dark"
		assert tab.view.backgroundBrush().color() == change.palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.
			CANVAS_SURROUND,
		)
	finally:
		if not tab.is_disposed:
			tab.dispose()
		window.close()
		window.deleteLater()


#============================================
def test_disposed_document_reports_typed_unavailable_properties(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A disposed Ferrum tab cannot expose a stale property observation."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "typed-observation.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		assert isinstance(
			tab.resolve_live_property_observation(),
			ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable,
		)
	finally:
		tab.dispose()
	assert isinstance(
		tab.resolve_live_property_observation(),
		ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationUnavailable,
	)


#============================================
def test_ordinary_window_exposes_native_status_bar_view_controls(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> None:
	"""The running product exposes one visible status-bar client for View actions."""
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "shared-seams.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.resize(1440, 900)
		window.show()
		qapp.processEvents()
		assert window._window_mode_sync.active_state.mode_id is None
		assert isinstance(window._native_property_dock, ferrum_qt.widgets.property_dock.PropertyDock)
		assert isinstance(window.statusBar(), ferrum_qt.widgets.status_bar.StatusBar)
		qapp.sendPostedEvents(None, PySide6.QtCore.QEvent.Type.DeferredDelete)
		qapp.processEvents()
		properties_toggle = window._action_registry.get_qt_action(
			"view.properties.toggle",
		)
		assert properties_toggle is window._property_dock_toggle_action
		assert properties_toggle.isChecked()
		properties_toggle.trigger()
		qapp.processEvents()
		assert not window._native_property_dock.isVisible()
		properties_toggle.trigger()
		qapp.processEvents()
		assert window._native_property_dock.isVisible()
		controls = window._native_view_status_controls
		assert controls.isVisible()
		assert controls.width() >= controls.minimumSizeHint().width()
		assert not hasattr(window, "_shared_zoom_controls")
		reset = next(
			button for button in controls.findChildren(PySide6.QtWidgets.QToolButton)
			if button.accessibleName() == "Reset zoom to 100%"
		)
		content = next(
			button for button in controls.findChildren(PySide6.QtWidgets.QToolButton)
			if button.accessibleName() == "Zoom to Content"
		)
		slider = controls.findChild(PySide6.QtWidgets.QSlider, "")
		assert reset.isVisible() and content.isVisible()
		assert slider is not None and slider.isVisible()
		triggered: list[bool] = []
		window._zoom_100_action.triggered.connect(lambda: triggered.append(True))
		PySide6.QtTest.QTest.mouseClick(reset, PySide6.QtCore.Qt.MouseButton.LeftButton)
		qapp.processEvents()
		assert triggered == [True]
		action = window._action_registry.get_qt_action("draw.bond")
		assert action is window._draw_bond_action
		draw_group = next(
			group for group in window._authoring_ribbon.groups_for_tab("home")
			if group.layout_data.id == "draw"
		)
		assert draw_group.accessibleName()
		assert draw_group.direct_button_for(action).defaultAction() is action
		action.trigger()
		qapp.processEvents()
		assert window._window_mode_sync.active_state.mode_id == "draw"
		assert action.isChecked()
		assert window._window_mode_sync.select_action(action)
		action.setEnabled(False)
		assert not window._window_mode_sync.select_action(action)
		mode_label = window.statusBar().findChild(
			PySide6.QtWidgets.QLabel, "active-editing-mode",
		)
		assert mode_label is not None
		assert mode_label.text() == "Mode: Draw Bond"
	finally:
		window._cancel_atom_insertion()
		window._cancel_line_gesture()
		if not tab.is_disposed:
			tab.dispose()
		window.close()


#============================================
def test_shown_window_keeps_1440_by_900_across_all_authoring_tabs(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> None:
	"""Every allocator-owned ribbon page preserves the requested 16:10 outer window."""
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		window.resize(1440, 900)
		window.show()
		qapp.processEvents()
		assert window.size() == PySide6.QtCore.QSize(1440, 900)
		ribbon = window._authoring_ribbon
		for index in range(ribbon._tabs.count()):
			ribbon._tabs.setCurrentIndex(index)
			qapp.processEvents()
			assert window.size() == PySide6.QtCore.QSize(1440, 900)
			page = ribbon._tabs.currentWidget()
			assert page is not None and page.contentsRect().contains(
				PySide6.QtCore.QRect(
					page.contentsRect().topLeft(),
					PySide6.QtCore.QSize(max(0, page.minimumSizeHint().width()), 1),
				),
			)
			for group in ribbon.groups_for_tab(ribbon._layouts[index].id):
				assert group.visible_actions()
				assert group.focus_target_for(group.visible_actions()[0]).accessibleName()
	finally:
		window.close()


#============================================
def test_structure_ribbon_allocator_restores_expanded_groups_from_live_hints(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> None:
	"""The page allocator reduces at 16:10 and restores at its measured wide surface."""
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		window.resize(1440, 900)
		window.show()
		qapp.processEvents()
		ribbon = window._authoring_ribbon
		structure_index = next(index for index, layout in enumerate(ribbon._layouts)
			if layout.id == "structure")
		ribbon._tabs.setCurrentIndex(structure_index)
		qapp.processEvents()
		page = ribbon._tabs.currentWidget()
		assert page is not None
		states = tuple(group.display_state for group in ribbon.groups_for_tab("structure"))
		assert any(state.value != "expanded" for state in states)
		expanded = type(states[0]).EXPANDED
		full_width = page._required_width(tuple(expanded for _state in states))
		window.resize(window.width() + full_width - page.width(), 900)
		qapp.processEvents()
		assert all(group.display_state is expanded
			for group in ribbon.groups_for_tab("structure"))
	finally:
		window.close()


#============================================
def test_ordinary_window_exposes_command_palette_through_view_menu_and_shortcut(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> None:
	"""The running application gives one live View-menu action a portable shortcut."""
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		action = window._action_registry.get_qt_action("view.command_palette")
		assert action in window._view_menu.actions()
		assert action.shortcut().toString(
			PySide6.QtGui.QKeySequence.SequenceFormat.PortableText,
		) == "Ctrl+K"
		assert action.shortcutContext() == (
			PySide6.QtCore.Qt.ShortcutContext.WindowShortcut
		)
		window.show()
		action.trigger()
		qapp.processEvents()
		assert window._command_palette_controller.dialog.isVisible()
		assert window._command_palette_controller.dialog.search_field.hasFocus()
	finally:
		window.close()


#============================================
def test_window_mode_sync_owns_registered_action_selection_and_cancellation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Controller state follows one registered QAction without a ribbon bridge."""
	parent = PySide6.QtWidgets.QWidget()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	action = PySide6.QtGui.QAction("Draw Test Tool", parent)
	action.setToolTip("Draw through the test feature endpoint")
	action.setCheckable(True)
	registry.register_existing(
		"draw.test_tool", action, shortcut_exemption_reason="Test action.",
	)
	controller = ferrum_qt.ferrum.window_mode_sync.FerrumWindowModeSync(registry)
	context = lambda: ferrum_qt.modes.base_mode.ModeContext(None, parent)
	recorded: list[ferrum_qt.modes.base_mode.ModeIntent] = []
	binding = ferrum_qt.ferrum.window_mode_sync.FerrumWindowToolBinding(
		action, ferrum_qt.modes.base_mode.ModeId.DRAW,
		ferrum_qt.modes.controllers.DrawMode(), "Draw Test Tool", True, context,
		lambda _context: None, lambda _context, intent: recorded.append(intent),
		lambda _context: None,
	)
	controller.register_tool(binding)
	assert controller.select_action(action)
	qapp.processEvents()
	assert action.isChecked() and controller.active_state.supplies_drawing_defaults
	controller.handle_pointer(ferrum_qt.modes.base_mode.PointerInput(
		ferrum_qt.modes.base_mode.PointerPhase.PRESS,
		ferrum_qt.modes.base_mode.ScenePoint(1.0, 2.0),
	))
	controller.handle_pointer(ferrum_qt.modes.base_mode.PointerInput(
		ferrum_qt.modes.base_mode.PointerPhase.RELEASE,
		ferrum_qt.modes.base_mode.ScenePoint(3.0, 4.0),
	))
	assert recorded == [ferrum_qt.modes.base_mode.ModeIntent(
		"bond.draw", (
			ferrum_qt.modes.base_mode.ScenePoint(1.0, 2.0),
			ferrum_qt.modes.base_mode.ScenePoint(3.0, 4.0),
		),
	)]
	assert controller.cancel() and not action.isChecked()
	action.setEnabled(False)
	assert not controller.select_action(action)
	with pytest.raises(RuntimeError, match="unregistered"):
		controller.select_action(PySide6.QtGui.QAction("Other", parent))
	parent.close()
	parent.deleteLater()


#============================================
def test_declined_tool_activation_rolls_back_provisional_controller_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A declined endpoint sees provisional state, then leaves no active tool."""
	parent = PySide6.QtWidgets.QWidget()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	first = PySide6.QtGui.QAction("First Tool", parent)
	declined = PySide6.QtGui.QAction("Declined Tool", parent)
	for action, action_id in ((first, "draw.first"), (declined, "draw.declined")):
		action.setCheckable(True)
		registry.register_existing(action_id, action, shortcut_exemption_reason="Test action.")
	controller = ferrum_qt.ferrum.window_mode_sync.FerrumWindowModeSync(registry)
	context = lambda: ferrum_qt.modes.base_mode.ModeContext(None, parent)
	cancelled: list[str] = []
	controller.register_tool(ferrum_qt.ferrum.window_mode_sync.FerrumWindowToolBinding(
		first, ferrum_qt.modes.base_mode.ModeId.DRAW,
		ferrum_qt.modes.controllers.DrawMode(), "First Tool", True, context,
		lambda _context: True, lambda _context, _intent: None,
		lambda _context: cancelled.append("first"),
	))

	def decline(_context: ferrum_qt.modes.base_mode.ModeContext) -> bool:
		"""Observe the transaction before declining its native resource."""
		cancelled.append("provisional" if declined.isChecked() else "unchecked")
		return False

	controller.register_tool(ferrum_qt.ferrum.window_mode_sync.FerrumWindowToolBinding(
		declined, ferrum_qt.modes.base_mode.ModeId.EDIT,
		ferrum_qt.modes.controllers.EditMode(), "Declined Tool", False, context,
		decline, lambda _context, _intent: None,
		lambda _context: cancelled.append("declined"),
	))
	assert controller.select_action(first)
	assert controller.select_action(declined)
	qapp.processEvents()
	assert cancelled == ["first", "provisional", "declined"]
	assert controller.active_state.mode_id is None and not first.isChecked() and not declined.isChecked()
	parent.close()
	parent.deleteLater()


#============================================
def test_native_atom_input_dispatches_to_rust_mutation_through_window_mode_sync(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""One native canvas click follows adapter, controller, and Add Atom to Rust."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"atom-dispatch.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		qapp.processEvents()
		action = window._action_registry.get_qt_action("draw.atom_at_point")
		assert action is window._add_atom_action and window._window_mode_sync.select_action(action)
		scene_point = PySide6.QtCore.QPointF(40.0, 50.0)
		local_point = tab.view.mapFromScene(scene_point)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, local_point,
		)
		qapp.processEvents()
		assert len(tab.current_document_observation().projection.molecules[0].atoms) == 2
	finally:
		window._window_mode_sync.cancel()


#============================================
def test_native_structure_selection_preserves_shift_additive_targets(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Visible native clicks retain ordinary and Shift-toggle Rust structure selection."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' name='C'><point x='10' y='10'/></atom><atom id='a2' name='O'><point x='70' y='10'/></atom></molecule></cdml>",
		"structure-shift-selection.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		qapp.processEvents()
		assert window._window_mode_sync.select_action(window._select_structure_action)
		for modifier, point in (
			(PySide6.QtCore.Qt.KeyboardModifier.NoModifier, PySide6.QtCore.QPointF(10.0, 10.0)),
			(PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier, PySide6.QtCore.QPointF(70.0, 10.0)),
		):
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				modifier, tab.view.mapFromScene(point),
			)
		qapp.processEvents()
		assert len(window._structure_selection.targets) == 2
	finally:
		window._window_mode_sync.cancel()


#============================================
def test_controller_viewport_input_releases_on_tab_switch_and_close(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""A controller mode cannot receive stale viewport clicks after its tab leaves."""
	window = main_window
	first = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='first'><atom id='a' name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"first.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	second = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='second'><atom id='b' name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"second.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(first, activate=True)
		window._register_native_tab(second, activate=False)
		window.show()
		qapp.processEvents()
		action = window._action_registry.get_qt_action("draw.atom_at_point")
		assert window._window_mode_sync.select_action(action)
		window._tab_widget.setCurrentWidget(second)
		qapp.processEvents()
		before = first.current_snapshot.revision
		PySide6.QtTest.QTest.mouseClick(
			first.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			first.view.mapFromScene(PySide6.QtCore.QPointF(40.0, 50.0)),
		)
		qapp.processEvents()
		assert not action.isChecked() and first.current_snapshot.revision == before
		assert window._window_mode_sync.select_action(action)
		assert window._close_native_tab_at(
			window._tab_widget.indexOf(second),
			ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		) is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
		assert window._window_mode_sync.active_state.mode_id is None
	finally:
		window._window_mode_sync.cancel()


#============================================
def test_explicit_discard_closes_a_real_dirty_tab_without_mutating_its_dirty_state(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""A lifecycle caller can discard real Rust changes without a prompt or save."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"discard-close.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	atom = window._action_registry.get_qt_action("draw.atom_at_point")
	assert window._window_mode_sync.select_action(atom)
	point = PySide6.QtCore.QPointF(40.0, 50.0)
	local = PySide6.QtCore.QPointF(tab.view.mapFromScene(point))
	global_point = PySide6.QtCore.QPointF(tab.view.viewport().mapToGlobal(local.toPoint()))
	PySide6.QtCore.QCoreApplication.sendEvent(tab.view.viewport(), PySide6.QtGui.QMouseEvent(
		PySide6.QtCore.QEvent.Type.MouseButtonRelease, local, point, global_point,
		PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.MouseButton.NoButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		PySide6.QtGui.QPointingDevice.primaryPointingDevice(),
	))
	qapp.processEvents()
	assert tab.is_dirty
	index = window._tab_widget.indexOf(tab)
	assert window._close_native_tab_at(
		index, ferrum_qt.ferrum.close_decision.CloseDecision.KEEP_OPEN,
	) is ferrum_qt.ferrum.close_decision.CloseResult.DIRTY_REQUIRES_DECISION
	assert tab in window._native_tabs_by_page and tab.is_dirty
	assert window._close_native_tab_at(
		index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
	) is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	assert tab.is_disposed and tab not in window._native_tabs_by_page


#============================================
def test_explicit_discard_retains_refresh_required_tab(
		main_window: object, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Discard cannot bypass authoritative refresh recovery or force disposal."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "refresh-required-close.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	monkeypatch.setattr(type(tab), "requires_refresh", property(lambda _tab: True))
	result = window._close_native_tab_at(
		window._tab_widget.indexOf(tab),
		ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
	)
	assert result is ferrum_qt.ferrum.close_decision.CloseResult.REFRESH_REQUIRED
	assert tab in window._native_tabs_by_page and not tab.is_disposed
	monkeypatch.undo()


#============================================
def test_explicit_save_closes_dirty_tab_through_real_native_save(
		qapp: PySide6.QtWidgets.QApplication, main_window: object, tmp_path: pathlib.Path,
		) -> None:
	"""An explicit Save publishes a dirty tab, then completes its close."""
	window = main_window
	destination = tmp_path / "typed-close-save.cdml"
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"typed-close-save.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	qapp.processEvents()
	tab.save_atomic(destination)
	_add_atom_through_active_canvas(qapp, window, tab)
	result = window._close_native_tab_at(
		window._tab_widget.indexOf(tab),
		ferrum_qt.ferrum.close_decision.CloseDecision.SAVE,
	)
	assert result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	assert destination.is_file() and tab.is_disposed


#============================================
def test_explicit_save_failure_retains_dirty_tab(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A refused save reports its typed result and preserves the active tab."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"typed-close-save-failed.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	qapp.processEvents()
	_add_atom_through_active_canvas(qapp, window, tab)
	monkeypatch.setattr(window, "_on_save", lambda: False)
	result = window._close_native_tab_at(
		window._tab_widget.indexOf(tab),
		ferrum_qt.ferrum.close_decision.CloseDecision.SAVE,
	)
	assert result is ferrum_qt.ferrum.close_decision.CloseResult.SAVE_FAILED
	assert tab in window._native_tabs_by_page and tab.is_dirty


#============================================
def test_explicit_keep_open_retains_dirty_tab_without_prompt(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""An explicit Cancel-equivalent decision preserves unsaved native work."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"typed-close-keep-open.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	qapp.processEvents()
	_add_atom_through_active_canvas(qapp, window, tab)
	result = window._close_native_tab_at(
		window._tab_widget.indexOf(tab),
		ferrum_qt.ferrum.close_decision.CloseDecision.KEEP_OPEN,
	)
	assert result is ferrum_qt.ferrum.close_decision.CloseResult.DIRTY_REQUIRES_DECISION
	assert tab in window._native_tabs_by_page and tab.is_dirty


#============================================
def test_invalid_close_index_returns_no_tab_without_window_mutation(main_window: object) -> None:
	"""An absent tab index returns NO_TAB without changing the visible document."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "typed-close-invalid-index.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	window._register_native_tab(tab, activate=True)
	result = window._close_native_tab_at(
		-1, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
	)
	assert result is ferrum_qt.ferrum.close_decision.CloseResult.NO_TAB
	assert window._tab_widget.currentWidget() is tab and tab in window._native_tabs_by_page


#============================================
def test_programmatic_tool_transitions_dispatch_to_newly_active_selection_feature(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""Programmatic QAction selection cancels prior tools and drives Structure input."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"programmatic-mode-transitions.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		bond = window._action_registry.get_qt_action("draw.bond")
		text = window._action_registry.get_qt_action("draw.text")
		selection = window._action_registry.get_qt_action("draw.selection.structure")
		assert bond is window._draw_bond_action and text is window._insert_text_action and selection is window._select_structure_action
		assert window._window_mode_sync.select_action(bond)
		assert window._window_mode_sync.select_action(text)
		assert not bond.isChecked() and text.isChecked()
		assert window._window_mode_sync.select_action(selection)
		atom = tab.current_document_observation().projection.molecules[0].atoms[0]
		window._window_mode_sync.handle_pointer(ferrum_qt.modes.base_mode.PointerInput(
			ferrum_qt.modes.base_mode.PointerPhase.PRESS,
			ferrum_qt.modes.base_mode.ScenePoint(atom.position.x, atom.position.y),
		))
		assert selection.isChecked() and window._structure_selection is not None
		assert window._window_mode_sync.handle_key("Escape") and not selection.isChecked()
	finally:
		window._window_mode_sync.cancel()


#============================================
def test_line_dispatch_rejects_mismatched_normalized_tool_without_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		main_window: object,
		) -> None:
	"""A corrupted controller intent fails before it reaches the Rust document seam."""
	window = main_window
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "mismatched-line-intent.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		bond_action = window._action_registry.get_qt_action("draw.bond")
		bond_action.trigger()
		qapp.processEvents()
		before_revision = tab.current_snapshot.revision
		context = ferrum_qt.modes.base_mode.ModeContext(None, tab)
		intent = ferrum_qt.modes.base_mode.ModeIntent(
			"line.draw_arrow.press", (ferrum_qt.modes.base_mode.ScenePoint(10.0, 10.0),),
		)
		with pytest.raises(RuntimeError, match="different active tool"):
			window._window_mode_sync._dispatch_intent(context, intent)
		assert tab.current_snapshot.revision == before_revision
		assert window._window_mode_sync.active_state.mode_id == "draw"
		assert bond_action.isChecked()
		assert window._window_mode_sync.cancel()
		assert window._window_mode_sync.active_state.mode_id is None
		assert not bond_action.isChecked()
	finally:
		if not tab.is_disposed:
			tab.dispose()
