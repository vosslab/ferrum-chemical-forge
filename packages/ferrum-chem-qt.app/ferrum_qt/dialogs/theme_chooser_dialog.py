"""Theme-selection dialog that leaves theme application to ThemeManager."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog

# local repo modules
import ferrum_qt.themes.theme_loader


#============================================
class ThemeChooserDialog(FerrumAccessibleDialog):
	"""Let a user choose one available Ferrum theme without applying it.

	The caller owns the ThemeManager and applies an accepted selection.  This
	keeps palette mutation and QSettings persistence out of this short dialog.
	"""

	#============================================
	def __init__(self, current_theme: str,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Build a keyboard-accessible list with the current theme selected."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Choose Theme"))
		self.setMinimumWidth(280)
		self._build_ui(current_theme)

	#============================================
	def _build_ui(self, current_theme: str) -> None:
		"""Create the theme list, default action, and rejection path."""
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		intro = PySide6.QtWidgets.QLabel(self.tr(
			"Choose the appearance theme for Ferrum windows.",
		), self)
		intro.setWordWrap(True)
		intro.setAccessibleName(self.tr("Theme chooser instructions"))
		layout.addWidget(intro)

		self._theme_list = PySide6.QtWidgets.QListWidget(self)
		self._theme_list.setAccessibleName(self.tr("Available application themes"))
		self._theme_list.setAccessibleDescription(self.tr(
			"Select one theme, then choose OK to apply it.",
		))
		self._theme_list.addItems(ferrum_qt.themes.theme_loader.get_theme_names())
		matching_items = self._theme_list.findItems(
			current_theme, PySide6.QtCore.Qt.MatchFlag.MatchExactly,
		)
		if matching_items:
			self._theme_list.setCurrentItem(matching_items[0])
		elif self._theme_list.count() > 0:
			self._theme_list.setCurrentRow(0)
		self._theme_list.itemDoubleClicked.connect(self.accept)
		layout.addWidget(self._theme_list)

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
		self._ok_button.setAccessibleName(self.tr("Apply selected theme"))
		self._cancel_button.setAccessibleName(self.tr("Cancel theme selection"))
		self._ok_button.setDefault(True)
		self._button_box.accepted.connect(self.accept)
		self._button_box.rejected.connect(self.reject)
		layout.addWidget(self._button_box)
		self.setTabOrder(self._theme_list, self._ok_button)
		self.setTabOrder(self._ok_button, self._cancel_button)
		self._theme_list.setFocus(
			PySide6.QtCore.Qt.FocusReason.OtherFocusReason,
		)

	#============================================
	def selected_theme(self) -> str | None:
		"""Return the selected available theme, or ``None`` without a selection."""
		current_item = self._theme_list.currentItem()
		if current_item is None:
			return None
		selected = current_item.text()
		return selected

	#============================================
	@staticmethod
	def choose_theme(current_theme: str,
			parent: PySide6.QtWidgets.QWidget | None = None) -> str | None:
		"""Return an accepted theme name, or ``None`` for a rejected dialog."""
		dialog = ThemeChooserDialog(current_theme, parent)
		result = dialog.exec()
		if result == PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return dialog.selected_theme()
		return None
