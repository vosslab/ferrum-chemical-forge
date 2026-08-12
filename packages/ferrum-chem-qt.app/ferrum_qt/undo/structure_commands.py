"""Undo commands for atom, bond, and atom-geometry structure edits."""

import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.canvas.items.atom_item
import ferrum_qt.canvas.items.bond_item
import ferrum_qt.models.atom_model
import ferrum_qt.models.bond_model
import ferrum_qt.models.fragment_model
import ferrum_qt.models.molecule_model


ConnectedBond = tuple[
	ferrum_qt.models.bond_model.BondModel,
	ferrum_qt.canvas.items.bond_item.BondItem,
]
FragmentSnapshot = tuple[ferrum_qt.models.fragment_model.FragmentModel, ...]
FragmentChange = tuple[
	ferrum_qt.models.molecule_model.MoleculeModel,
	FragmentSnapshot,
	FragmentSnapshot,
]
AtomItemOffset = tuple[ferrum_qt.canvas.items.atom_item.AtomItem, float, float]


#============================================
class AddAtomCommand(PySide6.QtGui.QUndoCommand):
	"""Add or remove one atom model with its Qt projection."""
	def __init__(self, scene: PySide6.QtWidgets.QGraphicsScene,
			molecule_model: ferrum_qt.models.molecule_model.MoleculeModel,
			atom_model: ferrum_qt.models.atom_model.AtomModel,
			atom_item: ferrum_qt.canvas.items.atom_item.AtomItem,
			text: str = "Add Atom") -> None:
		super().__init__(text); self._scene = scene; self._molecule_model = molecule_model
		self._atom_model = atom_model; self._atom_item = atom_item
		self._fragments_before = molecule_model.fragment_snapshot()
	def redo(self) -> None:
		if not ferrum_qt.canvas.graphics_retirement.can_add_item_to_captured_scene(self._scene, self._atom_item): return
		self._molecule_model.add_atom(self._atom_model); self._molecule_model.restore_fragment_snapshot(self._fragments_before)
		ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, self._atom_item)
	def undo(self) -> None:
		if ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(self._scene, self._atom_item): self._molecule_model.remove_atom(self._atom_model)
	def graphics_items(self) -> list[PySide6.QtWidgets.QGraphicsItem]: return [self._atom_item]


#============================================
class RemoveAtomCommand(PySide6.QtGui.QUndoCommand):
	"""Remove an atom and connected bonds, retaining their exact restoration data."""
	def __init__(self, scene: PySide6.QtWidgets.QGraphicsScene,
			molecule_model: ferrum_qt.models.molecule_model.MoleculeModel,
			atom_model: ferrum_qt.models.atom_model.AtomModel,
			atom_item: ferrum_qt.canvas.items.atom_item.AtomItem,
			connected_bonds: list[ConnectedBond], text: str = "Remove Atom") -> None:
		super().__init__(text); self._scene = scene; self._molecule_model = molecule_model
		self._atom_model = atom_model; self._atom_item = atom_item; self._connected_bonds = list(connected_bonds)
		self._bond_endpoints = [(model.atom1, model.atom2) for model, _item in self._connected_bonds]
		self._fragments_before = molecule_model.fragment_snapshot()
	def redo(self) -> None:
		items = [self._atom_item, *(item for _model, item in self._connected_bonds)]
		if not all(ferrum_qt.canvas.graphics_retirement.item_belongs_to_scene(self._scene, item) for item in items): return
		for bond_model, bond_item in self._connected_bonds:
			ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(self._scene, bond_item); self._molecule_model.remove_bond(bond_model)
		ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(self._scene, self._atom_item); self._molecule_model.remove_atom(self._atom_model)
	def undo(self) -> None:
		items = [self._atom_item, *(item for _model, item in self._connected_bonds)]
		if not all(ferrum_qt.canvas.graphics_retirement.can_add_item_to_captured_scene(self._scene, item) for item in items): return
		self._molecule_model.add_atom(self._atom_model); ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, self._atom_item)
		for (bond_model, bond_item), (atom1, atom2) in zip(self._connected_bonds, self._bond_endpoints, strict=True):
			if atom1 is not None and atom2 is not None: self._molecule_model.add_bond(atom1, atom2, bond_model)
			ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, bond_item)
		self._molecule_model.restore_fragment_snapshot(self._fragments_before)
	def graphics_items(self) -> list[PySide6.QtWidgets.QGraphicsItem]: return [self._atom_item, *(item for _model, item in self._connected_bonds)]


