"""Rust-backed Ferrum bond creation operations for document tabs."""


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
			raise TypeError("Ferrum bond creation requires exact durable atom identifiers")
		import ferrum_qt.ferrum.engine as engine
		if type(presentation) is not engine.DocumentBondPresentationV1:
			raise TypeError("Ferrum bond creation requires an exact Ferrum presentation value")
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
		import ferrum_qt.ferrum.engine as engine
		return self.add_bond_between_atoms(
			selected[0], selected[1], engine.DocumentBondPresentationV1.normal_single,
		)
