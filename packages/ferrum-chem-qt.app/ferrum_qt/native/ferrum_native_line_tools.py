"""Revision-bound Qt pointer tools for the standalone Ferrum-native window."""

# Standard Library
import dataclasses
import enum
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_rotation
import ferrum_qt.native.ferrum_native_translation


#============================================
class _NativeLineTool(enum.Enum):
	"""Closed native tools that share one revision-bound line gesture."""

	DRAW_SINGLE_BOND = "draw_single_bond"
	CREATE_WAVY = "create_wavy"
	CREATE_RECTANGULAR_BRACKET = "create_rectangular_bracket"
	CREATE_ROUND_BRACKET = "create_round_bracket"
	MOVE_ATOM = "move_atom"
	ROTATE_ATOMS = "rotate_atoms"
	TRANSLATE_ROOTS = "translate_roots"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _LineGestureIntent:
	"""One revision-bound atom pointer gesture and its local preview."""

	tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	tool: _NativeLineTool
	start_atom_id: str | None = None
	start_scene: PySide6.QtCore.QPointF | None = None
	press_scene: PySide6.QtCore.QPointF | None = None
	preview: (
		PySide6.QtWidgets.QGraphicsLineItem
		| PySide6.QtWidgets.QGraphicsRectItem
		| None
	) = None
	rotation_selection: (
		ferrum_qt.native.ferrum_native_rotation.FerrumNativeRotationSelection | None
	) = None
	rotation_preview: (
		ferrum_qt.native.ferrum_native_rotation.FerrumNativeRotationPreview | None
	) = None
	translation_selection: (
		ferrum_qt.native.ferrum_native_translation.FerrumNativeTranslationSelection
		| None
	) = None
	translation_preview: (
		ferrum_qt.native.ferrum_native_translation.FerrumNativeTranslationPreview
		| None
	) = None
	translation_delta: tuple[float, float] = (0.0, 0.0)
	last_angle: float | None = None
	accumulated_angle: float = 0.0


