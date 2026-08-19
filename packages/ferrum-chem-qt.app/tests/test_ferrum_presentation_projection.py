"""Behavioral tests for the exact Ferrum presentation polyline projection."""

# Standard Library
import types

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.canvas.ferrum_presentation_projection
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.canvas.items.ferrum_text_item


def _observation(cdml: str) -> object:
	"""Load one current immutable observation from the installed direct extension."""
	ferrum_chem = pytest.importorskip("ferrum_chem")
	return ferrum_chem.DocumentSession.load(cdml).observe(0)


def _successive_observations() -> tuple[object, object]:
	"""Return two real extension observations whose presentation root survives an edit."""
	ferrum_chem = pytest.importorskip("ferrum_chem")
	session = ferrum_chem.DocumentSession.load(
		'<cdml><molecule id="m"><atom id="a" element="C">'
		'<point x="0" y="0"/></atom></molecule>'
		'<polyline id="line" line_color="#112233" width="2">'
		'<point x="1" y="2"/><point x="4" y="5"/></polyline></cdml>',
	)
	before = session.observe(0)
	after = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_atom_element("a", "N"),
	).observation
	return before, after


#============================================
def test_projects_actual_extension_stroke_source_order_and_durable_target() -> None:
	"""A direct-wheel polyline becomes a selectable, noncosmetic scene item."""
	observation = _observation(
		'<cdml><polyline id="line" line_color="#112233" width="2">'
		'<point x="1" y="2"/><point x="4" y="5"/></polyline></cdml>',
	)
	projection = ferrum_qt.canvas.ferrum_presentation_projection.build_presentation_projection(
		observation,
	)
	item = next(iter(projection.durable_items.values()))
	assert item.pen().color().name() == "#112233" and not item.pen().isCosmetic()
	assert item.target.source_id == "line" and item.target.source_order == 0
	assert projection.roots == (item,) and item.parentItem() is None


#============================================
def test_multisegment_polyline_preserves_every_authored_bend() -> None:
	"""Qt follows the ordered Rust path rather than joining only its endpoints."""
	observation = _observation(
		'<cdml><polyline id="line" line_color="#112233" width="2">'
		'<point x="1" y="2"/><point x="4" y="5"/>'
		'<point x="2" y="7"/><point x="8" y="3"/></polyline></cdml>',
	)
	projection = ferrum_qt.canvas.ferrum_presentation_projection.build_presentation_projection(
		observation,
	)
	path = projection.items[0].path()
	assert [
		(path.elementAt(index).x, path.elementAt(index).y)
		for index in range(path.elementCount())
	] == [(1.0, 2.0), (4.0, 5.0), (2.0, 7.0), (8.0, 3.0)]


