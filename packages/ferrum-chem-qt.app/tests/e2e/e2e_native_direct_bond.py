"""Prove installed Ferrum Draw Bond atomically authors one normal C-C bond."""

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
import zipfile


#============================================
class NativeDirectBondE2eError(RuntimeError):
	"""Raised when the installed Draw Bond workflow loses its bounded contract."""


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one bytecode-free subprocess and return standard output."""
	result = subprocess.run(
		command, env=environment, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise NativeDirectBondE2eError(
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
			raise NativeDirectBondE2eError(
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
			raise NativeDirectBondE2eError(
				f"native wheel must contain one root ferrum_chem extension, found {members!r}",
			)
		return _wheel_member_digest(wheel, members[0])


#============================================
def _viewport_point(tab: object, atom_id: str) -> object:
	"""Map one durable atom position through the public viewport seam."""
	return tab.view.mapFromScene(tab.durable_atom_scene_position(atom_id))


#============================================
def _molecule(tab: object) -> object:
	"""Return the one authoritative molecule used by this bounded proof."""
	molecules = tab.current_document_observation().projection.molecules
	if len(molecules) != 1:
		raise NativeDirectBondE2eError("Draw Bond proof expected exactly one molecule")
	return molecules[0]


#============================================
def _assert_projection(tab: object, *, atom_count: int,
		order_counts: dict[str, int]) -> None:
	"""Require the public projection to retain the authored normal-order matrix."""
	molecule = _molecule(tab)
	if len(molecule.atoms) != atom_count or any(atom.element != "C" for atom in molecule.atoms):
		raise NativeDirectBondE2eError(
			"Draw Bond projection lost its carbon endpoint facts: " +
			json.dumps(_projection_failure_facts(tab), sort_keys=True),
		)
	actual_counts: dict[str, int] = {}
	for bond in molecule.bonds:
		actual_counts[bond.source_type] = actual_counts.get(bond.source_type, 0) + 1
	if actual_counts != order_counts:
		raise NativeDirectBondE2eError(
			"Draw Bond projection orders differ: %r != %r" % (actual_counts, order_counts),
		)


#============================================
def _projection_failure_facts(tab: object) -> dict[str, object]:
	"""Return aggregate commit and installed-observation facts without identities."""
	def observation_facts(observation: object) -> dict[str, object]:
		"""Summarize one observation using only projected chemistry facts."""
		projection = observation.projection
		molecules = []
		for molecule in projection.molecules:
			presentations: dict[str, int] = {}
			for bond in molecule.bonds:
				name = str(bond.source_type)
				presentations[name] = presentations.get(name, 0) + 1
			molecules.append({
				"atom_count": len(molecule.atoms),
				"elements": tuple(atom.element for atom in molecule.atoms),
				"bond_count": len(molecule.bonds),
				"presentation_orders": presentations,
			})
		return {
			"digest": projection.digest,
			"molecule_count": len(projection.molecules),
			"molecules": molecules,
			"revision": projection.revision,
		}

	commit_observation = tab.current_document_observation()
	installed_observation = tab._document_observation
	try:
		equality: object = commit_observation == installed_observation
	except Exception as exc:
		equality = type(exc).__name__
	return {
		"commit_observation": observation_facts(commit_observation),
		"current_snapshot": {
			"digest": tab.current_snapshot.digest,
			"revision": tab.current_snapshot.revision,
		},
		"installed_tab_observation": observation_facts(installed_observation),
		"observation_equality": equality,
	}


#============================================
def _select_normal_order(host: object, order_name: str) -> None:
	"""Choose one visible normal order through the product Next Drawing QAction."""
	from PySide6 import QtCore, QtWidgets
	application = QtWidgets.QApplication.instance()
	if application is None:
		raise NativeDirectBondE2eError("Next Drawing requires a running Qt application")
	if host._draw_bond_action.isChecked():
		host._draw_bond_action.trigger()
		application.processEvents()
	if host._draw_bond_action.isChecked() or host._line_gesture_intent is not None:
		raise NativeDirectBondE2eError("Draw Bond did not deactivate before order selection")

	def accept_choice() -> None:
		"""Set the dialog's public combo controls then close its modal client."""
		dialogs = [
			widget for widget in QtWidgets.QApplication.topLevelWidgets()
			if widget.windowTitle() == "Next Drawing" and hasattr(widget, "client")
		]
		if len(dialogs) != 1:
			raise NativeDirectBondE2eError("Next Drawing QAction did not open one dialog")
		dialog = dialogs[0]
		client = dialog.client
		client.presentation_combo.setCurrentIndex(client.presentation_combo.findData("normal"))
		index = client.order_combo.findData(order_name)
		if index < 0:
			raise NativeDirectBondE2eError("Next Drawing dialog lacks %r order" % order_name)
		client.order_combo.setCurrentIndex(index)
		dialog.accept()

	QtCore.QTimer.singleShot(0, accept_choice)
	host._next_drawing_action.trigger()
	application.processEvents()
	if any(
			widget.windowTitle() == "Next Drawing"
			for widget in QtWidgets.QApplication.topLevelWidgets()
		):
		raise NativeDirectBondE2eError("Next Drawing modal client did not retire")
	if application.activeModalWidget() is not None:
		raise NativeDirectBondE2eError("a modal Qt client remained after Next Drawing")
	if host._drawing_parameters.snapshot().order_name != order_name:
		raise NativeDirectBondE2eError("Next Drawing QAction did not retain %r" % order_name)


