"""Private Qt bridge for one Rust-owned attached cyclohexane gesture."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore


#============================================
class FerrumNativeAttachedCyclohexaneTabMixin:
	"""Keep the opaque C6 candidate behind the native document-tab boundary."""

	def begin_attached_cyclohexane(self, atom_id: str,
			release: PySide6.QtCore.QPointF) -> object:
		"""Start one fenced Rust C6 candidate from copied pointer facts."""
		self._require_mutable()
		if type(atom_id) is not str or not atom_id:
			raise ValueError("Choose an eligible existing atom to attach a cyclohexane ring.")
		if not math.isfinite(release.x()) or not math.isfinite(release.y()):
			raise ValueError("Choose a finite attachment direction and try again.")
		snapshot = self.current_snapshot
		return self._session._begin_attach_cyclohexane_v1(
			snapshot.revision, snapshot.digest, atom_id,
			float(release.x()), float(release.y()),
		)

	def preview_attached_cyclohexane(self, pending: object) -> tuple[PySide6.QtCore.QPointF, ...]:
		"""Copy and validate Rust preview vertices before Qt paints them."""
		self._require_mutable()
		preview = self._session._preview_attach_cyclohexane_v1(pending)
		vertices = tuple(
			PySide6.QtCore.QPointF(float(vertex.x), float(vertex.y))
			for vertex in preview.vertices
		)
		if len(vertices) != 6 or any(
			not math.isfinite(vertex.x()) or not math.isfinite(vertex.y())
			for vertex in vertices
		):
			raise ValueError("Ferrum attachment preview is unavailable; try again.")
		return vertices

	def commit_attached_cyclohexane(self, pending: object) -> object:
		"""Commit once, then reobserve the authoritative Rust document."""
		self._require_mutable()
		result = self._session._commit_attach_cyclohexane_v1(pending)
		self._refresh_from_current_revision()
		return result

	def cancel_attached_cyclohexane(self, pending: object) -> None:
		"""Retire one opaque Rust candidate without mutating the document."""
		self._session._cancel_attach_cyclohexane_v1(pending)
