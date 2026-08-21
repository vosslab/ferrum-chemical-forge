"""Order-sensitive handoff for Qt actions that take canvas interaction ownership."""

# Standard Library
import inspect

# PIP3 modules
import PySide6.QtGui


#============================================
class FerrumInteractionActionHandoff:
	"""Cancel one temporary canvas capture before an incoming tool activates."""

	#============================================
	def __init__(self) -> None:
		"""Start without a capture client; construction order is deliberately harmless."""
		self._cancel_capture: object | None = None
		self._actions: dict[PySide6.QtGui.QAction, object] = {}

	#============================================
	def set_capture_canceller(self, canceller: object | None) -> None:
		"""Install the one current temporary-capture cancellation client."""
		if canceller is not None and not callable(canceller):
			raise TypeError("Ferrum interaction cancellation client must be callable")
		self._cancel_capture = canceller

	#============================================
	def connect(self, action: PySide6.QtGui.QAction, handler: object) -> None:
		"""Connect one pointer-owning action through its cancellation guard.

		This is the sole action-registration seam for canvas ownership.  The
		guard and handler are one slot rather than independently ordered Qt
		subscribers, so capture retirement finishes before the real command begins.
		"""
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
			"""Retire capture, then invoke this exact command handler."""
			self._before_incoming_action(checked)
			if accepts_checked:
				handler(checked)
			else:
				handler()

		# Retain the Python callable for the QAction lifetime.
		self._actions[action] = dispatch
		action.triggered.connect(dispatch)

	#============================================
	def _before_incoming_action(self, _checked: bool = False) -> None:
		"""Retire selected-root capture synchronously before the tool handler."""
		canceller = self._cancel_capture
		if callable(canceller):
			canceller()
