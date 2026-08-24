"""Public Qt coverage for Rust-owned catalog placement."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.catalog_palette
import ferrum_qt.ferrum.document_installation
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the ordinary offscreen application host."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _canvas(window: PySide6.QtWidgets.QMainWindow) -> PySide6.QtWidgets.QGraphicsView:
	"""Find the product's declared drawing canvas through public Qt metadata."""
	return next(
		view for view in window.findChildren(PySide6.QtWidgets.QGraphicsView)
		if view.accessibleName() == "Ferrum drawing canvas"
	)


#============================================
def _action(window: PySide6.QtWidgets.QMainWindow,
		text: str) -> PySide6.QtGui.QAction:
	"""Find one visible application action by its public label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == text)


#============================================
def _catalog_key(qtbot: object,
		window: PySide6.QtWidgets.QMainWindow) -> str:
	"""Choose one matching catalog entry through its public dialog controls."""
	palette = ferrum_qt.ferrum.catalog_palette.FerrumCatalogPalette(window)
	try:
		search = next(
			widget for widget in palette.findChildren(PySide6.QtWidgets.QLineEdit)
			if widget.accessibleName() == "Search templates"
		)
		results = next(
			widget for widget in palette.findChildren(PySide6.QtWidgets.QListWidget)
			if widget.accessibleName() == "Ferrum template results"
		)
		search.setText("benzene")
		assert results.currentItem() is not None
		assert "benzene" in results.currentItem().text().lower()
		place_button = next(
			button for button in palette.findChildren(PySide6.QtWidgets.QPushButton)
			if button.text() == "Place on Canvas"
		)
		with qtbot.waitSignal(palette.finished, timeout=1000):
			PySide6.QtTest.QTest.mouseClick(
				place_button, PySide6.QtCore.Qt.MouseButton.LeftButton,
			)
		key = palette.selected_key()
		assert type(key) is str and key
		return key
	finally:
		palette.deleteLater()


#============================================
def test_catalog_placement_receipt_enables_the_next_canvas_interaction(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""A catalog receipt fences one ordinary follow-up selection interaction."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	qtbot.addWidget(window)
	window.show()
	try:
		key = _catalog_key(qtbot, window)
		canvas = _canvas(window)
		point = canvas.viewport().rect().center()
		with qtbot.waitSignal(window.document_installation_completed, timeout=10000) as completed:
			assert window.start_catalog_placement(key)
			PySide6.QtTest.QTest.mouseClick(
				canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
			)
		receipt = completed.args[0]
		assert type(receipt) is ferrum_qt.ferrum.document_installation.FerrumDocumentInstallationV1
		assert receipt.installation_kind == "catalog_template"
		assert receipt.current_revision > receipt.source_revision
		assert receipt.current_digest_hex != receipt.source_digest_hex
		assert receipt.installed_record_count == 1
		assert receipt.accessible_summary == "Ferrum installed one catalog template."

		select = _action(window, "Select Structure")
		assert select.isEnabled()
		select.trigger()
		assert select.isChecked()
		PySide6.QtTest.QTest.mouseClick(
			canvas.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
		)
	finally:
		window.close()
		window.deleteLater()