#============================================
def _drag_bond(host: object, tab: object, start_id: str, end: object,
		*, order_name: str, expected_atom_count: int,
		expected_orders: dict[str, int], refusals: list[object]) -> None:
	"""Drive one accepted Draw Bond drag and prove its exact state transition."""
	from PySide6 import QtCore, QtTest

	start = _viewport_point(tab, start_id)
	if tab.durable_direct_bond_start_atom_at_viewport_point(start) != start_id:
		raise NativeDirectBondE2eError("Draw Bond direct start picker lost its expected atom")
	before = tab.current_snapshot.revision
	if not host._draw_bond_action.isChecked():
		host._draw_bond_action.trigger()
	if not host._draw_bond_action.isChecked():
		raise NativeDirectBondE2eError("Draw Bond QAction did not arm")
	QtTest.QTest.mousePress(tab.view.viewport(), QtCore.Qt.MouseButton.LeftButton,
		QtCore.Qt.KeyboardModifier.NoModifier, start)
	QtTest.QTest.mouseMove(tab.view.viewport(), end)
	QtWidgets = importlib.import_module("PySide6.QtWidgets")
	QtWidgets.QApplication.processEvents()
	intent = host._line_gesture_intent
	if intent is None or intent.direct_bond_admission is None or intent.preview is None:
		raise NativeDirectBondE2eError(
			"Draw Bond did not retain an admitted preview receipt: " + json.dumps({
				"action_checked": host._draw_bond_action.isChecked(),
				"admission_present": getattr(intent, "direct_bond_admission", None) is not None,
				"direct_bond_gesture_present": getattr(intent, "direct_bond_gesture", None) is not None,
				"line_intent_present": intent is not None,
				"typed_refusals": tuple(type(request).__name__ for request in refusals),
			}, sort_keys=True),
		)
	QtTest.QTest.mouseRelease(tab.view.viewport(), QtCore.Qt.MouseButton.LeftButton,
		QtCore.Qt.KeyboardModifier.NoModifier, end)
	QtWidgets.QApplication.processEvents()
	if tab.current_snapshot.revision != before + 1:
		raise NativeDirectBondE2eError("accepted Draw Bond did not add one revision")
	_assert_projection(
		tab, atom_count=expected_atom_count, order_counts=expected_orders,
	)


