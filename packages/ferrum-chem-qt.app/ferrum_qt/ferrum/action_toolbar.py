"""Responsive action client for the ordinary Ferrum product window."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


ActionIconGroup = tuple[
	tuple[PySide6.QtGui.QAction, PySide6.QtWidgets.QStyle.StandardPixmap], ...
]


#============================================
class FerrumNativeActionToolbar(PySide6.QtWidgets.QToolBar):
	"""Expose existing Ferrum actions without creating another command owner."""

	#============================================
	def __init__(self, groups: tuple[ActionIconGroup, ...],
			parent: PySide6.QtWidgets.QMainWindow) -> None:
		"""Group labeled action clients and leave narrow overflow to Qt layout."""
		super().__init__(parent.tr("Main Toolbar"), parent)
		self.setObjectName("native-main-action-toolbar")
		self.setAccessibleName(parent.tr("Main document toolbar"))
		self.setMovable(False)
		self.setFloatable(False)
		self.setAllowedAreas(PySide6.QtCore.Qt.ToolBarArea.TopToolBarArea)
		self.setSizePolicy(
			PySide6.QtWidgets.QSizePolicy.Policy.Ignored,
			PySide6.QtWidgets.QSizePolicy.Policy.Fixed,
		)
		self.setIconSize(PySide6.QtCore.QSize(24, 24))
		self.setToolButtonStyle(
			PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextUnderIcon,
		)
		for group_index, group in enumerate(groups):
			if group_index:
				self.addSeparator()
			for action, icon_source in group:
				self._add_existing_action(action, icon_source)

	#============================================
	def _add_existing_action(self, action: PySide6.QtGui.QAction,
			icon_source: PySide6.QtWidgets.QStyle.StandardPixmap) -> None:
		"""Add one shared action with a platform icon and explicit accessible label."""
		if action.icon().isNull():
			action.setIcon(self.style().standardIcon(icon_source))
		self.addAction(action)
		button = self.widgetForAction(action)
		if isinstance(button, PySide6.QtWidgets.QToolButton):
			button.setAccessibleName(action.text())
			button.setAccessibleDescription(action.toolTip())

	#============================================
	def minimumSizeHint(self) -> PySide6.QtCore.QSize:
		"""Never make the document window's minimum width follow toolbar contents."""
		hint = super().minimumSizeHint()
		return PySide6.QtCore.QSize(0, hint.height())


#============================================
def install_native_action_toolbar(window: PySide6.QtWidgets.QMainWindow,
		groups: tuple[ActionIconGroup, ...]) -> FerrumNativeActionToolbar:
	"""Install one top toolbar plus its ordinary View-menu visibility action."""
	toolbar = FerrumNativeActionToolbar(groups, window)
	window.addToolBar(PySide6.QtCore.Qt.ToolBarArea.TopToolBarArea, toolbar)
	toggle = toolbar.toggleViewAction()
	toggle.setText(window.tr("Main Toolbar"))
	toggle.setToolTip(window.tr("Show or hide the main document toolbar"))
	window._view_menu.addSeparator()
	window._view_menu.addAction(toggle)
	return toolbar
