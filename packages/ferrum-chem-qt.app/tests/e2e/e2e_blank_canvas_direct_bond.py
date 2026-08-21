"""Prove installed Ferrum Draw Bond creates one C-C bond from a blank canvas."""

# Standard Library
import argparse
import hashlib
import importlib
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import defusedxml.ElementTree
import zipfile


#============================================
class BlankCanvasDirectBondE2eError(RuntimeError):
	"""Raised when the installed blank-canvas Draw Bond contract is lost."""


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one bytecode-free subprocess and return its standard output."""
	result = subprocess.run(
		command, env=environment, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise BlankCanvasDirectBondE2eError(
			"command failed (%d): %s\n%s" % (
				result.returncode, " ".join(command), result.stderr.strip(),
			),
		)
	return result.stdout


#============================================
def _proof_environment() -> dict[str, str]:
	"""Return an isolated offscreen environment that cannot write bytecode."""
	environment = os.environ.copy()
	for variable in (
		"DYLD_LIBRARY_PATH", "DYLD_FALLBACK_LIBRARY_PATH", "DYLD_FRAMEWORK_PATH",
		"DYLD_FALLBACK_FRAMEWORK_PATH", "PYTHONHOME", "PYTHONPATH",
	):
		environment.pop(variable, None)
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	environment["QT_QPA_PLATFORM"] = "offscreen"
	return environment


#============================================
def _wheel_member_digest(wheel: pathlib.Path, member: str) -> str:
	"""Return one exact package-member digest from a supplied wheel."""
	with zipfile.ZipFile(wheel) as archive:
		if member not in archive.namelist():
			raise BlankCanvasDirectBondE2eError(
				f"supplied wheel lacks required package member {member!r}",
			)
		return hashlib.sha256(archive.read(member)).hexdigest()


#============================================
def _extension_member_digest(wheel: pathlib.Path) -> str:
	"""Return the supplied wheel's sole root Ferrum chemistry extension digest."""
	with zipfile.ZipFile(wheel) as archive:
		members = [
			member for member in archive.namelist()
			if member.startswith("ferrum_chem") and member.endswith(".so") and "/" not in member
		]
		if len(members) != 1:
			raise BlankCanvasDirectBondE2eError(
				"native wheel must contain one root ferrum_chem extension, found %r" % members,
			)
		return _wheel_member_digest(wheel, members[0])


#============================================
def _action(host: object, name: str) -> object:
	"""Return one visible product QAction identified by its accessible label."""
	from PySide6 import QtGui
	actions = [
		action for action in host.findChildren(QtGui.QAction)
		if action.text().replace("&", "").replace("...", "").strip() == name
	]
	if len(actions) != 1:
		raise BlankCanvasDirectBondE2eError(
			"expected one public QAction %r, found %d" % (name, len(actions)),
		)
	return actions[0]


#============================================
def _viewport(host: object) -> object:
	"""Return the sole visible product graphics viewport without controller access."""
	from PySide6 import QtWidgets
	views = [
		view for view in host.findChildren(QtWidgets.QGraphicsView)
		if view.isVisible() and view.viewport().isVisible()
	]
	if len(views) != 1:
		raise BlankCanvasDirectBondE2eError(
			"expected one visible document view, found %d" % len(views),
		)
	return views[0].viewport()


#============================================
def _image_digest(viewport: object) -> str:
	"""Return a stable viewport-only screenshot digest for the public canvas."""
	from PySide6 import QtCore
	buffer = QtCore.QBuffer()
	buffer.open(QtCore.QIODevice.OpenModeFlag.WriteOnly)
	if not viewport.grab().save(buffer, "PNG"):
		raise BlankCanvasDirectBondE2eError("viewport capture failed")
	return hashlib.sha256(bytes(buffer.data())).hexdigest()


