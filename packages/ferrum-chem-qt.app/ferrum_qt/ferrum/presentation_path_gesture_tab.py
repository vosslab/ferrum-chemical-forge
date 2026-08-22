"""Rust-owned multi-point presentation-path methods for one Ferrum document tab."""


#============================================
class FerrumNativePresentationPathGestureTabMixin:
	"""Keep opaque renderer-preflighted path handles inside the tab boundary."""

	#============================================
	def begin_presentation_path_gesture(self, kind: object) -> object:
		"""Begin one fenced Rust Polyline or Polygon gesture."""
		self._require_mutable()
		snapshot = self.current_snapshot
		return self._session.begin_presentation_path_gesture_v1(
			snapshot.revision, snapshot.digest, kind,
		)

	#============================================
	def preview_presentation_path_gesture(
			self, gesture: object, points: tuple[tuple[float, float], ...],
			) -> object:
		"""Return one Rust-issued path preview for ordered exact scene points."""
		self._require_mutable()
		if type(points) is not tuple or any(
			type(point) is not tuple or len(point) != 2
			or type(point[0]) is not float or type(point[1]) is not float
			for point in points
		):
			raise TypeError("Ferrum path points must be ordered exact float pairs")
		return self._session.preview_presentation_path_gesture_v1(gesture, points)

	#============================================
	def prepare_presentation_path_gesture(self, gesture: object, preview: object) -> object:
		"""Ask the Rust renderer bridge to preflight one opaque path candidate."""
		self._require_mutable()
		return self._session.prepare_presentation_path_gesture_v1(gesture, preview)

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
