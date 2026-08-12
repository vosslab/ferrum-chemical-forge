"""Project disposable document presentation models into a Qt scene."""

# Standard Library
import warnings

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.items.arrow_item
import ferrum_qt.canvas.items.graphics_item
import ferrum_qt.canvas.items.text_item
import ferrum_qt.canvas.graphics_retirement


#============================================
def _number(value: object, default: float = 0.0) -> float:
	"""Return a CDML numeric value, using the intentional display default."""
	if value is None:
		return default
	result = float(value)
	return result


#============================================
def _coordinate(value: object) -> float:
	"""Return a CDML coordinate as scene-space points."""
	text = str(value)
	if text.endswith("cm"):
		result = float(text[:-2]) * 72.0 / 2.54
		return result
	if text.endswith("px"):
		result = float(text[:-2])
		return result
	result = float(text)
	return result


#============================================
def _color(attributes: dict[str, str]) -> str | None:
	"""Return a legacy CDML line/text color when one is specified."""
	value = attributes.get("line_color")
	if value is None:
		value = attributes.get("color")
	return value


#============================================
def _brush(attributes: dict[str, str]) -> PySide6.QtGui.QBrush:
	"""Build a fill brush from legacy CDML area color attributes."""
	color = attributes.get("area_color")
	if color is None:
		color = attributes.get("background-color")
	if color is None or color == "" or color == "none":
		brush = PySide6.QtGui.QBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
		return brush
	brush = PySide6.QtGui.QBrush(PySide6.QtGui.QColor(color))
	return brush


#============================================
def _pen(attributes: dict[str, str]) -> PySide6.QtGui.QPen:
	"""Build a pen from legacy CDML line attributes."""
	color = _color(attributes)
	qt_color = PySide6.QtGui.QColor(color) if color is not None else PySide6.QtGui.QColor("black")
	width = _number(attributes.get("width"), 1.0)
	pen = PySide6.QtGui.QPen(qt_color, width)
	return pen


#============================================
def _points(model: object) -> list[PySide6.QtCore.QPointF]:
	"""Convert document model coordinates to Qt points."""
	converted = []
	for x, y, _z in model.points:
		converted.append(PySide6.QtCore.QPointF(x, y))
	return converted


#============================================
def _bounds(model: object) -> PySide6.QtCore.QRectF:
	"""Return model bounds or compatible legacy x/y/width/height attributes."""
	bounds = model.bounds
	if bounds is not None:
		x, y, width, height = bounds
	else:
		attributes = model.attributes
		x = _number(attributes.get("x"))
		y = _number(attributes.get("y"))
		width = _number(attributes.get("width"), _number(attributes.get("w")))
		height = _number(attributes.get("height"), _number(attributes.get("h")))
	rect = PySide6.QtCore.QRectF(x, y, width, height).normalized()
	return rect


#============================================
def _presentation_text(model: object) -> str:
	"""Return the display text for a projected text-like presentation."""
	if model.kind == "plus":
		return "+"
	text = model.display_text
	return text


#============================================
def _set_item_interaction(item: PySide6.QtWidgets.QGraphicsItem,
		model: object) -> None:
	"""Attach model identity while keeping loaded artwork non-movable."""
	item.document_object_model = model
	item.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable, True)
	item.setFlag(PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable, False)


