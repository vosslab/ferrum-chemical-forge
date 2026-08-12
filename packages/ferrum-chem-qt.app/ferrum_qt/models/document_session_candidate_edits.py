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


class DocumentSessionCandidateEditsMixin:
	def _build_atom_element_edit(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact AtomMode element substitution to the OASA request."""
		payload = dict(request.payload)
		if set(payload) != {
				"expected_revision", "molecule_id", "atom_id", "element",
			}:
			raise ValueError("Atom element payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Atom element expected_revision must be an integer")
		for field_name in ("molecule_id", "atom_id", "element"):
			value = payload[field_name]
			if not isinstance(value, str) or not value:
				raise ValueError("Atom element %s must be a nonempty string" % field_name)
		molecule_id = payload["molecule_id"]
		atom_id = payload["atom_id"]
		expected_target_keys = frozenset({
			("molecule", molecule_id), ("atom", atom_id),
		})
		if request.target_keys != expected_target_keys:
			raise ValueError("Atom element target keys must match durable edit targets")
		atom_element_request = oasa.cdml_document.CDMLAtomElementEditRequest(
			expected_revision=expected_revision,
			molecule_id=molecule_id,
			atom_id=atom_id,
			element=payload["element"],
		)
		return _PreparedPersistentOperation(
			"atom-element-edit", atom_element_request.expected_revision,
			atom_element_request,
		)

	#============================================
	def _build_atom_properties_patch(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact atom dialog intent to the OASA patch request."""
		payload = dict(request.payload)
		if set(payload) != {
				"expected_revision", "molecule_id", "atom_id", "changes",
			}:
			raise ValueError("Atom properties payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Atom properties expected_revision must be an integer")
		if payload["expected_revision"] != _snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Atom properties expected revision does not match the current snapshot",
			)
		for field_name in ("molecule_id", "atom_id"):
			if not isinstance(payload[field_name], str) or not payload[field_name]:
				raise ValueError("Atom properties %s must be a nonempty string" % field_name)
		if type(payload["changes"]) is not tuple:
			raise ValueError("Atom properties changes must be an immutable tuple")
		molecule_id = payload["molecule_id"]
		atom_id = payload["atom_id"]
		if request.target_keys != frozenset({("molecule", molecule_id), ("atom", atom_id)}):
			raise ValueError("Atom properties target keys must match durable edit targets")
		atom_request = oasa.cdml_document.CDMLAtomPropertiesPatch(**payload)
		return _PreparedPersistentOperation(
			"atom-properties-patch", atom_request.expected_revision, atom_request,
			frozenset({("atom", atom_id)}),
		)

	#============================================
	def _build_text_properties_patch(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact plain Text dialog intent to OASA's direct-root patch."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "text_id", "changes"}:
			raise ValueError("Text properties payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Text properties expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Text properties expected revision does not match the current snapshot",
			)
		text_id = payload["text_id"]
		if type(text_id) is not str or not text_id.strip():
			raise ValueError("Text properties text_id must contain a non-whitespace character")
		if type(payload["changes"]) is not tuple:
			raise ValueError("Text properties changes must be an immutable tuple")
		if request.target_keys != frozenset({("presentation", text_id)}):
			raise ValueError("Text properties target keys must match the durable Text target")
		text_request = oasa.cdml_document.CDMLTextPropertiesPatch(**payload)
		return _PreparedPersistentOperation(
			"text-properties-patch", text_request.expected_revision, text_request,
			frozenset({("presentation", text_id)}),
		)

	#============================================
	def _build_rich_text_patch(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind plain frontend runs to one OASA-only rich Text patch adapter."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "text_id", "runs", "changes"}:
			raise ValueError("Rich Text payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Rich Text expected_revision must be an integer")
		if expected_revision != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Rich Text expected revision does not match the current snapshot",
			)
		text_id = payload["text_id"]
		if type(text_id) is not str or not text_id.strip():
			raise ValueError("Rich Text text_id must contain a non-whitespace character")
		if request.target_keys != frozenset({("presentation", text_id)}):
			raise ValueError("Rich Text target keys must match the durable Text target")
		patch = rich_text_patch_from_plain_runs(
			expected_revision, text_id, payload["runs"], payload["changes"],
		)
		return _PreparedPersistentOperation(
			"rich-text-patch", patch.expected_revision, patch,
			frozenset({("presentation", text_id)}),
		)

	#============================================
	def _build_plus_properties_patch(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact plain Plus dialog intent to OASA's root patch."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "plus_id", "changes"}:
			raise ValueError("Plus properties payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Plus properties expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Plus properties expected revision does not match the current snapshot",
			)
		plus_id = payload["plus_id"]
		if type(plus_id) is not str or not plus_id.strip():
			raise ValueError("Plus properties plus_id must contain a non-whitespace character")
		if type(payload["changes"]) is not tuple:
			raise ValueError("Plus properties changes must be an immutable tuple")
		if request.target_keys != frozenset({("presentation", plus_id)}):
			raise ValueError("Plus properties target keys must match the durable Plus target")
		plus_request = oasa.cdml_document.CDMLPlusPropertiesPatch(**payload)
		return _PreparedPersistentOperation(
			"plus-properties-patch", plus_request.expected_revision, plus_request,
			frozenset({("presentation", plus_id)}),
		)

	#============================================
	def _build_wavy_properties_patch(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact plain Wavy dialog intent to OASA's root patch."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "wavy_id", "changes"}:
			raise ValueError("Wavy properties payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Wavy properties expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Wavy properties expected revision does not match the current snapshot",
			)
		wavy_id = payload["wavy_id"]
		if type(wavy_id) is not str or not wavy_id.strip():
			raise ValueError("Wavy properties wavy_id must contain a non-whitespace character")
		if type(payload["changes"]) is not tuple:
			raise ValueError("Wavy properties changes must be an immutable tuple")
		if request.target_keys != frozenset({("presentation", wavy_id)}):
			raise ValueError("Wavy properties target keys must match the durable Wavy target")
		wavy_request = oasa.cdml_document.CDMLWavyPropertiesPatch(**payload)
		return _PreparedPersistentOperation(
			"wavy-properties-patch", wavy_request.expected_revision, wavy_request,
			frozenset({("presentation", wavy_id)}),
		)

	#============================================
	def _build_fragment_create(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one selected molecule fragment intent to the OASA operation."""
		payload = dict(request.payload)
		if set(payload) != {
				"expected_revision", "molecule_id", "name", "fragment_type",
				"atom_ids", "bond_ids",
			}:
			raise ValueError("Fragment creation payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Fragment creation expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Fragment creation expected revision does not match the current snapshot",
			)
		for field_name in ("molecule_id", "name", "fragment_type"):
			if type(payload[field_name]) is not str:
				raise ValueError("Fragment creation %s must be a string" % field_name)
		for field_name in ("atom_ids", "bond_ids"):
			if type(payload[field_name]) is not tuple:
				raise ValueError("Fragment creation %s must be an immutable tuple" % field_name)
		molecule_id = payload["molecule_id"]
		atom_ids = payload["atom_ids"]
		bond_ids = payload["bond_ids"]
		expected_targets = frozenset({("molecule", molecule_id)}) | frozenset(
			("atom", atom_id) for atom_id in atom_ids
		) | frozenset(("bond", bond_id) for bond_id in bond_ids)
		if request.target_keys != expected_targets:
			raise ValueError("Fragment creation target keys must match durable selection")
		fragment_request = oasa.cdml_document.CDMLFragmentCreateRequest(**payload)
		return _PreparedPersistentOperation(
			"fragment-create", fragment_request.expected_revision, fragment_request,
			frozenset({("molecule", molecule_id)}),
		)

	#============================================
	def _build_fragment_delete(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one immutable ordinary fragment deletion target to OASA."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "molecule_id", "fragment_id"}:
			raise ValueError("Fragment deletion payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Fragment deletion expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Fragment deletion expected revision does not match the current snapshot",
			)
		molecule_id = payload["molecule_id"]
		fragment_id = payload["fragment_id"]
		if type(molecule_id) is not str or type(fragment_id) is not str:
			raise ValueError("Fragment deletion targets must be strings")
		if request.target_keys != frozenset({("molecule", molecule_id)}):
			raise ValueError("Fragment deletion target keys must match durable molecule")
		fragment_request = oasa.cdml_document.CDMLFragmentDeleteRequest(**payload)
		return _PreparedPersistentOperation(
			"fragment-delete", fragment_request.expected_revision, fragment_request,
			frozenset({("molecule", molecule_id)}),
		)

	#============================================
	def _build_implicit_group_expand(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one direct implicit-group target to the OASA transaction."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "molecule_id", "group_id"}:
			raise ValueError("Implicit group expansion payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Implicit group expansion expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Implicit group expansion expected revision does not match the current snapshot",
			)
		for field_name in ("molecule_id", "group_id"):
			if type(payload[field_name]) is not str or not payload[field_name]:
				raise ValueError("Implicit group expansion %s must be a durable ID" % field_name)
		molecule_id = payload["molecule_id"]
		group_id = payload["group_id"]
		if request.target_keys != frozenset({("molecule", molecule_id), ("group", group_id)}):
			raise ValueError("Implicit group expansion target keys must match durable targets")
		expand_request = oasa.cdml_document.CDMLImplicitGroupExpandRequest(**payload)
		return _PreparedPersistentOperation(
			"implicit-group-expand", expand_request.expected_revision, expand_request,
		)

	#============================================
	def _build_linear_form_convert(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one durable path intent to OASA's closed linear-form grammar."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "molecule_id", "atom_ids"}:
			raise ValueError("Linear form payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Linear form expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Linear form expected revision does not match the current snapshot",
			)
		molecule_id = payload["molecule_id"]
		atom_ids = payload["atom_ids"]
		if type(molecule_id) is not str or not molecule_id or type(atom_ids) is not tuple:
			raise ValueError("Linear form requires a durable molecule and immutable atom IDs")
		expected_targets = frozenset({("molecule", molecule_id)}) | frozenset(
			("atom", atom_id) for atom_id in atom_ids
		)
		if request.target_keys != expected_targets:
			raise ValueError("Linear form target keys must match durable selection")
		linear_request = oasa.cdml_document.CDMLLinearFormConvertRequest(**payload)
		return _PreparedPersistentOperation(
			"linear-form-convert", linear_request.expected_revision, linear_request,
			frozenset(("atom", atom_id) for atom_id in atom_ids),
		)

	#============================================
	def _build_atom_mark_operation(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact MarkMode intent to OASA's atom-mark operation."""
		payload = dict(request.payload)
		if set(payload) not in (
				{"expected_revision", "molecule_id", "atom_id", "action", "mark_type"},
				{
					"expected_revision", "molecule_id", "atom_id", "action", "mark_type",
					"matching_mark_index",
				},
			):
			raise ValueError("Atom mark payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Atom mark expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Atom mark expected revision does not match the current snapshot",
			)
		for field_name in ("molecule_id", "atom_id", "action", "mark_type"):
			value = payload[field_name]
			if not isinstance(value, str) or not value:
				raise ValueError("Atom mark %s must be a nonempty string" % field_name)
		molecule_id = payload["molecule_id"]
		atom_id = payload["atom_id"]
		if request.target_keys != frozenset({
				("molecule", molecule_id), ("atom", atom_id),
			}):
			raise ValueError("Atom mark target keys must match durable edit targets")
		mark_request = oasa.cdml_document.CDMLAtomMarkOperationRequest(**payload)
		return _PreparedPersistentOperation(
			"atom-mark-operation", mark_request.expected_revision, mark_request,
			frozenset({("atom", atom_id)}),
		)

	#============================================
	def _build_atom_number_edit(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact MiscMode number assignment or clear to OASA."""
		payload = dict(request.payload)
		if set(payload) != {
				"expected_revision", "molecule_id", "atom_id", "number", "show_number",
			}:
			raise ValueError("Atom number payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Atom number expected_revision must be an integer")
		for field_name in ("molecule_id", "atom_id"):
			value = payload[field_name]
			if not isinstance(value, str) or not value:
				raise ValueError("Atom number %s must be a nonempty string" % field_name)
		number = payload["number"]
		show_number = payload["show_number"]
		if number is None and show_number is None:
			pass
		elif type(number) is int and number > 0 and type(show_number) is bool:
			pass
		else:
			raise ValueError(
				"Atom number requires a positive integer and bool, or an exact clear pair",
			)
		molecule_id = payload["molecule_id"]
		atom_id = payload["atom_id"]
		expected_target_keys = frozenset({
			("molecule", molecule_id), ("atom", atom_id),
		})
		if request.target_keys != expected_target_keys:
			raise ValueError("Atom number target keys must match durable edit targets")
		atom_number_request = oasa.cdml_document.CDMLAtomNumberEditRequest(
			expected_revision=expected_revision,
			molecule_id=molecule_id,
			atom_id=atom_id,
			number=number,
			show_number=show_number,
		)
		return _PreparedPersistentOperation(
			"atom-number-edit", atom_number_request.expected_revision,
			atom_number_request,
		)

	#============================================
	def _build_molecule_name_edit(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact direct-root molecule display-name edit to OASA."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "molecule_id", "name"}:
			raise ValueError("Molecule name payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		molecule_id = payload["molecule_id"]
		name = payload["name"]
		if type(expected_revision) is not int:
			raise ValueError("Molecule name expected_revision must be an integer")
		if not isinstance(molecule_id, str) or not molecule_id:
			raise ValueError("Molecule name molecule_id must be a nonempty string")
		if not isinstance(name, str):
			raise ValueError("Molecule name name must be a string")
		if request.target_keys != frozenset({("molecule", molecule_id)}):
			raise ValueError("Molecule name target keys must match durable root target")
		name_request = oasa.cdml_document.CDMLMoleculeNameEditRequest(
			expected_revision=expected_revision, molecule_id=molecule_id, name=name,
		)
		return _PreparedPersistentOperation(
			"molecule-name-edit", name_request.expected_revision, name_request,
		)

	#============================================
	def _structural_target_keys(
			self, kind: str, payload: dict[str, object],
			) -> frozenset[tuple[str, str]]:
		"""Return durable target identities for one exact structural operation."""
		if kind == "create-bonded-pair":
			return frozenset()
		molecule_id = payload["molecule_id"]
		if not isinstance(molecule_id, str):
			raise ValueError("Draw structure molecule_id must be a nonempty durable ID")
		target_keys = {("molecule", molecule_id)}
		if kind == "apply-bond-tool":
			bond_id = payload["bond_id"]
			if not isinstance(bond_id, str):
				raise ValueError("Draw structure bond_id must be a nonempty durable ID")
			target_keys.add(("bond", bond_id))
		else:
			source_atom_id = payload["source_atom_id"]
			if not isinstance(source_atom_id, str):
				raise ValueError("Draw structure source_atom_id must be a nonempty durable ID")
			target_keys.add(("atom", source_atom_id))
			if kind == "join-atoms":
				target_atom_id = payload["target_atom_id"]
				if not isinstance(target_atom_id, str):
					raise ValueError(
						"Draw structure target_atom_id must be a nonempty durable ID",
					)
				target_keys.add(("atom", target_atom_id))
		return frozenset(target_keys)

	#============================================
	def _build_top_level_delete(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind a plain direct-root deletion request to the OASA executor."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "root_ids"}:
			raise ValueError("Top-level Delete payload has unsupported fields")
		root_ids = payload["root_ids"]
		if not isinstance(root_ids, tuple):
			raise ValueError("Top-level Delete root_ids must be an immutable tuple")
		if request.target_keys != frozenset(
			("molecule", identifier) for identifier in root_ids
		) and request.target_keys != frozenset(
			("presentation", identifier) for identifier in root_ids
		):
			# Mixed root families are represented by their durable IDs; require each
			# key to be one of the two direct-root families without leaking Qt types.
			if {
				identifier for _kind, identifier in request.target_keys
			} != set(root_ids) or any(
				kind not in {"molecule", "presentation"}
				for kind, _identifier in request.target_keys
			):
				raise ValueError("Top-level Delete target keys must match root IDs")
		delete_request = oasa.cdml_document.CDMLTopLevelDeleteRequest(
			expected_revision=payload["expected_revision"],
			root_ids=root_ids,
			label=request.label,
		)
		return _PreparedPersistentOperation(
			"top-level-delete", delete_request.expected_revision, delete_request,
		)

	#============================================
	def _build_structure_delete(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact partial atom/bond deletion to the OASA executor."""
		payload = dict(request.payload)
		if set(payload) != {
				"expected_revision", "molecule_id", "atom_ids", "bond_ids",
			}:
			raise ValueError("Structure Delete payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		molecule_id = payload["molecule_id"]
		atom_ids = payload["atom_ids"]
		bond_ids = payload["bond_ids"]
		if type(expected_revision) is not int:
			raise ValueError("Structure Delete expected_revision must be an integer")
		if expected_revision != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Structure Delete expected revision does not match the current snapshot",
			)
		if type(molecule_id) is not str or not molecule_id.strip():
			raise ValueError("Structure Delete molecule_id must be a nonblank durable ID")
		for identifiers in (atom_ids, bond_ids):
			if type(identifiers) is not tuple:
				raise ValueError("Structure Delete target IDs must be immutable tuples")
			if any(
				type(identifier) is not str or not identifier.strip()
				for identifier in identifiers
			):
				raise ValueError("Structure Delete target IDs must be nonblank strings")
			if len(set(identifiers)) != len(identifiers):
				raise ValueError("Structure Delete target IDs must be unique")
		if not atom_ids and not bond_ids:
			raise ValueError("Structure Delete requires at least one atom or bond")
		if set(atom_ids).intersection(bond_ids):
			raise ValueError("Structure Delete atom and bond IDs must be distinct")
		expected_targets = (
			frozenset({("molecule", molecule_id)})
			| frozenset(("atom", identifier) for identifier in atom_ids)
			| frozenset(("bond", identifier) for identifier in bond_ids)
		)
		if request.target_keys != expected_targets:
			raise ValueError("Structure Delete target keys must match its durable targets")
		delete_request = oasa.cdml_document.CDMLStructureDeleteRequest(
			expected_revision=expected_revision,
			molecule_id=molecule_id,
			atom_ids=atom_ids,
			bond_ids=bond_ids,
			label=request.label,
		)
		return _PreparedPersistentOperation(
			"structure-delete", expected_revision, delete_request,
		)

	#============================================
	def _build_top_level_transform(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact durable-root transform request to OASA."""
		payload = dict(request.payload)
		expected_fields = {
			"expected_revision", "mode", "root_ids", "scale_x", "scale_y", "delta",
		}
		if set(payload) != expected_fields:
			raise ValueError("Top-level transform payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Top-level transform expected_revision must be an integer")
		if payload["expected_revision"] != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Top-level transform expected revision does not match the current snapshot",
			)
		root_ids = payload["root_ids"]
		if (
			type(root_ids) is not tuple or not root_ids
			or any(type(identifier) is not str or not identifier for identifier in root_ids)
			or len(set(root_ids)) != len(root_ids)
		):
			raise ValueError("Top-level transform root_ids must be unique nonempty strings")
		if request.target_keys != frozenset(
			(kind, identifier)
			for kind, identifier in request.target_keys
			if kind in {"molecule", "presentation"} and identifier in root_ids
		) or {identifier for _kind, identifier in request.target_keys} != set(root_ids):
			raise ValueError("Top-level transform target keys must match root IDs")
		canonical_document = oasa.cdml_document.CDMLDocument.parse(
			snapshot.cdml, validation="compat",
		)
		presentation_names = {
			"arrow", "text", "plus", "rect", "square", "oval", "circle",
			"polygon", "polyline",
		}
		canonical_root_keys = frozenset(
			("molecule", record.identifier)
			if record.local_name == "molecule"
			else ("presentation", record.identifier)
			for record in canonical_document.objects()
			if record.identifier is not None and (
				record.local_name == "molecule"
				or record.local_name in presentation_names
			)
		)
		if request.target_keys - canonical_root_keys:
			raise ValueError(
				"Top-level transform target keys must match authoritative root kinds",
			)
		transform_request = oasa.cdml_document.CDMLTopLevelTransformRequest(**payload)
		return _PreparedPersistentOperation(
			"top-level-transform", transform_request.expected_revision, transform_request,
			request.target_keys,
		)

	#============================================
