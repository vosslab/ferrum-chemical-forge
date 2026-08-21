"""Revision-bound Qt pointer tools for the standalone Ferrum window."""

# PIP3 modules

# local repo modules
import ferrum_qt.ferrum.keyboard_authoring
import ferrum_qt.ferrum.transform_gestures
from ferrum_qt.ferrum.line_tool_actions import FerrumNativeLineToolActionsMixin
from ferrum_qt.ferrum.line_tool_gestures import FerrumNativeLineToolGesturesMixin
from ferrum_qt.ferrum.line_tool_interaction import FerrumNativeLineToolInteractionMixin


class FerrumNativeLineToolsMixin(
		FerrumNativeLineToolActionsMixin,
		FerrumNativeLineToolGesturesMixin,
		FerrumNativeLineToolInteractionMixin,
		ferrum_qt.ferrum.keyboard_authoring.FerrumKeyboardAuthoringMixin,
		ferrum_qt.ferrum.transform_gestures.FerrumNativeTransformGesturesMixin,
		):
	"""Compose Ferrum's pointer-tool responsibilities for a document host."""
