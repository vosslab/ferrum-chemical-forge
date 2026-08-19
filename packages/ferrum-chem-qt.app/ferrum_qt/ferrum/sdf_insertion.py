"""Atomic insertion of one exact Rust-prepared SDF record batch."""


#============================================
class FerrumNativeSdfInsertionTabMixin:
	"""Commit all source SDF records as one revision-bound document edit."""

	#============================================
	def insert_prepared_sdf_records(self, batch: object) -> object:
		"""Commit one frozen SDF batch and select every inserted atom."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(batch) is not engine.SdfMoleculeBatchInsertionV1:
			raise TypeError("Ferrum SDF insertion requires exact frozen Ferrum data")
		revision = self.current_snapshot.revision
		prepared = self._session.prepare_insert_sdf_records_v1(revision, batch)
		selection = tuple(
			("atom", atom_identifier)
			for record in prepared.atom_identifiers
			for atom_identifier in record
		)
		result = self._session.commit_create_sdf_records(revision, prepared)
		self._install_mutation_result(result, selection)
		return result
