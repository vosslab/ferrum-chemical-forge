"""Shared-mode publication and typed-refusal helpers for the native window."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.line_tool_intent


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumActiveToolState:
	"""Public transient-tool state consumed by shared window chrome."""

	mode_id: str | None
	status_label: str


_INACTIVE_TOOL_STATE = FerrumActiveToolState(None, "None")

_LINE_TOOL_STATES = {
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_BOND:
		FerrumActiveToolState("draw", "Draw Bond"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_ARROW:
		FerrumActiveToolState("arrow", "Draw Arrow"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_EQUILIBRIUM_ARROW:
		FerrumActiveToolState("arrow", "Draw Equilibrium Arrow"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_CURVED_ELECTRON_ARROW:
		FerrumActiveToolState("arrow", "Draw Curved Electron Arrow"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_CURVED_RETRO_ARROW:
		FerrumActiveToolState("arrow", "Draw Curved Retro Arrow"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_CURVED_REACTION_ARROW:
		FerrumActiveToolState("arrow", "Draw Curved Reaction Arrow"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_CURVED_EQUILIBRIUM_ARROW:
		FerrumActiveToolState("arrow", "Draw Curved Equilibrium Arrow"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_PLUS:
		FerrumActiveToolState(None, "Draw Plus"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_LINE:
		FerrumActiveToolState(None, "Draw Line"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_RECTANGLE:
		FerrumActiveToolState("vector", "Draw Rectangle"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_SQUARE:
		FerrumActiveToolState("vector", "Draw Square"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_OVAL:
		FerrumActiveToolState("vector", "Draw Oval"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_CIRCLE:
		FerrumActiveToolState("vector", "Draw Circle"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_POLYLINE:
		FerrumActiveToolState("vector", "Draw Polyline"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_POLYGON:
		FerrumActiveToolState("vector", "Draw Polygon"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.INSERT_TEXT:
		FerrumActiveToolState(None, "Insert Text"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.CREATE_WAVY:
		FerrumActiveToolState(None, "Draw Wavy Line"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.CREATE_RECTANGULAR_BRACKET:
		FerrumActiveToolState("bracket", "Draw Rectangular Bracket"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.CREATE_ROUND_BRACKET:
		FerrumActiveToolState(None, "Draw Round Bracket"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.MOVE_ATOM:
		FerrumActiveToolState("edit", "Move Atom"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.ROTATE_ATOMS:
		FerrumActiveToolState(None, "Rotate Atoms"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.TRANSLATE_ROOTS:
		FerrumActiveToolState(None, "Translate Roots"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.INSERT_CYCLOHEXANE_RING:
		FerrumActiveToolState(None, "Insert Cyclohexane Ring"),
	ferrum_qt.ferrum.line_tool_intent._NativeLineTool.ATTACH_CYCLOHEXANE_RING:
		FerrumActiveToolState(None, "Attach Cyclohexane Ring"),
}


#============================================
class FerrumNativeWindowModeSyncMixin:
	"""Keep transient tool chrome and typed warning requests at one boundary."""

	#============================================
	def _synchronize_mode_state(self, mode_id: str | None = None) -> None:
		"""Publish actual transient tool ownership to shared mode presentation."""
		if mode_id is None:
			if self._atom_insertion_intent is not None:
				state = FerrumActiveToolState("atom", "Add Atom")
			elif self._line_gesture_intent is not None:
				state = _LINE_TOOL_STATES[self._line_gesture_intent.tool]
			else:
				state = _INACTIVE_TOOL_STATE
		else:
			state = FerrumActiveToolState(mode_id, mode_id.replace("_", " ").title())
		from ferrum_qt.ferrum import window_shared_seams
		window_shared_seams.synchronize_active_tool_state(self, state)

	#============================================
	@staticmethod
	def _typed_refusal(context: str, outcome: str, details: str,
			primary_message: str | None = None) -> object:
		"""Build an exact refusal at the native-window boundary."""
		refusal = ferrum_qt.dialogs.refusal_presenter
		return refusal.RefusalRequest(
			refusal.RefusalTaskContext(context), refusal.RefusalOutcome(outcome),
			technical_details=details, primary_message=primary_message,
		)

	#============================================
	@staticmethod
	def _unavailable_edit_refusal(details: str,
			primary_message: str | None = None) -> object:
		"""Build the explicit refused-edit fact for one feature boundary."""
		return FerrumNativeWindowModeSyncMixin._typed_refusal(
			"edit_document", "unavailable_operation", details, primary_message,
		)
