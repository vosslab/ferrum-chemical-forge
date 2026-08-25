"""Build and install frozen Ferrum render observations without document authority.

This is deliberately an isolated canvas seam.  It accepts only copied PyO3
DTOs, owns only disposable scenes and graphics items, and never opens CDML or
calls a document session.
"""

# Standard Library
import collections.abc
import dataclasses
import re

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.canvas.ferrum_presentation_render_plan
import ferrum_qt.canvas.ferrum_presentation_target
from ferrum_qt.canvas.ferrum_render_target import RenderTargetKey
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.items.ferrum_plan_item
import ferrum_qt.canvas.items.ferrum_paper_item
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.canvas.items.ferrum_text_item


_OBSERVATION_SCHEMA = "ferrum-document-render-observation-v1"
_PLAN_SCHEMA = "ferrum-render-plan-v2"
_PAPER_SCHEMA = "ferrum-document-paper-layout-v1"
_DIGEST = re.compile(r"^[0-9a-f]{64}$")
_U32_RANGE = range(2**32)
_PRESENTATION_KINDS = frozenset((
	"arrow", "plus", "text", "polyline", "rectangle", "square", "oval", "circle",
	"polygon",
))
_STRUCTURAL_KIND_MAP = {
	"Atom": "atom",
	"Bond": "bond",
	"Group": "compact_group",
}
ObservationValidator = collections.abc.Callable[[object], None]
PlanItemFactory = collections.abc.Callable[
	[object, int, object, PySide6.QtWidgets.QGraphicsItem],
	PySide6.QtWidgets.QGraphicsItem,
]
PresentationSceneFactory = collections.abc.Callable[
	[object],
	ferrum_qt.canvas.ferrum_presentation_render_plan.FerrumPresentationScene,
]
PaperItemFactory = collections.abc.Callable[
	[object], ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem,
]


#============================================
class FerrumRenderProjectionError(ValueError):
	"""Raised when a copied render observation violates the V1 contract."""


@dataclasses.dataclass(frozen=True, slots=True)
class MoleculeRootKey:
	"""Detached document-root identity and order for one molecule plan."""

	identifier: str | None
	projection_key: str
	source_id: str | None
	source_order: int


@dataclasses.dataclass(frozen=True, slots=True)
class RenderIssue:
	"""Visible Rust diagnostic that deliberately has no graphics item."""

	kind: str
	target: RenderTargetKey | str
	detail: str


@dataclasses.dataclass(frozen=True, slots=True)
class RenderProjectionLatch:
	"""Session-owned generation and exact observation provenance at delivery."""

	revision: int
	digest: str
	generation: int


@dataclasses.dataclass(slots=True)
class FerrumRenderProjection:
	"""A complete detached scene and its immutable selection/issue state."""

	scene: PySide6.QtWidgets.QGraphicsScene
	revision: int
	digest: str
	paper: ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem
	presentation: (
		ferrum_qt.canvas.ferrum_presentation_render_plan.FerrumPresentationScene | None
	)
	roots: tuple[PySide6.QtWidgets.QGraphicsItem, ...]
	molecule_roots: dict[PySide6.QtWidgets.QGraphicsItemGroup, MoleculeRootKey]
	items: tuple[PySide6.QtWidgets.QGraphicsItem, ...]
	item_targets: dict[PySide6.QtWidgets.QGraphicsItem, RenderTargetKey]
	durable_items: dict[tuple[str, str], PySide6.QtWidgets.QGraphicsItem]
	local_items: dict[RenderTargetKey, PySide6.QtWidgets.QGraphicsItem]
	issues: tuple[RenderIssue, ...]
	_disposed: bool = dataclasses.field(default=False, init=False, repr=False)

	#============================================
	def selected_targets(self) -> tuple[RenderTargetKey, ...]:
		"""Return selected current targets without promoting local keys to IDs."""
		selected = ferrum_qt.canvas.graphics_retirement.selected_items_from_captured_scene(
			self.scene,
		)
		result = tuple(
			self.item_targets[item] for item in self.items
			if item in selected and item.isSelected()
		)
		return result

	#============================================
	def selected_durable_targets(self) -> tuple[RenderTargetKey, ...]:
		"""Return only targets safe for a Rust mutation request."""
		result = tuple(target for target in self.selected_targets() if target.is_durable)
		return result

	#============================================
	def select_durable(self, targets: tuple[tuple[str, str], ...]) -> None:
		"""Restore selection only through Rust-issued durable identities."""
		requested = frozenset(targets)
		for item, target in self.item_targets.items():
			item.setSelected(
				target.is_durable and target.durable_selection_key() in requested,
			)

	#============================================
	def dispose(self) -> None:
		"""Terminally retire this projection's graphics and detached scene once."""
		if self._disposed:
			return
		coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
		coordinator.retire_scene_projection_items(self.scene, list(self.roots))
		coordinator.raise_if_callback_failed("Ferrum render projection retirement failed")
		self.scene.deleteLater()
		self._disposed = True