#============================================
def _refresh_presentation(item: PySide6.QtWidgets.QGraphicsItem,
		model: object) -> None:
	"""Refresh an already projected graphics item after model mutation."""
	attributes = model.attributes
	if isinstance(item, ferrum_qt.canvas.items.arrow_item.ArrowItem):
		points = _points(model)
		if len(points) >= 2:
			item.start = points[0]
			item.end = points[-1]
			item.control_points = points[1:-1]
			item.spline = attributes.get("spline", "no") in (
				"yes", "true", "1",
			)
		item.start_head = attributes.get("start", "no") in ("yes", "true", "1", "both")
		item.end_head = attributes.get("end", "yes") not in ("no", "false", "0")
		item.line_width = _number(attributes.get("width"), 1.0)
		item.color = _color(attributes)
		return
	if isinstance(item, ferrum_qt.canvas.items.text_item.TextItem):
		points = _points(model)
		position = points[0] if points else _bounds(model).topLeft()
		item.setPos(position)
		font = item.font()
		font_attributes = model.font_attributes
		family = font_attributes.get("family")
		if family is not None:
			font.setFamily(family)
		size = font_attributes.get("size")
		if size is None:
			size = attributes.get("font_size")
		if size is None and model.kind == "plus":
			size = "14"
		if size is not None:
			if model.kind == "text" and (
				type(size) is not str or not size.isdecimal()
			):
				font.setPointSizeF(12.0)
			else:
				font.setPointSizeF(_number(size, 12.0))
		item.setFont(font)
		if model.kind == "plus" and points:
			bounds = item.boundingRect()
			position = PySide6.QtCore.QPointF(
				points[0].x() - bounds.width() / 2.0,
				points[0].y() - bounds.height() / 2.0,
			)
			item.setPos(position)
		color = font_attributes.get("color")
		if color is None:
			color = _color(attributes)
		if color is None and model.kind == "plus":
			color = "#000000"
		if color is not None:
			item.set_color(color)
		if model.kind == "text" and model.formatted_text_runs is not None:
			item.set_formatted_text_runs(model.formatted_text_runs)
		else:
			item.set_text(_presentation_text(model))
		return
	if isinstance(item, (ferrum_qt.canvas.items.graphics_item.RectItem,
			ferrum_qt.canvas.items.graphics_item.OvalItem)):
		item.setRect(_bounds(model))
		item.setPen(_pen(attributes))
		item.setBrush(_brush(attributes))
		return
	if isinstance(item, ferrum_qt.canvas.items.graphics_item.PolygonItem):
		item.setPolygon(PySide6.QtGui.QPolygonF(_points(model)))
		item.setPen(_pen(attributes))
		item.setBrush(_brush(attributes))
		return
	if isinstance(item, PySide6.QtWidgets.QGraphicsPathItem):
		path = PySide6.QtGui.QPainterPath()
		points = _points(model)
		if points:
			path.moveTo(points[0])
			for point in points[1:]:
				path.lineTo(point)
		item.setPath(path)
		item.setPen(_pen(attributes))
		item.setBrush(_brush(attributes))


#============================================
class _ProjectionBinding(PySide6.QtCore.QObject):
	"""Own a model signal connection for one projected graphics item."""

	#============================================
	def __init__(self, model: object, item: PySide6.QtWidgets.QGraphicsItem,
			refresh_callback: object = None) -> None:
		"""Connect a model's changed signal to its graphics refresh callback."""
		super().__init__()
		self._model = model
		self._item = item
		self._refresh_callback = refresh_callback
		self._connected = True
		model.changed.connect(self.refresh)

	#============================================
	def refresh(self) -> None:
		"""Update the item while it remains owned by a live scene."""
		if self._connected:
			if self._refresh_callback is None:
				_refresh_presentation(self._item, self._model)
			else:
				self._refresh_callback(self._item, self._model)

	#============================================
	def dispose(self) -> None:
		"""Release every strong callback edge exactly once before item deletion."""
		try:
			if self._connected:
				with warnings.catch_warnings():
					warnings.simplefilter("ignore", RuntimeWarning)
					self._model.changed.disconnect(self.refresh)
		except (RuntimeError, TypeError):
			# QObject/model teardown may already have severed this connection.
			pass
		finally:
			self._connected = False
			self._model = None
			self._item = None
		self._refresh_callback = None


#============================================
def is_bound_presentation_projection(
		item: PySide6.QtWidgets.QGraphicsItem, model: object,
		) -> bool:
	"""Return whether ``item`` is this projection's live binding for ``model``."""
	binding = getattr(item, "_projection_binding", None)
	return (
		getattr(item, "document_object_model", None) is model
		and isinstance(binding, _ProjectionBinding)
		and binding._model is model
		and binding._item is item
	)


#============================================
def _attach_binding(model: object, item: PySide6.QtWidgets.QGraphicsItem,
		refresh_callback: object = None) -> None:
	"""Associate a projection binding without replacing an item's cleanup hook."""
	item._projection_binding = _ProjectionBinding(model, item, refresh_callback)


