"""Create bounded context menus using shared Ferrum action instances."""

# PIP3 modules
import PySide6.QtWidgets


#============================================
def build_context_menu(
		parent: PySide6.QtWidgets.QWidget, registry: object,
		action_ids: tuple[str, ...],
		) -> PySide6.QtWidgets.QMenu:
	"""Return a menu made only from already-registered product actions."""
	menu = PySide6.QtWidgets.QMenu(parent)
	menu.setAccessibleName(parent.tr("Drawing actions"))
	for action_id in action_ids:
		action = registry.get_qt_action(action_id)
		if action is None:
			raise KeyError(f"Ferrum context action is not registered: '{action_id}'.")
		menu.addAction(action)
	return menu
