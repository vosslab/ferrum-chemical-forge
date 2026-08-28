"""One Qt-thread delivery boundary for one immutable Local Open intent."""

# Standard Library
import enum
import pathlib
from collections.abc import Callable

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.ferrum.local_document_open_contract as local_open_contract
from ferrum_qt.ferrum.local_document_open_types import FerrumNativeLocalDocumentOpenFailure
import ferrum_qt.ferrum.operation_leases


#============================================
class _LocalDocumentOpenWorkerRelay(PySide6.QtCore.QObject):
	"""Queued Qt receiver that preserves worker sender identity."""

	#============================================
	def __init__(
			self, delivery: "LocalDocumentOpenDelivery",
			parent: PySide6.QtCore.QObject,
			) -> None:
		"""Create the one relay parented to the controller.

		Args:
			delivery: Per-intent Qt-thread delivery owner.
			parent: Controller whose teardown must disconnect queued calls.
		"""
		super().__init__(parent)
		self._delivery = delivery

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_prepared(self, prepared: object) -> None:
		"""Stage one prepared receipt without installing it."""
		self._delivery.stage_prepared(self.sender(), prepared)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Stage one typed failure without presenting it."""
		self._delivery.stage_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Enter the controller's sole terminal worker-finish boundary."""
		self._delivery.finish(self.sender())


#============================================
class _HostResolutionState(enum.Enum):
	"""Keep one host publication or replacement result visibly one-shot."""

	PENDING = enum.auto()
	REFUSED = enum.auto()
	COMMITTED = enum.auto()


