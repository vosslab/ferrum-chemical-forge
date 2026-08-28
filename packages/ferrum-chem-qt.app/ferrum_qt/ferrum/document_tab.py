"""Rust-owned document tab with a disposable Ferrum Qt projection."""

# Standard Library
import dataclasses
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.ferrum.bracket_creation as native_bracket_creation
import ferrum_qt.ferrum.bond_creation as native_bond_creation
import ferrum_qt.ferrum.clipboard_paste_tab as native_clipboard_paste_tab
import ferrum_qt.ferrum.clipboard_cut_tab as native_clipboard_cut_tab
import ferrum_qt.ferrum.document_tab_construction as native_tab_construction
import ferrum_qt.ferrum.document_display_tab as native_document_display_tab
import ferrum_qt.ferrum.geometric_properties as native_geometric_properties
import ferrum_qt.ferrum.geometry_repair as native_geometry_repair
import ferrum_qt.ferrum.graphics_view
import ferrum_qt.ferrum.linear_form as native_linear_form
import ferrum_qt.ferrum.molecule_name as native_molecule_name
import ferrum_qt.ferrum.paper_properties as native_paper_properties
import ferrum_qt.ferrum.presentation_deletion as native_presentation_deletion
import ferrum_qt.ferrum.presentation_stack as native_presentation_stack
import ferrum_qt.ferrum.property_observation as native_property_observation
import ferrum_qt.ferrum.rotation as native_rotation
import ferrum_qt.ferrum.regular_ring_tab as native_regular_ring_tab
import ferrum_qt.ferrum.attached_cyclohexane_tab as native_attached_cyclohexane_tab
import ferrum_qt.ferrum.compact_group_authoring as native_compact_group_authoring
import ferrum_qt.ferrum.free_compact_group_placement as native_free_compact_group_placement
import ferrum_qt.ferrum.haworth_tab as native_haworth_tab
import ferrum_qt.ferrum.direct_glycosidic_haworth_tab as native_direct_haworth_tab
import ferrum_qt.ferrum.direct_bond_gesture_tab as native_direct_bond_gesture_tab
import ferrum_qt.ferrum.presentation_creation_gesture_tab as native_presentation_creation_gesture_tab
import ferrum_qt.ferrum.presentation_vector_gesture_tab as native_presentation_vector_gesture_tab
import ferrum_qt.ferrum.presentation_path_gesture_tab as native_presentation_path_gesture_tab
import ferrum_qt.ferrum.direct_root_interaction_tab as native_direct_root_interaction_tab
import ferrum_qt.ferrum.structure_interaction_tab as native_structure_interaction_tab
import ferrum_qt.ferrum.sdf_insertion as native_sdf_insertion
import ferrum_qt.ferrum.text_properties as native_text_properties
import ferrum_qt.ferrum.text_placement_gesture_tab as native_text_placement_gesture_tab
import ferrum_qt.ferrum.top_level_transform as native_top_level_transform
import ferrum_qt.ferrum.tab_view_state
import ferrum_qt.ferrum.wavy_properties as native_wavy_properties
import ferrum_qt.ferrum.document_tab_publication as native_publication
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors
import ferrum_qt.ferrum.document_tab_molecules as native_document_tab_molecules
import ferrum_qt.ferrum.document_tab_selection as native_document_tab_selection
import ferrum_qt.ferrum.drawing_standard as native_drawing_standard
import ferrum_qt.ferrum.explicit_fragment_tab as native_explicit_fragment
import ferrum_qt.ferrum.local_document_origin_tab as native_local_document_origin_tab
import ferrum_qt.ferrum.live_document_transaction as native_live_document_transaction
import ferrum_qt.ferrum.smarts_selected_root_capture_tab as native_smarts_selected_root_capture
import ferrum_qt.themes.document_display_palette


#============================================
FerrumNativeDocumentTabError = native_document_tab_errors.FerrumNativeDocumentTabError


#============================================
FerrumNativeDocumentTabUnrenderableMoleculeError = (
	native_document_tab_errors.FerrumNativeDocumentTabUnrenderableMoleculeError
)


#============================================
FerrumNativeDocumentTabSavePresentationError = (
	native_publication.FerrumNativeDocumentTabSavePresentationError
)


#============================================
FerrumNativeDocumentTabMutationPresentationError = (
	native_document_tab_errors.FerrumNativeDocumentTabMutationPresentationError
)


# A deliberately narrow device-space target for unlabelled atoms at explicit
# gesture starts. It is not a general canvas hit-test tolerance.
_IMPLICIT_ATOM_PICK_RADIUS_PX = 6


_PRESENTATION_ROOT_KIND_NAMES = frozenset({
	"arrow", "plus", "text", "polyline", "rectangle", "square", "oval", "circle",
	"polygon",
})


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _ImplicitAtomPick:
	"""One durable atom selected from the installed projection."""

	object_id: str


