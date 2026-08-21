"""Focused tests for Ferrum's shared action and keyboard seams."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.menu_builder
import ferrum_qt.actions.platform_menu
import ferrum_qt.config.keybindings
import ferrum_qt.config.preferences

_ACTION_ATTRIBUTES = (
	"_action_new", "_open_action", "_save_action", "_save_as_action",
	"_close_action", "_quit_action", "_undo_action", "_redo_action",
	"_cut_action", "_copy_action", "_paste_action", "_zoom_in_action",
	"_zoom_out_action", "_zoom_100_action", "_show_hex_grid_action",
	"_snap_hex_grid_action", "_add_atom_action", "_draw_bond_action",
	"_attach_cyclohexane_ring_action",
	"_cancel_tool_action", "_preferences_action", "_about_action",
)


#============================================
class _Preferences:
	"""In-memory preference source with no local machine state."""

	#============================================
	def value(self, key: str) -> None:
		"""Return no saved override for any shortcut key."""
		return self.values.get(key)

	#============================================
	def __init__(self) -> None:
		"""Keep shortcut overrides in memory for one focused test."""
		self.values: dict[str, str] = {}

	#============================================
	def set_value(self, key: str, value: str) -> None:
		"""Remember one persisted shortcut override."""
		self.values[key] = value

	#============================================
	def remove_value(self, key: str) -> None:
		"""Forget one persisted shortcut override."""
		self.values.pop(key, None)


#============================================
def _window_with_actions() -> PySide6.QtWidgets.QMainWindow:
	"""Return a minimal window supplying the product's shared QActions."""
	window = PySide6.QtWidgets.QMainWindow()
	for attribute in _ACTION_ATTRIBUTES:
		setattr(window, attribute, PySide6.QtGui.QAction(attribute, window))
	return window


#============================================
def test_registry_binds_stable_action_ids_to_existing_qactions(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Portable action IDs name the live commands already owned by the window."""
	del qapp
	window = _window_with_actions()
	window.setMenuBar(PySide6.QtWidgets.QMenuBar(window))
	registry = ferrum_qt.actions.action_registry.register_main_window_actions(window)
	assert set(registry.all_actions()) == {
		"file.new", "file.open", "file.save", "file.save_as", "file.close",
		"file.quit", "edit.undo", "edit.redo", "edit.cut", "edit.copy",
		"edit.paste", "view.zoom_in", "view.zoom_out", "view.reset_zoom",
		"view.toggle_grid", "view.toggle_grid_snap", "mode.atom", "mode.draw",
		"tool.cancel", "options.preferences", "help.about",
	}
	assert registry.get_qt_action("file.save") is window._save_action
	assert window._save_action.objectName() == "file.save"
	window.deleteLater()


#============================================
def test_keybinding_manager_applies_standard_and_ferrum_shortcuts(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Every declared workflow command receives one conflict-free shortcut."""
	del qapp
	monkeypatch.setattr(
		ferrum_qt.config.preferences.Preferences,
		"instance", classmethod(lambda cls: _Preferences()),
	)
	window = _window_with_actions()
	registry = ferrum_qt.actions.action_registry.register_main_window_actions(window)
	manager = ferrum_qt.config.keybindings.KeybindingManager(window, registry)
	manager.setup_shortcuts()
	assert window._save_action.shortcut().matches(
		PySide6.QtGui.QKeySequence(PySide6.QtGui.QKeySequence.StandardKey.Save),
	) is PySide6.QtGui.QKeySequence.SequenceMatch.ExactMatch
	assert manager.get_binding("tool.cancel") == "Esc"
	for action_id in registry.all_actions():
		action = registry.get_qt_action(action_id)
		assert (
			not action.shortcut().isEmpty()
			or registry.get(action_id).shortcut_exemption_reason
		)
	window.deleteLater()


#============================================
def test_duplicate_keyboard_shortcuts_are_rejected() -> None:
	"""Conflict validation reports both commands before modifying QActions."""
	with pytest.raises(
			ferrum_qt.config.keybindings.KeybindingConflictError,
			match="file.save.*edit.undo",
			):
		ferrum_qt.config.keybindings.KeybindingManager.validate_binding_map({
			"file.save": "Ctrl+S",
			"edit.undo": "Ctrl+S",
		})


#============================================
def test_keybindings_persist_set_and_reset(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A user override takes effect immediately and reset removes its storage."""
	del qapp
	prefs = _Preferences()
	monkeypatch.setattr(
		ferrum_qt.config.preferences.Preferences,
		"instance", classmethod(lambda cls: prefs),
	)
	window = _window_with_actions()
	registry = ferrum_qt.actions.action_registry.register_main_window_actions(window)
	manager = ferrum_qt.config.keybindings.KeybindingManager(window, registry)
	manager.set_binding("mode.atom", "Ctrl+Alt+A")
	assert manager.get_binding("mode.atom") == "Ctrl+Alt+A"
	assert prefs.values["keybindings/mode.atom"] == "Ctrl+Alt+A"
	manager.reset_defaults()
	assert manager.get_binding("mode.atom") == "Ctrl+8"
	assert "keybindings/mode.atom" not in prefs.values
	window.deleteLater()


#============================================
def test_menu_builder_and_platform_roles_reuse_registered_actions(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Declared menus only receive the existing registry action instances."""
	del qapp
	window = _window_with_actions()
	window.setMenuBar(PySide6.QtWidgets.QMenuBar(window))
	registry = ferrum_qt.actions.action_registry.register_main_window_actions(window)
	menus = ferrum_qt.actions.menu_builder.build_declared_menus(window, registry)
	assert registry.get_qt_action("file.save") in menus["file"].actions()
	assert registry.get_qt_action("help.about") in menus["help"].actions()
	ferrum_qt.actions.platform_menu.apply_platform_menu_roles(registry)
	assert registry.get_qt_action("file.quit").menuRole() is (
		PySide6.QtGui.QAction.MenuRole.QuitRole
	)
	window.deleteLater()


#============================================
def test_registry_records_stateful_and_dynamic_action_lifecycles(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Transient cancellation and regenerated menu families remain auditable."""
	del qapp
	window = _window_with_actions()
	window._cancel_coordinates_action = PySide6.QtGui.QAction("Cancel Coordinates", window)
	registry = ferrum_qt.actions.action_registry.register_main_window_actions(window)
	stateful = next(
		action for action in registry.all_actions().values()
		if action.lifecycle == "stateful-cancel"
	)
	assert stateful.shortcut_exemption_reason
	registry.declare_dynamic_lifecycle("recent-files", "Entries rebuild from preferences.")
	assert registry.dynamic_lifecycles()["recent-files"].startswith("Entries rebuild")
	window.deleteLater()
