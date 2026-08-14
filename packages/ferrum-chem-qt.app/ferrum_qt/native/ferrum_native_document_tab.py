"""Native Rust-owned document tab with a disposable Ferrum Qt projection."""

# Standard Library
import dataclasses
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.native.ferrum_native_bracket_creation as native_bracket_creation
import ferrum_qt.native.ferrum_native_document_tab_construction as native_tab_construction
import ferrum_qt.native.ferrum_native_geometric_properties as native_geometric_properties
import ferrum_qt.native.ferrum_native_geometry_repair as native_geometry_repair
import ferrum_qt.native.ferrum_native_graphics_view
import ferrum_qt.native.ferrum_native_molecule_name as native_molecule_name
import ferrum_qt.native.ferrum_native_paper_properties as native_paper_properties
import ferrum_qt.native.ferrum_native_presentation_deletion as native_presentation_deletion
import ferrum_qt.native.ferrum_native_presentation_stack as native_presentation_stack
import ferrum_qt.native.ferrum_native_rotation as native_rotation
import ferrum_qt.native.ferrum_native_snapshot_export as native_snapshot_export
import ferrum_qt.native.ferrum_native_sdf_insertion as native_sdf_insertion
import ferrum_qt.native.ferrum_native_text_properties as native_text_properties
import ferrum_qt.native.ferrum_native_top_level_transform as native_top_level_transform
import ferrum_qt.native.ferrum_native_tab_view_state
import ferrum_qt.native.ferrum_native_wavy_properties as native_wavy_properties
import ferrum_qt.native.ferrum_native_document_tab_publication as native_publication
import ferrum_qt.native.ferrum_native_document_tab_errors as native_document_tab_errors
import ferrum_qt.native.ferrum_native_drawing_standard as native_drawing_standard


#============================================
FerrumNativeDocumentTabError = native_document_tab_errors.FerrumNativeDocumentTabError


#============================================
FerrumNativeDocumentTabSavePresentationError = (
	native_publication.FerrumNativeDocumentTabSavePresentationError
)


#============================================
class FerrumNativeDocumentTabMutationPresentationError(FerrumNativeDocumentTabError):
	"""A Rust-accepted edit whose authoritative render is pending refresh."""

	#============================================
	def __init__(self, result: object) -> None:
		"""Retain the accepted Rust result without pretending the old scene is current."""
		self.result = result
		super().__init__(
			"Rust accepted the native edit, but its authoritative render could not be "
			"installed; refresh before saving or editing again",
		)


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeChoice:
	"""One durable molecule choice copied from the installed Rust projection."""

	object_id: str
	label: str
	source_order: int


