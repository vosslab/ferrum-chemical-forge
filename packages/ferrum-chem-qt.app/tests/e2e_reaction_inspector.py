"""Installed-wheel public Qt workflow for the Rust-owned Reaction Inspector."""

# Standard Library
import argparse
import json
import os
import pathlib
import sys
import tempfile

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_chem
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine
import ferrum_qt.main_window


_CDML = """<cdml version='26.08'>
<molecule id='left'><atom id='left-a' name='C'><point x='0' y='0'/></atom></molecule>
<molecule id='right'><atom id='right-a' name='O'><point x='160' y='0'/></atom></molecule>
<molecule id='spare'><atom id='spare-a' name='N'><point x='240' y='0'/></atom></molecule>
<arrow id='arrow'><point x='40' y='0'/><point x='120' y='0'/></arrow>
</cdml>"""


#============================================
def _trace(event: str) -> None:
	"""Emit opt-in lifecycle markers for timeout-bound installed-wheel diagnosis."""
	if os.environ.get("FERRUM_E2E_TRACE"):
		print(f"reaction-inspector-e2e: {event}", flush=True)


#============================================
def _parse_args() -> argparse.Namespace:
	"""Require the isolated installed site that supplies both built Ferrum wheels."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--site-packages", required=True, type=pathlib.Path)
	args = parser.parse_args()
	args.site_packages = args.site_packages.resolve()
	if not args.site_packages.is_dir():
		raise RuntimeError(f"installed site-packages directory does not exist: {args.site_packages}")
	return args


#============================================
def _assert_installed_imports(site_packages: pathlib.Path) -> dict[str, str]:
	"""Prove both public packages came from the freshly installed wheel site."""
	paths = {
		"ferrum_chem": pathlib.Path(ferrum_chem.__file__).resolve(),
		"ferrum_qt": pathlib.Path(ferrum_qt.main_window.__file__).resolve(),
	}
	for package_name, module_path in paths.items():
		if site_packages not in module_path.parents:
			raise RuntimeError(
				f"{package_name} imported from {module_path}, not isolated wheel site {site_packages}",
			)
	return {package_name: str(module_path) for package_name, module_path in paths.items()}


#============================================
def _action(window: PySide6.QtWidgets.QWidget, text: str) -> PySide6.QtGui.QAction:
	"""Find one visible product action by its user-facing text."""
	for action in window.findChildren(PySide6.QtGui.QAction):
		if action.text() == text:
			return action
	raise RuntimeError(f"Ferrum did not expose action {text!r}")


#============================================
def _button(widget: PySide6.QtWidgets.QWidget, text: str) -> PySide6.QtWidgets.QPushButton:
	"""Find one visible user-facing button without depending on layout position."""
	for button in widget.findChildren(PySide6.QtWidgets.QPushButton):
		if button.text() == text:
			return button
	raise RuntimeError(f"Ferrum did not expose button {text!r}")


#============================================
def _select_members(window: object, tab: object) -> None:
	"""Select complete backend roots before opening the public composer action."""
	observation = tab.observe_direct_root_interaction()
	selection = None
	for identifier in ("left", "right", "arrow"):
		modifier = (
			ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
			if selection is None else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle
		)
		selection = tab.select_direct_roots(
			observation, selection,
			ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(identifier, modifier),
		)
	window._replace_render_interaction_selection(selection, tab)


#============================================
def _check(panel: object, role: str, identifier: str) -> None:
	"""Choose one ordinary modeless composer row by durable displayed identity."""
	list_widget = panel._lists[role]
	for index in range(list_widget.count()):
		item = list_widget.item(index)
		if item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == identifier:
			item.setCheckState(PySide6.QtCore.Qt.CheckState.Checked)
			return
	raise RuntimeError(f"missing reaction composer role row {identifier!r}")


#============================================
def _create_reaction(window: object, tab: object, app: PySide6.QtWidgets.QApplication) -> None:
	"""Create one reaction through the live Create Reaction QAction and form."""
	_select_members(window, tab)
	_action(window, "Create Reaction...").trigger()
	app.processEvents()
	panel = window._reaction_composer._panel
	if panel is None:
		raise RuntimeError("Create Reaction did not open its public modeless form")
	_check(panel, "reactants", "left")
	_check(panel, "products", "right")
	_check(panel, "arrow", "arrow")
	panel.submitted.emit()
	app.processEvents()
	if '<reaction id="rxn-1"' not in tab.current_snapshot.cdml:
		raise RuntimeError("public Create Reaction did not commit one Rust reaction")


#============================================
def _schedule_editor(app: PySide6.QtWidgets.QApplication, accept: bool) -> None:
	"""Drive the real modal role editor after its nested Qt event loop starts."""
	def act() -> None:
		editor = app.activeModalWidget()
		if editor is None or editor.windowTitle() != "Edit Reaction":
			raise RuntimeError("Reaction Inspector did not present its modal role editor")
		if accept:
			reactants = editor.findChild(PySide6.QtWidgets.QListWidget, "reaction-inspector-reactants")
			if reactants is None:
				raise RuntimeError("Reaction Inspector role editor omitted reactants")
			for index in range(reactants.count()):
				item = reactants.item(index)
				identifier = item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)
				if identifier == "left":
					item.setCheckState(PySide6.QtCore.Qt.CheckState.Unchecked)
				if identifier == "spare":
					item.setCheckState(PySide6.QtCore.Qt.CheckState.Checked)
			button_box = editor.findChild(PySide6.QtWidgets.QDialogButtonBox)
			if button_box is None:
				raise RuntimeError("Reaction Inspector role editor omitted its decision buttons")
			button_box.button(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok).click()
		else:
			_button(editor, "Cancel").click()
	PySide6.QtCore.QTimer.singleShot(0, act)


#============================================
def _schedule_delete(app: PySide6.QtWidgets.QApplication, controller: object,
		confirm: bool) -> dict[str, str | None]:
	"""Choose the explicit delete or cancel control in Ferrum's real confirmation."""
	result = {"error": None}
	def act() -> None:
		dialog = controller._owned_dialog
		_trace(f"delete-driver dialog={getattr(dialog, 'objectName', lambda: None)()!r}")
		if dialog is None or dialog.objectName() != "reaction-inspector-delete-dialog":
			result["error"] = "Reaction Inspector did not present its Ferrum delete confirmation"
			if isinstance(dialog, PySide6.QtWidgets.QDialog):
				dialog.reject()
			else:
				app.quit()
			return
		button_name = (
			"reaction-inspector-delete-confirm" if confirm
			else "reaction-inspector-delete-cancel"
		)
		button = dialog.findChild(PySide6.QtWidgets.QPushButton, button_name)
		if button is None:
			result["error"] = "Reaction Inspector delete confirmation omitted its explicit control"
			dialog.reject()
			return
		button.click()
		_trace(f"delete-driver clicked={button_name}")
	PySide6.QtCore.QTimer.singleShot(0, act)
	return result


