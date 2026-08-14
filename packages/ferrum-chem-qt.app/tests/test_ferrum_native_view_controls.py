"""Public behavior coverage for display-only Rust-native View controls."""

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
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.native.ferrum_native_coordinate_generation
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_graphics_view
import ferrum_qt.native.ferrum_native_main_window
import ferrum_qt.native.ferrum_native_statusbar_view_controls


_MOLECULE_CDML = """<cdml version='26.08'>
<molecule id='mol-1'><atom id='atom-c' name='C'><point x='10' y='20'/></atom>
<atom id='atom-o' name='O'><point x='40' y='20'/></atom>
<bond id='bond-co' start='atom-c' end='atom-o' type='n2'/></molecule>
<plus id='plus-1'><point x='80' y='40'/></plus></cdml>"""


_EMPTY_CDML = "<cdml version='26.08'/>"


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one deterministic offscreen Qt application."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _open_tab(
		qapp: PySide6.QtWidgets.QApplication, cdml: str,
		) -> tuple[object, object]:
	"""Show one host and current native tab through public host registration."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		cdml, "view-controls.cdml",
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	qapp.processEvents()
	return window, tab


#============================================
def _close_window(window: object) -> None:
	"""Retire only clean test tabs through the normal native close path."""
	while window._tab_widget.count():
		window._close_tab_at(0)
	window.close()
	window.deleteLater()


#============================================
def _same_transform(left: object, right: object) -> bool:
	"""Compare view transforms with normal floating-point tolerance."""
	return all(
		abs(first - second) < 1.0e-9
		for first, second in zip(
			(left.m11(), left.m12(), left.m13(), left.m21(), left.m22(), left.m23()),
			(right.m11(), right.m12(), right.m13(), right.m21(), right.m22(), right.m23()),
			strict=True,
		)
	)


#============================================
def _status_controls(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Find the installed permanent View client through the public status-bar tree."""
	controls_type = (
		ferrum_qt.native.ferrum_native_statusbar_view_controls.
		FerrumNativeStatusBarViewControls
	)
	for controls in window.statusBar().findChildren(controls_type):
		return controls
	raise AssertionError("Native status bar has no View controls.")


#============================================
def _status_button(controls: object, accessible_name: str) -> PySide6.QtWidgets.QToolButton:
	"""Find one visible status action through its stable user-facing accessible name."""
	for button in controls.findChildren(PySide6.QtWidgets.QToolButton):
		if button.accessibleName() == accessible_name:
			return button
	raise AssertionError(f"Native View status control is missing {accessible_name!r}.")


#============================================
def _status_slider(controls: object) -> PySide6.QtWidgets.QSlider:
	"""Find the continuous zoom client through its user-facing accessible name."""
	for slider in controls.findChildren(PySide6.QtWidgets.QSlider):
		if slider.accessibleName() == "Zoom percentage slider":
			return slider
	raise AssertionError("Native View status controls have no zoom percentage slider.")


