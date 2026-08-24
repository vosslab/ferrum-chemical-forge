"""Renderer-plan presentation scene behavior."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

pytest.importorskip("ferrum_chem")

# local repo modules
import ferrum_qt.canvas.ferrum_presentation_render_plan
import ferrum_qt.ferrum.engine


#============================================
@pytest.fixture(scope="module")
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Return the shared offscreen application for plan-scene selection."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _renderer_plan(cdml: str) -> object:
	"""Request the public renderer plan fenced to one current document snapshot."""
	session = ferrum_qt.ferrum.engine.DocumentSession.load(cdml)
	observation = session.observe(0)
	return session.observe_presentation_render_plan_v1(
		observation.snapshot.revision, observation.snapshot.digest,
	)


#============================================
def test_renderer_plan_scene_preserves_durable_target_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A renderer plan installs one durable root that remains selectable by identity."""
	del qapp
	plan = _renderer_plan(
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal" '
		'start="no" end="yes"><point x="0" y="0"/>'
		'<point x="40" y="0"/></arrow></cdml>',
	)
	scene = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
		plan, ferrum_qt.ferrum.engine.verified_telex_regular(),
	)
	graphics_scene = PySide6.QtWidgets.QGraphicsScene()
	for root in scene.roots:
		graphics_scene.addItem(root)
	durable_id = scene.roots[0].target.id
	assert durable_id is not None
	scene.select_durable((durable_id,))
	selected = scene.selected_targets(graphics_scene)
	assert len(selected) == 1 and selected[0].id == durable_id
	scene.dispose_detached()


#============================================
def test_renderer_plan_scene_rejects_non_native_plan() -> None:
	"""Plan replay accepts the immutable renderer type rather than a structural look-alike."""
	with pytest.raises(
		ferrum_qt.canvas.ferrum_presentation_render_plan.PresentationRenderPlanError,
	):
		ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
			object(), ferrum_qt.ferrum.engine.verified_telex_regular(),
		)


#============================================
def test_curved_arrow_plan_scene_retains_renderer_target(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A curved authored arrow is displayed through its public renderer-plan target."""
	del qapp
	plan = _renderer_plan(
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="curve" type="curved-normal">'
		'<point x="0" y="0"/><point x="20" y="20"/>'
		'<point x="40" y="0"/></arrow></cdml>',
	)
	scene = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
		plan, ferrum_qt.ferrum.engine.verified_telex_regular(),
	)
	assert scene.roots[0].target.source_id == "curve"
	scene.dispose_detached()