#============================================
def _schedule_save_failure_capture(
		app: PySide6.QtWidgets.QApplication,
		) -> tuple[PySide6.QtCore.QTimer, dict[str, str | None]]:
	"""Expose a save refusal from its real modal instead of leaving an E2E loop blocked."""
	result = {"error": None}
	timer = PySide6.QtCore.QTimer()
	timer.setSingleShot(True)
	def capture() -> None:
		dialog = app.activeModalWidget()
		if not isinstance(dialog, PySide6.QtWidgets.QMessageBox):
			return
		result["error"] = f"native Save As failure: {dialog.text()}"
		dialog.reject()
	timer.timeout.connect(capture)
	timer.start(0)
	return timer, result


#============================================
def _open_and_wait(window: object, path: pathlib.Path) -> object:
	"""Await the ordinary asynchronous native CDML open boundary."""
	_trace("before-async-open")
	completed = []
	loop = PySide6.QtCore.QEventLoop()
	timeout = PySide6.QtCore.QTimer()
	timeout.setSingleShot(True)
	def receive(file_path: str, success: bool) -> None:
		if pathlib.Path(file_path) == path:
			completed.append(success)
			_trace(f"async-open-completed={success}")
			loop.quit()
	window.local_document_open_completed.connect(receive)
	timeout.timeout.connect(loop.quit)
	try:
		if not window.open_file_path(str(path)):
			raise RuntimeError("Ferrum native Open did not accept the saved reaction document")
		timeout.start(10_000)
		loop.exec()
	finally:
		timeout.stop()
		window.local_document_open_completed.disconnect(receive)
	if completed != [True]:
		raise RuntimeError("Ferrum native Open did not complete the saved reaction document")
	return window._active_native_tab()


#============================================
def _require_member_order(cdml: str, reactant: str) -> None:
	"""Require the visible complete role sequence Rust retained after an edit or undo."""
	role_tokens = (
		f'<reactant idref="{reactant}"', '<product idref="right"', '<arrow idref="arrow"',
	)
	positions = [cdml.find(token) for token in role_tokens]
	if -1 in positions or positions != sorted(positions):
		raise RuntimeError("Reaction Inspector did not retain the authoritative member role order")


#============================================
def _save_dirty_tabs_for_teardown(window: object) -> None:
	"""Keep the product's real unsaved-close policy out of an unattended E2E teardown."""
	with tempfile.TemporaryDirectory(
		prefix="ferrum-reaction-inspector-teardown-", dir="/private/tmp",
		) as directory:
		for index, tab in enumerate(tuple(window._native_tabs_by_page.values())):
			if tab.is_dirty:
				tab.save_atomic(pathlib.Path(directory) / f"cleanup-{index}.cdml")


