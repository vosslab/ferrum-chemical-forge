"""Define the immutable render target retained by every Qt projection item."""

# Standard Library
import dataclasses


#============================================
class RenderTargetKeyError(ValueError):
	"""A caller requested a durable key from a target without one."""


@dataclasses.dataclass(frozen=True, slots=True)
class RenderTargetKey:
	"""Opaque durable document-object identity for one current graphics item."""

	kind: str
	document_object_id: str

	#============================================
	@property
	def is_durable(self) -> bool:
		"""Return whether this target can be submitted to a Rust operation."""
		return True

	#============================================
	def durable_selection_key(self) -> tuple[str, str]:
		"""Return this target's exact Rust-issued selection identity."""
		return self.kind, self.document_object_id
