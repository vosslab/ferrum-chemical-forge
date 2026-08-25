"""Rust-backed Ferrum bond creation operations for document tabs."""

from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabError


#============================================
class FerrumNativeBondCreationMixin:
	"""Commit explicit durable atom endpoints through closed Rust operations."""

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
		# Resolve the live intent through the authoritative session so it owns the
		# durable-target, revision-fence, and authoring-capability checks.
		request = self._session.resolve_create_bond_v1(
			self.current_snapshot.revision, start_atom_id, end_atom_id, presentation,
		)
		prepared = self._session.prepare_session_operation_transition_v1(request)
		result = self._session.commit_session_operation_transition_v1(prepared)
		outcome = result.outcome
		if outcome.kind != "bond_created_v1" or outcome.bond_created is None:
			raise FerrumNativeDocumentTabError("Ferrum bond creation returned an unknown operation outcome")
		self._install_mutation_result(result, (("bond", outcome.bond_created.bond_identifier),))
		return result

	#============================================
	def add_single_bond_between_selected_atoms(self) -> object:
		"""Connect exactly two selected durable atoms with the explicit Single command."""
		selected = self._selected_atom_identifiers(2)
		import ferrum_qt.ferrum.engine as engine
		return self.add_bond_between_atoms(
			selected[0], selected[1], engine.DocumentBondPresentationV1.normal_single,
		)
