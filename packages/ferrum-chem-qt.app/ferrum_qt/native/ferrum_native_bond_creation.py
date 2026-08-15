"""Rust-backed native bond creation operations for document tabs."""


#============================================
class FerrumNativeBondCreationMixin:
	"""Commit explicit durable atom endpoints through prepared Rust operations."""

	#============================================
	def add_bond_between_atoms(self, start_atom_id: str, end_atom_id: str,
			presentation: object) -> object:
		"""Create one explicit atom-to-atom bond without changing selection first."""
		self._require_mutable()
		if (
			type(start_atom_id) is not str
			or not start_atom_id
			or type(end_atom_id) is not str
			or not end_atom_id
		):
			raise TypeError("native bond creation requires exact durable atom identifiers")
		import ferrum_chem
		if type(presentation) is not ferrum_chem.DocumentBondPresentationV1:
			raise TypeError("native bond creation requires an exact Ferrum presentation value")
		start, end = self._atom_object_ids((start_atom_id, end_atom_id))
		revision = self.current_snapshot.revision
		prepared = self._session.prepare_create_bond_v2(revision, start, end, presentation)
		result = self._session.commit_create_bond(revision, prepared)
		self._install_mutation_result(result, (("bond", prepared.identifier),))
		return result

	#============================================
	def add_single_bond_between_selected_atoms(self) -> object:
		"""Connect exactly two selected durable atoms with the explicit Single command."""
		selected = self._selected_atom_identifiers(2)
		import ferrum_chem
		return self.add_bond_between_atoms(
			selected[0], selected[1], ferrum_chem.DocumentBondPresentationV1.normal_single,
		)

	#============================================
	def add_bonded_atom_at(self, start_atom_id: str, element: str,
			x: float, y: float, presentation: object) -> object:
		"""Create one atom plus an explicit-order bond through one Rust transaction."""
		self._require_mutable()
		if type(start_atom_id) is not str or not start_atom_id or type(element) is not str:
			raise TypeError("native bonded-atom insertion requires atom and element strings")
		if type(x) is not float or type(y) is not float:
			raise TypeError("native bonded-atom insertion coordinates must be floats")
		import ferrum_chem
		if type(presentation) is not ferrum_chem.DocumentBondPresentationV1:
			raise TypeError("native bonded-atom insertion requires an exact Ferrum presentation value")
		(start_object_id,) = self._atom_object_ids((start_atom_id,))
		revision = self.current_snapshot.revision
		prepared = self._session.prepare_create_bonded_atom_v2(
			revision, start_object_id, element, x, y, 0.0, presentation,
		)
		result = self._session.commit_create_bonded_atom(revision, prepared)
		self._install_mutation_result(result, (("atom", prepared.atom_identifier),))
		return result