#============================================
def _wait_for_local_document_open(host: object, path: pathlib.Path, *, require_success: bool) -> None:
	"""Wait for the production local-open completion before using its tab."""
	from PySide6 import QtCore

	completed: list[bool] = []
	loop = QtCore.QEventLoop()
	timeout = QtCore.QTimer()
	timeout.setSingleShot(True)

	def receive_completion(completed_path: str, success: bool) -> None:
		"""Record exactly the inline-CDML load completion."""
		if pathlib.Path(completed_path) == path:
			completed.append(success)
			if not host.has_pending_local_document_open():
				loop.quit()

	host.local_document_open_completed.connect(receive_completion)
	timeout.timeout.connect(loop.quit)
	try:
		if host.has_pending_local_document_open():
			timeout.start(10000)
			loop.exec()
		if not completed or host.has_pending_local_document_open():
			raise NativeDirectBondE2eError("inline CDML did not complete within 10 seconds")
		if require_success and completed != [True]:
			raise NativeDirectBondE2eError("inline CDML did not complete one successful open")
	finally:
		timeout.stop()
		host.local_document_open_completed.disconnect(receive_completion)


#============================================
def _drain_local_document_open(host: object, path: pathlib.Path) -> None:
	"""Cancel and join a failed local-open worker before closing its Qt host."""
	if not host.has_pending_local_document_open():
		return
	host._cancel_local_document_open()
	_wait_for_local_document_open(host, path, require_success=False)
	if host.has_pending_local_document_open():
		raise NativeDirectBondE2eError("inline CDML worker remained pending after cancellation")


