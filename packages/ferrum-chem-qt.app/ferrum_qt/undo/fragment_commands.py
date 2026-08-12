"""Undo commands for molecule fragment metadata."""

# PIP3 modules
import PySide6.QtGui

# local repo modules
import ferrum_qt.models.fragment_model
import ferrum_qt.models.molecule_model


#============================================
class AddFragmentCommand(PySide6.QtGui.QUndoCommand):
	"""Add immutable fragment metadata without changing the molecular graph."""

	#============================================
	def __init__(
			self, molecule_model: ferrum_qt.models.molecule_model.MoleculeModel,
			fragment: ferrum_qt.models.fragment_model.FragmentModel,
			atom_id_changes: tuple[tuple[object, str, str], ...] = (),
			bond_id_changes: tuple[tuple[object, str, str], ...] = (),
			text: str = "Create Fragment",
			) -> None:
		"""Capture metadata and its required deterministic ID normalization."""
		super().__init__(text)
		self._molecule_model = molecule_model
		self._fragment = fragment
		self._position = len(molecule_model.fragments)
		self._atom_id_changes = tuple(atom_id_changes)
		self._bond_id_changes = tuple(bond_id_changes)

	#============================================
	def redo(self) -> None:
		"""Apply stable IDs, then insert the fragment at its original position."""
		self._apply_ids(after=True)
		self._molecule_model.insert_fragment(self._position, self._fragment)

	#============================================
	def undo(self) -> None:
		"""Remove metadata and restore every prior atom and bond ID exactly."""
		self._molecule_model.remove_fragment(self._fragment.fragment_id)
		self._apply_ids(after=False)

	#============================================
	def _apply_ids(self, after: bool) -> None:
		"""Apply the captured ID plan without changing graph topology."""
		for model, before, after_value in self._atom_id_changes:
			model.atom_id = after_value if after else before
		for model, before, after_value in self._bond_id_changes:
			model.bond_id = after_value if after else before


#============================================
class RemoveFragmentCommand(PySide6.QtGui.QUndoCommand):
	"""Remove editable fragment metadata without changing the molecular graph."""

	#============================================
	def __init__(
			self, molecule_model: ferrum_qt.models.molecule_model.MoleculeModel,
			fragment_id: str,
			text: str = "Remove Fragment",
			) -> None:
		"""Capture the exact fragment and its durable list position."""
		super().__init__(text)
		self._molecule_model = molecule_model
		for position, fragment in enumerate(molecule_model.fragments):
			if fragment.fragment_id == fragment_id:
				self._position = position
				self._fragment = fragment
				break
		else:
			raise ValueError("fragment ID is not editable metadata for this molecule")

	#============================================
	def redo(self) -> None:
		"""Remove the captured metadata from its owning molecule."""
		self._molecule_model.remove_fragment(self._fragment.fragment_id)

	#============================================
	def undo(self) -> None:
		"""Restore the exact immutable metadata at its original position."""
		self._molecule_model.insert_fragment(self._position, self._fragment)