#============================================
def _require_blank_projection(viewport: object, point: object) -> None:
	"""Require a viewport point to have no public graphics projection item."""
	from PySide6 import QtWidgets
	view = viewport.parentWidget()
	if not isinstance(view, QtWidgets.QGraphicsView):
		raise BlankCanvasDirectBondE2eError("viewport did not retain its public graphics view")
	if view.itemAt(point) is not None:
		raise BlankCanvasDirectBondE2eError("Draw Bond endpoint is not blank in the public projection")


#============================================
def _save_as(host: object, path: pathlib.Path) -> None:
	"""Use Save As QAction and its real Qt dialog to publish one local CDML file."""
	from PySide6 import QtCore, QtWidgets
	completed: list[bool] = []

	def accept_dialog() -> None:
		"""Choose the supplied local CDML target in the product file dialog."""
		dialogs = [
			widget for widget in QtWidgets.QApplication.topLevelWidgets()
			if isinstance(widget, QtWidgets.QFileDialog) and widget.isVisible()
		]
		if len(dialogs) != 1:
			return
		dialogs[0].selectFile(str(path))
		dialogs[0].accept()
		completed.append(True)

	QtCore.QTimer.singleShot(0, accept_dialog)
	_action(host, "Save As").trigger()
	QtWidgets.QApplication.processEvents()
	if completed != [True] or not path.is_file():
		raise BlankCanvasDirectBondE2eError("Save As QAction did not publish local CDML")


#============================================
def _saved_carbon_bond_facts(path: pathlib.Path) -> dict[str, int]:
	"""Read only the public saved CDML and count its C-C bond graph facts."""
	root = defusedxml.ElementTree.parse(path).getroot()
	atoms = root.findall(".//atom")
	carbon_atoms = [atom for atom in atoms if atom.get("name") == "C"]
	bonds = root.findall(".//bond")
	if len(carbon_atoms) != 2 or len(atoms) != 2 or len(bonds) != 1:
		raise BlankCanvasDirectBondE2eError(
			"saved blank-canvas graph is not one C-C bond: atoms=%d carbons=%d bonds=%d" % (
				len(atoms), len(carbon_atoms), len(bonds),
			),
		)
	return {"atoms": len(atoms), "bonds": len(bonds), "carbons": len(carbon_atoms)}


#============================================
def _saved_blank_facts(path: pathlib.Path, document_name: str) -> None:
	"""Require one named public New document to save with no authored graph content."""
	root = defusedxml.ElementTree.parse(path).getroot()
	atoms = root.findall(".//atom")
	bonds = root.findall(".//bond")
	if atoms or bonds:
		raise BlankCanvasDirectBondE2eError(
			"%s saved New document is not blank: path=%s atoms=%d bonds=%d" % (
				document_name, path, len(atoms), len(bonds),
			),
		)


#============================================
def _drag(viewport: object, start: object, end: object, *, release: bool) -> None:
	"""Drive one direct-bond drag entirely through native viewport events."""
	from PySide6 import QtCore, QtTest, QtWidgets
	QtTest.QTest.mousePress(viewport, QtCore.Qt.MouseButton.LeftButton,
		QtCore.Qt.KeyboardModifier.NoModifier, start)
	QtTest.QTest.mouseMove(viewport, end)
	QtWidgets.QApplication.processEvents()
	if release:
		QtTest.QTest.mouseRelease(viewport, QtCore.Qt.MouseButton.LeftButton,
			QtCore.Qt.KeyboardModifier.NoModifier, end)
		QtWidgets.QApplication.processEvents()


