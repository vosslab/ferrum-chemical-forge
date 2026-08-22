"""Ferrum render-interaction selection and line-tool lifecycle helpers."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.ferrum.direct_root_preview
import ferrum_qt.ferrum.line_tool_intent

_NativeLineTool = ferrum_qt.ferrum.line_tool_intent._NativeLineTool
_LineGestureIntent = ferrum_qt.ferrum.line_tool_intent._LineGestureIntent


class FerrumNativeLineToolInteractionMixin:
	"""Own root interaction selection and shared line-tool lifecycle helpers."""
	def _start_translation_gesture(self, intent: _LineGestureIntent,
			press_scene: PySide6.QtCore.QPointF) -> None:
		"""Select or drag complete roots only through Rust-issued interaction facts."""
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = intent.tab.observe_direct_root_interaction()
			seed = self._render_interaction_selection
			if seed is None:
				seed = self._seed_render_interaction_selection(intent.tab, observation)
			else:
				seed = self._revalidate_render_interaction_selection(
					intent.tab, observation, seed,
				)
			if seed is not None and intent.tab.direct_root_selection_contains_point(
					seed, float(press_scene.x()), float(press_scene.y()),
				):
				selection = seed
			else:
				modifier = (
					engine.RenderInteractionModifierV1.toggle
					if PySide6.QtWidgets.QApplication.keyboardModifiers()
					& PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier
					else engine.RenderInteractionModifierV1.replace
				)
				query = engine.RenderInteractionQueryV1.point(
					float(press_scene.x()), float(press_scene.y()), modifier,
				)
				selection = intent.tab.select_direct_roots(observation, seed, query)
			if not selection.roots:
				self._replace_render_interaction_selection(None, intent.tab)
				marquee = self._new_bracket_preview(intent.tab, press_scene)
				self._line_gesture_intent = dataclasses.replace(
					intent, press_scene=press_scene, direct_root_observation=observation,
					direct_root_selection=None, direct_root_marquee=marquee,
				)
				return
			self._replace_render_interaction_selection(selection, intent.tab)
			snap = self._render_interaction_snap(
				intent.tab, engine.RenderInteractionAxisV1.free,
			)
			gesture = intent.tab.begin_direct_root_translation(
				selection, float(press_scene.x()), float(press_scene.y()), snap,
			)
			preview = intent.tab.preview_direct_root_translation(
				gesture, float(press_scene.x()), float(press_scene.y()),
			)
			preview_item = ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
				intent.tab, preview.bounds,
			)
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._render_interaction_refusal(exc))
			return
		self._line_gesture_intent = dataclasses.replace(
			intent,
			press_scene=press_scene,
			direct_root_observation=observation,
			direct_root_selection=selection,
			direct_root_gesture=gesture,
			direct_root_preview=preview,
			direct_root_preview_item=preview_item,
		)

	#============================================
	def _update_translation_gesture(self, intent: _LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Ask Rust for each translation preview; Qt only replaces issued bounds."""
		if intent.press_scene is None:
			return
		current = intent.tab.view.mapToScene(event.position().toPoint())
		if intent.direct_root_gesture is None:
			if intent.direct_root_marquee is not None:
				intent.direct_root_marquee.setRect(_normalized_rect(intent.press_scene, current))
			return
		try:
			preview = intent.tab.preview_direct_root_translation(
				intent.direct_root_gesture, float(current.x()), float(current.y()),
			)
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._render_interaction_refusal(exc))
			return
		self._retire_line_preview(intent.direct_root_preview_item)
		preview_item = ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
			intent.tab, preview.bounds,
		)
		self._line_gesture_intent = dataclasses.replace(
			intent, direct_root_preview=preview, direct_root_preview_item=preview_item,
		)

	#============================================
	def _complete_translation_gesture(self, intent: _LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Commit an exact Rust preview, or resolve one Rust marquee selection."""
		if intent.direct_root_gesture is None:
			if intent.direct_root_marquee is None or intent.press_scene is None:
				return
			current = intent.tab.view.mapToScene(event.position().toPoint())
			try:
				import ferrum_qt.ferrum.engine as engine
				modifier = (
					engine.RenderInteractionModifierV1.toggle
					if event.modifiers() & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier
					else engine.RenderInteractionModifierV1.replace
				)
				query = engine.RenderInteractionQueryV1.marquee(
					float(intent.press_scene.x()), float(intent.press_scene.y()),
					float(current.x()), float(current.y()),
					modifier,
				)
				selection = intent.tab.select_direct_roots(
					intent.direct_root_observation, self._render_interaction_selection, query,
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._render_interaction_refusal(exc))
				return
			self._retire_line_preview(intent.direct_root_marquee)
			self._replace_render_interaction_selection(selection, intent.tab)
			self._line_gesture_intent = dataclasses.replace(
				intent, direct_root_selection=selection, direct_root_marquee=None,
			)
			self.statusBar().showMessage(self.tr(
				"Selected {0} complete Ferrum roots. Drag a selected root to move them."
			).format(len(selection.roots)), 5000)
			return
		if intent.direct_root_preview is None:
			return
		self._update_translation_gesture(intent, event)
		current = self._line_gesture_intent
		if current is None or current.direct_root_gesture is None or current.direct_root_preview is None:
			return
		self._reset_line_gesture_start()
		try:
			commit = intent.tab.commit_direct_root_translation(
				current.direct_root_gesture, current.direct_root_preview,
			)
		except Exception as exc:
			if self._recover_accepted_translation_presentation_error(intent, exc):
				return
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._render_interaction_refusal(exc))
			return
		if commit.changed:
			# A committed handle is fenced to the preceding Rust snapshot.  Never
			# reuse it against the new revision; a later click obtains fresh proof.
			self._replace_render_interaction_selection(None, intent.tab)
		if not commit.changed:
			self.statusBar().showMessage(
				self.tr("Move Complete Roots remains active; no move was requested."), 5000,
			)
			return
		self._finish_line_gesture(
			intent,
			self.tr("Moved complete Ferrum roots; drag again or press Esc."),
		)

	#============================================
	def _replace_render_interaction_selection(self, selection: object | None,
			tab: object) -> None:
		"""Retain only the opaque Rust selection and its issued visual bounds."""
		self._retire_line_preview(self._render_interaction_selection_item)
		self._render_interaction_selection = selection
		self._render_interaction_selection_item = (
			None if selection is None else
			ferrum_qt.ferrum.direct_root_preview.create_direct_root_selection_preview(
				tab, selection,
			)
		)

	#============================================
	def _seed_render_interaction_selection(self, tab: object, observation: object) -> object | None:
		"""Authenticate an existing durable whole-root selection through Rust names."""
		try:
			selectors, _restore = tab.selected_top_level_transform_targets()
		except Exception:
			return None
		if not selectors:
			return None
		import ferrum_qt.ferrum.engine as engine
		selection = None
		for selector in selectors:
			query = engine.RenderInteractionQueryV1.root(
				selector.root_id,
				engine.RenderInteractionModifierV1.toggle if selection is not None
				else engine.RenderInteractionModifierV1.replace,
			)
			selection = tab.select_direct_roots(observation, selection, query)
		return selection

	#============================================
	def _revalidate_render_interaction_selection(
			self, tab: object, observation: object, previous: object,
			) -> object:
		"""Refresh retained durable roots through named Rust queries before a gesture."""
		import ferrum_qt.ferrum.engine as engine
		selection = None
		for root in previous.roots:
			query = engine.RenderInteractionQueryV1.root(
				root.identifier,
				engine.RenderInteractionModifierV1.toggle if selection is not None
				else engine.RenderInteractionModifierV1.replace,
			)
			selection = tab.select_direct_roots(observation, selection, query)
		return selection

	#============================================
	def _render_interaction_refusal(self, error: Exception) -> object:
		"""Present closed Rust interaction recovery without interpreting CDML."""
		import ferrum_qt.ferrum.engine as engine
		category = getattr(error, "category", None)
		if category == engine.RenderInteractionCategoryV1.unrenderable_depiction:
			message = "Selection and drawing are unchanged. This root cannot be drawn; change its presentation and try again."
		elif category == engine.RenderInteractionCategoryV1.ambiguous_root_identifier:
			message = "Selection and drawing are unchanged. This root identifier is ambiguous; repair the document and try again."
		elif category == engine.RenderInteractionCategoryV1.display_only:
			message = "Selection and drawing are unchanged. This visible root is display-only; add a durable supported presentation before moving it."
		elif category in (
				engine.RenderInteractionCategoryV1.stale_revision,
				engine.RenderInteractionCategoryV1.stale_digest,
				engine.RenderInteractionCategoryV1.selection_changed,
				):
			message = "Selection and drawing are unchanged. Refresh the Rust view and repeat the gesture."
		else:
			message = "Selection and drawing are unchanged. Select a renderable complete molecule and try again."
		return self._unavailable_edit_refusal(message)

	#============================================
	def _nudge_render_interaction_selection(
			self, dx: float, dy: float) -> bool:
		"""Commit one keyboard movement through the same opaque Rust gesture API."""
		intent = self._line_gesture_intent
		selection = self._render_interaction_selection
		if (
			intent is None or intent.tool is not _NativeLineTool.TRANSLATE_ROOTS
			or selection is None
		):
			return False
		try:
			import ferrum_qt.ferrum.engine as engine
			press = intent.tab.view.show_keyboard_cursor()
			axis = (
				engine.RenderInteractionAxisV1.horizontal if dy == 0.0
				else engine.RenderInteractionAxisV1.vertical
			)
			gesture = intent.tab.begin_direct_root_translation(
				selection, float(press.x()), float(press.y()),
				self._render_interaction_snap(intent.tab, axis),
			)
			preview = intent.tab.preview_direct_root_translation(
				gesture, float(press.x() + dx), float(press.y() + dy),
			)
			commit = intent.tab.commit_direct_root_translation(gesture, preview)
		except Exception as exc:
			if self._recover_accepted_translation_presentation_error(intent, exc):
				return True
			self._replace_render_interaction_selection(None, intent.tab)
			self._show_edit_refusal(self._render_interaction_refusal(exc))
			return True
		if commit.changed:
			self._replace_render_interaction_selection(None, intent.tab)
			self._finish_line_gesture(intent, self.tr("Moved selected Ferrum root; select it again to continue."))
		return True

	#============================================
	def _recover_accepted_translation_presentation_error(
			self, intent: _LineGestureIntent, error: Exception,
			) -> bool:
		"""Refresh one Rust-accepted root translation whose scene install failed."""
		from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
		if not isinstance(error, FerrumNativeDocumentTabMutationPresentationError):
			return False
		if error.accepted_receipt is None:
			return False
		self._replace_render_interaction_selection(None, intent.tab)
		if intent.tab.refresh_authoritative():
			self._finish_line_gesture(
				intent,
				self.tr("Moved complete Ferrum roots and refreshed the authoritative canvas."),
			)
			return True
		self._cancel_line_gesture()
		self._refresh_actions()
		self.statusBar().showMessage(self.tr(
			"Move Complete Roots was accepted, but the canvas could not refresh."
		), 5000)
		return True

	#============================================
	@staticmethod
	def _render_interaction_snap(tab: object, axis: object) -> object:
		"""Map only Ferrum's existing view-grid preference to Rust snap policy."""
		import ferrum_qt.ferrum.engine as engine
		policy = (
			engine.RenderInteractionGridSnapPolicyV1.view_hex_grid
			if tab.view.hex_grid_snap_enabled
			else engine.RenderInteractionGridSnapPolicyV1.free
		)
		return engine.RenderInteractionSnapV1.with_grid_policy(axis, policy)

	#============================================
	def _finish_line_gesture(self, intent: _LineGestureIntent, message: str) -> None:
		"""Advance a still-active tool to the exact accepted Rust provenance."""
		snapshot = intent.tab.current_snapshot
		current = self._line_gesture_intent
		if current is not None:
			self._line_gesture_intent = dataclasses.replace(
				current, revision=snapshot.revision, digest=snapshot.digest,
			)
		self.statusBar().showMessage(message, 5000)
		self._synchronize_mode_state()
		self._refresh_actions()

	#============================================
	def _new_line_preview(self, tab: object,
			start: PySide6.QtCore.QPointF) -> PySide6.QtWidgets.QGraphicsLineItem:
		"""Create one scene-owned, non-authoritative interaction preview."""
		scene = tab.view.scene()
		if scene is None:
			raise RuntimeError("Ferrum document has no current scene")
		color = PySide6.QtWidgets.QApplication.palette().color(
			PySide6.QtGui.QPalette.ColorRole.Highlight,
		)
		pen = PySide6.QtGui.QPen(color)
		pen.setWidthF(1.5)
		pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
		pen.setCosmetic(False)
		preview = scene.addLine(PySide6.QtCore.QLineF(start, start), pen)
		preview.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		preview.setZValue(1_000_000.0)
		return preview

	#============================================
	def _update_line_preview(self, intent: _LineGestureIntent,
			current: PySide6.QtCore.QPointF) -> None:
		"""Move one disposable line preview without changing native document state."""
		if not isinstance(intent.preview, PySide6.QtWidgets.QGraphicsLineItem):
			raise RuntimeError("Ferrum line gesture requires a QGraphicsLineItem preview")
		if intent.start_scene is None:
			raise RuntimeError("Ferrum line gesture requires a start scene point")
		intent.preview.setLine(PySide6.QtCore.QLineF(intent.start_scene, current))

	#============================================
	def _new_bracket_preview(self, tab: object,
			start: PySide6.QtCore.QPointF) -> PySide6.QtWidgets.QGraphicsRectItem:
		"""Create one scene-owned, non-authoritative bracket-bounds preview."""
		scene = tab.view.scene()
		if scene is None:
			raise RuntimeError("Ferrum document has no current scene")
		color = PySide6.QtWidgets.QApplication.palette().color(
			PySide6.QtGui.QPalette.ColorRole.Highlight,
		)
		pen = PySide6.QtGui.QPen(color)
		pen.setWidthF(1.5)
		pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
		pen.setCosmetic(False)
		preview = scene.addRect(PySide6.QtCore.QRectF(start, start), pen)
		preview.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		preview.setZValue(1_000_000.0)
		return preview

	#============================================
	def _reset_line_gesture_start(self) -> None:
		"""Retire one preview while keeping the checked pointer tool active."""
		intent = self._line_gesture_intent
		if intent is None:
			return
		self._retire_line_preview(intent.preview)
		self._retire_line_preview(
			None if intent.rotation_preview is None else intent.rotation_preview.root,
		)
		self._retire_line_preview(
			None
			if intent.translation_preview is None
			else intent.translation_preview.root,
		)
		self._retire_line_preview(intent.direct_root_preview_item)
		self._retire_line_preview(intent.direct_root_marquee)
		self._line_gesture_intent = dataclasses.replace(
			intent,
			drawing=None,
			start_atom_id=None,
			start_scene=None,
			press_scene=None,
			preview=None,
			rotation_selection=None,
			rotation_preview=None,
			translation_selection=None,
			translation_preview=None,
			translation_snap_enabled=None,
			translation_delta=(0.0, 0.0),
			direct_root_observation=None,
			direct_root_gesture=None,
			direct_root_preview=None,
			direct_root_preview_item=None,
			direct_root_marquee=None,
			last_angle=None,
			accumulated_angle=0.0,
			regular_ring_prepared=None,
			attached_cyclohexane_pending=None,
			direct_bond_gesture=None,
			direct_bond_admission=None,
			presentation_gesture=None,
			presentation_preview=None,
			curved_electron_points=(),
			vector_gesture=None,
			vector_preview=None,
			path_gesture=None,
			path_points=(),
			path_preview=None,
		)

	#============================================
	def _cancel_line_gesture(self, clear_status: bool = True) -> bool:
		"""Retire a gesture, retaining a failed C6 cancellation for safe retry."""
		intent = self._line_gesture_intent
		if intent is not None and intent.attached_cyclohexane_pending is not None:
			try:
				intent.tab.cancel_attached_cyclohexane(intent.attached_cyclohexane_pending)
			except Exception:
				self._retire_line_preview(intent.preview)
				self._line_gesture_intent = dataclasses.replace(
					intent,
					start_atom_id=None,
					start_scene=None,
					press_scene=None,
					preview=None,
					attached_cyclohexane_cancel_blocked=True,
				)
				if clear_status:
					self.statusBar().showMessage(self.tr(
						"Ferrum could not retire the pending cyclohexane attachment; "
						"the gesture is blocked until cancellation succeeds.",
					), 5000)
				self._refresh_cancel_tool_action()
				self._synchronize_mode_state()
				return False
		self._line_gesture_intent = None
		self._draw_bond_action.setChecked(False)
		self._draw_arrow_action.setChecked(False)
		self._draw_equilibrium_arrow_action.setChecked(False)
		self._draw_curved_electron_arrow_action.setChecked(False)
		self._draw_plus_action.setChecked(False)
		for action in self._draw_vector_actions.values():
			action.setChecked(False)
		for action in self._draw_path_actions.values():
			action.setChecked(False)
		self._insert_text_action.setChecked(False)
		self._insert_cyclohexane_ring_action.setChecked(False)
		self._attach_cyclohexane_ring_action.setChecked(False)
		self._draw_wavy_action.setChecked(False)
		self._draw_bracket_action.setChecked(False)
		self._draw_round_bracket_action.setChecked(False)
		self._move_atom_action.setChecked(False)
		self._rotate_atoms_action.setChecked(False)
		self._translate_roots_action.setChecked(False)
		if intent is not None:
			intent.viewport.removeEventFilter(self)
			intent.tab.view.hide_keyboard_cursor()
			self._retire_line_preview(intent.preview)
			self._retire_line_preview(
				None if intent.rotation_preview is None else intent.rotation_preview.root,
			)
			self._retire_line_preview(
				None
				if intent.translation_preview is None
				else intent.translation_preview.root,
			)
			self._retire_line_preview(intent.direct_root_preview_item)
			self._retire_line_preview(intent.direct_root_marquee)
		if clear_status:
			self.statusBar().clearMessage()
		self._refresh_cancel_tool_action()
		self._synchronize_mode_state()
		return True

	#============================================
	@staticmethod
	def _line_tool_stale_title(tool: _NativeLineTool) -> str:
		"""Return one actionable title for a gesture invalidated by a document edit."""
		if tool is _NativeLineTool.DRAW_BOND:
			return "Draw Bond Stale"
		if tool is _NativeLineTool.DRAW_ARROW:
			return "Draw Arrow Stale"
		if tool is _NativeLineTool.DRAW_PLUS:
			return "Draw Plus Stale"
		if tool is _NativeLineTool.CREATE_WAVY:
			return "Draw Wavy Stale"
		if tool is _NativeLineTool.CREATE_RECTANGULAR_BRACKET:
			return "Draw Bracket Stale"
		if tool is _NativeLineTool.CREATE_ROUND_BRACKET:
			return "Draw Round Bracket Stale"
		if tool is _NativeLineTool.ROTATE_ATOMS:
			return "Rotate Atoms Stale"
		if tool is _NativeLineTool.TRANSLATE_ROOTS:
			return "Move Complete Roots Stale"
		if tool is _NativeLineTool.INSERT_CYCLOHEXANE_RING:
			return "Cyclohexane Ring Stale"
		if tool is _NativeLineTool.ATTACH_CYCLOHEXANE_RING:
			return "Attach Cyclohexane Ring Stale"
		return "Move Atom Stale"

	#============================================
	@staticmethod
	def _line_tool_error_title(tool: _NativeLineTool) -> str:
		"""Return the exact bracket action title for a rejected mutation."""
		if tool is _NativeLineTool.CREATE_ROUND_BRACKET:
			return "Draw Round Bracket Error"
		return "Draw Bracket Error"

	#============================================
	def _retire_line_preview(self,
			preview: PySide6.QtWidgets.QGraphicsItem | None) -> None:
		"""Retire a preview through the shared explicit graphics owner boundary."""
		scene = ferrum_qt.canvas.graphics_retirement.native_scene_for_item(preview)
		if scene is None:
			return
		coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
		coordinator.retire_scene_projection_items(scene, [preview])

	#============================================
	def _line_gesture_is_current(self, intent: _LineGestureIntent) -> bool:
		"""Require exact active-tab and Rust snapshot provenance for the gesture."""
		snapshot = intent.tab.current_snapshot
		return (
			self._active_native_tab() is intent.tab
			and not intent.attached_cyclohexane_cancel_blocked
			and not intent.tab.requires_refresh
			and snapshot.revision == intent.revision
			and snapshot.digest == intent.digest
		)




#============================================
def _normalized_rect(first: PySide6.QtCore.QPointF,
		second: PySide6.QtCore.QPointF) -> PySide6.QtCore.QRectF:
	"""Return exact normalized finite scene bounds for one local preview."""
	return PySide6.QtCore.QRectF(
		min(first.x(), second.x()), min(first.y(), second.y()),
		abs(second.x() - first.x()), abs(second.y() - first.y()),
	)
