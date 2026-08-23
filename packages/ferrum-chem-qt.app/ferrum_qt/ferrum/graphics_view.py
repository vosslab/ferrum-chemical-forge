"""Projection-local graphics view behavior for ordinary Ferrum tabs."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.bridge.display_geometry
import ferrum_qt.config.geometry_units
import ferrum_qt.ferrum.hex_grid
import ferrum_qt.ferrum.keyboard_canvas

ZOOM_PERCENT_MINIMUM = 10
ZOOM_PERCENT_MAXIMUM = 1000
ZOOM_PERCENT_STEP = 5
_ZOOM_FACTOR_PER_NOTCH = 1.15
_WHEEL_UNITS_PER_NOTCH = 120.0


#============================================
def effective_zoom_percent(
		view: PySide6.QtWidgets.QGraphicsView | None,
		) -> float | None:
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
class FerrumNativeGraphicsView(PySide6.QtWidgets.QGraphicsView):
	"""Own disposable native-document display transforms and their notification."""

	display_transform_changed = PySide6.QtCore.Signal()

	#============================================
	def __init__(
			self, parent: PySide6.QtWidgets.QWidget | None = None,
			) -> None:
		"""Initialize one view with no retained direct-zoom sequence anchor."""
		super().__init__(parent)
		self.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		self.viewport().setMouseTracking(True)
		self.setAccessibleName(self.tr("Ferrum drawing canvas"))
		self.setAccessibleDescription(self.tr(
			"Document-space cursor. Arrow keys move by one grid increment; "
			"Shift+Arrow moves by a fine increment.",
		))
		self._direct_zoom_anchor_scene: PySide6.QtCore.QPointF | None = None
		self._direct_zoom_change_in_progress = False
		self._hex_grid_requested_visible = True
		self._hex_grid_snap_enabled = True
		self._hex_grid_item: (
			ferrum_qt.ferrum.hex_grid.FerrumNativeHexGridItem | None
		) = None
		self._keyboard_cursor_scene: PySide6.QtCore.QPointF | None = None
		self._keyboard_cursor_item: PySide6.QtWidgets.QGraphicsPathItem | None = None
		self.horizontalScrollBar().valueChanged.connect(
			self._invalidate_direct_zoom_anchor,
		)
		self.verticalScrollBar().valueChanged.connect(
			self._invalidate_direct_zoom_anchor,
		)

	#============================================
	def setScene(self, scene: PySide6.QtWidgets.QGraphicsScene | None) -> None:
		"""Install a projection scene, its grid decoration, and a fresh zoom anchor."""
		self._invalidate_direct_zoom_anchor()
		self._hex_grid_item = None
		self._keyboard_cursor_scene = None
		self._keyboard_cursor_item = None
		super().setScene(scene)
		self._install_hex_grid_item(scene)

	#============================================
	def show_keyboard_cursor(self) -> PySide6.QtCore.QPointF:
		"""Show and return the focusable author's document-space cursor."""
		if self._keyboard_cursor_scene is None:
			point = self.mapToScene(self.viewport().rect().center())
			self._keyboard_cursor_scene = self.snap_authored_scene_point(point)
		self._ensure_keyboard_cursor_item()
		self._update_keyboard_cursor_accessibility()
		return PySide6.QtCore.QPointF(self._keyboard_cursor_scene)

	#============================================
	def move_keyboard_cursor(self, dx: float, dy: float) -> PySide6.QtCore.QPointF:
		"""Move the visible cursor by one caller-selected document increment."""
		if type(dx) is not float or type(dy) is not float:
			raise TypeError("Ferrum keyboard cursor movement requires float increments")
		current = self.show_keyboard_cursor()
		self._keyboard_cursor_scene = self.snap_authored_scene_point(
			PySide6.QtCore.QPointF(current.x() + dx, current.y() + dy),
		)
		self._ensure_keyboard_cursor_item()
		self._update_keyboard_cursor_accessibility()
		return PySide6.QtCore.QPointF(self._keyboard_cursor_scene)

	#============================================
	def set_keyboard_cursor_scene(
			self, point: PySide6.QtCore.QPointF,
			) -> PySide6.QtCore.QPointF:
		"""Set the disposable cursor to one finite authored scene point."""
		self._keyboard_cursor_scene = self.snap_authored_scene_point(point)
		self._ensure_keyboard_cursor_item()
		self._update_keyboard_cursor_accessibility()
		return PySide6.QtCore.QPointF(self._keyboard_cursor_scene)

	#============================================
	def keyboard_cursor_scene(self) -> PySide6.QtCore.QPointF | None:
		"""Return a copy of the current disposable keyboard cursor position."""
		if self._keyboard_cursor_scene is None:
			return None
		return PySide6.QtCore.QPointF(self._keyboard_cursor_scene)

	#============================================
	def hide_keyboard_cursor(self) -> None:
		"""Retire only the disposable cursor marker, retaining its location."""
		if self._keyboard_cursor_item is not None:
			self._keyboard_cursor_item.setVisible(False)

	#============================================
	def _ensure_keyboard_cursor_item(self) -> None:
		"""Install or reposition a non-interactive high-contrast cursor marker."""
		scene = self.scene()
		point = self._keyboard_cursor_scene
		if scene is None or point is None:
			return
		item = self._keyboard_cursor_item
		if item is None or item.scene() is not scene:
			path = PySide6.QtGui.QPainterPath()
			path.moveTo(-8.0, 0.0)
			path.lineTo(8.0, 0.0)
			path.moveTo(0.0, -8.0)
			path.lineTo(0.0, 8.0)
			item = scene.addPath(path)
			item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
			item.setZValue(1_000_001.0)
			self._keyboard_cursor_item = item
		color = PySide6.QtWidgets.QApplication.palette().color(
			PySide6.QtGui.QPalette.ColorRole.Highlight,
		)
		pen = PySide6.QtGui.QPen(color)
		pen.setWidthF(2.0)
		item.setPen(pen)
		item.setPos(point)
		item.setVisible(True)

	#============================================
	def _update_keyboard_cursor_accessibility(self) -> None:
		"""Expose the visible cursor position to ordinary Qt accessibility clients."""
		point = self._keyboard_cursor_scene
		if point is None:
			return
		self.setAccessibleDescription(self.tr(
			"Document cursor at {0:.1f}, {1:.1f}. Arrow keys move by {2:.0f} "
			"points; Shift+Arrow moves by {3:.0f} points."
		).format(
			point.x(), point.y(),
			ferrum_qt.ferrum.keyboard_canvas.KEYBOARD_CURSOR_GRID_INCREMENT_PT,
			ferrum_qt.ferrum.keyboard_canvas.KEYBOARD_CURSOR_FINE_INCREMENT_PT,
		))

	#============================================
	@property
	def hex_grid_visible(self) -> bool:
		"""Return whether the active projection has a visible grid decoration."""
		return self._hex_grid_item is not None and self._hex_grid_item.isVisible()

	#============================================
	def set_hex_grid_visible(self, visible: bool) -> None:
		"""Apply one application-owned visibility choice to this disposable view."""
		if type(visible) is not bool:
			raise TypeError("Ferrum hex-grid visibility must be a boolean")
		self._hex_grid_requested_visible = visible
		if self._hex_grid_item is not None:
			self._hex_grid_item.setVisible(visible)

	#============================================
	@property
	def hex_grid_snap_enabled(self) -> bool:
		"""Return whether authored points snap to the shared hex-grid lattice."""
		return self._hex_grid_snap_enabled

	#============================================
	def set_hex_grid_snap_enabled(self, enabled: bool) -> None:
		"""Apply one exact application-owned authored-point policy to this view."""
		if type(enabled) is not bool:
			raise TypeError("Ferrum hex-grid snapping must be a boolean")
		self._hex_grid_snap_enabled = enabled

	#============================================
	def snap_authored_scene_point(
			self, raw: PySide6.QtCore.QPointF,
			) -> PySide6.QtCore.QPointF:
		"""Return one finite authored point under this view's shared snap policy."""
		if type(raw) is not PySide6.QtCore.QPointF:
			raise TypeError("Ferrum authored scene point must be an exact QPointF")
		x = raw.x()
		y = raw.y()
		if not math.isfinite(x) or not math.isfinite(y):
			raise ValueError("Ferrum authored scene point must have finite coordinates")
		return self.resolve_authored_scene_point(raw, self._hex_grid_snap_enabled)

	#============================================
	def resolve_authored_scene_point(
			self, raw: PySide6.QtCore.QPointF, snap_enabled: bool,
			) -> PySide6.QtCore.QPointF:
		"""Resolve one finite authored point under an explicit captured snap policy."""
		if type(raw) is not PySide6.QtCore.QPointF:
			raise TypeError("Ferrum authored scene point must be an exact QPointF")
		if type(snap_enabled) is not bool:
			raise TypeError("Ferrum authored snap policy must be a boolean")
		x = raw.x()
		y = raw.y()
		if not math.isfinite(x) or not math.isfinite(y):
			raise ValueError("Ferrum authored scene point must have finite coordinates")
		if not snap_enabled:
			return PySide6.QtCore.QPointF(raw)
		snapped_x, snapped_y = ferrum_qt.bridge.display_geometry.snap_to_hex_grid(
			x, y, ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT,
		)
		if not math.isfinite(snapped_x) or not math.isfinite(snapped_y):
			raise ValueError("Ferrum hex-grid snap returned non-finite coordinates")
		return PySide6.QtCore.QPointF(snapped_x, snapped_y)

	#============================================
	def _install_hex_grid_item(
			self, scene: PySide6.QtWidgets.QGraphicsScene | None) -> None:
		"""Decorate one installed scene without making display failure authoritative."""
		if scene is None:
			return
		try:
			item = (
				ferrum_qt.ferrum.hex_grid.FerrumNativeHexGridItem(
					scene.sceneRect(),
				)
			)
			item.setVisible(self._hex_grid_requested_visible)
			scene.addItem(item)
		except (RuntimeError, TypeError, ValueError):
			return
		self._hex_grid_item = item

	#============================================
	def changeEvent(self, event: PySide6.QtCore.QEvent) -> None:
		"""Refresh grid colors when the application palette family changes."""
		super().changeEvent(event)
		if (
			event.type() == PySide6.QtCore.QEvent.Type.PaletteChange
			and self._hex_grid_item is not None
		):
			self._hex_grid_item.refresh_application_style()

	#============================================
	def resizeEvent(self, event: PySide6.QtGui.QResizeEvent) -> None:
		"""Rebase absolute zoom after the viewport geometry changes."""
		super().resizeEvent(event)
		self._invalidate_direct_zoom_anchor()

	#============================================
	def _viewport_center_scene_precise(self) -> PySide6.QtCore.QPointF:
		"""Return the viewport center in scene coordinates without integer-center loss."""
		return self.mapToScene(self.viewport().rect()).boundingRect().center()

	#============================================
	def _recenter_with_correction(
			self, target: PySide6.QtCore.QPointF,
			) -> None:
		"""Center on one scene point with a residual scrollbar-quantization correction."""
		self.centerOn(target)
		current = self._viewport_center_scene_precise()
		delta = target - current
		if abs(delta.x()) <= 1.0e-9 and abs(delta.y()) <= 1.0e-9:
			return
		self.centerOn(target + delta)

	#============================================
	def _invalidate_direct_zoom_anchor(self) -> None:
		"""Start the next absolute-zoom sequence from the then-current center."""
		if not self._direct_zoom_change_in_progress:
			self._direct_zoom_anchor_scene = None

	#============================================
	def _resolve_direct_zoom_anchor(self) -> PySide6.QtCore.QPointF:
		"""Keep consecutive absolute slider changes on one stable scene center."""
		current = self._viewport_center_scene_precise()
		if self._direct_zoom_anchor_scene is None:
			self._direct_zoom_anchor_scene = PySide6.QtCore.QPointF(current)
		return PySide6.QtCore.QPointF(self._direct_zoom_anchor_scene)

	#============================================
	def zoom_by_factor(self, factor: float) -> bool:
		"""Apply one bounded relative zoom while preserving the viewport center."""
		current = effective_zoom_percent(self)
		if current is None or not math.isfinite(factor) or factor <= 0.0:
			return False
		target = min(
			float(ZOOM_PERCENT_MAXIMUM),
			max(float(ZOOM_PERCENT_MINIMUM), current * factor),
		)
		if abs(target - current) <= 1.0e-12:
			return False
		center = self._viewport_center_scene_precise()
		anchor = self.transformationAnchor()
		self.setTransformationAnchor(
			PySide6.QtWidgets.QGraphicsView.ViewportAnchor.AnchorViewCenter,
		)
		self.scale(target / current, target / current)
		self.setTransformationAnchor(anchor)
		self._recenter_with_correction(center)
		self._invalidate_direct_zoom_anchor()
		self.display_transform_changed.emit()
		return True

	#============================================
	def set_zoom_percent(self, percent: int) -> bool:
		"""Set one exact, bounded integer percentage through an absolute transform."""
		if (
			type(percent) is not int
			or not ZOOM_PERCENT_MINIMUM <= percent <= ZOOM_PERCENT_MAXIMUM
			or effective_zoom_percent(self) is None
		):
			return False
		center = self._resolve_direct_zoom_anchor()
		self._direct_zoom_change_in_progress = True
		try:
			self.resetTransform()
			self.scale(float(percent) / 100.0, float(percent) / 100.0)
			self._recenter_with_correction(center)
		finally:
			self._direct_zoom_change_in_progress = False
		self.display_transform_changed.emit()
		return True

	#============================================
	def reset_zoom(self) -> None:
		"""Restore an upright identity transform without moving the scene center."""
		center = self._viewport_center_scene_precise()
		self.resetTransform()
		self._recenter_with_correction(center)
		self._invalidate_direct_zoom_anchor()
		self.display_transform_changed.emit()

	#============================================
	def fit_display_bounds(self, bounds: PySide6.QtCore.QRectF) -> bool:
		"""Fit one caller-owned semantic rectangle as disposable view state."""
		if not bounds.isValid() or bounds.isEmpty():
			return False
		self.fitInView(bounds, PySide6.QtCore.Qt.AspectRatioMode.KeepAspectRatio)
		self._invalidate_direct_zoom_anchor()
		self.display_transform_changed.emit()
		return True

	#============================================
	def wheelEvent(self, event: PySide6.QtGui.QWheelEvent) -> None:
		"""Zoom about the event position without changing the Rust document."""
		vertical_delta = event.angleDelta().y()
		if vertical_delta == 0:
			event.accept()
			return
		percent = effective_zoom_percent(self)
		if percent is None:
			event.ignore()
			return
		current_scale = percent / 100.0
		if (
			(vertical_delta > 0 and percent >= ZOOM_PERCENT_MAXIMUM)
			or (vertical_delta < 0 and percent <= ZOOM_PERCENT_MINIMUM)
		):
			event.accept()
			return
		notches = vertical_delta / _WHEEL_UNITS_PER_NOTCH
		log_target = math.log(current_scale) + notches * math.log(_ZOOM_FACTOR_PER_NOTCH)
		bounded_log_target = min(
			math.log(ZOOM_PERCENT_MAXIMUM / 100.0),
			max(math.log(ZOOM_PERCENT_MINIMUM / 100.0), log_target),
		)
		target_scale = math.exp(bounded_log_target)
		factor = target_scale / current_scale
		if factor == 1.0:
			event.accept()
			return
		viewport_position = event.position().toPoint()
		original_anchor = self.transformationAnchor()
		self.setTransformationAnchor(
			PySide6.QtWidgets.QGraphicsView.ViewportAnchor.NoAnchor,
		)
		anchored_scene_position = self.mapToScene(viewport_position)
		self.scale(factor, factor)
		shifted_scene_position = self.mapToScene(viewport_position)
		correction = shifted_scene_position - anchored_scene_position
		self.translate(correction.x(), correction.y())
		self.setTransformationAnchor(original_anchor)
		self._invalidate_direct_zoom_anchor()
		self.display_transform_changed.emit()
		event.accept()
