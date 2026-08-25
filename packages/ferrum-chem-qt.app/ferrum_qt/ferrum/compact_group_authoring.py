"""One-shot Rust-owned attached-Me compact-group authoring."""

# Standard Library
import dataclasses
import functools
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.ferrum.direct_bond_overlay
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors
import ferrum_qt.ferrum.interaction_action_handoff


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _AttachMethylCompactGroupIntent:
	"""One fixed Rust selection fence awaiting its sole canvas release."""

	tab: object
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	anchor_object_id: str


#============================================
class FerrumAttachMethylCompactGroupDialog(PySide6.QtWidgets.QDialog):
	"""Present the one public compact-group choice without authoring state."""

	def __init__(self, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build the focused Me-only confirmation dialog."""
		super().__init__(parent)
		self.setObjectName("attach-compact-group-chooser")
		self.setWindowTitle(self.tr("Attach Compact Group"))
		self.setAccessibleName(self.tr("Attach compact group to selected atom"))
		self.setModal(True)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		prompt = PySide6.QtWidgets.QLabel(self.tr("Available compact group"), self)
		prompt.setObjectName("attach-compact-group-prompt")
		layout.addWidget(prompt)
		self.choice = PySide6.QtWidgets.QLabel(self.tr("Me"), self)
		self.choice.setObjectName("attach-compact-group-choice-me")
		self.choice.setAccessibleName(self.tr("Compact group Me"))
		layout.addWidget(self.choice)
		buttons = PySide6.QtWidgets.QDialogButtonBox(self)
		self.attach_button = buttons.addButton(
			self.tr("Attach to Selected Atom"),
			PySide6.QtWidgets.QDialogButtonBox.ButtonRole.AcceptRole,
		)
		self.attach_button.setObjectName("attach-compact-group-confirm")
		self.attach_button.setAccessibleName(self.tr("Attach to Selected Atom"))
		self.attach_button.setDefault(True)
		self.cancel_button = buttons.addButton(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
		)
		self.cancel_button.setObjectName("attach-compact-group-cancel")
		self.cancel_button.setAccessibleName(self.tr("Cancel"))
		buttons.accepted.connect(self.accept)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)


#============================================
class _AttachMethylCompactGroupCapture(PySide6.QtCore.QObject):
	"""Own one viewport release without becoming a chemistry or geometry tool."""

	def __init__(self, owner: "FerrumNativeCompactGroupAuthoringWindowMixin",
			intent: _AttachMethylCompactGroupIntent) -> None:
		"""Retain the one window intent until it reaches a terminal state."""
		super().__init__(owner)
		self._owner = owner
		self._intent = intent
		intent.viewport.destroyed.connect(self._cancel)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Consume only the terminal release or Escape for this captured viewport."""
		if watched is not self._intent.viewport:
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			key_event = event
			if isinstance(key_event, PySide6.QtGui.QKeyEvent) and (
				key_event.key() == PySide6.QtCore.Qt.Key.Key_Escape
			):
				self._owner._cancel_compact_group_authoring()
				return True
		if event.type() != PySide6.QtCore.QEvent.Type.MouseButtonRelease:
			return False
		mouse_event = event
		if not isinstance(mouse_event, PySide6.QtGui.QMouseEvent):
			return False
		if mouse_event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
			return False
		self._owner._complete_compact_group_authoring(self._intent, mouse_event)
		return True

	@PySide6.QtCore.Slot()
	def _cancel(self) -> None:
		"""Retire the pending authoring state when Qt destroys the viewport."""
		if self._owner._compact_group_authoring_intent is self._intent:
			self._owner._cancel_compact_group_authoring()


#============================================
def _attach_methyl_observation_is_current(facts: object, revision: int,
		digest: str, anchor_object_id: str) -> bool:
	"""Accept only an observation for the exact selected document state."""
	return (
		facts.revision == revision
		and facts.digest == digest
		and facts.anchor_object_id == anchor_object_id
	)


#============================================
class FerrumNativeCompactGroupAuthoringTabMixin:
	"""Keep attached-Me Rust calls behind the native document-tab boundary."""

	def attach_methyl_compact_group_availability(self, anchor_object_id: str) -> object:
		"""Observe one current durable atom's Rust-owned attached-Me admission."""
		self._require_mutable()
		if type(anchor_object_id) is not str or not anchor_object_id:
			raise ValueError("Select one durable atom before attaching a compact group.")
		snapshot = self.current_snapshot
		return self._session._attach_methyl_compact_group_availability_v1(
			snapshot.revision, snapshot.digest, anchor_object_id,
		)

	def begin_attached_methyl_compact_group(self, revision: int, digest: str,
			anchor_object_id: str, release: PySide6.QtCore.QPointF) -> object:
		"""Start one opaque Rust candidate from the chooser's frozen facts."""
		self._require_mutable()
		if type(revision) is not int or type(digest) is not str or not digest:
			raise ValueError("The compact-group request no longer has a valid document fence.")
		if type(anchor_object_id) is not str or not anchor_object_id:
			raise ValueError("Select one durable atom before attaching a compact group.")
		if not math.isfinite(release.x()) or not math.isfinite(release.y()):
			raise ValueError("Choose a finite attachment direction and try again.")
		snapshot = self.current_snapshot
		if snapshot.revision != revision or snapshot.digest != digest:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"The document changed; choose Attach Compact Group again.",
			)
		return self._session._begin_attach_methyl_compact_group_v1(
			revision, digest, anchor_object_id, float(release.x()), float(release.y()),
		)

	def preview_attached_methyl_compact_group(self, pending: object) -> object:
		"""Return Rust-issued identifier-free overlay facts for one candidate."""
		self._require_mutable()
		return self._session._preview_attach_methyl_compact_group_v1(pending).overlay

	def commit_attached_methyl_compact_group(self, pending: object) -> object:
		"""Commit one opaque candidate through the Rust-owned session."""
		self._require_mutable()
		return self._session._commit_attach_methyl_compact_group_v1(pending)

	def install_attached_methyl_compact_group_result(self, result: object) -> None:
		"""Install one committed result through the tab's authoritative refresh seam."""
		self._refresh_from_current_revision()
		self._require_projection().select_durable((
			("atom", result.focus_object_id),
		))
		self.selection_changed.emit()

	def cancel_attached_methyl_compact_group(self, pending: object) -> None:
		"""Retire one opaque Rust candidate without mutating the document."""
		self._session._cancel_attach_methyl_compact_group_v1(pending)


