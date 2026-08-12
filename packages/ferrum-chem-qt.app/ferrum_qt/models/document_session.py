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
_BLANK_CDML = ferrum_qt.models.document_session_requests._BLANK_CDML

import ferrum_qt.models.document_session_candidate_edits
import ferrum_qt.models.document_session_candidates
import ferrum_qt.models.document_session_commits
import ferrum_qt.models.document_session_lifecycle
import ferrum_qt.models.document_session_persistence
import ferrum_qt.models.document_session_projection
import ferrum_qt.models.document_session_submission

class DocumentSession(
	ferrum_qt.models.document_session_submission.DocumentSessionSubmissionMixin,
	ferrum_qt.models.document_session_candidates.DocumentSessionCandidatesMixin,
	ferrum_qt.models.document_session_candidate_edits.DocumentSessionCandidateEditsMixin,
	ferrum_qt.models.document_session_commits.DocumentSessionCommitsMixin,
	ferrum_qt.models.document_session_projection.DocumentSessionProjectionMixin,
	ferrum_qt.models.document_session_persistence.DocumentSessionPersistenceMixin,
	ferrum_qt.models.document_session_lifecycle.DocumentSessionLifecycleMixin,
	PySide6.QtCore.QObject,
):
	"""Own one tab's transient Qt projection and backend CDML staging seam.

	The private OASA session owns the authoritative complete CDML snapshot.  The
	Qt document, scene, view, mode manager, and import state remain its live
	projection and interaction state.  Until all legacy actions migrate, their
	changes only invalidate the synchronization latch; they do not create a
	backend commit.

	Args:
		parent: QObject that owns this session (normally MainWindow).
		theme_manager: ThemeManager for the initial canvas theme.
		prefs: Preferences singleton.
		mode_host: Window-like object used by FileActionsMode.
		view_parent: Optional QWidget initially parenting the ChemView.
		file_path: Optional native document path for the initial title.
		display_name: Optional non-native label for loading/imported content.
		origin_path: Optional source path used for duplicate-open detection.
		prepared_native_cdml: One-use native staging result from
			:meth:`prepare_native_cdml`.  Its canonical CDML is parsed into this
			session's independently owned backend authority.
		prepared_imported_cdml: One-use imported-content staging result whose
			canonical CDML initializes this session's backend authority.
		user_template_catalog: Immutable admitted saved-template records copied
			into this session's frontend-owned delivery mapping.
	"""

	title_changed = PySide6.QtCore.Signal(str)
	disposed = PySide6.QtCore.Signal()

	#============================================
	def __init__(
		self, parent: PySide6.QtCore.QObject, theme_manager: object,
		prefs: object, mode_host: object,
		view_parent: PySide6.QtWidgets.QWidget | None = None,
		file_path: str | None = None, display_name: str | None = None,
		origin_path: str | None = None,
		prepared_native_cdml: PreparedNativeCDML | None = None,
		prepared_imported_cdml: PreparedImportedCDML | None = None,
		user_template_catalog: tuple[
			ferrum_qt.io.user_template_catalog.UserTemplateCatalogEntry, ...
			] = (),
		) -> None:
		"""Create a clean, independently owned document session."""
		super().__init__(parent)
		self._disposed = False
		self._teardown_phase = "live"
		self._teardown_diagnostics: list[BaseException] = []
		self._retained_detached_graphics = None
		from ferrum_qt.canvas.graphics_retirement import DetachedGraphicsRetirementReaper
		self._projection_retirement_reaper = DetachedGraphicsRetirementReaper()
		self._import_generation = 0
		self._import_workers = set()
		self._display_name = display_name
		self._origin_path = origin_path or file_path
		self._backend_session = None
		self._backend_projection_synchronized = False
		self._projected_backend_snapshot = None
		self._projected_persistent_generation = None
		self._projection_replacing = False
		self._projection_error = None
		self._projection_lifecycle_generation = 0
		self._projection_lifecycle_port = None
		self._accepted_projection_selection = None
		self._provisional_action_sequence = 0
		self._backend_history = None
		(
			self._user_template_entries,
			self._user_templates_by_key,
			self._user_template_mode_descriptors,
		) = _freeze_user_template_catalog(user_template_catalog)
		self._operation_dispatcher = {
			"arrow.add": self._build_arrow_candidate,
			"text.add": self._build_text_candidate,
			"plus.add": self._build_plus_candidate,
			"vector.add": self._build_vector_candidate,
			"bracket.add": self._build_bracket_candidate,
			"wavy.add": self._build_wavy_candidate,
			"molecule.insert": self._build_molecule_insertion,
			"template.insert": self._build_template_insertion,
			"biotemplate.insert": self._build_biomolecule_template_insertion,
			"user-template.insert": self._build_user_template_insertion,
			"geometry.repair": self._build_geometry_repair,
			"atom.align": self._build_atom_align,
			"atom.translate": self._build_atom_translate,
			"selection.translate": self._build_selection_translate,
			"atom.rotate": self._build_atom_rotate,
			"bond.order.set": self._build_bond_order_edit,
			"bond.type.set": self._build_bond_type_edit,
			"bond.properties.patch": self._build_bond_properties_patch,
			"atom.properties.patch": self._build_atom_properties_patch,
			"text.properties.patch": self._build_text_properties_patch,
			"text.rich.patch": self._build_rich_text_patch,
			"plus.properties.patch": self._build_plus_properties_patch,
			"wavy.properties.patch": self._build_wavy_properties_patch,
			"fragment.create": self._build_fragment_create,
			"fragment.delete": self._build_fragment_delete,
			"group.expand.implicit": self._build_implicit_group_expand,
			"linear-form.convert": self._build_linear_form_convert,
			"atom.mark.apply": self._build_atom_mark_operation,
			"draw.structure": self._build_structural_edit,
			"atom.element.set": self._build_atom_element_edit,
			"atom.number.set": self._build_atom_number_edit,
			"molecule.name.set": self._build_molecule_name_edit,
			"paper.properties.set": self._build_paper_properties_patch,
			"presentation.stack.reorder": self._build_presentation_stack_reorder,
			"top-level.delete": self._build_top_level_delete,
			"structure.delete": self._build_structure_delete,
			"top-level.transform.apply": self._build_top_level_transform,
		}
		self._operation_commit_executors = {
			"complete-candidate": self._commit_complete_candidate,
			"molecule-insertion": self._commit_molecule_insertion,
			"user-template-insertion": self._commit_user_template_insertion,
			"geometry-repair": self._commit_geometry_repair,
			"atom-align": self._commit_atom_align,
			"atom-translate": self._commit_atom_translate,
			"selection-translate": self._commit_selection_translate,
			"atom-rotate": self._commit_atom_rotate,
			"bond-order-edit": self._commit_bond_order_edit,
			"bond-type-edit": self._commit_bond_type_edit,
			"bond-properties-patch": self._commit_bond_properties_patch,
			"atom-properties-patch": self._commit_atom_properties_patch,
			"text-properties-patch": self._commit_text_properties_patch,
			"rich-text-patch": self._commit_rich_text_patch,
			"plus-properties-patch": self._commit_plus_properties_patch,
			"wavy-properties-patch": self._commit_wavy_properties_patch,
			"fragment-create": self._commit_fragment_create,
			"fragment-delete": self._commit_fragment_delete,
			"implicit-group-expand": self._commit_implicit_group_expand,
			"linear-form-convert": self._commit_linear_form_convert,
			"atom-mark-operation": self._commit_atom_mark_operation,
			"structural-edit": self._commit_structural_edit,
			"atom-element-edit": self._commit_atom_element_edit,
			"atom-number-edit": self._commit_atom_number_edit,
			"molecule-name-edit": self._commit_molecule_name_edit,
			"paper-properties-patch": self._commit_paper_properties_patch,
			"top-level-delete": self._commit_top_level_delete,
			"structure-delete": self._commit_structure_delete,
			"top-level-transform": self._commit_top_level_transform,
		}
		self._legacy_isolated = False
		self._document = None
		self._document_modified_connected = False
		self._document_persistent_mutation_connected = False
		self._scene = None
		self._view = None
		self._mode_manager = None
		staged_document = None
		try:
			bootstrap_backend_projection = True
			if prepared_native_cdml is None and prepared_imported_cdml is None:
				self._backend_session = oasa.cdml_document.CDMLDocumentSession.load(
					_BLANK_CDML,
				)
			elif prepared_native_cdml is not None:
				canonical_cdml, staged_document = prepared_native_cdml._peek()
				self._backend_session = oasa.cdml_document.CDMLDocumentSession.load(
					canonical_cdml,
				)
				bootstrap_backend_projection = True
				# Keep this document detached until every new session root is viable.
			else:
				canonical_cdml, staged_document = prepared_imported_cdml._peek()
				self._backend_session = oasa.cdml_document.CDMLDocumentSession.load_imported(
					canonical_cdml,
				)
				bootstrap_backend_projection = True
			self._document = (
				staged_document
				if staged_document is not None
				else ferrum_qt.models.document.Document()
			)
			self._document.set_graphics_retirement_reaper(
				self._projection_retirement_reaper,
			)
			if file_path is not None:
				self._document.file_path = file_path
			self._scene, self._view = ferrum_qt.setup.canvas_setup.create_canvas(
				view_parent, theme_manager, prefs, self._document, owner=self,
			)
			self._backend_history = (
				ferrum_qt.models.backend_revision_history.BackendRevisionHistory.baseline(
					"Document", self._backend_session.revision,
				)
			)
			self._mode_manager = ferrum_qt.setup.mode_setup.setup_modes(
				self._view, mode_host, parent=self,
				persistent_action=self.submit_persistent_operation,
				atom_align_action=self.submit_atom_align,
				atom_translate_action=self.submit_atom_translate,
				atom_rotate_action=self.submit_atom_rotate,
				atom_translate_authority=self.atom_translate_drag_authority,
				presentation_translate_action=self.submit_top_level_transform,
				presentation_translate_context=self.presentation_translate_drag_context,
				selection_translate_action=self.submit_selection_translate,
				selection_translate_context=self.selection_translate_drag_context,
				top_level_delete_context=self.top_level_delete_context,
				structure_delete_context=self.structure_delete_context,
				atom_mark_delete_context=self.atom_mark_delete_context,
				atom_number_context=self.atom_number_context,
				atom_mark_revision=self.atom_mark_revision,
				template_names=oasa.template_placement.system_template_names(),
				template_action=self.submit_system_template,
				biomolecule_catalog=(
					oasa.biomolecule_template_placement.biomolecule_template_catalog()
				),
				biotemplate_action=self.submit_biomolecule_template,
				user_template_catalog=self._user_template_mode_descriptors,
				user_template_action=self.submit_user_template,
				graphics_retirement_reaper=self._projection_retirement_reaper,
			)
			# The backend imported-load baseline is empty, so this projection starts
			# visibly dirty before it becomes a live session.  Qt reflects that
			# backend fact; it does not create an independent local mutation.
			if prepared_imported_cdml is not None:
				self._document.mark_dirty()
			self._document.setParent(self)
			self._document.modified_changed.connect(self._on_modified_changed)
			self._document_modified_connected = True
			self._document.persistent_mutated.connect(self._on_persistent_mutated)
			self._document_persistent_mutation_connected = True
			if bootstrap_backend_projection:
				self._projected_backend_snapshot = self._backend_session.snapshot()
				self._projected_persistent_generation = self._document.persistent_generation
				self._backend_projection_synchronized = True
			if prepared_native_cdml is not None:
				prepared_native_cdml._finalize()
			if prepared_imported_cdml is not None:
				prepared_imported_cdml._finalize()
		except Exception:
			self._dispose_failed_construction(staged_document)
			raise

	# ------------------------------------------------------------------
	# Backend CDML authority staging
	# ------------------------------------------------------------------

	#============================================
	@property
	def backend_snapshot(self) -> oasa.cdml_document.CDMLSnapshot:
		"""Return the current immutable, backend-owned complete CDML snapshot."""
		return self._backend_session.snapshot()

	#============================================
	def paper_catalog(self) -> dict[str, list[float] | None]:
		"""Return the OASA-owned plain paper catalog for this live client session."""
		self._require_live_persistent_operation()
		return self._backend_session.paper_catalog()

	#============================================
	def paper_properties_context(self) -> dict[str, object]:
		"""Return OASA's plain editable-paper observation for this session."""
		return self._backend_session.paper_properties_context()

	#============================================
	def query_molecule_smiles(
			self, expected_revision: int, molecule_id: str,
			) -> oasa.cdml_document.CDMLMoleculeSmilesResult:
		"""Observe one synchronized direct-root molecule through OASA CDML.

		The Qt session supplies only immutable scalar revision and durable-ID
		data.  This query creates no candidate, history entry, dirty transition,
		or projection replacement.
		"""
		self._require_live_persistent_operation()
		if not self.can_write_authoritative_snapshot:
			raise BackendProjectionOutOfSyncError(
				"Cannot query molecule SMILES while the Qt projection is not a "
				"current authoritative projection",
			)
		request = oasa.cdml_document.CDMLMoleculeSmilesQuery(
			expected_revision=expected_revision,
			molecule_id=molecule_id,
		)
		return self._backend_session.query_molecule_smiles(request)

	#============================================
	def observe_atom_chemistry_facts(
			self, expected_revision: int,
			) -> oasa.cdml_document.CDMLAtomChemistryFactsObservation:
		"""Return one read-only OASA chemistry observation for this projection."""
		self._require_live_persistent_operation()
		if not self.can_write_authoritative_snapshot:
			raise BackendProjectionOutOfSyncError(
				"Cannot observe atom chemistry while the Qt projection is not a "
				"current authoritative projection",
			)
		return self._backend_session.atom_chemistry_facts(
			oasa.cdml_document.CDMLAtomChemistryFactsQuery(expected_revision),
		)

	#============================================
	def atom_number_context(self) -> tuple[int, int]:
		"""Return revision and next transient candidate from backend CDML.

		The returned scalar is compatibility presentation state.  The canonical
		snapshot remains the sole persistent source, including hidden numbers.
		"""
		snapshot = self.backend_snapshot
		# Accept the complete document at the CDML boundary before compatibility
		# DOM inspection identifies direct core molecule/atom records.
		oasa.cdml_document.CDMLDocument.parse(snapshot.cdml, validation="compat")
		document = oasa.safe_xml.parse_dom_from_string(snapshot.cdml)
		highest_number = 0
		root = document.documentElement
		for molecule in _direct_core_cdml_children(root, "molecule"):
			for atom in _direct_core_cdml_children(molecule, "atom"):
				number_text = atom.getAttribute("number")
				if not number_text.isdecimal():
					continue
				number = int(number_text)
				if number > highest_number:
					highest_number = number
		next_number = highest_number + 1
		context = (snapshot.revision, next_number)
		return context

	#============================================
	def atom_mark_revision(self) -> int:
		"""Return the exact current backend revision for one MarkMode gesture."""
		self._require_live_persistent_operation()
		return self.backend_snapshot.revision

	#============================================
	def capture_visual_render_request(
			self, format_name: str, scope: str = "page",
			) -> oasa.cdml_render.CDMLRenderRequest | oasa.cdml_render.CDMLRenderFailure:
		"""Capture one exact backend snapshot and durable Qt selection keys.

		The resulting request contains no live Qt object.  Page and content output
		remain available while a projection is stale because the backend snapshot is
		the only persistent render source.  Selection has one additional Qt-only
		capture step and reports a typed outcome when no live projection exists.
		"""
		if self._disposed or self._backend_session is None:
			return oasa.cdml_render.CDMLRenderFailure(
				"session-unavailable", "Visual export requires a live backend session",
			)
		try:
			snapshot = self._backend_session.snapshot()
		except Exception:
			return oasa.cdml_render.CDMLRenderFailure(
				"session-unavailable", "Visual export requires a readable backend snapshot",
			)
		selection_keys = ()
		if scope == "selection":
			if not self._selection_projection_matches_snapshot(snapshot):
				return oasa.cdml_render.CDMLRenderFailure(
					"selection-unavailable",
					"Selection export requires the current Qt projection", snapshot.revision,
				)
			try:
				items = ferrum_qt.canvas.graphics_retirement.selected_items_from_captured_scene(
					self._scene,
				)
				if not items:
					return oasa.cdml_render.CDMLRenderFailure(
						"selection-unavailable", "Selection export requires a durable selection",
						snapshot.revision,
					)
				seen = set()
				captured = []
				for item in items:
					if not self._document.is_current_projection_item(item):
						return oasa.cdml_render.CDMLRenderFailure(
							"selection-unavailable",
							"Selection export requires current projection items",
							snapshot.revision,
						)
					key = ferrum_qt.canvas.document_projection.persistent_selection_key(item)
					if key is None:
						return oasa.cdml_render.CDMLRenderFailure(
							"selection-unavailable",
							"Selection export requires durable selection IDs",
							snapshot.revision,
						)
					if key in seen:
						continue
					seen.add(key)
					captured.append(oasa.cdml_render.CDMLRenderSelectionKey(*key))
				selection_keys = tuple(captured)
			except Exception:
				return oasa.cdml_render.CDMLRenderFailure(
					"selection-unavailable", "Could not capture durable selection IDs",
					snapshot.revision,
				)
		try:
			return oasa.cdml_render.CDMLRenderRequest(
				snapshot=snapshot, format_name=format_name, scope=scope,
				selection_keys=selection_keys,
			)
		except (TypeError, ValueError) as exc:
			return oasa.cdml_render.CDMLRenderFailure(
				"invalid-render-request", str(exc), snapshot.revision,
			)

	#============================================
	def _selection_projection_matches_snapshot(
			self, snapshot: oasa.cdml_document.CDMLSnapshot,
			) -> bool:
		"""Return whether one captured snapshot has its installed Qt projection.

		Selection is frontend interaction state, unlike page/content export.  Its
		durable keys are meaningful only while the registered projection still
		represents this exact immutable backend snapshot.  This check deliberately
		uses the snapshot already captured for the render request rather than
		reading the backend a second time.
		"""
		return (
			not self._disposed
			and not self._projection_replacing
			and self._projection_error is None
			and self._backend_projection_synchronized
			and self._document is not None
			and self._scene is not None
			and self._view is not None
			and self._projected_backend_snapshot == snapshot
			and self._document._scene is self._scene
			and self._view.document is self._document
		)

	#============================================
	@property
	def backend_projection_synchronized(self) -> bool:
		"""Return whether the live Qt document matches the backend snapshot."""
		return self._backend_projection_synchronized

	#============================================
	@property
	def projection_error(self) -> Exception | None:
		"""Return the diagnostic from an unrecoverable projection replacement."""
		return self._projection_error

	#============================================
	def commit_complete_candidate(
			self, complete_cdml: str,
			) -> oasa.cdml_document.CDMLCommit:
		"""Accept a complete CDML candidate without changing the Qt projection."""
		self._require_live_persistent_operation()
		commit = self._backend_session.commit(
			expected_revision=self._backend_session.revision,
			complete_cdml=complete_cdml,
		)
		self._backend_projection_synchronized = False
		return commit

	#============================================
	@property
	def projection_lifecycle_generation(self) -> int:
		"""Return the generation that invalidates stale lifecycle delivery."""
		return self._projection_lifecycle_generation

	#============================================
	def install_projection_lifecycle_port(
			self, port: ferrum_qt.models.projection_lifecycle.SessionProjectionLifecyclePort,
			) -> None:
		"""Install one explicitly session-bound projection delivery port."""
		if self._disposed or not port.is_bound_to(self):
			raise ValueError("A live session requires its own projection lifecycle port")
		self._projection_lifecycle_port = port

	#============================================
	def owns_projection_lifecycle_port(self, port: object) -> bool:
		"""Return whether a delivery port is this live session's current owner."""
		return not self._disposed and self._projection_lifecycle_port is port

	#============================================
	def clear_projection_lifecycle_port(self) -> None:
		"""Invalidate and remove this session's projection delivery port."""
		self._projection_lifecycle_generation += 1
		self._projection_lifecycle_port = None

	#============================================
	@property
	def legacy_isolated(self) -> bool:
		"""Return whether Qt-local persistent edits block backend actions."""
		return self._legacy_isolated

	#============================================
	@property
	def can_commit_persistent_action(self) -> bool:
		"""Return whether a persistent backend action can start safely now."""
		available = (
			self._projection_lifecycle_port is not None
			and not self._legacy_isolated
			and self.can_write_authoritative_snapshot
		)
		return available

	#============================================
	def replace_user_template_catalog(
			self,
			entries: tuple[ferrum_qt.io.user_template_catalog.UserTemplateCatalogEntry, ...],
			) -> None:
		"""Atomically replace this session's frozen saved-template delivery data."""
		if self._disposed:
			raise RuntimeError("Cannot replace a disposed session's user template catalog")
		frozen_entries, by_key, descriptors = _freeze_user_template_catalog(entries)
		mode = self._mode_manager.mode("usertemplate")
		set_catalog = getattr(mode, "set_catalog", None)
		if not callable(set_catalog):
			raise RuntimeError("User template mode is unavailable")
		previous_entries = self._user_template_entries
		previous_by_key = self._user_templates_by_key
		previous_descriptors = self._user_template_mode_descriptors
		try:
			set_catalog(descriptors)
			self._user_template_entries = frozen_entries
			self._user_templates_by_key = by_key
			self._user_template_mode_descriptors = descriptors
		except Exception:
			set_catalog(previous_descriptors)
			self._user_template_entries = previous_entries
			self._user_templates_by_key = previous_by_key
			self._user_template_mode_descriptors = previous_descriptors
			raise

	#============================================
	def atom_translate_drag_authority(self) -> str:
		"""Return the current frontend-only authority for an EditMode atom drag.

		The installed translation callback alone cannot distinguish a normal
		backend session from a legacy-isolated projection: every session installs
		the callback so keyboard nudging has one narrow interface.  This query
		keeps that distinction at the session boundary without carrying Qt
		objects across the backend-facing request boundary.
		"""
		return self._edit_drag_authority()

	#============================================
	def presentation_translate_drag_authority(self) -> str:
		"""Return the current frontend-only authority for a presentation drag.

		Presentation-only EditMode drags use the same session/projection provenance
		gate as atom drags.  The separate public name keeps the mode's two durable
		request grammars explicit while this session owns their common lifecycle
		state.
		"""
		return self._edit_drag_authority()

	#============================================
	def presentation_translate_drag_context(self) -> tuple[str, int | None]:
		"""Return one immutable authority/revision pair for an EditMode drag."""
		authority = self.presentation_translate_drag_authority()
		if authority == "backend":
			return authority, self.backend_snapshot.revision
		return authority, None


	#============================================
	def selection_translate_drag_context(self) -> tuple[str, int | None]:
		"""Return one immutable authority/revision pair for a mixed EditMode drag."""
		authority = self._edit_drag_authority()
		if authority == "backend":
			return authority, self.backend_snapshot.revision
		return authority, None

	#============================================
	def top_level_delete_authority(self) -> str:
		"""Return the current frontend-only authority for complete-root Delete.

		Complete-root Delete shares the session/projection provenance gate used by
		EditMode drags.  The public name makes its local transitional route and
		unavailable synchronized outcome explicit at the interaction boundary.
		"""
		return self._edit_drag_authority()

	#============================================
	def top_level_delete_context(self) -> tuple[str, int | None]:
		"""Return one immutable authority/revision pair for complete-root Delete."""
		authority = self.top_level_delete_authority()
		if authority == "backend":
			return authority, self.backend_snapshot.revision
		return authority, None

	#============================================
	def structure_delete_authority(self) -> str:
		"""Return the current frontend-only authority for partial structure Delete."""
		return self._edit_drag_authority()

	#============================================
	def structure_delete_context(self) -> tuple[str, int | None]:
		"""Return one immutable authority/revision pair for partial structure Delete."""
		authority = self.structure_delete_authority()
		if authority == "backend":
			return authority, self.backend_snapshot.revision
		return authority, None

	#============================================
	def atom_mark_delete_context(self) -> tuple[str, int | None]:
		"""Return one immutable authority/revision pair for selected-mark Delete."""
		authority = self._edit_drag_authority()
		if authority == "backend":
			return authority, self.backend_snapshot.revision
		return authority, None

	#============================================
	def _edit_drag_authority(self) -> str:
		"""Classify one in-flight EditMode gesture without exposing Qt state."""
		if (
				self._disposed
				or self._projection_replacing
				or self._projection_error is not None
				or self._backend_session is None
				or self._document is None
				or self._scene is None
				or self._view is None
				or self._projection_lifecycle_port is None
			):
			return "unavailable"
		if self._legacy_isolated:
			return "local"
		if self.can_commit_persistent_action:
			return "backend"
		return "unavailable"

	#============================================
	@property
	def can_undo_backend(self) -> bool:
		"""Return whether the preceding logical backend entry is available."""
		available = self.can_commit_persistent_action and self._backend_history.can_undo
		return available

	#============================================
	@property
	def has_backend_navigation(self) -> bool:
		"""Return whether this session owns generic backend history entries."""
		return self._backend_history is not None

	#============================================
	@property
	def can_redo_backend(self) -> bool:
		"""Return whether the succeeding logical backend entry is available."""
		available = (
			self.can_commit_persistent_action
			and self._backend_history.can_redo
		)
		return available

	#============================================
	def _next_arrow_provisional_id(self, revision: int) -> str:
		"""Allocate a frontend-only correlation token for one candidate arrow."""
		self._provisional_action_sequence += 1
		token = "__bkchem_new__arrow-r%s-%s" % (
			revision, self._provisional_action_sequence,
		)
		return token

	#============================================
	def _next_text_provisional_id(self, revision: int) -> str:
		"""Allocate a frontend-only correlation token for one candidate text."""
		self._provisional_action_sequence += 1
		token = "__bkchem_new__text-r%s-%s" % (
			revision, self._provisional_action_sequence,
		)
		return token

	#============================================
	def _next_plus_provisional_id(self, revision: int) -> str:
		"""Allocate a frontend-only correlation token for one candidate Plus."""
		self._provisional_action_sequence += 1
		token = "__bkchem_new__plus-r%s-%s" % (
			revision, self._provisional_action_sequence,
		)
		return token

	#============================================
	def _next_vector_provisional_id(self, revision: int) -> str:
		"""Allocate a frontend-only correlation token for one candidate Vector."""
		self._provisional_action_sequence += 1
		token = "__bkchem_new__vector-r%s-%s" % (
			revision, self._provisional_action_sequence,
		)
		return token

	#============================================
	def _next_bracket_provisional_ids(self, revision: int) -> tuple[str, str]:
		"""Allocate two distinct frontend-only tokens for one bracket pair."""
		self._provisional_action_sequence += 1
		stem = "__bkchem_new__bracket-r%s-%s" % (
			revision, self._provisional_action_sequence,
		)
		return stem + "-left", stem + "-right"

	#============================================
	def _next_wavy_provisional_id(self, revision: int) -> str:
		"""Allocate a frontend-only correlation token for one candidate Wavy."""
		self._provisional_action_sequence += 1
		token = "__bkchem_new__wavy-r%s-%s" % (
			revision, self._provisional_action_sequence,
		)
		return token

	#============================================
	def _next_template_token_stem(self, revision: int) -> str:
		"""Allocate one session-local provisional stem for OASA template preparation."""
		self._provisional_action_sequence += 1
		token_stem = "template-r%s-%s" % (
			revision, self._provisional_action_sequence,
		)
		return token_stem

	#============================================
	def _next_biomolecule_token_stem(self, revision: int) -> str:
		"""Allocate one session-local provisional stem for biomolecule placement."""
		self._provisional_action_sequence += 1
		return "biomolecule-r%s-%s" % (revision, self._provisional_action_sequence)

	#============================================
