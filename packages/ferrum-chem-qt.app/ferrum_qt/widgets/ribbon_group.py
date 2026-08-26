"""Reusable labelled command group for Ferrum's task-oriented ribbon."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.authoring_ribbon_layout


#============================================
class RibbonGroup(PySide6.QtWidgets.QWidget):
	"""Project live actions as a labelled group with local supporting overflow."""

	#============================================
	def __init__(self, layout: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonGroupLayout,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Build one group without altering action behavior, state, or shortcuts."""
		super().__init__(parent)
		self.layout_data = layout
		self.setObjectName(f"ribbon-group-{layout.id}")
		self.setAccessibleName(self.tr(f"{layout.label_key} commands"))
		self.setAccessibleDescription(self.tr(f"Commands for the {layout.label_key} task."))
		self.setSizePolicy(PySide6.QtWidgets.QSizePolicy.Policy.Maximum,
			PySide6.QtWidgets.QSizePolicy.Policy.Preferred)
		root = PySide6.QtWidgets.QVBoxLayout(self)
		root.setContentsMargins(8, 3, 8, 3)
		root.setSpacing(2)
		self._actions = PySide6.QtWidgets.QWidget(self)
		self._action_layout = PySide6.QtWidgets.QHBoxLayout(self._actions)
		self._action_layout.setContentsMargins(0, 0, 0, 0)
		self._action_layout.setSpacing(3)
		root.addWidget(self._actions)
		caption = PySide6.QtWidgets.QLabel(self.tr(layout.label_key), self)
		caption.setObjectName(f"ribbon-group-caption-{layout.id}")
		caption.setAlignment(PySide6.QtCore.Qt.AlignmentFlag.AlignHCenter)
		root.addWidget(caption)
		self._primary_buttons: list[PySide6.QtWidgets.QToolButton] = []
		self._supporting_entries: list[tuple[
			PySide6.QtWidgets.QToolButton,
			ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry,
		]] = []
		self._overflow_actions: list[PySide6.QtGui.QAction] = []
		for entry in layout.entries:
			button = self._button_for_entry(entry)
			self._action_layout.addWidget(button)
			if entry.role == "primary":
				self._primary_buttons.append(button)
			else:
				self._supporting_entries.append((button, entry))
				self._overflow_actions.append(entry.action)
		self._more_button = PySide6.QtWidgets.QToolButton(self._actions)
		self._more_button.setObjectName(f"ribbon-more-{layout.id}")
		self._more_button.setText(self.tr("More"))
		self._more_button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
		self._more_button.setPopupMode(PySide6.QtWidgets.QToolButton.ToolButtonPopupMode.InstantPopup)
		self._more_button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		self._more_button.setAccessibleName(self.tr(layout.overflow_label_key))
		self._more_button.setToolTip(self.tr(layout.overflow_label_key))
		menu = PySide6.QtWidgets.QMenu(self._more_button)
		for action in self._overflow_actions:
			menu.addAction(action)
		self._more_button.setMenu(menu)
		self._action_layout.addWidget(self._more_button)
		self._more_button.setVisible(False)
		self._overflow_sync_pending = False
		self._syncing_overflow = False
		self._overflow_sync_timer = PySide6.QtCore.QTimer(self)
		self._overflow_sync_timer.setSingleShot(True)
		self._overflow_sync_timer.timeout.connect(self._sync_overflow)

	#============================================
	def resizeEvent(self, event: PySide6.QtGui.QResizeEvent) -> None:
		"""Keep primaries visible and collapse only this group's supporting clients."""
		super().resizeEvent(event)
		self._schedule_overflow_sync()

	#============================================
	def showEvent(self, event: PySide6.QtGui.QShowEvent) -> None:
		"""Measure after Qt assigns the active tab's real available width."""
		super().showEvent(event)
		self._schedule_overflow_sync()

	#============================================
	def direct_button_for(self, action: PySide6.QtGui.QAction) -> PySide6.QtWidgets.QToolButton | None:
		"""Return one direct action client for semantic UI tests and diagnostics."""
		return next((button for button in self.findChildren(PySide6.QtWidgets.QToolButton)
			if button.defaultAction() is action), None)

	#============================================
	def _button_for_entry(self, entry: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonEntry,
			) -> PySide6.QtWidgets.QToolButton:
		"""Make a text-labelled client delegating all state to the existing action."""
		button = PySide6.QtWidgets.QToolButton(self._actions)
		button.setDefaultAction(entry.action)
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setAccessibleName(entry.action.text())
		button.setAccessibleDescription(entry.action.toolTip() or entry.action.text())
		button.setToolTip(entry.action.toolTip() or entry.action.text())
		if entry.role == "primary":
			button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
			button.setIconSize(PySide6.QtCore.QSize(32, 32))
			button.setMinimumSize(72, 70)
		else:
			button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
			button.setIconSize(PySide6.QtCore.QSize(18, 18))
			button.setMinimumHeight(30)
		return button

	#============================================
	def _schedule_overflow_sync(self) -> None:
		"""Coalesce layout-triggered resize events into one stable reconciliation."""
		if self._overflow_sync_pending or self._syncing_overflow:
			return
		self._overflow_sync_pending = True
		self._overflow_sync_timer.start(0)

	#============================================
	def _sync_overflow(self) -> None:
		"""Apply one priority-aware overflow partition without resize recursion."""
		self._overflow_sync_pending = False
		if self._syncing_overflow or not self._supporting_entries:
			return
		self._syncing_overflow = True
		try:
			visible = {button for button, _entry in self._supporting_entries}
			available = self._actions.contentsRect().width()
			if available > 0 and self._width_for(visible, False) > available:
				for priority in ("normal", "required"):
					for button, entry in reversed(self._supporting_entries):
						if entry.priority != priority:
							continue
						visible.remove(button)
						if self._width_for(visible, True) <= available:
							break
					if self._width_for(visible, True) <= available:
						break
			overflow_visible = len(visible) != len(self._supporting_entries)
			for button, _entry in self._supporting_entries:
				if button.isVisible() != (button in visible):
					button.setVisible(button in visible)
			if self._more_button.isVisible() != overflow_visible:
				self._more_button.setVisible(overflow_visible)
		finally:
			self._syncing_overflow = False

	#============================================
	def _width_for(self, supporting: set[PySide6.QtWidgets.QToolButton],
			include_more: bool) -> int:
		"""Measure the direct clients that remain visible in one group."""
		buttons = self._primary_buttons + [
			button for button, _entry in self._supporting_entries if button in supporting
		]
		if include_more:
			buttons.append(self._more_button)
		return sum(button.sizeHint().width() for button in buttons) + self._action_layout.spacing() * (
			len(buttons) - 1
		)
