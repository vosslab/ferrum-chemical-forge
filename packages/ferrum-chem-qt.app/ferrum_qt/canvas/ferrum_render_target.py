"""Define the immutable render target retained by every Qt projection item."""

# Standard Library
import dataclasses


#============================================
class RenderTargetKeyError(ValueError):
	"""A caller requested a durable key from a target without one."""


@dataclasses.dataclass(frozen=True, slots=True)
class RenderTargetKey:
	"""Detached visual and durable identities for one current graphics item."""

	kind: str
	render_identifier: str | None
	source_order: int
	durable_object_id: str | None = None
	durable_molecule_object_id: str | None = None

	#============================================
	@property
	def is_durable(self) -> bool:
		"""Return whether this target can be submitted to a Rust operation."""
		return self.durable_object_id is not None

	#============================================
	def durable_selection_key(self) -> tuple[str, str]:
		"""Return this target's exact Rust-issued selection identity."""
		if self.durable_object_id is None:
			raise RenderTargetKeyError("render target is not durable")
		return self.kind, self.durable_object_id
