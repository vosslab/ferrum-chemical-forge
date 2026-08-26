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
	def resolve_presentation_path_gesture(self, gesture: object, preview: object) -> object:
		"""Resolve one validated path gesture into a generic transition request."""
		self._require_mutable()
		return self._session.resolve_presentation_path_gesture_v1(gesture, preview)

	#============================================
	def cancel_presentation_path_gesture(self, gesture: object) -> None:
		"""Cancel one opaque Rust path capability without changing the document."""
		self._session.cancel_presentation_path_gesture_v1(gesture)

	#============================================