#============================================
def main() -> int:
	"""Exercise public inspector cancel, patch, move, delete, undo, save, and reopen routes."""
	args = _parse_args()
	imports = _assert_installed_imports(args.site_packages)
	PySide6.QtCore.QCoreApplication.setAttribute(
		PySide6.QtCore.Qt.ApplicationAttribute.AA_DontUseNativeDialogs,
	)
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "reaction-inspector-e2e.cdml")
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		app.processEvents()
		_create_reaction(window, tab, app)
		inspector_action = _action(window, "Reaction Inspector")
		ribbon_button = None
		for candidate in window._authoring_ribbon.findChildren(PySide6.QtWidgets.QToolButton):
			if candidate.defaultAction() is inspector_action:
				ribbon_button = candidate
				break
		if ribbon_button is None or not ribbon_button.isVisible():
			raise RuntimeError("Reaction Inspector QAction lacks a visible shared ribbon client")
		ribbon_button.click()
		app.processEvents()
		controller = window._reaction_inspector
		if controller._dock is None or not controller._dock.isVisible():
			raise RuntimeError("Reaction Inspector QAction did not show the modeless public dock")
		_trace("inspector-open")
		before_cancel = tab.current_snapshot.cdml
		_schedule_editor(app, accept=False)
		_button(controller._dock, "Edit Roles...").click()
		app.processEvents()
		if tab.current_snapshot.cdml != before_cancel:
			raise RuntimeError("cancelling Reaction Inspector role edit mutated authoritative CDML")
		inspector_action.trigger()
		app.processEvents()
		if controller._dock is None or not controller._dock.isVisible():
			raise RuntimeError("Reaction Inspector did not reopen after terminal role-edit cancellation")
		_schedule_editor(app, accept=True)
		_button(controller._dock, "Edit Roles...").click()
		app.processEvents()
		patched = tab.current_snapshot.cdml
		if '<reactant idref="spare"' not in patched or '<reactant idref="left"' in patched:
			raise RuntimeError("accepted Reaction Inspector edit did not replace complete role membership")
		_require_member_order(patched, "spare")
		_button(controller._dock, "Nudge Right").click()
		app.processEvents()
		if tab.current_snapshot.cdml == patched:
			raise RuntimeError("Reaction Inspector nudge did not commit an aggregate movement")
		_action(window, "Undo").trigger()
		app.processEvents()
		if tab.current_snapshot.cdml != patched:
			raise RuntimeError("public Undo did not restore the patched reaction aggregate")
		if controller._tab is not tab or controller._reaction().reaction_id != "rxn-1":
			raise RuntimeError("Reaction Inspector lost its authoritative selected reaction before deletion")
		_trace("before-delete-cancel")
		delete_result = _schedule_delete(app, controller, confirm=False)
		_button(controller._dock, "Delete Definition...").click()
		app.processEvents()
		if delete_result["error"] is not None:
			raise RuntimeError(delete_result["error"])
		if tab.current_snapshot.cdml != patched:
			raise RuntimeError("cancelling definition deletion mutated authoritative CDML")
		_trace("after-delete-cancel")
		inspector_action.trigger()
		app.processEvents()
		if controller._dock is None or not controller._dock.isVisible():
			raise RuntimeError("Reaction Inspector did not reopen after terminal deletion cancellation")
		_trace("before-delete-confirm")
		delete_result = _schedule_delete(app, controller, confirm=True)
		_button(controller._dock, "Delete Definition...").click()
		app.processEvents()
		if delete_result["error"] is not None:
			raise RuntimeError(delete_result["error"])
		_trace("after-delete-confirm")
		deleted = tab.current_snapshot.cdml
		if '<reaction id="rxn-1"' in deleted or not all(token in deleted for token in (
			'<molecule id="left"', '<molecule id="right"', '<molecule id="spare"', '<arrow id="arrow"',
		)):
			raise RuntimeError("definition-only deletion did not retain all durable member roots")
		_action(window, "Undo").trigger()
		app.processEvents()
		restored = tab.current_snapshot.cdml
		if '<reaction id="rxn-1"' not in restored or '<reactant idref="spare"' not in restored:
			raise RuntimeError("public Undo did not restore the reaction definition and its reference")
		_trace("after-delete-undo")
		_require_member_order(restored, "spare")
		with tempfile.TemporaryDirectory(
			prefix="ferrum-reaction-inspector-e2e-", dir="/private/tmp",
			) as directory:
			path = pathlib.Path(directory) / "reaction-inspector.cdml"
			_trace("before-save-as")
			save_timer, save_failure = _schedule_save_failure_capture(app)
			saved = window.save_active_to_path(str(path))
			save_timer.stop()
			if save_failure["error"] is not None:
				raise RuntimeError(save_failure["error"])
			if not saved:
				raise RuntimeError("public Save As route did not publish the reaction document")
			_trace("after-save-as")
			rust_reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if '<reaction id="rxn-1"' not in rust_reopened.snapshot().cdml:
				raise RuntimeError("Rust reopen did not preserve the restored reaction")
			_trace("after-rust-reopen")
			gui_reopened = _open_and_wait(window, path)
			if gui_reopened is None or '<reaction id="rxn-1"' not in gui_reopened.current_snapshot.cdml:
				raise RuntimeError("asynchronous native GUI reopen did not preserve the reaction")
		_trace("after-async-reopen")
		print(json.dumps({
			"schema": "ferrum-reaction-inspector-public-e2e-v1",
			"status": "ok",
			"imports": imports,
		}, sort_keys=True))
		return 0
	finally:
		_save_dirty_tabs_for_teardown(window)
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())
