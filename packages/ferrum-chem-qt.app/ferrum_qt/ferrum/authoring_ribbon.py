"""Task-oriented Ferrum ribbon projecting existing registry-owned actions."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.authoring_ribbon_layout
import ferrum_qt.ferrum.drawing_parameters_client
import ferrum_qt.widgets.ribbon_group


#============================================
class AuthoringRibbon(PySide6.QtWidgets.QToolBar):
	"""Present labelled task groups without duplicating Ferrum command ownership."""

	#============================================
	def __init__(self, registry: object, mode_sync: object, drawing_parameters: object,
			next_drawing_action: PySide6.QtGui.QAction, cancel_action: PySide6.QtGui.QAction,
			parent: PySide6.QtWidgets.QMainWindow) -> None:
		"""Resolve all live actions before building the ribbon's visible clients."""
		super().__init__(parent.tr("Authoring Ribbon"), parent)
		self.setObjectName("ferrum-authoring-ribbon")
		self.setAccessibleName(parent.tr("Ferrum authoring ribbon"))
		self.setAccessibleDescription(parent.tr(
			"Frequent chemistry authoring commands organized by task.",
		))
		self.setMovable(False)
		self.setFloatable(False)
		self.setAllowedAreas(PySide6.QtCore.Qt.ToolBarArea.TopToolBarArea)
		self.setSizePolicy(PySide6.QtWidgets.QSizePolicy.Policy.Expanding,
			PySide6.QtWidgets.QSizePolicy.Policy.Fixed)
		self._layouts = ferrum_qt.ferrum.authoring_ribbon_layout.load_ribbon_layout(registry)
		self._active_tool_state = mode_sync.active_state
		self._groups_by_tab: dict[str, tuple[ferrum_qt.widgets.ribbon_group.RibbonGroup, ...]] = {}
		self._content = PySide6.QtWidgets.QWidget(self)
		content_layout = PySide6.QtWidgets.QVBoxLayout(self._content)
		content_layout.setContentsMargins(2, 2, 2, 2)
		content_layout.setSpacing(2)
		self._tabs = PySide6.QtWidgets.QTabWidget(self._content)
		self._tabs.setObjectName("authoring-ribbon-tabs")
		self._tabs.setAccessibleName(self.tr("Authoring tasks"))
		self._tabs.setUsesScrollButtons(True)
		for tab in self._layouts:
			self._tabs.addTab(self._page_for_tab(tab), self.tr(tab.label_key))
		content_layout.addWidget(self._tabs)
		self._context_row = PySide6.QtWidgets.QWidget(self._content)
		context_layout = PySide6.QtWidgets.QHBoxLayout(self._context_row)
		context_layout.setContentsMargins(8, 0, 8, 0)
		context_layout.setSpacing(6)
		self._drawing_parameters_client = (
			ferrum_qt.ferrum.drawing_parameters_client.FerrumNativeDrawingParametersClient(
				drawing_parameters, cancel_action, parent=self._context_row,
			)
		)
		self._drawing_parameters_client.setObjectName("authoring-drawing-defaults")
		context_layout.addWidget(self._drawing_parameters_client)
		self._next_drawing_button = self._context_button(next_drawing_action)
		context_layout.addWidget(self._next_drawing_button)
		self._context_instruction = PySide6.QtWidgets.QLabel(self._context_row)
		self._context_instruction.setObjectName("authoring-context-instruction")
		self._context_instruction.setAccessibleName(self.tr("Current tool instruction"))
		self._context_instruction.setSizePolicy(PySide6.QtWidgets.QSizePolicy.Policy.Expanding,
			PySide6.QtWidgets.QSizePolicy.Policy.Preferred)
		context_layout.addWidget(self._context_instruction, 1)
		content_layout.addWidget(self._context_row)
		self.addWidget(self._content)
		mode_sync.subscribe(self._set_active_tool_state)
		self._refresh_context()

	#============================================
	def _set_active_tool_state(self, state: object) -> None:
		"""Render the controller-owned active-tool capability without interpreting IDs."""
		self._active_tool_state = state
		self._refresh_context()

	#============================================
	def groups_for_tab(self, tab_id: str) -> tuple[ferrum_qt.widgets.ribbon_group.RibbonGroup, ...]:
		"""Return a tab's task groups for semantic UI tests and diagnostics."""
		return self._groups_by_tab[tab_id]

	#============================================
	def _page_for_tab(self, tab: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonTabLayout,
			) -> PySide6.QtWidgets.QWidget:
		"""Build one task page in YAML order without fixed-width assumptions."""
		page = PySide6.QtWidgets.QWidget(self._tabs)
		page.setAccessibleName(self.tr(tab.label_key))
		layout = PySide6.QtWidgets.QHBoxLayout(page)
		layout.setContentsMargins(4, 2, 4, 2)
		layout.setSpacing(14)
		groups: list[ferrum_qt.widgets.ribbon_group.RibbonGroup] = []
		for group_layout in tab.groups:
			group = ferrum_qt.widgets.ribbon_group.RibbonGroup(group_layout, page)
			layout.addWidget(group)
			groups.append(group)
		layout.addStretch(1)
		self._groups_by_tab[tab.id] = tuple(groups)
		return page

	#============================================
	def _context_button(self, action: PySide6.QtGui.QAction) -> PySide6.QtWidgets.QToolButton:
		"""Create one labelled client for the existing next-drawing action."""
		button = PySide6.QtWidgets.QToolButton(self._context_row)
		button.setDefaultAction(action)
		button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
		button.setAccessibleName(action.text())
		button.setAccessibleDescription(action.toolTip() or action.text())
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		return button

	#============================================
	def _refresh_context(self) -> None:
		"""Show defaults only when the active feature binding supplies them."""
		active = self._active_tool_state
		uses_defaults = active.supplies_drawing_defaults
		self._drawing_parameters_client.setVisible(uses_defaults)
		self._next_drawing_button.setVisible(uses_defaults)
		self._context_row.setVisible(uses_defaults or active.mode_id is not None)
		if uses_defaults:
			message = self.tr("Next atom/bond defaults.")
			accessible_message = self.tr("Drawing defaults apply to the next atom or bond.")
		elif active.mode_id is not None:
			message = active.status_label
			accessible_message = active.status_label
		else:
			message = self.tr("Choose a tool, then work directly on the canvas.")
			accessible_message = message
		self._context_instruction.setText(message)
		self._context_instruction.setAccessibleDescription(accessible_message)
