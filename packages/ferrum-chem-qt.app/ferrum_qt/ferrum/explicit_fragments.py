"""Ordinary Ferrum Qt client for Rust-owned explicit fragment annotations."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_qt.ferrum.engine as engine
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeExplicitFragmentCapture:
	"""One exact selected direct-root membership receipt for a name dialog."""

	tab: object
	revision: int
	digest: str
	molecule_id: str
	atom_ids: tuple[str, ...]
	bond_ids: tuple[str, ...]


#============================================
def capture_explicit_fragment_selection(
		tab: object) -> FerrumNativeExplicitFragmentCapture | None:
	"""Freeze one durable selected-membership request for a direct molecule root."""
	if getattr(tab, "requires_refresh", True):
		return None
	targets = tab.selected_structure_targets()
	if type(targets) is not tuple or not targets:
		return None
	selected_molecule_id = None
	selected_atoms = set()
	selected_bonds = set()
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
		(
			selected_atoms
			if target.kind == engine.StructureTargetKindV1.atom
			else selected_bonds
		).add(
			target.object_id,
		)
	if selected_molecule_id is None:
		return None
	if (
		any(type(identifier) is not str or not identifier for identifier in selected_atoms)
		or any(type(identifier) is not str or not identifier for identifier in selected_bonds)
	):
		return None
	observation = tab.current_document_observation()
	snapshot = observation.snapshot
	return FerrumNativeExplicitFragmentCapture(
		tab,
		snapshot.revision,
		snapshot.digest,
		selected_molecule_id,
		tuple(sorted(selected_atoms)),
		tuple(sorted(selected_bonds)),
	)


#============================================
class _CreateExplicitFragmentDialog(FerrumAccessibleDialog):
	"""One labelled, retry-friendly name dialog without a local fragment model."""

	#============================================
	def __init__(self, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build the small explanatory form from scalar UI state only."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Create Fragment"))
		self.setAccessibleName(self.tr("Create Fragment"))
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		explanation = PySide6.QtWidgets.QLabel(self.tr(
			"This labels the selected atoms and bonds without changing the molecule "
			"or creating a reusable template.",
		), self)
		explanation.setWordWrap(True)
		layout.addWidget(explanation)
		form = PySide6.QtWidgets.QFormLayout()
		self.name_edit = PySide6.QtWidgets.QLineEdit(self)
		self.name_edit.setAccessibleName(self.tr("Fragment name"))
		form.addRow(self.tr("Fragment name:"), self.name_edit)
		layout.addLayout(form)
		self.error_label = PySide6.QtWidgets.QLabel(self)
		self.error_label.setWordWrap(True)
		self.error_label.setStyleSheet("color: #a40000;")
		self.error_label.setAccessibleName(self.tr("Fragment name error"))
		self.error_label.hide()
		layout.addWidget(self.error_label)
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel, self,
		)
		self.create_button = buttons.addButton(
			self.tr("Create fragment"),
			PySide6.QtWidgets.QDialogButtonBox.ButtonRole.AcceptRole,
		)
		self.create_button.clicked.connect(self.accept)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)
		self.name_edit.setFocus()

	#============================================
	def show_error(self, message: str) -> None:
		"""Keep the proposed name and return correction focus to its field."""
		self.error_label.setText(message)
		self.error_label.show()
		self.name_edit.setFocus()


#============================================
class _ExplicitFragmentViewDialog(FerrumAccessibleDialog):
	"""Read-only scalar observation presentation for supported fragment records."""

	#============================================
	def __init__(self, observation: object,
			parent: PySide6.QtWidgets.QWidget) -> None:
		"""Render only Rust-described V1 facts and one retention notice."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Fragments in this drawing"))
		self.setAccessibleName(self.tr("Fragments in this drawing"))
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		heading = PySide6.QtWidgets.QLabel(self.tr("Fragments in this drawing"), self)
		heading.setAccessibleName(self.tr("Fragment list heading"))
		layout.addWidget(heading)
		if observation.records:
			rows = PySide6.QtWidgets.QTreeWidget(self)
			rows.setAccessibleName(self.tr("Named fragments"))
			rows.setHeaderLabels((self.tr("Name"), self.tr("Molecule"), self.tr("Type")))
			for record in observation.records:
				row = PySide6.QtWidgets.QTreeWidgetItem((
					record.name, self.tr("Molecule {0}").format(record.molecule_id),
					self.tr("Explicit"),
				))
				rows.addTopLevelItem(row)
			rows.resizeColumnToContents(0)
			rows.resizeColumnToContents(1)
			layout.addWidget(rows)
		else:
			empty = PySide6.QtWidgets.QLabel(self.tr(
				"No named fragments yet. Select part of one molecule, then choose "
				"Chemistry -> Create Fragment...",
			), self)
			empty.setWordWrap(True)
			layout.addWidget(empty)
		if observation.has_retained_fragment_metadata:
			notice = PySide6.QtWidgets.QLabel(self.tr(
				"Some imported fragment metadata is retained but cannot be edited here.",
			), self)
			notice.setWordWrap(True)
			layout.addWidget(notice)
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close, self,
		)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)


