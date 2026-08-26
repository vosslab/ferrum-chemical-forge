"""Behavior coverage for ordinary Ferrum user templates."""

# Standard Library
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window


_TEMPLATE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><standard line_width="9"/><paper id="paper"/>
<molecule id="source" name="Reusable pair">
 <atom id="a" name="C"><point x="0" y="0"/></atom>
 <atom id="b" name="O"><point x="10" y="0"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/>
</molecule></cdml>
"""


#============================================
def _window_with_tab(
		directory: pathlib.Path, cdml: str = "<cdml xmlns='urn:ferrum:cdml' version='26.07'/>",
		) -> tuple[object, object]:
	"""Return one configured Ferrum window with a selected Rust-owned tab."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow(
		user_template_directory=directory,
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		cdml, "template-test.cdml",
	)
	window._register_native_tab(tab, activate=True)
	return window, tab


#============================================
def _close_window(window: object, tab: object) -> None:
	"""Undo test edits and close the Ferrum window without a dirty close prompt."""
	while tab.current_snapshot.is_dirty:
		tab.undo()
	window._close_tab_at(window.centralWidget().indexOf(tab))
	window.deleteLater()


#============================================
def _snapshot_facts(snapshot: object) -> tuple[str, int, str, bool]:
	"""Return public backend facts for semantic nonmutation checks."""
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


#============================================
class _StalePlacementRefusalObserver(PySide6.QtCore.QObject):
	"""Acknowledge and retain visible stale-placement refusal evidence."""

	def __init__(self, refusals: list[tuple[str, str, str, str]]) -> None:
		"""Keep the caller-owned visible-refusal evidence."""
		super().__init__()
		self._refusals = refusals

	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Queue acknowledgement after the intended modal enters its event loop."""
		if event.type() != PySide6.QtCore.QEvent.Type.Show:
			return False
		if not isinstance(watched, PySide6.QtWidgets.QMessageBox):
			return False
		if watched.accessibleName() != "Action Not Available":
			return False
		PySide6.QtCore.QTimer.singleShot(0, lambda: self._capture_and_dismiss(watched))
		return False

	def _capture_and_dismiss(self, dialog: PySide6.QtWidgets.QMessageBox) -> None:
		"""Record and acknowledge the actionable public refusal."""
		self._refusals.append((dialog.windowTitle(), dialog.accessibleName(),
			dialog.text(), dialog.detailedText()))
		button = dialog.button(PySide6.QtWidgets.QMessageBox.StandardButton.Ok)
		if button is None:
			raise AssertionError("Stale template refusal has no visible acknowledgement")
		PySide6.QtTest.QTest.mouseClick(
			button, PySide6.QtCore.Qt.MouseButton.LeftButton,
		)


#============================================
def test_catalog_choice_places_only_authored_scale_molecule_at_scene_click(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path) -> None:
	"""One visible placement intent maps the click through the shared point policy."""
	directory = tmp_path / "templates"
	directory.mkdir()
	(directory / "pair.cdml").write_text(_TEMPLATE, encoding="utf-8")
	window, tab = _window_with_tab(directory)
	try:
		entry = window.user_template_catalog.entries[0]
		window.start_user_template_placement(entry.catalog_key)
		viewport_point = PySide6.QtCore.QPoint(120, 90)
		expected_anchor = tab.view.snap_authored_scene_point(
			tab.view.mapToScene(viewport_point),
		)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			pos=viewport_point,
		)
		qapp.processEvents()
		molecule = tab.current_document_observation().projection.molecules[0]
		atom_a, atom_b = molecule.atoms

		assert (
			(atom_a.position.x + atom_b.position.x) / 2.0,
			(atom_a.position.y + atom_b.position.y) / 2.0,
			atom_b.position.x - atom_a.position.x,
		) == pytest.approx((expected_anchor.x(), expected_anchor.y(), 10.0), abs=0.02)
	finally:
		_close_window(window, tab)
		del qapp


#============================================
def test_save_and_refresh_publish_eligible_snapshot_without_saving_document(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		monkeypatch: pytest.MonkeyPatch) -> None:
	"""Template publication preserves session state and isolates a malformed neighbor."""
	directory = tmp_path / "templates"
	window, tab = _window_with_tab(directory, _TEMPLATE)
	messages = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QMessageBox, "information",
		lambda *_args: messages.append(str(_args[-1])),
	)
	try:
		before = tab.current_snapshot
		assert (
			window.save_active_as_user_template_to_path(directory / "saved.cdml"),
			_snapshot_facts(tab.current_snapshot),
			window.user_template_catalog.entries[0].label,
		) == (True, _snapshot_facts(before), "Reusable pair")

		(directory / "broken.cdml").write_text("<cdml xmlns='urn:ferrum:cdml'>", encoding="utf-8")
		snapshot = window.refresh_user_templates()
		assert (
			bool(snapshot.entries),
			bool(snapshot.failures),
			snapshot.failures[0].source_name in messages[-1],
		) == (True, True, True)
	finally:
		_close_window(window, tab)
		del qapp


#============================================
def test_escape_and_stale_provenance_cancel_without_a_template_mutation(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path) -> None:
	"""Cancelled or obsolete click intents leave the authoritative state unchanged."""
	directory = tmp_path / "templates"
	directory.mkdir()
	(directory / "pair.cdml").write_text(_TEMPLATE, encoding="utf-8")
	window, tab = _window_with_tab(directory)
	refusals: list[tuple[str, str, str, str]] = []
	observer = _StalePlacementRefusalObserver(refusals)
	try:
		entry = window.user_template_catalog.entries[0]
		baseline = tab.current_snapshot
		escape_started = window.start_user_template_placement(entry.catalog_key)
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		assert (
			escape_started,
			_snapshot_facts(tab.current_snapshot),
		) == (True, _snapshot_facts(baseline))

		stale_started = window.start_user_template_placement(entry.catalog_key)
		result = tab.insert_user_template(entry.native_plan, 0.0, 0.0)
		accepted = result.operation.observation.snapshot
		qapp.installEventFilter(observer)
		try:
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				pos=PySide6.QtCore.QPoint(),
			)
		finally:
			qapp.removeEventFilter(observer)
		qapp.processEvents()
		assert (
			stale_started,
			_snapshot_facts(tab.current_snapshot),
		) == (True, _snapshot_facts(accepted))
		assert any(
			accessible_name == "Action Not Available"
			and "Select the required item or change the drawing, then try again." in text
			and "The document or catalog changed; choose the template again." in details
			for _title, accessible_name, text, details in refusals
		)
	finally:
		_close_window(window, tab)
		del qapp
