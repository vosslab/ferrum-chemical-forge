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
	selection: tuple[tuple[str, str], ...]


#============================================
class FerrumNativeMoleculeNameTabMixin:
	"""Commit one already-authenticated molecule name through the Rust session."""

	#============================================
	def set_durable_molecule_name_v1(self, expected_revision: int,
			expected_digest: str, molecule_id: str, name: str,
			selection: tuple[tuple[str, str], ...]) -> object:
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
			type(target) is not tuple
			or len(target) != 2
			or type(target[0]) is not str
			or type(target[1]) is not str
			for target in selection
		):
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
	def _build_molecule_name_action(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the Ferrum name action beside the existing molecule information route."""
		menu.addSeparator()
		self._set_molecule_name_action = PySide6.QtGui.QAction(
			self.tr("Set Molecule Name..."), self,
		)
		self._set_molecule_name_action.setToolTip(self.tr(
			"Replace or clear one selected durable molecule name through Rust",
		))
		self._set_molecule_name_action.triggered.connect(self._on_set_molecule_name)
		menu.addAction(self._set_molecule_name_action)

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
		address = native_molecule_inspection.selected_durable_molecule_address(tab)
		if address is None:
			return None
		root = _matching_projection_root(tab, address)
		if root is None:
			return None
		targets = tab.selected_molecule_information_targets()
		selection = tuple((target.kind, target.identifier) for target in targets)
		if not selection:
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
	"""Return the sole projection root matching every captured corroborator."""
	matches = tuple(
		root for root in tab.current_document_observation().projection.molecules
		if root.id == address.molecule_id
		and root.projection_key == address.projection_key
		and root.source_id == address.source_id
		and root.source_order == address.document_root_order
	)
	return matches[0] if len(matches) == 1 else None
