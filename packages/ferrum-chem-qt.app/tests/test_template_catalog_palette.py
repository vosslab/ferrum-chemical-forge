"""Visible Rust-owned shipped-template authoring through Ferrum Qt."""

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_qt.ferrum.catalog_palette
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _ribbon_exposes_action(window: object, action: object) -> bool:
	"""Return whether one visible Authoring Ribbon button owns the QAction."""
	return any(
		button.defaultAction() is action
		for button in window._authoring_ribbon.findChildren(
			PySide6.QtWidgets.QToolButton,
		)
	)


def test_catalog_palette_filters_rust_summaries_and_places_benzene(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Search uses immutable Rust facts and pointer placement carries opaque handles."""
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(qapp)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	try:
		tab = window._active_native_tab()
		assert tab is not None
		assert _ribbon_exposes_action(window, window._insert_catalog_template_action)
		palette = ferrum_qt.ferrum.catalog_palette.FerrumCatalogPalette(window)
		palette.search.setText("benzene")
		qapp.processEvents()
		assert palette.selected_key() == "system/rings/benzene"
		assert "benzene" in palette.results.currentItem().text().lower()
		assert "provenance" in palette.results.currentItem().toolTip().lower()
		assert window.start_catalog_placement(palette.selected_key())
		window.show()
		qapp.processEvents()
		before = tab.current_snapshot.revision
		point = _point(tab, 80.0, 60.0)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), point)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before
		PySide6.QtTest.QTest.mouseClick(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before + 1
		assert "Benzene" in tab.current_snapshot.cdml
	finally:
		window.close()
		window.deleteLater()