#============================================
class FerrumNativeCompactGroupAuthoringWindowMixin:
	"""Expose the limited attached-Me authoring workflow in Chemistry."""

	def _initialize_compact_group_authoring(self) -> None:
		"""Initialize one action, chooser, and one-shot pointer capture slot."""
		self._attach_compact_group_action: PySide6.QtGui.QAction | None = None
		self._compact_group_authoring_chooser: PySide6.QtWidgets.QDialog | None = None
		self._compact_group_authoring_intent: _AttachMethylCompactGroupIntent | None = None
		self._compact_group_authoring_capture: _AttachMethylCompactGroupCapture | None = None

	def _build_compact_group_authoring_action(
			self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the accessible Rust-owned attached-Me Chemistry action."""
		action = PySide6.QtGui.QAction(self.tr("Attach Compact Group..."), self)
		action.setObjectName("attach-compact-group-action")
		action.setIconText(self.tr("Attach Compact Group"))
		action.setStatusTip(self.tr(
			"Attach the available compact group to the one selected atom.",
		))
		action.setToolTip(self.tr(
			"Choose Me, then release on the canvas to set its Rust-owned direction.",
		))
		action.setWhatsThis(self.tr(
			"Attach Me to the selected atom. Ferrum Rust validates chemistry, "
			"geometry, identifiers, and rendering before accepting the release.",
		))
		self._connect_interaction_action_v1(action, self._choose_compact_group_to_attach)
		self._add_interaction_action_to_menu_v1(menu, action)
		self._attach_compact_group_action = action

	def _refresh_compact_group_authoring_action(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Enable only when Rust admits the exactly one selected durable atom."""
		action = self._attach_compact_group_action
		if action is None:
			return
		selection_guidance = self.tr(
			"Select exactly one atom before attaching a compact group.",
		)
		unavailable_guidance = self.tr(
			"Me cannot attach to the selected atom. Select another atom and try again.",
		)
		normal_status_tip = self.tr(
			"Attach the available compact group to the one selected atom.",
		)
		normal_tool_tip = self.tr(
			"Choose Me, then release on the canvas to set its Rust-owned direction.",
		)
		normal_whats_this = self.tr(
			"Attach Me to the selected atom. Ferrum Rust validates chemistry, "
			"geometry, identifiers, and rendering before accepting the release.",
		)
		current = False
		guidance = selection_guidance
		if active and not pending and not busy_elsewhere and (
			self._compact_group_authoring_intent is None
		):
			tab = self._active_native_tab()
			try:
				if tab is not None:
					anchor_object_id = tab._selected_atom_identifier()
					snapshot = tab.current_snapshot
					facts = tab.attach_methyl_compact_group_availability(anchor_object_id)
					current = _attach_methyl_observation_is_current(
						facts, snapshot.revision, snapshot.digest, anchor_object_id,
					)
					if current and not facts.available:
						guidance = unavailable_guidance
			except (native_document_tab_errors.FerrumNativeDocumentTabError,
					TypeError, ValueError, RuntimeError):
				current = False
		action.setEnabled(current)
		if current and facts.available:
			action.setStatusTip(normal_status_tip)
			action.setToolTip(normal_tool_tip)
			action.setWhatsThis(normal_whats_this)
		else:
			action.setStatusTip(guidance)
			action.setToolTip(guidance)
			action.setWhatsThis(guidance)

	def _choose_compact_group_to_attach(self) -> None:
		"""Open the focused Me-only chooser after a new Rust admission check."""
		if self._compact_group_authoring_chooser is not None:
			return
		try:
			tab, revision, digest, anchor_object_id = self._current_attach_methyl_target()
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return
		dialog = FerrumAttachMethylCompactGroupDialog(self)
		dialog.finished.connect(functools.partial(
			self._finish_compact_group_choice, dialog, tab, revision, digest,
			anchor_object_id,
		))
		self._compact_group_authoring_chooser = dialog
		dialog.open()
		dialog.attach_button.setFocus()

	def _finish_compact_group_choice(self, dialog: PySide6.QtWidgets.QDialog,
			tab: object, revision: int, digest: str, anchor_object_id: str,
			result: int) -> None:
		"""Arm one release only after the focused chooser accepts current facts."""
		if self._compact_group_authoring_chooser is dialog:
			self._compact_group_authoring_chooser = None
		if result != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			self._require_current_attach_methyl_target(
				tab, revision, digest, anchor_object_id,
			)
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return
		self._cancel_compact_group_authoring()
		intent = _AttachMethylCompactGroupIntent(
			tab, tab.view.viewport(), revision, digest, anchor_object_id,
		)
		capture = _AttachMethylCompactGroupCapture(self, intent)
		intent.viewport.installEventFilter(capture)
		intent.viewport.setFocus()
		self._compact_group_authoring_intent = intent
		self._compact_group_authoring_capture = capture
		self.statusBar().showMessage(self.tr(
			"Release once on the canvas to attach Me to the selected atom; Escape cancels.",
		), 5000)
		self._refresh_actions()

	def _current_attach_methyl_target(self) -> tuple[object, int, str, str]:
		"""Return one Rust-admitted atom and its frozen document fence."""
		tab = self._active_native_tab()
		if tab is None or self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Open a ready Ferrum drawing and select one atom first.",
			)
		anchor_object_id = tab._selected_atom_identifier()
		snapshot = tab.current_snapshot
		facts = tab.attach_methyl_compact_group_availability(anchor_object_id)
		if not _attach_methyl_observation_is_current(
				facts, snapshot.revision, snapshot.digest, anchor_object_id,
		):
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum cannot attach Me to the selected atom. Select another atom and try again.",
			)
		if not facts.available:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum cannot attach Me to the selected atom. Select another atom and try again.",
			)
		return tab, snapshot.revision, snapshot.digest, anchor_object_id

	def _require_current_attach_methyl_target(self, tab: object, revision: int,
			digest: str, anchor_object_id: str) -> None:
		"""Reject a chooser result when its tab, selection, or fence changed."""
		current, current_revision, current_digest, current_anchor = (
			self._current_attach_methyl_target()
		)
		if (
			current is not tab
			or current_revision != revision
			or current_digest != digest
			or current_anchor != anchor_object_id
		):
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"The selected atom changed; choose Attach Compact Group again.",
			)

	def _compact_group_authoring_is_current(
			self, intent: _AttachMethylCompactGroupIntent) -> bool:
		"""Return whether the one release still belongs to its live Rust fence."""
		try:
			self._require_current_attach_methyl_target(
				intent.tab, intent.revision, intent.digest, intent.anchor_object_id,
			)
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError):
			return False
		return True

	def _complete_compact_group_authoring(self,
			intent: _AttachMethylCompactGroupIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Begin, preview, and commit the sole captured release through Rust."""
		if self._compact_group_authoring_intent is not intent:
			return
		if not self._compact_group_authoring_is_current(intent):
			self._cancel_compact_group_authoring()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document or selected atom changed; choose Attach Compact Group again.",
			))
			self._refresh_actions()
			return
		self._retire_compact_group_capture(clear_status=False)
		pending = None
		preview = None
		succeeded = False
		try:
			release = intent.tab.view.mapToScene(event.position().toPoint())
			pending = intent.tab.begin_attached_methyl_compact_group(
				intent.revision, intent.digest, intent.anchor_object_id, release,
			)
			overlay = intent.tab.preview_attached_methyl_compact_group(pending)
			preview = ferrum_qt.ferrum.direct_bond_overlay.create_overlay(intent.tab, overlay)
			result = intent.tab.commit_attached_methyl_compact_group(pending)
			pending = None
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			if pending is not None:
				intent.tab.cancel_attached_methyl_compact_group(pending)
		else:
			try:
				intent.tab.install_attached_methyl_compact_group_result(result)
			except (native_document_tab_errors.FerrumNativeDocumentTabError,
					TypeError, ValueError, RuntimeError) as exc:
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			else:
				succeeded = True
		finally:
			if preview is not None:
				scene = ferrum_qt.canvas.graphics_retirement.native_scene_for_item(preview)
				if scene is not None:
					coordinator = (
						ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
					)
					coordinator.retire_scene_projection_items(scene, [preview])
			self._compact_group_authoring_intent = None
			self._refresh_actions()
		if succeeded:
			self.statusBar().showMessage(self.tr(
				"Attached Me to the selected Ferrum atom.",
			), 5000)

	def _retire_compact_group_capture(self, *, clear_status: bool) -> None:
		"""Detach only this one viewport filter and release its Qt ownership."""
		intent = self._compact_group_authoring_intent
		capture = self._compact_group_authoring_capture
		self._compact_group_authoring_capture = None
		if intent is not None and capture is not None:
			intent.viewport.removeEventFilter(capture)
			capture.deleteLater()
		if clear_status:
			self.statusBar().clearMessage()

	def _cancel_compact_group_authoring(self, clear_status: bool = True) -> None:
		"""Retire the one-shot release owner without changing Rust chemistry."""
		self._retire_compact_group_capture(clear_status=clear_status)
		self._compact_group_authoring_intent = None
