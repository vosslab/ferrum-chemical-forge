"""Deterministic V4 Qt replay checks for Rust-issued atom-label geometry."""

# PIP3 modules
import pytest
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.canvas.items.ferrum_plan_item
import ferrum_qt.canvas.telex_glyph_outline
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.themes.theme_loader


if not hasattr(engine, "RenderPlanV4"):
	pytest.skip("requires the rebuilt ferrum_chem V4 render binding", allow_module_level=True)


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml"><standard area_color="#ffffff"/><molecule id="m">
<atom id="c" name="C"><point x="0" y="0"/></atom>
<atom id="o" name="O"><point x="0" y="30"/></atom>
<atom id="cl" name="Cl"><point x="30" y="30"/></atom>
<atom id="n" name="N" charge="1" explicit_hydrogens="3" hydrogens="on"><point x="60" y="60"/></atom>
<bond id="co" start="c" end="o" type="n1"/>
<bond id="ocl" start="o" end="cl" type="n1"/>
<bond id="cln" start="cl" end="n" type="n1"/>
</molecule></cdml>
"""
_TOLERANCE = 0.000_001
_INTERLEAVED_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
	'<atom id="a" name="C"><point x="0" y="0"/></atom>'
	'<atom id="hidden" name="O" show="no"><point x="20" y="0"/></atom>'
	'<atom id="b" name="N"><point x="40" y="0"/></atom>'
	'</molecule></cdml>'
)
_AUTHORED_ATOMS = {
	"c": ("C", 0.0, 0.0, "C"),
	"o": ("O", 0.0, 30.0, "O"),
	"cl": ("Cl", 30.0, 30.0, "Cl"),
	"n": ("N", 60.0, 60.0, "NH3+"),
}


#============================================
def _label_path(label: object, anchor: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		runs: tuple[object, ...]) -> PySide6.QtGui.QPainterPath:
	"""Build only the renderer-issued Telex glyph outlines for one label run tuple."""
	font = telex.raw_font(label.text.size)
	origin = PySide6.QtCore.QPointF(
		label.text.origin.x + anchor.x, label.text.origin.y + anchor.y,
	)
	return ferrum_qt.canvas.telex_glyph_outline.path_from_runs(runs, origin, font)


#============================================
def _bounds(path: PySide6.QtGui.QPainterPath) -> tuple[float, float, float, float]:
	"""Return stable Qt outline extents for comparison with Rust-issued ink bounds."""
	rect = path.boundingRect()
	return rect.left(), rect.top(), rect.right(), rect.bottom()


#============================================
def _issue_v4_observation() -> object:
	"""Build the one real V4 document observation used by the offscreen consumer checks."""
	return engine.DocumentSession.load(_SOURCE).observe_render(0)


#============================================
def _atom_batch_by_source_id(observation: object) -> dict[str, object]:
	"""Map authored atom IDs through projection identity before selecting V4 batches."""
	projection_atoms = observation.document.projection.molecules[0].atoms
	by_source_id = {}
	for source_id, (element, x, y, _expected_label) in _AUTHORED_ATOMS.items():
		matches = tuple(
			atom for atom in projection_atoms
			if (atom.element, atom.position.x, atom.position.y) == (element, x, y)
		)
		assert len(matches) == 1
		document_object_id = matches[0].document_object_id
		batch = next(
			batch for batch in observation.molecule_plans[0].plan.batches
			if batch.target.document_object_id == document_object_id
		)
		assert type(batch.content) is engine.AtomRenderBatchV1
		by_source_id[source_id] = batch
	return by_source_id


#============================================
def _bond_axis_kinds(plan: object) -> set[str]:
	"""Classify nonzero final axes only from closed bond-operation payloads."""
	kinds = set()
	for batch in plan.batches:
		if batch.content.kind != "bond":
			continue
		for operation in batch.content.typed_operations:
			if operation.kind not in {"line", "double_bond_carrier_mark"}:
				continue
			payload = operation.operation
			dx = payload.end.x - payload.start.x
			dy = payload.end.y - payload.start.y
			assert (dx, dy) != (0.0, 0.0)
			if dy == 0.0:
				kinds.add("horizontal")
			elif dx == 0.0:
				kinds.add("vertical")
			else:
				kinds.add("diagonal")
	return kinds


#============================================
def test_v4_replay_uses_declared_atom_core_runs_and_issued_bounds(
		qapp: object,
		) -> None:
	"""C, O, Cl, and NH3+ retain their explicit core-run and full-label geometry."""
	observation = _issue_v4_observation()
	telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(engine.verified_telex_regular())
	atom_batches = _atom_batch_by_source_id(observation)
	assert len(tuple(
		batch for batch in observation.molecule_plans[0].plan.batches
		if batch.content.kind == "atom"
	)) == 4
	assert len(atom_batches) == len(_AUTHORED_ATOMS)
	for source_id, batch in atom_batches.items():
		content = batch.content
		label = content.label
		core_run = label.text.runs[label.core_element_run_index]
		full_path = _label_path(label, content.atom_local_anchor, telex, label.text.runs)
		core_path = _label_path(label, content.atom_local_anchor, telex, (core_run,))
		assert "".join(run.text for run in label.text.runs) == _AUTHORED_ATOMS[source_id][3]
		assert label.core_element_run_index == 0
		full = _bounds(full_path)
		core = _bounds(core_path)
		expected_full = tuple(
			value + offset for value, offset in zip(
				(label.full_ink_bounds.min_x, label.full_ink_bounds.min_y,
					label.full_ink_bounds.max_x, label.full_ink_bounds.max_y),
				(content.atom_local_anchor.x, content.atom_local_anchor.y,
					content.atom_local_anchor.x, content.atom_local_anchor.y), strict=True,
			)
		)
		expected_core = tuple(
			value + offset for value, offset in zip(
				(label.core_element_ink_bounds.min_x, label.core_element_ink_bounds.min_y,
					label.core_element_ink_bounds.max_x, label.core_element_ink_bounds.max_y),
				(content.atom_local_anchor.x, content.atom_local_anchor.y,
					content.atom_local_anchor.x, content.atom_local_anchor.y), strict=True,
			)
		)
		assert all(abs(actual - expected) <= _TOLERANCE for actual, expected in zip(full, expected_full, strict=True))
		assert all(abs(actual - expected) <= _TOLERANCE for actual, expected in zip(core, expected_core, strict=True))
		assert abs((core[0] + core[2]) / 2.0 - content.atom_local_anchor.x) <= _TOLERANCE
		assert abs((core[1] + core[3]) / 2.0 - content.atom_local_anchor.y) <= _TOLERANCE


#============================================
def test_v4_projection_preserves_mask_order_and_clears_bond_ink(
		qapp: object,
		) -> None:
	"""The ordinary Qt item keeps closed operation order and bond ink clear of labels."""
	observation = _issue_v4_observation()
	plan = observation.molecule_plans[0].plan
	atom_batches = _atom_batch_by_source_id(observation)
	atom_content_batches = tuple(batch for batch in plan.batches if batch.content.kind == "atom")
	bond_batches = tuple(batch for batch in plan.batches if batch.content.kind == "bond")
	assert len(atom_content_batches) == 4
	assert len(bond_batches) == 3
	assert tuple(batch.paint_order for batch in plan.batches) == (1, 3, 5, 7, 9, 11, 13)
	assert tuple(issue.paint_order for issue in plan.issues) == ()
	assert _bond_axis_kinds(plan) == {"horizontal", "vertical", "diagonal"}
	telex_resource = engine.verified_telex_regular()
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	items = tuple(
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem(
			plan, index, telex_resource, palette,
		)
		for index in range(len(plan.batches))
	)
	atom_items = tuple((item, batch) for item, batch in zip(items, plan.batches, strict=True)
		if batch.target.document_object_id in {
			value.target.document_object_id for value in atom_batches.values()
		})
	bond_items = tuple(item for item, batch in zip(items, plan.batches, strict=True)
		if batch.content.kind == "bond")
	assert len(atom_items) == 4
	assert all(type(batch.content.label.mask) is engine.MaskOpV1 for _item, batch in atom_items)
	assert all(
		tuple(command.z for command in item._commands[:2]) == (
			batch.content.label.mask.z, batch.content.label.text.z,
		)
		and all(left.z < right.z for left, right in zip(item._commands, item._commands[1:]))
		for item, batch in atom_items
	)
	telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
	for item, batch in zip(items, plan.batches, strict=True):
		if batch.content.kind != "atom":
			continue
		content = batch.content
		full_label_path = _label_path(
			content.label, content.atom_local_anchor, telex, content.label.text.runs,
		)
		for bond_item in bond_items:
			assert not full_label_path.intersects(bond_item._shape_path)


#============================================
def test_v4_projection_accepts_an_issue_interleaved_between_painted_batches(
		qapp: object,
		) -> None:
	"""A nonpainted exclusion retains source order without constraining batch replay."""
	session = engine.DocumentSession.load(_INTERLEAVED_SOURCE)
	snapshot = session.snapshot()
	observation = session.observe_render(snapshot.revision)
	plan = observation.molecule_plans[0].plan
	assert tuple(batch.paint_order for batch in plan.batches) == (0, 2)
	assert tuple(issue.paint_order for issue in plan.issues) == (1,)
	presentation = session.observe_presentation_render_plan_v1(
		snapshot.revision, snapshot.digest,
	)
	projection = ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
		observation,
		engine.verified_telex_regular(),
		presentation,
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	try:
		assert len(projection.items) == 2
		assert tuple(issue.paint_order for issue in projection.issues) == (1,)
		assert {projection.item_targets[item].document_object_id for item in projection.items} == {
			batch.target.document_object_id for batch in plan.batches
		}
	finally:
		projection.dispose()
