"""Asynchronous Rust-owned local-document admission for Ferrum windows."""

# Standard Library
import collections
import os
import pathlib
import sys

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
from ferrum_qt.ferrum.local_document_open_types import (
	FerrumNativeLocalDocumentOpenWorker,
)

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.ferrum.local_document_open_contract as local_open_contract
import ferrum_qt.ferrum.local_document_open_delivery

_LocalDocumentOpenIntent = local_open_contract.LocalDocumentOpenIntent
_LocalDocumentOpenDisposition = local_open_contract.LocalDocumentOpenDisposition
_LocalDocumentOpenOutcome = local_open_contract.LocalDocumentOpenOutcome
_ExplicitReplacementFence = local_open_contract.ExplicitReplacementFence


#============================================
class _LocalDocumentOpenStartupTransaction:
	"""Own one not-yet-started worker, lease, intent, and delivery."""

	#============================================
	def __init__(
			self, controller: "LocalDocumentOpenController",
			request: local_open_contract.LocalDocumentOpenRequest,
			) -> None:
		"""Capture the one controller and immutable request for startup."""
		self._controller = controller
		self._request = request
		self.worker: FerrumNativeLocalDocumentOpenWorker | None = None
		self.lease: object | None = None
		self.intent: _LocalDocumentOpenIntent | None = None
		self.delivery: (
			ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery | None
		) = None
		self.started = False

	#============================================
	def start(self) -> None:
		"""Publish one complete startup transaction before starting its worker."""
		controller = self._controller
		self.worker = controller._create_local_document_open_worker(
			self._request.path, self._request.descriptor.route_handle,
		)
		if self.worker.parent() is None:
			self.worker.setParent(controller)
		self.lease = controller._operation_leases.acquire(
			controller._local_document_open_capability, tab=self._request.source,
			close_policy=ferrum_qt.ferrum.operation_leases.ClosePolicy.BLOCK_UNTIL_SETTLED,
		)
		self.intent = _LocalDocumentOpenIntent(self._request, self.lease, self.worker)
		self.delivery = ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery(
			controller._host, controller._operation_leases,
			controller._local_document_open_capability, controller,
			controller._finish_local_document_open_delivery, controller._present_refusal,
			self.intent,
		)
		self.delivery.connect()
		controller._local_document_open_intent = self.intent
		controller._local_document_open_delivery = self.delivery
		controller._host.show_status(controller.tr("Working: opening drawing..."), 0)
		controller._host.action_refresh()
		self.worker.start()
		self.started = True

	#============================================
	def abort(self) -> None:
		"""Settle and retire an incomplete startup without hiding its primary error."""
		original_error = sys.exception()
		try:
			if self.lease is not None:
				try:
					self._controller._operation_leases.settle(
						self._controller._local_document_open_capability, self.lease,
						ferrum_qt.ferrum.operation_leases.LeaseState.FAILED,
					)
				except ferrum_qt.ferrum.operation_leases.OperationLeaseError:
					if original_error is None:
						raise
		finally:
			self._controller._clear_current_local_document_open(self.intent)
			if self.delivery is not None:
				self.delivery.retire()
			elif self.worker is not None:
				self.worker.deleteLater()
			self._controller._host.action_refresh()


