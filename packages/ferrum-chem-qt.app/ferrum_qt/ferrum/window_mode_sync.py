"""Per-window active-tool ownership and typed tool-state publication."""

# Standard Library
import collections.abc
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.mode_manager


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumActiveToolState:
	"""One active feature tool, or the explicit inactive window state."""

	mode_id: str | None
	status_label: str
	supplies_drawing_defaults: bool = False


_INACTIVE_TOOL_STATE = FerrumActiveToolState(None, "None")


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumWindowToolBinding:
	"""Feature-owned registration for one exact QAction tool client."""

	action: PySide6.QtGui.QAction
	mode_id: ferrum_qt.modes.base_mode.ModeId
	mode_controller: ferrum_qt.modes.base_mode.InteractionMode
	status_label: str
	supplies_drawing_defaults: bool
	context_provider: collections.abc.Callable[[], ferrum_qt.modes.base_mode.ModeContext]
	activate_endpoint: collections.abc.Callable[[ferrum_qt.modes.base_mode.ModeContext], bool | None]
	dispatch_endpoint: collections.abc.Callable[
		[ferrum_qt.modes.base_mode.ModeContext, ferrum_qt.modes.base_mode.ModeIntent], None,
	]
	cancel_endpoint: collections.abc.Callable[[ferrum_qt.modes.base_mode.ModeContext], None]


