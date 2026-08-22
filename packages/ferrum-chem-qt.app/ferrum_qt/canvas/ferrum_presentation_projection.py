"""Expose Ferrum presentation projection types and scene replacement control."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
from ferrum_qt.canvas._ferrum_presentation_projection_builder import (
	ArrowProjectionItem,
	BracketPair,
	FerrumPresentationProjection,
	PolylineProjectionItem,
	PresentationIssue,
	PresentationProjectionError,
	PresentationProjectionItem,
	PresentationTarget,
	ShapeProjectionItem,
	build_presentation_projection,
	_target as build_presentation_target,
)


__all__ = (
	"ArrowProjectionItem",
	"BracketPair",
	"FerrumPresentationProjection",
	"FerrumPresentationProjectionController",
	"PolylineProjectionItem",
	"PresentationIssue",
	"PresentationProjectionError",
	"PresentationProjectionItem",
	"PresentationTarget",
	"ShapeProjectionItem",
	"build_presentation_projection",
	"presentation_target_from_dto",
)


#============================================
def presentation_target_from_dto(value: object, extension: object,
		expected_kind: str | None = None) -> PresentationTarget:
	"""Copy one authenticated native target into immutable projection state."""
	return build_presentation_target(value, extension, expected_kind)


class FerrumPresentationProjectionController:
	"""Atomically expose exact current Ferrum observations in one Qt scene."""

	#============================================
	def __init__(self, scene: PySide6.QtWidgets.QGraphicsScene) -> None:
		"""Bind this disposable controller to one live Qt scene."""
		if not isinstance(scene, PySide6.QtWidgets.QGraphicsScene):
			raise TypeError("Ferrum presentation projection controller requires a graphics scene")
		self._scene = scene
		self.projection: FerrumPresentationProjection | None = None
		self.last_replacement_error: Exception | None = None
		self.retained_disposal_errors: list[Exception] = []
		self.retained_transition_errors: list[Exception] = []
		self.retained_prior_projection: FerrumPresentationProjection | None = None
		self.retained_candidate_projection: FerrumPresentationProjection | None = None
		self._transition_invalidated = False

	#============================================
	def replace(self, observation: object) -> bool:
		"""Install one newer exact observation; invalid or stale input changes nothing."""
		try:
			prepared = build_presentation_projection(observation)
		except PresentationProjectionError as exc:
			self.last_replacement_error = exc
			return False
		previous = self.projection
		if self._transition_invalidated:
			self.last_replacement_error = RuntimeError("Ferrum presentation transition is invalidated")
			self._discard_prepared(prepared)
			return False
		if previous is not None and prepared.revision <= previous.revision:
			self.last_replacement_error = PresentationProjectionError("presentation observation is stale")
			self._discard_prepared(prepared)
			return False
		if previous is not None and not self._preflight_previous_retirement(previous):
			self._discard_prepared(prepared)
			return False
		if not self._replace_scene_items(previous, prepared):
			return False
		self.projection = prepared
		if previous is not None:
			coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
			coordinator.retire_detached_projection_items(
				list(previous.roots), callbacks_already_disposed=True,
			)
			if coordinator.report.callback_errors:
				self.last_replacement_error = RuntimeError("Ferrum presentation retirement was retained")
				return True
		self.last_replacement_error = None
		return True

	#============================================
	def _replace_scene_items(self, previous: FerrumPresentationProjection | None,
			prepared: FerrumPresentationProjection) -> bool:
		"""Move scene ownership reversibly before publishing either projection.

		Previous roots remain ordinary live items until every candidate root can join
		the scene. A failed removal or attachment restores the exact old projection,
		then terminally retires only the never-published candidate.
		"""
		try:
			if previous is not None:
				for root in previous.roots:
					if not ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(
							self._scene, root,
						):
						raise RuntimeError("Cannot detach prior Ferrum presentation projection")
			for root in prepared.roots:
				if not ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(
						self._scene, root,
					):
					raise RuntimeError("Cannot attach prepared Ferrum presentation projection")
		except Exception as exc:
			if not self._rollback_scene_transition(previous, prepared):
				self._transition_invalidated = True
				self.retained_prior_projection = previous
				self.retained_candidate_projection = prepared
				self.projection = None
				self.last_replacement_error = exc
				return False
			self.last_replacement_error = exc
			self._discard_prepared(prepared)
			return False
		return True

	#============================================
	def _preflight_previous_retirement(self, previous: FerrumPresentationProjection) -> bool:
		"""Reject a callback failure before its current scene ownership can change."""
		try:
			for item in previous.items:
				item.dispose()
		except Exception as exc:
			self.last_replacement_error = exc
			return False
		return True

	#============================================
	def _restore_previous(self, projection: FerrumPresentationProjection) -> None:
		"""Restore a failed transition before the old projection is ever replaced."""
		for root in projection.roots:
			if ferrum_qt.canvas.graphics_retirement.native_scene_for_item(root) is self._scene:
				continue
			if not ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene(
					self._scene, root,
				):
				raise RuntimeError("Cannot restore prior Ferrum presentation projection")

	#============================================
	def _remove_prepared_root(self, projection: FerrumPresentationProjection) -> None:
		"""Detach a candidate that a Ferrum handoff attached before reporting failure."""
		for root in projection.roots:
			if ferrum_qt.canvas.graphics_retirement.native_scene_for_item(root) is not self._scene:
				continue
			if not ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene(
					self._scene, root,
				):
				raise RuntimeError("Cannot detach failed Ferrum presentation candidate")

	#============================================
	def _rollback_scene_transition(self, previous: FerrumPresentationProjection | None,
			prepared: FerrumPresentationProjection) -> bool:
		"""Restore ownership or explicitly invalidate this controller on a second failure."""
		try:
			self._remove_prepared_root(prepared)
			if previous is not None:
				self._restore_previous(previous)
		except Exception as exc:
			self.retained_transition_errors.append(exc)
			return False
		return True

	#============================================
	def _discard_prepared(self, projection: FerrumPresentationProjection) -> None:
		"""Retire a never-published candidate and retain a visible failure record."""
		try:
			projection.dispose_detached()
		except Exception as exc:
			self.retained_disposal_errors.append(exc)
