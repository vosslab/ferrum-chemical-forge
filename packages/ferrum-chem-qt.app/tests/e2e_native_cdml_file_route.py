"""Prove the installed Ferrum wheel owns the offscreen native CDML file route."""

# Standard Library
import argparse
import json
import math
import os
import pathlib
import subprocess
import sys
import tempfile


APP_ROOT = pathlib.Path(__file__).resolve().parents[1]


#============================================
class NativeCdmlRouteE2eError(RuntimeError):
	"""Raised when the native CDML user path contradicts its durable contract."""


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one child with the explicit bytecode-free proof environment."""
	result = subprocess.run(
		command,
		env=environment,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)
	if result.returncode:
		raise NativeCdmlRouteE2eError(
			"command failed (%d): %s\n%s" % (
				result.returncode, " ".join(command), result.stderr.strip(),
			),
		)
	return result.stdout


#============================================
def _proof_environment() -> dict[str, str]:
	"""Return a child environment that neither imports nor writes legacy bytecode."""
	environment = os.environ.copy()
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	environment["QT_QPA_PLATFORM"] = "offscreen"
	return environment


#============================================
def _probe() -> dict[str, object]:
	"""Exercise open, render, save, and reopen using the installed Rust extension."""
	os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
	sys.path.insert(0, str(APP_ROOT))

	import PySide6.QtCore
	import PySide6.QtTest
	import PySide6.QtWidgets
	import ferrum_chem
	import ferrum_qt.native.native_app
	import ferrum_qt.native.ferrum_native_main_window

	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	# The sealed wheel's publication proof uses the owned venv directory,
	# which is also a stable directory-sync target on macOS.
	root = pathlib.Path(sys.prefix)
	source_path = root / "native-route-source.cdml"
	saved_path = root / "native-route-saved.cdml"
	source = (
		'<cdml version="26.08" xmlns:v="urn:vendor">'
		'<molecule id="molecule-1"><atom id="atom-c" name="C">'
		'<point x="0cm" y="0cm"/></atom><atom id="atom-o" name="O">'
		'<point x="2cm" y="0cm"/></atom></molecule>'
		'<v:opaque-root id="payload-1" keep="literal"><v:item/>opaque</v:opaque-root>'
		'</cdml>'
	)
	source_path.write_text(source, encoding="utf-8")
	host = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	if not host.open_file_path(str(source_path)):
		raise NativeCdmlRouteE2eError("native CDML open returned false")
	tab = host._active_native_tab()
	if tab is None or tab.file_path != source_path or tab.is_dirty:
		raise NativeCdmlRouteE2eError("native open did not retain its clean loaded truth")
	if tab.view.scene() is None or not tab.view.scene().items():
		raise NativeCdmlRouteE2eError("native CDML open did not render the molecule")
	host.show()
	app.processEvents()
	tab.select_atom("atom-c")
	tab.change_selected_atom_element("N")
	changed = tab.current_snapshot
	if not tab.is_dirty or changed.revision <= 0:
		raise NativeCdmlRouteE2eError("native element edit did not install a dirty Rust revision")
	tab.undo()
	undone = tab.current_snapshot
	if undone.revision <= changed.revision:
		raise NativeCdmlRouteE2eError("native undo did not create a fresh Rust revision")
	tab.redo()
	redone = tab.current_snapshot
	if redone.revision <= undone.revision:
		raise NativeCdmlRouteE2eError("native redo did not create a fresh Rust revision")

	def atom_viewport_point(atom_id: str) -> PySide6.QtCore.QPoint:
		"""Return one interior hit point from the installed Rust projection item."""
		item = tab._controller.projection.durable_items[("atom", atom_id)]
		shape = item.shape()
		bounds = shape.boundingRect()
		for x_step in range(1, 10):
			for y_step in range(1, 10):
				point = PySide6.QtCore.QPointF(
					bounds.left() + bounds.width() * x_step / 10.0,
					bounds.top() + bounds.height() * y_step / 10.0,
				)
				if shape.contains(point):
					return tab.view.mapFromScene(item.mapToScene(point))
		raise NativeCdmlRouteE2eError("native atom has no hit-test interior")

	def empty_viewport_point() -> PySide6.QtCore.QPoint:
		"""Return one visible point that does not resolve to a durable atom."""
		rect = tab.view.viewport().rect().adjusted(12, 12, -12, -12)
		for x_step in range(1, 10):
			for y_step in range(1, 10):
				point = PySide6.QtCore.QPoint(
					rect.left() + rect.width() * x_step // 10,
					rect.top() + rect.height() * y_step // 10,
				)
				if tab.durable_atom_at_viewport_point(point) is None:
					return point
		raise NativeCdmlRouteE2eError("native viewport has no empty insertion point")

	start = atom_viewport_point("atom-c")
	end = atom_viewport_point("atom-o")
	host._draw_single_bond_action.trigger()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	bonded = tab.current_snapshot
	selected = tab._controller.projection.selected_durable_targets()
	if bonded.revision <= redone.revision or len(selected) != 1 or selected[0].kind != "bond":
		raise NativeCdmlRouteE2eError("native bond transaction did not install its Rust bond")
	created_bond_id = selected[0].identifier
	if created_bond_id is None:
		raise NativeCdmlRouteE2eError("native bond transaction did not retain a durable ID")
	if host._line_gesture_intent is None or host._line_gesture_intent.preview is not None:
		raise NativeCdmlRouteE2eError("native bond gesture did not retire its local preview")
	host._cancel_line_gesture()
	tab.select_bond(created_bond_id)
	order_changed = tab.set_selected_bond_order(
		ferrum_chem.DocumentBondOrderV1.double,
	).observation
	changed_bond = tuple(
		bond for molecule in order_changed.projection.molecules
		for bond in molecule.bonds if bond.source_id == created_bond_id
	)
	if len(changed_bond) != 1 or changed_bond[0].source_type != "n2":
		raise NativeCdmlRouteE2eError("native bond-order edit did not persist n2")
	order_plan = tab._session.observe_render(
		order_changed.snapshot.revision,
	).molecule_plans[0].plan
	order_batch = tuple(
		batch for batch in order_plan.batches
		if batch.target.record_id.id == created_bond_id
	)
	if len(order_batch) != 1 or len(order_batch[0].operations) != 2:
		raise NativeCdmlRouteE2eError("native double bond did not render two line operations")
	order_undone = tab.undo().observation.projection.molecules[0]
	if tuple(
		bond.source_type for bond in order_undone.bonds
		if bond.source_id == created_bond_id
	) != ("n1",):
		raise NativeCdmlRouteE2eError("native bond-order undo did not restore n1")
	order_redone = tab.redo().observation.projection.molecules[0]
	if tuple(
		bond.source_type for bond in order_redone.bonds
		if bond.source_id == created_bond_id
	) != ("n2",):
		raise NativeCdmlRouteE2eError("native bond-order redo did not restore n2")
	before_coordinates = tuple(
		(atom.position.x, atom.position.y) for atom in order_redone.atoms
	)
	before_centroid = (
		sum(point[0] for point in before_coordinates) / len(before_coordinates),
		sum(point[1] for point in before_coordinates) / len(before_coordinates),
	)
	before_bond_length = math.dist(before_coordinates[0], before_coordinates[1])
	pre_coordinate_snapshot = tab.current_snapshot
	host._generate_coordinates_action.trigger()
	coordinate_intent = host._coordinate_generation_intent
	if coordinate_intent is None or not coordinate_intent.worker.wait(10000):
		raise NativeCdmlRouteE2eError("native coordinate worker did not finish")
	for _iteration in range(3):
		app.processEvents()
	if host._coordinate_generation_intent is not None:
		raise NativeCdmlRouteE2eError("native coordinate worker was not released")
	generated_molecule = tab._document_observation.projection.molecules[0]
	generated_positions = tuple(
		(atom.position.x, atom.position.y) for atom in generated_molecule.atoms
	)
	generated_centroid = (
		sum(point[0] for point in generated_positions) / len(generated_positions),
		sum(point[1] for point in generated_positions) / len(generated_positions),
	)
	if (
		tab.current_snapshot.revision <= pre_coordinate_snapshot.revision
		or not math.isclose(generated_centroid[0], before_centroid[0], abs_tol=1e-10)
		or not math.isclose(generated_centroid[1], before_centroid[1], abs_tol=1e-10)
		or not math.isclose(
			math.dist(generated_positions[0], generated_positions[1]),
			before_bond_length,
			rel_tol=1e-12,
		)
	):
		raise NativeCdmlRouteE2eError(
			"native coordinate generation did not retain molecule placement",
		)
	bonded = tab.current_snapshot
	host._draw_single_bond_action.trigger()

	start = atom_viewport_point("atom-o")
	end = empty_viewport_point()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	extended = tab.current_snapshot
	selected = tab._controller.projection.selected_durable_targets()
	root_molecule = tuple(
		molecule for molecule in tab._document_observation.projection.molecules
		if molecule.source_id == "molecule-1"
	)[0]
	if extended.revision <= bonded.revision or len(selected) != 1 or selected[0].kind != "atom":
		raise NativeCdmlRouteE2eError("empty-space drag did not install its Rust atom")
	extended_atom_id = selected[0].identifier
	if extended_atom_id is None or len(root_molecule.atoms) != 3 or len(root_molecule.bonds) != 2:
		raise NativeCdmlRouteE2eError("empty-space drag did not create one atom and one bond")
	extended_bond_ids = tuple(
		bond.source_id for bond in root_molecule.bonds if bond.source_id != created_bond_id
	)
	if len(extended_bond_ids) != 1 or extended_bond_ids[0] is None:
		raise NativeCdmlRouteE2eError("empty-space drag did not retain its generated bond ID")
	extended_bond_id = extended_bond_ids[0]
	host._draw_single_bond_action.trigger()

	start = atom_viewport_point(extended_atom_id)
	end = empty_viewport_point()
	start_pointer = tab.view.mapToScene(start)
	end_pointer = tab.view.mapToScene(end)
	anchor = tab.durable_atom_scene_position(extended_atom_id)
	expected_moved = anchor + (end_pointer - start_pointer)
	host._move_atom_action.trigger()
	PySide6.QtTest.QTest.mousePress(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
	)
	PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
	PySide6.QtTest.QTest.mouseRelease(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
	)
	moved = tab.current_snapshot
	selected = tab._controller.projection.selected_durable_targets()
	moved_atom = tuple(
		atom for molecule in tab._document_observation.projection.molecules
		for atom in molecule.atoms if atom.source_id == extended_atom_id
	)
	if (
		moved.revision <= extended.revision
		or len(selected) != 1
		or selected[0].identifier != extended_atom_id
		or len(moved_atom) != 1
		or (moved_atom[0].position.x, moved_atom[0].position.y)
		!= (expected_moved.x(), expected_moved.y())
	):
		raise NativeCdmlRouteE2eError("native move gesture did not install its Rust position")
	host._move_atom_action.trigger()
	tab.select_atom("atom-c")
	host._delete_atom_action.trigger()
	deleted = tab.current_snapshot
	root_molecule = tuple(
		molecule for molecule in tab._document_observation.projection.molecules
		if molecule.source_id == "molecule-1"
	)[0]
	if (
		deleted.revision <= moved.revision
		or any(atom.source_id == "atom-c" for atom in root_molecule.atoms)
		or any(bond.source_id == created_bond_id for bond in root_molecule.bonds)
		or not any(bond.source_id == extended_bond_id for bond in root_molecule.bonds)
	):
		raise NativeCdmlRouteE2eError("native deletion did not remove atom-c and its bond")
	restored = tab.undo().observation.projection.molecules[0]
	if (
		not any(atom.source_id == "atom-c" for atom in restored.atoms)
		or not any(bond.source_id == created_bond_id for bond in restored.bonds)
	):
		raise NativeCdmlRouteE2eError("native deletion undo did not restore atom-c and its bond")
	redone_delete = tab.redo().observation.projection.molecules[0]
	if (
		any(atom.source_id == "atom-c" for atom in redone_delete.atoms)
		or any(bond.source_id == created_bond_id for bond in redone_delete.bonds)
	):
		raise NativeCdmlRouteE2eError("native deletion redo did not restore the deletion")
	tab.select_bond(extended_bond_id)
	host._delete_bond_action.trigger()
	bond_deleted = tab.current_snapshot
	without_selected_bond = tab._document_observation.projection.molecules[0]
	if (
		bond_deleted.revision <= deleted.revision
		or not any(atom.source_id == extended_atom_id for atom in without_selected_bond.atoms)
		or any(bond.source_id == extended_bond_id for bond in without_selected_bond.bonds)
	):
		raise NativeCdmlRouteE2eError("native bond deletion changed an endpoint or kept the bond")
	bond_restored = tab.undo().observation.projection.molecules[0]
	if not any(bond.source_id == extended_bond_id for bond in bond_restored.bonds):
		raise NativeCdmlRouteE2eError("native bond deletion undo did not restore the bond")
	bond_redone = tab.redo().observation.projection.molecules[0]
	if any(bond.source_id == extended_bond_id for bond in bond_redone.bonds):
		raise NativeCdmlRouteE2eError("native bond deletion redo did not restore the deletion")
	if not host.start_smiles_import("CCO"):
		raise NativeCdmlRouteE2eError("public native SMILES import did not start")
	intent = host._smiles_import_intent
	if intent is None or not intent.worker.wait(10000):
		raise NativeCdmlRouteE2eError("native SMILES worker did not finish")
	for _iteration in range(3):
		app.processEvents()
	if host._smiles_import_intent is not None:
		raise NativeCdmlRouteE2eError("native SMILES worker was not released")
	inserted = tab.current_snapshot
	if inserted.revision <= bond_deleted.revision or not tab.is_dirty:
		raise NativeCdmlRouteE2eError("native SMILES transaction did not install a dirty revision")
	inserted_projection = tab._document_observation.projection
	cco_molecules = tuple(
		molecule for molecule in inserted_projection.molecules
		if tuple(atom.element for atom in molecule.atoms) == ("C", "C", "O")
		and len(molecule.bonds) == 2
	)
	if len(cco_molecules) != 1:
		raise NativeCdmlRouteE2eError("native SMILES transaction did not project exact CCO")
	if not host.save_active_to_path(str(saved_path)):
		raise NativeCdmlRouteE2eError("native CDML save returned false")
	if tab.file_path != saved_path or tab.is_dirty or tab.title != saved_path.name:
		raise NativeCdmlRouteE2eError("confirmed save did not update the native tab truth")
	saved = ferrum_chem.DocumentSession.load(saved_path.read_text(encoding="utf-8"))
	reopened = saved.snapshot()
	reopened_projection = saved.observe_render(0).document.projection
	output = reopened.cdml
	for required in (
		'<molecule id="molecule-1">', 'id="atom-o"',
		f'id="{extended_atom_id}" name="C"',
		'<v:opaque-root id="payload-1" keep="literal">',
	):
		if required not in output:
			raise NativeCdmlRouteE2eError("saved Rust CDML lost %s" % required)
	for deleted_value in (
		'id="atom-c"', f'id="{created_bond_id}"', f'id="{extended_bond_id}"',
	):
		if deleted_value in output:
			raise NativeCdmlRouteE2eError("saved Rust CDML restored %s" % deleted_value)
	if not output.index('id="atom-o"') < output.index(f'id="{extended_atom_id}"'):
		raise NativeCdmlRouteE2eError("saved Rust CDML changed molecule source order")
	reopened_cco = tuple(
		molecule for molecule in reopened_projection.molecules
		if tuple(atom.element for atom in molecule.atoms) == ("C", "C", "O")
		and len(molecule.bonds) == 2
	)
	if len(reopened_cco) != 1:
		raise NativeCdmlRouteE2eError("save/reopen lost the native SMILES molecule")
	reopened_moved = tuple(
		atom for molecule in reopened_projection.molecules for atom in molecule.atoms
		if atom.source_id == extended_atom_id
	)
	if (
		len(reopened_moved) != 1
		or (reopened_moved[0].position.x, reopened_moved[0].position.y)
		!= (expected_moved.x(), expected_moved.y())
	):
		raise NativeCdmlRouteE2eError("save/reopen lost the Rust atom movement")
	reopened_atom_o = tuple(
		atom for molecule in reopened_projection.molecules for atom in molecule.atoms
		if atom.source_id == "atom-o"
	)
	if (
		len(reopened_atom_o) != 1
		or (reopened_atom_o[0].position.x, reopened_atom_o[0].position.y)
		!= generated_positions[1]
	):
		raise NativeCdmlRouteE2eError("save/reopen lost generated molecule coordinates")
	if inserted.digest != reopened.digest:
		raise NativeCdmlRouteE2eError("save/reopen changed the authoritative digest")
	if reopened.is_dirty:
		raise NativeCdmlRouteE2eError("reopened saved Rust document is unexpectedly dirty")
	host._close_current_tab()
	if host._native_tabs_by_page:
		raise NativeCdmlRouteE2eError("native host did not close and dispose the saved tab")
	host.close()
	app.processEvents()
	entry_exit = ferrum_qt.native.native_app.main([str(saved_path)], 0.05)
	if entry_exit != 0:
		raise NativeCdmlRouteE2eError("public native application entry returned %d" % entry_exit)
	return {
		"schema": "ferrum-native-cdml-route-e2e-v9",
		"revision": reopened.revision,
		"digest": reopened.digest,
		"clean": not reopened.is_dirty,
		"opaque_root": "payload-1" in output,
		"smiles_atoms": len(reopened_cco[0].atoms),
		"smiles_bonds": len(reopened_cco[0].bonds),
		"created_bond": created_bond_id,
		"changed_double_bond": created_bond_id,
		"generated_coordinates": "molecule-1",
		"extended_atom": extended_atom_id,
		"extended_bond": extended_bond_id,
		"moved_atom": extended_atom_id,
		"deleted_atom": "atom-c",
		"deleted_incident_bond": created_bond_id,
		"deleted_selected_bond": extended_bond_id,
		"native_entry_exit": entry_exit,
		"oasa_imported": any(
			name == "oasa" or name.startswith("oasa.") for name in sys.modules
		),
	}


#============================================
def main() -> int:
	"""Install the sealed wheel in an isolated environment and run the user path."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--wheel", type=pathlib.Path)
	parser.add_argument("--probe", action="store_true")
	arguments = parser.parse_args()
	if arguments.probe:
		print(json.dumps(_probe(), sort_keys=True))
		return 0
	if arguments.wheel is None or not arguments.wheel.is_file():
		raise NativeCdmlRouteE2eError("--wheel must name one installed-wheel artifact")
	environment = _proof_environment()
	with tempfile.TemporaryDirectory(prefix="ferrum-native-cdml-wheel-") as directory:
		venv = pathlib.Path(directory) / "venv"
		_run(sys.executable, "-B", "-m", "venv", "--system-site-packages", str(venv), environment=environment)
		python = venv / "bin" / "python"
		_run(str(python), "-B", "-m", "pip", "install", "--no-deps", str(arguments.wheel.resolve()), environment=environment)
		output = _run(str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe", environment=environment)
	value = json.loads(output)
	if value["oasa_imported"]:
		raise NativeCdmlRouteE2eError("native CDML controller imported OASA")
	if (
		not value["clean"]
		or not value["opaque_root"]
		or (value["smiles_atoms"], value["smiles_bonds"]) != (3, 2)
		):
		raise NativeCdmlRouteE2eError("native CDML controller did not preserve durable output truth")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
