"""Verified full-window and dialog surfaces for Ferrum documentation capture."""

# Standard Library
import pathlib
import shutil
import subprocess

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
class CaptureError(RuntimeError):
	"""A scene did not reach its documented, observable ready state."""


#============================================
def capture_with_qt(window: PySide6.QtWidgets.QMainWindow, output: pathlib.Path) -> None:
	"""Capture the same visible top-level Ferrum window through its current screen."""
	handle = window.windowHandle()
	if handle is None or handle.screen() is None:
		raise CaptureError("Ferrum window has no screen for the Qt capture fallback")
	pixmap = handle.screen().grabWindow(window.winId())
	if pixmap.isNull():
		pixmap = window.grab()
	if pixmap.isNull() or not pixmap.save(str(output), "PNG"):
		raise CaptureError("Qt could not capture the visible Ferrum window")


#============================================
def capture_with_easy_screenshot(window: PySide6.QtWidgets.QMainWindow,
		output: pathlib.Path) -> bool:
	"""Attempt the optional macOS backend for this exact titled Ferrum window."""
	command = shutil.which("screenshot")
	if command is None:
		return False
	result = subprocess.run(
		[command, "-A", PySide6.QtWidgets.QApplication.applicationName(),
			"-t", window.windowTitle(), "-f", str(output)],
		capture_output=True, text=True, check=False,
	)
	return result.returncode == 0 and output.is_file() and output.stat().st_size > 0


#============================================
def verify_full_window_capture_surface(
		window: PySide6.QtWidgets.QMainWindow, output: pathlib.Path,
		expected_size: PySide6.QtCore.QSize,
		) -> None:
	"""Require the expected window, ribbon, status bar, and PNG geometry."""
	if window.size() != expected_size:
		raise CaptureError(
			f"Ferrum capture window is {window.width()}x{window.height()}, "
			f"not the required full-window {expected_size.width()}x"
			f"{expected_size.height()} surface"
		)
	ribbon = window.findChild(PySide6.QtWidgets.QToolBar, "ferrum-authoring-ribbon")
	if ribbon is None or not ribbon.isVisible():
		raise CaptureError("Ferrum capture requires the visible authoring ribbon")
	if not window.statusBar().isVisible():
		raise CaptureError("Ferrum capture requires the visible status bar")
	image = PySide6.QtGui.QImage(str(output))
	if image.isNull() or image.width() < 200 or image.height() < 200:
		raise CaptureError("capture output is not a usable window PNG")
	if image.width() * expected_size.height() != image.height() * expected_size.width():
		raise CaptureError(
			f"capture backend produced {image.width()}x{image.height()}, not the required "
			"full Ferrum window aspect ratio. The backend likely included window "
			"decoration or cropped the application; use the Qt backend or configure "
			"the window capture backend for only the Ferrum application surface."
		)


#============================================
def save_dialog_over_window(window: PySide6.QtWidgets.QMainWindow,
		dialog: PySide6.QtWidgets.QDialog, output: pathlib.Path,
		background: PySide6.QtGui.QPixmap | None = None,
		) -> None:
	"""Capture one real visible child dialog over the complete application surface."""
	if not dialog.isVisible():
		raise CaptureError("Ferrum dialog overlay is not visible for capture")
	position = dialog.frameGeometry().topLeft() - window.frameGeometry().topLeft()
	if background is None:
		dialog.hide()
		PySide6.QtWidgets.QApplication.processEvents()
		pixmap = window.grab()
		dialog.show()
		dialog.raise_()
		PySide6.QtWidgets.QApplication.processEvents()
	else:
		pixmap = background.copy()
	dialog_pixmap = dialog.grab()
	painter = PySide6.QtGui.QPainter(pixmap)
	painter.drawPixmap(position, dialog_pixmap)
	painter.end()
	if pixmap.isNull() or not pixmap.save(str(output), "PNG"):
		raise CaptureError("Qt could not capture the visible Ferrum dialog overlay")
	_verify_overlay_window_chrome(pixmap, output, position, dialog_pixmap.size())


#============================================
def _verify_overlay_window_chrome(background: PySide6.QtGui.QPixmap,
		output: pathlib.Path, position: PySide6.QtCore.QPoint,
		dialog_size: PySide6.QtCore.QSize) -> None:
	"""Prove an overlay retained the complete live window above and below itself."""
	background_image = background.toImage()
	overlay_image = PySide6.QtGui.QImage(str(output))
	if background_image.size() != overlay_image.size():
		raise CaptureError("Ferrum overlay capture changed the full-window surface dimensions")
	width = background_image.width()
	height = background_image.height()
	top_height = max(0, min(position.y(), height))
	bottom_start = max(0, min(position.y() + dialog_size.height(), height))
	if top_height <= 0 or bottom_start >= height:
		raise CaptureError("Ferrum overlay does not leave visible ribbon and status-bar surfaces")
	if (
		overlay_image.copy(0, 0, width, top_height)
		!= background_image.copy(0, 0, width, top_height)
		or overlay_image.copy(0, bottom_start, width, height - bottom_start)
		!= background_image.copy(0, bottom_start, width, height - bottom_start)
		):
		raise CaptureError("Ferrum overlay failed to preserve its ribbon, tab, or status surface")
