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
def _application() -> PySide6.QtWidgets.QApplication:
	"""Return the process application required for native scene construction."""
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
def test_renderer_plan_scene_preserves_durable_target_selection() -> None:
	"""A renderer plan installs one durable root that remains selectable by identity."""
	_application()
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
	target = scene.roots[0].target
	assert target.kind == "document_object"
	assert target.document_object_id
	scene.select_durable((target.durable_selection_key(),))
	selected = scene.selected_targets(graphics_scene)
	assert selected[0].durable_selection_key() == target.durable_selection_key()
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
def test_curved_arrow_plan_scene_retains_renderer_target() -> None:
	"""A curved authored arrow is displayed through its public renderer-plan target."""
	_application()
	plan = _renderer_plan(
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="curve" type="curved-normal">'
		'<point x="0" y="0"/><point x="20" y="20"/>'
		'<point x="40" y="0"/></arrow></cdml>',
	)
	scene = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
		plan, ferrum_qt.ferrum.engine.verified_telex_regular(),
	)
	target = scene.roots[0].target
	assert target.kind == "document_object"
	assert target.document_object_id
	scene.dispose_detached()
