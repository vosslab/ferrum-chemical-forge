"""Prove installed Ferrum Change Element preserves durable selected-atom truth."""

# Standard Library
import argparse
import hashlib
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import zipfile


APP_ROOT = pathlib.Path(__file__).resolve().parents[1]


#============================================
class NativeChangeElementE2eError(RuntimeError):
	"""Raised when the installed Change Element path loses durable truth."""


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one bytecode-free subprocess and return standard output."""
	result = subprocess.run(
		command, env=environment, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise NativeChangeElementE2eError(
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
		"DYLD_LIBRARY_PATH",
		"DYLD_FALLBACK_LIBRARY_PATH",
		"DYLD_FRAMEWORK_PATH",
		"DYLD_FALLBACK_FRAMEWORK_PATH",
		"PYTHONHOME",
		"PYTHONPATH",
	):
		environment.pop(variable, None)
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	environment["QT_QPA_PLATFORM"] = "offscreen"
	return environment


#============================================
def _wheel_member_digest(wheel: pathlib.Path, member: str) -> str:
	"""Return one exact regular package member digest from a supplied wheel."""
	with zipfile.ZipFile(wheel) as archive:
		if member not in archive.namelist():
			raise NativeChangeElementE2eError(
				f"supplied wheel lacks required package member {member!r}",
			)
		return hashlib.sha256(archive.read(member)).hexdigest()


#============================================
def _extension_member_digest(wheel: pathlib.Path) -> str:
	"""Return the supplied wheel's sole direct Ferrum chemistry extension digest."""
	with zipfile.ZipFile(wheel) as archive:
		members = [
			member for member in archive.namelist()
			if member.startswith("ferrum_chem") and member.endswith(".so") and "/" not in member
		]
		if len(members) != 1:
			raise NativeChangeElementE2eError(
				f"native wheel must contain one direct ferrum_chem extension, found {members!r}",
			)
		return _wheel_member_digest(wheel, members[0])


#============================================
def _selected_atom_id(tab: object) -> str:
	"""Return the one selected durable atom identifier from public tab state."""
	selected = tab._controller.projection.selected_durable_targets()
	if len(selected) != 1 or selected[0].kind != "atom" or selected[0].identifier is None:
		raise NativeChangeElementE2eError("installed Change Element lost its durable selection")
	return selected[0].identifier


#============================================
def _element(tab: object, atom_id: str) -> str:
	"""Return one authoritative projected atom element by durable source identifier."""
	for molecule in tab._document_observation.projection.molecules:
		for atom in molecule.atoms:
			if atom.source_id == atom_id:
				return atom.element
	raise NativeChangeElementE2eError("installed projection lacks the selected atom")


#============================================
def _wait_for_local_document_open(
		host: object, path: pathlib.Path, *, require_success: bool,
		) -> None:
	"""Wait for one public local-CDML completion without weakening its async contract."""
	from PySide6 import QtCore

	completed: list[bool] = []
	loop = QtCore.QEventLoop()
	timeout = QtCore.QTimer()
	timeout.setSingleShot(True)

	def receive_completion(completed_path: str, success: bool) -> None:
		"""Record only the requested local file route completion."""
		if pathlib.Path(completed_path) != path:
			return
		completed.append(success)
		if not host.has_pending_local_document_open():
			loop.quit()

	host.local_document_open_completed.connect(receive_completion)
	timeout.timeout.connect(loop.quit)
	try:
		if host.has_pending_local_document_open():
			timeout.start(10000)
			loop.exec()
		if not completed:
			raise NativeChangeElementE2eError(
				"Ferrum local CDML open did not complete within 10 seconds",
			)
		if host.has_pending_local_document_open():
			raise NativeChangeElementE2eError(
				"Ferrum local CDML completion retained a pending open",
			)
		if require_success and completed != [True]:
			raise NativeChangeElementE2eError(
				"Ferrum local CDML open completed unsuccessfully",
			)
	finally:
		timeout.stop()
		host.local_document_open_completed.disconnect(receive_completion)


#============================================
def _drain_local_document_open(host: object, path: pathlib.Path) -> None:
	"""Cancel then join a pending local-CDML worker before disposing its host."""
	if not host.has_pending_local_document_open():
		return
	host._cancel_local_document_open()
	_wait_for_local_document_open(host, path, require_success=False)
	if host.has_pending_local_document_open():
		raise NativeChangeElementE2eError(
			"Ferrum local CDML worker remained pending after cancellation",
		)


