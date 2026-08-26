"""Exact Rust-owned molecule-name editing for the ordinary Ferrum window."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.molecule_inspection as native_molecule_inspection


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculeNameCapture:
	"""One exact tab, selection, and direct-root address frozen before the dialog."""

	tab: object
	revision: int
	digest: str
	address: native_molecule_inspection.FerrumNativeMoleculeInspectionAddress
	current_name: str
	selection: tuple[str, ...]


#============================================
class FerrumNativeMoleculeNameTabMixin:
	"""Commit one already-authenticated molecule name through the Rust session."""

	#============================================
	def set_durable_molecule_name_v1(self, expected_revision: int,
			expected_digest: str, molecule_id: str, name: str,
			selection: tuple[str, ...]) -> object:
		"""Submit exact name intent and install only a changed observation."""
		self._require_mutable()
		if (
			type(expected_revision) is not int
			or type(expected_digest) is not str
			or type(molecule_id) is not str
			or type(name) is not str
			or type(selection) is not tuple
		):
			raise TypeError("Ferrum molecule name requires exact frozen Ferrum inputs")
		if any(
			type(object_id) is not str or not object_id for object_id in selection
		) or len(frozenset(selection)) != len(selection):
			raise TypeError("Ferrum molecule name requires exact durable selection targets")
		snapshot = self.current_snapshot
		if snapshot.revision != expected_revision or snapshot.digest != expected_digest:
			raise RuntimeError("document changed while the molecule name dialog was open")
		result = self._session.set_document_molecule_name_v1(
			expected_revision, expected_digest, molecule_id, name,
		)
		authoritative = result.observation.snapshot
		if (
			authoritative.revision == snapshot.revision
			and authoritative.digest == snapshot.digest
		):
			return result
		self._install_mutation_result(result, selection)
		return result


#============================================
class FerrumNativeMoleculeNameWindowMixin:
	"""Own the synchronous selected-molecule name action and modal-state fence."""

	#============================================
	def _build_molecule_name_action(self) -> None:
		"""Create and register the Ferrum molecule-name action."""
		self._set_molecule_name_action = PySide6.QtGui.QAction(
			self.tr("Set Molecule Name..."), self,
		)
		self._set_molecule_name_action.setToolTip(self.tr(
			"Replace or clear one selected durable molecule name through Rust",
		))
		self._set_molecule_name_action.triggered.connect(self._on_set_molecule_name)
		self._action_registry.register_existing(
			"chemistry.molecule.name", self._set_molecule_name_action,
			shortcut_exemption_reason="Available by its labelled Chemistry menu client.",
		)

	#============================================
	def _refresh_molecule_name_action(self, active: bool, pending: bool,
			busy: bool) -> None:
		"""Enable name editing only for one current direct-root selection."""
		self._set_molecule_name_action.setEnabled(
			active and not pending and not busy and self._molecule_name_capture() is not None,
		)

	#============================================
	def _on_set_molecule_name(self) -> bool:
		"""Collect exact text and commit only while every captured fact remains current."""
		capture = self._molecule_name_capture()
		if capture is None:
			return False
		name, accepted = PySide6.QtWidgets.QInputDialog.getText(
			self,
			self.tr("Set Molecule Name"),
			self.tr("Molecule name (leave empty to clear):"),
			text=capture.current_name,
		)
		if not accepted:
			return False
		if not self._molecule_name_capture_is_current(capture):
			self._show_edit_refusal(self._unavailable_edit_refusal("The active molecule selection changed while the name dialog was open."))
			return False
		try:
			capture.tab.set_durable_molecule_name_v1(
				capture.revision,
				capture.digest,
				capture.address.molecule_id,
				name,
				capture.selection,
			)
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return False
		self.statusBar().showMessage(self.tr("Updated the molecule name."), 5000)
		self._refresh_actions()
		return True

	#============================================
	def _molecule_name_capture(self) -> _MoleculeNameCapture | None:
		"""Freeze one exact current root, name, and durable child selection."""
		tab = self._active_native_tab()
		if tab is None:
			return None
		address = _selected_molecule_name_address(tab)
		if address is None:
			return None
		root = _matching_projection_root(tab, address)
		if root is None or (root.name is not None and type(root.name) is not str):
			return None
		selection = _selected_document_object_selection(tab)
		if selection is None:
			return None
		snapshot = tab.current_snapshot
		return _MoleculeNameCapture(
			tab,
			snapshot.revision,
			snapshot.digest,
			address,
			root.name or "",
			selection,
		)

	#============================================
	def _molecule_name_capture_is_current(self, capture: _MoleculeNameCapture) -> bool:
		"""Reauthenticate tab, provenance, root, and exact child selection after the dialog."""
		tab = capture.tab
		if (
			tab not in self._native_tabs_by_page
			or self._active_native_tab() is not tab
			or tab.requires_refresh
		):
			return False
		snapshot = tab.current_snapshot
		current = self._molecule_name_capture()
		return (
			snapshot.revision == capture.revision
			and snapshot.digest == capture.digest
			and current is not None
			and current.address == capture.address
			and current.selection == capture.selection
		)


#============================================
def _matching_projection_root(tab: object,
		address: native_molecule_inspection.FerrumNativeMoleculeInspectionAddress) -> object | None:
	"""Return the sole projection root matching the captured durable root ID."""
	import ferrum_qt.ferrum.engine as engine
	matches = tuple(
		root for root in tab.current_document_observation().projection.molecules
		if (
			type(root) is engine.MoleculeProjectionV1
			and root.document_object_id == address.molecule_id
		)
	)
	return matches[0] if len(matches) == 1 else None


#============================================
def _selected_molecule_name_address(
		tab: object) -> native_molecule_inspection.FerrumNativeMoleculeInspectionAddress | None:
	"""Resolve one selected root or one molecule-owned structural selection."""
	import ferrum_qt.ferrum.engine as engine
	if getattr(tab, "requires_refresh", True):
		return None
	observation = tab.current_document_observation()
	if type(observation) is not engine.SessionDocumentObservationV1:
		return None
	projection = observation.projection
	if type(projection) is not engine.DocumentProjectionV1:
		return None
	molecules = projection.molecules
	direct_roots = projection.direct_roots
	if type(molecules) is not list or type(direct_roots) is not tuple:
		return None
	molecule_ids: set[str] = set()
	for molecule in molecules:
		if (
			type(molecule) is not engine.MoleculeProjectionV1
			or type(molecule.document_object_id) is not str
			or not molecule.document_object_id
			or molecule.document_object_id in molecule_ids
		):
			return None
		molecule_ids.add(molecule.document_object_id)
	direct_molecule_ids: set[str] = set()
	direct_root_ids: set[str] = set()
	for root in direct_roots:
		object_id = getattr(root, "document_object_id", None)
		kind = getattr(root, "kind", None)
		if (
			type(object_id) is not str
			or not object_id
			or type(kind) is not str
			or not kind
			or object_id in direct_root_ids
		):
			return None
		direct_root_ids.add(object_id)
		if kind == "molecule":
			direct_molecule_ids.add(object_id)
	if direct_molecule_ids != molecule_ids:
		return None
	selection = _selected_document_object_selection(tab)
	if selection is None:
		return None
	selected_ids = selection
	if len(selected_ids) == 1 and selected_ids[0] in direct_molecule_ids:
		return native_molecule_inspection.FerrumNativeMoleculeInspectionAddress(selected_ids[0])
	if any(object_id in direct_molecule_ids for object_id in selected_ids):
		return None
	try:
		members = tab.selected_structure_targets()
	except (RuntimeError, TypeError, ValueError):
		return None
	if type(members) is not tuple or not members:
		return None
	member_ids: set[str] = set()
	member_molecule_ids: set[str] = set()
	for member in members:
		object_id = getattr(member, "object_id", None)
		molecule_id = getattr(member, "molecule_object_id", None)
		if (
			type(object_id) is not str
			or not object_id
			or type(molecule_id) is not str
			or not molecule_id
			or object_id in member_ids
		):
			return None
		member_ids.add(object_id)
		member_molecule_ids.add(molecule_id)
	if set(selected_ids) != member_ids or len(member_molecule_ids) != 1:
		return None
	molecule_id = next(iter(member_molecule_ids))
	if molecule_id not in direct_molecule_ids:
		return None
	return native_molecule_inspection.FerrumNativeMoleculeInspectionAddress(molecule_id)


#============================================
def _selected_document_object_selection(
		tab: object) -> tuple[str, ...] | None:
	"""Freeze exact generic canvas identities for post-mutation restoration."""
	from ferrum_qt.canvas.ferrum_render_target import RenderTargetKey
	targets = tab.selected_molecule_information_targets()
	if type(targets) is not tuple or not targets:
		return None
	selection = []
	seen = set()
	for target in targets:
		if (
			type(target) is not RenderTargetKey
			or target.kind != "document_object"
			or type(target.document_object_id) is not str
			or not target.document_object_id
			or target.document_object_id in seen
		):
			return None
		seen.add(target.document_object_id)
		selection.append(target.document_object_id)
	return tuple(selection)
