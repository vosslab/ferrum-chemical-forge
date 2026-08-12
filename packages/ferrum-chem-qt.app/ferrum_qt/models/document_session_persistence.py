"""Per-tab ownership and teardown boundary for Ferrum-Qt documents."""

# Standard Library
import os

# PIP3 modules

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
import oasa.cdml_document
import oasa.cdml_ftext
import oasa.cdml_render

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
_PREPARED_NATIVE_FACTORY_TOKEN = (
	ferrum_qt.models.document_session_support._PREPARED_NATIVE_FACTORY_TOKEN
)
_PREPARED_IMPORTED_FACTORY_TOKEN = (
	ferrum_qt.models.document_session_support._PREPARED_IMPORTED_FACTORY_TOKEN
)


class DocumentSessionPersistenceMixin:
	def write_backend_snapshot(self, file_path: str) -> oasa.cdml_document.CDMLSnapshot:
		"""Write one exact synchronized backend snapshot, then mark it saved."""
		self._require_live_persistent_operation()
		if not self.can_write_authoritative_snapshot:
			raise BackendProjectionOutOfSyncError(
				"Cannot save backend CDML while the Qt projection is not a current "
				"authoritative projection",
			)
		snapshot = self._backend_session.snapshot()
		if (
				self._projected_backend_snapshot != snapshot
				or self._document.persistent_generation != self._projected_persistent_generation
			):
			raise BackendProjectionOutOfSyncError(
				"Cannot save backend CDML after Qt-local persistent mutation",
			)
		_write_backend_snapshot(file_path, snapshot)
		try:
			saved_snapshot = self._backend_session.mark_saved(
				expected_revision=snapshot.revision,
			)
		except Exception as exc:
			raise BackendSnapshotPublicationError(
				"CDML target was atomically replaced and may contain the canonical "
				"snapshot, but backend saved-state marking failed; this Save attempt "
				"did not change the backend saved baseline",
			) from exc
		self._projected_backend_snapshot = saved_snapshot
		self._projected_persistent_generation = self._document.persistent_generation
		self._backend_projection_synchronized = True
		try:
			self._document.mark_clean()
		except Exception:
			# Publication and the backend saved baseline already succeeded.  Keep a
			# conservative dirty/ineligible projection rather than reporting a
			# completed Save as failed because local presentation cleanup faulted.
			pass
		return saved_snapshot

	#============================================
	@classmethod
	def prepare_native_cdml(cls, cdml_text: str) -> PreparedNativeCDML:
		"""Validate CDML and stage a detached projection without live mutation."""
		backend_session = oasa.cdml_document.CDMLDocumentSession.load(cdml_text)
		from ferrum_qt.io import cdml_document_io
		projection_snapshot = backend_session.projection_snapshot()
		document = cdml_document_io.hydrate_synchronized_cdml_document(
			projection_snapshot,
		)
		return PreparedNativeCDML(
			factory_token=_PREPARED_NATIVE_FACTORY_TOKEN,
			snapshot=projection_snapshot.snapshot,
			document=document,
		)

	#============================================
	@classmethod
	def prepare_imported_cdml(cls, cdml_text: str) -> PreparedImportedCDML:
		"""Stage imported external content against the backend empty baseline."""
		backend_session = oasa.cdml_document.CDMLDocumentSession.load_imported(cdml_text)
		from ferrum_qt.io import cdml_document_io
		projection_snapshot = backend_session.projection_snapshot()
		document = cdml_document_io.hydrate_synchronized_cdml_document(
			projection_snapshot,
		)
		return PreparedImportedCDML(
			factory_token=_PREPARED_IMPORTED_FACTORY_TOKEN,
			snapshot=projection_snapshot.snapshot,
			document=document,
		)

	# ------------------------------------------------------------------
	# Owned state and tab title
	# ------------------------------------------------------------------

	#============================================
	@property
	def document(self) -> ferrum_qt.models.document.Document | None:
		"""Return this session's live Qt projection and interaction model."""
		return self._document

	#============================================
	@property
	def has_live_projection(self) -> bool:
		"""Return whether this session can serve legacy Qt document operations."""
		return not self._disposed and self._document is not None

	#============================================
	@property
	def can_write_authoritative_snapshot(self) -> bool:
		"""Return whether this Qt projection may publish the backend snapshot.

		The predicate is intentionally total.  It proves controlled projection
		provenance; it never treats a Qt serializer as evidence that a locally
		edited document equals the backend-owned CDML.
		"""
		if (
				self._disposed
				or self._projection_replacing
				or self._projection_error is not None
				or self._backend_session is None
				or self._document is None
				or self._scene is None
				or self._view is None
				or self._projected_backend_snapshot is None
				or self._projected_persistent_generation is None
				or not self._backend_projection_synchronized
			):
			return False
		try:
			current_snapshot = self._backend_session.snapshot()
			return (
				self._view.document is self._document
				and self._document._scene is self._scene
				and self._projected_backend_snapshot == current_snapshot
				and self._document.dirty == current_snapshot.is_dirty
				and self._document.persistent_generation
				== self._projected_persistent_generation
			)
		except Exception:
			return False

	#============================================
	def _current_recovery_snapshot(self) -> oasa.cdml_document.CDMLSnapshot:
		"""Return one current snapshot or reject a terminal/malformed backend."""
		if self._disposed or self._backend_session is None:
			raise RuntimeError("Recovery Export requires a live backend session")
		try:
			snapshot = self._backend_session.snapshot()
		except Exception as exc:
			raise RuntimeError(
				"Recovery Export requires a readable backend snapshot",
			) from exc
		if not isinstance(snapshot, oasa.cdml_document.CDMLSnapshot):
			raise RuntimeError("Recovery Export requires an immutable backend snapshot")
		return snapshot

	#============================================
	@property
	def can_recovery_export(self) -> bool:
		"""Return whether this live session can publish one backend snapshot."""
		try:
			self._current_recovery_snapshot()
		except Exception:
			return False
		return True

	#============================================
	def close_state(self) -> CloseState:
		"""Return document-free facts that govern confirmation before disposal."""
		snapshot = self._current_recovery_snapshot()
		backend_unseen = (
			not self._backend_projection_synchronized
			or self._projected_backend_snapshot != snapshot
		)
		state = CloseState(
			backend_dirty=snapshot.is_dirty,
			backend_unseen=backend_unseen,
			legacy_local_pending=self._legacy_isolated,
			authoritative_save_eligible=self.can_write_authoritative_snapshot,
		)
		return state

	#============================================
	def export_backend_snapshot(self, file_path: str) -> oasa.cdml_document.CDMLSnapshot:
		"""Publish one exact backend snapshot without changing this session."""
		snapshot = self._current_recovery_snapshot()
		_write_backend_snapshot(file_path, snapshot)
		return snapshot

	#============================================
	@property
	def scene(self) -> object:
		"""Return this session's ChemScene."""
		return self._scene

	#============================================
	@property
	def view(self) -> object:
		"""Return the ChemView suitable for direct insertion into a tab."""
		return self._view

	#============================================
	@property
	def mode_manager(self) -> object:
		"""Return the ModeManager that dispatches this view's events."""
		return self._mode_manager

	#============================================
	@property
	def title(self) -> str:
		"""Return the visible tab title, including the unsaved marker."""
		file_path = self._origin_path
		if self._document is not None:
			file_path = self._document.file_path
		base_name = self._display_name
		if not base_name:
			if file_path:
				base_name = os.path.basename(file_path)
			elif self._document is None:
				base_name = "Projection Error"
			else:
				base_name = "Untitled"
		dirty = self._document.dirty if self._document is not None else True
		return base_name + (" *" if dirty else "")

	#============================================
	def set_file_path(self, file_path: str | None) -> None:
		"""Update the native path and notify tab hosts of the new title."""
		if self._document is None:
			raise ProjectionReplacementError(
				"Cannot change a file path while the Qt projection is unavailable",
			)
		self._document.file_path = file_path
		self._display_name = None
		if file_path is not None:
			self._origin_path = file_path
		self.title_changed.emit(self.title)

	#============================================
	@property
	def origin_path(self) -> str | None:
		"""Return the native, imported, or pending source path for deduplication."""
		return self._origin_path

	#============================================
	def set_origin_path(self, origin_path: str | None) -> None:
		"""Set or clear the source path used for duplicate-open detection."""
		self._origin_path = origin_path

	#============================================
	def set_display_name(self, display_name: str | None) -> None:
		"""Set an import/loading label without making it a native save path."""
		self._display_name = display_name
		self.title_changed.emit(self.title)

	#============================================
	@property
	def is_disposed(self) -> bool:
		"""Return whether deterministic teardown has already begun."""
		return self._disposed
