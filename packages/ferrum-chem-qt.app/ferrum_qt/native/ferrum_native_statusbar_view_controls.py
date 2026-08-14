"""Accessible status-bar clients for window-owned native View actions."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
def effective_percent(view: PySide6.QtWidgets.QGraphicsView | None) -> float | None:
	"""Return an exactly-supported uniform display scale without changing *view*."""
	if view is None:
		return None
	transform = view.transform()
	values = (
		transform.m11(), transform.m12(), transform.m13(), transform.m21(),
		transform.m22(), transform.m23(), transform.m31(), transform.m32(),
		transform.m33(),
	)
	if not all(math.isfinite(value) for value in values):
		return None
	if (
		transform.m13() != 0.0 or transform.m23() != 0.0 or transform.m33() != 1.0
		or transform.m12() != 0.0 or transform.m21() != 0.0
		or transform.m11() != transform.m22() or transform.m11() <= 0.0
	):
		return None
	return transform.m11() * 100.0


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
	"""Present five existing View actions without owning their display state."""

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
				self._zoom_page_button, self._zoom_content_button,
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
		percent = effective_percent(view)
		if percent is None:
			self._zoom_100_button.setText(self.tr("--"))
			if self._zoom_100_action.isEnabled():
				description = "Current zoom unavailable; activate to reset zoom to 100%."
			else:
				description = "Current zoom percentage unavailable; reset zoom unavailable."
		else:
			text = f"{percent:g}%"
			self._zoom_100_button.setText(text)
			description = f"Current zoom is {text}. Reset zoom to 100%."
		self._zoom_100_button.setMinimumWidth(0)
		self._zoom_100_button.setMinimumWidth(self._zoom_100_button.sizeHint().width())
		self.updateGeometry()
		self._zoom_100_button.setAccessibleDescription(self.tr(description))
		self._zoom_100_button.setToolTip(self.tr(description))
