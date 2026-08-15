"""Prove installed persistent atom annotations remain a native Rust route."""

# Standard Library
import argparse
import json
import os
import pathlib
import subprocess
import sys
import tempfile


APP_ROOT = pathlib.Path(__file__).resolve().parents[1]
_MARK_CASES = (
	("atom-a", "plus", ("ellipse", "line", "line")),
	("atom-b", "minus", ("ellipse", "line")),
	("atom-c", "radical", ("ellipse",)),
	("atom-d", "biradical", ("ellipse", "ellipse")),
	("atom-e", "electronpair", ("line",)),
	("atom-f", "dotted_electronpair", ("ellipse", "ellipse")),
	("atom-g", "pz_orbital", ("ellipse", "ellipse")),
)


#============================================
class NativeAtomNumberE2eError(RuntimeError):
	"""Raised when installed atom numbering loses authoritative truth."""


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one bytecode-free subprocess and return its standard output."""
	result = subprocess.run(
		command, env=environment, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise NativeAtomNumberE2eError(
			"command failed (%d): %s\n%s" % (
				result.returncode, " ".join(command), result.stderr.strip(),
			),
		)
	return result.stdout


#============================================
def _proof_environment() -> dict[str, str]:
	"""Return an isolated offscreen environment that cannot write bytecode."""
	environment = os.environ.copy()
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	environment["QT_QPA_PLATFORM"] = "offscreen"
	return environment


#============================================
def _atom(projection: object, atom_id: str = "atom-a") -> object:
	"""Return one named durable atom from an installed document projection."""
	atoms = tuple(
		atom for molecule in projection.molecules for atom in molecule.atoms
		if atom.source_id == atom_id
	)
	if len(atoms) != 1:
		raise NativeAtomNumberE2eError(
			f"installed projection does not contain exactly one {atom_id}",
		)
	return atoms[0]


#============================================
def _atom_operations(tab: object, atom_id: str = "atom-a") -> tuple[object, ...]:
	"""Return exact current Rust render operations for one durable atom."""
	observation = tab._session.observe_render(tab.current_snapshot.revision)
	for entry in observation.molecule_plans:
		plan = entry.plan
		for batch in plan.batches:
			if batch.target.record_id.kind == "Atom" and batch.target.record_id.id == atom_id:
				return batch.operations
	raise NativeAtomNumberE2eError(f"render observation lacks {atom_id} batch")


#============================================
def _assert_number(tab: object, number: int | None, show_number: bool | None) -> None:
	"""Assert one exact document pair and its visible render consequence."""
	atom = _atom(tab._document_observation.projection)
	if (atom.number, atom.show_number) != (number, show_number):
		raise NativeAtomNumberE2eError("document projection lost the atom number pair")
	operations = _atom_operations(tab)
	if number is not None and show_number is True:
		if len(operations) != 2:
			raise NativeAtomNumberE2eError("visible atom number lacks its text operation")
		number_operation = operations[1]
		if (
			number_operation.kind != "text"
			or number_operation.operation.runs[0].text != str(number)
			or number_operation.operation.paint != "0000c8"
			or number_operation.operation.size != 9.0
		):
			raise NativeAtomNumberE2eError("atom number render facts are not explicit")
	elif len(operations) != 1:
		raise NativeAtomNumberE2eError("hidden or absent atom number was still painted")


#============================================
def _assert_single_mark(tab: object, atom_id: str, kind: object,
		expected_operations: tuple[str, ...]) -> None:
	"""Assert one authored mark and its closed semantic render primitives."""
	atom = _atom(tab._document_observation.projection, atom_id)
	if len(atom.marks) != 1 or atom.marks[0].kind != kind:
		raise NativeAtomNumberE2eError(f"document projection lost the mark on {atom_id}")
	operations = tuple(
		operation.kind for operation in _atom_operations(tab, atom_id)
		if operation.operation.z >= 50
	)
	if operations != expected_operations:
		raise NativeAtomNumberE2eError(
			f"mark render operations differ for {atom_id}: {operations}",
		)
	selected = tab._controller.projection.selected_durable_targets()
	if len(selected) != 1 or selected[0].identifier != atom_id:
		raise NativeAtomNumberE2eError("mark mutation lost durable atom selection")


#============================================
def _probe() -> dict[str, object]:
	"""Open, annotate, undo/redo, save, and reopen through public seams."""
	os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
	sys.path.insert(0, str(APP_ROOT))

	import PySide6.QtWidgets
	import ferrum_chem
	import ferrum_qt.native.ferrum_native_main_window

	if hasattr(ferrum_chem, "__path__") or pathlib.Path(ferrum_chem.__file__).suffix != ".so":
		raise NativeAtomNumberE2eError("Ferrum chemistry did not load as a root extension")
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	root = pathlib.Path(sys.prefix)
	source_path = root / "native-atom-number-source.cdml"
	saved_path = root / "native-atom-number-saved.cdml"
	atoms = "".join(
		'<atom id="atom-%s" name="C"><point x="%d" y="2"/></atom>' % (
			letter, index * 20,
		)
		for index, letter in enumerate("abcdefg", start=1)
	)
	source_path.write_text(
		'<cdml version="26.07" xmlns:v="urn:vendor"><molecule id="molecule-1" '
		'vendor_keep="yes"><v:keep/>' + atoms + '</molecule>'
		'<v:opaque id="retained" keep="literal"/></cdml>',
		encoding="utf-8",
	)
	host = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	if not host.open_file_path(str(source_path)):
		raise NativeAtomNumberE2eError("native CDML open returned false")
	tab = host._active_native_tab()
	if tab is None:
		raise NativeAtomNumberE2eError("native host did not create a tab")
	tab.select_atom("atom-a")
	_assert_number(tab, None, None)

	before = tab.current_snapshot
	for number, show_number in ((True, True), (0, True), (-1, True), (1, 1)):
		try:
			ferrum_chem.DocumentOperationV1.set_atom_number(
				"molecule-1", "atom-a", number, show_number,
			)
		except ferrum_chem.OperationValidationError:
			pass
		else:
			raise NativeAtomNumberE2eError("hostile atom-number intent was accepted")
	if (tab.current_snapshot.revision, tab.current_snapshot.digest) != (
		before.revision, before.digest,
	):
		raise NativeAtomNumberE2eError("rejected atom-number intent mutated the session")

	tab.set_selected_atom_number(17, True)
	_assert_number(tab, 17, True)
	if not tab.has_one_selected_atom():
		raise NativeAtomNumberE2eError("number assignment lost durable selection")
	tab.undo()
	_assert_number(tab, None, None)
	tab.redo()
	_assert_number(tab, 17, True)
	tab.set_selected_atom_number(17, False)
	_assert_number(tab, 17, False)
	tab.set_selected_atom_number(42, True)
	_assert_number(tab, 42, True)
	tab.clear_selected_atom_number()
	_assert_number(tab, None, None)
	tab.undo()
	_assert_number(tab, 42, True)

	for atom_id, kind_name, expected_operations in _MARK_CASES:
		tab.select_atom(atom_id)
		kind = getattr(ferrum_chem.AtomMarkKindV1, kind_name)
		tab.apply_selected_atom_mark(
			ferrum_chem.AtomMarkActionV1.add, kind, None,
		)
		_assert_single_mark(tab, atom_id, kind, expected_operations)
	tab.undo()
	if _atom(tab._document_observation.projection, "atom-g").marks:
		raise NativeAtomNumberE2eError("mark undo did not remove the last exact mark")
	tab.redo()
	_assert_single_mark(
		tab, "atom-g", ferrum_chem.AtomMarkKindV1.pz_orbital,
		("ellipse", "ellipse"),
	)

	if not host.save_active_to_path(str(saved_path)):
		raise NativeAtomNumberE2eError("native atom-annotation save returned false")
	reopened = ferrum_chem.DocumentSession.load(saved_path.read_text(encoding="utf-8"))
	reopened_snapshot = reopened.snapshot()
	reopened_projection = reopened.observe_render(0).document.projection
	reopened_atom = _atom(reopened_projection)
	if (reopened_atom.number, reopened_atom.show_number) != (42, True):
		raise NativeAtomNumberE2eError("save/reopen lost atom number facts")
	for atom_id, kind_name, _expected_operations in _MARK_CASES:
		marks = _atom(reopened_projection, atom_id).marks
		kind = getattr(ferrum_chem.AtomMarkKindV1, kind_name)
		if len(marks) != 1 or marks[0].kind != kind:
			raise NativeAtomNumberE2eError(f"save/reopen lost the mark on {atom_id}")
	if "vendor_keep=\"yes\"" not in reopened_snapshot.cdml or "v:opaque" not in reopened_snapshot.cdml:
		raise NativeAtomNumberE2eError("save/reopen lost retained extension content")
	host.close()
	app.processEvents()
	return {
		"schema": "ferrum-native-atom-annotations-e2e-v1",
		"clean": not reopened_snapshot.is_dirty,
		"mark_count": sum(len(_atom(reopened_projection, atom_id).marks)
			for atom_id, _kind_name, _operations in _MARK_CASES),
		"number": reopened_atom.number,
		"show_number": reopened_atom.show_number,
		"root_extension": pathlib.Path(ferrum_chem.__file__).name,
	}


#============================================
def main() -> int:
	"""Install one direct wheel in a temporary venv and execute the public proof."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--wheel", type=pathlib.Path)
	parser.add_argument("--probe", action="store_true")
	arguments = parser.parse_args()
	if arguments.probe:
		print(json.dumps(_probe(), sort_keys=True))
		return 0
	if arguments.wheel is None or not arguments.wheel.is_file():
		raise NativeAtomNumberE2eError("--wheel must name one direct wheel artifact")
	environment = _proof_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-native-atom-annotations-wheel-") as directory:
		venv = pathlib.Path(directory) / "venv"
		_run(
			sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv),
			environment=environment,
		)
		python = venv / "bin" / "python"
		_run(
			str(python), "-B", "-m", "pip", "install", "--no-deps",
			str(arguments.wheel.resolve()), environment=environment,
		)
		output = _run(
			str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe",
			environment=environment,
		)
	value = json.loads(output)
	if (
		not value["clean"]
		or value["number"] != 42 or value["mark_count"] != len(_MARK_CASES)
	):
		raise NativeAtomNumberE2eError("native atom-annotation proof lost durable output truth")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
