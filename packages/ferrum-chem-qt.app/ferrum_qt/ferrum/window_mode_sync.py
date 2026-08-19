"""Shared-mode publication and typed-refusal helpers for the native window."""

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.line_tool_intent


#============================================
class FerrumNativeWindowModeSyncMixin:
	"""Keep transient tool chrome and typed warning requests at one boundary."""

	#============================================
	def _synchronize_mode_state(self, mode_id: str | None = None) -> None:
		"""Publish actual transient tool ownership to shared mode presentation."""
		if mode_id is None:
			if self._atom_insertion_intent is not None:
				mode_id = "atom"
			elif self._line_gesture_intent is not None:
				mode_id = {
					ferrum_qt.ferrum.line_tool_intent._NativeLineTool.DRAW_BOND: "draw",
					ferrum_qt.ferrum.line_tool_intent._NativeLineTool.CREATE_RECTANGULAR_BRACKET: "bracket",
					ferrum_qt.ferrum.line_tool_intent._NativeLineTool.MOVE_ATOM: "edit",
				}.get(self._line_gesture_intent.tool)
		from ferrum_qt.ferrum import window_shared_seams
		window_shared_seams.synchronize_active_tool_mode(self, mode_id)

	#============================================
	@staticmethod
	def _typed_refusal(context: str, outcome: str, details: str) -> object:
		"""Build an exact refusal at the native-window boundary."""
		refusal = ferrum_qt.dialogs.refusal_presenter
		return refusal.RefusalRequest(
			refusal.RefusalTaskContext(context), refusal.RefusalOutcome(outcome),
			technical_details=details,
		)

	#============================================
	@staticmethod
	def _unavailable_edit_refusal(details: str) -> object:
		"""Build the explicit refused-edit fact for one feature boundary."""
		return FerrumNativeWindowModeSyncMixin._typed_refusal(
			"edit_document", "unavailable_operation", details,
		)
