"""Private Rust regular-ring session seam for a Ferrum document tab."""


class FerrumNativeRegularRingTabMixin:
	"""Prepare/commit one opaque detached ring receipt through the authoritative session."""

	def prepare_regular_ring(self, size: int, center_x: float, center_y: float,
			side_length: float) -> object:
		self._require_mutable()
		return self._session.prepare_admitted_regular_ring_insertion_v1(
			self.current_snapshot.revision, size, center_x, center_y, side_length,
		)

	def commit_regular_ring(self, prepared: object) -> object:
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(prepared) is not engine.AdmittedRegularRingInsertionV1:
			raise TypeError("Ferrum regular ring requires an exact Rust prepared receipt")
		result = self._session.commit_admitted_regular_ring_insertion_v1(
			self.current_snapshot.revision, prepared,
		)
		selection = tuple(("atom", identifier) for identifier in prepared.atom_identifiers)
		self._install_mutation_result(result, selection)
		return result
