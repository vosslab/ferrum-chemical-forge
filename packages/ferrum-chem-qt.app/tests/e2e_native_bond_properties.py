"""Prove installed Ferrum bond-property editing remains a Ferrum Rust route."""

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
class NativeBondPropertiesE2eError(RuntimeError):
	"""Raised when the installed Ferrum bond-properties path loses durable truth."""


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one bytecode-free subprocess and return its standard output."""
	result = subprocess.run(
		command, env=environment, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise NativeBondPropertiesE2eError(
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
def _bond(projection: object) -> object:
	"""Return the durable bond from one installed observation projection."""
	bonds = tuple(
		bond for molecule in projection.molecules for bond in molecule.bonds
		if bond.source_id == "bond-ab"
	)
	if len(bonds) != 1:
		raise NativeBondPropertiesE2eError("installed projection lacks bond-ab")
	return bonds[0]


#============================================
def _assert_bond(bond: object) -> None:
	"""Assert all seven authored facts and durable endpoint identity."""
	import ferrum_chem
	if (
		bond.source_type, bond.order, bond.style, bond.center, bond.line_width,
		bond.bond_width, bond.wedge_width, bond.color,
	) != (
		"n2", ferrum_chem.DocumentBondOrderV1.double,
		ferrum_chem.DocumentBondStyleV1.normal, True, 2.5, 4.0,
		5.0, "#aabbcc",
	):
		raise NativeBondPropertiesE2eError("Ferrum patch did not retain bond facts")
	if (bond.start.source_id, bond.end.source_id) != ("atom-a", "atom-b"):
		raise NativeBondPropertiesE2eError("Ferrum patch changed durable bond endpoints")


#============================================
def _assert_unchanged(tab: object, before: object, label: str) -> None:
	"""Assert a rejected intent left the tab's authoritative snapshot untouched."""
	after = tab.current_snapshot
	if (after.revision, after.digest) != (before.revision, before.digest):
		raise NativeBondPropertiesE2eError(label + " mutated the Ferrum session")


#============================================
def _assert_visual_adapter_refuses_unrepresentable_facts(
		cdml: str, ferrum_chem: object) -> None:
	"""Keep unsupported visual forms from authoring a replacement Ferrum edit."""
	import ferrum_qt.ferrum.bond_properties
	changes = ferrum_chem.DocumentBondPropertyChangeV1
	cases = (
		("bond_width_negative", changes.bond_width(-4.0)),
		("non_normal_style", changes.style(ferrum_chem.DocumentBondStyleV1.dashed)),
	)
	for label, change in cases:
		session = ferrum_chem.DocumentSession.load(cdml)
		changed = session.submit(
			0, ferrum_chem.DocumentOperationV1.set_bond_properties("bond-ab", (change,)),
		).observation.snapshot
		bond = _bond(session.observe(changed.revision).projection)
		try:
			ferrum_qt.ferrum.bond_properties.dialog_model_from_projection(bond)
		except ValueError:
			pass
		else:
			raise NativeBondPropertiesE2eError(
				"Ferrum visual adapter accepted unrepresentable " + label,
			)
		after = session.snapshot()
		if (after.revision, after.digest) != (changed.revision, changed.digest):
			raise NativeBondPropertiesE2eError(
				"visual adapter mutated " + label + " Ferrum document",
			)


