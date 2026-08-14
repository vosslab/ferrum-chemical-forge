"""Behavior coverage for OASA-free native clipboard Copy."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem
import pytest

# local repo modules
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.io.clipboard_mime
import ferrum_qt.native.ferrum_native_clipboard
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_main_window


_SOURCE = """\
<cdml version="26.07"><plus id="p"><point x="30" y="40"/></plus>
<molecule id="m" name="chain">
 <atom id="a" name="C"><point x="0" y="0"/></atom>
 <atom id="b" name="N"><point x="10" y="0"/></atom>
 <atom id="c" name="O"><point x="20" y="0"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/>
 <bond id="bc" start="b" end="c" type="n1"/>
</molecule></cdml>
"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one reusable offscreen Qt application."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _window_with_source() -> tuple[object, object]:
	"""Return one standalone native host with a selected clean source tab."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_SOURCE, "copy.cdml",
	)
	window._register_native_tab(tab, activate=True)
	return window, tab


#============================================
def _action(window: object, label: str) -> PySide6.QtGui.QAction:
	"""Find one user-reachable action through the public QObject tree."""
	matches = tuple(
		action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == label
	)
	assert len(matches) == 1
	return matches[0]


#============================================
def _wait_for_copy(window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Wait for the one action-created worker, then deliver its queued result."""
	workers = tuple(
		worker for worker in window.findChildren(
			ferrum_qt.native.ferrum_native_clipboard.FerrumNativeClipboardCopyWorker,
		)
	)
	assert len(workers) == 1 and workers[0].wait(10000)
	qapp.processEvents()


#============================================
def _select_plus(tab: object) -> None:
	"""Add the one visible Plus item to the current scene selection."""
	items = tuple(
		item for item in tab.view.scene().items()
		if type(item) is ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem
	)
	assert len(items) == 1
	items[0].setSelected(True)


#============================================
def test_copy_action_publishes_connected_bond_fragment_without_mutation(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The public Copy action closes a selected bond over both endpoint atoms."""
	window, tab = _window_with_source()
	clipboard = qapp.clipboard()
	try:
		tab.select_bond("bc")
		before = tab.current_snapshot
		selected = tab.selected_molecule_information_targets()
		copy_action = _action(window, "Copy")
		assert copy_action.shortcut() == PySide6.QtGui.QKeySequence(
			PySide6.QtGui.QKeySequence.StandardKey.Copy,
		)
		assert copy_action.isEnabled()
		copy_action.trigger()
		_wait_for_copy(window, qapp)

		mime_data = clipboard.mimeData()
		assert mime_data.hasFormat(ferrum_qt.native.ferrum_native_clipboard.CDML_MIME_TYPE)
		assert mime_data.property(
			ferrum_qt.io.clipboard_mime.FERRUM_OWNED_MIME_PROPERTY,
		) is True
		fragment = mime_data.text()
		assert 'id="b"' in fragment and 'id="c"' in fragment
		assert 'id="bc"' in fragment and 'id="a"' not in fragment
		assert ferrum_chem.DocumentSession.load(fragment).snapshot().revision == 0
		assert tab.current_snapshot == before
		assert tab.selected_molecule_information_targets() == selected
	finally:
		_action(window, "Close Tab").trigger()
		window.deleteLater()


#============================================
def test_mixed_atom_and_plus_copy_complete_roots_in_source_order(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Mixed structure/artwork selection preserves both complete direct roots."""
	window, tab = _window_with_source()
	try:
		tab.select_atom("a")
		_select_plus(tab)
		_action(window, "Copy").trigger()
		_wait_for_copy(window, qapp)
		fragment = qapp.clipboard().text()
		assert fragment.index("<plus") < fragment.index("<molecule")
		assert 'id="a"' in fragment and 'id="c"' in fragment
		assert 'id="ab"' in fragment and 'id="bc"' in fragment
	finally:
		_action(window, "Close Tab").trigger()
		window.deleteLater()


#============================================
def test_failed_copy_preserves_existing_clipboard_and_reports_actionable_error(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""A disconnected selection cannot replace pre-existing clipboard content."""
	window, tab = _window_with_source()
	warnings = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QMessageBox, "warning",
		lambda _parent, title, text: warnings.append((title, text)),
	)
	qapp.clipboard().setText("existing clipboard")
	try:
		tab.select_atoms(("a", "c"))
		_action(window, "Copy").trigger()
		_wait_for_copy(window, qapp)
		assert qapp.clipboard().text() == "existing clipboard"
		assert not qapp.clipboard().mimeData().hasFormat(
			ferrum_qt.native.ferrum_native_clipboard.CDML_MIME_TYPE,
		)
		assert warnings and warnings[-1][0] == "Native Copy Error"
		assert "must be connected" in warnings[-1][1]
	finally:
		_action(window, "Close Tab").trigger()
		window.deleteLater()


#============================================
def test_switching_tabs_suppresses_stale_clipboard_delivery(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A result from an inactive source tab cannot replace clipboard content."""
	window, tab = _window_with_source()
	other = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_SOURCE, "other.cdml",
	)
	qapp.clipboard().setText("current clipboard")
	try:
		tab.select_bond("bc")
		_action(window, "Copy").trigger()
		window._register_native_tab(other, activate=True)
		_wait_for_copy(window, qapp)
		assert qapp.clipboard().text() == "current clipboard"
	finally:
		_action(window, "Close Tab").trigger()
		window.centralWidget().setCurrentWidget(tab)
		_action(window, "Close Tab").trigger()
		window.deleteLater()


#============================================
def test_changing_selection_suppresses_stale_clipboard_delivery(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A result for an old selection cannot replace clipboard content."""
	window, tab = _window_with_source()
	qapp.clipboard().setText("current clipboard")
	try:
		tab.select_bond("bc")
		_action(window, "Copy").trigger()
		tab.select_atom("a")
		_wait_for_copy(window, qapp)
		assert qapp.clipboard().text() == "current clipboard"
	finally:
		_action(window, "Close Tab").trigger()
		window.deleteLater()


#============================================
def test_cancel_and_close_keep_source_live_until_copy_worker_finishes(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""Cancellation suppresses publication and close retains the worker source tab."""
	window, tab = _window_with_source()
	warnings = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QMessageBox, "warning",
		lambda _parent, title, text: warnings.append((title, text)),
	)
	qapp.clipboard().setText("retained clipboard")
	tab.select_bond("bc")
	copy_action = _action(window, "Copy")
	cancel_action = _action(window, "Cancel Copy")
	close_action = _action(window, "Close Tab")
	try:
		copy_action.trigger()
		assert not copy_action.isEnabled() and cancel_action.isEnabled()
		cancel_action.trigger()
		window.centralWidget().tabCloseRequested.emit(window.centralWidget().indexOf(tab))
		assert window.centralWidget().indexOf(tab) >= 0
		workers = window.findChildren(
			ferrum_qt.native.ferrum_native_clipboard.FerrumNativeClipboardCopyWorker,
		)
		assert workers and workers[0].wait(10000)
		qapp.processEvents()
		assert qapp.clipboard().text() == "retained clipboard"
		assert warnings and warnings[-1][0] == "Native Copy Still Running"
		close_action.trigger()
		assert window.centralWidget().indexOf(tab) < 0
	finally:
		window.deleteLater()
