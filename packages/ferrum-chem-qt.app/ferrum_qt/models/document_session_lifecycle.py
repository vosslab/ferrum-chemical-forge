"""Per-tab ownership and teardown boundary for Ferrum-Qt documents."""

# Standard Library

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.setup.canvas_setup
import ferrum_qt.setup.mode_setup
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.io.cdml_candidate
import ferrum_qt.io.user_template_catalog
import ferrum_qt.models.backend_revision_history
import ferrum_qt.models.document
import ferrum_qt.models.projection_lifecycle
import ferrum_qt.undo.commands
import ferrum_qt.wavy_geometry

import ferrum_qt.models.document_session_requests
import ferrum_qt.models.document_session_support

orphaned_import_worker_count = ferrum_qt.models.document_session_requests.orphaned_import_worker_count
_release_orphaned_import_worker = ferrum_qt.models.document_session_requests._release_orphaned_import_worker
_adopt_orphaned_import_worker = ferrum_qt.models.document_session_requests._adopt_orphaned_import_worker
_freeze_plain_payload = ferrum_qt.models.document_session_requests._freeze_plain_payload
_direct_core_cdml_children = ferrum_qt.models.document_session_requests._direct_core_cdml_children
_is_unchanged_authoritative_snapshot = ferrum_qt.models.document_session_requests._is_unchanged_authoritative_snapshot
BackendProjectionOutOfSyncError = ferrum_qt.models.document_session_requests.BackendProjectionOutOfSyncError
ProjectionReplacementError = ferrum_qt.models.document_session_requests.ProjectionReplacementError
BackendFragmentExtractionError = ferrum_qt.models.document_session_requests.BackendFragmentExtractionError
PersistentOperationRequest = ferrum_qt.models.document_session_requests.PersistentOperationRequest
_UserTemplateModeDescriptor = ferrum_qt.models.document_session_requests._UserTemplateModeDescriptor
_freeze_user_template_catalog = ferrum_qt.models.document_session_requests._freeze_user_template_catalog
build_user_template_insert_request = ferrum_qt.models.document_session_requests.build_user_template_insert_request
build_atom_element_request = ferrum_qt.models.document_session_requests.build_atom_element_request
build_atom_align_request = ferrum_qt.models.document_session_requests.build_atom_align_request
build_atom_translate_request = ferrum_qt.models.document_session_requests.build_atom_translate_request
build_selection_translate_request = ferrum_qt.models.document_session_requests.build_selection_translate_request
build_atom_rotate_request = ferrum_qt.models.document_session_requests.build_atom_rotate_request
build_bond_order_request = ferrum_qt.models.document_session_requests.build_bond_order_request
build_bond_type_request = ferrum_qt.models.document_session_requests.build_bond_type_request
build_bond_properties_patch_request = ferrum_qt.models.document_session_requests.build_bond_properties_patch_request
build_atom_properties_patch_request = ferrum_qt.models.document_session_requests.build_atom_properties_patch_request
build_text_properties_patch_request = ferrum_qt.models.document_session_requests.build_text_properties_patch_request
build_rich_text_patch_request = ferrum_qt.models.document_session_requests.build_rich_text_patch_request
rich_text_patch_from_plain_runs = ferrum_qt.models.document_session_requests.rich_text_patch_from_plain_runs
build_plus_properties_patch_request = ferrum_qt.models.document_session_requests.build_plus_properties_patch_request
build_wavy_properties_patch_request = ferrum_qt.models.document_session_requests.build_wavy_properties_patch_request
build_fragment_create_request = ferrum_qt.models.document_session_requests.build_fragment_create_request
build_fragment_delete_request = ferrum_qt.models.document_session_requests.build_fragment_delete_request
build_implicit_group_expand_request = ferrum_qt.models.document_session_requests.build_implicit_group_expand_request
build_linear_form_convert_request = ferrum_qt.models.document_session_requests.build_linear_form_convert_request
build_atom_mark_request = ferrum_qt.models.document_session_requests.build_atom_mark_request
build_structure_delete_request = ferrum_qt.models.document_session_requests.build_structure_delete_request
build_structure_fragment_extraction_query = ferrum_qt.models.document_session_requests.build_structure_fragment_extraction_query
build_top_level_fragment_extraction_query = ferrum_qt.models.document_session_requests.build_top_level_fragment_extraction_query
build_molecule_name_request = ferrum_qt.models.document_session_requests.build_molecule_name_request
build_paper_properties_request = ferrum_qt.models.document_session_requests.build_paper_properties_request
build_presentation_stack_request = ferrum_qt.models.document_session_requests.build_presentation_stack_request
build_top_level_transform_request = ferrum_qt.models.document_session_requests.build_top_level_transform_request
PersistentActionOutcome = ferrum_qt.models.document_session_support.PersistentActionOutcome
_PreparedPersistentOperation = ferrum_qt.models.document_session_support._PreparedPersistentOperation
CloseState = ferrum_qt.models.document_session_support.CloseState
PreparedNativeCDML = ferrum_qt.models.document_session_support.PreparedNativeCDML
PreparedImportedCDML = ferrum_qt.models.document_session_support.PreparedImportedCDML
BackendSnapshotPublicationError = ferrum_qt.models.document_session_support.BackendSnapshotPublicationError
_resolved_publication_target = ferrum_qt.models.document_session_support._resolved_publication_target
_write_backend_snapshot = ferrum_qt.models.document_session_support._write_backend_snapshot


