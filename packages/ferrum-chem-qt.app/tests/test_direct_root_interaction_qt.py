"""Focused Qt boundary tests for Rust-owned direct-root interaction helpers."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.direct_root_preview


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
	def __init__(self, scene: PySide6.QtWidgets.QGraphicsScene) -> None:
		self.view = _View(scene)


#============================================
def test_direct_root_bounds_preview_draws_only_issued_bounds(qapp: object) -> None:
	"""The overlay copies Rust bounds and does not inspect projection items."""
	scene = PySide6.QtWidgets.QGraphicsScene()
	root = ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
		_Tab(scene), (_Bounds(10.0, 20.0, 40.0, 70.0),),
	)
	children = root.childItems()
	assert len(children) == 1
	rectangle = children[0].rect()
	assert rectangle.x() == 10.0
	assert rectangle.y() == 20.0
	assert rectangle.width() == 30.0
	assert rectangle.height() == 50.0

