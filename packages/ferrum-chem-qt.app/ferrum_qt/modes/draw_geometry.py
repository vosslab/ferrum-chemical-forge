"""Pure placement and snapping calculations for Draw mode.

This module deliberately knows only about model coordinates.  Qt scene work and
document mutations stay in the Draw-mode coordinator.
"""

# Standard Library
import math

# local repo modules
from ferrum_qt.models.atom_model import AtomModel
from ferrum_qt.models.molecule_model import MoleculeModel


ANGLE_RESOLUTION = 30


#============================================
def get_angle(a1_model: AtomModel, a2_model: AtomModel) -> float:
	"""Return the angle from ``a1_model`` to ``a2_model`` in radians."""
	dx = a2_model.x - a1_model.x
	dy = a2_model.y - a1_model.y
	return math.atan2(dy, dx)


#============================================
def point_on_circle(cx: float, cy: float, radius: float, dx: float, dy: float,
		resolution: int = ANGLE_RESOLUTION) -> tuple[float, float]:
	"""Return a circle point in a direction, optionally snapped by angle."""
	if resolution:
		resolution_radians = math.pi * resolution / 180.0
		angle = round(math.atan2(dy, dx) / resolution_radians) * resolution_radians
	else:
		angle = math.atan2(dy, dx)
	x = cx + round(math.cos(angle) * radius, 2)
	y = cy + round(math.sin(angle) * radius, 2)
	return (x, y)


#============================================
def on_which_side(a_model: AtomModel, b_model: AtomModel, px: float, py: float) -> int:
	"""Return the side of directed line ``a_model`` to ``b_model`` for a point."""
	cross = ((b_model.x - a_model.x) * (py - a_model.y)
			- (b_model.y - a_model.y) * (px - a_model.x))
	if abs(cross) < 1e-6:
		return 0
	return 1 if cross > 0 else -1


#============================================
def find_least_crowded_place(atom_model: AtomModel, mol_model: MoleculeModel,
		distance: float) -> tuple[float, float]:
	"""Return the midpoint of the largest neighbor-angle gap."""
	connections = mol_model.connected_display_atoms(atom_model)
	if not connections:
		return (atom_model.x + distance, atom_model.y)
	angles = []
	for neighbor_model, _bond_order in connections:
		angle = math.atan2(neighbor_model.y - atom_model.y,
			neighbor_model.x - atom_model.x)
		if angle < 0:
			angle += 2 * math.pi
		angles.append(angle)
	angles.sort()
	angles.append(angles[0] + 2 * math.pi)
	max_difference = 0.0
	max_index = 0
	for index in range(len(angles) - 1):
		difference = angles[index + 1] - angles[index]
		if difference > max_difference:
			max_difference = difference
			max_index = index
	best_angle = (angles[max_index] + angles[max_index + 1]) / 2.0
	x = atom_model.x + distance * math.cos(best_angle)
	y = atom_model.y + distance * math.sin(best_angle)
	return (x, y)


#============================================
def find_place(atom_model: AtomModel, mol_model: MoleculeModel, bond_length: float,
		placement_sign: int, last_used_atom_id: str | None,
		added_order: int = 1) -> tuple[tuple[float, float], int, str | None]:
	"""Return a smart bond endpoint and the next transoid placement state.

	The returned state makes transoid alternation explicit: DrawMode owns it, while
	this calculation remains independent of Qt items and persistent document state.
	"""
	connections = mol_model.connected_display_atoms(atom_model)
	if len(connections) == 0:
		x = atom_model.x + math.cos(math.pi / 6) * bond_length
		y = atom_model.y - math.sin(math.pi / 6) * bond_length
		return (x, y), placement_sign, last_used_atom_id
	if len(connections) >= 2:
		point = find_least_crowded_place(atom_model, mol_model, bond_length)
		return point, placement_sign, last_used_atom_id
	neighbor_model, existing_order = connections[0]
	if existing_order == 3 or added_order == 3:
		angle = get_angle(atom_model, neighbor_model) + math.pi
		point = (atom_model.x + math.cos(angle) * bond_length,
			atom_model.y + math.sin(angle) * bond_length)
		return point, placement_sign, last_used_atom_id
	neighbor_connections = mol_model.connected_display_atoms(neighbor_model)
	if atom_model.atom_id is not None and atom_model.atom_id == last_used_atom_id:
		placement_sign = -placement_sign
	elif len(neighbor_connections) != 2:
		placement_sign = -placement_sign
	angle = get_angle(atom_model, neighbor_model) + placement_sign * 2 * math.pi / 3
	x = atom_model.x + math.cos(angle) * bond_length
	y = atom_model.y + math.sin(angle) * bond_length
	if len(neighbor_connections) == 2 and atom_model.atom_id != last_used_atom_id:
		other_neighbor = None
		for candidate_model, _bond_order in neighbor_connections:
			if candidate_model is not atom_model:
				other_neighbor = candidate_model
				break
		if other_neighbor is not None:
			new_side = on_which_side(neighbor_model, atom_model, x, y)
			other_side = on_which_side(neighbor_model, atom_model,
				other_neighbor.x, other_neighbor.y)
			if new_side == other_side and new_side != 0:
				placement_sign = -placement_sign
				angle = get_angle(atom_model, neighbor_model)
				angle += placement_sign * 2 * math.pi / 3
				x = atom_model.x + math.cos(angle) * bond_length
				y = atom_model.y + math.sin(angle) * bond_length
	if atom_model.atom_id is not None:
		last_used_atom_id = atom_model.atom_id
	return (x, y), placement_sign, last_used_atom_id
