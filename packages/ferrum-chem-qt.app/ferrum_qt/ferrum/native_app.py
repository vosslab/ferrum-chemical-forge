"""Standalone public startup for Ferrum's Ferrum bounded CDML editor."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.main_window


#============================================
def main(files: list[str] | None = None,
		smoke_exit_seconds: float | None = None) -> int:
	"""Start the native-only public host and optionally open CDML paths.

	This remains an internal standalone host while the ordinary product route
	uses ``ferrum_qt.app``. It uses the same Ferrum document contracts as
	the ordinary application.
	"""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	window.show()
	for file_path in files or []:
		if not window.open_file_path(file_path):
			window.close()
			return 1
	if smoke_exit_seconds is not None:
		milliseconds = int(smoke_exit_seconds * 1000)
		PySide6.QtCore.QTimer.singleShot(milliseconds, window.close)
		PySide6.QtCore.QTimer.singleShot(milliseconds + 1, app.quit)
	return app.exec()
