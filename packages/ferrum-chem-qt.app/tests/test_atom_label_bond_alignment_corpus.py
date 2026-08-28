"""Installed Qt fidelity checks for the shared atom-label/bond alignment corpus."""

# Standard Library
import json
import math
import pathlib
import xml.etree.ElementTree

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.telex_glyph_outline
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.themes.theme_loader


_CORPUS_PATH = (
	pathlib.Path(__file__).resolve().parents[2]
	/ "ferrum-rust"
	/ "crates"
	/ "document"
	/ "tests"
	/ "fixtures"
	/ "atom_label_bond_alignment_cases_v1.json"
)
_SCHEMA = "atom_label_bond_alignment_cases_v1"
_TOLERANCE = 0.000_001
_THIRD_LABEL_DETAIL = "bond final ink intersects a non-endpoint atom label"
_CASE_KEYS = frozenset(("name", "cdml", "expected_outcome", "offending_bond", "checks"))
_REQUIRED_CASE_KEYS = frozenset(("name", "cdml", "expected_outcome", "checks"))
_CHECK_KEYS = frozenset((
	"finite_geometry", "ordered_operations", "positive_bond_content",
	"full_ink_clearance", "require_mask", "core_run", "leading_superscript", "runs",
))


#============================================
def _corpus() -> tuple[dict[str, object], ...]:
	"""Read the one Rust-owned corpus after rejecting unknown JSON contract fields."""
	with _CORPUS_PATH.open(encoding="ascii") as input_file:
		value = json.load(input_file)
	assert type(value) is dict
	assert set(value) == {"schema", "cases"}
	assert value["schema"] == _SCHEMA
	cases = value["cases"]
	assert type(cases) is list and cases
	parsed = []
	for case in cases:
		assert type(case) is dict
		assert _REQUIRED_CASE_KEYS <= set(case) <= _CASE_KEYS
		assert type(case["name"]) is str and case["name"]
		assert type(case["cdml"]) is str and case["cdml"]
		assert case["expected_outcome"] in {"render", "unrenderable_target"}
		if case["expected_outcome"] == "unrenderable_target":
			assert type(case.get("offending_bond")) is str and case["offending_bond"]
		else:
			assert "offending_bond" not in case
		checks = case["checks"]
		assert type(checks) is dict and set(checks) <= _CHECK_KEYS
		for required in ("finite_geometry", "ordered_operations", "full_ink_clearance"):
			assert type(checks[required]) is bool
		for optional in ("positive_bond_content", "require_mask"):
			if optional in checks:
				assert type(checks[optional]) is bool
		if "core_run" in checks:
			core_run = checks["core_run"]
			assert type(core_run) is dict and set(core_run) == {"text", "index"}
			assert type(core_run["text"]) is str and type(core_run["index"]) is int
		if "leading_superscript" in checks:
			assert type(checks["leading_superscript"]) is str
		if "runs" in checks:
			assert type(checks["runs"]) is list
			for run in checks["runs"]:
				assert type(run) is dict and set(run) == {"text", "script"}
				assert type(run["text"]) is str
				assert run["script"] in {"baseline", "subscript", "superscript"}
		parsed.append(case)
	return tuple(parsed)


#============================================
def _label_path(content: object, telex: object,
		runs: tuple[object, ...]) -> PySide6.QtGui.QPainterPath:
	"""Replay only the issued Telex glyph identities and origins for one label."""
	label = content.label
	font = telex.raw_font(label.text.size)
	origin = PySide6.QtCore.QPointF(
		label.text.origin.x + content.atom_local_anchor.x,
		label.text.origin.y + content.atom_local_anchor.y,
	)
	return ferrum_qt.canvas.telex_glyph_outline.path_from_runs(runs, origin, font)


#============================================
def _bounds(path: PySide6.QtGui.QPainterPath) -> tuple[float, float, float, float]:
	"""Return exact Qt outline bounds in the same order as Rust InkBoundsV1."""
	rect = path.boundingRect()
	return rect.left(), rect.top(), rect.right(), rect.bottom()


#============================================
def _expected_bounds(bounds: object, anchor: object) -> tuple[float, float, float, float]:
	"""Translate a Rust-issued atom-local rectangle without remeasuring it."""
	return (
		bounds.min_x + anchor.x, bounds.min_y + anchor.y,
		bounds.max_x + anchor.x, bounds.max_y + anchor.y,
	)


#============================================
def _source_positions(cdml: str) -> dict[str, tuple[float, float]]:
	"""Read authored atom coordinates solely to join source bond IDs to issued IDs."""
	root = xml.etree.ElementTree.fromstring(cdml)
	result = {}
	for atom in root.findall(".//{urn:ferrum:cdml}atom"):
		point = atom.find("{urn:ferrum:cdml}point")
		assert point is not None
		result[atom.attrib["id"]] = (float(point.attrib["x"]), float(point.attrib["y"]))
	return result


#============================================
def _atom_batches_by_source_id(observation: object,
		positions: dict[str, tuple[float, float]]) -> dict[str, object]:
	"""Join source atoms to opaque durable IDs through the issued projection facts."""
	projection_atoms = observation.document.projection.molecules[0].atoms
	plan = observation.molecule_plans[0].plan
	result = {}
	for source_id, position in positions.items():
		matches = tuple(
			atom for atom in projection_atoms
			if (atom.position.x, atom.position.y) == position
		)
		assert len(matches) == 1, source_id
		document_object_id = matches[0].document_object_id
		batch = next(
			batch for batch in plan.batches
			if batch.target.document_object_id == document_object_id
		)
		assert type(batch.content) is engine.AtomRenderBatchV1
		result[source_id] = batch
	return result


