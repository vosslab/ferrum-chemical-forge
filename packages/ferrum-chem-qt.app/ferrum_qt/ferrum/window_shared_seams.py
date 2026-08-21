"""Live application clients for Ferrum's declarative Qt interaction seams."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.graphics_view
import ferrum_qt.ferrum.line_tool_intent
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.mode_manager
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls


_MODE_ACTIONS = (
	("atom", "mode.atom"),
	("draw", "mode.draw"),
	("bracket", "mode.bracket"),
	("edit", "mode.edit"),
)


#============================================
def install_shared_window_seams(window: object, registry: object) -> None:
	"""Install shared widget clients around existing Rust-backed window actions."""
	window._mode_manager = ferrum_qt.modes.mode_manager.ModeManager(
		lambda context, intent: _dispatch_mode_intent(window, context, intent),
	)
	toolbar = getattr(window, "_authoring_ribbon", None)
	if toolbar is None:
		raise RuntimeError("Ferrum shared seams require the authoring ribbon")
	for mode_id, action_id in _MODE_ACTIONS:
		toolbar.add_mode(mode_id, registry.get_qt_action(action_id))
	toolbar.mode_selected.connect(
		lambda mode_id: _activate_mode(window, toolbar, mode_id),
	)
	window._shared_mode_toolbar = toolbar

	old_dock = getattr(window, "_native_property_dock", None)
	if old_dock is not None:
		old_dock.hide()
		window._legacy_property_dock = old_dock
	shared_dock = ferrum_qt.widgets.property_dock.PropertyDock(registry, window)
	window.addDockWidget(PySide6.QtCore.Qt.DockWidgetArea.RightDockWidgetArea, shared_dock)
	window._native_property_dock = shared_dock

	shared_status = window.statusBar()
	old_zoom = getattr(window, "_native_view_status_controls", None)
	if old_zoom is not None:
		old_zoom.hide()
	zoom = ferrum_qt.widgets.zoom_controls.ZoomControls(registry, shared_status)
	zoom.zoom_percent_requested.connect(window._set_active_view_zoom_percent)
	shared_status.addPermanentWidget(zoom)
	window._shared_status_bar = shared_status
	window._shared_zoom_controls = zoom
	refresh_shared_window_seams(window)


#============================================
def refresh_shared_window_seams(window: object) -> None:
	"""Refresh shared view-only clients from the current Rust observation."""
	toolbar = getattr(window, "_shared_mode_toolbar", None)
	manager = getattr(window, "_mode_manager", None)
	if toolbar is not None and manager is not None:
		toolbar.apply_mode_manager(manager)
	status = getattr(window, "_shared_status_bar", None)
	zoom = getattr(window, "_shared_zoom_controls", None)
	dock = getattr(window, "_native_property_dock", None)
	tab = window._active_native_tab()
	legacy_dock = getattr(window, "_legacy_property_dock", None)
	if legacy_dock is not None:
		legacy_dock.refresh(tab)
	if tab is None or getattr(tab, "_disposed", False) or tab.requires_refresh:
		if dock is not None:
			dock.refresh(None)
		if zoom is not None:
			zoom.update_zoom_display(None)
		return
	try:
		observation = tab.observe_properties()
	except Exception:
		observation = None
	if dock is not None:
		dock.refresh(observation)
	view = window._active_native_view()
	percent = ferrum_qt.ferrum.graphics_view.effective_zoom_percent(view)
	if zoom is not None:
		zoom.update_zoom_display(percent)
	if status is not None and manager is not None:
		line_intent = getattr(window, "_line_gesture_intent", None)
		if (
			line_intent is not None
			and line_intent.tool
			is ferrum_qt.ferrum.line_tool_intent._NativeLineTool.ATTACH_CYCLOHEXANE_RING
		):
			label = "Attach Cyclohexane Ring"
		else:
			active = manager.active_mode_id
			label = "None" if active is None else active.value.replace("_", " ").title()
		status.update_mode(label)


#============================================
def _activate_mode(window: object, toolbar: object, mode_id: str) -> None:
	"""Synchronize a reused Qt tool action with the document-free mode manager."""
	try:
		mode = ferrum_qt.modes.base_mode.ModeId(mode_id)
	except ValueError:
		return
	tab = window._active_native_tab()
	if tab is None or tab.requires_refresh:
		return
	try:
		observation = tab.observe_properties()
	except Exception:
		return
	context = ferrum_qt.modes.base_mode.ModeContext(
		observation=observation,
		dispatch_context={"window": window, "tab_title": tab.title},
	)
	window._mode_manager.synchronize_presentation(mode, context)
	toolbar.apply_mode_manager(window._mode_manager)
	refresh_shared_window_seams(window)


#============================================
def synchronize_active_tool_mode(window: object, mode_id: str | None) -> None:
	"""Reflect the live Rust-tab tool intent in the shared mode clients.

	This is the one adapter between legacy event-owning tools and the declarative
	mode chrome.  It does not dispatch an action, so cancellation and stale
	refusals cannot accidentally re-arm a tool.
	"""
	manager = getattr(window, "_mode_manager", None)
	if manager is None:
		return
	tab = window._active_native_tab()
	if tab is None or tab.requires_refresh:
		mode_id = None
	try:
		observation = None if tab is None else tab.observe_properties()
	except Exception:
		observation = None
	context = ferrum_qt.modes.base_mode.ModeContext(
		observation=observation,
		dispatch_context={"window": window, "tab_title": None if tab is None else tab.title},
	)
	if mode_id is None:
		manager.cancel(context)
	else:
		try:
			manager.synchronize_presentation(
				ferrum_qt.modes.base_mode.ModeId(mode_id), context,
			)
		except ValueError:
			manager.cancel(context)
	refresh_shared_window_seams(window)


#============================================
def _dispatch_mode_intent(window: object, context: object, intent: object) -> None:
	"""Keep future mode-event dispatch bounded to the established Qt action seams.

	The existing pointer tools retain event ownership today.  This adapter gives
	the manager an explicit, non-document-owning dispatch boundary for semantic
	keyboard/pointer intents without introducing a second Python document path.
	"""
	del context
	operation_id = getattr(intent, "operation_id", None)
	action_names = {
		"atom.place": "_add_atom_action",
		"bond.draw": "_draw_bond_action",
		"bracket.create": "_draw_bracket_action",
		"selection.edit": "_move_atom_action",
	}
	action_name = action_names.get(operation_id)
	if action_name is None:
		return
	action = getattr(window, action_name, None)
	if action is not None and action.isEnabled() and not action.isChecked():
		action.trigger()
