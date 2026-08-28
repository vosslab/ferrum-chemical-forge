"""Transactional lifecycle coverage for registered Ferrum native tabs."""

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.operation_leases
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
def _make_window(
		qapp: PySide6.QtWidgets.QApplication,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary Ferrum product window."""
	return ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)


#============================================
def _current_native_tab(
		window: ferrum_qt.main_window.MainWindow,
		) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
	"""Return the current exact Ferrum document page."""
	tab = window._tab_widget.currentWidget()
	assert isinstance(tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab)
	return tab


#============================================
def _close_test_window(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow,
		) -> None:
	"""Discard every exact test tab before ordinary window shutdown can prompt."""
	for tab in tuple(window._native_tabs_by_page.values()):
		index = window._tab_widget.indexOf(tab)
		result = window._close_native_tab_at(
			index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	assert not window._native_tabs_by_page
	window.close()
	window.deleteLater()
	qapp.processEvents()


#============================================
class _ReplacementResolution:
	"""Record the exact one-shot replacement result for lifecycle failures."""

	#============================================
	def __init__(self) -> None:
		"""Create one pending test resolution."""
		self.refused = False

	#============================================
	def accept_replacement(self, _receipt: object) -> None:
		"""Accept only if an unexpected post-commit path reaches this test double."""

	#============================================
	def refuse_replacement(self) -> None:
		"""Record complete rollback and returned candidate ownership."""
		self.refused = True


#============================================
def test_registered_replacement_rolls_back_after_shared_registration_failure(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed provisional integration leaves the old live tab as the sole page."""
	window = _make_window(qapp)
	old = _current_native_tab(window)
	old_identity = window._operation_leases.bind_tab(old)
	new = window._create_empty_native_tab()
	finish_registration = window._finish_native_tab_registration

	def refuse_after_shared_registration(
			candidate: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		finish_registration(candidate)
		raise RuntimeError("forced shared registration failure")

	monkeypatch.setattr(window, "_finish_native_tab_registration", refuse_after_shared_registration)
	try:
		with pytest.raises(RuntimeError, match="forced shared registration failure"):
			window._replace_registered_native_tab(old, new, 0)
		assert (
			_current_native_tab(window) is old
			and window._operation_leases.bind_tab(old) == old_identity
			and old in window._native_tabs_by_page
		)
		assert new.is_disposed and window._tab_widget.indexOf(new) < 0
		with pytest.raises(
			ferrum_qt.ferrum.operation_leases.OperationLeaseError,
			match="not bound",
		):
			window._operation_leases.unregister_tab(new)
	finally:
		monkeypatch.undo()
		_close_test_window(qapp, window)


#============================================
def test_native_tab_registration_retires_a_partially_integrated_candidate(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed ordinary add leaves its prior live tab as the sole integration owner."""
	window = _make_window(qapp)
	old = _current_native_tab(window)
	old_identity = window._operation_leases.bind_tab(old)
	candidate = window._create_empty_native_tab()
	finish_registration = window._finish_native_tab_registration

	def refuse_after_shared_registration(
			new_tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			) -> None:
		finish_registration(new_tab)
		raise RuntimeError("forced ordinary registration failure")

	monkeypatch.setattr(window, "_finish_native_tab_registration", refuse_after_shared_registration)
	try:
		with pytest.raises(RuntimeError, match="forced ordinary registration failure"):
			window._register_native_tab(candidate, activate=True)
		assert (
			_current_native_tab(window) is old
			and window._operation_leases.bind_tab(old) == old_identity
			and old in window._native_tabs_by_page
		)
		assert candidate.is_disposed and window._tab_widget.indexOf(candidate) < 0
		with pytest.raises(
			ferrum_qt.ferrum.operation_leases.OperationLeaseError,
			match="not bound",
		):
			window._operation_leases.unregister_tab(candidate)
	finally:
		monkeypatch.undo()
		_close_test_window(qapp, window)


#============================================
def test_registered_replacement_rolls_back_after_old_unregistration_refusal(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A typed old-registration refusal preserves the reachable old document."""
	window = _make_window(qapp)
	old = _current_native_tab(window)
	old_identity = window._operation_leases.bind_tab(old)
	new = window._create_empty_native_tab()
	unregister = window._operation_leases.unregister_tab

	def refuse_old_unregistration(tab: object) -> None:
		if tab is old:
			raise ferrum_qt.ferrum.operation_leases.OperationLeaseError(
				"forced old unregistration refusal",
			)
		unregister(tab)

	monkeypatch.setattr(
		window._operation_leases, "unregister_tab", refuse_old_unregistration,
	)
	try:
		with pytest.raises(
			ferrum_qt.ferrum.operation_leases.OperationLeaseError,
			match="forced old unregistration refusal",
		):
			window._replace_registered_native_tab(old, new, 0)
		assert (
			_current_native_tab(window) is old
			and window._operation_leases.bind_tab(old) == old_identity
			and old in window._native_tabs_by_page
		)
		assert new.is_disposed and window._tab_widget.indexOf(new) < 0
	finally:
		monkeypatch.undo()
		_close_test_window(qapp, window)


#============================================
def test_registered_replacement_rolls_back_after_old_disposal_refusal(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A typed old-disposal refusal restores the old live bound page exactly."""
	window = _make_window(qapp)
	old = _current_native_tab(window)
	new = window._create_empty_native_tab()
	capability = window._local_document_open_controller._local_document_open_capability
	lease = window._operation_leases.acquire(
		capability, tab=old,
		close_policy=ferrum_qt.ferrum.operation_leases.ClosePolicy.BLOCK_UNTIL_SETTLED,
	)

	def refuse_old_disposal() -> None:
		raise ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError(
			"forced old disposal refusal",
		)

	monkeypatch.setattr(old, "dispose", refuse_old_disposal)
	resolution = _ReplacementResolution()
	try:
		with pytest.raises(
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError,
			match="forced old disposal refusal",
		):
			window._commit_local_open_replacement(
				old, new, 0, capability, lease, resolution,
			)
		assert (
			_current_native_tab(window) is old
			and not old.is_disposed
			and window._native_tabs_by_page.get(old) is old
			and not new.is_disposed
			and window._tab_widget.indexOf(new) < 0
			and window._operation_leases.active_for_tab(old) == (lease,)
			and resolution.refused
		)
	finally:
		monkeypatch.undo()
		if not new.is_disposed:
			new.dispose()
		window._operation_leases.settle(
			capability, lease, ferrum_qt.ferrum.operation_leases.LeaseState.FAILED,
		)
		_close_test_window(qapp, window)


#============================================
def test_registered_replacement_publishes_the_new_integrated_tab(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A completed replacement retires its old page and admits the new canvas."""
	window = _make_window(qapp)
	old = _current_native_tab(window)
	new = window._create_empty_native_tab()
	new._adopt_local_document_origin(
		"/private/tmp/replacement.cml", "cml", object(),
	)
	try:
		assert window._replace_registered_native_tab(old, new, 0) is new
		assert _current_native_tab(window) is new and old.is_disposed
		assert window._tab_widget.tabToolTip(0) == new.local_document_source_description
		with pytest.raises(
			ferrum_qt.ferrum.operation_leases.OperationLeaseError,
			match="not bound",
		):
			window._operation_leases.unregister_tab(old)
		assert window._template_catalog_controller.start_placement(object(), "replacement")
		assert window._template_catalog_controller.cancel_for_tab(new, "test_cleanup")
	finally:
		_close_test_window(qapp, window)


#============================================
def test_native_tab_registration_publishes_its_source_tooltip_before_activation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An ordinary registration presents admitted source context on its tab entry."""
	window = _make_window(qapp)
	candidate = window._create_empty_native_tab()
	candidate._adopt_local_document_origin(
		"/private/tmp/ordinary.cml", "cml", object(),
	)
	try:
		assert window._register_native_tab(candidate, activate=True) is candidate
		index = window._tab_widget.indexOf(candidate)
		assert window._tab_widget.tabToolTip(index) == candidate.local_document_source_description
	finally:
		_close_test_window(qapp, window)
