"""Projection-local Draw helpers retained for legacy editing callers.

New Draw gestures submit immutable structural requests.  These helpers support
older in-process construction and undo paths without becoming persistent state.
"""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore

# local repo modules
import ferrum_qt.canvas.items.atom_item
import ferrum_qt.canvas.items.bond_item
import ferrum_qt.undo.commands
from ferrum_qt.models.molecule_model import MoleculeModel


SNAP_RADIUS = 15.0
OVERLAP_THRESHOLD = 4.0


#============================================
class DrawLegacyEditingMixin:
	"""Keep retired local construction APIs separate from structural gestures."""

	#============================================
	def _create_atom_at(self, x: float, y: float, symbol: str | None = None) -> ferrum_qt.canvas.items.atom_item.AtomItem | None:
		"""Create a projection atom through the existing undo contract."""
		scene = self._env.scene
		if scene is None:
			return None
		element = symbol or self._current_element
		molecule = self._get_active_molecule()
		if molecule is None:
			return None
		atom_model = molecule.create_atom(symbol=element)
		atom_model.x = x
		atom_model.y = y
		atom_item = ferrum_qt.canvas.items.atom_item.AtomItem(atom_model)
		undo_stack = self._env.undo_stack
		if undo_stack is None:
			molecule.add_atom(atom_model)
			scene.addItem(atom_item)
		else:
			command = ferrum_qt.undo.commands.AddAtomCommand(
				scene, molecule, atom_model, atom_item,
			)
			undo_stack.push(command)
		self.status_message.emit(f"Added {element} atom")
		return atom_item

	#============================================
	def _create_bond_between(self, atom1_item: ferrum_qt.canvas.items.atom_item.AtomItem,
			atom2_item: ferrum_qt.canvas.items.atom_item.AtomItem,
			) -> ferrum_qt.canvas.items.bond_item.BondItem | None:
		"""Create a projection bond through the existing undo contract."""
		scene = self._env.scene
		molecule = self._get_active_molecule()
		if scene is None or molecule is None:
			return None
		bond_model = molecule.create_bond(
			order=self._current_bond_order, bond_type=self._current_bond_type,
		)
		bond_model.simple_double = self._simple_double
		molecule.add_bond(atom1_item.atom_model, atom2_item.atom_model, bond_model)
		bond_item = ferrum_qt.canvas.items.bond_item.BondItem(bond_model)
		scene.addItem(bond_item)
		undo_stack = self._env.undo_stack
		if undo_stack is not None:
			command = ferrum_qt.undo.commands.AddBondCommand(
				scene, molecule, bond_model, bond_item,
			)
			command._first_redo = True
			undo_stack.push(command)
		order_name = {1: "single", 2: "double", 3: "triple"}.get(
			self._current_bond_order, str(self._current_bond_order),
		)
		self.status_message.emit(f"Added {order_name} bond")
		return bond_item

	#============================================
	def _find_atom_at(self, scene_pos: PySide6.QtCore.QPointF) -> ferrum_qt.canvas.items.atom_item.AtomItem | None:
		"""Return the nearest atom item within the draw snap radius."""
		scene = self._env.scene
		if scene is None:
			return None
		snap_rect = PySide6.QtCore.QRectF(
			scene_pos.x() - SNAP_RADIUS, scene_pos.y() - SNAP_RADIUS,
			SNAP_RADIUS * 2, SNAP_RADIUS * 2,
		)
		best_item = None
		best_distance = SNAP_RADIUS
		for item in scene.items(snap_rect):
			if not isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				continue
			model = item.atom_model
			distance = math.hypot(model.x - scene_pos.x(), model.y - scene_pos.y())
			if distance < best_distance:
				best_item = item
				best_distance = distance
		return best_item

	#============================================
	def _find_bond_at(self, scene_pos: PySide6.QtCore.QPointF) -> ferrum_qt.canvas.items.bond_item.BondItem | None:
		"""Return the first bond projection at a scene point."""
		scene = self._env.scene
		if scene is None:
			return None
		for item in scene.items(scene_pos):
			if isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
				return item
		return None

	#============================================
	def _toggle_bond_type(self, bond_item: ferrum_qt.canvas.items.bond_item.BondItem) -> None:
		"""Apply the established type/order/display variant cycle with undo."""
		model = bond_item.bond_model
		undo_stack = self._env.undo_stack
		if undo_stack is None:
			return
		old_values = {
			"order": model.order, "type": model.type, "center": model.center,
			"bond_width": model.bond_width, "auto_bond_sign": model.auto_bond_sign,
			"simple_double": model.simple_double, "atom1": model.atom1, "atom2": model.atom2,
		}
		if self._current_bond_type != model.type:
			model.type = self._current_bond_type
			model.order = self._current_bond_order
		elif self._current_bond_order == 1 and self._current_bond_type in ("n", "d"):
			model.order = (model.order % 3) + 1
		elif self._current_bond_order != model.order:
			model.order = self._current_bond_order
		elif self._current_bond_type in ("h", "a"):
			model.atom1, model.atom2 = model.atom2, model.atom1
		elif self._current_bond_type == "w":
			model.atom1, model.atom2 = model.atom2, model.atom1
			if not model.center:
				model.bond_width = -model.bond_width
		elif model.order == 2:
			if model.center:
				model.bond_width = -model.bond_width
				model.auto_bond_sign = -model.auto_bond_sign
				model.center = False
			elif model.bond_width > 0:
				model.bond_width = -model.bond_width
				model.auto_bond_sign = -model.auto_bond_sign
			else:
				model.center = True
		model.simple_double = self._simple_double
		undo_stack.beginMacro("Toggle Bond")
		for property_name, old_value in old_values.items():
			new_value = getattr(model, property_name)
			if new_value == old_value:
				continue
			setattr(model, property_name, old_value)
			command = ferrum_qt.undo.commands.ChangePropertyCommand(
				model, property_name, old_value, new_value,
				text=f"Toggle {property_name}",
			)
			undo_stack.push(command)
		undo_stack.endMacro()
		self.status_message.emit(f"Toggled bond: order={model.order} type={model.type}")

	#============================================
	def _handle_overlap(self) -> None:
		"""Merge same-scene atom projections that are visually coincident."""
		scene = self._env.scene
		if scene is None:
			return
		atom_items = [item for item in scene.items()
			if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem)]
		removed_items = set()
		for first_index, first_item in enumerate(atom_items):
			if id(first_item) in removed_items:
				continue
			for second_item in atom_items[first_index + 1:]:
				if id(second_item) in removed_items:
					continue
				first = first_item.atom_model
				second = second_item.atom_model
				if (abs(first.x - second.x) < OVERLAP_THRESHOLD
						and abs(first.y - second.y) < OVERLAP_THRESHOLD):
					self._merge_atoms(first_item, second_item)
					removed_items.add(id(second_item))

	#============================================
	def _merge_atoms(self, survivor_item: ferrum_qt.canvas.items.atom_item.AtomItem,
			duplicate_item: ferrum_qt.canvas.items.atom_item.AtomItem) -> None:
		"""Redirect duplicate-bond endpoints, then remove the duplicate atom."""
		scene = self._env.scene
		undo_stack = self._env.undo_stack
		if scene is None or undo_stack is None:
			return
		survivor = survivor_item.atom_model
		duplicate = duplicate_item.atom_model
		undo_stack.beginMacro("Merge Overlapping Atoms")
		for item in scene.items():
			if not isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
				continue
			bond = item.bond_model
			if bond.atom1 is duplicate and bond.atom2 is not survivor:
				property_name = "atom1"
			elif bond.atom2 is duplicate and bond.atom1 is not survivor:
				property_name = "atom2"
			else:
				continue
			command = ferrum_qt.undo.commands.ChangePropertyCommand(
				bond, property_name, duplicate, survivor, text="Merge atom endpoint",
			)
			undo_stack.push(command)
		document = self._env.document
		if document is not None:
			for molecule in document.molecules:
				if duplicate in molecule.atoms:
					command = ferrum_qt.undo.commands.RemoveAtomCommand(
						scene, molecule, duplicate, duplicate_item, [],
					)
					undo_stack.push(command)
					break
		undo_stack.endMacro()

	#============================================
	def _get_active_molecule(self) -> MoleculeModel | None:
		"""Return the active local molecule, creating a disposable one if needed."""
		document = self._env.document
		if document is None:
			return None
		if document.molecules:
			return document.molecules[0]
		molecule = MoleculeModel()
		document.add_molecule(molecule, mark_dirty=False)
		return molecule
