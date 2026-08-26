"""Asynchronous Rust-owned local-document admission for Ferrum windows."""

# Standard Library
import collections
import dataclasses
import enum
import os
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
from ferrum_qt.ferrum.local_document_open_types import (
	FerrumNativeLocalDocumentOpenFailure,
	FerrumNativeLocalDocumentOpenWorker,
	_LocalDocumentSourceKind,
	_current_tab_replacement_source_kind_for_path,
	_interchange_route_handle_for_path,
	_local_document_source_kind_for_path,
)
import ferrum_qt.ferrum.engine as engine

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.canvas_interaction
import ferrum_qt.ferrum.tab_operations


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _LocalDocumentOpenIntent:
	"""One immutable local-document admission request and its sole worker."""

	path: str
	source_kind: _LocalDocumentSourceKind
	route_handle: object | None
	disposition: "_LocalDocumentOpenDisposition"
	target: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None
	target_revision: int | None
	target_digest: str | None
	target_canvas_idle: bool
	focus_target: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None
	activate_if_still_current: bool
	recent_request: bool
	worker: FerrumNativeLocalDocumentOpenWorker
	replacement_fence: "_ExplicitReplacementFence | None" = None


#============================================
class _LocalDocumentOpenDisposition(enum.Enum):
	"""Qt-owned installation policy fixed before Rust admission begins."""

	NEW_TAB = enum.auto()
	REPLACE_PRISTINE_TARGET = enum.auto()
	REPLACE_EXPLICIT_CURRENT_TARGET = enum.auto()


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _ExplicitReplacementFence:
	"""Qt-owned facts proving one intentional populated-tab destination."""

	target: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	index: int
	revision: int
	digest: str
	dirty: bool
	file_path: str | None
	origin_token: object | None


#============================================
class _LocalDocumentOpenRelay(PySide6.QtCore.QObject):
	"""Deliver admission outcomes to the owning window on the Qt thread."""

	#============================================
	def __init__(self, owner: object) -> None:
		"""Retain the window that owns the current Open intent."""
		super().__init__(owner)
		self._owner = owner

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_prepared(self, prepared: object) -> None:
		"""Forward one admitted session receipt with its exact worker."""
		self._owner._on_local_document_open_prepared(self.sender(), prepared)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_failed(self, failure: object) -> None:
		"""Forward one copied admission failure with its exact worker."""
		self._owner._on_local_document_open_failed(self.sender(), failure)

	#============================================
	@PySide6.QtCore.Slot()
	def on_finished(self) -> None:
		"""Release the exact worker after its Ferrum call has stopped."""
		self._owner._on_local_document_open_finished(self.sender())


