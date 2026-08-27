"""Ferrum line-tool action setup and activation."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.line_tool_intent
import ferrum_qt.ferrum.line_tool_mode
import ferrum_qt.ferrum.regular_ring
import ferrum_qt.ferrum.window_mode_sync
import ferrum_qt.modes.base_mode

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
	def _build_line_tool_actions(self) -> None:
		"""Add the checkable Ferrum pointer tools to the host's Draw menu."""
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
		self._register_line_tool_action(
			"draw.bond", self._draw_bond_action, ferrum_qt.modes.base_mode.ModeId.DRAW,
			_NativeLineTool.DRAW_BOND, "Draw Bond", self._on_toggle_draw_bond,
		)
		self._draw_solid_wedge_bond_action = PySide6.QtGui.QAction(
			self.tr("Draw Solid Wedge Bond"), self,
		)
		self._draw_solid_wedge_bond_action.setCheckable(True)
		self._draw_solid_wedge_bond_action.setToolTip(self.tr(
			"Drag from the stereo tip to the stereo base. Creates a Rust-owned solid wedge bond; Escape cancels.",
		))
		self._draw_solid_wedge_bond_action.setStatusTip(self.tr(
			"Draw a solid wedge from tip to base. Escape cancels.",
		))
		self._register_line_tool_action(
			"draw.bond.solid_wedge", self._draw_solid_wedge_bond_action,
			ferrum_qt.modes.base_mode.ModeId.DRAW, _NativeLineTool.DRAW_BOND,
			"Draw Solid Wedge Bond", self._on_toggle_draw_solid_wedge_bond,
		)
		self._draw_hashed_wedge_bond_action = PySide6.QtGui.QAction(
			self.tr("Draw Hashed Wedge Bond"), self,
		)
		self._draw_hashed_wedge_bond_action.setCheckable(True)
		self._draw_hashed_wedge_bond_action.setToolTip(self.tr(
			"Drag from the stereo tip to the stereo base. Creates a Rust-owned hashed wedge bond; Escape cancels.",
		))
		self._draw_hashed_wedge_bond_action.setStatusTip(self.tr(
			"Draw a hashed wedge from tip to base. Escape cancels.",
		))
		self._register_line_tool_action(
			"draw.bond.hashed_wedge", self._draw_hashed_wedge_bond_action,
			ferrum_qt.modes.base_mode.ModeId.DRAW, _NativeLineTool.DRAW_BOND,
			"Draw Hashed Wedge Bond", self._on_toggle_draw_hashed_wedge_bond,
		)
		self._draw_arrow_action = PySide6.QtGui.QAction(self.tr("Draw Arrow"), self)
		self._draw_arrow_action.setCheckable(True)
		self._draw_arrow_action.setToolTip(self.tr(
			"Drag a straight normal reaction arrow; Esc cancels without changing the document",
		))
		self._draw_arrow_action.setStatusTip(self.tr(
			"Draw a straight normal reaction arrow. Escape cancels.",
		))
		self._register_line_tool_action(
			"draw.arrow", self._draw_arrow_action, ferrum_qt.modes.base_mode.ModeId.ARROW,
			_NativeLineTool.DRAW_ARROW, "Draw Arrow", self._on_toggle_draw_arrow,
		)
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
		self._register_line_tool_action(
			"draw.arrow.equilibrium", self._draw_equilibrium_arrow_action,
			ferrum_qt.modes.base_mode.ModeId.ARROW, _NativeLineTool.DRAW_EQUILIBRIUM_ARROW,
			"Draw Equilibrium Arrow", self._on_toggle_draw_equilibrium_arrow,
		)
		self._draw_curved_electron_arrow_action = PySide6.QtGui.QAction(
			self.tr("Draw Curved Electron Arrow"), self,
		)
		self._draw_curved_electron_arrow_action.setCheckable(True)
		self._draw_curved_electron_arrow_action.setToolTip(self.tr(
			"Click start, bend, and endpoint to create one Rust-owned curved electron arrow; Esc cancels",
		))
		self._register_line_tool_action(
			"draw.arrow.curved_electron", self._draw_curved_electron_arrow_action,
			ferrum_qt.modes.base_mode.ModeId.ARROW, _NativeLineTool.DRAW_CURVED_ELECTRON_ARROW,
			"Draw Curved Electron Arrow", self._on_toggle_draw_curved_electron_arrow,
		)
		self._draw_curved_retro_arrow_action = PySide6.QtGui.QAction(
			self.tr("Draw Curved Retro Arrow"), self,
		)
		self._draw_curved_retro_arrow_action.setCheckable(True)
		self._draw_curved_retro_arrow_action.setToolTip(self.tr(
			"Click start, bend, and endpoint to create one Rust-owned curved retro arrow; Esc cancels",
		))
		self._register_line_tool_action(
			"draw.arrow.curved_retro", self._draw_curved_retro_arrow_action,
			ferrum_qt.modes.base_mode.ModeId.ARROW, _NativeLineTool.DRAW_CURVED_RETRO_ARROW,
			"Draw Curved Retro Arrow", self._on_toggle_draw_curved_retro_arrow,
		)
		self._draw_curved_reaction_arrow_action = PySide6.QtGui.QAction(
			self.tr("Draw Curved Reaction Arrow"), self,
		)
		self._draw_curved_reaction_arrow_action.setCheckable(True)
		self._draw_curved_reaction_arrow_action.setToolTip(self.tr(
			"Click start, bend, and endpoint to create one Rust-owned curved reaction arrow; Esc cancels",
		))
		self._register_line_tool_action(
			"draw.arrow.curved_reaction", self._draw_curved_reaction_arrow_action,
			ferrum_qt.modes.base_mode.ModeId.ARROW, _NativeLineTool.DRAW_CURVED_REACTION_ARROW,
			"Draw Curved Reaction Arrow", self._on_toggle_draw_curved_reaction_arrow,
		)
		self._draw_curved_equilibrium_arrow_action = PySide6.QtGui.QAction(
			self.tr("Draw Curved Equilibrium Arrow"), self,
		)
		self._draw_curved_equilibrium_arrow_action.setCheckable(True)
		self._draw_curved_equilibrium_arrow_action.setToolTip(self.tr(
			"Click start, bend, and endpoint to create one Rust-owned curved equilibrium arrow; Esc cancels",
		))
		self._register_line_tool_action(
			"draw.arrow.curved_equilibrium", self._draw_curved_equilibrium_arrow_action,
			ferrum_qt.modes.base_mode.ModeId.ARROW, _NativeLineTool.DRAW_CURVED_EQUILIBRIUM_ARROW,
			"Draw Curved Equilibrium Arrow", self._on_toggle_draw_curved_equilibrium_arrow,
		)
		self._draw_plus_action = PySide6.QtGui.QAction(self.tr("Draw Plus"), self)
		self._draw_plus_action.setCheckable(True)
		self._draw_plus_action.setToolTip(self.tr("Click to place one Plus; Escape cancels without changing the document"))
		self._register_line_tool_action(
			"draw.plus", self._draw_plus_action, ferrum_qt.modes.base_mode.ModeId.VECTOR,
			_NativeLineTool.DRAW_PLUS, "Draw Plus", self._on_toggle_draw_plus,
		)
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
			self._draw_vector_actions[tool] = action
			self._register_line_tool_action(
				f"draw.vector.{tool.name.lower().removeprefix('draw_')}", action,
				ferrum_qt.modes.base_mode.ModeId.VECTOR, tool, label,
				lambda checked, tool=tool: self._on_toggle_draw_vector(tool, checked),
			)
		self._draw_path_actions = {}
		for tool, label in (
			(_NativeLineTool.DRAW_POLYLINE, "Draw Polyline"),
			(_NativeLineTool.DRAW_POLYGON, "Draw Polygon"),
		):
			action = PySide6.QtGui.QAction(self.tr(label), self)
			action.setCheckable(True)
			action.setToolTip(self.tr("Click ordered points, then press Enter or double-click to create one Rust-owned {0}; Esc cancels").format(label[5:].lower()))
			self._draw_path_actions[tool] = action
			self._register_line_tool_action(
				f"draw.path.{tool.name.lower().removeprefix('draw_')}", action,
				ferrum_qt.modes.base_mode.ModeId.VECTOR, tool, label,
				lambda checked, tool=tool: self._on_toggle_draw_path(tool, checked),
			)
		self._completion_click_actions = frozenset((
			*self._draw_path_actions,
			_NativeLineTool.DRAW_CURVED_ELECTRON_ARROW,
			_NativeLineTool.DRAW_CURVED_RETRO_ARROW,
			_NativeLineTool.DRAW_CURVED_REACTION_ARROW,
			_NativeLineTool.DRAW_CURVED_EQUILIBRIUM_ARROW,
		))
		self._insert_text_action = PySide6.QtGui.QAction(self.tr("Insert Text"), self)
		self._insert_text_action.setCheckable(True)
		self._insert_text_action.setToolTip(self.tr(
			"Insert Text: click a page location, enter text, then Save. Escape cancels.",
		))
		self._insert_text_action.setStatusTip(self.tr(
			"Insert Text: click a page location, enter text, then Save. Escape cancels.",
		))
		self._register_line_tool_action(
			"draw.text", self._insert_text_action, ferrum_qt.modes.base_mode.ModeId.EDIT,
			_NativeLineTool.INSERT_TEXT, "Insert Text", self._on_toggle_insert_text,
		)
		self._regular_ring_actions: dict[int, PySide6.QtGui.QAction] = {}
		for size, name in ferrum_qt.ferrum.regular_ring.REGULAR_RING_NAMES.items():
			action = PySide6.QtGui.QAction(self.tr("{0} (C{1})").format(name, size), self)
			action.setCheckable(True)
			instruction = self.tr(
				"Click an empty page location to insert one {0} ring; Escape cancels.",
			).format(name.lower())
			action.setToolTip(instruction)
			action.setStatusTip(instruction)
			self._regular_ring_actions[size] = action
			self._register_line_tool_action(
				f"draw.ring.regular.c{size}", action, ferrum_qt.modes.base_mode.ModeId.DRAW,
				_NativeLineTool.INSERT_REGULAR_RING, action.text(),
				lambda checked, size=size, action=action: self._on_toggle_insert_regular_ring(size, action, checked),
			)
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
		self._register_line_tool_action(
			"draw.ring.cyclohexane.insert", self._insert_cyclohexane_ring_action,
			ferrum_qt.modes.base_mode.ModeId.DRAW, _NativeLineTool.INSERT_REGULAR_RING,
			"Insert Cyclohexane Ring",
			lambda checked: self._on_toggle_insert_regular_ring(6, self._insert_cyclohexane_ring_action, checked),
		)
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
		self._register_line_tool_action(
			"draw.ring.cyclohexane.attach", self._attach_cyclohexane_ring_action,
			ferrum_qt.modes.base_mode.ModeId.DRAW, _NativeLineTool.ATTACH_CYCLOHEXANE_RING,
			"Attach Cyclohexane Ring", self._on_toggle_attach_cyclohexane_ring,
		)
		self._draw_wavy_action = PySide6.QtGui.QAction(self.tr("Draw Wavy Line"), self)
		self._draw_wavy_action.setCheckable(True)
		self._draw_wavy_action.setToolTip(
			self.tr("Drag between two page points; Esc cancels"),
		)
		self._register_line_tool_action(
			"draw.wavy", self._draw_wavy_action, ferrum_qt.modes.base_mode.ModeId.VECTOR,
			_NativeLineTool.CREATE_WAVY, "Draw Wavy Line", self._on_toggle_draw_wavy,
		)
		self._draw_bracket_action = PySide6.QtGui.QAction(
			self.tr("Draw Rectangular Bracket"), self,
		)
		self._draw_bracket_action.setCheckable(True)
		self._draw_bracket_action.setToolTip(
			self.tr("Drag a finite nonempty rectangle; Esc cancels"),
		)
		self._register_line_tool_action(
			"draw.bracket.rectangular", self._draw_bracket_action,
			ferrum_qt.modes.base_mode.ModeId.BRACKET,
			_NativeLineTool.CREATE_RECTANGULAR_BRACKET, "Draw Rectangular Bracket",
			self._on_toggle_draw_bracket,
		)
		self._draw_round_bracket_action = PySide6.QtGui.QAction(
			self.tr("Draw Round Bracket"), self,
		)
		self._draw_round_bracket_action.setCheckable(True)
		self._draw_round_bracket_action.setToolTip(
			self.tr("Drag a finite nonempty rectangle; Esc cancels"),
		)
		self._register_line_tool_action(
			"draw.bracket.round", self._draw_round_bracket_action,
			ferrum_qt.modes.base_mode.ModeId.BRACKET,
			_NativeLineTool.CREATE_ROUND_BRACKET, "Draw Round Bracket",
			self._on_toggle_draw_round_bracket,
		)
		self._move_atom_action = PySide6.QtGui.QAction(self.tr("Move Atom"), self)
		self._move_atom_action.setCheckable(True)
		self._move_atom_action.setToolTip(
			self.tr("Drag one existing atom to an exact new scene point; Esc cancels"),
		)
		self._register_line_tool_action(
			"draw.arrange.move_atom", self._move_atom_action,
			ferrum_qt.modes.base_mode.ModeId.EDIT, _NativeLineTool.MOVE_ATOM,
			"Move Atom", self._on_toggle_move_atom,
		)
		self._rotate_atoms_action = PySide6.QtGui.QAction(
			self.tr("Rotate Selected Atoms"), self,
		)
		self._rotate_atoms_action.setCheckable(True)
		self._rotate_atoms_action.setToolTip(
			self.tr(
				"Drag around the selected atoms' center; Esc cancels without changing Rust",
			),
		)
		self._register_line_tool_action(
			"draw.arrange.rotate_selected_atoms", self._rotate_atoms_action,
			ferrum_qt.modes.base_mode.ModeId.EDIT, _NativeLineTool.ROTATE_ATOMS,
			"Rotate Selected Atoms", self._on_toggle_rotate_atoms,
		)
		self._translate_roots_action = PySide6.QtGui.QAction(
			self.tr("Move Complete Roots"), self,
		)
		self._translate_roots_action.setCheckable(True)
		self._translate_roots_action.setToolTip(
			self.tr(
				"Drag selected complete roots; the View snap setting applies; Esc cancels",
			),
		)
		self._register_line_tool_action(
			"draw.arrange.move_complete_roots", self._translate_roots_action,
			ferrum_qt.modes.base_mode.ModeId.EDIT, _NativeLineTool.TRANSLATE_ROOTS,
			"Move Complete Roots", self._on_toggle_translate_roots,
		)
		self._cancel_tool_action = PySide6.QtGui.QAction(self.tr("Cancel Tool"), self)
		self._cancel_tool_action.setShortcut(PySide6.QtGui.QKeySequence.StandardKey.Cancel)
		self._cancel_tool_action.setToolTip(self.tr(
			"Cancel the active editing tool; selection and document stay unchanged",
		))
		self._cancel_tool_action.triggered.connect(self._on_cancel_tool)
		self._cancel_tool_action.setEnabled(False)
		self._register_action("tool.cancel", self._cancel_tool_action,
			lifecycle="stateful-cancel")

	#============================================
	def _register_line_tool_action(self, action_id: str, action: PySide6.QtGui.QAction,
			mode_id: ferrum_qt.modes.base_mode.ModeId, tool: _NativeLineTool,
			status_label: str, activation: object) -> None:
		"""Register and bind one line QAction beside its feature construction."""
		self._register_action(action_id, action)
		self._register_line_tool_mode_binding(action, mode_id, tool, status_label, activation)

	#============================================
	def _register_line_tool_mode_binding(self, action: PySide6.QtGui.QAction,
			mode_id: ferrum_qt.modes.base_mode.ModeId, tool: _NativeLineTool,
			status_label: str, activation: object) -> None:
		"""Bind one already-registered line QAction to its feature-owned mode."""
		if not callable(activation):
			raise TypeError("Ferrum line-tool activation must be callable.")
		binding = ferrum_qt.ferrum.window_mode_sync.FerrumWindowToolBinding(
			action, mode_id, ferrum_qt.ferrum.line_tool_mode.LineToolMode(mode_id, tool),
			status_label, True, self._mode_context,
			lambda _context, callback=activation: callback(True),
			self._dispatch_line_mode_intent,
			lambda _context: self._cancel_line_gesture(),
		)
		self._window_mode_sync.register_tool(binding)

	def _refresh_line_tool_actions(self, enabled: bool) -> None:
		"""Apply the host's authoritative action policy to both pointer tools."""
		self._draw_bond_action.setEnabled(enabled)
		self._draw_solid_wedge_bond_action.setEnabled(enabled)
		self._draw_hashed_wedge_bond_action.setEnabled(enabled)
		self._draw_arrow_action.setEnabled(enabled)
		self._draw_equilibrium_arrow_action.setEnabled(enabled)
		self._draw_curved_electron_arrow_action.setEnabled(enabled)
		self._draw_curved_retro_arrow_action.setEnabled(enabled)
		self._draw_curved_reaction_arrow_action.setEnabled(enabled)
		self._draw_curved_equilibrium_arrow_action.setEnabled(enabled)
		self._draw_plus_action.setEnabled(enabled)
		for action in self._draw_vector_actions.values():
			action.setEnabled(enabled)
		for action in self._draw_path_actions.values():
			action.setEnabled(enabled)
		self._insert_text_action.setEnabled(enabled)
		for action in self._regular_ring_actions.values():
			action.setEnabled(enabled)
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
		self._activate_line_tool(
			_NativeLineTool.DRAW_BOND,
			ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation.NORMAL,
		)

	#============================================
	def _on_toggle_draw_solid_wedge_bond(self, checked: bool) -> None:
		"""Enter or leave the typed solid-wedge direct-bond authoring mode."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(
			_NativeLineTool.DRAW_BOND,
			ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation.SOLID_WEDGE,
		)

	#============================================
	def _on_toggle_draw_hashed_wedge_bond(self, checked: bool) -> None:
		"""Enter or leave the typed hashed-wedge direct-bond authoring mode."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(
			_NativeLineTool.DRAW_BOND,
			ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation.HASHED_WEDGE,
		)

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
	def _on_toggle_draw_curved_electron_arrow(self, checked: bool) -> None:
		"""Enter or leave Rust-owned three-point curved electron-arrow creation."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_CURVED_ELECTRON_ARROW)

	#============================================
	def _on_toggle_draw_curved_retro_arrow(self, checked: bool) -> None:
		"""Enter or leave Rust-owned three-point curved retro-arrow creation."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_CURVED_RETRO_ARROW)

	#============================================
	def _on_toggle_draw_curved_reaction_arrow(self, checked: bool) -> None:
		"""Enter or leave Rust-owned three-point curved reaction-arrow creation."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_CURVED_REACTION_ARROW)

	#============================================
	def _on_toggle_draw_curved_equilibrium_arrow(self, checked: bool) -> None:
		"""Enter or leave Rust-owned three-point curved-equilibrium creation."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(_NativeLineTool.DRAW_CURVED_EQUILIBRIUM_ARROW)

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
	def _on_toggle_draw_path(self, tool: _NativeLineTool, checked: bool) -> None:
		"""Enter or leave one renderer-preflighted multi-point path tool."""
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

	def _on_toggle_insert_regular_ring(self, size: int,
			action: PySide6.QtGui.QAction, checked: bool) -> None:
		"""Arm one direct, detached, Rust-owned placement from the closed family."""
		if not checked:
			self._cancel_line_gesture()
			return
		self._activate_line_tool(
			_NativeLineTool.INSERT_REGULAR_RING,
			regular_ring_size=size,
			regular_ring_action=action,
		)

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
	def _activate_line_tool(self, tool: _NativeLineTool,
			direct_bond_presentation: (
				ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation
				| None
			) = None,
			regular_ring_size: int | None = None,
			regular_ring_action: PySide6.QtGui.QAction | None = None) -> None:
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
			if direct_bond_presentation is (
				ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation.SOLID_WEDGE
			):
				action = self._draw_solid_wedge_bond_action
			elif direct_bond_presentation is (
				ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation.HASHED_WEDGE
			):
				action = self._draw_hashed_wedge_bond_action
			else:
				action = self._draw_bond_action
		elif tool is _NativeLineTool.DRAW_ARROW:
			action = self._draw_arrow_action
		elif tool is _NativeLineTool.DRAW_EQUILIBRIUM_ARROW:
			action = self._draw_equilibrium_arrow_action
		elif tool is _NativeLineTool.DRAW_CURVED_ELECTRON_ARROW:
			action = self._draw_curved_electron_arrow_action
		elif tool is _NativeLineTool.DRAW_CURVED_RETRO_ARROW:
			action = self._draw_curved_retro_arrow_action
		elif tool is _NativeLineTool.DRAW_CURVED_REACTION_ARROW:
			action = self._draw_curved_reaction_arrow_action
		elif tool is _NativeLineTool.DRAW_CURVED_EQUILIBRIUM_ARROW:
			action = self._draw_curved_equilibrium_arrow_action
		elif tool is _NativeLineTool.DRAW_PLUS:
			action = self._draw_plus_action
		elif tool in self._draw_vector_actions:
			action = self._draw_vector_actions[tool]
		elif tool in self._draw_path_actions:
			action = self._draw_path_actions[tool]
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
		elif tool is _NativeLineTool.INSERT_REGULAR_RING:
			if regular_ring_action is None:
				raise ValueError("A regular-ring action must choose the admitted ring size.")
			ferrum_qt.ferrum.regular_ring.display_name(regular_ring_size)
			action = regular_ring_action
		elif tool is _NativeLineTool.ATTACH_CYCLOHEXANE_RING:
			action = self._attach_cyclohexane_ring_action
		else:
			raise ValueError(f"Unhandled native line tool: {tool!r}")
		action.setChecked(True)
		snapshot = tab.current_snapshot
		viewport = tab.view.viewport()
		presentation = (
			ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation.NORMAL
			if direct_bond_presentation is None else direct_bond_presentation
		)
		self._line_gesture_intent = _LineGestureIntent(
			tab, viewport, snapshot.revision, snapshot.digest, tool,
			direct_bond_presentation=presentation,
			regular_ring_size=regular_ring_size,
			regular_ring_action=regular_ring_action,
		)
		self._synchronize_mode_state()
		self._refresh_cancel_tool_action()
		self._refresh_actions()
		viewport.setFocus()
		self._restore_line_tool_focus_on_next_turn(self._line_gesture_intent)
		tab.view.show_keyboard_cursor()
		if tool is _NativeLineTool.DRAW_BOND:
			drawing = self._drawing_parameters.snapshot()
			message = self._draw_bond_feedback(drawing, presentation)
			self._draw_bond_action.setToolTip(message)
		elif tool is _NativeLineTool.DRAW_ARROW:
			message = self.tr("Draw Arrow: drag a straight normal reaction arrow; Esc cancels.")
		elif tool is _NativeLineTool.DRAW_EQUILIBRIUM_ARROW:
			message = self.tr("Draw Equilibrium Arrow: drag a straight equilibrium reaction arrow; Esc cancels.")
		elif tool is _NativeLineTool.DRAW_CURVED_ELECTRON_ARROW:
			message = self.tr("Draw Curved Electron Arrow: click start, bend, and endpoint; Esc cancels.")
		elif tool is _NativeLineTool.DRAW_CURVED_RETRO_ARROW:
			message = self.tr("Draw Curved Retro Arrow: click start, bend, and endpoint; Esc cancels.")
		elif tool is _NativeLineTool.DRAW_CURVED_EQUILIBRIUM_ARROW:
			message = self.tr("Draw Curved Equilibrium Arrow: click start, bend, and endpoint; Esc cancels.")
		elif tool is _NativeLineTool.DRAW_PLUS:
			message = self.tr("Draw Plus: click once to place a Plus; Esc cancels.")
		elif tool in self._draw_vector_actions:
			message = self.tr("{0}: drag to create one renderer-preflighted shape; Esc cancels.").format(
				self._draw_vector_actions[tool].text(),
			)
		elif tool in self._draw_path_actions:
			message = self.tr("{0}: click points, then press Enter or double-click to commit; Esc cancels.").format(self._draw_path_actions[tool].text())
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
		elif tool is _NativeLineTool.INSERT_REGULAR_RING:
			assert regular_ring_size is not None
			message = self.tr(
				"Insert {0}: click an empty page location; Escape cancels.",
			).format(ferrum_qt.ferrum.regular_ring.display_name(regular_ring_size))
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
			presentation: ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation,
			) -> str:
		"""Name the frozen Draw Bond contract in human wording."""
		if presentation is not ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation.NORMAL:
			return self.tr(
				"Draw {0} Bond: drag from stereo tip to stereo base; Esc cancels."
			).format(presentation.description().title())
		return self.tr(
			"Draw Bond: Normal {0}; drag between atoms or empty canvas locations. "
			"Shift+Arrow is fine movement; Esc cancels."
		).format(drawing.order_name)

	#============================================
