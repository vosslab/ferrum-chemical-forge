"""Shared immutable state for Ferrum revision-bound pointer gestures."""

# Standard Library
import dataclasses
import enum

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.rotation
import ferrum_qt.ferrum.curved_equilibrium_arrow
import ferrum_qt.ferrum.terminal_arrow


#============================================
class _NativeLineTool(enum.Enum):
	"""Closed Ferrum tools that share one revision-bound line gesture."""

	DRAW_BOND = "draw_bond"
	DRAW_ARROW = "draw_arrow"
	DRAW_EQUILIBRIUM_ARROW = "draw_equilibrium_arrow"
	DRAW_CURVED_ELECTRON_ARROW = "draw_curved_electron_arrow"
	DRAW_CURVED_RETRO_ARROW = "draw_curved_retro_arrow"
	DRAW_CURVED_REACTION_ARROW = "draw_curved_reaction_arrow"
	DRAW_CURVED_EQUILIBRIUM_ARROW = "draw_curved_equilibrium_arrow"
	DRAW_PLUS = "draw_plus"
	DRAW_LINE = "draw_line"
	DRAW_RECTANGLE = "draw_rectangle"
	DRAW_SQUARE = "draw_square"
	DRAW_OVAL = "draw_oval"
	DRAW_CIRCLE = "draw_circle"
	DRAW_POLYLINE = "draw_polyline"
	DRAW_POLYGON = "draw_polygon"
	INSERT_TEXT = "insert_text"
	CREATE_WAVY = "create_wavy"
	CREATE_RECTANGULAR_BRACKET = "create_rectangular_bracket"
	CREATE_ROUND_BRACKET = "create_round_bracket"
	MOVE_ATOM = "move_atom"
	ROTATE_ATOMS = "rotate_atoms"
	TRANSLATE_ROOTS = "translate_roots"
	INSERT_REGULAR_RING = "insert_regular_ring"
	ATTACH_CYCLOHEXANE_RING = "attach_cyclohexane_ring"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _LineGestureIntent:
	"""One revision-bound Ferrum pointer gesture and its local projection."""

	tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	tool: _NativeLineTool
	drawing: ferrum_qt.ferrum.drawing_parameters.FerrumNativeDrawingParametersSnapshot | None = None
	direct_bond_presentation: (
		ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation
	) = ferrum_qt.ferrum.drawing_parameters.DirectBondPresentation.NORMAL
	start_atom_id: str | None = None
	start_scene: PySide6.QtCore.QPointF | None = None
	press_scene: PySide6.QtCore.QPointF | None = None
	preview: PySide6.QtWidgets.QGraphicsItem | None = None
	rotation_selection: ferrum_qt.ferrum.rotation.FerrumNativeRotationSelection | None = None
	rotation_preview: ferrum_qt.ferrum.rotation.FerrumNativeRotationPreview | None = None
	last_angle: float | None = None
	accumulated_angle: float = 0.0
	regular_ring_center: PySide6.QtCore.QPointF | None = None
	regular_ring_size: int | None = None
	regular_ring_action: PySide6.QtGui.QAction | None = None
	attached_cyclohexane_pending: object | None = None
	attached_cyclohexane_cancel_blocked: bool = False
	direct_bond_start_probe: object | None = None
	direct_bond_snap_enabled: bool = False
	direct_bond_gesture: object | None = None
	prepared_transition: object | None = None
	presentation_gesture: object | None = None
	presentation_preview: object | None = None
	curved_equilibrium_arrow: ferrum_qt.ferrum.curved_equilibrium_arrow.CurvedEquilibriumArrowState | None = None
	terminal_arrow: ferrum_qt.ferrum.terminal_arrow.TerminalArrowState | None = None
	vector_gesture: object | None = None
	vector_preview: object | None = None
	path_gesture: object | None = None
	path_progress: object | None = None
	path_preview: object | None = None
	# Transient Qt/QTest press/double-click de-duplication token, never durable
	# geometry or Rust validation.
	last_accepted_path_press_viewport: PySide6.QtCore.QPoint | None = None
	text_gesture: object | None = None
	text_preview: object | None = None
	direct_root_observation: object | None = None
	direct_root_selection: object | None = None
	direct_root_gesture: object | None = None
	direct_root_preview: object | None = None
	direct_root_preview_item: PySide6.QtWidgets.QGraphicsItemGroup | None = None
	direct_root_marquee: PySide6.QtWidgets.QGraphicsRectItem | None = None
