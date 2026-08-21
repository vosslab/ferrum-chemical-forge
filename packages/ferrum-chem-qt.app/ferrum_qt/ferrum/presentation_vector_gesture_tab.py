"""Rust-owned two-point vector gesture methods for one Ferrum document tab."""


#============================================
class FerrumNativePresentationVectorGestureTabMixin:
	"""Keep opaque renderer-preflighted vector handles inside the tab boundary."""

	#============================================
	def begin_presentation_vector_gesture(self, kind: object, x: float, y: float) -> object:
		"""Begin one fenced Rust vector gesture from a raw Qt pointer coordinate."""
		self._require_mutable()
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum vector start coordinates must be exact floats")
		snapshot = self.current_snapshot
		return self._session.begin_presentation_vector_gesture_v1(
			snapshot.revision, snapshot.digest, kind, x, y,
		)

	#============================================
	def preview_presentation_vector_gesture(
			self, gesture: object, x: float, y: float,
			) -> object:
		"""Return one Rust-issued vector preview for the raw pointer endpoint."""
		self._require_mutable()
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum vector endpoint coordinates must be exact floats")
		return self._session.preview_presentation_vector_gesture_v1(gesture, x, y)

	#============================================
	def prepare_presentation_vector_gesture(self, gesture: object, preview: object) -> object:
		"""Ask the renderer bridge to preflight one opaque vector candidate."""
		self._require_mutable()
		return self._session.prepare_presentation_vector_gesture_v1(gesture, preview)

	#============================================
	def commit_presentation_vector_gesture(self, prepared: object) -> object:
		"""Commit one renderer-preflighted opaque receipt and install Rust truth."""
		self._require_mutable()
		commit = self._session.commit_presentation_vector_gesture_v1(prepared)
		try:
			self._install_mutation_result(commit.result)
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit
