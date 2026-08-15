"""Private Rust standalone-Haworth seam for one native document tab."""


class FerrumNativeHaworthTabMixin:
	"""Prepare and commit an opaque native D-glucose Haworth receipt."""

	def prepare_standalone_haworth(self, recipe: str, center_x: float,
			center_y: float) -> object:
		self._require_mutable()
		return self._session.prepare_create_standalone_haworth_v1(
			self.current_snapshot.revision, recipe, center_x, center_y,
		)

	def commit_standalone_haworth(self, prepared: object) -> object:
		self._require_mutable()
		import ferrum_chem
		if type(prepared) is not ferrum_chem.PreparedStandaloneHaworthInsertionV1:
			raise TypeError("native Haworth insertion requires an exact Rust prepared receipt")
		result = self._session.commit_create_standalone_haworth_v1(
			self.current_snapshot.revision, prepared,
		)
		selection = tuple(("atom", identifier) for identifier in prepared.atom_identifiers)
		self._install_mutation_result(result, selection)
		return result