#============================================
def _probe(expected_extension_digest: str, expected_qt_module_digest: str) -> dict[str, object]:
	"""Run the public window action through load, history, save, and reopen."""
	os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
	import PySide6.QtWidgets
	import ferrum_chem
	import ferrum_qt.ferrum.main_window

	if hasattr(ferrum_chem, "__path__") or pathlib.Path(ferrum_chem.__file__).suffix != ".so":
		raise NativeChangeElementE2eError("Ferrum chemistry did not load as a root extension")
	extension_path = pathlib.Path(ferrum_chem.__file__).resolve()
	expected_parent = pathlib.Path(sys.prefix) / "lib" / "python3.12" / "site-packages"
	if extension_path.parent != expected_parent:
		raise NativeChangeElementE2eError("Ferrum chemistry did not load from the isolated venv")
	if hashlib.sha256(extension_path.read_bytes()).hexdigest() != expected_extension_digest:
		raise NativeChangeElementE2eError("installed Ferrum chemistry differs from the supplied wheel")
	qt_module_path = pathlib.Path(ferrum_qt.ferrum.main_window.__file__).resolve()
	expected_qt_module_path = (
		pathlib.Path(sys.prefix) / "lib" / "python3.12" / "site-packages" /
		"ferrum_qt" / "ferrum" / "main_window.py"
	)
	if qt_module_path != expected_qt_module_path:
		raise NativeChangeElementE2eError("Ferrum Qt did not load from the supplied venv wheel")
	if hashlib.sha256(qt_module_path.read_bytes()).hexdigest() != expected_qt_module_digest:
		raise NativeChangeElementE2eError("installed Ferrum Qt differs from the supplied wheel")
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	root = pathlib.Path(sys.prefix)
	source_path = root / "native-change-element-source.cdml"
	saved_path = root / "native-change-element-saved.cdml"
	capture_path = root / "native-change-element-view.png"
	source_path.write_text(
		'<cdml version="26.08"><molecule id="molecule-1">'
		'<atom id="atom-c" name="C"><point x="0" y="0"/></atom>'
		'<atom id="atom-o" name="O"><point x="30" y="0"/></atom>'
		'</molecule></cdml>', encoding="utf-8",
	)
	host = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	try:
		if not host.open_file_path(str(source_path)):
			raise NativeChangeElementE2eError("Ferrum CDML open returned false")
		_wait_for_local_document_open(host, source_path, require_success=True)
	except BaseException:
		_drain_local_document_open(host, source_path)
		host.close()
		app.processEvents()
		raise
	tab = host._active_native_tab()
	if tab is None:
		raise NativeChangeElementE2eError("Ferrum host did not create a tab")
	tab.select_atom("atom-c")
	app.processEvents()
	if not host._change_element_action.isEnabled():
		raise NativeChangeElementE2eError("Change Element action was not eligible")
	prior_dialog = PySide6.QtWidgets.QInputDialog.getText
	PySide6.QtWidgets.QInputDialog.getText = lambda *_args: ("N", True)
	try:
		host._change_element_action.trigger()
	finally:
		PySide6.QtWidgets.QInputDialog.getText = prior_dialog
	app.processEvents()
	if _element(tab, "atom-c") != "N" or _selected_atom_id(tab) != "atom-c":
		raise NativeChangeElementE2eError("Change Element did not render N with atom-c selected")
	if not tab.view.viewport().grab().save(str(capture_path)) or not capture_path.is_file():
		raise NativeChangeElementE2eError("offscreen Change Element view capture failed")
	host._undo_action.trigger()
	app.processEvents()
	if _element(tab, "atom-c") != "C":
		raise NativeChangeElementE2eError("Change Element undo did not restore carbon")
	host._redo_action.trigger()
	app.processEvents()
	if _element(tab, "atom-c") != "N" or _selected_atom_id(tab) != "atom-c":
		raise NativeChangeElementE2eError("Change Element redo did not restore N selection")
	if not host.save_active_to_path(str(saved_path)):
		raise NativeChangeElementE2eError("Change Element save returned false")
	reopened = ferrum_chem.DocumentSession.load(saved_path.read_text(encoding="utf-8"))
	reopened_atom = reopened.observe_render(0).document.projection.molecules[0].atoms[0]
	if reopened_atom.source_id != "atom-c" or reopened_atom.element != "N":
		raise NativeChangeElementE2eError("saved Change Element document did not reopen as N")
	capture_digest = hashlib.sha256(capture_path.read_bytes()).hexdigest()
	host.close()
	app.processEvents()
	return {
		"schema": "ferrum-native-change-element-e2e-v1",
		"capture_sha256": capture_digest,
		"clean": not reopened.snapshot().is_dirty,
		"selected_atom": "atom-c",
	}


#============================================
def main() -> int:
	"""Install exact native and Qt wheels in a temporary venv, then prove the workflow."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--native-wheel", type=pathlib.Path)
	parser.add_argument("--qt-wheel", type=pathlib.Path)
	parser.add_argument("--expected-extension-digest")
	parser.add_argument("--expected-qt-module-digest")
	parser.add_argument("--probe", action="store_true")
	arguments = parser.parse_args()
	if arguments.probe:
		if arguments.expected_extension_digest is None or arguments.expected_qt_module_digest is None:
			raise NativeChangeElementE2eError(
				"--probe requires native and Qt supplied-wheel digests",
			)
		print(json.dumps(_probe(
			arguments.expected_extension_digest, arguments.expected_qt_module_digest,
		), sort_keys=True))
		return 0
	if arguments.native_wheel is None or arguments.qt_wheel is None:
		raise NativeChangeElementE2eError("both --native-wheel and --qt-wheel are required")
	for artifact in (arguments.native_wheel, arguments.qt_wheel):
		if not artifact.is_file() or artifact.is_symlink() or artifact.suffix != ".whl":
			raise NativeChangeElementE2eError("wheel artifacts must be regular .whl files")
	native_wheel = arguments.native_wheel.resolve()
	qt_wheel = arguments.qt_wheel.resolve()
	environment = _proof_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-native-change-element-wheel-") as directory:
		venv = pathlib.Path(directory) / "venv"
		_run(sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv), environment=environment)
		python = venv / "bin" / "python"
		_run(
			str(python), "-B", "-m", "pip", "install", "--ignore-installed", "--no-deps",
			str(native_wheel), str(qt_wheel), environment=environment,
		)
		output = _run(
			str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe",
			"--expected-extension-digest", _extension_member_digest(native_wheel),
			"--expected-qt-module-digest", _wheel_member_digest(
				qt_wheel, "ferrum_qt/ferrum/main_window.py",
			),
			environment=environment,
		)
	value = json.loads(output)
	if not value["clean"] or value["selected_atom"] != "atom-c":
		raise NativeChangeElementE2eError("installed Change Element proof lost durable output truth")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