class DocumentSessionLifecycleMixin:
	#============================================
	@PySide6.QtCore.Slot(bool)
	def _on_modified_changed(self, _dirty: bool) -> None:
		"""Forward the tab title after a Qt dirty-state transition."""
		self.title_changed.emit(self.title)

	#============================================
	@PySide6.QtCore.Slot(int)
	def _on_persistent_mutated(self, _generation: int) -> None:
		"""Permanently revoke backend-write provenance after a Qt-local edit."""
		self._backend_projection_synchronized = False
		self._legacy_isolated = True

	#============================================
	def _clear_mode_persistent_actions(self) -> None:
		"""Break mode callback references before session-owned Qt teardown."""
		if self._mode_manager is None:
			return
		for mode in self._mode_manager._modes.values():
			installer = getattr(mode, "set_persistent_operation", None)
			if callable(installer):
				installer(None)
			align_installer = getattr(mode, "set_atom_align_operation", None)
			if callable(align_installer):
				align_installer(None)
			translate_installer = getattr(mode, "set_atom_translate_operation", None)
			if callable(translate_installer):
				translate_installer(None)
			translate_authority_installer = getattr(mode, "set_atom_translate_authority", None)
			if callable(translate_authority_installer):
				translate_authority_installer(None)
			presentation_translate_installer = getattr(mode, "set_presentation_translate_operation", None)
			if callable(presentation_translate_installer):
				presentation_translate_installer(None)
			presentation_context_installer = getattr(mode, "set_presentation_translate_context", None)
			if callable(presentation_context_installer):
				presentation_context_installer(None)
			selection_translate_installer = getattr(mode, "set_selection_translate_operation", None)
			if callable(selection_translate_installer):
				selection_translate_installer(None)
			selection_context_installer = getattr(mode, "set_selection_translate_context", None)
			if callable(selection_context_installer):
				selection_context_installer(None)
			delete_context_installer = getattr(mode, "set_top_level_delete_context", None)
			if callable(delete_context_installer):
				delete_context_installer(None)
			structure_delete_installer = getattr(mode, "set_structure_delete_context", None)
			if callable(structure_delete_installer):
				structure_delete_installer(None)
			atom_mark_delete_installer = getattr(mode, "set_atom_mark_delete_context", None)
			if callable(atom_mark_delete_installer):
				atom_mark_delete_installer(None)
			rotate_installer = getattr(mode, "set_atom_rotate_operation", None)
			if callable(rotate_installer):
				rotate_installer(None)
			candidate_installer = getattr(mode, "set_atom_number_context", None)
			if callable(candidate_installer):
				candidate_installer(None)
			mark_revision_installer = getattr(mode, "set_atom_mark_revision", None)
			if callable(mark_revision_installer):
				mark_revision_installer(None)
			template_installer = getattr(mode, "set_template_action", None)
			if callable(template_installer):
				template_installer(None)
			biotemplate_installer = getattr(mode, "set_biotemplate_action", None)
			if callable(biotemplate_installer):
				biotemplate_installer(None)
			user_template_installer = getattr(mode, "set_user_template_action", None)
			if callable(user_template_installer):
				user_template_installer(None)

	#============================================
	def _require_live_persistent_operation(self) -> None:
		"""Reject backend mutation or persistence after this session is terminal."""
		if self._disposed:
			raise RuntimeError("Cannot change or save backend CDML after session disposal")

	#============================================
	def _dispose_failed_construction(
			self, staged_document: ferrum_qt.models.document.Document | None,
			) -> None:
		"""Undo a failed constructor without consuming staged native content.

		The staged document is deliberately restored as detached state instead of
		being cleared or queued for deletion.  That leaves its prepared value
		reusable when canvas or mode setup fails after backend parsing succeeds.
		"""
		self._disposed = True
		self.clear_projection_lifecycle_port()
		self._clear_mode_persistent_actions()
		self.invalidate_import_requests()
		self._stop_import_workers()
		if self._document is not None:
			if self._document_modified_connected:
				try:
					self._document.modified_changed.disconnect(self._on_modified_changed)
				except (RuntimeError, TypeError):
					pass
				self._document_modified_connected = False
			if self._document_persistent_mutation_connected:
				try:
					self._document.persistent_mutated.disconnect(self._on_persistent_mutated)
				except (RuntimeError, TypeError):
					pass
				self._document_persistent_mutation_connected = False
			try:
				self._document.set_scene(None)
			except (RuntimeError, TypeError):
				pass
		if self._view is not None:
			try:
				self._view.set_mode_manager(None)
			except (RuntimeError, TypeError):
				pass
			try:
				self._view.set_document(None)
			except (RuntimeError, TypeError):
				pass
			try:
				self._view.setScene(None)
			except (RuntimeError, TypeError):
				pass
		if self._mode_manager is not None:
			try:
				self._mode_manager.dispose()
			except (RuntimeError, TypeError):
				pass
			try:
				self._mode_manager.setParent(None)
				self._mode_manager.deleteLater()
			except (RuntimeError, TypeError):
				pass
		for child in tuple(self.children()):
			if child in (self._document, self._scene, self._mode_manager):
				continue
			dispose = getattr(child, "dispose", None)
			if callable(dispose):
				try:
					dispose()
				except (RuntimeError, TypeError):
					pass
			try:
				child.setParent(None)
				child.deleteLater()
			except (RuntimeError, TypeError):
				pass
		if self._scene is not None:
			try:
				self._scene.dispose_contents(self._projection_retirement_reaper)
			except (RuntimeError, TypeError):
				pass
			finally:
				# A constructor that never returns has no session-close owner.  Move
				# any explicit native-delete failure into the process reaper rather
				# than allowing its wrapper to reach Python finalization.
				from ferrum_qt.canvas.graphics_retirement import (
					detached_graphics_retirement_reaper,
				)
				detached_graphics_retirement_reaper.retain_graphics_records(
					self._projection_retirement_reaper.take_retained_graphics_records(),
				)
			try:
				self._scene.setParent(None)
				self._scene.deleteLater()
			except (RuntimeError, TypeError):
				pass
		if self._view is not None:
			try:
				self._view.setParent(None)
				self._view.deleteLater()
			except (RuntimeError, TypeError):
				pass
		if self._document is not None:
			try:
				self._document.setParent(None)
			except (RuntimeError, TypeError):
				pass
			if self._document is not staged_document:
				try:
					self._document.deleteLater()
				except (RuntimeError, TypeError):
					pass
		self._document = None
		self._scene = None
		self._view = None
		self._mode_manager = None
		try:
			self.setParent(None)
			self.deleteLater()
		except (RuntimeError, TypeError):
			pass

	# ------------------------------------------------------------------
	# Import request and worker lifetime
	# ------------------------------------------------------------------

	#============================================
	def begin_import_request(self) -> int:
		"""Invalidate earlier imports and return this request's session token."""
		self._import_generation += 1
		return self._import_generation

	#============================================
	def invalidate_import_requests(self) -> None:
		"""Prevent all prior asynchronous callbacks from changing this session."""
		self._import_generation += 1

	#============================================
	def import_request_is_current(self, token: int) -> bool:
		"""Return whether an import result may still be delivered here."""
		return not self._disposed and token == self._import_generation

	#============================================
	def track_import_worker(self, worker: PySide6.QtCore.QThread) -> None:
		"""Retain a live worker until its native thread has finished."""
		if self._disposed:
			worker.requestInterruption()
			_adopt_orphaned_import_worker(worker)
			return
		self._import_workers.add(worker)

	#============================================
	def retire_import_workers(self) -> tuple[PySide6.QtCore.QThread, ...]:
		"""Invalidate delivery and surrender live workers to a retirement owner.

		Interruption is a truthful delivery fence only: opaque OASA, RDKit, and
		transport calls continue until their native call returns.  A live window
		must retain the returned workers and their relays through ``finished``.
		"""
		self.invalidate_import_requests()
		workers = tuple(self._import_workers)
		self._import_workers.clear()
		for worker in workers:
			worker.requestInterruption()
		return workers

	#============================================
	def release_import_worker(self, worker: PySide6.QtCore.QThread) -> None:
		"""Release one stopped worker and schedule its Qt wrapper for deletion."""
		self._import_workers.discard(worker)
		if not worker.isRunning():
			worker.deleteLater()

	# ------------------------------------------------------------------
	# Deterministic teardown
	# ------------------------------------------------------------------

	#============================================
	def dispose(self) -> None:
		"""Disconnect this tab's callbacks before Qt or Python wrappers die.

		This method is idempotent. It intentionally performs callback disposal
		before clearing undo history or the scene, because undone commands may
		be the final Python owners of off-scene graphics items.
		"""
		if self._disposed:
			return
		self._disposed = True
		self.disposed.emit()
		self.clear_projection_lifecycle_port()
		self._clear_mode_persistent_actions()
		self.invalidate_import_requests()
		self._stop_import_workers()

		self._mode_manager.dispose()
		if self._document_modified_connected and self._document is not None:
			try:
				self._document.modified_changed.disconnect(self._on_modified_changed)
			except (RuntimeError, TypeError):
				pass
			self._document_modified_connected = False
		if self._document_persistent_mutation_connected and self._document is not None:
			try:
				self._document.persistent_mutated.disconnect(self._on_persistent_mutated)
			except (RuntimeError, TypeError):
				pass
			self._document_persistent_mutation_connected = False
		self._view.set_mode_manager(None)
		self._view.set_document(None)
		self._view.setScene(None)
		graphics_error = None
		self._merge_retained_detached_graphics(
			self._projection_retirement_reaper.take_retained_detached_graphics(),
		)
		if self._document is not None:
			self._document.set_scene(None)
			try:
				self._dispose_graphics_items()
			except Exception as exc:
				graphics_error = exc
				self._teardown_diagnostics.append(exc)
			self._document.undo_stack.clear()
		self._teardown_phase = "callbacks_detached"
		scene_error = None
		try:
			self._scene.dispose_contents(self._projection_retirement_reaper)
		except Exception as exc:
			# A coordinator-recorded native deletion failure already has a
			# session-owned reaper record.  The remaining scene has crossed its
			# terminal transition, so finish queuing the session and transfer that
			# explicit record to MainWindow.  Other scene failures still stop here:
			# they have no safe terminal ownership proof.
			if not self._projection_retirement_reaper.has_retained_graphics:
				self._teardown_diagnostics.append(exc)
				raise RuntimeError("Session scene retirement did not complete") from exc
			self._merge_retained_detached_graphics(
				self._projection_retirement_reaper.take_retained_detached_graphics(),
			)
			scene_error = exc
			self._teardown_diagnostics.append(exc)
		self._teardown_phase = "scene_retired"
		if self._document is not None:
			# Clear model ownership only after the scene has explicitly retired its
			# graphics. Document.clear() detaches molecule/presentation QObjects so
			# deleting the document cannot move the same parent-cascade hazard there.
			self._document.clear()

		# Python-wrapped QGraphicsScene children can crash Shiboken when they are
		# destroyed recursively by a Python-wrapped QObject parent.  Break that
		# cascade and queue each independent root while its Python wrapper remains
		# retained by this terminal session.  MainWindow queues the now-childless
		# session only after dispose() returns.
		self._mode_manager.setParent(None)
		self._scene.setParent(None)
		self._mode_manager.deleteLater()
		if self._document is not None:
			self._document.setParent(None)
			self._document.deleteLater()
		self._scene.deleteLater()

		# The tab page was normally detached from QTabWidget by MainWindow.
		# Reparent defensively so direct DocumentSession users get the same
		# single-owner teardown contract.
		self._view.setParent(None)
		self._view.deleteLater()
		self._teardown_phase = "roots_queued"
		if graphics_error is not None:
			raise RuntimeError(
				"Session was retired after a graphics callback disposal failure",
			) from graphics_error
		if scene_error is not None:
			raise RuntimeError(
				"Session was retired after a scene graphics retirement failure",
			) from scene_error

	#============================================
	def release_python_references(self) -> None:
		"""Flatten the terminal wrapper graph after a reaper retains its roots.

		Native objects have already been queued for deletion by :meth:`dispose`.
		A caller retains QObject roots and any failed detached-graphics record
		before calling this method.  Scene-owned item sentinels were already
		released by :meth:`ChemScene.dispose_contents`.
		"""
		if self._teardown_phase != "roots_queued":
			raise RuntimeError(
				"Session roots must be queued before releasing Python references",
			)
		self._mode_manager.release_python_references()
		if self._document is not None:
			self._document._undo_stack = None
		self._mode_manager = None
		self._document = None
		self._scene = None
		self._view = None

	#============================================
	def take_retained_detached_graphics(self) -> object:
		"""Transfer failed detached graphics to the MainWindow terminal reaper."""
		self._merge_retained_detached_graphics(
			self._projection_retirement_reaper.take_retained_detached_graphics(),
		)
		retained = self._retained_detached_graphics
		self._retained_detached_graphics = None
		return retained

	#============================================
	def take_retained_graphics_records(self) -> object:
		"""Transfer every terminal graphics record to the MainWindow owner.

		The aggregate keeps failed scene-removal records together with detached
		root failures, so closing a session never changes their ownership to the
		process-level fallback while the MainWindow can still retry them.
		"""
		records = self._projection_retirement_reaper.take_retained_graphics_records()
		self._merge_retained_detached_graphics(records.detached)
		records.detached = self._retained_detached_graphics
		self._retained_detached_graphics = None
		return records

	#============================================
	def _merge_retained_detached_graphics(self, retained: object) -> None:
		"""Keep every failed projection root under this session's terminal owner."""
		if retained is None:
			return
		if self._retained_detached_graphics is None:
			self._retained_detached_graphics = retained
			return
		self._retained_detached_graphics.roots.extend(retained.roots)
		self._retained_detached_graphics.diagnostics.extend(retained.diagnostics)

	#============================================
	def _stop_import_workers(self) -> None:
		"""Invalidate local worker delivery without joining native work.

		This fallback is only safe when no worker was started during failed
		construction.  Registered sessions transfer workers to MainWindow before
		disposal, which remains their terminal Qt owner.
		"""
		for worker in self.retire_import_workers():
			_adopt_orphaned_import_worker(worker)

	#============================================
	def _dispose_graphics_items(self) -> None:
		"""Disconnect live and undo-retained graphics callbacks in order."""
		from ferrum_qt.canvas.graphics_retirement import GraphicsRetirementCoordinator
		coordinator = GraphicsRetirementCoordinator()
		coordinator.prepare_scene_retirement(
			self._scene, self._document.undo_stack,
			destroy_detached_undo_items=True,
			reaper=self._projection_retirement_reaper,
		)
		coordinator.raise_if_callback_failed(
			"Session graphics callbacks were released after a disposal failure",
		)
