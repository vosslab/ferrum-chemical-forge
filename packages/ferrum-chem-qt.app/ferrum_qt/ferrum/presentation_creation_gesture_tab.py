"""Rust-owned presentation-creation gesture methods for one Ferrum tab."""


#============================================
class FerrumNativePresentationCreationGestureTabMixin:
	"""Keep opaque presentation gesture handles inside the Rust session boundary."""

	#============================================
	def begin_straight_normal_arrow_gesture(
			self, x: float, y: float, snap: object,
			) -> object:
		"""Begin one backend-owned direct normal-arrow creation gesture."""
		self._require_mutable()
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum Arrow start coordinates must be exact floats")
		import ferrum_qt.ferrum.engine as engine
		if type(snap) is not engine.PresentationGestureSnapPolicyV1:
			raise TypeError("Ferrum Arrow creation requires an exact Rust snap policy")
		snapshot = self.current_snapshot
		return self._session.begin_presentation_creation_gesture_v1(
			snapshot.revision, snapshot.digest,
			engine.PresentationGestureKindV1.straight_normal_arrow,
			x, y, engine.ArrowGestureStyleV1(), snap,
		)

	#============================================
	def begin_plus_placement_gesture(self, x: float, y: float) -> object:
		"""Begin one backend-owned direct Plus placement gesture."""
		self._require_mutable()
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum Plus placement coordinates must be exact floats")
		snapshot = self.current_snapshot
		return self._session.begin_plus_placement_gesture_v1(
			snapshot.revision, snapshot.digest, x, y,
		)

	def preview_plus_placement_gesture(self, gesture: object) -> object:
		"""Return Rust's immutable Plus overlay for one click placement."""
		self._require_mutable()
		return self._session.preview_plus_placement_gesture_v1(gesture)

	def commit_plus_placement_gesture(self, gesture: object, preview: object) -> object:
		"""Commit one checked Plus gesture and install its authoritative projection."""
		self._require_mutable()
		commit = self._session.commit_plus_placement_gesture_v1(gesture, preview)
		try:
			self._install_mutation_result(commit.result, (("plus", commit.identifier),))
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit

	#============================================
	def preview_straight_normal_arrow_gesture(
			self, gesture: object, x: float, y: float,
			) -> object:
		"""Return Rust's immutable Arrow overlay for one raw pointer endpoint."""
		self._require_mutable()
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum Arrow endpoint coordinates must be exact floats")
		return self._session.preview_presentation_creation_gesture_v1(gesture, x, y)

	#============================================
	def commit_straight_normal_arrow_gesture(self, gesture: object, preview: object) -> object:
		"""Commit one checked opaque gesture and install its authoritative projection."""
		self._require_mutable()
		commit = self._session.commit_presentation_creation_gesture_v1(gesture, preview)
		try:
			self._install_mutation_result(commit.result, (("arrow", commit.root.identifier),))
		except Exception as exc:
			# The Rust receipt is already authoritative. Preserve it for the thin
			# controller recovery path when only disposable Qt installation failed.
			from ferrum_qt.ferrum.document_tab_errors import \
				FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit
