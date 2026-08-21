"""One-shot viewport capture for the private live SMARTS selected-query token."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets


#============================================
class FerrumSmartsSelectedRootCaptureController(PySide6.QtCore.QObject):
	"""Turn one renderer hit into an opaque selected-query token, then forget it."""

	def __init__(self, window: PySide6.QtWidgets.QMainWindow, dock: object) -> None:
		super().__init__(window)
		self._window = window
		self._dock = dock
		self._viewport: PySide6.QtWidgets.QWidget | None = None
		self._tab: object | None = None
		self._ready_tab: object | None = None
		self._selected_query_token: object | None = None

	#============================================
	def begin(self) -> None:
		"""Install a temporary point-only capture after retiring older authoring input."""
		self.clear_ready_v1()
		self.cancel(None)
		for name in (
			"_cancel_structure_selection", "_cancel_catalog_placement",
			"_cancel_atom_insertion", "_cancel_line_gesture",
		):
			cancel = getattr(self._window, name, None)
			if callable(cancel):
				try:
					cancel()
				except TypeError:
					cancel(clear_status=False)
		tab = getattr(self._window, "_active_native_tab")()
		if tab is None or tab._disposed or tab.requires_refresh:
			self._dock._selected_capture_refused_v1(
				"Open a ready Ferrum drawing, then choose one molecule on the canvas.",
			)
			return
		self._tab = tab
		self._viewport = tab.view.viewport()
		self._viewport.installEventFilter(self)
		self._viewport.setFocus()
		self._dock._selected_capture_started_v1()

	#============================================
	def cancel(self, message: str | None) -> None:
		"""Retire the event capture without retaining a renderer selection."""
		if self._viewport is not None:
			self._viewport.removeEventFilter(self)
		self._viewport = None
		self._tab = None
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
	def is_armed_v1(self) -> bool:
		"""Report whether the viewport still owns one uncaptured molecule choice."""
		return self._viewport is not None and self._tab is not None

	#============================================
	def consume_selected_query_v1(self, tab: object, per_molecule_limit: int,
			total_limit: int) -> object:
		"""Consume the token synchronously at live-query admission, then forget it."""
		token = self._selected_query_token
		if token is None or self._ready_tab is not tab:
			raise RuntimeError("Ferrum selected molecule query is not ready")
		self.clear_ready_v1()
		return tab._run_live_smarts_selected_query_token_v1(
			token, per_molecule_limit, total_limit,
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
		tab = self._tab
		if tab is None or tab is not getattr(self._window, "_active_native_tab")():
			self.cancel("Molecule choice is no longer current. Choose one molecule again.")
			return
		selection: object | None = None
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = tab.observe_direct_root_interaction()
			scene = tab.view.mapToScene(point)
			selection = tab.select_direct_roots(
				observation, None,
				engine.RenderInteractionQueryV1.point(
					float(scene.x()), float(scene.y()), engine.RenderInteractionModifierV1.replace,
				),
			)
			token = tab._capture_live_smarts_selected_query_v1(selection)
		except Exception:
			self.cancel("Ferrum could not use that choice. Choose exactly one direct molecule and try again.")
			return
		finally:
			# The generic renderer selection never becomes dock state or a query input.
			selection = None
		self.cancel(None)
		self._ready_tab = tab
		self._selected_query_token = token
		self._dock._selected_capture_ready_v1(tab)

	#============================================
	def _cancel_for_interaction_action_handoff_v1(self) -> None:
		"""Retire this pointer mode before another registered tool owns the canvas."""
		if self._viewport is not None:
			self.cancel("Molecule choice cancelled because another tool was selected.")
