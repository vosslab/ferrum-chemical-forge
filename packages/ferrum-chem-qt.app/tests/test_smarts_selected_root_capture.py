"""Event-level tests for one-shot selected-molecule SMARTS capture."""

# Standard Library
import types

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum
import ferrum_qt.ferrum.smarts_selected_root_capture


#============================================
class _Dock:
	"""Record copied capture outcomes without accepting a generic selection."""

	def __init__(self) -> None:
		self.ready: object | None = None
		self.refusals: list[str] = []
		self.started = 0

	def _selected_capture_started_v1(self) -> None:
		self.started += 1

	def _selected_capture_ready_v1(self, tab: object) -> None:
		self.ready = tab

	def _selected_capture_refused_v1(self, message: str) -> None:
		self.refusals.append(message)


#============================================
class _Tab:
	"""Minimal renderer boundary that records selection consumption only."""

	_disposed = False
	requires_refresh = False

	def __init__(self) -> None:
		self.view = PySide6.QtWidgets.QGraphicsView()
		self.view.setScene(PySide6.QtWidgets.QGraphicsScene(self.view))
		self.selection = object()
		self.token = object()
		self.captured: object | None = None
		self.direct_observation = object()
		self.direct_selections: list[tuple[object, object, object]] = []

	def observe_direct_root_interaction(self) -> object:
		return self.direct_observation

	def select_direct_roots(self, observation: object, prior: object,
			query: object) -> object:
		self.direct_selections.append((observation, prior, query))
		assert observation is self.direct_observation
		assert prior is None
		return self.selection

	def observe_structure_interaction(self) -> object:
		raise AssertionError("selected-root capture must not observe structural children")

	def select_structure_interaction(self, *_args: object) -> object:
		raise AssertionError("selected-root capture must not select structural children")

	def _capture_live_smarts_selected_query_v1(self, selection: object) -> object:
		assert selection is self.selection
		self.captured = selection
		return self.token


#============================================
class _Window(PySide6.QtWidgets.QMainWindow):
	"""Host the temporary viewport controller without persistent selection state."""

	def __init__(self, tab: _Tab) -> None:
		super().__init__()
		self._tab = tab
		self.setCentralWidget(tab.view)

	def _active_native_tab(self) -> _Tab:
		return self._tab

	def _cancel_structure_selection(self) -> None:
		return None

	def _cancel_catalog_placement(self) -> None:
		return None

	def _cancel_atom_insertion(self) -> None:
		return None

	def _cancel_line_gesture(self, *, clear_status: bool = False) -> None:
		return None


#============================================
def _install_engine_stub(monkeypatch: object) -> None:
	"""Provide only the renderer query factory required by the event adapter."""
	def point(x: float, y: float, modifier: object) -> object:
		return types.SimpleNamespace(x=x, y=y, modifier=modifier)

	engine = types.SimpleNamespace(
		RenderInteractionModifierV1=types.SimpleNamespace(replace=object()),
		RenderInteractionQueryV1=types.SimpleNamespace(point=point),
	)
	monkeypatch.setattr(ferrum_qt.ferrum, "engine", engine)
	monkeypatch.setitem(__import__("sys").modules, "ferrum_qt.ferrum.engine", engine)


#============================================
def test_canvas_click_consumes_generic_selection_into_opaque_token(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""A point event leaves the dock only with the tab-private opaque token."""
	_install_engine_stub(monkeypatch)
	tab = _Tab()
	window = _Window(tab)
	dock = _Dock()
	controller = (
		ferrum_qt.ferrum.smarts_selected_root_capture.
		FerrumSmartsSelectedRootCaptureController(window, dock)
	)
	window.show()
	controller.begin()
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		PySide6.QtCore.QPoint(4, 4),
	)
	assert dock.ready is tab
	assert tab.captured is tab.selection
	assert len(tab.direct_selections) == 1
	observation, previous, query = tab.direct_selections[0]
	assert observation is tab.direct_observation
	assert previous is None
	expected_scene = tab.view.mapToScene(PySide6.QtCore.QPoint(4, 4))
	assert query.x == float(expected_scene.x())
	assert query.y == float(expected_scene.y())
	assert controller._viewport is None
	assert controller._selected_query_token is tab.token
	assert not hasattr(dock, "selection")
	window.close()


#============================================
def test_escape_retires_viewport_capture_without_token(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Escape cancels the event capture before a generic selection can be retained."""
	tab = _Tab()
	window = _Window(tab)
	dock = _Dock()
	controller = (
		ferrum_qt.ferrum.smarts_selected_root_capture.
		FerrumSmartsSelectedRootCaptureController(window, dock)
	)
	window.show()
	controller.begin()
	PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
	assert controller._viewport is None
	assert dock.ready is None
	assert dock.refusals and "cancelled" in dock.refusals[-1]
	window.close()
