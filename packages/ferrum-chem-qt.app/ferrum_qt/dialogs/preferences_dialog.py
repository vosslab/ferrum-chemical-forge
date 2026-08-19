"""Application-preference form that returns typed intent without persistence."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.themes.theme_loader
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class PreferencesDialogResult:
	"""Application-owned settings selected by one accepted dialog."""

	theme: str
	remember_workspace: bool
	show_hex_grid: bool
	snap_authored_points_to_hex_grid: bool


#============================================
class PreferencesDialog(FerrumAccessibleDialog):
	"""Edit Ferrum application settings without applying or persisting them.

	The caller owns QSettings persistence and theme application.  This dialog
	therefore cannot modify a document, mutate a palette, or change settings
	when the user cancels it.
	"""

	#============================================
	def __init__(self, current: PreferencesDialogResult,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Build a focused application-settings form from current intent."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Preferences"))
		self.setMinimumWidth(430)
		self._build_ui(current)

	#============================================
	def _build_ui(self, current: PreferencesDialogResult) -> None:
		"""Create accessible controls and an explicit keyboard traversal order."""
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		intro = PySide6.QtWidgets.QLabel(self.tr(
			"These settings belong to Ferrum. Chemical documents keep their "
			"own drawing and chemistry data.",
		), self)
		intro.setWordWrap(True)
		intro.setAccessibleName(self.tr("Preferences scope"))
		intro.setAccessibleDescription(self.tr(
			"Ferrum settings do not alter the current chemical document.",
		))
		layout.addWidget(intro)

		form = PySide6.QtWidgets.QFormLayout()
		self._theme_combo = PySide6.QtWidgets.QComboBox(self)
		self._theme_combo.setAccessibleName(self.tr("Application theme"))
		self._theme_combo.setAccessibleDescription(self.tr(
			"Select the appearance theme for Ferrum windows.",
		))
		self._theme_combo.addItems(ferrum_qt.themes.theme_loader.get_theme_names())
		self._theme_combo.setCurrentText(current.theme)
		form.addRow(self.tr("Theme:"), self._theme_combo)

		self._show_hex_grid = PySide6.QtWidgets.QCheckBox(self.tr(
			"Show the drawing grid on document pages",
		), self)
		self._show_hex_grid.setAccessibleName(self.tr("Show hex grid"))
		self._show_hex_grid.setAccessibleDescription(self.tr(
			"Display the hexagonal guide grid on document pages.",
		))
		self._show_hex_grid.setChecked(current.show_hex_grid)
		form.addRow(self.tr("Canvas:"), self._show_hex_grid)

		self._snap_authored_points_to_hex_grid = PySide6.QtWidgets.QCheckBox(
			self.tr("Snap new and moved points to the hex grid"), self,
		)
		self._snap_authored_points_to_hex_grid.setAccessibleName(self.tr(
			"Snap new and moved points to hex grid",
		))
		self._snap_authored_points_to_hex_grid.setAccessibleDescription(self.tr(
			"Use the hex grid for newly authored and moved drawing points.",
		))
		self._snap_authored_points_to_hex_grid.setChecked(
			current.snap_authored_points_to_hex_grid,
		)
		form.addRow(self.tr("Drawing:"), self._snap_authored_points_to_hex_grid)

		self._remember_workspace = PySide6.QtWidgets.QCheckBox(self.tr(
			"Remember window size, toolbar, and Properties panel layout",
		), self)
		self._remember_workspace.setAccessibleName(self.tr("Remember workspace layout"))
		self._remember_workspace.setAccessibleDescription(self.tr(
			"Restore window size, toolbar visibility, and Properties panel layout.",
		))
		self._remember_workspace.setChecked(current.remember_workspace)
		form.addRow(self.tr("Workspace:"), self._remember_workspace)
		layout.addLayout(form)

		self._button_box = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok
			| PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
			parent=self,
		)
		self._ok_button = self._button_box.button(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok,
		)
		self._cancel_button = self._button_box.button(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
		)
		self._ok_button.setAccessibleName(self.tr("Apply preferences"))
		self._cancel_button.setAccessibleName(self.tr("Cancel preferences"))
		self._ok_button.setDefault(True)
		self._button_box.accepted.connect(self.accept)
		self._button_box.rejected.connect(self.reject)
		layout.addWidget(self._button_box)

		self.setTabOrder(self._theme_combo, self._show_hex_grid)
		self.setTabOrder(
			self._show_hex_grid, self._snap_authored_points_to_hex_grid,
		)
		self.setTabOrder(
			self._snap_authored_points_to_hex_grid, self._remember_workspace,
		)
		self.setTabOrder(self._remember_workspace, self._ok_button)
		self.setTabOrder(self._ok_button, self._cancel_button)
		self._theme_combo.setFocus(
			PySide6.QtCore.Qt.FocusReason.OtherFocusReason,
		)

	#============================================
	def selected_preferences(self) -> PreferencesDialogResult:
		"""Return the currently visible typed preference intent."""
		result = PreferencesDialogResult(
			theme=self._theme_combo.currentText(),
			remember_workspace=self._remember_workspace.isChecked(),
			show_hex_grid=self._show_hex_grid.isChecked(),
			snap_authored_points_to_hex_grid=(
				self._snap_authored_points_to_hex_grid.isChecked()
			),
		)
		return result

	#============================================
	@staticmethod
	def choose_preferences(
			current: PreferencesDialogResult,
			parent: PySide6.QtWidgets.QWidget | None = None,
			) -> PreferencesDialogResult | None:
		"""Return accepted typed intent, or ``None`` when the dialog is rejected."""
		dialog = PreferencesDialog(current, parent)
		result = dialog.exec()
		if result == PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return dialog.selected_preferences()
		return None