#============================================
class FerrumRenderProjectionController:
	"""Atomically swap complete validated render scenes into one graphics view."""

	#============================================
	def __init__(self, view: PySide6.QtWidgets.QGraphicsView,
			telex_resource: object) -> None:
		"""Bind to one UI-thread view without taking document/session ownership."""
		telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
		self._initialize(
			view, telex_resource, telex, _require_pyo3_observation,
			ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem,
			ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem,
			lambda plan: _build_presentation_scene(plan, telex_resource),
		)

	#============================================
	def _initialize(self, view: PySide6.QtWidgets.QGraphicsView,
			telex_resource: object, telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
			validator: ObservationValidator, item_factory: PlanItemFactory,
			paper_factory: PaperItemFactory,
			presentation_factory: PresentationSceneFactory | None) -> None:
		"""Initialize the production controller or the private fixture seam."""
		if not isinstance(view, PySide6.QtWidgets.QGraphicsView):
			raise TypeError("Ferrum render projection controller requires a graphics view")
		if not isinstance(telex, ferrum_qt.canvas.ferrum_telex.FerrumTelex):
			raise TypeError("Ferrum render projection controller requires verified Telex")
		self._view = view
		self._telex_resource = telex_resource
		self._telex = telex
		self._validator = validator
		self._item_factory = item_factory
		self._paper_factory = paper_factory
		self._presentation_factory = presentation_factory
		self._generation = 0
		self._disposed = False
		self.projection: FerrumRenderProjection | None = None

	#============================================
	@property
	def generation(self) -> int:
		"""Return the current delivery generation for the owning tab session."""
		return self._generation

	#============================================
	def replace(self, observation: object, latch: RenderProjectionLatch,
			presentation_plan: object | None = None) -> bool:
		"""Build then install one current observation without disturbing prior state."""
		if self._disposed or latch.generation != self._generation:
			return False
		try:
			prepared = _build_render_projection(
				observation, self._telex_resource, self._telex, self._validator,
				self._item_factory, self._paper_factory, self._presentation_factory,
				presentation_plan,
			)
		except FerrumRenderProjectionError:
			return False
		if not self._latch_matches(prepared, latch):
			prepared.dispose()
			return False
		previous = self.projection
		selected = () if previous is None else tuple(
			_durable_target_key(target) for target in previous.selected_durable_targets()
		)
		try:
			self._view.setScene(prepared.scene)
		except RuntimeError:
			prepared.dispose()
			return False
		self.projection = prepared
		prepared.select_durable(selected)
		if previous is not None:
			previous.dispose()
		return True

	#============================================
	def invalidate_delivery(self) -> int:
		"""Make any already-captured worker or callback result stale."""
		self._generation += 1
		return self._generation

	#============================================
	def dispose(self) -> None:
		"""Invalidate delivery and retire the installed disposable projection once."""
		if self._disposed:
			return
		self._disposed = True
		self.invalidate_delivery()
		projection = self.projection
		self.projection = None
		if projection is None:
			return
		if self._view.scene() is projection.scene:
			self._view.setScene(None)
		projection.dispose()

	#============================================
	def _latch_matches(
			self, projection: FerrumRenderProjection, latch: RenderProjectionLatch,
			) -> bool:
		"""Require the still-current session generation and exact provenance."""
		return (
			not self._disposed
			and latch.generation == self._generation
			and projection.revision == latch.revision
			and projection.digest == latch.digest
		)