#============================================
class _AdmittedCandidateTransaction(
		local_open_contract.LocalOpenPublicationResolution,
		local_open_contract.LocalOpenReplacementResolution,
		):
	"""Resolve one candidate without guessing whether a host adopted it.

	An unresolved host exit leaves ownership uncertain and therefore never permits
	delivery-side disposal.  A host refusal permits disposal only after rollback.
	"""

	#============================================
	def __init__(self, delivery: "LocalDocumentOpenDelivery") -> None:
		"""Create an empty transaction for one admitted Rust receipt."""
		self._delivery = delivery
		self._candidate: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None = None
		self._host_resolution_state = _HostResolutionState.PENDING
		self._publication_receipt: local_open_contract.LocalOpenNewTabPublicationReceipt | None = None
		self._replacement_receipt: local_open_contract.LocalOpenReplacementCommitReceipt | None = None
		self._replacement_old: object | None = None
		self._replacement_index: int | None = None
		self._ownership_uncertain = False
		self.transferred = False

	#============================================
	def build(
			self, session: object, observation: object, source_kind: str,
			origin_token: object,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Create and retain a candidate before its origin adoption can fail."""
		candidate = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_admitted_local_open(
			session, pathlib.Path(self._delivery.intent.path).name, observation,
			self._delivery._host.palette(),
		)
		self._candidate = candidate
		candidate._adopt_local_document_origin(
			self._delivery.intent.path, source_kind, origin_token,
			local_document_origin_display_name(self._delivery.intent, source_kind),
		)
		return candidate

	#============================================
	def publish_new_tab(self) -> local_open_contract.LocalOpenNewTabPublicationReceipt:
		"""Ask the host to publish and require one closed resolution."""
		candidate = self._require_candidate()
		try:
			self._delivery._host.publish_open_tab(candidate, self)
		finally:
			if self._host_resolution_state is _HostResolutionState.PENDING:
				self._ownership_uncertain = True
		if self._host_resolution_state is _HostResolutionState.PENDING:
			raise RuntimeError("Ferrum Local Open publication returned without resolution")
		if self._host_resolution_state is _HostResolutionState.REFUSED:
			raise RuntimeError("Ferrum Local Open publication returned after refusal")
		receipt = self._publication_receipt
		if receipt is None:
			raise RuntimeError("Ferrum Local Open publication committed another receipt")
		return receipt

	#============================================
	def replace_open_tab(
			self, old: object, index: int,
			) -> local_open_contract.LocalOpenReplacementCommitReceipt:
		"""Ask the host to replace and require one closed resolution."""
		if index < 0:
			raise ValueError("Ferrum Open replacement target is no longer registered")
		self._replacement_old = old
		self._replacement_index = index
		try:
			self._delivery._host.commit_open_replacement(
				old, self._require_candidate(), index, self._delivery._capability,
				self._delivery.intent.lease, self,
			)
		finally:
			if self._host_resolution_state is _HostResolutionState.PENDING:
				self._ownership_uncertain = True
		if self._host_resolution_state is _HostResolutionState.PENDING:
			raise RuntimeError("Ferrum Local Open replacement returned without resolution")
		if self._host_resolution_state is _HostResolutionState.REFUSED:
			raise RuntimeError("Ferrum Local Open replacement returned after refusal")
		receipt = self._replacement_receipt
		if receipt is None:
			raise RuntimeError("Ferrum Local Open replacement committed another receipt")
		return receipt

	#============================================
	def accept_publication(
			self, receipt: local_open_contract.LocalOpenNewTabPublicationReceipt,
			) -> None:
		"""Accept one exact new-tab publication before receipt validation."""
		self._resolve_committed(replacement=False)
		self._publication_receipt = receipt
		self._validate_new_tab_receipt(receipt, self._require_candidate())

	#============================================
	def refuse_publication(self) -> None:
		"""Accept returned ownership after complete publication rollback."""
		self._resolve_refused()

	#============================================
	def accept_replacement(
			self, receipt: local_open_contract.LocalOpenReplacementCommitReceipt,
			) -> None:
		"""Accept one exact replacement commit before receipt validation."""
		self._resolve_committed(replacement=True)
		self._replacement_receipt = receipt
		self._validate_replacement_receipt(receipt)

	#============================================
	def refuse_replacement(self) -> None:
		"""Accept returned ownership after complete replacement rollback."""
		self._resolve_refused()

	#============================================
	def dispose_uncommitted(self) -> None:
		"""Dispose the candidate only while this transaction has certain ownership."""
		if (
				self._candidate is None
				or self.transferred
				or self._ownership_uncertain
				or self._candidate.is_disposed
			):
			return
		self._candidate.dispose()

	#============================================
	def _resolve_committed(self, *, replacement: bool) -> None:
		"""Transfer ownership before receipt validation can report a contract fault."""
		if self._host_resolution_state is not _HostResolutionState.PENDING:
			raise RuntimeError("Ferrum Local Open host resolution was reused")
		self._host_resolution_state = _HostResolutionState.COMMITTED
		self.transferred = True
		self._delivery.outcome = local_open_contract.LocalDocumentOpenOutcome.COMPLETED
		if replacement:
			self._delivery.replacement_lease_settled = True

	#============================================
	def _resolve_refused(self) -> None:
		"""Keep candidate ownership after the host confirms complete rollback."""
		if self._host_resolution_state is not _HostResolutionState.PENDING:
			raise RuntimeError("Ferrum Local Open host resolution was reused")
		self._host_resolution_state = _HostResolutionState.REFUSED

	#============================================
	def _require_candidate(self) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Return the candidate after successful construction."""
		if self._candidate is None:
			raise RuntimeError("Ferrum Open candidate has not been built")
		return self._candidate

	#============================================
	def _validate_new_tab_receipt(
			self, receipt: object,
			candidate: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		"""Require exact candidate identity after terminal ownership is recorded."""
		if type(receipt) is not local_open_contract.LocalOpenNewTabPublicationReceipt:
			raise TypeError("Ferrum Local Open publication did not return its receipt")
		if receipt.tab is not candidate or receipt.index < 0:
			raise RuntimeError("Ferrum Local Open publication returned another candidate")
		if not self._delivery._host.tab_is_registered(candidate):
			raise RuntimeError("Ferrum Local Open candidate was not registered")
		if self._delivery._host.tab_widget_index(candidate) != receipt.index:
			raise RuntimeError("Ferrum Local Open candidate moved before transfer")

	#============================================
	def _validate_replacement_receipt(self, receipt: object) -> None:
		"""Require exact swap facts after delivery records its completed terminal truth."""
		candidate = self._require_candidate()
		lease = self._delivery.intent.lease
		if type(receipt) is not local_open_contract.LocalOpenReplacementCommitReceipt:
			raise TypeError("Ferrum Local Open replacement did not return its receipt")
		if receipt.old is not self._replacement_old or receipt.new is not candidate:
			raise RuntimeError("Ferrum Local Open replacement returned another tab swap")
		if receipt.index != self._replacement_index:
			raise RuntimeError("Ferrum Local Open replacement returned another tab position")
		if receipt.lease_id != lease.lease_id or receipt.tab_identity != lease.tab_identity:
			raise RuntimeError("Ferrum Local Open replacement returned another source lease")
		if not self._delivery._host.tab_is_registered(candidate):
			raise RuntimeError("Ferrum Local Open replacement did not register its candidate")
		if self._delivery._host.tab_widget_index(candidate) != receipt.index:
			raise RuntimeError("Ferrum Local Open replacement moved its candidate")


#============================================
class LocalDocumentOpenDelivery:
	"""Own staged worker facts and the one finish-only delivery result."""

	#============================================
	def __init__(
			self, host: local_open_contract.LocalDocumentOpenHost,
			registry: object, capability: object, parent: PySide6.QtCore.QObject,
			finish_callback: Callable[["LocalDocumentOpenDelivery"], None],
			present_refusal: Callable[[ferrum_qt.dialogs.refusal_presenter.RefusalRequest], None],
			intent: local_open_contract.LocalDocumentOpenIntent,
			) -> None:
		"""Bind one immutable intent to its worker relay.

		Args:
			host: Explicit window port for installation and presentation.
			registry: Exact source-tab operation lease registry.
			capability: Local Open family capability issued by the registry.
			parent: Controller QObject that owns the relay lifetime.
			finish_callback: Explicit controller terminal callback for this delivery.
			present_refusal: Explicit presentation callback bound to this intent.
			intent: Exact request, source lease, and worker being delivered.
		"""
		self.intent = intent
		self._host = host
		self._registry = registry
		self._capability = capability
		self._finish_callback = finish_callback
		self._present_refusal = present_refusal
		self.relay = _LocalDocumentOpenWorkerRelay(self, parent)
		self.prepared: object | None = None
		self.failure: object | None = None
		self.conflict: Exception | None = None
		self.outcome: local_open_contract.LocalDocumentOpenOutcome | None = None
		self.replacement_lease_settled = False
		self.retired = False

	#============================================
	def connect(self) -> None:
		"""Connect exact worker signals to queued named relay slots."""
		self.intent.worker.prepared.connect(
			self.relay.on_prepared, PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.intent.worker.failed.connect(
			self.relay.on_failed, PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.intent.worker.finished.connect(
			self.relay.on_finished, PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)

	#============================================
	def stage_prepared(self, worker: object, prepared: object) -> None:
		"""Retain only one current-worker prepared fact."""
		if worker is not self.intent.worker or self.intent.worker.delivery_cancelled:
			return
		if self.prepared is not None or self.failure is not None or self.conflict is not None:
			self.conflict = RuntimeError(
				"Ferrum Open worker delivered more than one admission result",
			)
			return
		self.prepared = prepared

	#============================================
	def stage_failed(self, worker: object, failure: object) -> None:
		"""Retain only one current-worker typed failure fact."""
		if worker is not self.intent.worker or self.intent.worker.delivery_cancelled:
			return
		if self.prepared is not None or self.failure is not None or self.conflict is not None:
			self.conflict = RuntimeError(
				"Ferrum Open worker delivered more than one admission result",
			)
			return
		if type(failure) is not FerrumNativeLocalDocumentOpenFailure:
			self.conflict = TypeError("Ferrum Open worker delivered an invalid failure payload")
			return
		self.failure = failure

	#============================================
	def finish(self, worker: object) -> None:
		"""Give the exact stopped worker to the controller's terminal owner."""
		if worker is self.intent.worker and not self.retired:
			self._finish_callback(self)

	#============================================
	def retire(self) -> None:
		"""Delete stopped Qt objects after the controller settles the source lease."""
		if self.retired:
			return
		self.retired = True
		self.intent.worker.deleteLater()
		self.relay.deleteLater()

	#============================================
	def deliver(self) -> None:
		"""Present and install the staged result only at worker finish."""
		if self.intent.worker.delivery_cancelled:
			return
		if self.conflict is not None:
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.FAILED
			raise self.conflict
		if self.failure is not None:
			self._present_failure(self.failure)
			return
		if self.prepared is None:
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.FAILED
			self._host.show_status(
				self._host.translate("Open did not return an admission result."), 5000,
			)
			return
		if type(self.prepared) is not engine.PreparedLocalDocumentOpenV2:
			raise TypeError("Ferrum returned an invalid local Open receipt")
		self._deliver_prepared(self.prepared)

	#============================================
	def _delivery_is_live(self) -> bool:
		"""Require the exact active source lease before any nested delivery step."""
		if self.intent.worker.delivery_cancelled:
			return False
		return any(
			lease.lease_id == self.intent.lease.lease_id
			and lease.state is ferrum_qt.ferrum.operation_leases.LeaseState.ACTIVE
			for lease in self._registry.active_for_tab(self.intent.source)
		)

	#============================================
	def _deliver_prepared(self, prepared: engine.PreparedLocalDocumentOpenV2) -> None:
		"""Install a prepared Rust receipt through exact current source fences."""
		session, observation, origin_token, source_kind, summary = prepared.take_admission_v2()
		if (
			self.intent.disposition
			is local_open_contract.LocalDocumentOpenDisposition.REPLACE_EXPLICIT_CURRENT_TARGET
		):
			self._deliver_explicit(session, observation, origin_token, source_kind, summary)
			return
		existing = self._host.native_tab_for_origin_token(origin_token)
		if existing is not None:
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.COMPLETED
			if self._host.tab_widget_current() is self.intent.source:
				self._host.tab_widget_set_current_index(self._host.tab_widget_index(existing))
			self._host.record_recent_success(self.intent.path)
			return
		transaction = _AdmittedCandidateTransaction(self)
		installed = False
		try:
			transaction.build(session, observation, source_kind, origin_token)
			self._publish_standard_candidate(transaction)
			installed = True
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.FAILED
			self._report_installation_failure()
		finally:
			transaction.dispose_uncommitted()
		if installed:
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.COMPLETED
			self._host.record_recent_success(self.intent.path)
			self._host.show_status(
				self._host.translate(local_document_open_success(self.intent, summary)), 3000,
			)

	#============================================
	def _can_replace_pristine(self) -> bool:
		"""Revalidate the initial placeholder directly before replacement."""
		target = self.intent.target
		return (
			getattr(self.intent.descriptor, "allows_current_tab_replacement", False)
			and self.intent.disposition
			is local_open_contract.LocalDocumentOpenDisposition.REPLACE_PRISTINE_TARGET
			and target is not None
			and self._host.tab_is_registered(target)
			and self._host.tab_widget_current() is target
			and target.is_pristine_initial_placeholder()
			and target.current_snapshot.revision == self.intent.target_revision
			and target.current_snapshot.digest == self.intent.target_digest
			and self.intent.target_canvas_idle
			and not self._host.tab_has_active_canvas_interaction(target)
		)

	#============================================
	def _publish_standard_candidate(self, transaction: _AdmittedCandidateTransaction) -> None:
		"""Publish a standard candidate through its captured delivery policy."""
		if self._can_replace_pristine():
			self._finish_replacement(
				transaction.replace_open_tab(
					self.intent.target,
					self._host.tab_widget_index(self.intent.target),
				),
			)
			return
		receipt = transaction.publish_new_tab()
		self._finish_new_tab_publication(receipt, self._activate_new_tab())

	#============================================
	def _activate_new_tab(self) -> bool:
		"""Keep author focus when the captured source is no longer current."""
		return self._host.tab_widget_current() is self.intent.source and (
			self.intent.focus_target is None or (
				self.intent.activate_if_still_current
				and self._host.tab_widget_current() is self.intent.focus_target
			)
		)

	#============================================
	def _finish_replacement(self, receipt: object) -> None:
		"""Present a committed replacement without changing its terminal truth."""
		try:
			self._host.finish_open_replacement(receipt)
		except local_open_contract.LocalOpenPostCommitPresentationError:
			self._report_postcommit_failure()

	#============================================
	def _finish_new_tab_publication(
			self, receipt: local_open_contract.LocalOpenNewTabPublicationReceipt,
			activate: bool,
			) -> None:
		"""Present a committed new tab without changing its terminal truth."""
		try:
			self._host.finish_open_publication(receipt, activate)
		except local_open_contract.LocalOpenPostCommitPresentationError:
			self._report_postcommit_failure()

	#============================================
	def _fence_holds(self, fence: local_open_contract.ExplicitReplacementFence) -> bool:
		"""Check the exact populated-tab fence before every irreversible step."""
		target = fence.target
		if (
			self._host.shutdown_prepared()
			or not self._host.tab_is_registered(target)
			or self._host.tab_widget_current() is not target
			or self._host.tab_widget_index(target) != fence.index
			or target.is_disposed
			or target.requires_refresh
			or self._host.tab_has_active_canvas_interaction(target)
			or self._host.tab_has_conflict_except_lease(target, self.intent.lease)
		):
			return False
		snapshot = target.current_snapshot
		return (
			snapshot.revision == fence.revision
			and snapshot.digest == fence.digest
			and target.is_dirty == fence.dirty
			and (None if target.file_path is None else str(target.file_path))
			== fence.file_path
			and target.local_document_origin_token == fence.origin_token
		)

	#============================================
	def _deliver_explicit(
			self, session: object, observation: object, origin_token: object,
			source_kind: str, summary: object | None,
			) -> None:
		"""Install only into the exact surviving explicit replacement target."""
		fence = self.intent.replacement_fence
		if not self._delivery_is_live():
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.CANCELLED
			return
		if fence is None or not self._fence_holds(fence):
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.REFUSED
			self._report_stale()
			return
		existing = self._host.native_tab_for_origin_token(origin_token)
		if existing is not None:
			# The receipt is already true before optional focus/recent/status presentation.
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.COMPLETED
			self._host.tab_widget_set_current_index(self._host.tab_widget_index(existing))
			self._host.record_recent_success(self.intent.path)
			self._host.show_status(
				self._host.translate(
					f'"{pathlib.Path(self.intent.path).name}" is already open.',
				), 3000,
			)
			return
		# The dialog may run a nested Qt loop, so verify both source lease and fence again.
		while fence.dirty:
			choice = self._confirm_dirty(fence)
			if not self._delivery_is_live():
				self.outcome = local_open_contract.LocalDocumentOpenOutcome.CANCELLED
				return
			if choice == "cancel":
				self.outcome = local_open_contract.LocalDocumentOpenOutcome.REFUSED
				return
			if choice == "save":
				fence = self._recapture_fence(fence.target)
				if not self._fence_holds(fence):
					self.outcome = local_open_contract.LocalDocumentOpenOutcome.REFUSED
					self._report_stale()
					return
				if not fence.dirty:
					break
				continue
			if choice == "replace" and self._fence_holds(fence):
				break
			if choice == "retry" and self._fence_holds(fence):
				continue
			if not self._fence_holds(fence):
				self.outcome = local_open_contract.LocalDocumentOpenOutcome.REFUSED
				self._report_stale()
				return
		transaction = _AdmittedCandidateTransaction(self)
		transferred = False
		try:
			transferred = self._commit_explicit_candidate(
				transaction, session, observation, source_kind, origin_token, fence,
			)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.FAILED
			self._report_installation_failure()
		finally:
			transaction.dispose_uncommitted()
		if transferred:
			self._host.record_recent_success(self.intent.path)
			self._host.show_status(
				self._host.translate(local_document_open_success(self.intent, summary)), 3000,
			)

	#============================================
	def _commit_explicit_candidate(
			self, transaction: _AdmittedCandidateTransaction, session: object,
			observation: object, source_kind: str, origin_token: object,
			fence: local_open_contract.ExplicitReplacementFence,
			) -> bool:
		"""Commit one candidate only while its source lease and fence survive."""
		if not self._delivery_is_live():
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.CANCELLED
			return False
		transaction.build(session, observation, source_kind, origin_token)
		if not self._delivery_is_live():
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.CANCELLED
			return False
		if not self._fence_holds(fence):
			self.outcome = local_open_contract.LocalDocumentOpenOutcome.REFUSED
			self._report_stale()
			return False
		self._finish_replacement(transaction.replace_open_tab(fence.target, fence.index))
		return transaction.transferred

	#============================================
	def _recapture_fence(
			self, target: object,
			) -> local_open_contract.ExplicitReplacementFence:
		"""Capture fresh post-save facts for the same explicit target."""
		snapshot = target.current_snapshot
		return local_open_contract.ExplicitReplacementFence(
			target, self._host.tab_widget_index(target), snapshot.revision,
			snapshot.digest, target.is_dirty,
			None if target.file_path is None else str(target.file_path),
			target.local_document_origin_token,
		)

	#============================================
	def _confirm_dirty(self, fence: local_open_contract.ExplicitReplacementFence) -> str:
		"""Ask whether to save, replace, or cancel the exact dirty target."""
		target = fence.target
		name = target.title if target.file_path is not None else "this untitled document"
		message = self._host.translate(
			f'Save changes to "{name}" before replacing it with '
			f'"{pathlib.Path(self.intent.path).name}"?',
		)
		box = PySide6.QtWidgets.QMessageBox(
			PySide6.QtWidgets.QMessageBox.Icon.Warning,
			self._host.translate("Replace Current Tab"), message,
			parent=self._host.parent,
		)
		save = box.addButton(
			self._host.translate("Save"),
			PySide6.QtWidgets.QMessageBox.ButtonRole.AcceptRole,
		)
		replace = box.addButton(
			self._host.translate("Replace"),
			PySide6.QtWidgets.QMessageBox.ButtonRole.DestructiveRole,
		)
		cancel = box.addButton(
			self._host.translate("Cancel"),
			PySide6.QtWidgets.QMessageBox.ButtonRole.RejectRole,
		)
		box.setDefaultButton(save)
		box.setEscapeButton(cancel)
		box.exec()
		if box.clickedButton() is replace:
			return "replace"
		if box.clickedButton() is not save:
			return "cancel"
		if target.file_path is None:
			return "save" if self._host.prompt_native_save(target, True) else "retry"
		return "save" if self._host.save_native_tab_to_path(target, str(target.file_path)) else "retry"

	#============================================
	def _present_failure(self, failure: object) -> None:
		"""Present a bounded worker failure without exposing backend internals."""
		self.outcome = local_open_contract.LocalDocumentOpenOutcome.FAILED
		if type(failure) is not FerrumNativeLocalDocumentOpenFailure:
			raise TypeError("Ferrum Open worker delivered an invalid failure payload")
		if self.intent.recent_request and self._host.handle_recent_failure(self.intent.path, failure):
			return
		_outcome, guidance = local_document_open_guidance(failure)
		if failure.stage == "source_policy" or failure.category == "source_rejected":
			outcome = ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SOURCE_NOT_ALLOWED
		else:
			outcome = ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.INVALID_DOCUMENT
		self._present_refusal(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
				outcome, pathlib.Path(self.intent.path).name, guidance,
			),
		)

	#============================================
	def _report_installation_failure(self) -> None:
		"""Present bounded candidate-installation recovery."""
		self._present_refusal(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.DOCUMENT_DISPLAY_FAILED,
				pathlib.Path(self.intent.path).name,
				"Ferrum could not display the opened drawing. The current document was "
				"left unchanged; check the file and try again.",
			),
		)

	#============================================
	def _report_postcommit_failure(self) -> None:
		"""Report refresh recovery after a truthful completed terminal commit."""
		self._present_refusal(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.DOCUMENT_DISPLAY_FAILED,
				pathlib.Path(self.intent.path).name,
				"The drawing was opened and is current, but its view refresh did not "
				"complete. Choose Refresh Authoritative View before continuing.",
			),
		)

	#============================================
	def _report_stale(self) -> None:
		"""Keep an explicit request refused rather than reanchoring it."""
		self._present_refusal(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNAVAILABLE_OPERATION,
				technical_details=(
					"Open in Current Tab did not replace the changed document; choose the "
					"command again."
				),
			),
		)


