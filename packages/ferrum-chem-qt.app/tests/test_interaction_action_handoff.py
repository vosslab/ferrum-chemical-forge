"""Ordering tests for Ferrum canvas-action capture handoff."""

# PIP3 modules
import unittest.mock

import PySide6.QtGui
import PySide6.QtCore
import PySide6.QtWidgets
import PySide6.QtTest
import shiboken6

# local repo modules
import ferrum_qt.ferrum.interaction_action_handoff
import ferrum_qt.ferrum.line_tools
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

	def __init__(self, *, popup_watchdog_ms: int = 250) -> None:
		"""Build one visible canvas host."""
		super().__init__()
		self.tab = _Tab(self)
		self.setCentralWidget(self.tab.view)
		self.handoff_failures: list[str] = []
		self._interaction_action_handoff = (
			ferrum_qt.ferrum.interaction_action_handoff.
			FerrumInteractionActionHandoff(
				self, self.handoff_failures.append,
				popup_watchdog_ms=popup_watchdog_ms,
			)
		)

	def _active_native_tab(self) -> _Tab:
		"""Return the only live test tab."""
		return self.tab

	#============================================
	def _connect_interaction_action_v1(self, action: PySide6.QtGui.QAction,
			handler: object) -> None:
		"""Use the MainWindow pointer-ownership registration seam."""
		self._interaction_action_handoff.connect(action, handler)

	def _add_interaction_action_to_menu_v1(self, menu: PySide6.QtWidgets.QMenu,
			action: PySide6.QtGui.QAction) -> None:
		"""Use the MainWindow popup-lifecycle insertion seam."""
		self._interaction_action_handoff.add_registered_action_to_menu(menu, action)

	#============================================
	def _set_interaction_capture_canceller_v1(self, canceller: object | None) -> None:
		"""Use the MainWindow current-capture binding seam."""
		self._interaction_action_handoff.set_capture_canceller(canceller)


#============================================
class _LineToolActionWindow(
		ferrum_qt.ferrum.line_tools.FerrumNativeLineToolsMixin,
		PySide6.QtWidgets.QMainWindow,
		):
	"""Construct line-tool QActions without a native document session."""

	def _connect_interaction_action_v1(self, _action: PySide6.QtGui.QAction,
			handler: object) -> None:
		"""Retain only the construction boundary; handlers are not invoked here."""
		assert callable(handler)

	def _add_interaction_action_to_menu_v1(self, menu: PySide6.QtWidgets.QMenu,
			action: PySide6.QtGui.QAction) -> None:
		"""Expose each constructed action through the ordinary menu surface."""
		menu.addAction(action)


