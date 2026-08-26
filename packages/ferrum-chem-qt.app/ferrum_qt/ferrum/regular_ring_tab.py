"""Generic Rust regular-ring mutations for a Ferrum document tab."""


class FerrumNativeRegularRingTabMixin:
	"""Commit an ordinary ring through the generic document operation lifecycle."""

	def insert_regular_ring(self, size: int, center_x: float, center_y: float,
			side_length: float) -> object:
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		operation = engine.DocumentOperationV1.insert_regular_ring_v1(
			size, center_x, center_y, side_length,
		)
		request = operation.transition_request_v1(self.current_snapshot.revision)
		prepared = self.prepare_session_operation_transition_v1(request)
		result = self.commit_session_operation_transition_v1(prepared)
		outcome = result.outcome
		if outcome.kind != "molecule_inserted_v1" or outcome.molecule_inserted is None:
			raise RuntimeError("Ferrum regular-ring insertion returned an unknown operation outcome")
		selection = tuple(outcome.molecule_inserted.atom_identifiers)
		self._install_mutation_result(result, selection)
		return result
