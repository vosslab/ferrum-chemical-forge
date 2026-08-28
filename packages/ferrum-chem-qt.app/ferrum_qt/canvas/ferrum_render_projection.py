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
import ferrum_qt.canvas.graphics_disposal
import ferrum_qt.canvas.ferrum_presentation_render_plan
import ferrum_qt.canvas.ferrum_presentation_target
from ferrum_qt.canvas.ferrum_render_target import RenderTargetKey
import ferrum_qt.canvas.ferrum_telex
import ferrum_qt.canvas.items.ferrum_molecule_root_item
import ferrum_qt.canvas.items.ferrum_plan_item
import ferrum_qt.canvas.items.ferrum_paper_item
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.canvas.items.ferrum_text_item
import ferrum_qt.themes.document_display_palette
import ferrum_qt.themes.theme_loader


_OBSERVATION_SCHEMA = "ferrum-document-render-observation-v1"
_PLAN_SCHEMA = "ferrum-render-plan-v3"
_PAPER_SCHEMA = "ferrum-document-paper-layout-v1"
_DIGEST = re.compile(r"^[0-9a-f]{64}$")
_U32_RANGE = range(2**32)
_PRESENTATION_KINDS = frozenset((
	"arrow", "plus", "text", "polyline", "rectangle", "square", "oval", "circle",
	"polygon",
))
_DOCUMENT_OBJECT_KIND = "document_object"
_DIRECT_ROOT_KINDS = _PRESENTATION_KINDS | frozenset(("molecule", "rejected_presentation"))
ObservationValidator = collections.abc.Callable[[object], None]
PlanItemFactory = collections.abc.Callable[
	[object, int, object, ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		PySide6.QtWidgets.QGraphicsItem],
	PySide6.QtWidgets.QGraphicsItem,
]
MoleculeRootItemFactory = collections.abc.Callable[
	[object, object],
	ferrum_qt.canvas.items.ferrum_molecule_root_item.FerrumMoleculeRootItem,
]
PresentationSceneFactory = collections.abc.Callable[
	[object, ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1],
	ferrum_qt.canvas.ferrum_presentation_render_plan.FerrumPresentationScene,
]
PaperItemFactory = collections.abc.Callable[
	[object], ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem,
]


#============================================
class FerrumRenderProjectionError(ValueError):
	"""Raised when a copied render observation violates the V1 contract."""


