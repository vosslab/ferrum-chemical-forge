"""Standalone public startup for Ferrum's OASA-free bounded CDML editor."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_main_window


#============================================
def main(files: list[str] | None = None,
		smoke_exit_seconds: float | None = None) -> int:
	"""Start the native-only public host and optionally open CDML paths.

	This remains an internal standalone host while the ordinary product route
	uses ``ferrum_qt.app``.  It does not import the legacy MainWindow, OASA, or
	any worker-backed codec.
	"""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
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