#============================================
def local_document_open_success(
		intent: local_open_contract.LocalDocumentOpenIntent, interchange_summary: object | None,
		) -> str:
	"""Describe one successfully delivered Rust-owned local document."""
	name = pathlib.Path(intent.path).name
	if interchange_summary is not None:
		return (
			f"Opened {interchange_summary.imported_record_count} interchange record(s): "
			f"{name}"
		)
	return f"Opened local document: {name}"


#============================================
def local_document_origin_display_name(
		intent: local_open_contract.LocalDocumentOpenIntent, receipt_source_kind: str,
		) -> str | None:
	"""Return a Rust-issued interchange display label without suffix reconstruction."""
	if receipt_source_kind != "interchange":
		return None
	display_name = intent.descriptor.display_name
	if type(display_name) is not str or not display_name:
		raise ValueError("Ferrum interchange Open descriptor lacks a display name")
	return display_name


#============================================
def local_document_open_guidance(
		failure: FerrumNativeLocalDocumentOpenFailure,
		) -> tuple[str, str]:
	"""Return bounded recovery language for a Rust-owned admission failure."""
	if failure.stage == "source_policy" or failure.category == "source_rejected":
		return "Local Source Rejected", "Choose a regular, non-symlink file from File/Open."
	if failure.stage == "bytes" or failure.category == "resource_limit":
		return "Local Document Resource Limit", "Choose a smaller supported document."
	if failure.stage == "utf8":
		return "Local Document Text Rejected", "Choose a UTF-8 supported document."
	return (
		"Local Document Rejected",
		"The current tab is unchanged; choose another File/Open document.",
	)
