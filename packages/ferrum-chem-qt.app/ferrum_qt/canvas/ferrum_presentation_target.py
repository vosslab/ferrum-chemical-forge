"""Validate presentation DTOs into the shared immutable Qt render target."""

# local repo modules
import ferrum_qt.canvas.ferrum_render_target


_DOCUMENT_OBJECT_KIND = "document_object"


#============================================
class PresentationTargetError(ValueError):
	"""A renderer-issued presentation target violates the shared Qt contract."""


#============================================
def presentation_target_from_dto(value: object, extension: object,
		) -> ferrum_qt.canvas.ferrum_render_target.RenderTargetKey:
	"""Copy one authenticated opaque document-object target into scene-local state."""
	if type(value) is not extension.RenderTargetV1:
		raise PresentationTargetError("presentation target has the wrong DTO type")
	if value.kind != _DOCUMENT_OBJECT_KIND:
		raise PresentationTargetError("presentation target kind is invalid")
	if type(value.document_object_id) is not str or not value.document_object_id:
		raise PresentationTargetError("presentation document-object identity is invalid")
	return ferrum_qt.canvas.ferrum_render_target.RenderTargetKey(
		value.kind, value.document_object_id,
	)
