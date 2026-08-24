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
	def resolve_reaction_create(self, reactants: list[str], products: list[str],
			arrow: str, conditions: list[str], pluses: list[str]) -> object:
		"""Resolve one authored reaction into an opaque generic transition request."""
		self._require_mutable()
		snapshot = self.current_snapshot
		gesture = self._session.begin_reaction_gesture_v1(
			snapshot.revision, snapshot.digest, reactants, products, arrow, conditions, pluses,
		)
		return self._session.resolve_reaction_gesture_v1(gesture)

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
	def resolve_reaction_membership_patch(
			self, selection: object, reactants: list[str], products: list[str],
			arrow: str, conditions: list[str], pluses: list[str],
			) -> object:
		"""Resolve one role replacement into an opaque generic transition request."""
		self._require_mutable()
		gesture = self._session.begin_reaction_membership_patch_v1(
			selection, self.current_snapshot.revision,
			reactants, products, arrow, conditions, pluses,
		)
		return self._session.resolve_reaction_lifecycle_v1(gesture)

	#============================================
	def resolve_reaction_definition_delete(self, selection: object) -> object:
		"""Resolve one strict-definition deletion into a generic transition request."""
		self._require_mutable()
		gesture = self._session.begin_reaction_definition_delete_v1(selection)
		return self._session.resolve_reaction_lifecycle_v1(gesture)

	#============================================
	def resolve_reaction_translation(
			self, selection: object, delta_x: float, delta_y: float,
			view_hex_grid: bool,
			) -> object:
		"""Resolve one aggregate nudge into an opaque generic transition request."""
		self._require_mutable()
		gesture = self._session.begin_reaction_translation_v1(
			selection, 0.0, 0.0, view_hex_grid,
		)
		return self._session.resolve_reaction_translation_v1(gesture, delta_x, delta_y)

	#============================================
	def install_reaction_created_result(self, result: object) -> object:
		"""Install a generic reaction-create result and return its typed outcome."""
		outcome = result.outcome.reaction_created
		if outcome is None:
			raise RuntimeError("Ferrum reaction transition did not create a reaction")
		self._install_reaction_transition_result(result)
		return outcome

	#============================================
	def install_reaction_membership_replaced_result(self, result: object) -> object:
		"""Install a generic membership result and return its typed outcome."""
		outcome = result.outcome.reaction_membership_replaced
		if outcome is None:
			raise RuntimeError("Ferrum reaction transition did not replace membership")
		self._install_reaction_transition_result(result)
		return outcome

	#============================================
	def install_reaction_definition_deleted_result(self, result: object) -> object:
		"""Install a generic definition-delete result and return its typed outcome."""
		outcome = result.outcome.reaction_definition_deleted
		if outcome is None:
			raise RuntimeError("Ferrum reaction transition did not delete a definition")
		self._install_reaction_transition_result(result)
		return outcome

	#============================================
	def install_reaction_translation_result(self, result: object) -> object:
		"""Install one generic translation while leaving selection ownership to the caller."""
		if result.outcome.kind != "standard":
			raise RuntimeError("Ferrum reaction translation returned an unknown outcome")
		self._install_reaction_transition_result(result)
		return result.outcome

	#============================================
	def _install_reaction_transition_result(self, result: object) -> None:
		"""Install one already-validated generic reaction result."""
		try:
			self._install_mutation_result(result)
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = result
			raise

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
	def direct_root_selection_contains_point(
			self, selection: object, x: float, y: float,
			) -> bool:
		"""Ask Rust whether a press lands on one current selected complete root."""
		self._require_mutable()
		return self._session.render_interaction_selection_contains_point_v1(selection, x, y)

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
	def commit_direct_root_translation(self, gesture: object,
			release_x: float, release_y: float) -> object:
		"""Commit one Rust translation at the actual release scene point."""
		self._require_mutable()
		commit = self._session.commit_render_interaction_translation_v1(
			gesture, release_x, release_y,
		)
		try:
			self._install_mutation_result(commit.result)
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit
