"""Undo commands for document-owned presentation models and projections."""

import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.models.atom_model
import ferrum_qt.models.document
import ferrum_qt.models.document_object
import ferrum_qt.models.fragment_model
import ferrum_qt.models.molecule_model


# Presentation geometry is a frontend-only projection snapshot. Persistent
# ownership stays with the backend document session.
GeometryPoint = tuple[float, float, float | None]
Bounds = tuple[float, float, float, float]
PresentationGeometry = tuple[list[GeometryPoint], Bounds | None]
PresentationChange = tuple[
	ferrum_qt.models.document_object.PresentationObject,
	PresentationGeometry,
	PresentationGeometry,
]
AtomGeometryChange = tuple[
	ferrum_qt.models.atom_model.AtomModel,
	tuple[float, float],
	tuple[float, float],
]
FragmentSnapshot = tuple[ferrum_qt.models.fragment_model.FragmentModel, ...]
FragmentChange = tuple[
	ferrum_qt.models.molecule_model.MoleculeModel,
	FragmentSnapshot,
	FragmentSnapshot,
]


#============================================
class AddPresentationObjectCommand(PySide6.QtGui.QUndoCommand):
	"""Add one presentation model and its scene projection."""
	def __init__(self, document: ferrum_qt.models.document.Document,
			scene: PySide6.QtWidgets.QGraphicsScene,
			object_model: ferrum_qt.models.document_object.PresentationObject,
			graphics_item: PySide6.QtWidgets.QGraphicsItem,
			text: str = "Add Drawing Object") -> None:
		super().__init__(text); self._document = document; self._scene = scene; self._object_model = object_model; self._graphics_item = graphics_item
	def redo(self) -> None:
		if not self._document.is_current_projection_scene(self._scene): return
		if not ferrum_qt.canvas.graphics_retirement.can_add_item_to_captured_scene(self._scene, self._graphics_item): return
		self._document.add_presentation_object(self._object_model, mark_dirty=False); ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, self._graphics_item)
	def undo(self) -> None:
		if not self._document.is_current_projection_scene(self._scene): return
		if not ferrum_qt.canvas.graphics_retirement.item_belongs_to_scene(self._scene, self._graphics_item): return
		ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(self._scene, self._graphics_item); self._document.remove_presentation_object(self._object_model, mark_dirty=False)
	def graphics_items(self) -> list[PySide6.QtWidgets.QGraphicsItem]: return [self._graphics_item]


#============================================
class RemovePresentationObjectCommand(PySide6.QtGui.QUndoCommand):
	"""Remove one presentation model while retaining its projection for undo."""
	def __init__(self, document: ferrum_qt.models.document.Document,
			scene: PySide6.QtWidgets.QGraphicsScene,
			object_model: ferrum_qt.models.document_object.PresentationObject,
			graphics_item: PySide6.QtWidgets.QGraphicsItem,
			text: str = "Remove Drawing Object") -> None:
		super().__init__(text); self._document = document; self._scene = scene; self._object_model = object_model; self._graphics_item = graphics_item; self._stack_index = document.object_index(object_model)
	def redo(self) -> None:
		if not self._document.is_current_projection_scene(self._scene): return
		if not ferrum_qt.canvas.graphics_retirement.item_belongs_to_scene(self._scene, self._graphics_item): return
		ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(self._scene, self._graphics_item); self._document.remove_presentation_object(self._object_model, mark_dirty=False); self._document._synchronize_scene_object_stack()
	def undo(self) -> None:
		if not self._document.is_current_projection_scene(self._scene): return
		if not ferrum_qt.canvas.graphics_retirement.can_add_item_to_captured_scene(self._scene, self._graphics_item): return
		self._document.insert_presentation_object(self._object_model, index=self._stack_index, mark_dirty=False); ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, self._graphics_item); self._document._synchronize_scene_object_stack()
	def graphics_items(self) -> list[PySide6.QtWidgets.QGraphicsItem]: return [self._graphics_item]


#============================================
class MovePresentationObjectsCommand(PySide6.QtGui.QUndoCommand):
	"""Apply captured presentation geometry through models, not scene mutation."""
	def __init__(self, changes: list[PresentationChange], text: str = "Move Drawing Objects") -> None:
		super().__init__(text)
		self._changes = [(model, (list(before_points), tuple(before_bounds) if before_bounds is not None else None), (list(after_points), tuple(after_bounds) if after_bounds is not None else None)) for model, (before_points, before_bounds), (after_points, after_bounds) in changes]
	def redo(self) -> None: self._apply_geometry(True)
	def undo(self) -> None: self._apply_geometry(False)
	def _apply_geometry(self, after: bool) -> None:
		for model, before_geometry, after_geometry in self._changes:
			points, bounds = after_geometry if after else before_geometry; model.set_points(points); model.set_bounds(bounds)


