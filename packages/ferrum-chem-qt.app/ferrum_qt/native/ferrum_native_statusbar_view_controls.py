"""Accessible status-bar clients for window-owned native View actions."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_graphics_view


#============================================
class _CaptionToolButton(PySide6.QtWidgets.QToolButton):
	"""Keep a custom caption while making Return match ordinary button activation."""

	#============================================
	def keyPressEvent(self, event: PySide6.QtGui.QKeyEvent) -> None:
		"""Dispatch Return through the same enabled click signal as a mouse press."""
		if (
				event.key() in (PySide6.QtCore.Qt.Key.Key_Return, PySide6.QtCore.Qt.Key.Key_Enter)
				and self.isEnabled()
		):
			self.click()
			event.accept()
			return
		super().keyPressEvent(event)


#============================================
class FerrumNativeStatusBarViewControls(PySide6.QtWidgets.QWidget):
	"""Project five View actions and one absolute-scale request without owning state."""

	zoom_percent_requested = PySide6.QtCore.Signal(int)

	#============================================
	def __init__(
			self,
			zoom_out_action: PySide6.QtGui.QAction,
			zoom_100_action: PySide6.QtGui.QAction,
			zoom_in_action: PySide6.QtGui.QAction,
			zoom_page_action: PySide6.QtGui.QAction,
			zoom_content_action: PySide6.QtGui.QAction,
			parent: PySide6.QtWidgets.QWidget | None = None,
			) -> None:
		"""Build compact captions that dispatch only the supplied actions."""
		super().__init__(parent)
		self._zoom_out_action = zoom_out_action
		self._zoom_100_action = zoom_100_action
		self._zoom_in_action = zoom_in_action
		self._zoom_page_action = zoom_page_action
		self._zoom_content_action = zoom_content_action
		self._zoom_out_button = self._make_button("-", "Zoom out", "Decrease display zoom")
		self._zoom_100_button = self._make_button(
			"--", "Reset zoom to 100%", "Current zoom percentage unavailable; reset zoom unavailable.",
		)
		self._zoom_in_button = self._make_button("+", "Zoom in", "Increase display zoom")
		self._zoom_slider = PySide6.QtWidgets.QSlider(
			PySide6.QtCore.Qt.Orientation.Horizontal, self,
		)
		self._zoom_slider.setRange(
			ferrum_qt.native.ferrum_native_graphics_view.ZOOM_PERCENT_MINIMUM,
			ferrum_qt.native.ferrum_native_graphics_view.ZOOM_PERCENT_MAXIMUM,
		)
		self._zoom_slider.setSingleStep(
			ferrum_qt.native.ferrum_native_graphics_view.ZOOM_PERCENT_STEP,
		)
		self._zoom_slider.setPageStep(25)
		self._zoom_slider.setValue(100)
		self._zoom_slider.setMinimumWidth(48)
		self._zoom_slider.setMaximumWidth(120)
		self._zoom_slider.setSizePolicy(
			PySide6.QtWidgets.QSizePolicy.Policy.MinimumExpanding,
			PySide6.QtWidgets.QSizePolicy.Policy.Fixed,
		)
		self._zoom_slider.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		self._zoom_slider.setAccessibleName(self.tr("Zoom percentage slider"))
		self._zoom_slider.setAccessibleDescription(self.tr(
			"Set active display zoom from 10% to 1000%",
		))
		self._zoom_slider.setToolTip(self.tr("Drag to set active display zoom"))
		self._zoom_slider.valueChanged.connect(self.zoom_percent_requested.emit)
		self._zoom_page_button = self._make_button("Page", "Zoom to Page", "Fit the active page")
		self._zoom_content_button = self._make_button(
			"Content", "Zoom to Content", "Fit active document content",
		)
		self._zoom_out_button.clicked.connect(self._trigger_zoom_out)
		self._zoom_100_button.clicked.connect(self._trigger_zoom_100)
		self._zoom_in_button.clicked.connect(self._trigger_zoom_in)
		self._zoom_page_button.clicked.connect(self._trigger_zoom_page)
		self._zoom_content_button.clicked.connect(self._trigger_zoom_content)
		self._zoom_out_action.changed.connect(self._mirror_zoom_out_enabled)
		self._zoom_100_action.changed.connect(self._mirror_zoom_100_enabled)
		self._zoom_in_action.changed.connect(self._mirror_zoom_in_enabled)
		self._zoom_page_action.changed.connect(self._mirror_zoom_page_enabled)
		self._zoom_content_action.changed.connect(self._mirror_zoom_content_enabled)
		layout = PySide6.QtWidgets.QHBoxLayout(self)
		layout.setContentsMargins(0, 0, 0, 0)
		layout.setSpacing(2)
		for button in (
				self._zoom_out_button, self._zoom_100_button, self._zoom_in_button,
				self._zoom_slider, self._zoom_page_button, self._zoom_content_button,
				):
			layout.addWidget(button)
		self._mirror_zoom_out_enabled()
		self._mirror_zoom_100_enabled()
		self._mirror_zoom_in_enabled()
		self._mirror_zoom_page_enabled()
		self._mirror_zoom_content_enabled()

	#============================================
	def _make_button(
			self, caption: str, accessible_name: str, description: str,
			) -> PySide6.QtWidgets.QToolButton:
		"""Create one keyboard-reachable, visible-caption tool button."""
		button = _CaptionToolButton(self)
		button.setText(self.tr(caption))
		button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		button.setAccessibleName(self.tr(accessible_name))
		button.setAccessibleDescription(self.tr(description))
		button.setToolTip(self.tr(description))
		size = button.sizeHint()
		button.setMinimumHeight(size.height())
		if caption in ("-", "+"):
			button.setMinimumWidth(size.width())
		return button

	#============================================
	@PySide6.QtCore.Slot()
	def _trigger_zoom_out(self) -> None:
		"""Dispatch only the existing Zoom Out action."""
		self._zoom_out_action.trigger()

	#============================================
	@PySide6.QtCore.Slot()
	def _trigger_zoom_100(self) -> None:
		"""Dispatch only the existing Zoom to 100 percent action."""
		self._zoom_100_action.trigger()

	#============================================
	@PySide6.QtCore.Slot()
	def _trigger_zoom_in(self) -> None:
		"""Dispatch only the existing Zoom In action."""
		self._zoom_in_action.trigger()

	#============================================
	@PySide6.QtCore.Slot()
	def _trigger_zoom_page(self) -> None:
		"""Dispatch only the existing Zoom to Page action."""
		self._zoom_page_action.trigger()

	#============================================
	@PySide6.QtCore.Slot()
	def _trigger_zoom_content(self) -> None:
		"""Dispatch only the existing Zoom to Content action."""
		self._zoom_content_action.trigger()

	#============================================
	@PySide6.QtCore.Slot()
	def _mirror_zoom_out_enabled(self) -> None:
		"""Mirror only Zoom Out reachability."""
		self._zoom_out_button.setEnabled(self._zoom_out_action.isEnabled())

	#============================================
	@PySide6.QtCore.Slot()
	def _mirror_zoom_100_enabled(self) -> None:
		"""Mirror only Zoom to 100 percent reachability."""
		self._zoom_100_button.setEnabled(self._zoom_100_action.isEnabled())

	#============================================
	@PySide6.QtCore.Slot()
	def _mirror_zoom_in_enabled(self) -> None:
		"""Mirror only Zoom In reachability."""
		self._zoom_in_button.setEnabled(self._zoom_in_action.isEnabled())

	#============================================
	@PySide6.QtCore.Slot()
	def _mirror_zoom_page_enabled(self) -> None:
		"""Mirror only Zoom to Page reachability."""
		self._zoom_page_button.setEnabled(self._zoom_page_action.isEnabled())

	#============================================
	@PySide6.QtCore.Slot()
	def _mirror_zoom_content_enabled(self) -> None:
		"""Mirror only Zoom to Content reachability."""
		self._zoom_content_button.setEnabled(self._zoom_content_action.isEnabled())

	#============================================
	def refresh(self, view: PySide6.QtWidgets.QGraphicsView | None) -> None:
		"""Refresh only the observed active-view percentage and recovery wording."""
		percent = ferrum_qt.native.ferrum_native_graphics_view.effective_zoom_percent(view)
		if percent is None:
			self._zoom_100_button.setText(self.tr("--"))
			self._zoom_slider.setEnabled(False)
			if self._zoom_100_action.isEnabled():
				description = "Current zoom unavailable; activate to reset zoom to 100%."
			else:
				description = "Current zoom percentage unavailable; reset zoom unavailable."
		else:
			bounded = min(
				ferrum_qt.native.ferrum_native_graphics_view.ZOOM_PERCENT_MAXIMUM,
				max(
					ferrum_qt.native.ferrum_native_graphics_view.ZOOM_PERCENT_MINIMUM,
					round(percent),
					),
				)
			text = f"{bounded}%"
			self._zoom_100_button.setText(text)
			blocked = self._zoom_slider.blockSignals(True)
			self._zoom_slider.setValue(bounded)
			self._zoom_slider.blockSignals(blocked)
			self._zoom_slider.setEnabled(self._zoom_100_action.isEnabled())
			description = f"Current zoom is {text}. Reset zoom to 100%."
		self._zoom_100_button.setMinimumWidth(0)
		self._zoom_100_button.setMinimumWidth(self._zoom_100_button.sizeHint().width())
		self.updateGeometry()
		self._zoom_100_button.setAccessibleDescription(self.tr(description))
		self._zoom_100_button.setToolTip(self.tr(description))
