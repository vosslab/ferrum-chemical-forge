"""Ferrum pointer capture and primary line-gesture dispatch."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.bond_preview
import ferrum_qt.ferrum.direct_bond_preview
import ferrum_qt.ferrum.direct_bond_gesture_tab
import ferrum_qt.ferrum.direct_root_preview
import ferrum_qt.ferrum.line_tool_intent
import ferrum_qt.ferrum.presentation_creation_preview
import ferrum_qt.ferrum.presentation_vector_preview
import ferrum_qt.ferrum.presentation_path_preview
import ferrum_qt.ferrum.regular_ring
import ferrum_qt.ferrum.rotation
import ferrum_qt.ferrum.text_placement
import ferrum_qt.ferrum.text_placement_preview
import ferrum_qt.ferrum.top_level_transform
import ferrum_qt.ferrum.translation
from ferrum_qt.ferrum.line_tool_interaction import _normalized_rect

_NativeLineTool = ferrum_qt.ferrum.line_tool_intent._NativeLineTool
_LineGestureIntent = ferrum_qt.ferrum.line_tool_intent._LineGestureIntent


class FerrumNativeLineToolPointerMixin:
	"""Own pointer capture and primary gesture state transitions."""
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Capture Ferrum pointer intent for native creation and Rust-issued overlays."""
		line_intent = self._line_gesture_intent
		if line_intent is not None and watched is line_intent.viewport:
			return self._line_gesture_event(event)
		intent = self._atom_insertion_intent
		if intent is None or watched is not intent.viewport:
			return super().eventFilter(watched, event)
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			if (
				self._line_gesture_intent is not None
				and self._line_gesture_intent.tool in self._completion_click_actions
				and event.key() in (PySide6.QtCore.Qt.Key.Key_Return, PySide6.QtCore.Qt.Key.Key_Enter)
			):
				self._complete_click_presentation_gesture(self._line_gesture_intent)
				return True
			if self._keyboard_canvas_key_event(event):
				return True
			return super().eventFilter(watched, event)
		if event.type() != PySide6.QtCore.QEvent.Type.MouseButtonPress:
			return super().eventFilter(watched, event)
		if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
			return False
		self._complete_atom_insertion(event)
		return True

	#============================================
	def _line_gesture_event(self, event: PySide6.QtCore.QEvent) -> bool:
		"""Consume one pointer gesture without creating a Qt document model."""
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			if (
				self._line_gesture_intent is not None
				and self._line_gesture_intent.tool in self._completion_click_actions
				and event.key() in (PySide6.QtCore.Qt.Key.Key_Return, PySide6.QtCore.Qt.Key.Key_Enter)
			):
				self._complete_click_presentation_gesture(self._line_gesture_intent)
				return True
			if self._keyboard_canvas_key_event(event):
				return True
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.FocusOut:
			self._cancel_line_tool_after_focus_handoff(self._line_gesture_intent)
			return False
		if event.type() in (
			PySide6.QtCore.QEvent.Type.UngrabMouse,
			PySide6.QtCore.QEvent.Type.Hide,
		):
			self._cancel_line_gesture()
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonPress:
			if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
				return False
			try:
				if (
					self._line_gesture_intent is not None
					and self._line_gesture_intent.tool in self._completion_click_actions
					and (
						self._line_gesture_intent.path_gesture is not None
						or self._line_gesture_intent.presentation_gesture is not None
						or self._line_gesture_intent.curved_electron_points
					)
				):
					self._append_click_presentation_point(event)
					return True
				self._start_line_gesture(event)
			except (TypeError, ValueError) as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return True
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonDblClick:
			if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
				return False
			if self._line_gesture_intent is not None and self._line_gesture_intent.tool in self._completion_click_actions:
				self._append_click_presentation_point(event)
				if self._line_gesture_intent is not None:
					self._complete_click_presentation_gesture(self._line_gesture_intent)
				return True
		if event.type() == PySide6.QtCore.QEvent.Type.MouseMove:
			try:
				self._update_line_gesture(event)
			except (TypeError, ValueError) as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return self._line_gesture_intent is not None
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonRelease:
			if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
				return False
			if self._line_gesture_intent is not None and self._line_gesture_intent.tool in self._draw_path_actions:
				return True
			try:
				self._complete_line_gesture(event)
			except (TypeError, ValueError) as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return True
		return False

	#============================================
	def _restore_line_tool_focus_on_next_turn(self, intent: _LineGestureIntent) -> None:
		"""Reclaim the viewport after a menu action finishes its focus teardown."""
		PySide6.QtCore.QTimer.singleShot(
			0, lambda: self._restore_line_tool_focus_if_current(intent),
		)

	#============================================
	def _restore_line_tool_focus_if_current(self, intent: _LineGestureIntent) -> None:
		"""Request focus only for the same still-armed pointer intent."""
		if self._line_gesture_intent is intent:
			intent.viewport.setFocus()

	#============================================
	def _cancel_line_tool_after_focus_handoff(self, intent: _LineGestureIntent) -> None:
		"""Defer focus-loss retirement so a popup-to-canvas handoff can settle."""
		PySide6.QtCore.QTimer.singleShot(
			0, lambda: self._cancel_line_tool_if_still_unfocused(intent),
		)

	#============================================
	def _cancel_line_tool_if_still_unfocused(self, intent: _LineGestureIntent) -> None:
		"""Retire only the exact intent that still owns an unfocused viewport."""
		if self._line_gesture_intent is intent and not intent.viewport.hasFocus():
			self._cancel_line_gesture()

	#============================================
	def _start_line_gesture(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Capture a durable start atom and create one disposable local preview."""
		intent = self._line_gesture_intent
		if intent is None:
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal("The document changed before the gesture; start the tool again."))
			return
		point = event.position().toPoint()
		press_scene = intent.tab.view.mapToScene(point)
		if intent.tool in (
			_NativeLineTool.DRAW_ARROW,
			_NativeLineTool.DRAW_EQUILIBRIUM_ARROW,
		):
			try:
				import ferrum_qt.ferrum.engine as engine
				if intent.tool is _NativeLineTool.DRAW_ARROW:
					gesture = intent.tab.begin_straight_normal_arrow_gesture(
						float(press_scene.x()), float(press_scene.y()),
						engine.PresentationGestureSnapPolicyV1(),
					)
				else:
					gesture = intent.tab.begin_straight_equilibrium_arrow_gesture(
						float(press_scene.x()), float(press_scene.y()),
						engine.PresentationGestureSnapPolicyV1(),
					)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._presentation_gesture_refusal(exc, intent.tool))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, press_scene=press_scene, presentation_gesture=gesture,
			)
			return
		if intent.tool in self._draw_vector_actions:
			try:
				import ferrum_qt.ferrum.engine as engine
				kind = {
					_NativeLineTool.DRAW_LINE: engine.PresentationVectorKindV1.line,
					_NativeLineTool.DRAW_RECTANGLE: engine.PresentationVectorKindV1.rectangle,
					_NativeLineTool.DRAW_SQUARE: engine.PresentationVectorKindV1.square,
					_NativeLineTool.DRAW_OVAL: engine.PresentationVectorKindV1.oval,
					_NativeLineTool.DRAW_CIRCLE: engine.PresentationVectorKindV1.circle,
				}[intent.tool]
				gesture = intent.tab.begin_presentation_vector_gesture(
					kind, float(press_scene.x()), float(press_scene.y()),
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._vector_gesture_refusal(exc))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, press_scene=press_scene, vector_gesture=gesture,
			)
			return
		if intent.tool in self._draw_path_actions:
			try:
				import ferrum_qt.ferrum.engine as engine
				kind = {
					_NativeLineTool.DRAW_POLYLINE: engine.PresentationPathKindV1.polyline,
					_NativeLineTool.DRAW_POLYGON: engine.PresentationPathKindV1.polygon,
				}[intent.tool]
				gesture = intent.tab.begin_presentation_path_gesture(kind)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._vector_gesture_refusal(exc))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, press_scene=press_scene, path_gesture=gesture,
				path_points=((float(press_scene.x()), float(press_scene.y())),),
			)
			self._show_presentation_path_point_guidance(intent.tool, 1)
			return
		if intent.tool is _NativeLineTool.DRAW_CURVED_ELECTRON_ARROW:
			self._line_gesture_intent = dataclasses.replace(
				intent, press_scene=press_scene,
				curved_electron_points=((float(press_scene.x()), float(press_scene.y())),),
			)
			self._show_curved_electron_point_guidance(1)
			return
		if intent.tool is _NativeLineTool.DRAW_PLUS:
			try:
				gesture = intent.tab.begin_plus_placement_gesture(
					float(press_scene.x()), float(press_scene.y()),
				)
				preview = intent.tab.preview_plus_placement_gesture(gesture)
				overlay = ferrum_qt.ferrum.presentation_creation_preview.create_plus_overlay(
					intent.tab, preview.overlay,
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._presentation_gesture_refusal(exc))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, press_scene=press_scene, presentation_gesture=gesture,
				presentation_preview=preview, preview=overlay,
			)
			return
		if intent.tool is _NativeLineTool.INSERT_TEXT:
			dialog = None
			font = None
			runs = None
			try:
				gesture = intent.tab.begin_text_placement_gesture(
					float(press_scene.x()), float(press_scene.y()),
				)
				defaults = intent.tab.text_placement_defaults(gesture)
				model = ferrum_qt.ferrum.text_placement.dialog_model_from_defaults(defaults)
				dialog = ferrum_qt.ferrum.text_placement.dialog_for_placement(model, self)
				accepted = dialog.exec() == PySide6.QtWidgets.QDialog.DialogCode.Accepted
				if accepted:
					font = dialog.font_values()
					runs = ferrum_qt.ferrum.text_placement.runs_from_dialog(dialog.get_runs())
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._text_placement_refusal(exc))
				return
			finally:
				if dialog is not None:
					dialog.deleteLater()
			if not accepted:
				self._cancel_line_gesture(clear_status=False)
				intent.viewport.setFocus()
				self.statusBar().showMessage(self.tr(
					"Text insertion cancelled. Selection and document are unchanged.",
				), 5000)
				return
			try:
				if type(font) is not dict or type(runs) is not tuple:
					raise RuntimeError("Ferrum Text dialog did not return immutable authoring values")
				preview = intent.tab.preview_text_placement_gesture(
					gesture, runs,
					None if font["font_size"] == model.font_size else font["font_size"],
					None if font["font_color"].lower() == model.color.lower()
					else font["font_color"],
				)
				overlay = ferrum_qt.ferrum.text_placement_preview.create_text_placement_overlay(
					intent.tab, preview.overlay,
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._text_placement_refusal(exc))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, press_scene=press_scene, text_gesture=gesture,
				text_preview=preview, preview=overlay,
			)
			self._complete_text_placement_gesture()
			return
		if intent.tool is _NativeLineTool.INSERT_CYCLOHEXANE_RING:
			center = intent.tab.view.snap_authored_scene_point(press_scene)
			if (
				intent.tab.durable_atom_at_viewport_point(point) is not None
				or intent.tab.durable_atom_at_viewport_point(
					intent.tab.view.mapFromScene(center),
				) is not None
			):
				self.statusBar().showMessage(
					self.tr("Choose an empty page location to insert a separate ring."), 5000,
				)
				return
			try:
				prepared = ferrum_qt.ferrum.regular_ring.prepare_cyclohexane(
					intent.tab, center,
				)
				preview = ferrum_qt.ferrum.regular_ring.create_preview(
					intent.tab, prepared,
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, start_scene=center, press_scene=press_scene, preview=preview,
				regular_ring_prepared=prepared,
			)
			return
		if intent.tool is _NativeLineTool.ATTACH_CYCLOHEXANE_RING:
			atom_id = intent.tab.durable_attachment_atom_at_viewport_point(point)
			if atom_id is None:
				self.statusBar().showMessage(self.tr(
					"Attach Cyclohexane Ring needs an eligible existing atom; try again.",
				), 5000)
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, start_atom_id=atom_id, press_scene=press_scene,
			)
			return
		if intent.tool is _NativeLineTool.DRAW_BOND:
			try:
				endpoint = intent.tab.direct_bond_endpoint_at_viewport_point(point)
			except ferrum_qt.ferrum.direct_bond_gesture_tab.DirectBondEndpointAmbiguity:
				self.statusBar().showMessage(
					self.tr("Draw Bond needs one atom clearly or empty canvas."), 5000,
				)
				return
			except Exception:
				self._cancel_line_gesture()
				raise
			try:
				# Freeze the current shared next-drawing choice only after a valid
				# endpoint begins this gesture.
				drawing = self._drawing_parameters.snapshot()
				gesture = intent.tab.begin_direct_bond_gesture(
					endpoint.endpoint, drawing.bond_presentation(),
					intent.tab.view.hex_grid_snap_enabled,
				)
			except Exception as exc:
				self._cancel_line_gesture()
				if not self._is_direct_bond_begin_refusal(exc):
					raise
				self._show_edit_refusal(self._unavailable_edit_refusal(
					self._direct_bond_refusal_message(exc),
				))
				return
			self.statusBar().showMessage(self.tr(
				"Drawing a normal {0} bond. Release over an atom or empty space.",
			).format(drawing.order_name))
			self._line_gesture_intent = dataclasses.replace(
				intent,
				drawing=drawing,
				start_atom_id=endpoint.source_id,
				start_scene=press_scene,
				press_scene=press_scene,
				direct_bond_gesture=gesture,
			)
			return
		if intent.tool is _NativeLineTool.ROTATE_ATOMS:
			self._start_rotation_gesture(intent, press_scene)
			return
		if intent.tool is _NativeLineTool.TRANSLATE_ROOTS:
			self._start_translation_gesture(intent, press_scene)
			return
		if intent.tool in (
			_NativeLineTool.CREATE_WAVY,
			_NativeLineTool.CREATE_RECTANGULAR_BRACKET,
			_NativeLineTool.CREATE_ROUND_BRACKET,
		):
			start_scene = intent.tab.view.snap_authored_scene_point(press_scene)
			try:
				preview = (
					self._new_bracket_preview(intent.tab, start_scene)
					if intent.tool in (
						_NativeLineTool.CREATE_RECTANGULAR_BRACKET,
						_NativeLineTool.CREATE_ROUND_BRACKET,
					)
					else self._new_line_preview(intent.tab, start_scene)
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, start_scene=start_scene, press_scene=press_scene, preview=preview,
			)
			return
		atom_id = intent.tab.durable_atom_at_viewport_point(point)
		if atom_id is None:
			message = (
				self.tr("Draw Bond must start on an existing atom.")
				if intent.tool is _NativeLineTool.DRAW_BOND
				else self.tr("Move Atom must start on an existing atom.")
			)
			self.statusBar().showMessage(message, 5000)
			return
		start_scene = intent.tab.durable_atom_scene_position(atom_id)
		try:
			preview = self._new_line_preview(intent.tab, start_scene)
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		self._line_gesture_intent = dataclasses.replace(
			intent, start_atom_id=atom_id, start_scene=start_scene,
			press_scene=press_scene, preview=preview,
		)

	#============================================
	@staticmethod
	def _is_direct_bond_begin_refusal(error: Exception) -> bool:
		"""Accept only native begin failures that the user can correct."""
		import ferrum_qt.ferrum.engine as engine
		if type(error) not in (
			engine.DirectBondGestureError,
			engine.RevisionConflictError,
		):
			return False
		return getattr(error, "category", None) in (
			engine.DirectBondGestureCategoryV1.stale_revision,
			engine.DirectBondGestureCategoryV1.stale_digest,
			engine.DirectBondGestureCategoryV1.unknown_start_atom,
			engine.DirectBondGestureCategoryV1.unsupported_presentation,
			engine.DirectBondGestureCategoryV1.non_finite_point,
			engine.DirectBondGestureCategoryV1.invalid_snap_policy,
			engine.DirectBondGestureCategoryV1.session_conflict,
		)

	#============================================
	def _append_presentation_path_point(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Append one click to the transient ordered path without mutating Rust."""
		intent = self._line_gesture_intent
		if intent is None or intent.path_gesture is None:
			return
		point = intent.tab.view.mapToScene(event.position().toPoint())
		coordinate = (float(point.x()), float(point.y()))
		if intent.path_points and coordinate == intent.path_points[-1]:
			return
		self._line_gesture_intent = dataclasses.replace(
			intent, path_points=intent.path_points + (coordinate,),
		)
		current = self._line_gesture_intent
		minimum = 3 if intent.tool is _NativeLineTool.DRAW_POLYGON else 2
		if current is not None and len(current.path_points) < minimum:
			self._show_presentation_path_point_guidance(intent.tool, len(current.path_points))
			return
		if current is not None and len(current.path_points) >= minimum:
			self._update_presentation_path_gesture(current, event.position().toPoint(), False)

	#============================================
	def _append_click_presentation_point(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Route one click to the exact multi-click presentation authoring contract."""
		intent = self._line_gesture_intent
		if intent is None:
			return
		if intent.tool in self._draw_path_actions:
			self._append_presentation_path_point(event)
		elif intent.tool is _NativeLineTool.DRAW_CURVED_ELECTRON_ARROW:
			self._append_curved_electron_arrow_point(event)

	#============================================
	def _append_curved_electron_arrow_point(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Capture only start/control/end points; Rust owns all curve geometry."""
		intent = self._line_gesture_intent
		if intent is None:
			return
		point = intent.tab.view.mapToScene(event.position().toPoint())
		coordinate = (float(point.x()), float(point.y()))
		if coordinate in intent.curved_electron_points:
			return
		points = intent.curved_electron_points + (coordinate,)
		if len(points) == 2:
			import ferrum_qt.ferrum.engine as engine
			try:
				gesture = intent.tab.begin_curved_electron_arrow_gesture(points[0], points[1])
			except engine.CurvedElectronArrowGestureError as exc:
				self._cancel_line_gesture(clear_status=False)
				self._show_edit_refusal(self._curved_electron_arrow_refusal(exc))
				return
			except Exception:
				self._cancel_line_gesture(clear_status=False)
				raise
			self._line_gesture_intent = dataclasses.replace(
				intent, curved_electron_points=points, presentation_gesture=gesture,
			)
			self._show_curved_electron_point_guidance(2)
			return
		if len(points) == 3:
			self._line_gesture_intent = dataclasses.replace(intent, curved_electron_points=points)
			self._update_curved_electron_arrow_gesture(
				self._line_gesture_intent, event.position().toPoint(),
			)
			current = self._line_gesture_intent
			if current is not None:
				self._complete_curved_electron_arrow_gesture(current)

	#============================================
	def _complete_click_presentation_gesture(self, intent: _LineGestureIntent) -> None:
		"""Complete the current click-driven path or quadratic-arrow contract."""
		if intent.tool in self._draw_path_actions:
			self._complete_presentation_path_gesture(intent)
		elif intent.tool is _NativeLineTool.DRAW_CURVED_ELECTRON_ARROW:
			self._complete_curved_electron_arrow_gesture(intent)

	#============================================
	def _update_line_gesture(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Move only the disposable Qt-local preview line."""
		intent = self._line_gesture_intent
		if intent is not None and intent.tool is _NativeLineTool.ROTATE_ATOMS:
			self._update_rotation_gesture(intent, event)
			return
		if intent is not None and intent.tool is _NativeLineTool.TRANSLATE_ROOTS:
			self._update_translation_gesture(intent, event)
			return
		if intent is not None and intent.tool in (
			_NativeLineTool.DRAW_ARROW,
			_NativeLineTool.DRAW_EQUILIBRIUM_ARROW,
		):
			self._update_presentation_gesture(intent, event.position().toPoint())
			return
		if intent is not None and intent.tool in self._draw_vector_actions:
			self._update_vector_gesture(intent, event.position().toPoint())
			return
		if intent is not None and intent.tool in self._draw_path_actions:
			self._update_presentation_path_gesture(intent, event.position().toPoint(), True)
			return
		if intent is not None and intent.tool is _NativeLineTool.DRAW_CURVED_ELECTRON_ARROW:
			self._update_curved_electron_arrow_gesture(intent, event.position().toPoint())
			return
		if intent is not None and intent.tool is _NativeLineTool.ATTACH_CYCLOHEXANE_RING:
			if intent.start_atom_id is None or intent.attached_cyclohexane_pending is not None:
				return
			try:
				pending = intent.tab.begin_attached_cyclohexane(
					intent.start_atom_id,
					intent.tab.view.mapToScene(event.position().toPoint()),
				)
				vertices = intent.tab.preview_attached_cyclohexane(pending)
				path = PySide6.QtGui.QPainterPath(vertices[0])
				for vertex in vertices[1:]:
					path.lineTo(vertex)
				path.closeSubpath()
				pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor("#49719c"))
				pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
				pen.setWidthF(1.5)
				preview = intent.tab.view.scene().addPath(path, pen)
				preview.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
				preview.setZValue(1_000_000.0)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, preview=preview, attached_cyclohexane_pending=pending,
			)
			return
		if intent is None or intent.preview is None or intent.start_scene is None:
			if (
				intent is not None
				and intent.tool is _NativeLineTool.DRAW_BOND
				and intent.direct_bond_gesture is not None
			):
				self._update_direct_bond_gesture(intent, event.position().toPoint())
			return
		if intent.tool is _NativeLineTool.INSERT_CYCLOHEXANE_RING:
			return
		if intent.tool is _NativeLineTool.DRAW_BOND and intent.direct_bond_gesture is not None:
			self._update_direct_bond_gesture(intent, event.position().toPoint())
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal("The document changed during the gesture; no operation was accepted."))
			return
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(intent.preview):
			self._cancel_line_gesture()
			return
		current = self._line_gesture_preview_target(intent, event.position().toPoint())
		if intent.tool in (
			_NativeLineTool.CREATE_RECTANGULAR_BRACKET,
			_NativeLineTool.CREATE_ROUND_BRACKET,
		):
			assert isinstance(intent.preview, PySide6.QtWidgets.QGraphicsRectItem)
			intent.preview.setRect(_normalized_rect(intent.start_scene, current))
		else:
			self._update_line_preview(intent, current)

	#============================================
	def _complete_line_gesture(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Commit one still-current line-tool gesture and keep its tool available."""
		intent = self._line_gesture_intent
		if intent is not None and intent.tool is _NativeLineTool.ROTATE_ATOMS:
			self._complete_rotation_gesture(intent, event)
			return
		if intent is not None and intent.tool is _NativeLineTool.TRANSLATE_ROOTS:
			self._complete_translation_gesture(intent, event)
			return
		if intent is not None and intent.tool in (
			_NativeLineTool.DRAW_ARROW,
			_NativeLineTool.DRAW_EQUILIBRIUM_ARROW,
		):
			self._complete_presentation_gesture(intent, event.position().toPoint())
			return
		if intent is not None and intent.tool in self._draw_vector_actions:
			self._complete_vector_gesture(intent, event.position().toPoint())
			return
		if intent is not None and intent.tool is _NativeLineTool.DRAW_PLUS:
			self._complete_plus_gesture(intent)
			return
		if intent is not None and intent.tool is _NativeLineTool.INSERT_TEXT:
			self._complete_text_placement_gesture()
			return
		# Draw Bond owns opaque Rust endpoint admission, including intentional
		# blank-canvas starts that have no durable Qt atom identifier.  Complete it
		# before the generic durable-origin guard required by committed tools.
		if intent is not None and intent.tool is _NativeLineTool.DRAW_BOND:
			if intent.direct_bond_gesture is None:
				return
			self._update_direct_bond_gesture(intent, event.position().toPoint())
			current = self._line_gesture_intent
			if (
				current is None
				or current.tool is not _NativeLineTool.DRAW_BOND
				or current.direct_bond_gesture is None
				or current.direct_bond_admission is None
			):
				return
			admission = current.direct_bond_admission
			self._reset_line_gesture_start()
			import ferrum_qt.ferrum.engine as engine
			try:
				commit = current.tab.commit_direct_bond_admission(admission)
			except (engine.DirectBondGestureError, engine.RevisionConflictError) as exc:
				if not self._is_direct_bond_commit_refusal(exc):
					raise
				self._cancel_line_gesture()
				self._refresh_actions()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			result_message = (
				self.tr("Added one Ferrum carbon and normal bond; drag again or press Esc.")
				if commit.created_new_atom
				else self.tr("Added one Ferrum normal bond; drag again or press Esc.")
			)
			self._finish_line_gesture(current, result_message)
			return
		if (
			intent is None
			or (
				intent.tool is not _NativeLineTool.ATTACH_CYCLOHEXANE_RING
				and intent.start_scene is None
			)
			or intent.press_scene is None
			or (
				intent.tool not in (
					_NativeLineTool.CREATE_WAVY,
					_NativeLineTool.CREATE_RECTANGULAR_BRACKET,
					_NativeLineTool.CREATE_ROUND_BRACKET,
					_NativeLineTool.INSERT_CYCLOHEXANE_RING,
					_NativeLineTool.ATTACH_CYCLOHEXANE_RING,
				)
				and intent.start_atom_id is None
			)
		):
			return
		if intent.tool is _NativeLineTool.INSERT_CYCLOHEXANE_RING:
			prepared = intent.regular_ring_prepared
			if prepared is None:
				return
			if not self._line_gesture_is_current(intent):
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal("The document changed before the ring was inserted. Try again."))
				return
			self._reset_line_gesture_start()
			try:
				intent.tab.commit_regular_ring(prepared)
			except Exception as exc:
				self._cancel_line_gesture()
				self._refresh_actions()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			self._finish_line_gesture(intent, self.tr(
				"Inserted one Ferrum cyclohexane ring; click again or press Esc.",
			))
			return
		if intent.tool is _NativeLineTool.ATTACH_CYCLOHEXANE_RING:
			pending = intent.attached_cyclohexane_pending
			if pending is None or intent.start_atom_id is None:
				if pending is not None:
					self._cancel_line_gesture()
					self._show_edit_refusal(self._unavailable_edit_refusal(
						"The attachment anchor is no longer available. Try again.",
					))
					return
				self.statusBar().showMessage(self.tr(
					"Drag away from an eligible atom to choose an attachment direction.",
				), 5000)
				return
			if not self._line_gesture_is_current(intent):
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(
					"The document changed before the ring was attached. Try again.",
				))
				return
			try:
				intent.tab.commit_attached_cyclohexane(pending)
			except Exception as exc:
				self._cancel_line_gesture()
				self._refresh_actions()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			self._reset_line_gesture_start()
			self._finish_line_gesture(intent, self.tr(
				"Attached one Ferrum cyclohexane ring; drag again or press Esc.",
			))
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal("The document changed during the gesture; no operation was accepted."))
			return
		release_point = event.position().toPoint()
		release_scene = self._line_gesture_preview_target(intent, release_point)
		self._reset_line_gesture_start()
		if intent.tool is _NativeLineTool.CREATE_WAVY:
			try:
				intent.tab.create_wavy(
					float(intent.start_scene.x()), float(intent.start_scene.y()),
					float(release_scene.x()), float(release_scene.y()),
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._refresh_actions()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			self._finish_line_gesture(
				intent,
				self.tr("Added one Ferrum Wavy line; drag again or press Esc."),
			)
			return
		if intent.tool in (
			_NativeLineTool.CREATE_RECTANGULAR_BRACKET,
			_NativeLineTool.CREATE_ROUND_BRACKET,
		):
			rectangle = _normalized_rect(intent.start_scene, release_scene)
			try:
				create = (
					intent.tab.create_rectangular_bracket
					if intent.tool is _NativeLineTool.CREATE_RECTANGULAR_BRACKET
					else intent.tab.create_round_bracket
				)
				create(
					float(rectangle.left()), float(rectangle.top()),
					float(rectangle.right()), float(rectangle.bottom()),
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._refresh_actions()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			self._finish_line_gesture(
				intent,
				self.tr(
					"Added one Ferrum bracket pair; drag again or press Esc.",
				),
			)
			return
		end_atom_id = intent.tab.durable_atom_at_viewport_point(release_point)
		start_atom_id = intent.start_atom_id
		assert start_atom_id is not None
		if intent.tool is _NativeLineTool.MOVE_ATOM:
			try:
				intent.tab.move_atom_to(
					start_atom_id, float(release_scene.x()), float(release_scene.y()),
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._refresh_actions()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			result_message = self.tr(
				"Moved one Ferrum atom; drag again or press Esc.",
			)
			self._finish_line_gesture(intent, result_message)
			return
		if end_atom_id == start_atom_id:
			self.statusBar().showMessage(
				self.tr("Release Draw Bond on a different atom or in empty space."), 5000,
			)
			return
		drawing = intent.drawing
		if drawing is None:
			raise RuntimeError("Ferrum Draw Bond gesture has no frozen drawing parameters")
		presentation = drawing.bond_presentation()
		try:
			if end_atom_id is None:
				intent.tab.add_bonded_atom_at(
					start_atom_id, drawing.element, float(release_scene.x()),
					float(release_scene.y()), presentation,
				)
				result_message = self.tr(
					"Added one Ferrum {0} and {1} bond; drag again or press Esc."
				).format(drawing.element, drawing.presentation_name.replace("_", " "))
			else:
				intent.tab.add_bond_between_atoms(start_atom_id, end_atom_id, presentation)
				result_message = self.tr(
					"Added one Ferrum {0} bond; drag again or press Esc."
				).format(drawing.presentation_name.replace("_", " "))
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		self._finish_line_gesture(intent, result_message)

	#============================================
	def _line_gesture_preview_target(
			self, intent: _LineGestureIntent, viewport_point: PySide6.QtCore.QPoint,
			) -> PySide6.QtCore.QPointF:
		"""Return the exact committed target for one mutable gesture preview."""
		raw_scene = intent.tab.view.mapToScene(viewport_point)
		if intent.tool is _NativeLineTool.MOVE_ATOM:
			if intent.start_scene is None or intent.press_scene is None:
				raise RuntimeError("Ferrum Move Atom gesture has no captured start point")
			target = intent.start_scene + raw_scene - intent.press_scene
			return intent.tab.view.snap_authored_scene_point(target)
		if intent.tool is _NativeLineTool.DRAW_BOND:
			end_atom_id = intent.tab.durable_atom_at_viewport_point(viewport_point)
			if end_atom_id is not None:
				return intent.tab.durable_atom_scene_position(end_atom_id)
		return intent.tab.view.snap_authored_scene_point(raw_scene)
