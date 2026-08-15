"""Application-owned preferences for the ordinary Rust-native window."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.config.preferences
import ferrum_qt.themes.theme_loader


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativePreferencesV1:
	"""Accepted application appearance and workspace-persistence choices."""

	theme: str
	remember_workspace: bool
	show_hex_grid: bool = True
	snap_authored_points_to_hex_grid: bool = True


#============================================
class FerrumNativePreferencesDialog(PySide6.QtWidgets.QDialog):
	"""Edit settings owned by the application rather than a CDML document."""

	#============================================
	def __init__(self, current: FerrumNativePreferencesV1,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Build a focused form that advertises only implemented preferences."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Preferences"))
		self.setMinimumWidth(430)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		intro = PySide6.QtWidgets.QLabel(self.tr(
			"These settings belong to Ferrum-Qt. Chemical documents keep their "
			"own drawing and chemistry data.",
		))
		intro.setWordWrap(True)
		layout.addWidget(intro)
		form = PySide6.QtWidgets.QFormLayout()
		self._theme = PySide6.QtWidgets.QComboBox()
		self._theme.setAccessibleName(self.tr("Application theme"))
		self._theme.addItems(ferrum_qt.themes.theme_loader.get_theme_names())
		index = self._theme.findText(current.theme)
		if index >= 0:
			self._theme.setCurrentIndex(index)
		form.addRow(self.tr("Theme:"), self._theme)
		self._show_hex_grid = PySide6.QtWidgets.QCheckBox(self.tr(
			"Show the drawing grid on document pages",
		))
		self._show_hex_grid.setAccessibleName(self.tr("Show hex grid"))
		self._show_hex_grid.setChecked(current.show_hex_grid)
		form.addRow(self.tr("Canvas:"), self._show_hex_grid)
		self._snap_authored_points_to_hex_grid = PySide6.QtWidgets.QCheckBox(self.tr(
			"Snap new and moved points to the hex grid",
		))
		self._snap_authored_points_to_hex_grid.setAccessibleName(self.tr(
			"Snap new and moved points to hex grid",
		))
		self._snap_authored_points_to_hex_grid.setToolTip(self.tr(
			"Use the hex grid for new and moved drawing points",
		))
		self._snap_authored_points_to_hex_grid.setChecked(
			current.snap_authored_points_to_hex_grid,
		)
		form.addRow(self.tr("Drawing:"), self._snap_authored_points_to_hex_grid)
		self._remember_workspace = PySide6.QtWidgets.QCheckBox(self.tr(
			"Remember window size, toolbar, and Properties panel layout",
		))
		self._remember_workspace.setAccessibleName(self.tr("Remember workspace layout"))
		self._remember_workspace.setChecked(current.remember_workspace)
		form.addRow(self.tr("Workspace:"), self._remember_workspace)
		layout.addLayout(form)
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok
			| PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
		)
		buttons.accepted.connect(self.accept)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)

	#============================================
	def preferences(self) -> FerrumNativePreferencesV1:
		"""Return the choices currently visible in the form."""
		return FerrumNativePreferencesV1(
			theme=self._theme.currentText(),
			remember_workspace=self._remember_workspace.isChecked(),
			show_hex_grid=self._show_hex_grid.isChecked(),
			snap_authored_points_to_hex_grid=(
				self._snap_authored_points_to_hex_grid.isChecked()
			),
		)

	#============================================
	@staticmethod
	def choose_preferences(parent: PySide6.QtWidgets.QWidget,
			current: FerrumNativePreferencesV1) -> FerrumNativePreferencesV1 | None:
		"""Return accepted application settings or preserve cancellation."""
		dialog = FerrumNativePreferencesDialog(current, parent)
		if dialog.exec() == PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return dialog.preferences()
		return None


#============================================
def remembered_workspace_preference(prefs: object) -> bool:
	"""Read the one boolean setting without treating string text as truthy."""
	value = prefs.value(
		ferrum_qt.config.preferences.Preferences.KEY_REMEMBER_WORKSPACE, True,
	)
	if type(value) is bool:
		return value
	if type(value) is int and value in (0, 1):
		return bool(value)
	if type(value) is str:
		normalized = value.strip().lower()
		if normalized in {"true", "1", "yes", "on"}:
			return True
		if normalized in {"false", "0", "no", "off"}:
			return False
	return True


#============================================
def hex_grid_visible_preference(prefs: object) -> bool:
	"""Read the application grid preference without treating arbitrary text as true."""
	value = prefs.value(
		ferrum_qt.config.preferences.Preferences.KEY_GRID_VISIBLE, True,
	)
	if type(value) is bool:
		return value
	if type(value) is int and value in (0, 1):
		return bool(value)
	if type(value) is str:
		normalized = value.strip().lower()
		if normalized in {"true", "1", "yes", "on"}:
			return True
		if normalized in {"false", "0", "no", "off"}:
			return False
	return True


#============================================
def hex_grid_snap_enabled_preference(prefs: object) -> bool:
	"""Read the authored-point preference without treating arbitrary text as true."""
	value = prefs.value(
		ferrum_qt.config.preferences.Preferences.KEY_GRID_SNAP_ENABLED, True,
	)
	if type(value) is bool:
		return value
	if type(value) is int and value in (0, 1):
		return bool(value)
	if type(value) is str:
		normalized = value.strip().lower()
		if normalized in {"true", "1", "yes", "on"}:
			return True
		if normalized in {"false", "0", "no", "off"}:
			return False
	return True


#============================================
def install_native_preferences_action(window: object,
		options_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install one application-settings action on the ordinary product window."""
	action = PySide6.QtGui.QAction(window.tr("Preferences..."), window)
	action.setToolTip(window.tr("Choose application appearance and workspace behavior"))
	action.triggered.connect(lambda _checked=False: _on_preferences(window))
	options_menu.addAction(action)
	return action


#============================================
def _on_preferences(window: object) -> None:
	"""Apply accepted settings while preserving document and cancellation state."""
	current = FerrumNativePreferencesV1(
		theme=window._theme_manager.current_theme,
		remember_workspace=remembered_workspace_preference(window._prefs),
		show_hex_grid=hex_grid_visible_preference(window._prefs),
		snap_authored_points_to_hex_grid=hex_grid_snap_enabled_preference(window._prefs),
	)
	chosen = FerrumNativePreferencesDialog.choose_preferences(window, current)
	if chosen is None:
		return
	window._prefs.set_value(
		ferrum_qt.config.preferences.Preferences.KEY_REMEMBER_WORKSPACE,
		chosen.remember_workspace,
	)
	window._prefs.set_value(
		ferrum_qt.config.preferences.Preferences.KEY_GRID_VISIBLE,
		chosen.show_hex_grid,
	)
	window._set_native_hex_grid_visible(chosen.show_hex_grid)
	window._prefs.set_value(
		ferrum_qt.config.preferences.Preferences.KEY_GRID_SNAP_ENABLED,
		chosen.snap_authored_points_to_hex_grid,
	)
	window._set_native_hex_grid_snap_enabled(
		chosen.snap_authored_points_to_hex_grid,
	)
	if not chosen.remember_workspace:
		window._prefs.remove_value(
			ferrum_qt.config.preferences.Preferences.KEY_WINDOW_GEOMETRY,
		)
		window._prefs.remove_value(
			ferrum_qt.config.preferences.Preferences.KEY_WINDOW_STATE,
		)
	if chosen.theme != current.theme:
		window._theme_manager.apply_theme(chosen.theme)
	window.statusBar().showMessage(window.tr("Preferences updated."), 3000)
