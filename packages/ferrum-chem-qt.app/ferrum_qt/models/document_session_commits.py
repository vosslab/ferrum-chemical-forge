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


class DocumentSessionCommitsMixin:
	def _commit_complete_candidate(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLCommit:
		"""Execute one validated complete-CDML operation through OASA."""
		if not isinstance(prepared.value, str):
			raise ValueError("Complete CDML operation requires a string candidate")
		commit = self._backend_session.commit(
			expected_revision=prepared.expected_revision,
			complete_cdml=prepared.value,
		)
		return commit

	#============================================
	def _commit_molecule_insertion(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLCommit:
		"""Execute one validated molecule proposal through OASA composition."""
		if not isinstance(
				prepared.value, oasa.cdml_document.CDMLMoleculeInsertionRequest,
			):
			raise ValueError("Molecule insertion requires a molecule insertion request")
		commit = self._backend_session.insert_molecules(prepared.value)
		return commit

	#============================================
	def _commit_user_template_insertion(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLCommit:
		"""Submit one prepared serialized user-template insertion to OASA."""
		if type(prepared.value) is not oasa.cdml_document.CDMLUserTemplateInsertionRequest:
			raise ValueError("User template insertion requires an exact insertion request")
		return self._backend_session.insert_user_template(prepared.value)

	#============================================
	def _commit_geometry_repair(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLGeometryRepairResult:
		"""Execute one backend-owned geometry repair without a Qt candidate."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLGeometryRepairRequest):
			raise ValueError("Geometry repair requires a geometry repair request")
		return self._backend_session.repair_geometry(prepared.value)

	#============================================
	def _commit_atom_align(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLAtomAlignResult:
		"""Execute one backend-owned direct atom alignment."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLAtomAlignRequest):
			raise ValueError("Atom alignment requires an atom alignment request")
		return self._backend_session.align_atoms(prepared.value)

	#============================================
	def _commit_atom_translate(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLAtomTranslateResult:
		"""Execute one backend-owned direct atom translation."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLAtomTranslateRequest):
			raise ValueError("Atom translation requires an atom translation request")
		return self._backend_session.translate_atoms(prepared.value)

	#============================================
	def _commit_selection_translate(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLSelectionTranslateResult:
		"""Execute one backend-owned mixed atom/presentation translation."""
		if type(prepared.value) is not oasa.cdml_document.CDMLSelectionTranslateRequest:
			raise ValueError("Selection translation requires an exact translation request")
		return self._backend_session.translate_selection(prepared.value)

	#============================================
	def _commit_atom_rotate(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLAtomRotateResult:
		"""Execute one backend-owned direct atom rotation."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLAtomRotateRequest):
			raise ValueError("Atom rotation requires an atom rotation request")
		return self._backend_session.rotate_atoms(prepared.value)

	#============================================
	def _commit_bond_order_edit(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLBondOrderEditResult:
		"""Execute one backend-owned exact bond-order edit."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLBondOrderEditRequest):
			raise ValueError("Bond order requires a bond order edit request")
		return self._backend_session.set_bond_order(prepared.value)

	#============================================
	def _commit_bond_type_edit(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLBondTypeEditResult:
		"""Execute one backend-owned exact bond-type edit."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLBondTypeEditRequest):
			raise ValueError("Bond type requires a bond type edit request")
		return self._backend_session.set_bond_type(prepared.value)

	#============================================
	def _commit_bond_properties_patch(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLBondPropertiesPatchResult:
		"""Execute one backend-owned explicit bond-properties patch."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLBondPropertiesPatch):
			raise ValueError("Bond properties requires a bond properties patch")
		return self._backend_session.patch_bond_properties(prepared.value)

	#============================================
	def _commit_structural_edit(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLStructuralEditResult:
		"""Execute one backend-owned Draw-mode operation without a CDML candidate."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLStructuralEditRequest):
			raise ValueError("Draw structure requires a structural edit request")
		return self._backend_session.edit_structure(prepared.value)

	#============================================
	def _commit_atom_element_edit(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLCommit:
		"""Execute one backend-owned AtomMode element substitution."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLAtomElementEditRequest):
			raise ValueError("Atom element requires an element edit request")
		return self._backend_session.set_atom_element(prepared.value)

	#============================================
	def _commit_atom_properties_patch(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLAtomPropertiesPatchResult:
		"""Execute one backend-owned explicit atom-properties patch."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLAtomPropertiesPatch):
			raise ValueError("Atom properties requires an atom properties patch")
		return self._backend_session.patch_atom_properties(prepared.value)

	#============================================
	def _commit_text_properties_patch(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLTextPropertiesPatchResult:
		"""Execute one backend-owned explicit plain Text-properties patch."""
		if type(prepared.value) is not oasa.cdml_document.CDMLTextPropertiesPatch:
			raise ValueError("Text properties requires an exact Text properties patch")
		return self._backend_session.patch_text_properties(prepared.value)

	#============================================
	def _commit_rich_text_patch(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLRichTextPatchResult:
		"""Execute one backend-owned authored rich Text patch."""
		if type(prepared.value) is not oasa.cdml_document.CDMLRichTextPatch:
			raise ValueError("Rich Text requires an exact rich Text patch")
		return self._backend_session.patch_rich_text(prepared.value)

	#============================================
	def _commit_plus_properties_patch(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLPlusPropertiesPatchResult:
		"""Execute one backend-owned explicit plain Plus-properties patch."""
		if type(prepared.value) is not oasa.cdml_document.CDMLPlusPropertiesPatch:
			raise ValueError("Plus properties requires an exact Plus properties patch")
		return self._backend_session.patch_plus_properties(prepared.value)

	#============================================
	def _commit_wavy_properties_patch(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLWavyPropertiesPatchResult:
		"""Execute one backend-owned explicit plain Wavy-properties patch."""
		if type(prepared.value) is not oasa.cdml_document.CDMLWavyPropertiesPatch:
			raise ValueError("Wavy properties requires an exact Wavy properties patch")
		return self._backend_session.patch_wavy_properties(prepared.value)

	#============================================
	def _commit_fragment_create(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLFragmentCreateResult:
		"""Execute one backend-owned ordinary fragment creation."""
		if type(prepared.value) is not oasa.cdml_document.CDMLFragmentCreateRequest:
			raise ValueError("Fragment creation requires an exact fragment request")
		return self._backend_session.create_fragment(prepared.value)

	#============================================
	def _commit_fragment_delete(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLFragmentDeleteResult:
		"""Execute one backend-owned ordinary fragment deletion."""
		if type(prepared.value) is not oasa.cdml_document.CDMLFragmentDeleteRequest:
			raise ValueError("Fragment deletion requires an exact fragment request")
		return self._backend_session.delete_fragment(prepared.value)

	#============================================
	def _commit_implicit_group_expand(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLImplicitGroupExpandResult:
		"""Execute one backend-owned implicit-group expansion."""
		if type(prepared.value) is not oasa.cdml_document.CDMLImplicitGroupExpandRequest:
			raise ValueError("Implicit group expansion requires an exact request")
		return self._backend_session.expand_implicit_group(prepared.value)

	#============================================
	def _commit_linear_form_convert(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLLinearFormConvertResult:
		"""Execute one backend-owned atom-path linear-form conversion."""
		if type(prepared.value) is not oasa.cdml_document.CDMLLinearFormConvertRequest:
			raise ValueError("Linear form requires an exact conversion request")
		return self._backend_session.convert_linear_form(prepared.value)

	#============================================
	def _commit_atom_mark_operation(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLAtomMarkOperationResult:
		"""Execute one backend-owned atom-mark add or removal."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLAtomMarkOperationRequest):
			raise ValueError("Atom mark requires an atom-mark operation request")
		return self._backend_session.apply_atom_mark(prepared.value)

	#============================================
	def _commit_atom_number_edit(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLCommit:
		"""Execute one backend-owned atom-number assignment or clear."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLAtomNumberEditRequest):
			raise ValueError("Atom number requires an atom number edit request")
		return self._backend_session.set_atom_number(prepared.value)

	#============================================
	def _commit_molecule_name_edit(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLCommit:
		"""Execute one backend-owned direct-root molecule display-name edit."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLMoleculeNameEditRequest):
			raise ValueError("Molecule name requires a molecule name edit request")
		return self._backend_session.set_molecule_name(prepared.value)

	#============================================
	def _commit_paper_properties_patch(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLCommit:
		"""Apply one backend-owned explicit paper-properties patch."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLPaperPropertiesPatch):
			raise ValueError("Paper properties requires a paper properties patch")
		return self._backend_session.patch_paper_properties(prepared.value)

	#============================================
	def _commit_top_level_delete(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLCommit:
		"""Execute one backend-owned direct-root deletion."""
		if not isinstance(prepared.value, oasa.cdml_document.CDMLTopLevelDeleteRequest):
			raise ValueError("Top-level Delete requires a deletion request")
		return self._backend_session.delete_top_level(prepared.value)

	#============================================
	def _commit_structure_delete(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLStructureDeleteResult:
		"""Execute one backend-owned partial atom/bond deletion."""
		if type(prepared.value) is not oasa.cdml_document.CDMLStructureDeleteRequest:
			raise ValueError("Structure Delete requires an exact deletion request")
		result = self._backend_session.delete_structure(prepared.value)
		if type(result) is not oasa.cdml_document.CDMLStructureDeleteResult:
			raise ValueError("Structure Delete requires an exact deletion result")
		return result

	#============================================
	def _commit_top_level_transform(
			self, prepared: _PreparedPersistentOperation,
			) -> oasa.cdml_document.CDMLTopLevelTransformResult:
		"""Execute one backend-owned durable-root affine transform."""
		if type(prepared.value) is not oasa.cdml_document.CDMLTopLevelTransformRequest:
			raise ValueError("Top-level transform requires a transform request")
		return self._backend_session.apply_top_level_transform(prepared.value)

	#============================================
