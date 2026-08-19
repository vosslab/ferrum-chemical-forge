"""Display-only View controls for ordinary Ferrum tabs."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.config.preferences
import ferrum_qt.ferrum.graphics_view
import ferrum_qt.ferrum.statusbar_view_controls


_ZOOM_FACTOR = 1.15


#============================================
class FerrumNativeViewControlsMixin:
	"""Own View actions and one guarded initial paper framing per registered tab."""

	#============================================
	def _initialize_view_controls(self) -> None:
		"""Create window-owned framing state before Ferrum tabs can become current."""
		self._view_controls_closing = False
		self._initial_view_frame_requested: set[object] = set()
		self._initial_view_frame_completed: set[object] = set()
		self._native_hex_grid_visible = True
		self._native_hex_grid_snap_enabled = True

	#============================================
	def _build_view_controls_actions(self) -> None:
		"""Install the compact Ferrum View menu."""
		menu = self.menuBar().addMenu(self.tr("View"))
		self._view_menu = menu
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
		menu.addSeparator()
		self._show_hex_grid_action = PySide6.QtGui.QAction(
			self.tr("Show Hex Grid"), self,
		)
		self._show_hex_grid_action.setCheckable(True)
		self._show_hex_grid_action.setChecked(self._native_hex_grid_visible)
		self._show_hex_grid_action.setToolTip(self.tr(
			"Show a paper-local Rust-generated drawing grid",
		))
		self._show_hex_grid_action.triggered.connect(
			self._on_native_hex_grid_visibility_changed,
		)
		menu.addAction(self._show_hex_grid_action)
		self._snap_hex_grid_action = PySide6.QtGui.QAction(
			self.tr("Snap New and Moved Points to Hex Grid"), self,
		)
		self._snap_hex_grid_action.setCheckable(True)
		self._snap_hex_grid_action.setChecked(self._native_hex_grid_snap_enabled)
		self._snap_hex_grid_action.setShortcut(
			PySide6.QtGui.QKeySequence(self.tr("Ctrl+Shift+G")),
		)
		self._snap_hex_grid_action.setToolTip(self.tr(
			"Place new and moved drawing points on the hex grid",
		))
		self._snap_hex_grid_action.setStatusTip(self.tr(
			"Controls whether new and moved drawing points use the hex grid lattice",
		))
		self._snap_hex_grid_action.setWhatsThis(self.tr(
			"Choose whether new and moved drawing points use the hex grid lattice.",
		))
		self._snap_hex_grid_action.triggered.connect(
			self._on_native_hex_grid_snap_changed,
		)
		menu.addAction(self._snap_hex_grid_action)

	#============================================
	def _install_native_view_status_controls(self) -> None:
		"""Install one permanent action client after the View actions and status bar exist."""
		self._native_view_status_controls = (
			ferrum_qt.ferrum.statusbar_view_controls.
			FerrumNativeStatusBarViewControls(
				self._zoom_out_action, self._zoom_100_action, self._zoom_in_action,
				self._zoom_page_action, self._zoom_content_action, self.statusBar(),
			)
		)
		self._native_view_status_controls.zoom_percent_requested.connect(
			self._set_active_view_zoom_percent,
		)
		self.statusBar().addPermanentWidget(self._native_view_status_controls)

	#============================================
	def _refresh_native_view_status(self) -> None:
		"""Push the active view's observed transform to the permanent action client."""
		self._native_view_status_controls.refresh(self._active_native_view())

	#============================================
	def _active_native_view(
			self,
			) -> ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView | None:
		"""Return only the live graphics view owned by the current registered tab."""
		if self._view_controls_closing:
			return None
		tab = self._active_native_tab()
		if tab is None or tab not in self._native_tabs_by_page or tab._disposed:
			return None
		view = tab.view
		if (
			not isinstance(
				view,
				ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView,
			)
			or view.scene() is None
		):
			return None
		return view

	#============================================
	def _refresh_view_controls_actions(self) -> None:
		"""Keep display controls reachable for every active installed scene."""
		available = self._active_native_view() is not None
		for action in (
			self._zoom_in_action, self._zoom_out_action, self._zoom_100_action,
			self._zoom_page_action, self._zoom_content_action, self._show_hex_grid_action,
			self._snap_hex_grid_action,
		):
			action.setEnabled(available)
		self._show_hex_grid_action.setChecked(self._native_hex_grid_visible)
		self._snap_hex_grid_action.setChecked(self._native_hex_grid_snap_enabled)
		self._refresh_native_view_status()

	#============================================
	def _on_native_hex_grid_visibility_changed(self, visible: bool) -> None:
		"""Apply and persist one application-owned display preference."""
		self._set_native_hex_grid_visible(visible)
		prefs = getattr(self, "_prefs", None)
		if prefs is not None:
			prefs.set_value(
				ferrum_qt.config.preferences.Preferences.KEY_GRID_VISIBLE,
				visible,
			)

	#============================================
	def _set_native_hex_grid_visible(self, visible: bool) -> None:
		"""Project one visibility choice across current and future Ferrum tabs."""
		if type(visible) is not bool:
			raise TypeError("Ferrum hex-grid visibility must be a boolean")
		self._native_hex_grid_visible = visible
		if hasattr(self, "_show_hex_grid_action"):
			self._show_hex_grid_action.setChecked(visible)
		for tab in self._native_tabs_by_page.values():
			tab.view.set_hex_grid_visible(visible)

	#============================================
	def _on_native_hex_grid_snap_changed(self, enabled: bool) -> None:
		"""Apply and persist the authored-point policy with brief status feedback."""
		self._set_native_hex_grid_snap_enabled(enabled)
		prefs = getattr(self, "_prefs", None)
		if prefs is not None:
			prefs.set_value(
				ferrum_qt.config.preferences.Preferences.KEY_GRID_SNAP_ENABLED,
				enabled,
			)
		message = (
			self.tr("New and moved points snap to the hex grid.")
			if enabled else self.tr("New and moved points keep their exact pointer positions.")
		)
		self.statusBar().showMessage(message, 3000)

	#============================================
	def _set_native_hex_grid_snap_enabled(self, enabled: bool) -> None:
		"""Project one authored-point policy across current and future Ferrum tabs."""
		if type(enabled) is not bool:
			raise TypeError("Ferrum hex-grid snapping must be a boolean")
		self._native_hex_grid_snap_enabled = enabled
		if hasattr(self, "_snap_hex_grid_action"):
			self._snap_hex_grid_action.setChecked(enabled)
		for tab in self._native_tabs_by_page.values():
			tab.view.set_hex_grid_snap_enabled(enabled)

	#============================================
	def _install_native_hex_grid_for_tab(self, tab: object) -> None:
		"""Apply current grid visibility and authored-point policy to one tab view."""
		tab.view.set_hex_grid_visible(self._native_hex_grid_visible)
		tab.view.set_hex_grid_snap_enabled(self._native_hex_grid_snap_enabled)

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
		"""Request one bounded center-preserving relative display zoom."""
		view = self._active_native_view()
		if view is not None:
			view.zoom_by_factor(factor)

	#============================================
	@PySide6.QtCore.Slot(int)
	def _set_active_view_zoom_percent(self, percent: int) -> None:
		"""Set one supported active view to an exact bounded uniform percentage."""
		if (
			type(percent) is not int
			or not ferrum_qt.ferrum.graphics_view.
			ZOOM_PERCENT_MINIMUM <= percent <= (
				ferrum_qt.ferrum.graphics_view.
				ZOOM_PERCENT_MAXIMUM
			)
		):
			return
		view = self._active_native_view()
		if view is not None:
			view.set_zoom_percent(percent)

	#============================================
	def _reset_active_view_zoom(self) -> None:
		"""Restore one active view's identity transform without moving its center."""
		view = self._active_native_view()
		if view is None:
			return
		view.reset_zoom()

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
		view.fit_display_bounds(bounds)

	#============================================
	def _fit_view_to_page(
			self,
			view: ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView,
			) -> None:
		"""Frame one live view's renderer-owned page rectangle exactly."""
		scene = view.scene()
		if scene is not None:
			view.fit_display_bounds(scene.sceneRect())

	#============================================
	def _on_native_view_tab_changed(self) -> None:
		"""Request first framing only when a new current tab has never been framed."""
		self._request_current_initial_view_frame()

	#============================================
	def _request_current_initial_view_frame(self) -> None:
		"""Queue one visibility-fenced page fit for the current unframed Ferrum tab."""
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
