"""Lifecycle behavior for the explicit Template Catalog controller."""

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.themes.theme_loader


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
