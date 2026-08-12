"""Per-tab ownership and teardown boundary for Ferrum-Qt documents."""

# Standard Library

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


class DocumentSessionProjectionMixin:
	def _record_accepted_history(self, label: str, revision: int) -> None:
		"""Append an accepted edit after dropping logical redo entries."""
		self._backend_history = self._backend_history.append_accepted(label, revision)

	#============================================
	def _durable_selection_keys(
			self, prepared: _PreparedPersistentOperation,
			commit: oasa.cdml_document.CDMLCommit,
			) -> tuple[frozenset[tuple[str, str]], str | None]:
		"""Translate optional proposal tokens only to accepted direct-root records."""
		if not prepared.provisional_selection_keys:
			return frozenset(), None
		if prepared.executor_key in (
			"atom-align", "atom-translate", "selection-translate", "atom-rotate", "bond-order-edit", "bond-type-edit",
			"bond-properties-patch", "atom-properties-patch", "text-properties-patch",
			"rich-text-patch",
			"plus-properties-patch",
			"wavy-properties-patch",
			"atom-mark-operation",
			"fragment-create", "fragment-delete",
			"linear-form-convert",
			"top-level-transform",
			):
			# These direct-core edits preserve durable IDs; retain only their immutable
			# target selections across the replacement projection.
			return prepared.provisional_selection_keys, None
		canonical_document = oasa.cdml_document.CDMLDocument.parse(
			commit.snapshot.cdml, validation="compat",
		)
		direct_root_keys = frozenset(
			(record.local_name, record.identifier)
			for record in canonical_document.objects()
			if record.identifier is not None
		)
		selection_keys = []
		for kind, identifier in prepared.provisional_selection_keys:
			if identifier not in commit.id_map:
				return frozenset(), (
					"Persistent edit was accepted but selection correlation is unavailable"
				)
			durable_identifier = commit.id_map[identifier]
			if not isinstance(durable_identifier, str) or not durable_identifier:
				return frozenset(), (
					"Persistent edit was accepted but selection correlation is unavailable"
				)
			if (kind, durable_identifier) not in direct_root_keys:
				return frozenset(), (
					"Persistent edit was accepted but selection correlation is unavailable"
				)
			selection_keys.append((kind, durable_identifier))
		return frozenset(selection_keys), None

	#============================================
	def _project_accepted_commit(
			self, commit: oasa.cdml_document.CDMLCommit, success_message: str,
			structural_result: oasa.cdml_document.CDMLStructuralEditResult | None = None,
			selection_keys: frozenset[tuple[str, str]] | None = None,
			selection_error: str | None = None,
			) -> PersistentActionOutcome:
		"""Project accepted backend state without ever rolling it back."""
		self._backend_projection_synchronized = False
		if selection_keys is not None:
			self._accepted_projection_selection = (
				commit.snapshot.revision, selection_keys,
			)
		port = self._projection_lifecycle_port
		if port is None:
			projected = ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult(
				ferrum_qt.models.projection_lifecycle.ProjectionLifecycleStatus.SESSION_UNAVAILABLE,
				ferrum_qt.models.projection_lifecycle.ProjectionLifecyclePhase.SESSION,
			)
		else:
			projected = port.project(commit.snapshot)
		if projected.installed:
			self._clear_accepted_projection_selection(commit.snapshot)
			if selection_error is not None:
				return PersistentActionOutcome(
					"selection-unavailable", selection_error, commit, True, structural_result,
				)
			return PersistentActionOutcome(
				"accepted", success_message, commit, True, structural_result,
			)
		return PersistentActionOutcome(
			"unavailable",
			"Persistent edit was accepted but its projection is unavailable; retry or reopen",
			commit, True, structural_result,
		)

	#============================================
	def _clear_accepted_projection_selection(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			) -> None:
		"""Drop a one-shot durable selection intent after its snapshot is projected."""
		selection = self._accepted_projection_selection
		if selection is not None and selection[0] == snapshot.revision:
			self._accepted_projection_selection = None

	#============================================
	def retry_current_backend_projection(self) -> PersistentActionOutcome:
		"""Rebuild exactly the current backend snapshot after a failed projection."""
		if self._legacy_isolated:
			return PersistentActionOutcome(
				"unavailable",
				"Qt-local edits are isolated; discard them before backend reprojection",
				None,
			)
		return self._retry_current_backend_projection()

	#============================================
	def _discard_legacy_and_retry_projection(self) -> PersistentActionOutcome:
		"""Rebuild from backend after a frontend has confirmed Qt-edit discard."""
		return self._retry_current_backend_projection()

	#============================================
	def _retry_current_backend_projection(self) -> PersistentActionOutcome:
		"""Run one exact snapshot reprojection after an explicit safe recovery."""
		if self._disposed or self._projection_lifecycle_port is None:
			return PersistentActionOutcome(
				"unavailable", "Document projection retry is unavailable", None,
			)
		snapshot = self.backend_snapshot
		projected = self._projection_lifecycle_port.project(snapshot)
		if not projected.installed:
			return PersistentActionOutcome(
				"unavailable", "Document projection retry is unavailable", None,
			)
		self._legacy_isolated = False
		self._clear_accepted_projection_selection(snapshot)
		return PersistentActionOutcome("accepted", "Backend projection restored", None)

	#============================================
	def undo_backend(self) -> PersistentActionOutcome:
		"""Restore the predecessor logical history entry through OASA."""
		return self._restore_backend_navigation("undo")

	#============================================
	def redo_backend(self) -> PersistentActionOutcome:
		"""Restore the successor logical history entry through OASA."""
		return self._restore_backend_navigation("redo")

	#============================================
	def _restore_backend_navigation(self, direction: str) -> PersistentActionOutcome:
		"""Restore one adjacent entry and replace only its physical revision."""
		if not self.can_commit_persistent_action:
			return PersistentActionOutcome(
				"unavailable", "Backend %s is unavailable" % direction, None,
			)
		target = self._backend_history.adjacent_target(direction)
		if target is None:
			return PersistentActionOutcome(
				"unavailable", "Backend %s is unavailable" % direction, None,
			)
		destination, entry = target
		before_revision = self.backend_snapshot.revision
		try:
			commit = self._backend_session.restore(
				target_revision=entry.revision, expected_revision=before_revision,
			)
		except oasa.cdml_document.CDMLRevisionUnavailableError as exc:
			return PersistentActionOutcome("unavailable", str(exc), None)
		except oasa.cdml_document.CDMLDocumentError as exc:
			return PersistentActionOutcome("rejected", str(exc), None)
		self._backend_history = self._backend_history.record_restored(
			destination, commit.snapshot.revision,
		)
		success_message = "%s %s" % (
			entry.label,
			"undone" if direction == "undo" else "redone",
		)
		return self._project_accepted_commit(commit, success_message)

	#============================================
	def replace_projection_from_backend_snapshot(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			) -> ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult:
		"""Replace this Qt projection from one exact current backend snapshot.

		Only a snapshot returned by this session's current backend authority can
		be installed.  The requested current snapshot is prepared before any live
		Qt projection is retired; an accepted backend revision is never rolled back
		to an older displayed projection after a Qt failure.
		"""
		if (
				self._disposed
				or self._projection_replacing
				or snapshot != self.backend_snapshot
			):
			return ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult(
				ferrum_qt.models.projection_lifecycle.ProjectionLifecycleStatus.SESSION_UNAVAILABLE,
				ferrum_qt.models.projection_lifecycle.ProjectionLifecyclePhase.SESSION,
			)
		from ferrum_qt.io import cdml_document_io
		try:
			projection_snapshot = self._backend_session.projection_snapshot()
			if projection_snapshot.snapshot != snapshot:
				raise ValueError("backend projection envelope does not match the requested snapshot")
			candidate = cdml_document_io.prepare_synchronized_projection(
				projection_snapshot, self._projection_retirement_reaper,
			)
		except Exception as exc:
			self._backend_projection_synchronized = False
			self._projection_error = ProjectionReplacementError(
				"Could not prepare the current backend CDML projection",
			)
			self._projection_error.__cause__ = exc
			self.title_changed.emit(self.title)
			return ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult(
				ferrum_qt.models.projection_lifecycle.ProjectionLifecycleStatus.PREPARATION_UNAVAILABLE,
				ferrum_qt.models.projection_lifecycle.ProjectionLifecyclePhase.PREPARATION,
				self._projection_error,
			)

		self._projection_replacing = True
		retirement_started = False
		result = None
		try:
			file_path = self._origin_path
			selected_keys = self._accepted_selection_keys_for_snapshot(snapshot)
			if self._document is not None:
				file_path = self._document.file_path
				# Validate immediately before both native selection boundaries.
				if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(self._scene):
					raise ProjectionReplacementError("Current projection scene is unavailable")
				if selected_keys is None:
					selected_keys = frozenset(
						key for key in (
							ferrum_qt.canvas.document_projection.persistent_selection_key(item)
							for item in self._scene.selectedItems()
						) if key is not None
					)
			if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(self._scene):
				raise ProjectionReplacementError("Current projection scene is unavailable")
			self._scene.clearSelection()
			if selected_keys is None:
				selected_keys = frozenset()
			retirement_started = self._document is not None
			if retirement_started:
				self._dispose_current_projection()
			self._install_prepared_projection(candidate, selected_keys, file_path, snapshot)
			self._projected_backend_snapshot = snapshot
			self._backend_projection_synchronized = True
			self._projection_error = None
			result = ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult(
				ferrum_qt.models.projection_lifecycle.ProjectionLifecycleStatus.INSTALLED,
				ferrum_qt.models.projection_lifecycle.ProjectionLifecyclePhase.COMPLETE,
			)
		except Exception as exc:
			try:
				self._dispose_prepared_projection(candidate)
			except Exception as cleanup_exc:
				# The failed candidate remains terminal frontend-only state.  Keep its
				# cleanup diagnostic without allowing it to replace the failure that
				# caused projection replacement to fail.
				self._teardown_diagnostics.append(cleanup_exc)
			self._backend_projection_synchronized = False
			phase = (
				ferrum_qt.models.projection_lifecycle.ProjectionLifecyclePhase.INSTALLATION
				if retirement_started
				else ferrum_qt.models.projection_lifecycle.ProjectionLifecyclePhase.RETIREMENT
			)
			status = (
				ferrum_qt.models.projection_lifecycle.ProjectionLifecycleStatus.INSTALLATION_FAILED
				if retirement_started
				else ferrum_qt.models.projection_lifecycle.ProjectionLifecycleStatus.PREPARATION_UNAVAILABLE
			)
			message = (
				"Current backend projection installation failed after retirement"
				if retirement_started else "Current projection replacement could not begin"
			)
			self._projection_error = ProjectionReplacementError(message)
			self._projection_error.__cause__ = exc
			if retirement_started:
				self._document = None
			self.title_changed.emit(self.title)
			result = ferrum_qt.models.projection_lifecycle.ProjectionLifecycleResult(
				status, phase, self._projection_error,
			)
		finally:
			self._projection_replacing = False
		return result

	#============================================
	def _accepted_selection_keys_for_snapshot(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			) -> frozenset[tuple[str, str]] | None:
		"""Return a pending accepted selection only for its exact backend snapshot."""
		selection = self._accepted_projection_selection
		if selection is None or selection[0] != snapshot.revision:
			return None
		return selection[1]

	#============================================
	def _dispose_current_projection(self) -> None:
		"""Terminally detach the current generation without scene furniture.

		This is deliberately a cleanup transaction, rather than an all-or-nothing
		series of calls.  Once replacement starts, no part of the old Qt document
		may remain available for recovery: recovery is always reconstructed from a
		backend snapshot.  Continue every independent teardown step after a
		callback failure, then re-raise the original diagnostic for the caller to
		record as a failed replacement.
		"""
		old_document = self._document
		if old_document is None:
			return
		first_error = None
		if self._document_modified_connected:
			try:
				old_document.modified_changed.disconnect(self._on_modified_changed)
			except Exception as exc:
				first_error = exc
			self._document_modified_connected = False
		if self._document_persistent_mutation_connected:
			try:
				old_document.persistent_mutated.disconnect(self._on_persistent_mutated)
			except Exception as exc:
				if first_error is None:
					first_error = exc
			self._document_persistent_mutation_connected = False
		try:
			old_document._dispose_document_graphics(self._projection_retirement_reaper)
		except Exception as exc:
			if first_error is None:
				first_error = exc
		try:
			old_document.undo_stack.clear()
		except Exception as exc:
			if first_error is None:
				first_error = exc
		try:
			old_document.set_scene(None)
		except Exception as exc:
			if first_error is None:
				first_error = exc
		try:
			self._view.set_document(None)
		except Exception as exc:
			if first_error is None:
				first_error = exc
		try:
			old_document.clear()
		except Exception as exc:
			if first_error is None:
				first_error = exc
		finally:
			# Never leave a partially cleared document parented to the session.
			# Deleting it later is safer than allowing a second projection to share
			# its models, callbacks, or QGraphicsItem wrappers.
			try:
				old_document.setParent(None)
			except Exception as exc:
				if first_error is None:
					first_error = exc
			try:
				old_document.deleteLater()
			except Exception as exc:
				if first_error is None:
					first_error = exc
			self._document = None
		if first_error is not None:
			raise ProjectionReplacementError(
				"Old Qt projection was detached after a disposal failure",
			) from first_error

	#============================================
	def _install_prepared_projection(
			self, prepared: object, selected_keys: frozenset[tuple[str, str]],
			file_path: str | None, projected_snapshot: oasa.cdml_document.CDMLSnapshot,
			) -> None:
		"""Install one fully prepared projection without decoding or serialization."""
		document = prepared.document
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(document):
			raise ProjectionReplacementError("Prepared Document wrapper is unavailable")
		document.file_path = file_path
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(document):
			raise ProjectionReplacementError("Prepared Document wrapper is unavailable")
		document.set_graphics_retirement_reaper(
			self._projection_retirement_reaper,
		)
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(document):
			raise ProjectionReplacementError("Prepared Document wrapper is unavailable")
		document.setParent(self)
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(document):
			raise ProjectionReplacementError("Prepared Document wrapper is unavailable")
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(self._scene):
			raise ProjectionReplacementError("Projection scene is unavailable")
		document.set_scene(self._scene)
		if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(self._view):
			raise ProjectionReplacementError("Projection view is unavailable")
		self._view.set_document(document)
		def add_scene_root(item: object, role: str) -> None:
			"""Cross one checked native scene-add boundary for a prepared root."""
			if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(self._scene):
				raise ProjectionReplacementError("Projection scene is unavailable")
			if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(item):
				raise ProjectionReplacementError("Prepared %s wrapper is unavailable" % role)
			self._scene.addItem(item)
		for _molecule, items in prepared.molecule_projections:
			for item in items:
				add_scene_root(item, "molecule")
		for item in prepared.presentation_items:
			add_scene_root(item, "presentation")
		for atom_item, mark_items in prepared.mark_parent_items:
			if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(atom_item):
				raise ProjectionReplacementError("Prepared mark parent wrapper is unavailable")
			for item in mark_items:
				if not ferrum_qt.canvas.graphics_retirement.is_valid_native_wrapper(item):
					raise ProjectionReplacementError("Prepared mark wrapper is unavailable")
				if item.parentItem() is not atom_item:
					raise ProjectionReplacementError("Prepared mark lost atom-parent ownership")
		projection_items = tuple(
			item
			for _molecule, items in prepared.molecule_projections
			for item in items
		) + tuple(prepared.presentation_items) + tuple(prepared.mark_items)
		document.register_current_projection_items(projection_items)
		if hasattr(self._scene, "apply_paper_model"):
			self._scene.apply_paper_model(document.paper)
		ferrum_qt.canvas.document_projection.synchronize_document_stack_z_order(
			document, self._scene,
		)
		ferrum_qt.canvas.document_projection.select_projected_persistent_keys(
			self._scene, selected_keys,
		)
		if projected_snapshot.is_dirty:
			document.mark_dirty()
		else:
			document.mark_clean()
		self._document = document
		document.modified_changed.connect(self._on_modified_changed)
		self._document_modified_connected = True
		document.persistent_mutated.connect(self._on_persistent_mutated)
		self._document_persistent_mutation_connected = True
		self._projected_backend_snapshot = projected_snapshot
		self._projected_persistent_generation = document.persistent_generation
		self._backend_projection_synchronized = True
		# Dirty state was established before this connection so backend-derived
		# dirtiness cannot invalidate the synchronization latch.  Publish the
		# replacement afterwards so registered tabs receive one title refresh.
		self.title_changed.emit(self.title)

	#============================================
	def _dispose_prepared_projection(self, prepared: object) -> None:
		"""Release an uninstalled or partially installed frontend-only bundle."""
		from ferrum_qt.io import cdml_document_io
		document = prepared.document
		if self._document_modified_connected and document is self._document:
			try:
				document.modified_changed.disconnect(self._on_modified_changed)
			except (RuntimeError, TypeError):
				pass
			self._document_modified_connected = False
		if self._document_persistent_mutation_connected and document is self._document:
			try:
				document.persistent_mutated.disconnect(self._on_persistent_mutated)
			except (RuntimeError, TypeError):
				pass
			self._document_persistent_mutation_connected = False
		if self._view.document is document:
			self._view.set_document(None)
		try:
			document.set_scene(None)
		except (RuntimeError, TypeError):
			pass
		cdml_document_io.dispose_prepared_projection(
			prepared, self._projection_retirement_reaper,
		)

	#============================================