#============================================
class AddBondCommand(PySide6.QtGui.QUndoCommand):
	"""Add or remove one bond model with its Qt projection."""
	def __init__(self, scene: PySide6.QtWidgets.QGraphicsScene,
			molecule_model: ferrum_qt.models.molecule_model.MoleculeModel,
			bond_model: ferrum_qt.models.bond_model.BondModel,
			bond_item: ferrum_qt.canvas.items.bond_item.BondItem,
			text: str = "Add Bond") -> None:
		super().__init__(text); self._scene = scene; self._molecule_model = molecule_model
		self._bond_model = bond_model; self._bond_item = bond_item; self._atom1 = bond_model.atom1; self._atom2 = bond_model.atom2
		self._fragments_before = molecule_model.fragment_snapshot(); self._first_redo = False
	def redo(self) -> None:
		if self._first_redo: self._first_redo = False; return
		if not ferrum_qt.canvas.graphics_retirement.can_add_item_to_captured_scene(self._scene, self._bond_item): return
		self._molecule_model.add_bond(self._atom1, self._atom2, self._bond_model); self._molecule_model.restore_fragment_snapshot(self._fragments_before)
		ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, self._bond_item)
	def undo(self) -> None:
		if ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(self._scene, self._bond_item): self._molecule_model.remove_bond(self._bond_model)
	def graphics_items(self) -> list[PySide6.QtWidgets.QGraphicsItem]: return [self._bond_item]


#============================================
class RemoveBondCommand(PySide6.QtGui.QUndoCommand):
	"""Remove or restore one bond model with its Qt projection."""
	def __init__(self, scene: PySide6.QtWidgets.QGraphicsScene,
			molecule_model: ferrum_qt.models.molecule_model.MoleculeModel,
			bond_model: ferrum_qt.models.bond_model.BondModel,
			bond_item: ferrum_qt.canvas.items.bond_item.BondItem,
			text: str = "Remove Bond") -> None:
		super().__init__(text); self._scene = scene; self._molecule_model = molecule_model
		self._bond_model = bond_model; self._bond_item = bond_item; self._atom1 = bond_model.atom1; self._atom2 = bond_model.atom2
		self._fragments_before = molecule_model.fragment_snapshot()
	def redo(self) -> None:
		if ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(self._scene, self._bond_item): self._molecule_model.remove_bond(self._bond_model)
	def undo(self) -> None:
		if not ferrum_qt.canvas.graphics_retirement.can_add_item_to_captured_scene(self._scene, self._bond_item): return
		if self._atom1 is not None and self._atom2 is not None: self._molecule_model.add_bond(self._atom1, self._atom2, self._bond_model)
		self._molecule_model.restore_fragment_snapshot(self._fragments_before); ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, self._bond_item)
	def graphics_items(self) -> list[PySide6.QtWidgets.QGraphicsItem]: return [self._bond_item]


#============================================
class MoveAtomsCommand(PySide6.QtGui.QUndoCommand):
	"""Merge continuous atom drags and retain linear-fragment lifecycle snapshots."""
	_MERGE_ID = 1001
	def __init__(self, items_and_offsets: list[AtomItemOffset], text: str = "Move Atoms") -> None:
		super().__init__(text); self._items_and_offsets = list(items_and_offsets); self._first_redo = True
		self._fragment_changes = self._linear_fragment_changes()
	def id(self) -> int: return self._MERGE_ID
	def mergeWith(self, other: PySide6.QtGui.QUndoCommand) -> bool:
		if not isinstance(other, MoveAtomsCommand): return False
		item_index = {id(item): index for index, (item, _dx, _dy) in enumerate(self._items_and_offsets)}
		for item, dx, dy in other._items_and_offsets:
			if id(item) in item_index:
				index = item_index[id(item)]; old_item, old_dx, old_dy = self._items_and_offsets[index]; self._items_and_offsets[index] = (old_item, old_dx + dx, old_dy + dy)
			else: self._items_and_offsets.append((item, dx, dy))
		return True
	def redo(self) -> None:
		if self._first_redo: self._first_redo = False; self._apply_fragment_snapshots(True); return
		for item, dx, dy in self._items_and_offsets: item.atom_model.x += dx; item.atom_model.y += dy
		self._apply_fragment_snapshots(True)
	def undo(self) -> None:
		for item, dx, dy in self._items_and_offsets: item.atom_model.x -= dx; item.atom_model.y -= dy
		self._apply_fragment_snapshots(False)
	def _linear_fragment_changes(self) -> list[FragmentChange]:
		molecules = []
		for item, _dx, _dy in self._items_and_offsets:
			molecule = getattr(item.atom_model, "_molecule_model", None)
			if molecule is not None and molecule not in molecules: molecules.append(molecule)
		return [(molecule, before, after) for molecule in molecules for before, after in [(molecule.fragment_snapshot(), molecule.linear_fragment_snapshot_after_geometry({}))] if after != before]
	def _apply_fragment_snapshots(self, after: bool) -> None:
		for molecule, before, after_fragments in self._fragment_changes: molecule.restore_fragment_snapshot(after_fragments if after else before)
	def graphics_items(self) -> list[PySide6.QtWidgets.QGraphicsItem]: return [item for item, _dx, _dy in self._items_and_offsets]
