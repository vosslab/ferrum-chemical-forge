"""Ordering tests for Ferrum canvas-action capture handoff."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtCore
import PySide6.QtWidgets
import PySide6.QtTest

# local repo modules
import ferrum_qt.ferrum.interaction_action_handoff
import ferrum_qt.ferrum.smarts_selected_root_capture
import ferrum_qt.ferrum.smarts_query_dock


#============================================
class _Dock:
	"""Record only opaque-capture presentation outcomes."""

	def __init__(self) -> None:
		"""Start with no issued token."""
		self.ready: list[object] = []

	def _selected_capture_started_v1(self) -> None:
		"""Accept the bounded capture-state notification."""

	def _selected_capture_ready_v1(self, tab: object) -> None:
		"""Record copied readiness only if the viewport filter mints a token."""
		self.ready.append(tab)

	def _selected_capture_refused_v1(self, _message: str) -> None:
		"""Accept the closed recovery notification."""


#============================================
class _Tab:
	"""Supply the minimal ready tab shape needed to arm capture."""

	def __init__(self, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Create an ordinary viewport with no chemistry handler available."""
		self._disposed = False
		self.requires_refresh = False
		self.view = PySide6.QtWidgets.QGraphicsView(parent)


#============================================
class _Window(PySide6.QtWidgets.QMainWindow):
	"""Expose the selected-root controller's authoritative active-tab seam."""

	def __init__(self) -> None:
		"""Build one visible canvas host."""
		super().__init__()
		self.tab = _Tab(self)
		self.setCentralWidget(self.tab.view)
		self._interaction_action_handoff = (
			ferrum_qt.ferrum.interaction_action_handoff.
			FerrumInteractionActionHandoff()
		)

	def _active_native_tab(self) -> _Tab:
		"""Return the only live test tab."""
		return self.tab

	#============================================
	def _connect_interaction_action_v1(self, action: PySide6.QtGui.QAction,
			handler: object) -> None:
		"""Use the MainWindow pointer-ownership registration seam."""
		self._interaction_action_handoff.connect(action, handler)

	#============================================
	def _set_interaction_capture_canceller_v1(self, canceller: object | None) -> None:
		"""Use the MainWindow current-capture binding seam."""
		self._interaction_action_handoff.set_capture_canceller(canceller)


#============================================
def _armed_capture(
		qapp: PySide6.QtWidgets.QApplication,
		) -> tuple[_Window, _Dock, ferrum_qt.ferrum.smarts_selected_root_capture.FerrumSmartsSelectedRootCaptureController]:
	"""Create a real armed capture whose viewport filter can be observed."""
	window = _Window()
	window.show()
	qapp.processEvents()
	dock = _Dock()
	capture = (
		ferrum_qt.ferrum.smarts_selected_root_capture.
		FerrumSmartsSelectedRootCaptureController(window, dock)
	)
	window._set_interaction_capture_canceller_v1(
		capture._cancel_for_interaction_action_handoff_v1,
	)
	capture.begin()
	assert capture._viewport is window.tab.view.viewport()
	return window, dock, capture


#============================================
def _assert_capture_cannot_issue_token(
		window: _Window,
		dock: _Dock,
		capture: ferrum_qt.ferrum.smarts_selected_root_capture.FerrumSmartsSelectedRootCaptureController,
		) -> None:
	"""Prove the retired pointer mode cannot leave a selected-query capability."""
	assert capture._viewport is None
	assert not capture.is_ready_for(window.tab)
	try:
		capture.consume_selected_query_v1(window.tab, 8, 64)
	except RuntimeError as error:
		assert str(error) == "Ferrum selected molecule query is not ready"
	else:
		raise AssertionError("retired SMARTS capture issued a selected-query token")
	assert dock.ready == []


#============================================
def test_pre_registered_action_cancels_later_capture_before_its_handler(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A tool registered before capture construction receives its later guard."""
	window = _Window()
	events: list[str] = []
	action = PySide6.QtGui.QAction("Earlier incoming tool", window)

	def install_tool(_checked: bool = False) -> None:
		assert capture._viewport is None
		events.append("tool-installed")

	window._connect_interaction_action_v1(action, install_tool)
	window.show()
	qapp.processEvents()
	dock = _Dock()
	capture = (
		ferrum_qt.ferrum.smarts_selected_root_capture.
		FerrumSmartsSelectedRootCaptureController(window, dock)
	)
	window._set_interaction_capture_canceller_v1(
		capture._cancel_for_interaction_action_handoff_v1,
	)
	capture.begin()
	assert capture._viewport is window.tab.view.viewport()

	action.trigger()
	PySide6.QtTest.QTest.mouseClick(
		window.tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
	)

	assert events == ["tool-installed"]
	_assert_capture_cannot_issue_token(window, dock, capture)
	window.close()


#============================================
def test_late_created_action_uses_same_handoff_before_its_handler(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A dynamic future tool cannot bypass capture retirement by being added late."""
	window, dock, capture = _armed_capture(qapp)
	events: list[str] = []
	late_action = PySide6.QtGui.QAction("Late incoming tool")

	def install_late_tool(_checked: bool = False) -> None:
		assert capture._viewport is None
		events.append("late-tool-installed")

	window._connect_interaction_action_v1(late_action, install_late_tool)

	late_action.trigger()
	PySide6.QtTest.QTest.mouseClick(
		window.tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
	)

	assert events == ["late-tool-installed"]
	_assert_capture_cannot_issue_token(window, dock, capture)
	window.close()


#============================================
def test_installed_smarts_action_does_not_cancel_armed_capture(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The canonical SMARTS action preserves capture, while a tool handoff retires it."""
	window = _Window()
	window.show()
	qapp.processEvents()
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)
	menu = PySide6.QtWidgets.QMenu(window)
	action = controller.install_action(menu)
	capture = controller._selected_capture
	capture.begin()
	assert capture._viewport is window.tab.view.viewport()
	assert window.tab.view.viewport().hasFocus()

	action.trigger()
	qapp.processEvents()

	assert controller.dock.isVisible()
	assert capture._viewport is window.tab.view.viewport()
	assert window.tab.view.viewport().hasFocus()
	assert not capture.is_ready_for(window.tab)
	incoming_tool = PySide6.QtGui.QAction("Incoming canvas tool", window)
	events: list[str] = []

	def take_canvas_ownership(_checked: bool = False) -> None:
		assert capture._viewport is None
		events.append("tool")

	window._connect_interaction_action_v1(incoming_tool, take_canvas_ownership)
	incoming_tool.trigger()
	assert events == ["tool"]
	_assert_capture_cannot_issue_token(window, _Dock(), capture)
	controller.close()
	window.close()