#============================================
def _probe() -> dict[str, object]:
	"""Open, change, undo/redo, save, and reopen through public Ferrum seams."""
	os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
	sys.path.insert(0, str(APP_ROOT))

	import PySide6.QtWidgets
	import ferrum_chem
	import ferrum_qt.ferrum.main_window

	if hasattr(ferrum_chem, "__path__") or pathlib.Path(ferrum_chem.__file__).suffix != ".so":
		raise NativeBondPropertiesE2eError("Ferrum chemistry did not load as a root extension")
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	root = pathlib.Path(sys.prefix)
	source_path = root / "native-bond-properties-source.cdml"
	saved_path = root / "native-bond-properties-saved.cdml"
	source_cdml = (
		'<cdml version="26.08" xmlns:v="urn:vendor"><molecule id="molecule-1">'
		'<atom id="atom-a" name="C" vendor_keep="yes"><point x="0" y="0"/></atom>'
		'<atom id="atom-b" name="O"><point x="30" y="0"/></atom>'
		'<bond id="bond-ab" start="atom-a" end="atom-b" type="n1"/></molecule>'
		'<v:opaque id="retained" keep="literal"><v:keep/></v:opaque></cdml>'
	)
	source_path.write_text(source_cdml, encoding="utf-8")
	host = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	if not host.open_file_path(str(source_path)):
		raise NativeBondPropertiesE2eError("Ferrum CDML open returned false")
	tab = host._active_native_tab()
	if tab is None:
		raise NativeBondPropertiesE2eError("Ferrum host did not create a tab")
	try:
		tab.select_bond("bond-ab")
	except Exception as error:
		raise NativeBondPropertiesE2eError(
			"Ferrum projection did not expose bond-ab: %r" % (
				tuple(tab._controller.projection.durable_items),
			),
		) from error
	changes_type = ferrum_chem.DocumentBondPropertyChangeV1
	_assert_visual_adapter_refuses_unrepresentable_facts(source_cdml, ferrum_chem)
	order_change = changes_type.order(ferrum_chem.DocumentBondOrderV1.double)
	before_rejected = tab.current_snapshot
	for invalid in (0.0, float("nan"), float("inf")):
		try:
			changes_type.bond_width(invalid)
		except ferrum_chem.OperationValidationError:
			pass
		else:
			raise NativeBondPropertiesE2eError("invalid signed width was accepted")
	_assert_unchanged(tab, before_rejected, "invalid PyO3 bond-property intent")

	class TupleSubclass(tuple):
		"""Hostile tuple-shaped input for the exact-boundary check."""

	for hostile in (TupleSubclass((order_change,)), (order_change,) * 8):
		try:
			ferrum_chem.DocumentOperationV1.set_bond_properties("bond-ab", hostile)
		except ferrum_chem.OperationValidationError:
			pass
		else:
			raise NativeBondPropertiesE2eError("hostile bond-properties tuple was accepted")
	_assert_unchanged(tab, before_rejected, "hostile PyO3 bond-properties tuple")
	changes = (
		order_change, changes_type.style(ferrum_chem.DocumentBondStyleV1.normal),
		changes_type.center(True), changes_type.line_width(2.5),
		changes_type.bond_width(4.0), changes_type.wedge_width(5.0),
		changes_type.color("#aBc"),
	)
	result = tab.apply_selected_bond_properties(changes)
	changed = tab.current_snapshot
	if changed.revision != 1 or not tab.is_dirty:
		raise NativeBondPropertiesE2eError("Ferrum patch did not create one dirty revision")
	if result.observation.snapshot.revision != changed.revision:
		raise NativeBondPropertiesE2eError("Ferrum patch result disagrees with tab truth")
	if not tab.has_one_selected_bond() or tab.selected_bond_projection().source_id != "bond-ab":
		plan = tab._session.observe_render(changed.revision).molecule_plans[0].plan
		raise NativeBondPropertiesE2eError(
			"Ferrum patch did not retain bond selection: %r; issues=%r" % (
				tuple(tab._controller.projection.durable_items), tuple(plan.issues),
			),
		)
	_assert_bond(_bond(tab._document_observation.projection))
	stale_before = tab.current_snapshot
	try:
		tab._session.submit(0, ferrum_chem.DocumentOperationV1.set_bond_properties(
			"bond-ab", (order_change,),
		))
	except ferrum_chem.RevisionConflictError:
		pass
	else:
		raise NativeBondPropertiesE2eError("stale bond-properties operation was accepted")
	_assert_unchanged(tab, stale_before, "stale PyO3 bond-properties operation")
	undone = tab.undo().observation
	if undone.snapshot.revision <= changed.revision or _bond(undone.projection).source_type != "n1":
		raise NativeBondPropertiesE2eError("Ferrum bond-properties undo did not restore source facts")
	redone = tab.redo().observation
	if redone.snapshot.revision <= undone.snapshot.revision:
		raise NativeBondPropertiesE2eError("Ferrum bond-properties redo did not advance history")
	_assert_bond(_bond(redone.projection))
	if not host.save_active_to_path(str(saved_path)):
		raise NativeBondPropertiesE2eError("Ferrum bond-properties save returned false")
	if tab.is_dirty or tab.file_path != saved_path:
		raise NativeBondPropertiesE2eError("Ferrum save did not install its clean published truth")
	reopened = ferrum_chem.DocumentSession.load(saved_path.read_text(encoding="utf-8"))
	reopened_snapshot = reopened.snapshot()
	_assert_bond(_bond(reopened.observe_render(0).document.projection))
	if '<v:opaque id="retained" keep="literal"' not in reopened_snapshot.cdml:
		raise NativeBondPropertiesE2eError("save/reopen lost the opaque CDML extension")
	if "vendor_keep=\"yes\"" not in reopened_snapshot.cdml or "<v:keep" not in reopened_snapshot.cdml:
		raise NativeBondPropertiesE2eError("save/reopen lost unknown CDML content")
	if reopened_snapshot.is_dirty:
		raise NativeBondPropertiesE2eError("reopened saved Ferrum document is unexpectedly dirty")
	host.close()
	app.processEvents()
	return {
		"schema": "ferrum-native-bond-properties-e2e-v1",
		"revision": reopened_snapshot.revision,
		"clean": not reopened_snapshot.is_dirty,
		"opaque_extension": "retained" in reopened_snapshot.cdml,
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
		raise NativeBondPropertiesE2eError("--wheel must name one direct wheel artifact")
	environment = _proof_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-native-bond-properties-wheel-") as directory:
		venv = pathlib.Path(directory) / "venv"
		_run(sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv), environment=environment)
		python = venv / "bin" / "python"
		_run(str(python), "-B", "-m", "pip", "install", "--no-deps", str(arguments.wheel.resolve()), environment=environment)
		output = _run(str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe", environment=environment)
	value = json.loads(output)
	if not value["clean"] or not value["opaque_extension"]:
		raise NativeBondPropertiesE2eError("Ferrum bond-properties proof lost durable output truth")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
