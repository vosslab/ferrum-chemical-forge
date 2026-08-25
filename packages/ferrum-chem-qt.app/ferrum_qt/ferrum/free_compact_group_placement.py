"""One-shot native placement of a free compact group on the Ferrum canvas."""

# Standard Library
import dataclasses
import functools
import math

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
_METHYL_CATALOG_KEY = "methyl"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _FreeCompactGroupPlacementIntent:
	"""One frozen native request awaiting exactly one canvas release."""

	tab: object
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	catalog_key: str


#============================================
class FerrumPlaceFreeCompactGroupDialog(PySide6.QtWidgets.QDialog):
	"""Present the initially supported free compact group without document state."""

	def __init__(self, parent: PySide6.QtWidgets.QWidget) -> None:
		"""Build the focused Me-only placement chooser."""
		super().__init__(parent)
		self.setObjectName("place-compact-group-chooser")
		self.setWindowTitle(self.tr("Place Compact Group"))
		self.setAccessibleName(self.tr("Place compact group on canvas"))
		self.setModal(True)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		prompt = PySide6.QtWidgets.QLabel(self.tr("Available compact group"), self)
		prompt.setObjectName("place-compact-group-prompt")
		layout.addWidget(prompt)
		self.choice = PySide6.QtWidgets.QLabel(self.tr("Me"), self)
		self.choice.setObjectName("place-compact-group-choice-me")
		self.choice.setAccessibleName(self.tr("Compact group Me"))
		layout.addWidget(self.choice)
		buttons = PySide6.QtWidgets.QDialogButtonBox(self)
		self.place_button = buttons.addButton(
			self.tr("Place on Canvas"),
			PySide6.QtWidgets.QDialogButtonBox.ButtonRole.AcceptRole,
		)
		self.place_button.setObjectName("place-compact-group-confirm")
		self.place_button.setAccessibleName(self.tr("Place on Canvas"))
		self.place_button.setDefault(True)
		self.cancel_button = buttons.addButton(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
		)
		self.cancel_button.setObjectName("place-compact-group-cancel")
		self.cancel_button.setAccessibleName(self.tr("Cancel"))
		buttons.accepted.connect(self.accept)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)