#============================================
def _projection_for(case: dict[str, object]) -> tuple[object, object, object]:
	"""Issue real V4/V2 observations and install them through the production route."""
	session = engine.DocumentSession.load(case["cdml"])
	snapshot = session.snapshot()
	observation = session.observe_render(snapshot.revision)
	assert type(observation) is engine.RenderObservationV2
	presentation = session.observe_presentation_render_plan_v1(snapshot.revision, snapshot.digest)
	projection = ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
		observation,
		engine.verified_telex_regular(),
		presentation,
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	return observation, projection, engine.verified_telex_regular()


#============================================
def _assert_label_geometry(atom_batches: dict[str, object], telex: object) -> None:
	"""Verify every issued full/core label outline against the exact Rust rectangles."""
	verified_telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex)
	for batch in atom_batches.values():
		content = batch.content
		label = content.label
		assert all(math.isfinite(value) for value in (
			label.full_ink_bounds.min_x, label.full_ink_bounds.min_y,
			label.full_ink_bounds.max_x, label.full_ink_bounds.max_y,
			label.core_element_ink_bounds.min_x, label.core_element_ink_bounds.min_y,
			label.core_element_ink_bounds.max_x, label.core_element_ink_bounds.max_y,
		))
		core_run = label.text.runs[label.core_element_run_index]
		full_path = _label_path(content, verified_telex, label.text.runs)
		core_path = _label_path(content, verified_telex, (core_run,))
		assert all(abs(actual - expected) <= _TOLERANCE for actual, expected in zip(
			_bounds(full_path),
			_expected_bounds(label.full_ink_bounds, content.atom_local_anchor),
			strict=True,
		))
		assert all(abs(actual - expected) <= _TOLERANCE for actual, expected in zip(
			_bounds(core_path),
			_expected_bounds(label.core_element_ink_bounds, content.atom_local_anchor),
			strict=True,
		))


#============================================
def _assert_case_checks(case: dict[str, object], atom_batches: dict[str, object]) -> None:
	"""Check optional corpus semantic facts without a parallel rendering model."""
	checks = case["checks"]
	labels = tuple(batch.content.label for batch in atom_batches.values())
	if checks.get("require_mask"):
		assert any(label.mask is not None for label in labels)
	if "core_run" in checks:
		expected = checks["core_run"]
		assert any(
			label.core_element_run_index == expected["index"]
			and label.text.runs[label.core_element_run_index].text == expected["text"]
			for label in labels
		)
	if "leading_superscript" in checks:
		expected = checks["leading_superscript"]
		assert any(
			label.text.runs[0].text == expected and label.text.runs[0].script == "superscript"
			and label.core_element_run_index == 1 and label.text.runs[1].script == "baseline"
			for label in labels
		)
	if "runs" in checks:
		expected_runs = tuple((value["text"], value["script"]) for value in checks["runs"])
		assert any(
			tuple((run.text, run.script) for run in label.text.runs) == expected_runs
			for label in labels
		)


#============================================
def test_installed_qt_projection_consumes_every_render_alignment_case(
		qapp: object,
		) -> None:
	"""Every successful shared row reaches Qt with issued order, bounds, and clearance."""
	for case in _corpus():
		if case["expected_outcome"] != "render":
			continue
		observation, projection, telex = _projection_for(case)
		try:
			plan = observation.molecule_plans[0].plan
			positions = _source_positions(case["cdml"])
			atom_batches = _atom_batches_by_source_id(observation, positions)
			_assert_label_geometry(atom_batches, telex)
			_assert_case_checks(case, atom_batches)
			assert len(projection.items) == len(plan.batches)
			assert tuple(
				projection.item_targets[item].document_object_id for item in projection.items
			) == tuple(batch.target.document_object_id for batch in plan.batches)
			assert tuple(batch.paint_order for batch in plan.batches) == tuple(sorted(
				batch.paint_order for batch in plan.batches
			))
			for item, batch in zip(projection.items, plan.batches, strict=True):
				assert tuple(command.z for command in item._commands) == tuple(
					operation.operation.z for operation in batch.content.operations
				)
				assert all(left.z < right.z for left, right in zip(item._commands, item._commands[1:]))
			bond_items = tuple(
				item for item, batch in zip(projection.items, plan.batches, strict=True)
				if batch.content.kind == "bond"
			)
			assert bool(bond_items) is case["checks"].get("positive_bond_content", False)
			verified_telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex)
			for bond_item in bond_items:
				for atom_batch in atom_batches.values():
					label_path = _label_path(
						atom_batch.content, verified_telex, atom_batch.content.label.text.runs,
					)
					assert not bond_item.shape().intersects(label_path)
		finally:
			projection.dispose()


#============================================
def test_installed_qt_projection_keeps_refused_alignment_targets_unpainted(
		qapp: object,
		) -> None:
	"""Refusal rows retain the one Rust issue and create no target graphics item."""
	for case in _corpus():
		if case["expected_outcome"] != "unrenderable_target":
			continue
		observation, projection, _telex = _projection_for(case)
		try:
			plan = observation.molecule_plans[0].plan
			offending = next(
			bond for bond in observation.document.projection.molecules[0].bonds
			if bond.source_id == case["offending_bond"]
		)
			issues = tuple(
				issue for issue in plan.issues
				if issue.target.document_object_id == offending.document_object_id
			)
			assert len(plan.issues) == 1
			assert len(issues) == 1
			assert issues[0].kind == "unrenderable_target"
			if case["name"] == "third_label_crossing_refusal":
				assert issues[0].detail == _THIRD_LABEL_DETAIL
			assert all(
				item_target.document_object_id != offending.document_object_id
				for item_target in projection.item_targets.values()
			)
			assert tuple(issue.document_object_id for issue in projection.issues) == tuple(
				issue.target.document_object_id for issue in plan.issues
			)
		finally:
			projection.dispose()
