"""Rust-owned direct atom/bond selection and atomic deletion for a Ferrum tab."""


#============================================
class FerrumNativeStructureInteractionTabMixin:
	"""Keep structural child selection opaque to the PySide6 scene."""

	#============================================
	def observe_structure_interaction(self) -> object:
		"""Return the exact fenced Rust child-hit observation for this snapshot."""
		self._require_mutable()
		snapshot = self.current_snapshot
		return self._session.observe_structure_interaction_v1(
			snapshot.revision, snapshot.digest,
		)

	#============================================
	def select_structure_interaction(
			self, observation: object, previous: object | None, query: object,
			) -> object:
		"""Resolve a Rust-owned direct atom/bond click, marquee, or clear request."""
		self._require_mutable()
		return self._session.select_structure_interaction_v1(
			observation, previous, query,
		)

	#============================================
	def commit_structure_deletion(self, selection: object) -> object:
		"""Commit one opaque atomic Rust structural deletion and reproject once."""
		self._require_mutable()
		commit = self._session.commit_structure_deletion_v1(selection)
		self._install_mutation_result(commit.result)
		return commit
