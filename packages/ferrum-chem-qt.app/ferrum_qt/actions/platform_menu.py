"""Apply platform menu roles to Ferrum's shared action instances."""

# PIP3 modules
import PySide6.QtGui


#============================================
def apply_platform_menu_roles(registry: object) -> None:
	"""Use Qt-standard application roles without allocating replacement actions."""
	roles = {
		"file.quit": PySide6.QtGui.QAction.MenuRole.QuitRole,
		"options.preferences": PySide6.QtGui.QAction.MenuRole.PreferencesRole,
		"help.about": PySide6.QtGui.QAction.MenuRole.AboutRole,
	}
	for action_id, role in roles.items():
		action = registry.get_qt_action(action_id)
		if action is not None:
			action.setMenuRole(role)
