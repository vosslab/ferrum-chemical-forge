"""Central keyboard shortcut policy for Ferrum actions."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.config.preferences

STANDARD_KEYBINDINGS = {
	"file.new": PySide6.QtGui.QKeySequence.StandardKey.New,
	"file.open": PySide6.QtGui.QKeySequence.StandardKey.Open,
	"file.save": PySide6.QtGui.QKeySequence.StandardKey.Save,
	"file.save_as": PySide6.QtGui.QKeySequence.StandardKey.SaveAs,
	"file.close": PySide6.QtGui.QKeySequence.StandardKey.Close,
	"file.quit": PySide6.QtGui.QKeySequence.StandardKey.Quit,
	"edit.undo": PySide6.QtGui.QKeySequence.StandardKey.Undo,
	"edit.redo": PySide6.QtGui.QKeySequence.StandardKey.Redo,
	"edit.cut": PySide6.QtGui.QKeySequence.StandardKey.Cut,
	"edit.copy": PySide6.QtGui.QKeySequence.StandardKey.Copy,
	"edit.paste": PySide6.QtGui.QKeySequence.StandardKey.Paste,
}

DEFAULT_KEYBINDINGS = {
	"view.zoom_in": "Ctrl++",
	"view.zoom_out": "Ctrl+-",
	"view.zoom_100": "Ctrl+0",
	"view.grid.visible": "Ctrl+G",
	"view.grid.snap": "Ctrl+Shift+G",
	"draw.atom_at_point": "Ctrl+8",
	"draw.bond": "Ctrl+2",
	"tool.cancel": "Esc",
}

_SETTINGS_PREFIX = "keybindings/"
SHORTCUT_EXEMPT_ACTIONS = frozenset({"help.about", "options.preferences"})


#============================================
class KeybindingConflictError(ValueError):
	"""Report two active commands configured for the same key sequence."""


#============================================
class KeybindingRegistrationError(ValueError):
	"""Report a shortcut with no registered QAction target."""


#============================================
class KeybindingManager(PySide6.QtCore.QObject):
	"""Apply portable shortcut policy to the window's registered actions."""

	#============================================
	def __init__(self, window: object, registry: object) -> None:
		"""Load the default and persisted bindings for one window."""
		super().__init__(window)
		self._registry = registry
		self._bindings = self.default_bindings()
		self._load_from_preferences()

	#============================================
	def default_bindings(self) -> dict[str, str]:
		"""Return portable text for every standard and Ferrum-specific command."""
		bindings = {}
		for action_id, standard_key in STANDARD_KEYBINDINGS.items():
			sequence = PySide6.QtGui.QKeySequence(standard_key)
			bindings[action_id] = sequence.toString(
				PySide6.QtGui.QKeySequence.SequenceFormat.PortableText,
			)
		bindings.update(DEFAULT_KEYBINDINGS)
		return bindings

	#============================================
	def _load_from_preferences(self) -> None:
		"""Overlay saved portable sequences without inventing missing actions."""
		prefs = ferrum_qt.config.preferences.Preferences.instance()
		for action_id in self._bindings:
			saved = prefs.value(_SETTINGS_PREFIX + action_id)
			if type(saved) is str:
				self._bindings[action_id] = saved

	#============================================
	@staticmethod
	def validate_binding_map(bindings: dict[str, str]) -> None:
		"""Reject invalid or conflicting nonempty shortcut sequences."""
		owners: dict[str, list[str]] = {}
		for action_id, text in bindings.items():
			if not text:
				continue
			sequence = PySide6.QtGui.QKeySequence(text)
			normalized = sequence.toString(
				PySide6.QtGui.QKeySequence.SequenceFormat.PortableText,
			)
			if not normalized:
				raise KeybindingRegistrationError(
					f"Invalid key sequence for '{action_id}': {text!r}",
				)
			owners.setdefault(normalized, []).append(action_id)
		conflicts = {
			sequence: action_ids
			for sequence, action_ids in owners.items()
			if len(action_ids) > 1
		}
		if conflicts:
			details = "; ".join(
				f"{sequence}: {', '.join(action_ids)}"
				for sequence, action_ids in sorted(conflicts.items())
			)
			raise KeybindingConflictError(
				f"Conflicting Ferrum keyboard shortcuts: {details}",
			)

	#============================================
	def setup_shortcuts(self) -> None:
		"""Apply each binding to its existing QAction client."""
		self.validate_binding_map(self._bindings)
		for action_id, text in self._bindings.items():
			action = self._registry.get_qt_action(action_id)
			if action is None:
				raise KeybindingRegistrationError(
					f"No Ferrum action is registered for '{action_id}'.",
				)
			action.setShortcut(PySide6.QtGui.QKeySequence(text))
			action.setShortcutContext(PySide6.QtCore.Qt.ShortcutContext.WindowShortcut)

	#============================================
	def set_binding(self, action_id: str, text: str) -> None:
		"""Persist and apply one user-selected portable shortcut sequence."""
		if action_id not in self._bindings:
			raise KeybindingRegistrationError(
				f"No default Ferrum keybinding is declared for '{action_id}'.",
			)
		if type(text) is not str:
			raise KeybindingRegistrationError("Ferrum shortcuts must be text.")
		candidate = self.get_all_bindings()
		candidate[action_id] = text
		self.validate_binding_map(candidate)
		self._bindings = candidate
		prefs = ferrum_qt.config.preferences.Preferences.instance()
		prefs.set_value(_SETTINGS_PREFIX + action_id, text)
		self.setup_shortcuts()

	#============================================
	def reset_defaults(self) -> None:
		"""Remove all saved overrides and restore Ferrum's shipped shortcuts."""
		prefs = ferrum_qt.config.preferences.Preferences.instance()
		for action_id in self._bindings:
			prefs.remove_value(_SETTINGS_PREFIX + action_id)
		self._bindings = self.default_bindings()
		self.setup_shortcuts()

	#============================================
	def get_binding(self, action_id: str) -> str:
		"""Return the current portable sequence for one command."""
		return self._bindings[action_id]

	#============================================
	def get_all_bindings(self) -> dict[str, str]:
		"""Return an isolated snapshot of the current shortcut map."""
		return dict(self._bindings)
