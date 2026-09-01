"""Reusable labelled command group for Ferrum's task-oriented ribbon."""

# Standard Library
import enum

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.authoring_ribbon_layout
import ferrum_qt.ribbon_contract


#============================================
class RibbonGroupDisplayState(enum.Enum):
	"""The bounded presentation choices owned by one task group."""

	EXPANDED = "expanded"
	COMPACT = "compact"
	COLLAPSED = "collapsed"


#============================================
class RibbonGroup(PySide6.QtWidgets.QWidget):
	"""Project registry actions as a labelled group in one explicit state."""

	#============================================
	def __init__(self, layout: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonGroupLayout,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Build action clients without taking ownership of QAction behavior."""
		super().__init__(parent)
		self.layout_data = layout
		self.setObjectName(f"ribbon-group-{layout.id}")
		self.setAttribute(PySide6.QtCore.Qt.WidgetAttribute.WA_StyledBackground, True)
		self.setProperty("ribbonGroup", "true")
		self.setProperty("ribbonAccent", layout.accent)
		self.setAccessibleName(self.tr(f"{layout.label_key} commands"))
		self.setAccessibleDescription(self.tr(f"Commands for the {layout.label_key} task."))
		self.setSizePolicy(PySide6.QtWidgets.QSizePolicy.Policy.Maximum,
			PySide6.QtWidgets.QSizePolicy.Policy.Preferred)
		metrics = ferrum_qt.ribbon_contract.METRICS
		self._root_layout = PySide6.QtWidgets.QVBoxLayout(self)
		self._root_layout.setContentsMargins(*metrics.group_margins)
		self._root_layout.setSpacing(metrics.group_label_spacing)
		self._actions = PySide6.QtWidgets.QWidget(self)
		self._action_layout = PySide6.QtWidgets.QHBoxLayout(self._actions)
		self._action_layout.setContentsMargins(0, 0, 0, 0)
		self._action_layout.setSpacing(metrics.action_spacing)
		self._root_layout.addWidget(self._actions)
		self._caption = PySide6.QtWidgets.QLabel(self.tr(layout.label_key), self)
		self._caption.setObjectName(f"ribbon-group-caption-{layout.id}")
		self._caption.setProperty("ribbonCaption", "true")
		self._caption.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignHCenter)
		self._root_layout.addWidget(self._caption)
		self._direct_buttons: list[tuple[
			PySide6.QtWidgets.QToolButton,
			ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry,
		]] = []
		for entry in layout.entries:
			button = self._button_for_entry(entry)
			self._direct_buttons.append((button, entry))
			if entry.role == "primary":
				self._action_layout.addWidget(button)
		self._supporting_columns = self._build_supporting_columns()
		for column in self._supporting_columns:
			self._action_layout.addWidget(column)
		self._more_button = self._popup_button(
			f"ribbon-more-{layout.id}", self.tr("More"), self.tr(layout.overflow_label_key),
		)
		self._more_menu = PySide6.QtWidgets.QMenu(self._more_button)
		for _button, entry in self._supporting_entries():
			self._more_menu.addAction(entry.action)
		self._more_button.setMenu(self._more_menu)
		self._action_layout.addWidget(self._more_button)
		self._group_button = self._popup_button(
			f"ribbon-group-popup-{layout.id}", self.tr("More"),
			self.tr(f"{layout.label_key} commands"),
		)
		self._group_menu = PySide6.QtWidgets.QMenu(self._group_button)
		for entry in layout.entries:
			self._group_menu.addAction(entry.action)
		self._group_button.setMenu(self._group_menu)
		self._action_layout.addWidget(self._group_button)
		self._display_state = RibbonGroupDisplayState.EXPANDED
		self.set_display_state(self._display_state)

	#============================================
	@property
	def display_state(self) -> RibbonGroupDisplayState:
		"""Return the tab allocator's currently requested presentation state."""
		return self._display_state

	#============================================
	def set_display_state(self, state: RibbonGroupDisplayState) -> None:
		"""Expose exactly the clients belonging to one measured presentation state."""
		if state is self._display_state and self._visible_clients_match(state):
			return
		focused_action = self._focused_direct_action()
		self._display_state = state
		for button, entry in self._direct_buttons:
			button.setVisible(
				state is RibbonGroupDisplayState.EXPANDED
				or state is RibbonGroupDisplayState.COMPACT and entry.role == "primary",
			)
		for column in self._supporting_columns:
			column.setVisible(state is RibbonGroupDisplayState.EXPANDED)
		self._more_button.setVisible(
			state is RibbonGroupDisplayState.COMPACT and bool(self._supporting_entries()),
		)
		self._group_button.setVisible(state is RibbonGroupDisplayState.COLLAPSED)
		self.updateGeometry()
		if focused_action is not None:
			target = self.focus_target_for(focused_action)
			if target is not None and target is not PySide6.QtWidgets.QApplication.focusWidget():
				target.setFocus(PySide6.QtCore.Qt.FocusReason.OtherFocusReason)

	#============================================
	def width_for(self, state: RibbonGroupDisplayState) -> int:
		"""Measure live control hints plus label and group margins for one state."""
		components = self._components_for_state(state)
		row_width = sum(component.width() for component in components)
		if len(components) > 1:
			row_width += self._action_layout.spacing() * (len(components) - 1)
		margins = self._root_layout.contentsMargins()
		return max(row_width, self._caption.sizeHint().width()) + margins.left() + margins.right()

	#============================================
	def minimum_width_for(self, state: RibbonGroupDisplayState) -> int:
		"""Return a state floor derived from current live control minimum hints."""
		return self.width_for(state)

	#============================================
	def direct_button_for(self, action: PySide6.QtGui.QAction) -> PySide6.QtWidgets.QToolButton | None:
		"""Return the stable direct client for one declared action."""
		return next((button for button, entry in self._direct_buttons if entry.action is action), None)

	#============================================
	def focus_target_for(self, action: PySide6.QtGui.QAction) -> PySide6.QtWidgets.QToolButton | None:
		"""Return the exposed keyboard client for an action in the current state."""
		button = self.direct_button_for(action)
		if button is not None and not button.isHidden():
			return button
		if self._display_state is RibbonGroupDisplayState.COMPACT:
			return self._more_button
		if self._display_state is RibbonGroupDisplayState.COLLAPSED:
			return self._group_button
		return None

	#============================================
	def visible_actions(self) -> tuple[PySide6.QtGui.QAction, ...]:
		"""Return every registry action reachable exactly once in the visible state."""
		if self._display_state is RibbonGroupDisplayState.COLLAPSED:
			return tuple(self._group_menu.actions())
		direct = tuple(entry.action for button, entry in self._direct_buttons if not button.isHidden())
		return direct + (
			tuple(self._more_menu.actions())
			if self._display_state is RibbonGroupDisplayState.COMPACT else ()
		)

	#============================================
	def _button_for_entry(self, entry: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry,
			) -> PySide6.QtWidgets.QToolButton:
		"""Make one labelled client delegating all state to its existing action."""
		button = PySide6.QtWidgets.QToolButton(self._actions)
		button.setDefaultAction(entry.action)
		button.setProperty("ribbonRole", entry.role)
		button.setProperty("ribbonAccent", self.layout_data.accent)
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setAccessibleName(entry.action.text())
		button.setAccessibleDescription(entry.action.toolTip() or entry.action.text())
		button.setToolTip(entry.action.toolTip() or entry.action.text())
		if entry.role == "primary":
			button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
			button.setIconSize(PySide6.QtCore.QSize(36, 36))
			metrics = ferrum_qt.ribbon_contract.METRICS
			width = ferrum_qt.ribbon_contract.quantized_control_width(
				button.sizeHint().width(), metrics.primary_minimum_width, metrics.primary_maximum_width,
			)
			button.setFixedSize(width, metrics.action_height)
		else:
			button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
			button.setIconSize(PySide6.QtCore.QSize(20, 20))
		return button

	#============================================
	def _build_supporting_columns(self) -> tuple[PySide6.QtWidgets.QWidget, ...]:
		"""Pack supporting commands into aligned two-row components."""
		metrics = ferrum_qt.ribbon_contract.METRICS
		buttons = tuple(button for button, entry in self._direct_buttons if entry.role == "supporting")
		columns: list[PySide6.QtWidgets.QWidget] = []
		for offset in range(0, len(buttons), 2):
			column_buttons = buttons[offset:offset + 2]
			column = PySide6.QtWidgets.QWidget(self._actions)
			column.setProperty("ribbonStack", "supporting")
			column_layout = PySide6.QtWidgets.QVBoxLayout(column)
			column_layout.setContentsMargins(0, 0, 0, 0)
			column_layout.setSpacing(metrics.supporting_row_spacing)
			width = ferrum_qt.ribbon_contract.quantized_control_width(
				max(button.sizeHint().width() for button in column_buttons),
				metrics.supporting_minimum_width, metrics.supporting_maximum_width,
			)
			row_height = (metrics.supporting_row_height if len(column_buttons) == 2
				else metrics.action_height)
			for button in column_buttons:
				button.setFixedSize(width, row_height)
				column_layout.addWidget(button)
			column.setFixedSize(width, metrics.action_height)
			columns.append(column)
		return tuple(columns)

	#============================================
	def _popup_button(self, object_name: str, text: str,
			accessible_name: str) -> PySide6.QtWidgets.QToolButton:
		"""Create one labelled, keyboard-reachable popup trigger."""
		button = PySide6.QtWidgets.QToolButton(self._actions)
		button.setObjectName(object_name)
		button.setProperty("ribbonRole", "overflow")
		button.setProperty("ribbonAccent", self.layout_data.accent)
		button.setText(text)
		button.setIcon(self.style().standardIcon(
			PySide6.QtWidgets.QStyle.StandardPixmap.SP_ToolBarHorizontalExtensionButton,
		))
		button.setIconSize(PySide6.QtCore.QSize(28, 28))
		button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
		button.setPopupMode(PySide6.QtWidgets.QToolButton.ToolButtonPopupMode.InstantPopup)
		metrics = ferrum_qt.ribbon_contract.METRICS
		button.setFixedSize(metrics.popup_width, metrics.action_height)
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setAccessibleName(accessible_name)
		button.setAccessibleDescription(accessible_name)
		button.setToolTip(accessible_name)
		return button

	#============================================
	def _components_for_state(self, state: RibbonGroupDisplayState) -> tuple[PySide6.QtWidgets.QWidget, ...]:
		"""List fixed-grid components participating in one presentation state."""
		if state is RibbonGroupDisplayState.COLLAPSED:
			return (self._group_button,)
		primary = tuple(button for button, entry in self._direct_buttons if entry.role == "primary")
		if state is RibbonGroupDisplayState.EXPANDED:
			return primary + self._supporting_columns
		return primary + ((self._more_button,) if self._supporting_entries() else ())

	#============================================
	def _supporting_entries(self) -> tuple[tuple[PySide6.QtWidgets.QToolButton,
			ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry], ...]:
		"""Return declared supporting entries in their YAML order."""
		return tuple((button, entry) for button, entry in self._direct_buttons
			if entry.role == "supporting")

	#============================================
	def _focused_direct_action(self) -> PySide6.QtGui.QAction | None:
		"""Identify a direct action losing visibility during this state transition."""
		focus = PySide6.QtWidgets.QApplication.focusWidget()
		if not isinstance(focus, PySide6.QtWidgets.QToolButton):
			return None
		return next((entry.action for button, entry in self._direct_buttons if button is focus), None)

	#============================================
	def _visible_clients_match(self, state: RibbonGroupDisplayState) -> bool:
		"""Avoid redundant visibility mutation during repeated page allocation."""
		return (
			all((not button.isHidden()) == (
				state is RibbonGroupDisplayState.EXPANDED
				or state is RibbonGroupDisplayState.COMPACT and entry.role == "primary"
			) for button, entry in self._direct_buttons)
			and all((not column.isHidden()) == (state is RibbonGroupDisplayState.EXPANDED)
				for column in self._supporting_columns)
			and (not self._more_button.isHidden()) == (
				state is RibbonGroupDisplayState.COMPACT and bool(self._supporting_entries())
			)
			and (not self._group_button.isHidden()) == (state is RibbonGroupDisplayState.COLLAPSED)
		)
