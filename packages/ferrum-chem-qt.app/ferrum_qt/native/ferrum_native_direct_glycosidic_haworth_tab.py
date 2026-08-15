"""Private Rust direct-glycosidic Haworth seam for a native document tab."""


class FerrumNativeDirectGlycosidicHaworthTabMixin:
	"""Keep structural SMILES and every durable drawing fact inside Rust."""

	def prepare_direct_glycosidic_haworth_source(self, smiles: str) -> object:
		"""Validate one structural request without changing this document."""
		self._require_mutable()
		import ferrum_chem
		return ferrum_chem.prepare_direct_haworth_from_smiles_v1(smiles)

	def prepare_direct_glycosidic_haworth_placement(self, source: object,
			anchor_x: float, anchor_y: float) -> object:
		"""Ask Rust to bind one opaque source receipt to one exact anchor."""
		self._require_mutable()
		return self._session.prepare_create_direct_haworth_v1(
			self.current_snapshot.revision, source, anchor_x, anchor_y,
		)

	def commit_direct_glycosidic_haworth(self, prepared: object) -> object:
		"""Commit one authenticated Rust receipt and install its normal observation."""
		self._require_mutable()
		import ferrum_chem
		if type(prepared) is not ferrum_chem.PreparedDirectHaworthInsertionV1:
			raise TypeError("native direct-glycosidic Haworth requires a Rust receipt")
		result = self._session.commit_create_direct_haworth_v1(
			self.current_snapshot.revision, prepared,
		)
		selection = tuple(("atom", identifier) for identifier in prepared.atom_identifiers)
		self._install_mutation_result(result, selection)
		return result
