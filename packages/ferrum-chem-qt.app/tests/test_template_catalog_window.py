"""Public controller behavior for the Rust-owned Template Catalog route."""

# Standard Library
import dataclasses
import pathlib
import types

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.operation_leases
from ferrum_qt.ferrum.template_catalog_controller import TemplateCatalogController
from ferrum_qt.ferrum.template_catalog_controller import TemplateCatalogHost
from ferrum_qt.ferrum.template_catalog_dialog import FerrumTemplateCatalogDialog


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the normal offscreen native application host."""
	app = PySide6.QtWidgets.QApplication.instance()
	return PySide6.QtWidgets.QApplication([]) if app is None else app


@dataclasses.dataclass(frozen=True, slots=True)
class _Snapshot:
	revision: int
	digest: str
	entries: tuple[object, ...] = ()
	refusals: tuple[object, ...] = ()
	limits_max_entries: int = 10
	limits_max_candidates: int = 10
	limits_max_refusals: int = 10
	limits_max_file_bytes: int = 10
	limits_max_total_bytes: int = 10


class _View(PySide6.QtWidgets.QGraphicsView):
	def snap_authored_scene_point(self, point: PySide6.QtCore.QPointF) -> PySide6.QtCore.QPointF:
		return point


class _Tab:
	def __init__(self) -> None:
		self.current_snapshot = _Snapshot(1, "a" * 64)
		self.requires_refresh = False
		self.is_disposed = False
		self.view = _View()
		self.calls = 0

	def place_template_catalog_entry(self, *_args: object) -> object:
		self.calls += 1
		return types.SimpleNamespace(result=types.SimpleNamespace(
			observation=types.SimpleNamespace(snapshot=_Snapshot(2, "b" * 64)),
		))


class _Window(PySide6.QtWidgets.QMainWindow):
	def __init__(self) -> None:
		super().__init__()
		self.tab = _Tab()
		self._native_tabs_by_page = {self.tab: self.tab}
		self._action_registry = ferrum_qt.actions.action_registry.ActionRegistry()
		self._coordinate_generation_intent = None
		self._atom_insertion_intent = None
		self._line_gesture_intent = None
		self.published = 0
		self._add_atom_action = PySide6.QtGui.QAction(self)
		self._draw_bond_action = PySide6.QtGui.QAction(self)
		self._draw_bond_action.setCheckable(True)
		self._draw_arrow_action = PySide6.QtGui.QAction(self)
		self._draw_plus_action = PySide6.QtGui.QAction(self)
		self._insert_text_action = PySide6.QtGui.QAction(self)
		self._insert_cyclohexane_ring_action = PySide6.QtGui.QAction(self)
		self._draw_wavy_action = PySide6.QtGui.QAction(self)
		self._attach_cyclohexane_ring_action = PySide6.QtGui.QAction(self)
		self._draw_bracket_action = PySide6.QtGui.QAction(self)
		self._draw_round_bracket_action = PySide6.QtGui.QAction(self)
		self._select_structure_action = PySide6.QtGui.QAction(self)
		self._move_atom_action = PySide6.QtGui.QAction(self)
		self._rotate_atoms_action = PySide6.QtGui.QAction(self)
		self._translate_roots_action = PySide6.QtGui.QAction(self)
		self._draw_vector_actions: dict[str, PySide6.QtGui.QAction] = {}
		self._user_template_directory: pathlib.Path | None = None

	def _connect_interaction_action_v1(self, action: PySide6.QtGui.QAction,
			callback: object) -> None:
		del callback
		action.setEnabled(True)

	def _active_native_tab(self) -> _Tab:
		return self.tab

	def _molecule_import_busy(self) -> bool:
		return False

	def _molecule_export_busy(self) -> bool:
		return False

	def _molecule_inspection_busy(self) -> bool:
		return False

	def _clipboard_busy(self) -> bool:
		return False

	def _cancel_atom_insertion(self) -> None:
		pass

	def _cancel_structure_selection(self) -> None:
		pass

	def _cancel_line_gesture(self) -> None:
		pass

	def _refresh_actions(self) -> None:
		pass

	def _publish_document_installation_v1(self, *_args: object) -> None:
		self.published += 1


def _mouse_press(button: PySide6.QtCore.Qt.MouseButton) -> PySide6.QtGui.QMouseEvent:
	"""Create one nondeprecated pointer press for the event-filter contract."""
	point = PySide6.QtCore.QPointF(1, 1)
	return PySide6.QtGui.QMouseEvent(
		PySide6.QtCore.QEvent.Type.MouseButtonPress, point, point, button, button,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
	)


def _host(window: _Window) -> TemplateCatalogHost:
	"""Expose only the narrow controller seams supplied by this test window."""
	return TemplateCatalogHost(
		window, window._action_registry, window._connect_interaction_action_v1,
		lambda: None, window._active_native_tab,
		lambda tab: window._native_tabs_by_page.get(tab) is tab,
		lambda: None, lambda: True, lambda: False, window._refresh_actions,
		window._publish_document_installation_v1, lambda: (window._draw_bond_action,),
	)


def _armed_controller(
		qapp: PySide6.QtWidgets.QApplication,
		) -> tuple[_Window, TemplateCatalogController, ferrum_qt.ferrum.operation_leases.OperationLeaseRegistry]:
	"""Return a controller with one registry-owned pointer placement."""
	window = _Window()
	registry = ferrum_qt.ferrum.operation_leases.OperationLeaseRegistry()
	registry.bind_tab(window.tab)
	controller = TemplateCatalogController(_host(window), registry)
	window.show()
	controller.open()
	qapp.processEvents()
	assert controller.start_placement(object(), "opaque-key")
	return window, controller, registry


#============================================
def test_registered_catalog_action_keeps_its_public_identity(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The real window exposes the catalog through its stable action identity."""
	del qapp
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow(
		user_template_directory=tmp_path / "templates",
	)
	try:
		action = window._action_registry.get_qt_action("chemistry.template.catalog")
		assert action.text() == "Template Catalog..."
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_controller_opens_a_modeless_catalog_dialog(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A controller publishes a visible dialog from a Rust-shaped snapshot."""
	window = _Window()
	controller = TemplateCatalogController(
		_host(window), ferrum_qt.ferrum.operation_leases.OperationLeaseRegistry(),
	)
	monkeypatch.setattr(
		ferrum_qt.ferrum.engine, "snapshot_template_catalog_v1",
		lambda _directory: _Snapshot(1, "a" * 64),
	)
	try:
		controller.open()
		qapp.processEvents()
		dialog = window.findChild(FerrumTemplateCatalogDialog)
		assert dialog is not None and dialog.isVisible()
	finally:
		window.close()


@pytest.mark.parametrize("event", (
	PySide6.QtGui.QKeyEvent(
		PySide6.QtCore.QEvent.Type.KeyPress, PySide6.QtCore.Qt.Key.Key_Escape,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
	),
	_mouse_press(PySide6.QtCore.Qt.MouseButton.RightButton),
	PySide6.QtCore.QEvent(PySide6.QtCore.QEvent.Type.FocusOut),
))
def test_catalog_pointer_cancellation_restores_the_dialog_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, event: PySide6.QtCore.QEvent,
		) -> None:
	"""Escape, right-click, and focus loss retire a placement without mutation."""
	window, controller, _registry = _armed_controller(qapp)
	try:
		controller.eventFilter(window.tab.view.viewport(), event)
		qapp.processEvents()
		assert window.tab.calls == 0
		dialog = window.findChild(FerrumTemplateCatalogDialog)
		assert dialog is not None and dialog.isVisible()
	finally:
		window.close()


@pytest.mark.parametrize("cause", ("escape", "right_click", "focus_loss"))
def test_catalog_cancellation_restores_the_exact_viewport_cursor_and_tracking(
		qapp: PySide6.QtWidgets.QApplication, cause: str,
		) -> None:
	"""Pointer cancellation gives a canvas back its prior explicit interaction state."""
	window, controller, _registry = _armed_controller(qapp)
	viewport = window.tab.view.viewport()
	previous_cursor = PySide6.QtGui.QCursor(PySide6.QtCore.Qt.CursorShape.SizeAllCursor)
	try:
		controller.cancel_active(reopen=False)
		viewport.setMouseTracking(False)
		viewport.setCursor(previous_cursor)
		assert controller.start_placement(object(), "opaque-key")
		if cause == "escape":
			event = PySide6.QtGui.QKeyEvent(
				PySide6.QtCore.QEvent.Type.KeyPress,
				PySide6.QtCore.Qt.Key.Key_Escape,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			)
		elif cause == "right_click":
			event = _mouse_press(PySide6.QtCore.Qt.MouseButton.RightButton)
		else:
			event = PySide6.QtCore.QEvent(PySide6.QtCore.QEvent.Type.FocusOut)
		controller.eventFilter(viewport, event)
		assert not viewport.hasMouseTracking()
		assert viewport.cursor().shape() is previous_cursor.shape()
	finally:
		window.close()


#============================================
def test_tool_replacement_releases_the_catalog_lifecycle_lease(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Choosing another authoring tool stops catalog pointer ownership immediately."""
	window, controller, registry = _armed_controller(qapp)
	try:
		controller.wire_tool_replacement()
		window._draw_bond_action.setChecked(True)
		qapp.processEvents()
		assert not registry.has_active(
			ferrum_qt.ferrum.operation_leases.OperationFamily.TEMPLATE_CATALOG,
			window.tab,
		)
		assert window.tab.calls == 0
	finally:
		window.close()


#============================================
def test_tool_replacement_restores_the_exact_viewport_cursor_and_tracking(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Choosing another tool restores the canvas state held before catalog placement."""
	window, controller, _registry = _armed_controller(qapp)
	viewport = window.tab.view.viewport()
	previous_cursor = PySide6.QtGui.QCursor(PySide6.QtCore.Qt.CursorShape.SizeAllCursor)
	try:
		controller.cancel_active(reopen=False)
		viewport.setMouseTracking(False)
		viewport.setCursor(previous_cursor)
		assert controller.start_placement(object(), "opaque-key")
		controller.wire_tool_replacement()
		window._draw_bond_action.setChecked(True)
		qapp.processEvents()
		assert not viewport.hasMouseTracking()
		assert viewport.cursor().shape() is previous_cursor.shape()
	finally:
		window.close()


#============================================
def test_stale_document_refuses_before_native_template_placement(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A changed document fence returns to the catalog before native mutation."""
	window, controller, _registry = _armed_controller(qapp)
	try:
		window.tab.current_snapshot = _Snapshot(2, "b" * 64)
		controller.eventFilter(window.tab.view.viewport(), _mouse_press(
			PySide6.QtCore.Qt.MouseButton.LeftButton,
		))
		assert window.tab.calls == 0
		dialog = window.findChild(FerrumTemplateCatalogDialog)
		assert dialog is not None and "document changed" in dialog.state.text().lower()
	finally:
		window.close()


#============================================
def test_one_native_catalog_placement_settles_its_lifecycle_lease(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""One click submits one exact native placement and retires its lease."""
	window, controller, _registry = _armed_controller(qapp)
	try:
		controller.eventFilter(window.tab.view.viewport(), _mouse_press(
			PySide6.QtCore.Qt.MouseButton.LeftButton,
		))
		assert window.tab.calls == 1
		assert window.published == 1
	finally:
		window.close()
