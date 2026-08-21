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
import ferrum_qt.ferrum.direct_bond_preview
import ferrum_qt.ferrum.presentation_creation_preview
import ferrum_qt.ferrum.presentation_vector_preview
import ferrum_qt.ferrum.text_placement
import ferrum_qt.ferrum.text_placement_preview
import ferrum_qt.ferrum.direct_root_preview
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
		self._render_interaction_selection: object | None = None
		self._render_interaction_selection_item: PySide6.QtWidgets.QGraphicsItemGroup | None = None

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
		self._connect_interaction_action_v1(self._draw_bond_action, self._on_toggle_draw_bond)
		edit_menu.addAction(self._draw_bond_action)
		self._draw_arrow_action = PySide6.QtGui.QAction(self.tr("Draw Arrow"), self)
		self._draw_arrow_action.setCheckable(True)
		self._draw_arrow_action.setToolTip(self.tr(
			"Drag a straight normal reaction arrow; Esc cancels without changing the document",
		))
		self._draw_arrow_action.setStatusTip(self.tr(
			"Draw a straight normal reaction arrow. Escape cancels.",
		))
		self._connect_interaction_action_v1(self._draw_arrow_action, self._on_toggle_draw_arrow)
		edit_menu.addAction(self._draw_arrow_action)
		self._draw_plus_action = PySide6.QtGui.QAction(self.tr("Draw Plus"), self)
		self._draw_plus_action.setCheckable(True)
		self._draw_plus_action.setToolTip(self.tr("Click to place one Plus; Escape cancels without changing the document"))
		self._connect_interaction_action_v1(self._draw_plus_action, self._on_toggle_draw_plus)
		edit_menu.addAction(self._draw_plus_action)
		self._draw_vector_actions = {}
		for tool, label in (
			(_NativeLineTool.DRAW_LINE, "Draw Line"),
			(_NativeLineTool.DRAW_RECTANGLE, "Draw Rectangle"),
			(_NativeLineTool.DRAW_SQUARE, "Draw Square"),
			(_NativeLineTool.DRAW_OVAL, "Draw Oval"),
			(_NativeLineTool.DRAW_CIRCLE, "Draw Circle"),
		):
			action = PySide6.QtGui.QAction(self.tr(label), self)
			action.setCheckable(True)
			action.setToolTip(self.tr("Drag one Rust-owned {0}; Esc cancels without changing the document").format(label[5:].lower()))
			self._connect_interaction_action_v1(
				action, lambda checked, tool=tool: self._on_toggle_draw_vector(tool, checked),
			)
			edit_menu.addAction(action)
			self._draw_vector_actions[tool] = action
		self._insert_text_action = PySide6.QtGui.QAction(self.tr("Insert Text"), self)
		self._insert_text_action.setCheckable(True)
		self._insert_text_action.setToolTip(self.tr(
			"Insert Text: click a page location, enter text, then Save. Escape cancels.",
		))
		self._insert_text_action.setStatusTip(self.tr(
			"Insert Text: click a page location, enter text, then Save. Escape cancels.",
		))
		self._connect_interaction_action_v1(self._insert_text_action, self._on_toggle_insert_text)
		edit_menu.addAction(self._insert_text_action)
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
		self._connect_interaction_action_v1(
			self._insert_cyclohexane_ring_action, self._on_toggle_insert_cyclohexane_ring,
		)
		edit_menu.addAction(self._insert_cyclohexane_ring_action)
		self._draw_wavy_action = PySide6.QtGui.QAction(self.tr("Draw Wavy Line"), self)
		self._draw_wavy_action.setCheckable(True)
		self._draw_wavy_action.setToolTip(
			self.tr("Drag between two page points; Esc cancels"),
		)
		self._connect_interaction_action_v1(self._draw_wavy_action, self._on_toggle_draw_wavy)
		edit_menu.addAction(self._draw_wavy_action)
		self._draw_bracket_action = PySide6.QtGui.QAction(
			self.tr("Draw Rectangular Bracket"), self,
		)
		self._draw_bracket_action.setCheckable(True)
		self._draw_bracket_action.setToolTip(
			self.tr("Drag a finite nonempty rectangle; Esc cancels"),
		)
		self._connect_interaction_action_v1(self._draw_bracket_action, self._on_toggle_draw_bracket)
		edit_menu.addAction(self._draw_bracket_action)
		self._draw_round_bracket_action = PySide6.QtGui.QAction(
			self.tr("Draw Round Bracket"), self,
		)
		self._draw_round_bracket_action.setCheckable(True)
		self._draw_round_bracket_action.setToolTip(
			self.tr("Drag a finite nonempty rectangle; Esc cancels"),
		)
		self._connect_interaction_action_v1(
			self._draw_round_bracket_action, self._on_toggle_draw_round_bracket,
		)
		edit_menu.addAction(self._draw_round_bracket_action)
		self._move_atom_action = PySide6.QtGui.QAction(self.tr("Move Atom"), self)
		self._move_atom_action.setCheckable(True)
		self._move_atom_action.setToolTip(
			self.tr("Drag one existing atom to an exact new scene point; Esc cancels"),
		)
		self._connect_interaction_action_v1(self._move_atom_action, self._on_toggle_move_atom)
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
		self._connect_interaction_action_v1(self._rotate_atoms_action, self._on_toggle_rotate_atoms)
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
		self._connect_interaction_action_v1(
			self._translate_roots_action, self._on_toggle_translate_roots,
		)
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
		self._draw_arrow_action.setEnabled(enabled)
		self._draw_plus_action.setEnabled(enabled)
		for action in self._draw_vector_actions.values():
			action.setEnabled(enabled)
		self._insert_text_action.setEnabled(enabled)
		self._insert_cyclohexane_ring_action.setEnabled(enabled)
		self._draw_wavy_action.setEnabled(enabled)
		self._draw_bracket_action.setEnabled(enabled)
		self._draw_round_bracket_action.setEnabled(enabled)
		self._move_atom_action.setEnabled(enabled)
		tab = self._active_native_tab() if enabled else None
		self._rotate_atoms_action.setEnabled(
			tab is not None and tab.has_rotatable_atom_selection(),
		)
		self._translate_roots_action.setEnabled(tab is not None)
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

	#============================================
	def _on_toggle_draw_arrow(self, checked: bool) -> None:
		"""Enter or leave Rust-owned straight normal Arrow creation."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_ARROW)

	#============================================
	def _on_toggle_draw_plus(self, checked: bool) -> None:
		"""Enter or leave Rust-owned direct Plus placement."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_PLUS)

	#============================================
	def _on_toggle_draw_vector(self, tool: _NativeLineTool, checked: bool) -> None:
		"""Enter or leave one renderer-preflighted ordinary vector tool."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(tool)

	#============================================
	def _on_toggle_insert_text(self, checked: bool) -> None:
		"""Enter or leave Rust-owned click-to-place standalone Text authoring."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.INSERT_TEXT)

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
		getattr(self, "_cancel_structure_selection", lambda: None)()
		self._cancel_line_gesture(clear_status=False)
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			self._cancel_line_gesture()
			return
		if tool is _NativeLineTool.DRAW_BOND:
			action = self._draw_bond_action
		elif tool is _NativeLineTool.DRAW_ARROW:
			action = self._draw_arrow_action
		elif tool is _NativeLineTool.DRAW_PLUS:
			action = self._draw_plus_action
		elif tool in self._draw_vector_actions:
			action = self._draw_vector_actions[tool]
		elif tool is _NativeLineTool.INSERT_TEXT:
			action = self._insert_text_action
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
		elif tool is _NativeLineTool.DRAW_ARROW:
			message = self.tr("Draw Arrow: drag a straight normal reaction arrow; Esc cancels.")
		elif tool is _NativeLineTool.DRAW_PLUS:
			message = self.tr("Draw Plus: click once to place a Plus; Esc cancels.")
		elif tool in self._draw_vector_actions:
			message = self.tr("{0}: drag to create one renderer-preflighted shape; Esc cancels.").format(
				self._draw_vector_actions[tool].text(),
			)
		elif tool is _NativeLineTool.INSERT_TEXT:
			message = self.tr("Insert Text: click a page location, enter text, then Save. Escape cancels.")
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
		if event.type() == PySide6.QtCore.QEvent.Type.FocusOut:
			self._cancel_line_gesture()
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
		if intent.tool is _NativeLineTool.DRAW_ARROW:
			try:
				import ferrum_qt.ferrum.engine as engine
				gesture = intent.tab.begin_straight_normal_arrow_gesture(
					float(press_scene.x()), float(press_scene.y()),
					engine.PresentationGestureSnapPolicyV1(),
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_edit_refusal(self._presentation_gesture_refusal(exc))
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
		if intent.tool is _NativeLineTool.DRAW_BOND:
			drawing = self._drawing_parameters.snapshot()
			intent = dataclasses.replace(intent, drawing=drawing)
			self._line_gesture_intent = intent
			message = self._draw_bond_feedback(drawing)
			self._draw_bond_action.setToolTip(message)
			self.statusBar().showMessage(message)
			# P0.1's Rust gesture deliberately has a fixed-carbon endpoint grammar.
			# Keep configurable normal drawing on the established prepared-operation
			# route until its richer element/order/snap contract is redesigned in Rust.
			if (
				drawing.presentation_name == "normal"
				and drawing.element == "C"
				and drawing.order_name == "single"
			):
				atom_id = intent.tab.durable_atom_at_viewport_point(point)
				if atom_id is None:
					self.statusBar().showMessage(
						self.tr("Draw Bond must start on an existing atom."), 5000,
					)
					return
				try:
					gesture = intent.tab.begin_direct_bond_gesture(
						atom_id, drawing.bond_presentation(),
						intent.tab.view.hex_grid_snap_enabled,
					)
				except Exception as exc:
					self._cancel_line_gesture()
					self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
					return
				self._line_gesture_intent = dataclasses.replace(
					intent,
					start_atom_id=atom_id,
					start_scene=intent.tab.durable_atom_scene_position(atom_id),
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
	def _update_line_gesture(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Move only the disposable Qt-local preview line."""
		intent = self._line_gesture_intent
		if intent is not None and intent.tool is _NativeLineTool.ROTATE_ATOMS:
			self._update_rotation_gesture(intent, event)
			return
		if intent is not None and intent.tool is _NativeLineTool.TRANSLATE_ROOTS:
			self._update_translation_gesture(intent, event)
			return
		if intent is not None and intent.tool is _NativeLineTool.DRAW_ARROW:
			self._update_presentation_gesture(intent, event.position().toPoint())
			return
		if intent is not None and intent.tool in self._draw_vector_actions:
			self._update_vector_gesture(intent, event.position().toPoint())
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
		if intent is not None and intent.tool is _NativeLineTool.DRAW_ARROW:
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
		if intent.tool is _NativeLineTool.DRAW_BOND and intent.direct_bond_gesture is not None:
			self._update_direct_bond_gesture(intent, event.position().toPoint())
			current = self._line_gesture_intent
			if current is None:
				return
			gesture = current.direct_bond_gesture
			preview = current.direct_bond_preview
			if gesture is None or preview is None:
				self._cancel_line_gesture()
				return
			self._reset_line_gesture_start()
			try:
				commit = intent.tab.commit_direct_bond_gesture(gesture, preview)
			except Exception as exc:
				self._cancel_line_gesture()
				self._refresh_actions()
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				return
			result_message = (
				self.tr("Added one Ferrum carbon and normal bond; drag again or press Esc.")
				if commit.created_new_atom
				else self.tr("Added one Ferrum normal bond; drag again or press Esc.")
			)
			self._finish_line_gesture(intent, result_message)
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
	def _update_direct_bond_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Ask Rust for one opaque direct-bond preview and project only its overlay."""
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed during the gesture; no operation was accepted.",
			))
			return
		gesture = intent.direct_bond_gesture
		if gesture is None:
			return
		end_atom_id = intent.tab.durable_atom_at_viewport_point(viewport_point)
		endpoint = (
			intent.tab.direct_bond_existing_endpoint(end_atom_id)
			if end_atom_id is not None
			else intent.tab.direct_bond_new_endpoint(
				float(intent.tab.view.mapToScene(viewport_point).x()),
				float(intent.tab.view.mapToScene(viewport_point).y()),
			)
		)
		outcome = intent.tab.preview_direct_bond_gesture(gesture, endpoint)
		self._retire_line_preview(intent.preview)
		import ferrum_qt.ferrum.engine as engine
		if type(outcome) is engine.DirectBondPreviewRefusalV1:
			self._cancel_line_gesture(clear_status=False)
			self._show_edit_refusal(self._unavailable_edit_refusal(
				self._direct_bond_refusal_message(outcome),
			))
			return
		if type(outcome) is not engine.DirectBondPreviewV1:
			raise RuntimeError("Ferrum direct-bond preview returned an unknown result")
		overlay = ferrum_qt.ferrum.direct_bond_preview.create_overlay(
			intent.tab, outcome.overlay,
		)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, direct_bond_preview=outcome,
		)

	#============================================
	def _update_presentation_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Request and paint exactly one backend-issued Arrow overlay."""
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed during Draw Arrow; start the tool again.",
			))
			return
		gesture = intent.presentation_gesture
		if gesture is None:
			return
		point = intent.tab.view.mapToScene(viewport_point)
		try:
			preview = intent.tab.preview_straight_normal_arrow_gesture(
				gesture, float(point.x()), float(point.y()),
			)
			overlay = ferrum_qt.ferrum.presentation_creation_preview.create_straight_normal_arrow_overlay(
				intent.tab, preview.overlay,
			)
		except Exception as exc:
			self._cancel_line_gesture(clear_status=False)
			self._show_edit_refusal(self._presentation_gesture_refusal(exc))
			return
		self._retire_line_preview(intent.preview)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, presentation_preview=preview,
		)

	#============================================
	def _complete_presentation_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Commit only the exact Arrow preview and then restore Rust selection."""
		self._update_presentation_gesture(intent, viewport_point)
		current = self._line_gesture_intent
		if current is None or current.presentation_gesture is None or current.presentation_preview is None:
			return
		self._reset_line_gesture_start()
		try:
			commit = current.tab.commit_straight_normal_arrow_gesture(
				current.presentation_gesture, current.presentation_preview,
			)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			# Rust accepted the Arrow and the tab retained its exact pending snapshot.
			# Reproject from that authority; do not reuse the preview or call it refused.
			self._replace_render_interaction_selection(None, current.tab)
			recovered = current.tab.refresh_authoritative()
			self._finish_line_gesture(current, self.tr(
				"Added one Ferrum reaction arrow; the display was refreshed after installation recovery."
				if recovered else
				"Added one Ferrum reaction arrow. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The reaction arrow was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"The reaction arrow was added, but its authoritative display still needs recovery; refresh before saving or editing.",
			))
			return
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._presentation_gesture_refusal(exc))
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = current.tab.observe_direct_root_interaction()
			selection = current.tab.select_direct_roots(
				observation, None, engine.RenderInteractionQueryV1.root(
					commit.root.identifier, engine.RenderInteractionModifierV1.replace,
				),
			)
		except Exception as exc:
			# Commit already installed its accepted Rust snapshot. Selection recovery
			# is secondary and must never describe this persisted Arrow as unchanged.
			self._replace_render_interaction_selection(None, current.tab)
			self._finish_line_gesture(current, self.tr(
				"Added one Ferrum reaction arrow. Selection was unavailable; refresh the Rust view before moving it.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The reaction arrow was added. Its selection could not be restored; refresh the Rust view before selecting or moving it.",
			))
			return
		self._replace_render_interaction_selection(selection, current.tab)
		self._finish_line_gesture(current, self.tr(
			"Added one Ferrum reaction arrow; drag again or press Esc.",
		))

	#============================================
	def _update_vector_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Request and paint only one Rust-issued ordinary vector overlay."""
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed during vector drawing; refresh the Rust view and start again.",
			))
			return
		if intent.vector_gesture is None:
			return
		point = intent.tab.view.mapToScene(viewport_point)
		try:
			preview = intent.tab.preview_presentation_vector_gesture(
				intent.vector_gesture, float(point.x()), float(point.y()),
			)
			overlay = ferrum_qt.ferrum.presentation_vector_preview.create_overlay(
				intent.tab, preview.overlay,
			)
		except Exception as exc:
			self._cancel_line_gesture(clear_status=False)
			self._show_edit_refusal(self._vector_gesture_refusal(exc))
			return
		self._retire_line_preview(intent.preview)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, vector_preview=preview,
		)

	#============================================
	def _complete_vector_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Preflight then commit exactly the opaque Rust vector receipt."""
		self._update_vector_gesture(intent, viewport_point)
		current = self._line_gesture_intent
		if current is None or current.vector_gesture is None or current.vector_preview is None:
			return
		try:
			prepared = current.tab.prepare_presentation_vector_gesture(
				current.vector_gesture, current.vector_preview,
			)
			self._reset_line_gesture_start()
			commit = current.tab.commit_presentation_vector_gesture(prepared)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, current.tab)
			recovered = current.tab.refresh_authoritative()
			self._finish_line_gesture(current, self.tr(
				"Added one Ferrum vector; the display was refreshed after installation recovery."
				if recovered else
				"Added one Ferrum vector. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The vector was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"The vector was added, but its authoritative display still needs recovery; refresh before saving or editing."
			))
			return
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._vector_gesture_refusal(exc))
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = current.tab.observe_direct_root_interaction()
			selection = current.tab.select_direct_roots(
				observation, None,
				engine.RenderInteractionQueryV1.root(commit.identifier),
			)
			self._replace_render_interaction_selection(selection, current.tab)
		except Exception:
			self._replace_render_interaction_selection(None, current.tab)
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The vector was added, but selection could not be restored; refresh before moving it.",
			))
		self._finish_line_gesture(current, self.tr(
			"Added one Ferrum {0}; drag again or press Esc."
		).format(self._draw_vector_actions[current.tool].text()[5:].lower()))

	#============================================
	def _complete_plus_gesture(self, intent: _LineGestureIntent) -> None:
		"""Commit one backend-owned Plus click and restore durable selection."""
		if (
			not self._line_gesture_is_current(intent)
			or intent.presentation_gesture is None
			or intent.presentation_preview is None
		):
			self._cancel_line_gesture()
			return
		try:
			commit = intent.tab.commit_plus_placement_gesture(
				intent.presentation_gesture, intent.presentation_preview,
			)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, intent.tab)
			recovered = intent.tab.refresh_authoritative()
			self._finish_line_gesture(intent, self.tr(
				"Added one Ferrum Plus; the display was refreshed after installation recovery."
				if recovered else
				"Added one Ferrum Plus. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The Plus was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"The Plus was added, but its authoritative display still needs recovery; refresh or reopen before saving or editing.",
			))
			return
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._presentation_gesture_refusal(exc))
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = intent.tab.observe_direct_root_interaction()
			selection = intent.tab.select_direct_roots(
				observation, None,
				engine.RenderInteractionQueryV1.root(commit.identifier),
			)
			self._replace_render_interaction_selection(selection, intent.tab)
		except Exception:
			self._replace_render_interaction_selection(None, intent.tab)
		self._finish_line_gesture(intent, self.tr(
			"Added one Ferrum Plus; click again or press Esc.",
		))

	#============================================
	def _complete_text_placement_gesture(self) -> None:
		"""Commit one exact Text preview, then select its durable Rust root."""
		intent = self._line_gesture_intent
		if (
			intent is None or intent.tool is not _NativeLineTool.INSERT_TEXT
			or not self._line_gesture_is_current(intent)
			or intent.text_gesture is None or intent.text_preview is None
		):
			return
		try:
			commit = intent.tab.commit_text_placement_gesture(
				intent.text_gesture, intent.text_preview,
			)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, intent.tab)
			recovered = intent.tab.refresh_authoritative()
			self._finish_line_gesture(intent, self.tr(
				"Text added; the display was refreshed after installation recovery."
				if recovered else
				"Text added. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Text was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"Text was added, but its authoritative display still needs recovery; refresh or reopen before saving or editing.",
			))
			return
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._text_placement_refusal(exc))
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = intent.tab.observe_direct_root_interaction()
			selection = intent.tab.select_direct_roots(
				observation, None, engine.RenderInteractionQueryV1.root(commit.identifier),
			)
			self._replace_render_interaction_selection(selection, intent.tab)
		except Exception:
			self._replace_render_interaction_selection(None, intent.tab)
			recovered = intent.tab.refresh_authoritative()
			self._finish_line_gesture(intent, self.tr(
				"Text added; the display was refreshed after selection recovery."
				if recovered else
				"Text added. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Text was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"Text was added, but its durable selection could not be restored; refresh or reopen before selecting or moving it.",
			))
			return
		self._finish_line_gesture(intent, self.tr(
			"Text added; click another page location or press Esc.",
		))

	#============================================
	def _text_placement_refusal(self, error: Exception) -> object:
		"""Present closed Rust Text failures without parsing error prose for policy."""
		import ferrum_qt.ferrum.engine as engine
		category = getattr(error, "category", None)
		if category is engine.TextPlacementErrorCategoryV1.unrenderable_standard:
			message = "Text insertion is unchanged. Repair the drawing standard, then start the tool again."
		elif category in (
			engine.TextPlacementErrorCategoryV1.blank_content,
			engine.TextPlacementErrorCategoryV1.unsupported_style,
			engine.TextPlacementErrorCategoryV1.invalid_font_override,
		):
			message = "Text insertion is unchanged. Correct the text or supported formatting and try again."
		elif category in (
			engine.TextPlacementErrorCategoryV1.stale_snapshot,
			engine.TextPlacementErrorCategoryV1.session_conflict,
		):
			message = "Text insertion is unchanged. Refresh the Rust view and start the tool again."
		else:
			message = "Text insertion is unchanged. Choose another location or restart the tool."
		return self._unavailable_edit_refusal(message)

	#============================================
	def _presentation_gesture_refusal(self, error: Exception) -> object:
		"""Map closed Rust Arrow refusal categories to actionable recovery text."""
		import ferrum_qt.ferrum.engine as engine
		category = getattr(error, "category", None)
		if category in (
			engine.PresentationGestureCategoryV1.collapsed_endpoint,
			engine.PresentationGestureCategoryV1.below_minimum_length,
		):
			message = "Draw Arrow is unchanged. Drag to a clearly different endpoint and try again."
		elif category in (
			engine.PresentationGestureCategoryV1.stale_revision,
			engine.PresentationGestureCategoryV1.stale_digest,
			engine.PresentationGestureCategoryV1.session_conflict,
		):
			message = "Draw Arrow is unchanged. Refresh the Rust view and start the tool again."
		else:
			message = "Draw Arrow is unchanged. Adjust the endpoint or tool style and try again."
		return self._unavailable_edit_refusal(message)

	#============================================
	def _vector_gesture_refusal(self, error: Exception) -> object:
		"""Present closed render-bridge vector failures with their recovery class."""
		import ferrum_qt.ferrum.engine as engine
		category = getattr(error, "category", None)
		if category in (
			engine.PresentationVectorGestureCategoryV1.degenerate_geometry,
			engine.PresentationVectorGestureCategoryV1.invalid_point,
		):
			message = "Vector drawing is unchanged. Drag to a clearly different finite endpoint and try again."
		elif category is engine.PresentationVectorGestureCategoryV1.unrenderable_standard:
			message = "Vector drawing is unchanged. Choose a supported drawing appearance, then try again."
		elif category in (
			engine.PresentationVectorGestureCategoryV1.stale_snapshot,
			engine.PresentationVectorGestureCategoryV1.session_conflict,
			engine.PresentationVectorGestureCategoryV1.replayed_gesture,
		):
			message = "Vector drawing is unchanged. Refresh the Rust view and start the tool again."
		else:
			message = "Vector drawing is unchanged. Adjust the shape or drawing appearance and try again."
		return self._unavailable_edit_refusal(message)

	#============================================
	@staticmethod
	def _direct_bond_refusal_message(refusal: object) -> str:
		"""Explain a typed ordinary Rust endpoint refusal without parsing strings."""
		import ferrum_qt.ferrum.engine as engine
		if refusal.category is engine.DirectBondGestureCategoryV1.self_loop:
			return "Choose a different atom or an empty endpoint, then start Draw Bond again."
		if refusal.category is engine.DirectBondGestureCategoryV1.cross_molecule:
			return "Choose an atom in the same molecule, then start Draw Bond again."
		if refusal.category is engine.DirectBondGestureCategoryV1.duplicate_bond:
			return "Those atoms already have a bond. Choose another endpoint, then start Draw Bond again."
		return "Choose a different Draw Bond endpoint, then start the tool again."

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
			modifier = (
				engine.RenderInteractionModifierV1.toggle
				if PySide6.QtWidgets.QApplication.keyboardModifiers()
				& PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier
				else engine.RenderInteractionModifierV1.replace
			)
			query = engine.RenderInteractionQueryV1.point(
				float(press_scene.x()), float(press_scene.y()), modifier,
			)
			hit = intent.tab.select_direct_roots(observation, None, query)
			if (
				seed is not None and hit.roots
				and any(root.identifier == hit.roots[0].identifier for root in seed.roots)
				and modifier == engine.RenderInteractionModifierV1.replace
			):
				selection = seed
			else:
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
			self._replace_render_interaction_selection(None, intent.tab)
			self._show_edit_refusal(self._render_interaction_refusal(exc))
			return True
		if commit.changed:
			self._replace_render_interaction_selection(None, intent.tab)
			self._finish_line_gesture(intent, self.tr("Moved selected Ferrum root; select it again to continue."))
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
			direct_bond_gesture=None,
			direct_bond_preview=None,
			presentation_gesture=None,
			presentation_preview=None,
			vector_gesture=None,
			vector_preview=None,
		)

	#============================================
	def _cancel_line_gesture(self, clear_status: bool = True) -> None:
		"""Release pointer capture and terminally retire its preview."""
		intent = self._line_gesture_intent
		self._line_gesture_intent = None
		self._draw_bond_action.setChecked(False)
		self._draw_arrow_action.setChecked(False)
		self._draw_plus_action.setChecked(False)
		for action in self._draw_vector_actions.values():
			action.setChecked(False)
		self._insert_text_action.setChecked(False)
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
			self._retire_line_preview(intent.direct_root_preview_item)
			self._retire_line_preview(intent.direct_root_marquee)
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
