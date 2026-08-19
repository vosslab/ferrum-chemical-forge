"""Accessible status display with no document or session ownership."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets


MIN_COORDS_WIDTH = 110
MIN_MODE_WIDTH = 120
PREFERRED_COORDS_WIDTH = 180
PREFERRED_MODE_WIDTH = 140


#============================================
class StatusBar(PySide6.QtWidgets.QStatusBar):
	"""Present host-supplied cursor, mode, and message state.

	The status bar retains only view text.  It does not inspect a canvas,
	document projection, or interaction controller.
	"""

	#============================================
	def __init__(self, parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Create one keyboard and screen-reader friendly status display."""
		super().__init__(parent)
		self.setAccessibleName(self.tr("Ferrum status bar"))
		self._message_label = PySide6.QtWidgets.QLabel(self)
		self._message_label.setAccessibleName(self.tr("Status message"))
		self._message_label.setMinimumWidth(0)
		self._message_label.setSizePolicy(
			PySide6.QtWidgets.QSizePolicy.Policy.Ignored,
			PySide6.QtWidgets.QSizePolicy.Policy.Preferred,
		)
		self.addWidget(self._message_label, 1)
		self._context_message = ""
		self._transient_message = ""
		self._message_timer = PySide6.QtCore.QTimer(self)
		self._message_timer.setSingleShot(True)
		self._message_timer.timeout.connect(self.clearMessage)
		self._coords_label = self._make_label(
			"cursor-coordinates", "Cursor coordinates", "X: --  Y: --",
			MIN_COORDS_WIDTH, PREFERRED_COORDS_WIDTH,
		)
		self._mode_label = self._make_label(
			"active-editing-mode", "Active editing mode", "Mode: None",
			MIN_MODE_WIDTH, PREFERRED_MODE_WIDTH,
		)
		self.addPermanentWidget(self._coords_label)
		self.addPermanentWidget(self._mode_label)

	#============================================
	def _make_label(self, object_name: str, accessible_name: str, text: str,
			minimum_width: int, maximum_width: int) -> PySide6.QtWidgets.QLabel:
		"""Build one compact permanent label whose full text remains exposed."""
		label = PySide6.QtWidgets.QLabel(self.tr(text), self)
		label.setObjectName(object_name)
		label.setAccessibleName(self.tr(accessible_name))
		label.setMinimumWidth(minimum_width)
		label.setMaximumWidth(maximum_width)
		label.setSizePolicy(
			PySide6.QtWidgets.QSizePolicy.Policy.Preferred,
			PySide6.QtWidgets.QSizePolicy.Policy.Preferred,
		)
		label.setToolTip(label.text())
		return label

	#============================================
	@property
	def context_message(self) -> str:
		"""Return persistent host guidance that follows temporary results."""
		return self._context_message

	#============================================
	@property
	def visible_message(self) -> str:
		"""Return text currently presented in the expandable message area."""
		return self._message_label.text()

	#============================================
	def update_coords(self, x: float, y: float) -> None:
		"""Display normalized scene coordinates supplied by the host adapter."""
		text = self.tr(f"X: {x:.1f}  Y: {y:.1f}")
		self._coords_label.setText(text)
		self._coords_label.setToolTip(text)

	#============================================
	def update_mode(self, name: str) -> None:
		"""Display the human-readable active mode supplied by ModeManager glue."""
		text = self.tr(f"Mode: {name}")
		self._mode_label.setText(text)
		self._mode_label.setToolTip(text)

	#============================================
	def set_context_message(self, text: str) -> None:
		"""Set persistent interaction guidance without changing document state."""
		self._context_message = text
		if not self._transient_message:
			self._message_label.setText(text)

	#============================================
	def showMessage(self, text: str, timeout: int = 0) -> None:
		"""Show a transient host result and restore context when it expires."""
		self._message_timer.stop()
		self._transient_message = text
		self._message_label.setText(text)
		self.messageChanged.emit(text)
		if timeout > 0:
			self._message_timer.start(timeout)

	#============================================
	def clearMessage(self) -> None:
		"""Restore persistent context after clearing a transient message."""
		self._message_timer.stop()
		if not self._transient_message:
			return
		self._transient_message = ""
		self._message_label.setText(self._context_message)
		self.messageChanged.emit("")

	#============================================
	def currentMessage(self) -> str:
		"""Return the active transient message using QStatusBar semantics."""
		return self._transient_message

	#============================================
	def show_message(self, text: str, timeout: int = 3000) -> None:
		"""Offer a readable snake-case convenience client for hosts."""
		self.showMessage(text, timeout)