#============================================
def _build_fixture_controller(view: PySide6.QtWidgets.QGraphicsView,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		validator: ObservationValidator) -> FerrumRenderProjectionController:
	"""Construct a controller for focused DTO fixtures outside production routing."""
	controller = object.__new__(FerrumRenderProjectionController)
	controller._initialize(
		view, telex, telex, validator,
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture,
		ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem._from_fixture, None,
	)
	return controller


#============================================
def build_render_projection(observation: object, telex_resource: object,
		presentation_plan: object) -> FerrumRenderProjection:
	"""Validate one whole observation and return a fully populated detached scene."""
	telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
	return _build_render_projection(
		observation, telex_resource, telex, _require_pyo3_observation,
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem,
		ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem,
		lambda plan: _build_presentation_scene(plan, telex_resource), presentation_plan,
	)


#============================================
def _build_fixture_render_projection(observation: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		validator: ObservationValidator,
		presentation_factory: PresentationSceneFactory | None = None,
		) -> FerrumRenderProjection:
	"""Build fixture DTOs only through an explicitly private test seam."""
	return _build_render_projection(
		observation, telex, telex, validator,
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem._from_fixture,
		ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem._from_fixture,
		presentation_factory,
	)


#============================================
def _build_render_projection(observation: object, telex_resource: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		validator: ObservationValidator, item_factory: PlanItemFactory,
		paper_factory: PaperItemFactory,
		presentation_factory: PresentationSceneFactory | None,
		presentation_plan: object | None = None) -> FerrumRenderProjection:
	"""Build a validated observation after its caller selected the entry contract."""
	if not isinstance(telex, ferrum_qt.canvas.ferrum_telex.FerrumTelex):
		raise FerrumRenderProjectionError("render projection requires verified Telex")
	revision, digest, paper_layout, plans, plus_renders, text_renders, issues = (
		_validate_observation(
			observation, validator,
		)
	)
	if presentation_factory is not None:
		plus_renders = ()
		text_renders = ()
	scene = PySide6.QtWidgets.QGraphicsScene()
	paper: ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem | None = None
	roots: list[PySide6.QtWidgets.QGraphicsItem] = []
	molecule_roots: dict[PySide6.QtWidgets.QGraphicsItemGroup, MoleculeRootKey] = {}
	items: list[PySide6.QtWidgets.QGraphicsItem] = []
	item_targets: dict[PySide6.QtWidgets.QGraphicsItem, RenderTargetKey] = {}
	durable_items: dict[tuple[str, str], PySide6.QtWidgets.QGraphicsItem] = {}
	local_items: dict[RenderTargetKey, PySide6.QtWidgets.QGraphicsItem] = {}
	all_issues: list[RenderIssue] = []
	presentation = None
	last_root_order = -1
	document_root_orders: set[int] = set()
	seen_projection_keys: set[str] = set()
	seen_molecule_ids: set[str] = set()
	try:
		paper = paper_factory(paper_layout)
		scene.addItem(paper)
		scene.setSceneRect(paper.rect())
		roots.append(paper)
		for plan_entry in plans:
			molecule = _molecule_root(getattr(plan_entry, "molecule", None))
			if molecule.source_order <= last_root_order:
				raise FerrumRenderProjectionError("molecule plans are not in document root order")
			if molecule.projection_key in seen_projection_keys:
				raise FerrumRenderProjectionError("duplicate molecule projection key")
			if molecule.identifier is not None and molecule.identifier in seen_molecule_ids:
				raise FerrumRenderProjectionError("duplicate durable molecule identity")
			last_root_order = molecule.source_order
			document_root_orders.add(molecule.source_order)
			seen_projection_keys.add(molecule.projection_key)
			if molecule.identifier is not None:
				seen_molecule_ids.add(molecule.identifier)
			root, _plan_items, plan_issues = _build_plan(
				scene, plan_entry, molecule, revision, digest, telex_resource, item_factory,
			)
			roots.append(root)
			molecule_roots[root] = molecule
			for item, target in _plan_items:
				if target in local_items:
					raise FerrumRenderProjectionError("duplicate render target")
				if target.is_durable:
					durable_key = _durable_target_key(target)
					if durable_key in durable_items:
						raise FerrumRenderProjectionError("duplicate durable render target")
					durable_items[durable_key] = item
				local_items[target] = item
				item_targets[item] = target
				items.append(item)
			all_issues.extend(plan_issues)
		if presentation_factory is not None:
			if presentation_plan is None:
				raise FerrumRenderProjectionError("renderer presentation plan is required")
			presentation = presentation_factory(presentation_plan)
			if presentation.revision != revision or presentation.digest != digest:
				raise FerrumRenderProjectionError(
					"presentation scene provenance differs from render observation",
				)
			if len(presentation.roots) != len(presentation.items):
				raise FerrumRenderProjectionError("presentation root ownership is incomplete")
			for root, item in zip(presentation.roots, presentation.items, strict=True):
				order = item.target.source_order
				if order in document_root_orders:
					raise FerrumRenderProjectionError("duplicate document root source order")
				document_root_orders.add(order)
				root.setZValue(float(order))
				scene.addItem(root)
				roots.append(root)
				target = _presentation_target(item.target)
				if target in local_items:
					raise FerrumRenderProjectionError("duplicate presentation render target")
				if target.is_durable:
					durable_key = _durable_target_key(target)
					if durable_key in durable_items:
						raise FerrumRenderProjectionError("duplicate durable presentation target")
					durable_items[durable_key] = item
				local_items[target] = item
				item_targets[item] = target
				items.append(item)
		last_plus_order = -1
		for plus in plus_renders:
			item = ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem._from_observation(
				plus, telex,
			)
			target = _presentation_target(item.target)
			if target.source_order <= last_plus_order:
				raise FerrumRenderProjectionError("plus renders are not source ordered")
			if target.source_order in document_root_orders:
				raise FerrumRenderProjectionError("duplicate document root source order")
			if target in local_items:
				raise FerrumRenderProjectionError("duplicate render target")
			last_plus_order = target.source_order
			document_root_orders.add(target.source_order)
			if target.is_durable:
				durable_key = _durable_target_key(target)
				if durable_key in durable_items:
					raise FerrumRenderProjectionError("duplicate durable render target")
				durable_items[durable_key] = item
			local_items[target] = item
			item_targets[item] = target
			items.append(item)
			scene.addItem(item)
			roots.append(item)
		last_text_order = -1
		for text_render in text_renders:
			item = ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem._from_observation(
				text_render, telex,
			)
			target = _presentation_target(item.target)
			if target.source_order <= last_text_order:
				raise FerrumRenderProjectionError("Text renders are not source ordered")
			if target.source_order in document_root_orders:
				raise FerrumRenderProjectionError("duplicate document root source order")
			if target in local_items:
				raise FerrumRenderProjectionError("duplicate render target")
			last_text_order = target.source_order
			document_root_orders.add(target.source_order)
			if target.is_durable:
				durable_key = _durable_target_key(target)
				if durable_key in durable_items:
					raise FerrumRenderProjectionError("duplicate durable render target")
				durable_items[durable_key] = item
			local_items[target] = item
			item_targets[item] = target
			items.append(item)
			scene.addItem(item)
			roots.append(item)
		all_issues.extend(_observation_issue(value) for value in issues)
	except (
		AttributeError,
		TypeError,
		ValueError,
		FerrumRenderProjectionError,
		ferrum_qt.canvas.ferrum_presentation_render_plan.PresentationRenderPlanError,
		ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItemError,
		ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItemError,
		ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItemError,
	) as exc:
		_retire_failed_projection(scene, presentation)
		if isinstance(exc, FerrumRenderProjectionError):
			raise
		raise FerrumRenderProjectionError("invalid frozen render observation DTO") from exc
	if paper is None:
		raise FerrumRenderProjectionError("render projection has no paper background")
	result = FerrumRenderProjection(
		scene, revision, digest, paper, presentation, tuple(roots), molecule_roots, tuple(items),
		item_targets, durable_items, local_items, tuple(all_issues),
	)
	return result


