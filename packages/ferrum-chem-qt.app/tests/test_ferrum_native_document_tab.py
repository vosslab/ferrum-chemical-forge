"""Behavioral coverage for the isolated Rust-owned Ferrum document tab."""

# Standard Library
import ast
import collections.abc
import dataclasses
import hashlib
import os
import pathlib


# Qt reads the platform selection before this isolated test creates an application.
os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.live_document_transaction


_EDITABLE_CDML = (
	'<cdml version="26.08"><molecule id="molecule-1"><atom id="atom-c" name="C">'
	'<point x="0cm" y="0cm"/></atom></molecule></cdml>'
)

_BOND_CDML = (
	'<cdml version="26.08"><molecule id="molecule-1">'
	'<atom id="atom-c" name="C"><point x="0" y="0"/></atom>'
	'<atom id="atom-o" name="O"><point x="30" y="0"/></atom>'
	'</molecule></cdml>'
)

_UNRENDERABLE_LABEL_CDML = (
	'<cdml version="26.08"><standard font_family="helvetica"/><molecule id="molecule-1"><atom id="atom-c" name="C">'
	'<point x="0cm" y="0cm"/><ftext>C&lt;sub&gt;2&lt;/sub&gt;</ftext>'
	'</atom></molecule></cdml>'
)

_LIVE_SMARTS_MULTIROW_CDML = (
	'<cdml version="26.08">'
	'<molecule id="molecule-1"><atom id="atom-c1" name="C">'
	'<point x="0" y="0"/></atom></molecule>'
	'<molecule id="molecule-2"><atom id="atom-c2" name="C">'
	'<point x="30" y="0"/></atom></molecule>'
	'<molecule id="molecule-3"><atom id="atom-c3" name="C">'
	'<point x="60" y="0"/></atom></molecule>'
	'</cdml>'
)


#============================================
class _ObserveFailureSession:
	"""Real-session wrapper that fails only the post-accept render observation."""

	#============================================
	def __init__(self, session: object) -> None:
		"""Retain the exact real Rust session for every non-render call."""
		self._session = session

	#============================================
	def __getattr__(self, name: str) -> object:
		"""Delegate the closed PyO3 API unchanged except for rendering."""
		return getattr(self._session, name)

	#============================================
	def observe_render(self, _revision: int) -> object:
		"""Make the presentation follow-up fail after Rust already accepted."""
		raise RuntimeError("injected render observation failure")

	#============================================
	def _publish_live_render_plan_v1(self, revision: int) -> object:
		"""Fail the exact private publication seam used by the current tab."""
		return self.observe_render(revision)


@dataclasses.dataclass(frozen=True, slots=True)
class _Snapshot:
	"""Compact immutable Rust snapshot fixture."""

	revision: int
	digest: str
	is_dirty: bool


@dataclasses.dataclass(frozen=True, slots=True)
class _DocumentObservation:
	"""Compact document envelope used by the render fixture."""

	snapshot: _Snapshot


@dataclasses.dataclass(frozen=True, slots=True)
class _RenderObservation:
	"""Render fixture that carries the same durable snapshot provenance."""

	document: _DocumentObservation


@dataclasses.dataclass(frozen=True, slots=True)
class _Outcome:
	"""Immutable publication confirmation fact."""

	is_confirmed: bool


@dataclasses.dataclass(frozen=True, slots=True)
class _Publication:
	"""Rust publication result fixture."""

	snapshot: _Snapshot
	outcome: _Outcome


class _Session:
	"""Owned-value Ferrum session fake with explicit current snapshots."""

	#============================================
	def __init__(self, current: _Snapshot, saved: _Snapshot,
			confirmed: bool) -> None:
		"""Retain explicit backend facts for one deterministic interaction."""
		self._current = current
		self._saved = saved
		self._confirmed = confirmed
		self._published = False

	#============================================
	def snapshot(self) -> _Snapshot:
		"""Return the current backend snapshot."""
		return self._current

	#============================================
	def observe_render(self, revision: int) -> _RenderObservation:
		"""Return the observation associated with the requested current revision."""
		snapshot = self._saved if self._published else self._current
		if revision != snapshot.revision:
			raise ValueError("unexpected revision")
		return _RenderObservation(_DocumentObservation(snapshot))

	#============================================
	def _retire_live_document_smarts_query_v1(self) -> None:
		"""Accept the private retirement call used before every live transition."""

	#============================================
	def _retire_live_document_smarts_receipts_v1(self) -> None:
		"""Accept receipt cleanup without retiring the fixture's render plan."""

	#============================================
	def _publish_live_render_plan_v1(self, revision: int) -> _RenderObservation:
		"""Return one fake API-owned render-plan publication."""
		return self.observe_render(revision)

	#============================================
	def save_atomic(self, _path: object, revision: int) -> _Publication:
		"""Publish only the current requested revision."""
		if revision != self._current.revision:
			raise ValueError("unexpected revision")
		self._published = self._confirmed
		return _Publication(self._saved, _Outcome(self._confirmed))


#============================================
class _TransitionSession(_Session):
	"""Record the exact tab state visible at a pre-mutation proxy boundary."""

	#============================================
	def __init__(self, current: _Snapshot, saved: _Snapshot,
			confirmed: bool, events: list[tuple[str, bool]]) -> None:
		"""Keep the event log outside the tab so assertions cannot inspect internals."""
		super().__init__(current, saved, confirmed)
		self._events = events

	#============================================
	def submit(self, *_unused: object) -> object:
		"""Record whether a temporary overlay survived until mutation began."""
		self._events.append(("mutation", False))
		return object()

	#============================================
	def set_document_molecule_name_v1(self, *_unused: object) -> object:
		"""Record the first reviewed direct mutation route."""
		self._events.append(("set_document_molecule_name_v1", False))
		return object()

	#============================================
	def convert_linear_form_v1(self, *_unused: object) -> object:
		"""Record the second reviewed direct mutation route."""
		self._events.append(("convert_linear_form_v1", False))
		return object()


#============================================
class _RetirementFailureSession(_Session):
	"""Raise only while invalidating a live-query receipt."""

	#============================================
	def __init__(self, current: _Snapshot, events: list[str]) -> None:
		"""Keep explicit operation evidence for the fail-closed boundary test."""
		super().__init__(current, current, True)
		self._events = events
		self._should_fail = False

	#============================================
	def _retire_live_document_smarts_query_v1(self) -> None:
		"""Prove the native retirement boundary can fail before a document action."""
		if not self._should_fail:
			return
		self._events.append("native_retire")
		raise RuntimeError("injected private retirement failure")

	#============================================
	def set_document_molecule_name_v1(self, *_unused: object) -> object:
		"""Record an operation which must never run after retirement failure."""
		self._events.append("mutation")
		return object()


#============================================
class _PublicationTraceSession(_Session):
	"""Record the exact private API publication boundary for one tab fixture."""

	#============================================
	def __init__(self, current: _Snapshot, events: list[tuple[str, bool]],
			visual_is_retired: collections.abc.Callable[[], bool]) -> None:
		"""Keep the visual ordering oracle outside the session fake."""
		super().__init__(current, current, True)
		self._events = events
		self._visual_is_retired = visual_is_retired

	#============================================
	def _publish_live_render_plan_v1(self, revision: int) -> _RenderObservation:
		"""Record publication after the tab has retired its visual receipt."""
		self._events.append(("publish", self._visual_is_retired()))
		return super()._publish_live_render_plan_v1(revision)


#============================================
class _LivePlanReceiptSession(_Session):
	"""Model one private plan whose query receipts survive installation only."""

	#============================================
	def __init__(self, current: _Snapshot, events: list[tuple[str, object]],
			visual_is_retired: collections.abc.Callable[[], bool]) -> None:
		"""Keep the plan state and event log outside the Qt transaction mixin."""
		super().__init__(current, current, True)
		self._events = events
		self._visual_is_retired = visual_is_retired
		self._plan_is_live = False
		self._issued_receipt: object | None = None

	#============================================
	def _retire_live_document_smarts_query_v1(self) -> None:
		"""Revoke the old plan and receipt before a new document transition."""
		self._events.append(("rust_retire", self._visual_is_retired()))
		self._plan_is_live = False
		self._issued_receipt = None

	#============================================
	def _publish_live_render_plan_v1(self, revision: int) -> _RenderObservation:
		"""Commit one query-capable native plan after the sole retirement fence."""
		observation = super()._publish_live_render_plan_v1(revision)
		self._plan_is_live = True
		self._events.append(("rust_publish", self._plan_is_live))
		return observation

	#============================================
	def _run_live_document_smarts_query_v1(self, *_unused: object) -> object:
		"""Issue one opaque receipt only while the published plan remains live."""
		if not self._plan_is_live:
			raise RuntimeError("SMARTS query was refused")
		self._issued_receipt = object()
		self._events.append(("query_receipt", self._issued_receipt is not None))
		return type("LiveQuery", (), {"receipt": self._issued_receipt})()

	#============================================
	def set_document_molecule_name_v1(self, *_unused: object) -> object:
		"""Prove the proxy has revoked the current receipt before mutation begins."""
		self._events.append(("mutation", self._issued_receipt is None))
		return object()


class _Controller:
	"""Projection controller fake that retains accepted latches and terminal state."""

	#============================================
	def __init__(self, acceptances: tuple[bool, ...] = (True,)) -> None:
		"""Create one current generation with deterministic render decisions."""
		self.generation = 0
		self._acceptances = iter(acceptances)
		self.disposed = False
		self.installed: _RenderObservation | None = None

	#============================================
	def replace(self, observation: _RenderObservation, latch: object) -> bool:
		"""Accept only current non-terminal delivery with matching provenance."""
		if self.disposed or latch.generation != self.generation:
			return False
		if observation.document.snapshot.revision != latch.revision:
			return False
		if observation.document.snapshot.digest != latch.digest:
			return False
		accepted = next(self._acceptances)
		if accepted:
			self.installed = observation
		return accepted

	#============================================
	def dispose(self) -> None:
		"""Make all later render deliveries terminally stale."""
		self.disposed = True
		self.generation += 1


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an isolated offscreen QApplication without the legacy app host."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _tab(current: _Snapshot, saved: _Snapshot, confirmed: bool,
		acceptances: tuple[bool, ...] = (True, True)) -> tuple[
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			_Controller,
		]:
	"""Build a tab only through its explicitly private owned-value fixture seam."""
	controller = _Controller(acceptances)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", _Session(current, saved, confirmed), controller,
	)
	return tab, controller


#============================================
def _install_transient_overlay(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		) -> tuple[PySide6.QtWidgets.QGraphicsRectItem, int]:
	"""Attach one disposable paint item without interpreting any chemistry facts."""
	scene = PySide6.QtWidgets.QGraphicsScene(tab)
	tab._view.setScene(scene)
	item = scene.addRect(0.0, 0.0, 3.0, 3.0)
	token = tab._install_live_smarts_query_overlay_v1(item, object())
	return item, token


#============================================
def _unattached_transient_overlay() -> PySide6.QtWidgets.QGraphicsRectItem:
	"""Build a renderer-like paint item without attaching it to any scene."""
	return PySide6.QtWidgets.QGraphicsRectItem(0.0, 0.0, 3.0, 3.0)


#============================================
def _direct_tab_session_methods() -> frozenset[str]:
	"""Derive every direct `self._session.method(...)` use from Qt source.

	The allowlist below is intentionally in this test rather than the transaction
	implementation. A newly introduced direct session call therefore fails closed
	until an author independently classifies it as read-only or protects it at the
	transaction boundary.
	"""
	package_root = pathlib.Path(__file__).parents[1] / "ferrum_qt" / "ferrum"
	methods: set[str] = set()
	for source_path in package_root.rglob("*.py"):
		tree = ast.parse(source_path.read_text(encoding="utf-8"), filename=str(source_path))
		for node in ast.walk(tree):
			if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Attribute):
				continue
			session = node.func.value
			if (
				isinstance(session, ast.Attribute)
				and session.attr == "_session"
				and isinstance(session.value, ast.Name)
				and session.value.id == "self"
			):
				methods.add(node.func.attr)
	return frozenset(methods)


#============================================
def test_direct_session_mutation_surface_is_owned_by_transaction_proxy() -> None:
	"""Reject a new direct Rust mutation unless the retirement seam protects it."""
	# These are the reviewed direct calls that do not change a document fence or
	# publish/reproject a scene. Adding a name is an intentional audit decision,
	# not a proxy implementation detail.
	read_only = frozenset((
		"begin_catalog_placement_v2",
		"begin_direct_bond_gesture_v1",
		"begin_plus_placement_gesture_v1",
		"begin_presentation_creation_gesture_v1",
		"begin_presentation_vector_gesture_v1",
		"begin_reaction_definition_delete_v1",
		"begin_reaction_membership_patch_v1",
		"begin_reaction_translation_v1",
		"begin_render_interaction_translation_v1",
		"begin_text_placement_gesture_v1",
		"cancel_catalog_placement_gesture_v2",
		"observe_reaction_authoring_choices_v1",
		"observe_reaction_list_v1",
		"observe_render_interaction_v1",
		"observe_structure_interaction_v1",
		"observe_top_level_translation_anchor_v1",
		"prepare_catalog_placement_v2",
		"prepare_create_atom_v1",
		"prepare_create_bond_v2",
		"prepare_create_bonded_atom_v2",
		"prepare_create_bracket_v1",
		"prepare_create_direct_haworth_v1",
		"prepare_create_regular_ring_v1",
		"prepare_create_standalone_haworth_v1",
		"prepare_create_wavy_v1",
		"prepare_insert_molecule_v1",
		"prepare_insert_sdf_records_v1",
		"prepare_presentation_vector_gesture_v1",
		"prepare_reaction_lifecycle_v1",
		"prepare_reaction_translation_v1",
		"preview_catalog_placement_v2",
		"preview_direct_bond_gesture_v1",
		"preview_plus_placement_gesture_v1",
		"preview_presentation_creation_gesture_v1",
		"preview_presentation_vector_gesture_v1",
		"preview_render_interaction_translation_v1",
		"preview_reaction_translation_v1",
		"preview_text_placement_gesture_v1",
		"recovery_export",
		"release_catalog_placement_preview_v2",
		"select_reaction_v1",
		"select_render_interaction_roots_v1",
		"select_structure_interaction_v1",
		"snapshot",
		"text_placement_defaults_v1",
		"validate_reaction_authoring_choices_v1",
	))
	methods = _direct_tab_session_methods()
	protected = (
		ferrum_qt.ferrum.live_document_transaction._RetiringDocumentSessionV1
	)
	protected_names = protected._MUTATING_NAMES | protected._REPROJECTION_NAMES
	protected_calls = frozenset(
		method for method in methods
		if method in protected_names or method.startswith(protected._MUTATING_PREFIXES)
	)
	# These private bridge calls are issued only by the tab-local SMARTS transaction
	# owner. They create or consume opaque query capabilities without changing the
	# document fence, so they cannot be routed through the generic mutation proxy.
	smarts_transaction_calls = frozenset((
		"_capture_live_document_smarts_selected_query_v1",
		"_run_live_document_smarts_query_v1",
	))
	assert methods == read_only | protected_calls | smarts_transaction_calls, (
		"Direct Rust session call is neither independently audited read-only nor "
		"owned by the live-document SMARTS transaction: "
		f"{sorted(methods - read_only - protected_calls - smarts_transaction_calls)}"
	)


#============================================
def test_live_overlay_retires_before_session_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The proxy removes transient paint before any direct session mutation runs."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	events: list[tuple[str, bool]] = []
	controller = _Controller()
	session = _TransitionSession(current, current, True, events)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, controller,
	)
	item, _token = _install_transient_overlay(tab)
	def record_mutation(*_unused: object) -> object:
		events.append(("mutation", item.scene() is None))
		return object()
	session.submit = record_mutation
	tab._session.submit(current.revision, object())
	assert events == [("mutation", True)] and item.scene() is None
	tab.dispose()


#============================================
def test_native_retirement_failure_on_deactivation_is_terminal(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A failed tab switch fence cannot later publish or mutate the failed tab."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	events: list[str] = []
	controller = _Controller()
	session = _RetirementFailureSession(current, events)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, controller,
	)
	item, token = _install_transient_overlay(tab)
	receipt = tab._live_smarts_receipt_v1
	session._should_fail = True
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._require_live_smarts_retirement_v1("tab_deactivated")
	assert (
		events == ["native_retire"]
		and item.scene() is None
		and tab._live_smarts_receipt_v1 is receipt
		and tab._live_smarts_active_run_token_v1 == token
	)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._session.set_document_molecule_name_v1("molecule-1", "ethanol")
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._publish_live_render_plan_v1(current.revision)
	assert events == ["native_retire"] and controller.installed is not None


#============================================
def test_direct_named_mutation_routes_are_closed_proxy_inventory(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Reviewed name and linear-form routes cannot silently leave the fence set."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	for method_name in ("set_document_molecule_name_v1", "convert_linear_form_v1"):
		events: list[tuple[str, bool]] = []
		controller = _Controller()
		session = _TransitionSession(current, current, True, events)
		tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
			"Untitled", session, controller,
		)
		item, _token = _install_transient_overlay(tab)
		method = getattr(session, method_name)
		def record(*_unused: object, name: str = method_name) -> object:
			events.append((name, item.scene() is None))
			return object()
		setattr(session, method_name, record)
		getattr(tab._session, method_name)(object())
		assert events == [(method_name, True)] and item.scene() is None
		tab.dispose()
	assert {
		"set_document_molecule_name_v1", "convert_linear_form_v1",
	} <= ferrum_qt.ferrum.live_document_transaction._RetiringDocumentSessionV1._MUTATING_NAMES


#============================================
def test_native_retirement_failure_blocks_mutation_and_publication(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A failed private receipt invalidation is a typed recovery boundary."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	events: list[str] = []
	controller = _Controller()
	session = _RetirementFailureSession(current, events)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, controller,
	)
	item, _token = _install_transient_overlay(tab)
	session._should_fail = True
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._session.set_document_molecule_name_v1("molecule-1", "ethanol")
	assert events == ["native_retire"] and item.scene() is None
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._publish_live_render_plan_v1(current.revision)
	assert events == ["native_retire"] and controller.installed is not None
	assert not tab._live_smarts_retirement_available_v1
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.dispose()
	assert not tab.is_disposed and not controller.disposed


