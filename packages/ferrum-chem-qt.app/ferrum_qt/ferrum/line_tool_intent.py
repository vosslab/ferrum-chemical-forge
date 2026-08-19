"""Shared immutable state for Ferrum revision-bound pointer gestures."""

# Standard Library
import dataclasses
import enum

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.rotation
import ferrum_qt.ferrum.translation


#============================================
class _NativeLineTool(enum.Enum):
	"""Closed Ferrum tools that share one revision-bound line gesture."""

	DRAW_BOND = "draw_bond"
	CREATE_WAVY = "create_wavy"
	CREATE_RECTANGULAR_BRACKET = "create_rectangular_bracket"
	CREATE_ROUND_BRACKET = "create_round_bracket"
	MOVE_ATOM = "move_atom"
	ROTATE_ATOMS = "rotate_atoms"
	TRANSLATE_ROOTS = "translate_roots"
	INSERT_CYCLOHEXANE_RING = "insert_cyclohexane_ring"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _LineGestureIntent:
	"""One revision-bound atom pointer gesture and its local preview."""

	tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	tool: _NativeLineTool
	drawing: ferrum_qt.ferrum.drawing_parameters.FerrumNativeDrawingParametersSnapshot | None = None
	start_atom_id: str | None = None
	start_scene: PySide6.QtCore.QPointF | None = None
	press_scene: PySide6.QtCore.QPointF | None = None
	preview: PySide6.QtWidgets.QGraphicsItem | None = None
	rotation_selection: ferrum_qt.ferrum.rotation.FerrumNativeRotationSelection | None = None
	rotation_preview: ferrum_qt.ferrum.rotation.FerrumNativeRotationPreview | None = None
	translation_selection: ferrum_qt.ferrum.translation.FerrumNativeTranslationSelection | None = None
	translation_preview: ferrum_qt.ferrum.translation.FerrumNativeTranslationPreview | None = None
	translation_snap_enabled: bool | None = None
	translation_delta: tuple[float, float] = (0.0, 0.0)
	last_angle: float | None = None
	accumulated_angle: float = 0.0
	regular_ring_prepared: object | None = None
