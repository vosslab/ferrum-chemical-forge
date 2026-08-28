"""Closed Qt ports and immutable facts for local-document Open.

This module is deliberately presentation-only.  Rust continues to own file
admission, the prepared receipt, and document semantics; the controller owns
only the immutable request facts and the lifecycle transition around them.
"""

# Standard Library
import abc
import dataclasses
import enum
from collections.abc import Callable

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.ferrum.local_document_open_types
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.themes.document_display_palette


#============================================
class LocalDocumentOpenDisposition(enum.Enum):
	"""Qt-owned installation policy fixed before Rust admission begins."""

	NEW_TAB = enum.auto()
	REPLACE_PRISTINE_TARGET = enum.auto()
	REPLACE_EXPLICIT_CURRENT_TARGET = enum.auto()


#============================================
class LocalDocumentOpenOutcome(enum.Enum):
	"""Closed terminal classifications owned by the Open controller."""

	COMPLETED = enum.auto()
	FAILED = enum.auto()
	REFUSED = enum.auto()
	CANCELLED = enum.auto()


#============================================
class LocalOpenPostCommitPresentationError(Exception):
	"""Expected display failure after an irreversible Local Open commit."""


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class ExplicitReplacementFence:
	"""Qt facts proving one intentional populated-tab destination."""

	target: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	index: int
	revision: int
	digest: str
	dirty: bool
	file_path: str | None
	origin_token: object | None


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class LocalDocumentOpenRequest:
	"""One immutable local-document admission request before worker creation."""

	path: str
	descriptor: engine.LocalDocumentOpenDescriptorV2
	disposition: LocalDocumentOpenDisposition
	target: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None
	target_revision: int | None
	target_digest: str | None
	target_canvas_idle: bool
	source: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	focus_target: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None
	activate_if_still_current: bool
	recent_request: bool
	replacement_fence: ExplicitReplacementFence | None = None


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class LocalDocumentOpenIntent:
	"""One immutable admitted request and its exact worker/operation lease."""

	request: LocalDocumentOpenRequest
	lease: ferrum_qt.ferrum.operation_leases.OperationLease
	worker: ferrum_qt.ferrum.local_document_open_types.FerrumNativeLocalDocumentOpenWorker

	@property
	def path(self) -> str:
		"""Expose the request path without duplicating mutable controller state."""
		return self.request.path

	@property
	def descriptor(self) -> engine.LocalDocumentOpenDescriptorV2:
		"""Expose the Rust-issued descriptor captured at request time."""
		return self.request.descriptor

	@property
	def disposition(self) -> LocalDocumentOpenDisposition:
		"""Expose the immutable installation policy."""
		return self.request.disposition

	@property
	def target(self) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None:
		"""Expose the optional replacement destination."""
		return self.request.target

	@property
	def target_revision(self) -> int | None:
		"""Expose the captured pristine revision."""
		return self.request.target_revision

	@property
	def target_digest(self) -> str | None:
		"""Expose the captured pristine digest."""
		return self.request.target_digest

	@property
	def target_canvas_idle(self) -> bool:
		"""Expose the captured pointer-idle fact."""
		return self.request.target_canvas_idle

	@property
	def source(self) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Expose the exact lease source."""
		return self.request.source

	@property
	def focus_target(self) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None:
		"""Expose the focus fence."""
		return self.request.focus_target

	@property
	def activate_if_still_current(self) -> bool:
		"""Expose the non-focus-theft policy."""
		return self.request.activate_if_still_current

	@property
	def recent_request(self) -> bool:
		"""Expose recent-file provenance."""
		return self.request.recent_request

	@property
	def replacement_fence(self) -> ExplicitReplacementFence | None:
		"""Expose the deliberate populated-tab fence."""
		return self.request.replacement_fence


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class LocalOpenNewTabPublicationReceipt:
	"""Record one irreversible Local Open new-tab publication."""

	tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	index: int


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class LocalOpenReplacementCommitReceipt:
	"""Record one irreversible Local Open tab swap and settled source lease."""

	old: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	new: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	index: int
	lease_id: ferrum_qt.ferrum.operation_leases.OperationLeaseId
	tab_identity: ferrum_qt.ferrum.operation_leases.TabLeaseIdentity


#============================================
class LocalOpenPublicationResolution(abc.ABC):
	"""Resolve one host publication attempt exactly once.

	The host accepts only after irreversible publication.  A pre-commit refusal
	returns candidate ownership to delivery after host rollback is complete.
	"""

	#============================================
	@abc.abstractmethod
	def accept_publication(self, receipt: LocalOpenNewTabPublicationReceipt) -> None:
		"""Record one host-owned new-tab publication exactly once."""

	#============================================
	@abc.abstractmethod
	def refuse_publication(self) -> None:
		"""Return one unpublished candidate after complete host rollback."""


#============================================
class LocalOpenReplacementResolution(abc.ABC):
	"""Resolve one host replacement attempt exactly once.

	The host accepts only after the old tab is irreversibly replaced and its lease
	is settled.  A pre-commit refusal returns the candidate after full rollback.
	"""

	#============================================
	@abc.abstractmethod
	def accept_replacement(self, receipt: LocalOpenReplacementCommitReceipt) -> None:
		"""Record one host-owned replacement commit exactly once."""

	#============================================
	@abc.abstractmethod
	def refuse_replacement(self) -> None:
		"""Return one uncommitted replacement candidate after complete rollback."""


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class LocalDocumentOpenHost:
	"""Callback-only window port for one local-document Open controller.

	No window service locator belongs here.  Each callback documents a specific
	presentation or lifecycle capability that remains owned by the window.
	Publication and replacement callbacks return ``None`` and must resolve their
	one-shot capability.  An unresolved return or exception is an invariant fault;
	delivery conservatively retains the candidate because ownership is uncertain.
	"""

	parent: PySide6.QtWidgets.QMainWindow
	translate: Callable[[str], str]
	register_action: Callable[[str, PySide6.QtGui.QAction, str], None]
	action_refresh: Callable[[], None]
	active_tab: Callable[[], ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None]
	tab_is_registered: Callable[[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab], bool]
	tab_widget_current: Callable[[], PySide6.QtWidgets.QWidget | None]
	tab_widget_index: Callable[[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab], int]
	tab_widget_set_current_index: Callable[[int], None]
	publish_open_tab: Callable[
		[
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			LocalOpenPublicationResolution,
		],
		None,
	]
	finish_open_publication: Callable[[LocalOpenNewTabPublicationReceipt, bool], None]
	commit_open_replacement: Callable[
		[
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			int,
			ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability,
			ferrum_qt.ferrum.operation_leases.OperationLease,
			LocalOpenReplacementResolution,
		],
		None,
	]
	finish_open_replacement: Callable[[LocalOpenReplacementCommitReceipt], None]
	palette: Callable[[], ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1]
	present_refusal: Callable[[ferrum_qt.dialogs.refusal_presenter.RefusalRequest], None]
	show_status: Callable[[str, int], None]
	snapshot_busy: Callable[[], bool]
	shutdown_prepared: Callable[[], bool]
	tab_has_active_canvas_interaction: Callable[
		[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab], bool,
	]
	cancel_active_pointer_authoring: Callable[[], None]
	tab_has_active_operation: Callable[
		[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab], bool,
	]
	tab_has_conflict_except_lease: Callable[
		[
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			ferrum_qt.ferrum.operation_leases.OperationLease,
		],
		bool,
	]
	native_tab_for_origin_token: Callable[
		[object], ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None,
	]
	prompt_native_save: Callable[[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, bool], bool]
	save_native_tab_to_path: Callable[
		[ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab, str], bool,
	]
	record_recent_success: Callable[[str], None]
	handle_recent_failure: Callable[
		[str, ferrum_qt.ferrum.local_document_open_types.FerrumNativeLocalDocumentOpenFailure],
		bool,
	]
	emit_completed: Callable[[str, bool], None]
	emit_queue_drained: Callable[[bool], None]
