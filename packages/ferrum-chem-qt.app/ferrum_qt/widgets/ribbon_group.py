"""Reusable labelled command group for Ferrum's task-oriented ribbon."""

# Standard Library
import dataclasses
import enum

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.authoring_ribbon_layout
import ferrum_qt.ribbon_contract


#============================================
@dataclasses.dataclass(frozen=True)
class _Placement:
	"""A single two-row compact-grid placement for one entry."""

	entry: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry
	row: int
	column: int
	row_span: int
	column_span: int
	presentation: str


#============================================
class RibbonGroupDisplayState(enum.Enum):
	"""The bounded presentation choices owned by one task group."""

	EXPANDED = "expanded"
	COMPACT = "compact"
	COLLAPSED = "collapsed"


#============================================
class _RibbonActionButton(PySide6.QtWidgets.QToolButton):
	"""Keep a declarative display caption separate from a registry QAction label."""

	#============================================
	def __init__(self, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Create one action client with a click-through compact caption layer."""
		super().__init__(parent)
		self._caption = PySide6.QtWidgets.QLabel(self)
		self._caption.setProperty("ribbonControlCaption", "true")
		self._caption.setAlignment(
			PySide6.QtCore.Qt.AlignmentFlag.AlignHCenter
			| PySide6.QtCore.Qt.AlignmentFlag.AlignVCenter,
		)
		self._caption.setAttribute(
			PySide6.QtCore.Qt.WidgetAttribute.WA_TransparentForMouseEvents,
		)
		self._caption.hide()

	#============================================
	def set_ribbon_caption(self, caption: str | None) -> None:
		"""Expose a short YAML-owned caption without changing the QAction label."""
		self.setProperty("ribbonControlCaptionVisible", "true" if caption is not None else "false")
		self._caption.setText(caption or "")
		self._caption.setVisible(caption is not None)

	#============================================
	def resizeEvent(self, event: PySide6.QtGui.QResizeEvent) -> None:
		"""Reserve the lower edge for the caption at every fixed control size."""
		super().resizeEvent(event)
		self._caption.setGeometry(1, self.height() - 10, max(0, self.width() - 2), 9)


#============================================
class RibbonGroup(PySide6.QtWidgets.QWidget):
	"""Project registry actions as a labelled group in one explicit state."""

	_ROW_COUNT = 2

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
		self._action_layout = PySide6.QtWidgets.QGridLayout(self._actions)
		self._action_layout.setContentsMargins(0, 0, 0, 0)
		self._action_layout.setHorizontalSpacing(metrics.action_spacing)
		self._action_layout.setVerticalSpacing(metrics.action_spacing)
		self._actions.setFixedHeight(metrics.compact_grid_height)
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
		self._button_by_entry: dict[ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry,
			PySide6.QtWidgets.QToolButton] = {}
		for entry in layout.entries:
			button = self._button_for_entry(entry)
			self._direct_buttons.append((button, entry))
			self._button_by_entry[entry] = button
		self._more_button = self._popup_button(
			f"ribbon-overflow-{layout.id}", self.tr(layout.overflow_label_key),
		)
		self._more_menu = PySide6.QtWidgets.QMenu(self._more_button)
		self._more_button.setMenu(self._more_menu)
		self._display_state = RibbonGroupDisplayState.COLLAPSED
		self.set_display_state(RibbonGroupDisplayState.EXPANDED)

	#============================================
	@property
	def display_state(self) -> RibbonGroupDisplayState:
		"""Return the tab allocator's currently requested presentation state."""
		return self._display_state

	#============================================
	def set_display_state(self, state: RibbonGroupDisplayState) -> None:
		"""Expose one known presentation state and reflow buttons for layout height."""
		if state is self._display_state and self._visible_clients_match(state):
			return
		focused_widget = PySide6.QtWidgets.QApplication.focusWidget()
		focused_direct_action = self._focused_direct_action()
		focused_overflow = focused_widget is self._more_button
		self._display_state = state
		visible_entries = self._visible_entries(state)
		visible_actions = {entry.action for entry in visible_entries}
		for button, entry in self._direct_buttons:
			self._apply_entry_presentation(button, entry, state)
			button.setVisible(entry.action in visible_actions)
		self._rebuild_layout(visible_entries, bool(self._overflow_entries(state)))
		self._rebuild_overflow_menu(self._overflow_entries(state))
		self._more_button.setVisible(state is not RibbonGroupDisplayState.EXPANDED
			and bool(self._overflow_entries(state)))
		self.updateGeometry()
		if focused_direct_action is not None:
			target = self.focus_target_for(focused_direct_action)
			if target is not None and target is not focused_widget:
				target.setFocus(PySide6.QtCore.Qt.FocusReason.OtherFocusReason)
			return
		if focused_overflow:
			if self._more_button.isVisible():
				self._more_button.setFocus(PySide6.QtCore.Qt.FocusReason.OtherFocusReason)
			elif visible_entries:
				self.direct_button_for(visible_entries[0].action).setFocus(
					PySide6.QtCore.Qt.FocusReason.OtherFocusReason,
				)

	#============================================
	def width_for(self, state: RibbonGroupDisplayState) -> int:
		"""Measure live grid hints plus label and group margins for one state."""
		visible_entries = self._visible_entries(state)
		overflow_entries = self._overflow_entries(state)
		_unused_placements, columns = self._layout_plan(visible_entries, state)
		if state is not RibbonGroupDisplayState.EXPANDED and overflow_entries:
			columns += 1
		metrics = ferrum_qt.ribbon_contract.METRICS
		row_width = columns * metrics.compact_control_size
		row_width += self._action_layout.horizontalSpacing() * max(0, columns - 1)
		margins = self._root_layout.contentsMargins()
		return max(row_width, self._caption.sizeHint().width()) + margins.left() + margins.right()

	#============================================
	def minimum_width_for(self, state: RibbonGroupDisplayState) -> int:
		"""Return a state floor derived from active presentation hints."""
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
		if self._display_state is not RibbonGroupDisplayState.EXPANDED and self._needs_overflow():
			return self._more_button
		return None

	#============================================
	def visible_actions(self) -> tuple[PySide6.QtGui.QAction, ...]:
		"""Return every ribbon action reachable exactly once in the visible state."""
		direct = tuple(entry.action for entry in self._visible_entries(self._display_state))
		overflow = tuple(entry.action for entry in self._overflow_entries(self._display_state))
		return tuple(dict.fromkeys(direct + overflow).keys())

	#============================================
	def _button_for_entry(self,
			entry: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry,
			) -> PySide6.QtWidgets.QToolButton:
		"""Make one tool client delegating all state to its existing action."""
		button = _RibbonActionButton(self._actions)
		button.setDefaultAction(entry.action)
		button.setProperty("ribbonRole", entry.role)
		button.setProperty("ribbonPresentation", entry.presentation)
		button.setProperty("ribbonCompactLabel", "true" if entry.compact_label is not None else "false")
		button.setProperty("ribbonAccent", self.layout_data.accent)
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setAccessibleName(entry.action.text())
		button.setAccessibleDescription(entry.action.toolTip() or entry.action.text())
		button.setToolTip(entry.action.toolTip() or entry.action.text())
		self._apply_entry_presentation(button, entry, RibbonGroupDisplayState.EXPANDED)
		return button

	#============================================
	def _rebuild_overflow_menu(
			self,
			overflow_entries: tuple[ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry, ...],
		) -> None:
		"""Build one canonical overflow route for hidden actions."""
		self._more_menu.clear()
		for entry in overflow_entries:
			self._more_menu.addAction(entry.action)

	#============================================
	def _rebuild_layout(self,
			visible_entries: tuple[ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry, ...],
			show_overflow: bool,
			) -> None:
		"""Reflow one deterministic two-row compact command grid."""
		while self._action_layout.count():
			item = self._action_layout.takeAt(0)
			if item is not None and item.widget() is not None:
				item.widget().setParent(self._actions)
		placements, columns = self._layout_plan(visible_entries, self._display_state)
		for placement in placements:
			button = self._button_by_entry[placement.entry]
			if placement.row_span == self._ROW_COUNT and placement.presentation != "large":
				self._action_layout.addWidget(
					button, placement.row, placement.column, placement.row_span,
					placement.column_span, PySide6.QtCore.Qt.AlignmentFlag.AlignVCenter,
				)
			else:
				self._action_layout.addWidget(
					button, placement.row, placement.column, placement.row_span,
					placement.column_span,
				)
		if show_overflow:
			overflow_column = columns
			self._action_layout.addWidget(
				self._more_button,
				1,
				overflow_column,
			)

	#============================================
	def _layout_plan(
			self,
			visible_entries: tuple[ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry, ...],
			state: RibbonGroupDisplayState,
		) -> tuple[tuple[_Placement, ...], int]:
		"""Place all visible entries into a fixed two-row compact grid."""
		if len(visible_entries) == 1:
			entry = visible_entries[0]
			presentation = self._presentation_for_state(entry, state)
			_unused_row_span, column_span = self._placement_spans(presentation)
			return (_Placement(entry, 0, 0, self._ROW_COUNT, column_span, presentation),), column_span
		placements: list[_Placement] = []
		occupied: set[tuple[int, int]] = set()
		for entry in visible_entries:
			presentation = self._presentation_for_state(entry, state)
			row_span, column_span = self._placement_spans(presentation)
			column = 0
			while True:
				for row in range(self._ROW_COUNT):
					cells = tuple(
						(row + row_offset, column + column_offset)
						for row_offset in range(row_span)
						for column_offset in range(column_span)
					)
					if row + row_span <= self._ROW_COUNT and not occupied.intersection(cells):
						placements.append(_Placement(
							entry, row, column, row_span, column_span, presentation,
						))
						occupied.update(cells)
						break
				else:
					column += 1
					continue
				break
		columns = max((placement.column + placement.column_span for placement in placements), default=0)
		return tuple(placements), columns


	#============================================
	def _placement_spans(self, presentation: str) -> tuple[int, int]:
		"""Return grid spans for Ferrum's three fixed control envelopes."""
		if presentation == "compact":
			return 1, 1
		if presentation == "standard":
			return 1, 2
		return self._ROW_COUNT, 2

	#============================================
	def _needs_overflow(self) -> bool:
		"""Expose whether any action is not shown directly in current state."""
		return bool(self._overflow_entries(self._display_state))

	#============================================
	def _popup_button(self, object_name: str, accessible_name: str) -> PySide6.QtWidgets.QToolButton:
		"""Create one icon-only popup trigger for hidden commands."""
		button = PySide6.QtWidgets.QToolButton(self._actions)
		button.setObjectName(object_name)
		button.setProperty("ribbonRole", "overflow")
		button.setProperty("ribbonAccent", self.layout_data.accent)
		button.setText("")
		button.setIcon(self.style().standardIcon(
			PySide6.QtWidgets.QStyle.StandardPixmap.SP_ToolBarVerticalExtensionButton,
		))
		button.setIconSize(PySide6.QtCore.QSize(12, 12))
		button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonIconOnly)
		button.setPopupMode(PySide6.QtWidgets.QToolButton.ToolButtonPopupMode.InstantPopup)
		metrics = ferrum_qt.ribbon_contract.METRICS
		button.setFixedSize(metrics.compact_control_size, metrics.compact_control_size)
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setAccessibleName(accessible_name)
		button.setAccessibleDescription(accessible_name)
		button.setToolTip(accessible_name)
		return button

	#============================================
	def _visible_entries(self, state: RibbonGroupDisplayState) -> tuple[
			ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry, ...]:
		"""Return entries directly exposed for a display state."""
		if state is RibbonGroupDisplayState.EXPANDED:
			return tuple(entry for _button, entry in self._direct_buttons)
		if state is RibbonGroupDisplayState.COMPACT:
			return tuple(entry for _button, entry in self._direct_buttons
				if entry.priority == "required")
		return tuple()

	#============================================
	def _overflow_entries(self, state: RibbonGroupDisplayState) -> tuple[
			ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry, ...]:
		"""Return entries hidden from direct control space by state."""
		if state is RibbonGroupDisplayState.EXPANDED:
			return tuple()
		if state is RibbonGroupDisplayState.COMPACT:
			visible = {entry for entry in self._visible_entries(state)}
			return tuple(entry for _button, entry in self._direct_buttons if entry not in visible)
		return tuple(entry for _button, entry in self._direct_buttons)

	#============================================
	def _presentation_for_state(
			self,
			entry: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry,
			state: RibbonGroupDisplayState,
		) -> str:
		"""Apply compact fallback outside expanded state; keep expanded presentation explicit."""
		if state is RibbonGroupDisplayState.EXPANDED:
			return entry.presentation
		return "compact"

	#============================================
	def _apply_entry_presentation(self, button: _RibbonActionButton,
			entry: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry,
			state: RibbonGroupDisplayState) -> None:
		"""Apply one concrete button geometry and text policy from presentation size."""
		presentation = self._presentation_for_state(entry, state)
		button.setProperty("ribbonPresentation", presentation)
		metrics = ferrum_qt.ribbon_contract.METRICS
		caption = entry.compact_label if presentation == "compact" else entry.presentation_label
		button.set_ribbon_caption(caption)
		if presentation == "compact":
			button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonIconOnly)
			icon_size = metrics.compact_caption_icon_size if caption is not None else metrics.compact_icon_size
			button.setIconSize(PySide6.QtCore.QSize(icon_size, icon_size))
		elif presentation == "standard":
			button.setToolButtonStyle(
				PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonIconOnly
				if caption is not None else PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextBesideIcon,
			)
			button.setIconSize(PySide6.QtCore.QSize(
				metrics.standard_icon_size, metrics.standard_icon_size,
			))
		else:
			button.setToolButtonStyle(
				PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonIconOnly
				if caption is not None else PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextUnderIcon,
			)
			button.setIconSize(PySide6.QtCore.QSize(metrics.large_icon_size, metrics.large_icon_size))
		button.setFixedSize(self._presentation_size(presentation))

	#============================================
	def _presentation_size(self, presentation: str) -> PySide6.QtCore.QSize:
		"""Return the fixed control envelope for one presentation size."""
		metrics = ferrum_qt.ribbon_contract.METRICS
		if presentation == "compact":
			return PySide6.QtCore.QSize(metrics.compact_control_size, metrics.compact_control_size)
		if presentation == "standard":
			return PySide6.QtCore.QSize(metrics.standard_control_width, metrics.standard_control_height)
		return PySide6.QtCore.QSize(metrics.large_control_width, metrics.large_control_height)

	#============================================
	def _focused_direct_action(self) -> PySide6.QtGui.QAction | None:
		"""Identify a direct action losing visibility during this state transition."""
		focus = PySide6.QtWidgets.QApplication.focusWidget()
		if not isinstance(focus, PySide6.QtWidgets.QToolButton):
			return None
		return next((entry.action for button, entry in self._direct_buttons if button is focus), None)

	#============================================
	def _visible_clients_match(self, state: RibbonGroupDisplayState) -> bool:
		"""Avoid redundant visibility mutation during repeated allocation passes."""
		visible_entries = set(self._visible_entries(state))
		overflow_entries = self._overflow_entries(state)
		return (
			all((not button.isHidden()) == (entry in visible_entries)
				for button, entry in self._direct_buttons)
			and self._more_button.isVisible() == (state is not RibbonGroupDisplayState.EXPANDED and bool(overflow_entries))
		)
