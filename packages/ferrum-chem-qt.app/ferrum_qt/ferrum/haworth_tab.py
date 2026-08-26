"""Private Rust standalone-Haworth seam for one Ferrum document tab."""


class FerrumNativeHaworthTabMixin:
	"""Resolve and redeem standalone Haworth transitions through generic authority."""

	def resolve_standalone_haworth_transition(self, recipe: str, center_x: float,
			center_y: float) -> object:
		self._require_mutable()
		return self._session.resolve_standalone_haworth_transition_v1(
			self.current_snapshot.revision, recipe, center_x, center_y,
		)

	def prepare_standalone_haworth_transition(self, request: object) -> object:
		self._require_mutable()
		return self._session.prepare_session_operation_transition_v1(request)

	def commit_standalone_haworth_transition(self, prepared: object) -> object:
		self._require_mutable()
		result = self._session.commit_session_operation_transition_v1(prepared)
		selection = tuple(result.outcome.molecule_inserted.atom_identifiers)
		self._install_mutation_result(result, selection)
		return result