#============================================
def _durable_target_key(target: RenderTargetKey) -> tuple[str, str]:
	"""Return an exact durable key after the caller establishes durability."""
	return target.durable_selection_key()


#============================================
def _validate_observation(observation: object,
		validator: ObservationValidator,
		) -> tuple[
			int, str, object, tuple[object, ...], tuple[object, ...], tuple[object, ...],
			tuple[object, ...],
		]:
	"""Return observation contents only after all cross-layer provenance agrees."""
	validator(observation)
	if not _is_frozen_dto(observation):
		raise FerrumRenderProjectionError("render observation must be a frozen DTO")
	if getattr(observation, "schema", None) != _OBSERVATION_SCHEMA:
		raise FerrumRenderProjectionError("unknown render observation schema")
	if getattr(observation, "suppression", None) is not None:
		raise FerrumRenderProjectionError("render observation is explicitly suppressed")
	document = getattr(observation, "document", None)
	snapshot = getattr(document, "snapshot", None)
	projection = getattr(document, "projection", None)
	revision = _revision(getattr(snapshot, "revision", None))
	digest = _digest(getattr(snapshot, "digest", None))
	if _revision(getattr(projection, "revision", None)) != revision:
		raise FerrumRenderProjectionError("document projection revision differs from snapshot")
	if _digest(getattr(projection, "digest", None)) != digest:
		raise FerrumRenderProjectionError("document projection digest differs from snapshot")
	paper_layout = getattr(projection, "paper_layout", None)
	if getattr(paper_layout, "schema", None) != _PAPER_SCHEMA:
		raise FerrumRenderProjectionError("unknown paper layout schema")
	if _revision(getattr(paper_layout, "revision", None)) != revision:
		raise FerrumRenderProjectionError("paper layout revision differs from snapshot")
	if _digest(getattr(paper_layout, "digest", None)) != digest:
		raise FerrumRenderProjectionError("paper layout digest differs from snapshot")
	provenance = getattr(observation, "provenance", None)
	if provenance is not None:
		if _revision(getattr(provenance, "revision", None)) != revision:
			raise FerrumRenderProjectionError("observation provenance revision differs from snapshot")
		if _digest(getattr(provenance, "digest", None)) != digest:
			raise FerrumRenderProjectionError("observation provenance digest differs from snapshot")
	plans = _ordered_dtos(getattr(observation, "molecule_plans", None), "molecule plans")
	plus_renders = _ordered_dtos(getattr(observation, "plus_renders", None), "plus renders")
	text_renders = _ordered_dtos(getattr(observation, "text_renders", None), "Text renders")
	issues = _ordered_dtos(getattr(observation, "issues", None), "render issues")
	return revision, digest, paper_layout, plans, plus_renders, text_renders, issues


