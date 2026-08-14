"""Behavior checks for the isolated frozen Ferrum render-plan graphics item."""

# Standard Library
import dataclasses
import pathlib

# PIP3 modules
import pytest
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.items.ferrum_plan_item


@dataclasses.dataclass(frozen=True)
class PointV1:
	x: float
	y: float


@dataclasses.dataclass(frozen=True)
class GlyphV1:
	glyph_index: int
	origin: PointV1


@dataclasses.dataclass(frozen=True)
class RunV1:
	text: str
	script: str
	origin: PointV1
	glyphs: tuple[GlyphV1, ...]
	scale: float


@dataclasses.dataclass(frozen=True)
class TextPayloadV1:
	origin: PointV1
	runs: tuple[RunV1, ...]
	face: str
	size: float
	paint: str
	z: int


@dataclasses.dataclass(frozen=True)
class LinePayloadV1:
	start: PointV1
	end: PointV1
	width: float
	paint: str
	z: int


@dataclasses.dataclass(frozen=True)
class MaskPayloadV1:
	origin: PointV1
	width: float
	height: float
	paint: str
	z: int


@dataclasses.dataclass(frozen=True)
class EllipsePayloadV1:
	center: PointV1
	radius_x: float
	radius_y: float
	rotation_degrees: float
	stroke_width: float | None
	stroke_paint: str | None
	fill_paint: str | None
	z: int


@dataclasses.dataclass(frozen=True)
class OperationV1:
	kind: str
	operation: TextPayloadV1 | LinePayloadV1 | MaskPayloadV1 | EllipsePayloadV1


@dataclasses.dataclass(frozen=True)
class RecordIdV1:
	kind: str
	id: str | None


@dataclasses.dataclass(frozen=True)
class TargetV1:
	record_id: RecordIdV1
	source_order: int


@dataclasses.dataclass(frozen=True)
class AtomLocalV1:
	kind: str
	anchor: PointV1


@dataclasses.dataclass(frozen=True)
class SceneV1:
	kind: str


@dataclasses.dataclass(frozen=True)
class BatchFixtureV1:
	target: TargetV1
	coordinate_space: AtomLocalV1 | SceneV1
	operations: tuple[OperationV1, ...]


#============================================
@dataclasses.dataclass(frozen=True)
class RenderProvenanceV1:
	"""Frozen PyO3-shaped plan provenance fixture."""

	revision: int
	digest: str


#============================================
@dataclasses.dataclass(frozen=True)
class RenderPlanV1:
	"""Frozen PyO3-shaped render plan fixture with its owned batches."""

	schema: str
	provenance: RenderProvenanceV1
	batches: tuple[BatchFixtureV1, ...]
	issues: tuple[object, ...]


#============================================
@dataclasses.dataclass(frozen=True)
class MoleculeRenderRootV1:
	"""Frozen PyO3-shaped document-root molecule fixture."""

	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int


#============================================
@dataclasses.dataclass(frozen=True)
class DocumentMoleculeRenderPlanV1:
	"""Frozen PyO3-shaped owner envelope for one render plan."""

	molecule: MoleculeRenderRootV1
	plan: RenderPlanV1


#============================================
@dataclasses.dataclass(frozen=True)
class DocumentSnapshotV1:
	"""Frozen nested authoritative snapshot fixture exposed by PyO3."""

	revision: int
	digest: str


#============================================
@dataclasses.dataclass(frozen=True)
class SessionDocumentObservationV1:
	"""Frozen document observation fixture carried by RenderObservationV1."""

	snapshot: DocumentSnapshotV1


#============================================
@dataclasses.dataclass(frozen=True)
class RenderObservationV1:
	"""Exact PyO3 observation hierarchy used to obtain an immutable render plan."""

	schema: str
	document: SessionDocumentObservationV1
	profile: str
	molecule_plans: tuple[DocumentMoleculeRenderPlanV1, ...]
	plus_renders: tuple[object, ...]
	text_renders: tuple[object, ...]
	issues: tuple[object, ...]
	suppression: str | None


#============================================
def _telex() -> ferrum_qt.canvas.ferrum_telex.FerrumTelex:
	"""Return verified vendored Telex bytes without a system font database."""
	repository = pathlib.Path(__file__).resolve().parents[3]
	path = repository / "packages/ferrum-rust/crates/render/assets/fonts/Telex-Regular.ttf"
	resource = ferrum_qt.canvas.ferrum_telex.FerrumTelex(path.read_bytes())
	return resource


