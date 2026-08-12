"""Per-tab ownership and teardown boundary for Ferrum-Qt documents."""

# Standard Library
import dataclasses
import math
import numbers

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


class DocumentSessionCandidatesMixin:
	def _build_arrow_candidate(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
		) -> _PreparedPersistentOperation:
		"""Build the complete CDML candidate owned by the Arrow dispatcher key."""
		payload = dict(request.payload)
		start = payload["start"]
		end = payload["end"]
		if not isinstance(start, tuple) or not isinstance(end, tuple):
			raise ValueError("Arrow coordinates must be immutable coordinate tuples")
		candidate = ferrum_qt.io.cdml_candidate.append_arrow_candidate(
			snapshot.cdml, self._next_arrow_provisional_id(snapshot.revision), start, end,
		)
		return self._prepare_complete_candidate(snapshot.revision, candidate)

	#============================================
	def _build_text_candidate(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
		) -> _PreparedPersistentOperation:
		"""Build the complete CDML candidate owned by the Text dispatcher key."""
		payload = dict(request.payload)
		if set(payload) != {"text", "position"}:
			raise ValueError("Text payload must contain exactly text and position")
		text = payload["text"]
		position = payload["position"]
		if not isinstance(text, str) or not text or text != text.strip():
			raise ValueError("Text must be a nonblank stripped string")
		if not isinstance(position, tuple) or len(position) != 2:
			raise ValueError("Text position must be a two-coordinate immutable tuple")
		if any(
				isinstance(value, bool)
				or not isinstance(value, numbers.Real)
				or not math.isfinite(value)
				for value in position
				):
			raise ValueError("Text position coordinates must be finite real numbers")
		candidate = ferrum_qt.io.cdml_candidate.append_text_candidate(
			snapshot.cdml, self._next_text_provisional_id(snapshot.revision),
			position, text,
		)
		return self._prepare_complete_candidate(snapshot.revision, candidate)

	#============================================
	def _build_plus_candidate(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
		) -> _PreparedPersistentOperation:
		"""Build the complete CDML candidate owned by the Plus dispatcher key."""
		payload = dict(request.payload)
		if set(payload) != {"position"}:
			raise ValueError("Plus payload must contain exactly position")
		position = payload["position"]
		if not isinstance(position, tuple) or len(position) != 2:
			raise ValueError("Plus position must be a two-coordinate immutable tuple")
		if any(
				isinstance(value, bool)
				or not isinstance(value, numbers.Real)
				or not math.isfinite(value)
				for value in position
				):
			raise ValueError("Plus position coordinates must be finite real numbers")
		candidate = ferrum_qt.io.cdml_candidate.append_plus_candidate(
			snapshot.cdml, self._next_plus_provisional_id(snapshot.revision), position,
		)
		return self._prepare_complete_candidate(snapshot.revision, candidate)

	#============================================
	def _build_vector_candidate(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
		) -> _PreparedPersistentOperation:
		"""Build one validated complete-CDML candidate for a Vector gesture."""
		if request.target_keys:
			raise ValueError("Vector creation does not accept persistent targets")
		payload = dict(request.payload)
		if set(payload) != {"shape", "start", "end"}:
			raise ValueError("Vector payload must contain exactly shape, start, and end")
		shape = payload["shape"]
		start = payload["start"]
		end = payload["end"]
		if shape not in {"rect", "oval", "polyline"}:
			raise ValueError("Vector shape is unsupported")
		for name, point in (("start", start), ("end", end)):
			if type(point) is not tuple or len(point) != 2:
				raise ValueError("Vector %s must be a two-coordinate immutable tuple" % name)
			if any(
					isinstance(value, bool)
					or not isinstance(value, numbers.Real)
					or not math.isfinite(value)
					for value in point
				):
				raise ValueError("Vector %s coordinates must be finite real numbers" % name)
		provisional_id = self._next_vector_provisional_id(snapshot.revision)
		candidate = ferrum_qt.io.cdml_candidate.append_vector_candidate(
			snapshot.cdml, provisional_id, shape, start, end,
		)
		return self._prepare_complete_candidate(snapshot.revision, candidate)

	#============================================
	def _build_bracket_candidate(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
		) -> _PreparedPersistentOperation:
		"""Build one atomic complete-CDML rectangular bracket candidate."""
		if request.target_keys:
			raise ValueError("Bracket creation does not accept persistent targets")
		payload = dict(request.payload)
		if set(payload) != {"bounds"}:
			raise ValueError("Bracket payload must contain exactly bounds")
		bounds = payload["bounds"]
		if type(bounds) is not tuple or len(bounds) != 4:
			raise ValueError("Bracket bounds must be an immutable four-coordinate tuple")
		if any(
				isinstance(value, bool)
				or not isinstance(value, numbers.Real)
				or not math.isfinite(value)
				for value in bounds
			):
			raise ValueError("Bracket bounds must contain finite real numbers")
		left, top, right, bottom = bounds
		if not left < right or not top < bottom:
			raise ValueError("Bracket bounds must have strict left-right and top-bottom order")
		candidate = ferrum_qt.io.cdml_candidate.append_rectangular_bracket_candidate(
			snapshot.cdml, self._next_bracket_provisional_ids(snapshot.revision), bounds,
		)
		prepared = self._prepare_complete_candidate(snapshot.revision, candidate)
		return dataclasses.replace(prepared, preserve_existing_selection=True)

	#============================================
	def _build_wavy_candidate(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
		) -> _PreparedPersistentOperation:
		"""Build one validated complete-CDML candidate for a Wavy gesture."""
		if request.target_keys:
			raise ValueError("Wavy creation does not accept persistent targets")
		payload = dict(request.payload)
		if set(payload) != {"start", "end"}:
			raise ValueError("Wavy payload must contain exactly start and end")
		start = payload["start"]
		end = payload["end"]
		points = ferrum_qt.wavy_geometry.wavy_points(start, end)
		if len(points) < 2:
			raise ValueError("Wavy gesture must have nonzero length")
		candidate = ferrum_qt.io.cdml_candidate.append_wavy_candidate(
			snapshot.cdml, self._next_wavy_provisional_id(snapshot.revision), points,
		)
		return self._prepare_complete_candidate(snapshot.revision, candidate)

	#============================================
	def _build_paper_properties_patch(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind explicit dialog intent to OASA's paper-properties patch API."""
		if request.target_keys:
			raise ValueError("Paper properties does not accept persistent targets")
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "changes"}:
			raise ValueError("Paper properties payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Paper properties expected_revision must be an integer")
		if expected_revision != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Paper properties expected revision does not match the current snapshot",
			)
		changes = payload["changes"]
		if type(changes) is not tuple:
			raise ValueError("Paper properties changes must be an immutable tuple")
		paper_patch = oasa.cdml_document.CDMLPaperPropertiesPatch(
			expected_revision=expected_revision,
			changes=changes,
		)
		return _PreparedPersistentOperation(
			"paper-properties-patch", expected_revision, paper_patch,
		)

	#============================================
	def _prepare_complete_candidate(
			self, expected_revision: int, candidate: str,
			) -> _PreparedPersistentOperation:
		"""Bind a complete candidate to the shared complete-CDML executor."""
		prepared = _PreparedPersistentOperation(
			"complete-candidate", expected_revision, candidate,
		)
		return prepared

	#============================================
	def _build_presentation_stack_reorder(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Validate a revision-bound presentation-only root reorder candidate."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "mode", "root_ids"}:
			raise ValueError("Presentation stack payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		mode = payload["mode"]
		root_ids = payload["root_ids"]
		if type(expected_revision) is not int:
			raise ValueError("Presentation stack expected_revision must be an integer")
		if expected_revision != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Presentation stack expected revision does not match the current snapshot",
			)
		if mode not in {"bring-to-front", "send-back", "swap-at-slots"}:
			raise ValueError("Presentation stack mode is unsupported")
		if not isinstance(root_ids, tuple) or not root_ids:
			raise ValueError("Presentation stack root_ids must be a nonempty immutable tuple")
		if any(
				not isinstance(identifier, str) or not identifier.strip()
				for identifier in root_ids
			):
			raise ValueError("Presentation stack root IDs must be nonblank strings")
		if len(set(root_ids)) != len(root_ids):
			raise ValueError("Presentation stack root IDs must be unique")
		if mode == "swap-at-slots" and len(root_ids) < 2:
			raise ValueError("Presentation stack swap requires at least two roots")
		expected_targets = frozenset(
			("presentation", identifier) for identifier in root_ids
		)
		if request.target_keys != expected_targets:
			raise ValueError("Presentation stack target keys must match root IDs")
		candidate = ferrum_qt.io.cdml_candidate.reorder_presentation_roots_candidate(
			snapshot.cdml, root_ids, mode,
		)
		return self._prepare_complete_candidate(expected_revision, candidate)

	#============================================
	def _build_molecule_insertion(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Validate an immutable molecule-only proposal without revising it."""
		if request.target_keys:
			raise ValueError("Molecule insertion does not accept persistent targets")
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "proposal_cdml"}:
			raise ValueError(
				"Molecule insertion payload must contain expected_revision and proposal_cdml",
			)
		expected_revision = payload["expected_revision"]
		proposal_cdml = payload["proposal_cdml"]
		if isinstance(expected_revision, bool) or not isinstance(expected_revision, int):
			raise ValueError("Molecule insertion revision must be an integer")
		if not isinstance(proposal_cdml, str) or not proposal_cdml:
			raise ValueError("Molecule insertion proposal must be a nonempty string")
		insertion_request = oasa.cdml_document.CDMLMoleculeInsertionRequest(
			expected_revision=expected_revision,
			proposal_cdml=proposal_cdml,
			label=request.label,
		)
		prepared = _PreparedPersistentOperation(
			"molecule-insertion", expected_revision, insertion_request,
		)
		return prepared

	#============================================
	def _build_template_insertion(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Prepare one detached template proposal in OASA for normal insertion."""
		if request.target_keys:
			raise ValueError("Template insertion does not accept persistent targets")
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "template_name", "anchor"}:
			raise ValueError(
				"Template insertion payload must contain expected_revision, template_name, and anchor",
			)
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Template insertion expected_revision must be an integer")
		if expected_revision != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Template insertion expected revision does not match the current snapshot",
			)
		template_name = payload["template_name"]
		if not isinstance(template_name, str) or not template_name:
			raise ValueError("Template insertion template_name must be a nonempty string")
		anchor = payload["anchor"]
		if (
				type(anchor) is not tuple
				or len(anchor) != 2
				or any(
					isinstance(value, bool)
					or not isinstance(value, numbers.Real)
					or not math.isfinite(value)
					for value in anchor
				)
			):
			raise ValueError(
				"Template insertion anchor must be a finite two-value immutable tuple",
			)
		prepared_template = oasa.template_placement.prepare_template_molecule_insertion(
			oasa.template_placement.CDMLTemplatePlacementRequest(
				template_name=template_name,
				anchor=anchor,
				token_stem=self._next_template_token_stem(snapshot.revision),
			),
		)
		if not isinstance(
				prepared_template,
				oasa.template_placement.CDMLPreparedMoleculeInsertion,
			):
			raise ValueError("Template preparation returned an invalid detached proposal")
		insertion_request = oasa.cdml_document.CDMLMoleculeInsertionRequest(
			expected_revision=expected_revision,
			proposal_cdml=prepared_template.proposal_cdml,
			label=request.label,
		)
		return _PreparedPersistentOperation(
			"molecule-insertion", expected_revision, insertion_request,
			frozenset(
				("molecule", identifier)
				for identifier in prepared_template.root_provisional_molecule_ids
			),
		)

	#============================================
	def _build_biomolecule_template_insertion(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Prepare one OASA-owned packaged biomolecule through molecule insertion."""
		if request.target_keys:
			raise ValueError("Biomolecule insertion does not accept persistent targets")
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "catalog_key", "anchor"}:
			raise ValueError("Biomolecule insertion payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Biomolecule insertion expected_revision must be an integer")
		if expected_revision != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Biomolecule insertion expected revision does not match the current snapshot",
			)
		prepared = oasa.biomolecule_template_placement.prepare_biomolecule_template_insertion(
			oasa.biomolecule_template_placement.BiomoleculeTemplatePlacementRequest(
				catalog_key=payload["catalog_key"], anchor=payload["anchor"],
				token_stem=self._next_biomolecule_token_stem(snapshot.revision),
			),
		)
		if type(prepared) is not oasa.template_placement.CDMLPreparedMoleculeInsertion:
			raise ValueError("Biomolecule preparation returned an invalid detached proposal")
		insertion_request = oasa.cdml_document.CDMLMoleculeInsertionRequest(
			expected_revision, prepared.proposal_cdml, request.label,
		)
		return _PreparedPersistentOperation(
			"molecule-insertion", expected_revision, insertion_request,
			frozenset(("molecule", identifier) for identifier in prepared.root_provisional_molecule_ids),
		)

	#============================================
	def _build_user_template_insertion(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one session-frozen saved template to OASA's insertion request."""
		if request.target_keys:
			raise ValueError("User template insertion does not accept persistent targets")
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "catalog_key", "anchor"}:
			raise ValueError("User template insertion payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("User template insertion expected_revision must be an integer")
		if expected_revision != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"User template insertion expected revision does not match the current snapshot",
			)
		catalog_key = payload["catalog_key"]
		if type(catalog_key) is not str or not catalog_key.strip():
			raise ValueError("User template insertion catalog_key must be nonblank")
		anchor = payload["anchor"]
		if (
			type(anchor) is not tuple or len(anchor) != 2
			or any(
				isinstance(value, bool) or not isinstance(value, numbers.Real)
				or not math.isfinite(value)
				for value in anchor
			)
		):
			raise ValueError("User template insertion anchor must be a finite point tuple")
		if catalog_key not in self._user_templates_by_key:
			raise ValueError("User template catalog key is unavailable")
		entry = self._user_templates_by_key[catalog_key]
		insertion_request = oasa.cdml_document.CDMLUserTemplateInsertionRequest(
			expected_revision=expected_revision,
			template_cdml=entry.template_cdml,
			anchor=(float(anchor[0]), float(anchor[1])),
			label=entry.label,
		)
		return _PreparedPersistentOperation(
			"user-template-insertion", expected_revision, insertion_request,
		)

	#============================================
	def _build_geometry_repair(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one plain geometry-repair request to the OASA executor."""
		payload = dict(request.payload)
		if set(payload) != {
			"expected_revision", "molecule_ids", "kind", "target_spacing_pt",
		}:
			raise ValueError("Geometry repair payload has unsupported fields")
		molecule_ids = payload["molecule_ids"]
		if not isinstance(molecule_ids, tuple):
			raise ValueError("Geometry repair molecule_ids must be an immutable tuple")
		if request.target_keys != frozenset(("molecule", identifier) for identifier in molecule_ids):
			raise ValueError("Geometry repair target keys must match molecule IDs")
		repair_request = oasa.cdml_document.CDMLGeometryRepairRequest(
			expected_revision=payload["expected_revision"],
			molecule_ids=molecule_ids,
			kind=payload["kind"],
			target_spacing_pt=payload["target_spacing_pt"],
		)
		return _PreparedPersistentOperation(
			"geometry-repair", repair_request.expected_revision, repair_request,
			preserve_existing_selection=True,
		)

	#============================================
	def _build_atom_align(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one direct atom selection to OASA's narrow alignment operation."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "axis", "targets"}:
			raise ValueError("Atom alignment payload has unsupported fields")
		targets = payload["targets"]
		if not isinstance(targets, tuple):
			raise ValueError("Atom alignment targets must be an immutable tuple")
		if request.target_keys != (
				frozenset(("molecule", molecule_id) for molecule_id, _atom_id in targets)
				| frozenset(("atom", atom_id) for _molecule_id, atom_id in targets)
			):
			raise ValueError("Atom alignment target keys must match atom targets")
		align_request = oasa.cdml_document.CDMLAtomAlignRequest(
			expected_revision=payload["expected_revision"], axis=payload["axis"], targets=targets,
		)
		return _PreparedPersistentOperation(
			"atom-align", align_request.expected_revision, align_request,
			frozenset(("atom", atom_id) for _molecule_id, atom_id in targets),
		)

	#============================================
	def _build_atom_translate(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one direct atom nudge to OASA's atomic translation operation."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "targets", "delta"}:
			raise ValueError("Atom translation payload has unsupported fields")
		targets = payload["targets"]
		delta = payload["delta"]
		if not isinstance(targets, tuple) or not isinstance(delta, tuple):
			raise ValueError("Atom translation targets and delta must be immutable tuples")
		if request.target_keys != (
				frozenset(("molecule", molecule_id) for molecule_id, _atom_id in targets)
				| frozenset(("atom", atom_id) for _molecule_id, atom_id in targets)
			):
			raise ValueError("Atom translation target keys must match atom targets")
		translate_request = oasa.cdml_document.CDMLAtomTranslateRequest(
			expected_revision=payload["expected_revision"], targets=targets, delta=delta,
		)
		return _PreparedPersistentOperation(
			"atom-translate", translate_request.expected_revision, translate_request,
			frozenset(("atom", atom_id) for _molecule_id, atom_id in targets),
		)

	#============================================
	def _build_selection_translate(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one mixed durable selection to OASA's atomic translation operation."""
		payload = dict(request.payload)
		if set(payload) != {
				"expected_revision", "atom_targets", "presentation_root_ids", "delta",
			}:
			raise ValueError("Selection translation payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		atom_targets = payload["atom_targets"]
		presentation_root_ids = payload["presentation_root_ids"]
		delta = payload["delta"]
		if type(expected_revision) is not int:
			raise ValueError("Selection translation expected_revision must be an integer")
		if expected_revision != snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Selection translation expected revision does not match the current snapshot",
			)
		if type(atom_targets) is not tuple or type(presentation_root_ids) is not tuple:
			raise ValueError("Selection translation targets must be immutable tuples")
		if type(delta) is not tuple:
			raise ValueError("Selection translation delta must be an immutable tuple")
		if not atom_targets:
			raise ValueError("Selection translation requires durable atom targets")
		if not presentation_root_ids:
			raise ValueError("Selection translation requires durable presentation roots")
		if any(
				type(target) is not tuple or len(target) != 2
				or type(target[0]) is not str or not target[0].strip()
				or type(target[1]) is not str or not target[1].strip()
				for target in atom_targets
			):
			raise ValueError("Selection translation atom targets must be durable ID pairs")
		if any(type(identifier) is not str or not identifier.strip() for identifier in presentation_root_ids):
			raise ValueError("Selection translation presentation IDs must be nonblank strings")
		expected_target_keys = (
			frozenset(("molecule", molecule_id) for molecule_id, _atom_id in atom_targets)
			| frozenset(("atom", atom_id) for _molecule_id, atom_id in atom_targets)
			| frozenset(("presentation", identifier) for identifier in presentation_root_ids)
		)
		if request.target_keys != expected_target_keys:
			raise ValueError("Selection translation target keys must match durable targets")
		translate_request = oasa.cdml_document.CDMLSelectionTranslateRequest(
			expected_revision=expected_revision,
			atom_targets=atom_targets,
			presentation_root_ids=presentation_root_ids,
			delta=delta,
		)
		selection_keys = (
			frozenset(("atom", atom_id) for _molecule_id, atom_id in atom_targets)
			| frozenset(("presentation", identifier) for identifier in presentation_root_ids)
		)
		return _PreparedPersistentOperation(
			"selection-translate", expected_revision, translate_request, selection_keys,
		)

	#============================================
	def _build_atom_rotate(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one direct atom rotation to OASA's atomic operation."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "targets", "center", "angle_radians"}:
			raise ValueError("Atom rotation payload has unsupported fields")
		targets = payload["targets"]
		center = payload["center"]
		if not isinstance(targets, tuple) or not isinstance(center, tuple):
			raise ValueError("Atom rotation targets and center must be immutable tuples")
		if request.target_keys != (
				frozenset(("molecule", molecule_id) for molecule_id, _atom_id in targets)
				| frozenset(("atom", atom_id) for _molecule_id, atom_id in targets)
			):
			raise ValueError("Atom rotation target keys must match atom targets")
		rotate_request = oasa.cdml_document.CDMLAtomRotateRequest(
			expected_revision=payload["expected_revision"], targets=targets,
			center=center, angle_radians=payload["angle_radians"],
		)
		return _PreparedPersistentOperation(
			"atom-rotate", rotate_request.expected_revision, rotate_request,
			frozenset(("atom", atom_id) for _molecule_id, atom_id in targets),
		)

	#============================================
	def _build_bond_order_edit(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact context-menu bond order request to OASA."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "molecule_id", "bond_id", "order"}:
			raise ValueError("Bond order payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Bond order expected_revision must be an integer")
		for field_name in ("molecule_id", "bond_id"):
			value = payload[field_name]
			if not isinstance(value, str) or not value:
				raise ValueError("Bond order %s must be a nonempty string" % field_name)
		if type(payload["order"]) is not int or payload["order"] not in (1, 2, 3):
			raise ValueError("Bond order must be 1, 2, or 3")
		molecule_id = payload["molecule_id"]
		bond_id = payload["bond_id"]
		if request.target_keys != frozenset({("molecule", molecule_id), ("bond", bond_id)}):
			raise ValueError("Bond order target keys must match durable edit targets")
		bond_order_request = oasa.cdml_document.CDMLBondOrderEditRequest(**payload)
		return _PreparedPersistentOperation(
			"bond-order-edit", bond_order_request.expected_revision, bond_order_request,
			frozenset({("bond", bond_id)}),
		)

	#============================================
	def _build_bond_type_edit(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact context-menu bond type request to OASA."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "molecule_id", "bond_id", "bond_type"}:
			raise ValueError("Bond type payload has unsupported fields")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Bond type expected_revision must be an integer")
		for field_name in ("molecule_id", "bond_id"):
			value = payload[field_name]
			if not isinstance(value, str) or not value:
				raise ValueError("Bond type %s must be a nonempty string" % field_name)
		if payload["bond_type"] not in ("n", "w", "h", "a", "b", "d", "o", "s"):
			raise ValueError("Bond type must be an ordinary type character")
		molecule_id = payload["molecule_id"]
		bond_id = payload["bond_id"]
		if request.target_keys != frozenset({("molecule", molecule_id), ("bond", bond_id)}):
			raise ValueError("Bond type target keys must match durable edit targets")
		bond_type_request = oasa.cdml_document.CDMLBondTypeEditRequest(**payload)
		return _PreparedPersistentOperation(
			"bond-type-edit", bond_type_request.expected_revision, bond_type_request,
			frozenset({("bond", bond_id)}),
		)

	#============================================
	def _build_bond_properties_patch(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one immutable direct-core bond-properties patch to OASA."""
		payload = dict(request.payload)
		if set(payload) != {"expected_revision", "molecule_id", "bond_id", "changes"}:
			raise ValueError("Bond properties payload has unsupported fields")
		if type(payload["expected_revision"]) is not int:
			raise ValueError("Bond properties expected_revision must be an integer")
		if payload["expected_revision"] != _snapshot.revision:
			raise oasa.cdml_document.CDMLRevisionConflictError(
				"Bond properties expected revision does not match the current snapshot",
			)
		for field_name in ("molecule_id", "bond_id"):
			value = payload[field_name]
			if not isinstance(value, str) or not value:
				raise ValueError("Bond properties %s must be a nonempty string" % field_name)
		if type(payload["changes"]) is not tuple:
			raise ValueError("Bond properties changes must be an immutable tuple")
		molecule_id = payload["molecule_id"]
		bond_id = payload["bond_id"]
		if request.target_keys != frozenset({("molecule", molecule_id), ("bond", bond_id)}):
			raise ValueError("Bond properties target keys must match durable edit targets")
		patch = oasa.cdml_document.CDMLBondPropertiesPatch(**payload)
		return _PreparedPersistentOperation(
			"bond-properties-patch", patch.expected_revision, patch,
			frozenset({("bond", bond_id)}),
		)

	#============================================
	def _build_structural_edit(
			self, _snapshot: oasa.cdml_document.CDMLSnapshot,
			request: PersistentOperationRequest,
			) -> _PreparedPersistentOperation:
		"""Bind one exact plain Draw-mode operation to the OASA structural grammar."""
		payload = dict(request.payload)
		if "kind" not in payload:
			raise ValueError("Draw structure kind must be a string")
		kind = payload["kind"]
		if not isinstance(kind, str):
			raise ValueError("Draw structure kind must be a string")
		fields_by_kind = {
			"create-bonded-pair": {
				"expected_revision", "kind", "source_position", "target_position",
				"element", "bond_type", "bond_order", "simple_double",
			},
			"extend-atom": {
				"expected_revision", "kind", "molecule_id", "source_atom_id",
				"target_position", "element", "bond_type", "bond_order", "simple_double",
			},
			"join-atoms": {
				"expected_revision", "kind", "molecule_id", "source_atom_id",
				"target_atom_id", "bond_type", "bond_order", "simple_double",
			},
			"apply-bond-tool": {
				"expected_revision", "kind", "molecule_id", "bond_id", "bond_type",
				"bond_order", "simple_double",
			},
		}
		expected_fields = fields_by_kind.get(kind)
		if expected_fields is None or set(payload) != expected_fields:
			raise ValueError("Draw structure payload does not match its operation kind")
		expected_revision = payload["expected_revision"]
		if type(expected_revision) is not int:
			raise ValueError("Draw structure expected_revision must be an integer")
		for position_name in ("source_position", "target_position"):
			if position_name not in payload:
				continue
			position = payload[position_name]
			if (
					type(position) is not tuple
					or len(position) != 2
					or any(
						isinstance(value, bool)
						or not isinstance(value, numbers.Real)
						or not math.isfinite(value)
						for value in position
					)
				):
				raise ValueError(
					"Draw structure positions must be finite two-value immutable tuples",
				)
		for identifier_name in (
				"molecule_id", "source_atom_id", "target_atom_id", "bond_id",
			):
			if identifier_name not in payload:
				continue
			identifier = payload[identifier_name]
			if not isinstance(identifier, str) or not identifier:
				raise ValueError(
					"Draw structure %s must be a nonempty durable ID" % identifier_name,
				)
		if "element" in payload and not isinstance(payload["element"], str):
			raise ValueError("Draw structure element must be a string")
		if "bond_type" not in payload or not isinstance(payload["bond_type"], str):
			raise ValueError("Draw structure bond_type must be a string")
		if type(payload["bond_order"]) is not int:
			raise ValueError("Draw structure bond_order must be an integer")
		if type(payload["simple_double"]) is not bool:
			raise ValueError("Draw structure simple_double must be a bool")
		expected_target_keys = self._structural_target_keys(kind, payload)
		if request.target_keys != expected_target_keys:
			raise ValueError("Draw structure target keys must match durable edit targets")
		structural_request = oasa.cdml_document.CDMLStructuralEditRequest(**payload)
		return _PreparedPersistentOperation(
			"structural-edit", structural_request.expected_revision, structural_request,
		)

	#============================================
