"""Behavior coverage for selected source-fact molecule inspection."""

# Standard Library
import dataclasses
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import PySide6.QtGui
import pytest
import ferrum_chem

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_main_window
import ferrum_qt.native.ferrum_native_molecule_inspection


_SOURCE = """<cdml version='26.08'><molecule id='mol-1' name='Ethanal'>
<atom id='atom-c' name='C' charge='0'><point x='10' y='20'/></atom>
<atom id='atom-o' name='O' charge='0'><point x='40' y='20'/></atom>
<bond id='bond-co' start='atom-c' end='atom-o' type='n2'/>
</molecule></cdml>"""

_MULTI_SOURCE = """<cdml version='26.08'>
<molecule id='methane' name='Methane'>
 <atom id='atom-c' name='C'><point x='0' y='0'/></atom>
</molecule>
<molecule id='water' name='Water'>
 <atom id='atom-o' name='O' explicit_hydrogens='2'><point x='3' y='0'/></atom>
</molecule>
</cdml>"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide a reusable offscreen Qt application for the native host."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _Child:
	"""Minimal copied projection child fact for ambiguity boundaries."""

	source_id: str
	source_order: int


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _Root:
	"""Minimal copied direct-root projection fact for ambiguity boundaries."""

	id: str | None
	projection_key: str
	source_id: str | None
	source_order: int
	atoms: tuple[_Child, ...]
	bonds: tuple[_Child, ...] = ()


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _Target:
	"""Minimal selected render-target fact for resolver behavior."""

	kind: str
	identifier: str | None
	source_order: int


#============================================
class _SelectionTab:
	"""Small public-selection collaborator with no session or render ownership."""

	def __init__(self, roots: tuple[_Root, ...],
			targets: tuple[_Target, ...] | None = None) -> None:
		"""Retain controlled projection facts for resolver-only behavior."""
		self.requires_refresh = False
		self._roots = roots
		self._targets = targets or (_Target("atom", "atom-source", 7),)

	def selected_molecule_information_targets(self) -> tuple[_Target, ...]:
		"""Return the complete controlled selection."""
		return self._targets

	def current_document_observation(self) -> object:
		"""Expose the copied direct-root projection facts."""
		return type("Observation", (), {"projection": type("Projection", (), {
			"molecules": self._roots,
		})()})()


#============================================
def test_resolver_refuses_duplicate_child_facts_and_idless_roots(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Only one literal child fact in one id-bearing root is a valid address."""
	del qapp
	child = _Child("atom-source", 7)
	valid = _Root("opaque-root", "root-key", "root-source", 3, (child,))
	duplicate = _Root("opaque-root", "root-key", "root-source", 3, (child, child))
	idless = _Root(None, "root-key", "root-source", 3, (child,))
	resolve = ferrum_qt.native.ferrum_native_molecule_inspection.selected_durable_molecule_address
	assert resolve(_SelectionTab((valid,))).molecule_id == "opaque-root"
	assert resolve(_SelectionTab((duplicate,))) is None
	assert resolve(_SelectionTab((valid, valid))) is None
	assert resolve(_SelectionTab((idless,))) is None
	artwork = _SelectionTab((valid,), (_Target("plus", "plus-1", 0),))
	assert resolve(artwork) is None