#============================================
def _build_plan(
		scene: PySide6.QtWidgets.QGraphicsScene, plan_entry: object,
		molecule: MoleculeRootKey,
		revision: int, digest: str, telex_resource: object, item_factory: PlanItemFactory,
		) -> tuple[
			PySide6.QtWidgets.QGraphicsItemGroup,
			tuple[tuple[PySide6.QtWidgets.QGraphicsItem, RenderTargetKey], ...],
			tuple[RenderIssue, ...],
		]:
	"""Build one exact plan in source order and attach it only to the candidate scene."""
	if not _is_frozen_dto(plan_entry):
		raise FerrumRenderProjectionError("molecule render entry has the wrong DTO shape")
	plan = getattr(plan_entry, "plan", None)
	if not _is_frozen_dto(plan) or getattr(plan, "schema", None) != _PLAN_SCHEMA:
		raise FerrumRenderProjectionError("unknown render plan schema")
	provenance = getattr(plan, "provenance", None)
	if _revision(getattr(provenance, "revision", None)) != revision:
		raise FerrumRenderProjectionError("render plan revision differs from observation")
	if _digest(getattr(provenance, "digest", None)) != digest:
		raise FerrumRenderProjectionError("render plan digest differs from observation")
	batches = _ordered_dtos(getattr(plan, "batches", None), "render batches")
	issues = _ordered_dtos(getattr(plan, "issues", None), "plan render issues")
	last_order = -1
	seen_targets: set[RenderTargetKey] = set()
	result = []
	root = PySide6.QtWidgets.QGraphicsItemGroup()
	root.setHandlesChildEvents(False)
	root.setZValue(float(molecule.source_order))
	scene.addItem(root)
	for batch_index, batch in enumerate(batches):
		target = _target(getattr(batch, "target", None))
		if target.source_order <= last_order:
			raise FerrumRenderProjectionError("render batches are not source ordered")
		if target in seen_targets:
			raise FerrumRenderProjectionError("render plan has duplicate target")
		last_order = target.source_order
		seen_targets.add(target)
		item = item_factory(plan, batch_index, telex_resource, root)
		layer = getattr(batch, "display_layer", None)
		if layer not in {"ordinary", "haworth_front_stroke", "haworth_front_wedge"}:
			raise FerrumRenderProjectionError("render batch has an unknown display layer")
		layer_offset = {"ordinary": 0.0, "haworth_front_stroke": 0.1, "haworth_front_wedge": 0.2}[layer]
		item.setZValue(float(target.source_order) + layer_offset)
		result.append((item, target))
	plan_issues = _plan_issues(issues, seen_targets)
	return root, tuple(result), plan_issues


