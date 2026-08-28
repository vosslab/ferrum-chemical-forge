"""Create bounded context menus using shared Ferrum action instances."""

# PIP3 modules
import PySide6.QtWidgets
import shiboken6


#============================================
def build_context_menu(
		parent: PySide6.QtWidgets.QWidget, registry: object,
		action_groups: tuple[tuple[str, ...], ...], accessible_name: str,
		) -> PySide6.QtWidgets.QMenu | None:
	"""Return enabled registered actions in declared groups, or no menu."""
	menu = PySide6.QtWidgets.QMenu(parent)
	menu.setAccessibleName(parent.tr(accessible_name))
	added_group = False
	for action_ids in action_groups:
		actions = []
		for action_id in action_ids:
			action = registry.get_qt_action(action_id)
			if action is not None and action.isEnabled():
				actions.append(action)
		if not actions:
			continue
		if added_group:
			menu.addSeparator()
		for action in actions:
			menu.addAction(action)
		added_group = True
	if not added_group:
		menu.deleteLater()
		return None
	return menu


#============================================
def present_context_menu(menu: PySide6.QtWidgets.QMenu,
		viewport: PySide6.QtWidgets.QWidget,
		global_position: PySide6.QtCore.QPoint) -> None:
	"""Present one transient menu and return focus to its invoking viewport."""
	menu.setAttribute(PySide6.QtCore.Qt.WidgetAttribute.WA_DeleteOnClose)
	menu.aboutToHide.connect(
		lambda: _restore_viewport_focus_after_menu(menu, viewport),
	)
	menu.popup(global_position)


#============================================
def _restore_viewport_focus_after_menu(menu: PySide6.QtWidgets.QMenu,
		viewport: PySide6.QtWidgets.QWidget) -> None:
	"""Wait until the transient menu has relinquished its focus ownership."""
	def restore_after_menu_destroyed(*_args: object) -> None:
		"""Restore after Qt has completed the menu's terminal close boundary."""
		PySide6.QtCore.QTimer.singleShot(0, lambda: _restore_viewport_focus(viewport))

	menu.destroyed.connect(
		restore_after_menu_destroyed,
		PySide6.QtCore.Qt.ConnectionType.SingleShotConnection,
	)


#============================================
def _restore_viewport_focus(viewport: PySide6.QtWidgets.QWidget) -> None:
	"""Restore context-menu focus unless a modal widget now owns interaction."""
	modal = PySide6.QtWidgets.QApplication.activeModalWidget()
	if isinstance(modal, PySide6.QtWidgets.QDialog):
		_restore_viewport_focus_after_dialog(modal, viewport)
		return
	if modal is None and shiboken6.isValid(viewport):
		viewport.window().activateWindow()
		viewport.setFocus()


#============================================
def _restore_viewport_focus_after_dialog(dialog: PySide6.QtWidgets.QDialog,
		viewport: PySide6.QtWidgets.QWidget) -> None:
	"""Restore after this dialog, unless another modal takes its place."""
	def restore_after_dialog_finished(*_args: object) -> None:
		"""Wait until Qt releases the finished dialog's modal ownership."""
		PySide6.QtCore.QTimer.singleShot(
			0, lambda: _restore_viewport_focus_after_modal(viewport),
		)

	dialog.finished.connect(
		restore_after_dialog_finished,
		PySide6.QtCore.Qt.ConnectionType(
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection.value
			| PySide6.QtCore.Qt.ConnectionType.SingleShotConnection.value,
		),
	)


#============================================
def _restore_viewport_focus_after_modal(viewport: PySide6.QtWidgets.QWidget) -> None:
	"""Restore only when no successor modal owns interaction."""
	if (PySide6.QtWidgets.QApplication.activeModalWidget() is None
			and shiboken6.isValid(viewport)):
		viewport.window().activateWindow()
		viewport.setFocus()
