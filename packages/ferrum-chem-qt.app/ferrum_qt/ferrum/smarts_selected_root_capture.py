"""One-shot viewport capture for the private live SMARTS selected-query token."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.smarts_selected_root_contract


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSmartsSelectedQueryAvailabilityV1:
	"""Copied closed admission facts for one capture-owned opaque query token."""

	available: bool
	category: object | None
	reason: object | None
	recovery: object | None


#============================================
class FerrumSmartsSelectedRootCaptureController(PySide6.QtCore.QObject):
	"""Turn one renderer hit into an opaque selected-query token, then forget it."""

	def __init__(self, window: PySide6.QtWidgets.QMainWindow, dock: object) -> None:
		super().__init__(window)
		self._window = window
		self._dock = dock
		self._viewport: PySide6.QtWidgets.QWidget | None = None
		self._target: object | None = None
		self._ready_tab: object | None = None
		self._selected_query_token: object | None = None

	#============================================
	def begin(self) -> None:
		"""Install a temporary point-only capture after retiring older authoring input."""
		self.clear_ready_v1()
		outcome = self._window.begin_smarts_selected_root_capture()
		contract = ferrum_qt.ferrum.smarts_selected_root_contract
		if isinstance(outcome, contract.FerrumSmartsSelectedRootCaptureUnavailable):
			self._dock._selected_capture_refused_v1(outcome.message)
			return
		self._target = outcome
		self._viewport = outcome.viewport
		self._viewport.installEventFilter(self)
		self._viewport.setFocus()
		self._dock._selected_capture_started_v1()

	#============================================
	def cancel(self, message: str | None) -> None:
		"""Cancel the event capture without retaining a renderer selection."""
		if self._viewport is not None:
			self._viewport.removeEventFilter(self)
		self._viewport = None
		self._target = None
		if message is not None:
			self._dock._selected_capture_refused_v1(message)

	#============================================
	def clear_ready_v1(self) -> None:
		"""Drop the uninspectable capability retained only by this capture owner."""
		self._selected_query_token = None
		self._ready_tab = None

	#============================================
	def is_ready_for(self, tab: object) -> bool:
		"""Expose only selected-source readiness to the presentation controller."""
		return self._selected_query_token is not None and self._ready_tab is tab

	#============================================
	def selected_query_availability_v1(self, tab: object | None,
			) -> FerrumSmartsSelectedQueryAvailabilityV1:
		"""Copy native token admission facts after enforcing this owner's tab identity."""
		token = self._selected_query_token
		if token is None or self._ready_tab is not tab:
			return FerrumSmartsSelectedQueryAvailabilityV1(False, None, None, None)
		readiness = tab.live_smarts_selected_query_readiness_v1(token)
		return FerrumSmartsSelectedQueryAvailabilityV1(
			bool(readiness.available), readiness.category, readiness.reason, readiness.recovery,
		)

	#============================================
	def is_armed_v1(self) -> bool:
		"""Report whether the viewport still owns one uncaptured molecule choice."""
		return self._viewport is not None and self._target is not None

	#============================================
	def consume_selected_query_v1(self, tab: object, per_molecule_limit: int,
			total_limit: int) -> object:
		"""Consume the token synchronously at live-query admission, then forget it."""
		token = self._selected_query_token
		if token is None or self._ready_tab is not tab:
			raise RuntimeError("Ferrum selected molecule query is not ready")
		self.clear_ready_v1()
		return self._window.run_smarts_selected_root_query(
			tab, token, per_molecule_limit, total_limit,
		)

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Consume only the one pointer choice and closed cancellation gestures."""
		if watched is not self._viewport:
			return super().eventFilter(watched, event)
		if event.type() == PySide6.QtCore.QEvent.Type.FocusOut:
			self.cancel("Molecule choice cancelled because the canvas lost focus.")
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			if event.key() == PySide6.QtCore.Qt.Key.Key_Escape:
				self.cancel("Molecule choice cancelled. Choose one molecule on the canvas to try again.")
				return True
			return False
		if event.type() != PySide6.QtCore.QEvent.Type.MouseButtonPress:
			return False
		if event.button() == PySide6.QtCore.Qt.MouseButton.RightButton:
			self.cancel("Molecule choice cancelled. Choose one molecule on the canvas to try again.")
			return True
		if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
			return False
		self._capture_at(event.position().toPoint())
		return True

	#============================================
	def _capture_at(self, point: PySide6.QtCore.QPoint) -> None:
		"""Ask Rust for one root and consume that generic selection immediately."""
		target = self._target
		if target is None:
			self.cancel("Molecule choice is no longer current. Choose one molecule again.")
			return
		outcome = self._window.capture_smarts_selected_root_query(target, point)
		contract = ferrum_qt.ferrum.smarts_selected_root_contract
		if isinstance(outcome, contract.FerrumSmartsSelectedRootCaptureUnavailable):
			self.cancel(outcome.message)
			return
		if isinstance(outcome, contract.FerrumSmartsSelectedRootCaptureRejected):
			message, _ = self._dock._closed_failure_message(outcome.error)
			self.cancel(message)
			return
		self.cancel(None)
		self._ready_tab = outcome.tab
		self._selected_query_token = outcome.token
		self._dock._selected_capture_ready_v1(outcome.tab)

	#============================================
	def cancel_for_pointer_authoring(self, clear_status: bool) -> None:
		"""Cancel this one capture through the window's explicit handoff contract."""
		self.cancel(None)
		if clear_status:
			self._window.statusBar().clearMessage()