#============================================
def test_page_content_and_identity_controls_use_public_document_geometry(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Page and content frame their public geometry, while 100 percent is identity."""
	window, tab = _open_tab(qapp, _MOLECULE_CDML)
	try:
		window._zoom_page_action.trigger()
		page_transform = tab.view.transform()
		window._zoom_content_action.trigger()
		assert tab.view.transform().m11() > page_transform.m11()
		window._zoom_100_action.trigger()
		assert tab.view.transform().isIdentity()
	finally:
		_close_window(window)


#============================================
def test_empty_content_is_exact_page_fallback(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An empty native document treats Content exactly as Page."""
	window, tab = _open_tab(qapp, _EMPTY_CDML)
	try:
		assert tab.document_content_bounds() is None
		window._zoom_page_action.trigger()
		page_transform = tab.view.transform()
		window._zoom_100_action.trigger()
		window._zoom_content_action.trigger()
		assert _same_transform(tab.view.transform(), page_transform)
	finally:
		_close_window(window)


#============================================
def test_zoom_preserves_center_without_changing_native_or_selection_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Display scaling leaves the current Rust receipt, scene, and selection intact."""
	window, tab = _open_tab(qapp, _MOLECULE_CDML)
	try:
		tab.select_atom("atom-c")
		before_snapshot = tab.current_snapshot
		before_scene = tab.view.scene()
		before_selection_id = tab.selected_atom_projection().source_id
		center = tab.view.mapToScene(tab.view.viewport().rect().center())
		window._zoom_in_action.trigger()
		window._zoom_out_action.trigger()
		after_center = tab.view.mapToScene(tab.view.viewport().rect().center())
		assert (after_center - center).manhattanLength() < 1.0e-6
		assert (
			tab.current_snapshot == before_snapshot
			and tab.view.scene() is before_scene
			and tab.selected_atom_projection().source_id == before_selection_id
		)
	finally:
		_close_window(window)


#============================================
def test_wheel_zoom_preserves_cursor_anchor_and_durable_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A real viewport wheel event changes only the active tab's display transform."""
	window, tab = _open_tab(qapp, _MOLECULE_CDML)
	try:
		tab.select_atom("atom-c")
		tab.view.resetTransform()
		tab.view.scale(3.0, 3.0)
		tab.view.centerOn(PySide6.QtCore.QPointF(80.0, 40.0))
		qapp.processEvents()
		zoom_100 = _status_button(_status_controls(window), "Reset zoom to 100%")
		before_percent = zoom_100.text()
		viewport_position = PySide6.QtCore.QPointF(40.0, 35.0)
		before_scene_position = tab.view.mapToScene(viewport_position.toPoint())
		before_snapshot = tab.current_snapshot
		before_selection = tab.selected_atom_projection().source_id
		before_scale = tab.view.transform().m11()
		event = PySide6.QtGui.QWheelEvent(
			viewport_position, viewport_position,
			PySide6.QtCore.QPoint(), PySide6.QtCore.QPoint(0, 120),
			PySide6.QtCore.Qt.MouseButton.NoButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			PySide6.QtCore.Qt.ScrollPhase.ScrollUpdate, False,
		)
		PySide6.QtWidgets.QApplication.sendEvent(tab.view.viewport(), event)
		after_scene_position = tab.view.mapToScene(viewport_position.toPoint())
		assert (
			tab.view.transform().m11() > before_scale
			and zoom_100.text() != before_percent
			and (after_scene_position - before_scene_position).manhattanLength() < 1.0e-6
		)
		assert (
			tab.current_snapshot == before_snapshot
			and tab.view.scene() is not None
			and tab.selected_atom_projection().source_id == before_selection
		)
	finally:
		_close_window(window)


#============================================
def test_retained_zoom_shortcuts_dispatch_existing_view_actions(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The retained Ctrl zoom keys operate only the active native display."""
	window, tab = _open_tab(qapp, _MOLECULE_CDML)
	try:
		tab.view.setFocus()
		before_scale = tab.view.transform().m11()
		modifier = PySide6.QtCore.Qt.KeyboardModifier.ControlModifier
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Plus, modifier)
		assert tab.view.transform().m11() > before_scale
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Minus, modifier)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_0, modifier)
		assert tab.view.transform().isIdentity()
	finally:
		_close_window(window)


#============================================
def test_each_tab_retains_its_own_completed_view_transform(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Returning to a completed tab retains its transform rather than refitting it."""
	window, first = _open_tab(qapp, _MOLECULE_CDML)
	second = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_MOLECULE_CDML, "second.cdml",
	)
	try:
		window._zoom_in_action.trigger()
		first_transform = first.view.transform()
		window._register_native_tab(second, activate=True)
		qapp.processEvents()
		window._zoom_out_action.trigger()
		window._tab_widget.setCurrentIndex(window._tab_widget.indexOf(first))
		qapp.processEvents()
		assert _same_transform(first.view.transform(), first_transform)
	finally:
		_close_window(window)


#============================================
def test_hidden_first_fit_retries_on_the_next_window_show(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A hidden queued first fit is discarded and later show requests it again."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_MOLECULE_CDML, "hidden.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		window.hide()
		qapp.processEvents()
		assert tab.view.transform().isIdentity()
		window.show()
		qapp.processEvents()
		assert not tab.view.transform().isIdentity()
	finally:
		_close_window(window)


#============================================
def test_removed_and_zero_page_hosts_leave_display_controls_unavailable(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Queued display work cannot revive a removed tab and empty hosts disable actions."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_MOLECULE_CDML, "removed.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		window._close_tab_at(window._tab_widget.indexOf(tab))
		qapp.processEvents()
		assert not window._zoom_page_action.isEnabled()
		assert not window._zoom_content_action.isEnabled()
	finally:
		_close_window(window)


#============================================
def test_live_scene_view_controls_stay_enabled_for_pending_and_busy_chemistry(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Display reachability survives stale rendering and an active inspection action."""
	window, tab = _open_tab(qapp, _MOLECULE_CDML)
	try:
		tab.select_atom("atom-c")
		tab.change_selected_atom_element("N")
		replace = ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjectionController.replace
		monkeypatch.setattr(
			ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjectionController,
			"replace", lambda _self, _observation, _latch: False,
		)
		with pytest.raises(
			ferrum_qt.native.ferrum_native_document_tab.
			FerrumNativeDocumentTabMutationPresentationError,
		):
			tab.undo()
		window._refresh_actions()
		assert tab.requires_refresh and window._zoom_page_action.isEnabled()
		monkeypatch.setattr(
			ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjectionController,
			"replace", replace,
		)
		tab.refresh_authoritative()
		choice = tab.durable_molecule_choices()[0]
		snapshot = tab.current_snapshot
		worker = (
			ferrum_qt.native.ferrum_native_coordinate_generation.
			FerrumNativeCoordinatePreparationWorker(
				tab.current_document_observation(), choice.object_id,
			)
		)
		window._coordinate_generation_intent = (
			ferrum_qt.native.ferrum_native_coordinate_generation.
			FerrumNativeCoordinateGenerationIntent(
				tab, snapshot.revision, snapshot.digest, worker,
			)
		)
		window._refresh_actions()
		slider = _status_slider(_status_controls(window))
		assert (
			window._cancel_coordinates_action.isEnabled()
			and window._zoom_in_action.isEnabled()
			and window._zoom_out_action.isEnabled()
			and window._zoom_100_action.isEnabled()
			and window._zoom_content_action.isEnabled()
			and slider.isEnabled()
		)
		window._coordinate_generation_intent = None
		window._refresh_actions()
	finally:
		window._coordinate_generation_intent = None
		if tab.requires_refresh:
			monkeypatch.undo()
			tab.refresh_authoritative()
		_close_window(window)


#============================================
def test_status_controls_dispatch_view_actions_without_mutating_the_document(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Status controls change display state while preserving durable state."""
	window, tab = _open_tab(qapp, _MOLECULE_CDML)
	try:
		tab.select_atom("atom-c")
		controls = _status_controls(window)
		zoom_100 = _status_button(controls, "Reset zoom to 100%")
		zoom_in = _status_button(controls, "Zoom in")
		zoom_page = _status_button(controls, "Zoom to Page")
		zoom_content = _status_button(controls, "Zoom to Content")
		before_snapshot = tab.current_snapshot
		before_selection = tab.selected_atom_projection().source_id
		zoom_in.click()
		zoom_page.click()
		assert zoom_100.text() == f"{_status_slider(controls).value()}%"
		zoom_content.click()
		zoom_100.click()
		assert (
			zoom_100.text() == "100%"
			and tab.current_snapshot == before_snapshot
			and tab.selected_atom_projection().source_id == before_selection
		)
	finally:
		_close_window(window)


#============================================
def test_status_slider_sets_absolute_zoom_without_changing_durable_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The upstream continuous control becomes only an absolute native view client."""
	window, tab = _open_tab(qapp, _MOLECULE_CDML)
	try:
		tab.select_atom("atom-c")
		controls = _status_controls(window)
		slider = _status_slider(controls)
		before_snapshot = tab.current_snapshot
		before_scene = tab.view.scene()
		before_selection = tab.selected_atom_projection().source_id
		before_center = (
			tab.view.mapToScene(tab.view.viewport().rect()).boundingRect().center()
		)
		slider.setValue(275)
		after_center = (
			tab.view.mapToScene(tab.view.viewport().rect()).boundingRect().center()
		)
		assert (
			ferrum_qt.native.ferrum_native_graphics_view.
			effective_zoom_percent(tab.view)
		) == pytest.approx(275.0) and (
			after_center - before_center
		).manhattanLength() <= 1.0
		assert (
			tab.current_snapshot == before_snapshot
			and tab.view.scene() is before_scene
			and tab.selected_atom_projection().source_id == before_selection
		)
	finally:
		_close_window(window)


#============================================
def test_status_controls_retain_each_active_tab_percent_and_empty_page_fallback(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The action client observes only the active tab and exact empty Content fallback."""
	window, first = _open_tab(qapp, _MOLECULE_CDML)
	second = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "empty.cdml",
	)
	try:
		controls = _status_controls(window)
		zoom_in = _status_button(controls, "Zoom in")
		zoom_100 = _status_button(controls, "Reset zoom to 100%")
		zoom_page = _status_button(controls, "Zoom to Page")
		zoom_content = _status_button(controls, "Zoom to Content")
		slider = _status_slider(controls)
		zoom_in.click()
		first_percent = zoom_100.text()
		first_slider = slider.value()
		window._register_native_tab(second, activate=True)
		qapp.processEvents()
		zoom_page.click()
		page_percent = zoom_100.text()
		zoom_100.click()
		zoom_content.click()
		assert zoom_100.text() == page_percent
		window._tab_widget.setCurrentIndex(window._tab_widget.indexOf(first))
		qapp.processEvents()
		assert zoom_100.text() == first_percent and slider.value() == first_slider
	finally:
		_close_window(window)


#============================================
def test_status_controls_cover_unavailable_and_keyboard_recovery_states(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An unsupported transform keeps one keyboard-reachable reset path."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	try:
		controls = _status_controls(window)
		zoom_100 = _status_button(controls, "Reset zoom to 100%")
		slider = _status_slider(controls)
		tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
			_MOLECULE_CDML, "invalid-transform.cdml",
		)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tab.view.rotate(10.0)
		window._refresh_native_view_status()
		unsupported = tab.view.transform()
		assert zoom_100.text() == "--" and zoom_100.isEnabled() and not slider.isEnabled()
		slider.setValue(250)
		zoom_100.setFocus()
		PySide6.QtTest.QTest.keyClick(
			zoom_100, PySide6.QtCore.Qt.Key.Key_Space,
		)
		assert (
			tab.view.transform() != unsupported
			and tab.view.transform().isIdentity()
			and zoom_100.text() == "100%"
			and slider.isEnabled()
		)
	finally:
		_close_window(window)


#============================================
def test_status_controls_reject_a_disposed_registered_tab(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A retired tab cannot keep either menu or status display controls reachable."""
	window, tab = _open_tab(qapp, _MOLECULE_CDML)
	try:
		tab.dispose()
		window._refresh_actions()
		assert not window._zoom_page_action.isEnabled()
		zoom_100 = _status_button(_status_controls(window), "Reset zoom to 100%")
		assert zoom_100.text() == "--" and not zoom_100.isEnabled()
	finally:
		window._native_tabs_by_page.pop(tab, None)
		window._tab_widget.removeTab(window._tab_widget.indexOf(tab))
		window.close()
		window.deleteLater()


#============================================
def test_effective_zoom_percent_accepts_only_exact_uniform_affine_transforms(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The display observer accepts translation but rejects unsupported matrix forms."""
	view = PySide6.QtWidgets.QGraphicsView()
	try:
		valid = PySide6.QtGui.QTransform(1.25, 0.0, 0.0, 0.0, 1.25, 0.0, 3.0, 4.0, 1.0)
		view.setTransform(valid)
		assert ferrum_qt.native.ferrum_native_graphics_view.effective_zoom_percent(
			view,
		) == 125.0
		results = []
		for transform in (
				PySide6.QtGui.QTransform(1.0, 0.1, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
				PySide6.QtGui.QTransform(1.0, 0.0, 0.1, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
				PySide6.QtGui.QTransform(1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0),
				PySide6.QtGui.QTransform(-1.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0),
				PySide6.QtGui.QTransform(float("inf"), 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0),
			):
			view.setTransform(transform)
			results.append(
				ferrum_qt.native.ferrum_native_graphics_view.effective_zoom_percent(view),
			)
		assert all(result is None for result in results)
	finally:
		view.deleteLater()
		del qapp