#============================================
class FerrumNativeLocalDocumentOpenMixin:
	"""Own local-document Open without frontend document parsing."""

	#============================================
	def _initialize_local_document_open(self) -> None:
		"""Create the sole local-document Open intent and Qt-thread relay."""
		self._local_document_open_intent: _LocalDocumentOpenIntent | None = None
		self._local_document_open_queue: collections.deque[tuple[str, _LocalDocumentSourceKind, object | None, _LocalDocumentOpenDisposition, object | None, int | None, str | None, bool, object | None, bool, bool]] = collections.deque()
		self._local_document_open_outcome: bool | None = None
		self._local_document_open_batch_success = True
		self._local_document_open_delivery_active = False
		self._local_document_open_finished_while_delivering: object | None = None
		self._local_document_open_relay = _LocalDocumentOpenRelay(self)

	#============================================
	def _build_local_document_open_action(self) -> PySide6.QtGui.QAction:
		"""Construct explicit cancellation next to the host-owned Open action."""
		self._open_action.setToolTip(self.tr(
			"Open a local CDML drawing or SVG containing embedded CDML",
		))
		action = PySide6.QtGui.QAction(self.tr("Cancel Open"), self)
		action.triggered.connect(self._cancel_local_document_open)
		self._cancel_open_action = action
		self._register_action("file.open.cancel", action, lifecycle="stateful-cancel")
		return action

	#============================================
	def _native_new_document_filter(self) -> str:
		"""Build ordinary File/Open solely from Rust-owned route descriptors."""
		filters = [
			f"{descriptor.display_name} ({' '.join('*' + suffix for suffix in descriptor.suffixes)})"
			for descriptor in self._local_ingress_registry.local_document_open_descriptors
		]
		filters.append("All Files (*)")
		return ";;".join(filters)

	#============================================
	def _current_tab_replacement_filter(self) -> str:
		"""Expose only registry routes allowed to replace the current tab."""
		filters = [
			f"{descriptor.display_name} ({' '.join('*' + suffix for suffix in descriptor.suffixes)})"
			for descriptor in self._local_ingress_registry.local_document_open_descriptors
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
			self, self.tr("Open Ferrum Chemical Drawing in Current Tab"), "",
			self.tr(self._current_tab_replacement_filter()),
		)[0]
		if not path:
			return False
		return self.open_in_current_tab_path(path)

	#============================================
	def open_in_current_tab_path(self, file_path: str) -> bool:
		"""Prepare one source without allowing explicit replacement to become NewTab."""
		if type(file_path) is not str:
			raise TypeError("Ferrum local-document Open requires an exact path string")
		if not self._can_begin_explicit_current_replacement():
			return False
		absolute_path = os.path.abspath(file_path)
		source_kind = _current_tab_replacement_source_kind_for_path(
			absolute_path, self._local_ingress_registry.local_document_open_descriptors,
		)
		if source_kind is None:
			self._show_unsupported_local_document(absolute_path)
			return False
		target = self._active_native_tab()
		fence = self._capture_explicit_replacement_fence(target)
		self._local_document_open_batch_success = True
		self._start_local_document_open(
			absolute_path, source_kind, None, _LocalDocumentOpenDisposition.REPLACE_EXPLICIT_CURRENT_TARGET,
			target, fence.revision, fence.digest, True, target, True, False,
			replacement_fence=fence,
		)
		return True

	#============================================
	def _can_begin_explicit_current_replacement(self) -> bool:
		"""Keep the command bound to a live idle current Ferrum tab."""
		tab = self._active_native_tab()
		if (
			self._local_document_open_intent is not None
			or self._snapshot_export_is_busy()
			or getattr(self, "_shutdown_prepared", False)
		):
			return False
		return self._explicit_replacement_target_is_admissible(tab)

	#============================================
	def _explicit_replacement_target_is_admissible(self, target: object) -> bool:
		"""Share one exact target lifecycle predicate across action, capture, and swap."""
		return (
			type(target) is ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
			and target is self._active_native_tab()
			and target in self._native_tabs_by_page
			and not target.is_disposed
			and not target.requires_refresh
			and not self._tab_has_active_native_canvas_interaction(target)
			and not ferrum_qt.ferrum.tab_operations.
			tab_has_active_native_operation(self, target)
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
			target, self._tab_widget.indexOf(target), snapshot.revision, snapshot.digest,
			target.is_dirty,
			None if target.file_path is None else str(target.file_path),
			target.local_cdml_origin_token,
		)

	#============================================
	def _on_open(self) -> bool:
		"""Choose one bounded local drawing for Rust-owned admission."""
		if self._snapshot_export_is_busy():
			return False
		path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self, self.tr("Open Ferrum Chemical Drawing"), "",
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
		"""Begin one profile-owned Rust admission into a Ferrum tab."""
		if type(file_path) is not str:
			raise TypeError("Ferrum local-document Open requires an exact path string")
		if self._snapshot_export_is_busy():
			return False
		if replace_current:
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum drawings open in a new Ferrum tab."))
			return False
		absolute_path = os.path.abspath(file_path)
		source_kind = _local_document_source_kind_for_path(
			absolute_path, self._local_ingress_registry.local_document_open_descriptors,
		)
		if source_kind is None:
			self._show_unsupported_local_document(absolute_path)
			return False
		route_handle = _interchange_route_handle_for_path(
			absolute_path, self._local_ingress_registry.local_document_open_descriptors,
		)
		focus_target = self._active_native_tab() if interactive else None
		focus_busy = (
			focus_target is not None
			and self._tab_has_active_native_canvas_interaction(focus_target)
		)
		# File/Open is a terminal document command.  Capture the pre-cancellation
		# activity only to preserve its NewTab/focus policy, then close its
		# transient pointer owners before detached Rust admission can change tabs.
		if focus_busy:
			self.cancel_active_pointer_authoring()
		disposition = self._open_disposition_for_request(
			source_kind, interactive and not force_new_tab and not focus_busy,
		)
		target = focus_target if disposition is _LocalDocumentOpenDisposition.REPLACE_PRISTINE_TARGET else None
		target_revision, target_digest, target_canvas_idle = self._capture_pristine_target_fence(target)
		activate_if_still_current = not focus_busy
		if self._local_document_open_intent is not None:
			if self._local_document_open_intent.path == absolute_path:
				return True
			if not any(path == absolute_path for path, *_unused in self._local_document_open_queue):
				self._local_document_open_queue.append(
					(
					absolute_path, source_kind, route_handle, disposition, target, target_revision, target_digest,
						target_canvas_idle, focus_target, activate_if_still_current, recent_request,
					),
				)
			self.statusBar().showMessage(self.tr("Queued Ferrum drawing Open request."), 3000)
			self._refresh_actions()
			return True
		self._local_document_open_batch_success = True
		self._start_local_document_open(
			absolute_path, source_kind, route_handle, disposition, target, target_revision, target_digest, target_canvas_idle,
			focus_target, activate_if_still_current, recent_request,
		)
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
		elif suffix == ".cdxml":
			message = (
				"Ferrum does not import ChemDraw XML (.cdxml). Use the source application "
				"or a converter to make a supported .cdml drawing. This document has not changed."
			)
		else:
			message = (
				"Ferrum opens uncompressed .cdml drawings and decoded .svg files containing "
				"embedded CDML. The selected file has not been opened and the current document "
				"has not changed."
			)
		self._show_edit_refusal(self._unavailable_edit_refusal(message))

	#============================================
	def _open_disposition_for_request(
			self, source_kind: _LocalDocumentSourceKind, interactive: bool,
			) -> _LocalDocumentOpenDisposition:
		"""Reserve first-Open replacement for native CDML admission only."""
		tab = self._active_native_tab()
		if (
			source_kind is _LocalDocumentSourceKind.CDML
			and interactive and tab is not None and tab.is_pristine_initial_placeholder()
			and not self._tab_has_active_native_canvas_interaction(tab)
		):
			return _LocalDocumentOpenDisposition.REPLACE_PRISTINE_TARGET
		return _LocalDocumentOpenDisposition.NEW_TAB

	#============================================
	def _capture_pristine_target_fence(self, target: object | None) -> tuple[int | None, str | None, bool]:
		"""Copy the target's authoritative provenance before detached admission."""
		if target is None:
			return None, None, False
		snapshot = target.current_snapshot
		return (
			snapshot.revision, snapshot.digest,
			not self._tab_has_active_native_canvas_interaction(target),
		)

	#============================================
	def _tab_has_active_native_canvas_interaction(self, tab: object) -> bool:
		"""Query the host-owned Ferrum pointer lifecycle for this exact tab."""
		return ferrum_qt.ferrum.canvas_interaction \
			.tab_has_active_native_canvas_interaction(self, tab)

	#============================================
	def _start_local_document_open(
			self, absolute_path: str, source_kind: _LocalDocumentSourceKind, route_handle: object | None,
			disposition: _LocalDocumentOpenDisposition,
			target: object | None, target_revision: int | None, target_digest: str | None,
			target_canvas_idle: bool,
			focus_target: object | None, activate_if_still_current: bool,
			recent_request: bool, *, replacement_fence: _ExplicitReplacementFence | None = None,
			) -> None:
		"""Start one already-validated path as the current queue head."""
		worker = self._create_local_document_open_worker(absolute_path, source_kind, route_handle)
		if target is not None and type(target) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum Open target must be an exact Ferrum document tab")
		if focus_target is not None and type(focus_target) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum Open focus target must be an exact Ferrum document tab")
		self._local_document_open_intent = _LocalDocumentOpenIntent(
			absolute_path, source_kind, route_handle, disposition, target, target_revision, target_digest,
			target_canvas_idle, focus_target, activate_if_still_current, recent_request, worker,
			replacement_fence,
		)
		self._local_document_open_outcome = None
		worker.prepared.connect(
			self._local_document_open_relay.on_prepared,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.failed.connect(
			self._local_document_open_relay.on_failed,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		worker.finished.connect(
			self._local_document_open_relay.on_finished,
			PySide6.QtCore.Qt.ConnectionType.QueuedConnection,
		)
		self.statusBar().showMessage(self.tr("Opening drawing with Ferrum Rust..."), 0)
		self._refresh_actions()
		worker.start()

	#============================================
	def _create_local_document_open_worker(
			self, path: str, source_kind: _LocalDocumentSourceKind, route_handle: object | None,
			) -> FerrumNativeLocalDocumentOpenWorker:
		"""Construct the one worker responsible for this admission."""
		return FerrumNativeLocalDocumentOpenWorker(path, source_kind, route_handle)

	#============================================
	def _native_tab_for_origin_token(
			self, token: object,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None:
		"""Return a live tab whose Rust-issued descriptor identity matches."""
		for tab in self._native_tabs_by_page.values():
			if tab.local_cdml_origin_token == token:
				return tab
		return None

	#============================================
	def _can_replace_pristine_target(self, intent: _LocalDocumentOpenIntent) -> bool:
		"""Revalidate the exact bootstrap page after detached admission succeeds."""
		target = intent.target
		return (
			intent.source_kind is _LocalDocumentSourceKind.CDML
			and intent.disposition is _LocalDocumentOpenDisposition.REPLACE_PRISTINE_TARGET
			and target is not None
			and target in self._native_tabs_by_page
			and self._tab_widget.currentWidget() is target
			and target.is_pristine_initial_placeholder()
			and target.current_snapshot.revision == intent.target_revision
			and target.current_snapshot.digest == intent.target_digest
			and intent.target_canvas_idle
			and not self._tab_has_active_native_canvas_interaction(target)
		)

	#============================================
	def _replace_pristine_native_tab(self, old: object, new: object) -> None:
		"""Install a complete replacement at the old index before retiring it."""
		if type(old) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum Open replacement requires an exact old Ferrum tab")
		if type(new) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum Open replacement requires an exact new Ferrum tab")
		self._replace_native_tab_at_index(old, new, self._tab_widget.indexOf(old))

	#============================================
	def _replace_native_tab_at_index(self, old: object, new: object, index: int) -> None:
		"""Atomically install a complete replacement before retiring its target."""
		if index < 0 or old not in self._native_tabs_by_page:
			raise ValueError("Ferrum Open replacement target is no longer registered")
		self._tab_widget.insertTab(index, new, new.title)
		self._tab_widget.setTabToolTip(
			index, new.local_document_source_description or "",
		)
		self._native_tabs_by_page[new] = new
		self._install_native_hex_grid_for_tab(new)
		new.selection_changed.connect(self._on_native_selection_changed)
		new.view.display_transform_changed.connect(self._refresh_native_view_status)
		self._tab_widget.setCurrentIndex(index)
		self._tab_widget.removeTab(index + 1)
		self._native_tabs_by_page.pop(old)
		old.hide()
		old.setParent(None)
		old.dispose()
		new.view.setFocus()
		self._on_native_view_tab_changed()
		self._refresh_actions()

	#============================================
	def _explicit_replacement_fence_holds(self, fence: _ExplicitReplacementFence) -> bool:
		"""Require the same live selected tab and authoritative facts at each boundary."""
		target = fence.target
		if (
			getattr(self, "_shutdown_prepared", False)
			or target not in self._native_tabs_by_page
			or self._tab_widget.currentWidget() is not target
			or self._tab_widget.indexOf(target) != fence.index
			or target.is_disposed
			or target.requires_refresh
			or self._tab_has_active_native_canvas_interaction(target)
			or ferrum_qt.ferrum.tab_operations.
			tab_has_active_native_operation(self, target)
		):
			return False
		snapshot = target.current_snapshot
		return (
			snapshot.revision == fence.revision
			and snapshot.digest == fence.digest
			and target.is_dirty == fence.dirty
			and (None if target.file_path is None else str(target.file_path)) == fence.file_path
			and target.local_cdml_origin_token == fence.origin_token
		)

	#============================================
	def _report_explicit_replacement_stale(self) -> None:
		"""Contain a stale explicit request without redirecting its source elsewhere."""
		self._show_edit_refusal(self._unavailable_edit_refusal("Open in Current Tab did not replace the changed document; choose the command again."))

	#============================================
	def _deliver_explicit_current_replacement(
			self, intent: _LocalDocumentOpenIntent, session: object, observation: object,
			origin_token: object, receipt_source_kind: str,
		) -> bool:
		"""Apply one admitted receipt only to its still-current explicit destination."""
		fence = intent.replacement_fence
		if fence is None or not self._explicit_replacement_fence_holds(fence):
			self._report_explicit_replacement_stale()
			return False
		existing = self._native_tab_for_origin_token(origin_token)
		if existing is not None:
			self._tab_widget.setCurrentIndex(self._tab_widget.indexOf(existing))
			self._record_confirmed_native_recent_path(intent.path)
			self.statusBar().showMessage(self.tr(f'"{pathlib.Path(intent.path).name}" is already open.'), 3000)
			return True
		dirty_choice = "replace"
		while fence.dirty:
			dirty_choice = self._confirm_dirty_explicit_replacement(fence, intent.path)
			if dirty_choice == "cancel":
				return False
			if dirty_choice == "save":
				fence = self._capture_explicit_replacement_fence(fence.target)
				if fence.dirty or not self._explicit_replacement_fence_holds(fence):
					self._report_explicit_replacement_stale()
					return False
				break
			if dirty_choice == "replace" and self._explicit_replacement_fence_holds(fence):
				break
			if dirty_choice == "retry" and self._explicit_replacement_fence_holds(fence):
				continue
			self._report_explicit_replacement_stale()
			return False
		tab = None
		try:
			tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab.from_admitted_local_open(
				session, pathlib.Path(intent.path).name, observation,
			)
			tab._adopt_local_document_origin(
				intent.path, receipt_source_kind, origin_token,
			)
			if not self._explicit_replacement_fence_holds(fence):
				self._report_explicit_replacement_stale()
				tab.dispose()
				return False
			self._replace_native_tab_at_index(fence.target, tab, fence.index)
		except Exception:
			if tab is not None and not tab.is_disposed:
				tab.dispose()
			self._report_local_document_installation_failed(intent)
			return False
		self._record_confirmed_native_recent_path(intent.path)
		self.statusBar().showMessage(self.tr(_local_document_open_success(intent)), 3000)
		return True

	#============================================
	def _confirm_dirty_explicit_replacement(
			self, fence: _ExplicitReplacementFence, source_path: str,
		) -> str:
		"""Offer Save, intentional Replace, or Cancel only after successful admission."""
		target = fence.target
		name = target.title if target.file_path is not None else "this untitled document"
		message = self.tr(
			f'Save changes to "{name}" before replacing it with "{pathlib.Path(source_path).name}"?',
		)
		box = PySide6.QtWidgets.QMessageBox(
			PySide6.QtWidgets.QMessageBox.Icon.Warning,
			self.tr("Replace Current Tab"), message,
			parent=self,
		)
		save = box.addButton(self.tr("Save"), PySide6.QtWidgets.QMessageBox.ButtonRole.AcceptRole)
		replace = box.addButton(self.tr("Replace"), PySide6.QtWidgets.QMessageBox.ButtonRole.DestructiveRole)
		cancel = box.addButton(self.tr("Cancel"), PySide6.QtWidgets.QMessageBox.ButtonRole.RejectRole)
		box.setDefaultButton(save)
		box.setEscapeButton(cancel)
		box.exec()
		if box.clickedButton() is replace:
			return "replace" if self._explicit_replacement_fence_holds(fence) else "cancel"
		if box.clickedButton() is not save:
			return "cancel"
		if target.file_path is None:
			return "save" if self._prompt_native_save(target, force_save_as=True) else "retry"
		return "save" if self._save_native_tab_to_path(target, str(target.file_path)) else "retry"

	#============================================
	def _activate_new_tab_for_intent(self, intent: _LocalDocumentOpenIntent) -> bool:
		"""Preserve a later or busy interactive focus while detached Open completes."""
		if intent.focus_target is None:
			return True
		return (
			intent.activate_if_still_current
			and self._tab_widget.currentWidget() is intent.focus_target
		)

	#============================================
	def _on_local_document_open_prepared(self, worker: object, prepared: object) -> None:
		"""Install one exact still-current admitted session on the Qt thread."""
		intent = self._local_document_open_intent
		if intent is None or worker is not intent.worker or intent.worker.delivery_cancelled:
			return
		if type(prepared) is not engine.PreparedLocalDocumentOpenV1:
			self._local_document_open_outcome = False
			self._report_local_document_installation_failed(intent)
			return
		tab = None
		try:
			session, observation, origin_token, receipt_source_kind = prepared.take_admission_v1()
			if (
				(intent.source_kind is _LocalDocumentSourceKind.INTERCHANGE and not receipt_source_kind)
				or (intent.source_kind is not _LocalDocumentSourceKind.INTERCHANGE
					and receipt_source_kind != intent.source_kind.value)
			):
				raise RuntimeError("Ferrum returned a receipt for a different source kind")
			if intent.disposition is _LocalDocumentOpenDisposition.REPLACE_EXPLICIT_CURRENT_TARGET:
				self._local_document_open_delivery_active = True
				try:
					self._local_document_open_outcome = self._deliver_explicit_current_replacement(
						intent, session, observation, origin_token, receipt_source_kind,
					)
				finally:
					self._local_document_open_delivery_active = False
					finished = self._local_document_open_finished_while_delivering
					self._local_document_open_finished_while_delivering = None
					if finished is not None:
						self._on_local_document_open_finished(finished)
				return
			existing = self._native_tab_for_origin_token(origin_token)
			if existing is not None:
				self._local_document_open_outcome = True
				self._tab_widget.setCurrentIndex(self._tab_widget.indexOf(existing))
				self._record_confirmed_native_recent_path(intent.path)
				return
			title = pathlib.Path(intent.path).name
			tab = (
				ferrum_qt.ferrum.document_tab.
				FerrumNativeDocumentTab.from_admitted_local_open(
					session, title, observation,
				)
			)
			tab._adopt_local_document_origin(
				intent.path, receipt_source_kind, origin_token,
			)
			if self._can_replace_pristine_target(intent):
				self._replace_pristine_native_tab(intent.target, tab)
			else:
				self._register_native_tab(
					tab, activate=self._activate_new_tab_for_intent(intent),
				)
				index = self._tab_widget.indexOf(tab)
				if index >= 0:
					self._tab_widget.setTabToolTip(
						index, tab.local_document_source_description or "",
					)
		except Exception as exc:
			self._local_document_open_outcome = False
			if tab is not None:
				tab.dispose()
			self._report_local_document_installation_failed(intent, exc)
			return
		self._local_document_open_outcome = True
		self._record_confirmed_native_recent_path(intent.path)
		self.statusBar().showMessage(self.tr(_local_document_open_success(intent)), 3000)

	#============================================
	def _on_local_document_open_failed(self, worker: object, failure: object) -> None:
		"""Present one current typed Rust admission failure."""
		intent = self._local_document_open_intent
		if intent is None or worker is not intent.worker or intent.worker.delivery_cancelled:
			return
		if type(failure) is not FerrumNativeLocalDocumentOpenFailure:
			self._local_document_open_outcome = False
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum returned an invalid drawing Open failure."))
			return
		self._local_document_open_outcome = False
		if intent.recent_request and self._handle_failed_native_recent_open(intent.path, failure):
			return
		_title, guidance = _local_document_open_guidance(intent.source_kind, failure)
		if failure.stage == "source_policy" or failure.category == "source_rejected":
			outcome = (
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SOURCE_NOT_ALLOWED
			)
		else:
			outcome = ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.INVALID_DOCUMENT
		self._show_edit_refusal(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
				outcome, pathlib.Path(intent.path).name,
				f"{guidance}\n\n{failure.message}",
			),
		)

	#============================================
	def _report_local_document_installation_failed(
			self, intent: _LocalDocumentOpenIntent, error: Exception,
			) -> None:
		"""Contain one post-admission construction failure without exposing internals."""
		self._show_edit_refusal(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.DOCUMENT_DISPLAY_FAILED,
				pathlib.Path(intent.path).name, str(error),
			),
		)

	#============================================
	def _on_local_document_open_finished(self, worker: object) -> None:
		"""Dispose one exact stopped worker and restore Open reachability."""
		intent = self._local_document_open_intent
		if intent is None or worker is not intent.worker:
			return
		if self._local_document_open_delivery_active:
			self._local_document_open_finished_while_delivering = worker
			return
		outcome = self._local_document_open_outcome is True
		self._local_document_open_batch_success &= outcome
		self._local_document_open_intent = None
		self._local_document_open_outcome = None
		worker.deleteLater()
		self.local_document_open_completed.emit(intent.path, outcome)
		if self._local_document_open_queue and not getattr(self, "_shutdown_prepared", False):
			(
				next_path, next_source_kind, next_route_handle, disposition, target, revision, digest, canvas_idle,
				focus_target, activate_if_still_current, recent_request,
			) = self._local_document_open_queue.popleft()
			self._start_local_document_open(
				next_path, next_source_kind, next_route_handle, disposition, target, revision, digest, canvas_idle,
				focus_target, activate_if_still_current, recent_request,
			)
			return
		batch_success = self._local_document_open_batch_success
		self._local_document_open_batch_success = True
		self._refresh_actions()
		self.local_document_open_queue_drained.emit(batch_success)

	#============================================
	def _cancel_local_document_open(self) -> None:
		"""Invalidate delivery while bounded Rust admission finishes normally."""
		intent = self._local_document_open_intent
		if intent is None or intent.worker.delivery_cancelled:
			return
		self._local_document_open_queue.clear()
		self._local_document_open_outcome = False
		intent.worker.cancel_delivery()
		self.statusBar().showMessage(
			self.tr("Cancelling drawing Open delivery; waiting for Rust to finish..."), 0,
		)
		self._refresh_actions()

	#============================================
	def _cancel_local_document_open_for_close(self) -> bool:
		"""Cancel a live Open delivery and require a later close attempt."""
		if self._local_document_open_intent is None:
			return False
		self._cancel_local_document_open()
		return True

	#============================================
	def _cancel_explicit_replacement_for_target_close(self, tab: object) -> bool:
		"""Invalidate a prepared explicit destination before that tab closes."""
		intent = self._local_document_open_intent
		if intent is None or intent.replacement_fence is None:
			return False
		if intent.replacement_fence.target is not tab:
			return False
		self._cancel_local_document_open()
		self.statusBar().showMessage(self.tr("Cancelled Open in Current Tab delivery."), 3000)
		return True

	#============================================
	def _record_confirmed_native_recent_path(self, path: str) -> None:
		"""Delegate completed admission to the optional ordinary recent-file owner."""
		recent_files = getattr(self, "_native_recent_files", None)
		if recent_files is not None:
			recent_files.record_confirmed_path(path)

	#============================================
	def _handle_failed_native_recent_open(
			self, path: str, failure: FerrumNativeLocalDocumentOpenFailure,
			) -> bool:
		"""Use a single recovery dialog only for typed stale recent failures."""
		recent_files = getattr(self, "_native_recent_files", None)
		if recent_files is None:
			return False
		return recent_files.handle_failed_recent_open(path, failure)

	#============================================
	def has_pending_local_document_open(self) -> bool:
		"""Return whether Rust admission or a queued launch path remains pending."""
		return self._local_document_open_intent is not None or bool(self._local_document_open_queue)

	#============================================
	def _snapshot_export_is_busy(self) -> bool:
		"""Query the host-owned export lifecycle without importing its mixin."""
		busy = getattr(self, "_snapshot_export_busy", None)
		return callable(busy) and busy()

	#============================================
	def _refresh_local_document_open_action(self) -> None:
		"""Mirror the one-worker lifecycle onto Open and Cancel Open.

		The window lifecycle coordinator owns invocation of this refresh.  Keeping
		that direction one-way prevents line-tool action refreshes from recursively
		re-entering the complete action refresh.
		"""
		intent = self._local_document_open_intent
		shutdown = getattr(self, "_shutdown_prepared", False)
		self._open_action.setEnabled(
			intent is None and not self._snapshot_export_is_busy() and not shutdown,
		)
		self._cancel_open_action.setEnabled(
			intent is not None and not intent.worker.delivery_cancelled,
		)
		if hasattr(self, "_open_in_current_tab_action"):
			can_replace = self._can_begin_explicit_current_replacement()
			self._open_in_current_tab_action.setEnabled(can_replace)
			tab = self._active_native_tab()
			if not can_replace and tab is not None and self._tab_has_active_native_canvas_interaction(tab):
				message = self.tr("Finish or cancel the active canvas action before replacing this tab.")
				self._open_in_current_tab_action.setToolTip(message)
				self._open_in_current_tab_action.setStatusTip(message)
			elif not can_replace and tab is not None and ferrum_qt.ferrum.tab_operations.tab_has_active_native_operation(self, tab):
				message = self.tr("Finish or cancel the current document operation before replacing this tab.")
				self._open_in_current_tab_action.setToolTip(message)
				self._open_in_current_tab_action.setStatusTip(message)
			else:
				message = self.tr("Open a Ferrum drawing in place of the current tab.")
				self._open_in_current_tab_action.setToolTip(message)
				self._open_in_current_tab_action.setStatusTip(message)


#============================================
def _local_document_open_success(intent: _LocalDocumentOpenIntent) -> str:
	"""Describe a successful admission without claiming SVG wrapper preservation."""
	if intent.source_kind is _LocalDocumentSourceKind.DECODED_CDSVG:
		return f"Opened embedded CDML from SVG; Save writes CDML: {intent.path}"
	if intent.source_kind is _LocalDocumentSourceKind.INTERCHANGE:
		return f"Imported local interchange document: {intent.path}"
	return f"Loaded Rust CDML: {intent.path}"


#============================================
def _local_document_open_guidance(
		source_kind: _LocalDocumentSourceKind, failure: FerrumNativeLocalDocumentOpenFailure,
		) -> tuple[str, str]:
	"""Return bounded recovery language for one Rust-owned admission category."""
	if source_kind is _LocalDocumentSourceKind.DECODED_CDSVG:
		guidance = {
			"source_rejected": (
				"SVG Source Rejected", "Choose a regular, non-symlink decoded .svg file.",
			),
			"wrapper_rejected": (
				"SVG Wrapper Rejected", "Choose valid UTF-8 SVG containing embedded CDML.",
			),
			"embedded_cdml_not_found": (
				"Embedded CDML Not Found",
				"This SVG has no canonical embedded CDML; artwork is not chemistry.",
			),
			"multiple_embedded_cdml": (
				"Multiple Embedded CDML Documents", "Choose an SVG containing exactly one embedded CDML document.",
			),
			"resource_limit": (
				"SVG Resource Limit", "The file exceeds Ferrum's decoded CD-SVG V1 envelope.",
			),
			"embedded_cdml_rejected": (
				"Embedded CDML Rejected", "The embedded CDML is unsupported by Ferrum.",
			),
		}
		return guidance.get(
			failure.category,
			("SVG Open Error", "The current tab is unchanged; choose another decoded SVG."),
		)
	if failure.error_type == "DocumentInputError":
		if failure.stage == "source_policy":
			return "CDML Source Rejected", "Choose a regular, non-symlink .cdml file."
		if failure.stage == "bytes":
			return "CDML Resource Limit", "The file exceeds Ferrum's local-CDML V1 source envelope."
		if failure.stage == "utf8":
			return "CDML Text Rejected", "The file must contain valid UTF-8 CDML text."
		return "CDML Document Rejected", "The file does not satisfy Ferrum's local-CDML V1 profile."
	if failure.error_type == "DocumentLoadError":
		return (
			"CDML Document Rejected",
			"The selected file is not a supported Ferrum CDML document. "
			"Choose another .cdml file or use File > Import for a chemistry format.",
		)
	return "CDML Open Error", "Ferrum could not complete the local CDML admission."
