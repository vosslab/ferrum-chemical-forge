"""Undo commands that add or remove complete molecule projections."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.models.document
import ferrum_qt.models.molecule_model


#============================================
class AddMoleculeCommand(PySide6.QtGui.QUndoCommand):
	"""Add or remove one complete molecule and its graphics atomically."""

	#============================================
	def __init__(
			self, document: ferrum_qt.models.document.Document,
			scene: PySide6.QtWidgets.QGraphicsScene,
			molecule_model: ferrum_qt.models.molecule_model.MoleculeModel,
			graphics_items: list[PySide6.QtWidgets.QGraphicsItem],
			text: str = "Add Molecule", index: int | None = None,
			) -> None:
		"""Capture one complete molecule projection and its stack slot."""
		super().__init__(text)
		self._document = document
		self._scene = scene
		self._molecule_model = molecule_model
		self._graphics_items = list(graphics_items)
		self._stack_index = index

	#============================================
	def redo(self) -> None:
		"""Add the molecule model and all of its graphics items."""
		if not all(ferrum_qt.canvas.graphics_retirement.can_add_item_to_captured_scene(
				self._scene, item) for item in self._graphics_items):
			return
		self._document.add_molecule(self._molecule_model, mark_dirty=False,
			index=self._stack_index)
		for item in self._graphics_items:
			ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, item)

	#============================================
	def undo(self) -> None:
		"""Remove all graphics items and the molecule model."""
		if not all(ferrum_qt.canvas.graphics_retirement.item_belongs_to_scene(
				self._scene, item) for item in self._graphics_items):
			return
		for item in reversed(self._graphics_items):
			ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(
				self._scene, item)
		self._document.remove_molecule(self._molecule_model, mark_dirty=False)

	#============================================
	def graphics_items(self) -> list:
		"""Return graphics retained by this command across undo states."""
		return list(self._graphics_items)


#============================================
class RemoveMoleculeCommand(PySide6.QtGui.QUndoCommand):
	"""Remove one complete molecule and retain its scene projection for undo."""

	#============================================
	def __init__(
			self, document: ferrum_qt.models.document.Document,
			scene: PySide6.QtWidgets.QGraphicsScene,
			molecule_model: ferrum_qt.models.molecule_model.MoleculeModel,
			graphics_items: list[PySide6.QtWidgets.QGraphicsItem],
			text: str = "Remove Molecule",
			) -> None:
		"""Capture a molecule and its original canonical stack position."""
		super().__init__(text)
		self._document = document
		self._scene = scene
		self._molecule_model = molecule_model
		self._graphics_items = list(graphics_items)
		self._stack_index = document.object_index(molecule_model)

	#============================================
	def redo(self) -> None:
		"""Detach molecule graphics and remove the model from its document."""
		if not self._document.is_current_projection_scene(self._scene):
			return
		if not all(ferrum_qt.canvas.graphics_retirement.item_belongs_to_scene(
				self._scene, item) for item in self._graphics_items):
			return
		for item in reversed(self._graphics_items):
			ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(
				self._scene, item)
		self._document.remove_molecule(self._molecule_model, mark_dirty=False)
		self._document._synchronize_scene_object_stack()

	#============================================
	def undo(self) -> None:
		"""Restore molecule ownership, graphics, and its original stack slot."""
		if not self._document.is_current_projection_scene(self._scene):
			return
		if not all(ferrum_qt.canvas.graphics_retirement.can_add_item_to_captured_scene(
				self._scene, item) for item in self._graphics_items):
			return
		self._document.insert_molecule(self._molecule_model, index=self._stack_index,
			mark_dirty=False)
		for item in self._graphics_items:
			ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(self._scene, item)
		self._document._synchronize_scene_object_stack()

	#============================================
	def graphics_items(self) -> list:
		"""Return graphics retained by this command across undo states."""
		return list(self._graphics_items)
