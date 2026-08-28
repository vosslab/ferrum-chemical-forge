"""Visible direct Plus authoring through Ferrum Qt."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_CDML = "<cdml xmlns='urn:ferrum:cdml'><molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom></molecule></cdml>"


def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return one visible action by its user-facing label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == text)


def _close_window(window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Discard exact test tabs before ordinary window shutdown can prompt."""
	window.cancel_active_pointer_authoring()
	for tab in tuple(window._native_tabs_by_page.values()):
		index = window._tab_widget.indexOf(tab)
		result = window._close_native_tab_at(
			index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	assert not window._native_tabs_by_page
	window.close()
	window.deleteLater()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete)
	qapp.processEvents()


def test_plus_click_commits_a_durable_plus_and_undo_removes_it(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A user can place a Plus and undo the visible document change."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "plus.cdml", ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		point = tab.view.mapFromScene(PySide6.QtCore.QPointF(72.0, 36.0))
		_action(window, "Draw Plus").trigger()
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
		)
		qapp.processEvents()
		assert "<plus" in tab.current_snapshot.cdml
		_action(window, "Undo").trigger()
		qapp.processEvents()
		assert "<plus" not in tab.current_snapshot.cdml
	finally:
		_close_window(window, qapp)
