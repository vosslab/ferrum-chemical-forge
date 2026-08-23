"""Visible Rust-owned multi-point path authoring through Ferrum Qt."""

# Standard Library
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_chem
import ferrum_qt.canvas.ferrum_presentation_projection
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.presentation_creation_preview
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
def _startup_tab(
		window: PySide6.QtWidgets.QMainWindow,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return the initial document through the visible window's public tab surface."""
	tabs = window.centralWidget()
	if type(tabs) is not PySide6.QtWidgets.QTabWidget:
		raise RuntimeError("Ferrum window has no visible document tab widget")
	tab = tabs.currentWidget()
	if type(tab) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		raise RuntimeError("Ferrum window did not admit its startup native document")
	return tab


#============================================
def _curved_equilibrium_preview_items(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[PySide6.QtWidgets.QGraphicsPathItem, PySide6.QtWidgets.QGraphicsPathItem]:
	"""Return the visible dashed lanes and filled heads from one Rust-issued preview."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum curved-equilibrium preview has no visible scene")
	axis_item = next(
		item for item in scene.items()
		if type(item) is PySide6.QtWidgets.QGraphicsPathItem
		and item.pen().style() is PySide6.QtCore.Qt.PenStyle.DashLine
		and item.brush().style() is PySide6.QtCore.Qt.BrushStyle.NoBrush
	)
	head_item = next(
		item for item in scene.items()
		if type(item) is PySide6.QtWidgets.QGraphicsPathItem
		and item.pen().style() is PySide6.QtCore.Qt.PenStyle.NoPen
		and item.brush().style() is not PySide6.QtCore.Qt.BrushStyle.NoBrush
	)
	return axis_item, head_item


#============================================
def _curve_count(path: PySide6.QtGui.QPainterPath) -> int:
	"""Count cubic segments already issued in a Qt path without calculating one."""
	return sum(
		path.elementAt(index).type is PySide6.QtGui.QPainterPath.ElementType.CurveToElement
		for index in range(path.elementCount())
	)


#============================================
def _subpath_count(path: PySide6.QtGui.QPainterPath) -> int:
	"""Count closed issued head polygons without interpreting their coordinates."""
	return sum(
		path.elementAt(index).type is PySide6.QtGui.QPainterPath.ElementType.MoveToElement
		for index in range(path.elementCount())
	)


#============================================
def _selected_curved_equilibrium_arrow(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> object:
	"""Select and return the typed committed arrow through public Rust observations."""
	arrow = next(
		root.arrow
		for root in tab.current_document_observation().projection.presentation_stack.roots
		if root.kind == "arrow" and root.arrow.geometry.kind == "curved_equilibrium"
	)
	source_id = arrow.target.source_id
	assert source_id is not None
	observation = tab.observe_direct_root_interaction()
	selection = tab.select_direct_roots(
		observation, None,
		ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(
			source_id,
			ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace,
		),
	)
	assert selection.roots[0].identifier == source_id
	return arrow


#============================================
def _curved_equilibrium_points(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[PySide6.QtCore.QPoint, PySide6.QtCore.QPoint, PySide6.QtCore.QPoint]:
	"""Return one visible, well-spaced three-click curved-equilibrium gesture."""
	start = tab.view.viewport().rect().center()
	return start, start + PySide6.QtCore.QPoint(40, 20), start + PySide6.QtCore.QPoint(80, 0)


#============================================
def _click_curved_equilibrium_arrow(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		points: tuple[PySide6.QtCore.QPoint, ...],
		) -> None:
	"""Send visible pointer clicks to the currently armed curved-equilibrium action."""
	for point in points:
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
		)


#============================================
def _snapshot_identity(snapshot: object) -> tuple[int, str]:
	"""Return the public durable identity of one native document snapshot."""
	return snapshot.revision, snapshot.cdml


#============================================
def _commit_curved_equilibrium_arrow(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		action: PySide6.QtGui.QAction,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Commit one valid curved-equilibrium arrow through its public QAction."""
	action.trigger()
	_click_curved_equilibrium_arrow(tab, _curved_equilibrium_points(tab))
	qapp.processEvents()


#============================================
def test_path_actions_collect_ordered_clicks_and_commit_one_rust_root(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Visible path actions commit ordered points through Enter and double-click."""
	window = ferrum_qt.main_window.MainWindow(object())
	try:
		window.show()
		qapp.processEvents()
		tab = _startup_tab(window)
		undo_action = _action(window, "Undo")
		redo_action = _action(window, "Redo")
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
			undo_action.trigger()
			qapp.processEvents()
			assert tab.current_snapshot.cdml == before_cdml
			redo_action.trigger()
			qapp.processEvents()
			assert tab.current_snapshot.cdml == accepted_cdml
			before_cdml = accepted_cdml
		path = tmp_path / "presentation-paths.cdml"
		tab.save_atomic(path)
		reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
		assert reopened.snapshot().digest == tab.current_snapshot.digest
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_path_actions_keep_incomplete_polygon_armed_until_double_click_completes(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Escape cancels, while incomplete Polygon authoring stays available to finish."""
	window = ferrum_qt.main_window.MainWindow(object())
	try:
		window.show()
		qapp.processEvents()
		tab = _startup_tab(window)
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
def test_path_action_keeps_typed_invalid_point_refusal_nonmutating_and_correctable(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A Rust geometry refusal keeps the visible path tool ready for correction."""
	window = ferrum_qt.main_window.MainWindow(object())
	try:
		window.show()
		qapp.processEvents()
		tab = _startup_tab(window)
		baseline = tab.current_snapshot
		action = _action(window, "Draw Polyline")
		action.trigger()
		point = _point(tab, 20.0, 20.0)
		for click_point in (point, point):
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, click_point,
		)
		qapp.processEvents()
		assert tab.current_snapshot == baseline
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 60.0, 40.0),
		)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Return)
		qapp.processEvents()
		assert tab.current_snapshot.revision == baseline.revision + 1
	finally:
		window.close()
		window.deleteLater()


#============================================
@pytest.mark.parametrize(("action_text", "arrow_type"), (
	("Draw Curved Electron Arrow", "electron"),
	("Draw Curved Retro Arrow", "retro"),
	("Draw Curved Reaction Arrow", "curved-normal"),
))
def test_curved_terminal_arrow_action_commits_one_rust_root_and_escape_cancels(
		action_text: str, arrow_type: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Each public three-click terminal-arrow action creates visible Undo/Redo history."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml' version='26.07'/>", "electron-arrow.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		action = _action(window, action_text)
		undo_action = _action(window, "Undo")
		redo_action = _action(window, "Redo")
		action.trigger()
		assert action.isChecked()
		for x, y in ((20.0, 20.0), (50.0, 50.0), (80.0, 20.0)):
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, x, y),
			)
		qapp.processEvents()
		assert f'type="{arrow_type}"' in tab.current_snapshot.cdml, window.statusBar().currentMessage()
		assert undo_action.isEnabled()
		undo_action.trigger()
		qapp.processEvents()
		assert redo_action.isEnabled()
		redo_action.trigger()
		qapp.processEvents()
		assert undo_action.isEnabled()
		accepted_cdml = tab.current_snapshot.cdml
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, _point(tab, 120.0, 20.0),
		)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert not action.isChecked()
		assert tab.current_snapshot.cdml == accepted_cdml
		assert undo_action.isEnabled()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_curved_equilibrium_preview_uses_rust_cubics_and_filled_issued_heads(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The real Qt scene separates dashed Rust lanes from its two filled Rust heads."""
	window = ferrum_qt.main_window.MainWindow(object())
	try:
		window.show()
		qapp.processEvents()
		tab = _startup_tab(window)
		start = tab.view.viewport().rect().center()
		control = start + PySide6.QtCore.QPoint(40, 20)
		end = start + PySide6.QtCore.QPoint(80, 0)
		start_scene = tab.view.mapToScene(start)
		control_scene = tab.view.mapToScene(control)
		end_scene = tab.view.mapToScene(end)
		gesture = tab.begin_curved_equilibrium_arrow_gesture(
			(float(start_scene.x()), float(start_scene.y())),
			(float(control_scene.x()), float(control_scene.y())),
		)
		preview = tab.preview_curved_equilibrium_arrow_gesture(
			gesture, (float(end_scene.x()), float(end_scene.y())),
		)
		ferrum_qt.ferrum.presentation_creation_preview.create_curved_equilibrium_arrow_overlay(
			tab, preview.overlay,
		)
		axis_item, head_item = _curved_equilibrium_preview_items(tab)
		assert _curve_count(axis_item.path()) == 2
		assert _subpath_count(head_item.path()) == 2
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_curved_equilibrium_arrow_action_escape_cancels_armed_gesture(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Escape returns an armed visible curved-equilibrium gesture to its baseline."""
	window = ferrum_qt.main_window.MainWindow(object())
	try:
		window.show()
		qapp.processEvents()
		tab = _startup_tab(window)
		action = _action(window, "Draw Curved Equilibrium Arrow")
		baseline = _snapshot_identity(tab.current_document_observation().snapshot)
		action.trigger()
		_click_curved_equilibrium_arrow(tab, _curved_equilibrium_points(tab)[:2])
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert _snapshot_identity(tab.current_document_observation().snapshot) == baseline
		assert not action.isChecked()
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_curved_equilibrium_arrow_action_commits_typed_projection_and_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A committed visible action has one selected typed Rust projection."""
	window = ferrum_qt.main_window.MainWindow(object())
	try:
		window.show()
		qapp.processEvents()
		tab = _startup_tab(window)
		_commit_curved_equilibrium_arrow(
			tab, _action(window, "Draw Curved Equilibrium Arrow"), qapp,
		)
		selected_arrow = _selected_curved_equilibrium_arrow(tab)
		arrow_item = next(
			item for item in tab.view.scene().items()
			if type(item) is ferrum_qt.canvas.ferrum_presentation_projection.ArrowProjectionItem
		)
		assert selected_arrow.geometry.kind == "curved_equilibrium"
		assert (_curve_count(arrow_item.axis_path), _subpath_count(arrow_item.head_path)) == (2, 2)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_curved_equilibrium_arrow_undo_redo_restores_typed_root(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Undo then Redo restores the committed curved-equilibrium root through public actions."""
	window = ferrum_qt.main_window.MainWindow(object())
	try:
		window.show()
		qapp.processEvents()
		tab = _startup_tab(window)
		_commit_curved_equilibrium_arrow(
			tab, _action(window, "Draw Curved Equilibrium Arrow"), qapp,
		)
		_action(window, "Undo").trigger()
		qapp.processEvents()
		_action(window, "Redo").trigger()
		qapp.processEvents()
		assert _selected_curved_equilibrium_arrow(tab).geometry.kind == "curved_equilibrium"
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_curved_equilibrium_arrow_invalid_gesture_preserves_durable_document(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A Rust-refused visible gesture leaves the document and tool state unchanged."""
	window = ferrum_qt.main_window.MainWindow(object())
	try:
		window.show()
		qapp.processEvents()
		tab = _startup_tab(window)
		action = _action(window, "Draw Curved Equilibrium Arrow")
		baseline = _snapshot_identity(tab.current_document_observation().snapshot)
		action.trigger()
		refusal_point = _curved_equilibrium_points(tab)[2] + PySide6.QtCore.QPoint(80, 0)
		_click_curved_equilibrium_arrow(tab, (refusal_point, refusal_point, refusal_point))
		qapp.processEvents()
		assert _snapshot_identity(tab.current_document_observation().snapshot) == baseline
		assert not action.isChecked()
	finally:
		window.close()
		window.deleteLater()
