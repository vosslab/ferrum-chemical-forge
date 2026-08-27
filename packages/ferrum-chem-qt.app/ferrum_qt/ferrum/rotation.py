"""Projection-only preview and Rust commit for selected-atom rotation."""

# Standard Library
import dataclasses
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_display_refresh
import ferrum_qt.themes.document_display_palette


_MARKER_RADIUS = 3.0


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeRotationSelection:
	"""Immutable projection facts captured for one Ferrum rotation gesture."""

	addresses: tuple[tuple[str, str], ...]
	durable_selection: tuple[str, ...]
	positions: tuple[tuple[str, float, float], ...]
	affected_bonds: tuple[tuple[str, str], ...]
	center: PySide6.QtCore.QPointF


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeRotationPreview:
	"""One scene-owned, non-authoritative rotation skeleton."""

	selection: FerrumNativeRotationSelection
	root: PySide6.QtWidgets.QGraphicsItemGroup
	markers: tuple[tuple[str, PySide6.QtWidgets.QGraphicsEllipseItem], ...]
	bonds: tuple[tuple[str, str, PySide6.QtWidgets.QGraphicsLineItem], ...]


#============================================
class FerrumNativeRotationTabMixin:
	"""Resolve selected Rust facts and submit one atomic rotation operation."""

	#============================================
	def has_rotatable_atom_selection(self) -> bool:
		"""Return whether Rust resolves one molecule-owned atom selection."""
		if self.requires_refresh:
			return False
		try:
			selected = self.selected_structure_targets()
		except (RuntimeError, TypeError, ValueError):
			return False
		import ferrum_qt.ferrum.engine as engine
		return (
			bool(selected)
			and all(target.kind is engine.StructureTargetKindV1.atom for target in selected)
			and len({target.molecule_object_id for target in selected}) == 1
		)

	#============================================
	def selected_atom_rotation(self) -> FerrumNativeRotationSelection:
		"""Copy one exact Rust-resolved atom selection from the current projection."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		selected_targets = self.selected_structure_targets()
		if (
				not selected_targets
				or any(target.kind is not engine.StructureTargetKindV1.atom
						for target in selected_targets)
			):
			raise ValueError("select one or more current atoms before rotating")
		addresses = tuple(
			(target.molecule_object_id, target.object_id)
			for target in selected_targets
		)
		if any(
				type(molecule_id) is not str or not molecule_id
				or type(atom_id) is not str or not atom_id
				for molecule_id, atom_id in addresses
			):
			raise ValueError("selected rotation atom lacks durable identity")
		molecule_ids = frozenset(molecule_id for molecule_id, _atom_id in addresses)
		if len(molecule_ids) != 1:
			raise ValueError("select atoms from one molecule before rotating")
		selected = frozenset(atom_id for _molecule_id, atom_id in addresses)
		observation = self.current_document_observation()
		molecule_id = next(iter(molecule_ids))
		molecule_matches = tuple(
			molecule for molecule in observation.projection.molecules
			if molecule.document_object_id == molecule_id
		)
		if len(molecule_matches) != 1:
			raise ValueError("selected rotation molecule is absent from the Rust projection")
		molecule = molecule_matches[0]
		positions = []
		affected_bonds = []
		selected_positions = []
		for atom in molecule.atoms:
			atom_id = atom.document_object_id
			position = (atom_id, float(atom.position.x), float(atom.position.y))
			positions.append(position)
			if atom_id in selected:
				selected_positions.append(position)
		for bond in molecule.bonds:
			start = bond.start.document_object_id
			end = bond.end.document_object_id
			if start is not None and end is not None and (
					start in selected or end in selected
					):
				affected_bonds.append((start, end))
		resolved = frozenset(position[0] for position in selected_positions)
		if resolved != selected:
			raise ValueError("selected rotation atom is absent from the Rust projection")
		center_x = sum(position[1] for position in selected_positions) / len(selected_positions)
		center_y = sum(position[2] for position in selected_positions) / len(selected_positions)
		return FerrumNativeRotationSelection(
			addresses,
			tuple(target.object_id for target in selected_targets),
			tuple(positions),
			tuple(affected_bonds),
			PySide6.QtCore.QPointF(center_x, center_y),
		)

	#============================================
	def apply_selected_atom_rotation(self, selection: FerrumNativeRotationSelection,
			center: tuple[float, float], angle_radians: float) -> object:
		"""Submit one still-current projection selection through the Rust session."""
		self._require_mutable()
		if type(selection) is not FerrumNativeRotationSelection:
			raise TypeError("Ferrum rotation requires exact captured selection facts")
		if (
				type(center) is not tuple
				or len(center) != 2
				or any(type(value) is not float or not math.isfinite(value) for value in center)
				or type(angle_radians) is not float
				or not math.isfinite(angle_radians)
			):
			raise TypeError("Ferrum rotation requires finite float center and angle values")
		current = self.selected_atom_rotation()
		if current.addresses != selection.addresses:
			raise ValueError("Ferrum rotation selection changed during the gesture")
		snapshot = self.current_snapshot
		result = self._live_document_session_v1.rotate_live_document_atoms_v1(
			snapshot.revision, snapshot.digest, selection.addresses,
			center[0], center[1], angle_radians,
		)
		self._install_mutation_result(result, selection.durable_selection)
		return result


#============================================
def create_rotation_preview(tab: object,
		selection: FerrumNativeRotationSelection) -> FerrumNativeRotationPreview:
	"""Create one dashed scene skeleton without changing authoritative plan items."""
	scene = tab.view.scene()
	if scene is None:
		raise RuntimeError("Ferrum document has no current scene")
	positions = _position_map(selection.positions)
	root = PySide6.QtWidgets.QGraphicsItemGroup()
	root.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	root.setZValue(1_000_000.0)
	markers = []
	selected_ids = frozenset(atom_id for _, atom_id in selection.addresses)
	for atom_id in selected_ids:
		position = positions[atom_id]
		marker = PySide6.QtWidgets.QGraphicsEllipseItem(root)
		_set_marker_position(marker, position)
		markers.append((atom_id, marker))
	bonds = []
	for start_id, end_id in selection.affected_bonds:
		if start_id not in positions or end_id not in positions:
			continue
		line = PySide6.QtWidgets.QGraphicsLineItem(root)
		line.setLine(PySide6.QtCore.QLineF(positions[start_id], positions[end_id]))
		bonds.append((start_id, end_id, line))
	scene.addItem(root)
	preview = FerrumNativeRotationPreview(selection, root, tuple(markers), tuple(bonds))
	refreshable = _RotationDisplayRefreshable(preview)
	refreshable.refresh_document_display_palette(tab.document_display_palette)
	ferrum_qt.ferrum.document_display_refresh.register_attached_document_display_refreshable(
		tab, root, refreshable,
	)
	return preview


#============================================
class _RotationDisplayRefreshable(
		ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRefreshableV1):
	"""Replace only the material of one retained rotation skeleton."""

	def __init__(self, preview: FerrumNativeRotationPreview) -> None:
		"""Retain the preview items without copying selection or geometry facts."""
		self._preview = preview

	#============================================
	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Refresh marker and skeleton material while preserving current rotation."""
		outline = palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PREVIEW_OUTLINE,
		)
		pen = PySide6.QtGui.QPen(outline)
		pen.setWidthF(1.5)
		pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
		pen.setCosmetic(False)
		brush = PySide6.QtGui.QBrush(palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PREVIEW_FILL,
		))
		for _atom_id, marker in self._preview.markers:
			marker.setPen(pen)
			marker.setBrush(brush)
		for _start_id, _end_id, line in self._preview.bonds:
			line.setPen(pen)


