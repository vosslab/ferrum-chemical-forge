"""Rust-owned direct normal-bond gesture methods for one Ferrum document tab."""


#============================================
class FerrumNativeDirectBondGestureTabMixin:
	"""Keep opaque direct-bond handles inside the tab's Rust session boundary."""

	#============================================
	def begin_direct_bond_gesture(
			self, start_atom_id: str, presentation: object, snap_enabled: bool,
			) -> object:
		"""Begin one revision-fenced Rust normal-bond gesture from an installed atom."""
		self._require_mutable()
		if type(start_atom_id) is not str or not start_atom_id:
			raise TypeError("Ferrum direct bond start requires a durable atom identifier")
		if type(snap_enabled) is not bool:
			raise TypeError("Ferrum direct bond snap setting must be a boolean")
		import ferrum_qt.ferrum.engine as engine
		snapshot = self.current_snapshot
		snap = engine.DirectBondSnapPolicyV1(hex_grid=snap_enabled)
		return self._session.begin_direct_bond_gesture_v1(
			snapshot.revision, snapshot.digest,
			self._direct_bond_object_id(start_atom_id), presentation, "C", snap,
		)

	#============================================
	def preview_direct_bond_gesture(self, gesture: object, endpoint: object) -> object:
		"""Return Rust's immutable direct-bond preview or ordinary refusal."""
		self._require_mutable()
		return self._session.preview_direct_bond_gesture_v1(gesture, endpoint)

	#============================================
	def direct_bond_existing_endpoint(self, atom_id: str) -> object:
		"""Translate one Qt durable source identifier at the private Rust boundary."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		return engine.DirectBondEndIntentV1.existing_atom(
			self._direct_bond_object_id(atom_id),
		)

	#============================================
	def direct_bond_new_endpoint(self, x: float, y: float) -> object:
		"""Create one frozen raw-pointer endpoint input without Qt-side snapping."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		return engine.DirectBondEndIntentV1.new_atom_at(x, y)

	#============================================
	def commit_direct_bond_gesture(self, gesture: object, preview: object) -> object:
		"""Commit one opaque checked gesture and install Rust's resulting observation."""
		self._require_mutable()
		commit = self._session.commit_direct_bond_gesture_v1(gesture, preview)
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
