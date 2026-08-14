"""Projection-local graphics view behavior for ordinary Rust-native tabs."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_statusbar_view_controls


_MINIMUM_SCALE = 0.10
_MAXIMUM_SCALE = 10.0
_ZOOM_FACTOR_PER_NOTCH = 1.15
_WHEEL_UNITS_PER_NOTCH = 120.0


#============================================
class FerrumNativeGraphicsView(PySide6.QtWidgets.QGraphicsView):
	"""Keep native document wheel zoom local to disposable view state."""

	display_transform_changed = PySide6.QtCore.Signal()

	#============================================
	def wheelEvent(self, event: PySide6.QtGui.QWheelEvent) -> None:
		"""Zoom about the event position without changing the Rust document."""
		vertical_delta = event.angleDelta().y()
		if vertical_delta == 0:
			event.accept()
			return
		percent = (
			ferrum_qt.native.ferrum_native_statusbar_view_controls.
			effective_percent(self)
		)
		if percent is None:
			event.ignore()
			return
		current_scale = percent / 100.0
		if (
			(vertical_delta > 0 and current_scale >= _MAXIMUM_SCALE)
			or (vertical_delta < 0 and current_scale <= _MINIMUM_SCALE)
		):
			event.accept()
			return
		notches = vertical_delta / _WHEEL_UNITS_PER_NOTCH
		log_target = math.log(current_scale) + notches * math.log(_ZOOM_FACTOR_PER_NOTCH)
		bounded_log_target = min(
			math.log(_MAXIMUM_SCALE), max(math.log(_MINIMUM_SCALE), log_target),
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
		self.display_transform_changed.emit()
		event.accept()