#============================================
def dispose_item_callbacks(item: PySide6.QtWidgets.QGraphicsItem) -> None:
	"""Release one item's projection and native callbacks before retirement.

	The item attribute is cleared while its Qt wrapper is still valid.  The
	item-specific ``dispose`` implementation is looked up only for this call, so
	no bound method or closure is retained on the graphics item.
	"""
	first_error = None
	binding = getattr(item, "_projection_binding", None)
	if binding is not None:
		try:
			binding.dispose()
		except Exception as exc:
			first_error = exc
		finally:
			try:
				item._projection_binding = None
			except Exception as exc:
				# An already-retired wrapper cannot accept the attribute update, but
				# its item-specific cleanup must still get a chance to release its
				# own model callbacks before the caller completes retirement.
				if first_error is None:
					first_error = exc
	try:
		dispose = getattr(item, "dispose", None)
		if callable(dispose):
			dispose()
	except Exception as exc:
		if first_error is None:
			first_error = exc
	if first_error is not None:
		raise first_error


#============================================
def create_presentation_item(model: object) -> PySide6.QtWidgets.QGraphicsItem | None:
	"""Create one Qt item for a supported document presentation model."""
	if not model.supported:
		return None
	points = _points(model)
	kind = model.kind
	if kind == "arrow":
		if len(points) < 2:
			return None
		item = ferrum_qt.canvas.items.arrow_item.ArrowItem(points[0], points[-1])
	elif kind in ("text", "plus"):
		text = _presentation_text(model)
		item = ferrum_qt.canvas.items.text_item.TextItem(text)
	elif kind in ("rect", "square"):
		item = ferrum_qt.canvas.items.graphics_item.RectItem(_bounds(model))
	elif kind in ("oval", "circle"):
		item = ferrum_qt.canvas.items.graphics_item.OvalItem(_bounds(model))
	elif kind == "polygon":
		item = ferrum_qt.canvas.items.graphics_item.PolygonItem(points)
	elif kind == "polyline":
		item = PySide6.QtWidgets.QGraphicsPathItem()
	else:
		return None
	_set_item_interaction(item, model)
	_refresh_presentation(item, model)
	_attach_binding(model, item)
	return item


#============================================
def project_presentation_objects(document: object,
		scene: PySide6.QtWidgets.QGraphicsScene) -> dict[object, PySide6.QtWidgets.QGraphicsItem]:
	"""Project all document presentation objects and return their item mapping."""
	projected = {}
	for model in document.presentation_objects:
		item = create_presentation_item(model)
		if item is None:
			continue
		scene.addItem(item)
		projected[model] = item
	return projected


#============================================
def _molecule_for_item(document: object,
		item: PySide6.QtWidgets.QGraphicsItem) -> object | None:
	"""Resolve one molecule graphics item through its model identity."""
	molecule = getattr(item, "molecule_model", None)
	if molecule in document.molecules:
		return molecule
	atom_model = getattr(item, "atom_model", None)
	if atom_model is not None:
		return document._find_molecule_for_atom(atom_model)
	bond_model = getattr(item, "bond_model", None)
	if bond_model is not None:
		return document._find_molecule_for_bond(bond_model)
	parent = ferrum_qt.canvas.graphics_retirement.native_parent_for_item(item)
	if parent is not None:
		return _molecule_for_item(document, parent)
	return None


#============================================
def synchronize_document_stack_z_order(
		document: object, scene: PySide6.QtWidgets.QGraphicsScene,
		) -> None:
	"""Apply ``Document.objects`` order to every projected top-level object.

	Molecules occupy a small z band so bonds remain below their atoms while the
	whole molecule still follows document order relative to presentation
	objects.  Model identities on graphics items, never scene enumeration, are
	the authority for this synchronization.
	"""
	stack_indices = {
		id(object_model): index
		for index, object_model in enumerate(document.objects)
	}
	for item in scene.items():
		object_model = getattr(item, "document_object_model", None)
		if object_model is not None and id(object_model) in stack_indices:
			item.setZValue(float(stack_indices[id(object_model)] * 100 + 20))
			continue
		molecule = _molecule_for_item(document, item)
		if molecule is None:
			continue
		stack_index = stack_indices.get(id(molecule))
		if stack_index is None:
			continue
		if getattr(item, "atom_model", None) is not None:
			layer = 10
		elif getattr(item, "bond_model", None) is not None:
			layer = 5
		else:
			layer = 11
		item.setZValue(float(stack_index * 100 + layer))