#============================================
def _probe(expected_extension_digest: str, expected_qt_member_digests: str) -> dict[str, object]:
	"""Drive normal-order authoring through the product window in supplied wheels."""
	os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
	from PySide6 import QtCore, QtTest, QtWidgets
	import ferrum_chem
	import ferrum_qt.main_window
	import ferrum_qt.themes.theme_manager

	site_packages = pathlib.Path(sys.prefix) / "lib" / "python3.12" / "site-packages"
	if hasattr(ferrum_chem, "__path__") or pathlib.Path(ferrum_chem.__file__).suffix != ".so":
		raise NativeDirectBondE2eError("Ferrum chemistry did not load as a root extension")
	extension_path = pathlib.Path(ferrum_chem.__file__).resolve()
	if extension_path.parent != site_packages:
		raise NativeDirectBondE2eError("Ferrum chemistry did not load from the isolated venv")
	if hashlib.sha256(extension_path.read_bytes()).hexdigest() != expected_extension_digest:
		raise NativeDirectBondE2eError("installed Ferrum chemistry differs from the supplied wheel")
	for member, expected_digest in json.loads(expected_qt_member_digests).items():
		module_name = member.removesuffix(".py").replace("/", ".")
		module_path = pathlib.Path(importlib.import_module(module_name).__file__).resolve()
		if module_path != site_packages / member:
			raise NativeDirectBondE2eError("Ferrum Qt did not load %s from its wheel" % member)
		if hashlib.sha256(module_path.read_bytes()).hexdigest() != expected_digest:
			raise NativeDirectBondE2eError("installed Ferrum Qt differs for %s" % member)
	QtCore.QSettings.setDefaultFormat(QtCore.QSettings.Format.IniFormat)
	QtCore.QSettings.setPath(
		QtCore.QSettings.Format.IniFormat, QtCore.QSettings.Scope.UserScope,
		str(pathlib.Path(sys.prefix) / "settings"),
	)
	app = QtWidgets.QApplication.instance() or QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	host = ferrum_qt.main_window.MainWindow(theme_manager)
	refusals: list[object] = []
	host._show_edit_refusal = refusals.append
	source = pathlib.Path(sys.prefix) / "native-direct-bond-source.cdml"
	saved = pathlib.Path(sys.prefix) / "native-direct-bond-saved.cdml"
	capture = pathlib.Path(sys.prefix) / "native-direct-bond-view.png"
	source.write_text(
		'<cdml version="26.08"><molecule id="molecule-1">'
		'<atom id="new-single" name="C"><point x="0" y="0"/></atom>'
		'<atom id="new-double" name="C"><point x="0" y="100"/></atom>'
		'<atom id="new-triple" name="C"><point x="0" y="200"/></atom>'
		'<atom id="old-single-a" name="C"><point x="200" y="0"/></atom>'
		'<atom id="old-single-b" name="C"><point x="250" y="0"/></atom>'
		'<atom id="old-double-a" name="C"><point x="200" y="100"/></atom>'
		'<atom id="old-double-b" name="C"><point x="250" y="100"/></atom>'
		'<atom id="old-triple-a" name="C"><point x="200" y="200"/></atom>'
		'<atom id="old-triple-b" name="C"><point x="250" y="200"/></atom>'
		'</molecule></cdml>', encoding="utf-8",
	)
	try:
		if not host.open_file_path(str(source)):
			raise NativeDirectBondE2eError("inline CDML open returned false")
		_wait_for_local_document_open(host, source, require_success=True)
		tab = host._active_native_tab()
		if tab is None:
			raise NativeDirectBondE2eError("Ferrum product host did not create an inline-CDML tab")
		host.show()
		app.processEvents()
		order_types = {"single": "n1", "double": "n2", "triple": "n3"}
		orders: dict[str, int] = {}
		initial_atom_count = len(_molecule(tab).atoms)
		atom_count = initial_atom_count
		for order_name, start_id, end_scene in (
			("single", "new-single", QtCore.QPointF(60.0, 0.0)),
			("double", "new-double", QtCore.QPointF(60.0, 100.0)),
			("triple", "new-triple", QtCore.QPointF(60.0, 200.0)),
		):
			_select_normal_order(host, order_name)
			orders[order_types[order_name]] = orders.get(order_types[order_name], 0) + 1
			atom_count += 1
			_drag_bond(
				host, tab, start_id, tab.view.mapFromScene(end_scene), order_name=order_name,
				expected_atom_count=atom_count, expected_orders=dict(orders), refusals=refusals,
			)
		for order_name, start_id, end_id in (
			("single", "old-single-a", "old-single-b"),
			("double", "old-double-a", "old-double-b"),
			("triple", "old-triple-a", "old-triple-b"),
		):
			_select_normal_order(host, order_name)
			orders[order_types[order_name]] = orders.get(order_types[order_name], 0) + 1
			_drag_bond(
				host, tab, start_id, _viewport_point(tab, end_id), order_name=order_name,
				expected_atom_count=atom_count, expected_orders=dict(orders), refusals=refusals,
			)
		if not host._undo_action.isEnabled():
			raise NativeDirectBondE2eError("Draw Bond matrix did not create public history")
		for _ in range(6):
			host._undo_action.trigger()
			app.processEvents()
		_assert_projection(tab, atom_count=initial_atom_count, order_counts={})
		for _ in range(6):
			host._redo_action.trigger()
			app.processEvents()
		_assert_projection(tab, atom_count=initial_atom_count + 3, order_counts=orders)
		before_escape = tab.current_snapshot.revision
		_select_normal_order(host, "single")
		escape_start = _viewport_point(tab, "new-single")
		escape_end = tab.view.mapFromScene(QtCore.QPointF(120.0, 0.0))
		host._draw_bond_action.trigger()
		QtTest.QTest.mousePress(tab.view.viewport(), QtCore.Qt.MouseButton.LeftButton,
			QtCore.Qt.KeyboardModifier.NoModifier, escape_start)
		QtTest.QTest.mouseMove(tab.view.viewport(), escape_end)
		app.processEvents()
		intent = host._line_gesture_intent
		if intent is None or intent.direct_bond_admission is None or intent.preview is None:
			raise NativeDirectBondE2eError("Escape branch did not reach post-admission preview")
		QtTest.QTest.keyClick(tab.view.viewport(), QtCore.Qt.Key.Key_Escape)
		app.processEvents()
		if tab.current_snapshot.revision != before_escape:
			raise NativeDirectBondE2eError("Escape after admission mutated the document")
		if host._line_gesture_intent is not None or host._draw_bond_action.isChecked():
			raise NativeDirectBondE2eError("Escape retained a direct-bond receipt or overlay")
		_assert_projection(tab, atom_count=initial_atom_count + 3, order_counts=orders)
		if not tab.view.viewport().grab().save(str(capture)) or not capture.is_file():
			raise NativeDirectBondE2eError("offscreen Draw Bond view capture failed")
		if not host.save_active_to_path(str(saved)):
			raise NativeDirectBondE2eError("public Save did not publish direct-bond orders")
		if tab.current_snapshot.is_dirty:
			raise NativeDirectBondE2eError("Save did not install a clean direct-bond baseline")
		host.close()
		app.processEvents()
		host = ferrum_qt.main_window.MainWindow(theme_manager)
		if not host.open_file_path(str(saved)):
			raise NativeDirectBondE2eError("restart host did not reopen saved CDML")
		_wait_for_local_document_open(host, saved, require_success=True)
		tab = host._active_native_tab()
		if tab is None:
			raise NativeDirectBondE2eError("restart host lost the reopened document")
		_assert_projection(tab, atom_count=initial_atom_count + 3, order_counts=orders)
		return {
			"capture_sha256": hashlib.sha256(capture.read_bytes()).hexdigest(),
			"clean": not tab.current_snapshot.is_dirty,
			"orders": orders,
			"schema": "ferrum-p01-normal-orders-dual-wheel-e2e-v1",
		}
	finally:
		_drain_local_document_open(host, source if host.has_pending_local_document_open() else saved)
		host.close()
		app.processEvents()


