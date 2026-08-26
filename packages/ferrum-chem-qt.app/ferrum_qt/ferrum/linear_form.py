"""Ordinary Ferrum Qt action for Rust-owned linear-form conversion."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeLinearFormCapture:
	"""One authenticated direct-root selection ready for synchronous submission."""

	tab: object
	revision: int
	digest: str
	molecule_id: str
	atom_ids: tuple[str, ...]


#============================================
def capture_linear_form_selection(tab: object) -> FerrumNativeLinearFormCapture | None:
	"""Expand one Rust-resolved atom/bond selection into ordered durable atoms."""
	if getattr(tab, "requires_refresh", True):
		return None
	targets = tab.selected_structure_targets()
	if type(targets) is not tuple or not targets:
		return None
	import ferrum_qt.ferrum.engine as engine
	observation = tab.current_document_observation()
	selected_molecule_id = None
	selected_atom_ids = set()
	for target in targets:
		if (
			target.kind not in (
				engine.StructureTargetKindV1.atom,
				engine.StructureTargetKindV1.bond,
			)
			or type(target.object_id) is not str
			or not target.object_id
			or type(target.molecule_object_id) is not str
			or not target.molecule_object_id
		):
			return None
		if selected_molecule_id is None:
			selected_molecule_id = target.molecule_object_id
		elif selected_molecule_id != target.molecule_object_id:
			return None
	if selected_molecule_id is None:
		return None
	molecule_matches = [
		molecule for molecule in observation.projection.molecules
		if molecule.document_object_id == selected_molecule_id
	]
	if len(molecule_matches) != 1:
		return None
	molecule = molecule_matches[0]
	atom_ids_by_document_object = set()
	for atom in molecule.atoms:
		atom_id = atom.document_object_id
		if (
			type(atom_id) is not str
			or not atom_id
			or atom_id in atom_ids_by_document_object
		):
			return None
		atom_ids_by_document_object.add(atom_id)
	bonds_by_document_object = {}
	for bond in molecule.bonds:
		bond_id = bond.document_object_id
		if (
			type(bond_id) is not str
			or not bond_id
			or bond_id in bonds_by_document_object
		):
			return None
		bonds_by_document_object[bond_id] = bond
	for target in targets:
		if target.kind is engine.StructureTargetKindV1.atom:
			if target.object_id not in atom_ids_by_document_object:
				return None
			selected_atom_ids.add(target.object_id)
			continue
		bond = bonds_by_document_object.get(target.object_id)
		if bond is None:
			return None
		endpoints = (bond.start, bond.end)
		if any(
			endpoint.kind != "atom"
			or type(endpoint.document_object_id) is not str
			or not endpoint.document_object_id
			or endpoint.document_object_id not in atom_ids_by_document_object
			for endpoint in endpoints
		):
			return None
		selected_atom_ids.update(endpoint.document_object_id for endpoint in endpoints)
	ordered_atoms = tuple(
		atom.document_object_id for atom in molecule.atoms
		if atom.document_object_id in selected_atom_ids
	)
	if (
		not ordered_atoms
		or len(ordered_atoms) != len(selected_atom_ids)
		or len(frozenset(ordered_atoms)) != len(ordered_atoms)
	):
		return None
	snapshot = observation.snapshot
	return FerrumNativeLinearFormCapture(
		tab, snapshot.revision, snapshot.digest, selected_molecule_id, ordered_atoms,
	)


#============================================
class FerrumNativeLinearFormTabMixin:
	"""Commit one already-resolved atom selection through the Rust session."""

	#============================================
	def convert_durable_linear_form_v1(self, expected_revision: int,
			expected_digest: str, molecule_id: str,
			atom_ids: tuple[str, ...]) -> object:
		"""Submit exact linear-form intent and install only a changed observation."""
		self._require_mutable()
		if (
			type(expected_revision) is not int
			or type(expected_digest) is not str
			or type(molecule_id) is not str
			or type(atom_ids) is not tuple
		):
			raise TypeError("Ferrum linear-form conversion requires exact frozen inputs")
		if (
			not atom_ids
			or any(type(atom_id) is not str or not atom_id for atom_id in atom_ids)
			or len(frozenset(atom_ids)) != len(atom_ids)
		):
			raise ValueError(
				"Ferrum linear-form conversion requires distinct durable atom IDs",
			)
		snapshot = self.current_snapshot
		if snapshot.revision != expected_revision or snapshot.digest != expected_digest:
			raise RuntimeError("document changed before linear-form conversion")
		result = self._session.convert_linear_form_v1(
			expected_revision, expected_digest, molecule_id, atom_ids,
		)
		authoritative = result.observation.snapshot
		if (
			authoritative.revision == snapshot.revision
			and authoritative.digest == snapshot.digest
		):
			return result
		self._install_mutation_result(result, atom_ids)
		return result


#============================================
class FerrumNativeLinearFormWindowMixin:
	"""Own the synchronous ordinary-native linear-form action."""

	#============================================
	def _build_linear_form_action(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the Rust-owned selection conversion to the Chemistry menu."""
		menu.addSeparator()
		self._convert_linear_form_action = PySide6.QtGui.QAction(
			self.tr("Convert selection to linear form"), self,
		)
		self._convert_linear_form_action.setToolTip(self.tr(
			"Lay out one selected atom path and record its linear form through Rust",
		))
		self._convert_linear_form_action.triggered.connect(self._on_convert_linear_form)
		menu.addAction(self._convert_linear_form_action)

	#============================================
	def _refresh_linear_form_action(self, active: bool, pending: bool,
			busy: bool) -> None:
		"""Enable conversion only for one current durable direct-root selection."""
		self._convert_linear_form_action.setEnabled(
			active and not pending and not busy and self._linear_form_capture() is not None,
		)

	#============================================
	def _on_convert_linear_form(self, _checked: bool = False) -> bool:
		"""Submit one current selection through a direct Rust operation."""
		capture = self._linear_form_capture()
		if capture is None:
			return False
		if not self._linear_form_capture_is_current(capture):
			self._show_edit_refusal(self._unavailable_edit_refusal("The active atom selection changed before conversion."))
			self._refresh_actions()
			return False
		try:
			result = capture.tab.convert_durable_linear_form_v1(
				capture.revision,
				capture.digest,
				capture.molecule_id,
				capture.atom_ids,
			)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return False
		authoritative = result.observation.snapshot
		message = (
			"Converted the selected atom path to a Rust-owned linear form."
			if authoritative.revision != capture.revision
			else "The selected atom path already has its canonical linear form."
		)
		self.statusBar().showMessage(self.tr(message), 5000)
		self._refresh_actions()
		return True

	#============================================
	def _linear_form_capture(self) -> FerrumNativeLinearFormCapture | None:
		"""Freeze the active tab's exact revision, root, and expanded atom selection."""
		tab = self._active_native_tab()
		if tab is None:
			return None
		try:
			return capture_linear_form_selection(tab)
		except (RuntimeError, TypeError, ValueError):
			return None

	#============================================
	def _linear_form_capture_is_current(
			self, capture: FerrumNativeLinearFormCapture) -> bool:
		"""Reauthenticate active tab, snapshot, root, and expanded selection."""
		tab = capture.tab
		if (
			tab not in self._native_tabs_by_page
			or self._active_native_tab() is not tab
			or tab.requires_refresh
		):
			return False
		snapshot = tab.current_snapshot
		return (
			snapshot.revision == capture.revision
			and snapshot.digest == capture.digest
			and self._linear_form_capture() == capture
		)
