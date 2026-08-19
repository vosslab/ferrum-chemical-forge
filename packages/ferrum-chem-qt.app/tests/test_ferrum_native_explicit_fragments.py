"""Visible ordinary-product behavior for Rust-owned explicit fragments."""

import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import PySide6.QtCore
import PySide6.QtWidgets
import ferrum_chem

import ferrum_qt.main_window
import ferrum_qt.ferrum.document_tab


CDML = (
	"<cdml><molecule id='m'><atom id='a' name='C'><point x='0' y='0'/></atom>"
	"<atom id='b' name='O'><point x='10' y='0'/></atom>"
	"<bond id='ab' type='n1' start='a' end='b'/></molecule></cdml>"
)


def _trigger_menu_action(window: PySide6.QtWidgets.QMainWindow, label: str) -> None:
	"""Invoke one labelled ordinary Chemistry action from the visible menu."""
	for top_level in window.menuBar().actions():
		menu = top_level.menu()
		if menu is None:
			continue
		for action in menu.actions():
			if action.text().replace("&", "") != label:
				continue
			action.trigger()
			return
	raise AssertionError(f"No visible menu action is labelled {label!r}")


def _active_dialog() -> PySide6.QtWidgets.QDialog:
	"""Return the modal dialog after its nested event loop has begun."""
	dialog = PySide6.QtWidgets.QApplication.activeModalWidget()
	assert isinstance(dialog, PySide6.QtWidgets.QDialog)
	return dialog


def test_visible_create_and_view_fragment_uses_captured_native_selection(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Create and View surface Rust's authoritative owner and label facts."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(CDML, "part.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		tab._controller.projection.select_durable((("bond", "ab"),))
		window._refresh_actions()
		qapp.processEvents()
		selected_before_create = tab._controller.projection.selected_durable_targets()

		def name_fragment() -> None:
			dialog = _active_dialog()
			field = dialog.findChild(PySide6.QtWidgets.QLineEdit)
			assert field is not None
			field.setText("carbonyl part")
			dialog.accept()

		PySide6.QtCore.QTimer.singleShot(0, name_fragment)
		_trigger_menu_action(window, "Create Fragment...")
		observation = tab.current_document_observation()
		owner = observation.projection.molecules[0].id
		assert any(
			record.name == "carbonyl part" and record.molecule_id == owner
			for record in ferrum_chem.inspect_document_explicit_fragments_v1(
				observation, observation.snapshot.revision, observation.snapshot.digest,
			).records
		)
		assert tab._controller.projection.selected_durable_targets() == selected_before_create

		def confirm_view() -> None:
			dialog = _active_dialog()
			rows = dialog.findChild(PySide6.QtWidgets.QTreeWidget)
			assert rows is not None
			assert any(rows.topLevelItem(index).text(0) == "carbonyl part"
				for index in range(rows.topLevelItemCount()))
			dialog.reject()

		PySide6.QtCore.QTimer.singleShot(0, confirm_view)
		_trigger_menu_action(window, "View Fragments...")
	finally:
		window.close()
		window.deleteLater()


def test_visible_fragment_cancel_preserves_document_and_durable_selection(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Cancelling the name dialog leaves the captured Ferrum drawing untouched."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(CDML, "cancel.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		tab._controller.projection.select_durable((("bond", "ab"),))
		window._refresh_actions()
		before_snapshot = tab.current_snapshot
		before_selection = tab._controller.projection.selected_durable_targets()
		PySide6.QtCore.QTimer.singleShot(0, lambda: _active_dialog().reject())
		_trigger_menu_action(window, "Create Fragment...")
		assert tab.current_snapshot == before_snapshot
		assert tab._controller.projection.selected_durable_targets() == before_selection
	finally:
		window.close()
		window.deleteLater()


def test_visible_fragment_view_retires_when_its_source_tab_changes(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A read-only View cannot outlive the active Ferrum tab that supplied its facts."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(CDML, "view.cdml")
	other = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(CDML, "other.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window._register_native_tab(other, activate=False)
		window.show()
		before = tab.current_snapshot
		PySide6.QtCore.QTimer.singleShot(
			0, lambda: window._tab_widget.setCurrentWidget(other),
		)
		_trigger_menu_action(window, "View Fragments...")
		assert tab.current_snapshot == before
	finally:
		window.close()
		window.deleteLater()
