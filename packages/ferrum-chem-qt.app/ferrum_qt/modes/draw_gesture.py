"""Transient, projection-only state for a single Draw pointer gesture."""

# Standard Library
from dataclasses import dataclass


#============================================
@dataclass
class DrawGestureState:
	"""Durable scalar facts captured before a projection can be replaced."""

	kind: str | None = None
	source_molecule_id: str | None = None
	source_atom_id: str | None = None
	source_position: tuple[float, float] | None = None
	default_target_position: tuple[float, float] | None = None
	bond_id: str | None = None
	dragging: bool = False
	press_position: tuple[float, float] | None = None
	preview_line: object | None = None
	preview_scene: object | None = None

	#============================================
	def clear(self) -> tuple[object | None, object | None]:
		"""Release all gesture references and return the preview for retirement."""
		preview = (self.preview_line, self.preview_scene)
		self.kind = None
		self.source_molecule_id = None
		self.source_atom_id = None
		self.source_position = None
		self.default_target_position = None
		self.bond_id = None
		self.dragging = False
		self.press_position = None
		self.preview_line = None
		self.preview_scene = None
		return preview
