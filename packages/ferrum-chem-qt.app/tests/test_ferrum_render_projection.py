"""Behavioral coverage for the isolated frozen Ferrum render projection seam."""

# Standard Library
import dataclasses
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import pytest
import shiboken6

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.canvas.ferrum_telex


@dataclasses.dataclass(frozen=True, slots=True)
class _Point:
	x: float
	y: float


@dataclasses.dataclass(frozen=True, slots=True)
class _Record:
	kind: str
	id: str | None


@dataclasses.dataclass(frozen=True, slots=True)
class _Target:
	record_id: _Record
	source_order: int


@dataclasses.dataclass(frozen=True, slots=True)
class _SceneSpace:
	kind: str = "scene"


@dataclasses.dataclass(frozen=True, slots=True)
class _Line:
	start: _Point
	end: _Point
	width: float = 1.0
	paint: str = "112233"
	z: int = 1


@dataclasses.dataclass(frozen=True, slots=True)
class _Operation:
	kind: str
	operation: _Line


@dataclasses.dataclass(frozen=True, slots=True)
class _Batch:
	target: _Target
	coordinate_space: _SceneSpace
	operations: tuple[_Operation, ...]
	display_layer: str = "ordinary"


@dataclasses.dataclass(frozen=True, slots=True)
class _Provenance:
	revision: int
	digest: str


@dataclasses.dataclass(frozen=True, slots=True)
class _Plan:
	schema: str
	provenance: _Provenance
	batches: tuple[_Batch, ...]
	issues: tuple[object, ...] = ()


@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculeRoot:
	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int


@dataclasses.dataclass(frozen=True, slots=True)
class _MoleculePlan:
	molecule: _MoleculeRoot
	plan: _Plan


@dataclasses.dataclass(frozen=True, slots=True)
class _Snapshot:
	revision: int
	digest: str


@dataclasses.dataclass(frozen=True, slots=True)
class _PaperPage:
	scene_left: float = 0.0
	scene_top: float = 0.0
	scene_right: float = 595.0
	scene_bottom: float = 842.0
	issue: object | None = None


@dataclasses.dataclass(frozen=True, slots=True)
class _PaperLayout:
	schema: str
	revision: int
	digest: str
	page: _PaperPage


@dataclasses.dataclass(frozen=True, slots=True)
class _Projection:
	revision: int
	digest: str
	paper_layout: _PaperLayout


@dataclasses.dataclass(frozen=True, slots=True)
class _Document:
	snapshot: _Snapshot
	projection: _Projection


@dataclasses.dataclass(frozen=True, slots=True)
class _Issue:
	code: str
	target: str
	detail: str


@dataclasses.dataclass(frozen=True, slots=True)
class _PlanIssue:
	target: _Target
	kind: str
	detail: str


@dataclasses.dataclass(frozen=True, slots=True)
class _Observation:
	schema: str
	document: _Document
	molecule_plans: tuple[_MoleculePlan, ...]
	plus_renders: tuple[object, ...] = ()
	text_renders: tuple[object, ...] = ()
	issues: tuple[_Issue, ...] = ()
	suppression: str | None = None


#============================================
def _test_observation_validator(_observation: object) -> None:
	"""Inject the frozen fixture seam while production requires a PyO3 class."""


#============================================
def _telex() -> ferrum_qt.canvas.ferrum_telex.FerrumTelex:
	"""Return the verified font bytes required by the Ferrum painter seam."""
	repository = pathlib.Path(__file__).resolve().parents[3]
	data = (repository / "packages/ferrum-rust/crates/render/assets/fonts/Telex-Regular.ttf").read_bytes()
	return ferrum_qt.canvas.ferrum_telex.FerrumTelex(data)


#============================================
def _observation(identifier: str | None = "bond-1", revision: int = 5) -> _Observation:
	"""Return one compact, frozen PyO3-shaped complete render observation."""
	digest = "a" * 64
	target = _Target(_Record("Bond", identifier), 2)
	batch = _Batch(target, _SceneSpace(), (_Operation("line", _Line(_Point(1.0, 2.0), _Point(8.0, 2.0))),))
	plan = _Plan("ferrum-render-plan-v2", _Provenance(revision, digest), (batch,))
	molecule = _MoleculeRoot("molecule-1", "ferrum-projection-local-v1/0", "m1", 1)
	paper = _PaperLayout("ferrum-document-paper-layout-v1", revision, digest, _PaperPage())
	document = _Document(_Snapshot(revision, digest), _Projection(revision, digest, paper))
	return _Observation("ferrum-document-render-observation-v1", document, (_MoleculePlan(molecule, plan),))


#============================================
def _latch(observation: _Observation, generation: int = 0) -> ferrum_qt.canvas.ferrum_render_projection.RenderProjectionLatch:
	"""Bind one delivery to the exact observation facts received by a tab."""
	return ferrum_qt.canvas.ferrum_render_projection.RenderProjectionLatch(
		observation.document.snapshot.revision,
		observation.document.snapshot.digest,
		generation,
	)