#============================================
def test_live_overlay_retires_before_plan_publication_not_installation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The published-plan installer never performs a second native retirement."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	events: list[tuple[str, object]] = []
	visual_is_retired = lambda: False
	session = _LivePlanReceiptSession(current, events, lambda: visual_is_retired())
	controller = _Controller((True, True))
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, controller,
	)
	events.clear()
	item, _token = _install_transient_overlay(tab)
	visual_is_retired = lambda: item.scene() is None
	events.clear()
	replace = controller.replace
	def record_replace(observation: _RenderObservation, latch: object) -> bool:
		events.append(("controller_replace", session._plan_is_live))
		return replace(observation, latch)
	controller.replace = record_replace
	tab._install_mutation_result(type(
		"Result", (), {"observation": _DocumentObservation(current)},
	)())
	assert events == [
		("rust_retire", True),
		("rust_publish", True),
		("controller_replace", True),
	]
	query = session._run_live_document_smarts_query_v1("C", 5, 20)
	assert query.receipt is not None
	tab._session.set_document_molecule_name_v1("molecule-1", "methane")
	assert events[-3:] == [
		("query_receipt", True),
		("rust_retire", True),
		("mutation", True),
	]
	with pytest.raises(RuntimeError, match="^SMARTS query was refused$"):
		session._run_live_document_smarts_query_v1("C", 5, 20)
	assert item.scene() is None
	tab.dispose()


#============================================
def test_live_plan_publication_retires_before_private_publish_and_projection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A real tab replacement reaches the private publication before Qt painting."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	events: list[tuple[str, bool]] = []
	visual_is_retired = lambda: False
	session = _PublicationTraceSession(current, events, lambda: visual_is_retired())
	controller = _Controller((True, True))
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, controller,
	)
	events.clear()
	item, _token = _install_transient_overlay(tab)
	visual_is_retired = lambda: item.scene() is None
	original_replace = controller.replace
	def replace(observation: _RenderObservation, latch: object) -> bool:
		events.append(("replace", item.scene() is None))
		return original_replace(observation, latch)
	controller.replace = replace
	tab._install_mutation_result(type(
		"Result", (), {"observation": _DocumentObservation(current)},
	)())
	assert events == [("publish", True), ("replace", True)]
	tab.dispose()


#============================================
def test_live_plan_publication_absence_and_failure_are_typed_recovery(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A missing or failing private publication entry point cannot paint a scene."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	class MissingPublication:
		def snapshot(self) -> _Snapshot:
			return current

		def _retire_live_document_smarts_query_v1(self) -> None:
			return None
	class FailingPublication(_Session):
		def _publish_live_render_plan_v1(self, _revision: int) -> _RenderObservation:
			raise RuntimeError("injected private publication failure")
	for session in (MissingPublication(), FailingPublication(current, current, True)):
		with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
				"Untitled", session, _Controller(),
			)


#============================================
def test_render_publication_routes_use_the_private_live_plan_transaction() -> None:
	"""Qt source cannot reintroduce a raw render observation publication route."""
	root = pathlib.Path(__file__).parents[1] / "ferrum_qt" / "ferrum"
	paths = (
		root / "document_tab.py",
		root / "document_tab_publication.py",
		root / "document_tab_construction.py",
	)
	sources = {path.name: path.read_text(encoding="utf-8") for path in paths}
	assert all("self._session.observe_render(" not in source for source in sources.values())
	assert sources["document_tab.py"].count("_publish_live_render_plan_v1(") >= 3
	assert "_publish_live_render_plan_v1(" in sources["document_tab_publication.py"]
	assert "_publish_live_render_plan_v1(" in sources["document_tab_construction.py"]
	transaction = (root / "live_document_transaction.py").read_text(encoding="utf-8")
	assert '"_publish_live_render_plan_v1"' in transaction
	assert '"_retire_live_document_smarts_query_v1"' in transaction
	assert "def _install_published_render_plan_v1(" in transaction
	assert "self._install_published_render_plan_v1(" in sources["document_tab.py"]


