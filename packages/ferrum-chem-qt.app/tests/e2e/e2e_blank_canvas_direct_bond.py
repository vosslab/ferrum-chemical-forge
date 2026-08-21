"""Prove installed Ferrum Draw Bond creates one C-C bond from a blank canvas."""

# Standard Library
import argparse
import hashlib
import json
import os
import pathlib
import stat
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
def _is_safe_qt_python_member(member: str) -> bool:
	"""Return whether one wheel name is a canonical Ferrum Qt Python member."""
	path = pathlib.PurePosixPath(member)
	return (
		member == path.as_posix()
		and not path.is_absolute()
		and path.parts[:1] == ("ferrum_qt",)
		and len(path.parts) > 1
		and all(part not in ("", ".", "..") for part in path.parts)
		and path.suffix == ".py"
	)


#============================================
def _qt_wheel_member_digests(wheel: pathlib.Path) -> dict[str, str]:
	"""Return exact digests for every safe regular Ferrum Qt Python wheel member."""
	with zipfile.ZipFile(wheel) as archive:
		members: dict[str, str] = {}
		for info in archive.infolist():
			if not _is_safe_qt_python_member(info.filename):
				continue
			if not stat.S_ISREG(info.external_attr >> 16):
				raise BlankCanvasDirectBondE2eError(
				"Ferrum Qt wheel member is not a regular file: %s" % info.filename,
			)
			if info.filename in members:
				raise BlankCanvasDirectBondE2eError(
				"Ferrum Qt wheel repeats a package member: %s" % info.filename,
			)
			members[info.filename] = hashlib.sha256(archive.read(info)).hexdigest()
	if not members:
		raise BlankCanvasDirectBondE2eError("Ferrum Qt wheel has no Python package members")
	return members


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
def _require_sha256(value: object, label: str) -> None:
	"""Require one canonical lower-level SHA-256 field from a receipt."""
	if not isinstance(value, str) or len(value) != 64 or any(
		character not in "0123456789abcdef" for character in value.lower()
	):
		raise BlankCanvasDirectBondE2eError("%s must be a 64-character SHA-256" % label)


#============================================
def _validate_source_closure(value: object, schema: str, fields: set[str]) -> None:
	"""Validate one exact native-wheel source-closure receipt record."""
	if not isinstance(value, dict) or set(value) != fields or value["schema"] != schema:
		raise BlankCanvasDirectBondE2eError("native receipt has an invalid %s record" % schema)
	files = value["files"]
	if not isinstance(files, list) or not files:
		raise BlankCanvasDirectBondE2eError("native receipt %s files must be non-empty" % schema)
	for entry in files:
		if not isinstance(entry, dict) or set(entry) != {"path", "sha256"}:
			raise BlankCanvasDirectBondE2eError("native receipt %s has an invalid file record" % schema)
		if not isinstance(entry["path"], str) or not entry["path"]:
			raise BlankCanvasDirectBondE2eError("native receipt %s has an empty file path" % schema)
		_require_sha256(entry["sha256"], "%s file digest" % schema)
	_require_sha256(value["fingerprint_sha256"], "%s fingerprint" % schema)


#============================================
def _wheel_member_manifest(wheel: pathlib.Path) -> dict[str, object]:
	"""Return the exact safe wheel-member manifest used by pair publication."""
	with zipfile.ZipFile(wheel) as archive:
		members: list[dict[str, str]] = []
		seen: set[str] = set()
		for info in archive.infolist():
			name = info.filename
			path = pathlib.PurePosixPath(name)
			if not name or name.endswith("/") or path.is_absolute() or ".." in path.parts:
				raise BlankCanvasDirectBondE2eError("wheel has an unsafe member path: %r" % name)
			if name in seen:
				raise BlankCanvasDirectBondE2eError("wheel repeats a member: %s" % name)
			seen.add(name)
			members.append({"path": name, "sha256": hashlib.sha256(archive.read(info)).hexdigest()})
	payload = json.dumps(members, separators=(",", ":"), sort_keys=True).encode("utf-8")
	return {"members": sorted(members, key=lambda item: item["path"]),
		"fingerprint_sha256": hashlib.sha256(payload).hexdigest()}


