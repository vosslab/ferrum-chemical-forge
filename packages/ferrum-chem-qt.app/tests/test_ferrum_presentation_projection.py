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
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.canvas.items.ferrum_text_item
import ferrum_qt.ferrum.engine
import ferrum_qt.themes.theme_loader


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
def _palette(name: str) -> object:
	"""Select one explicit YAML-owned document palette for focused scene proofs."""
	return ferrum_qt.themes.theme_loader.get_document_display_palette(name)


#============================================
def _preview_plan(kind: str) -> object:
	"""Build one real native preview plan for a documented presentation gesture."""
	session = ferrum_qt.ferrum.engine.DocumentSession.load(
		'<cdml xmlns="urn:ferrum:cdml"><standard font_size="18"/></cdml>',
	)
	snapshot = session.snapshot()
	if kind == "vector":
		gesture = session.begin_presentation_creation_gesture_v1(
			snapshot.revision, snapshot.digest,
			ferrum_qt.ferrum.engine.PresentationGestureKindV1.straight_normal_arrow,
			0.0, 0.0, ferrum_qt.ferrum.engine.ArrowGestureStyleV1(),
			ferrum_qt.ferrum.engine.PresentationGestureSnapPolicyV1(),
		)
		return session.preview_presentation_creation_gesture_v1(gesture, 30.0, 0.0).plan
	if kind == "plus":
		gesture = session.begin_presentation_creation_gesture_v1(
			snapshot.revision, snapshot.digest,
			ferrum_qt.ferrum.engine.PresentationGestureKindV1.plus,
			72.0, 36.0, None,
			ferrum_qt.ferrum.engine.PresentationGestureSnapPolicyV1(),
		)
		return session.preview_presentation_creation_gesture_v1(gesture, 72.0, 36.0).plan
	raise AssertionError(f"unknown presentation preview kind: {kind}")


#============================================
def _material_color(root: object) -> str:
	"""Return one retained foreground color from each approved persistent root type."""
	plan = ferrum_qt.canvas.ferrum_presentation_render_plan
	if type(root) in (plan.RendererPlanRootItem, plan.RendererPreviewRootItem):
		pen = root._commands[0].pen
		assert pen is not None
		return pen.color().name()
	if type(root) is ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem:
		return root.foreground_color.name()
	if type(root) is ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem:
		return root.foreground_color.name()
	if type(root) is plan.RendererPreviewPlusItem:
		return root._foreground.color().name()
	raise AssertionError(f"unrecognized retained presentation root: {type(root)!r}")


#============================================
def _root_identity(root: object) -> tuple[object, ...]:
	"""Capture retained Qt and renderer facts that a palette refresh must preserve."""
	plan = ferrum_qt.canvas.ferrum_presentation_render_plan
	if type(root) in (plan.RendererPlanRootItem, plan.RendererPreviewRootItem):
		return root, root.boundingRect(), root.shape(), root.isSelected(), root.zValue()
	if type(root) is ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem:
		return (
			root, root.boundingRect(), root.shape(), root.glyph_path, id(root.target),
			root.isSelected(), root.zValue(),
		)
	if type(root) is ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem:
		return (
			root, root.boundingRect(), root.shape(), root.glyph_path, id(root.target),
			root.isSelected(), root.zValue(),
		)
	if type(root) is plan.RendererPreviewPlusItem:
		return (
			root, root.boundingRect(), root._glyph_path, root._background_path,
			root.isSelected(), root.zValue(),
		)
	raise AssertionError(f"unrecognized retained presentation root: {type(root)!r}")


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
		plan, ferrum_qt.ferrum.engine.verified_telex_regular(), _palette("light"),
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
			object(), ferrum_qt.ferrum.engine.verified_telex_regular(), _palette("light"),
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
		plan, ferrum_qt.ferrum.engine.verified_telex_regular(), _palette("light"),
	)
	target = scene.roots[0].target
	assert target.kind == "document_object"
	assert target.document_object_id
	scene.dispose_detached()


