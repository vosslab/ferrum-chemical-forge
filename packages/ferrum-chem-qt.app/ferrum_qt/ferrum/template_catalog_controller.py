"""Explicit Qt controller for Rust-owned template catalog placement."""

# Standard Library
import dataclasses
import math
import collections.abc

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
from ferrum_qt.ferrum.document_tab_errors import (
	FerrumNativeDocumentTabError,
	FerrumNativeDocumentTabMutationPresentationError,
)
from ferrum_qt.ferrum.interaction_action_handoff import FerrumAdmittedInteractionCommand
from ferrum_qt.ferrum.operation_leases import ClosePolicy
from ferrum_qt.ferrum.operation_leases import LeaseState
from ferrum_qt.ferrum.operation_leases import OperationFamily
from ferrum_qt.ferrum.template_catalog_dialog import FerrumTemplateCatalogDialog


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _TemplateCatalogPlacement:
	"""Payload held by the controller while one lease owns canvas input."""

	lease: object
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	catalog_snapshot: object
	document_snapshot: object
	key: str
	mouse_tracking: bool
	had_explicit_cursor: bool
	cursor: PySide6.QtGui.QCursor


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class TemplateCatalogHost:
	"""Closed, window-owned seams the catalog controller may use."""

	parent: PySide6.QtWidgets.QMainWindow
	action_registry: object
	connect_action: collections.abc.Callable[[PySide6.QtGui.QAction, object], None]
	template_directory: collections.abc.Callable[[], str | None]
	active_tab: collections.abc.Callable[[], object | None]
	tab_is_registered: collections.abc.Callable[[object], bool]
	replace_authoring_owner: collections.abc.Callable[[], None]
	placement_compatible: collections.abc.Callable[[], bool]
	save_as_user_template: collections.abc.Callable[[], bool]
	refresh_actions: collections.abc.Callable[[], None]
	publish_installation: collections.abc.Callable[..., None]
	replacement_actions: collections.abc.Callable[[], tuple[PySide6.QtGui.QAction, ...]]


