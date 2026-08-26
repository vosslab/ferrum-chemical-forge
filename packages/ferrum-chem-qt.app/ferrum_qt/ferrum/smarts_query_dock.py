"""Modeless SMARTS-query dock backed only by the private live-session bridge."""

# Standard Library
import math

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.ferrum.smarts_selected_root_capture


_PER_MOLECULE_LIMIT = 50
_TOTAL_LIMIT = 200


#============================================
class _FerrumSmartsRevealProjectionFailure(RuntimeError):
	"""Report a known local canvas or paint-projection failure."""


#============================================
class FerrumSmartsQueryController(PySide6.QtCore.QObject):
	"""Present copied live-query summaries without owning chemistry or document data."""

	def __init__(self, window: PySide6.QtWidgets.QMainWindow) -> None:
		super().__init__(window)
		self._window = window
		self._tab: object | None = None
		self._bound_tab: object | None = None
		self._receipt: object | None = None
		self._row_ordinals: dict[int, int] = {}
		self._overlay_visible = False
		self._invalidation_blocked = False
		self._busy = False
		self._run_token = 0
		self._action: PySide6.QtGui.QAction | None = None
		self._closed_failure_messages = self._resolve_closed_failure_messages()
		self._selected_capture = (
			ferrum_qt.ferrum.smarts_selected_root_capture.
			FerrumSmartsSelectedRootCaptureController(window, self)
		)
		window._register_pointer_capture_canceller_v1(
			self._selected_capture.cancel_for_pointer_authoring,
		)
		self._dock = PySide6.QtWidgets.QDockWidget(window.tr("SMARTS Query"), window)
		self._dock.setObjectName("smarts-query-dock")
		self._dock.setAccessibleName(window.tr("SMARTS Query"))
		self._dock.setAccessibleDescription(window.tr(
			"Find structural SMARTS matches in the current Ferrum drawing.",
		))
		self._dock.setAllowedAreas(PySide6.QtCore.Qt.DockWidgetArea.RightDockWidgetArea)
		self._dock.setFeatures(
			PySide6.QtWidgets.QDockWidget.DockWidgetFeature.DockWidgetClosable
			| PySide6.QtWidgets.QDockWidget.DockWidgetFeature.DockWidgetMovable,
		)
		self._dock.visibilityChanged.connect(self._on_visibility_changed)
		self._dock.installEventFilter(self)
		self._build_widgets()
		window.addDockWidget(PySide6.QtCore.Qt.DockWidgetArea.RightDockWidgetArea, self._dock)
		self._dock.hide()

	#============================================
	@property
	def dock(self) -> PySide6.QtWidgets.QDockWidget:
		"""Return the one window-owned modeless dock."""
		return self._dock

	#============================================
	def install_action(self) -> PySide6.QtGui.QAction:
		"""Construct the one canonical Chemistry menu command."""
		if self._action is not None:
			return self._action
		action = PySide6.QtGui.QAction(self.tr("SMARTS Query..."), self._window)
		action.setObjectName("smarts-query-action")
		action.setToolTip(self.tr("Find SMARTS matches in the current drawing"))
		action.setStatusTip(self.tr("Open the SMARTS query panel for the current drawing"))
		action.setWhatsThis(self.tr(
			"Find structural SMARTS matches in the current Ferrum drawing.",
		))
		action.setShortcut(PySide6.QtGui.QKeySequence("Ctrl+Shift+F"))
		action.setShortcutContext(PySide6.QtCore.Qt.ShortcutContext.WindowShortcut)
		action.triggered.connect(self.open)
		self._window._register_action("chemistry.smarts.query", action)
		self._action = action
		return action

	#============================================
	def open(self) -> None:
		"""Show the dock without moving an intentional canvas capture away from it."""
		if not self._invalidation_blocked:
			self._activate_current_tab()
		self._dock.show()
		self._dock.raise_()
		if self._selected_capture.is_armed_v1():
			return
		if self._raw_source.isChecked():
			self._raw_input.setFocus()
		else:
			self._selected_source.setFocus()

	#============================================
	def on_tab_changed(self) -> None:
		"""Activate the current tab without taking native invalidation ownership."""
		self._activate_after_tab_switch_v1()

	#============================================
	def _deactivate_after_tab_invalidation_v1(self, invalidation_succeeded: bool) -> None:
		"""Release local state only after the window's native invalidation succeeds."""
		if not invalidation_succeeded:
			self._block_receipt_invalidation_v1(self.tr(
				"SMARTS results remain live in the previous drawing. Open this panel and choose "
				"Clear results to retry, or refresh that drawing before editing.",
			))
			self._dock.hide()
			return
		self._run_token += 1
		self._busy = False
		self._cancel_selected_capture_v1(None)
		self._clear_selected_query_token_v1()
		self._finish_receipt_invalidation_v1()
		self._update_controls(terminal_status=self.tr(
			"The active drawing changed. Run the query again.",
		))
		self._dock.hide()
		self._activate_after_tab_switch_v1()

	#============================================
	def _activate_after_tab_switch_v1(self) -> None:
		"""Bind the incoming tab after the native window completed its switch fence."""
		if not self._invalidation_blocked:
			self._activate_current_tab()

	#============================================
	def close(self) -> None:
		"""Clear a document-local receipt before the window begins disposal."""
		self._cancel_selected_capture_v1(None)
		self._clear_selected_query_token_v1()
		if not self._clear_results("tab_disposed", status=None):
			return
		self._dock.hide()

	#============================================
	def refresh_action(self, active: bool, pending: bool, busy: bool) -> None:
		"""Keep discovery available while refusing a query against an unready tab."""
		if self._action is not None:
			self._action.setEnabled(active and not pending and not busy)
		status = self.tr("Open a ready Ferrum drawing to search it.") if not active or pending else None
		self._update_controls(terminal_status=status)

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Scope Escape to this dock without stealing ordinary canvas shortcuts."""
		if event.type() != PySide6.QtCore.QEvent.Type.KeyPress:
			return super().eventFilter(watched, event)
		if event.key() != PySide6.QtCore.Qt.Key.Key_Escape:
			return super().eventFilter(watched, event)
		if watched in (self._results, self._results.viewport()):
			item = self._results.currentItem()
			if self._overlay_visible and item is not None and id(item) in self._row_ordinals:
				self._clear_only_overlay()
				return True
		if watched not in (
				self._dock, self._raw_input, self._raw_source, self._selected_source,
				self._find_button, self._clear_button, self._results,
				self._results.viewport(),
			):
			return super().eventFilter(watched, event)
		if self._clear_results("dock_rerun", status=self.tr("SMARTS results cleared.")):
			self._raw_input.setFocus()
			return True
		return super().eventFilter(watched, event)

	#============================================
	def _build_widgets(self) -> None:
		"""Build a compact, keyboard-first presentation surface."""
		content = PySide6.QtWidgets.QWidget(self._dock)
		layout = PySide6.QtWidgets.QVBoxLayout(content)
		layout.setContentsMargins(10, 10, 10, 10)
		layout.setSpacing(8)
		source_group = PySide6.QtWidgets.QGroupBox(self.tr("Query source"), content)
		source_group.setAccessibleName(self.tr("Query source"))
		source_layout = PySide6.QtWidgets.QVBoxLayout(source_group)
		self._raw_source = PySide6.QtWidgets.QRadioButton(self.tr("Enter SMARTS"), source_group)
		self._raw_source.setAccessibleName(self.tr("Enter SMARTS"))
		self._selected_source = PySide6.QtWidgets.QRadioButton(
			self.tr("Use chosen molecule"), source_group,
		)
		self._selected_source.setAccessibleName(self.tr("Use chosen molecule"))
		self._selected_source.setAccessibleDescription(self.tr(
			"Ferrum derives the selected molecule query privately and does not show its expression.",
		))
		self._raw_source.setChecked(True)
		source_layout.addWidget(self._raw_source)
		source_layout.addWidget(self._selected_source)
		layout.addWidget(source_group)
		self._choose_molecule_button = PySide6.QtWidgets.QPushButton(
			self.tr("Choose molecule on canvas"), content,
		)
		self._choose_molecule_button.setObjectName("smarts-query-choose-molecule")
		self._choose_molecule_button.setAccessibleName(self.tr("Choose molecule on canvas"))
		self._choose_molecule_button.setAccessibleDescription(self.tr(
			"Choose exactly one direct molecule for the private SMARTS query source.",
		))
		layout.addWidget(self._choose_molecule_button)
		self._raw_input = PySide6.QtWidgets.QLineEdit(content)
		self._raw_input.setObjectName("smarts-query-input")
		self._raw_input.setPlaceholderText(self.tr("SMARTS expression"))
		self._raw_input.setAccessibleName(self.tr("SMARTS expression"))
		self._raw_input.setClearButtonEnabled(True)
		layout.addWidget(self._raw_input)
		buttons = PySide6.QtWidgets.QHBoxLayout()
		self._find_button = PySide6.QtWidgets.QPushButton(self.tr("Find"), content)
		self._find_button.setObjectName("smarts-query-find")
		self._find_button.setAccessibleName(self.tr("Find SMARTS matches"))
		self._find_button.setDefault(True)
		self._clear_button = PySide6.QtWidgets.QPushButton(self.tr("Clear results"), content)
		self._clear_button.setAccessibleName(self.tr("Clear SMARTS results"))
		buttons.addWidget(self._find_button)
		buttons.addWidget(self._clear_button)
		layout.addLayout(buttons)
		self._status = PySide6.QtWidgets.QLabel(content)
		self._status.setObjectName("smarts-query-status")
		self._status.setWordWrap(True)
		self._status.setAccessibleName(self.tr("SMARTS query status"))
		self._status.setTextInteractionFlags(
			PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByKeyboard,
		)
		layout.addWidget(self._status)
		self._results = PySide6.QtWidgets.QTreeWidget(content)
		self._results.setObjectName("smarts-query-results")
		self._results.setAccessibleName(self.tr("SMARTS query results"))
		self._results.setHeaderHidden(True)
		self._results.setRootIsDecorated(True)
		self._results.setSelectionMode(
			PySide6.QtWidgets.QAbstractItemView.SelectionMode.SingleSelection,
		)
		self._raw_input.installEventFilter(self)
		self._raw_source.installEventFilter(self)
		self._selected_source.installEventFilter(self)
		self._find_button.installEventFilter(self)
		self._clear_button.installEventFilter(self)
		self._results.installEventFilter(self)
		self._results.viewport().installEventFilter(self)
		layout.addWidget(self._results, 1)
		self._dock.setWidget(content)
		self._raw_source.toggled.connect(self._source_changed)
		self._selected_source.toggled.connect(self._source_changed)
		self._raw_input.textChanged.connect(self._on_raw_input_changed)
		self._raw_input.returnPressed.connect(self._begin_run)
		self._find_button.clicked.connect(self._begin_run)
		self._choose_molecule_button.clicked.connect(self._begin_selected_capture_v1)
		self._clear_button.clicked.connect(
			lambda: self._clear_results("dock_rerun", status=self.tr("SMARTS results cleared.")),
		)
		# itemActivated is the one canonical user route for mouse and keyboard activation.
		# ASVS 2.3.1: redeem each opaque Rust receipt through one ordered UI path.
		self._results.itemActivated.connect(self._show_item)
		self._set_status(self.tr("Open a ready Ferrum drawing to search it."))

	#============================================
	def _activate_current_tab(self) -> None:
		"""Bind only the current tab; old results are never restored across tabs."""
		tab = self._window._active_native_tab()
		self._tab = None if tab is None or tab.is_disposed or tab.requires_refresh else tab
		self._bind_active_tab_invalidation()
		self._update_controls()

	#============================================
	def _bind_active_tab_invalidation(self) -> None:
		"""Let only the active tab invalidate this dock's copied presentation state."""
		if self._bound_tab is self._tab:
			return
		if self._bound_tab is not None:
			self._bound_tab._bind_live_smarts_invalidation_callback_v1(None)
		self._bound_tab = self._tab
		if self._bound_tab is not None:
			self._bound_tab._bind_live_smarts_invalidation_callback_v1(
				self._on_live_smarts_query_invalidated_v1,
			)

	#============================================
	def _on_live_smarts_query_invalidated_v1(self) -> None:
		"""Clear only copied dock facts after a tab transition already succeeded."""
		self._run_token += 1
		self._busy = False
		self._receipt = None
		self._cancel_selected_capture_v1(None)
		self._clear_selected_query_token_v1()
		self._overlay_visible = False
		self._row_ordinals.clear()
		self._results.clear()
		self._invalidation_blocked = False
		self._update_controls(
			terminal_status=self.tr("The drawing changed. Run the query again."),
		)

	#============================================
	def _source_changed(self, _checked: bool) -> None:
		"""Switch presentation source without inspecting or deriving a query locally."""
		if self._raw_source.isChecked():
			self._raw_input.setFocus()
		else:
			availability = self._selected_availability()
			if not availability.available:
				self._set_status(self._selected_recovery_guidance(availability))
		self._update_controls()

	#============================================
	def _on_raw_input_changed(self, _text: str) -> None:
		"""Refresh eligibility after user input without interpreting SMARTS locally."""
		self._update_controls()

	#============================================
	def _begin_run(self) -> None:
		"""Fence an older run before a deferred, non-reentrant native dispatch."""
		if self._busy or self._tab is None or self._invalidation_blocked:
			return
		query: str | None = None
		selected_mode = not self._raw_source.isChecked()
		if self._raw_source.isChecked():
			query = self._raw_input.text()
			if not query.strip():
				self._set_status(self.tr("Enter a SMARTS expression, then choose Find."))
				self._raw_input.setFocus()
				return
		else:
			availability = self._selected_availability()
			if not availability.available:
				self._set_status(self._selected_recovery_guidance(availability))
				self._update_controls()
				return
		if not self._clear_results("dock_rerun", status=None):
			return
		self._busy = True
		self._run_token += 1
		token = self._run_token
		self._set_status(self.tr("Searching the current drawing..."))
		self._update_controls()
		PySide6.QtCore.QTimer.singleShot(
			0, lambda: self._dispatch_run(token, query, selected_mode),
		)

	#============================================
	def _dispatch_run(self, token: int, query: str | None, selected_mode: bool) -> None:
		"""Run one private bridge operation after the UI has painted its busy state."""
		if token != self._run_token or self._tab is None:
			return
		try:
			self._tab._begin_live_smarts_query_run_v1()
			if selected_mode:
				run = self._selected_capture.consume_selected_query_v1(
					self._tab, _PER_MOLECULE_LIMIT, _TOTAL_LIMIT,
				)
			else:
				run = self._tab._session._run_live_document_smarts_query_v1(
					query, _PER_MOLECULE_LIMIT, _TOTAL_LIMIT,
				)
		except engine.LiveDocumentSmartsError as error:
			if token == self._run_token:
				self._present_error(error)
			return
		if token != self._run_token:
			return
		self._busy = False
		self._receipt = run.receipt
		self._overlay_visible = False
		status = self._populate_results(run)
		self._update_controls(terminal_status=status)

	#============================================
	def _populate_results(self, run: object) -> str:
		"""Render copied summary facts in native-returned source order only."""
		self._results.clear()
		self._row_ordinals.clear()
		ordinal = 0
		molecule_count = 0
		match_count = 0
		truncated = False
		for molecule in run.molecules:
			molecule_count += 1
			count = int(molecule.match_count)
			match_count += count
			is_truncated = molecule.completeness == "truncated"
			truncated = truncated or is_truncated
			label = self.tr("Molecule {0}: {1} match(es){2}").format(
				molecule_count, count,
				self.tr("; additional matches not shown") if is_truncated else "",
			)
			group = PySide6.QtWidgets.QTreeWidgetItem((label,))
			group.setFlags(group.flags() & ~PySide6.QtCore.Qt.ItemFlag.ItemIsSelectable)
			group.setData(
				0, PySide6.QtCore.Qt.ItemDataRole.AccessibleTextRole, label,
			)
			group.setData(
				0, PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole,
				self.tr("Match group for molecule {0}.").format(molecule_count),
			)
			self._results.addTopLevelItem(group)
			for index in range(count):
				leaf_label = self.tr("Match {0}").format(index + 1)
				leaf = PySide6.QtWidgets.QTreeWidgetItem((leaf_label,))
				leaf.setData(
					0, PySide6.QtCore.Qt.ItemDataRole.AccessibleTextRole, leaf_label,
				)
				leaf.setData(
					0, PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole,
					self.tr("Match {0}; available.").format(index + 1),
				)
				group.addChild(leaf)
				self._row_ordinals[id(leaf)] = ordinal
				ordinal += 1
			group.setExpanded(True)
		if molecule_count == 0:
			return self.tr("No matches in this drawing.")
		message = self.tr("Found {0} matches in {1} molecules.").format(match_count, molecule_count)
		if truncated:
			message += " " + self.tr("Additional matches not shown.")
		if run.traversal == "total_match_budget_reached":
			message += " " + self.tr("Unexamined molecules may contain matches.")
		return message

	#============================================
	def _show_item(self, item: PySide6.QtWidgets.QTreeWidgetItem) -> None:
		"""Redeem one opaque row and draw only Rust-issued finite presentation bounds."""
		ordinal = self._row_ordinals.get(id(item))
		if ordinal is None or self._busy or self._receipt is None or self._tab is None:
			return
		self._busy = True
		self._set_status(self.tr("Showing match..."))
		self._update_controls()
		try:
			paint = self._tab._session._show_live_document_smarts_match_v1(self._receipt, ordinal)
		except engine.LiveDocumentSmartsError as error:
			self._present_error(error, reveal=True)
			return
		try:
			item_graphics = self._paint_item(paint)
			scene = self._tab.view.scene()
			if scene is None:
				raise _FerrumSmartsRevealProjectionFailure("Ferrum SMARTS canvas is unavailable")
			if self._tab._live_smarts_overlay_item_v1 is None:
				scene.addItem(item_graphics)
				self._tab._install_live_smarts_query_overlay_v1(item_graphics, self._receipt)
			else:
				self._tab._replace_live_smarts_query_overlay_v1(item_graphics)
		except _FerrumSmartsRevealProjectionFailure:
			self._recover_reveal_projection_failure_v1()
			return
		self._busy = False
		item.setText(0, item.text(0) + self.tr(" (shown)"))
		item.setData(
			0, PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole,
			self.tr("Match shown; activate again to verify it remains available."),
		)
		self._overlay_visible = True
		self._update_controls(terminal_status=self.tr("Match shown."))

	#============================================
	def _recover_reveal_projection_failure_v1(self) -> None:
		"""Release a failed local reveal only after native receipt invalidation succeeds."""
		self._run_token += 1
		self._busy = False
		if self._receipt is not None and self._tab is not None:
			invalidated = self._tab._invalidate_live_smarts_receipts_v1("dock_rerun")
			if not invalidated:
				self._block_receipt_invalidation_v1(self.tr(
					"SMARTS match display could not recover. Choose Clear results to retry, or "
					"refresh the drawing before searching again.",
				))
				return
		self._finish_receipt_invalidation_v1()
		self._update_controls(terminal_status=self.tr(
			"SMARTS match display could not recover. Run the query again.",
		))
		self._raw_input.setFocus()

	#============================================
	def _paint_item(self, paint: object) -> PySide6.QtWidgets.QGraphicsItemGroup:
		"""Project finite identity-free paint bounds without a local geometry calculation."""
		root = PySide6.QtWidgets.QGraphicsItemGroup()
		root.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		root.setHandlesChildEvents(False)
		root.setZValue(1_000_000.0)
		color = PySide6.QtWidgets.QApplication.palette().highlight().color()
		pen = PySide6.QtGui.QPen(color)
		pen.setWidthF(1.75)
		pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
		for bounds in paint.atom_bounds:
			left, top, right, bottom = (float(value) for value in bounds)
			if not all(math.isfinite(value) for value in (left, top, right, bottom)):
				raise _FerrumSmartsRevealProjectionFailure("non-finite paint bounds")
			rect = PySide6.QtCore.QRectF(left, top, right - left, bottom - top).normalized()
			if rect.isEmpty():
				raise _FerrumSmartsRevealProjectionFailure("empty paint bounds")
			child = PySide6.QtWidgets.QGraphicsRectItem(rect, root)
			child.setPen(pen)
			child.setBrush(PySide6.QtCore.Qt.BrushStyle.NoBrush)
			child.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		return root

	#============================================
	def _present_error(self, error: engine.LiveDocumentSmartsError,
			*, reveal: bool = False) -> None:
		"""Translate only closed bridge categories into plain-language recovery text."""
		self._busy = False
		if reveal:
			self._clear_results("dock_rerun", status=self.tr(
				"That match is no longer available. Run the query again.",
			))
			self._raw_input.setFocus()
			return
		message, focus_query = self._closed_failure_message(error)
		if focus_query:
			self._raw_input.setFocus()
		self._update_controls(terminal_status=message)

	#============================================
	def _resolve_closed_failure_messages(
			self,
			) -> tuple[tuple[
				engine.LiveDocumentSmartsCategoryV1,
				engine.LiveDocumentSmartsReasonV1,
				engine.LiveDocumentSmartsRecoveryV1,
				str,
				bool,
				], ...]:
		"""Freeze the only accepted PyO3 outcome triples for this dock instance."""
		category = engine.LiveDocumentSmartsCategoryV1
		reason = engine.LiveDocumentSmartsReasonV1
		recovery = engine.LiveDocumentSmartsRecoveryV1
		return (
			(category.invalid_query, reason.empty_query, recovery.edit_query,
				self.tr("Enter a SMARTS expression, then choose Find."), True),
			(category.invalid_query, reason.query_too_long, recovery.edit_query,
				self.tr("This SMARTS expression is too long. Use a shorter query and try again."), True),
			(category.invalid_query, reason.invalid_query, recovery.edit_query,
				self.tr("Ferrum could not read that SMARTS query. Check its syntax and try again."), True),
			(category.resource_limit, reason.match_caps_inconsistent, recovery.reduce_scope,
				self.tr("This query exceeds Ferrum's search limit. Use a smaller query and try again."), False),
			(category.refused, reason.selected_root_empty, recovery.select_one_molecule,
				self.tr("Select one direct molecule to use it as the query."), False),
			(category.refused, reason.selected_root_multiple, recovery.select_one_molecule,
				self.tr("Select one direct molecule to use it as the query."), False),
			(category.refused, reason.selected_target_not_molecule, recovery.select_one_molecule,
				self.tr("Select one direct molecule to use it as the query."), False),
			(category.unsupported_document, reason.unsupported_document, recovery.refresh_and_rerun,
				self.tr("Ferrum cannot search one or more structures in this drawing."), False),
			(category.stale, reason.stale_document, recovery.refresh_and_rerun,
				self.tr("The drawing changed or is not ready. Refresh it, then run the query again."), False),
			(category.stale, reason.stale_selection, recovery.refresh_and_rerun,
				self.tr("The drawing changed or is not ready. Refresh it, then run the query again."), False),
			(category.refused, reason.foreign_selection, recovery.select_one_molecule,
				self.tr("Select one direct molecule to use it as the query."), False),
			(category.unavailable, reason.plan_not_published, recovery.retry,
				self.tr("SMARTS search is temporarily unavailable. Try again."), False),
			(category.unavailable, reason.native_runtime_unavailable, recovery.retry,
				self.tr("SMARTS search is temporarily unavailable. Try again."), False),
			(category.unavailable, reason.match_unavailable, recovery.retry,
				self.tr("SMARTS search is temporarily unavailable. Try again."), False),
			(category.unavailable, reason.receipt_unavailable, recovery.retry,
				self.tr("SMARTS search is temporarily unavailable. Try again."), False),
			(category.unavailable, reason.paint_unavailable, recovery.retry,
				self.tr("SMARTS search is temporarily unavailable. Try again."), False),
		)

	#============================================
	def _closed_failure_message(
			self, error: engine.LiveDocumentSmartsError,
			) -> tuple[str, bool]:
		"""Return the exact documented native failure triple or expose a contract error."""
		return self._closed_failure_message_from_facts(
			error.category, error.reason, error.recovery,
		)

	#============================================
	def _closed_failure_message_from_facts(self, category: object, reason: object,
			recovery: object) -> tuple[str, bool]:
		"""Map one closed native failure triple or expose a bridge contract failure."""
		for expected_category, expected_reason, expected_recovery, message, focus_query in self._closed_failure_messages:
			if (category is expected_category or category == expected_category) and (
				reason is expected_reason or reason == expected_reason
			) and (recovery is expected_recovery or recovery == expected_recovery):
				return message, focus_query
		raise RuntimeError("Ferrum returned an undocumented SMARTS failure triple")

	#============================================
	def _clear_only_overlay(self) -> bool:
		"""Honor first Escape in a result leaf without clearing its query receipt."""
		if self._tab is None:
			return False
		if not self._tab._clear_live_smarts_query_overlay_v1():
			self._set_status(self.tr(
				"SMARTS highlight cannot be cleared. Refresh the drawing before searching again.",
			))
			self._invalidation_blocked = True
			self._update_controls()
			return False
		self._overlay_visible = False
		self._results.setFocus()
		self._update_controls(
			terminal_status=self.tr("Match highlight cleared. Results remain available."),
		)
		return True

	#============================================
	def _clear_results(self, reason: str, *, status: str | None) -> bool:
		"""Clear query output while preserving a still-current render plan."""
		self._run_token += 1
		self._busy = False
		tab = self._tab
		if self._receipt is not None and tab is not None:
			if reason in ("tab_deactivated", "tab_disposed"):
				invalidated = tab._invalidate_live_smarts_query_v1(reason)
			else:
				invalidated = tab._invalidate_live_smarts_receipts_v1(reason)
			if not invalidated:
				self._block_receipt_invalidation_v1(self.tr(
					"SMARTS results cannot be cleared. Choose Clear results to retry, or refresh "
					"the drawing before searching again.",
				))
				return False
		self._finish_receipt_invalidation_v1()
		self._update_controls(terminal_status=status)
		return True

	#============================================
	def _block_receipt_invalidation_v1(self, message: str) -> None:
		"""Preserve live receipt ownership until a deliberate native retry succeeds."""
		self._busy = False
		self._invalidation_blocked = True
		self._set_status(message)
		self._update_controls()

	#============================================
	def _finish_receipt_invalidation_v1(self) -> None:
		"""Forget copied query state only after native invalidation proved it unavailable."""
		self._receipt = None
		self._overlay_visible = False
		self._row_ordinals.clear()
		self._results.clear()
		self._invalidation_blocked = False
		self._tab = None
		self._bind_active_tab_invalidation()
		self._activate_current_tab()

	#============================================
	def _on_visibility_changed(self, visible: bool) -> None:
		"""Closing or hiding the dock cannot leave a receipt or overlay live."""
		if not visible and not self._invalidation_blocked:
			self._clear_results("dock_rerun", status=None)

	#============================================
	def _set_status(self, message: str) -> None:
		"""Set one concise visible and accessible status message."""
		self._status.setText(message)
		self._status.setAccessibleDescription(message)

	#============================================
	def _update_controls(self, *, terminal_status: str | None = None) -> None:
		"""Refresh eligibility, then preserve an explicit action outcome when supplied."""
		ready = self._tab is not None and not self._busy and not self._invalidation_blocked
		raw = self._raw_source.isChecked()
		availability = self._selected_availability()
		selected_recovery = self._selected_recovery_guidance(availability)
		self._raw_input.setEnabled(ready and raw)
		self._choose_molecule_button.setEnabled(ready)
		self._selected_source.setEnabled(ready and availability.available)
		self._selected_source.setAccessibleDescription(selected_recovery)
		self._raw_source.setEnabled(ready)
		self._find_button.setEnabled(
			ready and (bool(self._raw_input.text().strip()) if raw else availability.available),
		)
		self._clear_button.setEnabled(not self._busy and self._receipt is not None)
		self._results.setEnabled(
			not self._busy and not self._invalidation_blocked and self._receipt is not None,
		)
		if not raw and not availability.available and not self._busy and not self._invalidation_blocked:
			self._set_status(selected_recovery)
		if terminal_status is not None:
			self._set_status(terminal_status)

	#============================================
	def _selected_availability(self) -> (
			ferrum_qt.ferrum.smarts_selected_root_capture.
			FerrumSmartsSelectedQueryAvailabilityV1
			):
		"""Return copied controller-owned readiness without inspecting an opaque token."""
		return self._selected_capture.selected_query_availability_v1(self._tab)

	#============================================
	def _selected_recovery_guidance(self, availability: (
			ferrum_qt.ferrum.smarts_selected_root_capture.
			FerrumSmartsSelectedQueryAvailabilityV1
			)) -> str:
		"""Map the selected-token availability DTO's closed native facts for display."""
		facts = (availability.category, availability.reason, availability.recovery)
		if availability.available:
			if facts != (None, None, None):
				raise RuntimeError("Ferrum returned available selected SMARTS readiness with failure facts")
			return self.tr("Chosen molecule is ready for this drawing.")
		if facts == (None, None, None):
			return self.tr("Choose one direct molecule on the canvas to use it as the query.")
		if None in facts:
			raise RuntimeError("Ferrum returned incomplete selected SMARTS readiness facts")
		message, _focus_query = self._closed_failure_message_from_facts(*facts)
		return message

	#============================================
	def _begin_selected_capture_v1(self) -> None:
		"""Begin one point capture without retaining generic selection data."""
		if self._tab is None or self._busy or self._invalidation_blocked:
			return
		self._clear_selected_query_token_v1()
		self._selected_capture.begin()

	#============================================
	def _selected_capture_started_v1(self) -> None:
		"""Present capture guidance while the viewport exclusively owns the pointer."""
		self._update_controls(terminal_status=self.tr(
			"Choose one direct molecule on the canvas. Esc or right-click cancels.",
		))

	#============================================
	def _selected_capture_ready_v1(self, tab: object) -> None:
		"""Store copied readiness while the capture owner retains Rust capability."""
		if tab is not self._tab or not self._selected_capture.is_ready_for(tab):
			return
		self._selected_source.setChecked(True)
		self._update_controls(terminal_status=self.tr(
			"Chosen molecule is ready. Choose Find to search this drawing.",
		))

	#============================================
	def _selected_capture_refused_v1(self, message: str) -> None:
		"""Show only closed recovery language after an unsuccessful capture."""
		self._clear_selected_query_token_v1()
		self._update_controls(terminal_status=message)

	#============================================
	def _cancel_selected_capture_v1(self, message: str | None) -> None:
		"""Cancel transient pointer capture at every dock/tab lifecycle boundary."""
		self._selected_capture.cancel(message)

	#============================================
	def _clear_selected_query_token_v1(self) -> None:
		"""Tell the capture owner to drop its opaque selected-query capability."""
		self._selected_capture.clear_ready_v1()

	#============================================
