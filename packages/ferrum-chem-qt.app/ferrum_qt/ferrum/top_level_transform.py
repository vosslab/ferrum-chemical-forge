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
			self) -> tuple[tuple[object, ...], tuple[str, ...]]:
		"""Return complete durable roots plus the selection to restore."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		from ferrum_qt.canvas.ferrum_render_target import RenderTargetKey
		projection = self._require_projection()
		selected = projection.selected_targets()
		if type(selected) is not tuple or not selected:
			raise _tab_error("select complete molecules or durable presentation roots first")
		selected_ids = set()
		for target in selected:
			if type(target) is not RenderTargetKey or target.kind != "document_object":
				raise _tab_error("selected canvas target is not a current document object")
			if type(target.document_object_id) is not str or not target.document_object_id:
				raise _tab_error("selected canvas target lacks a durable document identity")
			if target.document_object_id in selected_ids:
				raise _tab_error("selected canvas targets are not unique")
			selected_ids.add(target.document_object_id)
		observation = self._document_observation
		if type(observation) is not engine.SessionDocumentObservationV1:
			raise _tab_error("Ferrum tab has no installed document projection")
		document = observation.projection
		if type(document) is not engine.DocumentProjectionV1:
			raise _tab_error("Ferrum tab has no exact Rust document projection")
		direct_roots = document.direct_roots
		if type(direct_roots) is not tuple:
			raise _tab_error("Rust document direct roots are not an exact DTO tuple")
		kinds = engine.DocumentTopLevelRootKindV1
		kind_values = {
			"molecule": kinds.molecule,
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
		roots_by_id = {}
		paint_orders = set()
		for root in direct_roots:
			if type(root) is not engine.DocumentDirectRootV1:
				raise _tab_error("Rust document direct root has the wrong DTO type")
			object_id = root.document_object_id
			if (
				type(object_id) is not str
				or not object_id
				or type(root.kind) is not str
				or type(root.paint_order) is not int
				or root.paint_order < 0
				or root.paint_order >= 2**32
				or object_id in roots_by_id
				or root.paint_order in paint_orders
			):
				raise _tab_error("Rust document direct roots are invalid")
			roots_by_id[object_id] = (root.kind, root.paint_order)
			paint_orders.add(root.paint_order)
		selected_root_ids = selected_ids.intersection(roots_by_id)
		selected_member_ids = selected_ids.difference(selected_root_ids)
		selected_member_order = tuple(
			target.document_object_id for target in selected
			if target.document_object_id in selected_member_ids
		)
		resolved = []
		for object_id in selected_root_ids:
			root_kind, paint_order = roots_by_id[object_id]
			selector_kind = kind_values.get(root_kind)
			if selector_kind is None:
				raise _tab_error(
					"selection contains an unsupported top-level transform target",
				)
			resolved.append((paint_order, object_id, selector_kind))
		if selected_member_ids:
			selected_members = self.structure_targets_for_ids(selected_member_order)
			if len(selected_members) != len(selected_member_ids):
				raise _tab_error("selected canvas target has no current Rust structural fact")
			selected_atoms = set()
			molecule_ids = set()
			for target in selected_members:
				if type(target) is not engine.StructureInteractionTargetV1:
					raise _tab_error("Rust structure selection returned an invalid target")
				if target.kind == engine.StructureTargetKindV1.bond:
					raise _tab_error("bonds are not independent top-level transform roots")
				if target.kind != engine.StructureTargetKindV1.atom:
					raise _tab_error("selection contains an unsupported top-level transform target")
				if (
					type(target.object_id) is not str
					or not target.object_id
					or type(target.molecule_object_id) is not str
					or not target.molecule_object_id
					or target.object_id not in selected_member_ids
					or target.object_id in selected_atoms
				):
					raise _tab_error("Rust structure selection returned an invalid durable address")
				selected_atoms.add(target.object_id)
				molecule_ids.add(target.molecule_object_id)
			if selected_atoms != selected_member_ids:
				raise _tab_error("selected canvas target is absent from Rust structure selection")
			molecules_by_id = {}
			for molecule in document.molecules:
				if type(molecule) is not engine.MoleculeProjectionV1:
					raise _tab_error("Rust molecule projection has the wrong DTO type")
				molecule_id = molecule.document_object_id
				atom_ids = tuple(atom.document_object_id for atom in molecule.atoms)
				if (
					type(molecule_id) is not str
					or not molecule_id
					or not atom_ids
					or any(type(atom_id) is not str or not atom_id for atom_id in atom_ids)
					or len(set(atom_ids)) != len(atom_ids)
					or molecule_id in molecules_by_id
				):
					raise _tab_error("Rust molecule projection has an invalid durable identity")
				molecules_by_id[molecule_id] = frozenset(atom_ids)
			for molecule_id in molecule_ids:
				try:
					atom_ids = molecules_by_id[molecule_id]
					root_kind, paint_order = roots_by_id[molecule_id]
				except KeyError as exc:
					raise _tab_error(
						"selected atom is not part of a complete durable molecule",
					) from exc
				if root_kind != "molecule" or not atom_ids.issubset(selected_atoms):
					raise _tab_error(
						"select every atom of each molecule before transforming it",
					)
				resolved.append((paint_order, molecule_id, kinds.molecule))
		if len({object_id for _order, object_id, _kind in resolved}) != len(resolved):
			raise _tab_error("selection contains duplicate top-level transform roots")
		selectors = tuple(
			(object_id, selector_kind)
			for _paint_order, object_id, selector_kind in sorted(resolved)
		)
		restore = tuple(target.document_object_id for target in selected)
		return selectors, restore

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
			restore: tuple[str, ...], scale_x: float,
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