#============================================
class FerrumNativeLineToolsMixin:
	"""Own the disposable pointer gestures used by the native document host.

	The host supplies the active document tab, warning/status surfaces, and action
	refresh. This mixin owns pointer capture and local preview retirement only;
	all document mutation stays behind the tab's Rust session methods.
	"""

	#============================================
	def _initialize_line_tools(self) -> None:
		"""Initialize the one mutually exclusive pointer-tool intent."""
		self._line_gesture_intent: _LineGestureIntent | None = None

	#============================================
	def _build_line_tool_actions(self, edit_menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the checkable native pointer tools to the host's Edit menu."""
		self._draw_single_bond_action = PySide6.QtGui.QAction(
			self.tr("Draw Single Bond"), self,
		)
		self._draw_single_bond_action.setCheckable(True)
		self._draw_single_bond_action.setToolTip(
			self.tr("Drag from an atom to another atom or empty space; Esc cancels"),
		)
		self._draw_single_bond_action.triggered.connect(self._on_toggle_draw_single_bond)
		edit_menu.addAction(self._draw_single_bond_action)
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
				"Drag complete selected roots; Esc cancels without changing Rust",
			),
		)
		self._translate_roots_action.triggered.connect(self._on_toggle_translate_roots)
		edit_menu.addAction(self._translate_roots_action)

	#============================================
	def _refresh_line_tool_actions(self, enabled: bool) -> None:
		"""Apply the host's authoritative action policy to both pointer tools."""
		self._draw_single_bond_action.setEnabled(enabled)
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

	#============================================
	def _on_toggle_draw_single_bond(self, checked: bool) -> None:
		"""Enter or leave one revision-bound atom-to-atom drawing mode."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_SINGLE_BOND)

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
		if tool is _NativeLineTool.DRAW_SINGLE_BOND:
			action = self._draw_single_bond_action
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
		else:
			action = self._translate_roots_action
		action.setChecked(True)
		snapshot = tab.current_snapshot
		viewport = tab.view.viewport()
		self._line_gesture_intent = _LineGestureIntent(
			tab, viewport, snapshot.revision, snapshot.digest, tool,
		)
		viewport.installEventFilter(self)
		viewport.setFocus()
		if tool is _NativeLineTool.DRAW_SINGLE_BOND:
			message = self.tr(
				"Drag from an atom to another atom or empty space; Esc cancels Draw Bond.",
			)
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
		else:
			message = self.tr(
				"Drag complete selected roots; Esc cancels Move Complete Roots.",
			)
		self.statusBar().showMessage(message)

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Capture native atom-insertion or line-tool pointer intent."""
		line_intent = self._line_gesture_intent
		if line_intent is not None and watched is line_intent.viewport:
			return self._line_gesture_event(event)
		intent = self._atom_insertion_intent
		if intent is None or watched is not intent.viewport:
			return super().eventFilter(watched, event)
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			if event.key() == PySide6.QtCore.Qt.Key.Key_Escape:
				self._cancel_atom_insertion()
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
			if event.key() == PySide6.QtCore.Qt.Key.Key_Escape:
				self._cancel_line_gesture()
				return True
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonPress:
			if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
				return False
			self._start_line_gesture(event)
			return True
		if event.type() == PySide6.QtCore.QEvent.Type.MouseMove:
			self._update_line_gesture(event)
			return self._line_gesture_intent is not None
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonRelease:
			if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
				return False
			self._complete_line_gesture(event)
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
			title = self._line_tool_stale_title(intent.tool)
			self._show_native_file_warning(
				title,
				"The document changed before the gesture; start the tool again.",
			)
			return
		point = event.position().toPoint()
		press_scene = intent.tab.view.mapToScene(point)
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
			try:
				preview = (
					self._new_bracket_preview(intent.tab, press_scene)
					if intent.tool in (
						_NativeLineTool.CREATE_RECTANGULAR_BRACKET,
						_NativeLineTool.CREATE_ROUND_BRACKET,
					)
					else self._new_line_preview(intent.tab, press_scene)
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._show_native_file_warning("Native Pointer Preview Error", str(exc))
				return
			self._line_gesture_intent = dataclasses.replace(
				intent, start_scene=press_scene, press_scene=press_scene, preview=preview,
			)
			return
		atom_id = intent.tab.durable_atom_at_viewport_point(point)
		if atom_id is None:
			message = (
				self.tr("Draw Bond must start on an existing atom.")
				if intent.tool is _NativeLineTool.DRAW_SINGLE_BOND
				else self.tr("Move Atom must start on an existing atom.")
			)
			self.statusBar().showMessage(message, 5000)
			return
		start_scene = intent.tab.durable_atom_scene_position(atom_id)
		try:
			preview = self._new_line_preview(intent.tab, start_scene)
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_native_file_warning("Native Pointer Preview Error", str(exc))
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
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			title = self._line_tool_stale_title(intent.tool)
			self._show_native_file_warning(
				title,
				"The document changed during the gesture; no operation was accepted.",
			)
			return
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(intent.preview):
			self._cancel_line_gesture()
			return
		current = intent.tab.view.mapToScene(event.position().toPoint())
		if intent.tool in (
			_NativeLineTool.CREATE_RECTANGULAR_BRACKET,
			_NativeLineTool.CREATE_ROUND_BRACKET,
		):
			assert isinstance(intent.preview, PySide6.QtWidgets.QGraphicsRectItem)
			intent.preview.setRect(_normalized_rect(intent.start_scene, current))
		else:
			assert isinstance(intent.preview, PySide6.QtWidgets.QGraphicsLineItem)
			intent.preview.setLine(PySide6.QtCore.QLineF(intent.start_scene, current))

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
				)
				and intent.start_atom_id is None
			)
			):
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			title = self._line_tool_stale_title(intent.tool)
			self._show_native_file_warning(
				title,
				"The document changed during the gesture; no operation was accepted.",
			)
			return
		release_point = event.position().toPoint()
		release_scene = intent.tab.view.mapToScene(release_point)
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
				self._show_native_file_warning("Native Draw Wavy Error", str(exc))
				return
			self._finish_line_gesture(
				intent,
				self.tr("Added one Rust-native Wavy line; drag again or press Esc."),
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
				self._show_native_file_warning(
					self._line_tool_error_title(intent.tool), str(exc),
				)
				return
			self._finish_line_gesture(
				intent,
				self.tr(
					"Added one Rust-native bracket pair; drag again or press Esc.",
				),
			)
			return
		end_atom_id = intent.tab.durable_atom_at_viewport_point(release_point)
		start_atom_id = intent.start_atom_id
		assert start_atom_id is not None
		if intent.tool is _NativeLineTool.MOVE_ATOM:
			delta = release_scene - intent.press_scene
			target = intent.start_scene + delta
			try:
				intent.tab.move_atom_to(
					start_atom_id, float(target.x()), float(target.y()),
				)
			except Exception as exc:
				self._cancel_line_gesture()
				self._refresh_actions()
				self._show_native_file_warning("Native Move Atom Error", str(exc))
				return
			result_message = self.tr(
				"Moved one Rust-native atom; drag again or press Esc.",
			)
			self._finish_line_gesture(intent, result_message)
			return
		if end_atom_id == start_atom_id:
			self.statusBar().showMessage(
				self.tr("Release Draw Bond on a different atom or in empty space."), 5000,
			)
			return
		try:
			if end_atom_id is None:
				intent.tab.add_bonded_atom_at(
					start_atom_id, "C", float(release_scene.x()), float(release_scene.y()),
				)
				result_message = self.tr(
					"Added one Rust-native carbon and single bond; drag again or press Esc.",
				)
			else:
				intent.tab.select_atoms((start_atom_id, end_atom_id))
				intent.tab.add_single_bond_between_selected_atoms()
				result_message = self.tr(
					"Added one Rust-native single bond; drag again or press Esc.",
				)
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_native_file_warning("Native Draw Bond Error", str(exc))
			return
		self._finish_line_gesture(intent, result_message)

	#============================================
	def _start_rotation_gesture(self, intent: _LineGestureIntent,
			press_scene: PySide6.QtCore.QPointF) -> None:
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
			preview = ferrum_qt.native.ferrum_native_rotation.create_rotation_preview(
				intent.tab, selection,
			)
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_native_file_warning("Native Rotate Atoms Unavailable", str(exc))
			return
		self._line_gesture_intent = dataclasses.replace(
			intent,
			press_scene=press_scene,
			start_scene=selection.center,
			rotation_selection=selection,
			rotation_preview=preview,
			last_angle=math.atan2(dy, dx),
		)

	#============================================
	def _update_rotation_gesture(self, intent: _LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Advance one unwrapped angle and update only its disposable skeleton."""
		if (
				intent.rotation_preview is None
				or intent.rotation_selection is None
				or intent.last_angle is None
			):
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_native_file_warning(
				"Native Rotate Atoms Stale",
				"The document changed during the gesture; no operation was accepted.",
			)
			return
		current_scene = intent.tab.view.mapToScene(event.position().toPoint())
		center = intent.rotation_selection.center
		dx = current_scene.x() - center.x()
		dy = current_scene.y() - center.y()
		if not math.isfinite(dx) or not math.isfinite(dy):
			self._cancel_line_gesture()
			self._show_native_file_warning(
				"Native Rotate Atoms Error", "Rotation pointer position must be finite.",
			)
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
		ferrum_qt.native.ferrum_native_rotation.update_rotation_preview(
			intent.rotation_preview, float(angle),
		)
		self._line_gesture_intent = dataclasses.replace(
			intent, last_angle=current_angle, accumulated_angle=angle,
		)

	#============================================
	def _complete_rotation_gesture(self, intent: _LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
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
				self.tr("Rotate Selected Atoms remains active; no rotation was requested."),
				5000,
			)
			return
		try:
			intent.tab.apply_selected_atom_rotation(selection, center, angle)
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_native_file_warning("Native Rotate Atoms Error", str(exc))
			return
		self._finish_line_gesture(
			intent,
			self.tr("Rotated selected Rust-native atoms; drag again or press Esc."),
		)

	#============================================
	def _start_translation_gesture(self, intent: _LineGestureIntent,
			press_scene: PySide6.QtCore.QPointF) -> None:
		"""Capture complete roots and create one disposable bounds preview."""
		try:
			selection = intent.tab.selected_top_level_translation()
			preview = ferrum_qt.native.ferrum_native_translation.create_translation_preview(
				intent.tab, selection,
			)
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_native_file_warning(
				"Native Move Complete Roots Unavailable", str(exc),
			)
			return
		self._line_gesture_intent = dataclasses.replace(
			intent,
			press_scene=press_scene,
			translation_selection=selection,
			translation_preview=preview,
		)

	#============================================
	def _update_translation_gesture(self, intent: _LineGestureIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Move only the local complete-root bounds preview."""
		if intent.translation_preview is None or intent.press_scene is None:
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_native_file_warning(
				"Native Move Complete Roots Stale",
				"The document changed during the gesture; no operation was accepted.",
			)
			return
		current = intent.tab.view.mapToScene(event.position().toPoint())
		dx = float(current.x() - intent.press_scene.x())
		dy = float(current.y() - intent.press_scene.y())
		if not math.isfinite(dx) or not math.isfinite(dy):
			self._cancel_line_gesture()
			self._show_native_file_warning(
				"Native Move Complete Roots Error",
				"Translation pointer position must be finite.",
			)
			return
		ferrum_qt.native.ferrum_native_translation.update_translation_preview(
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
				selection.targets,
				selection.durable_selection,
				dx,
				dy,
			)
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_native_file_warning("Native Move Complete Roots Error", str(exc))
			return
		self._finish_line_gesture(
			intent,
			self.tr("Moved complete Rust-native roots; drag again or press Esc."),
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
		self._refresh_actions()

	#============================================
	def _new_line_preview(self, tab: object,
			start: PySide6.QtCore.QPointF) -> PySide6.QtWidgets.QGraphicsLineItem:
		"""Create one scene-owned, non-authoritative interaction preview."""
		scene = tab.view.scene()
		if scene is None:
			raise RuntimeError("native document has no current scene")
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
			raise RuntimeError("native document has no current scene")
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
			start_atom_id=None,
			start_scene=None,
			press_scene=None,
			preview=None,
			rotation_selection=None,
			rotation_preview=None,
			translation_selection=None,
			translation_preview=None,
			translation_delta=(0.0, 0.0),
			last_angle=None,
			accumulated_angle=0.0,
		)

	#============================================
	def _cancel_line_gesture(self, clear_status: bool = True) -> None:
		"""Release pointer capture and terminally retire its preview."""
		intent = self._line_gesture_intent
		self._line_gesture_intent = None
		self._draw_single_bond_action.setChecked(False)
		self._draw_wavy_action.setChecked(False)
		self._draw_bracket_action.setChecked(False)
		self._draw_round_bracket_action.setChecked(False)
		self._move_atom_action.setChecked(False)
		self._rotate_atoms_action.setChecked(False)
		self._translate_roots_action.setChecked(False)
		if intent is not None:
			intent.viewport.removeEventFilter(self)
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

	#============================================
	@staticmethod
	def _line_tool_stale_title(tool: _NativeLineTool) -> str:
		"""Return one actionable title for a gesture invalidated by a document edit."""
		if tool is _NativeLineTool.DRAW_SINGLE_BOND:
			return "Native Draw Bond Stale"
		if tool is _NativeLineTool.CREATE_WAVY:
			return "Native Draw Wavy Stale"
		if tool is _NativeLineTool.CREATE_RECTANGULAR_BRACKET:
			return "Native Draw Bracket Stale"
		if tool is _NativeLineTool.CREATE_ROUND_BRACKET:
			return "Native Draw Round Bracket Stale"
		if tool is _NativeLineTool.ROTATE_ATOMS:
			return "Native Rotate Atoms Stale"
		if tool is _NativeLineTool.TRANSLATE_ROOTS:
			return "Native Move Complete Roots Stale"
		return "Native Move Atom Stale"

	#============================================
	@staticmethod
	def _line_tool_error_title(tool: _NativeLineTool) -> str:
		"""Return the exact bracket action title for a rejected mutation."""
		if tool is _NativeLineTool.CREATE_ROUND_BRACKET:
			return "Native Draw Round Bracket Error"
		return "Native Draw Bracket Error"

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
