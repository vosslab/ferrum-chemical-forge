"""Portable action-ID registry bound to Ferrum-owned Qt actions."""

# Standard Library
import dataclasses
import re

# PIP3 modules
import PySide6.QtGui


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class MenuAction:
	"""One portable command declaration shared with menu and keybinding clients."""

	id: str
	label_key: str
	help_key: str
	accelerator: str | None
	handler: object
	enabled_when: object
	shortcut_exemption_reason: str | None = None
	lifecycle: str = "static"

	@property
	def label(self) -> str:
		"""Return the untranslated source label used by the Qt action."""
		return self.label_key

	@property
	def help_text(self) -> str:
		"""Return the untranslated source help text."""
		return self.help_key


#============================================
class ActionRegistry:
	"""Store portable action declarations and their live Qt clients."""

	#============================================
	def __init__(self) -> None:
		"""Create an empty registry."""
		self._actions: dict[str, MenuAction] = {}
		self._qt_actions: dict[str, PySide6.QtGui.QAction] = {}
		self._dynamic_lifecycles: dict[str, str] = {}

	#============================================
	def register(self, action: MenuAction) -> None:
		"""Register one declaration, rejecting duplicate action IDs."""
		if action.id in self._actions:
			raise ValueError(f"Duplicate action ID: '{action.id}'")
		if not action.label.strip() or not action.help_text.strip():
			raise ValueError(f"Ferrum action '{action.id}' needs text and help.")
		if action.accelerator is None and not action.shortcut_exemption_reason:
			raise ValueError(
			f"Ferrum action '{action.id}' needs a shortcut or exemption reason.",
		)
		self._actions[action.id] = action

	#============================================
	def bind_qt_action(
			self, action_id: str, qt_action: PySide6.QtGui.QAction,
			) -> None:
		"""Bind one declared ID to its existing Rust-backed QAction."""
		if action_id not in self._actions:
			raise KeyError(action_id)
		if action_id in self._qt_actions:
			raise ValueError(f"Duplicate Qt action binding: '{action_id}'")
		self._qt_actions[action_id] = qt_action
		qt_action.setObjectName(action_id)
		declaration = self._actions[action_id]
		if not qt_action.text().strip():
			qt_action.setText(declaration.label)
		if not qt_action.toolTip().strip():
			qt_action.setToolTip(declaration.help_text)
		if not qt_action.statusTip().strip():
			qt_action.setStatusTip(declaration.help_text)
		if not qt_action.whatsThis().strip():
			qt_action.setWhatsThis(declaration.help_text)

	#============================================
	def declare_dynamic_lifecycle(self, owner_id: str, reason: str) -> None:
		"""Record why one feature creates ephemeral actions at runtime.

		Dynamic entries are not retained in the static registry because their
		labels and callbacks are deliberately rebuilt from current presentation
		state.  Their owner must nevertheless make that lifecycle auditable.
		"""
		if not owner_id or not reason.strip():
			raise ValueError("Dynamic action lifecycles need an ID and reason.")
		if owner_id in self._dynamic_lifecycles:
			raise ValueError(f"Duplicate dynamic action lifecycle: '{owner_id}'.")
		self._dynamic_lifecycles[owner_id] = reason

	#============================================
	def dynamic_lifecycles(self) -> dict[str, str]:
		"""Return declared lifecycle reasons for ephemeral action families."""
		return dict(self._dynamic_lifecycles)

	#============================================
	def get(self, action_id: str) -> MenuAction:
		"""Return one declaration by stable dotted ID."""
		return self._actions[action_id]

	#============================================
	def get_qt_action(self, action_id: str) -> PySide6.QtGui.QAction | None:
		"""Return the live Qt client for an ID when the window supplies one."""
		return self._qt_actions.get(action_id)

	#============================================
	def __contains__(self, action_id: str) -> bool:
		"""Return whether the dotted ID is declared."""
		return action_id in self._actions

	#============================================
	def all_actions(self) -> dict[str, MenuAction]:
		"""Return a shallow declaration snapshot."""
		return dict(self._actions)

	#============================================
	def is_enabled(self, action_id: str, context: object) -> bool:
		"""Evaluate one declaration's optional enablement predicate."""
		predicate = self._actions[action_id].enabled_when
		if predicate is None:
			return True
		if callable(predicate):
			return bool(predicate())
		return bool(getattr(context, predicate, False))


#============================================
def _attribute_action_paths(value: object, prefix: str = "") -> dict[int, str]:
	"""Return bounded attribute paths for QActions held by the window."""
	paths: dict[int, str] = {}
	if isinstance(value, PySide6.QtGui.QAction):
		paths[id(value)] = prefix
	elif type(value) is dict:
		for key, item in value.items():
			if type(key) is str:
				paths.update(_attribute_action_paths(item, f"{prefix}.{key}"))
	elif type(value) in {list, tuple}:
		for index, item in enumerate(value):
			paths.update(_attribute_action_paths(item, f"{prefix}.{index}"))
	return paths


#============================================
def _fallback_action_id(attribute_path: str, action: PySide6.QtGui.QAction) -> str:
	"""Make one deterministic command ID for an already-owned static action."""
	if attribute_path:
		source = attribute_path.strip("_").replace("_", "-").replace(".", ".")
	else:
		source = action.text().lower().replace("&", "")
	source = re.sub(r"[^a-z0-9.]+", "-", source).strip("-.")
	return f"command.{source or 'unnamed'}"