@dataclasses.dataclass(frozen=True, slots=True)
class RenderIssue:
	"""Visible Rust diagnostic that deliberately has no graphics item."""

	category: str
	document_object_id: str
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
	molecule_roots: tuple[
		ferrum_qt.canvas.items.ferrum_molecule_root_item.FerrumMoleculeRootItem, ...
	]
	items: tuple[PySide6.QtWidgets.QGraphicsItem, ...]
	item_targets: dict[PySide6.QtWidgets.QGraphicsItem, RenderTargetKey]
	durable_items: dict[tuple[str, str], PySide6.QtWidgets.QGraphicsItem]
	local_items: dict[RenderTargetKey, PySide6.QtWidgets.QGraphicsItem]
	issues: tuple[RenderIssue, ...]
	_disposed: bool = dataclasses.field(default=False, init=False, repr=False)

	#============================================
	def selected_targets(self) -> tuple[RenderTargetKey, ...]:
		"""Return selected current targets without promoting local keys to IDs."""
		selected = ferrum_qt.canvas.graphics_disposal.selected_items_from_captured_scene(
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
		"""Terminally dispose this projection's graphics and detached scene once."""
		if self._disposed:
			return
		coordinator = ferrum_qt.canvas.graphics_disposal.GraphicsDisposalCoordinator()
		coordinator.dispose_scene_projection_items(self.scene, list(self.roots))
		coordinator.raise_if_callback_failed("Ferrum render projection disposal failed")
		self.scene.deleteLater()
		self._disposed = True


#============================================
class FerrumRenderProjectionController:
	"""Atomically swap complete validated render scenes into one graphics view."""

	#============================================
	def __init__(self, view: PySide6.QtWidgets.QGraphicsView,
			telex_resource: object,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> None:
		"""Bind to one UI-thread view without taking document/session ownership."""
		telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
		self._initialize(
			view, telex_resource, telex, _require_pyo3_observation,
			ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem,
			ferrum_qt.canvas.items.ferrum_molecule_root_item.FerrumMoleculeRootItem,
			ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem,
			lambda plan, display_palette: _build_presentation_scene(
				plan, telex_resource, display_palette,
			),
			palette,
		)

	#============================================
	def _initialize(self, view: PySide6.QtWidgets.QGraphicsView,
			telex_resource: object, telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
			validator: ObservationValidator, item_factory: PlanItemFactory,
			molecule_root_factory: MoleculeRootItemFactory,
			paper_factory: PaperItemFactory,
			presentation_factory: PresentationSceneFactory | None,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> None:
		"""Initialize the production controller or the private fixture seam."""
		if not isinstance(view, PySide6.QtWidgets.QGraphicsView):
			raise TypeError("Ferrum render projection controller requires a graphics view")
		if not isinstance(telex, ferrum_qt.canvas.ferrum_telex.FerrumTelex):
			raise TypeError("Ferrum render projection controller requires verified Telex")
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise TypeError("Ferrum render projection controller requires a document display palette")
		self._view = view
		self._telex_resource = telex_resource
		self._telex = telex
		self._validator = validator
		self._item_factory = item_factory
		self._molecule_root_factory = molecule_root_factory
		self._paper_factory = paper_factory
		self._presentation_factory = presentation_factory
		self._palette = palette
		self._generation = 0
		self._disposed = False
		self.projection: FerrumRenderProjection | None = None

	#============================================
	@property
	def generation(self) -> int:
		"""Return the current delivery generation for the owning tab session."""
		return self._generation

	#============================================
	@property
	def document_display_palette(self) -> ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
		"""Return the one immutable palette currently used by this controller."""
		return self._palette

	#============================================
	def replace(self, observation: object, latch: RenderProjectionLatch,
			presentation_plan: object | None = None) -> bool:
		"""Build then install one current observation without disturbing prior state."""
		if self._disposed or latch.generation != self._generation:
			return False
		try:
			prepared = _build_render_projection(
				observation, self._telex_resource, self._telex, self._validator,
				self._item_factory, self._molecule_root_factory, self._paper_factory,
				self._presentation_factory,
				presentation_plan, self._palette,
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
	def refresh_display_palette(self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1) -> None:
		"""Refresh retained display materials without requesting a new Rust observation."""
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise TypeError("Ferrum render projection requires a document display palette")
		self._palette = palette
		if self.projection is None:
			return
		self.projection.paper.refresh_display_palette(palette)
		presentation_items = frozenset(
			() if self.projection.presentation is None else self.projection.presentation.items,
		)
		for item in self.projection.items:
			if item in presentation_items:
				continue
			item.refresh_display_palette(palette)
		if self.projection.presentation is not None:
			self.projection.presentation.refresh_display_palette(palette)
		self.projection.scene.update()

	#============================================
	def invalidate_delivery(self) -> int:
		"""Make any already-captured worker or callback result stale."""
		self._generation += 1
		return self._generation

	#============================================
	def dispose(self) -> None:
		"""Invalidate delivery and dispose the installed disposable projection once."""
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
		ferrum_qt.canvas.items.ferrum_molecule_root_item.FerrumMoleculeRootItem._from_fixture,
		ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem._from_fixture, None,
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	return controller


#============================================
def build_render_projection(observation: object, telex_resource: object,
		presentation_plan: object,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> FerrumRenderProjection:
	"""Validate one whole observation and return a fully populated detached scene."""
	telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex_resource)
	return _build_render_projection(
		observation, telex_resource, telex, _require_pyo3_observation,
		ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem,
		ferrum_qt.canvas.items.ferrum_molecule_root_item.FerrumMoleculeRootItem,
		ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem,
		lambda plan, palette: _build_presentation_scene(plan, telex_resource, palette),
		presentation_plan, palette,
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
		ferrum_qt.canvas.items.ferrum_molecule_root_item.FerrumMoleculeRootItem._from_fixture,
		ferrum_qt.canvas.items.ferrum_paper_item.FerrumPaperItem._from_fixture,
		presentation_factory,
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)


#============================================
def _build_render_projection(observation: object, telex_resource: object,
		telex: ferrum_qt.canvas.ferrum_telex.FerrumTelex,
		validator: ObservationValidator, item_factory: PlanItemFactory,
		molecule_root_factory: MoleculeRootItemFactory,
		paper_factory: PaperItemFactory,
		presentation_factory: PresentationSceneFactory | None,
		presentation_plan: object | None = None,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1 | None = None) -> FerrumRenderProjection:
	"""Build a validated observation after its caller selected the entry contract."""
	if not isinstance(telex, ferrum_qt.canvas.ferrum_telex.FerrumTelex):
		raise FerrumRenderProjectionError("render projection requires verified Telex")
	revision, digest, paper_layout, direct_root_orders, plans, plus_renders, text_renders = (
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
	molecule_roots: list[
		ferrum_qt.canvas.items.ferrum_molecule_root_item.FerrumMoleculeRootItem
	] = []
	items: list[PySide6.QtWidgets.QGraphicsItem] = []
	item_targets: dict[PySide6.QtWidgets.QGraphicsItem, RenderTargetKey] = {}
	durable_items: dict[tuple[str, str], PySide6.QtWidgets.QGraphicsItem] = {}
	local_items: dict[RenderTargetKey, PySide6.QtWidgets.QGraphicsItem] = {}
	all_issues: list[RenderIssue] = []
	presentation = None
	seen_molecule_roots: set[str] = set()
	seen_presentation_roots: set[str] = set()
	expected_molecule_roots = {
		identifier for identifier, (kind, _order) in direct_root_orders.items()
		if kind == "molecule"
	}
	expected_presentation_roots = {
		identifier for identifier, (kind, _order) in direct_root_orders.items()
		if kind in _PRESENTATION_KINDS
	}
	try:
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise FerrumRenderProjectionError(
				"render projection requires a document display palette",
			)
		display_palette = palette
		paper = paper_factory(paper_layout, display_palette)
		ferrum_qt.canvas.ferrum_presentation_render_plan.require_display_palette_refreshable(
			paper, "paper render item",
		)
		scene.addItem(paper)
		scene.setSceneRect(paper.rect())
		roots.append(paper)
		for plan_entry in plans:
			molecule = getattr(plan_entry, "molecule", None)
			molecule_object_id = _molecule_root_identifier(molecule)
			if molecule_object_id in seen_molecule_roots:
				raise FerrumRenderProjectionError("duplicate molecule render root")
			root, _plan_items, plan_issues = _build_plan(
				plan_entry, molecule, revision, digest, telex_resource, item_factory,
				molecule_root_factory, display_palette,
			)
			root.setZValue(float(_direct_root_order(
				direct_root_orders, molecule_object_id, frozenset(("molecule",)),
			)))
			scene.addItem(root)
			seen_molecule_roots.add(molecule_object_id)
			roots.append(root)
			molecule_roots.append(root)
			for item, target in _plan_items:
				ferrum_qt.canvas.ferrum_presentation_render_plan.require_display_palette_refreshable(
					item, "molecule render item",
				)
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
			all_issues.extend(_member_issues(
				_ordered_dtos(getattr(plan_entry, "member_issues", None), "member render issues"),
			))
		if seen_molecule_roots != expected_molecule_roots:
			raise FerrumRenderProjectionError("molecule render roots do not exactly own direct roots")
		if presentation_factory is not None:
			if presentation_plan is None:
				raise FerrumRenderProjectionError("renderer presentation plan is required")
			presentation = presentation_factory(presentation_plan, display_palette)
			if presentation.revision != revision or presentation.digest != digest:
				raise FerrumRenderProjectionError(
					"presentation scene provenance differs from render observation",
				)
			if (
				len(presentation.roots) != len(presentation.items)
				or frozenset(presentation.roots) != frozenset(presentation.items)
			):
				raise FerrumRenderProjectionError("presentation root ownership is incomplete")
			for root in presentation.roots:
				target = _presentation_target(getattr(root, "target", None))
				if target.document_object_id in seen_presentation_roots:
					raise FerrumRenderProjectionError("duplicate presentation render root")
				root.setZValue(float(_direct_root_order(
					direct_root_orders, target.document_object_id, _PRESENTATION_KINDS,
				)))
				seen_presentation_roots.add(target.document_object_id)
				scene.addItem(root)
				roots.append(root)
				if target in local_items:
					raise FerrumRenderProjectionError("duplicate presentation render target")
				if target.is_durable:
					durable_key = _durable_target_key(target)
					if durable_key in durable_items:
						raise FerrumRenderProjectionError("duplicate durable presentation target")
					durable_items[durable_key] = root
				local_items[target] = root
				item_targets[root] = target
				items.append(root)
		for plus in plus_renders:
			item = ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem._from_observation(
				plus, telex, display_palette,
			)
			ferrum_qt.canvas.ferrum_presentation_render_plan.require_display_palette_refreshable(
				item, "Plus render item",
			)
			target = _presentation_target(item.target)
			if target.document_object_id in seen_presentation_roots:
				raise FerrumRenderProjectionError("duplicate presentation render root")
			if target in local_items:
				raise FerrumRenderProjectionError("duplicate render target")
			item.setZValue(float(_direct_root_order(
				direct_root_orders, target.document_object_id, frozenset(("plus",)),
			)))
			seen_presentation_roots.add(target.document_object_id)
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
		for text_render in text_renders:
			item = ferrum_qt.canvas.items.ferrum_text_item.FerrumTextItem._from_observation(
				text_render, telex, display_palette,
			)
			ferrum_qt.canvas.ferrum_presentation_render_plan.require_display_palette_refreshable(
				item, "Text render item",
			)
			target = _presentation_target(item.target)
			if target.document_object_id in seen_presentation_roots:
				raise FerrumRenderProjectionError("duplicate presentation render root")
			if target in local_items:
				raise FerrumRenderProjectionError("duplicate render target")
			item.setZValue(float(_direct_root_order(
				direct_root_orders, target.document_object_id, frozenset(("text",)),
			)))
			seen_presentation_roots.add(target.document_object_id)
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
		if seen_presentation_roots != expected_presentation_roots:
			raise FerrumRenderProjectionError(
				"presentation render roots do not exactly own direct roots",
			)
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
		_dispose_failed_projection(scene, presentation)
		if isinstance(exc, FerrumRenderProjectionError):
			raise
		raise FerrumRenderProjectionError("invalid frozen render observation DTO") from exc
	if paper is None:
		raise FerrumRenderProjectionError("render projection has no paper background")
	result = FerrumRenderProjection(
		scene, revision, digest, paper, presentation, tuple(roots), tuple(molecule_roots), tuple(items),
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
			int, str, object, dict[str, tuple[str, int]], tuple[object, ...],
			tuple[object, ...], tuple[object, ...],
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
	direct_root_orders = _direct_root_orders(
		_ordered_dtos(getattr(projection, "direct_roots", None), "document direct roots"),
	)
	provenance = getattr(observation, "provenance", None)
	if provenance is not None:
		if _revision(getattr(provenance, "revision", None)) != revision:
			raise FerrumRenderProjectionError("observation provenance revision differs from snapshot")
		if _digest(getattr(provenance, "digest", None)) != digest:
			raise FerrumRenderProjectionError("observation provenance digest differs from snapshot")
	plans = _ordered_dtos(getattr(observation, "molecule_plans", None), "molecule plans")
	plus_renders = _ordered_dtos(getattr(observation, "plus_renders", None), "plus renders")
	text_renders = _ordered_dtos(getattr(observation, "text_renders", None), "Text renders")
	return (
		revision, digest, paper_layout, direct_root_orders, plans, plus_renders, text_renders,
	)


#============================================
def _build_plan(
		plan_entry: object, molecule: object,
		revision: int, digest: str, telex_resource: object, item_factory: PlanItemFactory,
		molecule_root_factory: MoleculeRootItemFactory,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> tuple[
			ferrum_qt.canvas.items.ferrum_molecule_root_item.FerrumMoleculeRootItem,
			tuple[tuple[PySide6.QtWidgets.QGraphicsItem, RenderTargetKey], ...],
			tuple[RenderIssue, ...],
		]:
	"""Build one detached ownership hierarchy in exact source order."""
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
	seen_targets: set[RenderTargetKey] = set()
	result = []
	root = molecule_root_factory(molecule, getattr(plan_entry, "bounds", None))
	for batch_index, batch in enumerate(batches):
		target = _target(getattr(batch, "target", None))
		if target in seen_targets:
			raise FerrumRenderProjectionError("render plan has duplicate target")
		seen_targets.add(target)
		item = item_factory(plan, batch_index, telex_resource, palette, root)
		layer = getattr(batch, "display_layer", None)
		if layer not in {"ordinary", "haworth_front_stroke", "haworth_front_wedge"}:
			raise FerrumRenderProjectionError("render batch has an unknown display layer")
		layer_offset = {"ordinary": 0.0, "haworth_front_stroke": 0.1, "haworth_front_wedge": 0.2}[layer]
		item.setZValue(layer_offset)
		result.append((item, target))
	plan_issues = _plan_issues(issues, seen_targets)
	return root, tuple(result), plan_issues


#============================================
def _molecule_root_identifier(value: object) -> str:
	"""Copy one Rust-issued molecule-root identity without interpreting CDML."""
	if not _is_frozen_dto(value):
		raise FerrumRenderProjectionError("molecule render root has the wrong DTO shape")
	document_object_id = getattr(value, "document_object_id", None)
	if type(document_object_id) is not str or not document_object_id:
		raise FerrumRenderProjectionError("molecule document-object identity is invalid")
	return document_object_id


#============================================
def _target(value: object) -> RenderTargetKey:
	"""Copy one strict dual-identity document target into selection state."""
	if not _is_frozen_dto(value):
		raise FerrumRenderProjectionError("render target has the wrong DTO shape")
	if getattr(value, "kind", None) != _DOCUMENT_OBJECT_KIND:
		raise FerrumRenderProjectionError("render target kind is invalid")
	document_object_id = getattr(value, "document_object_id", None)
	if type(document_object_id) is not str or not document_object_id:
		raise FerrumRenderProjectionError("render target document-object identity is invalid")
	return RenderTargetKey(_DOCUMENT_OBJECT_KIND, document_object_id)


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
	seen_targets = set(batch_targets)
	result = []
	for value in values:
		if not _is_frozen_dto(value):
			raise FerrumRenderProjectionError("plan render issue has the wrong DTO shape")
		target = _target(getattr(value, "target", None))
		if target in seen_targets:
			raise FerrumRenderProjectionError("plan render issues do not partition targets")
		kind = getattr(value, "kind", None)
		detail = getattr(value, "detail", None)
		if type(kind) is not str or not kind or type(detail) is not str or not detail:
			raise FerrumRenderProjectionError("plan render issue is invalid")
		seen_targets.add(target)
		result.append(RenderIssue(kind, target.document_object_id, detail))
	return tuple(result)


#============================================
def _member_issues(values: tuple[object, ...]) -> tuple[RenderIssue, ...]:
	"""Copy molecule-member diagnostics in Rust order without folding plan issues into them."""
	result = []
	for value in values:
		if not _is_frozen_dto(value):
			raise FerrumRenderProjectionError("member render issue has the wrong DTO shape")
		document_object_id = getattr(value, "document_object_id", None)
		category = getattr(value, "category", None)
		detail = getattr(value, "detail", None)
		if (
			type(document_object_id) is not str or not document_object_id
			or type(category) is not str or not category
			or type(detail) is not str or not detail
		):
			raise FerrumRenderProjectionError("member render issue is invalid")
		result.append(RenderIssue(category, document_object_id, detail))
	return tuple(result)


#============================================
def _direct_root_orders(values: tuple[object, ...]) -> dict[str, tuple[str, int]]:
	"""Copy the global direct-root order map without trusting DTO list coincidence."""
	result = {}
	seen_orders = set()
	for value in values:
		if not _is_frozen_dto(value):
			raise FerrumRenderProjectionError("document direct root has the wrong DTO shape")
		document_object_id = getattr(value, "document_object_id", None)
		kind = getattr(value, "kind", None)
		paint_order = getattr(value, "paint_order", None)
		if type(document_object_id) is not str or not document_object_id:
			raise FerrumRenderProjectionError("document direct root identity is invalid")
		if type(kind) is not str or kind not in _DIRECT_ROOT_KINDS:
			raise FerrumRenderProjectionError("document direct root kind is invalid")
		if type(paint_order) is not int or paint_order not in _U32_RANGE:
			raise FerrumRenderProjectionError("document direct root paint order is invalid")
		if document_object_id in result or paint_order in seen_orders:
			raise FerrumRenderProjectionError("document direct roots are not unique")
		result[document_object_id] = kind, paint_order
		seen_orders.add(paint_order)
	return result


#============================================
def _direct_root_order(root_orders: dict[str, tuple[str, int]],
		document_object_id: str, expected_kinds: frozenset[str]) -> int:
	"""Require a rendered root to own one compatible document direct-root entry."""
	try:
		kind, paint_order = root_orders[document_object_id]
	except KeyError as exc:
		raise FerrumRenderProjectionError("render root has no document direct-root owner") from exc
	if kind not in expected_kinds:
		raise FerrumRenderProjectionError("render root direct-root kind differs from payload")
	return paint_order




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
def _dispose_failed_projection(
		scene: PySide6.QtWidgets.QGraphicsScene,
		presentation: (
			ferrum_qt.canvas.ferrum_presentation_render_plan.FerrumPresentationScene
			| None
		),
		) -> None:
	"""Dispose a partially built candidate without touching an installed projection."""
	detached = [] if presentation is None else [
		root for root in presentation.roots
		if ferrum_qt.canvas.graphics_disposal.native_scene_for_item(root) is not scene
	]
	coordinator = ferrum_qt.canvas.graphics_disposal.GraphicsDisposalCoordinator()
	items = list(scene.items())
	if items:
		coordinator.dispose_scene_projection_items(scene, items)
	if detached:
		coordinator.dispose_detached_projection_items(detached)


#============================================
def _build_presentation_scene(
		presentation_plan: object, telex_resource: object,
		palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
		) -> ferrum_qt.canvas.ferrum_presentation_render_plan.FerrumPresentationScene:
	"""Build presentation roots solely from the renderer-issued immutable plan."""
	return ferrum_qt.canvas.ferrum_presentation_render_plan.build_presentation_render_plan(
		presentation_plan, telex_resource, palette,
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