#============================================
class _FreeCompactGroupPlacementCapture(PySide6.QtCore.QObject):
	"""Own one canvas release for an already-confirmed free placement."""

	def __init__(self, owner: "FerrumNativeFreeCompactGroupPlacementWindowMixin",
			intent: _FreeCompactGroupPlacementIntent) -> None:
		"""Retain the window intent until a terminal Qt event occurs."""
		super().__init__(owner)
		self._owner = owner
		self._intent = intent
		intent.viewport.destroyed.connect(self._cancel)

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Consume Escape and the one terminal left-button release only."""
		if watched is not self._intent.viewport:
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			key_event = event
			if isinstance(key_event, PySide6.QtGui.QKeyEvent) and (
				key_event.key() == PySide6.QtCore.Qt.Key.Key_Escape
			):
				self._owner._cancel_free_compact_group_placement()
				return True
			return False
		if event.type() != PySide6.QtCore.QEvent.Type.MouseButtonRelease:
			return False
		mouse_event = event
		if not isinstance(mouse_event, PySide6.QtGui.QMouseEvent):
			return False
		if mouse_event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
			return False
		self._owner._complete_free_compact_group_placement(self._intent, mouse_event)
		return True

	@PySide6.QtCore.Slot()
	def _cancel(self) -> None:
		"""Release this capture when Qt destroys its owned viewport."""
		if self._owner._free_compact_group_placement_intent is self._intent:
			self._owner._cancel_free_compact_group_placement()


#============================================
class FerrumNativeFreeCompactGroupPlacementTabMixin:
	"""Keep free compact-group placement behind the native document-tab seam."""

	def begin_place_free_compact_group(self, revision: int, digest: str,
			catalog_key: str, scene_point: PySide6.QtCore.QPointF) -> object:
		"""Prepare one opaque Rust placement candidate from frozen public facts."""
		self._require_mutable()
		if type(revision) is not int or type(digest) is not str or not digest:
			raise ValueError("The compact-group request no longer has a valid document fence.")
		if type(catalog_key) is not str or not catalog_key:
			raise ValueError("Choose one compact group before placing it.")
		if type(scene_point) is not PySide6.QtCore.QPointF:
			raise TypeError("Ferrum compact-group placement requires a snapped scene point")
		if not math.isfinite(scene_point.x()) or not math.isfinite(scene_point.y()):
			raise ValueError("Choose a finite canvas location and try again.")
		snapshot = self.current_snapshot
		if snapshot.revision != revision or snapshot.digest != digest:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"The document changed; choose Place Compact Group again.",
			)
		return self._session._begin_place_free_compact_group_v1(
			revision, digest, catalog_key, float(scene_point.x()), float(scene_point.y()),
		)

	def commit_place_free_compact_group(self, pending: object) -> object:
		"""Commit one opaque free compact-group candidate through Rust."""
		self._require_mutable()
		return self._session._commit_place_free_compact_group_v1(pending)

	def cancel_place_free_compact_group(self, pending: object) -> None:
		"""Retire one opaque free compact-group candidate without mutation."""
		self._session._cancel_place_free_compact_group_v1(pending)

	def install_place_free_compact_group_result(self, result: object) -> None:
		"""Install the committed projection and select its durable compact group."""
		compact_group_object_id = result.compact_group_object_id
		if type(compact_group_object_id) is not str or not compact_group_object_id:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum did not return the placed compact-group identifier.",
			)
		self._install_mutation_result(
			result, durable_selection=(("compact_group", compact_group_object_id),),
		)


#============================================
class FerrumNativeFreeCompactGroupPlacementWindowMixin:
	"""Expose one independent free-Me placement workflow in Chemistry."""

	def _initialize_free_compact_group_placement(self) -> None:
		"""Initialize the action, chooser reference, and one-release capture owner."""
		self._place_free_compact_group_action: PySide6.QtGui.QAction | None = None
		self._free_compact_group_placement_chooser: PySide6.QtWidgets.QDialog | None = None
		self._free_compact_group_placement_intent: _FreeCompactGroupPlacementIntent | None = None
		self._free_compact_group_placement_capture: _FreeCompactGroupPlacementCapture | None = None

	def _build_free_compact_group_placement_action(
			self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the public checkable Chemistry action through the shared handoff."""
		action = PySide6.QtGui.QAction(self.tr("Place Compact Group..."), self)
		action.setObjectName("place-compact-group-action")
		action.setCheckable(True)
		action.setIconText(self.tr("Place Compact Group"))
		action.setStatusTip(self.tr("Choose Me, then release once on the canvas."))
		action.setToolTip(self.tr("Place Me at one snapped canvas location."))
		action.setWhatsThis(self.tr(
			"Place a free Me compact group. Ferrum Rust owns identifiers, orientation, "
			"history, chemistry, and rendering admission.",
		))
		self._connect_interaction_action_v1(action, self._choose_free_compact_group)
		action.toggled.connect(self._on_place_free_compact_group_toggled)
		self._add_interaction_action_to_menu_v1(menu, action)
		self._place_free_compact_group_action = action

	def _on_place_free_compact_group_toggled(self, checked: bool) -> None:
		"""Release this exact owner when the handoff unchecks its action."""
		if not checked:
			self._cancel_free_compact_group_placement()

	def _choose_free_compact_group(self, checked: bool = False) -> None:
		"""Open one chooser for the checked action after ready-tab validation."""
		if not checked:
			return
		if self._free_compact_group_placement_chooser is not None:
			return
		try:
			tab, revision, digest = self._current_free_compact_group_target()
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._clear_free_compact_group_action()
			self._refresh_actions()
			return
		dialog = FerrumPlaceFreeCompactGroupDialog(self)
		dialog.finished.connect(functools.partial(
			self._finish_free_compact_group_choice, dialog, tab, revision, digest,
		))
		self._free_compact_group_placement_chooser = dialog
		dialog.open()
		dialog.place_button.setFocus()

	def _finish_free_compact_group_choice(self, dialog: PySide6.QtWidgets.QDialog,
			tab: object, revision: int, digest: str, result: int) -> None:
		"""Freeze one ready tab and arm its sole left-button release after acceptance."""
		if self._free_compact_group_placement_chooser is dialog:
			self._free_compact_group_placement_chooser = None
		if result != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			self._clear_free_compact_group_action()
			self._refresh_actions()
			return
		try:
			self._require_current_free_compact_group_target(tab, revision, digest)
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._clear_free_compact_group_action()
			self._refresh_actions()
			return
		self._retire_free_compact_group_capture(clear_status=False)
		intent = _FreeCompactGroupPlacementIntent(
			tab, tab.view.viewport(), revision, digest, _METHYL_CATALOG_KEY,
		)
		capture = _FreeCompactGroupPlacementCapture(self, intent)
		intent.viewport.installEventFilter(capture)
		intent.viewport.setFocus()
		self._free_compact_group_placement_intent = intent
		self._free_compact_group_placement_capture = capture
		self.statusBar().showMessage(self.tr(
			"Release once on the canvas to place Me; Escape cancels.",
		), 5000)
		self._refresh_actions()

	def _current_free_compact_group_target(self) -> tuple[object, int, str]:
		"""Return the active mutable tab and one exact Rust document fence."""
		tab = self._active_native_tab()
		if tab is None or self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Open a ready Ferrum drawing before placing a compact group.",
			)
		if tab.requires_refresh:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Refresh the authoritative Rust view before placing a compact group.",
			)
		snapshot = tab.current_snapshot
		return tab, snapshot.revision, snapshot.digest

	def _require_current_free_compact_group_target(self, tab: object,
			revision: int, digest: str) -> None:
		"""Require the chooser and captured release to retain their exact tab fence."""
		current, current_revision, current_digest = self._current_free_compact_group_target()
		if current is not tab or current_revision != revision or current_digest != digest:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"The document changed; choose Place Compact Group again.",
			)

	def _free_compact_group_placement_is_current(
			self, intent: _FreeCompactGroupPlacementIntent) -> bool:
		"""Return whether one capture still owns an exact active native tab."""
		try:
			self._require_current_free_compact_group_target(
				intent.tab, intent.revision, intent.digest,
			)
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError):
			return False
		return True

	def _complete_free_compact_group_placement(self,
			intent: _FreeCompactGroupPlacementIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Map, snap, and submit exactly one release through the native tab seam."""
		if self._free_compact_group_placement_intent is not intent:
			return
		if not self._free_compact_group_placement_is_current(intent):
			self._cancel_free_compact_group_placement()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed; choose Place Compact Group again.",
			))
			self._refresh_actions()
			return
		self._retire_free_compact_group_capture(clear_status=False)
		pending = None
		succeeded = False
		try:
			scene_point = intent.tab.view.snap_authored_scene_point(
				intent.tab.view.mapToScene(event.position().toPoint()),
			)
			if not math.isfinite(scene_point.x()) or not math.isfinite(scene_point.y()):
				raise ValueError("Choose a finite canvas location and try again.")
			pending = intent.tab.begin_place_free_compact_group(
				intent.revision, intent.digest, intent.catalog_key, scene_point,
			)
			result = intent.tab.commit_place_free_compact_group(pending)
			pending = None
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			if pending is not None:
				intent.tab.cancel_place_free_compact_group(pending)
		else:
			try:
				intent.tab.install_place_free_compact_group_result(result)
			except (native_document_tab_errors.FerrumNativeDocumentTabError,
					TypeError, ValueError, RuntimeError) as exc:
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			else:
				succeeded = True
		finally:
			self._free_compact_group_placement_intent = None
			self._clear_free_compact_group_action()
			self._refresh_actions()
		if succeeded:
			self.statusBar().showMessage(self.tr("Placed Me on the Ferrum canvas."), 5000)

	def _retire_free_compact_group_capture(self, *, clear_status: bool) -> None:
		"""Detach only this workflow's viewport filter and Qt ownership."""
		intent = self._free_compact_group_placement_intent
		capture = self._free_compact_group_placement_capture
		self._free_compact_group_placement_capture = None
		if intent is not None and capture is not None:
			intent.viewport.removeEventFilter(capture)
			capture.deleteLater()
		if clear_status:
			self.statusBar().clearMessage()

	def _clear_free_compact_group_action(self) -> None:
		"""Clear the checked visual state without creating another authoring owner."""
		action = self._place_free_compact_group_action
		if action is not None and action.isChecked():
			action.setChecked(False)

	def _retire_free_compact_group_chooser(self) -> None:
		"""Reject the one live chooser before its action can lose interaction ownership."""
		dialog = self._free_compact_group_placement_chooser
		self._free_compact_group_placement_chooser = None
		if dialog is not None and dialog.isVisible():
			dialog.reject()

	def _cancel_free_compact_group_placement(self, clear_status: bool = True) -> None:
		"""Retire chooser/capture state without changing native chemistry."""
		self._retire_free_compact_group_chooser()
		self._retire_free_compact_group_capture(clear_status=clear_status)
		self._free_compact_group_placement_intent = None
		self._clear_free_compact_group_action()

	def _free_compact_group_placement_blocks_tab_close(self, tab: object) -> bool:
		"""Cancel this pending capture before a tab close can dispose its viewport."""
		intent = self._free_compact_group_placement_intent
		if intent is None or intent.tab is not tab:
			return False
		self._cancel_free_compact_group_placement()
		return True

	def _free_compact_group_placement_has_conflicting_owner(self) -> bool:
		"""Report another live canvas owner that must finish or be replaced first."""
		return any(getattr(self, attribute, None) is not None for attribute in (
			"_atom_insertion_intent",
			"_line_gesture_intent",
			"_structure_tab",
			"_compact_group_authoring_intent",
			"_catalog_placement_intent",
			"_user_template_placement_intent",
			"_direct_glycosidic_haworth_intent",
		))

	def _close_tab_at(self, index: int) -> None:
		"""Retire this capture for a closing tab before the ordinary lifecycle guard."""
		page = self._tab_widget.widget(index)
		tab = self._native_tabs_by_page.get(page)
		if tab is not None and self._free_compact_group_placement_blocks_tab_close(tab):
			return
		super()._close_tab_at(index)

	def _refresh_actions(self, *_unused: object) -> None:
		"""Include free placement in normal native action reachability."""
		super()._refresh_actions(*_unused)
		action = self._place_free_compact_group_action
		if action is None:
			return
		intent = self._free_compact_group_placement_intent
		if intent is not None and not self._free_compact_group_placement_is_current(intent):
			self._cancel_free_compact_group_placement()
		active = False
		try:
			self._current_free_compact_group_target()
		except (native_document_tab_errors.FerrumNativeDocumentTabError,
				TypeError, ValueError, RuntimeError):
			active = False
		else:
			active = (
				self._free_compact_group_placement_chooser is None
				and self._free_compact_group_placement_intent is None
				and not self._free_compact_group_placement_has_conflicting_owner()
			)
		action.setEnabled(active)
