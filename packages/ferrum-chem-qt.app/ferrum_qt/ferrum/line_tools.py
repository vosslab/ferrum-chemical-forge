"""Revision-bound Qt pointer tools for the standalone Ferrum window."""

# Standard Library
import dataclasses
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.bond_preview
import ferrum_qt.ferrum.regular_ring
import ferrum_qt.ferrum.rotation
import ferrum_qt.ferrum.translation
import ferrum_qt.ferrum.top_level_transform
import ferrum_qt.ferrum.line_tool_intent
import ferrum_qt.ferrum.keyboard_authoring
import ferrum_qt.ferrum.transform_gestures
_NativeLineTool = ferrum_qt.ferrum.line_tool_intent._NativeLineTool
_LineGestureIntent = ferrum_qt.ferrum.line_tool_intent._LineGestureIntent

class FerrumNativeLineToolsMixin(
		ferrum_qt.ferrum.keyboard_authoring.FerrumKeyboardAuthoringMixin,
		ferrum_qt.ferrum.transform_gestures.
		FerrumNativeTransformGesturesMixin,
		):
	"""Own the disposable pointer gestures used by the Ferrum document host.

	The host supplies the active document tab, warning/status surfaces, and action
	refresh. This mixin owns pointer capture and local preview retirement only;
	all document mutation stays behind the tab's Rust session methods.
	"""

	def _initialize_line_tools(self) -> None:
		"""Initialize the one mutually exclusive pointer-tool intent."""
		self._line_gesture_intent: _LineGestureIntent | None = None

	#============================================
	def _build_line_tool_actions(self, edit_menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the checkable Ferrum pointer tools to the host's Edit menu."""
		self._draw_bond_action = PySide6.QtGui.QAction(
			self.tr("Draw Bond"), self,
		)
		self._draw_bond_action.setCheckable(True)
		self._draw_bond_action.setToolTip(
			self.tr("Use Next atom and Next bond, then drag from an atom; Esc cancels"),
		)
		self._draw_bond_action.triggered.connect(self._on_toggle_draw_bond)
		edit_menu.addAction(self._draw_bond_action)
		self._insert_cyclohexane_ring_action = PySide6.QtGui.QAction(
			self.tr("Insert Cyclohexane Ring"), self,
		)
		self._insert_cyclohexane_ring_action.setCheckable(True)
		self._insert_cyclohexane_ring_action.setToolTip(self.tr(
			"Click an empty page location to insert a six-carbon ring; Escape cancels.",
		))
		self._insert_cyclohexane_ring_action.setStatusTip(self.tr(
			"Click an empty page location to insert a six-carbon ring; Escape cancels.",
		))
		self._insert_cyclohexane_ring_action.triggered.connect(self._on_toggle_insert_cyclohexane_ring)
		edit_menu.addAction(self._insert_cyclohexane_ring_action)
		self._draw_wavy_action = PySide6.QtGui.QAction(self.tr("Draw Wavy Line"), self)
		self._draw_wavy_action.setCheckable(True)
		self._draw_wavy_action.setToolTip(
			self.tr("Drag between two page points; Esc cancels"),
		)
		self._draw_wavy_action.triggered.connect(self._on_toggle_draw_wavy)
		edit_menu.addAction(self._draw_wavy_action)
		self._draw_bracket_action = PySide6.QtGui.QAction(
			self.tr("Draw Rectangular Bracket"), self,
		)
		self._draw_bracket_action.setCheckable(True)
		self._draw_bracket_action.setToolTip(
			self.tr("Drag a finite nonempty rectangle; Esc cancels"),
		)
		self._draw_bracket_action.triggered.connect(self._on_toggle_draw_bracket)
		edit_menu.addAction(self._draw_bracket_action)
		self._draw_round_bracket_action = PySide6.QtGui.QAction(
			self.tr("Draw Round Bracket"), self,
		)
		self._draw_round_bracket_action.setCheckable(True)
		self._draw_round_bracket_action.setToolTip(
			self.tr("Drag a finite nonempty rectangle; Esc cancels"),
		)
		self._draw_round_bracket_action.triggered.connect(
			self._on_toggle_draw_round_bracket,
		)
		edit_menu.addAction(self._draw_round_bracket_action)
		self._move_atom_action = PySide6.QtGui.QAction(self.tr("Move Atom"), self)
		self._move_atom_action.setCheckable(True)
		self._move_atom_action.setToolTip(
			self.tr("Drag one existing atom to an exact new scene point; Esc cancels"),
		)
		self._move_atom_action.triggered.connect(self._on_toggle_move_atom)
		edit_menu.addAction(self._move_atom_action)
		self._rotate_atoms_action = PySide6.QtGui.QAction(
			self.tr("Rotate Selected Atoms"), self,
		)
		self._rotate_atoms_action.setCheckable(True)
		self._rotate_atoms_action.setToolTip(
			self.tr(
				"Drag around the selected atoms' center; Esc cancels without changing Rust",
			),
		)
		self._rotate_atoms_action.triggered.connect(self._on_toggle_rotate_atoms)
		edit_menu.addAction(self._rotate_atoms_action)
		self._translate_roots_action = PySide6.QtGui.QAction(
			self.tr("Move Complete Roots"), self,
		)
		self._translate_roots_action.setCheckable(True)
		self._translate_roots_action.setToolTip(
			self.tr(
				"Drag selected complete roots; the View snap setting applies; Esc cancels",
			),
		)
		self._translate_roots_action.triggered.connect(self._on_toggle_translate_roots)
		edit_menu.addAction(self._translate_roots_action)
		self._cancel_tool_action = PySide6.QtGui.QAction(self.tr("Cancel Tool"), self)
		self._cancel_tool_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Cancel)
		self._cancel_tool_action.setToolTip(self.tr(
			"Cancel the active editing tool; selection and document stay unchanged",
		))
		self._cancel_tool_action.triggered.connect(self._on_cancel_tool)
		self._cancel_tool_action.setEnabled(False)
		edit_menu.addAction(self._cancel_tool_action)

	def _refresh_line_tool_actions(self, enabled: bool) -> None:
		"""Apply the host's authoritative action policy to both pointer tools."""
		self._draw_bond_action.setEnabled(enabled)
		self._insert_cyclohexane_ring_action.setEnabled(enabled)
		self._draw_wavy_action.setEnabled(enabled)
		self._draw_bracket_action.setEnabled(enabled)
		self._draw_round_bracket_action.setEnabled(enabled)
		self._move_atom_action.setEnabled(enabled)
		tab = self._active_native_tab() if enabled else None
		self._rotate_atoms_action.setEnabled(
			tab is not None and tab.has_rotatable_atom_selection(),
		)
		self._translate_roots_action.setEnabled(
			tab is not None and tab.can_transform_top_level_selection(),
		)
		self._refresh_cancel_tool_action()

	def _refresh_cancel_tool_action(self) -> None:
		"""Enable cancellation exactly while one Ferrum pointer intent exists."""
		self._cancel_tool_action.setEnabled(
			self._atom_insertion_intent is not None
			or self._line_gesture_intent is not None,
		)
		self._refresh_local_cdml_open_action()

	def _on_cancel_tool(self) -> None:
		"""Cancel a pointer tool while preserving Rust state and selection."""
		self._cancel_atom_insertion()
		self._cancel_line_gesture()
		self._synchronize_mode_state()
		self.statusBar().showMessage(
			self.tr("Tool cancelled. Selection and document are unchanged."), 3000,
		)

	def _on_toggle_draw_bond(self, checked: bool) -> None:
		"""Enter or leave one revision-bound atom-to-atom drawing mode."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_BOND)

	def _on_toggle_insert_cyclohexane_ring(self, checked: bool) -> None:
		"""Arm one direct, detached, Rust-owned cyclohexane placement."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.INSERT_CYCLOHEXANE_RING)

	#============================================
	def _on_toggle_draw_wavy(self, checked: bool) -> None:
		"""Enter or leave the revision-bound free-standing Wavy drawing mode."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.CREATE_WAVY)

	#============================================
	def _on_toggle_draw_bracket(self, checked: bool) -> None:
		"""Enter or leave the revision-bound rectangular bracket tool."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.CREATE_RECTANGULAR_BRACKET)

	#============================================
	def _on_toggle_draw_round_bracket(self, checked: bool) -> None:
		"""Enter or leave the revision-bound round bracket tool."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.CREATE_ROUND_BRACKET)

	#============================================
	def _on_toggle_move_atom(self, checked: bool) -> None:
		"""Enter or leave the revision-bound atom movement mode."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.MOVE_ATOM)

	#============================================
	def _on_toggle_rotate_atoms(self, checked: bool) -> None:
		"""Enter or leave one immutable-projection atom rotation gesture."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.ROTATE_ATOMS)

	#============================================
	def _on_toggle_translate_roots(self, checked: bool) -> None:
		"""Enter or leave one immutable-projection complete-root translation."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.TRANSLATE_ROOTS)

	#============================================
	def _activate_line_tool(self, tool: _NativeLineTool) -> None:
		"""Install one exact line tool after cancelling every competing intent."""
		self._cancel_atom_insertion()
		self._cancel_line_gesture(clear_status=False)
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			self._cancel_line_gesture()
			return
		if tool is _NativeLineTool.DRAW_BOND:
			action = self._draw_bond_action
		elif tool is _NativeLineTool.CREATE_WAVY:
			action = self._draw_wavy_action
		elif tool is _NativeLineTool.CREATE_RECTANGULAR_BRACKET:
			action = self._draw_bracket_action
		elif tool is _NativeLineTool.CREATE_ROUND_BRACKET:
			action = self._draw_round_bracket_action
		elif tool is _NativeLineTool.MOVE_ATOM:
			action = self._move_atom_action
		elif tool is _NativeLineTool.ROTATE_ATOMS:
			action = self._rotate_atoms_action
		elif tool is _NativeLineTool.TRANSLATE_ROOTS:
			action = self._translate_roots_action
		else:
			action = self._insert_cyclohexane_ring_action
		action.setChecked(True)
		snapshot = tab.current_snapshot
		viewport = tab.view.viewport()
		self._line_gesture_intent = _LineGestureIntent(
			tab, viewport, snapshot.revision, snapshot.digest, tool,
		)
		self._synchronize_mode_state()
		self._refresh_cancel_tool_action()
		viewport.installEventFilter(self)
		viewport.setFocus()
		tab.view.show_keyboard_cursor()
		if tool is _NativeLineTool.DRAW_BOND:
			drawing = self._drawing_parameters.snapshot()
			message = self._draw_bond_feedback(drawing)
			self._draw_bond_action.setToolTip(message)
		elif tool is _NativeLineTool.CREATE_WAVY:
			message = self.tr("Drag between two page points; Esc cancels Draw Wavy Line.")
		elif tool is _NativeLineTool.CREATE_RECTANGULAR_BRACKET:
			message = self.tr("Drag a rectangle; Esc cancels Draw Rectangular Bracket.")
		elif tool is _NativeLineTool.CREATE_ROUND_BRACKET:
			message = self.tr("Drag a rectangle; Esc cancels Draw Round Bracket.")
		elif tool is _NativeLineTool.MOVE_ATOM:
			message = self.tr("Drag one atom to its new scene point; Esc cancels Move Atom.")
		elif tool is _NativeLineTool.ROTATE_ATOMS:
			message = self.tr(
				"Drag around the selected atoms' center; Esc cancels Rotate Atoms.",
			)
		elif tool is _NativeLineTool.TRANSLATE_ROOTS:
			message = self.tr(
				"Drag complete selected roots; Esc cancels Move Complete Roots.",
			)
		else:
			message = self.tr(
				"Insert Cyclohexane Ring: click an empty page location; Escape cancels.",
			)
		self.statusBar().showMessage(message)

	#============================================
	def _draw_bond_feedback(
			self,
			drawing: ferrum_qt.ferrum.drawing_parameters.
			FerrumNativeDrawingParametersSnapshot,
			) -> str:
		"""Name the current normal or directed gesture contract in human wording."""
		if drawing.presentation_name == "solid_wedge":
			return self.tr(
				"Draw Bond: Solid wedge (Single); drag from the narrow tip to the wide end. "
				"Empty-space endpoints use {0}; Esc cancels."
			).format(drawing.element)
		if drawing.presentation_name == "hashed_wedge":
			return self.tr(
				"Draw Bond: Hashed wedge (Single); drag from the narrow tip to the wide end. "
				"Empty-space endpoints use {0}; Esc cancels."
			).format(drawing.element)
		return self.tr(
			"Draw Bond: {0} with a {1} bond; drag or use Arrow keys and Enter. "
			"Shift+Arrow is fine movement; Esc cancels."
		).format(drawing.element, drawing.order_name)

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Capture Ferrum atom-insertion or line-tool pointer intent."""
		line_intent = self._line_gesture_intent
		if line_intent is not None and watched is line_intent.viewport:
			return self._line_gesture_event(event)
		intent = self._atom_insertion_intent
		if intent is None or watched is not intent.viewport:
			return super().eventFilter(watched, event)
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
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
		"""Consume one drag gesture without creating any Qt document model."""
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			if self._keyboard_canvas_key_event(event):
				return True
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonPress:
			if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
				return False
			try:
				self._start_line_gesture(event)
			except (TypeError, ValueError) as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
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
			try:
				self._complete_line_gesture(event)
			except (TypeError, ValueError) as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return True
		return False

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
		if intent.tool is _NativeLineTool.DRAW_BOND:
			drawing = self._drawing_parameters.snapshot()
			intent = dataclasses.replace(intent, drawing=drawing)
			self._line_gesture_intent = intent
			message = self._draw_bond_feedback(drawing)
			self._draw_bond_action.setToolTip(message)
			self.statusBar().showMessage(message)
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
	def _update_line_gesture(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Move only the disposable Qt-local preview line."""
		intent = self._line_gesture_intent
		if intent is not None and intent.tool is _NativeLineTool.ROTATE_ATOMS:
			self._update_rotation_gesture(intent, event)
			return
		if intent is not None and intent.tool is _NativeLineTool.TRANSLATE_ROOTS:
			self._update_translation_gesture(intent, event)
			return
		if intent is None or intent.preview is None or intent.start_scene is None:
			return
		if intent.tool is _NativeLineTool.INSERT_CYCLOHEXANE_RING:
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
		if (
			intent is None
			or intent.start_scene is None
			or intent.press_scene is None
			or (
				intent.tool not in (
					_NativeLineTool.CREATE_WAVY,
					_NativeLineTool.CREATE_RECTANGULAR_BRACKET,
					_NativeLineTool.CREATE_ROUND_BRACKET,
					_NativeLineTool.INSERT_CYCLOHEXANE_RING,
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

	#============================================
	def _update_line_preview(self, intent: _LineGestureIntent,
			current: PySide6.QtCore.QPointF) -> None:
		"""Refresh directed previews from Rust V2 operations or update a normal line."""
		assert intent.preview is not None
		assert intent.start_scene is not None
		drawing = intent.drawing
		if (
			intent.tool is _NativeLineTool.DRAW_BOND
			and drawing is not None
			and drawing.presentation_name != "normal"
		):
			self._retire_line_preview(intent.preview)
			preview = ferrum_qt.ferrum.bond_preview.create_directed_preview(
				intent.tab, intent.start_scene, current, drawing.bond_presentation(),
			)
			self._line_gesture_intent = dataclasses.replace(intent, preview=preview)
			return
		assert isinstance(intent.preview, PySide6.QtWidgets.QGraphicsLineItem)
		intent.preview.setLine(PySide6.QtCore.QLineF(intent.start_scene, current))

	#============================================
	def _start_translation_gesture(self, intent: _LineGestureIntent,
			press_scene: PySide6.QtCore.QPointF) -> None:
		"""Capture complete roots and create one disposable bounds preview."""
		try:
			selection = intent.tab.selected_top_level_translation()
			if (
					selection.source_revision != intent.revision
					or selection.source_digest != intent.digest
				):
				raise (
					ferrum_qt.ferrum.top_level_transform.
					FerrumNativeTopLevelTranslationStaleError(
						"document changed before complete-root translation began",
					)
				)
			preview = ferrum_qt.ferrum.translation.create_translation_preview(
				intent.tab, selection,
			)
		except ferrum_qt.ferrum.top_level_transform.FerrumNativeTopLevelTranslationStaleError:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal("The selected roots or document changed. Select complete roots and drag again."))
			return
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		self._line_gesture_intent = dataclasses.replace(
			intent,
			press_scene=press_scene,
			translation_selection=selection,
			translation_preview=preview,
			translation_snap_enabled=intent.tab.view.hex_grid_snap_enabled,
		)

	#============================================
	def _update_translation_gesture(self, intent: _LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Move only the local complete-root bounds preview."""
		if (
				intent.translation_preview is None
				or intent.translation_selection is None
				or intent.translation_snap_enabled is None
				or intent.press_scene is None
			):
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal("The selected roots or document changed. Select complete roots and drag again."))
			return
		current = intent.tab.view.mapToScene(event.position().toPoint())
		raw_dx = float(current.x() - intent.press_scene.x())
		raw_dy = float(current.y() - intent.press_scene.y())
		if not math.isfinite(raw_dx) or not math.isfinite(raw_dy):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal("Translation pointer position must be finite."))
			return
		selection = intent.translation_selection
		try:
			resolved_anchor = intent.tab.view.resolve_authored_scene_point(
				PySide6.QtCore.QPointF(
					selection.anchor_x + raw_dx, selection.anchor_y + raw_dy,
				),
				intent.translation_snap_enabled,
			)
		except (RuntimeError, TypeError, ValueError) as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		dx = float(resolved_anchor.x() - selection.anchor_x)
		dy = float(resolved_anchor.y() - selection.anchor_y)
		ferrum_qt.ferrum.translation.update_translation_preview(
			intent.translation_preview, dx, dy,
		)
		self._line_gesture_intent = dataclasses.replace(
			intent, translation_delta=(dx, dy),
		)

	#============================================
	def _complete_translation_gesture(self, intent: _LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Retire local bounds, then submit one still-current Rust translation."""
		if intent.translation_selection is None or intent.translation_preview is None:
			return
		self._update_translation_gesture(intent, event)
		current = self._line_gesture_intent
		if current is None or current.translation_selection is None:
			return
		selection = current.translation_selection
		dx, dy = current.translation_delta
		self._reset_line_gesture_start()
		if dx == 0.0 and dy == 0.0:
			self.statusBar().showMessage(
				self.tr("Move Complete Roots remains active; no move was requested."),
				5000,
			)
			return
		try:
			intent.tab.translate_top_level_roots_at_revision(
				intent.revision,
				selection.source_digest,
				selection.targets,
				selection.durable_selection,
				dx,
				dy,
			)
		except ferrum_qt.ferrum.top_level_transform.FerrumNativeTopLevelTranslationStaleError:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._unavailable_edit_refusal("The selected roots or document changed. Select complete roots and drag again."))
			return
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		self._finish_line_gesture(
			intent,
			self.tr("Moved complete Ferrum roots; drag again or press Esc."),
		)

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
			last_angle=None,
			accumulated_angle=0.0,
			regular_ring_prepared=None,
		)

	#============================================
	def _cancel_line_gesture(self, clear_status: bool = True) -> None:
		"""Release pointer capture and terminally retire its preview."""
		intent = self._line_gesture_intent
		self._line_gesture_intent = None
		self._draw_bond_action.setChecked(False)
		self._insert_cyclohexane_ring_action.setChecked(False)
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
		if clear_status:
			self.statusBar().clearMessage()
		self._refresh_cancel_tool_action()
		self._synchronize_mode_state()

	#============================================
	@staticmethod
	def _line_tool_stale_title(tool: _NativeLineTool) -> str:
		"""Return one actionable title for a gesture invalidated by a document edit."""
		if tool is _NativeLineTool.DRAW_BOND:
			return "Draw Bond Stale"
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
