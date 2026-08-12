"""Project disposable atom-mark graphics from document-owned facts."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.canvas.items.mark_item
import ferrum_qt.canvas.presentation_projection

def _find_atom_item(scene: PySide6.QtWidgets.QGraphicsScene,
		atom_model: object) -> PySide6.QtWidgets.QGraphicsItem | None:
	"""Find the existing AtomItem by exact AtomModel identity."""
	for item in scene.items():
		if getattr(item, "atom_model", None) is atom_model:
			return item
	return None


#============================================
def _refresh_mark(item: PySide6.QtWidgets.QGraphicsItem, model: object) -> None:
	"""Refresh a projected mark from its persisted CDML geometry."""
	facts = model.rendering_facts
	if facts is None:
		angle, offset, size = _mark_geometry(model)
		draw_circle = _mark_draw_circle(model.attributes)
		line_width = _mark_line_width(model.attributes)
	else:
		angle, offset, size, draw_circle, line_width = facts
	item.angle = angle
	item.offset = offset
	item.size = size
	item.draw_circle = draw_circle
	item.line_width = line_width


#============================================
def _default_mark_size(mark_type: str) -> float:
	"""Return the legacy Ferrum diameter for an omitted CDML mark size."""
	if mark_type in ("plus", "minus", "electronpair", "electron_pair", "lone_pair"):
		return 10.0
	if mark_type == "pz_orbital":
		return 40.0
	return 4.0


#============================================
def _positive_display_number(value: object, default: float) -> float:
	"""Return a finite positive display scalar without changing CDML data."""
	if value is None:
		return default
	try:
		result = float(value)
	except (TypeError, ValueError):
		return default
	return result if math.isfinite(result) and result > 0.0 else default


#============================================
def _finite_display_number(value: object, default: float) -> float:
	"""Return a finite display scalar while preserving model data verbatim."""
	try:
		result = float(value)
	except (TypeError, ValueError):
		return default
	return result if math.isfinite(result) else default


#============================================
def _mark_coordinate(value: str) -> float | None:
	"""Decode one finite CDML coordinate for frontend placement only."""
	try:
		result = ferrum_qt.canvas.presentation_projection._coordinate(value)
	except ValueError:
		return None
	return result if math.isfinite(result) else None


#============================================
def _mark_draw_circle(attributes: dict[str, str]) -> bool:
	"""Return the explicit CDML charge-circle display setting."""
	return attributes.get("draw_circle", "yes") in ("yes", "true", "1", "on")


#============================================
def _mark_line_width(attributes: dict[str, str]) -> float:
	"""Return a finite positive electron-pair line width for Qt rendering."""
	return _positive_display_number(attributes.get("line_width"), 1.0)


#============================================
def _mark_geometry(model: object) -> tuple[float, float, float]:
	"""Return angle, radial offset, and diameter from an atom-mark model.

	CDML ``x``/``y`` are authoritative persisted coordinates. The angle and
	distance are derived together so importing a legacy mark cannot preserve one
	and silently replace the other with a display default. New marks with no
	explicit position retain the historic 12-point radial placement.
	"""
	attributes = model.attributes
	default_angle = _finite_display_number(attributes.get("angle"), 0.0)
	if "x" in attributes and "y" in attributes:
		x = _mark_coordinate(attributes["x"])
		y = _mark_coordinate(attributes["y"])
		if x is not None and y is not None:
			dx = x - model.atom_model.x
			dy = y - model.atom_model.y
			offset = math.hypot(dx, dy)
			angle = math.degrees(math.atan2(dy, dx)) if offset else default_angle
		else:
			angle = default_angle
			offset = 12.0
	else:
		angle = default_angle
		offset = 12.0
	size = _positive_display_number(
		attributes.get("size"), _default_mark_size(model.mark_type),
	)
	return (angle, offset, size)


#============================================
def create_mark_item(
		model: object, atom_item: PySide6.QtWidgets.QGraphicsItem,
		) -> PySide6.QtWidgets.QGraphicsItem | None:
	"""Create one supported atom-mark projection under an AtomItem."""
	if not model.supported:
		return None
	projection_mark_types = {
		"plus": ferrum_qt.canvas.items.mark_item.MARK_PLUS,
		"minus": ferrum_qt.canvas.items.mark_item.MARK_MINUS,
		"radical": ferrum_qt.canvas.items.mark_item.MARK_RADICAL,
		"biradical": ferrum_qt.canvas.items.mark_item.MARK_BIRADICAL,
		"electronpair": ferrum_qt.canvas.items.mark_item.MARK_ELECTRONPAIR,
		"dotted_electronpair": ferrum_qt.canvas.items.mark_item.MARK_DOTTED_ELECTRONPAIR,
		"pz_orbital": ferrum_qt.canvas.items.mark_item.MARK_PZ_ORBITAL,
		# Earlier Qt-only values remain import-compatible with transient callers.
		"electron_pair": ferrum_qt.canvas.items.mark_item.MARK_ELECTRON_PAIR,
		"lone_pair": ferrum_qt.canvas.items.mark_item.MARK_LONE_PAIR,
	}
	if model.mark_type not in projection_mark_types:
		return None
	facts = model.rendering_facts
	if facts is None:
		angle, offset, size = _mark_geometry(model)
		draw_circle = _mark_draw_circle(model.attributes)
		line_width = _mark_line_width(model.attributes)
	else:
		angle, offset, size, draw_circle, line_width = facts
	mark_type = projection_mark_types[model.mark_type]
	item = ferrum_qt.canvas.items.mark_item.MarkItem(
		atom_item, mark_type, angle, offset, size,
		draw_circle=draw_circle, line_width=line_width,
	)
	item.atom_mark_model = model
	item.setFlag(
		PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable,
		True,
	)
	item.setFlag(
		PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable,
		False,
	)
	ferrum_qt.canvas.presentation_projection._attach_binding(model, item, _refresh_mark)
	return item


#============================================
def dispose_detached_items(
		items: list[PySide6.QtWidgets.QGraphicsItem],
		reaper: object | None = None,
		) -> None:
	"""Terminally retire detached projection graphics through the shared reaper.

	Prepared projections create marks below detached atom items.  Disconnecting
	and explicitly deleting those children before their atom wrappers makes
	construction-failure cleanup deterministic without touching a retained live
	scene.  A failed native delete remains owned by the frontend reaper rather
	than by an unreferenced local coordinator.
	"""
	from ferrum_qt.canvas.graphics_retirement import GraphicsRetirementCoordinator
	coordinator = GraphicsRetirementCoordinator()
	coordinator.retire_detached_projection_items(items, reaper=reaper)
	coordinator.raise_if_callback_failed(
		"Detached graphics were released after a disposal failure",
	)


#============================================
def project_marks(document: object,
		scene: PySide6.QtWidgets.QGraphicsScene) -> dict[object, PySide6.QtWidgets.QGraphicsItem]:
	"""Project document atom marks beneath their matching AtomItem parents."""
	projected = {}
	for model in document.marks:
		atom_item = _find_atom_item(scene, model.atom_model)
		if atom_item is None:
			continue
		item = create_mark_item(model, atom_item)
		if item is None:
			continue
		projected[model] = item
	return projected


#============================================
