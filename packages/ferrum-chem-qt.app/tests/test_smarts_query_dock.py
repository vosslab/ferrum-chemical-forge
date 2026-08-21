"""Focused presentation and action-lifecycle tests for the SMARTS dock."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.smarts_query_dock
import ferrum_qt.ferrum.live_document_transaction


#============================================
class _Window(PySide6.QtWidgets.QMainWindow):
	"""Minimal host exposing only the dock's active-tab query."""

	def _active_native_tab(self) -> None:
		"""Report that no drawing is currently active."""
		return None

	#============================================
	def _cancel_live_smarts_selected_root_capture_v1(self, _message: object = None) -> None:
		"""Accept the dock's transient capture cancellation hook."""
		return None

	#============================================
	def _set_interaction_capture_canceller_v1(self, _canceller: object) -> None:
		"""Accept the dock's window-owned capture handoff seam."""
		return None


#============================================
def test_action_is_retained_and_refreshable(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The installed menu command is the action later refreshed by dock state."""
	window = _Window()
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)
	menu = window.menuBar().addMenu("Chemistry")
	action = controller.install_action(menu)
	controller.refresh_action(active=False, pending=False, busy=False)
	assert controller._action is action
	assert action.text() == "SMARTS Query..."
	assert action.shortcut() == PySide6.QtGui.QKeySequence("Ctrl+Shift+F")
	assert action.statusTip() and action.whatsThis()
	assert not action.isEnabled()
	window.close()


#============================================
def test_result_rows_use_supported_accessibility_item_data(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Result labels expose accessible text and recovery-neutral descriptions."""
	window = _Window()
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)

	class _Molecule:
		match_count = 1
		completeness = "complete"

	class _Run:
		molecules = (_Molecule(),)
		traversal = "complete"

	controller._populate_results(_Run())
	group = controller._results.topLevelItem(0)
	leaf = group.child(0)
	assert group.data(0, PySide6.QtCore.Qt.ItemDataRole.AccessibleTextRole) == group.text(0)
	assert group.data(0, PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole)
	assert leaf.data(0, PySide6.QtCore.Qt.ItemDataRole.AccessibleTextRole) == leaf.text(0)
	assert leaf.data(0, PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole)
	window.close()


#============================================
class _Availability:
	"""A copied selected-mode state without selection facts."""

	def __init__(self, available: bool, recovery: str) -> None:
		self.available = available
		self.recovery = recovery


#============================================
class _Tab:
	"""Exercise dock lifecycle through opaque receipts and copied results only."""

	_disposed = False
	requires_refresh = False

	def __init__(self) -> None:
		self.view = PySide6.QtWidgets.QGraphicsView()
		self.view.setScene(PySide6.QtWidgets.QGraphicsScene(self.view))
		self._live_smarts_overlay_item_v1 = None
		self._retire_ok = True
		self._retire_calls = 0
		self._retire_attempts = 0
		self._receipt_retire_calls = 0
		self._clear_overlay_ok = True
		self._runs: list[str] = []
		self._invalidation_callback: object | None = None
		self._availability = _Availability(True, "Selected molecule is available.")

	def _begin_live_smarts_query_run_v1(self) -> None:
		self._runs.append("begin")

	def _run_live_smarts_selected_query_token_v1(self, _token: object,
			_per_molecule: int, _total: int) -> object:
		self._runs.append("selected")
		return self._run()

	class _Session:
		def __init__(self, owner: object) -> None:
			self._owner = owner
		def _run_live_document_smarts_query_v1(self, _query: str,
				_per_molecule: int, _total: int) -> object:
			self._owner._runs.append("raw")
			return self._owner._run()
		def _show_live_document_smarts_match_v1(self, _receipt: object, ordinal: int) -> object:
			if ordinal in self._owner._shown:
				raise _ClosedError("refused")
			self._owner._shown.add(ordinal)
			return type("Paint", (), {"atom_bounds": ((0.0, 0.0, 2.0, 2.0),)})()

	def _run(self) -> object:
		self._shown: set[int] = set()
		molecule = type("Molecule", (), {"match_count": 1, "completeness": "complete"})()
		return type("Run", (), {
			"receipt": object(), "molecules": (molecule,), "traversal": "complete",
		})()

	@property
	def _session(self) -> object:
		return self._Session(self)

	def _install_live_smarts_query_overlay_v1(self, item: object, _receipt: object) -> None:
		self.view.scene().addItem(item)
		self._live_smarts_overlay_item_v1 = item

	def _replace_live_smarts_query_overlay_v1(self, item: object) -> None:
		if self._live_smarts_overlay_item_v1 is not None:
			self.view.scene().removeItem(self._live_smarts_overlay_item_v1)
		self.view.scene().addItem(item)
		self._live_smarts_overlay_item_v1 = item

	def _clear_live_smarts_query_overlay_v1(self) -> bool:
		if not self._clear_overlay_ok or self._live_smarts_overlay_item_v1 is None:
			return False
		self.view.scene().removeItem(self._live_smarts_overlay_item_v1)
		self._live_smarts_overlay_item_v1 = None
		return True

	def _retire_live_smarts_query_v1(self, _reason: str) -> bool:
		self._retire_attempts += 1
		if not self._retire_ok:
			raise _ClosedError("unavailable")
		self._retire_calls += 1
		self._clear_live_smarts_query_overlay_v1()
		return True

	def _retire_live_smarts_receipts_v1(self, _reason: str) -> bool:
		"""Model query-only cleanup without invalidating the fixture's plan."""
		if not self._retire_ok:
			raise _ClosedError("unavailable")
		self._receipt_retire_calls += 1
		self._clear_live_smarts_query_overlay_v1()
		return True

	#============================================
	def _bind_live_smarts_invalidation_callback_v1(self, callback: object) -> None:
		"""Retain only the active dock callback in this presentation fixture."""
		self._invalidation_callback = callback


