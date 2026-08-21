"""Rust-owned direct-root interaction methods for one Ferrum document tab."""


#============================================
class FerrumNativeDirectRootInteractionTabMixin:
	"""Keep direct-root interaction values inside the Rust session boundary."""

	#============================================
	def observe_direct_root_interaction(self) -> object:
		"""Return exact Rust-issued roots and interaction geometry for this snapshot."""
		self._require_mutable()
		snapshot = self.current_snapshot
		return self._session.observe_render_interaction_v1(
			snapshot.revision, snapshot.digest,
		)

	#============================================
	def observe_reaction_authoring_choices(self) -> object:
		"""Return Rust-classified complete roots for one fenced reaction form."""
		self._require_mutable()
		snapshot = self.current_snapshot
		return self._session.observe_reaction_authoring_choices_v1(
			snapshot.revision, snapshot.digest,
		)

	#============================================
	def validate_reaction_authoring_choices(self, choices: object) -> None:
		"""Reject a stale or foreign read-only reaction choice observation."""
		self._require_mutable()
		self._session.validate_reaction_authoring_choices_v1(choices)

	#============================================
	def create_reaction_v1(self, reactants: list[str], products: list[str],
			arrow: str, conditions: list[str], pluses: list[str]) -> object:
		"""Commit one renderer-preflighted reaction and install its Rust snapshot."""
		self._require_mutable()
		commit = self._session.create_reaction_v1(
			self.current_snapshot.revision, reactants, products, arrow, conditions, pluses,
		)
		self._install_mutation_result(commit.result)
		return commit

	#============================================
	def observe_reaction_list(self) -> object:
		"""Return the complete Rust-issued reaction inspection projection."""
		self._require_mutable()
		snapshot = self.current_snapshot
		return self._session.observe_reaction_list_v1(snapshot.revision, snapshot.digest)

	#============================================
	def select_reaction(self, observation: object, reaction_id: str) -> object:
		"""Acquire one opaque strict reaction capability from a fresh list."""
		self._require_mutable()
		return self._session.select_reaction_v1(observation, reaction_id)

	#============================================
	def patch_reaction_membership(
			self, selection: object, reactants: list[str], products: list[str],
			arrow: str, conditions: list[str], pluses: list[str],
			) -> object:
		"""Replace all roles through the renderer-preflighted Rust lifecycle bridge."""
		self._require_mutable()
		gesture = self._session.begin_reaction_membership_patch_v1(
			selection, self.current_snapshot.revision,
			reactants, products, arrow, conditions, pluses,
		)
		prepared = self._session.prepare_reaction_lifecycle_v1(gesture)
		commit = self._session.commit_reaction_lifecycle_v1(prepared)
		try:
			self._install_mutation_result(commit.result)
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit

	#============================================
	def delete_reaction_definition(self, selection: object) -> object:
		"""Remove only one selected strict definition through Rust."""
		self._require_mutable()
		gesture = self._session.begin_reaction_definition_delete_v1(selection)
		prepared = self._session.prepare_reaction_lifecycle_v1(gesture)
		commit = self._session.commit_reaction_lifecycle_v1(prepared)
		try:
			self._install_mutation_result(commit.result)
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit

	#============================================
	def translate_reaction(
			self, selection: object, delta_x: float, delta_y: float,
			view_hex_grid: bool,
			) -> object:
		"""Commit one opaque aggregate nudge without exposing member transforms."""
		self._require_mutable()
		gesture = self._session.begin_reaction_translation_v1(
			selection, 0.0, 0.0, view_hex_grid,
		)
		preview = self._session.preview_reaction_translation_v1(
			gesture, delta_x, delta_y,
		)
		prepared = self._session.prepare_reaction_translation_v1(gesture, preview)
		commit = self._session.commit_reaction_translation_v1(prepared)
		try:
			self._install_mutation_result(commit.result)
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit

	#============================================
	def select_direct_roots(
			self, observation: object, previous: object | None, query: object,
			) -> object:
		"""Resolve one Rust-owned point or marquee selection without Qt hit testing."""
		self._require_mutable()
		return self._session.select_render_interaction_roots_v1(
			observation, previous, query,
		)

	#============================================
	def begin_direct_root_translation(
			self, selection: object, x: float, y: float, snap: object,
			) -> object:
		"""Begin one opaque Rust-fenced direct-root translation gesture."""
		self._require_mutable()
		return self._session.begin_render_interaction_translation_v1(
			selection, x, y, snap,
		)

	#============================================
	def preview_direct_root_translation(
			self, gesture: object, x: float, y: float,
			) -> object:
		"""Request one disposable Rust-issued translation preview."""
		self._require_mutable()
		return self._session.preview_render_interaction_translation_v1(gesture, x, y)

	#============================================
	def commit_direct_root_translation(self, gesture: object, preview: object) -> object:
		"""Commit one checked Rust translation and install its observation."""
		self._require_mutable()
		commit = self._session.commit_render_interaction_translation_v1(gesture, preview)
		self._install_mutation_result(commit.result)
		return commit
