"""Ordinary native Qt action for Rust-owned linear-form conversion."""

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
	"""Expand selected bonds and freeze one source-ordered direct-root atom selection."""
	if getattr(tab, "requires_refresh", True):
		return None
	targets = tab.selected_molecule_information_targets()
	if type(targets) is not tuple or not targets:
		return None
	observation = tab.current_document_observation()
	molecules = observation.projection.molecules
	selected_root_index = None
	selected_atom_ids = set()
	for target in targets:
		if (
			target.kind not in ("atom", "bond")
			or type(target.identifier) is not str
			or not target.identifier
			or type(target.source_order) is not int
		):
			return None
		matches = []
		for root_index, molecule in enumerate(molecules):
			children = molecule.atoms if target.kind == "atom" else molecule.bonds
			matches.extend(
				(root_index, molecule, child) for child in children
				if child.source_id == target.identifier
				and child.source_order == target.source_order
			)
		if len(matches) != 1:
			return None
		root_index, _molecule, child = matches[0]
		if selected_root_index is None:
			selected_root_index = root_index
		elif selected_root_index != root_index:
			return None
		if target.kind == "atom":
			selected_atom_ids.add(target.identifier)
		else:
			endpoints = (child.start, child.end)
			if any(
				endpoint.kind != "atom"
				or type(endpoint.source_id) is not str
				or not endpoint.source_id
				for endpoint in endpoints
			):
				return None
			selected_atom_ids.update(endpoint.source_id for endpoint in endpoints)
	if selected_root_index is None:
		return None
	molecule = molecules[selected_root_index]
	if type(molecule.id) is not str or not molecule.id:
		return None
	ordered_atoms = tuple(
		atom for atom in molecule.atoms
		if atom.source_id in selected_atom_ids
	)
	atom_ids = tuple(atom.source_id for atom in ordered_atoms)
	if (
		not atom_ids
		or len(atom_ids) != len(selected_atom_ids)
		or any(type(atom_id) is not str or not atom_id for atom_id in atom_ids)
		or len(frozenset(atom_ids)) != len(atom_ids)
	):
		return None
	snapshot = observation.snapshot
	return FerrumNativeLinearFormCapture(
		tab, snapshot.revision, snapshot.digest, molecule.id, atom_ids,
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
			raise TypeError("native linear-form conversion requires exact frozen inputs")
		if (
			not atom_ids
			or any(type(atom_id) is not str or not atom_id for atom_id in atom_ids)
			or len(frozenset(atom_ids)) != len(atom_ids)
		):
			raise ValueError(
				"native linear-form conversion requires distinct durable atom IDs",
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
		selection = tuple(("atom", atom_id) for atom_id in atom_ids)
		self._install_mutation_result(result, selection)
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
			self._show_native_file_warning(
				"Convert to Linear Form",
				"The active atom selection changed before conversion.",
			)
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
			self._show_native_file_warning("Convert to Linear Form", str(exc))
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
