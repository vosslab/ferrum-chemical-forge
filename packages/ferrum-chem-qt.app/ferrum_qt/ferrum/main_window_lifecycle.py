"""Ferrum Qt tab lifecycle and action-refresh responsibilities."""
# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.atom_element
import ferrum_qt.ferrum.drawing_standard as native_drawing_standard
import ferrum_qt.ferrum.paper_properties as native_paper_properties
import ferrum_qt.ferrum.presentation_properties
import ferrum_qt.ferrum.arrow_properties
import ferrum_qt.ferrum.geometric_properties as native_geometric_properties
import ferrum_qt.ferrum.wavy_properties as native_wavy_properties
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.ferrum.local_document_open_contract

#============================================
class FerrumNativeMainWindowLifecycleMixin:
	"""Own tab registration, lifecycle guards, and action reachability."""

	#============================================
	def _structure_selection_mutation_eligible(self) -> bool:
		"""Return the sole current gate for a Rust structural-selection mutation."""
		tab = self._active_native_tab()
		if tab is None or tab.is_disposed or tab.requires_refresh:
			return False
		return not (
			self._molecule_import_busy()
			or self._molecule_export_busy()
			or self._molecule_inspection_busy()
			or self._molecule_diagnostics_busy()
			or self._atom_oxidation_busy()
			or self._compact_group_materialization_intent is not None
			or self._compact_group_authoring_intent is not None
			or self._clipboard_busy()
			or self._coordinate_generation_intent is not None
			or self._operation_leases.has_active(
				ferrum_qt.ferrum.operation_leases.OperationFamily.TEMPLATE_CATALOG,
				tab=tab,
			)
			or self._snapshot_export_busy()
		)

	def _register_native_tab(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			*, activate: bool = True,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Attach one exact Ferrum tab to this standalone public host."""
		if type(tab) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum window requires an exact FerrumNativeDocumentTab")
		if tab in self._native_tabs_by_page:
			raise ValueError("Ferrum tab is already registered")
		prior_tab = self._active_native_tab()
		self._operation_leases.bind_tab(tab)
		provisional_complete = False
		try:
			index = self._tab_widget.addTab(tab, tab.title)
			self._finish_native_tab_registration(tab)
			provisional_complete = True
		finally:
			if not provisional_complete:
				self._retire_provisional_native_tab(tab)
				self._restore_registered_tab_after_provisional_refusal(prior_tab)
		if activate:
			self._tab_widget.setCurrentIndex(index)
		# The first addTab() selects its page before this method can publish the
		# page-to-tab mapping.  Re-enter the authoritative activation lifecycle
		# only when that early Qt signal could not identify this mapped tab.
		if self._active_native_tab() is tab and self._last_native_tab is not tab:
			self._on_native_tab_changed(index)
		return tab

	#============================================
	def _publish_local_open_tab(
			self, tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> ferrum_qt.ferrum.local_document_open_contract.LocalOpenNewTabPublicationReceipt:
		"""Commit a Local Open candidate without optional activation or display work."""
		self._require_unregistered_native_tab(tab)
		self._operation_leases.bind_tab(tab)
		published = False
		try:
			with PySide6.QtCore.QSignalBlocker(self._tab_widget):
				index = self._tab_widget.addTab(tab, tab.title)
				self._finish_native_tab_registration(tab)
			published = True
		finally:
			if not published:
				self._unpublish_local_open_candidate(tab)
		receipt = ferrum_qt.ferrum.local_document_open_contract.LocalOpenNewTabPublicationReceipt(
			tab, index,
		)
		return receipt

	#============================================
	def _finish_local_open_publication(
			self, receipt: ferrum_qt.ferrum.local_document_open_contract.LocalOpenNewTabPublicationReceipt,
			activate: bool,
			) -> None:
		"""Apply optional activation after an already truthful new-tab publication."""
		self._require_local_open_publication_receipt(receipt)
		try:
			if activate:
				self._tab_widget.setCurrentIndex(receipt.index)
			if self._active_native_tab() is receipt.tab and self._last_native_tab is not receipt.tab:
				self._on_native_tab_changed(receipt.index)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError as error:
			raise ferrum_qt.ferrum.local_document_open_contract.LocalOpenPostCommitPresentationError(
				"Ferrum could not refresh the committed Local Open tab",
			) from error

	#============================================
	def _require_unregistered_native_tab(
			self, tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Validate one candidate before any publication-specific mutation."""
		if type(tab) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum window requires an exact FerrumNativeDocumentTab")
		if tab in self._native_tabs_by_page or tab.is_disposed:
			raise ValueError("Ferrum Local Open candidate is already unavailable")
		if self._tab_widget.indexOf(tab) >= 0:
			raise ValueError("Ferrum Local Open candidate is already registered")

	#============================================
	def _require_local_open_publication_receipt(
			self, receipt: ferrum_qt.ferrum.local_document_open_contract.LocalOpenNewTabPublicationReceipt,
			) -> None:
		"""Require the exact new tab and index committed by Local Open."""
		if type(receipt) is not ferrum_qt.ferrum.local_document_open_contract.LocalOpenNewTabPublicationReceipt:
			raise TypeError("Ferrum Local Open completion requires its publication receipt")
		if self._native_tabs_by_page.get(receipt.tab) is not receipt.tab:
			raise RuntimeError("Ferrum Local Open committed tab is no longer registered")
		if self._tab_widget.indexOf(receipt.tab) != receipt.index:
			raise RuntimeError("Ferrum Local Open committed tab moved before presentation")

	#============================================
	def _restore_registered_tab_after_provisional_refusal(
			self,
			prior_tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None,
			) -> None:
		"""Restore the exact prior page after a candidate registration refuses."""
		if prior_tab is None:
			return
		prior_index = self._tab_widget.indexOf(prior_tab)
		if prior_index < 0:
			return
		with PySide6.QtCore.QSignalBlocker(self._tab_widget):
			self._tab_widget.setCurrentIndex(prior_index)
		self._last_native_tab = None
		self._on_native_tab_changed(prior_index)

	#============================================
	def _finish_native_tab_registration(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Install one bound tab's shared host integrations before activation."""
		self._native_tabs_by_page[tab] = tab
		self._install_native_hex_grid_for_tab(tab)
		tab.selection_changed.connect(self._on_native_selection_changed)
		tab.view.display_transform_changed.connect(self._refresh_native_view_status)
		index = self._tab_widget.indexOf(tab)
		if index < 0:
			raise RuntimeError("Ferrum cannot finish registration without a tab page")
		self._tab_widget.setTabToolTip(
			index, tab.local_document_source_description or "",
		)

	#============================================
	def _replace_registered_native_tab(
			self,
			old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			index: int,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Replace one exact idle registered page through the sole tab lifecycle."""
		return self._replace_registered_native_tab_transition(old, new, index)

	#============================================
	def _commit_local_open_replacement(
			self, old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, index: int,
			capability: ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability,
			lease: ferrum_qt.ferrum.operation_leases.OperationLease,
			) -> ferrum_qt.ferrum.local_document_open_contract.LocalOpenReplacementCommitReceipt:
		"""Commit a prepared Local Open swap without post-commit presentation work."""
		self._validate_local_open_replacement(old, new, index, capability, lease)
		self._publish_provisional_replacement_candidate(new, index)
		prepared = self._prepare_local_open_source(capability, lease, old, new)
		receipt = ferrum_qt.ferrum.local_document_open_contract.LocalOpenReplacementCommitReceipt(
			old, new, index, lease.lease_id, lease.tab_identity,
		)
		self._dispose_or_restore_local_open_source(prepared, old, new)
		self._close_local_open_replacement(prepared, old, index)
		return receipt

	#============================================
	def _validate_local_open_replacement(
			self, old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, index: int,
			capability: ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability,
			lease: ferrum_qt.ferrum.operation_leases.OperationLease,
			) -> None:
		"""Complete every externally visible Local Open check before old disposal."""
		if type(old) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum replacement requires an exact old Ferrum tab")
		self._require_unregistered_native_tab(new)
		if self._native_tabs_by_page.get(old) is not old or old.is_disposed:
			raise ValueError("Ferrum replacement target is no longer registered")
		if self._tab_widget.indexOf(old) != index:
			raise ValueError("Ferrum replacement target changed its tab position")
		active_leases = self._operation_leases.active_for_tab(old)
		if len(active_leases) != 1 or active_leases[0].lease_id != lease.lease_id:
			raise ValueError("Ferrum Open replacement requires its sole active source lease")
		if active_leases[0].state is not ferrum_qt.ferrum.operation_leases.LeaseState.ACTIVE:
			raise ValueError("Ferrum Open replacement requires an active source lease")
		if type(capability) is not ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability:
			raise TypeError("Ferrum Open replacement requires its lease capability")

	#============================================
	def _publish_provisional_replacement_candidate(
			self, new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, index: int,
			) -> None:
		"""Integrate a recoverable replacement candidate without transferring ownership."""
		self._operation_leases.bind_tab(new)
		published = False
		try:
			with PySide6.QtCore.QSignalBlocker(self._tab_widget):
				self._tab_widget.insertTab(index, new, new.title)
				self._finish_native_tab_registration(new)
			published = True
		finally:
			if not published:
				self._unpublish_local_open_candidate(new)

	#============================================
	def _prepare_local_open_source(
			self, capability: ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability,
			lease: ferrum_qt.ferrum.operation_leases.OperationLease,
			old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> ferrum_qt.ferrum.operation_leases.PreparedTerminalReplacement:
		"""Prepare the exact source lease while replacement remains recoverable."""
		try:
			prepared = self._operation_leases.prepare_terminal_replacement(
				capability, lease, old,
			)
		except ferrum_qt.ferrum.operation_leases.OperationLeaseError:
			self._unpublish_local_open_candidate(new)
			raise
		return prepared

	#============================================
	def _dispose_or_restore_local_open_source(
			self, prepared: ferrum_qt.ferrum.operation_leases.PreparedTerminalReplacement,
			old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Dispose the source or restore every recoverable ownership edge on refusal."""
		try:
			old.dispose()
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
			self._operation_leases.restore_prepared_terminal_replacement(prepared, old)
			self._unpublish_local_open_candidate(new)
			raise

	#============================================
	def _close_local_open_replacement(
			self, prepared: ferrum_qt.ferrum.operation_leases.PreparedTerminalReplacement,
			old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, index: int,
			) -> None:
		"""Close only private registry and Qt structures after irreversible disposal."""
		with PySide6.QtCore.QSignalBlocker(self._tab_widget):
			self._tab_widget.removeTab(index + 1)
			del self._native_tabs_by_page[old]
		self._operation_leases.complete_prepared_terminal_replacement(prepared)

	#============================================
	def _replace_registered_native_tab_transition(
			self, old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, index: int,
			) -> object:
		"""Implement the recoverable registration and irreversible swap transaction."""
		if type(old) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum replacement requires an exact old Ferrum tab")
		if type(new) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum replacement requires an exact new Ferrum tab")
		if (
				self._native_tabs_by_page.get(old) is not old
				or old.is_disposed
				or self._tab_widget.indexOf(old) != index
			):
			raise ValueError("Ferrum replacement target is no longer registered")
		if (
				new in self._native_tabs_by_page
				or new.is_disposed
				or self._tab_widget.indexOf(new) >= 0
			):
			raise ValueError("Ferrum replacement tab is already registered")
		active_leases = self._operation_leases.active_for_tab(old)
		if active_leases:
			raise ValueError("Ferrum cannot replace a tab with an active operation")

		# Phase 1: make the incoming page a complete, but provisional, host tab.
		# The old tab remains live throughout this phase, so every failure has one
		# unambiguous recovery state.
		self._operation_leases.bind_tab(new)
		provisional_complete = False
		try:
			with PySide6.QtCore.QSignalBlocker(self._tab_widget):
				self._tab_widget.insertTab(index, new, new.title)
				self._finish_native_tab_registration(new)
			provisional_complete = True
		finally:
			if not provisional_complete:
				self._retire_provisional_native_tab(new)

		# Phase 2: remove the old lifetime only while it is still recoverable.
		# FerrumNativeDocumentTab.dispose() raises before changing disposal state,
		# so a typed refusal here can restore the exact old registration.
		try:
			self._operation_leases.unregister_tab(old)
		except ferrum_qt.ferrum.operation_leases.OperationLeaseError:
			self._retire_provisional_native_tab(new)
			raise
		try:
			old.dispose()
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
			self._operation_leases.bind_tab(old)
			self._retire_provisional_native_tab(new)
			raise

		# Phase 3: old disposal is irreversible.  Commit the already complete new
		# registration without offering a fictional rollback to a disposed page.
		# The provisional insertion at ``index`` deterministically shifted old to
		# ``index + 1`` while tab signals were blocked; no user or Qt re-entry can
		# alter that relation before this synchronous commit.
		with PySide6.QtCore.QSignalBlocker(self._tab_widget):
			self._tab_widget.removeTab(index + 1)
			del self._native_tabs_by_page[old]
		self._cancel_native_view_controls_for_tab(old)
		old.hide()
		old.setParent(None)
		self._tab_widget.setCurrentIndex(index)
		self._last_native_tab = None
		self._on_native_tab_changed(index)
		old.deleteLater()
		return new

	#============================================
	def _finish_local_open_replacement(
			self, receipt: ferrum_qt.ferrum.local_document_open_contract.LocalOpenReplacementCommitReceipt,
			) -> None:
		"""Apply presentation cleanup after an already-settled Local Open commit."""
		if type(receipt) is not ferrum_qt.ferrum.local_document_open_contract.LocalOpenReplacementCommitReceipt:
			raise TypeError("Ferrum Local Open completion requires its commit receipt")
		if self._native_tabs_by_page.get(receipt.new) is not receipt.new:
			raise RuntimeError("Ferrum Local Open committed tab is no longer registered")
		try:
			self._cancel_native_view_controls_for_tab(receipt.old)
		finally:
			receipt.old.hide()
			receipt.old.setParent(None)
			receipt.old.deleteLater()
		try:
			self._tab_widget.setCurrentIndex(receipt.index)
			self._last_native_tab = None
			self._on_native_tab_changed(receipt.index)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError as error:
			raise ferrum_qt.ferrum.local_document_open_contract.LocalOpenPostCommitPresentationError(
			"Ferrum could not refresh the committed Local Open tab",
		) from error

	#============================================
	def _retire_provisional_native_tab(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Retire a failed replacement page while preserving refusal recovery."""
		self._operation_leases.unregister_tab(tab)
		try:
			tab.dispose()
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
			self._operation_leases.bind_tab(tab)
			raise
		self._remove_provisional_native_tab_integrations(tab)
		with PySide6.QtCore.QSignalBlocker(self._tab_widget):
			index = self._tab_widget.indexOf(tab)
			if index >= 0:
				self._tab_widget.removeTab(index)
			self._native_tabs_by_page.pop(tab, None)
			self._cancel_native_view_controls_for_tab(tab)
			tab.hide()
			tab.setParent(None)
		tab.deleteLater()

	#============================================
	def _unpublish_local_open_candidate(
			self, tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Undo provisional Local Open integration while delivery retains the candidate."""
		self._operation_leases.unregister_tab(tab)
		self._remove_provisional_native_tab_integrations(tab)
		with PySide6.QtCore.QSignalBlocker(self._tab_widget):
			index = self._tab_widget.indexOf(tab)
			if index >= 0:
				self._tab_widget.removeTab(index)
			self._native_tabs_by_page.pop(tab, None)
			tab.hide()
			tab.setParent(None)

	#============================================
	def _remove_provisional_native_tab_integrations(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Allow a concrete host to undo integrations after provisional disposal."""
		del tab

	#============================================
	def save_active_to_path(self, path: str) -> bool:
		"""Publish the selected Ferrum tab to a caller-supplied CDML destination."""
		tab = self._active_native_tab()
		if tab is None:
			return False
		return self._save_native_tab_to_path(tab, path)

	#============================================
	@PySide6.QtCore.Slot(int)
	def _close_tab_at(self, index: int) -> None:
		"""Acquire an ordinary user close decision, then apply it once."""
		page = self._tab_widget.widget(index)
		tab = self._native_tabs_by_page.get(page)
		result = self._close_tab_at_with_decision(
			index, ferrum_qt.ferrum.close_decision.CloseDecision.KEEP_OPEN,
		)
		if result is ferrum_qt.ferrum.close_decision.CloseResult.DIRTY_REQUIRES_DECISION:
			if tab is not None:
				result = self._close_tab_at_with_decision(
					index, self._acquire_close_decision(tab),
				)
		if result not in (
				ferrum_qt.ferrum.close_decision.CloseResult.CLOSED,
				ferrum_qt.ferrum.close_decision.CloseResult.NO_TAB,
				ferrum_qt.ferrum.close_decision.CloseResult.DIRTY_REQUIRES_DECISION,
				ferrum_qt.ferrum.close_decision.CloseResult.SAVE_FAILED,
			):
			self._present_close_result_refusal(result)

	#============================================
	def _acquire_close_decision(self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> ferrum_qt.ferrum.close_decision.CloseDecision:
		"""Ask an ordinary author how to resolve one unsaved Ferrum tab."""
		choice = PySide6.QtWidgets.QMessageBox.warning(
			self, self.tr("Unsaved Drawing"),
			self.tr("Save changes to %s before closing?") % tab.title,
			PySide6.QtWidgets.QMessageBox.StandardButton.Save
			| PySide6.QtWidgets.QMessageBox.StandardButton.Discard
			| PySide6.QtWidgets.QMessageBox.StandardButton.Cancel,
			PySide6.QtWidgets.QMessageBox.StandardButton.Cancel,
		)
		if choice is PySide6.QtWidgets.QMessageBox.StandardButton.Save:
			return ferrum_qt.ferrum.close_decision.CloseDecision.SAVE
		if choice is PySide6.QtWidgets.QMessageBox.StandardButton.Discard:
			return ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD
		return ferrum_qt.ferrum.close_decision.CloseDecision.KEEP_OPEN

	#============================================
	def _close_tab_at_with_decision(self, index: int,
			decision: ferrum_qt.ferrum.close_decision.CloseDecision,
			) -> ferrum_qt.ferrum.close_decision.CloseResult:
		"""Apply one explicit decision through every ordinary close lifecycle guard."""
		if type(decision) is not ferrum_qt.ferrum.close_decision.CloseDecision:
			raise TypeError("Ferrum tab close requires an exact CloseDecision")
		page = self._tab_widget.widget(index)
		tab = self._native_tabs_by_page.get(page)
		if tab is None:
			return ferrum_qt.ferrum.close_decision.CloseResult.NO_TAB
		if self._cancel_explicit_replacement_for_target_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.LOCAL_DOCUMENT_OPEN_CANCELLATION_REQUESTED
		if self._molecule_import_blocks_tab_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.MOLECULE_IMPORT_BLOCKED
		if self._molecule_export_blocks_tab_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.MOLECULE_EXPORT_BLOCKED
		if self._snapshot_export_blocks_tab_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.SNAPSHOT_EXPORT_BLOCKED
		if self._molecule_inspection_blocks_tab_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.MOLECULE_INSPECTION_BLOCKED
		if self._molecule_diagnostics_blocks_tab_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.MOLECULE_DIAGNOSTICS_BLOCKED
		if self._atom_oxidation_blocks_tab_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.ATOM_OXIDATION_BLOCKED
		if self._clipboard_operation_blocks_tab_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.CLIPBOARD_OPERATION_BLOCKED
		if self._coordinate_generation_blocks_tab_close(tab):
			return ferrum_qt.ferrum.close_decision.CloseResult.COORDINATE_GENERATION_BLOCKED
		try:
			self._template_catalog_controller.cancel_for_tab(tab, "tab_close")
		except ferrum_qt.ferrum.operation_leases.OperationLeaseError:
			return ferrum_qt.ferrum.close_decision.CloseResult.OPERATION_CANCELLATION_FAILED
		if tab.requires_refresh:
			return ferrum_qt.ferrum.close_decision.CloseResult.REFRESH_REQUIRED
		if tab.is_dirty:
			if decision is ferrum_qt.ferrum.close_decision.CloseDecision.KEEP_OPEN:
				return ferrum_qt.ferrum.close_decision.CloseResult.DIRTY_REQUIRES_DECISION
			if decision is ferrum_qt.ferrum.close_decision.CloseDecision.SAVE:
				self._tab_widget.setCurrentIndex(index)
				if not self._on_save() or tab.is_dirty:
					return ferrum_qt.ferrum.close_decision.CloseResult.SAVE_FAILED
			if decision is not ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD and tab.is_dirty:
				return ferrum_qt.ferrum.close_decision.CloseResult.DIRTY_REQUIRES_DECISION
		if self._atom_insertion_intent is not None and self._atom_insertion_intent.tab is tab:
			self._cancel_atom_insertion()
		if self._line_gesture_intent is not None and self._line_gesture_intent.tab is tab:
			if not self._cancel_line_gesture():
				return ferrum_qt.ferrum.close_decision.CloseResult.LINE_GESTURE_CANCELLATION_FAILED
		if getattr(self, "_structure_tab", None) is tab:
			self._cancel_structure_selection()
		if self._active_native_tab() is tab:
			self._window_mode_sync.cancel()
		if (
			self._direct_glycosidic_haworth_intent is not None
			and self._direct_glycosidic_haworth_intent.tab is tab
		):
			self._cancel_direct_glycosidic_haworth_intent()
		try:
			self._operation_leases.unregister_tab(tab)
		except ferrum_qt.ferrum.operation_leases.OperationLeaseError:
			return ferrum_qt.ferrum.close_decision.CloseResult.OPERATION_CANCELLATION_FAILED
		try:
			tab.dispose()
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
			self._operation_leases.bind_tab(tab)
			return ferrum_qt.ferrum.close_decision.CloseResult.DISPOSAL_FAILED
		self._close_molecule_report_dialog_for_tab(tab)
		self._close_molecule_diagnostics_dialog_for_tab(tab)
		self._close_atom_oxidation_dialog_for_tab(tab)
		self._cancel_native_view_controls_for_tab(tab)
		self._dispose_closed_native_tab(tab, index)
		self._refresh_actions()
		return ferrum_qt.ferrum.close_decision.CloseResult.CLOSED

	#============================================
	def _close_native_tab_at(self, index: int,
			decision: ferrum_qt.ferrum.close_decision.CloseDecision,
			) -> ferrum_qt.ferrum.close_decision.CloseResult:
		"""Apply one explicit close decision to the specified Ferrum page."""
		return self._close_tab_at_with_decision(index, decision)

	#============================================
	def _present_close_result_refusal(self,
			result: ferrum_qt.ferrum.close_decision.CloseResult,
			) -> None:
		"""Present one ordinary UI refusal after typed close application declines."""
		if type(result) is not ferrum_qt.ferrum.close_decision.CloseResult:
			raise TypeError("Ferrum close refusal requires an exact CloseResult")
		messages = {
			ferrum_qt.ferrum.close_decision.CloseResult.OPERATION_CANCELLATION_FAILED:
				"Ferrum could not cancel the active template placement; keep this tab open and retry cancellation before closing.",
			ferrum_qt.ferrum.close_decision.CloseResult.REFRESH_REQUIRED:
				"Refresh the authoritative Rust view before closing this tab.",
			ferrum_qt.ferrum.close_decision.CloseResult.LINE_GESTURE_CANCELLATION_FAILED:
				"Ferrum could not cancel the pending cyclohexane attachment; retry cancellation before closing.",
			ferrum_qt.ferrum.close_decision.CloseResult.DISPOSAL_FAILED:
				"Ferrum could not clear the active SMARTS result; refresh before closing this tab.",
		}
		message = messages.get(result)
		if message is not None:
			self._show_edit_refusal(self._typed_refusal("close_document", "busy_close", message))

	#============================================
	def _dispose_closed_native_tab(self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			index: int,
			) -> None:
		"""Transfer a disposed tab from the tab host to Qt deferred deletion."""
		self._tab_widget.removeTab(index)
		self._native_tabs_by_page.pop(tab)
		tab.hide()
		tab.setParent(None)
		tab.deleteLater()

	#============================================
	def _close_current_tab(self) -> None:
		"""Close the selected page through the same clean-tab guard."""
		index = self._tab_widget.currentIndex()
		if index >= 0:
			self._close_tab_at(index)

	#============================================
	@PySide6.QtCore.Slot(int)
	def _on_native_tab_changed(self, _index: int) -> None:
		"""Invalidate one outgoing SMARTS plan, then bind the incoming tab once."""
		controller = getattr(self, "_smarts_query_controller", None)
		if hasattr(self, "_native_tabs_by_page"):
			previous = self._last_native_tab
			current = self._active_native_tab()
			if previous is not None and previous is not current:
				self._window_mode_sync.cancel()
				invalidation_succeeded = True
				try:
					previous._require_live_smarts_invalidation_v1("tab_deactivated")
				except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
					invalidation_succeeded = False
					self._show_edit_refusal(self._typed_refusal(
						"close_document", "busy_close",
						"Ferrum could not clear the active SMARTS result; refresh before editing this tab.",
					))
				if controller is not None:
					controller._deactivate_after_tab_invalidation_v1(invalidation_succeeded)
			self._last_native_tab = current
			self._window_mode_sync.synchronize_native_input_viewport()
			if controller is not None:
				# A first registration has no outgoing tab, so it only binds its
				# already-published incoming plan and leaves the modeless dock alone.
				controller._activate_after_tab_switch_v1()
			self._on_native_view_tab_changed()
			self._refresh_actions()

	#============================================
	@PySide6.QtCore.Slot()
	def _on_native_selection_changed(self) -> None:
		"""Queue one settled action refresh after scene selection changes."""
		if self._native_selection_refresh_timer.isActive():
			return
		self._native_selection_refresh_timer.start(0)

	#============================================
	@PySide6.QtCore.Slot()
	def _refresh_after_native_selection(self) -> None:
		"""Refresh actions after a synchronous projection transition settles."""
		self._refresh_actions()

	#============================================
	def _refresh_actions(self, *_unused: object) -> None:
		"""Make Ferrum Save and Close reachability follow the selected page."""
		self._refresh_local_document_open_action()
		tab = self._active_native_tab()
		active = tab is not None and not tab.is_disposed
		pending = active and tab.requires_refresh
		busy_import = self._molecule_import_busy()
		busy_export = self._molecule_export_busy()
		busy_inspection = self._molecule_inspection_busy()
		busy_diagnostics = self._molecule_diagnostics_busy()
		busy_atom_oxidation = self._atom_oxidation_busy()
		busy_compact_group_materialization = self._compact_group_materialization_intent is not None
		busy_compact_group_authoring = self._compact_group_authoring_intent is not None
		busy_clipboard = self._clipboard_busy()
		busy_coordinates = self._coordinate_generation_intent is not None
		busy_catalog_template = self._operation_leases.has_active(
			ferrum_qt.ferrum.operation_leases.OperationFamily.TEMPLATE_CATALOG,
			tab=tab,
		)
		busy_snapshot_export = self._snapshot_export_busy()
		busy_without_catalog = (
			busy_import or busy_export or busy_inspection or busy_diagnostics or busy_atom_oxidation or busy_compact_group_materialization or busy_compact_group_authoring or busy_clipboard or busy_coordinates
			or busy_snapshot_export
		)
		busy = busy_without_catalog or busy_catalog_template
		# A template placement is itself a terminal authoring intent.  Keep ordinary
		# document commands protected, but leave the exclusive authoring actions
		# reachable so selecting one can cancel the template owner before it arms.
		authoring_busy = (
			busy_import or busy_export or busy_inspection or busy_diagnostics or busy_atom_oxidation or busy_compact_group_materialization or busy_compact_group_authoring or busy_clipboard
			or busy_coordinates or busy_snapshot_export
		)
		selection_busy = (
			busy_import or busy_compact_group_materialization
			or busy_compact_group_authoring or busy_clipboard or busy_coordinates
		)
		if self._atom_insertion_intent is not None and (
			not active or self._atom_insertion_intent.tab is not tab or busy
		):
			self._cancel_atom_insertion()
		if self._line_gesture_intent is not None and (
			not active
			or self._line_gesture_intent.tab is not tab
			or busy
		):
			self._cancel_line_gesture()
		if self._compact_group_authoring_intent is not None and not (
			active and self._compact_group_authoring_is_current(
				self._compact_group_authoring_intent,
			)
		):
			self._cancel_compact_group_authoring()
		self._save_action.setEnabled(active and not pending and not busy)
		self._save_as_action.setEnabled(active and not pending and not busy)
		self._refresh_recovery_export_action(active, pending, busy)
		self._refresh_snapshot_export_actions(active, pending, busy)
		self._close_action.setEnabled(active and not pending and not busy_without_catalog)
		self._change_element_action.setEnabled(
			active
			and not busy
			and ferrum_qt.ferrum.atom_element.
			can_change_selected_atom_element(tab),
		)
		self._edit_atom_properties_action.setEnabled(
			active and not pending and not busy and tab.has_one_selected_atom(),
		)
		self._set_atom_number_action.setEnabled(
			active and not pending and not busy and tab.has_one_selected_atom(),
		)
		self._clear_atom_number_action.setEnabled(
			active and not pending and not busy and tab.selected_atom_has_number(),
		)
		can_mark = active and not pending and not busy and tab.has_one_selected_atom()
		for action in self._atom_mark_actions.values():
			action.setEnabled(can_mark)
		self._remove_atom_mark_action.setEnabled(
			can_mark and tab.selected_atom_has_marks(),
		)
		self._edit_bond_properties_action.setEnabled(
			active and not pending and not busy and tab.has_one_selected_bond(),
		)
		self._reverse_selected_wedge_direction_action.setEnabled(
			active
			and not pending
			and not busy
			and tab.can_reverse_selected_wedge_direction(),
		)
		native_paper_properties.refresh_paper_properties_action(
			self._paper_properties_action, active, pending, busy,
		)
		native_drawing_standard.refresh_drawing_standard_action(
			self._drawing_standard_action, active, pending, busy,
		)
		ferrum_qt.ferrum.presentation_properties.refresh_plus_properties_action(
			self._edit_plus_properties_action, tab, active, pending, busy,
		)
		ferrum_qt.ferrum.arrow_properties.refresh_arrow_properties_action(
			self._edit_arrow_properties_action, tab, active, pending, busy,
		)
		native_geometric_properties.refresh_geometric_properties_action(
			self._edit_geometric_properties_action, tab, active, pending, busy,
		)
		native_wavy_properties.refresh_wavy_properties_action(
			self._edit_wavy_properties_action, tab, active, pending, busy,
		)
		self._delete_atom_action.setEnabled(active and not pending and not busy)
		self._delete_bond_action.setEnabled(active and not pending and not busy)
		self._change_bond_order_action.setEnabled(active and not pending and not busy)
		self._undo_action.setEnabled(
			active and not pending and not busy and tab.can_undo(),
		)
		self._redo_action.setEnabled(
			active and not pending and not busy and tab.can_redo(),
		)
		can_add_atom = (
			active and not pending and not authoring_busy
			and bool(tab.durable_molecule_choices())
		)
		self._add_atom_action.setEnabled(can_add_atom)
		self._add_atom_action.setToolTip(self.tr(
			"Use Next atom, then click the canvas once; Esc cancels"
			if can_add_atom else
			"Requires an active document with a durable Rust molecule",
		))
		self._add_single_bond_action.setEnabled(active and not pending and not busy)
		self._refresh_line_tool_actions(active and not pending and not authoring_busy)
		self._refresh_structure_selection_action(
			active and not pending and not selection_busy,
		)
		self._refresh_top_level_transform_actions(tab, active, pending, authoring_busy)
		self._refresh_action.setEnabled(pending)
		self._refresh_molecule_import_actions(
			active, pending, busy_coordinates or busy_clipboard,
		)
		self._refresh_molfile_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_sdf_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_multi_sdf_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_molecule_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_molecule_inspection_actions(
			active,
			pending,
			busy_import or busy_export or busy_diagnostics or busy_coordinates or busy_clipboard,
		)
		self._refresh_molecule_diagnostics_action(
			active,
			pending,
			busy_import or busy_export or busy_inspection or busy_coordinates or busy_clipboard,
		)
		self._refresh_atom_oxidation_action(
			active, pending, busy_import or busy_export or busy_inspection or busy_diagnostics or busy_clipboard
			or busy_coordinates,
		)
		self._refresh_explicit_hydrogen_action(
			active, pending, busy_import or busy_export or busy_inspection or busy_diagnostics or busy_clipboard
			or busy_coordinates,
		)
		self._refresh_compact_group_materialization_action(
			active, pending, busy_import or busy_export or busy_inspection or busy_diagnostics or busy_clipboard
			or busy_coordinates or busy_atom_oxidation,
		)
		self._refresh_compact_group_authoring_action(
			active, pending, busy_import or busy_export or busy_inspection or busy_clipboard
			or busy_coordinates or busy_atom_oxidation or busy_compact_group_materialization,
		)
		self._refresh_native_clipboard_actions(
			active, pending,
			busy_import or busy_export or busy_inspection or busy_diagnostics or busy_coordinates,
		)
		self._refresh_molecule_name_action(active, pending, busy)
		self._refresh_linear_form_action(active, pending, busy)
		self._refresh_explicit_fragment_actions(active, pending, busy)
		self._refresh_direct_glycosidic_haworth_action(active, pending, busy)
		self._refresh_native_user_template_actions(
			active, pending,
			busy_import or busy_export or busy_inspection or busy_diagnostics or busy_clipboard or busy_coordinates,
		)
		self._template_catalog_controller.refresh_action(
			active, pending, busy_without_catalog,
		)
		self._generate_coordinates_action.setEnabled(
			active and not pending and not busy and bool(tab.durable_molecule_choices()),
		)
		self._cancel_coordinates_action.setEnabled(
			busy_coordinates
			and not self._coordinate_generation_intent.worker.delivery_cancelled,
		)
		self._refresh_view_controls_actions()
		self._native_property_dock.refresh(tab)
		controller = getattr(self, "_smarts_query_controller", None)
		if controller is not None:
			controller.refresh_action(active, pending, busy)

	#============================================
	def _show_edit_refusal(self, request: object) -> None:
		"""Present a typed author-facing refusal with separate diagnostic detail."""
		import ferrum_qt.ferrum.window_refusals
		import ferrum_qt.dialogs.refusal_presenter
		if type(request) is not ferrum_qt.dialogs.refusal_presenter.RefusalRequest:
			raise TypeError("Ferrum refusal presentation requires an exact RefusalRequest")
		ferrum_qt.ferrum.window_refusals.show_refusal(self, request)


	#============================================
	def _prepare_native_window_shutdown(self) -> bool:
		"""Settle every controller and retire each clean exact tab binding."""
		controller = getattr(self, "_smarts_query_controller", None)
		if controller is not None:
			controller.close()
		try:
			self._template_catalog_controller.cancel_active(reopen=False)
		except ferrum_qt.ferrum.operation_leases.OperationLeaseError:
			self._present_shutdown_refusal(
				"Ferrum could not cancel the active template placement; keep the window open.",
			)
			return False
		self._cancel_direct_glycosidic_haworth_intent()
		self._cancel_atom_insertion()
		self._window_mode_sync.cancel()
		if not self._cancel_line_gesture():
			self._present_shutdown_refusal(
				"Ferrum could not cancel the pending cyclohexane attachment; retry cancellation before closing.",
			)
			return False
		if self._cancel_local_document_open_for_close():
			self._present_shutdown_refusal(
				"Ferrum cancelled delivery; close again after Rust admission finishes.",
			)
			return False
		if self._cancel_molecule_imports_for_close():
			return False
		if self._cancel_molecule_export_for_close():
			return False
		if self._cancel_snapshot_export_for_close():
			return False
		if self._cancel_molecule_inspection_for_close():
			return False
		if self._cancel_molecule_diagnostics_for_close():
			return False
		if self._cancel_clipboard_operations_for_close():
			return False
		if self._coordinate_generation_intent is not None:
			self._cancel_coordinate_generation()
			self._present_shutdown_refusal(
				"Ferrum cancelled delivery; close again after the current operation finishes.",
			)
			return False
		if any(tab.requires_refresh for tab in self._native_tabs_by_page.values()):
			self._present_shutdown_refusal(
				"Refresh every pending authoritative Rust view before closing Ferrum.",
			)
			return False
		if any(tab.is_dirty for tab in self._native_tabs_by_page.values()):
			self._present_shutdown_refusal(
				"Save or discard every Ferrum document before closing Ferrum.",
			)
			return False
		self._prepare_native_view_controls_shutdown()
		for tab in tuple(self._native_tabs_by_page.values()):
			index = self._tab_widget.indexOf(tab)
			if index >= 0:
				result = self._close_tab_at_with_decision(
					index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
				)
				if result is not ferrum_qt.ferrum.close_decision.CloseResult.CLOSED:
					self._present_close_result_refusal(result)
					return False
		return not self._native_tabs_by_page

	#============================================
	def _present_shutdown_refusal(self, message: str) -> None:
		"""Present one exact shutdown refusal while every tab remains reachable."""
		self._show_edit_refusal(self._typed_refusal(
			"close_document", "busy_close", message,
		))

	#============================================
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Use the same lifecycle settlement contract for a base-host close."""
		if not self._prepare_native_window_shutdown():
			event.ignore()
			return
		event.accept()
