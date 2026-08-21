"""Lifetime-safe handoff for Qt actions that take canvas interaction ownership."""

# Standard Library
import inspect
import weakref

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import shiboken6


#============================================
class _PopupTerminalLatch(PySide6.QtCore.QObject):
	"""Remember one popup's terminal lifecycle event across action dispatch."""

	terminal = PySide6.QtCore.Signal()
	destroyed_without_terminal = PySide6.QtCore.Signal()

	def __init__(self, handoff: "FerrumInteractionActionHandoff",
			popup: PySide6.QtWidgets.QWidget) -> None:
		"""Observe a popup from its Show event until its destruction."""
		super().__init__(handoff)
		self._handoff = handoff
		self._popup_ref = weakref.ref(popup)
		self.terminal_seen = False
		popup.destroyed.connect(self._on_popup_destroyed)
		if isinstance(popup, PySide6.QtWidgets.QMenu):
			popup.aboutToHide.connect(self.mark_terminal)

	def mark_terminal(self) -> None:
		"""Latch the first genuine terminal signal and notify current listeners."""
		if self.terminal_seen:
			return
		self.terminal_seen = True
		self.terminal.emit()

	def rearm_for_show(self) -> None:
		"""Start a fresh terminal cycle when the same popup is shown again."""
		self.terminal_seen = False

	@PySide6.QtCore.Slot()
	def _on_popup_destroyed(self) -> None:
		"""Treat destruction without a terminal event as cancellation, not success."""
		self._handoff._release_popup_latch(self)
		if not self.terminal_seen:
			self.destroyed_without_terminal.emit()
		self.deleteLater()


#============================================
class _PopupActionContinuation(PySide6.QtCore.QObject):
	"""Own one action invocation until its transient popup has retired."""

	MAX_POPUP_TRANSITIONS = 3

	def __init__(self, handoff: "FerrumInteractionActionHandoff", action: PySide6.QtGui.QAction,
			handler: object, accepts_checked: bool, checked: bool, watchdog_ms: int) -> None:
		"""Retain exactly one guarded action invocation."""
		super().__init__(handoff)
		self._handoff = handoff
		self._action = action
		self._handler = handler
		self._accepts_checked = accepts_checked
		self._checked = checked
		self._state = "waiting"
		self._popup: PySide6.QtWidgets.QWidget | None = None
		self._popup_latch: _PopupTerminalLatch | None = None
		self._popup_transitions = 0
		self._settle_queued = False
		self._terminal_dispatch_authorized = False
		self._watchdog = PySide6.QtCore.QTimer(self)
		self._watchdog.setSingleShot(True)
		self._watchdog.setInterval(watchdog_ms)
		self._watchdog.timeout.connect(self._on_watchdog_timeout)

	def arm_popup(self, popup: PySide6.QtWidgets.QWidget,
			latch: _PopupTerminalLatch) -> None:
		"""Observe one live popup lifecycle without polling Qt state."""
		if self._state != "waiting":
			return
		self._disconnect_popup()
		self._popup_transitions += 1
		if self._popup_transitions > self.MAX_POPUP_TRANSITIONS:
			self._fail("popup replacement limit reached")
			return
		self._popup = popup
		self._popup_latch = latch
		# A replacement popup needs its own terminal authorization.
		self._terminal_dispatch_authorized = False
		latch.terminal.connect(self._on_popup_terminal)
		latch.destroyed_without_terminal.connect(self._on_popup_destroyed)
		self._watchdog.start()
		if latch.terminal_seen:
			self._on_popup_terminal()

	@PySide6.QtCore.Slot()
	def _on_popup_terminal(self) -> None:
		"""Queue one post-teardown turn after a real popup terminal event."""
		if self._state != "waiting" or self._settle_queued:
			return
		# Destruction after this terminal signal is ordinary popup teardown, not cancellation.
		self._terminal_dispatch_authorized = True
		self._settle_queued = True
		PySide6.QtCore.QMetaObject.invokeMethod(
			self, "_settle_popup_terminal", PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)

	@PySide6.QtCore.Slot()
	def _on_popup_destroyed(self) -> None:
		"""Cancel rather than dispatch when the observed popup is destroyed."""
		if not self._terminal_dispatch_authorized:
			self.cancel()

	@PySide6.QtCore.Slot()
	def _settle_popup_terminal(self) -> None:
		"""Run once after Qt has settled popup ownership."""
		self._settle_queued = False
		if self._state != "waiting":
			return
		if not self._handoff._owner_is_live() or not self._action_is_live():
			self.cancel()
			return
		popup = PySide6.QtWidgets.QApplication.activePopupWidget()
		if popup is None:
			self._disconnect_popup()
			self.run_once()
			return
		if popup is self._popup:
			self._fail("popup did not retire after its terminal event")
			return
		self.arm_popup(popup, self._handoff._popup_latch_for(popup))

	@PySide6.QtCore.Slot()
	def _on_watchdog_timeout(self) -> None:
		"""Fail closed when a popup never provides a terminal lifecycle event."""
		if self._state == "waiting":
			self._fail("popup teardown timed out")

	def run_once(self) -> None:
		"""Run the atomic guard then handler exactly once while both owners live."""
		if self._state != "waiting":
			return
		if not self._handoff._owner_is_live() or not self._action_is_live():
			self.cancel()
			return
		self._state = "running"
		try:
			self._handoff._before_incoming_action(self._checked)
		except Exception as error:
			self._fail("capture guard failed: " + str(error))
			return
		try:
			if self._accepts_checked:
				self._handler(self._checked)
			else:
				self._handler()
		except Exception as error:
			self._fail("action handler failed: " + str(error))
			return
		self._finish()

	def cancel(self) -> None:
		"""Discard a deferred invocation during normal Qt-object teardown."""
		if self._state in ("cancelled", "finished"):
			return
		self._state = "cancelled"
		self._cleanup()

	def _fail(self, detail: str) -> None:
		"""Finish one abnormal invocation and present exactly one typed refusal."""
		if self._state in ("cancelled", "finished"):
			return
		self._state = "finished"
		if self._action_is_live() and self._action.isCheckable():
			self._action.setChecked(False)
		self._cleanup()
		try:
			self._handoff._report_failure(detail)
		except Exception:
			return

	def _finish(self) -> None:
		"""Release all retained Qt and Python state after a successful invocation."""
		self._state = "finished"
		self._cleanup()

	def _cleanup(self) -> None:
		"""Stop the one watchdog and release this continuation from its handoff."""
		self._watchdog.stop()
		self._disconnect_popup()
		self._handoff._release_continuation(self._action, self)
		self._handler = None

	def _disconnect_popup(self) -> None:
		"""Remove only observers installed by this continuation."""
		latch = self._popup_latch
		self._popup = None
		self._popup_latch = None
		if latch is None:
			return
		try:
			latch.terminal.disconnect(self._on_popup_terminal)
			latch.destroyed_without_terminal.disconnect(self._on_popup_destroyed)
		except RuntimeError:
			return

	def _action_is_live(self) -> bool:
		"""Check the C++ QAction lifetime immediately before touching it."""
		return shiboken6.isValid(self._action)


