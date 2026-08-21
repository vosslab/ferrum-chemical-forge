"""Behavioral tests for the document-free Ferrum mode-controller seam."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.mode_manager


#============================================
@dataclasses.dataclass
class _DispatchRecorder:
	"""Record host-bound operations without supplying a document model."""

	intents: list[ferrum_qt.modes.base_mode.ModeIntent]

	def __call__(self, context: ferrum_qt.modes.base_mode.ModeContext,
			intent: ferrum_qt.modes.base_mode.ModeIntent) -> None:
		"""Keep the intent that a real Ferrum tab adapter would execute."""
		self.intents.append(intent)


#============================================
def _context() -> ferrum_qt.modes.base_mode.ModeContext:
	"""Return opaque immutable observation/context values for one interaction."""
	context = ferrum_qt.modes.base_mode.ModeContext(
		observation=("revision", 7), dispatch_context=("tab", "active"),
	)
	return context


#============================================
def _pointer(phase: ferrum_qt.modes.base_mode.PointerPhase,
		x: float, y: float) -> ferrum_qt.modes.base_mode.PointerInput:
	"""Normalize a small canvas event without importing Qt in this test."""
	pointer = ferrum_qt.modes.base_mode.PointerInput(
		phase, ferrum_qt.modes.base_mode.ScenePoint(x, y),
	)
	return pointer


#============================================
def test_draw_mode_dispatches_one_completed_bond_gesture() -> None:
	"""Draw mode retains only local pointer geometry until a release completes it."""
	recorder = _DispatchRecorder([])
	manager = ferrum_qt.modes.mode_manager.ModeManager(recorder)
	context = _context()
	manager.activate(ferrum_qt.modes.base_mode.ModeId.DRAW, context)
	manager.handle_pointer(_pointer(ferrum_qt.modes.base_mode.PointerPhase.PRESS, 1.0, 2.0), context)
	consumed = manager.handle_pointer(
		_pointer(ferrum_qt.modes.base_mode.PointerPhase.RELEASE, 4.0, 6.0), context,
	)
	assert consumed is True
	assert recorder.intents[-1].operation_id == "bond.draw"


#============================================
def test_switching_modes_retires_partial_bracket_gesture() -> None:
	"""A prior mode cannot commit stale local geometry after a mode switch."""
	recorder = _DispatchRecorder([])
	manager = ferrum_qt.modes.mode_manager.ModeManager(recorder)
	context = _context()
	manager.activate(ferrum_qt.modes.base_mode.ModeId.BRACKET, context)
	manager.handle_pointer(_pointer(ferrum_qt.modes.base_mode.PointerPhase.PRESS, 1.0, 2.0), context)
	manager.activate(ferrum_qt.modes.base_mode.ModeId.DRAW, context)
	consumed = manager.handle_pointer(
		_pointer(ferrum_qt.modes.base_mode.PointerPhase.RELEASE, 4.0, 6.0), context,
	)
	assert consumed is True
	assert recorder.intents == []


#============================================
def test_escape_cancels_without_dispatching_a_document_operation() -> None:
	"""Escape owns only Qt interaction state and leaves the host untouched."""
	recorder = _DispatchRecorder([])
	manager = ferrum_qt.modes.mode_manager.ModeManager(recorder)
	context = _context()
	manager.activate(ferrum_qt.modes.base_mode.ModeId.ATOM, context)
	cancelled = manager.handle_key("Escape", context)
	assert cancelled is True
	assert recorder.intents == []


#============================================
def test_arrow_mode_dispatches_the_existing_selected_arrow_seam() -> None:
	"""Arrow editing stays an injected operation, not a Python model mutation."""
	recorder = _DispatchRecorder([])
	manager = ferrum_qt.modes.mode_manager.ModeManager(recorder)
	context = _context()
	manager.activate(ferrum_qt.modes.base_mode.ModeId.ARROW, context)
	manager.handle_pointer(
		_pointer(ferrum_qt.modes.base_mode.PointerPhase.RELEASE, 3.0, 5.0), context,
	)
	assert recorder.intents[-1].operation_id == "arrow.edit_selected"

