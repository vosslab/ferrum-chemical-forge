"""Offscreen behavior checks for the responsive Ferrum-Qt shell."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.setup.toolbar_setup
import ferrum_qt.widgets.mode_toolbar
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls


#============================================
def _lay_out_narrow_window(window: PySide6.QtWidgets.QMainWindow,
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Lay out a window at a narrow usable width without platform rendering."""
	window.resize(720, 500)
	qapp.processEvents()


#============================================
class _ToolbarWindow(PySide6.QtWidgets.QMainWindow):
	"""Minimal window receiving toolbar action callbacks during setup."""

	#============================================
	def on_undo(self) -> None:
		"""Accept the undo action without invoking a document backend."""

	#============================================
	def on_redo(self) -> None:
		"""Accept the redo action without invoking a document backend."""


#============================================
class _ModeManager:
	"""Expose the backend-admitted modes used by the toolbar setup."""

	#============================================
	def mode_names(self) -> tuple[str, ...]:
		"""Return the two modes supported by this isolated backend seam."""
		return ("edit", "draw")


#============================================
class _UndoStack:
	"""Supply the action enablement state needed during toolbar setup."""

	#============================================
	def canUndo(self) -> bool:
		"""Report that no undo is available in this blank document."""
		return False

	#============================================
	def canRedo(self) -> bool:
		"""Report that no redo is available in this blank document."""
		return False


#============================================
class _Document:
	"""Minimal document seam required to build a property dock."""

	#============================================
	def __init__(self) -> None:
		"""Expose the undo stack consumed by the toolbar setup."""
		self.undo_stack = _UndoStack()


#============================================
class _ThemeManager:
	"""Supply the current theme used by the icon loader."""

	#============================================
	def __init__(self) -> None:
		"""Use the packaged light icon set."""
		self.current_theme = "light"


#============================================
def test_narrow_mode_chooser_reuses_only_registered_modes(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The compact chooser exposes and selects the registered drawing modes."""
	window = PySide6.QtWidgets.QMainWindow()
	toolbar = ferrum_qt.widgets.mode_toolbar.ModeToolbar(window)
	toolbar.add_mode("edit", "Edit", tooltip="Edit drawing")
	toolbar.add_mode("draw", "Draw", tooltip="Draw bonds")
	toolbar.set_active_mode("edit")
	toolbar.add_compact_chooser()
	window.addToolBar(toolbar)
	selected = []
	toolbar.mode_selected.connect(selected.append)
	_lay_out_narrow_window(window, qapp)

	chooser = toolbar.findChild(PySide6.QtGui.QAction, "mode-chooser-action")
	assert chooser is not None and chooser.isVisible()
	assert [action.text() for action in chooser.menu().actions()] == ["Edit", "Draw"]
	chooser.menu().actions()[1].trigger()
	assert selected == ["draw"]
	window.close()


#============================================
def test_narrow_status_and_zoom_controls_remain_identifiable(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Narrow status controls retain accessible labels and usable zoom input."""
	window = PySide6.QtWidgets.QMainWindow()
	status = ferrum_qt.widgets.status_bar.StatusBar(window)
	zoom = ferrum_qt.widgets.zoom_controls.ZoomControls(window)
	status.addPermanentWidget(zoom)
	window.setStatusBar(status)
	_lay_out_narrow_window(window, qapp)

	status.update_coords(12.5, -4.0)
	status.update_mode("Draw")
	status.set_context_message("Choose a bond type")
	zoom_in = window.findChild(PySide6.QtWidgets.QPushButton, "zoom-in")
	slider = window.findChild(PySide6.QtWidgets.QSlider, "zoom-percentage-slider")
	coords = window.findChild(PySide6.QtWidgets.QLabel, "cursor-coordinates")
	mode = window.findChild(PySide6.QtWidgets.QLabel, "active-editing-mode")
	assert zoom_in is not None and zoom_in.accessibleName() == "Zoom in"
	assert slider is not None and not slider.isHidden() and slider.accessibleName()
	assert coords is not None and coords.toolTip() == "X: 12.5  Y: -4.0"
	assert mode is not None and mode.toolTip() == "Mode: Draw"
	window.close()


#============================================
def test_toolbar_setup_admits_only_registered_backend_modes(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The production setup never offers a mode absent from its manager."""
	window = _ToolbarWindow()
	widgets = ferrum_qt.setup.toolbar_setup.setup_toolbars(
		window, _ModeManager(), _Document(), _ThemeManager(),
	)
	_lay_out_narrow_window(window, qapp)
	chooser = widgets["mode_toolbar"].findChild(
		PySide6.QtGui.QAction, "mode-chooser-action"
	)
	admitted = {str(action.data()) for action in chooser.menu().actions()}
	assert admitted <= set(_ModeManager().mode_names())
	window.close()
