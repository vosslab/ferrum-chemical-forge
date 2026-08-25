"""Validate presentation DTOs into the shared immutable Qt render target."""

# local repo modules
import ferrum_qt.canvas.ferrum_render_target


_RECORD_KINDS = frozenset((
	"arrow", "plus", "text", "polyline", "rectangle", "square", "oval", "circle",
	"polygon",
))
_U32_RANGE = range(2**32)


#============================================
class PresentationTargetError(ValueError):
	"""A renderer-issued presentation target violates the shared Qt contract."""


#============================================
def presentation_target_from_dto(value: object, extension: object,
		expected_kind: str | None = None,
		) -> ferrum_qt.canvas.ferrum_render_target.RenderTargetKey:
	"""Copy one authenticated dual-identity presentation target into scene-local state."""
	if type(value) is not extension.RenderTargetV1:
		raise PresentationTargetError("presentation target has the wrong DTO type")
	if type(value.kind) is not str or value.kind not in _RECORD_KINDS:
		raise PresentationTargetError("presentation record kind is invalid")
	if value.render_identifier is not None and (
			type(value.render_identifier) is not str or not value.render_identifier
		):
		raise PresentationTargetError("presentation render identifier is invalid")
	if value.durable_object_id is not None and (
			type(value.durable_object_id) is not str or not value.durable_object_id
		):
		raise PresentationTargetError("presentation durable object identity is invalid")
	if value.durable_molecule_object_id is not None:
		raise PresentationTargetError("presentation target unexpectedly has a molecule owner")
	if type(value.source_order) is not int or value.source_order not in _U32_RANGE:
		raise PresentationTargetError("presentation source order is invalid")
	if expected_kind is not None and value.kind != expected_kind:
		raise PresentationTargetError("presentation root and target kinds differ")
	return ferrum_qt.canvas.ferrum_render_target.RenderTargetKey(
		value.kind,
		value.render_identifier,
		value.source_order,
		value.durable_object_id,
		value.durable_molecule_object_id,
	)
