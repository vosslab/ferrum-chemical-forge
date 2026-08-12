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


class DocumentSessionSubmissionMixin:
	def commit_arrow(
			self, start: tuple[float, float], end: tuple[float, float],
			) -> PersistentActionOutcome:
		"""Adapt the established Arrow route to the generic request boundary."""
		request = PersistentOperationRequest(
			"arrow.add", "Arrow",
			(("start", tuple(start)), ("end", tuple(end))),
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_atom_align(
			self, axis: str, targets: tuple[tuple[str, str], ...],
			) -> PersistentActionOutcome:
		"""Submit durable atom alignment using this live session's snapshot."""
		self._require_live_persistent_operation()
		if not isinstance(targets, tuple):
			raise TypeError("Atom alignment targets must be an immutable tuple")
		request = build_atom_align_request(self.backend_snapshot.revision, axis, targets)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_top_level_transform(
			self, expected_revision: int, mode: str,
			root_keys: tuple[tuple[str, str], ...],
			scale_x: float | None = None, scale_y: float | None = None,
			delta: tuple[float, float] | None = None,
			) -> PersistentActionOutcome:
		"""Submit one durable mixed top-level transform through this session."""
		self._require_live_persistent_operation()
		request = build_top_level_transform_request(
			expected_revision, mode, root_keys, scale_x, scale_y, delta,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_atom_translate(
			self, targets: tuple[tuple[str, str], ...], delta: tuple[float, float],
			) -> PersistentActionOutcome:
		"""Submit one durable atom nudge using this live session's snapshot."""
		self._require_live_persistent_operation()
		if not isinstance(targets, tuple) or not isinstance(delta, tuple):
			raise TypeError("Atom translation targets and delta must be immutable tuples")
		request = build_atom_translate_request(self.backend_snapshot.revision, targets, delta)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_biomolecule_template(
			self, catalog_key: str, anchor: tuple[float, float],
			) -> PersistentActionOutcome:
		"""Submit one current revision-bound packaged biomolecule placement."""
		if self._disposed:
			return PersistentActionOutcome(
				"unavailable", "Document cannot accept a persistent edit", None, False,
			)
		self._require_live_persistent_operation()
		request = PersistentOperationRequest(
			"biotemplate.insert", "Place Biomolecule Template",
			(
				("expected_revision", self.backend_snapshot.revision),
				("catalog_key", catalog_key),
				("anchor", anchor),
			),
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_user_template(
			self, catalog_key: str, anchor: tuple[float, float],
			) -> PersistentActionOutcome:
		"""Submit one current revision-bound session-delivered saved template."""
		if self._disposed:
			return PersistentActionOutcome(
				"unavailable", "Document cannot accept a persistent edit", None, False,
			)
		self._require_live_persistent_operation()
		try:
			request = build_user_template_insert_request(
				self.backend_snapshot.revision, catalog_key, anchor,
			)
		except (TypeError, ValueError) as exc:
			return PersistentActionOutcome(
				"rejected", str(exc), None, False, None, "validation",
			)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_system_template(
			self, template_name: str, anchor: tuple[float, float],
			) -> PersistentActionOutcome:
		"""Submit one current revision-bound OASA system-template placement."""
		if self._disposed:
			return PersistentActionOutcome(
				"unavailable", "Document cannot accept a persistent edit", None, False,
			)
		self._require_live_persistent_operation()
		request = PersistentOperationRequest(
			"template.insert", "Place Template",
			(
				("expected_revision", self.backend_snapshot.revision),
				("template_name", template_name),
				("anchor", anchor),
			),
		)
		return self.submit_persistent_operation(request)


	#============================================
	def submit_selection_translate(
			self, expected_revision: int, atom_targets: tuple[tuple[str, str], ...],
			presentation_root_keys: tuple[tuple[str, str], ...], delta: tuple[float, float],
			) -> PersistentActionOutcome:
		"""Submit one press-revision-bound mixed selection translation."""
		self._require_live_persistent_operation()
		request = build_selection_translate_request(
			expected_revision, atom_targets, presentation_root_keys, delta,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_atom_rotate(
			self, targets: tuple[tuple[str, str], ...], center: tuple[float, float],
			angle_radians: float,
			) -> PersistentActionOutcome:
		"""Submit one durable 2D atom rotation using this live session snapshot."""
		self._require_live_persistent_operation()
		if not isinstance(targets, tuple) or not isinstance(center, tuple):
			raise TypeError("Atom rotation targets and center must be immutable tuples")
		request = build_atom_rotate_request(
			self.backend_snapshot.revision, targets, center, angle_radians,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_bond_order(
			self, molecule_id: str, bond_id: str, order: int,
			) -> PersistentActionOutcome:
		"""Submit one exact durable bond-order edit through this live session."""
		self._require_live_persistent_operation()
		request = build_bond_order_request(
			self.backend_snapshot.revision, molecule_id, bond_id, order,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_bond_type(
			self, molecule_id: str, bond_id: str, bond_type: str,
			) -> PersistentActionOutcome:
		"""Submit one exact durable bond-type edit through this live session."""
		self._require_live_persistent_operation()
		request = build_bond_type_request(
			self.backend_snapshot.revision, molecule_id, bond_id, bond_type,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_bond_properties_patch(
			self, expected_revision: int, molecule_id: str, bond_id: str,
			changes: tuple[tuple[str, object], ...],
			) -> PersistentActionOutcome:
		"""Submit one revision-bound durable bond-properties patch through this session."""
		self._require_live_persistent_operation()
		request = build_bond_properties_patch_request(
			expected_revision, molecule_id, bond_id, changes,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_atom_properties_patch(
			self, expected_revision: int, molecule_id: str, atom_id: str,
			changes: tuple[tuple[str, object], ...],
			) -> PersistentActionOutcome:
		"""Submit one revision-bound durable atom-properties patch through this session."""
		self._require_live_persistent_operation()
		request = build_atom_properties_patch_request(
			expected_revision, molecule_id, atom_id, changes,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_text_properties_patch(
			self, expected_revision: int, text_id: str,
			changes: tuple[tuple[str, object], ...],
			) -> PersistentActionOutcome:
		"""Submit one revision-bound durable plain Text patch through this session."""
		self._require_live_persistent_operation()
		request = build_text_properties_patch_request(
			expected_revision, text_id, changes,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_rich_text_patch(
			self, expected_revision: int, text_id: str,
			runs: tuple[tuple[str, tuple[str, ...]], ...],
			changes: tuple[tuple[str, object], ...] = (),
			) -> PersistentActionOutcome:
		"""Submit one revision-bound durable rich Text run patch through this session."""
		self._require_live_persistent_operation()
		request = build_rich_text_patch_request(expected_revision, text_id, runs, changes)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_plus_properties_patch(
			self, expected_revision: int, plus_id: str,
			changes: tuple[tuple[str, object], ...],
			) -> PersistentActionOutcome:
		"""Submit one revision-bound durable plain Plus patch through this session."""
		self._require_live_persistent_operation()
		request = build_plus_properties_patch_request(
			expected_revision, plus_id, changes,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_wavy_properties_patch(
			self, expected_revision: int, wavy_id: str,
			changes: tuple[tuple[str, object], ...],
			) -> PersistentActionOutcome:
		"""Submit one revision-bound durable plain Wavy patch through this session."""
		self._require_live_persistent_operation()
		request = build_wavy_properties_patch_request(
			expected_revision, wavy_id, changes,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_fragment_create(
			self, expected_revision: int, molecule_id: str, name: str,
			fragment_type: str, atom_ids: tuple[str, ...], bond_ids: tuple[str, ...],
			) -> PersistentActionOutcome:
		"""Submit one ordinary fragment metadata creation through this session."""
		self._require_live_persistent_operation()
		request = build_fragment_create_request(
			expected_revision, molecule_id, name, fragment_type, atom_ids, bond_ids,
		)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_fragment_delete(
			self, expected_revision: int, molecule_id: str, fragment_id: str,
			) -> PersistentActionOutcome:
		"""Submit one ordinary fragment metadata deletion through this session."""
		self._require_live_persistent_operation()
		request = build_fragment_delete_request(expected_revision, molecule_id, fragment_id)
		return self.submit_persistent_operation(request)

	#============================================
	def submit_implicit_group_expand(
			self, expected_revision: int, molecule_id: str, group_id: str,
			) -> PersistentActionOutcome:
		"""Submit one backend-authoritative implicit-group expansion."""
		self._require_live_persistent_operation()
		return self.submit_persistent_operation(build_implicit_group_expand_request(
			expected_revision, molecule_id, group_id,
		))

	#============================================
	def submit_linear_form_convert(
			self, expected_revision: int, molecule_id: str, atom_ids: tuple[str, ...],
			) -> PersistentActionOutcome:
		"""Submit one durable atom-path linear-form conversion."""
		self._require_live_persistent_operation()
		return self.submit_persistent_operation(build_linear_form_convert_request(
			expected_revision, molecule_id, atom_ids,
		))

	#============================================
	def submit_persistent_operation(
			self, request: PersistentOperationRequest,
			) -> PersistentActionOutcome:
		"""Dispatch, commit, record, and project one immutable plain request."""
		if not isinstance(request, PersistentOperationRequest):
			raise TypeError("Persistent operations require PersistentOperationRequest")
		if not self.can_commit_persistent_action:
			return PersistentActionOutcome(
				"unavailable", "Document cannot accept a persistent edit", None, False,
			)
		builder = self._operation_dispatcher.get(request.operation_key)
		if builder is None:
			return PersistentActionOutcome(
				"rejected", "Unsupported persistent operation: %s" % request.operation_key,
				None, False,
			)
		snapshot = self.backend_snapshot
		try:
			prepared = builder(snapshot, request)
			if (
					prepared.executor_key == "complete-candidate"
					and prepared.value == snapshot.cdml
				):
				return PersistentActionOutcome(
					"accepted", "%s made no persistent change" % request.label,
					None, True,
				)
			executor = self._operation_commit_executors[prepared.executor_key]
			execution_result = executor(prepared)
		except oasa.cdml_document.CDMLRevisionConflictError as exc:
			return PersistentActionOutcome(
				"rejected", str(exc), None, False, None, "revision-conflict",
			)
		except oasa.cdml_document.CDMLDocumentError as exc:
			return PersistentActionOutcome(
				"rejected", str(exc), None, False, None, "validation",
			)
		except ValueError as exc:
			return PersistentActionOutcome(
				"rejected", str(exc), None, False, None, "validation",
			)
		structural_result = None
		if isinstance(
				execution_result,
				(
					oasa.cdml_document.CDMLGeometryRepairResult,
					oasa.cdml_document.CDMLAtomAlignResult,
					oasa.cdml_document.CDMLAtomTranslateResult,
					oasa.cdml_document.CDMLSelectionTranslateResult,
					oasa.cdml_document.CDMLAtomRotateResult,
					oasa.cdml_document.CDMLBondOrderEditResult,
					oasa.cdml_document.CDMLBondTypeEditResult,
					oasa.cdml_document.CDMLBondPropertiesPatchResult,
					oasa.cdml_document.CDMLAtomPropertiesPatchResult,
					oasa.cdml_document.CDMLTextPropertiesPatchResult,
					oasa.cdml_document.CDMLRichTextPatchResult,
					oasa.cdml_document.CDMLPlusPropertiesPatchResult,
					oasa.cdml_document.CDMLWavyPropertiesPatchResult,
					oasa.cdml_document.CDMLAtomMarkOperationResult,
					oasa.cdml_document.CDMLTopLevelTransformResult,
					oasa.cdml_document.CDMLLinearFormConvertResult,
				),
			):
			if not execution_result.changed:
				return PersistentActionOutcome(
					"accepted", "%s made no persistent change" % request.label,
					None, True,
				)
			commit = execution_result.commit
			if commit is None:
				raise RuntimeError("Changed persistent operation requires an accepted commit")
		elif type(execution_result) is oasa.cdml_document.CDMLStructureDeleteResult:
			commit = execution_result.commit
		elif type(execution_result) in (
				oasa.cdml_document.CDMLFragmentCreateResult,
				oasa.cdml_document.CDMLFragmentDeleteResult,
				oasa.cdml_document.CDMLImplicitGroupExpandResult,
		):
			commit = execution_result.commit
		elif isinstance(execution_result, oasa.cdml_document.CDMLStructuralEditResult):
			commit = execution_result.commit
			structural_result = execution_result
		else:
			commit = execution_result
		if _is_unchanged_authoritative_snapshot(snapshot, commit.snapshot):
			return PersistentActionOutcome(
				"accepted", f"{request.label} made no persistent change",
				None, True, structural_result,
			)
		self._record_accepted_history(request.label, commit.snapshot.revision)
		if prepared.preserve_existing_selection:
			selection_keys, selection_error = None, None
		elif type(execution_result) is oasa.cdml_document.CDMLImplicitGroupExpandResult:
			selection_keys, selection_error = frozenset({
				("atom", execution_result.replacement_atom_id),
			}), None
		else:
			selection_keys, selection_error = self._durable_selection_keys(prepared, commit)
		return self._project_accepted_commit(
			commit, "%s accepted" % request.label, structural_result, selection_keys,
			selection_error,
		)

	#============================================
	def extract_structure_fragment(
			self, expected_revision: int, molecule_id: str,
			atom_ids: tuple[str, ...], bond_ids: tuple[str, ...],
			) -> oasa.cdml_document.CDMLStructureFragmentExtractionResult:
		"""Read one backend-authoritative structural clipboard fragment."""
		self._require_live_persistent_operation()
		query = build_structure_fragment_extraction_query(
			expected_revision, molecule_id, atom_ids, bond_ids,
		)
		try:
			return self._backend_session.extract_structure_fragment(query)
		except oasa.cdml_document.CDMLDocumentError as exc:
			raise BackendFragmentExtractionError(str(exc)) from exc

	#============================================
	def extract_top_level_fragment(
			self, expected_revision: int, root_ids: tuple[str, ...],
			) -> oasa.cdml_document.CDMLTopLevelFragmentExtractionResult:
		"""Read one authoritative direct-root clipboard fragment."""
		self._require_live_persistent_operation()
		query = build_top_level_fragment_extraction_query(expected_revision, root_ids)
		try:
			return self._backend_session.extract_top_level_fragment(query)
		except oasa.cdml_document.CDMLDocumentError as exc:
			raise BackendFragmentExtractionError(str(exc)) from exc

	#============================================
	def submit_clipboard_fragment(self, fragment_cdml: str) -> PersistentActionOutcome:
		"""Commit one raw complete clipboard fragment through the OASA boundary."""
		if not self.can_commit_persistent_action:
			return PersistentActionOutcome(
				"unavailable", "Document cannot accept a persistent edit", None, False,
			)
		snapshot = self.backend_snapshot
		request = oasa.cdml_document.CDMLTopLevelInsertionRequest(
			expected_revision=snapshot.revision,
			fragment_cdml=fragment_cdml,
			translation=(20.0, 20.0),
			label="Paste",
		)
		try:
			commit = self._backend_session.insert_top_level(request)
		except oasa.cdml_document.CDMLDocumentError as exc:
			return PersistentActionOutcome("rejected", str(exc), None, False)
		except ValueError as exc:
			return PersistentActionOutcome("rejected", str(exc), None, False)
		self._record_accepted_history("Paste", commit.snapshot.revision)
		return self._project_accepted_commit(commit, "Pasted")

	#============================================
