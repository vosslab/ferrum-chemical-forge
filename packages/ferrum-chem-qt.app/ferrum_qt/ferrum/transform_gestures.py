"""Rotation gesture behavior composed by the Ferrum pointer-tool host."""

# Standard Library
import dataclasses
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.ferrum.line_tool_intent
import ferrum_qt.ferrum.rotation


#============================================
class FerrumNativeTransformGesturesMixin:
	"""Own preview-only rotation behavior for the common pointer-tool host."""

	#============================================
	def _start_rotation_gesture(
			self,
			intent: ferrum_qt.ferrum.line_tool_intent._LineGestureIntent,
			press_scene: PySide6.QtCore.QPointF,
			) -> None:
		"""Capture exact projected atoms and create one local skeleton preview."""
		try:
			selection = intent.tab.selected_atom_rotation()
			dx = press_scene.x() - selection.center.x()
			dy = press_scene.y() - selection.center.y()
			if not math.isfinite(dx) or not math.isfinite(dy):
				raise ValueError("rotation pointer position must be finite")
			if dx == 0.0 and dy == 0.0:
				self.statusBar().showMessage(
					self.tr("Start the rotation drag away from the selection center."), 5000,
				)
				return
			preview = ferrum_qt.ferrum.rotation.create_rotation_preview(
				intent.tab, selection,
			)
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		self._line_gesture_intent = dataclasses.replace(
			intent, press_scene=press_scene, start_scene=selection.center,
			rotation_selection=selection, rotation_preview=preview,
			last_angle=math.atan2(dy, dx),
		)

	#============================================
	def _update_rotation_gesture(
			self,
			intent: ferrum_qt.ferrum.line_tool_intent._LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent,
			) -> None:
		"""Advance one unwrapped angle and update only its disposable skeleton."""
		if intent.rotation_preview is None or intent.rotation_selection is None or intent.last_angle is None:
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal("The document changed during the gesture; no operation was accepted."))
			return
		current_scene = intent.tab.view.mapToScene(event.position().toPoint())
		center = intent.rotation_selection.center
		dx = current_scene.x() - center.x()
		dy = current_scene.y() - center.y()
		if not math.isfinite(dx) or not math.isfinite(dy):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal("Rotation pointer position must be finite."))
			return
		if dx == 0.0 and dy == 0.0:
			return
		current_angle = math.atan2(dy, dx)
		delta = current_angle - intent.last_angle
		if delta > math.pi:
			delta -= math.tau
		elif delta < -math.pi:
			delta += math.tau
		angle = intent.accumulated_angle + delta
		ferrum_qt.ferrum.rotation.update_rotation_preview(
			intent.rotation_preview, float(angle),
		)
		self._line_gesture_intent = dataclasses.replace(
			intent, last_angle=current_angle, accumulated_angle=angle,
		)

	#============================================
	def _complete_rotation_gesture(
			self,
			intent: ferrum_qt.ferrum.line_tool_intent._LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent,
			) -> None:
		"""Retire the local preview, then submit one still-current Rust rotation."""
		if intent.rotation_selection is None or intent.rotation_preview is None:
			return
		self._update_rotation_gesture(intent, event)
		current = self._line_gesture_intent
		if current is None or current.rotation_selection is None:
			return
		selection = current.rotation_selection
		angle = float(current.accumulated_angle)
		center = (float(selection.center.x()), float(selection.center.y()))
		self._reset_line_gesture_start()
		if angle == 0.0:
			self.statusBar().showMessage(
				self.tr("Rotate Selected Atoms remains active; no rotation was requested."), 5000,
			)
			return
		try:
			intent.tab.apply_selected_atom_rotation(selection, center, angle)
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		self._finish_line_gesture(
			intent, self.tr("Rotated selected Ferrum atoms; drag again or press Esc."),
		)
