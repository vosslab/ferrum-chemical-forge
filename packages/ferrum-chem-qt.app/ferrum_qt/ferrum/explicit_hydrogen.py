"""Live Rust-owned explicit-hydrogen materialization action."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
_OPERATION_KIND = "document.molecule.hydrogen.materialize.v1"
#============================================
class FerrumNativeExplicitHydrogenWindowMixin:
	"""Expose one live Rust-owned materialization command in the Chemistry menu."""

	#============================================
	def _initialize_explicit_hydrogen(self) -> None:
		"""Initialize the action before the window constructs its menus."""
		self._explicit_hydrogen_action: PySide6.QtGui.QAction | None = None

	#============================================
	def _build_explicit_hydrogen_action(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the accessible public Chemistry action."""
		action = PySide6.QtGui.QAction(self.tr("Make Hydrogens Explicit"), self)
		action.setObjectName("make-hydrogens-explicit-action")
		action.setIconText(self.tr("Make Hydrogens Explicit"))
		action.setStatusTip(self.tr(
			"Materialize hydrogens for one selected molecule with Ferrum Rust.",
		))
		action.setToolTip(self.tr(
			"Materialize hydrogen atoms for the selected molecule with Ferrum Rust.",
		))
		action.setWhatsThis(self.tr(
			"Materialize explicit hydrogen atoms for the molecule containing one selected atom. "
			"Ferrum Rust validates chemistry, identifiers, geometry, and rendering.",
		))
		action.triggered.connect(self._make_hydrogens_explicit)
		menu.addAction(action)
		self._explicit_hydrogen_action = action

	#============================================
	def _make_hydrogens_explicit(self) -> bool:
		"""Materialize one selected molecule through the fenced live Rust adapter."""
		tab = self._active_native_tab()
		if tab is None or self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			return False
		if self._atom_insertion_intent is not None:
			self._cancel_atom_insertion()
		if self._line_gesture_intent is not None:
			self._cancel_line_gesture()
		if self._structure_tab is tab:
			self._cancel_structure_selection()
		try:
			address = tab.selected_molecule_atom_address()
			result = tab._session.materialize_live_molecule_hydrogens_v1(
				address.revision, address.digest, address.molecule_id, address.atom_id,
			)
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return False
		except (TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return False
		if self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			return False
		materialization = result.outcome.molecule_hydrogens_materialized
		if materialization is None:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum returned a live hydrogen result without its materialization outcome.",
			))
			self._refresh_actions()
			self._publish_operation_presentation_v1(
				tab, _OPERATION_KIND, "failed", "unchanged",
				address.revision, address.digest,
			)
			return False
		if materialization.changed:
			try:
				tab._install_mutation_result(result, (address.atom_id,))
			except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				self._refresh_actions()
				self._publish_operation_presentation_v1(
					tab, _OPERATION_KIND, "failed", "unchanged",
					address.revision, address.digest,
				)
				return False
			self.statusBar().showMessage(self.tr(
				"Made {0} hydrogens explicit with Ferrum Rust.".format(
					materialization.added_hydrogen_count,
				),
			), 5000)
			self._refresh_actions()
			self._publish_operation_presentation_v1(
				tab, _OPERATION_KIND, "succeeded", "updated",
				address.revision, address.digest,
			)
			return True
		self.statusBar().showMessage(self.tr(
			"Hydrogens are already explicit for the selected molecule.",
		), 5000)
		self._refresh_actions()
		self._publish_operation_presentation_v1(
			tab, _OPERATION_KIND, "succeeded", "unchanged",
			address.revision, address.digest,
		)
		return True

	#============================================
	def _refresh_explicit_hydrogen_action(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Enable only for a mutable live tab with exactly one durable atom selected."""
		action = self._explicit_hydrogen_action
		if action is None:
			return
		available = False
		if active and not pending and not busy_elsewhere:
			tab = self._active_native_tab()
			try:
				if tab is not None:
					tab.selected_molecule_atom_address()
			except native_document_tab_errors.FerrumNativeDocumentTabError:
				available = False
			else:
				available = tab is not None
		action.setEnabled(available)
