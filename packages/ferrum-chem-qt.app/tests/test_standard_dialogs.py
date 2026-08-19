"""Focused offscreen behavior checks for standard Ferrum application dialogs."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.preferences_dialog
import ferrum_qt.dialogs.theme_chooser_dialog
import ferrum_qt.themes.theme_loader


#============================================
def _preferences() -> ferrum_qt.dialogs.preferences_dialog.PreferencesDialogResult:
	"""Return one deliberately mixed application-preference state."""
	theme = ferrum_qt.themes.theme_loader.get_theme_names()[0]
	result = ferrum_qt.dialogs.preferences_dialog.PreferencesDialogResult(
		theme, False, True, False,
	)
	return result


#============================================
def test_preferences_dialog_returns_typed_visible_intent(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Changing form controls produces accepted caller-owned intent only."""
	del qapp
	dialog = ferrum_qt.dialogs.preferences_dialog.PreferencesDialog(_preferences())
	try:
		dialog._remember_workspace.setChecked(True)
		selected = dialog.selected_preferences()
		assert selected.theme == _preferences().theme and selected.remember_workspace
	finally:
		dialog.deleteLater()


#============================================
def test_preferences_dialog_starts_focus_and_rejects_with_escape(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Keyboard users start on the first control and can cancel safely."""
	dialog = ferrum_qt.dialogs.preferences_dialog.PreferencesDialog(_preferences())
	dialog.show()
	qapp.processEvents()
	try:
		assert dialog.focusWidget() is dialog._theme_combo
		PySide6.QtTest.QTest.keyClick(dialog, PySide6.QtCore.Qt.Key.Key_Escape)
		assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Rejected
	finally:
		dialog.deleteLater()


#============================================
def test_theme_chooser_preselects_and_returns_a_named_theme(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The theme chooser retains the active theme until a user selects another."""
	del qapp
	current_theme = ferrum_qt.themes.theme_loader.get_theme_names()[0]
	dialog = ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog(current_theme)
	try:
		initial = dialog.selected_theme()
		dialog._theme_list.setCurrentRow(0)
		selected = dialog.selected_theme()
		assert initial == current_theme and selected is not None
	finally:
		dialog.deleteLater()


#============================================
def test_theme_chooser_starts_at_list_and_has_default_accept(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The list is the keyboard start and Enter accepts the chosen theme."""
	current_theme = ferrum_qt.themes.theme_loader.get_theme_names()[0]
	dialog = ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog(current_theme)
	dialog.show()
	qapp.processEvents()
	try:
		assert dialog.focusWidget() is dialog._theme_list
		PySide6.QtTest.QTest.keyClick(dialog._theme_list, PySide6.QtCore.Qt.Key.Key_Return)
		assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Accepted
	finally:
		dialog.deleteLater()