#============================================
def test_vector_shapes_use_projected_bounds_points_stroke_fill_and_kind(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Closed vector roots use only the semantic geometry and appearance DTOs."""
	del qapp
	observation = _observation(
		'<cdml><standard line_color="#123456" line_width="3" area_color="#abc"/>'
		'<rect id="r" x1="10" y1="8" x2="2" y2="4" area_color="none"/>'
		'<square id="s" x1="1" y1="2" x2="5" y2="6"/>'
		'<oval id="o" x1="0" y1="0" x2="8" y2="4" area_color="#010203"/>'
		'<circle id="c" x1="0" y1="0" x2="4" y2="6" area_color=""/>'
		'<polygon id="p" line_color="#fedcba" width="2">'
		'<point x="0" y="0"/><point x="5" y="1"/><point x="2" y="7"/>'
		'</polygon></cdml>',
	)
	projection = ferrum_qt.canvas.ferrum_presentation_projection.build_presentation_projection(
		observation,
	)
	assert [item.target.record_kind for item in projection.items] == [
		"rectangle", "square", "oval", "circle", "polygon",
	]
	rectangle, square, oval, circle, polygon = projection.items
	assert rectangle.path().boundingRect() == PySide6.QtCore.QRectF(2.0, 4.0, 8.0, 4.0)
	assert rectangle.brush().style() == PySide6.QtCore.Qt.BrushStyle.NoBrush
	assert square.brush().color().name() == "#aabbcc"
	assert square.pen().color().name() == "#123456" and square.pen().widthF() == 3.0
	assert oval.path().contains(PySide6.QtCore.QPointF(4.0, 2.0))
	assert not oval.path().contains(PySide6.QtCore.QPointF(0.1, 0.1))
	assert oval.brush().color().name() == "#010203"
	assert circle.brush().style() == PySide6.QtCore.Qt.BrushStyle.NoBrush
	assert [
		(polygon.path().elementAt(index).x, polygon.path().elementAt(index).y)
		for index in range(3)
	] == [(0.0, 0.0), (5.0, 1.0), (2.0, 7.0)]
	assert polygon.pen().color().name() == "#fedcba"


#============================================
def test_normal_arrow_uses_rust_axis_heads_and_reports_unsupported_families(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Qt paints supplied normal-arrow geometry and never substitutes retro art."""
	del qapp
	observation = _observation(
		'<cdml><arrow id="a" type="normal" start="no" end="yes" spline="no" '
		'width="2" color="#123456" shape="(8,10,3)">'
		'<point x="0" y="0"/><point x="40" y="0"/></arrow>'
		'<arrow id="retro" type="retro"><point x="0" y="10"/>'
		'<point x="40" y="10"/></arrow></cdml>',
	)
	projection = ferrum_qt.canvas.ferrum_presentation_projection.build_presentation_projection(
		observation,
	)
	assert len(projection.items) == 1
	item = projection.items[0]
	assert isinstance(item, ferrum_qt.canvas.ferrum_presentation_projection.ArrowProjectionItem)
	assert item.target.record_kind == "arrow" and item.target.source_order == 0
	assert [
		(item.axis_path.elementAt(index).x, item.axis_path.elementAt(index).y)
		for index in range(item.axis_path.elementCount())
	] == [(0.0, 0.0), (32.0, 0.0)]
	assert [
		(item.head_path.elementAt(index).x, item.head_path.elementAt(index).y)
		for index in range(4)
	] == [(40.0, 0.0), (30.0, 3.0), (32.0, 0.0), (30.0, -3.0)]
	assert item.pen.color().name() == "#123456" and item.pen.widthF() == 2.0
	assert [(issue.target.source_id, issue.code) for issue in projection.issues] == [
		("retro", "unsupported_arrow_type"),
	]


#============================================
def test_mixed_render_scene_keeps_each_document_root_at_its_rust_order(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Molecules and presentation roots share one scene without flattening child order."""
	del qapp
	ferrum_chem = pytest.importorskip("ferrum_chem")
	session = ferrum_chem.DocumentSession.load(
		'<cdml><molecule id="m1"><atom id="a1" element="C">'
		'<point x="0" y="0"/></atom></molecule>'
		'<polyline id="p1" line_color="#112233" width="2">'
		'<point x="1" y="2"/><point x="4" y="5"/></polyline>'
		'<polyline id="p2" line_color="#445566" width="3">'
		'<point x="2" y="3"/><point x="5" y="6"/></polyline>'
		'<molecule id="m2"><atom id="a2" element="N">'
		'<point x="8" y="0"/></atom></molecule></cdml>',
	)
	projection = ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
		session.observe_render(0), ferrum_chem.verified_telex_regular(),
	)
	presentation = projection.presentation
	assert presentation is not None
	document_roots = (*projection.molecule_roots, *presentation.roots)
	assert sorted(root.zValue() for root in document_roots) == [0.0, 1.0, 2.0, 3.0]
	assert [item.target.source_order for item in presentation.items] == [1, 2]
	assert all(item.parentItem() is None for item in presentation.items)
	projection.dispose()


#============================================
def test_fixed_plus_uses_verified_render_glyphs_anchor_and_explicit_paints(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Qt caches the API-issued fixed glyph plan without measuring or shaping text."""
	del qapp
	ferrum_chem = pytest.importorskip("ferrum_chem")
	session = ferrum_chem.DocumentSession.load(
		'<cdml><molecule id="m"><atom id="a" element="C">'
		'<point x="0" y="0"/></atom></molecule>'
		'<plus id="p" font_size="18" color="#123456" background-color="#abcdef">'
		'<point x="10" y="20"/></plus></cdml>',
	)
	observation = session.observe_render(0)
	plus = observation.plus_renders[0]
	projection = ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
		observation, ferrum_chem.verified_telex_regular(),
	)
	item = projection.durable_items[("plus", plus.target.id)]
	assert isinstance(item, ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem)
	assert item.pos() == PySide6.QtCore.QPointF(10.0, 20.0) and item.zValue() == 1.0
	assert not item.glyph_path.isEmpty() and item.glyph_path.elementCount() > 0
	background = item.background_path.boundingRect()
	assert background == PySide6.QtCore.QRectF(
		plus.bounds.left, plus.bounds.top,
		plus.bounds.right - plus.bounds.left,
		plus.bounds.bottom - plus.bounds.top,
	)
	assert item.foreground_color.name() == "#123456"
	assert item.background_color is not None
	assert item.background_color.name() == "#abcdef"
	projection.dispose()


#============================================
def test_direct_text_uses_backend_glyph_layout_and_durable_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Qt paints and selects Text without shaping, measuring, or interpreting CDML."""
	del qapp
	ferrum_chem = pytest.importorskip("ferrum_chem")
	session = ferrum_chem.DocumentSession.load(
		'<cdml><text id="label" background-color="#abcdef">'
		'<point x="10" y="20"/><font size="18" color="#123456"/>'
		'<ftext>Line one\nH&lt;sub&gt;2&lt;/sub&gt;O</ftext></text></cdml>',
	)
	observation = session.observe_render(0)
	text_render = observation.text_renders[0]
	projection = ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
		observation, ferrum_chem.verified_telex_regular(),
	)
	item = projection.durable_items[("text", text_render.target.id)]
	assert type(item) is ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem
	assert item.pos() == PySide6.QtCore.QPointF(10.0, 20.0)
	assert not item.glyph_path.isEmpty()
	assert item.foreground_color.name() == "#123456"
	assert item.background_color is not None
	assert item.background_color.name() == "#abcdef"
	projection.select_durable((("text", text_render.target.id),))
	assert projection.selected_durable_targets()[0].identifier == text_render.target.id
	projection.dispose()


#============================================
def test_idless_root_is_projection_local_not_an_operation_target() -> None:
	"""An id-less Rust root has no durable map entry or operation identifier."""
	observation = _observation(
		'<cdml><polyline line_color="#112233" width="2">'
		'<point x="1" y="2"/><point x="4" y="5"/></polyline></cdml>',
	)
	projection = ferrum_qt.canvas.ferrum_presentation_projection.build_presentation_projection(
		observation,
	)
	assert not projection.durable_items
	assert next(iter(projection.local_items.values())).target.id is None


#============================================
def test_structural_impostor_is_rejected_at_the_extension_boundary() -> None:
	"""A slotted look-alike cannot cross the production PyO3 DTO boundary."""
	with pytest.raises(ferrum_qt.canvas.ferrum_presentation_projection.PresentationProjectionError):
		ferrum_qt.canvas.ferrum_presentation_projection.build_presentation_projection(
			types.SimpleNamespace(),
		)


#============================================
def test_stale_candidate_preserves_prior_scene_projection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A duplicate or stale observation cannot disturb the current complete scene."""
	del qapp
	observation = _observation(
		'<cdml><polyline id="line" line_color="#112233" width="2">'
		'<point x="1" y="2"/><point x="4" y="5"/></polyline></cdml>',
	)
	scene = PySide6.QtWidgets.QGraphicsScene()
	controller = ferrum_qt.canvas.ferrum_presentation_projection.FerrumPresentationProjectionController(scene)
	assert controller.replace(observation)
	prior = controller.projection
	assert not controller.replace(observation) and controller.projection is prior


#============================================
def test_attachment_failure_restores_the_prior_scene_and_projection(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An unattachable candidate is retired without exposing a partial replacement."""
	del qapp
	before, after = _successive_observations()
	scene = PySide6.QtWidgets.QGraphicsScene()
	controller = ferrum_qt.canvas.ferrum_presentation_projection.FerrumPresentationProjectionController(scene)
	assert controller.replace(before)
	prior = controller.projection
	prior_items = tuple(scene.items())
	add_item = ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene
	monkeypatch.setattr(
		ferrum_qt.canvas.graphics_retirement, "add_item_to_captured_scene",
		lambda captured_scene, item: (
			add_item(captured_scene, item) if item is prior.roots[0] else False
		),
	)
	assert not controller.replace(after)
	assert controller.projection is prior and tuple(scene.items()) == prior_items


#============================================
def test_old_retirement_failure_restores_the_prior_scene_and_projection(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed old-root removal leaves no candidate item or changed controller state."""
	del qapp
	before, after = _successive_observations()
	scene = PySide6.QtWidgets.QGraphicsScene()
	controller = ferrum_qt.canvas.ferrum_presentation_projection.FerrumPresentationProjectionController(scene)
	assert controller.replace(before)
	prior = controller.projection
	prior_items = tuple(scene.items())
	remove_item = ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene
	monkeypatch.setattr(
		ferrum_qt.canvas.graphics_retirement, "remove_item_from_captured_scene",
		lambda captured_scene, item: (
			False if item is prior.roots[0] else remove_item(captured_scene, item)
		),
	)
	assert not controller.replace(after)
	assert controller.projection is prior and tuple(scene.items()) == prior_items


#============================================
def test_old_retirement_callback_failure_preserves_the_prior_projection(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A callback rejection occurs before the single-root scene transition begins."""
	del qapp
	before, after = _successive_observations()
	scene = PySide6.QtWidgets.QGraphicsScene()
	controller = ferrum_qt.canvas.ferrum_presentation_projection.FerrumPresentationProjectionController(scene)
	assert controller.replace(before)
	prior = controller.projection
	prior_items = tuple(scene.items())
	monkeypatch.setattr(
		ferrum_qt.canvas.ferrum_presentation_projection.PolylineProjectionItem, "dispose",
		lambda item: (_ for _ in ()).throw(RuntimeError("injected retirement callback failure")),
	)
	assert not controller.replace(after)
	assert controller.projection is prior and tuple(scene.items()) == prior_items
	assert isinstance(controller.last_replacement_error, RuntimeError)


#============================================
def test_native_handoff_exception_restores_the_prior_scene_and_projection(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A Ferrum attach exception after attachment rolls back both scene roots."""
	del qapp
	before, after = _successive_observations()
	scene = PySide6.QtWidgets.QGraphicsScene()
	controller = ferrum_qt.canvas.ferrum_presentation_projection.FerrumPresentationProjectionController(scene)
	assert controller.replace(before)
	prior = controller.projection
	prior_items = tuple(scene.items())
	add_item = ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene

	def attach_then_fail(captured_scene: PySide6.QtWidgets.QGraphicsScene, item: object) -> bool:
		if item is prior.roots[0]:
			return add_item(captured_scene, item)
		add_item(captured_scene, item)
		raise ValueError("injected Ferrum attachment failure")

	monkeypatch.setattr(
		ferrum_qt.canvas.graphics_retirement, "add_item_to_captured_scene", attach_then_fail,
	)
	assert not controller.replace(after)
	assert controller.projection is prior and tuple(scene.items()) == prior_items


#============================================
def test_successful_replacement_never_repeats_old_dispose_callbacks(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The preflight callback is the old projection's sole disposal callback pass."""
	del qapp
	before, after = _successive_observations()
	scene = PySide6.QtWidgets.QGraphicsScene()
	controller = ferrum_qt.canvas.ferrum_presentation_projection.FerrumPresentationProjectionController(scene)
	assert controller.replace(before)
	prior = controller.projection
	count = 0

	def count_dispose(_item: object) -> None:
		nonlocal count
		count += 1

	monkeypatch.setattr(
		ferrum_qt.canvas.ferrum_presentation_projection.PolylineProjectionItem, "dispose", count_dispose,
	)
	assert controller.replace(after)
	assert count == len(prior.items)


#============================================
def test_failed_rollback_invalidates_and_retains_the_transition_diagnostic(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A second Ferrum failure closes the controller instead of publishing mixed ownership."""
	del qapp
	before, after = _successive_observations()
	scene = PySide6.QtWidgets.QGraphicsScene()
	controller = ferrum_qt.canvas.ferrum_presentation_projection.FerrumPresentationProjectionController(scene)
	assert controller.replace(before)
	prior = controller.projection
	monkeypatch.setattr(
		ferrum_qt.canvas.graphics_retirement, "add_item_to_captured_scene",
		lambda _scene, _item: False,
	)
	assert not controller.replace(after)
	assert controller.projection is None and controller.retained_prior_projection is prior
	assert controller.retained_transition_errors
	assert not controller.replace(after)
	assert isinstance(controller.last_replacement_error, RuntimeError)


#============================================
def test_attached_candidate_with_failed_rollback_is_retained_not_detached_disposed(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A compound Ferrum failure leaves both ambiguous roots under explicit recovery ownership."""
	del qapp
	before, after = _successive_observations()
	scene = PySide6.QtWidgets.QGraphicsScene()
	controller = ferrum_qt.canvas.ferrum_presentation_projection.FerrumPresentationProjectionController(scene)
	assert controller.replace(before)
	prior = controller.projection
	add_item = ferrum_qt.canvas.graphics_retirement.add_item_to_captured_scene
	remove_item = ferrum_qt.canvas.graphics_retirement.remove_item_from_captured_scene

	def attach_then_fail(captured_scene: PySide6.QtWidgets.QGraphicsScene, item: object) -> bool:
		if item is prior.roots[0]:
			return add_item(captured_scene, item)
		add_item(captured_scene, item)
		raise ValueError("injected attach failure after Ferrum ownership changed")

	monkeypatch.setattr(
		ferrum_qt.canvas.graphics_retirement, "add_item_to_captured_scene", attach_then_fail,
	)
	monkeypatch.setattr(
		ferrum_qt.canvas.graphics_retirement, "remove_item_from_captured_scene",
		lambda captured_scene, item: (
			False if item is not prior.roots[0] else remove_item(captured_scene, item)
		),
	)
	assert not controller.replace(after)
	assert controller.projection is None and controller.retained_prior_projection is prior
	candidate = controller.retained_candidate_projection
	assert candidate is not None and candidate.roots[0].scene() is scene