#============================================
class LocalDocumentOpenController(PySide6.QtCore.QObject):
	"""Own local-document Open without frontend document parsing."""

	#============================================
	def __init__(
			self, host: local_open_contract.LocalDocumentOpenHost, catalog: object, registry: object,
			) -> None:
		"""Own all local-document Open state for one exact Ferrum window.

		Args:
			host: Explicit window presentation and lifecycle callbacks.
			catalog: Rust-issued local-document route catalog.
			registry: Window-local Qt operation lease registry.
		"""
		if type(host) is not local_open_contract.LocalDocumentOpenHost:
			raise TypeError("Ferrum local Open requires an exact callback host")
		super().__init__(host.parent)
		self._host = host
		self._local_document_open_catalog = catalog
		self._operation_leases = registry
		self._local_document_open_capability = registry.register_family(
			ferrum_qt.ferrum.operation_leases.OperationFamily.LOCAL_DOCUMENT_OPEN,
		)
		self._initialize_local_document_open()

	#============================================
	def _initialize_local_document_open(self) -> None:
		"""Create the sole local-document Open intent and Qt-thread relay."""
		self._local_document_open_intent: _LocalDocumentOpenIntent | None = None
		self._local_document_open_queue: collections.deque[
			local_open_contract.LocalDocumentOpenRequest
		] = collections.deque()
		self._local_document_open_delivery: (
			ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery | None
		) = None
		self._local_document_open_batch_success = True

	#============================================
	def build_actions(self) -> None:
		"""Create and register the three stable File/Open actions."""
		self._open_action = PySide6.QtGui.QAction(self.tr("Open"), self)
		self._open_action.triggered.connect(self._on_open)
		self._register_action("file.open", self._open_action)
		self._build_open_in_current_tab_action()
		self._build_local_document_open_action()

	@property
	def open_action(self) -> PySide6.QtGui.QAction:
		"""Return the stable File/Open action."""
		return self._open_action

	@property
	def open_in_current_tab_action(self) -> PySide6.QtGui.QAction:
		"""Return the stable explicit replacement action."""
		return self._open_in_current_tab_action

	@property
	def cancel_open_action(self) -> PySide6.QtGui.QAction:
		"""Return the stable cancellation action."""
		return self._cancel_open_action

	def tr(self, text: str) -> str:
		"""Translate controller-owned author-facing text through its host port."""
		return self._host.translate(text)

	def _register_action(
			self, action_id: str, action: PySide6.QtGui.QAction, *,
			lifecycle: str = "static",
			) -> None:
		"""Publish one controller-owned action through its explicit host port."""
		self._host.register_action(action_id, action, lifecycle)

	def _present_refusal(self, request: ferrum_qt.dialogs.refusal_presenter.RefusalRequest) -> None:
		"""Present one typed refusal through the explicit host port."""
		self._host.present_refusal(request)

	def _unavailable_edit_refusal(
			self, details: str,
		) -> ferrum_qt.dialogs.refusal_presenter.RefusalRequest:
		"""Build a closed refusal without borrowing a window implementation."""
		return ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNAVAILABLE_OPERATION,
			technical_details=details,
		)

	#============================================
	def _build_local_document_open_action(self) -> PySide6.QtGui.QAction:
		"""Construct explicit cancellation next to the host-owned Open action."""
		self._open_action.setToolTip(self.tr(
			"Open a local Ferrum drawing or supported interchange document",
		))
		action = PySide6.QtGui.QAction(self.tr("Cancel Open"), self)
		action.setStatusTip(self.tr("Stop delivery and wait for the current Open to finish safely."))
		action.setToolTip(self.tr("Stop delivery and wait for the current Open to finish safely."))
		action.triggered.connect(self._cancel_local_document_open)
		self._cancel_open_action = action
		self._register_action("file.open.cancel", action, lifecycle="stateful-cancel")
		return action

	#============================================
	def _native_new_document_filter(self) -> str:
		"""Build ordinary File/Open solely from Rust-owned route descriptors."""
		filters = [
			f"{descriptor.display_name} ({' '.join('*' + suffix for suffix in descriptor.suffixes)})"
			for descriptor in self._local_document_open_catalog.descriptors
		]
		filters.append("All Files (*)")
		return ";;".join(filters)

	#============================================
	def _current_tab_replacement_filter(self) -> str:
		"""Expose only registry routes allowed to replace the current tab."""
		filters = [
			f"{descriptor.display_name} ({' '.join('*' + suffix for suffix in descriptor.suffixes)})"
			for descriptor in self._local_document_open_catalog.descriptors
			if descriptor.allows_current_tab_replacement
		]
		filters.append("All Files (*)")
		return ";;".join(filters)

	#============================================
	def _build_open_in_current_tab_action(self) -> PySide6.QtGui.QAction:
		"""Construct the deliberate populated-tab replacement command."""
		action = PySide6.QtGui.QAction(self.tr("Open in Current Tab..."), self)
		action.setShortcut(PySide6.QtGui.QKeySequence("Ctrl+Shift+O"))
		action.setStatusTip(self.tr("Open a Ferrum drawing in place of the current tab."))
		action.setToolTip(self.tr("Open a Ferrum drawing in place of the current tab."))
		action.triggered.connect(self._on_open_in_current_tab)
		self._open_in_current_tab_action = action
		self._register_action("file.open_current", action)
		return action

	#============================================
	def _on_open_in_current_tab(self) -> bool:
		"""Choose a source for one explicitly captured current Ferrum tab."""
		if not self._can_begin_explicit_current_replacement():
			return False
		path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self._host.parent, self.tr("Open Ferrum Chemical Drawing in Current Tab"), "",
			self.tr(self._current_tab_replacement_filter()),
		)[0]
		if not path:
			return False
		return self.open_in_current_tab_path(path)

	#============================================
	def open_in_current_tab_path(self, file_path: str) -> bool:
		"""Prepare one source without allowing explicit replacement to become NewTab.

		Args:
			file_path: Author-selected local path.

		Returns:
			Whether the exact source request was accepted for admission.
		"""
		if type(file_path) is not str:
			raise TypeError("Ferrum local-document Open requires an exact path string")
		if not self._can_begin_explicit_current_replacement():
			return False
		absolute_path = os.path.abspath(file_path)
		descriptor = self._local_document_open_catalog.replacement_descriptor_for_path(
			absolute_path,
		)
		if descriptor is None:
			self._show_unsupported_local_document(absolute_path)
			return False
		target = self._host.active_tab()
		fence = self._capture_explicit_replacement_fence(target)
		self._local_document_open_batch_success = True
		self._start_local_document_open(local_open_contract.LocalDocumentOpenRequest(
			absolute_path, descriptor, _LocalDocumentOpenDisposition.REPLACE_EXPLICIT_CURRENT_TARGET,
			target, fence.revision, fence.digest, True, target, target, True, False, fence,
		))
		return True

	#============================================
	def _can_begin_explicit_current_replacement(self) -> bool:
		"""Keep the command bound to a live idle current Ferrum tab."""
		tab = self._host.active_tab()
		if (
			self._local_document_open_intent is not None
			or self._host.snapshot_busy()
			or self._host.shutdown_prepared()
		):
			return False
		return self._explicit_replacement_target_is_admissible(tab)

	#============================================
	def _explicit_replacement_target_is_admissible(self, target: object) -> bool:
		"""Share one exact target lifecycle predicate across action, capture, and swap."""
		return (
			type(target) is ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
			and target is self._host.active_tab()
			and self._host.tab_is_registered(target)
			and not target.is_disposed
			and not target.requires_refresh
			and not self._host.tab_has_active_canvas_interaction(target)
			and not self._host.tab_has_active_operation(target)
		)

	#============================================
	def _capture_explicit_replacement_fence(
			self, target: object,
			) -> _ExplicitReplacementFence:
		"""Freeze the exact target facts before detached Rust admission begins."""
		if not self._explicit_replacement_target_is_admissible(target):
			raise ValueError("Open in Current Tab requires an idle current Ferrum document")
		snapshot = target.current_snapshot
		return _ExplicitReplacementFence(
			target, self._host.tab_widget_index(target), snapshot.revision, snapshot.digest,
			target.is_dirty,
			None if target.file_path is None else str(target.file_path),
			target.local_document_origin_token,
		)

	#============================================
	def _on_open(self) -> bool:
		"""Choose one bounded local drawing for Rust-owned admission."""
		if self._host.snapshot_busy():
			return False
		path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self._host.parent, self.tr("Open Ferrum Chemical Drawing"), "",
			self.tr(self._native_new_document_filter()),
		)[0]
		if not path:
			return False
		return self.open_file_path(path, interactive=True)

	#============================================
	def open_file_path(
			self, file_path: str, replace_current: bool = False, *,
			interactive: bool = False, force_new_tab: bool = False,
			recent_request: bool = False,
			) -> bool:
		"""Begin one profile-owned Rust admission into a Ferrum tab.

		Args:
			file_path: Author-selected local path.
			replace_current: Retained public request flag, rejected by this controller.
			interactive: Whether author focus may select a pristine replacement.
			force_new_tab: Whether current placeholder replacement is disallowed.
			recent_request: Whether recent-file recovery owns typed failures.

		Returns:
			Whether the request was accepted or queued.
		"""
		if type(file_path) is not str:
			raise TypeError("Ferrum local-document Open requires an exact path string")
		if self._host.snapshot_busy():
			return False
		if replace_current:
			self._present_refusal(self._unavailable_edit_refusal(
				"Ferrum drawings open in a new Ferrum tab.",
			))
			return False
		absolute_path = os.path.abspath(file_path)
		descriptor = self._local_document_open_catalog.descriptor_for_path(absolute_path)
		if descriptor is None:
			self._show_unsupported_local_document(absolute_path)
			return False
		source = self._host.active_tab()
		if (
			type(source) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
			or not self._host.tab_is_registered(source)
		):
			self._present_refusal(self._unavailable_edit_refusal(
				"Open requires a live Ferrum document tab.",
			))
			return False
		focus_target = self._host.active_tab() if interactive else None
		focus_busy = (
			focus_target is not None
			and self._host.tab_has_active_canvas_interaction(focus_target)
		)
		# File/Open is a terminal document command.  Capture the pre-cancellation
		# activity only to preserve its NewTab/focus policy, then close its
		# transient pointer owners before detached Rust admission can change tabs.
		if focus_busy:
			self._host.cancel_active_pointer_authoring()
		disposition = self._open_disposition_for_request(
			descriptor, interactive and not force_new_tab and not focus_busy,
		)
		target = (
			focus_target
			if disposition is _LocalDocumentOpenDisposition.REPLACE_PRISTINE_TARGET
			else None
		)
		target_revision, target_digest, target_canvas_idle = self._capture_pristine_target_fence(target)
		activate_if_still_current = not focus_busy
		request = local_open_contract.LocalDocumentOpenRequest(
			absolute_path, descriptor, disposition, target, target_revision, target_digest,
			target_canvas_idle, source, focus_target, activate_if_still_current, recent_request,
		)
		if self._local_document_open_intent is not None:
			if self._local_document_open_intent.path == absolute_path:
				return True
			if not any(queued.path == absolute_path for queued in self._local_document_open_queue):
				self._local_document_open_queue.append(request)
			self._host.show_status(self.tr("Queued Ferrum drawing Open request."), 3000)
			self._host.action_refresh()
			return True
		self._local_document_open_batch_success = True
		self._start_local_document_open(request)
		return True

	#============================================
	def open_recent_native_document_path(self, file_path: str) -> bool:
		"""Route a personal recent selection through the immutable NewTab policy."""
		return self.open_file_path(
			file_path, interactive=True, force_new_tab=True, recent_request=True,
		)

	#============================================
	def _show_unsupported_local_document(self, path: str) -> None:
		"""Explain the deliberately closed suffix contract without content sniffing."""
		suffixes = tuple(suffix.lower() for suffix in pathlib.Path(path).suffixes)
		suffix = suffixes[-1] if suffixes else ""
		compression_suffixes = {".bz2", ".gz", ".xz", ".zip", ".zst"}
		inner_suffix = (
			suffixes[-2]
			if len(suffixes) >= 2 and suffixes[-1] in compression_suffixes
			else None
		)
		if suffixes[-1:] == (".svgz",) or inner_suffix == ".svg":
			message = (
				"Compressed SVG files are not supported. Choose an uncompressed .svg file "
				"containing embedded CDML, or an uncompressed .cdml drawing."
			)
		elif inner_suffix in {".cdml", ".cdsvg"}:
			message = (
				"Compressed Ferrum drawings are not supported. Choose an uncompressed "
				".cdml drawing."
			)
		elif suffix == ".cdsvg":
			message = (
				"Ferrum does not open .cdsvg files. Choose a decoded .svg file containing "
				"embedded CDML, or an uncompressed .cdml drawing."
			)
		else:
			message = (
				"Ferrum opens the formats listed in File/Open. The selected file has not been "
				"opened and the current document has not changed."
			)
		self._present_refusal(self._unavailable_edit_refusal(message))

	#============================================
	def _open_disposition_for_request(
			self, descriptor: object, interactive: bool,
			) -> _LocalDocumentOpenDisposition:
		"""Honor the catalog's replacement fact for a pristine initial tab."""
		tab = self._host.active_tab()
		if (
			getattr(descriptor, "allows_current_tab_replacement", False)
			and interactive and tab is not None and tab.is_pristine_initial_placeholder()
			and not self._host.tab_has_active_canvas_interaction(tab)
		):
			return _LocalDocumentOpenDisposition.REPLACE_PRISTINE_TARGET
		return _LocalDocumentOpenDisposition.NEW_TAB

	#============================================
	def _capture_pristine_target_fence(
			self, target: object | None,
			) -> tuple[int | None, str | None, bool]:
		"""Copy the target's authoritative provenance before detached admission."""
		if target is None:
			return None, None, False
		snapshot = target.current_snapshot
		return (
			snapshot.revision, snapshot.digest,
			not self._host.tab_has_active_canvas_interaction(target),
		)

	#============================================
	def _start_local_document_open(
			self, request: local_open_contract.LocalDocumentOpenRequest,
			) -> bool:
		"""Start one already-validated path as the current queue head."""
		if (
			type(request.target) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
			and request.target is not None
		):
			raise TypeError("Ferrum Open target must be an exact Ferrum document tab")
		if (
			type(request.focus_target)
			is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
			and request.focus_target is not None
		):
			raise TypeError("Ferrum Open focus target must be an exact Ferrum document tab")
		if not self._host.tab_is_registered(request.source):
			return False
		startup = _LocalDocumentOpenStartupTransaction(self, request)
		try:
			startup.start()
			return True
		finally:
			if not startup.started:
				startup.abort()

	#============================================
	def _create_local_document_open_worker(
			self, path: str, route_handle: object,
			) -> FerrumNativeLocalDocumentOpenWorker:
		"""Construct the one worker responsible for this admission."""
		return FerrumNativeLocalDocumentOpenWorker(path, route_handle)

	#============================================
	def _require_current_delivery(
			self, intent: _LocalDocumentOpenIntent,
			) -> ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery:
		"""Return the exact delivery owner for the current immutable intent."""
		delivery = self._local_document_open_delivery
		if delivery is None or delivery.intent is not intent:
			raise RuntimeError("Ferrum Local Open delivery no longer owns its current intent")
		return delivery

	#============================================
	def _finish_local_document_open_delivery(
			self, delivery: ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery,
			) -> None:
		"""Dispose one exact stopped worker and restore Open reachability."""
		intent = self._local_document_open_intent
		if intent is None or delivery.intent is not intent:
			return
		delivered = False
		try:
			if not intent.worker.delivery_cancelled:
				delivery.deliver()
			delivered = True
		finally:
			if not delivered:
				self._finalize_escaped_local_document_open_delivery(intent, delivery)
		outcome = delivery.outcome is _LocalDocumentOpenOutcome.COMPLETED
		terminal = self._terminal_lease_state(delivery, intent.worker.delivery_cancelled)
		self._settle_and_retire_local_document_open(intent, delivery, terminal)
		self._local_document_open_batch_success &= outcome
		self._host.emit_completed(intent.path, outcome)
		while self._local_document_open_queue and not self._host.shutdown_prepared():
			next_request = self._local_document_open_queue.popleft()
			if self._start_local_document_open(next_request):
				return
			self._local_document_open_batch_success = False
			self._host.emit_completed(next_request.path, False)
			self._host.show_status(
				self.tr("Queued Open source is no longer available."), 5000,
			)
		batch_success = self._local_document_open_batch_success
		self._local_document_open_batch_success = True
		self._host.action_refresh()
		self._host.emit_queue_drained(batch_success)
		self._host.show_status(self.tr("Finished: Open request is settled."), 3000)

	#============================================
	def _finalize_escaped_local_document_open_delivery(
			self, intent: _LocalDocumentOpenIntent,
			delivery: ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery,
			) -> None:
		"""Retire an escaped delivery without rewriting an already committed receipt."""
		original_error = sys.exception()
		completed = (
			delivery.outcome is _LocalDocumentOpenOutcome.COMPLETED
			or delivery.replacement_lease_settled
		)
		delivery.outcome = (
			_LocalDocumentOpenOutcome.COMPLETED
			if completed else _LocalDocumentOpenOutcome.FAILED
		)
		terminal = (
			ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
			if completed else ferrum_qt.ferrum.operation_leases.LeaseState.FAILED
		)
		try:
			try:
				self._settle_and_retire_local_document_open(intent, delivery, terminal)
			except ferrum_qt.ferrum.operation_leases.OperationLeaseError:
				if original_error is None:
					raise
		finally:
			self._local_document_open_queue.clear()
			try:
				self._host.emit_completed(intent.path, completed)
			finally:
				try:
					self._host.emit_queue_drained(False)
				finally:
					self._host.action_refresh()

	#============================================
	def _terminal_lease_state(
			self,
			delivery: ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery,
			was_cancelled: bool,
			) -> ferrum_qt.ferrum.operation_leases.LeaseState:
		"""Map one closed delivery fact to its sole registry terminal state."""
		return {
			_LocalDocumentOpenOutcome.COMPLETED:
				ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED,
			_LocalDocumentOpenOutcome.REFUSED:
				ferrum_qt.ferrum.operation_leases.LeaseState.REFUSED,
			_LocalDocumentOpenOutcome.FAILED:
				ferrum_qt.ferrum.operation_leases.LeaseState.FAILED,
		}.get(
			delivery.outcome,
			ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLED
			if was_cancelled else ferrum_qt.ferrum.operation_leases.LeaseState.FAILED,
		)

	#============================================
	def _settle_and_retire_local_document_open(
			self, intent: _LocalDocumentOpenIntent,
			delivery: ferrum_qt.ferrum.local_document_open_delivery.LocalDocumentOpenDelivery,
			terminal: ferrum_qt.ferrum.operation_leases.LeaseState,
			) -> None:
		"""Clear and retire one delivery even when registry settlement raises."""
		try:
			if not delivery.replacement_lease_settled:
				self._operation_leases.settle(
					self._local_document_open_capability, intent.lease, terminal,
				)
		finally:
			self._clear_current_local_document_open(intent)
			delivery.retire()

	#============================================
	def _clear_current_local_document_open(self, intent: _LocalDocumentOpenIntent | None) -> None:
		"""Clear only the controller slots belonging to one exact local Open intent."""
		if intent is not None and self._local_document_open_intent is intent:
			self._local_document_open_intent = None
			self._local_document_open_delivery = None

	#============================================
	def _cancel_local_document_open(self) -> None:
		"""Invalidate delivery while bounded Rust admission finishes normally."""
		intent = self._local_document_open_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		cleared_count = len(self._local_document_open_queue)
		self._local_document_open_queue.clear()
		self._require_current_delivery(intent).outcome = _LocalDocumentOpenOutcome.CANCELLED
		self._operation_leases.request_cancellation(
			self._local_document_open_capability, intent.lease, "author_cancel",
		)
		intent.worker.cancel_delivery()
		self._host.show_status(self.tr(
			f"Cancelling Open; waiting for a safe finish. Cleared {cleared_count} queued request(s).",
		), 0)
		self._host.action_refresh()

	#============================================
	def request_close_cancellation(
			self, source: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None,
			) -> bool:
		"""Request exact-source or window-close cancellation.

		Args:
			source: Source tab being closed, or ``None`` for window shutdown.

		Returns:
			Whether a live exact source delivery was cancelled.
		"""
		intent = self._local_document_open_intent
		if intent is None or (source is not None and intent.source is not source):
			return False
		self._cancel_local_document_open()
		if source is not None:
			self._host.show_status(self.tr("Cancelled Open in Current Tab delivery."), 3000)
		return True

	#============================================
	def has_pending_local_document_open(self) -> bool:
		"""Return whether Rust admission or a queued launch path remains pending."""
		return self._local_document_open_intent is not None or bool(self._local_document_open_queue)

	#============================================
	def _refresh_local_document_open_action(self) -> None:
		"""Mirror the one-worker lifecycle onto Open and Cancel Open.

		The window lifecycle coordinator owns invocation of this refresh.  Keeping
		that direction one-way prevents line-tool action refreshes from recursively
		re-entering the complete action refresh.
		"""
		intent = self._local_document_open_intent
		shutdown = self._host.shutdown_prepared()
		cancelling = intent is not None and intent.worker.delivery_cancelled
		self._open_action.setEnabled(
			intent is None and not self._host.snapshot_busy() and not shutdown,
		)
		self._cancel_open_action.setEnabled(
			intent is not None and not intent.worker.delivery_cancelled,
		)
		can_replace = not cancelling and self._can_begin_explicit_current_replacement()
		self._open_in_current_tab_action.setEnabled(can_replace)
		tab = self._host.active_tab()
		if not can_replace and tab is not None and self._host.tab_has_active_canvas_interaction(tab):
				message = self.tr("Finish or cancel the active canvas action before replacing this tab.")
				self._open_in_current_tab_action.setToolTip(message)
				self._open_in_current_tab_action.setStatusTip(message)
		elif not can_replace and tab is not None and self._host.tab_has_active_operation(tab):
				message = self.tr("Finish or cancel the current document operation before replacing this tab.")
				self._open_in_current_tab_action.setToolTip(message)
				self._open_in_current_tab_action.setStatusTip(message)
		else:
				message = self.tr("Open a Ferrum drawing in place of the current tab.")
				self._open_in_current_tab_action.setToolTip(message)
				self._open_in_current_tab_action.setStatusTip(message)
