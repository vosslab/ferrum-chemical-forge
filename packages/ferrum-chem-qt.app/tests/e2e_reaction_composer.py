"""Installed-wheel Ferrum workflow: terminal focus loss, hide, then create a reaction."""

# Standard Library
import argparse
import json
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
<arrow id='arrow'><point x='40' y='0'/><point x='120' y='0'/></arrow>
</cdml>"""


#============================================
def _parse_args() -> argparse.Namespace:
	"""Require the isolated installed site that supplies both public packages."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"--site-packages", required=True, type=pathlib.Path,
		help="isolated site-packages directory containing both built Ferrum wheels",
	)
	args = parser.parse_args()
	args.site_packages = args.site_packages.resolve()
	if not args.site_packages.is_dir():
		raise RuntimeError(f"installed site-packages directory does not exist: {args.site_packages}")
	return args


#============================================
def _assert_installed_imports(site_packages: pathlib.Path) -> dict[str, str]:
	"""Prove the E2E imports both public packages from the requested wheel site."""
	imported_paths = {
		"ferrum_chem": pathlib.Path(ferrum_chem.__file__).resolve(),
		"ferrum_qt": pathlib.Path(ferrum_qt.main_window.__file__).resolve(),
	}
	for package_name, module_path in imported_paths.items():
		if site_packages not in module_path.parents:
			raise RuntimeError(
				f"{package_name} imported from {module_path}, not isolated wheel site {site_packages}",
			)
	return {package_name: str(module_path) for package_name, module_path in imported_paths.items()}


#============================================
def _select_members(window: object, tab: object) -> None:
	"""Project exactly the three backend-owned complete roots into the public window."""
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
	"""Choose one durable role row through its ordinary public modeless panel."""
	list_widget = panel._lists[role]
	for index in range(list_widget.count()):
		item = list_widget.item(index)
		if item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == identifier:
			item.setCheckState(PySide6.QtCore.Qt.CheckState.Checked)
			return
	raise RuntimeError(f"missing reaction role row {identifier!r}")


#============================================
def _open_composer(window: object, app: PySide6.QtWidgets.QApplication) -> object:
	"""Open the modeless composer only by its ordinary public action."""
	window._create_reaction_action.trigger()
	app.processEvents()
	panel = window._reaction_composer._panel
	if panel is None:
		raise RuntimeError("Create Reaction did not open the public modeless composer")
	return panel


#============================================
def _assert_terminal_retirement(window: object, tab: object, before: str) -> None:
	"""Check a terminal UI lifecycle path leaves CDML unchanged and authoring disposable."""
	if tab.current_snapshot.cdml != before:
		raise RuntimeError("reaction composer terminal lifecycle mutated authoritative CDML")
	composer = window._reaction_composer
	if composer._dock is not None or composer._panel is not None:
		raise RuntimeError("reaction composer terminal lifecycle retained its visible form")
	if composer._choices is not None or composer._revision is not None or composer._digest is not None:
		raise RuntimeError("reaction composer terminal lifecycle retained a prepared authoring state")
	if window._render_interaction_selection is not None:
		raise RuntimeError("reaction composer terminal lifecycle retained transient root selection")
	if not window._create_reaction_action.isEnabled():
		raise RuntimeError("Create Reaction did not become available after terminal cancellation")


#============================================
def _focus_external_window(
		app: PySide6.QtWidgets.QApplication,
		peer_window: PySide6.QtWidgets.QWidget,
		peer_input: PySide6.QtWidgets.QLineEdit,
		) -> None:
	"""Transfer real Qt focus to a separately shown external top-level window."""
	peer_window.show()
	peer_window.raise_()
	peer_window.activateWindow()
	peer_input.setFocus(PySide6.QtCore.Qt.FocusReason.ActiveWindowFocusReason)
	app.processEvents()
	if app.focusWidget() is not peer_input:
		raise RuntimeError("Qt did not transfer application focus to the shown external window")


#============================================
def _reactivate_ferrum(window: object, app: PySide6.QtWidgets.QApplication) -> None:
	"""Restore the public Ferrum window before proving its next normal workflow."""
	window.show()
	window.raise_()
	window.activateWindow()
	app.processEvents()


#============================================
def main() -> int:
	"""Run public focus-loss, hide, commit, and Rust reopen behavior from built wheels."""
	args = _parse_args()
	import_paths = _assert_installed_imports(args.site_packages)
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "reaction-e2e.cdml")
	peer_window = PySide6.QtWidgets.QWidget()
	peer_window.setWindowTitle("Ferrum reaction composer external focus probe")
	peer_layout = PySide6.QtWidgets.QVBoxLayout(peer_window)
	peer_input = PySide6.QtWidgets.QLineEdit(peer_window)
	peer_input.setAccessibleName("External focus probe")
	peer_layout.addWidget(peer_input)
	try:
		window._register_native_tab(tab, activate=True)
		_reactivate_ferrum(window, app)
		_select_members(window, tab)
		before = tab.current_snapshot.cdml
		_open_composer(window, app)
		_focus_external_window(app, peer_window, peer_input)
		_assert_terminal_retirement(window, tab, before)
		_reactivate_ferrum(window, app)
		_select_members(window, tab)
		_open_composer(window, app)
		window.hide()
		app.processEvents()
		_assert_terminal_retirement(window, tab, before)
		_reactivate_ferrum(window, app)
		_select_members(window, tab)
		panel = _open_composer(window, app)
		_check(panel, "reactants", "left")
		_check(panel, "products", "right")
		_check(panel, "arrow", "arrow")
		panel.submitted.emit()
		app.processEvents()
		if "<reaction id=\"rxn-1\"" not in tab.current_snapshot.cdml:
			raise RuntimeError("public reaction composer did not create one native reaction")
		with tempfile.TemporaryDirectory(prefix="ferrum-reaction-e2e-") as directory:
			path = pathlib.Path(directory) / "reaction.cdml"
			path.write_text(tab.current_snapshot.cdml, encoding="utf-8")
			reopened = ferrum_chem.DocumentSession.load(path.read_text(encoding="utf-8"))
			if "<reaction id=\"rxn-1\"" not in reopened.snapshot().cdml:
				raise RuntimeError("Rust reopen did not preserve the authored reaction")
		print(json.dumps({
			"schema": "ferrum-reaction-composer-e2e-v2",
			"status": "ok",
			"imports": import_paths,
		}))
		return 0
	finally:
		peer_window.close()
		peer_window.deleteLater()
		window.close()
		window.deleteLater()


if __name__ == "__main__":
	sys.exit(main())