#============================================
def test_resolver_deduplicates_roots_and_orders_multiple_molecules(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Selected atoms and bonds map globally, then deduplicate by direct root."""
	del qapp
	first_atom = _Child("a-first", 0)
	first_bond = _Child("b-first", 2)
	second_atom = _Child("a-second", 0)
	first = _Root("root-first", "key-first", "first", 1, (first_atom,), (first_bond,))
	second = _Root("root-second", "key-second", "second", 4, (second_atom,))
	targets = (
		_Target("atom", "a-second", 0),
		_Target("bond", "b-first", 2),
		_Target("atom", "a-first", 0),
	)
	resolve = (
		ferrum_qt.native.ferrum_native_molecule_inspection.
		selected_durable_molecule_addresses
	)
	addresses = resolve(_SelectionTab((first, second), targets))
	assert addresses is not None
	assert tuple(address.source_id for address in addresses) == ("first", "second")


#============================================
def test_public_action_is_reachable_for_supported_selected_atoms_or_bonds(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The ordinary host accepts one or more children from durable direct roots."""
	del qapp
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_SOURCE, "ethanal.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window._refresh_actions()
		assert window._inspect_selected_molecule_action.text() == "Molecule Information..."
		assert not window._inspect_selected_molecule_action.isEnabled()
		tab.select_atom("atom-c")
		window._refresh_actions()
		assert window._inspect_selected_molecule_action.isEnabled()
		tab.select_atoms(("atom-c", "atom-o"))
		window._refresh_actions()
		assert window._inspect_selected_molecule_action.isEnabled()
		tab.select_bond("bond-co")
		window._refresh_actions()
		assert window._inspect_selected_molecule_action.isEnabled()
	finally:
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_triggered_action_delivers_native_information_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The ordinary QAction reaches RDKit and preserves document and scene state."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_SOURCE, "ethanal.cdml",
	)
	shown = []
	monkeypatch.setattr(window, "_show_molecule_information_dialog", shown.append)
	try:
		window._register_native_tab(tab, activate=True)
		tab.select_atom("atom-c")
		projection = tab._controller.projection
		selected = projection.selected_durable_targets()
		before = tab.current_snapshot
		window._inspect_selected_molecule_action.trigger()
		intent = window._molecule_inspection_intent
		assert intent is not None and intent.worker.wait(10000)
		qapp.processEvents()
		assert len(shown) == 1
		result = shown[0]
		assert type(result) is ferrum_chem.DocumentMoleculeInformationV1
		assert len(result.records) == 1 and result.combined_selection is None
		assert result.records[0].source_facts.source_id == "mol-1"
		assert result.records[0].composition.formula == "CH2O"
		text = (
			ferrum_qt.native.ferrum_native_molecule_inspection.
			format_molecule_information(result)
		)
		assert "Name: Ethanal" in text and "Source ID: mol-1" in text
		assert "Authored graph: 2 atoms, 1 bond" in text
		assert "Authored elements: C: 1, O: 1" in text
		assert "Complete authored formal charge: +0" in text
		assert "Formula: CH2O" in text and "Monoisotopic mass:" in text
		assert tab.current_snapshot == before
		assert tab._controller.projection is projection
		assert projection.selected_durable_targets() == selected
	finally:
		if window._molecule_inspection_intent is not None:
			window._cancel_document_molecule_inspection()
			window._molecule_inspection_intent.worker.wait(10000)
			qapp.processEvents()
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_multi_root_dialog_is_selectable_accessible_and_combined(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The read-only dialog presents individual roots and one combined receipt."""
	del qapp
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_MULTI_SOURCE, "mixture.cdml",
	)
	dialog = None
	try:
		tab.select_atoms(("atom-c", "atom-o"))
		addresses = (
			ferrum_qt.native.ferrum_native_molecule_inspection.
			selected_durable_molecule_addresses(tab)
		)
		assert addresses is not None
		observation = tab.current_document_observation()
		result = ferrum_chem.inspect_document_molecule_information_v1(
			observation, observation.snapshot.revision, observation.snapshot.digest,
			tuple(address.molecule_id for address in addresses),
		)
		dialog = (
			ferrum_qt.native.ferrum_native_molecule_inspection.
			FerrumNativeMoleculeInformationDialog(result, tab)
		)
		details = dialog.findChild(PySide6.QtWidgets.QPlainTextEdit)
		assert dialog.windowTitle() == "Molecule Information"
		assert dialog.isModal() and dialog.minimumWidth() >= 600
		assert details is not None and details.isReadOnly()
		assert details.accessibleName() == "Molecule chemistry details"
		assert "Ferrum Rust" in details.accessibleDescription()
		assert "Formula: CH4" in details.toPlainText()
		assert "Formula: H2O" in details.toPlainText()
		assert "Combined selection" in details.toPlainText()
		assert "Formula: CH6O" in details.toPlainText()
		buttons = dialog.findChild(PySide6.QtWidgets.QDialogButtonBox)
		assert buttons is not None
		assert buttons.standardButtons() == PySide6.QtWidgets.QDialogButtonBox.StandardButton.Close
	finally:
		if dialog is not None:
			dialog.reject()
			dialog.deleteLater()
		tab.dispose()
		tab.deleteLater()


#============================================
def test_stale_or_foreign_delivery_never_shows_a_modal(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The one delivery fence suppresses stale result and failure callbacks."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_SOURCE, "ethanal.cdml",
	)
	shown = []
	monkeypatch.setattr(
		PySide6.QtWidgets.QMessageBox, "information",
		lambda _parent, title, text: shown.append(("info", title, text)),
	)
	monkeypatch.setattr(
		window, "_show_native_file_warning",
		lambda title, text: shown.append(("warning", title, text)),
	)
	try:
		window._register_native_tab(tab, activate=True)
		tab.select_atom("atom-c")
		window._inspect_selected_molecule_action.trigger()
		intent = window._molecule_inspection_intent
		assert intent is not None
		result = ferrum_chem.inspect_document_molecule_information_v1(
			tab.current_document_observation(), intent.revision, intent.digest,
			tuple(address.molecule_id for address in intent.addresses),
		)
		foreign = object()
		window._on_document_molecule_inspected(foreign, result)
		window._on_document_molecule_inspection_failed(
			foreign, ferrum_qt.native.ferrum_native_molecule_inspection.
			FerrumNativeMoleculeInspectionFailure("foreign"),
		)
		other = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
			_SOURCE, "other.cdml",
		)
		window._register_native_tab(other, activate=True)
		window._on_document_molecule_inspected(intent.worker, result)
		window._on_document_molecule_inspection_failed(
			intent.worker, ferrum_qt.native.ferrum_native_molecule_inspection.
			FerrumNativeMoleculeInspectionFailure("inactive"),
		)
		intent.worker.cancel_delivery()
		window._on_document_molecule_inspected(intent.worker, result)
		window._on_document_molecule_inspection_failed(
			intent.worker, ferrum_qt.native.ferrum_native_molecule_inspection.
			FerrumNativeMoleculeInspectionFailure("cancelled"),
		)
		assert shown == []
		assert intent.worker.wait(10000)
		qapp.processEvents()
		window._close_tab_at(window._tab_widget.indexOf(other))
	finally:
		if window._molecule_inspection_intent is not None:
			window._cancel_document_molecule_inspection()
			window._molecule_inspection_intent.worker.wait(10000)
			qapp.processEvents()
		if window._tab_widget.indexOf(tab) >= 0:
			window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


#============================================
def test_close_cancels_delivery_and_retains_source_until_worker_finishes(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Close leaves the source alive until cancellation and worker teardown finish."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		_SOURCE, "ethanal.cdml",
	)
	warnings = []
	monkeypatch.setattr(
		window, "_show_native_file_warning",
		lambda title, text: warnings.append((title, text)),
	)
	try:
		window._register_native_tab(tab, activate=True)
		tab.select_atom("atom-c")
		window._inspect_selected_molecule_action.trigger()
		intent = window._molecule_inspection_intent
		assert intent is not None
		window._close_tab_at(window._tab_widget.indexOf(tab))
		assert window._tab_widget.indexOf(tab) >= 0
		event = PySide6.QtGui.QCloseEvent()
		window.closeEvent(event)
		assert not event.isAccepted() and intent.worker.delivery_cancelled
		assert intent.worker.wait(10000)
		qapp.processEvents()
		assert window._molecule_inspection_intent is None
		window._close_tab_at(window._tab_widget.indexOf(tab))
		assert window._tab_widget.indexOf(tab) < 0 and warnings
	finally:
		window.deleteLater()
