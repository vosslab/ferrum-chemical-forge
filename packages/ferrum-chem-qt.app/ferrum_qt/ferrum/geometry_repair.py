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
	"""Map disposable atom/bond selection to durable Rust molecule repairs."""

	#============================================
	def selected_geometry_repair_molecules(
			self) -> tuple[tuple[str, ...], tuple[tuple[str, str], ...]]:
		"""Return ordered durable molecule IDs and exact selection to restore."""
		self._require_mutable()
		projection = self._require_projection()
		selected = projection.selected_targets()
		if any(not target.is_durable for target in selected):
			raise _tab_error("geometry repair requires durable selected objects")
		if any(target.kind not in ("atom", "bond") for target in selected):
			raise _tab_error("geometry repair selection may contain only atoms or bonds")
		if self._document_observation is None:
			raise _tab_error("Ferrum tab has no installed document projection")
		molecules = self._document_observation.projection.molecules
		if not molecules:
			raise _tab_error("document has no molecule to repair")
		if not selected:
			if any(molecule.source_id is None for molecule in molecules):
				raise _tab_error(
					"repairing every molecule requires every molecule to have a durable ID",
				)
			return tuple(molecule.source_id for molecule in molecules), ()
		requested = {(target.kind, target.identifier) for target in selected}
		molecule_ids = []
		matched = set()
		for molecule in molecules:
			members = {
				*( ("atom", atom.source_id) for atom in molecule.atoms ),
				*( ("bond", bond.source_id) for bond in molecule.bonds ),
			}
			selected_members = requested.intersection(members)
			if not selected_members:
				continue
			if molecule.source_id is None:
				raise _tab_error("selected molecule has no durable ID")
			molecule_ids.append(molecule.source_id)
			matched.update(selected_members)
		if matched != requested:
			raise _tab_error("selected object is not part of a durable projected molecule")
		restore = tuple((target.kind, target.identifier) for target in selected)
		return tuple(molecule_ids), restore

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
			self) -> tuple[tuple[str, ...], tuple[tuple[str, str], ...]]:
		"""Return durable document-object selectors for Ferrum preparation."""
		source_ids, restore = self.selected_geometry_repair_molecules()
		molecules = self._document_observation.projection.molecules
		object_ids = []
		for source_id in source_ids:
			matches = tuple(
				molecule for molecule in molecules if molecule.source_id == source_id
			)
			if len(matches) != 1 or matches[0].id is None:
				raise _tab_error("clean geometry requires durable molecule object IDs")
			object_ids.append(matches[0].id)
		return tuple(object_ids), restore

	#============================================
	def repair_geometry_at_revision(
			self, expected_revision: int, molecule_ids: tuple[str, ...],
			restore: tuple[tuple[str, str], ...], kind: object,
			target_spacing_points: float) -> object:
		"""Submit one captured repair through the exact frozen Rust boundary."""
		self._require_mutable()
		if type(expected_revision) is not int:
			raise TypeError("Ferrum geometry repair requires an exact revision")
		if type(molecule_ids) is not tuple or any(type(value) is not str for value in molecule_ids):
			raise TypeError("Ferrum geometry repair requires an exact tuple of molecule IDs")
		if type(restore) is not tuple:
			raise TypeError("Ferrum geometry repair requires an exact selection tuple")
		if type(target_spacing_points) not in (int, float):
			raise TypeError("Ferrum geometry repair spacing must be an exact number")
		import ferrum_qt.ferrum.engine as engine
		if type(kind) is not engine.DocumentGeometryRepairKindV1:
			raise TypeError("Ferrum geometry repair requires an exact Ferrum kind")
		operation = engine.DocumentOperationV1.repair_geometry(
			molecule_ids, kind, target_spacing_points,
		)
		result = self._session.submit(expected_revision, operation)
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
