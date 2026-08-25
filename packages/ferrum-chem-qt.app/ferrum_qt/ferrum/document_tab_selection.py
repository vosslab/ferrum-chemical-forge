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
		selected = self._selected_atom_identifier()
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			for atom in molecule.atoms:
				if atom.id == selected:
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
		self._install_mutation_result(result, (("atom", address.atom_id),))
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
		self._install_mutation_result(result, (("atom", address.atom_id),))
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
		self._install_mutation_result(result, (("atom", address.atom_id),))
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
		self._install_mutation_result(result, (("atom", address.atom_id),))
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
		selected = self._selected_durable_identifiers(1, "bond")[0]
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			for bond in molecule.bonds:
				if bond.id == selected:
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
		self._install_mutation_result(result, (("bond", address.bond_id),))
		return result

	#============================================
	def has_one_selected_atom(self) -> bool:
		"""Return whether the current disposable selection names exactly one atom."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "atom"
			and selected[0].durable_object_id is not None
		)

	#============================================
	def selected_atom_has_number(self) -> bool:
		"""Return whether the current single selected atom has a valid number fact."""
		if not self.has_one_selected_atom():
			return False
		return self.selected_atom_projection().number is not None

	#============================================
	def has_one_selected_bond(self) -> bool:
		"""Return whether the current disposable selection names exactly one bond."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "bond"
			and selected[0].durable_object_id is not None
		)

	#============================================
	def has_one_selected_plus(self) -> bool:
		"""Return whether the current selection names one rendered Plus."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "plus"
			and selected[0].durable_object_id is not None
		)

	#============================================
	def has_one_selected_arrow(self) -> bool:
		"""Return whether the current selection names one rendered Arrow."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "arrow"
			and selected[0].durable_object_id is not None
		)

	#============================================
	def selected_plus_projection(self) -> object:
		"""Return one selected frozen Rust Plus projection for a Ferrum dialog."""
		self._require_mutable()
		selected = self._selected_durable_identifiers(1, "plus")[0]
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for root in self._document_observation.projection.presentation_stack.roots:
			if root.kind == "plus" and root.plus.target.id == selected:
				return root.plus
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
		plus_id = self._selected_durable_identifiers(1, "plus")[0]
		snapshot = self.current_snapshot
		result = self._live_document_session_v1.set_plus_properties_v1(
			snapshot.revision, snapshot.digest, plus_id, changes,
		)
		self._install_mutation_result(result, (("plus", plus_id),))
		return result

	#============================================
	def selected_arrow_projection(self) -> object:
		"""Return one selected frozen Rust Arrow projection for a Ferrum dialog."""
		self._require_mutable()
		selected = self._selected_durable_identifiers(1, "arrow")[0]
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for root in self._document_observation.projection.presentation_stack.roots:
			if root.kind == "arrow" and root.arrow.target.id == selected:
				return root.arrow
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
		arrow_id = self._selected_durable_identifiers(1, "arrow")[0]
		snapshot = self.current_snapshot
		result = self._live_document_session_v1.set_arrow_properties_v1(
			snapshot.revision, snapshot.digest, arrow_id, changes,
		)
		self._install_mutation_result(result, (("arrow", arrow_id),))
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
		selected = self._selected_durable_identifiers(1, "bond")[0]
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
		selected = self._selected_durable_identifiers(1, "bond")[0]
		operation = engine.DocumentOperationV1.set_bond_order(selected, order)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (("bond", selected),))
		return result

	#============================================
	def _apply_current_selection_operation_v1(self, operation: object) -> object:
		"""Apply one verified closed operation against the installed selection fence."""
		import ferrum_qt.ferrum.engine as engine
		if type(operation) is not engine.DocumentOperationV1:
			raise TypeError("Ferrum selection mutation requires an exact closed operation")
		return self._apply_current_document_operation_v1(operation)

	#============================================
	def _selected_render_identifiers(self, expected: int, kind: str) -> tuple[str, ...]:
		"""Return exact presentation identifiers without treating them as document IDs."""
		selected = self._require_projection().selected_targets()
		if len(selected) != expected or any(target.kind != kind for target in selected):
			raise FerrumNativeDocumentTabError(
				f"select exactly {expected} {kind}{'s' if expected != 1 else ''} first",
			)
		identifiers = tuple(target.render_identifier for target in selected)
		if any(identifier is None for identifier in identifiers):
			raise FerrumNativeDocumentTabError(
				f"selected {kind} lacks a render identifier",
			)
		return tuple(identifier for identifier in identifiers if identifier is not None)

	#============================================
	def selected_molecule_atom_address(self) -> FerrumSelectedMoleculeAtomAddress:
		"""Return one selected durable molecule/atom pair and installed Rust fence."""
		self._require_mutable()
		selected = self._require_projection().selected_durable_targets()
		if (
			len(selected) != 1
			or selected[0].kind != "atom"
			or type(selected[0].durable_object_id) is not str
			or not selected[0].durable_object_id
			or type(selected[0].durable_molecule_object_id) is not str
			or not selected[0].durable_molecule_object_id
		):
			raise FerrumNativeDocumentTabError(
				"select exactly one current durable atom for a chemistry operation",
			)
		return self.molecule_atom_address(selected[0].durable_object_id)

	#============================================
	def molecule_atom_address(self, atom_id: str) -> FerrumSelectedMoleculeAtomAddress:
		"""Resolve one installed durable atom to its durable molecule owner and fence."""
		self._require_mutable()
		if type(atom_id) is not str or not atom_id:
			raise TypeError("Ferrum atom address requires a durable atom identifier")
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			if molecule.id is None:
				continue
			if any(atom.id == atom_id for atom in molecule.atoms):
				snapshot = self.current_snapshot
				return FerrumSelectedMoleculeAtomAddress(
					snapshot.revision, snapshot.digest, molecule.id, atom_id,
				)
		raise FerrumNativeDocumentTabError("atom is absent from the Rust document projection")

	#============================================
	def selected_molecule_bond_address(self) -> FerrumSelectedMoleculeBondAddress:
		"""Return one selected durable molecule/bond pair and installed Rust fence."""
		self._require_mutable()
		selected = self._require_projection().selected_durable_targets()
		if (
			len(selected) != 1
			or selected[0].kind != "bond"
			or type(selected[0].durable_object_id) is not str
			or not selected[0].durable_object_id
			or type(selected[0].durable_molecule_object_id) is not str
			or not selected[0].durable_molecule_object_id
		):
			raise FerrumNativeDocumentTabError(
				"select exactly one current durable bond for a chemistry operation",
			)
		target = selected[0]
		snapshot = self.current_snapshot
		return FerrumSelectedMoleculeBondAddress(
			snapshot.revision,
			snapshot.digest,
			target.durable_molecule_object_id,
			target.durable_object_id,
		)

	#============================================
	def selected_molecule_compact_group_address(self) -> FerrumSelectedMoleculeCompactGroupAddress:
		"""Return one selected durable compact address and installed Rust fence."""
		self._require_mutable()
		selected = self._require_projection().selected_durable_targets()
		if (
			len(selected) != 1
			or selected[0].kind != "compact_group"
			or type(selected[0].durable_object_id) is not str
			or not selected[0].durable_object_id
			or type(selected[0].durable_molecule_object_id) is not str
			or not selected[0].durable_molecule_object_id
		):
			raise FerrumNativeDocumentTabError(
				"select exactly one current compact group for materialization",
			)
		target = selected[0]
		snapshot = self.current_snapshot
		return FerrumSelectedMoleculeCompactGroupAddress(
			snapshot.revision,
			snapshot.digest,
			target.durable_molecule_object_id,
			target.durable_object_id,
		)