#============================================
def main() -> int:
	"""Install exact native and Qt wheels in a temporary venv, then prove Draw Bond."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--native-wheel", type=pathlib.Path)
	parser.add_argument("--qt-wheel", type=pathlib.Path)
	parser.add_argument("--expected-extension-digest")
	parser.add_argument("--expected-qt-member-digests")
	parser.add_argument("--probe", action="store_true")
	arguments = parser.parse_args()
	if arguments.probe:
		if arguments.expected_extension_digest is None or arguments.expected_qt_member_digests is None:
			raise NativeDirectBondE2eError("--probe requires supplied-wheel digests")
		print(json.dumps(_probe(
			arguments.expected_extension_digest, arguments.expected_qt_member_digests,
		), sort_keys=True))
		return 0
	if arguments.native_wheel is None or arguments.qt_wheel is None:
		raise NativeDirectBondE2eError("both --native-wheel and --qt-wheel are required")
	for artifact in (arguments.native_wheel, arguments.qt_wheel):
		if not artifact.is_file() or artifact.is_symlink() or artifact.suffix != ".whl":
			raise NativeDirectBondE2eError("wheel artifacts must be regular .whl files")
	native_wheel = arguments.native_wheel.resolve()
	qt_wheel = arguments.qt_wheel.resolve()
	environment = _proof_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-native-direct-bond-wheel-") as directory:
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
			"ferrum_qt/ferrum/drawing_parameters_client.py",
			"ferrum_qt/ferrum/line_tools.py",
		)
		output = _run(str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe",
			"--expected-extension-digest", _extension_member_digest(native_wheel),
			"--expected-qt-member-digests", json.dumps({
				member: _wheel_member_digest(qt_wheel, member) for member in qt_members
			}, sort_keys=True), environment=environment)
	value = json.loads(output)
	if value["schema"] != "ferrum-p01-normal-orders-dual-wheel-e2e-v1":
		raise NativeDirectBondE2eError("installed Draw Bond proof returned an unknown receipt")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
