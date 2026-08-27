"""Focused Qt boundary tests for Rust-owned direct-root interaction helpers."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.direct_root_preview
import ferrum_qt.ferrum.document_display_refresh
import ferrum_qt.themes.document_display_palette
import ferrum_qt.themes.theme_loader


#============================================
class _Bounds:
	"""One frozen stand-in for a Rust-issued bounds value."""

	#============================================
	def __init__(self, left: float, top: float, right: float, bottom: float) -> None:
		self.left = left
		self.top = top
		self.right = right
		self.bottom = bottom


#============================================
class _View:
	"""Minimal Qt scene holder used to prove overlay-only projection."""

	#============================================
	def __init__(self, scene: PySide6.QtWidgets.QGraphicsScene) -> None:
		self._scene = scene

	#============================================
	def scene(self) -> PySide6.QtWidgets.QGraphicsScene:
		return self._scene


#============================================
class _Tab:
	"""Minimal tab shape accepted by the overlay helper."""

	#============================================
	def __init__(
			self, scene: PySide6.QtWidgets.QGraphicsScene,
			document_display_palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
	) -> None:
		self.view = _View(scene)
		self.document_display_palette = document_display_palette
		self._document_display_refreshables = (
			ferrum_qt.ferrum.document_display_refresh.
			DocumentDisplayPaletteRefreshRegistryV1()
		)

	#============================================
	def register_document_display_refreshable(
			self,
			refreshable: ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRefreshableV1,
			) -> None:
		"""Retain one attached transient document-display object for theme refresh."""
		self._document_display_refreshables.register(refreshable)


#============================================
def test_direct_root_bounds_preview_draws_only_issued_bounds(qapp: object) -> None:
	"""The overlay copies Rust bounds and does not inspect projection items."""
	scene = PySide6.QtWidgets.QGraphicsScene()
	display_palette = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	root = ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
		_Tab(scene, display_palette), (
			_Bounds(10.0, 20.0, 40.0, 70.0),
			_Bounds(50.0, 80.0, 90.0, 100.0),
		),
	)
	assert isinstance(root, PySide6.QtWidgets.QGraphicsPathItem)
	assert root.childItems() == []
	assert tuple(tuple((point.x(), point.y()) for point in polygon)
		for polygon in root.path().toSubpathPolygons()) == (
		((10.0, 20.0), (40.0, 20.0), (40.0, 70.0), (10.0, 70.0), (10.0, 20.0)),
		((50.0, 80.0), (90.0, 80.0), (90.0, 100.0), (50.0, 100.0), (50.0, 80.0)),
	)
	assert root.pen().color() == display_palette.color(
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PREVIEW_OUTLINE,
	)
	assert root.brush().color() == display_palette.color(
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PREVIEW_FILL,
	)
	assert root.zValue() == 1_000_000.0
	assert root.acceptedMouseButtons() == PySide6.QtCore.Qt.MouseButton.NoButton
