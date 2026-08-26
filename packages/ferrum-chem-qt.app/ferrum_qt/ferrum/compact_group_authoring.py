"""One-shot Rust-owned attached compact-group authoring."""

# Standard Library
import dataclasses
import functools
import math
import sys

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.graphics_disposal
import ferrum_qt.ferrum.direct_bond_overlay
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.interaction_action_handoff


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _AttachedCompactGroupChoice:
	"""One Rust-reviewed catalog key and its presentation label."""

	catalog_key: str
	label: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _AttachCompactGroupIntent:
	"""One fixed Rust selection fence awaiting its sole canvas release."""

	tab: object
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	anchor_object_id: str
	catalog_key: str
	label: str


#============================================
class _AttachCompactGroupUnavailableError(
		native_document_tab_errors.FerrumNativeDocumentTabError):
	"""Report an exact-current attached-group refusal without a mutation."""

	def __init__(self, label: str) -> None:
		"""Preserve the selected Rust-derived label for the public refusal."""
		super().__init__(f"Rust refused {label} attachment for the selected atom.")
		self.label = label


#============================================
class FerrumAttachCompactGroupDialog(PySide6.QtWidgets.QDialog):
	"""Present Rust-reviewed compact-group choices without authoring state."""

	def __init__(self, parent: PySide6.QtWidgets.QWidget,
			choices: tuple[_AttachedCompactGroupChoice, ...]) -> None:
		"""Build a chooser directly from the Rust-projected choice facts."""
		super().__init__(parent)
		self.setObjectName("attach-compact-group-chooser")
		self.setWindowTitle(self.tr("Attach Compact Group"))
		self.setAccessibleName(self.tr("Attach compact group to selected atom"))
		self.setModal(True)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		prompt = PySide6.QtWidgets.QLabel(self.tr("Available compact groups"), self)
		prompt.setObjectName("attach-compact-group-prompt")
		layout.addWidget(prompt)
		self.choice = PySide6.QtWidgets.QComboBox(self)
		self.choice.setObjectName("attach-compact-group-choice")
		self.choice.setAccessibleName(self.tr("Compact group"))
		for choice in choices:
			self.choice.addItem(choice.label, choice.catalog_key)
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

	def selected_choice(self) -> _AttachedCompactGroupChoice:
		"""Return the selected Rust-derived choice without resolving an alias."""
		catalog_key = self.choice.currentData()
		label = self.choice.currentText()
		if type(catalog_key) is not str or not catalog_key or not label:
			raise ValueError("Choose one compact group before attaching it.")
		return _AttachedCompactGroupChoice(catalog_key, label)


#============================================
class _AttachCompactGroupCapture(PySide6.QtCore.QObject):
	"""Own one viewport release without becoming a chemistry or geometry tool."""

	def __init__(self, owner: "FerrumNativeCompactGroupAuthoringWindowMixin",
			intent: _AttachCompactGroupIntent) -> None:
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
		"""Cancel pending authoring when Qt destroys the viewport."""
		if self._owner._compact_group_authoring_intent is self._intent:
			self._owner._cancel_compact_group_authoring()


#============================================
def _attached_compact_group_observation_is_current(facts: object, revision: int,
		digest: str, anchor_object_id: str) -> bool:
	"""Accept only an observation for the exact selected document state."""
	return (
		facts.revision == revision
		and facts.digest == digest
		and facts.anchor_object_id == anchor_object_id
	)


#============================================
def _attached_compact_group_choice_has_current_known_anchor(facts: object, revision: int,
		digest: str, anchor_object_id: str, catalog_key: str) -> bool:
	"""Accept a current known anchor, including an unavailable chemistry choice."""
	category = facts.category
	return (
		_attached_compact_group_observation_is_current(
			facts, revision, digest, anchor_object_id,
		)
		and facts.catalog_key == catalog_key
		and category is not type(category).unknown_anchor
	)


