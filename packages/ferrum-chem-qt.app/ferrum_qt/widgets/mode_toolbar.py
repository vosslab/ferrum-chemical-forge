"""Responsive action-reusing toolbar for Ferrum interaction modes."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
class ModeToolbar(PySide6.QtWidgets.QToolBar):
	"""Project declarative modes into shared actions and a compact chooser."""

	mode_selected = PySide6.QtCore.Signal(str)
	_COMPACT_BREAKPOINT = 1120

	#============================================
	def __init__(self, registry: object, parent: PySide6.QtWidgets.QWidget | None = None,
			compact_breakpoint: int = _COMPACT_BREAKPOINT) -> None:
		"""Build an empty toolbar whose actions always remain registry-owned."""
		super().__init__(self.tr("Modes"), parent)
		if not callable(getattr(registry, "get_qt_action", None)):
			raise TypeError("Ferrum mode toolbar needs an ActionRegistry-like client")
		if type(compact_breakpoint) is not int or compact_breakpoint <= 0:
			raise ValueError("Ferrum compact breakpoint must be a positive integer")
		self._registry = registry
		self._compact_breakpoint = compact_breakpoint
		self.setObjectName("mode-toolbar")
		self.setAccessibleName(self.tr("Drawing modes toolbar"))
		self.setAccessibleDescription(self.tr("Choose a canvas editing mode."))
		self.setMovable(False)
		self.setFloatable(False)
		self.setSizePolicy(
			PySide6.QtWidgets.QSizePolicy.Policy.Ignored,
			PySide6.QtWidgets.QSizePolicy.Policy.Fixed,
		)
		self.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
		self._group = PySide6.QtGui.QActionGroup(self)
		self._group.setExclusive(True)
		self._mode_actions: dict[str, PySide6.QtGui.QAction] = {}
		self._mode_clients: list[PySide6.QtWidgets.QAction] = []
		self._compact_hidden: list[PySide6.QtWidgets.QAction] = []
		self._chooser: PySide6.QtWidgets.QToolButton | None = None
		self._chooser_client: PySide6.QtWidgets.QAction | None = None

	#============================================
	def add_mode(self, mode_id: str, action_id: str) -> None:
		"""Add one checkable client of an existing action registry entry."""
		if not mode_id or mode_id in self._mode_actions:
			raise ValueError("Ferrum mode IDs must be unique and nonempty")
		action = self._registry.get_qt_action(action_id)
		if not isinstance(action, PySide6.QtGui.QAction):
			raise KeyError(f"Ferrum mode action is not available: {action_id}")
		action.setCheckable(True)
		action.setData(mode_id)
		action.triggered.connect(
			lambda checked, selected_mode_id=mode_id: self._on_triggered(selected_mode_id, checked),
		)
		self._group.addAction(action)
		button = PySide6.QtWidgets.QToolButton(self)
		button.setDefaultAction(action)
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextUnderIcon)
		button.setAccessibleName(action.text())
		button.setAccessibleDescription(action.toolTip())
		self._mode_actions[mode_id] = action
		self._mode_clients.append(self.addWidget(button))
		self._sync_compact_chooser()

	#============================================
	def add_separator_marker(self) -> None:
		"""Add a grouping marker that collapses with the full mode row."""
		self._compact_hidden.append(self.addSeparator())

	#============================================
	def add_compact_chooser(self) -> None:
		"""Add one popup that reuses the exact same live mode actions."""
		if self._chooser is not None:
			raise RuntimeError("Ferrum mode toolbar already has a compact chooser")
		chooser = PySide6.QtWidgets.QToolButton(self)
		chooser.setObjectName("mode-chooser")
		chooser.setPopupMode(PySide6.QtWidgets.QToolButton.ToolButtonPopupMode.InstantPopup)
		chooser.setToolButtonStyle(PySide6.QtCore.Qt.ToolButtonStyle.ToolButtonTextBesideIcon)
		chooser.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		chooser.setAccessibleName(self.tr("Mode chooser"))
		menu = PySide6.QtWidgets.QMenu(chooser)
		for action in self._mode_actions.values():
			menu.addAction(action)
		chooser.setMenu(menu)
		self._chooser = chooser
		self._chooser_client = self.addWidget(chooser)
		self._sync_compact_chooser()

	#============================================
	def set_active_mode(self, mode_id: str | None) -> None:
		"""Reflect ModeManager state without emitting another selection request."""
		for known_mode_id, action in self._mode_actions.items():
			action.setChecked(known_mode_id == mode_id)
		self._sync_compact_chooser()

	#============================================
	def apply_mode_manager(self, manager: object) -> None:
		"""Read active state from an injected ModeManager-like object."""
		active_mode_id = getattr(manager, "active_mode_id", None)
		value = getattr(active_mode_id, "value", active_mode_id)
		self.set_active_mode(value if type(value) is str else None)

	#============================================
	def resizeEvent(self, event: PySide6.QtGui.QResizeEvent) -> None:
		"""Update compact presentation when the host lays out the toolbar."""
		super().resizeEvent(event)
		self._sync_compact_chooser()

	#============================================
	def _on_triggered(self, mode_id: str, checked: bool) -> None:
		"""Forward a mode request to injected application glue only when selected."""
		if checked:
			self.mode_selected.emit(mode_id)
			self._sync_compact_chooser()

	#============================================
	def _sync_compact_chooser(self) -> None:
		"""Preserve one action identity in both full and compact presentations."""
		if self._chooser is None or self._chooser_client is None:
			return
		compact = self.window().width() < self._compact_breakpoint
		for client in self._mode_clients + self._compact_hidden:
			client.setVisible(not compact)
		self._chooser_client.setVisible(compact)
		active = next((action for action in self._mode_actions.values() if action.isChecked()), None)
		if active is not None:
			self._chooser.setText(self.tr(f"Mode: {active.text()}"))
			self._chooser.setIcon(active.icon())
			description = self.tr(f"Current mode is {active.text()}. Open to select a mode.")
			self._chooser.setToolTip(description)
			self._chooser.setAccessibleDescription(description)