#============================================
def _molecule_root(value: object) -> MoleculeRootKey:
	"""Copy one Rust-issued molecule root without interpreting persistent CDML."""
	if not _is_frozen_dto(value):
		raise FerrumRenderProjectionError("molecule render root has the wrong DTO shape")
	identifier = getattr(value, "id", None)
	projection_key = getattr(value, "projection_key", None)
	source_id = getattr(value, "source_id", None)
	source_order = getattr(value, "source_order", None)
	if identifier is not None and (type(identifier) is not str or not identifier):
		raise FerrumRenderProjectionError("molecule durable identity is invalid")
	if type(projection_key) is not str or not projection_key:
		raise FerrumRenderProjectionError("molecule projection key is invalid")
	if source_id is not None and (type(source_id) is not str or not source_id):
		raise FerrumRenderProjectionError("molecule source identity is invalid")
	if (identifier is None) != (source_id is None):
		raise FerrumRenderProjectionError("molecule identities are inconsistent")
	if type(source_order) is not int or source_order not in _U32_RANGE:
		raise FerrumRenderProjectionError("molecule source order is invalid")
	return MoleculeRootKey(identifier, projection_key, source_id, source_order)


#============================================
def _target(value: object) -> RenderTargetKey:
	"""Copy one strict dual-identity document target into selection state."""
	if not _is_frozen_dto(value):
		raise FerrumRenderProjectionError("render target has the wrong DTO shape")
	kind = getattr(value, "kind", None)
	render_identifier = getattr(value, "render_identifier", None)
	durable_object_id = getattr(value, "durable_object_id", None)
	durable_molecule_object_id = getattr(value, "durable_molecule_object_id", None)
	if type(kind) is not str or kind not in _STRUCTURAL_KIND_MAP:
		raise FerrumRenderProjectionError("render target kind is invalid")
	for field_value, label in (
		(render_identifier, "render identifier"),
		(durable_object_id, "durable object identity"),
		(durable_molecule_object_id, "durable molecule identity"),
	):
		if type(field_value) is not str or not field_value:
			raise FerrumRenderProjectionError(f"render target {label} is invalid")
	order = getattr(value, "source_order", None)
	if type(order) is not int or order not in _U32_RANGE:
		raise FerrumRenderProjectionError("render target source order is invalid")
	target_kind = _STRUCTURAL_KIND_MAP[kind]
	return RenderTargetKey(
		target_kind, render_identifier, order, durable_object_id, durable_molecule_object_id,
	)


#============================================
def _presentation_target(value: object) -> RenderTargetKey:
	"""Return the canonical target already validated by presentation replay."""
	if type(value) is not RenderTargetKey:
		raise FerrumRenderProjectionError("presentation item has no canonical render target")
	return value


