"""Projection-only preview and Rust commit for selected-atom rotation."""

# Standard Library
import dataclasses
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


_MARKER_RADIUS = 3.0


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeRotationSelection:
	"""Immutable projection facts captured for one Ferrum rotation gesture."""

	addresses: tuple[tuple[str, str], ...]
	durable_selection: tuple[tuple[str, str], ...]
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
		"""Return whether every selected target is one durable rendered atom."""
		if self.requires_refresh:
			return False
		selected = self._require_projection().selected_targets()
		return bool(selected) and all(
			target.is_durable and target.kind == "atom" for target in selected
		)

	#============================================
	def selected_atom_rotation(self) -> FerrumNativeRotationSelection:
		"""Copy one exact durable atom selection from the installed projection."""
		self._require_mutable()
		selected_targets = self._require_projection().selected_targets()
		if (
				not selected_targets
				or any(not target.is_durable or target.kind != "atom" for target in selected_targets)
			):
			raise ValueError("select one or more durable atoms before rotating")
		selected_ids = tuple(target.identifier for target in selected_targets)
		if any(identifier is None for identifier in selected_ids):
			raise ValueError("selected rotation atom lacks durable identity")
		selected = frozenset(selected_ids)
		observation = self.current_document_observation()
		addresses = []
		positions = []
		affected_bonds = []
		selected_positions = []
		for molecule in observation.projection.molecules:
			molecule_selected = tuple(
				atom for atom in molecule.atoms if atom.source_id in selected
			)
			if molecule_selected and molecule.source_id is None:
				raise ValueError("selected rotation atom lacks a durable molecule")
			for atom in molecule.atoms:
				if atom.source_id is None:
					continue
				position = (atom.source_id, float(atom.position.x), float(atom.position.y))
				positions.append(position)
				if atom.source_id in selected:
					addresses.append((molecule.source_id, atom.source_id))
					selected_positions.append(position)
			for bond in molecule.bonds:
				start = bond.start.source_id
				end = bond.end.source_id
				if start is not None and end is not None and (
						start in selected or end in selected
						):
					affected_bonds.append((start, end))
		resolved = frozenset(atom_id for _, atom_id in addresses)
		if resolved != selected:
			raise ValueError("selected rotation atom is absent from the Rust projection")
		center_x = sum(position[1] for position in selected_positions) / len(selected_positions)
		center_y = sum(position[2] for position in selected_positions) / len(selected_positions)
		return FerrumNativeRotationSelection(
			tuple(addresses),
			tuple(("atom", atom_id) for _, atom_id in addresses),
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
		import ferrum_qt.ferrum.engine as engine
		targets = tuple(
			engine.DocumentAtomRotationTargetV1.create(molecule_id, atom_id)
			for molecule_id, atom_id in selection.addresses
		)
		operation = engine.DocumentOperationV1.rotate_atoms(
			targets, center[0], center[1], angle_radians,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
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
	color = PySide6.QtWidgets.QApplication.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Highlight,
	)
	pen = PySide6.QtGui.QPen(color)
	pen.setWidthF(1.5)
	pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
	pen.setCosmetic(False)
	fill = PySide6.QtGui.QColor(color)
	fill.setAlpha(96)
	root = PySide6.QtWidgets.QGraphicsItemGroup()
	root.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	root.setZValue(1_000_000.0)
	markers = []
	selected_ids = frozenset(atom_id for _, atom_id in selection.addresses)
	for atom_id in selected_ids:
		position = positions[atom_id]
		marker = PySide6.QtWidgets.QGraphicsEllipseItem(root)
		marker.setPen(pen)
		marker.setBrush(PySide6.QtGui.QBrush(fill))
		_set_marker_position(marker, position)
		markers.append((atom_id, marker))
	bonds = []
	for start_id, end_id in selection.affected_bonds:
		if start_id not in positions or end_id not in positions:
			continue
		line = PySide6.QtWidgets.QGraphicsLineItem(root)
		line.setPen(pen)
		line.setLine(PySide6.QtCore.QLineF(positions[start_id], positions[end_id]))
		bonds.append((start_id, end_id, line))
	scene.addItem(root)
	return FerrumNativeRotationPreview(selection, root, tuple(markers), tuple(bonds))


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