#============================================
class TemplateCatalogController(PySide6.QtCore.QObject):
	"""Own one catalog dialog and its temporary pointer-placement lifecycle."""

	def __init__(self, host: TemplateCatalogHost, registry: object) -> None:
		"""Install the catalog action against one exact Ferrum window registry."""
		if type(host) is not TemplateCatalogHost:
			raise TypeError("Ferrum template catalog requires an exact host port")
		if not isinstance(host.parent, PySide6.QtWidgets.QMainWindow):
			raise TypeError("Ferrum template catalog requires a QMainWindow host")
		super().__init__(host.parent)
		self._host = host
		self._registry = registry
		self._capability = registry.register_family(OperationFamily.TEMPLATE_CATALOG)
		self._snapshot: object | None = None
		self._dialog: FerrumTemplateCatalogDialog | None = None
		self._placements: dict[object, _TemplateCatalogPlacement] = {}
		self.action = PySide6.QtGui.QAction(self.tr("Template Catalog..."), self)
		self.action.setToolTip(self.tr(
			"Browse Rust-owned built-in and saved templates, then place one on the canvas",
		))
		host.connect_action(
			self.action,
			lambda _checked: FerrumAdmittedInteractionCommand(self.open),
		)
		host.action_registry.register_existing(
			"chemistry.template.catalog", self.action,
			shortcut_exemption_reason="Available by its labelled Chemistry menu client.",
		)

	def refresh_snapshot(self) -> object:
		"""Replace the retained immutable Rust snapshot for all catalog clients."""
		import ferrum_qt.ferrum.engine as engine
		directory = self._host.template_directory()
		snapshot = engine.snapshot_template_catalog_v1(
			None if directory is None else str(directory),
		)
		self._snapshot = snapshot
		if self._dialog is not None:
			self._dialog.replace_snapshot(snapshot)
		return snapshot

	def error_message(self, error: object) -> str:
		"""Map one typed Rust catalog error to a concrete author recovery."""
		messages = {
			("directory_open_failed", "fix_directory"): self.tr(
				"Ferrum cannot open the template directory. Fix it, then Refresh.",
			),
			("selection_not_found", "choose_entry"): self.tr(
				"The selected template is unavailable. Refresh and choose it again.",
			),
			("selection_snapshot_stale", "refresh"): self.tr(
				"The catalog changed. Refresh and choose the template again.",
			),
			("document_stale", "document_unchanged"): self.tr(
				"The document changed. Choose the template again; no change was made.",
			),
			("renderer_refused", "document_unchanged"): self.tr(
				"Ferrum could not render that template; no change was made.",
			),
			("invalid_point", "document_unchanged"): self.tr(
				"Ferrum cannot place a template at that canvas position.",
			),
			("catalog_limit_exceeded", "fix_file"): self.tr(
				"The template catalog reached a safety limit. Fix the file, then Refresh.",
			),
			("session_conflict", "document_unchanged"): self.tr(
				"The document changed. Choose the template again; no change was made.",
			),
		}
		return messages.get((error.category, error.recovery), self.tr(
			"Ferrum could not use this template. Refresh and choose it again.",
		))

	def open(self) -> None:
		"""Show the modeless catalog, cancelling its current pointer lease first."""
		if self._placements:
			self.cancel_active(reopen=True, message=self.tr(
				"Placement cancelled. Choose a template.",
			))
			return
		dialog = self._dialog
		if dialog is None:
			dialog = self._new_dialog()
			dialog.show()
			return
		dialog.showNormal()
		dialog.raise_()
		dialog.activateWindow()
		dialog.search.setFocus()

	def _new_dialog(self) -> FerrumTemplateCatalogDialog:
		"""Create the one retained modeless catalog dialog."""
		import ferrum_qt.ferrum.engine as engine
		unavailable_message: str | None = None
		try:
			snapshot = self.refresh_snapshot()
		except engine.TemplateCatalogError as error:
			snapshot = None
			unavailable_message = self.error_message(error)
		dialog = FerrumTemplateCatalogDialog(self._host.parent, snapshot)
		dialog.accepted.connect(self._arm_from_dialog)
		dialog.rejected.connect(self._close_dialog)
		dialog.refresh_button.clicked.connect(self._refresh_dialog)
		dialog.save_button.clicked.connect(self._save_and_refresh)
		self._dialog = dialog
		if unavailable_message is not None:
			dialog.set_unavailable(unavailable_message)
		return dialog

	def _close_dialog(self) -> None:
		"""Release the hidden dialog before a later explicit reopen."""
		if self._dialog is not None:
			self._dialog.deleteLater()
		self._dialog = None

	def _refresh_dialog(self) -> None:
		"""Refresh one visible dialog without re-entering the action seam."""
		import ferrum_qt.ferrum.engine as engine
		dialog = self._dialog
		if dialog is None:
			return
		dialog.set_refresh_busy(True)
		try:
			self.refresh_snapshot()
		except engine.TemplateCatalogError as error:
			dialog.set_unavailable(self.error_message(error))
		else:
			dialog.report_refresh_complete()
		finally:
			dialog.set_refresh_busy(False)

	def _save_and_refresh(self) -> None:
		"""Publish one user template, whose save flow refreshes this controller."""
		if self._host.save_as_user_template():
			dialog = self._dialog
			if dialog is not None and self._snapshot is not None:
				dialog.report_saved_and_refreshed()

	def _arm_from_dialog(self) -> None:
		"""Retain the dialog's opaque key until a single canvas disposition."""
		dialog = self._dialog
		if dialog is None:
			return
		key = dialog.selected_key()
		snapshot = dialog.selected_snapshot()
		if key is None or snapshot is None or not self.start_placement(snapshot, key):
			dialog.set_unavailable(self.tr(
				"The selected template is unavailable. Refresh and choose it again.",
			))
			dialog.show()
			dialog.search.setFocus()
			return
		dialog.hide()

	def wire_tool_replacement(self) -> None:
		"""Make selecting another pointer authoring tool cancel this lease."""
		for action in self._host.replacement_actions():
			action.toggled.connect(
				lambda checked: self.cancel_active(reopen=False) if checked else None,
			)

	def start_placement(self, catalog_snapshot: object, key: str) -> bool:
		"""Acquire one catalog lease and install its viewport-only input filter."""
		if self._placements or not self._host.placement_compatible():
			return False
		if type(key) is not str or not key:
			raise TypeError("Ferrum template placement requires an opaque Rust key")
		tab = self._host.active_tab()
		if tab is None or tab.requires_refresh:
			return False
		self._host.replace_authoring_owner()
		lease = self._registry.acquire(
			self._capability, tab=tab,
			close_policy=ClosePolicy.CANCEL_AND_BLOCK_TAB_CLOSE,
		)
		tab = lease.tab()
		snapshot = tab.current_snapshot
		viewport = tab.view.viewport()
		placement = _TemplateCatalogPlacement(
			lease, viewport, snapshot.revision, snapshot.digest, catalog_snapshot,
			snapshot, key, viewport.hasMouseTracking(),
			viewport.testAttribute(
				PySide6.QtCore.Qt.WidgetAttribute.WA_SetCursor,
			),
			PySide6.QtGui.QCursor(viewport.cursor()),
		)
		self._placements[lease.lease_id] = placement
		viewport.setMouseTracking(True)
		viewport.installEventFilter(self)
		viewport.setCursor(PySide6.QtCore.Qt.CursorShape.CrossCursor)
		viewport.setFocus()
		self._host.parent.statusBar().showMessage(self.tr(
			"Click canvas to place the template; Escape cancels and returns to the catalog.",
		))
		self._host.refresh_actions()
		return True

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Consume only this controller's exact viewport input events."""
		placement = self._placement_for_viewport(watched)
		if placement is None:
			return super().eventFilter(watched, event)
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			if event.key() == PySide6.QtCore.Qt.Key.Key_Escape:
				self.cancel_active(reopen=True)
				return True
		if event.type() == PySide6.QtCore.QEvent.Type.FocusOut:
			self.cancel_active(reopen=True)
			return False
		if event.type() != PySide6.QtCore.QEvent.Type.MouseButtonPress:
			return False
		if event.button() == PySide6.QtCore.Qt.MouseButton.RightButton:
			self.cancel_active(reopen=True)
		elif event.button() == PySide6.QtCore.Qt.MouseButton.LeftButton:
			self._commit(placement, event)
		else:
			return False
		return True

	def _placement_for_viewport(self, viewport: object) -> _TemplateCatalogPlacement | None:
		"""Find the one lease payload whose exact viewport received an event."""
		for placement in self._placements.values():
			if placement.viewport is viewport:
				return placement
		return None

	def _commit(self, placement: _TemplateCatalogPlacement,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Retire pointer state first, then submit exactly one fenced mutation."""
		import ferrum_qt.ferrum.engine as engine
		if not self._placement_current(placement):
			self._cancel_placement(placement, reopen=True, message=self.tr(
				"The document changed. Choose the template again; no change was made.",
			))
			return
		tab = placement.lease.tab()
		point = tab.view.snap_authored_scene_point(
			tab.view.mapToScene(event.position().toPoint()),
		)
		if not math.isfinite(point.x()) or not math.isfinite(point.y()):
			self._cancel_placement(placement, reopen=True, message=self.tr(
				"Ferrum cannot place a template at that canvas position.",
			))
			return
		self._deactivate_pointer(placement)
		try:
			commit = tab.place_template_catalog_entry(
				placement.catalog_snapshot, placement.key, placement.document_snapshot,
				float(point.x()), float(point.y()),
			)
		except engine.TemplateCatalogError as error:
			self._registry.settle(self._capability, placement.lease, LeaseState.REFUSED)
			self._retire_payload(placement)
			self._reopen(self.error_message(error))
			self._host.refresh_actions()
			return
		except FerrumNativeDocumentTabMutationPresentationError as error:
			self._registry.settle(self._capability, placement.lease, LeaseState.FAILED)
			self._retire_payload(placement)
			self._present_display_recovery(placement, error)
			return
		self._registry.settle(self._capability, placement.lease, LeaseState.COMPLETED)
		self._retire_payload(placement)
		self._host.parent.statusBar().showMessage(self.tr("Placed one Ferrum template."), 5000)
		self._host.refresh_actions()
		target = commit.result.observation.snapshot
		self._host.publish_installation(
			tab, "catalog_template", placement.revision, placement.digest,
			target.revision, target.digest, 1,
		)

	def _present_display_recovery(self, placement: _TemplateCatalogPlacement,
			error: object) -> None:
		"""Recover a committed native placement whose display install failed."""
		commit = error.accepted_receipt
		tab = placement.lease.tab()
		if tab.refresh_authoritative():
			self._host.parent.statusBar().showMessage(self.tr(
				"Template was placed; Ferrum refreshed the authoritative Rust display.",
			), 5000)
			target = commit.result.observation.snapshot
			self._host.publish_installation(
				tab, "catalog_template", placement.revision, placement.digest,
				target.revision, target.digest, 1,
			)
			return
		self._host.refresh_actions()
		self._reopen(self.tr("Template was placed, but the display still needs recovery."))

	def _placement_current(self, placement: _TemplateCatalogPlacement) -> bool:
		"""Require the exact registered current tab and original native fence."""
		tab = placement.lease.tab()
		if self._host.active_tab() is not tab:
			return False
		if not self._host.tab_is_registered(tab):
			return False
		if tab.is_disposed:
			return False
		try:
			snapshot = tab.current_snapshot
		except FerrumNativeDocumentTabError:
			return False
		return snapshot.revision == placement.revision and snapshot.digest == placement.digest

	def cancel_active(self, *, reopen: bool, message: str | None = None) -> None:
		"""Synchronously clean every retained pointer payload and settle cancelled."""
		for placement in tuple(self._placements.values()):
			self._cancel_placement(placement, reopen=False)
		if reopen:
			self._reopen(message or self.tr("Placement cancelled."))
		else:
			self._host.parent.statusBar().clearMessage()
		self._host.refresh_actions()

	def cancel_for_tab(self, tab: object, reason: str) -> bool:
		"""Cancel catalog pointer cleanup before an exact tab is disposed."""
		if type(reason) is not str or not reason:
			raise TypeError("Ferrum catalog cancellation requires a nonempty reason")
		cancelled = False
		for placement in tuple(self._placements.values()):
			if placement.lease.tab() is tab:
				self._cancel_placement(placement, reopen=False, reason=reason)
				cancelled = True
		return cancelled

	def _cancel_placement(self, placement: _TemplateCatalogPlacement, *, reopen: bool,
			message: str | None = None, reason: str = "pointer_cleanup") -> None:
		"""Dispose one exact viewport filter before terminally cancelling its lease."""
		self._deactivate_pointer(placement)
		self._registry.request_cancellation(self._capability, placement.lease, reason)
		self._registry.settle(self._capability, placement.lease, LeaseState.CANCELLED)
		self._retire_payload(placement)
		if reopen:
			self._reopen(message or self.tr("Placement cancelled."))

	def _deactivate_pointer(self, placement: _TemplateCatalogPlacement) -> None:
		"""Remove controller-only viewport capture without retiring lifecycle payload."""
		placement.viewport.removeEventFilter(self)
		placement.viewport.setMouseTracking(placement.mouse_tracking)
		if placement.had_explicit_cursor:
			placement.viewport.setCursor(placement.cursor)
		else:
			placement.viewport.unsetCursor()

	def _retire_payload(self, placement: _TemplateCatalogPlacement) -> None:
		"""Forget a payload only after its lease has reached a terminal state."""
		self._placements.pop(placement.lease.lease_id)

	def _reopen(self, message: str) -> None:
		"""Restore the existing modeless dialog and its keyboard starting point."""
		dialog = self._dialog
		if dialog is None:
			dialog = self._new_dialog()
		dialog.announce(message, focus_search=True)
		dialog.show()
		dialog.raise_()
		dialog.activateWindow()
		dialog.search.setFocus()

	def _record_recovery_without_focus(self, message: str) -> None:
		"""Retain stale-placement recovery without interrupting the current author."""
		dialog = self._dialog
		if dialog is not None:
			dialog.announce(message)
		self._host.parent.statusBar().showMessage(message, 5000)

	def refresh_action(self, active: bool, pending: bool, other_busy: bool) -> None:
		"""Refresh catalog eligibility from registry state and ordinary compatibility."""
		for placement in tuple(self._placements.values()):
			if not self._placement_current(placement):
				message = self.tr(
					"The document changed. Choose the template again; no change was made.",
				)
				if self._host.active_tab() is placement.lease.tab():
					self._cancel_placement(placement, reopen=True, message=message)
				else:
					self._cancel_placement(placement, reopen=False)
					self._record_recovery_without_focus(message)
		self.action.setEnabled(active and not pending and not other_busy)