#============================================
def _observation_and_batches() -> tuple[RenderObservationV1, BatchFixtureV1, BatchFixtureV1]:
	"""Return exact frozen PyO3 observation, atom batch, and bond batch fixtures."""
	atom = BatchFixtureV1(
		TargetV1(RecordIdV1("Atom", "a1"), 2), AtomLocalV1("atom_local", PointV1(10.0, 20.0)),
		(OperationV1("mask", MaskPayloadV1(PointV1(1.0, 2.0), 12.0, 9.0, "ffffff", 0)),
		OperationV1("text", TextPayloadV1(
			PointV1(2.0, 3.0), (RunV1("C", "baseline", PointV1(1.0, 2.0),
			(GlyphV1(13, PointV1(0.0, 0.0)),), 1.0),),
			"ferrum-telex-regular-v1", 20.0, "112233", 1,
		))),
	)
	line = BatchFixtureV1(
		TargetV1(RecordIdV1("Bond", "b1"), 5), SceneV1("scene"),
		(OperationV1("line", LinePayloadV1(PointV1(2.0, 7.0), PointV1(42.0, 7.0), 2.0, "aa3300", 0)),),
	)
	digest = "1" * 64
	plan = RenderPlanV1("ferrum-render-plan-v1", RenderProvenanceV1(7, digest), (atom, line), ())
	entry = DocumentMoleculeRenderPlanV1(
		MoleculeRenderRootV1("molecule-1", "ferrum-projection-local-v1/0", "m1", 0),
		plan,
	)
	observation = RenderObservationV1(
		"ferrum-render-observation-v1",
		SessionDocumentObservationV1(DocumentSnapshotV1(7, digest)),
		"ferrum-depiction-profile-v1", (entry,), (), (), (), None,
	)
	return observation, atom, line


