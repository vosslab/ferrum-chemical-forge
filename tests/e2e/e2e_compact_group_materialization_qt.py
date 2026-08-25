"""Materialize one selected compact group through the offscreen Ferrum UI."""

# Standard Library
import json
import pathlib
import sys

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import e2e_workspace
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
_COMPACT_DOCUMENT = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="source-molecule">'
	'<atom id="anchor" name="C"><point x="0" y="0"/></atom>'
	'<compact-group id="source-group" version="1" catalog-key="methyl" '
	'attachment-index="0" orientation-degrees="0"><point x="20" y="0"/></compact-group>'
	'<bond id="outside" start="anchor" end="source-group" type="n1"/>'
	'</molecule></cdml>'
)


#============================================
class CompactGroupMaterializationQtE2eError(RuntimeError):
	"""Report one failed public compact-group materialization workflow."""


#============================================
def _action(window: PySide6.QtWidgets.QMainWindow, text: str) -> PySide6.QtGui.QAction:
	"""Return one public action by its user-facing label."""
	try:
		return next(
			action for action in window.findChildren(PySide6.QtGui.QAction)
			if action.text() == text
		)
	except StopIteration as error:
		raise CompactGroupMaterializationQtE2eError(
			f"Ferrum did not expose the {text!r} action",
		) from error


#============================================
def _active_tab(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Return the document selected through Ferrum's public tab control."""
	tabs = window.centralWidget()
	if not isinstance(tabs, PySide6.QtWidgets.QTabWidget):
		raise CompactGroupMaterializationQtE2eError("Ferrum did not expose document tabs")
	tab = tabs.currentWidget()
	if tab is None:
		raise CompactGroupMaterializationQtE2eError("Ferrum did not select the opened document")
	return tab


#============================================
def _compact_group_item(tab: object) -> object | None:
	"""Return the one visible typed compact-group scene item, if present."""
	projection = tab._controller.projection
	if projection is None:
		return None
	return next((
		item for item, target in projection.item_targets.items()
		if target.kind == "compact_group"
	), None)


#============================================
def _select_compact_group(tab: object,
		application: PySide6.QtWidgets.QApplication) -> None:
	"""Select the rendered compact label into canonical durable Qt state."""
	item = _compact_group_item(tab)
	if item is None:
		raise CompactGroupMaterializationQtE2eError(
			"Ferrum did not render a selectable compact-group label",
		)
	projection = tab._controller.projection
	if projection is None:
		raise CompactGroupMaterializationQtE2eError(
			"Ferrum did not retain the compact-group render projection",
		)
	target = projection.item_targets[item]
	point = tab.view.mapFromScene(item.sceneBoundingRect().center())
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point,
	)
	application.processEvents()
	if projection.selected_durable_targets() != (target,):
		raise CompactGroupMaterializationQtE2eError(
			"clicking the compact-group label did not enter canonical durable selection",
		)
	address = tab.selected_molecule_compact_group_address()
	if (
		address.compact_group_id != target.identifier
		or address.molecule_id != target.molecule_identifier
	):
		raise CompactGroupMaterializationQtE2eError(
			"canonical compact-group selection did not retain Rust document identifiers",
		)


#============================================
def _selected_atom_is_visible(tab: object) -> bool:
	"""Return whether Rust's post-commit focus is visible as an atom selection."""
	projection = tab._controller.projection
	return projection is not None and any(
		target.kind == "atom" for target in projection.selected_durable_targets()
	)


#============================================
def main() -> int:
	"""Run the visible compact-group materialization workflow on a staged runtime."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	original_open = PySide6.QtWidgets.QFileDialog.getOpenFileName
	try:
		with e2e_workspace.E2EWorkspaceLease() as workspace_text:
			path = pathlib.Path(workspace_text) / "materializable-compact-group.cdml"
			path.write_text(_COMPACT_DOCUMENT, encoding="utf-8")
			opened: list[bool] = []
			completion = PySide6.QtCore.QEventLoop()
			window.local_document_open_completed.connect(
				lambda _path, success: (opened.append(success), completion.quit()),
			)
			PySide6.QtWidgets.QFileDialog.getOpenFileName = staticmethod(
				lambda *_args, **_kwargs: (str(path), "CDML (*.cdml)"),
			)
			window.show()
			app.processEvents()
			_action(window, "Open").trigger()
			completion.exec()
			if opened != [True]:
				raise CompactGroupMaterializationQtE2eError(
					"Ferrum did not complete the public compact document open",
				)
			tab = _active_tab(window)
			_select_compact_group(tab, app)
			action = _action(window, "Materialize Selected Compact Group")
			if not action.isEnabled():
				raise CompactGroupMaterializationQtE2eError(
					"Ferrum did not enable compact-group materialization for its selection",
				)
			completed: list[object] = []
			completion = PySide6.QtCore.QEventLoop()
			window.operation_presentation_completed.connect(
				lambda receipt: (completed.append(receipt), completion.quit()),
			)
			action.trigger()
			if not completed:
				completion.exec()
			if len(completed) != 1 or completed[0].terminal_kind != "succeeded":
				raise CompactGroupMaterializationQtE2eError(
					"Ferrum did not publish one successful compact-group terminal receipt",
				)
			if _compact_group_item(tab) is not None or not _selected_atom_is_visible(tab):
				raise CompactGroupMaterializationQtE2eError(
					"materialization did not replace the group and select Rust's focus atom",
				)
			print(json.dumps({
				"schema": "ferrum-compact-group-materialization-qt-e2e-v1",
				"status": "ok",
			}, sort_keys=True))
			return 0
	finally:
		PySide6.QtWidgets.QFileDialog.getOpenFileName = original_open
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except CompactGroupMaterializationQtE2eError as exc:
		print(f"e2e_compact_group_materialization_qt: {exc}", file=sys.stderr)
		raise SystemExit(1)
