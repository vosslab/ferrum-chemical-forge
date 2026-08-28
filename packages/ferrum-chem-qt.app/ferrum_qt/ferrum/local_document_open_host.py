"""Qt host transaction for Local Open publication and replacement."""

# Standard Library
from collections.abc import Callable

# PIP3 modules
import PySide6.QtCore

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.local_document_open_contract as local_open_contract
import ferrum_qt.ferrum.operation_leases


#============================================
class FerrumNativeLocalDocumentOpenHostMixin:
	"""Own the Qt side of one closed Local Open host resolution."""

	#============================================
	def _publish_local_open_tab(
			self, tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			resolution: local_open_contract.LocalOpenPublicationResolution,
			) -> None:
		"""Commit a candidate or explicitly return it after complete rollback."""
		published = False
		bound = False
		try:
			self._require_unregistered_native_tab(tab)
			self._operation_leases.bind_tab(tab)
			bound = True
			with PySide6.QtCore.QSignalBlocker(self._tab_widget):
				index = self._tab_widget.addTab(tab, tab.title)
				self._finish_native_tab_registration(tab)
			published = True
		finally:
			if not published:
				returned = False
				try:
					if bound:
						self._unpublish_local_open_candidate(tab)
					returned = True
				finally:
					if returned:
						resolution.refuse_publication()
		receipt = local_open_contract.LocalOpenNewTabPublicationReceipt(tab, index)
		resolution.accept_publication(receipt)

	#============================================
	def _finish_local_open_publication(
			self, receipt: local_open_contract.LocalOpenNewTabPublicationReceipt,
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
			raise local_open_contract.LocalOpenPostCommitPresentationError(
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
			self, receipt: local_open_contract.LocalOpenNewTabPublicationReceipt,
			) -> None:
		"""Require the exact new tab and index committed by Local Open."""
		if type(receipt) is not local_open_contract.LocalOpenNewTabPublicationReceipt:
			raise TypeError("Ferrum Local Open completion requires its publication receipt")
		if self._native_tabs_by_page.get(receipt.tab) is not receipt.tab:
			raise RuntimeError("Ferrum Local Open committed tab is no longer registered")
		if self._tab_widget.indexOf(receipt.tab) != receipt.index:
			raise RuntimeError("Ferrum Local Open committed tab moved before presentation")

	#============================================
	def _commit_local_open_replacement(
			self, old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, index: int,
			capability: ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability,
			lease: ferrum_qt.ferrum.operation_leases.OperationLease,
			resolution: local_open_contract.LocalOpenReplacementResolution,
			) -> None:
		"""Commit a replacement or explicitly return its candidate after rollback."""
		validated = False
		try:
			self._validate_local_open_replacement(old, new, index, capability, lease)
			validated = True
		finally:
			if not validated:
				resolution.refuse_replacement()
		self._publish_provisional_replacement_candidate(new, index, resolution)
		prepared = self._prepare_local_open_source(capability, lease, old, new, resolution)
		self._validate_local_open_irreversible_close(prepared, old, new, index, resolution)
		self._dispose_or_restore_local_open_source(prepared, old, new, resolution)
		# Old-tab disposal is the irreversible ownership boundary.  Every later
		# failure leaves the unresolved candidate conservatively host-owned.
		self._close_local_open_replacement(old, index)
		observer = self._local_open_replacement_resolution_observer(
			old, new, index, resolution,
		)
		self._operation_leases.complete_prepared_terminal_replacement(prepared, observer)

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
			resolution: local_open_contract.LocalOpenReplacementResolution,
			) -> None:
		"""Integrate a recoverable candidate or return it after complete rollback."""
		self._operation_leases.bind_tab(new)
		published = False
		try:
			with PySide6.QtCore.QSignalBlocker(self._tab_widget):
				self._tab_widget.insertTab(index, new, new.title)
				self._finish_native_tab_registration(new)
			published = True
		finally:
			if not published:
				returned = False
				try:
					self._unpublish_local_open_candidate(new)
					returned = True
				finally:
					if returned:
						resolution.refuse_replacement()

	#============================================
	def _prepare_local_open_source(
			self, capability: ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability,
			lease: ferrum_qt.ferrum.operation_leases.OperationLease,
			old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			resolution: local_open_contract.LocalOpenReplacementResolution,
			) -> ferrum_qt.ferrum.operation_leases.PreparedTerminalReplacement:
		"""Prepare the source or return the candidate after complete rollback."""
		prepared = False
		try:
			replacement = self._operation_leases.prepare_terminal_replacement(
				capability, lease, old,
			)
			prepared = True
			return replacement
		finally:
			if not prepared:
				returned = False
				try:
					self._unpublish_local_open_candidate(new)
					returned = True
				finally:
					if returned:
						resolution.refuse_replacement()

	#============================================
	def _dispose_or_restore_local_open_source(
			self, prepared: ferrum_qt.ferrum.operation_leases.PreparedTerminalReplacement,
			old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			resolution: local_open_contract.LocalOpenReplacementResolution,
			) -> None:
		"""Dispose the source or return the candidate after complete rollback."""
		disposed = False
		try:
			old.dispose()
			disposed = True
		finally:
			if not disposed:
				returned = False
				try:
					self._operation_leases.restore_prepared_terminal_replacement(prepared, old)
					self._unpublish_local_open_candidate(new)
					returned = True
				finally:
					if returned:
						resolution.refuse_replacement()

	#============================================
	def _validate_local_open_irreversible_close(
			self, prepared: ferrum_qt.ferrum.operation_leases.PreparedTerminalReplacement,
			old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, index: int,
			resolution: local_open_contract.LocalOpenReplacementResolution,
			) -> None:
		"""Prove the closed swap span or return the candidate after rollback."""
		validated = False
		try:
			if self._native_tabs_by_page.get(old) is not old:
				raise RuntimeError("Ferrum Local Open replacement lost its old tab mapping")
			if self._native_tabs_by_page.get(new) is not new:
				raise RuntimeError("Ferrum Local Open replacement lost its new tab mapping")
			if self._tab_widget.indexOf(new) != index:
				raise RuntimeError("Ferrum Local Open replacement moved its new tab")
			if self._tab_widget.indexOf(old) != index + 1:
				raise RuntimeError("Ferrum Local Open replacement moved its old tab")
			validated = True
		finally:
			if not validated:
				returned = False
				try:
					self._operation_leases.restore_prepared_terminal_replacement(prepared, old)
					self._unpublish_local_open_candidate(new)
					returned = True
				finally:
					if returned:
						resolution.refuse_replacement()

	#============================================
	def _close_local_open_replacement(
			self, old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			index: int,
			) -> None:
		"""Run the prevalidated no-callback removal span after source disposal."""
		# Signals stay blocked from the deterministic index removal through map removal.
		# No Qt callback can observe an intermediate old/new registration state.
		with PySide6.QtCore.QSignalBlocker(self._tab_widget):
			self._tab_widget.removeTab(index + 1)
			del self._native_tabs_by_page[old]

	#============================================
	def _local_open_replacement_resolution_observer(
			self, old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, index: int,
			resolution: local_open_contract.LocalOpenReplacementResolution,
			) -> Callable[[ferrum_qt.ferrum.operation_leases.OperationLease], None]:
		"""Accept the exact receipt only after the source registry is settled."""
		def observe(settled: ferrum_qt.ferrum.operation_leases.OperationLease) -> None:
			receipt = local_open_contract.LocalOpenReplacementCommitReceipt(
				old, new, index, settled.lease_id, settled.tab_identity,
			)
			resolution.accept_replacement(receipt)
		return observe

	#============================================
	def _finish_local_open_replacement(
			self, receipt: local_open_contract.LocalOpenReplacementCommitReceipt,
			) -> None:
		"""Apply presentation cleanup after an already-settled Local Open commit."""
		if type(receipt) is not local_open_contract.LocalOpenReplacementCommitReceipt:
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
			raise local_open_contract.LocalOpenPostCommitPresentationError(
				"Ferrum could not refresh the committed Local Open tab",
			) from error

	#============================================
	def _unpublish_local_open_candidate(
			self, tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Undo provisional integration before returning candidate ownership."""
		self._operation_leases.unregister_tab(tab)
		self._remove_provisional_native_tab_integrations(tab)
		with PySide6.QtCore.QSignalBlocker(self._tab_widget):
			index = self._tab_widget.indexOf(tab)
			if index >= 0:
				self._tab_widget.removeTab(index)
			self._native_tabs_by_page.pop(tab, None)
			tab.hide()
			tab.setParent(None)