#============================================
def test_canonical_atom_batch_uses_exact_glyph_and_mask_geometry(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Atom-local mask and text paths apply their anchor and supplied glyph origins once."""
	observation, atom_batch, unused_line_batch = _observation_and_batches()
	plan = observation.molecule_plans[0].plan
	item = ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(plan, 0, _telex())
	bounds = item.boundingRect()
	assert bounds.left() == 10.0
	assert bounds.top() < 10.0
	assert item.target() is atom_batch.target.record_id


#============================================
def test_atom_local_ellipse_uses_explicit_geometry_without_metrics_or_pixel_gate(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A Rust ellipse contributes its translated fill and outline to cached hit geometry."""
	observation, atom_batch, line_batch = _observation_and_batches()
	ellipse = OperationV1("ellipse", EllipsePayloadV1(
		PointV1(5.0, 6.0), 4.0, 2.0, 30.0, 1.0, "112233", "445566", 2,
	))
	marked_atom = dataclasses.replace(
		atom_batch, operations=(ellipse,),
	)
	plan = dataclasses.replace(
		observation.molecule_plans[0].plan, batches=(marked_atom, line_batch),
	)
	item = ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(
		plan, 0, _telex(),
	)

	assert item.shape().contains(PySide6.QtCore.QPointF(15.0, 26.0))
	assert item.boundingRect().contains(PySide6.QtCore.QPointF(15.0, 26.0))


#============================================
def test_canonical_scene_line_paints_explicit_rgb24_and_is_immovable(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A scene line retains Rust color/width semantics and Qt-local selection only."""
	observation, unused_atom_batch, line_batch = _observation_and_batches()
	plan = observation.molecule_plans[0].plan
	item = ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(plan, 1, _telex())
	flags = item.flags()
	assert flags & PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsSelectable
	assert not flags & PySide6.QtWidgets.QGraphicsItem.GraphicsItemFlag.ItemIsMovable
	image = PySide6.QtGui.QImage(48, 16, PySide6.QtGui.QImage.Format.Format_ARGB32_Premultiplied)
	image.fill(PySide6.QtGui.QColor("white"))
	painter = PySide6.QtGui.QPainter(image)
	item.paint(painter, PySide6.QtWidgets.QStyleOptionGraphicsItem())
	painter.end()
	assert image.pixelColor(20, 7) == PySide6.QtGui.QColor("#aa3300")
	item.setSelected(True)
	assert item.isSelected()


#============================================
def test_batch_index_rejects_bool_and_outside_the_owned_plan(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Only a non-bool in-range index can select a batch from the supplied plan."""
	observation, unused_atom_batch, unused_line_batch = _observation_and_batches()
	plan = observation.molecule_plans[0].plan
	for index in (True, -1, len(plan.batches)):
		with pytest.raises(ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanError):
			ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(plan, index, _telex())


#============================================
def test_public_constructor_rejects_frozen_duck_fixture_and_has_no_validator_argument(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Only the private fixture factory can admit local doubles before a wheel exists."""
	observation, unused_atom_batch, unused_line_batch = _observation_and_batches()
	plan = observation.molecule_plans[0].plan
	with pytest.raises(ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanError):
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem(plan, 0, _telex())
	with pytest.raises(TypeError):
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem(
			plan, 0, _telex(), validator=object(),
		)


#============================================
def test_same_target_from_another_plan_cannot_replace_owned_batch(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The plan index selects its own geometry even if another plan has that target."""
	observation, atom_batch, line_batch = _observation_and_batches()
	plan = observation.molecule_plans[0].plan
	other_line = dataclasses.replace(
		line_batch,
		operations=(OperationV1("line", LinePayloadV1(
			PointV1(2.0, 7.0), PointV1(42.0, 7.0), 2.0, "0066cc", 0,
		)),),
	)
	other_plan = RenderPlanV1(plan.schema, plan.provenance, (atom_batch, other_line), ())
	item = ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(plan, 1, _telex())
	image = PySide6.QtGui.QImage(48, 16, PySide6.QtGui.QImage.Format.Format_ARGB32_Premultiplied)
	image.fill(PySide6.QtGui.QColor("white"))
	painter = PySide6.QtGui.QPainter(image)
	item.paint(painter, PySide6.QtWidgets.QStyleOptionGraphicsItem())
	painter.end()
	assert image.pixelColor(20, 7) == PySide6.QtGui.QColor("#aa3300")
	other_item = ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(other_plan, 1, _telex())
	other_image = PySide6.QtGui.QImage(48, 16, PySide6.QtGui.QImage.Format.Format_ARGB32_Premultiplied)
	other_image.fill(PySide6.QtGui.QColor("white"))
	other_painter = PySide6.QtGui.QPainter(other_image)
	other_item.paint(other_painter, PySide6.QtWidgets.QStyleOptionGraphicsItem())
	other_painter.end()
	assert other_image.pixelColor(20, 7) == PySide6.QtGui.QColor("#0066cc")
	assert other_plan.batches[1].target == plan.batches[1].target


#============================================
def test_invalid_telex_bytes_fail_before_plan_item_allocation(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A mismatched resource is rejected before it can become drawable state."""
	with pytest.raises(ferrum_qt.canvas.ferrum_telex.FerrumTelexError):
		ferrum_qt.canvas.ferrum_telex.FerrumTelex(b"not the verified Telex resource")


#============================================
def test_unknown_operation_preserves_existing_scene_item(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A rejected detached batch leaves a previous complete scene projection intact."""
	observation, atom_batch, line_batch = _observation_and_batches()
	plan = observation.molecule_plans[0].plan
	scene = PySide6.QtWidgets.QGraphicsScene()
	previous = ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(plan, 1, _telex())
	scene.addItem(previous)
	invalid = dataclasses.replace(atom_batch, operations=(OperationV1("circle", atom_batch.operations[0].operation),))
	invalid_plan = dataclasses.replace(plan, batches=(invalid, line_batch))
	with pytest.raises(ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanError):
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(invalid_plan, 0, _telex())
	assert previous.scene() is scene
	scene.removeItem(previous)


#============================================
def test_missing_or_unknown_exact_glyph_fails_before_paint(qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The cached constructor rejects bad Rust glyph IDs instead of substituting text."""
	observation, atom_batch, unused_line_batch = _observation_and_batches()
	plan = observation.molecule_plans[0].plan
	text = atom_batch.operations[1].operation
	assert isinstance(text, TextPayloadV1)
	bad_run = dataclasses.replace(text.runs[0], glyphs=(GlyphV1(0, PointV1(0.0, 0.0)),))
	bad_text = dataclasses.replace(text, runs=(bad_run,))
	bad_batch = dataclasses.replace(atom_batch, operations=(atom_batch.operations[0], OperationV1("text", bad_text)))
	bad_plan = dataclasses.replace(plan, batches=(bad_batch, unused_line_batch))
	with pytest.raises(ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanError):
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture(bad_plan, 0, _telex())