#============================================
class _ClosedCategory:
	"""Frozen PyO3-shaped enum member with equality but intentionally no name."""

	def __init__(self, identity: str) -> None:
		self._identity = identity

	#============================================
	def __eq__(self, other: object) -> bool:
		"""Match the extension enum's value-equality contract without string fallback."""
		return type(other) is type(self) and self._identity == other._identity

	#============================================
	def __hash__(self) -> int:
		"""Keep this frozen fixture value hashable like the extension enum."""
		return hash(self._identity)


#============================================
class _ClosedEnumClass:
	"""Expose fixed extension-member attributes without Python Enum conveniences."""

	def __init__(self, *identities: str) -> None:
		for identity in identities:
			setattr(self, identity, _ClosedCategory(identity))


#============================================
class _ClosedEnums:
	"""Simulate the three frozen PyO3 enum classes used by the dock."""

	LiveDocumentSmartsCategoryV1 = _ClosedEnumClass(
		"invalid_query", "unsupported_document", "resource_limit", "stale", "unavailable", "refused",
	)
	LiveDocumentSmartsReasonV1 = _ClosedEnumClass(
		"empty_query", "query_too_long", "invalid_query", "match_caps_inconsistent",
		"selected_root_empty", "selected_root_multiple", "selected_source_not_molecule",
		"unsupported_document", "stale_document", "stale_selection", "foreign_selection",
		"plan_not_published", "native_runtime_unavailable", "match_unavailable",
		"receipt_unavailable", "paint_unavailable",
	)
	LiveDocumentSmartsRecoveryV1 = _ClosedEnumClass(
		"edit_query", "reduce_scope", "select_one_molecule", "refresh_and_rerun", "retry",
	)


#============================================
class _ClosedError(Exception):
	"""Closed test outcome whose visible text must never be read."""

	def __init__(self, category: object, reason: object = None,
			recovery: object = None) -> None:
		self.category = category
		self.reason = reason
		self.recovery = recovery
		super().__init__("native detail must not reach the dock")


#============================================
class _DockWindow(_Window):
	"""Host one active fake tab without exposing its selection to the dock."""

	def __init__(self, tab: _Tab) -> None:
		super().__init__()
		self._tab = tab

	def _active_native_tab(self) -> _Tab:
		return self._tab