#============================================
class FerrumNativeCompactGroupAuthoringTabMixin:
	"""Keep attached compact-group Rust calls behind the native tab boundary."""

	def attached_compact_group_choices(self) -> tuple[_AttachedCompactGroupChoice, ...]:
		"""Return the current Rust-reviewed choice facts without a Python catalog."""
		facts = self._session._attached_compact_group_choices_v1()
		choices = tuple(
			_AttachedCompactGroupChoice(choice.catalog_key, choice.label)
			for choice in facts
		)
		if not choices:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum has no reviewed compact groups available for attachment.",
			)
		if any(not choice.catalog_key or not choice.label for choice in choices):
			raise RuntimeError("Ferrum returned an invalid compact-group choice.")
		return choices

	def attach_compact_group_availability(self, anchor_object_id: str,
			catalog_key: str) -> object:
		"""Observe one current durable atom's Rust-owned attachment admission."""
		self._require_mutable()
		if type(anchor_object_id) is not str or not anchor_object_id:
			raise ValueError("Select one durable atom before attaching a compact group.")
		if type(catalog_key) is not str or not catalog_key:
			raise ValueError("Choose one compact group before attaching it.")
		snapshot = self.current_snapshot
		return self._session._attach_compact_group_availability_v1(
			snapshot.revision, snapshot.digest, anchor_object_id, catalog_key,
		)

	def begin_attached_compact_group(self, revision: int, digest: str,
			anchor_object_id: str, catalog_key: str,
			release: PySide6.QtCore.QPointF) -> object:
		"""Start one opaque Rust candidate from frozen chooser and fence facts."""
		self._require_mutable()
		if type(revision) is not int or type(digest) is not str or not digest:
			raise ValueError("The compact-group request no longer has a valid document fence.")
		if type(anchor_object_id) is not str or not anchor_object_id:
			raise ValueError("Select one durable atom before attaching a compact group.")
		if type(catalog_key) is not str or not catalog_key:
			raise ValueError("Choose one compact group before attaching it.")
		if not math.isfinite(release.x()) or not math.isfinite(release.y()):
			raise ValueError("Choose a finite attachment direction and try again.")
		snapshot = self.current_snapshot
		if snapshot.revision != revision or snapshot.digest != digest:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"The document changed; choose Attach Compact Group again.",
			)
		return self._session._begin_attach_compact_group_v1(
			revision, digest, anchor_object_id, catalog_key,
			float(release.x()), float(release.y()),
		)

	def preview_attached_compact_group(self, pending: object) -> object:
		"""Return Rust-issued identifier-free overlay facts for one candidate."""
		self._require_mutable()
		return self._session._preview_attach_compact_group_v1(pending).overlay

	def commit_attached_compact_group(self, pending: object) -> object:
		"""Commit one opaque candidate through the Rust-owned session."""
		self._require_mutable()
		return self._session._commit_attach_compact_group_v1(pending)

	def install_attached_compact_group_result(self, result: object) -> None:
		"""Install one committed result through the tab's authoritative refresh seam."""
		self._refresh_from_current_revision()
		self._require_projection().select_durable((("atom", result.focus_object_id),))
		self.selection_changed.emit()

	def cancel_attached_compact_group(self, pending: object) -> None:
		"""Cancel one opaque Rust candidate without mutating the document."""
		self._session._cancel_attach_compact_group_v1(pending)


