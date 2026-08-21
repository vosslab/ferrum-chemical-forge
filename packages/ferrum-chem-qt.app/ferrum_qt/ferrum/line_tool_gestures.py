"""Composition boundary for Ferrum's active line-tool pointer gestures."""

# local repo modules
from ferrum_qt.ferrum.line_tool_completion import FerrumNativeLineToolCompletionMixin
from ferrum_qt.ferrum.line_tool_pointer import FerrumNativeLineToolPointerMixin


class FerrumNativeLineToolGesturesMixin(
		FerrumNativeLineToolPointerMixin,
		FerrumNativeLineToolCompletionMixin,
		):
	"""Compose pointer dispatch with specialised completion handlers."""
