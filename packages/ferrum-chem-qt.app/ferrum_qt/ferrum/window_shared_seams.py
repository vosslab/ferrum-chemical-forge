"""Live application clients for Ferrum's declarative Qt interaction seams."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.graphics_view
import ferrum_qt.ferrum.property_observation
import ferrum_qt.ferrum.window_mode_sync
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls


#============================================
def install_shared_window_seams(window: object, registry: object) -> None:
	"""Install shared widget clients around existing Rust-backed window actions."""
	if not isinstance(
		getattr(window, "_window_mode_sync", None),
		ferrum_qt.ferrum.window_mode_sync.FerrumWindowModeSync,
	):
		raise RuntimeError("Ferrum shared clients require the native mode controller.")

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
	window._window_mode_sync.subscribe(
		lambda state: shared_status.update_mode(state.status_label),
	)
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
	tab = window._active_native_tab()
	resolved = _resolve_live_property_observation(window) if resolved is None else resolved
	if not isinstance(
			resolved, ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable,
		):
		_clear_shared_window_clients(window, tab, dock, zoom, status)
		return
	observation = resolved.observation
	if dock is not None:
		dock.refresh(observation)
	view = window._active_native_view()
	percent = ferrum_qt.ferrum.graphics_view.effective_zoom_percent(view)
	if zoom is not None:
		zoom.update_zoom_display(percent)
	if status is not None:
		status.update_mode(window._window_mode_sync.active_state.status_label)


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
	"""Clear every passive client state together when no current observation is available."""
	window._window_mode_sync.cancel()
	if dock is not None:
		dock.refresh(None)
	if zoom is not None:
		zoom.update_zoom_display(None)
	if status is not None:
		status.update_mode("None")
