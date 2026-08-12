"""Shared terminal-lifecycle values for the MainWindow facade."""

# Standard Library
import dataclasses
import enum


#============================================
@dataclasses.dataclass
class _PendingSessionDeletion:
	"""Long-lived Qt roots and detached graphics retained during terminal close."""

	wrappers: list[object]
	retained_graphics_records: object = None
	session_destroyed: bool = False

	#============================================
	@property
	def retained_detached_graphics(self) -> object:
		"""Expose detached roots for existing focused lifecycle assertions."""
		if self.retained_graphics_records is None:
			return None
		return self.retained_graphics_records.detached


class ShutdownState(enum.StrEnum):
	"""Public MainWindow shutdown lifecycle for Qt behavior tests and hosts."""

	LIVE = "live"
	DRAINING = "draining"
	READY = "ready"