#============================================
class FerrumNativeCompactGroupAuthoringWindowMixin:
	"""Expose the Rust-owned attached compact-group workflow in Chemistry."""

	def _initialize_compact_group_authoring(self) -> None:
		"""Initialize one action, chooser, and one-shot pointer capture slot."""
		self._attach_compact_group_action: PySide6.QtGui.QAction | None = None
		self._compact_group_authoring_chooser: PySide6.QtWidgets.QDialog | None = None
		self._compact_group_authoring_intent: _AttachCompactGroupIntent | None = None
		self._compact_group_authoring_capture: _AttachCompactGroupCapture | None = None

	def _build_compact_group_authoring_action(self) -> None:
		"""Construct the accessible Rust-owned attached-group Chemistry action."""
		action = PySide6.QtGui.QAction(self.tr("Attach Compact Group..."), self)
		action.setObjectName("attach-compact-group-action")
		action.setIconText(self.tr("Attach Compact Group"))
		action.setStatusTip(self.tr(
			"Choose a compact group to attach to the selected atom.",
		))
		action.setToolTip(self.tr(
			"Choose a compact group, then release on the canvas to set its Rust-owned direction.",
		))
		action.setWhatsThis(self.tr(
			"Attach a Rust-reviewed compact group to the selected atom. Ferrum Rust "
			"validates chemistry, geometry, identifiers, and rendering before accepting "
			"the release.",
		))
		self._connect_interaction_action_v1(action, self._choose_compact_group_to_attach)
		self._attach_compact_group_action = action
		self._action_registry.register_existing(
			"chemistry.compact_group.attach", action,
			shortcut_exemption_reason="Available by its labelled Draw menu client.",
		)

	def _refresh_compact_group_authoring_action(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Enable only a current selected atom with a Rust-reviewed choice set."""
		action = self._attach_compact_group_action
		if action is None:
			return
		selection_guidance = self.tr(
			"Select exactly one atom before attaching a compact group.",
		)
		normal_status_tip = self.tr(
			"Choose a compact group to attach to the selected atom.",
		)
		normal_tool_tip = self.tr(
			"Choose a compact group, then release on the canvas to set its Rust-owned direction.",
		)
		normal_whats_this = self.tr(
			"Attach a Rust-reviewed compact group to the selected atom. Ferrum Rust "
			"validates chemistry, geometry, identifiers, and rendering before accepting "
			"the release.",
		)
		current = False
		if active and not pending and not busy_elsewhere and (
			self._compact_group_authoring_intent is None
		):
			tab = self._active_native_tab()
			try:
				if tab is not None:
					anchor_object_id = tab._selected_atom_identifier()
					snapshot = tab.current_snapshot
					choices = tab.attached_compact_group_choices()
					current = any(
						_attached_compact_group_choice_has_current_known_anchor(
							tab.attach_compact_group_availability(
								anchor_object_id, choice.catalog_key,
							),
							snapshot.revision, snapshot.digest, anchor_object_id,
							choice.catalog_key,
						)
						for choice in choices
					)
			except native_document_tab_errors.FerrumNativeDocumentTabError:
				current = False
		action.setEnabled(current)
		guidance = normal_status_tip if current else selection_guidance
		action.setStatusTip(guidance)
		action.setToolTip(normal_tool_tip if current else guidance)
		action.setWhatsThis(normal_whats_this if current else guidance)

	def _choose_compact_group_to_attach(self) -> None:
		"""Open the Rust-projected chooser after a fresh selected-atom check."""
		if self._compact_group_authoring_chooser is not None:
			return
		try:
			tab, revision, digest, anchor_object_id, choices = (
				self._current_attach_compact_group_target()
			)
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._show_edit_refusal(self._attach_compact_group_refusal(exc))
			self._refresh_actions()
			return
		dialog = FerrumAttachCompactGroupDialog(self, choices)
		dialog.finished.connect(functools.partial(
			self._finish_compact_group_choice, dialog, tab, revision, digest,
			anchor_object_id,
		))
		self._compact_group_authoring_chooser = dialog
		dialog.open()
		dialog.attach_button.setFocus()

	def _finish_compact_group_choice(self, dialog: FerrumAttachCompactGroupDialog,
			tab: object, revision: int, digest: str, anchor_object_id: str,
			result: int) -> None:
		"""Arm one release only after the chooser accepts one current choice."""
		if self._compact_group_authoring_chooser is dialog:
			self._compact_group_authoring_chooser = None
		if result != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			choice = dialog.selected_choice()
			self._require_current_attach_compact_group_target(
				tab, revision, digest, anchor_object_id, choice,
			)
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._show_edit_refusal(self._attach_compact_group_refusal(exc))
			self._refresh_actions()
			return
		self._cancel_compact_group_authoring()
		intent = _AttachCompactGroupIntent(
			tab, tab.view.viewport(), revision, digest, anchor_object_id,
			choice.catalog_key, choice.label,
		)
		capture = _AttachCompactGroupCapture(self, intent)
		intent.viewport.installEventFilter(capture)
		intent.viewport.setFocus()
		self._compact_group_authoring_intent = intent
		self._compact_group_authoring_capture = capture
		self.statusBar().showMessage(self.tr(
			f"Release once on the canvas to attach {choice.label} to the selected atom; "
			"Escape cancels.",
		), 5000)
		self._refresh_actions()

	def _current_attach_compact_group_target(
			self) -> tuple[object, int, str, str, tuple[_AttachedCompactGroupChoice, ...]]:
		"""Return one current atom, its fence, and Rust-reviewed choices."""
		tab, revision, digest, anchor_object_id = (
			self._current_attach_compact_group_fence()
		)
		choices = tab.attached_compact_group_choices()
		known_anchor = False
		available_choices = []
		for choice in choices:
			facts = tab.attach_compact_group_availability(
				anchor_object_id, choice.catalog_key,
			)
			if not _attached_compact_group_choice_has_current_known_anchor(
				facts, revision, digest, anchor_object_id, choice.catalog_key,
			):
				continue
			known_anchor = True
			if facts.available:
				available_choices.append(choice)
		if not known_anchor:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Select another atom before attaching a compact group.",
			)
		if not available_choices:
			raise _AttachCompactGroupUnavailableError(choices[0].label)
		return tab, revision, digest, anchor_object_id, tuple(available_choices)

	def _current_attach_compact_group_fence(self) -> tuple[object, int, str, str]:
		"""Return the live tab fence before interpreting a choice-specific fact."""
		tab = self._active_native_tab()
		if tab is None or self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Open a ready Ferrum drawing and select one atom first.",
			)
		anchor_object_id = tab._selected_atom_identifier()
		snapshot = tab.current_snapshot
		return tab, snapshot.revision, snapshot.digest, anchor_object_id

	def _attach_compact_group_refusal(self, exc: Exception) -> object:
		"""Use selected Rust-derived learner text for a current unavailable choice."""
		primary_message = None
		if isinstance(exc, _AttachCompactGroupUnavailableError):
			primary_message = self.tr(
				f"{exc.label} cannot attach to the selected atom. Select another atom "
				"and try again.",
			)
		return self._unavailable_edit_refusal(str(exc), primary_message)

	def _require_current_attach_compact_group_target(self, tab: object, revision: int,
			digest: str, anchor_object_id: str,
			choice: _AttachedCompactGroupChoice) -> None:
		"""Reject changed intent before consulting selected-choice chemistry facts."""
		current, current_revision, current_digest, current_anchor = (
			self._current_attach_compact_group_fence()
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
		choices = current.attached_compact_group_choices()
		if choice not in choices:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"The compact-group choices changed; choose Attach Compact Group again.",
			)
		facts = current.attach_compact_group_availability(current_anchor, choice.catalog_key)
		if not _attached_compact_group_choice_has_current_known_anchor(
			facts, current_revision, current_digest, current_anchor, choice.catalog_key,
		):
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"The document or selected atom changed; choose Attach Compact Group again.",
			)
		if not facts.available:
			raise _AttachCompactGroupUnavailableError(choice.label)

	def _compact_group_authoring_is_current(
			self, intent: _AttachCompactGroupIntent) -> bool:
		"""Return whether the one release still belongs to its live Rust fence."""
		try:
			self._require_current_attach_compact_group_target(
				intent.tab, intent.revision, intent.digest, intent.anchor_object_id,
				_AttachedCompactGroupChoice(intent.catalog_key, intent.label),
			)
		except (
			native_document_tab_errors.FerrumNativeDocumentTabError,
			ferrum_qt.ferrum.engine.RevisionConflictError,
		):
			return False
		return True

	def _complete_compact_group_authoring(self,
			intent: _AttachCompactGroupIntent,
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
		self._dispose_compact_group_capture(clear_status=False)
		pending = None
		preview = None
		succeeded = False
		native_failure = False
		try:
			release = intent.tab.view.mapToScene(event.position().toPoint())
			pending = intent.tab.begin_attached_compact_group(
				intent.revision, intent.digest, intent.anchor_object_id,
				intent.catalog_key, release,
			)
			overlay = intent.tab.preview_attached_compact_group(pending)
			preview = ferrum_qt.ferrum.direct_bond_overlay.create_overlay(intent.tab, overlay)
			result = intent.tab.commit_attached_compact_group(pending)
			pending = None
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			native_failure = True
			self._show_edit_refusal(self._attach_compact_group_refusal(exc))
		else:
			try:
				intent.tab.install_attached_compact_group_result(result)
			except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
				native_failure = True
				self._show_edit_refusal(self._attach_compact_group_refusal(exc))
			else:
				succeeded = True
		finally:
			if pending is not None:
				try:
					intent.tab.cancel_attached_compact_group(pending)
				except native_document_tab_errors.FerrumNativeDocumentTabError:
					if not native_failure and sys.exception() is None:
						raise
			if preview is not None:
				scene = ferrum_qt.canvas.graphics_disposal.native_scene_for_item(preview)
				if scene is not None:
					coordinator = (
						ferrum_qt.canvas.graphics_disposal.GraphicsDisposalCoordinator()
					)
					coordinator.dispose_scene_projection_items(scene, [preview])
			self._compact_group_authoring_intent = None
			self._refresh_actions()
		if succeeded:
			self.statusBar().showMessage(self.tr(
				f"Attached {intent.label} to the selected Ferrum atom.",
			), 5000)

	def _dispose_compact_group_capture(self, *, clear_status: bool) -> None:
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
		"""Cancel the one-shot release owner without changing Rust chemistry."""
		self._dispose_compact_group_capture(clear_status=clear_status)
		self._compact_group_authoring_intent = None
