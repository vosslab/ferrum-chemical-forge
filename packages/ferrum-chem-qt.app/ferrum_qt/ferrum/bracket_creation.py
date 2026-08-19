"""Rust-owned bracket-pair creation for the standalone Ferrum document tab."""


#============================================
class FerrumNativeBracketCreationMixin:
	"""Create one durable bracket pair through the exact installed extension."""

	#============================================
	def create_rectangular_bracket(self, left: float, top: float,
			right: float, bottom: float) -> object:
		"""Create and select one Rust-owned rectangular bracket pair."""
		import ferrum_qt.ferrum.engine as engine
		return self._create_bracket_pair(
			engine.DocumentBracketStyleV1.rectangular,
			"polyline", left, top, right, bottom,
		)

	#============================================
	def create_round_bracket(self, left: float, top: float,
			right: float, bottom: float) -> object:
		"""Create and select one Rust-owned round bracket pair."""
		import ferrum_qt.ferrum.engine as engine
		return self._create_bracket_pair(
			engine.DocumentBracketStyleV1.round,
			"round_bracket", left, top, right, bottom,
		)

	#============================================
	def _create_bracket_pair(self, style: object, root_kind: str,
			left: float, top: float, right: float, bottom: float) -> object:
		"""Commit one exact bracket style and verify its durable projection."""
		self._require_mutable()
		if any(type(value) is not float for value in (left, top, right, bottom)):
			raise TypeError("Ferrum bracket bounds must be floats")
		import ferrum_qt.ferrum.engine as engine
		if type(style) is not engine.DocumentBracketStyleV1:
			raise TypeError("Ferrum bracket style must be an exact Rust bracket style")
		if root_kind not in ("polyline", "round_bracket"):
			raise ValueError("Ferrum bracket root kind is not supported")
		revision = self.current_snapshot.revision
		bounds = engine.DocumentBracketBoundsV1(left, top, right, bottom)
		prepared = self._session.prepare_create_bracket_v1(revision, style, bounds)
		result = self._session.commit_create_bracket(revision, prepared)
		stack = result.observation.projection.presentation_stack
		pairs = tuple(
			pair for pair in stack.bracket_pairs
			if pair.pair_id == prepared.pair_identifier
			and pair.member_ids == [
				prepared.left_identifier, prepared.right_identifier,
			]
			and pair.style is style
		)
		created = tuple(
			root.polyline for root in stack.roots
			if root.kind == root_kind
			and root.polyline.target.source_id in pairs[0].member_ids
		) if len(pairs) == 1 else ()
		if len(created) != 2 or any(
				polyline.target.id is None for polyline in created):
			self._install_mutation_result(result)
			import ferrum_qt.ferrum.document_tab
			raise (
				ferrum_qt.ferrum.document_tab
				.FerrumNativeDocumentTabError(
					"accepted bracket has no complete durable projected pair",
				)
			)
		self._install_mutation_result(
			result,
			tuple(("polyline", polyline.target.id) for polyline in created),
		)
		return result