#============================================
class FerrumNativeExplicitFragmentsWindowMixin:
	"""Own normal Chemistry actions and modal lifecycle around Rust receipts."""

	#============================================
	def _build_explicit_fragment_actions(self) -> None:
		"""Create and register explicit-only Create and read-only View actions."""
		self._create_explicit_fragment_action = PySide6.QtGui.QAction(
			self.tr("Create Fragment..."), self,
		)
		self._create_explicit_fragment_action.setToolTip(self.tr(
			"Name the selected part of one molecule for this drawing.",
		))
		self._create_explicit_fragment_action.setStatusTip(self.tr(
			"Name the selected part of one molecule for this drawing.",
		))
		self._create_explicit_fragment_action.triggered.connect(self._on_create_explicit_fragment)
		self._view_explicit_fragments_action = PySide6.QtGui.QAction(
			self.tr("View Fragments..."), self,
		)
		self._view_explicit_fragments_action.triggered.connect(self._on_view_explicit_fragments)
		self._view_explicit_fragments_action.setStatusTip(
			self._view_explicit_fragments_action.text(),
		)
		for action_id, action in (
			("chemistry.fragments.create", self._create_explicit_fragment_action),
			("chemistry.fragments.view", self._view_explicit_fragments_action),
		):
			self._action_registry.register_existing(
				action_id, action,
				shortcut_exemption_reason="Available by its labelled Chemistry menu client.",
			)

	#============================================
	def _refresh_explicit_fragment_actions(self, active: bool, pending: bool,
			busy: bool) -> None:
		"""Expose Create only for an exact single-root durable selection."""
		self._close_stale_explicit_fragment_view(active, pending, busy)
		capture = self._explicit_fragment_capture() if active and not pending else None
		self._create_explicit_fragment_action.setEnabled(
			active and not pending and not busy and capture is not None,
		)
		self._create_explicit_fragment_action.setToolTip(self.tr(
			"Name the selected part of one molecule for this drawing."
			if capture is not None else
			"Select atoms or bonds from one editable molecule to create a fragment.",
		))
		self._view_explicit_fragments_action.setEnabled(active and not pending and not busy)

	#============================================
	def _on_create_explicit_fragment(self, _checked: bool = False) -> bool:
		"""Capture provenance before collecting the one human label."""
		capture = self._explicit_fragment_capture()
		if capture is None:
			self.statusBar().showMessage(self.tr(
				"Select atoms or bonds from one editable molecule to create a fragment.",
			), 5000)
			return False
		dialog = _CreateExplicitFragmentDialog(self)
		while True:
			if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
				self._focus_explicit_fragment_source(capture)
				self._refresh_actions()
				return False
			if not dialog.name_edit.text().strip():
				dialog.show_error(self.tr("Enter a fragment name."))
				continue
			if not self._explicit_fragment_capture_is_current(capture):
				dialog.reject()
				self._fragment_changed_before_create()
				return False
			try:
				result = capture.tab.create_explicit_fragment_v1(
					capture.revision, capture.digest, capture.molecule_id,
					dialog.name_edit.text(), capture.atom_ids, capture.bond_ids,
				)
			except engine.DocumentExplicitFragmentError:
				if not self._explicit_fragment_capture_is_current(capture):
					dialog.reject()
					self._fragment_changed_before_create()
					return False
				dialog.show_error(self.tr(
					"The fragment could not be created. The name and selection are still "
					"available; review them and try again.",
				))
				continue
			self.statusBar().showMessage(self.tr(
				'Created fragment "{0}". The molecule is unchanged.',
			).format(result.fragment.name), 5000)
			self._focus_explicit_fragment_source(capture)
			self._refresh_actions()
			return True

	#============================================
	def _on_view_explicit_fragments(self, _checked: bool = False) -> bool:
		"""Present one immutable Rust observation without creating a Qt metadata owner."""
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		snapshot = tab.current_snapshot
		try:
			observation = engine.inspect_document_explicit_fragments_v1(
				tab.current_document_observation(), snapshot.revision, snapshot.digest,
			)
		except engine.DocumentExplicitFragmentError:
			self._show_edit_refusal(self._unavailable_edit_refusal(self.tr(
				"The fragments could not be shown for this drawing. Refresh the drawing "
				"and try again.",
			)))
			return False
		if (
			self._active_native_tab() is not tab
			or tab.requires_refresh
			or tab.current_snapshot.revision != snapshot.revision
			or tab.current_snapshot.digest != snapshot.digest
		):
			self._fragment_changed_before_create()
			return False
		dialog = _ExplicitFragmentViewDialog(observation, self)
		self._explicit_fragment_view = (dialog, tab, snapshot.revision, snapshot.digest)
		try:
			dialog.exec()
		finally:
			if getattr(self, "_explicit_fragment_view", None) is not None:
				view_dialog, _tab, _revision, _digest = self._explicit_fragment_view
				if view_dialog is dialog:
					self._explicit_fragment_view = None
			self._focus_explicit_fragment_view_source(tab)
		self._refresh_actions()
		return True

	#============================================
	def _close_stale_explicit_fragment_view(self, active: bool, pending: bool,
			busy: bool) -> None:
		"""Close a View dialog once its captured source is no longer authoritative."""
		view = getattr(self, "_explicit_fragment_view", None)
		if view is None:
			return
		dialog, tab, revision, digest = view
		if (
			active
			and not pending
			and not busy
			and self._native_tabs_by_page.get(tab) is tab
			and not tab.is_disposed
			and not tab.requires_refresh
			and self._active_native_tab() is tab
			and tab.current_snapshot.revision == revision
			and tab.current_snapshot.digest == digest
		):
			return
		self._explicit_fragment_view = None
		dialog.reject()

	#============================================
	def _focus_explicit_fragment_view_source(self, tab: object) -> None:
		"""Restore canvas focus only when this remains the active live source."""
		if (
			self._native_tabs_by_page.get(tab) is tab
			and not tab.is_disposed
			and self._active_native_tab() is tab
		):
			tab.view.viewport().setFocus()

	#============================================
	def _explicit_fragment_capture(self) -> FerrumNativeExplicitFragmentCapture | None:
		"""Freeze the active tab's exact ordinary selection without endpoint inference."""
		tab = self._active_native_tab()
		if tab is None:
			return None
		try:
			return capture_explicit_fragment_selection(tab)
		except (RuntimeError, TypeError, ValueError):
			return None

	#============================================
	def _explicit_fragment_capture_is_current(
			self, capture: FerrumNativeExplicitFragmentCapture) -> bool:
		"""Refuse stale, redirected, closed, or reselected modal submissions."""
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
			and self._explicit_fragment_capture() == capture
		)

	#============================================
	def _fragment_changed_before_create(self) -> None:
		"""Give one source-neutral recovery message for a discarded modal intent."""
		self.statusBar().showMessage(self.tr(
			"The drawing changed before the fragment was created. Select the part again.",
		), 5000)
		self._refresh_actions()

	#============================================
	def _focus_explicit_fragment_source(self,
			capture: FerrumNativeExplicitFragmentCapture) -> None:
		"""Return keyboard focus only to a still-live captured source canvas."""
		if capture.tab in self._native_tabs_by_page and not capture.tab.is_disposed:
			capture.tab.view.viewport().setFocus()
