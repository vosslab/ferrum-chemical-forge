"""Typed refusal presentation for the native main-window compatibility hook."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.refusal_presenter


#============================================
def show_refusal(
		window: object,
		request: ferrum_qt.dialogs.refusal_presenter.RefusalRequest,
		) -> None:
	"""Present one caller-classified refusal with optional copyable diagnostics."""
	if type(request) is not ferrum_qt.dialogs.refusal_presenter.RefusalRequest:
		raise TypeError("Ferrum refusal callers must provide an exact RefusalRequest")
	presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(request)
	window._last_refusal_technical_details = presentation.technical_details
	dialog = PySide6.QtWidgets.QMessageBox(window)
	dialog.setIcon(PySide6.QtWidgets.QMessageBox.Icon.Warning)
	dialog.setWindowTitle(window.tr(presentation.title))
	dialog.setText(window.tr(presentation.ordinary_text()))
	dialog.setStandardButtons(PySide6.QtWidgets.QMessageBox.StandardButton.Ok)
	if presentation.technical_details:
		dialog.setDetailedText(presentation.technical_details)
		dialog.setTextInteractionFlags(
			PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByMouse,
		)
		copy_button = dialog.addButton(
			window.tr("Copy Details"),
			PySide6.QtWidgets.QMessageBox.ButtonRole.ActionRole,
		)
		copy_button.setAccessibleName(window.tr("Copy technical details"))
		copy_button.clicked.connect(lambda: PySide6.QtWidgets.QApplication.clipboard().setText(
			presentation.technical_details or "",
		))
	dialog.exec()
