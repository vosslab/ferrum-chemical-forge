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
	def resolve_presentation_vector_gesture(self, gesture: object, preview: object) -> object:
		"""Resolve one validated vector gesture into a generic transition request."""
		self._require_mutable()
		return self._session.resolve_presentation_vector_gesture_v1(gesture, preview)
