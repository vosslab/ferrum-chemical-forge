"""Share immutable presentation target validation across detached Qt renderers."""

# Standard Library
import dataclasses


_RECORD_KINDS = frozenset((
	"arrow", "plus", "text", "polyline", "rectangle", "square", "oval", "circle",
	"polygon",
))
_U32_RANGE = range(2**32)


#============================================
class PresentationTargetError(ValueError):
	"""A renderer-issued presentation target violates the shared Qt contract."""


@dataclasses.dataclass(frozen=True, slots=True)
class PresentationTarget:
	"""One immutable backend target retained by a disposable Qt item."""

	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int
	record_kind: str


#============================================
def presentation_target_from_dto(value: object, extension: object,
		expected_kind: str | None = None) -> PresentationTarget:
	"""Copy one authenticated durable-or-local target into scene-local state."""
	if type(value) is not extension.PresentationTargetV1:
		raise PresentationTargetError("presentation target has the wrong DTO type")
	if value.id is not None and (type(value.id) is not str or not value.id):
		raise PresentationTargetError("presentation ID is invalid")
	if type(value.projection_key) is not str or not value.projection_key:
		raise PresentationTargetError("presentation projection key is invalid")
	if value.source_id is not None and (type(value.source_id) is not str or not value.source_id):
		raise PresentationTargetError("presentation source ID is invalid")
	if (value.id is None) != (value.source_id is None):
		raise PresentationTargetError("presentation durable target provenance is invalid")
	if type(value.source_order) is not int or value.source_order not in _U32_RANGE:
		raise PresentationTargetError("presentation source order is invalid")
	if type(value.record_kind) is not str or value.record_kind not in _RECORD_KINDS:
		raise PresentationTargetError("presentation record kind is invalid")
	if expected_kind is not None and value.record_kind != expected_kind:
		raise PresentationTargetError("presentation root and target kinds differ")
	return PresentationTarget(
		value.id, value.projection_key, value.source_id, value.source_order, value.record_kind,
	)
