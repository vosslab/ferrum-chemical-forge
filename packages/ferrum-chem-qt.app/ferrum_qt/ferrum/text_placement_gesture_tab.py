"""Rust-owned standalone Text placement methods for one Ferrum tab."""


class FerrumNativeTextPlacementGestureTabMixin:
	"""Delegate opaque text gestures without constructing CDML in Qt."""

	def begin_text_placement_gesture(self, x: float, y: float) -> object:
		self._require_mutable()
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum Text placement coordinates must be exact floats")
		snapshot = self.current_snapshot
		return self._session.begin_text_placement_gesture_v1(
			snapshot.revision, snapshot.digest, x, y,
		)

	def preview_text_placement_gesture(
			self, gesture: object, runs: tuple[object, ...],
			font_size: int | None = None, color: str | None = None,
			) -> object:
		self._require_mutable()
		if type(runs) is not tuple:
			raise TypeError("Ferrum Text runs must be an immutable tuple")
		return self._session.preview_text_placement_gesture_v1(
			gesture, runs, font_size, color,
		)

	def text_placement_defaults(self, gesture: object) -> object:
		"""Return Rust-resolved authoring defaults for one opaque Text gesture."""
		self._require_mutable()
		return self._session.text_placement_defaults_v1(gesture)

	def commit_text_placement_gesture(self, gesture: object, preview: object) -> object:
		self._require_mutable()
		commit = self._session.commit_text_placement_gesture_v1(gesture, preview)
		try:
			self._install_mutation_result(commit.result, (("text", commit.document_object_id),))
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import \
				FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit
