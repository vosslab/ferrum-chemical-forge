"""Display-only View controls for ordinary Rust-native Ferrum tabs."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_statusbar_view_controls


_ZOOM_FACTOR = 1.15


#============================================
class FerrumNativeViewControlsMixin:
	"""Own View actions and one guarded initial paper framing per registered tab."""

	#============================================
	def _initialize_view_controls(self) -> None:
		"""Create window-owned framing state before native tabs can become current."""
		self._view_controls_closing = False
		self._initial_view_frame_requested: set[object] = set()
		self._initial_view_frame_completed: set[object] = set()

	#============================================
	def _build_view_controls_actions(self) -> None:
		"""Install the compact native View menu without legacy view ownership."""
		menu = self.menuBar().addMenu(self.tr("View"))
		self._zoom_in_action = PySide6.QtGui.QAction(self.tr("Zoom In"), self)
		self._zoom_in_action.setShortcut(PySide6.QtGui.QKeySequence(self.tr("Ctrl++")))
		self._zoom_in_action.triggered.connect(self._zoom_in_active_view)
		menu.addAction(self._zoom_in_action)
		self._zoom_out_action = PySide6.QtGui.QAction(self.tr("Zoom Out"), self)
		self._zoom_out_action.setShortcut(PySide6.QtGui.QKeySequence(self.tr("Ctrl+-")))
		self._zoom_out_action.triggered.connect(self._zoom_out_active_view)
		menu.addAction(self._zoom_out_action)
		self._zoom_100_action = PySide6.QtGui.QAction(self.tr("Zoom to 100%"), self)
		self._zoom_100_action.setShortcut(PySide6.QtGui.QKeySequence(self.tr("Ctrl+0")))
		self._zoom_100_action.triggered.connect(self._reset_active_view_zoom)
		menu.addAction(self._zoom_100_action)
		menu.addSeparator()
		self._zoom_page_action = PySide6.QtGui.QAction(self.tr("Zoom to Page"), self)
		self._zoom_page_action.triggered.connect(self._fit_active_view_to_page)
		menu.addAction(self._zoom_page_action)
		self._zoom_content_action = PySide6.QtGui.QAction(self.tr("Zoom to Content"), self)
		self._zoom_content_action.triggered.connect(self._fit_active_view_to_content)
		menu.addAction(self._zoom_content_action)

	#============================================
	def _install_native_view_status_controls(self) -> None:
		"""Install one permanent action client after the View actions and status bar exist."""
		self._native_view_status_controls = (
			ferrum_qt.native.ferrum_native_statusbar_view_controls.
			FerrumNativeStatusBarViewControls(
				self._zoom_out_action, self._zoom_100_action, self._zoom_in_action,
				self._zoom_page_action, self._zoom_content_action, self.statusBar(),
			)
		)
		self.statusBar().addPermanentWidget(self._native_view_status_controls)

	#============================================
	def _refresh_native_view_status(self) -> None:
		"""Push the active view's observed transform to the permanent action client."""
		self._native_view_status_controls.refresh(self._active_native_view())

	#============================================
	def _active_native_view(self) -> PySide6.QtWidgets.QGraphicsView | None:
		"""Return only the live graphics view owned by the current registered tab."""
		if self._view_controls_closing:
			return None
		tab = self._active_native_tab()
		if tab is None or tab not in self._native_tabs_by_page or tab._disposed:
			return None
		view = tab.view
		return view if view.scene() is not None else None

	#============================================
	def _refresh_view_controls_actions(self) -> None:
		"""Keep display controls reachable for every active installed scene."""
		available = self._active_native_view() is not None
		for action in (
				self._zoom_in_action, self._zoom_out_action, self._zoom_100_action,
				self._zoom_page_action, self._zoom_content_action,
			):
			action.setEnabled(available)
		self._refresh_native_view_status()

	#============================================
	def _zoom_in_active_view(self) -> None:
		"""Increase the active view scale while preserving its scene center."""
		self._scale_active_view(_ZOOM_FACTOR)

	#============================================
	def _zoom_out_active_view(self) -> None:
		"""Decrease the active view scale while preserving its scene center."""
		self._scale_active_view(1.0 / _ZOOM_FACTOR)

	#============================================
	def _scale_active_view(self, factor: float) -> None:
		"""Apply one uniform display scale about the viewport-center scene point."""
		view = self._active_native_view()
		if view is None:
			return
		center = view.mapToScene(view.viewport().rect().center())
		anchor = view.transformationAnchor()
		view.setTransformationAnchor(
			PySide6.QtWidgets.QGraphicsView.ViewportAnchor.AnchorViewCenter,
		)
		view.scale(factor, factor)
		view.setTransformationAnchor(anchor)
		view.centerOn(center)
		self._refresh_native_view_status()

	#============================================
	def _reset_active_view_zoom(self) -> None:
		"""Restore one active view's identity transform without moving its center."""
		view = self._active_native_view()
		if view is None:
			return
		center = view.mapToScene(view.viewport().rect().center())
		view.resetTransform()
		view.centerOn(center)
		self._refresh_native_view_status()

	#============================================
	def _fit_active_view_to_page(self) -> None:
		"""Frame the active renderer-owned paper rectangle."""
		view = self._active_native_view()
		if view is not None:
			self._fit_view_to_page(view)
			self._refresh_native_view_status()

	#============================================
	def _fit_active_view_to_content(self) -> None:
		"""Frame active document roots, or exactly the page when none are drawable."""
		view = self._active_native_view()
		tab = self._active_native_tab()
		if view is None or tab is None:
			return
		bounds = tab.document_content_bounds()
		if bounds is None:
			self._fit_view_to_page(view)
			self._refresh_native_view_status()
			return
		view.fitInView(bounds, PySide6.QtCore.Qt.AspectRatioMode.KeepAspectRatio)
		self._refresh_native_view_status()

	#============================================
	def _fit_view_to_page(self, view: PySide6.QtWidgets.QGraphicsView) -> None:
		"""Frame one live view's renderer-owned page rectangle exactly."""
		scene = view.scene()
		if scene is not None:
			view.fitInView(
				scene.sceneRect(), PySide6.QtCore.Qt.AspectRatioMode.KeepAspectRatio,
			)

	#============================================
	def _on_native_view_tab_changed(self) -> None:
		"""Request first framing only when a new current tab has never been framed."""
		self._request_current_initial_view_frame()

	#============================================
	def _request_current_initial_view_frame(self) -> None:
		"""Queue one visibility-fenced page fit for the current unframed native tab."""
		if self._view_controls_closing:
			return
		tab = self._active_native_tab()
		if (
			tab is None
			or tab not in self._native_tabs_by_page
			or tab in self._initial_view_frame_requested
			or tab in self._initial_view_frame_completed
		):
			return
		view = tab.view
		if view.scene() is None:
			return
		self._initial_view_frame_requested.add(tab)
		PySide6.QtCore.QTimer.singleShot(
			0, lambda captured_tab=tab, captured_view=view:
			self._deliver_initial_view_frame(captured_tab, captured_view),
		)

	#============================================
	def _deliver_initial_view_frame(
			self, tab: object, view: PySide6.QtWidgets.QGraphicsView) -> None:
		"""Fit exactly one still-current visible tab, otherwise permit a later retry."""
		self._initial_view_frame_requested.discard(tab)
		if (
			self._view_controls_closing
			or not self.isVisible()
			or tab not in self._native_tabs_by_page
			or self._active_native_tab() is not tab
			or not tab.isVisible()
			or not view.isVisible()
			or tab.view is not view
			or view.scene() is None
		):
			return
		self._fit_view_to_page(view)
		self._initial_view_frame_completed.add(tab)
		self._refresh_native_view_status()

	#============================================
	def _cancel_native_view_controls_for_tab(self, tab: object) -> None:
		"""Forget queued and completed framing ownership before a tab is retired."""
		self._initial_view_frame_requested.discard(tab)
		self._initial_view_frame_completed.discard(tab)

	#============================================
	def _prepare_native_view_controls_shutdown(self) -> None:
		"""Invalidate every queued display callback before accepted window teardown."""
		self._view_controls_closing = True
		self._initial_view_frame_requested.clear()
		self._initial_view_frame_completed.clear()

	#============================================
	def showEvent(self, event: PySide6.QtGui.QShowEvent) -> None:
		"""Request the current unframed tab on every visible window transition."""
		super().showEvent(event)
		if hasattr(self, "_view_controls_closing"):
			self._request_current_initial_view_frame()