#============================================
def test_draw_bond_action_uses_qaction_compatible_accessibility_metadata(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Draw Bond construction needs no QWidget-only accessibility methods."""
	window = _LineToolActionWindow()
	menu = PySide6.QtWidgets.QMenu(window)
	window._build_line_tool_actions(menu)
	action = window._draw_bond_action
	assert not hasattr(action, "setAccessibleName")
	assert action.text() == "Draw Bond"
	assert action.toolTip() == (
		"Drag from an atom to another atom or empty space. Creates a normal single "
		"carbon bond. Escape cancels."
	)
	assert action.statusTip() == "Draw a normal single carbon bond. Escape cancels."
	assert action.whatsThis() == action.toolTip()
	assert menu.actions()[0] is action
	window.close()


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
def test_action_without_popup_dispatches_immediately(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""An ordinary action still invokes its handler in its triggered slot."""
	window = _Window()
	events: list[str] = []
	action = PySide6.QtGui.QAction("Immediate incoming tool", window)
	window._connect_interaction_action_v1(action, lambda: events.append("handler"))

	action.trigger()

	assert events == ["handler"]
	window.close()


#============================================
def test_popup_row_defers_the_handoff_until_the_popup_is_gone(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A real transient menu cannot let its later teardown cancel a new tool."""
	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Popup incoming tool", window)
	menu.addAction(action)
	popup_states: list[object] = []
	window._connect_interaction_action_v1(
		action, lambda: popup_states.append(PySide6.QtWidgets.QApplication.activePopupWidget()),
	)
	window.show()
	qapp.processEvents()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()

	PySide6.QtTest.QTest.mouseClick(
		menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, menu.actionGeometry(action).center(),
	)
	qapp.processEvents()

	assert popup_states == [None]
	menu.deleteLater()
	window.close()


#============================================
def test_registered_menu_latch_exists_before_popup_and_rearms_on_reuse(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The owned insertion seam needs no application Show-event observation."""
	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Registered popup tool", window)
	events: list[object] = []
	window._connect_interaction_action_v1(
		action, lambda: events.append(PySide6.QtWidgets.QApplication.activePopupWidget()),
	)
	window._add_interaction_action_to_menu_v1(menu, action)
	latch = window._interaction_action_handoff._popup_latch_for(menu)
	assert menu in window._interaction_action_handoff._registered_action_menus[action]
	assert not latch.terminal_seen
	qapp.removeEventFilter(window._interaction_action_handoff)
	window.show()
	qapp.processEvents()
	for _index in range(2):
		menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
		qapp.processEvents()
		assert not latch.terminal_seen
		PySide6.QtTest.QTest.mouseClick(
			menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, menu.actionGeometry(action).center(),
		)
		qapp.processEvents()
	assert events == [None, None]
	assert window.handoff_failures == []
	menu.deleteLater()
	qapp.processEvents()
	window.close()


#============================================
def test_reverse_menu_insertion_registers_latch_during_connect(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Existing QAction menu ownership is adopted before the first popup show."""
	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Reverse registered popup tool", window)
	menu.addAction(action)
	events: list[str] = []
	window._connect_interaction_action_v1(action, lambda: events.append("handler"))
	assert menu in window._interaction_action_handoff._registered_action_menus[action]
	assert menu in window._interaction_action_handoff._popup_latches
	window.show()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	PySide6.QtTest.QTest.mouseClick(
		menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, menu.actionGeometry(action).center(),
	)
	qapp.processEvents()
	assert events == ["handler"]
	menu.deleteLater()
	window.close()


#============================================
def test_latched_menu_terminal_before_action_dispatch_runs_once(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A menu terminal recorded before QAction dispatch still permits one handoff."""
	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Late terminal tool", window)
	menu.addAction(action)
	events: list[str] = []
	window._connect_interaction_action_v1(action, lambda: events.append("handler"))
	window.show()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	menu.aboutToHide.emit()
	action.trigger()
	menu.hide()
	qapp.processEvents()

	assert events == ["handler"]
	assert window.handoff_failures == []
	window.close()


#============================================
def test_reused_menu_requires_its_second_terminal_before_dispatch(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A reopened menu starts a new terminal cycle before its action can run."""
	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Reused popup tool", window)
	menu.addAction(action)
	events: list[str] = []
	window._connect_interaction_action_v1(action, lambda: events.append("handler"))
	window.show()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	latch = window._interaction_action_handoff._popup_latch_for(menu)
	menu.hide()
	qapp.processEvents()
	assert latch.terminal_seen

	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	assert not window._interaction_action_handoff.eventFilter(
		menu, PySide6.QtCore.QEvent(PySide6.QtCore.QEvent.Type.Show),
	)
	assert not latch.terminal_seen
	action.trigger()
	qapp.processEvents()
	assert events == []
	menu.hide()
	qapp.processEvents()

	assert events == ["handler"]
	menu.deleteLater()
	window.close()


#============================================
def test_destroyed_popups_release_latches_after_each_lifecycle(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Destroyed transient popups leave no retained latch children or registry entries."""
	window = _Window()
	handoff = window._interaction_action_handoff
	baseline_children = len([
		child for child in handoff.children()
		if isinstance(child, ferrum_qt.ferrum.interaction_action_handoff._PopupTerminalLatch)
	])
	window.show()
	qapp.processEvents()
	unrelated = PySide6.QtWidgets.QWidget(window)
	unrelated.show()
	qapp.processEvents()
	assert len(handoff._popup_latches) == 0
	assert len([
		child for child in handoff.children()
		if isinstance(child, ferrum_qt.ferrum.interaction_action_handoff._PopupTerminalLatch)
	]) == baseline_children
	unrelated.hide()

	for _index in range(3):
		menu = PySide6.QtWidgets.QMenu(window)
		assert not handoff.eventFilter(
			menu, PySide6.QtCore.QEvent(PySide6.QtCore.QEvent.Type.Show),
		)
		menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
		qapp.processEvents()
		assert len(handoff._popup_latches) == 1
		shiboken6.delete(menu)
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.DeferredDelete,
		)
		qapp.processEvents()
		assert len(handoff._popup_latches) == 0
		assert len([
			child for child in handoff.children()
			if isinstance(child, ferrum_qt.ferrum.interaction_action_handoff._PopupTerminalLatch)
		]) == baseline_children

	window.close()


#============================================
def test_replacement_popup_defers_dispatch_until_its_own_teardown(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A nested replacement menu cannot let an incoming tool start early."""
	window = _Window()
	first = PySide6.QtWidgets.QMenu(window)
	second = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Replacement popup tool", window)
	first.addAction(action)
	states: list[object] = []
	window._connect_interaction_action_v1(
		action, lambda: states.append(PySide6.QtWidgets.QApplication.activePopupWidget()),
	)
	window.show()
	first.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	action.trigger()
	continuation = window._interaction_action_handoff._continuations[action]
	with unittest.mock.patch.object(
			PySide6.QtWidgets.QApplication, "activePopupWidget", return_value=second,
		):
		continuation._settle_popup_terminal()
	assert states == []
	with unittest.mock.patch.object(
			PySide6.QtWidgets.QApplication, "activePopupWidget", return_value=None,
		):
		continuation._settle_popup_terminal()
	assert states == [None]
	first.hide()
	window.close()


#============================================
def test_destroyed_popup_cancels_the_deferred_action(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Popup destruction is teardown, never implicit permission to run a tool."""
	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Destroyed popup tool", window)
	menu.addAction(action)
	events: list[str] = []
	window._connect_interaction_action_v1(action, lambda: events.append("handler"))
	window.show()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	action.trigger()
	shiboken6.delete(menu)
	qapp.processEvents()
	assert events == []
	assert window.handoff_failures == []
	window.close()


#============================================
def test_terminal_popup_destruction_keeps_the_queued_action_authorized(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A terminal popup may be deleted before its authorized queued handoff runs."""
	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Terminal destroyed popup tool", window)
	menu.addAction(action)
	events: list[str] = []
	window._connect_interaction_action_v1(action, lambda: events.append("handler"))
	window.show()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	action.trigger()
	continuation = window._interaction_action_handoff._continuations[action]
	continuation._on_popup_terminal()
	shiboken6.delete(menu)
	qapp.processEvents()
	assert events == ["handler"]
	assert window.handoff_failures == []
	window.close()


#============================================
def test_destroyed_action_and_window_cancel_deferred_actions(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Neither action nor window teardown can dispatch a retained handler."""
	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Destroyed action tool", window)
	menu.addAction(action)
	events: list[str] = []
	window._connect_interaction_action_v1(action, lambda: events.append("action"))
	window.show()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	action.trigger()
	shiboken6.delete(action)
	menu.hide()
	qapp.processEvents()
	assert events == []
	assert window.handoff_failures == []

	window = _Window()
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Destroyed window tool", window)
	menu.addAction(action)
	window._connect_interaction_action_v1(action, lambda: events.append("window"))
	window.show()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	action.trigger()
	shiboken6.delete(window)
	qapp.processEvents()
	assert events == []


#============================================
def test_popup_watchdog_fails_closed_once(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A persistent popup cannot retain a checked action or invoke its handler."""
	window = _Window(popup_watchdog_ms=1)
	menu = PySide6.QtWidgets.QMenu(window)
	action = PySide6.QtGui.QAction("Watchdog tool", window)
	action.setCheckable(True)
	menu.addAction(action)
	events: list[str] = []
	window._connect_interaction_action_v1(action, lambda: events.append("handler"))
	window.show()
	menu.popup(window.mapToGlobal(PySide6.QtCore.QPoint(20, 20)))
	qapp.processEvents()
	action.trigger()
	PySide6.QtTest.QTest.qWait(20)
	qapp.processEvents()
	assert events == []
	assert len(window.handoff_failures) == 1
	assert not action.isChecked()
	qapp.processEvents()
	assert len(window.handoff_failures) == 1
	menu.hide()
	window.close()


#============================================
def test_guard_and_handler_failures_are_reported_once_and_fail_closed(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The common invocation boundary keeps exceptions out of Qt callbacks."""
	window = _Window()
	guard_action = PySide6.QtGui.QAction("Broken guard", window)
	handler_action = PySide6.QtGui.QAction("Broken handler", window)
	events: list[str] = []

	def broken_guard() -> None:
		raise RuntimeError("guard")

	def broken_handler() -> None:
		events.append("handler")
		raise RuntimeError("handler")

	window._set_interaction_capture_canceller_v1(broken_guard)
	window._connect_interaction_action_v1(guard_action, lambda: events.append("guard handler"))
	guard_action.trigger()
	window._set_interaction_capture_canceller_v1(None)
	window._connect_interaction_action_v1(handler_action, broken_handler)
	handler_action.trigger()
	qapp.processEvents()
	assert events == ["handler"]
	assert len(window.handoff_failures) == 2
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