#============================================
def test_complete_observation_builds_detached_scene_and_durable_map(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A valid observation has a complete candidate scene before a view sees it."""
	del qapp
	projection = ferrum_qt.canvas.ferrum_render_projection._build_fixture_render_projection(
		_observation(), _telex(), _test_observation_validator,
	)
	item = projection.durable_items[("bond", "bond-1")]
	root = item.parentItem()
	assert item.scene() is projection.scene and item.zValue() == 2.0
	assert projection.molecule_roots[root].source_order == 1 and root.zValue() == 1.0
	assert projection.paper.rect() == projection.scene.sceneRect()
	projection.dispose()


#============================================
def test_projection_disposal_detaches_roots_and_retires_its_scene_once(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A disposable projection owns its detached scene through terminal deletion."""
	projection = ferrum_qt.canvas.ferrum_render_projection._build_fixture_render_projection(
		_observation(), _telex(), _test_observation_validator,
	)
	scene = projection.scene
	assert scene.items()
	projection.dispose()
	assert not scene.items()
	projection.dispose()
	for _pass in range(4):
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.DeferredDelete,
		)
		qapp.processEvents()
		if not shiboken6.isValid(scene):
			break
	assert not shiboken6.isValid(scene)


#============================================
def test_source_owned_haworth_layer_projects_above_an_ordinary_bond(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A received Haworth layer controls child paint order without changing identity."""
	del qapp
	ordinary = _observation("ordinary")
	front = _observation("front")
	front_batch = dataclasses.replace(
		front.molecule_plans[0].plan.batches[0], display_layer="haworth_front_wedge",
	)
	front_plan = dataclasses.replace(front.molecule_plans[0].plan, batches=(front_batch,))
	front_root = dataclasses.replace(
		front.molecule_plans[0].molecule,
		id="molecule-2", projection_key="ferrum-projection-local-v1/1", source_id="m2", source_order=3,
	)
	projection = ferrum_qt.canvas.ferrum_render_projection._build_fixture_render_projection(
		dataclasses.replace(
			ordinary,
			molecule_plans=(
				ordinary.molecule_plans[0],
				_MoleculePlan(front_root, front_plan),
			),
		),
		_telex(), _test_observation_validator,
	)
	assert projection.durable_items[("bond", "front")].zValue() > projection.durable_items[("bond", "ordinary")].zValue()


#============================================
def test_molecule_roots_keep_document_order_separate_from_child_order(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Two molecule-local order-zero children remain under distinct root positions."""
	del qapp
	observation = _observation()
	first = observation.molecule_plans[0]
	second_target = _Target(_Record("Bond", "bond-2"), 2)
	second_batch = dataclasses.replace(first.plan.batches[0], target=second_target)
	second_plan = dataclasses.replace(first.plan, batches=(second_batch,))
	second_root = _MoleculeRoot(
		"molecule-2", "ferrum-projection-local-v1/4", "m2", 4,
	)
	two_molecules = dataclasses.replace(
		observation,
		molecule_plans=(first, _MoleculePlan(second_root, second_plan)),
	)
	projection = ferrum_qt.canvas.ferrum_render_projection._build_fixture_render_projection(
		two_molecules, _telex(), _test_observation_validator,
	)
	root_z = {
		facts.projection_key: root.zValue()
		for root, facts in projection.molecule_roots.items()
	}
	child_z = {
		target.identifier: (item.zValue(), item.parentItem().zValue())
		for item, target in projection.item_targets.items()
	}
	assert root_z == {"ferrum-projection-local-v1/0": 1.0, "ferrum-projection-local-v1/4": 4.0}
	assert child_z == {"bond-1": (2.0, 1.0), "bond-2": (2.0, 4.0)}
	projection.dispose()


#============================================
def test_rejected_candidate_preserves_installed_projection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Suppression and provenance errors cannot replace the prior complete scene."""
	del qapp
	view = PySide6.QtWidgets.QGraphicsView()
	controller = ferrum_qt.canvas.ferrum_render_projection._build_fixture_controller(
		view, _telex(), _test_observation_validator,
	)
	good = _observation()
	assert controller.replace(good, _latch(good))
	prior = controller.projection
	suppressed = dataclasses.replace(good, suppression="invalid_presentation_facts")
	assert not controller.replace(suppressed, _latch(suppressed))
	assert controller.projection is prior and view.scene() is prior.scene
	controller.dispose()


#============================================
def test_durable_selection_survives_accepted_replacement(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Selection restores through Rust identity, not a retired item wrapper."""
	del qapp
	view = PySide6.QtWidgets.QGraphicsView()
	controller = ferrum_qt.canvas.ferrum_render_projection._build_fixture_controller(
		view, _telex(), _test_observation_validator,
	)
	first = _observation()
	assert controller.replace(first, _latch(first))
	controller.projection.select_durable((("bond", "bond-1"),))
	second = _observation(revision=6)
	assert controller.replace(second, _latch(second))
	assert controller.projection.selected_durable_targets()[0].identifier == "bond-1"
	controller.dispose()


#============================================
def test_idless_target_never_becomes_mutation_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A local graphics target remains selectable but is absent from durable output."""
	del qapp
	projection = ferrum_qt.canvas.ferrum_render_projection._build_fixture_render_projection(
		_observation(None), _telex(), _test_observation_validator,
	)
	item = next(iter(projection.local_items.values()))
	item.setSelected(True)
	assert not projection.selected_durable_targets() and projection.selected_targets()[0].identifier is None
	projection.dispose()


#============================================
def test_explicit_issue_is_visible_without_a_substitute_item(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A Rust issue stays inspectable without Qt inventing fallback artwork."""
	del qapp
	observation = dataclasses.replace(_observation(), molecule_plans=(), issues=(
		_Issue("unsupported_feature", "bond-1", "aromatic bond"),
	))
	projection = ferrum_qt.canvas.ferrum_render_projection._build_fixture_render_projection(
		observation, _telex(), _test_observation_validator,
	)
	assert projection.issues[0].kind == "unsupported_feature" and not projection.items


#============================================
def test_plan_issue_is_visible_in_plan_order_without_a_graphics_item(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A target exclusion remains a Rust diagnostic rather than a Qt substitute."""
	del qapp
	observation = _observation()
	entry = observation.molecule_plans[0]
	issue_target = _Target(_Record("Atom", "atom-1"), 3)
	with_issue = dataclasses.replace(entry.plan, issues=(
		_PlanIssue(issue_target, "unsupported_feature", "atom label unsupported"),
	))
	projection = ferrum_qt.canvas.ferrum_render_projection._build_fixture_render_projection(
		dataclasses.replace(
			observation,
			molecule_plans=(dataclasses.replace(entry, plan=with_issue),),
		), _telex(),
		_test_observation_validator,
	)
	assert projection.issues[0].target.identifier == "atom-1" and len(projection.items) == 1


#============================================
def test_mutable_sequences_and_invalid_plan_issues_preserve_prior_scene(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Only frozen PyO3 tuples and valid target exclusions may enter a replacement."""
	del qapp
	view = PySide6.QtWidgets.QGraphicsView()
	controller = ferrum_qt.canvas.ferrum_render_projection._build_fixture_controller(
		view, _telex(), _test_observation_validator,
	)
	good = _observation()
	assert controller.replace(good, _latch(good))
	prior = controller.projection
	mutable = dataclasses.replace(good, molecule_plans=list(good.molecule_plans))
	entry = good.molecule_plans[0]
	bad_issue = dataclasses.replace(entry.plan, issues=(
		_PlanIssue(entry.plan.batches[0].target, "unsupported_feature", "duplicate"),
	))
	assert not controller.replace(mutable, _latch(good))
	assert not controller.replace(
		dataclasses.replace(
			good, molecule_plans=(dataclasses.replace(entry, plan=bad_issue),),
		), _latch(good),
	)
	assert controller.projection is prior and view.scene() is prior.scene
	controller.dispose()


#============================================
def test_same_revision_different_plan_digest_preserves_prior_scene(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A plan must match the accepted snapshot digest as well as its revision."""
	del qapp
	view = PySide6.QtWidgets.QGraphicsView()
	controller = ferrum_qt.canvas.ferrum_render_projection._build_fixture_controller(
		view, _telex(), _test_observation_validator,
	)
	good = _observation()
	assert controller.replace(good, _latch(good))
	prior = controller.projection
	entry = good.molecule_plans[0]
	wrong_digest = dataclasses.replace(entry.plan, provenance=_Provenance(5, "b" * 64))
	assert not controller.replace(
		dataclasses.replace(
			good,
			molecule_plans=(dataclasses.replace(entry, plan=wrong_digest),),
		), _latch(good),
	)
	assert controller.projection is prior and view.scene() is prior.scene
	controller.dispose()


#============================================
def test_production_entrance_rejects_fixture_impostor(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Production rejects fixture observations and local Telex objects together."""
	del qapp
	with pytest.raises(ferrum_qt.canvas.ferrum_telex.FerrumTelexError):
		ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
			_observation(), _telex(), object(),
		)


#============================================
def test_public_entrances_expose_no_fixture_validator_argument(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Callers cannot turn a production entrance into the fixture-only seam."""
	del qapp
	with pytest.raises(TypeError):
		ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
			_observation(), _telex(), object(), validator=_test_observation_validator,
		)
	with pytest.raises(TypeError):
		ferrum_qt.canvas.ferrum_render_projection.FerrumRenderProjectionController(
			PySide6.QtWidgets.QGraphicsView(), _telex(), _test_observation_validator,
		)


#============================================
def test_stale_generation_and_disposal_reject_delivery(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Closed or invalidated tab generations cannot mutate a graphics view."""
	del qapp
	view = PySide6.QtWidgets.QGraphicsView()
	controller = ferrum_qt.canvas.ferrum_render_projection._build_fixture_controller(
		view, _telex(), _test_observation_validator,
	)
	observation = _observation()
	controller.invalidate_delivery()
	assert not controller.replace(observation, _latch(observation, 0))
	assert controller.projection is None
	controller.dispose()
	assert not controller.replace(observation, _latch(observation, controller.generation))