#============================================
class FerrumWindowModeSync:
	"""Own active QAction state and document-free mode dispatch for one window."""

	def __init__(self, registry: object) -> None:
		"""Create a controller whose only QAction source is registry lookup."""
		if not callable(getattr(registry, "all_actions", None)):
			raise TypeError("Ferrum window mode sync needs an ActionRegistry lookup.")
		self._registry = registry
		self._bindings_by_action: dict[int, FerrumWindowToolBinding] = {}
		self._active_binding: FerrumWindowToolBinding | None = None
		self._activation_in_progress = False
		self._native_input_host: object | None = None
		self._subscribers: list[collections.abc.Callable[[FerrumActiveToolState], None]] = []
		self._mode_manager = ferrum_qt.modes.mode_manager.ModeManager(self._dispatch_intent)

	@property
	def mode_manager(self) -> ferrum_qt.modes.mode_manager.ModeManager:
		"""Return the controller-owned transient mode lifecycle manager."""
		return self._mode_manager

	@property
	def active_state(self) -> FerrumActiveToolState:
		"""Return the latest typed active-tool state."""
		binding = self._active_binding
		if binding is None:
			return _INACTIVE_TOOL_STATE
		return FerrumActiveToolState(
			binding.mode_id.value, binding.status_label, binding.supplies_drawing_defaults,
		)

	@property
	def activation_in_progress(self) -> bool:
		"""Report the provisional interval while a feature endpoint acquires state."""
		return self._activation_in_progress

	def subscribe(self, callback: collections.abc.Callable[[FerrumActiveToolState], None]) -> None:
		"""Register one passive window-chrome state client."""
		if not callable(callback):
			raise TypeError("Ferrum active-tool subscribers must be callable.")
		self._subscribers.append(callback)
		callback(self.active_state)

	#============================================
	def set_native_input_host(self, host: object) -> None:
		"""Bind the one window that owns this controller's native viewport filter."""
		if self._native_input_host is not None:
			raise RuntimeError("Ferrum mode sync already has a native input host.")
		if not callable(getattr(host, "_acquire_controller_native_viewport", None)):
			raise TypeError("Ferrum mode sync native host cannot acquire a viewport.")
		if not callable(getattr(host, "_release_controller_native_viewport", None)):
			raise TypeError("Ferrum mode sync native host cannot release a viewport.")
		self._native_input_host = host
		self._synchronize_native_input_viewport()

	def register_tool(self, binding: FerrumWindowToolBinding) -> None:
		"""Register one feature action and reject incomplete contracts early."""
		if type(binding) is not FerrumWindowToolBinding:
			raise TypeError("Ferrum tool registration requires a FerrumWindowToolBinding.")
		if not isinstance(binding.action, PySide6.QtGui.QAction):
			raise TypeError("Ferrum tool bindings require an existing QAction.")
		if type(binding.mode_id) is not ferrum_qt.modes.base_mode.ModeId:
			raise TypeError("Ferrum tool bindings require an exact ModeId.")
		if not isinstance(binding.mode_controller, ferrum_qt.modes.base_mode.InteractionMode):
			raise TypeError("Ferrum tool bindings require a feature InteractionMode.")
		if binding.mode_controller.mode_id is not binding.mode_id:
			raise RuntimeError("Ferrum feature mode controller must match its binding ModeId.")
		if type(binding.status_label) is not str or not binding.status_label:
			raise RuntimeError("Ferrum tool bindings require a visible status label.")
		if type(binding.supplies_drawing_defaults) is not bool:
			raise TypeError("Ferrum tool bindings require an explicit defaults capability.")
		if not callable(binding.context_provider):
			raise RuntimeError("Ferrum tool bindings require a context endpoint.")
		if not callable(binding.activate_endpoint):
			raise RuntimeError("Ferrum tool bindings require an activation endpoint.")
		if not callable(binding.dispatch_endpoint):
			raise RuntimeError("Ferrum tool bindings require a dispatch endpoint.")
		if not callable(binding.cancel_endpoint):
			raise RuntimeError("Ferrum tool bindings require a cancellation endpoint.")
		if not binding.action.isCheckable():
			raise RuntimeError("Ferrum tool bindings require a checkable feature QAction.")
		if id(binding.action) in self._bindings_by_action:
			raise RuntimeError("Ferrum feature QAction is already bound to a window tool.")
		if not any(
			self._registry.get_qt_action(action_id) is binding.action
			for action_id in self._registry.all_actions()
		):
			raise RuntimeError("Ferrum tool QAction must be registered before mode binding.")
		self._bindings_by_action[id(binding.action)] = binding
		binding.action.triggered.connect(
			lambda checked, target=binding: self._on_action_triggered(target, checked),
		)

	def select_action(self, action: PySide6.QtGui.QAction) -> bool:
		"""Programmatically select one registered feature QAction through its handler."""
		binding = self._bindings_by_action.get(id(action))
		if binding is None:
			raise RuntimeError("Ferrum cannot select an unregistered feature tool QAction.")
		if not action.isEnabled():
			return False
		if binding is self._active_binding and action.isChecked():
			self._on_action_triggered(binding, True)
			return True
		action.trigger()
		return True

	def cancel(self) -> bool:
		"""Retire one active feature through its declared cleanup endpoint."""
		binding = self._active_binding
		if binding is None:
			return False
		self._release_native_input_viewport()
		context = binding.context_provider()
		if type(context) is not ferrum_qt.modes.base_mode.ModeContext:
			raise TypeError("Ferrum tool context providers must return ModeContext.")
		try:
			self._active_binding = None
			self._mode_manager.cancel(context)
			binding.cancel_endpoint(context)
		finally:
			self._clear_active_state()
		return True

	def handle_pointer(self, pointer: ferrum_qt.modes.base_mode.PointerInput) -> bool:
		"""Send one normalized pointer input through the active feature binding."""
		binding = self._require_active_binding()
		context = self._context_for(binding)
		return self._mode_manager.handle_pointer(pointer, context)

	def handle_key(self, key: str,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers = PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			) -> bool:
		"""Send one normalized key through the active lifecycle owner."""
		binding = self._require_active_binding()
		context = self._context_for(binding)
		if key == "Escape":
			self.cancel()
			return True
		return self._mode_manager.handle_key(key, context, modifiers)

	def _on_action_triggered(self, binding: FerrumWindowToolBinding, checked: bool) -> None:
		"""Activate in order: manager, provisional binding/checks, endpoint, publish.

		A feature endpoint may observe the controller while it acquires its native
		resource.  Its activation therefore runs only after this controller has a
		consistent active binding and QAction state.  A false outcome or exception
		cancels that provisional lifecycle before inactive state is published.
		"""
		if not checked:
			if binding is self._active_binding:
				self.cancel()
			return
		if not binding.action.isEnabled():
			binding.action.setChecked(False)
			return
		if binding is self._active_binding:
			return
		context = self._context_for(binding)
		previous = self._active_binding
		if previous is not None:
			self.cancel()
		try:
			self._mode_manager.activate_feature_mode(binding.mode_controller, context)
			self._active_binding = binding
			for candidate in self._bindings_by_action.values():
				candidate.action.setChecked(candidate is binding)
			self._activation_in_progress = True
			try:
				activated = binding.activate_endpoint(context)
			finally:
				self._activation_in_progress = False
		except Exception:
			self._mode_manager.cancel(context)
			try:
				binding.cancel_endpoint(context)
			finally:
				self._clear_active_state()
			raise
		if activated is False:
			self._mode_manager.cancel(context)
			try:
				binding.cancel_endpoint(context)
			finally:
				self._clear_active_state()
			return
		self._synchronize_native_input_viewport()
		self._publish()

	#============================================
	def synchronize_native_input_viewport(self) -> None:
		"""Reconcile the controller filter with the current native document tab."""
		self._synchronize_native_input_viewport()

	def _dispatch_intent(self, context: ferrum_qt.modes.base_mode.ModeContext,
			intent: ferrum_qt.modes.base_mode.ModeIntent) -> None:
		"""Dispatch normalized input to the declared active feature endpoint."""
		binding = self._active_binding
		if binding is None:
			raise RuntimeError("Ferrum mode intent has no active feature binding.")
		binding.dispatch_endpoint(context, intent)

	def _context_for(self, binding: FerrumWindowToolBinding) -> ferrum_qt.modes.base_mode.ModeContext:
		"""Resolve one exact feature context at the controller boundary."""
		context = binding.context_provider()
		if type(context) is not ferrum_qt.modes.base_mode.ModeContext:
			raise TypeError("Ferrum tool context providers must return ModeContext.")
		return context

	def _require_active_binding(self) -> FerrumWindowToolBinding:
		"""Reject native input that escaped the feature-selection lifecycle."""
		binding = self._active_binding
		if binding is None:
			raise RuntimeError("Ferrum mode input has no active feature binding.")
		return binding

	def _clear_active_state(self) -> None:
		"""Leave every passive action client in the explicit inactive state."""
		self._active_binding = None
		for candidate in self._bindings_by_action.values():
			candidate.action.setChecked(False)
		self._publish()

	#============================================
	def _synchronize_native_input_viewport(self) -> None:
		"""Acquire the active viewport only while this controller has one tool."""
		host = self._native_input_host
		if host is None:
			return
		if self._active_binding is None:
			host._release_controller_native_viewport()
			return
		host._acquire_controller_native_viewport()

	#============================================
	def _release_native_input_viewport(self) -> None:
		"""Release the retained viewport before feature cancellation publishes state."""
		host = self._native_input_host
		if host is not None:
			host._release_controller_native_viewport()

	def _publish(self) -> None:
		"""Notify passive clients after every completed active-tool transition."""
		state = self.active_state
		for callback in tuple(self._subscribers):
			callback(state)