#============================================
class FerrumNativeDocumentTab(
		native_publication.FerrumNativeDocumentTabPublicationMixin,
		native_drawing_standard.FerrumNativeDrawingStandardTabMixin,
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
		native_snapshot_export.FerrumNativeSnapshotExportTabMixin,
		native_geometry_repair.FerrumNativeGeometryRepairTabMixin,
		native_top_level_transform.FerrumNativeTopLevelTransformTabMixin,
		ferrum_qt.native.ferrum_native_tab_view_state.FerrumNativeTabViewStateMixin,
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
			import ferrum_chem
			if type(cdml) is not str or type(title) is not str:
				raise TypeError("native document tab requires CDML and title strings")
			session = ferrum_chem.DocumentSession.load(cdml)
			resource = ferrum_chem.verified_telex_regular()
			view = ferrum_qt.native.ferrum_native_graphics_view.FerrumNativeGraphicsView(self)
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
			raise TypeError("native document tab fixture title must be a string")
		tab = cls.__new__(cls)
		PySide6.QtWidgets.QWidget.__init__(tab)
		try:
			view = ferrum_qt.native.ferrum_native_graphics_view.FerrumNativeGraphicsView(tab)
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
		self._session = session
		self._view = view
		self._controller = controller
		self._snapshot: object | None = None
		self._document_observation: object | None = None
		self._pending_result: object | None = None
		self._pending_snapshot: object | None = None
		self._pending_durable_selection: tuple[tuple[str, str], ...] | None = None
		self._selection_scene: PySide6.QtWidgets.QGraphicsScene | None = None
		self._file_path: pathlib.Path | None = None
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
			raise FerrumNativeDocumentTabError("native tab has no installed observation")
		return self._snapshot
	#============================================
	@property
	def is_dirty(self) -> bool:
		"""Return Rust dirty state, including an accepted render-pending mutation."""
		if self._pending_snapshot is not None:
			return self._pending_snapshot.is_dirty
		return self.current_snapshot.is_dirty
	#============================================
	@property
	def requires_refresh(self) -> bool:
		"""Return whether Rust is ahead of the installed disposable Qt projection."""
		return self._pending_result is not None
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
			raise ValueError("native document origins must be absolute paths")
		if origin.suffix.lower() != ".cdml":
			raise ValueError("native document origins must use the .cdml extension")
		self._file_path = origin
		self._title = origin.name
	#============================================
	def select_atom(self, atom_id: str) -> None:
		"""Select one current durable atom by Rust identifier for native actions."""
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
			raise ValueError("native atom selection requires distinct non-empty identifiers")
		projection = self._require_projection()
		if any(("atom", atom_id) not in projection.durable_items for atom_id in atom_ids):
			raise FerrumNativeDocumentTabError("selected atom is not in the current projection")
		projection.select_durable(tuple(("atom", atom_id) for atom_id in atom_ids))
	#============================================
	def select_bond(self, bond_id: str) -> None:
		"""Select one current durable bond by Rust identifier for native actions."""
		self._require_live()
		if type(bond_id) is not str or not bond_id:
			raise ValueError("native bond selection requires a non-empty identifier")
		projection = self._require_projection()
		if ("bond", bond_id) not in projection.durable_items:
			raise FerrumNativeDocumentTabError(
				"selected bond is not in the current projection",
			)
		projection.select_durable((("bond", bond_id),))
	#============================================
	def durable_atom_at_viewport_point(self, point: PySide6.QtCore.QPoint) -> str | None:
		"""Return the topmost durable Rust atom hit by one viewport point."""
		self._require_live()
		if not isinstance(point, PySide6.QtCore.QPoint):
			raise TypeError("native atom hit testing requires a QPoint")
		projection = self._require_projection()
		for item in self._view.items(point):
			current = item
			while current is not None:
				target = projection.item_targets.get(current)
				if target is not None:
					if target.kind == "atom" and target.identifier is not None:
						return target.identifier
					break
				current = current.parentItem()
		return None

	#============================================
	def durable_atom_scene_position(self, atom_id: str) -> PySide6.QtCore.QPointF:
		"""Return the exact installed Rust point for one durable atom."""
		self._require_live()
		if type(atom_id) is not str or not atom_id:
			raise TypeError("native atom position requires a durable atom identifier")
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
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
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
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
	def current_document_observation(self) -> object:
		"""Return the exact immutable observation installed in the native scene."""
		self._require_mutable()
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
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
		import ferrum_chem
		if type(prepared) is not ferrum_chem.PreparedMoleculeCoordinatesV1:
			raise TypeError("native coordinate update requires exact frozen Ferrum data")
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
		import ferrum_chem
		if type(prepared) is not ferrum_chem.PreparedCleanGeometryV1:
			raise TypeError("native clean geometry requires exact frozen Ferrum data")
		if type(restore) is not tuple or any(
			type(item) is not tuple
			or len(item) != 2
			or type(item[0]) is not str
			or item[0] not in ("atom", "bond")
			or type(item[1]) is not str
			or not item[1]
			for item in restore
		):
			raise TypeError("native clean geometry requires an exact selection tuple")
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
			raise TypeError("native atom insertion requires molecule and element strings")
		if type(x) is not float or type(y) is not float:
			raise TypeError("native atom insertion coordinates must be floats")
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
		import ferrum_chem
		if type(molecule) is not ferrum_chem.MoleculeInsertionV1:
			raise TypeError("native molecule insertion requires exact frozen Ferrum data")
		revision = self.current_snapshot.revision
		prepared = self._session.prepare_insert_molecule_v1(revision, molecule)
		result = self._session.commit_create_molecule(revision, prepared)
		self._install_mutation_result(result)
		return result

	#============================================
	def change_selected_atom_element(self, element: str) -> object:
		"""Submit one selected atom element change against the installed revision."""
		self._require_mutable()
		selected = self._selected_atom_identifier()
		import ferrum_chem
		operation = ferrum_chem.DocumentOperationV1.set_atom_element(selected, element)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result)
		return result

	#============================================
	def selected_atom_projection(self) -> object:
		"""Return one selected frozen Rust atom projection for a native dialog."""
		self._require_mutable()
		selected = self._selected_atom_identifier()
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			for atom in molecule.atoms:
				if atom.source_id == selected:
					return atom
		raise FerrumNativeDocumentTabError("selected atom is absent from the Rust projection")

	#============================================
	def apply_selected_atom_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust atom-properties patch for one selected atom."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("native atom properties require an exact change tuple")
		selected = self._selected_atom_identifier()
		import ferrum_chem
		if any(type(change) is not ferrum_chem.DocumentAtomPropertyChangeV1 for change in changes):
			raise TypeError("native atom properties require exact frozen Ferrum changes")
		operation = ferrum_chem.DocumentOperationV1.set_atom_properties(selected, changes)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("atom", selected),))
		return result

	#============================================
	def set_selected_atom_number(self, number: int, show_number: bool) -> object:
		"""Assign one selected atom number through the closed Rust operation."""
		self._require_mutable()
		if type(number) is not int or number <= 0 or type(show_number) is not bool:
			raise TypeError("native atom number requires a positive int and exact bool")
		molecule_id, atom_id = self._selected_atom_address()
		import ferrum_chem
		operation = ferrum_chem.DocumentOperationV1.set_atom_number(
			molecule_id, atom_id, number, show_number,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("atom", atom_id),))
		return result

	#============================================
	def clear_selected_atom_number(self) -> object:
		"""Clear one selected atom number through the closed Rust operation."""
		self._require_mutable()
		molecule_id, atom_id = self._selected_atom_address()
		import ferrum_chem
		operation = ferrum_chem.DocumentOperationV1.clear_atom_number(
			molecule_id, atom_id,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("atom", atom_id),))
		return result

	#============================================
	def apply_selected_atom_mark(self, action: object, kind: object,
			matching_mark_index: int | None) -> object:
		"""Apply exact frozen mark intent to one selected durable Rust atom."""
		self._require_mutable()
		import ferrum_chem
		if type(action) is not ferrum_chem.AtomMarkActionV1:
			raise TypeError("native atom mark action requires an exact Ferrum value")
		if type(kind) is not ferrum_chem.AtomMarkKindV1:
			raise TypeError("native atom mark kind requires an exact Ferrum value")
		if matching_mark_index is not None and type(matching_mark_index) is not int:
			raise TypeError("native atom mark selector requires an exact int or None")
		molecule_id, atom_id = self._selected_atom_address()
		operation = ferrum_chem.DocumentOperationV1.apply_atom_mark(
			molecule_id, atom_id, action, kind, matching_mark_index,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("atom", atom_id),))
		return result

	#============================================
	def toggle_selected_atom_mark(self, kind: object) -> object:
		"""Add a missing mark or remove the first matching mark in source order."""
		self._require_mutable()
		import ferrum_chem
		if type(kind) is not ferrum_chem.AtomMarkKindV1:
			raise TypeError("native atom mark kind requires an exact Ferrum value")
		atom = self.selected_atom_projection()
		matching = next((mark for mark in atom.marks if mark.kind == kind), None)
		if matching is None:
			action = ferrum_chem.AtomMarkActionV1.add
			ordinal = None
		else:
			action = ferrum_chem.AtomMarkActionV1.remove
			ordinal = matching.same_type_ordinal
		return self.apply_selected_atom_mark(action, kind, ordinal)

	#============================================
	def selected_atom_marks(self) -> tuple[object, ...]:
		"""Return exact current frozen marks for the single selected durable atom."""
		return tuple(self.selected_atom_projection().marks)

	#============================================
	def selected_atom_has_marks(self) -> bool:
		"""Return whether one selected atom currently owns a supported mark."""
		return self.has_one_selected_atom() and bool(self.selected_atom_marks())

	#============================================
	def selected_bond_projection(self) -> object:
		"""Return one selected frozen Rust bond projection for a native dialog."""
		self._require_mutable()
		selected = self._selected_durable_identifiers(1, "bond")[0]
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			for bond in molecule.bonds:
				if bond.source_id == selected:
					return bond
		raise FerrumNativeDocumentTabError("selected bond is absent from the Rust projection")

	#============================================
	def apply_selected_bond_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust bond-properties patch for one selected bond."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("native bond properties require an exact change tuple")
		selected = self._selected_durable_identifiers(1, "bond")[0]
		import ferrum_chem
		if any(type(change) is not ferrum_chem.DocumentBondPropertyChangeV1 for change in changes):
			raise TypeError("native bond properties require exact frozen Ferrum changes")
		operation = ferrum_chem.DocumentOperationV1.set_bond_properties(selected, changes)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("bond", selected),))
		return result

	#============================================
	def has_one_selected_atom(self) -> bool:
		"""Return whether the current disposable selection names exactly one atom."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "atom"
			and selected[0].identifier is not None
		)

	#============================================
	def selected_atom_has_number(self) -> bool:
		"""Return whether the current single selected atom has a valid number fact."""
		if not self.has_one_selected_atom():
			return False
		return self.selected_atom_projection().number is not None

	#============================================
	def has_one_selected_bond(self) -> bool:
		"""Return whether the current disposable selection names exactly one bond."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "bond"
			and selected[0].identifier is not None
		)

	#============================================
	def has_one_selected_plus(self) -> bool:
		"""Return whether the current selection names one durable rendered Plus."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "plus"
			and selected[0].identifier is not None
		)

	#============================================
	def has_one_selected_arrow(self) -> bool:
		"""Return whether the current selection names one durable rendered Arrow."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return (
			len(selected) == 1
			and selected[0].kind == "arrow"
			and selected[0].identifier is not None
		)

	#============================================
	def selected_plus_projection(self) -> object:
		"""Return one selected frozen Rust Plus projection for a native dialog."""
		self._require_mutable()
		selected = self._selected_durable_identifiers(1, "plus")[0]
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
		for root in self._document_observation.projection.presentation_stack.roots:
			if root.kind == "plus" and root.plus.target.id == selected:
				if root.plus.target.source_id is None:
					raise FerrumNativeDocumentTabError(
						"selected Plus has no durable authored source identifier",
					)
				return root.plus
		raise FerrumNativeDocumentTabError("selected Plus is absent from the Rust projection")

	#============================================
	def apply_selected_plus_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust Plus patch while retaining durable selection."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("native Plus properties require an exact change tuple")
		import ferrum_chem
		if any(type(change) is not ferrum_chem.DocumentPlusPropertyChangeV1
				for change in changes):
			raise TypeError("native Plus properties require exact frozen Ferrum changes")
		plus = self.selected_plus_projection()
		operation = ferrum_chem.DocumentOperationV1.set_plus_properties(
			plus.target.source_id, changes,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("plus", plus.target.id),))
		return result

	#============================================
	def selected_arrow_projection(self) -> object:
		"""Return one selected frozen Rust Arrow projection for a native dialog."""
		self._require_mutable()
		selected = self._selected_durable_identifiers(1, "arrow")[0]
		if self._document_observation is None:
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
		for root in self._document_observation.projection.presentation_stack.roots:
			if root.kind == "arrow" and root.arrow.target.id == selected:
				if root.arrow.target.source_id is None:
					raise FerrumNativeDocumentTabError(
						"selected Arrow has no durable authored source identifier",
					)
				return root.arrow
		raise FerrumNativeDocumentTabError("selected Arrow is absent from the Rust projection")

	#============================================
	def apply_selected_arrow_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed Rust Arrow patch while retaining durable selection."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("native Arrow properties require an exact change tuple")
		import ferrum_chem
		if any(type(change) is not ferrum_chem.DocumentArrowPropertyChangeV1
				for change in changes):
			raise TypeError("native Arrow properties require exact frozen Ferrum changes")
		arrow = self.selected_arrow_projection()
		operation = ferrum_chem.DocumentOperationV1.set_arrow_properties(
			arrow.target.source_id, changes,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("arrow", arrow.target.id),))
		return result

	#============================================
	def delete_selected_atom(self) -> object:
		"""Delete one selected durable atom and its incident bonds through Rust."""
		self._require_mutable()
		selected = self._selected_atom_identifier()
		import ferrum_chem
		operation = ferrum_chem.DocumentOperationV1.delete_atom(selected)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result)
		return result

	#============================================
	def delete_selected_bond(self) -> object:
		"""Delete one selected durable typed bond through Rust."""
		self._require_mutable()
		selected = self._selected_durable_identifiers(1, "bond")[0]
		import ferrum_chem
		operation = ferrum_chem.DocumentOperationV1.delete_bond(selected)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result)
		return result

	#============================================
	def set_selected_bond_order(self, order: object) -> object:
		"""Replace one selected bond order through the closed Rust operation."""
		self._require_mutable()
		import ferrum_chem
		if type(order) is not ferrum_chem.DocumentBondOrderV1:
			raise TypeError("native bond order requires an exact Ferrum order value")
		selected = self._selected_durable_identifiers(1, "bond")[0]
		operation = ferrum_chem.DocumentOperationV1.set_bond_order(selected, order)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("bond", selected),))
		return result

	#============================================
	def move_atom_to(self, atom_id: str, x: float, y: float) -> object:
		"""Move one durable atom to an exact finite scene point through Rust."""
		self._require_mutable()
		if type(atom_id) is not str or not atom_id:
			raise TypeError("native atom movement requires a durable atom identifier")
		if type(x) is not float or type(y) is not float:
			raise TypeError("native atom movement coordinates must be floats")
		if ("atom", atom_id) not in self._require_projection().durable_items:
			raise FerrumNativeDocumentTabError("moved atom is not in the current projection")
		import ferrum_chem
		operation = ferrum_chem.DocumentOperationV1.set_atom_position(atom_id, x, y, 0.0)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("atom", atom_id),))
		return result

	#============================================
	def add_single_bond_between_selected_atoms(self) -> object:
		"""Connect exactly two selected durable atoms through one Rust transaction."""
		self._require_mutable()
		selected = self._selected_atom_identifiers(2)
		start, end = self._atom_object_ids(selected)
		import ferrum_chem
		revision = self.current_snapshot.revision
		prepared = self._session.prepare_create_bond_v1(
			revision, start, end, ferrum_chem.DocumentBondOrderV1.single,
		)
		result = self._session.commit_create_bond(revision, prepared)
		self._install_mutation_result(result, (("bond", prepared.identifier),))
		return result

	#============================================
	def add_bonded_atom_at(self, start_atom_id: str, element: str,
			x: float, y: float) -> object:
		"""Create one atom and its bond from an existing durable atom atomically."""
		self._require_mutable()
		if type(start_atom_id) is not str or type(element) is not str:
			raise TypeError("native bonded-atom insertion requires atom and element strings")
		if type(x) is not float or type(y) is not float:
			raise TypeError("native bonded-atom insertion coordinates must be floats")
		(start_object_id,) = self._atom_object_ids((start_atom_id,))
		import ferrum_chem
		revision = self.current_snapshot.revision
		prepared = self._session.prepare_create_bonded_atom_v1(
			revision, start_object_id, element, x, y, 0.0,
			ferrum_chem.DocumentBondOrderV1.single,
		)
		result = self._session.commit_create_bonded_atom(revision, prepared)
		self._install_mutation_result(
			result, (("atom", prepared.atom_identifier),),
		)
		return result

	#============================================
	def create_wavy(self, start_x: float, start_y: float,
			end_x: float, end_y: float) -> object:
		"""Create one Rust-owned Wavy path from two exact scene endpoints."""
		self._require_mutable()
		if any(type(value) is not float for value in (start_x, start_y, end_x, end_y)):
			raise TypeError("native Wavy creation coordinates must be floats")
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
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
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
			observation = self._session.observe_render(self._pending_snapshot.revision)
			installed = self._install_observation(observation)
		except Exception:
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
		self._disposed = True
		self._controller.dispose()
		self._view.setScene(None)
		self._view.deleteLater()
	#============================================
	def _refresh_from_current_revision(self) -> None:
		"""Observe the current Rust revision and install it only if projection succeeds."""
		snapshot = self._session.snapshot()
		observation = self._session.observe_render(snapshot.revision)
		if not self._install_observation(observation):
			raise FerrumNativeDocumentTabError(
				"native tab could not install its render observation",
			)

	#============================================
	def _install_observation(self, observation: object) -> bool:
		"""Install exactly one current render observation through a provenance latch."""
		self._require_live()
		snapshot = observation.document.snapshot
		latch = ferrum_qt.canvas.ferrum_render_projection.RenderProjectionLatch(
			snapshot.revision, snapshot.digest, self._controller.generation,
		)
		installed = self._controller.replace(observation, latch)
		if installed:
			self._snapshot = snapshot
			self._document_observation = observation.document
			self._connect_current_selection_scene()
		return installed

	#============================================
	def _install_mutation_result(self, result: object,
			durable_selection: tuple[tuple[str, str], ...] | None = None) -> None:
		"""Install a Rust-accepted result or retain exact recovery ownership."""
		authoritative = result.observation
		self._pending_result = result
		self._pending_snapshot = authoritative.snapshot
		self._pending_durable_selection = durable_selection
		try:
			observation = self._session.observe_render(authoritative.snapshot.revision)
			installed = self._install_observation(observation)
		except Exception as exc:
			raise FerrumNativeDocumentTabMutationPresentationError(result) from exc
		if not installed:
			raise FerrumNativeDocumentTabMutationPresentationError(result)
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
			raise FerrumNativeDocumentTabError("native tab has no installed document projection")
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
			raise FerrumNativeDocumentTabError("native tab has no installed projection")
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
			raise FerrumNativeDocumentTabError("native document tab has been disposed")
	#============================================
	def _retire_partial_resources(self) -> None:
		"""Dispose partial projection resources after construction failure."""
		controller = getattr(self, "_controller", None)
		if controller is not None:
			controller.dispose()
		view = getattr(self, "_view", None)
		if view is not None:
			view.setScene(None)
			view.deleteLater()
