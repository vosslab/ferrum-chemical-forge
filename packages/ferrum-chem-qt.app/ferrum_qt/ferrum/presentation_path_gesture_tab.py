"""Rust-owned multi-point presentation-path methods for one Ferrum document tab."""


#============================================
class FerrumNativePresentationPathGestureTabMixin:
	"""Keep opaque incremental Rust path capabilities inside the tab boundary."""

	#============================================
	def begin_presentation_path_gesture(self, kind: object) -> object:
		"""Begin one fenced Rust Polyline or Polygon gesture."""
		self._require_mutable()
		snapshot = self.current_snapshot
		return self._session.begin_presentation_path_gesture_v1(
			snapshot.revision, snapshot.digest, kind,
		)

	#============================================
	def add_presentation_path_gesture_point(self, gesture: object,
			x: float, y: float) -> object:
		"""Add one exact scene point and return Rust-owned gesture progress."""
		self._require_mutable()
		return self._session.add_presentation_path_gesture_point_v1(gesture, x, y)

	#============================================
	def preview_presentation_path_gesture(self, gesture: object,
			hover: tuple[float, float] | None) -> object:
		"""Return one Rust-issued overlay for accepted points and optional hover."""
		self._require_mutable()
		return self._session.preview_presentation_path_gesture_v1(gesture, hover)

	#============================================
	def prepare_presentation_path_gesture(self, gesture: object, preview: object) -> object:
		"""Ask the Rust renderer bridge to preflight one opaque path candidate."""
		self._require_mutable()
		return self._session.prepare_presentation_path_gesture_v1(gesture, preview)

	#============================================
	def cancel_presentation_path_gesture(self, gesture: object) -> None:
		"""Retire one opaque Rust path capability without changing the document."""
		self._session.cancel_presentation_path_gesture_v1(gesture)

	#============================================
	def commit_presentation_path_gesture(self, prepared: object) -> object:
		"""Commit one renderer-preflighted opaque path receipt and install Rust truth."""
		self._require_mutable()
		commit = self._session.commit_presentation_path_gesture_v1(prepared)
		try:
			self._install_mutation_result(commit.result)
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit
