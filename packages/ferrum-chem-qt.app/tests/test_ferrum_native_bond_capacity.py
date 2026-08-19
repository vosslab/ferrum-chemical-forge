"""Behavior coverage for the Ferrum Bond Capacity Check action."""

import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

import ferrum_chem

import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.main_window
import ferrum_qt.ferrum.bond_capacity
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.window_refusals


_SOURCE = """\
<cdml version="26.08"><molecule id="mixed">
 <atom id="excess-c" name="C" explicit_hydrogens="4"><point x="0" y="0"/></atom>
 <atom id="within-o" name="O" charge="0"><point x="1" y="0"/></atom>
 <bond id="mixed-bond" start="excess-c" end="within-o" type="n1"/>
</molecule></cdml>
"""


def _click_visible_menu_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Use the ordinary labelled menu command a person can see and click."""
	for menu_action in window.menuBar().actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for candidate in menu.actions():
			if candidate.text().replace("&", "") != label:
				continue
			PySide6.QtTest.QTest.mouseClick(
				window.menuBar(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				window.menuBar().actionGeometry(menu_action).center(),
			)
			qapp.processEvents()
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(candidate).center(),
			)
			qapp.processEvents()
			return
	raise AssertionError(f"No visible menu action is labelled {label!r}")


class _ImmediateBondCapacityWorker(PySide6.QtCore.QThread):
	"""Deliver one real Rust receipt through the normal queued Qt boundary."""

	completed = PySide6.QtCore.Signal(object)
	failed = PySide6.QtCore.Signal(object)

	def __init__(self, observation: object, revision: int, digest: str,
			addresses: tuple) -> None:
		super().__init__()
		self._arguments = (
			observation, revision, digest,
			tuple(address.molecule_id for address in addresses),
		)
		self._delivery_cancelled = False

	@property
	def delivery_cancelled(self) -> bool:
		"""Expose the production worker's cancellation contract."""
		return self._delivery_cancelled

	def cancel_delivery(self) -> None:
		"""Invalidate a queued receipt before the Qt event turn consumes it."""
		self._delivery_cancelled = True

	def start(self) -> None:
		"""Queue the bounded Ferrum result without a timing-dependent thread wait."""
		if not self._delivery_cancelled:
			self.completed.emit(ferrum_chem.inspect_document_bond_capacity_v1(
				*self._arguments,
			))
		self.finished.emit()


class _FailingBondCapacityWorker(_ImmediateBondCapacityWorker):
	"""Deliver a typed operation failure through the same queued Qt boundary."""

	def start(self) -> None:
		"""Queue program-state guidance without simulating elapsed time."""
		self.failed.emit(
			ferrum_qt.ferrum.bond_capacity.FerrumNativeBondCapacityFailure(
				"operation unavailable",
			),
		)
		self.finished.emit()


class _DeferredBondCapacityWorker(_ImmediateBondCapacityWorker):
	"""Hold one real receipt until the test chooses its delivery turn."""

	current: "_DeferredBondCapacityWorker | None" = None

	def start(self) -> None:
		"""Retain one otherwise ordinary worker without a timing wait."""
		type(self).current = self

	def deliver(self) -> None:
		"""Emit a receipt even when cancellation raced the queued delivery."""
		self.completed.emit(ferrum_chem.inspect_document_bond_capacity_v1(
			*self._arguments,
		))
		self.finished.emit()


def test_public_bond_capacity_action_projects_rust_result_without_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A durable atom selection reaches its read-only Rust receipt."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_SOURCE, "mixed.cdml",
	)
	shown = []
	monkeypatch.setattr(
		ferrum_qt.ferrum.bond_capacity,
		"FerrumNativeBondCapacityWorker", _ImmediateBondCapacityWorker,
	)
	monkeypatch.setattr(
		ferrum_qt.ferrum.bond_capacity.FerrumNativeBondCapacityDialog,
		"exec", lambda dialog: shown.append(dialog.details_text),
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tab.select_atom("excess-c")
		before = tab.current_snapshot
		selection = tab.selected_molecule_information_targets()

		_click_visible_menu_action(window, "Check Bond Capacity...", qapp)
		qapp.processEvents()

		assert shown and all(fragment in shown[0] for fragment in (
			"excess-c: demand", "within-o: demand", "charge absent (used as 0)",
			"authored charge +0", "authored explicit H 4", "explicit H absent (used as 0)",
		))
		assert tab.current_snapshot == before
		assert tab.selected_molecule_information_targets() == selection
	finally:
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


def test_cancelled_bond_capacity_action_suppresses_queued_result(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Cancellation contains a pending receipt without editing its source tab."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_SOURCE, "mixed.cdml",
	)
	shown = []
	monkeypatch.setattr(
		ferrum_qt.ferrum.bond_capacity,
		"FerrumNativeBondCapacityWorker", _DeferredBondCapacityWorker,
	)
	monkeypatch.setattr(
		ferrum_qt.ferrum.bond_capacity.FerrumNativeBondCapacityDialog,
		"exec", lambda dialog: shown.append(dialog.details_text),
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tab.select_atom("excess-c")
		before = tab.current_snapshot

		_click_visible_menu_action(window, "Check Bond Capacity...", qapp)
		_click_visible_menu_action(window, "Cancel Bond Capacity Check", qapp)
		worker = _DeferredBondCapacityWorker.current
		assert worker is not None
		worker.deliver()
		qapp.processEvents()

		assert shown == []
		assert tab.current_snapshot == before
	finally:
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()


def test_bond_capacity_operation_failure_is_visible_without_a_document_edit(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A current operation failure stays separate from a chemistry finding."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_SOURCE, "mixed.cdml",
	)
	refusals: list[ferrum_qt.dialogs.refusal_presenter.RefusalRequest] = []
	monkeypatch.setattr(
		ferrum_qt.ferrum.bond_capacity,
		"FerrumNativeBondCapacityWorker", _FailingBondCapacityWorker,
	)
	monkeypatch.setattr(
		ferrum_qt.ferrum.window_refusals, "show_refusal",
		lambda _window, request: refusals.append(request),
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tab.select_atom("excess-c")
		before = tab.current_snapshot
		selection = tab.selected_molecule_information_targets()

		_click_visible_menu_action(window, "Check Bond Capacity...", qapp)
		qapp.processEvents()

		assert refusals
		presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(refusals[0])
		assert presentation.title == "Action Not Available"
		assert presentation.technical_details == "operation unavailable"
		assert tab.current_snapshot == before
		assert tab.selected_molecule_information_targets() == selection
	finally:
		window._close_tab_at(window._tab_widget.indexOf(tab))
		window.deleteLater()
