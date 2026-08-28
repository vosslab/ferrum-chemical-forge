"""Lifecycle behavior for the explicit Template Catalog controller."""

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.themes.theme_loader
from ferrum_qt.ferrum.template_catalog_dialog import FerrumTemplateCatalogDialog


#============================================
def test_catalog_controller_cancels_only_its_exact_owner_tab(
		main_window: object,
		) -> None:
	"""A catalog controller never releases pointer ownership from another tab."""
	window = main_window
	first = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "catalog-owner-first.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	second = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "catalog-owner-second.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	window._register_native_tab(first, activate=True)
	window._register_native_tab(second, activate=False)
	controller = window._template_catalog_controller
	assert controller.start_placement(object(), "opaque-key")
	assert not controller.cancel_for_tab(second, "tab_switch")
	assert controller.cancel_for_tab(first, "tab_close")


#============================================
def test_shutdown_cancels_catalog_placement_and_retires_every_bound_tab(
		main_window: object,
		) -> None:
	"""Window shutdown owns catalog cancellation before it retires its clean pages."""
	window = main_window
	first = window._active_native_tab()
	assert first is not None
	second = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "shutdown-second.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	window._register_native_tab(second, activate=False)
	assert window._template_catalog_controller.start_placement(object(), "shutdown")
	assert window.prepare_application_shutdown()
	assert first.is_disposed and second.is_disposed
	with pytest.raises(
			ferrum_qt.ferrum.operation_leases.OperationLeaseError,
			match="not bound",
	):
		window._operation_leases.unregister_tab(first)
	with pytest.raises(
			ferrum_qt.ferrum.operation_leases.OperationLeaseError,
			match="not bound",
	):
		window._operation_leases.unregister_tab(second)


#============================================
def test_catalog_tab_switch_cancels_without_stealing_focus_from_the_new_tab(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		) -> None:
	"""A stale A placement leaves the author on B until they reopen the catalog."""
	window = main_window
	first = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "catalog-switch-first.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	second = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "catalog-switch-second.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	window._register_native_tab(first, activate=True)
	window._register_native_tab(second, activate=False)
	window.show()
	qapp.processEvents()
	controller = window._template_catalog_controller
	before = first.current_snapshot
	assert controller.start_placement(object(), "opaque-key")
	window._tab_widget.setCurrentWidget(second)
	second.view.viewport().setFocus()
	qapp.processEvents()
	window._refresh_actions()
	assert first.current_snapshot is before
	assert window._tab_widget.currentWidget() is second
	assert qapp.focusWidget() is second.view
	assert not window._operation_leases.has_active(
		ferrum_qt.ferrum.operation_leases.OperationFamily.TEMPLATE_CATALOG, first,
	)
	controller.open()
	qapp.processEvents()
	dialog = window.findChild(FerrumTemplateCatalogDialog)
	assert dialog is not None and dialog.isVisible()