#============================================
class FerrumNativeWindowModeSyncMixin:
	"""Bridge native pointer cancellation to the per-window tool controller."""

	def _acquire_controller_native_viewport(self) -> None:
		"""Install this window once on the exact active native viewport."""
		tab = self._active_native_tab()
		viewport = None if tab is None or tab.is_disposed else tab.view.viewport()
		if viewport is self._controller_native_viewport:
			return
		self._release_controller_native_viewport()
		if viewport is not None:
			viewport.installEventFilter(self)
			self._controller_native_viewport = viewport

	#============================================
	def _release_controller_native_viewport(self) -> None:
		"""Remove this window from the exact viewport retained at acquisition."""
		viewport = self._controller_native_viewport
		if viewport is not None:
			viewport.removeEventFilter(self)
			self._controller_native_viewport = None

	def _synchronize_mode_state(self, mode_id: str | None = None) -> None:
		"""Publish only genuine native cancellation; QAction triggers select tools."""
		del mode_id
		if (
			self._atom_insertion_intent is None
			and self._line_gesture_intent is None
			and not self._window_mode_sync.activation_in_progress
		):
			self._window_mode_sync.cancel()

	@staticmethod
	def _typed_refusal(context: str, outcome: str, details: str,
			primary_message: str | None = None) -> object:
		"""Build an exact refusal at the native-window boundary."""
		refusal = ferrum_qt.dialogs.refusal_presenter
		return refusal.RefusalRequest(
			refusal.RefusalTaskContext(context), refusal.RefusalOutcome(outcome),
			technical_details=details, primary_message=primary_message,
		)

	@staticmethod
	def _unavailable_edit_refusal(details: str,
			primary_message: str | None = None) -> object:
		"""Build the explicit refused-edit fact for one feature boundary."""
		return FerrumNativeWindowModeSyncMixin._typed_refusal(
			"edit_document", "unavailable_operation", details, primary_message,
		)

	@staticmethod
	def _admitted_session_transition_refusal(error: Exception) -> object | None:
		"""Map generic Rust transition refusals without parsing diagnostic prose."""
		import ferrum_qt.ferrum.engine as engine
		details = str(error)
		if isinstance(error, (
			engine.RevisionConflictError,
			engine.PreparedOperationStaleSnapshotError,
		)):
			return FerrumNativeWindowModeSyncMixin._typed_refusal(
				"edit_document", "edit_stale_snapshot", details,
			)
		if isinstance(error, engine.PreparedOperationRendererAdmissionError):
			return FerrumNativeWindowModeSyncMixin._typed_refusal(
				"edit_document", "edit_renderer_refusal", details,
			)
		if isinstance(error, (
			engine.PreparedOperationForeignSessionError,
			engine.PreparedOperationConsumedError,
			engine.PreparedOperationProvisionalCapabilityError,
		)):
			return FerrumNativeWindowModeSyncMixin._typed_refusal(
				"edit_document", "edit_session_conflict", details,
			)
		return None
