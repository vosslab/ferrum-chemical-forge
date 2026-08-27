"""Task-oriented Ferrum ribbon projecting existing registry-owned actions."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.authoring_ribbon_layout
import ferrum_qt.ferrum.drawing_parameters_client
import ferrum_qt.widgets.ribbon_group


_PAGE_MARGINS = (4, 2, 4, 2)
_GROUP_SPACING = 14


#============================================
class _RibbonTabPage(PySide6.QtWidgets.QWidget):
	"""Allocate one task tab before child expanded minima can own its width."""

	#============================================
	def __init__(self, layout: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonTabLayout,
			parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build YAML-ordered groups without a horizontal group-owning layout."""
		super().__init__(parent)
		self.setAccessibleName(self.tr(layout.label_key))
		self._groups = tuple(ferrum_qt.widgets.ribbon_group.RibbonGroup(item, self)
			for item in layout.groups)
		self._allocating = False

	#============================================
	def minimumSizeHint(self) -> PySide6.QtCore.QSize:
		"""Expose all-collapsed width so inactive stacked pages remain bounded."""
		states = tuple(ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COLLAPSED
			for _group in self._groups)
		return PySide6.QtCore.QSize(self._required_width(states, minimum=True), self._group_height())

	#============================================
	def sizeHint(self) -> PySide6.QtCore.QSize:
		"""Report current measured allocation without creating a hard expanded floor."""
		return PySide6.QtCore.QSize(
			self._required_width(tuple(group.display_state for group in self._groups)),
			self._group_height(),
		)

	#============================================
	def resizeEvent(self, event: PySide6.QtGui.QResizeEvent) -> None:
		"""Reallocate from actual tab content width whenever Qt resizes this page."""
		super().resizeEvent(event)
		self.reallocate()

	#============================================
	def showEvent(self, event: PySide6.QtGui.QShowEvent) -> None:
		"""Allocate after this page becomes the visible stacked page."""
		super().showEvent(event)
		self.reallocate()

	#============================================
	def event(self, event: PySide6.QtCore.QEvent) -> bool:
		"""Refresh when state changes request geometry without a resize timer."""
		accepted = super().event(event)
		if event.type() is PySide6.QtCore.QEvent.Type.LayoutRequest:
			self.reallocate()
		return accepted

	#============================================
	def reallocate(self) -> None:
		"""Choose YAML-order states and directly assign group geometries."""
		if self._allocating:
			return
		self._allocating = True
		try:
			available = max(0, self.contentsRect().width())
			states = [ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.EXPANDED
				for _group in self._groups]
			self._reduce_to_fit(states, available,
				ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.EXPANDED,
				ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COMPACT)
			self._reduce_to_fit(states, available,
				ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COMPACT,
				ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState.COLLAPSED)
			for group, state in zip(self._groups, states, strict=True):
				group.set_display_state(state)
			x = self.contentsRect().x() + _PAGE_MARGINS[0]
			y = self.contentsRect().y() + _PAGE_MARGINS[1]
			height = max(0, self.contentsRect().height() - _PAGE_MARGINS[1] - _PAGE_MARGINS[3])
			for group, state in zip(self._groups, states, strict=True):
				width = group.width_for(state)
				group.setGeometry(x, y, width, height)
				x += width + _GROUP_SPACING
		finally:
			self._allocating = False

	#============================================
	def _reduce_to_fit(self, states: list[ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState],
			available: int, from_state: ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState,
			to_state: ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState) -> None:
		"""Use reverse YAML order as the deterministic width-reduction tie-breaker."""
		for index in reversed(range(len(states))):
			if self._required_width(tuple(states)) <= available:
				return
			if states[index] is from_state:
				states[index] = to_state

	#============================================
	def _required_width(self, states: tuple[ferrum_qt.widgets.ribbon_group.RibbonGroupDisplayState, ...],
			minimum: bool = False) -> int:
		"""Measure selected state controls from their current live size hints."""
		widths = (group.minimum_width_for(state) if minimum else group.width_for(state)
			for group, state in zip(self._groups, states, strict=True))
		return _PAGE_MARGINS[0] + _PAGE_MARGINS[2] + sum(widths) + _GROUP_SPACING * max(0, len(self._groups) - 1)

	#============================================
	def _group_height(self) -> int:
		"""Use tallest labelled group plus page margins as the page height hint."""
		return _PAGE_MARGINS[1] + _PAGE_MARGINS[3] + max(
			(group.sizeHint().height() for group in self._groups), default=0,
		)


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
		self.setAccessibleDescription(parent.tr("Frequent chemistry authoring commands organized by task."))
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
		self._tabs.currentChanged.connect(self._reallocate_current_page)
		content_layout.addWidget(self._tabs)
		self._context_row = PySide6.QtWidgets.QWidget(self._content)
		context_layout = PySide6.QtWidgets.QHBoxLayout(self._context_row)
		context_layout.setContentsMargins(8, 0, 8, 0)
		context_layout.setSpacing(6)
		self._drawing_parameters_client = ferrum_qt.ferrum.drawing_parameters_client.FerrumNativeDrawingParametersClient(
			drawing_parameters, cancel_action, parent=self._context_row,
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
		"""Render controller-owned active-tool capability without interpreting IDs."""
		self._active_tool_state = state
		self._refresh_context()

	#============================================
	def groups_for_tab(self, tab_id: str) -> tuple[ferrum_qt.widgets.ribbon_group.RibbonGroup, ...]:
		"""Return a tab's task groups for semantic UI tests and diagnostics."""
		return self._groups_by_tab[tab_id]

	#============================================
	def _page_for_tab(self, tab: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonTabLayout,
			) -> _RibbonTabPage:
		"""Build one allocator-owned task page in its declared YAML order."""
		page = _RibbonTabPage(tab, self._tabs)
		self._groups_by_tab[tab.id] = page._groups
		return page

	#============================================
	def _reallocate_current_page(self, _index: int) -> None:
		"""Refresh immediately when tab selection reveals another stacked page."""
		page = self._tabs.currentWidget()
		if isinstance(page, _RibbonTabPage):
			page.reallocate()

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
