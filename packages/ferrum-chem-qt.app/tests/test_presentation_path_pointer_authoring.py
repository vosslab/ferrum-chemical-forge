"""Visible Rust-owned multi-point path authoring through Ferrum Qt."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


#============================================
def _point(tab: object, x: float, y: float) -> PySide6.QtCore.QPoint:
	"""Map one backend scene coordinate through the visible viewport seam."""
	return tab.view.mapFromScene(PySide6.QtCore.QPointF(x, y))


#============================================
def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return the one visible QAction that owns one path authoring tool."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == text)


#============================================
def test_path_actions_collect_ordered_clicks_and_commit_one_rust_root(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Visible path actions commit ordered points through Enter and double-click."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml' version='26.07'/>", "paths.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		before_cdml = tab.current_snapshot.cdml
		for action_text, points, completes_with_double_click, marker in (
			("Draw Polyline", ((20.0, 20.0), (60.0, 40.0)), False, "<polyline"),
			("Draw Polygon", ((100.0, 20.0), (140.0, 20.0), (120.0, 60.0)), True, "<polygon"),
		):
			before = tab.current_snapshot.revision
			action = _action(window, action_text)
			action.trigger()
			for x, y in points[:-1] if completes_with_double_click else points:
				PySide6.QtTest.QTest.mouseClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, x, y),
				)
			qapp.processEvents()
			if completes_with_double_click:
				x, y = points[-1]
				PySide6.QtTest.QTest.mouseDClick(
					tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
					PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, x, y),
				)
			else:
				PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
			qapp.processEvents()
			assert tab.current_snapshot.revision == before + 1
			assert marker in tab.current_snapshot.cdml
			accepted_cdml = tab.current_snapshot.cdml
			tab.undo()
			assert tab.current_snapshot.cdml == before_cdml
			tab.redo()
			assert tab.current_snapshot.cdml == accepted_cdml
			before_cdml = accepted_cdml
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_path_actions_keep_incomplete_polygon_armed_until_double_click_completes(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Escape cancels, while incomplete Polygon authoring stays available to finish."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml' version='26.07'/>", "path-refusals.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		baseline = tab.current_snapshot
		polyline_action = _action(window, "Draw Polyline")
		polyline_action.trigger()
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 20.0, 20.0),
		)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert tab.current_snapshot == baseline
		assert not polyline_action.isChecked()
		polygon_action = _action(window, "Draw Polygon")
		polygon_action.trigger()
		for x, y in ((100.0, 20.0), (140.0, 20.0)):
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, x, y),
			)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		qapp.processEvents()
		assert tab.current_snapshot == baseline
		assert polygon_action.isChecked()
		assert "needs 1 more point" in window.statusBar().currentMessage()
		before = tab.current_snapshot.revision
		PySide6.QtTest.QTest.mouseDClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 120.0, 60.0),
		)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before + 1
		assert "<polygon" in tab.current_snapshot.cdml
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_curved_electron_arrow_action_commits_one_rust_root_and_escape_cancels(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The public three-click action creates history that visible Undo and Redo control."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml' version='26.07'/>", "electron-arrow.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		action = _action(window, "Draw Curved Electron Arrow")
		undo_action = _action(window, "Undo")
		redo_action = _action(window, "Redo")
		action.trigger()
		for x, y in ((20.0, 20.0), (50.0, 50.0), (80.0, 20.0)):
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, x, y),
			)
		qapp.processEvents()
		assert undo_action.isEnabled()
		undo_action.trigger()
		qapp.processEvents()
		assert redo_action.isEnabled()
		redo_action.trigger()
		qapp.processEvents()
		assert undo_action.isEnabled()
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 120.0, 20.0),
		)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert not action.isChecked()
		assert undo_action.isEnabled()
	finally:
		window.close()
		window.deleteLater()
