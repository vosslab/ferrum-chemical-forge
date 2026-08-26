"""Focused tests for Ferrum's action registry and declarative menu builder."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.menu_builder
import ferrum_qt.actions.platform_menu
import ferrum_qt.declarative_resources


#============================================
def _declarations() -> dict:
	"""Return a minimal hierarchy that exercises all declared menu node forms."""
	return {"menus": [
		{
			"id": "file", "label_key": "File", "help_key": "Document commands",
			"items": [{"section": {
				"id": "document", "label_key": "Document",
				"items": [{"action": "file.new"}, {"dynamic_menu": "file.recent"}],
			}}],
		},
		{
			"id": "draw", "label_key": "Draw",
			"help_key": "Canvas authoring commands",
			"items": [
				{"section": {
					"id": "bonds", "label_key": "Bonds",
					"items": [{"action": "draw.bond"}],
				}},
				{"section": {
					"id": "rings", "label_key": "Rings",
					"items": [{"submenu": {
						"id": "draw.regular_ring",
						"label_key": "Insert Regular Ring...",
						"help_key": "Insert a regular carbon ring",
						"items": [{"action": "draw.ring.regular.c6"}],
					}}],
				}},
			],
		},
	]}


#============================================
def _registered_action(
		registry: ferrum_qt.actions.action_registry.ActionRegistry,
		action_id: str, text: str, parent: PySide6.QtWidgets.QWidget,
		) -> PySide6.QtGui.QAction:
	"""Create one feature-owned action and bind it through its stable identity."""
	action = PySide6.QtGui.QAction(text, parent)
	action.setToolTip(f"Use {text}")
	registry.register_existing(
		action_id, action,
		shortcut_exemption_reason="The command is reachable by its labelled menu.",
	)
	return action


#============================================
def test_registry_preserves_existing_qaction_identity_and_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Registry binding retains the feature object's Qt-facing contract verbatim."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	action = PySide6.QtGui.QAction("Draw Equilibrium Arrow", window)
	action.setObjectName("drawEquilibriumArrowAction")
	action.setToolTip("Draw a reversible reaction arrow")
	action.setShortcut("Ctrl+E")
	action.setCheckable(True)
	action.setChecked(True)
	action.setEnabled(False)
	triggered: list[bool] = []
	def record_trigger() -> None:
		"""Record the zero-argument PySide signal delivery."""
		triggered.append(True)
	action.triggered.connect(record_trigger)
	registry.register_existing(
		"draw.arrow.equilibrium", action,
		shortcut_exemption_reason="The tool uses a feature-owned shortcut.",
	)
	assert registry.get_qt_action("draw.arrow.equilibrium") is action
	assert action.shortcut().toString() == "Ctrl+E"
	assert action.isCheckable() and action.isChecked() and not action.isEnabled()
	action.setEnabled(True)
	action.trigger()
	assert triggered == [True]
	window.deleteLater()


#============================================
def test_registry_requires_explicit_feature_binding_without_window_discovery(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An unregistered window action never becomes a registry fallback client."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	legacy_candidate = PySide6.QtGui.QAction("Legacy Draw Bond", window)
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	assert registry.get_qt_action("draw.bond") is None
	registry.register_existing(
		"draw.bond", legacy_candidate,
		shortcut_exemption_reason="The command is reachable by its labelled menu.",
	)
	assert registry.get_qt_action("draw.bond") is legacy_candidate
	window.deleteLater()


#============================================
def test_menu_builder_renders_sections_submenus_and_registered_dynamic_menu(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""YAML ordering creates labelled hierarchy while reusing owned Qt objects."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	new_action = _registered_action(registry, "file.new", "New", window)
	bond_action = _registered_action(registry, "draw.bond", "Draw Bond", window)
	ring_action = _registered_action(
		registry, "draw.ring.regular.c6", "Cyclohexane", window,
	)
	recent_menu = PySide6.QtWidgets.QMenu("Recent Files", window)
	registry.register_dynamic_menu(
		"file.recent", recent_menu, "Entries rebuild from preferences.",
	)
	monkeypatch.setattr(
		ferrum_qt.declarative_resources, "load_menu_declarations", _declarations,
	)
	menus = ferrum_qt.actions.menu_builder.build_declared_menus(window, registry)
	file_actions = menus["file"].actions()
	draw_actions = menus["draw"].actions()
	ring_menu = next(
		action.menu() for action in draw_actions
		if action.menu() is not None and action.menu().title() == "Insert Regular Ring..."
	)
	assert new_action in file_actions and recent_menu.menuAction() in file_actions
	assert bond_action in draw_actions and ring_action in ring_menu.actions()
	window.deleteLater()


#============================================
def test_menu_builder_rejects_repeat_assembly(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Menu placement stays a single deliberate construction boundary."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	_registered_action(registry, "file.new", "New", window)
	_registered_action(registry, "draw.bond", "Draw Bond", window)
	_registered_action(registry, "draw.ring.regular.c6", "Cyclohexane", window)
	recent_menu = PySide6.QtWidgets.QMenu("Recent Files", window)
	registry.register_dynamic_menu(
		"file.recent", recent_menu, "Entries rebuild from preferences.",
	)
	monkeypatch.setattr(
		ferrum_qt.declarative_resources, "load_menu_declarations", _declarations,
	)
	ferrum_qt.actions.menu_builder.build_declared_menus(window, registry)
	with pytest.raises(RuntimeError, match="already assembled"):
		ferrum_qt.actions.menu_builder.build_declared_menus(window, registry)
	window.deleteLater()


#============================================
def test_menu_builder_preflights_every_live_action_before_menu_bar_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An unbound later action leaves the one-time assembly boundary untouched."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	_registered_action(registry, "file.new", "New", window)
	registry.register(ferrum_qt.actions.action_registry.MenuAction(
		"file.unbound", "Unbound", "Unbound test action", None, None, None,
		"The failure-atomicity contract intentionally leaves this action unbound.",
	))
	declarations = {"menus": [{
		"id": "file", "label_key": "File", "help_key": "Document commands",
		"items": [{"action": "file.new"}, {"action": "file.unbound"}],
	}]}
	monkeypatch.setattr(
		ferrum_qt.declarative_resources, "load_menu_declarations", lambda: declarations,
	)
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="no bound QAction",
		):
		ferrum_qt.actions.menu_builder.build_declared_menus(window, registry)
	assert not window.menuBar().actions()
	with pytest.raises(ferrum_qt.declarative_resources.DeclarativeResourceError):
		ferrum_qt.actions.menu_builder.build_declared_menus(window, registry)
	window.deleteLater()


#============================================
def test_menu_builder_renders_atomically_after_a_late_client_resolution_failure(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A render-time fault leaves no clients attached and supports one clean retry."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	_registered_action(registry, "file.new", "New", window)
	bond_action = _registered_action(registry, "draw.bond", "Draw Bond", window)
	declarations = {"menus": [
		{
			"id": "file", "label_key": "File", "help_key": "Document commands",
			"items": [{"action": "file.new"}],
		},
		{
			"id": "draw", "label_key": "Draw", "help_key": "Drawing commands",
			"items": [{"action": "draw.bond"}],
		},
	]}
	monkeypatch.setattr(
		ferrum_qt.declarative_resources, "load_menu_declarations", lambda: declarations,
	)
	original_require = ferrum_qt.actions.menu_builder._require_bound_action
	def fail_late_client_resolution(
				target_registry: object, action_id: str,
				) -> PySide6.QtGui.QAction:
		"""Simulate an owner losing the later client after successful preflight."""
		if action_id == "draw.bond":
			raise ferrum_qt.declarative_resources.DeclarativeResourceError(
				"Late Draw Bond client resolution failure.",
			)
		return original_require(target_registry, action_id)
	monkeypatch.setattr(
		ferrum_qt.actions.menu_builder, "_require_bound_action",
		fail_late_client_resolution,
	)
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="Late Draw Bond client resolution failure",
		):
		ferrum_qt.actions.menu_builder.build_declared_menus(window, registry)
	assert not window.menuBar().actions()
	monkeypatch.setattr(
		ferrum_qt.actions.menu_builder, "_require_bound_action", original_require,
	)
	menus = ferrum_qt.actions.menu_builder.build_declared_menus(window, registry)
	assert bond_action in menus["draw"].actions()
	window.deleteLater()


#============================================
def test_dynamic_menu_registration_rejects_conflicting_lifecycle_reason(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A changing menu cannot replace the owner's prior lifecycle explanation."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	registry.declare_dynamic_lifecycle("file.recent", "First owner declaration.")
	with pytest.raises(ValueError, match="conflicting lifecycle reasons"):
		registry.register_dynamic_menu(
			"file.recent", PySide6.QtWidgets.QMenu("Recent Files", window),
			"A different owner declaration.",
		)
	window.deleteLater()


#============================================
def test_platform_roles_are_limited_to_standard_application_actions(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Menu migration preserves macOS roles without assigning one to drawing."""
	del qapp
	window = PySide6.QtWidgets.QMainWindow()
	registry = ferrum_qt.actions.action_registry.ActionRegistry()
	quit_action = _registered_action(registry, "file.quit", "Quit", window)
	preferences_action = _registered_action(
		registry, "options.preferences", "Preferences", window,
	)
	about_action = _registered_action(registry, "help.about", "About Ferrum", window)
	draw_action = _registered_action(registry, "draw.bond", "Draw Bond", window)
	draw_role = draw_action.menuRole()
	ferrum_qt.actions.platform_menu.apply_platform_menu_roles(registry)
	assert (
		quit_action.menuRole(), preferences_action.menuRole(), about_action.menuRole(),
	) == (
		PySide6.QtGui.QAction.MenuRole.QuitRole,
		PySide6.QtGui.QAction.MenuRole.PreferencesRole,
		PySide6.QtGui.QAction.MenuRole.AboutRole,
	)
	assert draw_action.menuRole() is draw_role
	window.deleteLater()
