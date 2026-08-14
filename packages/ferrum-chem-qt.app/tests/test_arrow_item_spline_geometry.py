"""Focused geometry tests for curved reaction-arrow projections."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_spline_path
import ferrum_qt.canvas.items.arrow_item


#============================================
def _render_arrow(
		item: ferrum_qt.canvas.items.arrow_item.ArrowItem,
		) -> PySide6.QtGui.QImage:
	"""Render one item onto a transparent image in its local coordinates."""
	image = PySide6.QtGui.QImage(
		200, 160, PySide6.QtGui.QImage.Format.Format_ARGB32_Premultiplied,
	)
	image.fill(PySide6.QtCore.Qt.GlobalColor.transparent)
	painter = PySide6.QtGui.QPainter(image)
	painter.setRenderHint(PySide6.QtGui.QPainter.RenderHint.Antialiasing)
	painter.translate(30.0, 30.0)
	item.paint(painter, PySide6.QtWidgets.QStyleOptionGraphicsItem(), None)
	painter.end()
	return image


#============================================
def _cubic_spline_arrow() -> ferrum_qt.canvas.items.arrow_item.ArrowItem:
	"""Return one cubic arrow with a visibly curved center section."""
	item = ferrum_qt.canvas.items.arrow_item.ArrowItem(
		PySide6.QtCore.QPointF(0.0, 0.0),
		PySide6.QtCore.QPointF(100.0, 0.0),
	)
	item.spline = True
	item.control_points = [
		PySide6.QtCore.QPointF(0.0, 100.0),
		PySide6.QtCore.QPointF(100.0, 100.0),
	]
	item.start_head = True
	return item


#============================================
def _quadratic_spline_arrow() -> ferrum_qt.canvas.items.arrow_item.ArrowItem:
	"""Return the legacy one-control-point spline representation."""
	item = ferrum_qt.canvas.items.arrow_item.ArrowItem(
		PySide6.QtCore.QPointF(0.0, 0.0),
		PySide6.QtCore.QPointF(100.0, 0.0),
	)
	item.spline = True
	item.control_points = [PySide6.QtCore.QPointF(50.0, 100.0)]
	return item


#============================================
def _multi_control_spline_arrow() -> ferrum_qt.canvas.items.arrow_item.ArrowItem:
	"""Return one arrow with three authored spline controls."""
	item = ferrum_qt.canvas.items.arrow_item.ArrowItem(
		PySide6.QtCore.QPointF(0.0, 0.0),
		PySide6.QtCore.QPointF(120.0, 0.0),
	)
	item.spline = True
	item.control_points = [
		PySide6.QtCore.QPointF(0.0, 100.0),
		PySide6.QtCore.QPointF(120.0, 100.0),
		PySide6.QtCore.QPointF(120.0, 50.0),
	]
	return item


#============================================
def test_spline_arrow_shape_and_bounds_follow_the_visible_curve(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A cubic arrow is clickable and bounded at its curved midpoint."""
	del qapp
	item = _cubic_spline_arrow()

	curve_midpoint = PySide6.QtCore.QPointF(50.0, 75.0)
	straight_chord = PySide6.QtCore.QPointF(50.0, 0.0)
	assert (
		item.shape().contains(curve_midpoint)
		and item.boundingRect().contains(curve_midpoint)
	)
	assert not item.shape().contains(straight_chord)

#============================================
def test_selected_spline_arrow_highlights_the_curve(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Selection paint adds visible emphasis at the cubic curve."""
	del qapp
	item = _cubic_spline_arrow()
	plain_image = _render_arrow(item)
	item.setSelected(True)
	highlighted_image = _render_arrow(item)

	assert (
		highlighted_image.pixelColor(80, 100).alpha()
		> plain_image.pixelColor(80, 100).alpha()
	)


#============================================
def test_loaded_three_point_spline_uses_its_interior_control_point(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A legacy start-control-end spline projects as a quadratic curve."""
	del qapp
	item = _quadratic_spline_arrow()
	quadratic_midpoint = PySide6.QtCore.QPointF(50.0, 50.0)
	straight_chord = PySide6.QtCore.QPointF(50.0, 0.0)
	assert (
		item.shape().contains(quadratic_midpoint)
		and item.boundingRect().contains(quadratic_midpoint)
	)
	assert not item.shape().contains(straight_chord)

#============================================
def test_loaded_three_point_spline_paints_at_its_quadratic_midpoint(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The legacy control point affects the rendered quadratic curve."""
	del qapp

	assert _render_arrow(
		_quadratic_spline_arrow(),
	).pixelColor(80, 80).alpha() > 0


#============================================
def test_multi_control_spline_uses_one_continuous_paint_and_hit_path(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Three controls remain visible and selectable across each smooth join."""
	del qapp
	item = _multi_control_spline_arrow()
	image = _render_arrow(item)
	first_segment = PySide6.QtCore.QPointF(15.0, 75.0)
	second_segment = PySide6.QtCore.QPointF(105.0, 93.75)
	final_segment = PySide6.QtCore.QPointF(120.0, 43.75)

	assert (
		item.shape().contains(first_segment)
		and item.shape().contains(second_segment)
		and item.shape().contains(final_segment)
	)
	assert (
		image.pixelColor(45, 105).alpha() > 0
		and image.pixelColor(135, 124).alpha() > 0
		and image.pixelColor(150, 74).alpha() > 0
	)


#============================================
def test_shared_spline_helper_handles_empty_and_two_point_inputs(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Empty presentations and two-point splines retain typed Qt path behavior."""
	del qapp
	start = PySide6.QtCore.QPointF(0.0, 0.0)
	end = PySide6.QtCore.QPointF(20.0, 0.0)
	empty = ferrum_qt.canvas.ferrum_spline_path.presentation_path([], True)
	two_point = ferrum_qt.canvas.ferrum_spline_path.presentation_path(
		[start, end], True,
	)

	assert empty.isEmpty()
	assert two_point.currentPosition() == end
