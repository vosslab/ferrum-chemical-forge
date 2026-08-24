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
import ferrum_qt.ferrum.user_templates as native_user_templates
import ferrum_qt.ferrum.tab_view_state
import ferrum_qt.ferrum.wavy_properties as native_wavy_properties
import ferrum_qt.ferrum.document_tab_publication as native_publication
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors
import ferrum_qt.ferrum.document_tab_selection as native_document_tab_selection
import ferrum_qt.ferrum.drawing_standard as native_drawing_standard
import ferrum_qt.ferrum.explicit_fragment_tab as native_explicit_fragment
import ferrum_qt.ferrum.local_cdml_origin_tab as native_local_cdml_origin_tab
import ferrum_qt.ferrum.catalog_palette as native_catalog_palette
import ferrum_qt.ferrum.live_document_transaction as native_live_document_transaction
import ferrum_qt.ferrum.smarts_selected_root_capture_tab as native_smarts_selected_root_capture


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


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _ImplicitAtomPick:
	"""One validated identity pair selected from the installed projection."""

	object_id: str
	source_id: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeChoice:
	"""One durable molecule choice copied from the installed Rust projection."""

	object_id: str
	label: str
	source_order: int


#============================================
class FerrumNativeDocumentTab(
		native_document_tab_selection.FerrumNativeDocumentSelectionMixin,
		native_smarts_selected_root_capture.FerrumNativeSmartsSelectedRootCaptureTabMixin,
		native_live_document_transaction.FerrumLiveDocumentTransactionMixin,
		native_catalog_palette.FerrumNativeCatalogPlacementTabMixin,
		native_local_cdml_origin_tab.FerrumNativeLocalCdmlOriginTabMixin,
		native_regular_ring_tab.FerrumNativeRegularRingTabMixin,
		native_attached_cyclohexane_tab.FerrumNativeAttachedCyclohexaneTabMixin,
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
		native_user_templates.FerrumNativeUserTemplateTabMixin,
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
	def __init__(self, cdml: str, title: str) -> None:
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
			view = ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView(self)
			controller = ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjectionController(
				view, resource,
			)
			self._initialize(title, session, view, controller)
			self._refresh_from_current_revision()
		except Exception:
				self._retire_partial_resources()
				raise

	#============================================
	@classmethod
	def from_session(cls, session: object, title: str) -> "FerrumNativeDocumentTab":
		"""Project one detached Rust-owned session without loading CDML in Qt."""
		tab = native_tab_construction.create_document_tab_from_session(
			cls, session, title,
		)
		return tab

	#============================================
	@classmethod
	def from_admitted_local_open(
			cls, session: object, title: str, observation: object,
			) -> "FerrumNativeDocumentTab":
		"""Install one worker-prepared session and matching immutable observation."""
		tab = native_tab_construction.create_admitted_local_document_tab(
			cls, session, title, observation, FerrumNativeDocumentTabError,
		)
		return tab

	#============================================
	@classmethod
	def _from_fixture(cls, title: str, session: object,
			controller: object) -> "FerrumNativeDocumentTab":
		"""Construct a tab with test-only owned-value collaborators."""
		if type(title) is not str:
			raise TypeError("Ferrum document tab fixture title must be a string")
		tab = cls.__new__(cls)
		PySide6.QtWidgets.QWidget.__init__(tab)
		try:
			view = ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView(tab)
			tab._initialize(title, session, view, controller)
			tab._refresh_from_current_revision()
		except Exception:
			tab._retire_partial_resources()
			raise
		return tab
	#============================================
	def _initialize(self, title: str, session: object,
			view: PySide6.QtWidgets.QGraphicsView, controller: object) -> None:
		"""Install one ownership graph shared by production and fixture construction."""
		self._title = title
		self._view = view
		self._controller = controller
		self._initialize_live_document_transaction_v1(session)
		self._snapshot: object | None = None
		self._document_observation: object | None = None
		self._render_observation: object | None = None
		self._pending_result: object | None = None
		self._pending_snapshot: object | None = None
		self._pending_durable_selection: tuple[tuple[str, str], ...] | None = None
		self._selection_scene: PySide6.QtWidgets.QGraphicsScene | None = None
		self._file_path: pathlib.Path | None = None
		self._initialize_local_cdml_origin()
		self._disposed = False
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
		"""Return whether this tab has terminally retired its projection boundary."""
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
		projection = self._require_projection()
		if any(("atom", atom_id) not in projection.durable_items for atom_id in atom_ids):
			raise FerrumNativeDocumentTabError("selected atom is not in the current projection")
		projection.select_durable(tuple(("atom", atom_id) for atom_id in atom_ids))
	#============================================
	def select_bond(self, bond_id: str) -> None:
		"""Select one current durable bond by Rust identifier for Ferrum actions."""
		self._require_live()
		if type(bond_id) is not str or not bond_id:
			raise ValueError("Ferrum bond selection requires a non-empty identifier")
		projection = self._require_projection()
		if ("bond", bond_id) not in projection.durable_items:
			raise FerrumNativeDocumentTabError(
				"selected bond is not in the current projection",
			)
		projection.select_durable((("bond", bond_id),))
	#============================================
	def durable_structure_at_viewport_point(
			self, point: PySide6.QtCore.QPoint,
			) -> tuple[str, str] | None:
		"""Return the topmost installed durable atom or bond at one viewport point."""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPoint):
			raise TypeError("Ferrum structure hit testing requires a QPoint")
		projection = self._require_projection()
		for item in self._view.items(point):
			current = item
			while current is not None:
				target = projection.item_targets.get(current)
				if target is not None:
					# The installed projection owns both this Qt item mapping and the
					# durable target vocabulary.  Keep window tools independent of
					# graphics-item classes and transient scene decoration.
					if (
						target.kind in ("atom", "bond")
						and target.identifier is not None
					):
						return target.kind, target.identifier
					# An overlaid presentation root is not durable chemical content.
					# Move to the next hit-stack item so it cannot mask a bond or
					# atom beneath it; climbing farther would only revisit its root.
					break
				current = current.parentItem()
		return None

	#============================================
	def durable_atom_at_viewport_point(self, point: PySide6.QtCore.QPoint) -> str | None:
		"""Return the topmost durable Rust atom hit by one viewport point."""
		target = self.durable_structure_at_viewport_point(point)
		if target is None or target[0] != "atom":
			return None
		return target[1]

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
						type(atom.id) is str and atom.id
						and type(atom.source_id) is str and atom.source_id
						and atom.source_id == rendered_atom_id
					):
						rendered_atoms.append(_ImplicitAtomPick(atom.id, atom.source_id))
			return rendered_atoms[0] if len(rendered_atoms) == 1 else None
		radius_squared = _IMPLICIT_ATOM_PICK_RADIUS_PX ** 2
		nearest_distance: int | None = None
		nearest_atoms: list[_ImplicitAtomPick] = []
		for molecule in self._document_observation.projection.molecules:
			for atom in molecule.atoms:
				if (
					type(atom.id) is not str or not atom.id
					or type(atom.source_id) is not str or not atom.source_id
				):
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
					nearest_atoms = [_ImplicitAtomPick(atom.id, atom.source_id)]
				elif distance_squared == nearest_distance:
					nearest_atoms.append(_ImplicitAtomPick(atom.id, atom.source_id))
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
			atom.source_id
			for molecule in self._document_observation.projection.molecules
			for atom in molecule.atoms
			if (
				atom.source_id is not None
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
				if atom.source_id == atom_id:
					return PySide6.QtCore.QPointF(atom.position.x, atom.position.y)
		raise FerrumNativeDocumentTabError("atom is not in the current Rust projection")

	#============================================
	def durable_molecule_choices(self) -> tuple[FerrumNativeMoleculeChoice, ...]:
		"""Return source-ordered durable molecules from the installed observation."""
		self._require_mutable()
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		choices = []
		ordinal = 0
		for molecule in self._document_observation.projection.molecules:
			if molecule.id is None:
				continue
			ordinal += 1
			position_label = f"Molecule {ordinal}"
			name = molecule.name
			label = position_label if name is None or not name.strip() else (
				f"{name} ({position_label})"
			)
			choices.append(
				FerrumNativeMoleculeChoice(molecule.id, label, molecule.source_order),
			)
		return tuple(choices)

	#============================================
	def canvas_authorable_molecule_choices(self) -> tuple[FerrumNativeMoleculeChoice, ...]:
		"""Return durable molecules proven by the installed Rust render plans."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		observation = self._render_observation
		if type(observation) is not engine.RenderObservationV1:
			raise FerrumNativeDocumentTabError(
				"Ferrum tab has no exact installed Rust render observation",
			)
		if (
			observation.document.snapshot.revision != self.current_snapshot.revision
			or observation.document.snapshot.digest != self.current_snapshot.digest
		):
			raise FerrumNativeDocumentTabError(
				"installed Rust render observation does not match the current document snapshot",
			)
		plan_ids = {
			plan.molecule.id
			for plan in observation.molecule_plans
			if plan.molecule.id is not None
		}
		return tuple(
			choice for choice in self.durable_molecule_choices()
			if choice.object_id in plan_ids
		)

	#============================================
	def _require_canvas_authorable_molecule(self, molecule_object_id: str) -> None:
		"""Require exact installed Rust render evidence before a canvas mutation."""
		if not any(
			choice.object_id == molecule_object_id
			for choice in self.canvas_authorable_molecule_choices()
		):
			raise FerrumNativeDocumentTabUnrenderableMoleculeError(molecule_object_id)

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
			self, prepared: object,
			restore: tuple[tuple[str, str], ...]) -> object:
		"""Commit one exact multi-molecule clean result and restore selection."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(prepared) is not engine.PreparedCleanGeometryV1:
			raise TypeError("Ferrum clean geometry requires exact frozen Ferrum data")
		if type(restore) is not tuple or any(
			type(item) is not tuple
			or len(item) != 2
			or type(item[0]) is not str
			or item[0] not in ("atom", "bond")
			or type(item[1]) is not str
			or not item[1]
			for item in restore
		):
			raise TypeError("Ferrum clean geometry requires an exact selection tuple")
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
		revision = self.current_snapshot.revision
		prepared = self._session.prepare_create_atom_v1(
			revision, molecule_object_id, element, x, y, 0.0,
		)
		result = self._session.commit_create_atom(revision, prepared)
		self._install_mutation_result(
			result, (("atom", prepared.identifier),),
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
		prepared = self._session.prepare_admitted_molecule_insertion_v1(revision, molecule)
		result = self._session.commit_admitted_molecule_insertion_v1(revision, prepared)
		self._install_mutation_result(result)
		return result

	#============================================
	def change_selected_atom_element(self, element: str) -> object:
		"""Submit one selected atom element change against the installed revision."""
		self._require_mutable()
		selected = self._selected_atom_identifier()
		import ferrum_qt.ferrum.engine as engine
		operation = engine.DocumentOperationV1.set_atom_element(selected, element)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("atom", selected),))
		return result

	#============================================
	def move_atom_to(self, atom_id: str, x: float, y: float) -> object:
		"""Move one durable atom to an exact finite scene point through Rust."""
		self._require_mutable()
		if type(atom_id) is not str or not atom_id:
			raise TypeError("Ferrum atom movement requires a durable atom identifier")
		if type(x) is not float or type(y) is not float:
			raise TypeError("Ferrum atom movement coordinates must be floats")
		if ("atom", atom_id) not in self._require_projection().durable_items:
			raise FerrumNativeDocumentTabError("moved atom is not in the current projection")
		import ferrum_qt.ferrum.engine as engine
		operation = engine.DocumentOperationV1.set_atom_position(atom_id, x, y, 0.0)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("atom", atom_id),))
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
		created = tuple(
			root.polyline for root in result.observation.projection.presentation_stack.roots
			if root.kind == "wavy"
			and root.polyline.target.source_id == prepared.identifier
		)
		if len(created) != 1 or created[0].target.id is None:
			self._install_mutation_result(result)
			raise FerrumNativeDocumentTabError(
				"accepted Wavy creation has no unique durable projected target",
			)
		self._install_mutation_result(result, (("polyline", created[0].target.id),))
		return result

	#============================================
	def _atom_object_ids(self, source_ids: tuple[str, ...]) -> tuple[str, ...]:
		"""Resolve selected render IDs through the exact installed Rust projection."""
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		requested = frozenset(source_ids)
		resolved = {}
		for molecule in self._document_observation.projection.molecules:
			for atom in molecule.atoms:
				if atom.source_id in requested and atom.id is not None:
					resolved[atom.source_id] = atom.id
		if resolved.keys() != requested:
			raise FerrumNativeDocumentTabError(
				"selected atom lacks a current durable document selector",
			)
		return tuple(resolved[source_id] for source_id in source_ids)

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
		self.selection_changed.emit()
		return True
	#============================================
	def dispose(self) -> None:
		"""Terminally invalidate render delivery before retiring the graphics view."""
		if self._disposed:
			return
		self._require_live_smarts_retirement_v1("tab_disposed")
		self._disposed = True
		self._controller.dispose()
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
		installed = self._install_published_render_plan_v1(
			self._controller.replace, render_observation, latch, observation.presentation_plan,
		)
		if installed:
			self._snapshot = snapshot
			self._document_observation = render_observation.document
			self._render_observation = render_observation
			self._connect_current_selection_scene()
		return installed

	#============================================
	def _install_mutation_result(self, result: object,
			durable_selection: tuple[tuple[str, str], ...] | None = None) -> None:
		"""Install a Rust-accepted result or retain exact recovery ownership."""
		keyboard_cursor_scene = self._view.keyboard_cursor_scene()
		authoritative = result.observation
		self._pending_result = result
		self._pending_snapshot = authoritative.snapshot
		self._pending_durable_selection = durable_selection
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
		self.selection_changed.emit()

	#============================================
	def _restore_pending_durable_selection(self) -> None:
		"""Apply a post-mutation selection only after its replacement scene installs."""
		if self._pending_durable_selection is None:
			return
		self._require_projection().select_durable(self._pending_durable_selection)

	#============================================
	def _connect_current_selection_scene(self) -> None:
		"""Forward current disposable selection changes without retaining scene authority."""
		scene = self._view.scene()
		if scene is None or scene is self._selection_scene:
			return
		scene.selectionChanged.connect(self._forward_selection_changed)
		self._selection_scene = scene
		self.selection_changed.emit()

	#============================================
	@PySide6.QtCore.Slot()
	def _forward_selection_changed(self) -> None:
		"""Forward one scene selection event through a typed Qt slot."""
		self.selection_changed.emit()

	#============================================
	def _selected_atom_identifier(self) -> str:
		"""Return the single current durable atom selected for a Rust operation."""
		return self._selected_atom_identifiers(1)[0]

	#============================================
	def _selected_atom_address(self) -> tuple[str, str]:
		"""Return the selected atom's exact direct-root molecule and atom IDs."""
		atom_id = self._selected_atom_identifier()
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			if molecule.source_id is None:
				continue
			if any(atom.source_id == atom_id for atom in molecule.atoms):
				return molecule.source_id, atom_id
		raise FerrumNativeDocumentTabError(
			"selected atom has no durable direct-root molecule address",
		)
	#============================================
	def _selected_atom_identifiers(self, expected: int) -> tuple[str, ...]:
		"""Return an exact count of current durable atom operation selectors."""
		return self._selected_durable_identifiers(expected, "atom")
	#============================================
	def _selected_durable_identifiers(self, expected: int,
			kind: str) -> tuple[str, ...]:
		"""Return exact current durable selectors of one closed record kind."""
		selected = self._require_projection().selected_durable_targets()
		if len(selected) != expected or any(target.kind != kind for target in selected):
			raise FerrumNativeDocumentTabError(
				f"select exactly {expected} {kind}{'s' if expected != 1 else ''} first",
			)
		identifiers = tuple(target.identifier for target in selected)
		if any(identifier is None for identifier in identifiers):
			raise FerrumNativeDocumentTabError(
				f"selected {kind} lacks a durable identifier",
			)
		return tuple(identifier for identifier in identifiers if identifier is not None)
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
	def _retire_partial_resources(self) -> None:
		"""Dispose partial projection resources after construction failure."""
		if getattr(self, "_session", None) is not None:
			self._retire_live_smarts_query_v1("construction_failure")
		controller = getattr(self, "_controller", None)
		if controller is not None:
			controller.dispose()
		view = getattr(self, "_view", None)
		if view is not None:
			view.setScene(None)
			view.deleteLater()
