"""Rust-owned presentation-creation gesture methods for one Ferrum tab."""

# local repo modules
import ferrum_qt.ferrum.curved_equilibrium_arrow
import ferrum_qt.ferrum.terminal_arrow


#============================================
class FerrumNativePresentationCreationGestureTabMixin:
	"""Keep opaque presentation gesture handles inside the Rust session boundary."""

	#============================================
	def begin_straight_normal_arrow_gesture(
			self, x: float, y: float, snap: object,
			) -> object:
		"""Begin one backend-owned direct normal-arrow creation gesture."""
		import ferrum_qt.ferrum.engine as engine
		return self._begin_straight_presentation_arrow_gesture(
			engine.PresentationGestureKindV1.straight_normal_arrow, x, y, snap,
		)

	#============================================
	def begin_straight_equilibrium_arrow_gesture(
			self, x: float, y: float, snap: object,
			) -> object:
		"""Begin one backend-owned direct equilibrium-arrow creation gesture."""
		import ferrum_qt.ferrum.engine as engine
		return self._begin_straight_presentation_arrow_gesture(
			engine.PresentationGestureKindV1.straight_equilibrium_arrow, x, y, snap,
		)

	#============================================
	def begin_terminal_arrow_gesture(self, kind: ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind,
			start: tuple[float, float], control: tuple[float, float]) -> object:
		"""Begin one explicit Rust terminal-arrow operation from two exact Qt points."""
		self._require_mutable()
		if not _is_exact_point(start) or not _is_exact_point(control):
			raise TypeError("Ferrum terminal-arrow points must be exact float pairs")
		snapshot = self.current_snapshot
		return ferrum_qt.ferrum.terminal_arrow.TerminalArrowOperation.begin(
			self._session, snapshot.revision, snapshot.digest, kind, start, control,
		)

	#============================================
	def begin_curved_equilibrium_arrow_gesture(self, start: tuple[float, float],
			control: tuple[float, float]) -> object:
		"""Begin one dedicated Rust curved-equilibrium operation."""
		self._require_mutable()
		if not _is_exact_point(start) or not _is_exact_point(control):
			raise TypeError("Ferrum curved-equilibrium points must be exact float pairs")
		snapshot = self.current_snapshot
		return ferrum_qt.ferrum.curved_equilibrium_arrow.CurvedEquilibriumArrowOperation.begin(
			self._session, snapshot.revision, snapshot.digest, start, control,
		)

	#============================================
	def preview_curved_equilibrium_arrow_gesture(self, gesture: object,
			end: tuple[float, float]) -> object:
		"""Return Rust's complete immutable two-lane equilibrium preview."""
		self._require_mutable()
		if not _is_exact_point(end):
			raise TypeError("Ferrum curved-equilibrium endpoint must be an exact float pair")
		return ferrum_qt.ferrum.curved_equilibrium_arrow.CurvedEquilibriumArrowOperation.preview(
			self._session, gesture, end,
		)

	#============================================
	def prepare_curved_equilibrium_arrow_gesture(self, gesture: object,
			preview: object) -> object:
		"""Renderer-preflight one opaque curved-equilibrium candidate."""
		self._require_mutable()
		return ferrum_qt.ferrum.curved_equilibrium_arrow.CurvedEquilibriumArrowOperation.prepare(
			self._session, gesture, preview,
		)

	#============================================
	def commit_curved_equilibrium_arrow_gesture(self, prepared: object) -> object:
		"""Commit one prepared curved-equilibrium receipt and install Rust truth."""
		self._require_mutable()
		commit = ferrum_qt.ferrum.curved_equilibrium_arrow.CurvedEquilibriumArrowOperation.commit(
			self._session, prepared,
		)
		try:
			self._install_mutation_result(commit.result, (("arrow", commit.root.identifier),))
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit

	#============================================
	def preview_terminal_arrow_gesture(self, kind: ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind,
			gesture: object, end: tuple[float, float]) -> object:
		"""Return Rust's complete immutable terminal-arrow overlay for one endpoint."""
		self._require_mutable()
		if not _is_exact_point(end):
			raise TypeError("Ferrum terminal-arrow endpoint must be an exact float pair")
		return ferrum_qt.ferrum.terminal_arrow.TerminalArrowOperation.preview(
			self._session, kind, gesture, end,
		)

	#============================================
	def prepare_terminal_arrow_gesture(self, kind: ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind,
			gesture: object, preview: object) -> object:
		"""Renderer-preflight one opaque terminal-arrow candidate."""
		self._require_mutable()
		return ferrum_qt.ferrum.terminal_arrow.TerminalArrowOperation.prepare(
			self._session, kind, gesture, preview,
		)

	#============================================
	def commit_terminal_arrow_gesture(self, kind: ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind,
			prepared: object) -> object:
		"""Commit one renderer-preflighted terminal arrow and install Rust truth."""
		self._require_mutable()
		commit = ferrum_qt.ferrum.terminal_arrow.TerminalArrowOperation.commit(
			self._session, kind, prepared,
		)
		try:
			self._install_mutation_result(commit.result, (("arrow", commit.root.identifier),))
		except Exception as exc:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit

	#============================================
	def _begin_straight_presentation_arrow_gesture(
			self, kind: object, x: float, y: float, snap: object,
			) -> object:
		"""Begin one exact Rust-owned straight presentation-arrow kind."""
		self._require_mutable()
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum Arrow start coordinates must be exact floats")
		import ferrum_qt.ferrum.engine as engine
		if type(snap) is not engine.PresentationGestureSnapPolicyV1:
			raise TypeError("Ferrum Arrow creation requires an exact Rust snap policy")
		snapshot = self.current_snapshot
		style = None
		if kind is engine.PresentationGestureKindV1.straight_normal_arrow:
			style = engine.ArrowGestureStyleV1()
		return self._session.begin_presentation_creation_gesture_v1(
			snapshot.revision, snapshot.digest,
			kind,
			x, y, style, snap,
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
	def preview_straight_presentation_arrow_gesture(
			self, gesture: object, x: float, y: float,
			) -> object:
		"""Return Rust's immutable straight-arrow overlay for one pointer endpoint."""
		self._require_mutable()
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum Arrow endpoint coordinates must be exact floats")
		return self._session.preview_presentation_creation_gesture_v1(gesture, x, y)

	#============================================
	def commit_straight_normal_arrow_gesture(self, gesture: object, preview: object) -> object:
		"""Commit one checked normal-arrow gesture and install its projection."""
		return self._commit_straight_presentation_arrow_gesture("arrow", gesture, preview)

	#============================================
	def commit_straight_equilibrium_arrow_gesture(self, gesture: object, preview: object) -> object:
		"""Commit one checked equilibrium-arrow gesture and install its projection."""
		return self._commit_straight_presentation_arrow_gesture("equilibrium_arrow", gesture, preview)

	#============================================
	def _commit_straight_presentation_arrow_gesture(
			self, root_kind: str, gesture: object, preview: object,
			) -> object:
		"""Commit one checked opaque straight-arrow gesture with its exact root kind."""
		self._require_mutable()
		commit = self._session.commit_presentation_creation_gesture_v1(gesture, preview)
		try:
			self._install_mutation_result(commit.result, ((root_kind, commit.root.identifier),))
		except Exception as exc:
			# The Rust receipt is already authoritative. Preserve it for the thin
			# controller recovery path when only disposable Qt installation failed.
			from ferrum_qt.ferrum.document_tab_errors import \
				FerrumNativeDocumentTabMutationPresentationError
			if isinstance(exc, FerrumNativeDocumentTabMutationPresentationError):
				exc.accepted_receipt = commit
			raise
		return commit


#============================================
def _is_exact_point(value: object) -> bool:
	"""Accept one immutable scene coordinate pair at the Qt/native boundary."""
	return (
		type(value) is tuple and len(value) == 2
		and type(value[0]) is float and type(value[1]) is float
	)
