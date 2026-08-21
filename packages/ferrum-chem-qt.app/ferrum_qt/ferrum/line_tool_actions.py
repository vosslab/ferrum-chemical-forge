"""Ferrum line-tool action setup and activation."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.line_tool_intent

_NativeLineTool = ferrum_qt.ferrum.line_tool_intent._NativeLineTool
_LineGestureIntent = ferrum_qt.ferrum.line_tool_intent._LineGestureIntent


class FerrumNativeLineToolActionsMixin:
	"""Build and activate the checkable Ferrum pointer tools."""

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
		self._draw_bond_action.setWhatsThis(self.tr(
			"Drag from an atom to another atom or empty space. Creates a normal bond "
			"using the visible Next Drawing order. Escape cancels.",
		))
		self._draw_bond_action.setToolTip(self.tr(
			"Drag from an atom to another atom or empty space. Creates a normal bond "
			"using the visible Next Drawing order. Escape cancels.",
		))
		self._draw_bond_action.setStatusTip(self.tr(
			"Draw a normal bond using the visible Next Drawing order. Escape cancels.",
		))
		self._connect_interaction_action_v1(self._draw_bond_action, self._on_toggle_draw_bond)
		self._add_interaction_action_to_menu_v1(edit_menu, self._draw_bond_action)
		self._draw_arrow_action = PySide6.QtGui.QAction(self.tr("Draw Arrow"), self)
		self._draw_arrow_action.setCheckable(True)
		self._draw_arrow_action.setToolTip(self.tr(
			"Drag a straight normal reaction arrow; Esc cancels without changing the document",
		))
		self._draw_arrow_action.setStatusTip(self.tr(
			"Draw a straight normal reaction arrow. Escape cancels.",
		))
		self._connect_interaction_action_v1(self._draw_arrow_action, self._on_toggle_draw_arrow)
		self._add_interaction_action_to_menu_v1(edit_menu, self._draw_arrow_action)
		self._draw_equilibrium_arrow_action = PySide6.QtGui.QAction(
			self.tr("Draw Equilibrium Arrow"), self,
		)
		self._draw_equilibrium_arrow_action.setObjectName("drawEquilibriumArrowAction")
		self._draw_equilibrium_arrow_action.setCheckable(True)
		self._draw_equilibrium_arrow_action.setWhatsThis(self.tr(
			"Drag a straight equilibrium reaction arrow. Escape cancels without changing the document.",
		))
		self._draw_equilibrium_arrow_action.setToolTip(self.tr(
			"Drag a straight equilibrium reaction arrow; Esc cancels without changing the document",
		))
		self._draw_equilibrium_arrow_action.setStatusTip(self.tr(
			"Draw a straight equilibrium reaction arrow. Escape cancels.",
		))
		self._connect_interaction_action_v1(
			self._draw_equilibrium_arrow_action, self._on_toggle_draw_equilibrium_arrow,
		)
		self._add_interaction_action_to_menu_v1(edit_menu, self._draw_equilibrium_arrow_action)
		self._draw_plus_action = PySide6.QtGui.QAction(self.tr("Draw Plus"), self)
		self._draw_plus_action.setCheckable(True)
		self._draw_plus_action.setToolTip(self.tr("Click to place one Plus; Escape cancels without changing the document"))
		self._connect_interaction_action_v1(self._draw_plus_action, self._on_toggle_draw_plus)
		self._add_interaction_action_to_menu_v1(edit_menu, self._draw_plus_action)
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
			self._add_interaction_action_to_menu_v1(edit_menu, action)
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
		self._add_interaction_action_to_menu_v1(edit_menu, self._insert_text_action)
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
		self._add_interaction_action_to_menu_v1(edit_menu, self._insert_cyclohexane_ring_action)
		self._attach_cyclohexane_ring_action = PySide6.QtGui.QAction(
			self.tr("Attach Cyclohexane Ring"), self,
		)
		self._attach_cyclohexane_ring_action.setCheckable(True)
		self._attach_cyclohexane_ring_action.setToolTip(self.tr(
			"Drag from an eligible atom to attach one six-carbon ring; Escape cancels.",
		))
		self._attach_cyclohexane_ring_action.setStatusTip(self.tr(
			"Attach Cyclohexane Ring: drag from an eligible atom; Escape cancels.",
		))
		self._connect_interaction_action_v1(
			self._attach_cyclohexane_ring_action, self._on_toggle_attach_cyclohexane_ring,
		)
		self._add_interaction_action_to_menu_v1(edit_menu, self._attach_cyclohexane_ring_action)
		self._draw_wavy_action = PySide6.QtGui.QAction(self.tr("Draw Wavy Line"), self)
		self._draw_wavy_action.setCheckable(True)
		self._draw_wavy_action.setToolTip(
			self.tr("Drag between two page points; Esc cancels"),
		)
		self._connect_interaction_action_v1(self._draw_wavy_action, self._on_toggle_draw_wavy)
		self._add_interaction_action_to_menu_v1(edit_menu, self._draw_wavy_action)
		self._draw_bracket_action = PySide6.QtGui.QAction(
			self.tr("Draw Rectangular Bracket"), self,
		)
		self._draw_bracket_action.setCheckable(True)
		self._draw_bracket_action.setToolTip(
			self.tr("Drag a finite nonempty rectangle; Esc cancels"),
		)
		self._connect_interaction_action_v1(self._draw_bracket_action, self._on_toggle_draw_bracket)
		self._add_interaction_action_to_menu_v1(edit_menu, self._draw_bracket_action)
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
		self._add_interaction_action_to_menu_v1(edit_menu, self._draw_round_bracket_action)
		self._move_atom_action = PySide6.QtGui.QAction(self.tr("Move Atom"), self)
		self._move_atom_action.setCheckable(True)
		self._move_atom_action.setToolTip(
			self.tr("Drag one existing atom to an exact new scene point; Esc cancels"),
		)
		self._connect_interaction_action_v1(self._move_atom_action, self._on_toggle_move_atom)
		self._add_interaction_action_to_menu_v1(edit_menu, self._move_atom_action)
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
		self._add_interaction_action_to_menu_v1(edit_menu, self._rotate_atoms_action)
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
		self._add_interaction_action_to_menu_v1(edit_menu, self._translate_roots_action)
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
		self._draw_equilibrium_arrow_action.setEnabled(enabled)
		self._draw_plus_action.setEnabled(enabled)
		for action in self._draw_vector_actions.values():
			action.setEnabled(enabled)
		self._insert_text_action.setEnabled(enabled)
		self._insert_cyclohexane_ring_action.setEnabled(enabled)
		self._attach_cyclohexane_ring_action.setEnabled(enabled)
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
		self._refresh_local_document_open_action()

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
	def _on_toggle_draw_equilibrium_arrow(self, checked: bool) -> None:
		"""Enter or leave Rust-owned straight equilibrium Arrow creation."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_EQUILIBRIUM_ARROW)

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

	def _on_toggle_attach_cyclohexane_ring(self, checked: bool) -> None:
		"""Arm one direct Rust-owned C6 attachment gesture."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.ATTACH_CYCLOHEXANE_RING)

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
		if not self._cancel_line_gesture(clear_status=False):
			return
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			self._cancel_line_gesture()
			return
		if tool is _NativeLineTool.DRAW_BOND:
			action = self._draw_bond_action
		elif tool is _NativeLineTool.DRAW_ARROW:
			action = self._draw_arrow_action
		elif tool is _NativeLineTool.DRAW_EQUILIBRIUM_ARROW:
			action = self._draw_equilibrium_arrow_action
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
		elif tool is _NativeLineTool.INSERT_CYCLOHEXANE_RING:
			action = self._insert_cyclohexane_ring_action
		else:
			action = self._attach_cyclohexane_ring_action
		action.setChecked(True)
		snapshot = tab.current_snapshot
		viewport = tab.view.viewport()
		drawing = self._drawing_parameters.snapshot() if tool is _NativeLineTool.DRAW_BOND else None
		self._line_gesture_intent = _LineGestureIntent(
			tab, viewport, snapshot.revision, snapshot.digest, tool, drawing,
		)
		self._synchronize_mode_state()
		self._refresh_cancel_tool_action()
		viewport.installEventFilter(self)
		viewport.setFocus()
		self._restore_line_tool_focus_on_next_turn(self._line_gesture_intent)
		tab.view.show_keyboard_cursor()
		if tool is _NativeLineTool.DRAW_BOND:
			if drawing is None:
				raise RuntimeError("Ferrum Draw Bond activation has no drawing snapshot")
			message = self._draw_bond_feedback(drawing)
			self._draw_bond_action.setToolTip(message)
		elif tool is _NativeLineTool.DRAW_ARROW:
			message = self.tr("Draw Arrow: drag a straight normal reaction arrow; Esc cancels.")
		elif tool is _NativeLineTool.DRAW_EQUILIBRIUM_ARROW:
			message = self.tr("Draw Equilibrium Arrow: drag a straight equilibrium reaction arrow; Esc cancels.")
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
		elif tool is _NativeLineTool.INSERT_CYCLOHEXANE_RING:
			message = self.tr(
				"Insert Cyclohexane Ring: click an empty page location; Escape cancels.",
			)
		else:
			message = self.tr(
				"Attach Cyclohexane Ring: drag from an eligible atom to choose direction; Escape cancels.",
			)
		self.statusBar().showMessage(message)

	#============================================
	def _draw_bond_feedback(
			self,
			drawing: ferrum_qt.ferrum.drawing_parameters.
			FerrumNativeDrawingParametersSnapshot,
			) -> str:
		"""Name the frozen normal-order Draw Bond contract in human wording."""
		return self.tr(
			"Draw Bond: Normal {0}; drag between atoms or empty canvas locations. "
			"Shift+Arrow is fine movement; Esc cancels."
		).format(drawing.order_name)

	#============================================
	@staticmethod
	def _normal_direct_bond_presentation(
			drawing: ferrum_qt.ferrum.drawing_parameters.
			FerrumNativeDrawingParametersSnapshot,
			) -> object:
		"""Map one frozen visible order to the closed normal Rust presentation."""
		import ferrum_qt.ferrum.engine as engine
		if drawing.order_name == "single":
			return engine.DocumentBondPresentationV1.normal_single
		if drawing.order_name == "double":
			return engine.DocumentBondPresentationV1.normal_double
		if drawing.order_name == "triple":
			return engine.DocumentBondPresentationV1.normal_triple
		raise ValueError("Ferrum Draw Bond activation contains an unknown order")

	#============================================
