"""Qt proof for Rust-authoritative reaction inspection and aggregate movement."""

# Standard Library
import pathlib
import types

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.reaction_inspector


_REACTION_CDML = (
	'<cdml><molecule id="left"><atom id="left-a" name="C"><point x="0" y="0"/>'
	'</atom></molecule><molecule id="right"><atom id="right-a" name="O"><point x="100" y="0"/>'
	'</atom></molecule><arrow id="arrow"><point x="25" y="0"/><point x="75" y="0"/>'
	'</arrow><reaction id="strict"><reactant idref="left"/><product idref="right"/>'
	'<arrow idref="arrow"/></reaction></cdml>'
)


#============================================
def _open_inspector(main_window: object) -> object:
	"""Register one strict Rust reaction and open the ordinary public command."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_REACTION_CDML, "reaction-inspector.cdml",
	)
	main_window._register_native_tab(tab, activate=True)
	main_window._reaction_inspector_action.trigger()
	return tab


#============================================
def test_inspector_highlights_exact_rust_issued_members(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The inspector projects every listed member through backend root selection."""
	tab = _open_inspector(main_window)
	controller = main_window._reaction_inspector
	qapp.processEvents()
	try:
		controller.highlight_members()
		selection = main_window._render_interaction_selection
		assert selection is not None
		assert {root.identifier for root in selection.roots} == {"left", "right", "arrow"}
	finally:
		controller.close()
		tab.dispose()


