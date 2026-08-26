"""Lifetime-safe handoff for Qt actions that take canvas interaction ownership."""

# Standard Library
import collections.abc
import inspect
import weakref

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import shiboken6


#============================================
class FerrumInteractionActionHandoffRefusal(Exception):
	"""An intentional user-visible refusal from one pointer-owning action."""

	def __init__(self, detail: str) -> None:
		"""Create a refusal with the precise explanation shown by the window."""
		if not isinstance(detail, str) or not detail:
			raise ValueError("Ferrum interaction refusal detail must be nonempty text")
		super().__init__(detail)


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
		self.terminal_seen = False
		popup.destroyed.connect(self._on_popup_destroyed)

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
	"""Own one action invocation until its transient popup has closed."""

	def __init__(self, handoff: "FerrumInteractionActionHandoff", action: PySide6.QtGui.QAction,
			handler: object, accepts_checked: bool, checked: bool) -> None:
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
		self._settle_queued = False
		self._terminal_dispatch_authorized = False

	def arm_popup(self, popup: PySide6.QtWidgets.QWidget,
			latch: _PopupTerminalLatch) -> None:
		"""Observe one live popup lifecycle without polling Qt state."""
		if self._state != "waiting":
			return
		self._disconnect_popup()
		self._popup = popup
		self._popup_latch = latch
		# A replacement popup needs its own terminal authorization.
		self._terminal_dispatch_authorized = False
		latch.terminal.connect(self._on_popup_terminal)
		latch.destroyed_without_terminal.connect(self._on_popup_destroyed)
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
			latch = self._popup_latch
			if latch is not None and latch.terminal_seen:
				# Qt can retain a just-hidden popup as active for this queued turn.
				self._disconnect_popup()
				self.run_once()
			return
		self.arm_popup(popup, self._handoff._popup_latch_for(popup))

	def run_once(self) -> None:
		"""Run the atomic guard then handler exactly once while both owners live."""
		if self._state != "waiting":
			return
		if not self._handoff._owner_is_live() or not self._action_is_live():
			self.cancel()
			return
		self._state = "running"
		try:
			self._handoff._before_incoming_action(self._action, self._checked)
		except FerrumInteractionActionHandoffRefusal as refusal:
			self._fail(str(refusal))
			return
		try:
			if self._accepts_checked:
				self._handler(self._checked)
			else:
				self._handler()
		except FerrumInteractionActionHandoffRefusal as refusal:
			self._fail(str(refusal))
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
		self._handoff._report_failure(detail)

	def _finish(self) -> None:
		"""Release all retained Qt and Python state after a successful invocation."""
		self._state = "finished"
		self._cleanup()

	def _cleanup(self) -> None:
		"""Release popup and action state after a terminal outcome."""
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
			) -> None:
		"""Create the window-owned shared action handoff."""
		super().__init__(owner)
		if not callable(failure_reporter):
			raise TypeError("Ferrum interaction failure reporter must be callable")
		self._owner = owner
		self._failure_reporter = failure_reporter
		self._owner_destroyed = False
		self._capture_canceller: collections.abc.Callable[[bool], None] | None = None
		self._actions: dict[PySide6.QtGui.QAction, object] = {}
		self._registered_action_menus: weakref.WeakKeyDictionary[
			PySide6.QtGui.QAction, weakref.WeakSet[PySide6.QtWidgets.QMenu],
		] = weakref.WeakKeyDictionary()
		self._continuations: dict[PySide6.QtGui.QAction, _PopupActionContinuation] = {}
		self._popup_latches: weakref.WeakKeyDictionary[
			PySide6.QtWidgets.QWidget, _PopupTerminalLatch] = weakref.WeakKeyDictionary()
		if PySide6.QtWidgets.QApplication.instance() is None:
			raise RuntimeError("Ferrum interaction handoff requires QApplication")
		owner.destroyed.connect(self._on_owner_destroyed)

	def register_pointer_capture_canceller(self,
			canceller: collections.abc.Callable[[bool], None]) -> None:
		"""Register the one capture cancelled by the window authoring transaction."""
		if not callable(canceller):
			raise TypeError("Ferrum pointer capture canceller must be callable")
		self._capture_canceller = canceller

	#============================================
	def cancel_registered_pointer_capture(self, *, clear_status: bool) -> None:
		"""Cancel the registered capture through its fixed explicit callback contract."""
		canceller = self._capture_canceller
		if canceller is not None:
			canceller(clear_status)

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
			self, action, handler, accepts_checked, checked,
		)
		self._continuations[action] = continuation
		popup = PySide6.QtWidgets.QApplication.activePopupWidget()
		if popup is None:
			continuation.run_once()
		else:
			continuation.arm_popup(popup, self._popup_latch_for(popup))

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Record terminals only for popups explicitly owned by a handoff."""
		if not isinstance(watched, PySide6.QtWidgets.QWidget):
			return False
		latch = self._popup_latches.get(watched)
		if latch is None:
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.Show:
			latch.rearm_for_show()
		elif event.type() == PySide6.QtCore.QEvent.Type.Hide:
			latch.mark_terminal()
		return False

	def _popup_latch_for(self, popup: PySide6.QtWidgets.QWidget) -> _PopupTerminalLatch:
		"""Return the popup-owned latch whose terminal state resets on each Show."""
		latch = self._popup_latches.get(popup)
		if latch is None:
			latch = _PopupTerminalLatch(self, popup)
			self._popup_latches[popup] = latch
			popup.installEventFilter(self)
		return latch

	def _release_popup_latch(self, latch: _PopupTerminalLatch) -> None:
		"""Forget a destroyed popup's latch before its queued QObject deletion."""
		for popup, registered_latch in tuple(self._popup_latches.items()):
			if registered_latch is latch:
				del self._popup_latches[popup]
				return

	def _before_incoming_action(self, action: PySide6.QtGui.QAction,
			checked: bool = False) -> None:
		"""Cancel prior pointer ownership while preserving the incoming tool state."""
		self._uncheck_other_registered_actions(action)
		self._owner.cancel_active_pointer_authoring(clear_status=False)
		if checked and action.isCheckable() and action.isEnabled():
			action.setChecked(True)

	def _uncheck_other_registered_actions(self, incoming: PySide6.QtGui.QAction) -> None:
		"""Clear checked pointer-tool presentation before the next action owns input."""
		for action in tuple(self._actions):
			if (
				action is not incoming
				and shiboken6.isValid(action)
				and action.isCheckable()
			):
				action.setChecked(False)

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
		for continuation in tuple(continuations.values()):
			continuation.cancel()
		self._actions.clear()
		self._registered_action_menus.clear()
		self._popup_latches.clear()
		self._capture_canceller = None

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
