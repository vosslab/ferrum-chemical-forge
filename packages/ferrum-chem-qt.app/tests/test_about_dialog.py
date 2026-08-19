"""Behavior checks for Ferrum's application-information dialog."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.about_dialog
import ferrum_qt.versioning


#============================================
def test_about_dialog_uses_only_ferrum_product_identity(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The visible identity, version, engine, and license facts are current."""
	del qapp
	dialog = ferrum_qt.dialogs.about_dialog.AboutDialog()
	try:
		visible_text = "\n".join(
			label.text()
			for label in dialog.findChildren(PySide6.QtWidgets.QLabel)
		)
		assert dialog.windowTitle() == "About Ferrum"
		assert ferrum_qt.versioning.application_version() in visible_text
		assert "Rust chemistry engine" in visible_text
		assert "GNU AGPL v3 only" in visible_text
		assert "OASA" not in visible_text and "BKChem" not in visible_text
	finally:
		dialog.deleteLater()
