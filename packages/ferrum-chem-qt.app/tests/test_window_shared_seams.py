"""Live-window coverage for Ferrum's shared frontend seam clients."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.ferrum.authoring_ribbon
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.property_observation
import ferrum_qt.ferrum.window_shared_seams
import ferrum_qt.widgets.property_dock
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls


#============================================
def test_disposed_document_reports_typed_unavailable_properties(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A real retired Rust tab cannot expose a stale property observation."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "typed-observation.cdml",
	)
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
def test_ordinary_window_uses_shared_declarative_clients(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The running product reuses shared actions in its mode, property, and zoom UI."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "shared-seams.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		qapp.processEvents()
		assert isinstance(
			window._shared_mode_toolbar, ferrum_qt.ferrum.authoring_ribbon.AuthoringRibbon,
		)
		assert isinstance(window._native_property_dock, ferrum_qt.widgets.property_dock.PropertyDock)
		assert len(window.findChildren(ferrum_qt.widgets.property_dock.PropertyDock)) == 1
		assert isinstance(window.statusBar(), ferrum_qt.widgets.status_bar.StatusBar)
		assert isinstance(window._shared_zoom_controls, ferrum_qt.widgets.zoom_controls.ZoomControls)
		for action_id in (
				"view.zoom_page", "view.zoom_content", "edit.atom_properties",
				"edit.bond_properties",
		):
			assert window._action_registry.get_qt_action(action_id) is not None
		window._action_registry.get_qt_action("mode.draw").trigger()
		qapp.processEvents()
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
