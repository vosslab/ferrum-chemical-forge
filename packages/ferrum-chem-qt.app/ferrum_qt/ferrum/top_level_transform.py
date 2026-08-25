"""actions for Rust-owned complete-root transforms."""

# Standard Library
import functools

# PIP3 modules
import PySide6.QtGui

# local repo modules


_ALIGNMENTS = (
	("top", "Align Top"),
	("bottom", "Align Bottom"),
	("left", "Align Left"),
	("right", "Align Right"),
	("center_x", "Align Centers Horizontally"),
	("center_y", "Align Centers Vertically"),
)


#============================================
class FerrumNativeTopLevelTransformTabMixin:
	"""Map complete disposable selections to closed durable Rust root selectors."""

	#============================================
	def selected_top_level_transform_targets(
			self) -> tuple[tuple[object, ...], tuple[tuple[str, str], ...]]:
		"""Return complete durable roots plus the selection to restore."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		selected = self._require_projection().selected_durable_targets()
		if not selected:
			raise _tab_error("select complete molecules or durable presentation roots first")
		if any(target.kind == "bond" for target in selected):
			raise _tab_error("bonds are not independent top-level transform roots")
		selected_atoms = {
			target.durable_object_id for target in selected if target.kind == "atom"
		}
		if any(atom_id is None for atom_id in selected_atoms):
			raise _tab_error("selected atom lacks a durable document identity")
		selectors = []
		consumed_atoms = set()
		kinds = engine.DocumentTopLevelRootKindV1
		if self._document_observation is None:
			raise _tab_error("Ferrum tab has no installed document projection")
		for molecule in self._document_observation.projection.molecules:
			atom_ids = tuple(atom.id for atom in molecule.atoms)
			if not selected_atoms.intersection(atom_ids):
				continue
			if (
				molecule.id is None
				or not atom_ids
				or any(identifier is None for identifier in atom_ids)
				or not set(atom_ids).issubset(selected_atoms)
			):
				raise _tab_error(
					"select every atom of each molecule before transforming it",
				)
			selectors.append(
				(molecule.id, kinds.molecule),
			)
			consumed_atoms.update(atom_ids)
		if consumed_atoms != selected_atoms:
			raise _tab_error("selected atom is not part of a complete durable molecule")
		kind_values = {
			"arrow": kinds.arrow,
			"plus": kinds.plus,
			"text": kinds.text,
			"polyline": kinds.polyline,
			"rectangle": kinds.rectangle,
			"square": kinds.square,
			"oval": kinds.oval,
			"circle": kinds.circle,
			"polygon": kinds.polygon,
		}
		for target in selected:
			if target.kind == "atom":
				continue
			kind = kind_values.get(target.kind)
			object_id = target.durable_object_id
			if kind is None or type(object_id) is not str or not object_id:
				raise _tab_error(
					"selection contains an unsupported top-level transform target",
				)
			selectors.append((object_id, kind))
		restore = tuple(target.durable_selection_key() for target in selected)
		return tuple(selectors), restore

	#============================================
	def can_align_top_level_selection(self) -> bool:
		"""Return whether current selection forms at least two complete roots."""
		if self._disposed or self.requires_refresh:
			return False
		try:
			targets, _restore = self.selected_top_level_transform_targets()
		except (RuntimeError, TypeError, ValueError):
			return False
		return len(targets) >= 2

	#============================================
	def can_transform_top_level_selection(self) -> bool:
		"""Return whether current selection forms at least one complete root."""
		if self._disposed or self.requires_refresh:
			return False
		try:
			targets, _restore = self.selected_top_level_transform_targets()
		except (RuntimeError, TypeError, ValueError):
			return False
		return bool(targets)

	#============================================
	def align_selected_top_level_roots(self, alignment: object) -> object:
		"""Align complete selected roots through one closed Rust operation."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(alignment) is not engine.DocumentTopLevelAlignmentV1:
			raise TypeError("Ferrum root alignment requires an exact Ferrum value")
		targets, restore = self.selected_top_level_transform_targets()
		snapshot = self.current_snapshot
		result = self._live_document_session_v1.align_live_document_roots_v1(
			snapshot.revision, snapshot.digest, targets, alignment,
		)
		self._install_mutation_result(result, restore)
		return result

	#============================================
	def scale_top_level_roots_at_revision(
			self, expected_revision: int, targets: tuple[object, ...],
			restore: tuple[tuple[str, str], ...], scale_x: float,
			scale_y: float) -> object:
		"""Scale captured roots while retaining the pre-dialog revision guard."""
		self._require_mutable()
		if type(expected_revision) is not int:
			raise TypeError("Ferrum root scale requires an exact revision")
		snapshot = self.current_snapshot
		if snapshot.revision != expected_revision:
			raise _tab_error("active Ferrum document changed while scale was open")
		result = self._live_document_session_v1.scale_live_document_roots_v1(
			expected_revision, snapshot.digest, targets, scale_x, scale_y,
		)
		self._install_mutation_result(result, restore)
		return result

	#============================================
	def mirror_selected_top_level_roots(self, orientation: object) -> object:
		"""Mirror complete selected roots through one closed Rust operation."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(orientation) is not engine.DocumentTopLevelMirrorV1:
			raise TypeError("Ferrum root mirror requires an exact Ferrum value")
		targets, restore = self.selected_top_level_transform_targets()
		snapshot = self.current_snapshot
		result = self._live_document_session_v1.mirror_live_document_roots_v1(
			snapshot.revision, snapshot.digest, targets, orientation,
		)
		self._install_mutation_result(result, restore)
		return result


#============================================
class FerrumNativeTopLevelTransformMixin:
	"""Install complete-root transforms without persistent Qt geometry."""

	#============================================
	def _build_top_level_transform_actions(self, edit_menu: object) -> None:
		"""Add closed transform actions for complete durable root selection."""
		menu = edit_menu.addMenu(self.tr("Transform Complete Roots"))
		menu.setToolTip(self.tr(
			"Select presentation roots or every atom of each molecule to transform",
		))
		self._top_level_scale_action = PySide6.QtGui.QAction(
			self.tr("Scale..."), self,
		)
		self._top_level_scale_action.triggered.connect(self._on_scale_top_level_roots)
		menu.addAction(self._top_level_scale_action)
		self._top_level_mirror_actions = {}
		for name, label in (
			("vertical", "Mirror Across Vertical Axis"),
			("horizontal", "Mirror Across Horizontal Axis"),
		):
			action = PySide6.QtGui.QAction(self.tr(label), self)
			action.triggered.connect(
				functools.partial(self._on_mirror_top_level_roots, name),
			)
			menu.addAction(action)
			self._top_level_mirror_actions[name] = action
		menu.addSeparator()
		self._top_level_alignment_actions = {}
		for name, label in _ALIGNMENTS:
			action = PySide6.QtGui.QAction(self.tr(label), self)
			action.triggered.connect(functools.partial(self._on_align_top_level_roots, name))
			menu.addAction(action)
			self._top_level_alignment_actions[name] = action

	#============================================
	def _on_align_top_level_roots(self, name: str, _checked: bool = False) -> None:
		"""Submit one exact Rust alignment for the current complete selection."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			alignment = getattr(engine.DocumentTopLevelAlignmentV1, name)
			tab.align_selected_top_level_roots(alignment)
		except Exception as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
		finally:
			self._refresh_actions()

	#============================================
	def _on_scale_top_level_roots(self, _checked: bool = False) -> None:
		"""Scale the exact pre-dialog root selection at its captured revision."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			expected_revision = tab.current_snapshot.revision
			targets, restore = tab.selected_top_level_transform_targets()
			from ferrum_qt.dialogs.scale_dialog import ScaleDialog
			factors = ScaleDialog.get_scale_factors(self)
			if factors is None:
				return
			if self._active_native_tab() is not tab:
				raise _tab_error("active Ferrum document changed while scale was open")
			tab.scale_top_level_roots_at_revision(
				expected_revision, targets, restore, factors[0], factors[1],
			)
		except Exception as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
		finally:
			self._refresh_actions()

	#============================================
	def _on_mirror_top_level_roots(
			self, name: str, _checked: bool = False) -> None:
		"""Submit one exact Rust mirror for the current complete selection."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			orientation = getattr(engine.DocumentTopLevelMirrorV1, name)
			tab.mirror_selected_top_level_roots(orientation)
		except Exception as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
		finally:
			self._refresh_actions()

	#============================================
	def _refresh_top_level_transform_actions(
			self, tab: object, active: bool, pending: bool, busy: bool) -> None:
		"""Enable actions only when their complete-root cardinality is met."""
		transform_available = (
			active and not pending and not busy
			and tab.can_transform_top_level_selection()
		)
		align_available = (
			active and not pending and not busy and tab.can_align_top_level_selection()
		)
		self._top_level_scale_action.setEnabled(transform_available)
		for action in self._top_level_mirror_actions.values():
			action.setEnabled(transform_available)
		for action in self._top_level_alignment_actions.values():
			action.setEnabled(align_available)


#============================================
def _tab_error(message: str) -> RuntimeError:
	"""Create the Ferrum tab's public error without introducing an import cycle."""
	from ferrum_qt.ferrum.document_tab import FerrumNativeDocumentTabError
	return FerrumNativeDocumentTabError(message)
