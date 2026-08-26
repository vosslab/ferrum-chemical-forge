"""actions for Rust-owned molecule geometry repair."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.coordinate_generation as native_coordinates


_UNUSED_STRAIGHTEN_SPACING_POINTS = 1.0


#============================================
class FerrumNativeGeometryRepairTabMixin:
	"""Submit one Rust-authenticated atom/bond selection for molecule repairs."""

	#============================================
	def selected_geometry_repair_molecules(
			self) -> tuple[tuple[str, ...], tuple[str, ...]]:
		"""Return one selected molecule and its exact Rust-authenticated members."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		targets = self.selected_structure_targets()
		if type(targets) is not tuple or not targets:
			raise _tab_error("select one or more atoms or bonds to repair their molecule")
		allowed_kinds = frozenset((
			engine.StructureTargetKindV1.atom,
			engine.StructureTargetKindV1.bond,
		))
		molecule_id: str | None = None
		restore: list[str] = []
		for target in targets:
			if type(target) is not engine.StructureInteractionTargetV1:
				raise _tab_error("geometry repair requires exact Rust structure targets")
			if target.kind not in allowed_kinds:
				raise _tab_error("geometry repair selection may contain only atoms or bonds")
			if type(target.molecule_object_id) is not str or not target.molecule_object_id:
				raise _tab_error("selected object has no durable molecule identity")
			if type(target.object_id) is not str or not target.object_id:
				raise _tab_error("selected object has no durable object identity")
			if molecule_id is None:
				molecule_id = target.molecule_object_id
			elif target.molecule_object_id != molecule_id:
				raise _tab_error("geometry repair selection must belong to exactly one molecule")
			restore.append(target.object_id)
		if molecule_id is None:
			raise _tab_error("geometry repair requires a current atom or bond selection")
		return (molecule_id,), tuple(restore)

	#============================================
	def can_repair_geometry_selection(self) -> bool:
		"""Return whether current selection resolves to durable molecules."""
		if self._disposed or self.requires_refresh:
			return False
		try:
			molecule_ids, _restore = self.selected_geometry_repair_molecules()
		except (RuntimeError, TypeError, ValueError):
			return False
		return bool(molecule_ids)

	#============================================
	def selected_clean_geometry_molecules(
			self) -> tuple[tuple[str, ...], tuple[str, ...]]:
		"""Return durable document-object selectors for Ferrum preparation."""
		return self.selected_geometry_repair_molecules()

	#============================================
	def repair_geometry_at_revision(
			self, expected_revision: int, molecule_ids: tuple[str, ...],
			restore: tuple[str, ...], kind: object,
			target_spacing_points: float) -> object:
		"""Submit one captured repair through the exact frozen Rust boundary."""
		self._require_mutable()
		if type(expected_revision) is not int:
			raise TypeError("Ferrum geometry repair requires an exact revision")
		if (
			type(molecule_ids) is not tuple
			or len(molecule_ids) != 1
			or type(molecule_ids[0]) is not str
			or not molecule_ids[0]
		):
			raise TypeError("Ferrum geometry repair requires one durable molecule ID")
		if type(restore) is not tuple:
			raise TypeError("Ferrum geometry repair requires an exact selection tuple")
		if type(target_spacing_points) not in (int, float):
			raise TypeError("Ferrum geometry repair spacing must be an exact number")
		import ferrum_qt.ferrum.engine as engine
		if type(kind) is not engine.DocumentGeometryRepairKindV1:
			raise TypeError("Ferrum geometry repair requires an exact Ferrum kind")
		snapshot = self.current_snapshot
		if snapshot.revision != expected_revision:
			raise _tab_error("Ferrum document changed while geometry repair was open")
		result = self._live_document_session_v1.repair_live_document_geometry_v1(
			expected_revision, snapshot.digest, molecule_ids, kind, target_spacing_points,
		)
		self._install_mutation_result(result, restore)
		return result