#============================================
def test_renderer_plan_refresh_replaces_vector_material_without_replacing_identity() -> None:
	"""A palette change preserves one vector root's renderer and Qt identity facts."""
	_application()
	plan = _renderer_plan(
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal" '
		'start="no" end="yes"><point x="0" y="0"/>'
		'<point x="40" y="0"/></arrow></cdml>',
	)
	scene = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
		plan, ferrum_qt.ferrum.engine.verified_telex_regular(), _palette("light"),
	)
	root = scene.roots[0]
	bounds = root.boundingRect()
	shape = root.shape()
	target = root.target
	root.setSelected(True)
	root.setZValue(37.0)
	before = root._commands[0].pen.color()
	scene.refresh_display_palette(_palette("dark"))
	after = root._commands[0].pen.color()
	assert before != after
	assert root.boundingRect() == bounds
	assert root.shape() == shape
	assert root.target is target
	assert root.isSelected()
	assert root.zValue() == 37.0
	assert scene.revision == plan.revision
	assert scene.digest == plan.digest
	scene.dispose_detached()


#============================================
@pytest.mark.parametrize(
	("route", "expected_types", "semantic_indexes", "authored_indexes"),
	(
		(
			"persistent",
			(
				ferrum_qt.canvas.ferrum_presentation_render_plan.RendererPlanRootItem,
				ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem,
				ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem,
			),
			(0, 1), (2,),
		),
		(
			"preview_vector",
			(ferrum_qt.canvas.ferrum_presentation_render_plan.RendererPreviewRootItem,),
			(0,), (),
		),
		(
			"preview_plus",
			(ferrum_qt.canvas.ferrum_presentation_render_plan.RendererPreviewPlusItem,),
			(0,), (),
		),
	),
)
def test_native_persistent_and_preview_roots_refresh_their_materials_in_place(
		route: str, expected_types: tuple[type[object], ...], semantic_indexes: tuple[int, ...],
		authored_indexes: tuple[int, ...],
		) -> None:
	"""Every admitted root refreshes material while retaining native geometry and identity."""
	_application()
	light = _palette("light")
	if route == "persistent":
		plan = _renderer_plan(
			'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal" '
			'start="no" end="yes"><point x="0" y="0"/><point x="40" y="0"/>'
			'</arrow><plus id="p"><point x="10" y="20"/></plus><text id="t">'
			'<point x="20" y="30"/><font size="12" color="#123456"/>'
			'<ftext>Ferrum</ftext></text></cdml>',
		)
		scene = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
			plan, ferrum_qt.ferrum.engine.verified_telex_regular(), light,
		)
	elif route == "preview_vector":
		scene = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_preview_render_plan(
			_preview_plan("vector"), ferrum_qt.ferrum.engine.verified_telex_regular(), light,
		)
	else:
		scene = ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_preview_render_plan(
			_preview_plan("plus"), ferrum_qt.ferrum.engine.verified_telex_regular(), light,
		)
	try:
		assert tuple(type(root) for root in scene.roots) == expected_types
		for root in scene.roots:
			root.setSelected(True)
			root.setZValue(37.0)
		before_identity = tuple(_root_identity(root) for root in scene.roots)
		before_colors = tuple(_material_color(root) for root in scene.roots)
		scene.refresh_display_palette(_palette("dark"))
		after_colors = tuple(_material_color(root) for root in scene.roots)
		assert tuple(_root_identity(root) for root in scene.roots) == before_identity
		for index in semantic_indexes:
			assert after_colors[index] != before_colors[index]
		for index in authored_indexes:
			assert before_colors[index] == after_colors[index] == "#123456"
	finally:
		scene.dispose_detached()


#============================================
def test_presentation_preview_scene_refuses_a_root_without_the_refresh_contract() -> None:
	"""A retained root cannot enter the preview scene without palette refresh behavior."""
	_application()
	class _StructuralRefreshLookalike(PySide6.QtWidgets.QGraphicsRectItem):
		"""A graphics item that exposes the method without joining the contract."""

		#============================================
		def refresh_display_palette(self, palette: object) -> None:
			"""Expose the structural shape that nominal admission must refuse."""
			del palette

	with pytest.raises(
			ferrum_qt.canvas.ferrum_presentation_render_plan.PresentationRenderPlanError,
			match="must implement refresh_display_palette",
	):
		ferrum_qt.canvas.ferrum_presentation_render_plan.FerrumPresentationPreviewScene(
			(_StructuralRefreshLookalike(),),
		)
