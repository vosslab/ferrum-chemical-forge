"""Private Rust direct-glycosidic Haworth seam for a Ferrum document tab."""


class FerrumNativeDirectGlycosidicHaworthTabMixin:
	"""Keep structural SMILES and every durable drawing fact inside Rust."""

	def prepare_direct_glycosidic_haworth_source(self, smiles: str) -> object:
		"""Validate one structural request without changing this document."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		return engine.prepare_direct_haworth_from_smiles_v1(smiles)

	def resolve_direct_glycosidic_haworth_transition(self, source: object,
			anchor_x: float, anchor_y: float) -> object:
		"""Resolve source and anchor into the generic document transition."""
		self._require_mutable()
		return self._session.resolve_direct_haworth_transition_v1(
			self.current_snapshot.revision, source, anchor_x, anchor_y,
		)

	def prepare_direct_glycosidic_haworth_transition(self, request: object) -> object:
		"""Prepare one Haworth request through generic authority."""
		self._require_mutable()
		return self._session.prepare_session_operation_transition_v1(request)

	def commit_direct_glycosidic_haworth_transition(self, prepared: object) -> object:
		"""Redeem one renderer-admitted generic Haworth transition."""
		self._require_mutable()
		result = self._session.commit_session_operation_transition_v1(prepared)
		selection = tuple(("atom", identifier)
			for identifier in result.outcome.molecule_inserted.atom_identifiers)
		self._install_mutation_result(result, selection)
		return result