#============================================
class TransformGeometryCommand(PySide6.QtGui.QUndoCommand):
	"""Apply atom and presentation geometry snapshots as one discrete operation."""
	def __init__(self, atom_changes: list[AtomGeometryChange],
			presentation_changes: list[PresentationChange],
			text: str = "Transform Objects") -> None:
		super().__init__(text); self._atom_changes = [(model, tuple(before), tuple(after)) for model, before, after in atom_changes]
		self._presentation_changes = [(model, (list(before_points), tuple(before_bounds) if before_bounds is not None else None), (list(after_points), tuple(after_bounds) if after_bounds is not None else None)) for model, (before_points, before_bounds), (after_points, after_bounds) in presentation_changes]
		self._fragment_changes = self._linear_fragment_changes()
	def redo(self) -> None: self._apply(True)
	def undo(self) -> None: self._apply(False)
	def _apply(self, after: bool) -> None:
		for model, before, after_state in self._atom_changes: model.x, model.y = after_state if after else before
		for model, before, after_state in self._presentation_changes:
			points, bounds = after_state if after else before; model.set_points(points); model.set_bounds(bounds)
		for molecule, before, after_fragments in self._fragment_changes: molecule.restore_fragment_snapshot(after_fragments if after else before)
	def _linear_fragment_changes(self) -> list[FragmentChange]:
		coordinates_by_molecule = {}
		for model, _before, after in self._atom_changes:
			molecule = getattr(model, "_molecule_model", None)
			if molecule is not None: coordinates_by_molecule.setdefault(molecule, {})[model] = after
		return [(molecule, before, after) for molecule, coordinates in coordinates_by_molecule.items() for before, after in [(molecule.fragment_snapshot(), molecule.linear_fragment_snapshot_after_geometry(coordinates))] if after != before]


#============================================
class ReorderDocumentObjectsCommand(PySide6.QtGui.QUndoCommand):
	"""Replace document stack order through the model identity validator."""
	def __init__(self, document: ferrum_qt.models.document.Document,
			ordered_objects: list[object], text: str = "Reorder Objects") -> None:
		super().__init__(text); self._document = document; self._before_order = document.objects; self._after_order = list(ordered_objects)
		document.replace_object_order(self._after_order, mark_dirty=False); document.replace_object_order(self._before_order, mark_dirty=False)
	def redo(self) -> None: self._document.replace_object_order(self._after_order, mark_dirty=False)
	def undo(self) -> None: self._document.replace_object_order(self._before_order, mark_dirty=False)


#============================================
class AddAtomMarkCommand(PySide6.QtGui.QUndoCommand):
	"""Add an atom-attached mark model and child graphics item."""
	def __init__(self, document: ferrum_qt.models.document.Document,
			mark_model: ferrum_qt.models.document_object.AtomMarkModel,
			mark_item: PySide6.QtWidgets.QGraphicsItem,
			parent_atom_item: PySide6.QtWidgets.QGraphicsItem,
			text: str = "Add Atom Mark") -> None:
		super().__init__(text); self._document = document; self._mark_model = mark_model; self._mark_item = mark_item; self._parent_atom_item = parent_atom_item
	def redo(self) -> None:
		scene = ferrum_qt.canvas.graphics_retirement.native_scene_for_item(self._parent_atom_item)
		if ferrum_qt.canvas.graphics_retirement.set_item_parent_in_captured_scene(self._mark_item, self._parent_atom_item, scene): self._document.add_mark(self._mark_model, mark_dirty=False)
	def undo(self) -> None:
		scene = ferrum_qt.canvas.graphics_retirement.native_scene_for_item(self._mark_item)
		if not ferrum_qt.canvas.graphics_retirement.set_item_parent_in_captured_scene(self._mark_item, None, scene): return
		if scene is not None: ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(scene, self._mark_item)
		self._document.remove_mark(self._mark_model, mark_dirty=False)
	def graphics_items(self) -> list[PySide6.QtWidgets.QGraphicsItem]: return [self._mark_item]


#============================================
class RemoveAtomMarkCommand(AddAtomMarkCommand):
	"""Remove an atom-attached mark while retaining it for undo."""
	def __init__(self, document: ferrum_qt.models.document.Document,
			mark_model: ferrum_qt.models.document_object.AtomMarkModel,
			mark_item: PySide6.QtWidgets.QGraphicsItem,
			parent_atom_item: PySide6.QtWidgets.QGraphicsItem,
			text: str = "Remove Atom Mark") -> None:
		super().__init__(document, mark_model, mark_item, parent_atom_item, text)
	def redo(self) -> None: AddAtomMarkCommand.undo(self)
	def undo(self) -> None: AddAtomMarkCommand.redo(self)