#============================================
class FerrumNativeGeometryRepairWindowMixin:
	"""Install Ferrum repair actions that never inspect or mutate CDML."""

	#============================================
	def _build_geometry_repair_actions(self) -> None:
		"""Add only geometry kinds implemented by the Rust document boundary."""
		menu = self.menuBar().addMenu(self.tr("Repair"))
		self._clean_geometry_action = PySide6.QtGui.QAction(
			self.tr("Clean Geometry..."), self,
		)
		self._clean_geometry_action.setToolTip(self.tr(
			"Regenerate selected molecules through Ferrum chemistry at an explicit spacing",
		))
		self._clean_geometry_action.triggered.connect(self._on_clean_geometry)
		menu.addAction(self._clean_geometry_action)
		self._normalize_bond_lengths_action = PySide6.QtGui.QAction(
			self.tr("Normalize Bond Lengths..."), self,
		)
		self._normalize_bond_lengths_action.setToolTip(self.tr(
			"Enter an explicit target length; Rust preserves eligible bond directions",
		))
		self._normalize_bond_lengths_action.triggered.connect(
			self._on_normalize_bond_lengths,
		)
		menu.addAction(self._normalize_bond_lengths_action)
		self._normalize_bond_angles_action = PySide6.QtGui.QAction(
			self.tr("Normalize Bond Angles..."), self,
		)
		self._normalize_bond_angles_action.setToolTip(self.tr(
			"Snap movable non-ring bonds to distinct 60-degree directions in Rust",
		))
		self._normalize_bond_angles_action.triggered.connect(
			self._on_normalize_bond_angles,
		)
		menu.addAction(self._normalize_bond_angles_action)
		self._normalize_rings_action = PySide6.QtGui.QAction(
			self.tr("Normalize Ring Geometry..."), self,
		)
		self._normalize_rings_action.setToolTip(self.tr(
			"Enter a target side length for one simple ring per molecule",
		))
		self._normalize_rings_action.triggered.connect(self._on_normalize_rings)
		menu.addAction(self._normalize_rings_action)
		self._snap_to_hex_grid_action = PySide6.QtGui.QAction(
			self.tr("Snap Molecules to Hex Grid..."), self,
		)
		self._snap_to_hex_grid_action.setToolTip(self.tr(
			"Enter an explicit scene-point spacing; Rust owns the complete repair",
		))
		self._snap_to_hex_grid_action.triggered.connect(self._on_snap_to_hex_grid)
		menu.addAction(self._snap_to_hex_grid_action)
		self._straighten_bonds_action = PySide6.QtGui.QAction(
			self.tr("Straighten Terminal Bonds"), self,
		)
		self._straighten_bonds_action.setToolTip(self.tr(
			"Snap only degree-one bond endpoints through the Rust document session",
		))
		self._straighten_bonds_action.triggered.connect(self._on_straighten_bonds)
		menu.addAction(self._straighten_bonds_action)

	#============================================
	def _on_clean_geometry(self, _checked: bool = False) -> None:
		"""Prepare an atomic Ferrum regeneration batch without blocking the UI."""
		tab = self._active_native_tab()
		if (
			tab is None
			or self._coordinate_generation_intent is not None
			or self._molecule_import_busy()
		):
			return
		try:
			snapshot = tab.current_snapshot
			molecule_ids, restore = tab.selected_clean_geometry_molecules()
			text, accepted = PySide6.QtWidgets.QInputDialog.getText(
				self, self.tr("Clean Geometry"),
				self.tr("Target bond length in scene points (positive number):"),
			)
			if not accepted:
				return
			spacing = _positive_finite_spacing(text)
			if self._active_native_tab() is not tab:
				raise _tab_error("active Ferrum document changed while spacing was open")
			worker = native_coordinates.FerrumNativeCleanGeometryPreparationWorker(
				tab.current_document_observation(), molecule_ids, spacing, restore,
			)
			self._coordinate_generation_intent = (
				native_coordinates.FerrumNativeCoordinateGenerationIntent(
					tab, snapshot.revision, snapshot.digest, worker,
				)
			)
			connection = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
			worker.prepared.connect(self._on_coordinates_prepared, connection)
			worker.failed.connect(self._on_coordinates_failed, connection)
			worker.finished.connect(self._on_coordinate_worker_finished, connection)
			self.statusBar().showMessage(
				self.tr("Cleaning molecule geometry through Ferrum chemistry..."), 0,
			)
			self._refresh_actions()
			worker.start()
		except Exception as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
			self._refresh_actions()

	#============================================
	def _on_snap_to_hex_grid(self, _checked: bool = False) -> None:
		"""Request explicit hex spacing before one captured Rust repair."""
		self._repair_with_explicit_spacing(
			"snap_to_hex_grid", "Snap Molecules to Hex Grid",
			"Hex-grid spacing in scene points (positive number):",
			"Hex-Grid Repair Failed",
		)

	#============================================
	def _on_normalize_bond_lengths(self, _checked: bool = False) -> None:
		"""Request an explicit target length before one captured Rust repair."""
		self._repair_with_explicit_spacing(
			"normalize_bond_lengths", "Normalize Bond Lengths",
			"Target bond length in scene points (positive number):",
			"Bond-Length Repair Failed",
		)

	#============================================
	def _on_normalize_bond_angles(self, _checked: bool = False) -> None:
		"""Request degenerate-vector spacing before one captured angle repair."""
		self._repair_with_explicit_spacing(
			"normalize_bond_angles", "Normalize Bond Angles",
			"Fallback length for coincident atoms in scene points (positive number):",
			"Bond-Angle Repair Failed",
		)

	#============================================
	def _on_normalize_rings(self, _checked: bool = False) -> None:
		"""Request an explicit ring side length before one captured Rust repair."""
		self._repair_with_explicit_spacing(
			"normalize_rings", "Normalize Ring Geometry",
			"Target ring side length in scene points (positive number):",
			"Ring Repair Failed",
		)

	#============================================
	def _repair_with_explicit_spacing(
			self, kind_name: str, title: str, prompt: str, warning_title: str) -> None:
		"""Capture durable targets and revision before requesting one spacing."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			revision = tab.current_snapshot.revision
			molecule_ids, restore = tab.selected_geometry_repair_molecules()
			text, accepted = PySide6.QtWidgets.QInputDialog.getText(
				self, self.tr(title), self.tr(prompt),
			)
			if not accepted:
				return
			spacing = _positive_finite_spacing(text)
			if self._active_native_tab() is not tab:
				raise _tab_error("active Ferrum document changed while spacing was open")
			import ferrum_qt.ferrum.engine as engine
			tab.repair_geometry_at_revision(
				revision, molecule_ids, restore,
				getattr(engine.DocumentGeometryRepairKindV1, kind_name), spacing,
			)
		except Exception as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
		finally:
			self._refresh_actions()

	#============================================
	def _on_straighten_bonds(self, _checked: bool = False) -> None:
		"""Straighten terminal endpoints; common-envelope spacing is unused."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			revision = tab.current_snapshot.revision
			molecule_ids, restore = tab.selected_geometry_repair_molecules()
			import ferrum_qt.ferrum.engine as engine
			tab.repair_geometry_at_revision(
				revision, molecule_ids, restore,
				engine.DocumentGeometryRepairKindV1.straighten_bonds,
				_UNUSED_STRAIGHTEN_SPACING_POINTS,
			)
		except Exception as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
		finally:
			self._refresh_actions()

	#============================================
	def _refresh_geometry_repair_actions(
			self, tab: object, active: bool, pending: bool, busy: bool) -> None:
		"""Enable both repairs only for a current resolvable molecule selection."""
		available = (
			active and not pending and not busy and tab.can_repair_geometry_selection()
		)
		self._snap_to_hex_grid_action.setEnabled(available)
		self._straighten_bonds_action.setEnabled(available)
		self._normalize_bond_lengths_action.setEnabled(available)
		self._normalize_bond_angles_action.setEnabled(available)
		self._normalize_rings_action.setEnabled(available)
		self._clean_geometry_action.setEnabled(available)


#============================================
def _positive_finite_spacing(text: str) -> float:
	"""Parse user-authored spacing without imposing an arbitrary UI ceiling."""
	if type(text) is not str:
		raise TypeError("geometry repair spacing must be text entered by the user")
	try:
		value = float(text.strip())
	except ValueError as error:
		raise ValueError("geometry repair spacing must be a number") from error
	if not math.isfinite(value) or value <= 0.0:
		raise ValueError("geometry repair spacing must be finite and greater than zero")
	return value


#============================================
def _tab_error(message: str) -> RuntimeError:
	"""Create the Ferrum tab's public error without an import cycle."""
	from ferrum_qt.ferrum.document_tab import FerrumNativeDocumentTabError
	return FerrumNativeDocumentTabError(message)