#============================================
def test_inspector_nudge_uses_one_aggregate_transaction_and_undo(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""One visible nudge commits through Rust and undo restores the exact source."""
	tab = _open_inspector(main_window)
	controller = main_window._reaction_inspector
	qapp.processEvents()
	try:
		controller.nudge(10.0, 0.0)
		changed = tab.current_snapshot
		undo = tab._session.undo(changed.revision)
		tab._install_mutation_result(undo)
		tab.save_atomic(tmp_path / "reaction-inspector.cdml")
		assert changed.revision == 1 and '<reaction id="strict"' in changed.cdml
		assert tab.current_snapshot.cdml == _REACTION_CDML
	finally:
		controller.close()
		tab.dispose()


#============================================
def test_role_editor_preserves_existing_role_ordinals_over_global_choice_order(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Opening and accepting unchanged roles does not rewrite durable member order."""
	reaction = types.SimpleNamespace(
		reaction_id="strict",
		members=(
			types.SimpleNamespace(identifier="right", role="product", role_ordinal=1),
			types.SimpleNamespace(identifier="left", role="reactant", role_ordinal=1),
			types.SimpleNamespace(identifier="arrow", role="arrow", role_ordinal=0),
			types.SimpleNamespace(identifier="first", role="reactant", role_ordinal=0),
		),
	)
	choices = types.SimpleNamespace(choices=(
		types.SimpleNamespace(identifier="left", kind="molecule", source_order=0,
			availability="eligible", label="Left"),
		types.SimpleNamespace(identifier="right", kind="molecule", source_order=1,
			availability="eligible", label="Right"),
		types.SimpleNamespace(identifier="first", kind="molecule", source_order=2,
			availability="eligible", label="First"),
		types.SimpleNamespace(identifier="arrow", kind="arrow", source_order=3,
			availability="eligible", label="Arrow"),
	))
	parent = PySide6.QtWidgets.QWidget()
	editor = ferrum_qt.ferrum.reaction_inspector._ReactionRoleEditor(reaction, choices, parent)
	try:
		assert editor.request() == (["first", "left"], ["right"], "arrow", [], [])
	finally:
		editor.deleteLater()
		parent.deleteLater()


#============================================
def test_inspector_escapes_hostile_reaction_detail_text(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Document-derived labels remain literal text rather than inspector HTML markup."""
	tab = _open_inspector(main_window)
	controller = main_window._reaction_inspector
	qapp.processEvents()
	try:
		hostile = '<img src="missing"/> & <b>not markup</b>'
		reaction = types.SimpleNamespace(
			reaction_id=hostile, disposition="compatible", union_bounds=None,
			members=(types.SimpleNamespace(
				identifier=hostile, role=hostile, role_ordinal=0,
			),), diagnostics=(),
		)
		controller._observation = types.SimpleNamespace(reactions=(reaction,))
		item = PySide6.QtWidgets.QListWidgetItem("hostile")
		item.setData(PySide6.QtCore.Qt.ItemDataRole.UserRole, hostile)
		controller._on_current_changed(item, None)
		assert controller._detail.toPlainText().count(hostile) == 3
		assert "<img" not in controller._detail.toHtml()
	finally:
		controller.close()
		tab.dispose()


#============================================
def test_inspector_recovers_rust_accepted_nudge_after_projection_failure(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: object,
		tmp_path: pathlib.Path,
		) -> None:
	"""A failed Qt install refreshes an accepted Rust mutation instead of calling it refused."""
	tab = _open_inspector(main_window)
	controller = main_window._reaction_inspector
	qapp.processEvents()
	try:
		replace = tab._controller.replace
		attempts = []
		def fail_once(observation: object, latch: object) -> bool:
			attempts.append(observation)
			if len(attempts) == 1:
				return False
			return replace(observation, latch)
		monkeypatch.setattr(tab._controller, "replace", fail_once)
		controller.nudge(10.0, 0.0)
		assert tab.current_snapshot.revision == 1 and not tab.requires_refresh
		assert len(attempts) == 2
		assert "display was refreshed after installation recovery" in main_window.statusBar().currentMessage()
		tab.save_atomic(tmp_path / "reaction-inspector-recovered-nudge.cdml")
	finally:
		controller.close()
		tab.dispose()


#============================================
def test_owned_modal_deactivation_keeps_inspector_open_until_rejected(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A foreground owned modal keeps inspection live and can still cancel normally."""
	tab = _open_inspector(main_window)
	controller = main_window._reaction_inspector
	qapp.processEvents()
	try:
		dialog = PySide6.QtWidgets.QDialog(main_window)
		dialog.setModal(True)
		controller._owned_dialog = dialog
		dialog.show()
		dialog.activateWindow()
		qapp.processEvents()
		qapp.sendEvent(main_window, PySide6.QtCore.QEvent(
			PySide6.QtCore.QEvent.Type.WindowDeactivate,
		))
		assert qapp.activeModalWidget() is dialog
		assert dialog.isActiveWindow()
		assert controller._dock is not None and controller._tab is tab
		dialog.reject()
		assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Rejected
		controller._owned_dialog = None
	finally:
		controller.close()
		tab.dispose()


#============================================
def test_external_peer_deactivation_retires_inspector_despite_visible_owned_modal(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A peer window taking activation retires inspection without mutating the CDML document."""
	tab = _open_inspector(main_window)
	controller = main_window._reaction_inspector
	qapp.processEvents()
	try:
		dialog = PySide6.QtWidgets.QDialog(main_window)
		dialog.setModal(True)
		controller._owned_dialog = dialog
		dialog.show()
		dialog.activateWindow()
		qapp.processEvents()
		peer = PySide6.QtWidgets.QWidget()
		peer.setWindowTitle("External peer")
		peer.show()
		peer.activateWindow()
		qapp.processEvents()
		qapp.sendEvent(main_window, PySide6.QtCore.QEvent(
			PySide6.QtCore.QEvent.Type.WindowDeactivate,
		))
		assert qapp.activeModalWidget() is dialog
		assert not dialog.isActiveWindow()
		assert controller._dock is None and controller._tab is None
		assert tab.current_snapshot.cdml == _REACTION_CDML
	finally:
		if controller._owned_dialog is not None:
			controller._owned_dialog.reject()
			controller._owned_dialog = None
		controller.close()
		tab.dispose()
		peer.close()


#============================================
def test_definition_delete_dialog_states_the_safe_boundary_and_uses_explicit_controls(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The Ferrum-owned confirmation exposes no mutation route until explicit acceptance."""
	parent = PySide6.QtWidgets.QWidget()
	dialog = ferrum_qt.ferrum.reaction_inspector._ReactionDefinitionDeleteDialog(
		"strict", parent,
	)
	try:
		assert dialog.objectName() == "reaction-inspector-delete-dialog"
		assert dialog.accessibleName() == "Delete Reaction Definition"
		consequence = dialog.findChild(
			PySide6.QtWidgets.QLabel, "reaction-inspector-delete-consequence",
		)
		assert consequence is not None and "member roots remain" in consequence.text()
		delete = dialog.findChild(
			PySide6.QtWidgets.QPushButton, "reaction-inspector-delete-confirm",
		)
		cancel = dialog.findChild(
			PySide6.QtWidgets.QPushButton, "reaction-inspector-delete-cancel",
		)
		assert delete is not None and cancel is not None
		assert delete.text() == "Delete reaction definition"
		assert cancel.isDefault()
		cancel.click()
		assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Rejected
	finally:
		dialog.deleteLater()
		parent.deleteLater()