#============================================
def _action_lifecycle(action: PySide6.QtGui.QAction) -> str:
	"""Classify static actions whose availability follows transient state."""
	text = action.text().lower()
	if text.startswith("cancel"):
		return "stateful-cancel"
	if text.startswith(("show ", "hide ", "toggle ")):
		return "stateful-visibility"
	return "static"


#============================================
def register_main_window_actions(window: object) -> ActionRegistry:
	"""Declare and bind the shared command IDs supplied by a Ferrum window."""
	registry = ActionRegistry()
	declarations = (
		("file.new", "_action_new", "New", "Create a new Ferrum document"),
		("file.open", "_open_action", "Open", "Open a CDML document"),
		("file.save", "_save_action", "Save", "Save the current document"),
		("file.save_as", "_save_as_action", "Save As", "Save to a new CDML path"),
		("file.close", "_close_action", "Close Tab", "Close the current document"),
		("file.quit", "_quit_action", "Quit", "Quit Ferrum"),
		("edit.undo", "_undo_action", "Undo", "Undo the last document change"),
		("edit.redo", "_redo_action", "Redo", "Redo the last undone change"),
		("edit.cut", "_cut_action", "Cut", "Cut the selected document roots"),
		("edit.copy", "_copy_action", "Copy", "Copy the selected document roots"),
		("edit.paste", "_paste_action", "Paste", "Paste Ferrum CDML content"),
		("view.zoom_in", "_zoom_in_action", "Zoom In", "Increase canvas zoom"),
		("view.zoom_out", "_zoom_out_action", "Zoom Out", "Decrease canvas zoom"),
		("view.reset_zoom", "_zoom_100_action", "Zoom to 100%", "Reset canvas zoom"),
		("view.zoom_page", "_zoom_page_action", "Zoom to Page", "Fit the active page"),
		(
			"view.zoom_content", "_zoom_content_action", "Zoom to Content",
			"Fit active document content",
		),
		("view.toggle_grid", "_show_hex_grid_action", "Show Hex Grid", "Toggle the grid"),
		(
			"view.toggle_grid_snap", "_snap_hex_grid_action", "Snap to Hex Grid",
			"Toggle grid snapping",
		),
		("mode.atom", "_add_atom_action", "Add Atom", "Activate atom drawing"),
		("mode.draw", "_draw_bond_action", "Draw Bond", "Activate bond drawing"),
		(
			"mode.draw_solid_wedge", "_draw_solid_wedge_bond_action",
			"Draw Solid Wedge Bond", "Activate solid-wedge bond drawing",
		),
		(
			"mode.draw_hashed_wedge", "_draw_hashed_wedge_bond_action",
			"Draw Hashed Wedge Bond", "Activate hashed-wedge bond drawing",
		),
		("mode.bracket", "_draw_bracket_action", "Draw Bracket", "Activate bracket drawing"),
		("mode.edit", "_move_atom_action", "Move Atom", "Activate atom movement"),
		(
			"edit.atom_properties", "_edit_atom_properties_action", "Edit Atom Properties",
			"Edit the selected atom",
		),
		(
			"edit.bond_properties", "_edit_bond_properties_action", "Edit Bond Properties",
			"Edit the selected bond",
		),
		("tool.cancel", "_cancel_tool_action", "Cancel Tool", "Cancel the active tool"),
		(
			"options.preferences", "_preferences_action", "Preferences",
			"Choose Ferrum settings",
		),
		("help.about", "_about_action", "About Ferrum", "Show Ferrum information"),
	)
	for action_id, attribute, label, help_text in declarations:
		qt_action = getattr(window, attribute, None)
		if not isinstance(qt_action, PySide6.QtGui.QAction):
			continue
		declaration = MenuAction(
			action_id, label, help_text, None, qt_action.trigger, qt_action.isEnabled,
			"Standard product command without a portable default shortcut.",
		)
		registry.register(declaration)
		registry.bind_qt_action(action_id, qt_action)
	attribute_paths: dict[int, str] = {}
	for attribute, value in vars(window).items():
		attribute_paths.update(_attribute_action_paths(value, attribute))
	direct_line_tool_actions = {
		getattr(window, "_attach_cyclohexane_ring_action", None),
	}
	for qt_action in window.findChildren(PySide6.QtGui.QAction):
		if (
			qt_action.parent() is not window
			or qt_action in registry._qt_actions.values()
			or qt_action in direct_line_tool_actions
		):
			continue
		action_id = _fallback_action_id(attribute_paths.get(id(qt_action), ""), qt_action)
		base_action_id = action_id
		index = 2
		while action_id in registry:
			action_id = f"{base_action_id}.{index}"
			index += 1
		label = qt_action.text().replace("&", "").strip() or action_id
		help_text = qt_action.toolTip().strip() or label
		registry.register(MenuAction(
			action_id, label, help_text, None, qt_action.trigger, qt_action.isEnabled,
			(
				"No portable default shortcut; this command is available by its "
				"labelled menu or toolbar client."
			),
			_action_lifecycle(qt_action),
		))
		registry.bind_qt_action(action_id, qt_action)
	return registry
