"""Rust-owned V3 direct-bond pointer probes for one Ferrum document tab."""

# PIP3 modules
import PySide6.QtCore


#============================================
class FerrumNativeDirectBondGestureTabMixin:
	"""Keep opaque V3 direct-bond handles inside the tab's Rust session boundary."""
	#============================================
	def direct_bond_pointer_probe_at_keyboard_scene_position(
			self, point: PySide6.QtCore.QPointF,
			) -> object:
		"""Capture one keyboard cursor position as immutable native pointer evidence."""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPointF):
			raise TypeError("Ferrum keyboard direct-bond probe requires a QPointF")
		return self.direct_bond_pointer_probe_at_viewport_point(self.view.mapFromScene(point))

	#============================================
	def direct_bond_pointer_probe_at_viewport_point(
			self, point: PySide6.QtCore.QPoint,
			) -> object:
		"""Report exact scene-hit facts and frame data to Rust without resolving endpoints."""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPoint):
			raise TypeError("Ferrum direct-bond pointer probe requires a QPoint")
		import ferrum_qt.ferrum.engine as engine
		projection = self._require_projection()
		atom_ids: set[str] = set()
		for item in self.view.items(point):
			current = item
			while current is not None:
				target = projection.item_targets.get(current)
				if target is not None:
					if target.kind == "atom" and type(target.identifier) is str and target.identifier:
						atom_ids.add(target.identifier)
					break
				current = current.parentItem()
		if len(atom_ids) == 1:
			hit_state = engine.DirectBondPointerHitStateV3.unique_atom
			direct_atom_id = next(iter(atom_ids))
		elif atom_ids:
			hit_state = engine.DirectBondPointerHitStateV3.ambiguous_atom
			direct_atom_id = None
		else:
			hit_state = engine.DirectBondPointerHitStateV3.none
			direct_atom_id = None
		viewport_to_scene, invertible = self.view.viewportTransform().inverted()
		if not invertible:
			raise ValueError("Ferrum direct-bond viewport transform is not invertible")
		frame = engine.DirectBondViewportToSceneV3(
			viewport_to_scene.m11(), viewport_to_scene.m12(),
			viewport_to_scene.m21(), viewport_to_scene.m22(),
			viewport_to_scene.dx(), viewport_to_scene.dy(),
		)
		scene = self.view.mapToScene(point)
		return engine.DirectBondPointerProbeV3(
			float(scene.x()), float(scene.y()), frame, hit_state, direct_atom_id,
		)

	#============================================
	def begin_direct_bond_gesture(
			self, start_probe: object, presentation: object, snap_enabled: bool,
			) -> object:
		"""Begin one revision-fenced Rust direct-bond gesture from V3 pointer facts."""
		self._require_mutable()
		if type(snap_enabled) is not bool:
			raise TypeError("Ferrum direct bond snap setting must be a boolean")
		import ferrum_qt.ferrum.engine as engine
		snapshot = self.current_snapshot
		snap = engine.DirectBondSnapPolicyV1(hex_grid=snap_enabled)
		return self._session.begin_direct_bond_gesture_v3(
			snapshot.revision, snapshot.digest,
			start_probe, presentation, "C", snap,
		)

	#============================================
	def admit_direct_bond_candidate(self, gesture: object, end_probe: object) -> object:
		"""Admit one V3 pointer probe and return Rust's opaque commit receipt."""
		self._require_mutable()
		return self._session.admit_direct_bond_candidate_v3(gesture, end_probe)

	#============================================
	def commit_direct_bond_admission(self, admission: object) -> object:
		"""Redeem one opaque V3 admission and install its Rust result once."""
		self._require_mutable()
		commit = self._session.commit_direct_bond_admission_v3(admission)
		self._install_mutation_result(commit.result, (("bond", commit.bond_identifier),))
		return commit