#============================================
def _validate_current_pair(native_wheel: pathlib.Path, qt_wheel: pathlib.Path) -> None:
	"""Require both supplied wheels to be exact members of the selected immutable pair."""
	repository = pathlib.Path(__file__).resolve().parents[4]
	current = repository / "output_native_wheel" / "current"
	if not current.is_symlink():
		raise BlankCanvasDirectBondE2eError("native wheel current publication is not a symlink")
	publication = current.resolve(strict=True)
	wheelhouse = publication / "wheelhouse"
	selected_wheel = wheelhouse / native_wheel.name
	selected_qt_wheel = wheelhouse / qt_wheel.name
	if native_wheel.resolve() != selected_wheel.resolve() or not selected_wheel.is_file():
		raise BlankCanvasDirectBondE2eError("native wheel is not the exact current pair artifact")
	if qt_wheel.resolve() != selected_qt_wheel.resolve() or not selected_qt_wheel.is_file():
		raise BlankCanvasDirectBondE2eError("Qt wheel is not the exact current pair artifact")
	try:
		receipt = json.loads((publication / "native-wheel-build-receipt.json").read_text(
			encoding="utf-8"
		))
	except (OSError, json.JSONDecodeError) as error:
		raise BlankCanvasDirectBondE2eError("current native publication receipt is unavailable") from error
	if not isinstance(receipt, dict):
		raise BlankCanvasDirectBondE2eError("current native publication receipt is not an object")
	wheel = receipt.get("wheel")
	if not isinstance(wheel, dict) or wheel.get("filename") != native_wheel.name:
		raise BlankCanvasDirectBondE2eError("current native publication receipt names another wheel")
	_require_sha256(wheel.get("sha256"), "current native wheel digest")
	if hashlib.sha256(selected_wheel.read_bytes()).hexdigest() != wheel["sha256"]:
		raise BlankCanvasDirectBondE2eError("current native wheel differs from its receipt")
	try:
		pair_receipt = json.loads((publication / "developer-wheel-publication-receipt.json").read_text(
			encoding="utf-8"
		))
	except (OSError, json.JSONDecodeError) as error:
		raise BlankCanvasDirectBondE2eError("current developer pair receipt is unavailable") from error
	if not isinstance(pair_receipt, dict) or pair_receipt.get("schema") != "ferrum-developer-wheel-publication-v1":
		raise BlankCanvasDirectBondE2eError("current developer pair receipt has an unknown schema")
	for label, supplied_wheel, record in (
		("native", native_wheel, pair_receipt.get("native_wheel")),
		("Qt", qt_wheel, pair_receipt.get("qt_wheel")),
	):
		if not isinstance(record, dict) or record.get("filename") != supplied_wheel.name:
			raise BlankCanvasDirectBondE2eError("current pair receipt names another %s wheel" % label)
		_require_sha256(record.get("sha256"), "current %s wheel digest" % label)
		if hashlib.sha256(supplied_wheel.read_bytes()).hexdigest() != record["sha256"]:
			raise BlankCanvasDirectBondE2eError("current pair receipt digest differs for %s wheel" % label)
	if pair_receipt.get("qt_wheel_members") != _wheel_member_manifest(qt_wheel):
		raise BlankCanvasDirectBondE2eError("current pair receipt Qt wheel member manifest differs")
	_validate_source_closure(
		receipt.get("ferrum_source_closure"), "ferrum-wheel-source-closure-v2",
		{"excluded_directories", "files", "fingerprint_sha256", "schema"},
	)
	_validate_source_closure(
		receipt.get("ferrum_worktree_source_closure"), "ferrum-wheel-worktree-source-v1",
		{"excluded_directories", "excluded_suffixes", "files", "fingerprint_sha256", "schema"},
	)


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
def _history_failure_facts(host: object, undo: object, redo: object) -> str:
	"""Return non-mutating native and QAction facts for a history assertion failure."""
	facts: dict[str, object] = {
		"undo_enabled": undo.isEnabled(),
		"redo_enabled": redo.isEnabled(),
	}
	tab = host._active_native_tab()
	for name in ("can_undo", "can_redo"):
		try:
			facts["native_%s" % name] = getattr(tab, name)()
		except Exception as error:
			facts["native_%s_error" % name] = "%s: %s" % (
				type(error).__name__, error,
			)
	try:
		snapshot = tab.current_snapshot
		facts["snapshot_revision"] = snapshot.revision
		facts["snapshot_digest"] = snapshot.digest
	except Exception as error:
		facts["snapshot_error"] = "%s: %s" % (type(error).__name__, error)
	return repr(facts)


#============================================
def _installed_qt_member_digests(site_packages: pathlib.Path) -> dict[str, str]:
	"""Return exact digests for every regular installed Ferrum Qt Python member."""
	package = site_packages / "ferrum_qt"
	if not package.is_dir() or package.is_symlink():
		raise BlankCanvasDirectBondE2eError("Ferrum Qt package is not a regular installed tree")
	members: dict[str, str] = {}
	for path in package.rglob("*.py"):
		member = path.relative_to(site_packages).as_posix()
		if not _is_safe_qt_python_member(member) or path.is_symlink() or not path.is_file():
			raise BlankCanvasDirectBondE2eError(
				"Ferrum Qt installed member is not a safe regular file: %s" % member,
			)
		members[member] = hashlib.sha256(path.read_bytes()).hexdigest()
	return members


#============================================
def _validate_installed_qt_package(
	site_packages: pathlib.Path, expected_qt_member_digests: str,
) -> None:
	"""Require the installed Ferrum Qt Python tree to exactly match its wheel."""
	try:
		expected = json.loads(expected_qt_member_digests)
	except json.JSONDecodeError as error:
		raise BlankCanvasDirectBondE2eError("Ferrum Qt wheel digest receipt is invalid JSON") from error
	if (
		not isinstance(expected, dict) or not expected
		or any(
			not isinstance(member, str) or not _is_safe_qt_python_member(member)
			or not isinstance(digest, str) or len(digest) != 64
			for member, digest in expected.items()
		)
	):
		raise BlankCanvasDirectBondE2eError("Ferrum Qt wheel digest receipt is invalid")
	installed = _installed_qt_member_digests(site_packages)
	if set(installed) != set(expected):
		raise BlankCanvasDirectBondE2eError(
			"installed Ferrum Qt package members differ from the supplied wheel",
		)
	for member, digest in expected.items():
		if installed[member] != digest:
			raise BlankCanvasDirectBondE2eError(
				"installed Ferrum Qt differs for %s" % member,
			)


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
	_validate_installed_qt_package(site_packages, expected_qt_member_digests)
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
			raise BlankCanvasDirectBondE2eError(
				"one blank-canvas bond did not create exactly one history step: %s" %
				_history_failure_facts(host, undo, redo),
			)
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
	_validate_current_pair(native_wheel, qt_wheel)
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
		output = _run(str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe",
			"--expected-extension-digest", _extension_member_digest(native_wheel),
			"--expected-qt-member-digests", json.dumps(
				_qt_wheel_member_digests(qt_wheel), sort_keys=True,
			), environment=environment)
	value = json.loads(output)
	if value["schema"] != "ferrum-blank-canvas-direct-bond-e2e-v2":
		raise BlankCanvasDirectBondE2eError("installed proof returned an unknown receipt")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
