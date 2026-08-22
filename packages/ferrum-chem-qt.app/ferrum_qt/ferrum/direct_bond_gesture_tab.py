"""Rust-owned direct normal-bond gesture methods for one Ferrum document tab."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore


_DIRECT_BOND_IMPLICIT_ATOM_PICK_RADIUS_PX = 6


class DirectBondEndpointAmbiguity(ValueError):
	"""The pointer is not a verified existing atom or verified empty space."""


@dataclasses.dataclass(frozen=True)
class DirectBondEndpoint:
	"""One tab-resolved opaque Rust endpoint with an optional visible source ID."""
	endpoint: object
	source_id: str | None


#============================================
class FerrumNativeDirectBondGestureTabMixin:
	"""Keep opaque direct-bond handles inside the tab's Rust session boundary."""
	#============================================
	def direct_bond_endpoint_at_keyboard_scene_position(
			self, point: PySide6.QtCore.QPointF,
			) -> DirectBondEndpoint:
		"""Resolve a keyboard cursor to one opaque Rust endpoint or ambiguity."""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPointF):
			raise TypeError("Ferrum keyboard direct-bond endpoint requires a QPointF")
		return self.direct_bond_endpoint_at_viewport_point(self.view.mapFromScene(point))

	#============================================
	def direct_bond_endpoint_at_viewport_point(
			self, point: PySide6.QtCore.QPoint,
			) -> DirectBondEndpoint:
		"""Resolve one pointer fact without turning ambiguous space into carbon.

		Rendered atom hits retain precedence.  Without one, direct bonding may use
		the same six-pixel unique-nearest projection tolerance as its origin picker.
		An equidistant tie is deliberately terminal: only verified empty space can
		become ``NewAtomAt`` at the Rust boundary.
		"""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPoint):
			raise TypeError("Ferrum direct-bond endpoint hit testing requires a QPoint")
		rendered_atom_id = self.durable_atom_at_viewport_point(point)
		if rendered_atom_id is not None:
			return DirectBondEndpoint(
				self.direct_bond_existing_endpoint(rendered_atom_id), rendered_atom_id,
			)
		observation = self.current_document_observation()
		radius_squared = _DIRECT_BOND_IMPLICIT_ATOM_PICK_RADIUS_PX ** 2
		nearest_distance: int | None = None
		nearest_source_ids: list[str] = []
		invalid_nearby_identity = False
		for molecule in observation.projection.molecules:
			for atom in molecule.atoms:
				viewport_atom = self.view.mapFromScene(PySide6.QtCore.QPointF(
					atom.position.x, atom.position.y,
				))
				delta_x = viewport_atom.x() - point.x()
				delta_y = viewport_atom.y() - point.y()
				distance_squared = delta_x * delta_x + delta_y * delta_y
				if distance_squared > radius_squared:
					continue
				if type(atom.source_id) is not str or not atom.source_id:
					invalid_nearby_identity = True
					continue
				if nearest_distance is None or distance_squared < nearest_distance:
					nearest_distance = distance_squared
					nearest_source_ids = [atom.source_id]
				elif distance_squared == nearest_distance:
					nearest_source_ids.append(atom.source_id)
		if invalid_nearby_identity:
			raise DirectBondEndpointAmbiguity()
		if not nearest_source_ids:
			scene = self.view.mapToScene(point)
			return DirectBondEndpoint(
				self.direct_bond_new_endpoint(float(scene.x()), float(scene.y())), None,
			)
		if len(nearest_source_ids) == 1:
			source_id = nearest_source_ids[0]
			return DirectBondEndpoint(
				self.direct_bond_existing_endpoint(source_id), source_id,
			)
		raise DirectBondEndpointAmbiguity()

	#============================================
	def begin_direct_bond_gesture(
			self, start: object, presentation: object, snap_enabled: bool,
			) -> object:
		"""Begin one revision-fenced Rust normal-bond gesture from an installed atom."""
		self._require_mutable()
		if type(snap_enabled) is not bool:
			raise TypeError("Ferrum direct bond snap setting must be a boolean")
		import ferrum_qt.ferrum.engine as engine
		snapshot = self.current_snapshot
		snap = engine.DirectBondSnapPolicyV1(hex_grid=snap_enabled)
		return self._session.begin_direct_bond_gesture_v2(
			snapshot.revision, snapshot.digest,
			start, presentation, "C", snap,
		)

	#============================================
	def admit_direct_bond_candidate(self, gesture: object, endpoint: object) -> object:
		"""Admit one endpoint and return Rust's opaque commit receipt or refusal."""
		self._require_mutable()
		return self._session.admit_direct_bond_candidate_v2(gesture, endpoint)

	#============================================
	def direct_bond_existing_endpoint(self, atom_id: str) -> object:
		"""Translate one Qt durable source identifier at the private Rust boundary."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		return engine.DirectBondEndpointIntentV2.existing_atom(
			self._direct_bond_object_id(atom_id),
		)

	#============================================
	def direct_bond_new_endpoint(self, x: float, y: float) -> object:
		"""Create one frozen raw-pointer endpoint input without Qt-side snapping."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		return engine.DirectBondEndpointIntentV2.new_atom_at(x, y)

	#============================================
	def commit_direct_bond_admission(self, admission: object) -> object:
		"""Redeem one opaque admitted candidate and install its Rust result once."""
		self._require_mutable()
		commit = self._session.commit_direct_bond_admission_v2(admission)
		self._install_mutation_result(commit.result, (("bond", commit.bond_identifier),))
		return commit

	#============================================
	def _direct_bond_object_id(self, source_id: str) -> str:
		"""Return Rust's object ID for one installed canvas source identifier."""
		if type(source_id) is not str or not source_id:
			raise TypeError("Ferrum direct bond atom identifier must be a non-empty string")
		observation = self.current_document_observation()
		for molecule in observation.projection.molecules:
			for atom in molecule.atoms:
				if atom.source_id == source_id:
					return atom.id
		raise ValueError("Ferrum direct bond atom is not in the installed projection")