#============================================
def update_rotation_preview(preview: FerrumNativeRotationPreview,
		angle_radians: float) -> None:
	"""Update only the disposable skeleton for one finite accumulated angle."""
	if type(preview) is not FerrumNativeRotationPreview:
		raise TypeError("rotation preview requires exact local preview state")
	if type(angle_radians) is not float or not math.isfinite(angle_radians):
		raise TypeError("rotation preview angle must be a finite float")
	positions = _position_map(preview.selection.positions)
	center = preview.selection.center
	for atom_id, marker in preview.markers:
		positions[atom_id] = _rotated_point(positions[atom_id], center, angle_radians)
		_set_marker_position(marker, positions[atom_id])
	for start_id, end_id, line in preview.bonds:
		line.setLine(PySide6.QtCore.QLineF(positions[start_id], positions[end_id]))


#============================================
def _position_map(positions: tuple[tuple[str, float, float], ...]) -> dict[str, PySide6.QtCore.QPointF]:
	"""Copy immutable projected positions into one preview-local lookup."""
	return {
		atom_id: PySide6.QtCore.QPointF(x, y)
		for atom_id, x, y in positions
	}


#============================================
def _rotated_point(point: PySide6.QtCore.QPointF, center: PySide6.QtCore.QPointF,
		angle_radians: float) -> PySide6.QtCore.QPointF:
	"""Return one scene point rotated around the captured selection center."""
	cosine = math.cos(angle_radians)
	sine = math.sin(angle_radians)
	dx = point.x() - center.x()
	dy = point.y() - center.y()
	return PySide6.QtCore.QPointF(
		center.x() + dx * cosine - dy * sine,
		center.y() + dx * sine + dy * cosine,
	)


#============================================
def _set_marker_position(marker: PySide6.QtWidgets.QGraphicsEllipseItem,
		position: PySide6.QtCore.QPointF) -> None:
	"""Center one local marker on a projected or previewed atom point."""
	marker.setRect(
		position.x() - _MARKER_RADIUS,
		position.y() - _MARKER_RADIUS,
		_MARKER_RADIUS * 2.0,
		_MARKER_RADIUS * 2.0,
	)
