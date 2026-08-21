"""Live-window coverage for Ferrum's shared frontend seam clients."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.ferrum.authoring_ribbon
import ferrum_qt.ferrum.window_shared_seams
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls


#============================================
class _ObservedTab:
	"""Supply the shared chrome with the one observation it requires."""

	requires_refresh = False
	title = "shared-seams.cdml"

	def observe_properties(self) -> object:
		"""Return a stable document-free observation for mode presentation."""
		return ("revision", 0)


#============================================
def test_ordinary_window_uses_shared_declarative_clients(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The running product reuses shared actions in its mode, property, and zoom UI."""
	del qapp
	window = ferrum_qt.main_window.MainWindow(object())
	tab = _ObservedTab()
	window._active_native_tab = lambda: tab
	try:
		assert isinstance(
			window._shared_mode_toolbar, ferrum_qt.ferrum.authoring_ribbon.AuthoringRibbon,
		)
		assert isinstance(window._native_property_dock, ferrum_qt.widgets.property_dock.PropertyDock)
		assert isinstance(window.statusBar(), ferrum_qt.widgets.status_bar.StatusBar)
		assert isinstance(window._shared_zoom_controls, ferrum_qt.widgets.zoom_controls.ZoomControls)
		for action_id in (
				"view.zoom_page", "view.zoom_content", "edit.atom_properties",
				"edit.bond_properties",
		):
			assert window._action_registry.get_qt_action(action_id) is not None
		window._legacy_property_dock = None
		window._native_property_dock = None
		assert window.statusBar().context_message == ""
	finally:
		window._cancel_atom_insertion()
		window.close()