#============================================
def test_tab_switch_uses_one_native_retirement_then_local_dock_deactivation(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The outgoing tab retires once; Qt has already made the next tab current."""
	previous = _Tab()
	incoming = _Tab()
	window = _DockWindow(previous)
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)
	controller._activate_current_tab()
	controller._raw_input.setText("C")
	controller._begin_run()
	qapp.processEvents()
	window.show()
	controller.dock.show()
	qapp.processEvents()
	assert controller._receipt is not None and controller.dock.isVisible()

	window._tab = incoming
	previous._retire_live_smarts_query_v1("tab_deactivated")
	controller._deactivate_after_tab_retirement_v1(True)
	assert previous._retire_attempts == 1 and previous._retire_calls == 1
	assert controller._receipt is None and controller._tab is incoming
	assert controller._results.topLevelItemCount() == 0 and not controller.dock.isVisible()
	assert previous._invalidation_callback is None

	controller._raw_input.setText("N")
	controller._begin_run()
	qapp.processEvents()
	assert incoming._runs[-2:] == ["begin", "raw"]
	assert incoming._retire_calls == 0 and incoming._receipt_retire_calls == 0
	window.close()


#============================================
def test_tab_switch_failure_discards_stale_dock_state_without_second_native_call(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A failed outgoing retirement leaves no stale result visible on the new tab."""
	previous = _Tab()
	incoming = _Tab()
	window = _DockWindow(previous)
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)
	controller._activate_current_tab()
	controller._raw_input.setText("C")
	controller._begin_run()
	qapp.processEvents()
	controller.dock.show()
	qapp.processEvents()
	previous._retire_ok = False
	try:
		previous._retire_live_smarts_query_v1("tab_deactivated")
	except _ClosedError:
		pass
	window._tab = incoming
	controller._deactivate_after_tab_retirement_v1(False)
	assert previous._retire_attempts == 1 and previous._retire_calls == 0
	assert controller._receipt is None and controller._results.topLevelItemCount() == 0
	assert not controller.dock.isVisible() and "previous drawing" in controller._status.text()

	controller._raw_input.setText("N")
	controller._begin_run()
	qapp.processEvents()
	assert incoming._runs[-2:] == ["begin", "raw"]
	window.close()


#============================================
def test_tab_switch_hides_cleared_outgoing_dock_before_binding_incoming_tab(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A hide callback cannot see or retire the already-current incoming plan."""
	previous = _Tab()
	incoming = _Tab()
	window = _DockWindow(previous)
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)
	controller._activate_current_tab()
	controller._raw_input.setText("C")
	controller._begin_run()
	qapp.processEvents()
	window.show()
	controller.dock.show()
	qapp.processEvents()
	hide_states: list[tuple[object | None, object | None]] = []
	controller.dock.visibilityChanged.connect(
		lambda visible: hide_states.append((controller._tab, incoming._invalidation_callback))
		if not visible else None,
	)

	# QTabWidget has changed its current page before the outgoing retirement
	# completion hook is delivered.
	window._tab = incoming
	previous._retire_live_smarts_query_v1("tab_deactivated")
	controller._deactivate_after_tab_retirement_v1(True)

	assert previous._retire_attempts == 1 and previous._retire_calls == 1
	assert incoming._retire_attempts == 0 and incoming._retire_calls == 0
	assert incoming._receipt_retire_calls == 0
	assert hide_states == [(None, None)]
	assert controller._tab is incoming
	assert incoming._invalidation_callback is not None
	controller._raw_input.setText("N")
	controller._begin_run()
	qapp.processEvents()
	assert incoming._runs[-2:] == ["begin", "raw"]
	window.close()


#============================================
def test_first_tab_activation_binds_without_hiding_the_dock(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""First registration has no outgoing plan and must not close the modeless dock."""
	tab = _Tab()
	window = _DockWindow(tab)
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)
	window.show()
	controller.dock.show()
	qapp.processEvents()
	controller._activate_after_tab_switch_v1()
	assert controller.dock.isVisible() and controller._tab is tab
	assert tab._retire_calls == 0 and tab._receipt_retire_calls == 0
	window.close()


#============================================
class _LifecycleSession:
	"""Record closed retirement and downstream transition order."""

	def __init__(self, events: list[str]) -> None:
		self._events = events

	#============================================
	def _retire_live_document_smarts_query_v1(self) -> None:
		self._events.append("native_retire")


#============================================
class _LifecycleTab(ferrum_qt.ferrum.live_document_transaction.FerrumLiveDocumentTransactionMixin):
	"""Minimal transaction owner for notification-order coverage."""

	_disposed = False
	requires_refresh = False

	def _require_live(self) -> None:
		"""Accept the focused fixture's live transition."""


#============================================
def _controller_with_run(qapp: PySide6.QtWidgets.QApplication) -> tuple[object, _Tab, _DockWindow]:
	"""Create one controller and drive a deferred raw query to populated rows."""
	tab = _Tab()
	window = _DockWindow(tab)
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)
	controller._activate_current_tab()
	controller._raw_input.setText("C")
	controller._begin_run()
	qapp.processEvents()
	return controller, tab, window


