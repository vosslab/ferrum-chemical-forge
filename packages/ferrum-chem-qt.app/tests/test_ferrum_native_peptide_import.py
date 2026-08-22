"""Behavior coverage for strict Ferrum peptide-template insertion."""

# Standard Library
import dataclasses
import os
import pathlib
import threading


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets
import ferrum_chem
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.peptide_import


_EMPTY_CDML = "<cdml xmlns='urn:ferrum:cdml'/>"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _ControlledFailure:
	"""Immutable failure facts emitted by the controlled delivery seam."""

	message: str


#============================================
class _ControlledPeptideWorker(PySide6.QtCore.QThread):
	"""One test-only worker whose delivery is released explicitly by the test."""

	prepared = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	#============================================
	def __init__(self, prepared: object | None, failure: str | None) -> None:
		"""Store one terminal outcome until the test releases Ferrum delivery."""
		super().__init__()
		self._prepared = prepared
		self._failure = failure
		self._released = threading.Event()
		self._delivery_cancelled = False

	#============================================
	@property
	def delivery_cancelled(self) -> bool:
		"""Return whether the host invalidated future result delivery."""
		return self._delivery_cancelled

	#============================================
	def cancel_delivery(self) -> None:
		"""Match production delivery-only cancellation semantics."""
		self._delivery_cancelled = True
		self.requestInterruption()

	#============================================
	def release(self) -> None:
		"""Allow the worker to emit its configured terminal result."""
		self._released.set()

	#============================================
	def run(self) -> None:
		"""Wait for explicit test release, then emit one uncancelled outcome."""
		self._released.wait()
		if self._delivery_cancelled or self.isInterruptionRequested():
			return
		if self._failure is not None:
			self.failed.emit(_ControlledFailure(self._failure))
			return
		self.prepared.emit(self._prepared)


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one offscreen application without legacy document ownership."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def _window_with_tab() -> tuple[object, object]:
	"""Create one ordinary Rust-owned document tab for a behavior test."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "native.cdml",
	)
	window._register_native_tab(tab, activate=True)
	return window, tab


#============================================
def _install_controlled_worker(window: object, prepared: object | None,
		failure: str | None) -> _ControlledPeptideWorker:
	"""Install one deliberate worker-construction seam on the public host."""
	worker = _ControlledPeptideWorker(prepared, failure)
	window._create_peptide_preparation_worker = lambda _sequence, _placement: worker
	return worker


#============================================
def _await_worker(qtbot: object, worker: _ControlledPeptideWorker) -> None:
	"""Release one controlled worker and wait through Qt's finished signal."""
	with qtbot.waitSignal(worker.finished):
		worker.release()


#============================================
def _prepared_molecule() -> object:
	"""Return one normal frozen insertion without exercising the peptide ABI."""
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	molecule = ferrum_chem.prepare_smiles_molecule_v1("CCO", placement)
	return molecule


#============================================
def test_worker_passes_exact_unmodified_sequence_to_the_native_contract() -> None:
	"""Whitespace and case reach the required Ferrum extension symbol unchanged."""
	passed = []

	def prepare(sequence: str, placement: object) -> object:
		"""Record immutable worker input without interpreting the sequence."""
		passed.append((sequence, placement))
		return object()

	operation = ferrum_chem.prepare_supported_peptide_template_molecule_v1
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	worker = (
		ferrum_qt.ferrum.peptide_import.
		FerrumNativePeptidePreparationWorker._from_fixture(" AN ", placement, prepare)
	)
	worker.run()

	assert callable(operation)
	assert passed == [(" AN ", placement)]
	worker.deleteLater()


#============================================
def test_cancelled_prompt_leaves_document_and_action_ready(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Dialog cancellation is the only text-input no-op path."""
	del qapp
	window, tab = _window_with_tab()
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("AN", False),
	)
	window._import_peptide_action.trigger()

	assert not tab.is_dirty
	assert window._import_peptide_action.isEnabled()
	tab.dispose()
	window.deleteLater()


#============================================
def test_accepted_blank_reports_failure_without_document_edit(
		qtbot: object, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An accepted blank reaches Ferrum preparation rather than being discarded."""
	window, tab = _window_with_tab()
	worker = _install_controlled_worker(window, None, "sequence must not be empty")
	warnings = []
	monkeypatch.setattr(
		window, "_show_edit_refusal", lambda request: warnings.append(request),
	)
	monkeypatch.setattr(
		PySide6.QtWidgets.QInputDialog, "getText", lambda *_args: ("", True),
	)
	window._import_peptide_action.trigger()
	_await_worker(qtbot, worker)

	assert not tab.is_dirty
	assert warnings[-1].outcome.value == "unavailable_operation"
	tab.dispose()
	window.deleteLater()


#============================================
def test_prepared_template_uses_ordinary_history_and_persistence(
		qtbot: object, tmp_path: pathlib.Path,
		) -> None:
	"""The shared delivery fence commits through ordinary Rust document behavior."""
	window, tab = _window_with_tab()
	worker = _install_controlled_worker(window, _prepared_molecule(), None)
	assert window.start_supported_peptide_import("AN")
	_await_worker(qtbot, worker)
	assert tab.is_dirty

	tab.undo()
	undone = not tab.is_dirty
	tab.redo()
	destination = tmp_path / "peptide-template.cdml"
	tab.save_atomic(destination)
	reopened = ferrum_chem.DocumentSession.load(destination.read_text(encoding="utf-8"))
	projection = reopened.observe_render(0).document.projection

	assert undone
	assert projection.molecules
	window.close()
	window.deleteLater()


#============================================
def test_stale_and_cancelled_delivery_leave_the_document_unchanged(
		qtbot: object,
		) -> None:
	"""A revision change or cancellation wins over one delayed prepared result."""
	window, tab = _window_with_tab()
	worker = _install_controlled_worker(window, _prepared_molecule(), None)
	assert window.start_supported_peptide_import("AN")
	tab.insert_prepared_molecule(_prepared_molecule())
	after_intervening_edit = tab.current_snapshot
	_await_worker(qtbot, worker)
	assert tab.current_snapshot == after_intervening_edit

	worker = _install_controlled_worker(window, _prepared_molecule(), None)
	assert window.start_supported_peptide_import("AN")
	window._cancel_peptide_action.trigger()
	_await_worker(qtbot, worker)

	assert tab.current_snapshot == after_intervening_edit
	tab.dispose()
	window.deleteLater()


#============================================
def test_close_actions_block_then_cancelled_delivery_leaves_no_edit(
		qtbot: object, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Tab and window close share the delivery cancellation boundary."""
	window, tab = _window_with_tab()
	worker = _install_controlled_worker(window, _prepared_molecule(), None)
	monkeypatch.setattr(window, "_show_edit_refusal", lambda _request: None)
	assert window.start_supported_peptide_import("AN")
	window._close_action.trigger()
	remained_open = window.centralWidget().indexOf(tab) >= 0
	window.close()
	_await_worker(qtbot, worker)

	assert remained_open
	assert not tab.is_dirty
	window.deleteLater()
