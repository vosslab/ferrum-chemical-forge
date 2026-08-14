"""Prove installed Ferrum atom-property editing remains a native Rust route."""

# Standard Library
import argparse
import json
import os
import pathlib
import subprocess
import sys
import tempfile


APP_ROOT = pathlib.Path(__file__).resolve().parents[1]


#============================================
class NativeAtomPropertiesE2eError(RuntimeError):
	"""Raised when the installed native atom-properties path loses durable truth."""


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one bytecode-free subprocess and return its standard output."""
	result = subprocess.run(
		command,
		env=environment,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)
	if result.returncode:
		raise NativeAtomPropertiesE2eError(
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
def _atom(projection: object) -> object:
	"""Return the one durable atom from an installed observation projection."""
	atoms = tuple(
		atom for molecule in projection.molecules for atom in molecule.atoms
		if atom.source_id == "atom-a"
	)
	if len(atoms) != 1:
		raise NativeAtomPropertiesE2eError("installed projection lacks atom-a")
	return atoms[0]


#============================================
def _assert_atom(atom: object) -> None:
	"""Assert the nine authored facts carried by one complete native patch."""
	if (
		atom.element, atom.formal_charge, atom.valence, atom.isotope,
		atom.multiplicity, atom.show, atom.show_hydrogens,
	) != ("O", -1, 2, 18, 2, True, True):
		raise NativeAtomPropertiesE2eError("native patch did not retain scalar atom facts")
	if atom.label_font is None or (
		atom.label_font.size, atom.label_font.color,
	) != (15.0, "#a0b1c2"):
		raise NativeAtomPropertiesE2eError("native patch did not retain label font facts")


#============================================
def _probe() -> dict[str, object]:
	"""Open, change, undo/redo, save, and reopen through public native seams."""
	os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
	sys.path.insert(0, str(APP_ROOT))

	import PySide6.QtWidgets
	import ferrum_chem
	import ferrum_qt.native.ferrum_native_main_window

	if hasattr(ferrum_chem, "__path__") or pathlib.Path(ferrum_chem.__file__).suffix != ".so":
		raise NativeAtomPropertiesE2eError("Ferrum chemistry did not load as a root extension")
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	root = pathlib.Path(sys.prefix)
	source_path = root / "native-atom-properties-source.cdml"
	saved_path = root / "native-atom-properties-saved.cdml"
	source_path.write_text(
		'<cdml version="26.08" xmlns:v="urn:vendor"><molecule id="molecule-1">'
		'<atom id="atom-a" name="C" charge="2" valency="4" isotope="13" '
		'multiplicity="3" hydrogens="off" vendor_keep="yes">'
		'<point x="1" y="2"/></atom>'
		'</molecule><v:opaque id="retained" keep="literal"/></cdml>',
		encoding="utf-8",
	)
	host = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	if not host.open_file_path(str(source_path)):
		raise NativeAtomPropertiesE2eError("native CDML open returned false")
	tab = host._active_native_tab()
	if tab is None:
		raise NativeAtomPropertiesE2eError("native host did not create a tab")
	tab.select_atom("atom-a")
	changes_type = ferrum_chem.DocumentAtomPropertyChangeV1
	before_rejected = tab.current_snapshot
	try:
		changes_type.isotope(0)
	except ferrum_chem.OperationValidationError:
		pass
	else:
		raise NativeAtomPropertiesE2eError(
			"installed PyO3 factory accepted unsupported isotope zero",
		)
	after_rejected = tab.current_snapshot
	if (
		after_rejected.revision, after_rejected.digest,
	) != (before_rejected.revision, before_rejected.digest):
		raise NativeAtomPropertiesE2eError(
			"rejected PyO3 atom-property intent mutated the native session",
		)
	changes = (
		changes_type.element("O"), changes_type.formal_charge(-1),
		changes_type.valence(2), changes_type.isotope(18),
		changes_type.multiplicity(2), changes_type.show(True),
		changes_type.show_hydrogens(True), changes_type.font_size(15.0),
		changes_type.label_color("#A0B1c2"),
	)
	result = tab.apply_selected_atom_properties(changes)
	changed = tab.current_snapshot
	if changed.revision != 1 or not tab.is_dirty:
		raise NativeAtomPropertiesE2eError("native patch did not create one dirty revision")
	if result.observation.snapshot.revision != changed.revision:
		raise NativeAtomPropertiesE2eError("native patch result disagrees with tab truth")
	if not tab.has_one_selected_atom() or tab.selected_atom_projection().source_id != "atom-a":
		raise NativeAtomPropertiesE2eError("native patch did not retain the atom selection")
	_assert_atom(_atom(tab._document_observation.projection))
	undone = tab.undo().observation
	if undone.snapshot.revision <= changed.revision or _atom(undone.projection).element != "C":
		raise NativeAtomPropertiesE2eError("native atom-properties undo did not restore source facts")
	redone = tab.redo().observation
	if redone.snapshot.revision <= undone.snapshot.revision:
		raise NativeAtomPropertiesE2eError("native atom-properties redo did not advance history")
	_assert_atom(_atom(redone.projection))
	if not host.save_active_to_path(str(saved_path)):
		raise NativeAtomPropertiesE2eError("native atom-properties save returned false")
	if tab.is_dirty or tab.file_path != saved_path:
		raise NativeAtomPropertiesE2eError("native save did not install its clean published truth")
	reopened = ferrum_chem.DocumentSession.load(saved_path.read_text(encoding="utf-8"))
	reopened_snapshot = reopened.snapshot()
	_assert_atom(_atom(reopened.observe_render(0).document.projection))
	if '<v:opaque id="retained" keep="literal"' not in reopened_snapshot.cdml:
		raise NativeAtomPropertiesE2eError("save/reopen lost the opaque CDML extension")
	if "vendor_keep=\"yes\"" not in reopened_snapshot.cdml:
		raise NativeAtomPropertiesE2eError("save/reopen lost unknown atom/font attributes")
	if reopened_snapshot.is_dirty:
		raise NativeAtomPropertiesE2eError("reopened saved native document is unexpectedly dirty")
	host.close()
	app.processEvents()
	return {
		"schema": "ferrum-native-atom-properties-e2e-v1",
		"revision": reopened_snapshot.revision,
		"clean": not reopened_snapshot.is_dirty,
		"opaque_extension": "retained" in reopened_snapshot.cdml,
		"oasa_imported": any(
			name == "oasa" or name.startswith("oasa.") for name in sys.modules
		),
		"root_extension": pathlib.Path(ferrum_chem.__file__).name,
	}


#============================================
def main() -> int:
	"""Install one wheel in an isolated venv and execute the public proof path."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--wheel", type=pathlib.Path)
	parser.add_argument("--probe", action="store_true")
	arguments = parser.parse_args()
	if arguments.probe:
		print(json.dumps(_probe(), sort_keys=True))
		return 0
	if arguments.wheel is None or not arguments.wheel.is_file():
		raise NativeAtomPropertiesE2eError("--wheel must name one direct wheel artifact")
	environment = _proof_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-native-atom-properties-wheel-") as directory:
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
	if value["oasa_imported"]:
		raise NativeAtomPropertiesE2eError("native atom-properties controller imported OASA")
	if not value["clean"] or not value["opaque_extension"]:
		raise NativeAtomPropertiesE2eError("native atom-properties proof lost durable output truth")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
