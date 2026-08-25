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
	"""Public selected-atom address and installed document fence for chemistry."""

	document: str
	revision: int
	digest: str
	molecule_id: str
	atom_id: str
	document_root_order: int


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSelectedMoleculeCompactGroupAddress:
	"""Public selected compact-group address and installed document fence."""

	document: str
	revision: int
	digest: str
	molecule_id: str
	compact_group_id: str
	document_root_order: int


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
				if atom.source_id == selected:
					return atom
		raise FerrumNativeDocumentTabError("selected atom is absent from the Rust projection")

	#============================================
	def apply_selected_atom_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust atom-properties patch for one selected atom."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("Ferrum atom properties require an exact change tuple")
		selected = self._selected_atom_identifier()
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentAtomPropertyChangeV1 for change in changes):
			raise TypeError("Ferrum atom properties require exact frozen Ferrum changes")
		operation = engine.DocumentOperationV1.set_atom_properties(selected, changes)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (("atom", selected),))
		return result

	#============================================
	def set_selected_atom_number(self, number: int, show_number: bool) -> object:
		"""Assign one selected atom number through the closed Rust operation."""
		self._require_mutable()
		if type(number) is not int or number <= 0 or type(show_number) is not bool:
			raise TypeError("Ferrum atom number requires a positive int and exact bool")
		molecule_id, atom_id = self._selected_atom_address()
		import ferrum_qt.ferrum.engine as engine
		operation = engine.DocumentOperationV1.set_atom_number(
			molecule_id, atom_id, number, show_number,
		)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (("atom", atom_id),))
		return result

	#============================================
	def clear_selected_atom_number(self) -> object:
		"""Clear one selected atom number through the closed Rust operation."""
		self._require_mutable()
		molecule_id, atom_id = self._selected_atom_address()
		import ferrum_qt.ferrum.engine as engine
		operation = engine.DocumentOperationV1.clear_atom_number(
			molecule_id, atom_id,
		)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (("atom", atom_id),))
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
		molecule_id, atom_id = self._selected_atom_address()
		operation = engine.DocumentOperationV1.apply_atom_mark(
			molecule_id, atom_id, action, kind, matching_mark_index,
		)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (("atom", atom_id),))
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
				if bond.source_id == selected:
					return bond
		raise FerrumNativeDocumentTabError("selected bond is absent from the Rust projection")

	#============================================
	def apply_selected_bond_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust bond-properties patch for one selected bond."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("Ferrum bond properties require an exact change tuple")
		selected = self._selected_durable_identifiers(1, "bond")[0]
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentBondPropertyChangeV1 for change in changes):
			raise TypeError("Ferrum bond properties require exact frozen Ferrum changes")
		operation = engine.DocumentOperationV1.set_bond_properties(selected, changes)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (("bond", selected),))
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
			and selected[0].identifier is not None
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
			and selected[0].identifier is not None
		)

	#============================================
	def has_one_selected_plus(self) -> bool:
		"""Return whether the current selection names one durable rendered Plus."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "plus"
			and selected[0].identifier is not None
		)

	#============================================
	def has_one_selected_arrow(self) -> bool:
		"""Return whether the current selection names one durable rendered Arrow."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "arrow"
			and selected[0].identifier is not None
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
				if root.plus.target.source_id is None:
					raise FerrumNativeDocumentTabError(
						"selected Plus has no durable authored source identifier",
					)
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
		plus = self.selected_plus_projection()
		operation = engine.DocumentOperationV1.set_plus_properties(
			plus.target.source_id, changes,
		)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (("plus", plus.target.id),))
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
				if root.arrow.target.source_id is None:
					raise FerrumNativeDocumentTabError(
						"selected Arrow has no durable authored source identifier",
					)
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
		arrow = self.selected_arrow_projection()
		operation = engine.DocumentOperationV1.set_arrow_properties(
			arrow.target.source_id, changes,
		)
		result = self._apply_current_selection_operation_v1(operation)
		self._install_mutation_result(result, (("arrow", arrow.target.id),))
		return result

	#============================================
	def delete_selected_atom(self) -> object:
		"""Delete one selected durable atom and its incident bonds through Rust."""
		self._require_mutable()
		selected = self._selected_atom_identifier()
		import ferrum_qt.ferrum.engine as engine
		operation = engine.DocumentOperationV1.delete_atom(selected)
		result = self._apply_current_selection_operation_v1(operation)
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
	def selected_molecule_atom_address(self) -> FerrumSelectedMoleculeAtomAddress:
		"""Resolve one selected atom through the current durable document projection."""
		self._require_mutable()
		selected = self._require_projection().selected_durable_targets()
		if (
			len(selected) != 1
			or selected[0].kind != "atom"
			or type(selected[0].identifier) is not str
			or not selected[0].identifier
			or type(selected[0].source_order) is not int
		):
			raise FerrumNativeDocumentTabError(
				"select exactly one current durable atom for a chemistry operation",
			)
		target = selected[0]
		observation = self.current_document_observation()
		matches = []
		for document_root_order, molecule in enumerate(observation.projection.molecules):
			for atom in molecule.atoms:
				if (
					atom.source_id == target.identifier
					and atom.source_order == target.source_order
				):
					matches.append((document_root_order, molecule, atom))
		if len(matches) != 1:
			raise FerrumNativeDocumentTabError(
				"selected atom does not map to one current durable document projection",
			)
		document_root_order, molecule, atom = matches[0]
		if (
			type(molecule.id) is not str
			or not molecule.id
			or type(atom.id) is not str
			or not atom.id
		):
			raise FerrumNativeDocumentTabError(
				"selected atom projection lacks durable document object identifiers",
			)
		snapshot = self.current_snapshot
		return FerrumSelectedMoleculeAtomAddress(
			snapshot.cdml,
			snapshot.revision,
			snapshot.digest,
			molecule.id,
			atom.id,
			document_root_order,
		)

	#============================================
	def selected_molecule_compact_group_address(self) -> FerrumSelectedMoleculeCompactGroupAddress:
		"""Resolve one selected compact group through the Rust projection."""
		self._require_mutable()
		selected = self._require_projection().selected_durable_targets()
		if (
			len(selected) != 1
			or selected[0].kind != "compact_group"
			or type(selected[0].identifier) is not str
			or not selected[0].identifier
			or type(selected[0].molecule_identifier) is not str
			or not selected[0].molecule_identifier
			or type(selected[0].source_order) is not int
		):
			raise FerrumNativeDocumentTabError(
				"select exactly one current compact group for materialization",
			)
		target = selected[0]
		observation = self.current_document_observation()
		matches = []
		for document_root_order, molecule in enumerate(observation.projection.molecules):
			for group in molecule.compact_groups:
				if (
					molecule.id == target.molecule_identifier
					and group.id == target.identifier
					and group.source_order == target.source_order
				):
					matches.append((document_root_order, molecule, group))
		if len(matches) != 1:
			raise FerrumNativeDocumentTabError(
				"selected compact group does not map to one current durable document projection",
			)
		document_root_order, molecule, group = matches[0]
		if (
			type(molecule.id) is not str
			or not molecule.id
			or type(group.id) is not str
			or not group.id
		):
			raise FerrumNativeDocumentTabError(
				"selected compact group projection lacks durable document object identifiers",
			)
		snapshot = self.current_snapshot
		return FerrumSelectedMoleculeCompactGroupAddress(
			snapshot.cdml,
			snapshot.revision,
			snapshot.digest,
			target.molecule_identifier,
			target.identifier,
			document_root_order,
		)
