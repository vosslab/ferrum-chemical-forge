"""Live application clients for Ferrum's declarative Qt interaction seams."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.graphics_view
import ferrum_qt.ferrum.property_observation
import ferrum_qt.ferrum.window_mode_sync
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar


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
	toggle = registry.get_qt_action("view.properties.toggle")
	if not isinstance(toggle, PySide6.QtGui.QAction):
		raise RuntimeError("Ferrum shared properties require the stable View action.")
	shared_dock.visibilityChanged.connect(toggle.setChecked)
	toggle.setChecked(shared_dock.isVisible())

	shared_status = window.statusBar()
	window._shared_status_bar = shared_status
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
	dock = getattr(window, "_native_property_dock", None)
	tab = window._active_native_tab()
	resolved = _resolve_live_property_observation(window) if resolved is None else resolved
	if not isinstance(
			resolved, ferrum_qt.ferrum.property_observation.FerrumLivePropertyObservationAvailable,
		):
		_clear_shared_window_clients(window, tab, dock, status)
		return
	observation = resolved.observation
	if dock is not None:
		dock.refresh(observation)
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
		dock: object | None, status: object | None) -> None:
	"""Clear every passive client state together when no current observation is available."""
	window._window_mode_sync.cancel()
	if dock is not None:
		dock.refresh(None)
	if status is not None:
		status.update_mode("None")