#============================================
class FerrumInteractionActionHandoff(PySide6.QtCore.QObject):
	"""Cancel one temporary canvas capture before an incoming tool activates."""

	def __init__(self, owner: PySide6.QtWidgets.QWidget, failure_reporter: object,
			*, popup_watchdog_ms: int = 250) -> None:
		"""Create the window-owned shared action handoff."""
		super().__init__(owner)
		if not callable(failure_reporter):
			raise TypeError("Ferrum interaction failure reporter must be callable")
		self._owner = owner
		self._failure_reporter = failure_reporter
		self._popup_watchdog_ms = popup_watchdog_ms
		self._owner_destroyed = False
		self._cancel_capture: object | None = None
		self._actions: dict[PySide6.QtGui.QAction, object] = {}
		self._registered_action_menus: weakref.WeakKeyDictionary[
			PySide6.QtGui.QAction, weakref.WeakSet[PySide6.QtWidgets.QMenu],
		] = weakref.WeakKeyDictionary()
		self._continuations: dict[PySide6.QtGui.QAction, _PopupActionContinuation] = {}
		self._popup_latches: weakref.WeakKeyDictionary[
			PySide6.QtWidgets.QWidget, _PopupTerminalLatch] = weakref.WeakKeyDictionary()
		self._application = PySide6.QtWidgets.QApplication.instance()
		if self._application is None:
			raise RuntimeError("Ferrum interaction handoff requires QApplication")
		self._application.installEventFilter(self)
		owner.destroyed.connect(self._on_owner_destroyed)

	def set_capture_canceller(self, canceller: object | None) -> None:
		"""Install the one current temporary-capture cancellation client."""
		if canceller is not None and not callable(canceller):
			raise TypeError("Ferrum interaction cancellation client must be callable")
		self._cancel_capture = canceller

	def connect(self, action: PySide6.QtGui.QAction, handler: object) -> None:
		"""Connect one pointer-owning action through its cancellation guard."""
		if not callable(handler):
			raise TypeError("Ferrum interaction action handler must be callable")
		if action in self._actions:
			raise ValueError("Ferrum interaction action registered twice")
		signature = inspect.signature(handler)
		accepts_checked = any(
			parameter.kind in (
				inspect.Parameter.POSITIONAL_ONLY,
				inspect.Parameter.POSITIONAL_OR_KEYWORD,
				inspect.Parameter.VAR_POSITIONAL,
			)
			for parameter in signature.parameters.values()
		)

		def dispatch(checked: bool = False) -> None:
			"""Associate this one trigger with the current popup lifecycle."""
			self._dispatch(action, handler, accepts_checked, checked)

		self._actions[action] = dispatch
		action.triggered.connect(dispatch)
		action.destroyed.connect(self._on_action_destroyed)
		for associated_object in action.associatedObjects():
			if isinstance(associated_object, PySide6.QtWidgets.QMenu):
				self._register_action_menu(action, associated_object)

	def add_registered_action_to_menu(self, menu: PySide6.QtWidgets.QMenu,
			action: PySide6.QtGui.QAction) -> None:
		"""Register popup lifecycle before inserting one pointer-owning action."""
		self._register_action_menu(action, menu)
		menu.addAction(action)

	def _register_action_menu(self, action: PySide6.QtGui.QAction,
			menu: PySide6.QtWidgets.QMenu) -> None:
		"""Make one registered action/menu pair lifecycle-ready exactly once."""
		if action not in self._actions:
			raise ValueError("Ferrum interaction action must be connected before menu insertion")
		menus = self._registered_action_menus.get(action)
		if menus is None:
			menus = weakref.WeakSet()
			self._registered_action_menus[action] = menus
		if menu in menus:
			return
		self._popup_latch_for(menu)
		menu.aboutToShow.connect(self._popup_latches[menu].rearm_for_show)
		menus.add(menu)

	def _dispatch(self, action: PySide6.QtGui.QAction, handler: object,
			accepts_checked: bool, checked: bool) -> None:
		"""Create the one continuation for this exact Qt action trigger."""
		if not self._owner_is_live() or not shiboken6.isValid(action):
			return
		previous = self._continuations.get(action)
		if previous is not None:
			previous.cancel()
		continuation = _PopupActionContinuation(
			self, action, handler, accepts_checked, checked, self._popup_watchdog_ms,
		)
		self._continuations[action] = continuation
		popup = PySide6.QtWidgets.QApplication.activePopupWidget()
		if popup is None:
			continuation.run_once()
		else:
			continuation.arm_popup(popup, self._popup_latch_for(popup))

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Record popup terminals from Show through Hide before QAction dispatch."""
		if not isinstance(watched, PySide6.QtWidgets.QWidget):
			return False
		if not self._is_popup_widget(watched):
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.Show:
			self._popup_latch_for(watched).rearm_for_show()
		elif event.type() == PySide6.QtCore.QEvent.Type.Hide:
			latch = self._popup_latches.get(watched)
			if latch is not None:
				latch.mark_terminal()
		return False

	def _popup_latch_for(self, popup: PySide6.QtWidgets.QWidget) -> _PopupTerminalLatch:
		"""Return the popup-owned latch whose terminal state resets on each Show."""
		latch = self._popup_latches.get(popup)
		if latch is None:
			latch = _PopupTerminalLatch(self, popup)
			self._popup_latches[popup] = latch
		return latch

	def _release_popup_latch(self, latch: _PopupTerminalLatch) -> None:
		"""Forget a destroyed popup's latch before its queued QObject deletion."""
		for popup, registered_latch in tuple(self._popup_latches.items()):
			if registered_latch is latch:
				del self._popup_latches[popup]
				return

	@staticmethod
	def _is_popup_widget(widget: PySide6.QtWidgets.QWidget) -> bool:
		"""Limit application-wide lifecycle bookkeeping to transient popup widgets."""
		return isinstance(widget, PySide6.QtWidgets.QMenu) or (
			widget.windowType() == PySide6.QtCore.Qt.WindowType.Popup
		)

	def _before_incoming_action(self, _checked: bool = False) -> None:
		"""Retire selected-root capture synchronously before the tool handler."""
		canceller = self._cancel_capture
		if callable(canceller):
			canceller()

	def _release_continuation(self, action: PySide6.QtGui.QAction,
			continuation: _PopupActionContinuation) -> None:
		"""Forget this exact terminal continuation without affecting a replacement."""
		if self._continuations.get(action) is continuation:
			del self._continuations[action]

	@PySide6.QtCore.Slot(PySide6.QtCore.QObject)
	def _on_owner_destroyed(self, _object: PySide6.QtCore.QObject) -> None:
		"""Cancel every invocation before the window's Qt ownership disappears."""
		continuations = getattr(self, "_continuations", None)
		if continuations is None:
			return
		self._owner_destroyed = True
		application = self._application
		if application is not None and shiboken6.isValid(application):
			application.removeEventFilter(self)
		self._application = None
		for continuation in tuple(continuations.values()):
			continuation.cancel()
		self._actions.clear()
		self._registered_action_menus.clear()
		self._popup_latches.clear()
		self._cancel_capture = None

	@PySide6.QtCore.Slot(PySide6.QtCore.QObject)
	def _on_action_destroyed(self, destroyed_object: PySide6.QtCore.QObject) -> None:
		"""Cancel only the destroyed action's deferred invocation."""
		for action in tuple(self._actions):
			if action is destroyed_object:
				continuation = self._continuations.get(action)
				if continuation is not None:
					continuation.cancel()
				self._actions.pop(action, None)
				self._registered_action_menus.pop(action, None)
				return

	def _owner_is_live(self) -> bool:
		"""Check both logical and C++ owner liveness before presentation or dispatch."""
		return not self._owner_destroyed and shiboken6.isValid(self._owner)

	def _report_failure(self, detail: str) -> None:
		"""Present one shared typed refusal only while the owning window remains live."""
		if self._owner_is_live():
			self._failure_reporter(detail)