#============================================
class FerrumNativeDocumentTab(
		native_document_display_tab.FerrumNativeDocumentDisplayTabMixin,
		native_document_tab_selection.FerrumNativeDocumentSelectionMixin,
		native_smarts_selected_root_capture.FerrumNativeSmartsSelectedRootCaptureTabMixin,
		native_live_document_transaction.FerrumLiveDocumentTransactionMixin,
		native_document_tab_molecules.FerrumNativeDocumentMoleculeChoicesMixin,
		native_local_document_origin_tab.FerrumNativeLocalDocumentOriginTabMixin,
		native_regular_ring_tab.FerrumNativeRegularRingTabMixin,
		native_attached_cyclohexane_tab.FerrumNativeAttachedCyclohexaneTabMixin,
		native_compact_group_authoring.FerrumNativeCompactGroupAuthoringTabMixin,
		native_free_compact_group_placement.FerrumNativeFreeCompactGroupPlacementTabMixin,
		native_haworth_tab.FerrumNativeHaworthTabMixin,
		native_direct_haworth_tab.FerrumNativeDirectGlycosidicHaworthTabMixin,
		native_direct_bond_gesture_tab.FerrumNativeDirectBondGestureTabMixin,
		native_presentation_creation_gesture_tab.FerrumNativePresentationCreationGestureTabMixin,
		native_presentation_vector_gesture_tab.FerrumNativePresentationVectorGestureTabMixin,
		native_presentation_path_gesture_tab.FerrumNativePresentationPathGestureTabMixin,
		native_text_placement_gesture_tab.FerrumNativeTextPlacementGestureTabMixin,
		native_direct_root_interaction_tab.FerrumNativeDirectRootInteractionTabMixin,
		native_structure_interaction_tab.FerrumNativeStructureInteractionTabMixin,
		native_bond_creation.FerrumNativeBondCreationMixin,
		native_publication.FerrumNativeDocumentTabPublicationMixin,
		native_property_observation.FerrumNativePropertyObservationMixin,
		native_clipboard_cut_tab.FerrumNativeClipboardCutTabMixin,
		native_clipboard_paste_tab.FerrumNativeClipboardPasteTabMixin,
		native_drawing_standard.FerrumNativeDrawingStandardTabMixin,
		native_explicit_fragment.FerrumNativeExplicitFragmentTabMixin,
		native_linear_form.FerrumNativeLinearFormTabMixin,
		native_molecule_name.FerrumNativeMoleculeNameTabMixin,
		native_sdf_insertion.FerrumNativeSdfInsertionTabMixin,
		native_bracket_creation.FerrumNativeBracketCreationMixin,
		native_paper_properties.FerrumNativePaperPropertiesMixin,
		native_wavy_properties.FerrumNativeWavyPropertiesMixin,
		native_geometric_properties.FerrumNativeGeometricPropertiesMixin,
		native_presentation_deletion.FerrumNativePresentationDeletionMixin,
		native_presentation_stack.FerrumNativePresentationStackMixin,
		native_text_properties.FerrumNativeTextPropertiesMixin,
		native_rotation.FerrumNativeRotationTabMixin,
		native_geometry_repair.FerrumNativeGeometryRepairTabMixin,
		native_top_level_transform.FerrumNativeTopLevelTransformTabMixin,
		ferrum_qt.ferrum.tab_view_state.FerrumNativeTabViewStateMixin,
		PySide6.QtWidgets.QWidget,
		):
	"""One self-contained Rust document session and its disposable Qt view.

	The Rust session is the sole authority for CDML, snapshots, dirty state, and
	publication.  This widget retains only the latest successfully projected
	immutable observation plus presentation state derived from that observation.
	"""

	selection_changed = PySide6.QtCore.Signal()

	#============================================
	def __init__(
			self,
			cdml: str,
			title: str,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Load complete CDML into one exact Ferrum session and paint it.

		Args:
			cdml: Complete CDML supplied to Rust without a Qt-side parser.
			title: Initial display title owned by this tab presentation.
		"""
		super().__init__()
		try:
			import ferrum_qt.ferrum.engine as engine
			if type(cdml) is not str or type(title) is not str:
				raise TypeError("Ferrum document tab requires CDML and title strings")
			session = engine.DocumentSession.load(cdml)
			resource = engine.verified_telex_regular()
			if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
				raise TypeError("Ferrum document tab requires a document display palette")
			view = ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView(palette, self)
			controller = ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjectionController(
				view, resource, palette,
			)
			self._initialize(title, session, view, controller, palette)
			self._refresh_from_current_revision()
		except Exception:
				self._dispose_partial_resources()
				raise

	@classmethod
	def from_session(
			cls,
			session: object,
			title: str,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> "FerrumNativeDocumentTab":
		"""Project one detached Rust-owned session without loading CDML in Qt."""
		tab = native_tab_construction.create_document_tab_from_session(
			cls, session, title, palette,
		)
		return tab

	#============================================
	@classmethod
	def from_admitted_local_open(
			cls,
			session: object,
			title: str,
			observation: object,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> "FerrumNativeDocumentTab":
		"""Install one worker-prepared session and matching immutable observation."""
		tab = native_tab_construction.create_admitted_local_document_tab(
			cls, session, title, observation, FerrumNativeDocumentTabError, palette,
		)
		return tab

	def place_template_catalog_entry(
			self, catalog_snapshot: object, key: str, document_snapshot: object,
			x: float, y: float,
			) -> object:
		self._require_mutable()
		commit = self._session.place_template_catalog_entry_v1(
			catalog_snapshot, key, document_snapshot, x, y,
		)
		try:
			selection = ()
			if commit.inserted_molecule_object_id is not None:
				selection = (commit.inserted_molecule_object_id,)
			self._install_mutation_result(commit.result, selection)
		except FerrumNativeDocumentTabMutationPresentationError as error:
			error.accepted_receipt = commit
			raise
		return commit
	#============================================
	@classmethod
	def _from_fixture(cls, title: str, session: object,
			controller: object,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> "FerrumNativeDocumentTab":
		"""Construct a tab with test-only owned-value collaborators."""
		if type(title) is not str:
			raise TypeError("Ferrum document tab fixture title must be a string")
		tab = cls.__new__(cls)
		PySide6.QtWidgets.QWidget.__init__(tab)
		try:
			view = ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView(palette, tab)
			tab._initialize(title, session, view, controller, palette)
			tab._refresh_from_current_revision()
		except Exception:
			tab._dispose_partial_resources()
			raise
		return tab
	#============================================
	def _initialize(self, title: str, session: object,
			view: PySide6.QtWidgets.QGraphicsView,
			controller: object,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Install one ownership graph shared by production and fixture construction."""
		self._title = title
		self._view = view
		self._controller = controller
		self._initialize_document_display_palette(palette)
		self._initialize_live_document_transaction_v1(session)
		self._snapshot: object | None = None
		self._document_observation: object | None = None
		self._render_observation: object | None = None
		self._pending_result: object | None = None
		self._pending_snapshot: object | None = None
		self._pending_durable_selection: tuple[str, ...] | None = None
		self._pending_focus_atom_object_id: str | None = None
		self._structure_action_selection_v1: object | None = None
		self._structure_action_targets_v1: tuple[object, ...] = ()
		self._file_path: pathlib.Path | None = None
		self._initialize_local_document_origin()
		self._disposed = False
		self._scene_selection_source: PySide6.QtWidgets.QGraphicsScene | None = None
		self._scene_selection_connection: PySide6.QtCore.QMetaObject.Connection | None = None
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		layout.setContentsMargins(0, 0, 0, 0)
		layout.addWidget(view)

	#============================================
	@property
	def title(self) -> str:
		"""Return the current presentation title after confirmed publication only."""
		return self._title
	#============================================
	@property
	def current_snapshot(self) -> object:
		"""Return the latest successfully installed Rust snapshot."""
		if self._snapshot is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed observation")
		return self._snapshot
	#============================================
	@property
	def is_dirty(self) -> bool:
		"""Return Rust dirty state, including an accepted render-pending mutation."""
		if self._pending_snapshot is not None:
			return self._pending_snapshot.is_dirty
		return self.current_snapshot.is_dirty
	#============================================
	def can_undo(self) -> bool:
		"""Return the Rust-owned availability of an earlier history state."""
		self._require_live()
		return self._session.can_undo
	#============================================
	def can_redo(self) -> bool:
		"""Return the Rust-owned availability of a later history state."""
		self._require_live()
		return self._session.can_redo
	#============================================
	@property
	def requires_refresh(self) -> bool:
		"""Return whether Rust is ahead of the installed disposable Qt projection."""
		return self._pending_result is not None

	#============================================
	@property
	def is_disposed(self) -> bool:
		"""Return whether this tab has terminally disposed its projection boundary."""
		return self._disposed
	#============================================
	@property
	def file_path(self) -> pathlib.Path | None:
		"""Return the loaded origin or confirmed publication destination, if known."""
		return self._file_path


	#============================================
	def _adopt_loaded_origin_path(self, path: str | pathlib.Path) -> None:
		"""Record a successfully loaded CDML origin without changing Rust state.

		Loading already established the Rust snapshot and clean baseline.  This
		method only records the frontend's true source location for duplicate-open
		detection and an ordinary subsequent Save; it never publishes or marks a
		snapshot clean.
		"""
		self._require_live()
		origin = pathlib.Path(path)
		if not origin.is_absolute():
			raise ValueError("Ferrum document origins must be absolute paths")
		if origin.suffix.lower() != ".cdml":
			raise ValueError("Ferrum document origins must use the .cdml extension")
		self._file_path = origin
		self._title = origin.name
	#============================================
	def select_atom(self, atom_id: str) -> None:
		"""Select one current durable atom by Rust identifier for Ferrum actions."""
		self.select_atoms((atom_id,))
	#============================================
	def select_atoms(self, atom_ids: tuple[str, ...]) -> None:
		"""Select exact current durable atoms by Rust identifier."""
		self._require_live()
		if (
			type(atom_ids) is not tuple
			or not atom_ids
			or any(type(atom_id) is not str or not atom_id for atom_id in atom_ids)
			or len(frozenset(atom_ids)) != len(atom_ids)
		):
			raise ValueError("Ferrum atom selection requires distinct non-empty identifiers")
		import ferrum_qt.ferrum.engine as engine
		targets = self.structure_targets_for_ids(atom_ids)
		if any(target.kind is not engine.StructureTargetKindV1.atom for target in targets):
			raise FerrumNativeDocumentTabError("selected atom is not in the current projection")
		self._require_projection().select_durable(
			tuple(("document_object", atom_id) for atom_id in atom_ids),
		)
	#============================================
	def select_bond(self, bond_id: str) -> None:
		"""Select one current durable bond by Rust identifier for Ferrum actions."""
		self._require_live()
		if type(bond_id) is not str or not bond_id:
			raise ValueError("Ferrum bond selection requires a non-empty identifier")
		import ferrum_qt.ferrum.engine as engine
		targets = self.structure_targets_for_ids((bond_id,))
		if targets[0].kind is not engine.StructureTargetKindV1.bond:
			raise FerrumNativeDocumentTabError(
				"selected bond is not in the current projection",
			)
		self._require_projection().select_durable((("document_object", bond_id),))
	#============================================
	def durable_structure_at_viewport_point(
			self, point: PySide6.QtCore.QPoint,
			) -> tuple[str, str] | None:
		"""Return the topmost installed opaque document target at one viewport point."""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPoint):
			raise TypeError("Ferrum structure hit testing requires a QPoint")
		projection = self._require_projection()
		for item in self._view.items(point):
			current = item
			while current is not None:
				target = projection.item_targets.get(current)
				if target is not None:
					# Canvas targets deliberately carry only opaque durable document
					# identities. Rust restores chemical type when an operation needs it.
					if target.kind == "document_object":
						return target.durable_selection_key()
					break
				current = current.parentItem()
		return None

	#============================================
	def durable_atom_at_viewport_point(self, point: PySide6.QtCore.QPoint) -> str | None:
		"""Return the topmost durable Rust atom hit by one viewport point."""
		target = self.durable_structure_at_viewport_point(point)
		if target is None:
			return None
		resolved = self.structure_targets_for_ids((target[1],))
		import ferrum_qt.ferrum.engine as engine
		if resolved[0].kind is not engine.StructureTargetKindV1.atom:
			return None
		return resolved[0].object_id

	#============================================
	def durable_attachment_atom_at_viewport_point(
			self, point: PySide6.QtCore.QPoint,
			) -> str | None:
		"""Return one C6 attachment atom from a hit or unique nearby projection.

		Rendered atom hits retain their established precedence.  Only when no
		rendered atom was hit, the Attach Cyclohexane Ring gesture may resolve an
		unlabelled projection atom within six device pixels.  This is intentionally
		not a general canvas-picker tolerance.
		"""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPoint):
			raise TypeError("Ferrum attachment hit testing requires a QPoint")
		picked = self._durable_or_unique_projection_atom_at_viewport_point(point)
		return picked.object_id if picked is not None else None

	def _durable_or_unique_projection_atom_at_viewport_point(
			self, point: PySide6.QtCore.QPoint,
			) -> _ImplicitAtomPick | None:
		"""Prefer one rendered atom, else one unique nearby projection atom."""
		rendered_atom_id = self.durable_atom_at_viewport_point(point)
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		if rendered_atom_id is not None:
			rendered_atoms: list[_ImplicitAtomPick] = []
			for molecule in self._document_observation.projection.molecules:
				for atom in molecule.atoms:
					if (
						type(atom.document_object_id) is str and atom.document_object_id
						and atom.document_object_id == rendered_atom_id
					):
						rendered_atoms.append(_ImplicitAtomPick(atom.document_object_id))
			return rendered_atoms[0] if len(rendered_atoms) == 1 else None
		radius_squared = _IMPLICIT_ATOM_PICK_RADIUS_PX ** 2
		nearest_distance: int | None = None
		nearest_atoms: list[_ImplicitAtomPick] = []
		for molecule in self._document_observation.projection.molecules:
			for atom in molecule.atoms:
				if type(atom.document_object_id) is not str or not atom.document_object_id:
					continue
				viewport_atom = self._view.mapFromScene(PySide6.QtCore.QPointF(
					atom.position.x, atom.position.y,
				))
				delta_x = viewport_atom.x() - point.x()
				delta_y = viewport_atom.y() - point.y()
				distance_squared = delta_x * delta_x + delta_y * delta_y
				if distance_squared > radius_squared:
					continue
				if nearest_distance is None or distance_squared < nearest_distance:
					nearest_distance = distance_squared
					nearest_atoms = [_ImplicitAtomPick(atom.document_object_id)]
				elif distance_squared == nearest_distance:
					nearest_atoms.append(_ImplicitAtomPick(atom.document_object_id))
		return nearest_atoms[0] if len(nearest_atoms) == 1 else None

	#============================================
	def durable_atom_at_scene_position(self, point: PySide6.QtCore.QPointF) -> str | None:
		"""Return one exact installed Rust atom at a keyboard cursor position.

		Pointer tools intentionally use rendered-item hit testing.  Keyboard tools
		instead address a document coordinate directly, including implicit-carbon
		atoms that deliberately have no visible text item to hit.
		"""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPointF):
			raise TypeError("Ferrum keyboard atom lookup requires a QPointF")
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		matches = tuple(
			atom.document_object_id
			for molecule in self._document_observation.projection.molecules
			for atom in molecule.atoms
			if (
				atom.document_object_id is not None
				and atom.position.x == point.x()
				and atom.position.y == point.y()
			)
		)
		if len(matches) > 1:
			raise FerrumNativeDocumentTabError(
				"more than one durable atom occupies the keyboard cursor position",
			)
		return None if not matches else matches[0]

	#============================================
	def durable_atom_scene_position(self, atom_id: str) -> PySide6.QtCore.QPointF:
		"""Return the exact installed Rust point for one durable atom."""
		self._require_live()
		if type(atom_id) is not str or not atom_id:
			raise TypeError("Ferrum atom position requires a durable atom identifier")
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			for atom in molecule.atoms:
				if atom.document_object_id == atom_id:
					return PySide6.QtCore.QPointF(atom.position.x, atom.position.y)
		raise FerrumNativeDocumentTabError("atom is not in the current Rust projection")

	#============================================
	def current_document_observation(self) -> object:
		"""Return the exact immutable observation installed in the Ferrum scene."""
		self._require_mutable()
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		return self._document_observation

	#============================================
	def selected_molecule_information_targets(self) -> tuple[object, ...]:
		"""Return every selected target so unsupported artwork cannot disappear."""
		if self._disposed or self.requires_refresh:
			return ()
		projection = self._controller.projection
		return () if projection is None else tuple(projection.selected_targets())

	#============================================
	def apply_prepared_molecule_coordinates(self, prepared: object) -> object:
		"""Commit one exact worker-prepared coordinate update through Rust."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(prepared) is not engine.PreparedMoleculeCoordinatesV1:
			raise TypeError("Ferrum coordinate update requires exact frozen Ferrum data")
		result = self._session.apply_molecule_coordinates_v1(
			self.current_snapshot.revision, prepared,
		)
		self._install_mutation_result(result)
		return result

	#============================================
	def apply_prepared_clean_geometry(
			self, prepared: object, restore: tuple[str, ...]) -> object:
		"""Commit one exact multi-molecule clean result and restore selection."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(prepared) is not engine.PreparedCleanGeometryV1:
			raise TypeError("Ferrum clean geometry requires exact frozen Ferrum data")
		if type(restore) is not tuple or any(
			type(object_id) is not str or not object_id for object_id in restore
		):
			raise TypeError("Ferrum clean geometry requires exact durable object IDs")
		result = self._session.apply_clean_geometry_v1(
			self.current_snapshot.revision, prepared,
		)
		self._install_mutation_result(result, restore)
		return result

	#============================================
	def add_atom_at(self, molecule_object_id: str, element: str,
			x: float, y: float) -> object:
		"""Create one Rust-allocated free-standing atom at an exact scene point."""
		self._require_mutable()
		if type(molecule_object_id) is not str or type(element) is not str:
			raise TypeError("Ferrum atom insertion requires molecule and element strings")
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum atom insertion coordinates must be floats")
		self._require_canvas_authorable_molecule(molecule_object_id)
		import ferrum_qt.ferrum.engine as engine
		operation = engine.DocumentOperationV1.create_atom_v1(
			molecule_object_id, element, x, y, 0.0,
		)
		result = self._apply_current_document_operation_v1(operation)
		outcome = result.outcome
		if outcome.kind != "atom_created_v1" or outcome.atom_created is None:
			raise FerrumNativeDocumentTabError("Ferrum atom creation returned an unknown operation outcome")
		self._install_mutation_result(
				result, (outcome.atom_created.atom_identifier,),
			)
		return result

	#============================================
	def insert_prepared_molecule(self, molecule: object) -> object:
		"""Commit one frozen worker-built molecule as an atomic Rust edit."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(molecule) is not engine.MoleculeInsertionV1:
			raise TypeError("Ferrum molecule insertion requires exact frozen Ferrum data")
		revision = self.current_snapshot.revision
		operation = engine.DocumentOperationV1.insert_molecule_v1(molecule)
		request = operation.transition_request_v1(revision)
		prepared = self.prepare_session_operation_transition_v1(request)
		result = self.commit_session_operation_transition_v1(prepared)
		outcome = result.outcome
		if outcome.kind != "molecule_inserted_v1" or outcome.molecule_inserted is None:
			raise FerrumNativeDocumentTabError(
				"Ferrum molecule insertion returned an unknown operation outcome",
			)
		selection = tuple(outcome.molecule_inserted.atom_identifiers)
		self._install_mutation_result(result, selection)
		return result

	#============================================
	def change_selected_atom_element(self, element: str) -> object:
		"""Submit one selected atom element change against the installed revision."""
		self._require_mutable()
		address = self.selected_molecule_atom_address()
		result = self._live_document_session_v1.set_atom_element_v1(
			address.revision, address.digest, address.molecule_id, address.atom_id, element,
		)
		self._install_mutation_result(result, (address.atom_id,))
		return result

	#============================================
	def _apply_current_document_operation_v1(self, operation: object) -> object:
		"""Apply one exact document operation against the installed snapshot revision."""
		import ferrum_qt.ferrum.engine as engine
		if type(operation) is not engine.DocumentOperationV1:
			raise TypeError("Ferrum document submission requires an exact document operation")
		return self._session.apply_document_operation_v1(
			self.current_snapshot.revision, operation,
		)

	#============================================
	def move_atom_to(self, atom_id: str, x: float, y: float) -> object:
		"""Move one durable atom to an exact finite scene point through Rust."""
		self._require_mutable()
		if type(atom_id) is not str or not atom_id:
			raise TypeError("Ferrum atom movement requires a durable atom identifier")
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum atom movement coordinates must be floats")
		address = self.molecule_atom_address(atom_id)
		result = self._live_document_session_v1.set_atom_position_v1(
			address.revision, address.digest, address.molecule_id, address.atom_id, x, y, 0.0,
		)
		self._install_mutation_result(result, (address.atom_id,))
		return result

	#============================================
	def create_wavy(self, start_x: float, start_y: float,
			end_x: float, end_y: float) -> object:
		"""Create one Rust-owned Wavy path from two exact scene endpoints."""
		self._require_mutable()
		if any(type(value) is not float for value in (start_x, start_y, end_x, end_y)):
			raise TypeError("Ferrum Wavy creation coordinates must be floats")
		revision = self.current_snapshot.revision
		prepared = self._session.prepare_create_wavy_v1(
			revision, start_x, start_y, end_x, end_y,
		)
		result = self._session.commit_create_wavy(revision, prepared)
		prior_wavy_ids = frozenset(
			entry.polyline.target.document_object_id
			for entry in self.current_document_observation().projection.presentation_stack.entries
			if entry.kind == "wavy"
			and entry.polyline is not None
			and entry.polyline.target.record_kind == "wavy"
		)
		created = tuple(
			entry.polyline.target.document_object_id
			for entry in result.observation.projection.presentation_stack.entries
			if entry.kind == "wavy"
			and entry.polyline is not None
			and entry.polyline.target.record_kind == "wavy"
			and entry.polyline.target.document_object_id not in prior_wavy_ids
		)
		if len(created) != 1:
			self._install_mutation_result(result)
			raise FerrumNativeDocumentTabError(
				"accepted Wavy creation has no unique durable projected target",
			)
		self._install_mutation_result(result, (created[0],))
		return result

	#============================================
	def undo(self) -> object:
		"""Ask Rust to undo exactly the currently installed authoritative revision."""
		self._require_mutable()
		result = self._session.undo(self.current_snapshot.revision)
		self._install_mutation_result(result)
		return result

	#============================================
	def redo(self) -> object:
		"""Ask Rust to redo exactly the currently installed authoritative revision."""
		self._require_mutable()
		result = self._session.redo(self.current_snapshot.revision)
		self._install_mutation_result(result)
		return result
	#============================================
	def refresh_authoritative(self) -> bool:
		"""Install the accepted Rust revision after a prior projection failure."""
		self._require_live()
		if self._pending_snapshot is None:
			return True
		try:
			observation = self._publish_live_render_plan_v1(self._pending_snapshot.revision)
			installed = self._install_observation(observation)
		except FerrumNativeDocumentTabError:
			return False
		if not installed:
			return False
		self._restore_pending_durable_selection()
		self._pending_result = None
		self._pending_snapshot = None
		self._pending_durable_selection = None
		self._pending_focus_atom_object_id = None
		self.selection_changed.emit()
		return True
	#============================================
	def dispose(self) -> None:
		"""Invalidate render delivery before disposing the graphics view."""
		if self._disposed:
			return
		self._require_live_smarts_invalidation_v1("tab_disposed")
		self.clear_structure_action_selection_v1()
		self._disposed = True
		self._retire_scene_selection_bridge()
		self._controller.dispose()
		self._dispose_document_display_refreshables()
		self._view.setScene(None)
		self._view.deleteLater()
		self._render_observation = None
	#============================================
	def _refresh_from_current_revision(self) -> None:
		"""Observe the current Rust revision and install it only if projection succeeds."""
		snapshot = self._session.snapshot()
		observation = self._publish_live_render_plan_v1(snapshot.revision)
		if not self._install_observation(observation):
			raise FerrumNativeDocumentTabError(
				"Ferrum tab could not install its render observation",
			)

	#============================================
	def _install_observation(self, observation: object) -> bool:
		"""Install exactly one current render observation through a provenance latch."""
		self._require_live()
		render_observation = observation.render_observation
		snapshot = render_observation.document.snapshot
		latch = ferrum_qt.canvas.ferrum_render_projection.RenderProjectionLatch(
			snapshot.revision, snapshot.digest, self._controller.generation,
		)
		prior_scene_selection_source = self._scene_selection_source
		prior_scene_selection_connection = self._scene_selection_connection
		self.clear_structure_action_selection_v1()
		self._retire_scene_selection_bridge()
		installed = self._install_published_render_plan_v1(
			self._controller.replace, render_observation, latch, observation.presentation_plan,
		)
		if not installed:
			if prior_scene_selection_source is not None and prior_scene_selection_connection is not None:
				self._scene_selection_source = prior_scene_selection_source
				self._scene_selection_connection = prior_scene_selection_source.selectionChanged.connect(
					self._on_scene_selection_changed,
				)
			return False
		self._snapshot = snapshot
		self._document_observation = render_observation.document
		self._render_observation = render_observation
		current_scene = self._view.scene()
		if current_scene is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed graphics scene")
		self._scene_selection_source = current_scene
		self._scene_selection_connection = current_scene.selectionChanged.connect(
			self._on_scene_selection_changed,
		)
		return installed

	#============================================
	@PySide6.QtCore.Slot()
	def _on_scene_selection_changed(self) -> None:
		"""Forward current-scene selection only while this tab remains live."""
		if not self._disposed:
			self.selection_changed.emit()

	#============================================
	def _retire_scene_selection_bridge(self) -> None:
		"""Disconnect and clear the tab-owned current scene selection bridge."""
		source = getattr(self, "_scene_selection_source", None)
		connection = getattr(self, "_scene_selection_connection", None)
		if source is not None and connection is not None:
			source.selectionChanged.disconnect(connection)
		self._scene_selection_source = None
		self._scene_selection_connection = None

	#============================================
	def _install_mutation_result(self, result: object,
			durable_selection: tuple[str, ...] | None = None, *,
			focus_atom_object_id: str | None = None) -> None:
		"""Install a Rust-accepted result or retain exact recovery ownership."""
		if focus_atom_object_id is not None and (
			type(focus_atom_object_id) is not str or not focus_atom_object_id
		):
			raise TypeError("Ferrum focus atom requires a durable document object identifier")
		if durable_selection is not None and (
			type(durable_selection) is not tuple
			or any(type(object_id) is not str or not object_id for object_id in durable_selection)
		):
			raise TypeError("Ferrum selection restore requires exact durable document object IDs")
		keyboard_cursor_scene = self._view.keyboard_cursor_scene()
		authoritative = result.observation
		self._pending_result = result
		self._pending_snapshot = authoritative.snapshot
		self._pending_durable_selection = durable_selection
		self._pending_focus_atom_object_id = focus_atom_object_id
		self.clear_structure_action_selection_v1()
		try:
			observation = self._publish_live_render_plan_v1(authoritative.snapshot.revision)
		except FerrumNativeDocumentTabError as exc:
			raise FerrumNativeDocumentTabMutationPresentationError(result) from exc
		try:
			installed = self._install_observation(observation)
		except Exception as exc:
			raise FerrumNativeDocumentTabMutationPresentationError(result) from exc
		if not installed:
			raise FerrumNativeDocumentTabMutationPresentationError(result)
		if keyboard_cursor_scene is not None:
			self._view.set_keyboard_cursor_scene(keyboard_cursor_scene)
		self._restore_pending_durable_selection()
		self._pending_result = None
		self._pending_snapshot = None
		self._pending_durable_selection = None
		self._pending_focus_atom_object_id = None
		self.selection_changed.emit()

	#============================================
	def _restore_pending_durable_selection(self) -> None:
		"""Apply a post-mutation selection only after its replacement scene installs."""
		focus_atom_object_id = self._pending_focus_atom_object_id
		if focus_atom_object_id is not None:
			projection = self._require_projection()
			import ferrum_qt.ferrum.engine as engine
			targets = self._resolve_structure_targets_for_ids((focus_atom_object_id,))
			if targets[0].kind is not engine.StructureTargetKindV1.atom:
				raise FerrumNativeDocumentTabError(
					"Ferrum focus atom does not map to one installed render target",
				)
			projection.select_durable((("document_object", focus_atom_object_id),))
			return
		if self._pending_durable_selection is None:
			return
		self._require_projection().select_durable(tuple(
			("document_object", object_id)
		for object_id in self._pending_durable_selection
		))

	#============================================
	def _selected_atom_identifier(self) -> str:
		"""Return the single current durable atom selected for a Rust operation."""
		return self._selected_atom_identifiers(1)[0]

	#============================================
	def _selected_atom_identifiers(self, expected: int) -> tuple[str, ...]:
		"""Return an exact count of current durable atom operation selectors."""
		import ferrum_qt.ferrum.engine as engine
		targets = self.selected_structure_targets()
		if (
			len(targets) != expected
			or any(target.kind is not engine.StructureTargetKindV1.atom for target in targets)
		):
			raise FerrumNativeDocumentTabError(
				f"select exactly {expected} atom{'s' if expected != 1 else ''} first",
			)
		return tuple(target.object_id for target in targets)

	#============================================
	def _selected_presentation_root_selectors(self) -> tuple[tuple[str, object], ...]:
		"""Resolve selected generic canvas IDs into exact Rust presentation selectors."""
		self._require_live()
		self._require_current_projection()
		import ferrum_qt.ferrum.engine as engine
		observation = self._document_observation
		if type(observation) is not engine.SessionDocumentObservationV1:
			raise FerrumNativeDocumentTabError(
				"Ferrum tab has no installed document projection",
			)
		document_projection = observation.projection
		if type(document_projection) is not engine.DocumentProjectionV1:
			raise FerrumNativeDocumentTabError(
				"Ferrum tab has no exact Rust document projection",
			)
		direct_roots = document_projection.direct_roots
		if type(direct_roots) is not tuple:
			raise FerrumNativeDocumentTabError(
				"Rust document direct roots are not an exact DTO tuple",
			)
		roots_by_id = {}
		paint_orders = set()
		for root in direct_roots:
			if type(root) is not engine.DocumentDirectRootV1:
				raise FerrumNativeDocumentTabError(
					"Rust document direct root has the wrong DTO type",
				)
			object_id = root.document_object_id
			kind = root.kind
			paint_order = root.paint_order
			if type(object_id) is not str or not object_id:
				raise FerrumNativeDocumentTabError(
					"Rust document direct-root identity is invalid",
				)
			if type(kind) is not str or not kind:
				raise FerrumNativeDocumentTabError(
					"Rust document direct-root kind is invalid",
				)
			if type(paint_order) is not int or paint_order < 0 or paint_order >= 2**32:
				raise FerrumNativeDocumentTabError(
					"Rust document direct-root paint order is invalid",
				)
			if object_id in roots_by_id or paint_order in paint_orders:
				raise FerrumNativeDocumentTabError(
					"Rust document direct roots are not unique",
				)
			roots_by_id[object_id] = (kind, paint_order)
			paint_orders.add(paint_order)
		selected = self._require_projection().selected_durable_targets()
		selected_ids = set()
		resolved = []
		presentation_kinds = engine.DocumentPresentationRootKindV1
		for target in selected:
			if target.kind != "document_object":
				raise FerrumNativeDocumentTabError(
					"selected canvas target is not a generic document object",
				)
			object_id = target.document_object_id
			if type(object_id) is not str or not object_id:
				raise FerrumNativeDocumentTabError(
					"selected canvas target lacks a durable document-object identity",
				)
			if object_id in selected_ids:
				raise FerrumNativeDocumentTabError(
					"selected canvas targets are not unique",
				)
			selected_ids.add(object_id)
			try:
				root_kind, paint_order = roots_by_id[object_id]
			except KeyError as exc:
				raise FerrumNativeDocumentTabError(
					"selected canvas target is absent from Rust document direct roots",
				) from exc
			if root_kind == "molecule":
				raise FerrumNativeDocumentTabError(
					"selected canvas target is a molecule, not a presentation root",
				)
			if root_kind == "rejected_presentation":
				raise FerrumNativeDocumentTabError(
					"selected canvas target is a rejected presentation root",
				)
			if root_kind not in _PRESENTATION_ROOT_KIND_NAMES:
				raise FerrumNativeDocumentTabError(
					"selected canvas target has an unsupported direct-root kind",
				)
			selector_kind = getattr(presentation_kinds, root_kind, None)
			if type(selector_kind) is not presentation_kinds:
				raise FerrumNativeDocumentTabError(
					"Rust presentation selector kind is unavailable",
				)
			resolved.append((paint_order, object_id, selector_kind))
		return tuple(
			(object_id, selector_kind)
			for _paint_order, object_id, selector_kind in sorted(resolved)
		)
	#============================================
	def _require_projection(self) -> object:
		"""Return the one current installed projection needed for local selection."""
		projection = self._controller.projection
		if projection is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed projection")
		return projection
	#============================================
	def _require_current_projection(self) -> None:
		"""Block actions that would incorrectly use a stale visible snapshot."""
		if self.requires_refresh:
			raise FerrumNativeDocumentTabError(
				"refresh the authoritative Rust observation before saving or editing",
			)
	#============================================
	def _require_mutable(self) -> None:
		"""Require a live tab whose Rust authority and Qt scene agree exactly."""
		self._require_live()
		self._require_current_projection()
	#============================================
	def _require_live(self) -> None:
		"""Reject operations after terminal disposal rather than reviving a tab."""
		if self._disposed:
			raise FerrumNativeDocumentTabError("Ferrum document tab has been disposed")
	#============================================
	def _dispose_partial_resources(self) -> None:
		"""Dispose partial projection resources after construction failure."""
		if getattr(self, "_session", None) is not None:
			self._invalidate_live_smarts_query_v1("construction_failure")
		self.clear_structure_action_selection_v1()
		self._retire_scene_selection_bridge()
		controller = getattr(self, "_controller", None)
		if controller is not None:
			controller.dispose()
		if getattr(self, "_document_display_refreshables", None) is not None:
			self._dispose_document_display_refreshables()
		view = getattr(self, "_view", None)
		if view is not None:
			view.setScene(None)
			view.deleteLater()
