"""Focused Qt boundary tests for Rust-owned direct-root interaction helpers."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.direct_root_interaction_tab
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


#============================================
def test_direct_root_tab_facade_delegates_opaque_handles() -> None:
	"""The tab façade forwards opaque values without interpreting their contents."""
	class Tab(ferrum_qt.ferrum.direct_root_interaction_tab.FerrumNativeDirectRootInteractionTabMixin):
		def __init__(self) -> None:
			self.current_snapshot = type("Snapshot", (), {"revision": 7, "digest": "d"})()
			self._session = type("Session", (), {})()
			self._session.observe_render_interaction_v1 = lambda revision, digest: (revision, digest)
			self._session.select_render_interaction_roots_v1 = lambda observation, previous, query: (observation, previous, query)
			self._session.begin_render_interaction_translation_v1 = lambda selection, x, y, snap: (selection, x, y, snap)
			self._session.preview_render_interaction_translation_v1 = lambda gesture, x, y: (gesture, x, y)
			self._session.commit_render_interaction_translation_v1 = lambda gesture, preview: type(
				"Commit", (), {"result": (gesture, preview)},
			)()
			self.installed = None

		def _require_mutable(self) -> None:
			return None

		def _install_mutation_result(self, result: object) -> None:
			self.installed = result

	tab = Tab()
	observation = tab.observe_direct_root_interaction()
	selection = tab.select_direct_roots(observation, None, "query")
	gesture = tab.begin_direct_root_translation(selection, 1.0, 2.0, "snap")
	preview = tab.preview_direct_root_translation(gesture, 3.0, 4.0)
	commit = tab.commit_direct_root_translation(gesture, preview)
	assert observation == (7, "d")
	assert selection == (observation, None, "query")
	assert gesture == (selection, 1.0, 2.0, "snap")
	assert preview == (gesture, 3.0, 4.0)
	assert tab.installed == (gesture, preview)
	assert commit.result == (gesture, preview)
