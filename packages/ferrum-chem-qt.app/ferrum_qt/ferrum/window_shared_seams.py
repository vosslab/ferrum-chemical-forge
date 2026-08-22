"""Live application clients for Ferrum's declarative Qt interaction seams."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.graphics_view
import ferrum_qt.ferrum.property_observation
import ferrum_qt.ferrum.window_mode_sync
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
		lambda context, intent: _dispatch_mode_intent(registry, context, intent),
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
		window.removeDockWidget(old_dock)
		old_dock.deleteLater()
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
def refresh_shared_window_seams(window: object,
		resolved: ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable
				| ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationUnavailable
				| ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationStale
				| None = None) -> None:
	"""Refresh shared view-only clients from the current Rust observation."""
	status = getattr(window, "_shared_status_bar", None)
	zoom = getattr(window, "_shared_zoom_controls", None)
	dock = getattr(window, "_native_property_dock", None)
	manager = getattr(window, "_mode_manager", None)
	toolbar = getattr(window, "_shared_mode_toolbar", None)
	tab = window._active_native_tab()
	resolved = _resolve_live_property_observation(window) if resolved is None else resolved
	if not isinstance(
			resolved, ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable,
		):
		_clear_shared_window_clients(window, tab, dock, zoom, status)
		return
	if toolbar is not None and manager is not None:
		toolbar.apply_mode_manager(manager)
	observation = resolved.observation
	if dock is not None:
		dock.refresh(observation)
	view = window._active_native_view()
	percent = ferrum_qt.ferrum.graphics_view.effective_zoom_percent(view)
	if zoom is not None:
		zoom.update_zoom_display(percent)
	if status is not None and manager is not None:
		state = getattr(
			window, "_shared_active_tool_state",
			ferrum_qt.ferrum.window_mode_sync._INACTIVE_TOOL_STATE,
		)
		status.update_mode(state.status_label)


#============================================
def _resolve_live_property_observation(window: object) -> (
		ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable
		| ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationUnavailable
		| ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationStale
		):
	"""Observe once, repairing one stale Qt projection through its tab owner."""
	tab = window._active_native_tab()
	if tab is None:
		return ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationUnavailable(
			"no_active_tab", False,
		)
	resolved = tab.resolve_live_property_observation()
	if isinstance(
			resolved, ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationStale,
		):
		return _recover_live_property_observation(tab)
	return resolved


#============================================
def _recover_live_property_observation(tab: object) -> (
		ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable
		| ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationUnavailable
		| ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationStale
		):
	"""Attempt one declared stale refresh; let unexpected tab failures surface."""
	if tab.refresh_authoritative():
		resolved = tab.resolve_live_property_observation()
		if not isinstance(
				resolved,
				ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationStale,
				):
			return resolved
	return ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationUnavailable(
		"authoritative_refresh_unavailable", True,
	)


#============================================
def _clear_shared_window_clients(window: object, tab: object | None,
		dock: object | None, zoom: object | None, status: object | None) -> None:
	"""Retire every passive client together when no current observation is available."""
	manager = getattr(window, "_mode_manager", None)
	toolbar = getattr(window, "_shared_mode_toolbar", None)
	if manager is not None:
		manager.cancel(ferrum_qt.modes.base_mode.ModeContext(
			observation=None,
			dispatch_context={"window": window, "tab_title": None if tab is None else tab.title},
		))
	if toolbar is not None and manager is not None:
		toolbar.apply_mode_manager(manager)
	if dock is not None:
		dock.refresh(None)
	if zoom is not None:
		zoom.update_zoom_display(None)
	if status is not None:
		status.update_mode("None")
	window._shared_active_tool_state = ferrum_qt.ferrum.window_mode_sync._INACTIVE_TOOL_STATE


#============================================
def _activate_mode(window: object, toolbar: object, mode_id: str) -> None:
	"""Synchronize a reused Qt tool action with the document-free mode manager."""
	try:
		mode = ferrum_qt.modes.base_mode.ModeId(mode_id)
	except ValueError:
		return
	tab = window._active_native_tab()
	resolved = _resolve_live_property_observation(window)
	if not isinstance(
			resolved, ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable,
		):
		refresh_shared_window_seams(window, resolved)
		return
	observation = resolved.observation
	context = ferrum_qt.modes.base_mode.ModeContext(
		observation=observation,
		dispatch_context={"window": window, "tab_title": tab.title},
	)
	window._mode_manager.synchronize_presentation(mode, context)
	toolbar.apply_mode_manager(window._mode_manager)
	refresh_shared_window_seams(window, resolved)


#============================================
def synchronize_active_tool_state(window: object,
		state: ferrum_qt.ferrum.window_mode_sync.FerrumActiveToolState) -> None:
	"""Reflect the live Rust-tab tool intent in the shared mode clients.

	This is the one adapter between legacy event-owning tools and the declarative
	mode chrome.  It does not dispatch an action, so cancellation and stale
	refusals cannot accidentally re-arm a tool.
	"""
	manager = getattr(window, "_mode_manager", None)
	if manager is None:
		return
	if type(state) is not ferrum_qt.ferrum.window_mode_sync.FerrumActiveToolState:
		raise TypeError("Ferrum shared mode chrome requires a FerrumActiveToolState")
	tab = window._active_native_tab()
	resolved = _resolve_live_property_observation(window)
	if not isinstance(
			resolved, ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable,
		):
		state = ferrum_qt.ferrum.window_mode_sync._INACTIVE_TOOL_STATE
	observation = resolved.observation if isinstance(
		resolved, ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable,
		) else None
	context = ferrum_qt.modes.base_mode.ModeContext(
		observation=observation,
		dispatch_context={"window": window, "tab_title": None if tab is None else tab.title},
	)
	if state.mode_id is None:
		manager.cancel(context)
	else:
		try:
			manager.synchronize_presentation(
				ferrum_qt.modes.base_mode.ModeId(state.mode_id), context,
			)
		except ValueError:
			manager.cancel(context)
			state = ferrum_qt.ferrum.window_mode_sync._INACTIVE_TOOL_STATE
	window._shared_active_tool_state = state
	refresh_shared_window_seams(window, resolved)


#============================================
def _dispatch_mode_intent(registry: object, context: object, intent: object) -> None:
	"""Keep future mode-event dispatch bounded to the established Qt action seams.

	The existing pointer tools retain event ownership today.  This adapter gives
	the manager an explicit, non-document-owning dispatch boundary for semantic
	keyboard/pointer intents without introducing a second Python document path.
	"""
	del context
	operation_id = getattr(intent, "operation_id", None)
	action_ids = {
		"atom.place": "mode.atom",
		"bond.draw": "mode.draw",
		"bracket.create": "mode.bracket",
		"selection.edit": "mode.edit",
	}
	action_id = action_ids.get(operation_id)
	if action_id is None:
		return
	action = registry.get_qt_action(action_id)
	if action is not None and action.isEnabled() and not action.isChecked():
		action.trigger()