#============================================
def _probe(expected_extension_digest: str, expected_qt_member_digests: str) -> dict[str, object]:
	"""Exercise blank-canvas Draw Bond solely through installed public Qt UI."""
	os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
	from PySide6 import QtCore, QtTest, QtWidgets
	import ferrum_chem
	import ferrum_qt.main_window
	import ferrum_qt.themes.theme_manager

	site_packages = pathlib.Path(sys.prefix) / "lib" / "python3.12" / "site-packages"
	if hasattr(ferrum_chem, "__path__") or pathlib.Path(ferrum_chem.__file__).suffix != ".so":
		raise BlankCanvasDirectBondE2eError("Ferrum chemistry did not load as a root extension")
	extension_path = pathlib.Path(ferrum_chem.__file__).resolve()
	if extension_path.parent != site_packages:
		raise BlankCanvasDirectBondE2eError("Ferrum chemistry did not load from the isolated venv")
	if hashlib.sha256(extension_path.read_bytes()).hexdigest() != expected_extension_digest:
		raise BlankCanvasDirectBondE2eError("installed Ferrum chemistry differs from supplied wheel")
	for member, expected_digest in json.loads(expected_qt_member_digests).items():
		module_name = member.removesuffix(".py").replace("/", ".")
		module_path = pathlib.Path(importlib.import_module(module_name).__file__).resolve()
		if module_path != site_packages / member:
			raise BlankCanvasDirectBondE2eError("Ferrum Qt did not load %s from its wheel" % member)
		if hashlib.sha256(module_path.read_bytes()).hexdigest() != expected_digest:
			raise BlankCanvasDirectBondE2eError("installed Ferrum Qt differs for %s" % member)
	QtCore.QSettings.setDefaultFormat(QtCore.QSettings.Format.IniFormat)
	QtCore.QSettings.setPath(QtCore.QSettings.Format.IniFormat,
		QtCore.QSettings.Scope.UserScope, str(pathlib.Path(sys.prefix) / "settings"))
	app = QtWidgets.QApplication.instance() or QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	host = ferrum_qt.main_window.MainWindow(theme_manager)
	saved = pathlib.Path(sys.prefix) / "blank-canvas-direct-bond.cdml"
	first_blank_saved = pathlib.Path(sys.prefix) / "first-blank-canvas.cdml"
	second_blank_saved = pathlib.Path(sys.prefix) / "second-blank-canvas.cdml"
	try:
		host.show()
		app.processEvents()
		new = _action(host, "New")
		new.trigger()
		app.processEvents()
		_save_as(host, first_blank_saved)
		_saved_blank_facts(first_blank_saved, "first")
		new.trigger()
		app.processEvents()
		viewport = _viewport(host)
		undo = _action(host, "Undo")
		redo = _action(host, "Redo")
		undo_enabled = undo.isEnabled()
		redo_enabled = redo.isEnabled()
		if undo_enabled:
			raise BlankCanvasDirectBondE2eError(
				"second blank New enabled Undo: undo_enabled=%r redo_enabled=%r" % (
					undo_enabled, redo_enabled,
				),
			)
		if redo_enabled:
			raise BlankCanvasDirectBondE2eError(
				"second blank New enabled Redo: undo_enabled=%r redo_enabled=%r" % (
					undo_enabled, redo_enabled,
				),
			)
		_save_as(host, second_blank_saved)
		_saved_blank_facts(second_blank_saved, "second")
		center = viewport.rect().center()
		first_end = center + QtCore.QPoint(96, 0)
		before = _image_digest(viewport)
		draw_bond = _action(host, "Draw Bond")
		draw_bond.trigger()
		if not draw_bond.isChecked():
			raise BlankCanvasDirectBondE2eError("Draw Bond QAction did not arm")
		_require_blank_projection(viewport, first_end)
		_drag(viewport, center, first_end, release=False)
		preview = _image_digest(viewport)
		if preview == before:
			raise BlankCanvasDirectBondE2eError("native Draw Bond preview did not alter viewport")
		_drag(viewport, center, first_end, release=True)
		committed = _image_digest(viewport)
		if committed == before:
			raise BlankCanvasDirectBondE2eError("blank-canvas drag did not render a committed bond")
		if not draw_bond.isChecked():
			raise BlankCanvasDirectBondE2eError("Draw Bond QAction did not stay armed after commit")
		if not undo.isEnabled() or redo.isEnabled():
			raise BlankCanvasDirectBondE2eError("one blank-canvas bond did not create exactly one history step")
		_save_as(host, saved)
		facts = _saved_carbon_bond_facts(saved)
		if not draw_bond.isChecked():
			draw_bond.trigger()
		if not draw_bond.isChecked():
			raise BlankCanvasDirectBondE2eError("Draw Bond QAction did not arm for the second gesture")
		second_end = first_end + QtCore.QPoint(72, 48)
		_require_blank_projection(viewport, second_end)
		_drag(viewport, first_end, second_end, release=False)
		if _image_digest(viewport) == committed:
			raise BlankCanvasDirectBondE2eError("second native preview did not alter viewport")
		QtTest.QTest.keyClick(viewport, QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		if (
			_image_digest(viewport) != committed or not undo.isEnabled() or redo.isEnabled()
			or draw_bond.isChecked()
		):
			raise BlankCanvasDirectBondE2eError(
				"Escape changed the committed document, history, or action state",
			)
		undo.trigger()
		app.processEvents()
		if _image_digest(viewport) != before or not redo.isEnabled():
			raise BlankCanvasDirectBondE2eError("Escape added history or Undo did not retire one bond")
		redo.trigger()
		app.processEvents()
		if _image_digest(viewport) != committed:
			raise BlankCanvasDirectBondE2eError("Redo did not restore the one committed bond")
		return {"schema": "ferrum-blank-canvas-direct-bond-e2e-v2", **facts}
	finally:
		host.close()
		app.processEvents()


#============================================
def main() -> int:
	"""Install exact native and Qt wheels in a temporary venv, then prove the slice."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--native-wheel", type=pathlib.Path)
	parser.add_argument("--qt-wheel", type=pathlib.Path)
	parser.add_argument("--expected-extension-digest")
	parser.add_argument("--expected-qt-member-digests")
	parser.add_argument("--probe", action="store_true")
	arguments = parser.parse_args()
	if arguments.probe:
		if arguments.expected_extension_digest is None or arguments.expected_qt_member_digests is None:
			raise BlankCanvasDirectBondE2eError("--probe requires supplied-wheel digests")
		print(json.dumps(_probe(arguments.expected_extension_digest,
			arguments.expected_qt_member_digests), sort_keys=True))
		return 0
	if arguments.native_wheel is None or arguments.qt_wheel is None:
		raise BlankCanvasDirectBondE2eError("both --native-wheel and --qt-wheel are required")
	for artifact in (arguments.native_wheel, arguments.qt_wheel):
		if not artifact.is_file() or artifact.is_symlink() or artifact.suffix != ".whl":
			raise BlankCanvasDirectBondE2eError("wheel artifacts must be regular .whl files")
	native_wheel = arguments.native_wheel.resolve()
	qt_wheel = arguments.qt_wheel.resolve()
	environment = _proof_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-blank-canvas-direct-bond-wheel-") as directory:
		venv = pathlib.Path(directory) / "venv"
		_run(sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv), environment=environment)
		python = venv / "bin" / "python"
		_run(str(python), "-B", "-m", "pip", "install", "--ignore-installed", "--no-deps",
			str(native_wheel), str(qt_wheel), environment=environment)
		_run(str(python), "-I", "-B", "-c",
			"import pathlib, sys; compile(pathlib.Path(sys.argv[1]).read_text(encoding='utf-8'), sys.argv[1], 'exec')",
			str(pathlib.Path(__file__).resolve()), environment=environment)
		qt_members = (
			"ferrum_qt/main_window.py",
			"ferrum_qt/ferrum/direct_bond_gesture_tab.py",
			"ferrum_qt/ferrum/line_tools.py",
		)
		output = _run(str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe",
			"--expected-extension-digest", _extension_member_digest(native_wheel),
			"--expected-qt-member-digests", json.dumps({
				member: _wheel_member_digest(qt_wheel, member) for member in qt_members
			}, sort_keys=True), environment=environment)
	value = json.loads(output)
	if value["schema"] != "ferrum-blank-canvas-direct-bond-e2e-v2":
		raise BlankCanvasDirectBondE2eError("installed proof returned an unknown receipt")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
