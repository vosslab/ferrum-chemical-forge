"""Atomic insertion of one exact Rust-prepared SDF record batch."""


#============================================
class FerrumNativeSdfInsertionTabMixin:
	"""Commit all source SDF records as one revision-bound document edit."""

	#============================================
	def insert_prepared_sdf_records(self, batch: object) -> object:
		"""Commit one frozen SDF batch and select every inserted atom."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(batch) is not engine.InterchangeRecordBatchInsertionV1:
			raise TypeError("Ferrum SDF insertion requires exact frozen Ferrum data")
		revision = self.current_snapshot.revision
		operation = engine.DocumentOperationV1.insert_interchange_record_batch_v1(batch)
		request = operation.transition_request_v1(revision)
		prepared = self.prepare_session_operation_transition_v1(request)
		result = self.commit_session_operation_transition_v1(prepared)
		outcome = result.outcome
		if (
			outcome.kind != "interchange_record_batch_inserted_v1"
			or outcome.interchange_record_batch_inserted is None
		):
			raise RuntimeError("Ferrum SDF insertion returned an unknown operation outcome")
		selection = tuple(
			atom_identifier
			for record in outcome.interchange_record_batch_inserted.records
			for atom_identifier in record.atom_identifiers
		)
		self._install_mutation_result(result, selection)
		return result