#============================================
def test_selected_query_dispatch_keeps_selection_out_of_dock_and_queue(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Selected mode calls the tab-private wrapper with one opaque token only."""
	controller, tab, window = _controller_with_run(qapp)
	controller._clear_results("dock_rerun", status=None)
	controller._selected_capture._selected_query_token = object()
	controller._selected_capture._ready_tab = tab
	controller._selected_source.setChecked(True)
	controller._begin_run()
	assert "_selected_query_token" not in controller.__dict__
	assert "_ready_tab" not in controller.__dict__
	assert controller._selected_capture._selected_query_token is not None
	qapp.processEvents()
	assert tab._runs[-2:] == ["begin", "selected"]
	assert "_structure_selection" not in controller.__dict__
	assert "_selected_query_token" not in controller.__dict__
	assert "_ready_tab" not in controller.__dict__
	window.close()


#============================================
def test_closed_error_never_uses_exception_text(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""Unknown or incomplete enum triples fail closed without native detail."""
	monkeypatch.setattr(ferrum_qt.ferrum.smarts_query_dock, "ferrum_chem", _ClosedEnums)
	controller, _tab, window = _controller_with_run(qapp)
	category = _ClosedEnums.LiveDocumentSmartsCategoryV1
	reason = _ClosedEnums.LiveDocumentSmartsReasonV1
	recovery = _ClosedEnums.LiveDocumentSmartsRecoveryV1
	controller._present_error(_ClosedError(_ClosedCategory("unexpected"), reason.invalid_query, recovery.edit_query))
	assert controller._status.text() == "SMARTS search is temporarily unavailable. Try again."
	controller._present_error(_ClosedError(category.invalid_query, reason.invalid_query))
	assert controller._status.text() == "SMARTS search is temporarily unavailable. Try again."
	assert "native detail" not in controller._status.text()
	window.close()


#============================================
def test_closed_pyo3_enum_triples_map_without_name_or_text_fallback(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""Every documented PyO3 error triple has an intentional UI recovery message."""
	monkeypatch.setattr(ferrum_qt.ferrum.smarts_query_dock, "ferrum_chem", _ClosedEnums)
	controller, _tab, window = _controller_with_run(qapp)
	category = _ClosedEnums.LiveDocumentSmartsCategoryV1
	reason = _ClosedEnums.LiveDocumentSmartsReasonV1
	recovery = _ClosedEnums.LiveDocumentSmartsRecoveryV1
	for outcome, expected in (
		((_ClosedError(category.invalid_query, reason.empty_query, recovery.edit_query),
			"Enter a SMARTS expression, then choose Find.")),
		((_ClosedError(category.invalid_query, reason.query_too_long, recovery.edit_query),
			"This SMARTS expression is too long. Use a shorter query and try again.")),
		((_ClosedError(category.invalid_query, reason.invalid_query, recovery.edit_query),
			"Ferrum could not read that SMARTS query. Check its syntax and try again.")),
		((_ClosedError(category.resource_limit, reason.match_caps_inconsistent, recovery.reduce_scope),
			"This query exceeds Ferrum's search limit. Use a smaller query and try again.")),
		((_ClosedError(category.refused, reason.selected_root_empty, recovery.select_one_molecule),
			"Select one direct molecule to use it as the query.")),
		((_ClosedError(category.refused, reason.selected_root_multiple, recovery.select_one_molecule),
			"Select one direct molecule to use it as the query.")),
		((_ClosedError(category.refused, reason.selected_source_not_molecule, recovery.select_one_molecule),
			"Select one direct molecule to use it as the query.")),
		((_ClosedError(category.unsupported_document, reason.unsupported_document, recovery.refresh_and_rerun),
			"Ferrum cannot search one or more structures in this drawing.")),
		((_ClosedError(category.stale, reason.stale_document, recovery.refresh_and_rerun),
			"The drawing changed or is not ready. Refresh it, then run the query again.")),
		((_ClosedError(category.stale, reason.stale_selection, recovery.refresh_and_rerun),
			"The drawing changed or is not ready. Refresh it, then run the query again.")),
		((_ClosedError(category.refused, reason.foreign_selection, recovery.select_one_molecule),
			"Select one direct molecule to use it as the query.")),
		((_ClosedError(category.unavailable, reason.plan_not_published, recovery.retry),
			"SMARTS search is temporarily unavailable. Try again.")),
		((_ClosedError(category.unavailable, reason.native_runtime_unavailable, recovery.retry),
			"SMARTS search is temporarily unavailable. Try again.")),
		((_ClosedError(category.unavailable, reason.match_unavailable, recovery.retry),
			"SMARTS search is temporarily unavailable. Try again.")),
		((_ClosedError(category.unavailable, reason.receipt_unavailable, recovery.retry),
			"SMARTS search is temporarily unavailable. Try again.")),
		((_ClosedError(category.unavailable, reason.paint_unavailable, recovery.retry),
			"SMARTS search is temporarily unavailable. Try again.")),
	):
		controller._present_error(outcome)
		assert controller._status.text() == expected
		assert "native detail" not in controller._status.text()
	window.close()


#============================================
def test_replay_reaches_bridge_then_retires_results(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A shown row remains activatable so the one-use bridge refusal is authoritative."""
	controller, tab, window = _controller_with_run(qapp)
	leaf = controller._results.topLevelItem(0).child(0)
	controller._show_item(leaf)
	assert id(leaf) in controller._row_ordinals and not leaf.isDisabled()
	controller._show_item(leaf)
	assert controller._receipt is None and tab._live_smarts_overlay_item_v1 is None
	window.close()


#============================================
def test_escape_clears_overlay_then_run_without_touching_raw_input(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Leaf Escape removes only paint; the next dock Escape retires the receipt."""
	controller, tab, window = _controller_with_run(qapp)
	controller._raw_input.setText("[C]")
	leaf = controller._results.topLevelItem(0).child(0)
	controller._results.setCurrentItem(leaf)
	controller._show_item(leaf)
	controller._results.setFocus()
	PySide6.QtTest.QTest.keyClick(controller._results, PySide6.QtCore.Qt.Key.Key_Escape)
	assert not controller._overlay_visible and controller._receipt is not None
	PySide6.QtTest.QTest.keyClick(controller._results, PySide6.QtCore.Qt.Key.Key_Escape)
	assert controller._receipt is None and controller._raw_input.text() == "[C]"
	assert tab._live_smarts_overlay_item_v1 is None
	window.close()


#============================================
def test_query_clear_uses_receipt_retirement_and_immediately_admits_raw_and_selected_runs(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Clear revokes only the old opaque result capability, not the current plan."""
	controller, tab, window = _controller_with_run(qapp)
	assert controller._clear_results("dock_rerun", status=None)
	assert tab._receipt_retire_calls == 1 and tab._retire_calls == 0
	controller._begin_run()
	qapp.processEvents()
	assert tab._runs[-2:] == ["begin", "raw"]
	assert controller._clear_results("dock_rerun", status=None)
	controller._selected_capture._selected_query_token = object()
	controller._selected_capture._ready_tab = tab
	controller._selected_source.setChecked(True)
	controller._begin_run()
	qapp.processEvents()
	assert tab._runs[-2:] == ["begin", "selected"]
	window.close()


#============================================
def test_escape_overlay_retirement_false_preserves_state_and_blocks_new_run(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A false typed overlay retirement is not presented as a cleared highlight."""
	controller, tab, window = _controller_with_run(qapp)
	leaf = controller._results.topLevelItem(0).child(0)
	controller._results.setCurrentItem(leaf)
	controller._show_item(leaf)
	tab._clear_overlay_ok = False
	controller._results.setFocus()
	PySide6.QtTest.QTest.keyClick(controller._results, PySide6.QtCore.Qt.Key.Key_Escape)
	assert controller._overlay_visible and controller._receipt is not None
	assert controller._retirement_blocked and not controller._find_button.isEnabled()
	assert not controller._results.isEnabled()
	assert "cannot be cleared" in controller._status.text()
	assert "highlight cleared" not in controller._status.text().lower()
	window.close()


#============================================
def test_escape_clears_a_populated_run_from_every_dock_control(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Every focused dock control keeps the same second-stage Escape recovery."""
	for control_name in (
		"_raw_input", "_raw_source", "_selected_source", "_find_button", "_clear_button",
	):
		controller, _tab, window = _controller_with_run(qapp)
		controller._raw_input.setText("[C]")
		control = getattr(controller, control_name)
		control.setFocus()
		PySide6.QtTest.QTest.keyClick(control, PySide6.QtCore.Qt.Key.Key_Escape)
		assert controller._receipt is None and controller._raw_input.text() == "[C]"
		window.close()


#============================================
def test_tab_invalidation_clears_only_copied_dock_state(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A successful tab transition immediately retires stale rows without re-retiring."""
	controller, tab, window = _controller_with_run(qapp)
	window.show()
	controller.dock.show()
	qapp.processEvents()
	controller._on_live_smarts_query_invalidated_v1()
	assert controller._receipt is None and controller._results.topLevelItemCount() == 0
	assert tab._retire_calls == 0 and controller.dock.isVisible()
	assert "Run the query again" in controller._status.text()
	window.close()


#============================================
def test_transition_notifies_after_native_retirement_before_mutation_or_reprojection() -> None:
	"""Both transition fences clear a dock only after their old query is closed."""
	events: list[str] = []
	tab = _LifecycleTab()
	tab._initialize_live_document_transaction_v1(_LifecycleSession(events))
	tab._bind_live_smarts_invalidation_callback_v1(lambda: events.append("dock_clear"))
	tab._retire_then_mutate_document_v1(lambda: events.append("mutation"))
	tab._retire_then_reproject_document_v1(lambda: events.append("reprojection"))
	assert events == [
		"native_retire", "dock_clear", "mutation",
		"native_retire", "dock_clear", "reprojection",
	]


#============================================
def test_selected_source_accessibility_and_status_hide_private_recovery_codes(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A missing token presents capture guidance, never tab-private codes."""
	controller, tab, window = _controller_with_run(qapp)
	controller._clear_results("dock_rerun", status=None)
	controller._selected_source.setChecked(True)
	assert controller._status.text() == (
		"Choose one direct molecule on the canvas to use it as the query."
	)
	assert controller._selected_source.accessibleDescription() == controller._status.text()
	assert "select_one_molecule" not in controller._status.text()
	assert "select_one_molecule" not in controller._selected_source.accessibleDescription()
	window.close()


#============================================
def test_find_eligibility_refreshes_from_qtest_input_and_source_state(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Find follows one refresh rule for raw input, source, tab, busy, and retirement."""
	tab = _Tab()
	window = _DockWindow(tab)
	controller = ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(window)
	controller._activate_current_tab()
	window.show()
	controller.dock.show()
	qapp.processEvents()
	assert not controller._find_button.isEnabled()
	controller._raw_input.setFocus()
	PySide6.QtTest.QTest.keyClicks(controller._raw_input, "C")
	qapp.processEvents()
	assert controller._find_button.isEnabled()
	PySide6.QtTest.QTest.mouseClick(
		controller._find_button, PySide6.QtCore.Qt.MouseButton.LeftButton,
	)
	qapp.processEvents()
	assert "raw" in tab._runs
	controller._selected_source.setChecked(True)
	qapp.processEvents()
	assert not controller._find_button.isEnabled()
	controller._selected_capture._selected_query_token = object()
	controller._selected_capture._ready_tab = tab
	controller._update_controls()
	assert controller._find_button.isEnabled()
	controller._selected_capture.clear_ready_v1()
	controller._update_controls()
	assert not controller._find_button.isEnabled()
	controller._raw_source.setChecked(True)
	controller._busy = True
	controller._update_controls()
	assert not controller._find_button.isEnabled()
	controller._busy = False
	controller._tab = None
	controller._update_controls()
	assert not controller._find_button.isEnabled()
	controller._tab = tab
	controller._retirement_blocked = True
	controller._update_controls()
	assert not controller._find_button.isEnabled()
	window.close()


#============================================
def test_selected_source_accessible_name_matches_visible_label(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Screen readers announce the same selected-source phrase the dock displays."""
	controller, _tab, window = _controller_with_run(qapp)
	assert controller._selected_source.text() == "Use chosen molecule"
	assert controller._selected_source.accessibleName() == controller._selected_source.text()
	window.close()


#============================================
def test_retirement_failure_preserves_results_and_blocks_new_run(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A typed retirement refusal does not let the dock falsely claim success."""
	controller, tab, window = _controller_with_run(qapp)
	tab._retire_ok = False
	assert not controller._clear_results("dock_rerun", status="SMARTS results cleared.")
	assert controller._receipt is not None and controller._retirement_blocked
	assert "cannot be cleared" in controller._status.text()
	assert not controller._find_button.isEnabled()
	window.close()