#============================================
def test_stale_live_overlay_completion_cannot_retire_newer_overlay(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An old asynchronous delivery token is a strict no-op after a rerun."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, _controller = _tab(current, current, True)
	old_item, old_token = _install_transient_overlay(tab)
	tab._begin_live_smarts_query_run_v1()
	new_item, new_token = _install_transient_overlay(tab)
	assert old_item.scene() is None and new_item.scene() is not None
	assert not tab._retire_if_current_live_run_v1(old_token, "stale_delivery")
	assert new_item.scene() is not None
	assert tab._retire_if_current_live_run_v1(new_token, "stale_delivery")
	assert new_item.scene() is None
	tab.dispose()


#============================================
def test_current_stale_delivery_receipt_retirement_failure_is_terminal(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A failed stale-delivery cleanup cannot claim its native receipt was revoked."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	events: list[str] = []
	session = _RetirementFailureSession(current, events)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, _Controller(),
	)
	item, token = _install_transient_overlay(tab)
	receipt = tab._live_smarts_receipt_v1
	session._should_fail = True
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._retire_if_current_live_run_v1(token, "stale_delivery")
	assert (
		events == ["native_retire"]
		and item.scene() is None
		and tab._live_smarts_overlay_item_v1 is item
		and tab._live_smarts_receipt_v1 is receipt
		and tab._live_smarts_active_run_token_v1 == token
		and not tab._live_smarts_retirement_available_v1
		and tab._live_smarts_retirement_error_v1 is not None
	)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._require_live_smarts_receipt_retirement_v1("stale_delivery")
	assert events == ["native_retire"]


#============================================
def test_live_smarts_rerun_retires_before_dispatch_and_install_does_not_retire(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The only ordinary retirement precedes a new bridge dispatch, not row paint."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	events: list[str] = []
	class _BridgeSequenceSession(_Session):
		"""Record the actual private bridge names used by the Qt lifecycle seam."""
		def _retire_live_document_smarts_query_v1(self) -> None:
			events.append("retire")
		def _run_live_document_smarts_query_v1(self, *_unused: object) -> object:
			events.append("run")
			return object()
	session = _BridgeSequenceSession(current, current, True)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, _Controller(),
	)
	events.clear()
	assert session._run_live_document_smarts_query_v1("C", 5, 20) is not None
	first_item, first_token = _install_transient_overlay(tab)
	assert events == ["run"] and tab._live_smarts_active_run_token_v1 == first_token
	tab._begin_live_smarts_query_run_v1()
	assert first_item.scene() is None and events == ["run", "retire"]
	assert session._run_live_document_smarts_query_v1("C", 5, 20) is not None
	second_item, second_token = _install_transient_overlay(tab)
	assert (
		second_item.scene() is not None
		and second_token != first_token
		and events == ["run", "retire", "run"]
	)
	tab.dispose()


#============================================
def test_fixture_receipt_cleanup_preserves_plan_for_rerun_but_full_retirement_refuses_it(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Fixture cleanup must model the native receipt and render-plan boundaries separately."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	class _ReceiptBoundarySession(_Session):
		"""Expose the plan and receipt states used by native query admission."""
		def __init__(self) -> None:
			"""Start with one published plan and no issued receipt."""
			super().__init__(current, current, True)
			self._plan_is_live = True
			self._receipt_is_live = False
		def _retire_live_document_smarts_query_v1(self) -> None:
			"""Model full retirement as revoking both native authorities."""
			self._plan_is_live = False
			self._receipt_is_live = False
		def _retire_live_document_smarts_receipts_v1(self) -> None:
			"""Delegate the narrow fixture hook without revoking the render plan."""
			super()._retire_live_document_smarts_receipts_v1()
			self._receipt_is_live = False
		def _run_live_document_smarts_query_v1(self) -> object:
			"""Issue a query receipt only while its render plan remains published."""
			if not self._plan_is_live:
				raise RuntimeError("fixture render plan is retired")
			self._receipt_is_live = True
			return object()
	session = _ReceiptBoundarySession()
	first_receipt = session._run_live_document_smarts_query_v1()
	session._retire_live_document_smarts_receipts_v1()
	second_receipt = session._run_live_document_smarts_query_v1()
	assert first_receipt is not second_receipt and session._receipt_is_live
	session._retire_live_document_smarts_query_v1()
	with pytest.raises(RuntimeError):
		session._run_live_document_smarts_query_v1()


#============================================
def test_live_smarts_install_over_active_run_retires_both_runs_fail_closed(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An omitted rerun fence cannot leave an old or newly issued receipt usable."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	events: list[str] = []
	class _RetirementSession(_Session):
		"""Expose retirement only through the real private session entry point."""
		def _retire_live_document_smarts_query_v1(self) -> None:
			events.append("retire")
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", _RetirementSession(current, current, True), _Controller(),
	)
	events.clear()
	old_item, _token = _install_transient_overlay(tab)
	new_item = _unattached_transient_overlay()
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._install_live_smarts_query_overlay_v1(new_item, object())
	assert (
		old_item.scene() is None
		and new_item.scene() is None
		and tab._live_smarts_receipt_v1 is None
		and tab._live_smarts_active_run_token_v1 is None
		and events == ["retire"]
	)
	tab.dispose()


#============================================
def test_live_overlay_replacement_keeps_one_run_for_multiple_unconsumed_rows(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""One live receipt can replace visual paint for separate native row redemptions."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, _controller = _tab(current, current, True)
	first_item, token = _install_transient_overlay(tab)
	receipt = tab._live_smarts_receipt_v1
	second_item = _unattached_transient_overlay()
	third_item = _unattached_transient_overlay()
	tab._replace_live_smarts_query_overlay_v1(second_item)
	assert (
		first_item.scene() is None
		and second_item.scene() is not None
		and tab._live_smarts_receipt_v1 is receipt
		and tab._live_smarts_active_run_token_v1 == token
	)
	tab._replace_live_smarts_query_overlay_v1(third_item)
	assert (
		second_item.scene() is None
		and third_item.scene() is not None
		and tab._live_smarts_receipt_v1 is receipt
		and tab._live_smarts_active_run_token_v1 == token
	)
	assert tab._retire_if_current_live_run_v1(token, "stale_delivery")
	assert (
		third_item.scene() is None
		and tab._live_smarts_overlay_item_v1 is None
		and tab._live_smarts_receipt_v1 is None
		and tab._live_smarts_active_run_token_v1 is None
	)
	tab.dispose()


#============================================
def test_live_overlay_restore_failure_retires_the_entire_run(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed restore cannot leave a valid receipt without owned paint."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, _controller = _tab(current, current, True)
	prior_item, _token = _install_transient_overlay(tab)
	candidate = _unattached_transient_overlay()
	def fail_attach(_scene: object, _item: object) -> None:
		"""Fail candidate attachment after detaching the current visual."""
		raise RuntimeError("injected candidate failure")
	def fail_restore(_scene: object, _item: object) -> bool:
		"""Model a destroyed old scene that cannot restore the old visual."""
		return False
	monkeypatch.setattr(tab, "_attach_live_smarts_overlay_item_v1", fail_attach)
	monkeypatch.setattr(tab, "_restore_replaced_live_smarts_overlay_item_v1", fail_restore)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._replace_live_smarts_query_overlay_v1(candidate)
	assert (
		prior_item.scene() is None
		and candidate.scene() is None
		and tab._live_smarts_overlay_item_v1 is None
		and tab._live_smarts_receipt_v1 is None
		and tab._live_smarts_active_run_token_v1 is None
	)
	tab.dispose()


#============================================
def _sealed_live_smarts_qt_session() -> object:
	"""Load one real installed ABI-5 bridge, never a source or fixture substitute."""
	configured = os.environ.get("FERRUM_SMARTS_QT_SEALED_WHEEL_ROOT")
	if not configured:
		pytest.skip("requires an isolated installed ABI-5 native and Qt wheel")
	root = pathlib.Path(configured).resolve()
	wheel = pathlib.Path(os.environ["FERRUM_SMARTS_QT_NATIVE_WHEEL"]).resolve()
	qt_wheel = pathlib.Path(os.environ["FERRUM_SMARTS_QT_WHEEL"]).resolve()
	for path, digest_name in (
		(wheel, "FERRUM_SMARTS_QT_NATIVE_WHEEL_SHA256"),
		(qt_wheel, "FERRUM_SMARTS_QT_WHEEL_SHA256"),
	):
		expected = os.environ.get(digest_name)
		if not path.is_file() or len(expected or "") != 64:
			raise RuntimeError("sealed SMARTS Qt artifact provenance is incomplete")
		if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
			raise RuntimeError("sealed SMARTS Qt artifact digest mismatch")
	import ferrum_chem
	import ferrum_qt
	if root not in pathlib.Path(ferrum_chem.__file__).resolve().parents:
		raise RuntimeError("sealed SMARTS Qt test imported a non-installed native bridge")
	if root not in pathlib.Path(ferrum_qt.__file__).resolve().parents:
		raise RuntimeError("sealed SMARTS Qt test imported a non-installed Qt wheel")
	session = ferrum_chem.DocumentSession.load(_LIVE_SMARTS_MULTIROW_CDML)
	session._publish_live_render_plan_v1(session.snapshot().revision)
	return session


#============================================
def _renderer_issued_smarts_overlay_item(paint: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
	"""Project only real private-bridge bounds into a noninteractive Qt overlay."""
	root = PySide6.QtWidgets.QGraphicsItemGroup()
	root.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
	for left, top, right, bottom in paint.atom_bounds:
		item = PySide6.QtWidgets.QGraphicsRectItem(root)
		item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		item.setRect(float(left), float(top), float(right - left), float(bottom - top))
	return root


#============================================
def test_sealed_live_bridge_multiple_rows_replay_and_restore_failure_retire(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Use the installed PyO3 bridge through two redemptions and fail closed."""
	del qapp
	import ferrum_chem
	native_session = _sealed_live_smarts_qt_session()
	class _RetirementSpy:
		"""Record only the actual private retirement call while delegating the wheel bridge."""
		def __init__(self, session: object) -> None:
			self._session = session
			self.calls = 0
		def __getattr__(self, name: str) -> object:
			return getattr(self._session, name)
		def _retire_live_document_smarts_query_v1(self) -> object:
			self.calls += 1
			return self._session._retire_live_document_smarts_query_v1()
	session = _RetirementSpy(native_session)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab._from_fixture(
		"Untitled", session, _Controller(),
	)
	calls_before_query = session.calls
	assert calls_before_query == 1
	scene = PySide6.QtWidgets.QGraphicsScene(tab)
	tab._view.setScene(scene)
	run = session._run_live_document_smarts_query_v1("[#6]", 3, 3)
	assert [(item.source_order, item.match_count) for item in run.molecules] == [
		(0, 1), (1, 1), (2, 1),
	]
	receipt = run.receipt
	first_item = _renderer_issued_smarts_overlay_item(
		session._show_live_document_smarts_match_v1(receipt, 0),
	)
	scene.addItem(first_item)
	token = tab._install_live_smarts_query_overlay_v1(first_item, receipt)
	assert session.calls == calls_before_query
	second_item = _renderer_issued_smarts_overlay_item(
		session._show_live_document_smarts_match_v1(receipt, 1),
	)
	tab._replace_live_smarts_query_overlay_v1(second_item)
	assert (
		first_item.scene() is None
		and second_item.scene() is scene
		and tab._live_smarts_receipt_v1 is receipt
		and tab._live_smarts_active_run_token_v1 == token
		and session.calls == calls_before_query
	)
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError,
			match="^SMARTS query cannot continue$"):
		session._show_live_document_smarts_match_v1(receipt, 0)
	assert tab._live_smarts_overlay_item_v1 is second_item
	candidate = _renderer_issued_smarts_overlay_item(
		session._show_live_document_smarts_match_v1(receipt, 2),
	)
	monkeypatch.setattr(tab, "_attach_live_smarts_overlay_item_v1",
		lambda _scene, _item: (_ for _ in ()).throw(RuntimeError("injected attach failure")))
	monkeypatch.setattr(tab, "_restore_replaced_live_smarts_overlay_item_v1",
		lambda _scene, _item: False)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._replace_live_smarts_query_overlay_v1(candidate)
	assert (
		second_item.scene() is None
		and candidate.scene() is None
		and tab._live_smarts_overlay_item_v1 is None
		and tab._live_smarts_receipt_v1 is None
		and tab._live_smarts_active_run_token_v1 is None
		and session.calls == calls_before_query + 1
	)
	for row_index in (0, 1, 2):
		with pytest.raises(ferrum_chem.LiveDocumentSmartsError,
				match="^SMARTS query cannot continue$"):
			session._show_live_document_smarts_match_v1(receipt, row_index)
	tab.dispose()


#============================================
def test_live_overlay_replacement_failure_restores_prior_visual_and_run(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A canvas attach failure cannot consume the prior visual or opaque run state."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, _controller = _tab(current, current, True)
	prior_item, token = _install_transient_overlay(tab)
	receipt = tab._live_smarts_receipt_v1
	candidate = _unattached_transient_overlay()
	def fail_attach(_scene: object, _item: object) -> None:
		"""Model a renderer scene ownership failure after prior paint is removed."""
		raise RuntimeError("injected scene attachment failure")
	monkeypatch.setattr(tab, "_attach_live_smarts_overlay_item_v1", fail_attach)
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab._replace_live_smarts_query_overlay_v1(candidate)
	assert (
		prior_item.scene() is not None
		and candidate.scene() is None
		and tab._live_smarts_overlay_item_v1 is prior_item
		and tab._live_smarts_receipt_v1 is receipt
		and tab._live_smarts_active_run_token_v1 == token
	)
	tab.dispose()


#============================================
def test_disposal_retires_live_overlay_before_controller_disposal(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Close/shutdown disposal cannot leave transient overlay paint in a scene."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, controller = _tab(current, current, True)
	item, _token = _install_transient_overlay(tab)
	tab.dispose()
	assert item.scene() is None and controller.disposed


#============================================
def test_valid_native_observation_installs_snapshot_and_dirty_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A valid Rust observation becomes the tab's user-visible backend state."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, controller = _tab(current, current, True)
	assert tab.current_snapshot is current and tab.is_dirty
	assert controller.installed is not None
	tab.dispose()


#============================================
def test_confirmed_save_adopts_saved_snapshot_and_destination_title(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Confirmed publication advances only with its matching render observation."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	saved = _Snapshot(4, "b" * 64, False)
	tab, controller = _tab(current, saved, True)
	tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert (
		tab.title == "ethanol.cdml"
		and tab.current_snapshot is saved
		and controller.installed.document.snapshot is saved
	)
	tab.dispose()


#============================================
def test_unconfirmed_save_preserves_dirty_snapshot_and_title(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""An uncertain publication cannot make the tab claim the document was saved."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	saved = _Snapshot(4, "b" * 64, False)
	tab, _controller = _tab(current, saved, False)
	tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert tab.title == "Untitled" and tab.current_snapshot is current and tab.is_dirty
	tab.dispose()


#============================================
def test_confirmed_save_with_rejected_refresh_preserves_prior_presentation(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A post-save projection failure cannot split tab and controller provenance."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	saved = _Snapshot(4, "b" * 64, False)
	tab, controller = _tab(current, saved, True, (True, False))
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert (
		tab.current_snapshot is current
		and controller.installed.document.snapshot is current
		and tab.title == "Untitled"
		and tab.file_path is None
	)
	tab.dispose()


#============================================
def test_disposal_rejects_late_native_operations(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A closed tab cannot revive projection delivery or save through a stale view."""
	del qapp
	current = _Snapshot(4, "a" * 64, True)
	tab, controller = _tab(current, current, True)
	tab.dispose()
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.save_atomic(str(tmp_path / "ethanol.cdml"))
	assert controller.disposed and controller.installed is not None


#============================================
def test_native_selection_edit_undo_redo_restores_durable_atom(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The public Ferrum loop changes one selected atom through Rust history."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	changed = tab.change_selected_atom_element("N").observation.snapshot
	undone = tab.undo().observation.snapshot
	redone = tab.redo().observation.snapshot
	selected = tab._controller.projection.selected_durable_targets()
	assert changed.revision < undone.revision < redone.revision and tab.is_dirty
	assert len(selected) == 1 and selected[0].kind == "atom" and selected[0].identifier == "atom-c"
	tab.dispose()


#============================================
def test_native_add_atom_uses_rust_identity_point_history_and_save(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One bounded insertion stays Rust-owned through selection, history, and reopen."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	choice = tab.durable_molecule_choices()[0]
	result = tab.add_atom_at(choice.object_id, "O", 30.5, 40.25)
	identifier = tab._controller.projection.selected_durable_targets()[0].identifier
	assert result.observation.snapshot.revision == 1 and identifier.startswith("ferrum-atom-v1-")
	assert 'x="30.5" y="40.25" z="0"' in result.observation.snapshot.cdml
	tab.undo()
	tab.redo()
	output = tmp_path / "inserted.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	reopened_ids = tuple(atom.source_id for atom in reopened.projection.molecules[0].atoms)
	assert identifier in reopened_ids and not tab.is_dirty
	tab.dispose()


#============================================
def test_native_add_atom_refuses_molecule_without_installed_rust_render_plan(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A rich-label molecule cannot allocate an invisible canvas-authored atom."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_UNRENDERABLE_LABEL_CDML, "unrenderable-label.cdml",
	)
	try:
		molecule_id = tab.durable_molecule_choices()[0].object_id
		assert not any(
			plan.molecule.id == molecule_id for plan in tab._render_observation.molecule_plans
		)
		before = tab.current_snapshot
		before_atom_ids = tuple(
			atom.source_id for atom in tab.current_document_observation().projection.molecules[0].atoms
		)
		before_selection = tab._controller.projection.selected_durable_targets()
		assert tab.canvas_authorable_molecule_choices() == ()
		with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabUnrenderableMoleculeError,
		) as raised:
			tab.add_atom_at(molecule_id, "O", 30.5, 40.25)
		assert raised.value.molecule_object_id == molecule_id
		after = tab.current_snapshot
		after_atom_ids = tuple(
			atom.source_id for atom in tab.current_document_observation().projection.molecules[0].atoms
		)
		assert after.revision == before.revision and after.digest == before.digest
		assert after.cdml == before.cdml and after_atom_ids == before_atom_ids
		assert tab._controller.projection.selected_durable_targets() == before_selection
		assert not tab.requires_refresh and not any(
			atom_id.startswith("ferrum-atom-v1-") for atom_id in after_atom_ids
		)
	finally:
		tab.dispose()


#============================================
def test_native_add_double_bond_uses_rust_identity_history_and_save(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Two selected atoms become one durable Rust bond through save and reopen."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	result = tab.add_bond_between_atoms(
		"atom-c", "atom-o", ferrum_chem.DocumentBondPresentationV1.normal_double,
	)
	selected_bond = tab.selected_bond_projection()
	bond_id = selected_bond.source_id
	assert result.observation.snapshot.revision == 1
	assert selected_bond.source_type == "n2"
	assert bond_id.startswith("ferrum-bond-v1-")
	tab.undo()
	assert "<bond" not in tab.current_snapshot.cdml
	tab.redo()
	output = tmp_path / "bonded.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	bond = reopened.projection.molecules[0].bonds[0]
	assert bond.source_id == bond_id and bond.source_type == "n2" and not tab.is_dirty
	tab.dispose()


#============================================
def test_native_add_single_bond_requires_exactly_two_selected_atoms(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Selection mistakes are rejected before a Rust candidate or revision exists."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond.cdml",
	)
	tab.select_atom("atom-c")
	before = tab.current_snapshot
	with pytest.raises(
		ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError,
	):
		tab.add_single_bond_between_selected_atoms()
	assert tab.current_snapshot is before and not tab.is_dirty and not tab.requires_refresh
	tab.dispose()


#============================================
def test_native_add_bonded_atom_is_one_rust_history_entry(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A new atom and its bond appear and disappear through one Rust revision."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bonded-atom.cdml",
	)
	import ferrum_chem
	result = tab.add_bonded_atom_at(
		"atom-c", "N", 60.0, 40.0,
		ferrum_chem.DocumentBondPresentationV1.normal_triple,
	)
	molecule = result.observation.projection.molecules[0]
	selected = tab._controller.projection.selected_durable_targets()
	assert result.observation.snapshot.revision == 1
	assert len(molecule.atoms) == 3 and len(molecule.bonds) == 1
	assert molecule.atoms[-1].source_id.startswith("ferrum-atom-v1-")
	assert (molecule.atoms[-1].position.x, molecule.atoms[-1].position.y) == (60.0, 40.0)
	assert molecule.bonds[0].source_id.startswith("ferrum-bond-v1-")
	assert molecule.bonds[0].source_type == "n3"
	assert len(selected) == 1 and selected[0].kind == "atom"
	undone = tab.undo().observation.projection.molecules[0]
	assert len(undone.atoms) == 2 and not undone.bonds
	redone = tab.redo().observation.projection.molecules[0]
	assert len(redone.atoms) == 3 and len(redone.bonds) == 1
	tab.dispose()


#============================================
def test_native_move_atom_uses_rust_position_history_and_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""One exact scene point becomes the authoritative Rust atom coordinate."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "move-atom.cdml",
	)
	result = tab.move_atom_to("atom-c", 55.5, 66.25)
	atom = result.observation.projection.molecules[0].atoms[0]
	selected = tab._controller.projection.selected_durable_targets()
	assert result.observation.snapshot.revision == 1
	assert (atom.position.x, atom.position.y, atom.position.z) == (55.5, 66.25, 0.0)
	assert len(selected) == 1 and selected[0].identifier == "atom-c"
	undone = tab.undo().observation.projection.molecules[0].atoms[0]
	assert (undone.position.x, undone.position.y) == (0.0, 0.0)
	redone = tab.redo().observation.projection.molecules[0].atoms[0]
	assert (redone.position.x, redone.position.y) == (55.5, 66.25)
	tab.dispose()


#============================================
def test_native_delete_atom_removes_incident_bonds_and_round_trips(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One selected atom deletion stays Rust-owned through undo and save."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "delete-atom.cdml",
	)
	tab.select_atoms(("atom-c", "atom-o"))
	tab.add_single_bond_between_selected_atoms()
	tab.select_atom("atom-o")
	deleted = tab.delete_selected_atom().observation
	molecule = deleted.projection.molecules[0]
	assert tuple(atom.source_id for atom in molecule.atoms) == ("atom-c",)
	assert not molecule.bonds
	restored = tab.undo().observation.projection.molecules[0]
	assert len(restored.atoms) == 2 and len(restored.bonds) == 1
	tab.redo()
	output = tmp_path / "deleted.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	assert tuple(atom.source_id for atom in reopened.projection.molecules[0].atoms) == ("atom-c",)
	assert not reopened.projection.molecules[0].bonds
	tab.dispose()


#============================================
def test_native_delete_bond_preserves_atoms_and_round_trips(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One selected bond deletion stays Rust-owned through undo and save."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "delete-bond.cdml",
	)
	tab.select_atoms(("atom-c", "atom-o"))
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	deleted = tab.delete_selected_bond().observation.projection.molecules[0]
	assert len(deleted.atoms) == 2 and not deleted.bonds
	restored = tab.undo().observation.projection.molecules[0]
	assert len(restored.atoms) == 2 and len(restored.bonds) == 1
	tab.redo()
	output = tmp_path / "deleted-bond.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	assert len(reopened.projection.molecules[0].atoms) == 2
	assert not reopened.projection.molecules[0].bonds
	tab.dispose()


#============================================
def test_native_bond_order_change_round_trips_and_selects_the_same_bond(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""One selected Rust bond becomes a visible double bond and remains undoable."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_BOND_CDML, "bond-order.cdml",
	)
	tab.select_atoms(("atom-c", "atom-o"))
	created = tab.add_single_bond_between_selected_atoms()
	bond_id = created.observation.projection.molecules[0].bonds[0].source_id
	tab.select_bond(bond_id)
	changed = tab.set_selected_bond_order(
		ferrum_chem.DocumentBondOrderV1.double,
	).observation
	assert changed.projection.molecules[0].bonds[0].source_type == "n2"
	selected = tab._controller.projection.selected_durable_targets()
	assert len(selected) == 1 and selected[0].identifier == bond_id
	plan = tab._session.observe_render(changed.snapshot.revision).molecule_plans[0].plan
	bond_batch = next(batch for batch in plan.batches if batch.target.record_id.kind == "Bond")
	assert len(bond_batch.operations) == 2
	assert tab.undo().observation.projection.molecules[0].bonds[0].source_type == "n1"
	assert tab.redo().observation.projection.molecules[0].bonds[0].source_type == "n2"
	output = tmp_path / "double-bond.cdml"
	tab.save_atomic(output)
	reopened = ferrum_chem.DocumentSession.load(output.read_text(encoding="utf-8")).observe(0)
	assert reopened.projection.molecules[0].bonds[0].source_type == "n2"
	tab.dispose()


#============================================
def test_native_add_atom_refresh_selects_only_after_replacement_succeeds(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An accepted insertion retains its Rust ID until Refresh installs its scene."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	choice = tab.durable_molecule_choices()[0]
	prior = tab.current_snapshot
	replace = tab._controller.replace
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	with pytest.raises(
		ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.add_atom_at(choice.object_id, "O", 30.5, 40.25)
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	assert tab._controller.projection.selected_durable_targets() == ()
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh
	selected = tab._controller.projection.selected_durable_targets()
	assert len(selected) == 1 and selected[0].identifier.startswith("ferrum-atom-v1-")
	tab.dispose()


#============================================
def test_accepted_mutation_projection_failure_blocks_save_until_exact_refresh(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		tmp_path: pathlib.Path,
		) -> None:
	"""A scene failure never hides an accepted Rust edit behind the prior display."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	replace = tab._controller.replace
	monkeypatch.setattr(tab._controller, "replace", lambda _observation, _latch: False)
	with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.change_selected_atom_element("N")
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	with pytest.raises(ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError):
		tab.save_atomic(str(tmp_path / "blocked-native-edit.cdml"))
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh
	tab.dispose()


#============================================
def test_stale_native_edit_keeps_the_installed_scene_truth(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A stale revision rejection cannot replace the current Ferrum projection."""
	del qapp
	import ferrum_chem
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	tab._session.submit(
		prior.revision, ferrum_chem.DocumentOperationV1.set_atom_element("atom-c", "N"),
	)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		tab.change_selected_atom_element("O")
	assert tab.current_snapshot is prior and not tab.requires_refresh
	tab.dispose()


#============================================
def test_post_accept_observe_exception_retains_authoritative_pending_state(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An observation exception after submit cannot make a clean tab closable."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	session = tab._session
	tab._session = _ObserveFailureSession(session)
	with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.change_selected_atom_element("N")
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	assert (
		tab._pending_snapshot.revision > prior.revision
		and tab._pending_snapshot.digest != prior.digest
	)
	tab._session = session
	assert tab.refresh_authoritative() and not tab.requires_refresh
	tab.dispose()


#============================================
def test_post_accept_replace_exception_retains_authority_until_refresh(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A controller exception after Rust acceptance preserves recovery ownership."""
	del qapp
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "editable.cdml",
	)
	tab.select_atom("atom-c")
	prior = tab.current_snapshot
	replace = tab._controller.replace
	def fail_replace(_observation: object, _latch: object) -> bool:
		"""Raise after the mutation result has become Rust authority."""
		raise RuntimeError("injected replacement failure")
	monkeypatch.setattr(tab._controller, "replace", fail_replace)
	with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError,
	):
		tab.change_selected_atom_element("N")
	assert tab.current_snapshot is prior and tab.requires_refresh and tab.is_dirty
	monkeypatch.setattr(tab._controller, "replace", replace)
	assert tab.refresh_authoritative() and not tab.requires_refresh
	tab.dispose()