#============================================
def _plan_issues(
		values: tuple[object, ...], batch_targets: set[RenderTargetKey],
		) -> tuple[RenderIssue, ...]:
	"""Copy plan exclusions in backend order without allocating fallback graphics."""
	previous_order = -1
	seen_targets = set(batch_targets)
	result = []
	for value in values:
		if not _is_frozen_dto(value):
			raise FerrumRenderProjectionError("plan render issue has the wrong DTO shape")
		target = _target(getattr(value, "target", None))
		if target.source_order <= previous_order or target in seen_targets:
			raise FerrumRenderProjectionError("plan render issues do not partition targets")
		kind = getattr(value, "kind", None)
		detail = getattr(value, "detail", None)
		if type(kind) is not str or not kind or type(detail) is not str or not detail:
			raise FerrumRenderProjectionError("plan render issue is invalid")
		previous_order = target.source_order
		seen_targets.add(target)
		result.append(RenderIssue(kind, target, detail))
	return tuple(result)


#============================================
def _observation_issue(value: object) -> RenderIssue:
	"""Copy one whole-observation depiction issue without allocating graphics."""
	if not _is_frozen_dto(value):
		raise FerrumRenderProjectionError("observation issue has the wrong DTO shape")
	code = getattr(value, "code", None)
	target = getattr(value, "target", None)
	detail = getattr(value, "detail", None)
	if type(code) is not str or not code or type(target) is not str or not target:
		raise FerrumRenderProjectionError("observation issue is invalid")
	if type(detail) is not str or not detail:
		raise FerrumRenderProjectionError("observation issue detail is invalid")
	return RenderIssue(code, target, detail)


#============================================
def _ordered_dtos(value: object, label: str) -> tuple[object, ...]:
	"""Copy one immutable PyO3 tuple without accepting mutable or lazy sequences."""
	if not isinstance(value, tuple):
		raise FerrumRenderProjectionError(f"{label} are not an ordered DTO sequence")
	return value


#============================================
def _revision(value: object) -> int:
	"""Validate one exact u64 revision copied from Rust."""
	if type(value) is not int or value < 0 or value >= 2**64:
		raise FerrumRenderProjectionError("render revision is invalid")
	return value


#============================================
def _digest(value: object) -> str:
	"""Validate one exact lowercase structural digest copied from Rust."""
	if type(value) is not str or _DIGEST.fullmatch(value) is None:
		raise FerrumRenderProjectionError("render digest is invalid")
	return value


#============================================
def _retire_failed_projection(
		scene: PySide6.QtWidgets.QGraphicsScene,
		presentation: (
			ferrum_qt.canvas.ferrum_presentation_render_plan.FerrumPresentationScene
			| None
		),
		) -> None:
	"""Dispose a partially built candidate without touching an installed projection."""
	detached = [] if presentation is None else [
		root for root in presentation.roots
		if ferrum_qt.canvas.graphics_retirement.native_scene_for_item(root) is not scene
	]
	coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
	items = list(scene.items())
	if items:
		coordinator.retire_scene_projection_items(scene, items)
	if detached:
		coordinator.retire_detached_projection_items(detached)


#============================================
def _build_presentation_scene(
		presentation_plan: object, telex_resource: object,
		) -> ferrum_qt.canvas.ferrum_presentation_render_plan.FerrumPresentationScene:
	"""Build presentation roots solely from the renderer-issued immutable plan."""
	return ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
		presentation_plan, telex_resource,
	)


#============================================
def _is_frozen_dto(value: object) -> bool:
	"""Reject mutable maps, XML, and normal Python model objects at the boundary."""
	return value is not None and not isinstance(value, dict) and not hasattr(value, "__dict__")


#============================================
def _require_pyo3_observation(observation: object) -> None:
	"""Accept only the extension-owned observation class on the production path."""
	try:
		import ferrum_qt.ferrum.engine as engine
		observation_type = engine.RenderObservationV1
	except (ImportError, AttributeError) as exc:
		raise FerrumRenderProjectionError(
			"Ferrum render observation binding is unavailable",
		) from exc
	if type(observation) is not observation_type:
		raise FerrumRenderProjectionError(
			"render observation must be the frozen engine.RenderObservationV1 DTO",
		)
