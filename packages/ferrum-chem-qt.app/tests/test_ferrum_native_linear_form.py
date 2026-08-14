"""Behavior coverage for ordinary native linear-form conversion."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_linear_form
import ferrum_qt.native.ferrum_native_main_window


_SOURCE = """\
<cdml version="26.07"><molecule id="m">
 <atom id="late" name="C"><point x="40" y="5"/></atom>
 <atom id="early" name="O"><point x="10" y="5"/></atom>
 <bond id="path" start="late" end="early" type="n1"/>
</molecule><molecule id="other"><atom id="foreign" name="N">
 <point x="0" y="0"/></atom></molecule></cdml>
"""

_BRANCH = """\
<cdml version="26.07"><molecule id="branch">
 <atom id="a" name="C"><point x="0" y="0"/></atom>
 <atom id="b" name="C"><point x="10" y="0"/></atom>
 <atom id="c" name="N"><point x="20" y="5"/></atom>
 <atom id="d" name="O"><point x="20" y="-5"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/>
 <bond id="bc" start="b" end="c" type="n1"/>
 <bond id="bd" start="b" end="d" type="n1"/>
</molecule></cdml>
"""


#============================================
def _action(window: object) -> PySide6.QtGui.QAction:
	"""Return the visible ordinary-native conversion action."""
	matches = tuple(
		action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == "Convert selection to linear form"
	)
	assert len(matches) == 1
	return matches[0]


#============================================
def _new_window_tab(cdml: str = _SOURCE) -> tuple[object, object]:
	"""Create one ordinary native window with one Rust-owned document tab."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		cdml, "linear.cdml",
	)
	window._register_native_tab(tab, activate=True)
	return window, tab


#============================================
def _selection(tab: object) -> tuple[tuple[str, str | None], ...]:
	"""Return current detached durable selection facts."""
	return tuple(
		(target.kind, target.identifier)
		for target in tab.selected_molecule_information_targets()
	)


#============================================
def _snapshot_facts(snapshot: object) -> tuple[str, int, str, bool]:
	"""Return complete public snapshot state for atomic-refusal checks."""
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


#============================================
def _close_clean(window: object, tab: object) -> None:
	"""Return the document to its loaded baseline and retire the test window."""
	while tab.current_snapshot.is_dirty:
		window._undo_action.trigger()
	window._close_tab_at(window.centralWidget().indexOf(tab))
	window.deleteLater()


#============================================
def test_selected_bond_converts_source_order_and_restores_atom_selection(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The action expands bond endpoints and restores the accepted atom selection."""
	window, tab = _new_window_tab()
	action = _action(window)
	try:
		tab.select_bond("path")
		window._refresh_actions()
		capture = ferrum_qt.native.ferrum_native_linear_form.capture_linear_form_selection(
			tab,
		)
		assert capture is not None and capture.atom_ids == ("late", "early")

		action.trigger()
		assert (
			tab.current_snapshot.revision == 1
			and _selection(tab) == (("atom", "late"), ("atom", "early"))
		)
	finally:
		_close_clean(window, tab)
		del qapp


#============================================
def test_cross_root_selection_is_not_offered(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The action remains unreachable when selected durable children span roots."""
	window, tab = _new_window_tab()
	try:
		tab.select_atoms(("late", "foreign"))
		window._refresh_actions()
		assert (
			not _action(window).isEnabled()
			and ferrum_qt.native.ferrum_native_linear_form.
			capture_linear_form_selection(tab) is None
		)
		assert tab.current_snapshot.revision == 0 and not tab.current_snapshot.is_dirty
	finally:
		_close_clean(window, tab)
		del qapp


#============================================
def test_rust_path_refusal_is_typed_visible_and_atomic(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch) -> None:
	"""A mapped fork reaches Rust, whose typed reason is shown without changing Qt."""
	window, tab = _new_window_tab(_BRANCH)
	warnings = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QMessageBox,
		"warning",
		lambda _parent, title, message: warnings.append((title, message)),
	)
	try:
		tab.select_atoms(("a", "b", "c", "d"))
		window._refresh_actions()
		action = _action(window)
		before = _snapshot_facts(tab.current_snapshot)
		before_scene = tab.view.scene()
		before_selection = _selection(tab)
		action.trigger()

		assert (
			_snapshot_facts(tab.current_snapshot) == before
			and tab.view.scene() is before_scene
			and _selection(tab) == before_selection
		)
		assert (
			warnings
			and warnings[-1][0] == "Convert to Linear Form"
			and "linear-form planning refused" in warnings[-1][1]
		)
	finally:
		_close_clean(window, tab)
		del qapp
