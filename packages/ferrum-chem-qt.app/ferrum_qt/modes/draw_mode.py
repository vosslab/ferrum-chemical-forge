"""Qt coordinator for Draw mode gestures and backend-owned structural edits."""

# Standard Library
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.bond_presentation
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.config.geometry_units
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.draw_geometry
import ferrum_qt.modes.draw_gesture
import ferrum_qt.modes.draw_legacy_editing
from ferrum_qt.canvas.items import render_ops_painter
from ferrum_qt.models.atom_model import AtomModel
from ferrum_qt.models.molecule_model import MoleculeModel


DRAG_THRESHOLD = 5.0
BOND_ORDER_BY_SUBMODE = {"single": 1, "double": 2, "triple": 3}
BOND_TYPE_BY_SUBMODE = ferrum_qt.bond_presentation.DRAW_BOND_TYPE_BY_SUBMODE


#============================================
class DrawMode(ferrum_qt.modes.draw_legacy_editing.DrawLegacyEditingMixin,
		ferrum_qt.modes.base_mode.BaseMode):
	"""Coordinate Draw input without retaining persistent document ownership."""

	#============================================
	def __init__(self, view: object, parent: PySide6.QtCore.QObject | None = None) -> None:
		"""Initialize configured drawing preferences and one transient gesture."""
		super().__init__(view, parent)
		self._name = "Draw"
		self._cursor = PySide6.QtCore.Qt.CursorShape.CrossCursor
		self._current_element = "C"
		self._current_bond_order = 1
		self._current_bond_type = "n"
		self._angle_resolution = ferrum_qt.modes.draw_geometry.ANGLE_RESOLUTION
		self._fixed_length = True
		self._simple_double = True
		self._persistent_operation = None
		self._gesture = ferrum_qt.modes.draw_gesture.DrawGestureState()
		self._sign = 1
		self._last_used_atom_id = None

	#============================================
	def set_persistent_operation(self, operation: object | None) -> None:
		"""Install the session-owned immutable persistent-operation callback."""
		if operation is not None and not callable(operation):
			raise TypeError("Draw persistent operation must be callable")
		self._persistent_operation = operation

	#============================================
	@property
	def status_hint(self) -> str:
		"""Return the brief Draw interaction hint used by the status bar."""
		return "Click atom to extend bond | Click empty space to start new | Drag to set angle"

	#============================================
	@property
	def current_element(self) -> str:
		"""Return the element used for newly created atoms."""
		return self._current_element

	#============================================
	@current_element.setter
	def current_element(self, symbol: str) -> None:
		"""Select the element used for newly created atoms."""
		self._current_element = str(symbol)

	#============================================
	@property
	def current_bond_order(self) -> int:
		"""Return the selected bond order."""
		return self._current_bond_order

	#============================================
	@current_bond_order.setter
	def current_bond_order(self, order: int) -> None:
		"""Select the bond order."""
		self._current_bond_order = int(order)

	#============================================
	@property
	def current_bond_type(self) -> str:
		"""Return the selected CDML bond type."""
		return self._current_bond_type

	#============================================
	@current_bond_type.setter
	def current_bond_type(self, bond_type: str) -> None:
		"""Select the CDML bond type."""
		self._current_bond_type = str(bond_type)

	#============================================
	@property
	def angle_resolution(self) -> int:
		"""Return the fixed-length drag angle increment."""
		return self._angle_resolution

	#============================================
	@property
	def fixed_length(self) -> bool:
		"""Return whether a drag keeps document bond length."""
		return self._fixed_length

	#============================================
	@property
	def simple_double(self) -> bool:
		"""Return whether a new double bond uses the simple display style."""
		return self._simple_double

	#============================================
	def on_submode_switch(self, submode_index: int, name: str) -> None:
		"""Apply the selected YAML Draw submode to persistent edit settings."""
		if submode_index == 0:
			self._angle_resolution = int(name)
		elif submode_index == 1:
			self.current_bond_order = BOND_ORDER_BY_SUBMODE[name]
		elif submode_index == 2:
			self.current_bond_type = BOND_TYPE_BY_SUBMODE[name]
		elif submode_index == 3:
			self._fixed_length = name == "fixed"
		elif submode_index == 4:
			self._simple_double = name == "simpledouble"

	#============================================
	def deactivate(self) -> None:
		"""Cancel the active preview before BaseMode releases this mode."""
		self._reset_gesture()
		super().deactivate()

	#============================================
	def _get_bond_length(self) -> float:
		"""Return current scene grid spacing or the documented default."""
		scene = self._env.scene
		if scene is not None and hasattr(scene, "grid_spacing_pt"):
			return scene.grid_spacing_pt
		return ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT

	#============================================
	@staticmethod
	def _grid_snap_enabled(scene: object) -> bool:
		"""Return whether optional scene snapping is enabled."""
		return not hasattr(scene, "grid_snap_enabled") or bool(scene.grid_snap_enabled)

	# Compatibility wrappers retain the established DrawMode helper surface.
	_get_angle = staticmethod(ferrum_qt.modes.draw_geometry.get_angle)
	_on_which_side = staticmethod(ferrum_qt.modes.draw_geometry.on_which_side)
	_find_least_crowded_place = staticmethod(ferrum_qt.modes.draw_geometry.find_least_crowded_place)
	_point_on_circle = staticmethod(ferrum_qt.modes.draw_geometry.point_on_circle)

	#============================================
	def _find_place(self, atom_model: AtomModel, mol_model: MoleculeModel,
			bond_length: float, added_order: int = 1) -> tuple[float, float]:
		"""Calculate an endpoint and retain only Draw's transoid alternation state."""
		point, self._sign, self._last_used_atom_id = ferrum_qt.modes.draw_geometry.find_place(
			atom_model, mol_model, bond_length, self._sign, self._last_used_atom_id,
			added_order,
		)
		return point

	#============================================
	def mouse_press(self, scene_pos: PySide6.QtCore.QPointF, event: object) -> None:
		"""Capture durable draw intent while the selected projection is current."""
		self._reset_gesture()
		gesture = self._gesture
		gesture.press_position = (scene_pos.x(), scene_pos.y())
		atom_item = self._find_atom_at(scene_pos)
		if atom_item is not None:
			self._capture_atom_gesture(atom_item)
			return
		bond_item = self._find_bond_at(scene_pos)
		if bond_item is not None:
			self._capture_bond_gesture(bond_item)
			return
		self._capture_blank_gesture(scene_pos)

	#============================================
	def mouse_move(self, scene_pos: PySide6.QtCore.QPointF, event: object) -> None:
		"""Render a disposable snapped preview for an active atom gesture."""
		gesture = self._gesture
		if gesture.kind != "atom" or gesture.source_position is None:
			return
		press_position = gesture.press_position
		if press_position is None:
			return
		if math.dist((scene_pos.x(), scene_pos.y()), press_position) < DRAG_THRESHOLD:
			return
		gesture.dragging = True
		start_x, start_y = gesture.source_position
		target_x, target_y = self._snap_drag_target(scene_pos)
		scene = self._env.scene
		if scene is None:
			return
		if gesture.preview_line is None:
			color = render_ops_painter.get_canvas_color("preview")
			pen = PySide6.QtGui.QPen(PySide6.QtGui.QColor(color))
			pen.setWidthF(1.5)
			pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
			gesture.preview_line = scene.addLine(start_x, start_y, target_x, target_y, pen)
			gesture.preview_scene = scene
		else:
			gesture.preview_line.setLine(start_x, start_y, target_x, target_y)

	#============================================
	def mouse_release(self, scene_pos: PySide6.QtCore.QPointF, event: object) -> None:
		"""Submit exactly one immutable backend operation for the completed gesture."""
		request_data = self._request_data_for_release(scene_pos)
		self._reset_gesture()
		if request_data is None:
			return
		request = self._make_persistent_request(*request_data)
		if request is None:
			self.status_message.emit("Document cannot accept a persistent edit")
			return
		outcome = self._persistent_operation(request)
		self._restore_result_selection(outcome)
		self.status_message.emit(outcome.message)

	#============================================
	def _capture_atom_gesture(self, atom_item: object) -> None:
		"""Reduce an atom projection to durable IDs and a scalar default endpoint."""
		atom = atom_item.atom_model
		molecule = self._env.find_molecule_for_atom(atom)
		molecule_id = getattr(molecule, "mol_id", None)
		atom_id = atom.backend_durable_id
		if not molecule_id or not atom_id:
			self.status_message.emit("Selected atom has no durable backend identity")
			return
		gesture = self._gesture
		gesture.kind = "atom"
		gesture.source_molecule_id = str(molecule_id)
		gesture.source_atom_id = str(atom_id)
		gesture.source_position = (atom.x, atom.y)
		gesture.default_target_position = self._find_place(
			atom, molecule, self._get_bond_length(), self._current_bond_order,
		)
		self.status_message.emit("Release to add bond; drag to set angle")

	#============================================
	def _capture_bond_gesture(self, bond_item: object) -> None:
		"""Reduce a bond projection to durable IDs before release."""
		bond = bond_item.bond_model
		molecule = self._env.find_molecule_for_bond(bond)
		molecule_id = getattr(molecule, "mol_id", None)
		bond_id = bond.backend_durable_id
		if not molecule_id or not bond_id:
			self.status_message.emit("Selected bond has no durable backend identity")
			return
		gesture = self._gesture
		gesture.kind = "bond"
		gesture.source_molecule_id = str(molecule_id)
		gesture.bond_id = str(bond_id)
		self.status_message.emit("Release to apply bond tool")

	#============================================
	def _capture_blank_gesture(self, scene_pos: PySide6.QtCore.QPointF) -> None:
		"""Capture a fresh bonded-pair intent without creating Qt models."""
		source_x, source_y = scene_pos.x(), scene_pos.y()
		scene = self._env.scene
		if scene is not None and self._grid_snap_enabled(scene):
			source_x, source_y = scene.snap_to_grid(source_x, source_y)
		bond_length = self._get_bond_length()
		gesture = self._gesture
		gesture.kind = "blank"
		gesture.source_position = (source_x, source_y)
		gesture.default_target_position = (
			source_x + math.cos(math.pi / 6) * bond_length,
			source_y - math.sin(math.pi / 6) * bond_length,
		)
		self.status_message.emit("Release to create a new bonded pair")

	#============================================
	def _request_data_for_release(self, scene_pos: PySide6.QtCore.QPointF) -> tuple | None:
		"""Build a scalar structural request from the captured transient state."""
		gesture = self._gesture
		if gesture.kind == "blank":
			if gesture.source_position is None or gesture.default_target_position is None:
				return None
			return self._structural_request_data("create-bonded-pair", "Draw bonded pair", (
				("source_position", gesture.source_position),
				("target_position", gesture.default_target_position),
				("element", self._current_element),
			), frozenset())
		if gesture.kind == "bond":
			if gesture.source_molecule_id is None or gesture.bond_id is None:
				return None
			return self._structural_request_data("apply-bond-tool", "Apply bond tool", (
				("molecule_id", gesture.source_molecule_id), ("bond_id", gesture.bond_id),
			), frozenset((("molecule", gesture.source_molecule_id), ("bond", gesture.bond_id))))
		if (gesture.kind != "atom" or gesture.source_molecule_id is None
				or gesture.source_atom_id is None or gesture.default_target_position is None):
			return None
		if gesture.dragging:
			end_item = self._find_atom_at(scene_pos)
			if end_item is not None:
				end_atom = end_item.atom_model
				end_molecule = self._env.find_molecule_for_atom(end_atom)
				end_id = end_atom.backend_durable_id
				if getattr(end_molecule, "mol_id", None) == gesture.source_molecule_id and end_id:
					if end_id == gesture.source_atom_id:
						return None
					return self._structural_request_data("join-atoms", "Join atoms", (
						("molecule_id", gesture.source_molecule_id),
						("source_atom_id", gesture.source_atom_id), ("target_atom_id", str(end_id)),
					), frozenset((("molecule", gesture.source_molecule_id),
						("atom", gesture.source_atom_id), ("atom", str(end_id)))))
				if end_molecule is not None:
					self.status_message.emit("Draw joins atoms only within one molecule")
					return None
			target = self._snap_drag_target(scene_pos)
		else:
			target = gesture.default_target_position
		return self._structural_request_data("extend-atom", "Extend atom", (
			("molecule_id", gesture.source_molecule_id),
			("source_atom_id", gesture.source_atom_id), ("target_position", target),
			("element", self._current_element),
		), frozenset((("molecule", gesture.source_molecule_id),
			("atom", gesture.source_atom_id))))

	#============================================
	def _snap_drag_target(self, scene_pos: PySide6.QtCore.QPointF) -> tuple[float, float]:
		"""Return the current fixed/free drag target using the shared snap policy."""
		start_x, start_y = self._gesture.source_position
		target_x, target_y = self._point_on_circle(
			start_x, start_y, self._get_bond_length(),
			scene_pos.x() - start_x, scene_pos.y() - start_y,
			resolution=self._angle_resolution,
		)
		if not self._fixed_length:
			target_x, target_y = scene_pos.x(), scene_pos.y()
		scene = self._env.scene
		if scene is not None and self._grid_snap_enabled(scene):
			target_x, target_y = scene.snap_to_grid(target_x, target_y)
		return (target_x, target_y)

	#============================================
	def _structural_request_data(self, kind: str, label: str, fields: tuple,
			target_keys: frozenset) -> tuple | None:
		"""Add revision and draw settings to a backend-owned structural operation."""
		owner = getattr(self._persistent_operation, "__self__", None)
		if owner is None or not hasattr(owner, "backend_snapshot"):
			return None
		payload = (("expected_revision", owner.backend_snapshot.revision), ("kind", kind),
			*fields, ("bond_type", self._current_bond_type),
			("bond_order", self._current_bond_order), ("simple_double", self._simple_double))
		return label, kind, payload, target_keys

	#============================================
	def _make_persistent_request(self, label: str, _kind: str, payload: tuple,
			target_keys: frozenset) -> object | None:
		"""Create the immutable session request without retaining session state."""
		if self._persistent_operation is None:
			return None
		import ferrum_qt.models.document_session
		return ferrum_qt.models.document_session.PersistentOperationRequest(
			"draw.structure", label, payload, target_keys,
		)

	#============================================
	def _restore_result_selection(self, outcome: object) -> None:
		"""Select only newly projected backend results after an accepted operation."""
		if getattr(outcome, "status", None) != "accepted":
			return
		result = getattr(outcome, "structural_result", None)
		if result is None:
			return
		keys = frozenset(("atom", identifier) for identifier in result.created_atom_ids)
		keys |= frozenset(("bond", identifier) for identifier in (
			*result.created_bond_ids, *result.updated_bond_ids,
		))
		if not keys or self._env.scene is None:
			return
		self._env.scene.clearSelection()
		ferrum_qt.canvas.document_projection.select_projected_persistent_keys(self._env.scene, keys)

	#============================================
	def _reset_gesture(self) -> None:
		"""Retire the preview through its recorded scene and clear all references."""
		preview_line, preview_scene = self._gesture.clear()
		if preview_line is None or preview_scene is None:
			return
		coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
		coordinator.retire_scene_projection_items(preview_scene, [preview_line],
			reaper=self._graphics_retirement_reaper)
		coordinator.raise_if_callback_failed("Draw preview retirement failed")
