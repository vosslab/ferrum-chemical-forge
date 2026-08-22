"""Behavior coverage for ordinary Rust-owned molecule-name editing."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_chem
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.window_refusals


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor" version="26.07">'
	'<molecule id="m" name="before" role="source">'
	'<atom id="a" name="C"><point x="1" y="2"/>'
	'<v:opaque retained="yes"/></atom></molecule>'
	'<molecule id="other" name="unrelated"><atom id="b" name="O">'
	'<point x="3" y="4"/></atom></molecule></cdml>'
)


#============================================
def _action(window: object, text: str) -> PySide6.QtGui.QAction:
	"""Find one public top-level-menu action by visible text."""
	for menu_action in window.menuBar().actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for action in menu.actions():
			if action.text() == text:
				return action
	raise AssertionError(f"No public action is labelled {text!r}")


#============================================
def _warnings(
		monkeypatch: object,
		) -> list[ferrum_qt.dialogs.refusal_presenter.RefusalPresentation]:
	"""Capture actionable warnings without opening another modal surface."""
	warnings: list[ferrum_qt.dialogs.refusal_presenter.RefusalPresentation] = []

	def record(_window: object, request: object) -> None:
		"""Retain one typed refusal presentation."""
		warnings.append(ferrum_qt.dialogs.refusal_presenter.present_refusal(request))

	monkeypatch.setattr(ferrum_qt.ferrum.window_refusals, "show_refusal", record)
	return warnings


#============================================
def _snapshot_facts(snapshot: object) -> tuple[str, int, str, bool]:
	"""Return the complete public snapshot state for nonmutation checks."""
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


#============================================
def _root_name(tab: object, index: int = 0) -> str | None:
	"""Read one installed direct-root authored name."""
	return tab.current_document_observation().projection.molecules[index].name


#============================================
def _new_window_tab() -> tuple[object, object]:
	"""Create one ordinary window with a current named Rust-owned tab."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		SOURCE, "names.cdml",
	)
	window._register_native_tab(tab, activate=True)
	return window, tab


#============================================
def _close_clean(window: object, tab: object) -> None:
	"""Close the test tab only after restoring its loaded baseline."""
	while tab.current_snapshot.is_dirty:
		_action(window, "Undo").trigger()
	window._close_tab_at(window.centralWidget().indexOf(tab))
	window.deleteLater()


#============================================
def test_public_action_commits_history_reopen_and_selection(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""The public action preserves exact text, selection, history, and saved CDML facts."""
	window, tab = _new_window_tab()
	action = _action(window, "Set Molecule Name...")
	try:
		assert not action.isEnabled()
		tab.select_atom("a")
		window._refresh_actions()
		assert action.isEnabled()
		before_selection = tuple(
			(target.kind, target.identifier)
			for target in tab.selected_molecule_information_targets()
		)
		monkeypatch.setattr(
			PySide6.QtWidgets.QInputDialog, "getText",
			lambda *_args, **_kwargs: ("  ", True),
		)
		action.trigger()
		changed = tab.current_snapshot
		assert _root_name(tab) == "  " and 'role="source"' in changed.cdml
		assert tuple(
			(target.kind, target.identifier)
			for target in tab.selected_molecule_information_targets()
		) == before_selection
		_action(window, "Undo").trigger()
		assert _root_name(tab) == "before"
		_action(window, "Redo").trigger()
		assert _root_name(tab) == "  "
		reopened = ferrum_chem.DocumentSession.load(tab.current_snapshot.cdml)
		assert reopened.observe(0).projection.molecules[0].name == "  "
	finally:
		_close_clean(window, tab)
		del qapp


#============================================
def test_same_cancel_clear_and_multiple_root_reachability(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""No-op/cancel remain exact, empty clears, and cross-root selection is refused."""
	window, tab = _new_window_tab()
	action = _action(window, "Set Molecule Name...")
	try:
		tab.select_atoms(("a", "b"))
		window._refresh_actions()
		assert not action.isEnabled()
		tab.select_atom("a")
		window._refresh_actions()
		before = _snapshot_facts(tab.current_snapshot)
		monkeypatch.setattr(
			PySide6.QtWidgets.QInputDialog, "getText",
			lambda *_args, **_kwargs: ("before", True),
		)
		action.trigger()
		assert _snapshot_facts(tab.current_snapshot) == before
		monkeypatch.setattr(
			PySide6.QtWidgets.QInputDialog, "getText",
			lambda *_args, **_kwargs: ("ignored", False),
		)
		action.trigger()
		assert _snapshot_facts(tab.current_snapshot) == before
		monkeypatch.setattr(
			PySide6.QtWidgets.QInputDialog, "getText",
			lambda *_args, **_kwargs: ("", True),
		)
		action.trigger()
		assert _root_name(tab) is None
	finally:
		_close_clean(window, tab)
		del qapp


#============================================
def test_post_dialog_selection_and_tab_switch_fences_are_nonmutating(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""A changed child target or active tab cannot consume the frozen name intent."""
	window, tab = _new_window_tab()
	other_tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		SOURCE, "other-names.cdml",
	)
	window._register_native_tab(other_tab, activate=False)
	action = _action(window, "Set Molecule Name...")
	warnings = _warnings(monkeypatch)
	try:
		tab.select_atom("a")
		window._refresh_actions()
		before = _snapshot_facts(tab.current_snapshot)

		def change_selection(*_args: object, **_kwargs: object) -> tuple[str, bool]:
			"""Move the selection after capture but before accepted delivery."""
			tab.select_atom("b")
			return "blocked", True

		monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getText", change_selection)
		action.trigger()
		assert _snapshot_facts(tab.current_snapshot) == before
		assert warnings[-1].technical_details is not None
		assert "selection changed" in warnings[-1].technical_details

		tab.select_atom("a")
		window.centralWidget().setCurrentIndex(window.centralWidget().indexOf(tab))
		window._refresh_actions()

		def switch_tab(*_args: object, **_kwargs: object) -> tuple[str, bool]:
			"""Activate another live tab while the modal request is open."""
			window.centralWidget().setCurrentWidget(other_tab)
			return "blocked", True

		monkeypatch.setattr(PySide6.QtWidgets.QInputDialog, "getText", switch_tab)
		action.trigger()
		assert _snapshot_facts(tab.current_snapshot) == before
	finally:
		window.centralWidget().setCurrentWidget(other_tab)
		window._close_tab_at(window.centralWidget().indexOf(other_tab))
		window.centralWidget().setCurrentWidget(tab)
		_close_clean(window, tab)
		del qapp


#============================================
def test_typed_name_failure_warns_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object) -> None:
	"""An XML-invalid accepted string remains a typed backend rejection."""
	window, tab = _new_window_tab()
	warnings = _warnings(monkeypatch)
	try:
		tab.select_atom("a")
		window._refresh_actions()
		before = _snapshot_facts(tab.current_snapshot)
		monkeypatch.setattr(
			PySide6.QtWidgets.QInputDialog, "getText",
			lambda *_args, **_kwargs: ("bad\x00name", True),
		)
		_action(window, "Set Molecule Name...").trigger()
		assert _snapshot_facts(tab.current_snapshot) == before
		assert warnings and warnings[-1].title == "Action Not Available"
		assert warnings[-1].technical_details is not None
	finally:
		_close_clean(window, tab)
		del qapp
