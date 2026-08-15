"""Editing-tool action client for the ordinary Rust-native product window."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.widgets.icon_loader
import ferrum_qt.native.ferrum_native_drawing_parameters
import ferrum_qt.native.ferrum_native_drawing_parameters_client


EditingTool = tuple[PySide6.QtGui.QAction, str]


#============================================
class FerrumNativeEditingToolsToolbar(PySide6.QtWidgets.QToolBar):
	"""Project existing native pointer actions without creating a tool owner."""

	#============================================
	def __init__(self, tools: tuple[EditingTool, ...],
			cancel_action: PySide6.QtGui.QAction, theme_manager: object,
			drawing_parameters: (
				ferrum_qt.native.ferrum_native_drawing_parameters.
				FerrumNativeDrawingParameters
			),
			next_drawing_action: PySide6.QtGui.QAction,
			parent: PySide6.QtWidgets.QMainWindow) -> None:
		"""Add shared tool actions in their ordinary editing workflow order."""
		super().__init__(parent.tr("Editing Tools"), parent)
		self.setObjectName("native-editing-tools-toolbar")
		self.setAccessibleName(parent.tr("Editing tools toolbar"))
		self.setAccessibleDescription(parent.tr(
			"Choose a canvas editing tool. Escape cancels the active tool.",
		))
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
		self._tools = tools
		self._cancel_action = cancel_action
		self._theme_manager = theme_manager
		self._drawing_parameters = drawing_parameters
		self._next_drawing_action = next_drawing_action
		self._add_visible_header(parent)
		self.addSeparator()
		for tool_index, (action, _icon_name) in enumerate(tools):
			if tool_index in (1, 5):
				self.addSeparator()
			self.addAction(action)
			if tool_index == 4:
				self.addSeparator()
				self._add_drawing_parameters_client(parent)
				self.addAction(next_drawing_action)
				self.addSeparator()
		self.addSeparator()
		self.addAction(cancel_action)
		for action, _icon_name in tools + ((cancel_action, "remove"),):
			action.changed.connect(self._refresh_accessible_buttons)
		next_drawing_action.changed.connect(self._refresh_accessible_buttons)
		self._apply_theme(self._theme_name())
		theme_changed = getattr(theme_manager, "theme_changed", None)
		if theme_changed is not None:
			theme_changed.connect(self._apply_theme)
		self._refresh_accessible_buttons()

	#============================================
	def _add_drawing_parameters_client(
			self, parent: PySide6.QtWidgets.QMainWindow) -> None:
		"""Project personal next-operation choices without creating a document owner."""
		self._drawing_parameters_client = (
			ferrum_qt.native.ferrum_native_drawing_parameters_client.
			FerrumNativeDrawingParametersClient(
				self._drawing_parameters, self._cancel_action, parent=self,
			)
		)
		self.addWidget(self._drawing_parameters_client)

	#============================================
	def _add_visible_header(self, parent: PySide6.QtWidgets.QMainWindow) -> None:
		"""Add the docked toolbar's visible category identity without state policy."""
		header = PySide6.QtWidgets.QLabel(parent.tr("Editing Tools"), self)
		header.setAccessibleName(parent.tr("Editing Tools"))
		header.setAccessibleDescription(parent.tr(
			"Canvas editing commands are available in the following toolbar controls.",
		))
		header.setContentsMargins(4, 0, 4, 0)
		header.setAlignment(
			PySide6.QtCore.Qt.AlignmentFlag.AlignVCenter
			| PySide6.QtCore.Qt.AlignmentFlag.AlignLeft,
		)
		font = header.font()
		font.setBold(True)
		header.setFont(font)
		self.addWidget(header)

	#============================================
	def _theme_name(self) -> str:
		"""Return the installed theme name or the safe light projection fallback."""
		theme_name = getattr(self._theme_manager, "current_theme", "light")
		if theme_name not in ("light", "dark"):
			return "light"
		return theme_name

	#============================================
	def _apply_theme(self, theme_name: str) -> None:
		"""Update existing action icons for one application-theme transition."""
		if theme_name not in ("light", "dark"):
			theme_name = "light"
		ferrum_qt.widgets.icon_loader.set_theme(theme_name)
		ferrum_qt.widgets.icon_loader.reload_icons()
		for action, icon_name in self._tools:
			action.setIcon(ferrum_qt.widgets.icon_loader.get_icon(icon_name))
		self._cancel_action.setIcon(ferrum_qt.widgets.icon_loader.get_icon("remove"))

	#============================================
	def _refresh_accessible_buttons(self) -> None:
		"""Name each shared-action client for keyboard and screen-reader use."""
		for action, _icon_name in self._tools + ((self._cancel_action, "remove"),):
			button = self.widgetForAction(action)
			if isinstance(button, PySide6.QtWidgets.QToolButton):
				button.setAccessibleName(action.text())
				button.setAccessibleDescription(action.toolTip())
		button = self.widgetForAction(self._next_drawing_action)
		if isinstance(button, PySide6.QtWidgets.QToolButton):
			button.setToolButtonStyle(
				PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonIconOnly,
			)
			button.setAccessibleName(self._next_drawing_action.text())
			button.setAccessibleDescription(self._next_drawing_action.toolTip())

	#============================================
	def minimumSizeHint(self) -> PySide6.QtCore.QSize:
		"""Let native Qt overflow keep a narrow window usable."""
		hint = super().minimumSizeHint()
		return PySide6.QtCore.QSize(0, hint.height())


#============================================
def install_native_editing_tools_toolbar(window: PySide6.QtWidgets.QMainWindow,
		tools: tuple[EditingTool, ...], cancel_action: PySide6.QtGui.QAction,
		theme_manager: object,
		drawing_parameters: (
			ferrum_qt.native.ferrum_native_drawing_parameters.
			FerrumNativeDrawingParameters
			), next_drawing_action: PySide6.QtGui.QAction) -> FerrumNativeEditingToolsToolbar:
	"""Install the ordinary toolbar and its View-menu visibility action."""
	window.addToolBarBreak(PySide6.QtCore.Qt.ToolBarArea.TopToolBarArea)
	toolbar = FerrumNativeEditingToolsToolbar(
		tools, cancel_action, theme_manager, drawing_parameters, next_drawing_action, window,
	)
	window.addToolBar(PySide6.QtCore.Qt.ToolBarArea.TopToolBarArea, toolbar)
	toggle = toolbar.toggleViewAction()
	toggle.setText(window.tr("Editing Tools"))
	toggle.setToolTip(window.tr("Show or hide the editing tools toolbar"))
	window._view_menu.addSeparator()
	window._view_menu.addAction(toggle)
	return toolbar
