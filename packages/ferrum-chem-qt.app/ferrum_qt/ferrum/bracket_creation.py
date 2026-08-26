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
		existing_member_pairs = frozenset(
			tuple(pair.members)
			for pair in self._require_projection().presentation_stack.bracket_pairs
			if type(pair) is engine.BracketPairProjectionV1
		)
		bounds = engine.DocumentBracketBoundsV1(left, top, right, bottom)
		prepared = self._session.prepare_create_bracket_v1(revision, style, bounds)
		result = self._session.commit_create_bracket(revision, prepared)
		stack = result.observation.projection.presentation_stack
		pairs = tuple(
			pair for pair in stack.bracket_pairs
			if type(pair) is engine.BracketPairProjectionV1
			and type(pair.members) in (list, tuple)
			and len(pair.members) == 2
			and all(type(member) is str for member in pair.members)
			and pair.members[0] != pair.members[1]
			and tuple(pair.members) not in existing_member_pairs
			and pair.style is style
		)
		created_by_member: dict[str, object] = {}
		if len(pairs) == 1:
			for root in stack.entries:
				if root.kind != root_kind or root.polyline is None:
					continue
				document_object_id = root.polyline.target.document_object_id
				if document_object_id not in pairs[0].members:
					continue
				if (
						type(document_object_id) is not str
						or document_object_id in created_by_member
					):
					created_by_member = {}
					break
				created_by_member[document_object_id] = root.polyline
		created = tuple(
			created_by_member.get(member) for member in pairs[0].members
		) if len(pairs) == 1 else ()
		if len(created) != 2 or any(polyline is None for polyline in created):
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
			tuple(polyline.target.document_object_id for polyline in created),
		)
		return result
