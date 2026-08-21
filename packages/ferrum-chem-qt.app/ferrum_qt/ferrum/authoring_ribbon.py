"""Two-row authoring ribbon that projects existing Ferrum actions."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.drawing_parameters_client
import ferrum_qt.widgets.icon_loader


_COMPACT_BREAKPOINT = 1120
_ICON_SIZE = 20


#============================================
class AuthoringRibbon(PySide6.QtWidgets.QToolBar):
	"""Present shared commands and tools without taking document ownership."""

	mode_selected = PySide6.QtCore.Signal(str)

	#============================================
	def __init__(self, command_actions: tuple[PySide6.QtGui.QAction, ...],
			tool_actions: tuple[tuple[PySide6.QtGui.QAction, str], ...],
			cancel_action: PySide6.QtGui.QAction, drawing_parameters: object,
			next_drawing_action: PySide6.QtGui.QAction, theme_manager: object,
			parent: PySide6.QtWidgets.QMainWindow) -> None:
		"""Build two visual rows from established action and preference clients."""
		super().__init__(parent.tr("Authoring Ribbon"), parent)
		self.setObjectName("ferrum-authoring-ribbon")
		self.setAccessibleName(parent.tr("Ferrum authoring ribbon"))
		self.setAccessibleDescription(parent.tr(
			"Frequent commands, active drawing tools, and contextual drawing defaults.",
		))
		self.setMovable(False)
		self.setFloatable(False)
		self.setAllowedAreas(PySide6.QtCore.Qt.ToolBarArea.TopToolBarArea)
		self.setIconSize(PySide6.QtCore.QSize(_ICON_SIZE, _ICON_SIZE))
		self.setSizePolicy(
			PySide6.QtWidgets.QSizePolicy.Policy.Expanding,
			PySide6.QtWidgets.QSizePolicy.Policy.Fixed,
		)
		self._command_actions = command_actions
		self._tool_actions = tool_actions
		self._cancel_action = cancel_action
		self._tool_action_group = PySide6.QtGui.QActionGroup(self)
		self._tool_action_group.setExclusionPolicy(
			PySide6.QtGui.QActionGroup.ExclusionPolicy.ExclusiveOptional,
		)
		self._mode_actions: dict[str, PySide6.QtGui.QAction] = {}
		self._context_action: PySide6.QtGui.QAction | None = None
		self._coordinating_tool_change = False
		self._compact = False
		self._command_buttons: list[PySide6.QtWidgets.QToolButton] = []
		self._tool_buttons: list[PySide6.QtWidgets.QToolButton] = []
		self._low_priority_buttons: list[PySide6.QtWidgets.QToolButton] = []
		self._content = PySide6.QtWidgets.QWidget(self)
		self._content.setObjectName("authoring-ribbon-content")
		self._content.setSizePolicy(
			PySide6.QtWidgets.QSizePolicy.Policy.Expanding,
			PySide6.QtWidgets.QSizePolicy.Policy.Fixed,
		)
		layout = PySide6.QtWidgets.QVBoxLayout(self._content)
		layout.setContentsMargins(4, 2, 4, 2)
		layout.setSpacing(2)
		self._command_row = PySide6.QtWidgets.QWidget(self._content)
		self._command_layout = self._new_row_layout(self._command_row)
		layout.addWidget(self._command_row)
		self._tool_row = PySide6.QtWidgets.QWidget(self._content)
		self._tool_layout = self._new_row_layout(self._tool_row)
		layout.addWidget(self._tool_row)
		self._build_command_row()
		self._build_tool_row(drawing_parameters, next_drawing_action)
		self.addWidget(self._content)
		self._apply_theme(getattr(theme_manager, "current_theme", "light"))
		theme_changed = getattr(theme_manager, "theme_changed", None)
		if theme_changed is not None:
			theme_changed.connect(self._apply_theme)
		for action, _icon_name in self._tool_actions:
			action.setCheckable(True)
			self._tool_action_group.addAction(action)
			action.toggled.connect(
				lambda checked, changed_action=action: self._on_tool_toggled(
					changed_action, checked,
				),
			)
		cancel_action.changed.connect(self._refresh_context)
		self._refresh_context()

	#============================================
	def add_mode(self, mode_id: str, action: PySide6.QtGui.QAction) -> None:
		"""Register an existing mode action for the established mode-manager seam."""
		if not mode_id or mode_id in self._mode_actions:
			raise ValueError("Ferrum mode IDs must be unique and nonempty")
		action.setCheckable(True)
		action.setData(mode_id)
		action.triggered.connect(
			lambda checked, selected_mode_id=mode_id: self._on_mode_triggered(
				selected_mode_id, checked,
			),
		)
		self._mode_actions[mode_id] = action

	#============================================
	def set_active_mode(self, mode_id: str | None) -> None:
		"""Reflect the document-free mode manager without dispatching a command."""
		for known_mode_id, action in self._mode_actions.items():
			action.setChecked(known_mode_id == mode_id)
		self._refresh_context()

	#============================================
	def apply_mode_manager(self, manager: object) -> None:
		"""Update mode clients from the existing manager observation."""
		active_mode_id = getattr(manager, "active_mode_id", None)
		value = getattr(active_mode_id, "value", active_mode_id)
		self.set_active_mode(value if type(value) is str else None)

	#============================================
	def resizeEvent(self, event: PySide6.QtGui.QResizeEvent) -> None:
		"""Use an explicit More menu before a native toolbar extension can appear."""
		super().resizeEvent(event)
		compact = self.width() < _COMPACT_BREAKPOINT
		if compact != self._compact:
			self._compact = compact
			for button in self._low_priority_buttons:
				button.setVisible(not compact)
			self._more_button.setVisible(compact)
			self._more_tools_button.setVisible(compact)

	#============================================
	def minimumSizeHint(self) -> PySide6.QtCore.QSize:
		"""Keep the one ribbon host shrinkable; its More menu handles narrow widths."""
		hint = super().minimumSizeHint()
		return PySide6.QtCore.QSize(0, hint.height())

	#============================================
	def _new_row_layout(self, row: PySide6.QtWidgets.QWidget) -> PySide6.QtWidgets.QHBoxLayout:
		"""Create one dense row with no unexplained whitespace."""
		layout = PySide6.QtWidgets.QHBoxLayout(row)
		layout.setContentsMargins(0, 0, 0, 0)
		layout.setSpacing(2)
		return layout

	#============================================
	def _button_for_action(self, action: PySide6.QtGui.QAction,
			parent: PySide6.QtWidgets.QWidget) -> PySide6.QtWidgets.QToolButton:
		"""Make an icon-first client of one live action with explicit a11y text."""
		button = PySide6.QtWidgets.QToolButton(parent)
		button.setDefaultAction(action)
		button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonIconOnly)
		button.setAutoRaise(False)
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setAccessibleName(action.text())
		button.setAccessibleDescription(action.toolTip())
		button.setToolTip(action.toolTip() or action.text())
		button.setFixedSize(28, 26)
		return button

	#============================================
	def _build_command_row(self) -> None:
		"""Keep file, history, view, grid, and snap direct at desktop widths."""
		for index, action in enumerate(self._command_actions):
			if index in (3, 5, 8, 11):
				self._command_layout.addSpacing(5)
			button = self._button_for_action(action, self._command_row)
			self._command_layout.addWidget(button)
			self._command_buttons.append(button)
			if index in (5, 6, 7, 8, 9, 10, 14):
				self._low_priority_buttons.append(button)
		self._more_button = PySide6.QtWidgets.QToolButton(self._command_row)
		self._more_button.setObjectName("authoring-more-menu")
		self._more_button.setText(self.tr("More"))
		self._more_button.setPopupMode(
			PySide6.QtWidgets.QToolButton.ToolButtonPopupMode.InstantPopup,
		)
		self._more_button.setAccessibleName(self.tr("More authoring commands"))
		self._more_button.setToolTip(self.tr("More authoring commands"))
		more_menu = PySide6.QtWidgets.QMenu(self._more_button)
		for index in (5, 6, 7, 8, 9, 10, 14):
			more_menu.addAction(self._command_actions[index])
		self._more_button.setMenu(more_menu)
		self._more_button.setVisible(False)
		self._command_layout.addWidget(self._more_button)
		self._command_layout.addStretch(1)

	#============================================
	def _build_tool_row(self, drawing_parameters: object,
			next_drawing_action: PySide6.QtGui.QAction) -> None:
		"""Expose primary tools and show defaults only for atom or bond authoring."""
		for index, (action, _icon_name) in enumerate(self._tool_actions):
			button = self._button_for_action(action, self._tool_row)
			self._tool_layout.addWidget(button)
			self._tool_buttons.append(button)
			if index >= 12:
				self._low_priority_buttons.append(button)
		self._more_tools_button = PySide6.QtWidgets.QToolButton(self._tool_row)
		self._more_tools_button.setObjectName("authoring-more-tools-menu")
		self._more_tools_button.setText(self.tr("More tools"))
		self._more_tools_button.setPopupMode(
			PySide6.QtWidgets.QToolButton.ToolButtonPopupMode.InstantPopup,
		)
		self._more_tools_button.setAccessibleName(self.tr("More tools"))
		self._more_tools_button.setAccessibleDescription(self.tr(
			"Choose an authoring tool that is hidden from the compact ribbon.",
		))
		self._more_tools_button.setToolTip(self.tr("More tools"))
		more_tools_menu = PySide6.QtWidgets.QMenu(self._more_tools_button)
		for action, _icon_name in self._tool_actions:
			more_tools_menu.addAction(action)
		self._more_tools_button.setMenu(more_tools_menu)
		self._more_tools_button.setVisible(False)
		self._tool_layout.addWidget(self._more_tools_button)
		self._tool_layout.addSpacing(5)
		self._drawing_parameters_client = (
			ferrum_qt.ferrum.drawing_parameters_client.
			FerrumNativeDrawingParametersClient(
				drawing_parameters, self._cancel_action, parent=self._tool_row,
			)
		)
		self._drawing_parameters_client.setObjectName("authoring-drawing-defaults")
		self._tool_layout.addWidget(self._drawing_parameters_client)
		self._next_drawing_button = self._button_for_action(
			next_drawing_action, self._tool_row,
		)
		self._tool_layout.addWidget(self._next_drawing_button)
		self._context_instruction = PySide6.QtWidgets.QLabel(self._tool_row)
		self._context_instruction.setObjectName("authoring-context-instruction")
		self._context_instruction.setWordWrap(False)
		self._context_instruction.setMinimumWidth(0)
		self._context_instruction.setAccessibleName(self.tr("Current tool instruction"))
		self._context_instruction.setSizePolicy(
			PySide6.QtWidgets.QSizePolicy.Policy.Expanding,
			PySide6.QtWidgets.QSizePolicy.Policy.Preferred,
		)
		self._tool_layout.addWidget(self._context_instruction, 1)

	#============================================
	def _apply_theme(self, theme_name: str) -> None:
		"""Project theme-specific icons without creating replacement actions."""
		if theme_name not in ("light", "dark"):
			theme_name = "light"
		ferrum_qt.widgets.icon_loader.set_theme(theme_name)
		ferrum_qt.widgets.icon_loader.reload_icons()
		for action, icon_name in self._tool_actions:
			icon = ferrum_qt.widgets.icon_loader.get_icon(icon_name)
			if icon.isNull():
				raise FileNotFoundError(
					f"Ferrum authoring icon is missing for {action.text()!r}: {icon_name!r}",
				)
			action.setIcon(icon)
		self._cancel_action.setIcon(ferrum_qt.widgets.icon_loader.get_icon("remove"))

	#============================================
	def _on_mode_triggered(self, mode_id: str, checked: bool) -> None:
		"""Forward only selected modes through the established dispatch adapter."""
		if checked:
			self.mode_selected.emit(mode_id)

	#============================================
	def _on_tool_toggled(self, action: PySide6.QtGui.QAction, checked: bool) -> None:
		"""Reflect one established tool lifecycle without replacing it.

		Each live QAction owns its real pointer lifecycle. The exclusive group
		unchecks its predecessor before the incoming QAction activates; this
		ribbon must not issue a later Cancel or restore a checkmark by itself.
		"""
		if action.isChecked():
			self._context_action = action
		elif self._context_action is action:
			self._context_action = next(
				(candidate for candidate, _icon in self._tool_actions if candidate.isChecked()),
				None,
			)
		self._refresh_context()

	#============================================
	def _refresh_context(self) -> None:
		"""Show drawing defaults only when their next-operation semantics apply."""
		active = self._context_action
		if active is None or not active.isChecked():
			active = next((action for action, _icon in self._tool_actions if action.isChecked()), None)
		uses_drawing_defaults = active is not None and active.text() in (
			"Add Atom at Point", "Draw Bond",
		)
		self._drawing_parameters_client.setVisible(uses_drawing_defaults)
		self._next_drawing_button.setVisible(uses_drawing_defaults)
		if uses_drawing_defaults:
			message = self.tr("Next atom/bond defaults.")
			accessible_message = self.tr(
				"Drawing defaults apply to the next atom or bond.",
			)
		elif active is not None:
			message = active.toolTip() or active.text()
			accessible_message = message
		else:
			message = self.tr("Choose a tool, then work directly on the canvas.")
			accessible_message = message
		self._context_instruction.setText(message)
		self._context_instruction.setAccessibleDescription(accessible_message)
