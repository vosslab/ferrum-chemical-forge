"""Selection-backed Ferrum document-tab actions."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
FerrumNativeDocumentTabError = native_document_tab_errors.FerrumNativeDocumentTabError


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSelectedMoleculeAtomAddress:
	"""Durable selected-atom address and installed document fence for live chemistry."""

	revision: int
	digest: str
	molecule_id: str
	atom_id: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSelectedMoleculeBondAddress:
	"""Durable selected-bond address and installed document fence for live chemistry."""

	revision: int
	digest: str
	molecule_id: str
	bond_id: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSelectedMoleculeCompactGroupAddress:
	"""Public selected compact-group address and installed document fence."""

	revision: int
	digest: str
	molecule_id: str
	compact_group_id: str


#============================================
class FerrumNativeDocumentSelectionMixin:
	"""Selection-backed document actions owned by the host tab session."""


	#============================================
	def selected_atom_projection(self) -> object:
		"""Return one selected frozen Rust atom projection for a Ferrum dialog."""
		self._require_mutable()
		selected = self.selected_molecule_atom_address().atom_id
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			for atom in molecule.atoms:
				if atom.document_object_id == selected:
					return atom
		raise FerrumNativeDocumentTabError("selected atom is absent from the Rust projection")

	#============================================
	def apply_selected_atom_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust atom-properties patch for one selected atom."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("Ferrum atom properties require an exact change tuple")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentAtomPropertyChangeV1 for change in changes):
			raise TypeError("Ferrum atom properties require exact frozen Ferrum changes")
		address = self.selected_molecule_atom_address()
		result = self._live_document_session_v1.set_atom_properties_v1(
			address.revision,
			address.digest,
			address.molecule_id,
			address.atom_id,
			changes,
		)
		self._install_mutation_result(result, (address.atom_id,))
		return result

	#============================================
	def set_selected_atom_number(self, number: int, show_number: bool) -> object:
		"""Assign one selected atom number through the closed Rust operation."""
		self._require_mutable()
		if type(number) is not int or number <= 0 or type(show_number) is not bool:
			raise TypeError("Ferrum atom number requires a positive int and exact bool")
		address = self.selected_molecule_atom_address()
		result = self._live_document_session_v1.set_atom_number_v1(
			address.revision,
			address.digest,
			address.molecule_id,
			address.atom_id,
			number,
			show_number,
		)
		self._install_mutation_result(result, (address.atom_id,))
		return result

	#============================================
	def clear_selected_atom_number(self) -> object:
		"""Clear one selected atom number through the closed Rust operation."""
		self._require_mutable()
		address = self.selected_molecule_atom_address()
		result = self._live_document_session_v1.clear_atom_number_v1(
			address.revision,
			address.digest,
			address.molecule_id,
			address.atom_id,
		)
		self._install_mutation_result(result, (address.atom_id,))
		return result

	#============================================
	def apply_selected_atom_mark(self, action: object, kind: object,
			matching_mark_index: int | None) -> object:
		"""Apply exact frozen mark intent to one selected durable Rust atom."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(action) is not engine.AtomMarkActionV1:
			raise TypeError("Ferrum atom mark action requires an exact Ferrum value")
		if type(kind) is not engine.AtomMarkKindV1:
			raise TypeError("Ferrum atom mark kind requires an exact Ferrum value")
		if matching_mark_index is not None and type(matching_mark_index) is not int:
			raise TypeError("Ferrum atom mark selector requires an exact int or None")
		address = self.selected_molecule_atom_address()
		result = self._live_document_session_v1.apply_atom_mark_v1(
			address.revision,
			address.digest,
			address.molecule_id,
			address.atom_id,
			action,
			kind,
			matching_mark_index,
		)
		self._install_mutation_result(result, (address.atom_id,))
		return result

	#============================================
	def toggle_selected_atom_mark(self, kind: object) -> object:
		"""Add a missing mark or remove the first matching mark in source order."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(kind) is not engine.AtomMarkKindV1:
			raise TypeError("Ferrum atom mark kind requires an exact Ferrum value")
		atom = self.selected_atom_projection()
		matching = next((mark for mark in atom.marks if mark.kind == kind), None)
		if matching is None:
			action = engine.AtomMarkActionV1.add
			ordinal = None
		else:
			action = engine.AtomMarkActionV1.remove
			ordinal = matching.same_type_ordinal
		return self.apply_selected_atom_mark(action, kind, ordinal)

	#============================================
	def selected_atom_marks(self) -> tuple[object, ...]:
		"""Return exact current frozen marks for the single selected durable atom."""
		return tuple(self.selected_atom_projection().marks)

	#============================================
	def selected_atom_has_marks(self) -> bool:
		"""Return whether one selected atom currently owns a supported mark."""
		return self.has_one_selected_atom() and bool(self.selected_atom_marks())

	#============================================
	def selected_bond_projection(self) -> object:
		"""Return one selected frozen Rust bond projection for a Ferrum dialog."""
		self._require_mutable()
		selected = self.selected_molecule_bond_address().bond_id
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			for bond in molecule.bonds:
				if bond.document_object_id == selected:
					return bond
		raise FerrumNativeDocumentTabError("selected bond is absent from the Rust projection")

	#============================================
	def apply_selected_bond_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust bond-properties patch for one selected bond."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("Ferrum bond properties require an exact change tuple")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentBondPropertyChangeV1 for change in changes):
			raise TypeError("Ferrum bond properties require exact frozen Ferrum changes")
		address = self.selected_molecule_bond_address()
		result = self._live_document_session_v1.set_bond_properties_v1(
			address.revision,
			address.digest,
			address.molecule_id,
			address.bond_id,
			changes,
		)
		self._install_mutation_result(result, (address.bond_id,))
		return result

	#============================================
	def has_one_selected_atom(self) -> bool:
		"""Return whether Rust resolves the one selected canvas key as an atom."""
		return self._has_one_selected_structure_target("atom")

	#============================================
	def selected_atom_has_number(self) -> bool:
		"""Return whether the current single selected atom has a valid number fact."""
		if not self.has_one_selected_atom():
			return False
		return self.selected_atom_projection().number is not None

	#============================================
	def has_one_selected_bond(self) -> bool:
		"""Return whether Rust resolves the one selected canvas key as a bond."""
		return self._has_one_selected_structure_target("bond")

	#============================================
	def has_one_selected_plus(self) -> bool:
		"""Return whether the current selection names one rendered Plus."""
		if self._disposed or self.requires_refresh:
			return False
		import ferrum_qt.ferrum.engine as engine
		try:
			self._selected_presentation_identifier(
				engine.DocumentPresentationRootKindV1.plus,
			)
		except FerrumNativeDocumentTabError:
			return False
		return True

	#============================================
	def has_one_selected_arrow(self) -> bool:
		"""Return whether the current selection names one rendered Arrow."""
		if self._disposed or self.requires_refresh:
			return False
		import ferrum_qt.ferrum.engine as engine
		try:
			self._selected_presentation_identifier(
				engine.DocumentPresentationRootKindV1.arrow,
			)
		except FerrumNativeDocumentTabError:
			return False
		return True

	#============================================
	def selected_plus_projection(self) -> object:
		"""Return one selected frozen Rust Plus projection for a Ferrum dialog."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		selected = self._selected_presentation_identifier(
			engine.DocumentPresentationRootKindV1.plus,
		)
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		matches = tuple(
			root.plus for root in self._document_observation.projection.presentation_stack.entries
			if (
				root.kind == "plus"
				and root.plus is not None
				and root.plus.target.record_kind == "plus"
				and root.plus.target.document_object_id == selected
			)
		)
		if len(matches) == 1:
			return matches[0]
		raise FerrumNativeDocumentTabError("selected Plus is absent from the Rust projection")

	#============================================
	def apply_selected_plus_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust Plus patch while retaining durable selection."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("Ferrum Plus properties require an exact change tuple")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentPlusPropertyChangeV1
				for change in changes):
			raise TypeError("Ferrum Plus properties require exact frozen Ferrum changes")
		plus_id = self._selected_presentation_identifier(
			engine.DocumentPresentationRootKindV1.plus,
		)
		snapshot = self.current_snapshot
		result = self._live_document_session_v1.set_plus_properties_v1(
			snapshot.revision, snapshot.digest, plus_id, changes,
		)
		self._install_mutation_result(result, (plus_id,))
		return result

	#============================================
	def selected_arrow_projection(self) -> object:
		"""Return one selected frozen Rust Arrow projection for a Ferrum dialog."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		selected = self._selected_presentation_identifier(
			engine.DocumentPresentationRootKindV1.arrow,
		)
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		matches = tuple(
			root.arrow for root in self._document_observation.projection.presentation_stack.entries
			if (
				root.kind == "arrow"
				and root.arrow is not None
				and root.arrow.target.record_kind == "arrow"
				and root.arrow.target.document_object_id == selected
			)
		)
		if len(matches) == 1:
			return matches[0]
		raise FerrumNativeDocumentTabError("selected Arrow is absent from the Rust projection")

	#============================================
	def apply_selected_arrow_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust Arrow patch while retaining durable selection."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("Ferrum Arrow properties require an exact change tuple")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentArrowPropertyChangeV1
				for change in changes):
			raise TypeError("Ferrum Arrow properties require exact frozen Ferrum changes")
		arrow_id = self._selected_presentation_identifier(
			engine.DocumentPresentationRootKindV1.arrow,
		)
		snapshot = self.current_snapshot
		result = self._live_document_session_v1.set_arrow_properties_v1(
			snapshot.revision, snapshot.digest, arrow_id, changes,
		)
		self._install_mutation_result(result, (arrow_id,))
		return result

	#============================================
	def delete_selected_atom(self) -> object:
		"""Delete one selected durable atom and its incident bonds through Rust."""
		self._require_mutable()
		address = self.selected_molecule_atom_address()
		result = self._live_document_session_v1.delete_atom_v1(
			address.revision, address.digest, address.molecule_id, address.atom_id,
		)
		self._install_mutation_result(result)
		return result

	#============================================
	def delete_selected_bond(self) -> object:
		"""Delete one selected durable typed bond through Rust."""
		self._require_mutable()
		selected = self.selected_molecule_bond_address().bond_id
		import ferrum_qt.ferrum.engine as engine
		operation = engine.DocumentOperationV1.delete_bond(selected)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result)
		return result

	#============================================
	def set_selected_bond_order(self, order: object) -> object:
		"""Replace one selected bond order through the closed Rust operation."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(order) is not engine.DocumentBondOrderV1:
			raise TypeError("Ferrum bond order requires an exact Ferrum order value")
		selected = self.selected_molecule_bond_address().bond_id
		operation = engine.DocumentOperationV1.set_bond_order(selected, order)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (selected,))
		return result

	#============================================
	def _apply_current_selection_operation_v1(self, operation: object) -> object:
		"""Apply one verified closed operation against the installed selection fence."""
		import ferrum_qt.ferrum.engine as engine
		if type(operation) is not engine.DocumentOperationV1:
			raise TypeError("Ferrum selection mutation requires an exact closed operation")
		return self._apply_current_document_operation_v1(operation)

	#============================================
	def selected_molecule_atom_address(self) -> FerrumSelectedMoleculeAtomAddress:
		"""Resolve one generic canvas key as an atom through Rust's current observation."""
		molecule_id, atom_id, revision, digest = self._selected_structure_address("atom")
		return FerrumSelectedMoleculeAtomAddress(revision, digest, molecule_id, atom_id)

	#============================================
	def molecule_atom_address(self, atom_id: str) -> FerrumSelectedMoleculeAtomAddress:
		"""Resolve one installed durable atom to its durable molecule owner and fence."""
		self._require_mutable()
		if type(atom_id) is not str or not atom_id:
			raise TypeError("Ferrum atom address requires a durable atom identifier")
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			if any(atom.document_object_id == atom_id for atom in molecule.atoms):
				snapshot = self.current_snapshot
				return FerrumSelectedMoleculeAtomAddress(
					snapshot.revision, snapshot.digest, molecule.document_object_id, atom_id,
				)
		raise FerrumNativeDocumentTabError("atom is absent from the Rust document projection")

	#============================================
	def selected_molecule_bond_address(self) -> FerrumSelectedMoleculeBondAddress:
		"""Resolve one generic canvas key as a bond through Rust's current observation."""
		molecule_id, bond_id, revision, digest = self._selected_structure_address("bond")
		return FerrumSelectedMoleculeBondAddress(revision, digest, molecule_id, bond_id)

	#============================================
	def selected_molecule_compact_group_address(self) -> FerrumSelectedMoleculeCompactGroupAddress:
		"""Resolve one generic canvas key as a compact group through Rust's observation."""
		molecule_id, compact_group_id, revision, digest = self._selected_structure_address(
			"compact_group",
		)
		return FerrumSelectedMoleculeCompactGroupAddress(
			revision, digest, molecule_id, compact_group_id,
		)

	#============================================
	def _has_one_selected_structure_target(self, kind: str) -> bool:
		"""Return whether the generic canvas selection resolves to one requested Rust kind."""
		if self._disposed or self.requires_refresh:
			return False
		try:
			self._selected_structure_address(kind)
		except FerrumNativeDocumentTabError:
			return False
		return True

	#============================================
	def _selected_presentation_identifier(self, expected_kind: object) -> str:
		"""Resolve one generic canvas target as one exact Rust presentation root."""
		selectors = self._selected_presentation_root_selectors()
		if len(selectors) != 1 or selectors[0][1] != expected_kind:
			raise FerrumNativeDocumentTabError(
				"select exactly one current presentation root of the required kind first",
			)
		return selectors[0][0]

	#============================================
	def _selected_structure_address(self, kind: str) -> tuple[str, str, int, str]:
		"""Resolve one generic selected canvas key through the fenced Rust structural view."""
		import ferrum_qt.ferrum.engine as engine
		kind_by_name = {
			"atom": engine.StructureTargetKindV1.atom,
			"bond": engine.StructureTargetKindV1.bond,
			"compact_group": engine.StructureTargetKindV1.compact_group,
		}
		expected_kind = kind_by_name.get(kind)
		if expected_kind is None:
			raise ValueError("Ferrum structural address kind is unsupported")
		targets = self.selected_structure_targets()
		if len(targets) != 1:
			raise FerrumNativeDocumentTabError(
				f"select exactly one current {kind.replace('_', ' ')} for this operation",
			)
		target = targets[0]
		if target.kind != expected_kind:
			raise FerrumNativeDocumentTabError(
				f"selected canvas target is not a current {kind.replace('_', ' ')}",
			)
		snapshot = self.current_snapshot
		return target.molecule_object_id, target.object_id, snapshot.revision, snapshot.digest

	#============================================
	def selected_structure_targets(self) -> tuple[object, ...]:
		"""Resolve every current generic canvas selection through Rust in Rust order.

		Canvas targets contribute durable identities only.  Rust authenticates their
		current fence, membership, molecule ownership, and structural kind.
		"""
		from ferrum_qt.canvas.ferrum_render_target import RenderTargetKey
		selected = self._require_projection().selected_targets()
		if type(selected) is not tuple:
			raise FerrumNativeDocumentTabError(
				"selected canvas targets are not an exact current Ferrum target tuple",
			)
		if not selected:
			return ()
		selected_ids: set[str] = set()
		for canvas_target in selected:
			if type(canvas_target) is not RenderTargetKey:
				raise FerrumNativeDocumentTabError(
					"selected canvas target is not an exact current Ferrum render target",
				)
			if canvas_target.kind != "document_object":
				raise FerrumNativeDocumentTabError(
					"selected canvas target does not name a durable document object",
				)
			document_object_id = canvas_target.document_object_id
			if type(document_object_id) is not str or not document_object_id:
				raise FerrumNativeDocumentTabError(
					"selected canvas target lacks a durable document-object identity",
				)
			if document_object_id in selected_ids:
				raise FerrumNativeDocumentTabError(
					"selected canvas targets contain a duplicate durable document-object identity",
				)
			selected_ids.add(document_object_id)
		return self.structure_targets_for_ids(tuple(selected_ids))

	#============================================
	def structure_targets_for_ids(self, object_ids: tuple[str, ...]) -> tuple[object, ...]:
		"""Resolve an exact nonempty durable-ID subset through Rust in Rust order.

		The returned exact frozen ``StructureInteractionTargetV1`` values are the
		only structural classification exposed to document-tab tools.  Rust
		authenticates the installed fence, membership, molecule ownership, and
		atom/bond/compact-group kind for every requested opaque ID.
		"""
		self._require_mutable()
		return self._resolve_structure_targets_for_ids(object_ids)

	#============================================
	def _resolve_structure_targets_for_ids(
			self, object_ids: tuple[str, ...]) -> tuple[object, ...]:
		"""Resolve IDs against the installed fence during an owned transition."""
		self._require_live()
		import ferrum_qt.ferrum.engine as engine
		if type(object_ids) is not tuple or not object_ids:
			raise TypeError("Ferrum structure target resolution requires a nonempty exact ID tuple")
		requested_ids: set[str] = set()
		for object_id in object_ids:
			if type(object_id) is not str or not object_id:
				raise TypeError("Ferrum structure target resolution requires durable object IDs")
			if object_id in requested_ids:
				raise FerrumNativeDocumentTabError(
					"Ferrum structure target resolution received a duplicate durable object ID",
				)
			requested_ids.add(object_id)
		snapshot = self.current_snapshot
		observation = self._session.observe_structure_interaction_v1(
			snapshot.revision, snapshot.digest,
		)
		if type(observation) is not engine.StructureInteractionObservationV1:
			raise FerrumNativeDocumentTabError(
				"Rust structure interaction returned an invalid observation DTO",
			)
		if (
			type(observation.revision) is not int
			or type(observation.digest) is not str
			or not observation.digest
			or observation.revision != snapshot.revision
			or observation.digest != snapshot.digest
		):
			raise FerrumNativeDocumentTabError(
				"Rust structure observation does not match the installed document fence",
			)
		targets = observation.targets
		if type(targets) is not tuple:
			raise FerrumNativeDocumentTabError(
				"Rust structure interaction returned a non-frozen target collection",
			)
		valid_kinds = frozenset((
			engine.StructureTargetKindV1.atom,
			engine.StructureTargetKindV1.bond,
			engine.StructureTargetKindV1.compact_group,
		))
		observed_ids: set[str] = set()
		resolved: list[object] = []
		for target in targets:
			if type(target) is not engine.StructureInteractionTargetV1:
				raise FerrumNativeDocumentTabError(
					"Rust structure interaction returned an invalid target DTO",
				)
			if (
				type(target.molecule_object_id) is not str
				or not target.molecule_object_id
				or type(target.object_id) is not str
				or not target.object_id
				or type(target.kind) is not engine.StructureTargetKindV1
				or target.kind not in valid_kinds
			):
				raise FerrumNativeDocumentTabError(
					"Rust structure observation returned an invalid durable target address",
				)
			if target.object_id in observed_ids:
				raise FerrumNativeDocumentTabError(
					"Rust structure observation contains an ambiguous durable target identity",
				)
			observed_ids.add(target.object_id)
			if target.object_id in requested_ids:
				resolved.append(target)
		if len(resolved) != len(requested_ids):
			raise FerrumNativeDocumentTabError(
				"requested durable object is absent from Rust structure observation",
			)
		return tuple(resolved)
