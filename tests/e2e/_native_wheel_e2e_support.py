"""Private support operations for the native-wheel direct E2E runner."""

from __future__ import annotations

# Standard-library imports.
import ast
import base64
import hashlib
import importlib.machinery
import importlib.util
import json
import os
import re
import subprocess
import sys
import tempfile
import zipfile
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType


# Local builder imports must remain runtime-only so this direct E2E runner
# cannot create source-tree bytecode merely by being imported or inspected.
REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_TOOL = REPO_ROOT / "packages/ferrum-rust/tools/build_native_wheel.py"
CHEMISTRY_MANIFEST = REPO_ROOT / "packages/ferrum-rust/crates/chemistry/Cargo.toml"
CHEMISTRY_EXAMPLE = "native_smiles_fcm1"
AMBIENT_LIBRARY_VARIABLES = (
	"DYLD_LIBRARY_PATH",
	"DYLD_FALLBACK_LIBRARY_PATH",
	"DYLD_FRAMEWORK_PATH",
	"DYLD_FALLBACK_FRAMEWORK_PATH",
	"PYTHONPATH",
	"PYTHONHOME",
)


#============================================
# Build-contract loading


@dataclass(frozen=True)
class BuildContract:
	"""Machine protocol values exported by the current native build tool."""

	target: str
	result_schema: str
	adapter_abi_version: int


class E2eError(RuntimeError):
	"""An actionable native-wheel proof failure."""


#============================================
# Evidence helpers


#============================================
def sha256(path: Path) -> str:
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


#============================================
def read_build_receipt(output_root: Path) -> dict[str, object]:
	path = output_root / "native-wheel-build-receipt.json"
	if not path.is_file():
		raise E2eError(f"native wheel builder did not publish its receipt: {path}")
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise E2eError(f"native wheel builder receipt is invalid JSON: {error.msg}") from error
	if not isinstance(value, dict):
		raise E2eError("native wheel builder receipt must be a JSON object")
	return value


#============================================
def replacement_proof(
	original_sha256: str, replacement_sha256: str, installed_adapter: Path,
	expected_abi_version: int, replacement_build_type: str,
) -> dict[str, object]:
	"""Validate durable facts that establish the adapter replacement target."""
	for label, digest in (
		("original adapter", original_sha256),
		("replacement adapter", replacement_sha256),
	):
		if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
			raise E2eError(f"{label} SHA-256 is not a lowercase 64-hex digest: {digest!r}")
	if original_sha256 == replacement_sha256:
		raise E2eError(
			"the deliberately different replacement build produced the original adapter bytes"
		)
	if replacement_build_type != "RelWithDebInfo":
		raise E2eError(f"unexpected replacement build type: {replacement_build_type}")
	if installed_adapter.name != "libferrum_chem.dylib":
		raise E2eError(f"replacement target must be the installed adapter library: {installed_adapter}")
	return {
		"library": "libferrum_chem.dylib",
		"package_relative_path": ".dylibs/libferrum_chem.dylib",
		"abi_version": expected_abi_version,
		"original_sha256": original_sha256,
		"replacement_sha256": replacement_sha256,
		"replacement_build_type": replacement_build_type,
	}


#============================================
def native_evidence(
	builder_receipt: dict[str, object], wheel: Path, closure_names: list[str],
	before: dict[str, object], after: dict[str, object], original_sha256: str,
	replacement_sha256: str, installed_adapter: Path,
	expected_abi_version: int, python_chemistry_before: dict[str, object],
	python_chemistry_after: dict[str, object], rust_chemistry_before: dict[str, object],
	rust_chemistry_after: dict[str, object], replacement_build_type: str,
) -> dict[str, object]:
	"""Assemble the retained reproducibility record without retaining binaries."""
	return {
		"schema": "ferrum-native-wheel-e2e-evidence-v4",
		"builder_receipt": builder_receipt,
		"wheel": {"filename": wheel.name, "sha256": sha256(wheel)},
		"closure": {"names": closure_names},
		"probes": {
			"expected_abi_version": expected_abi_version,
			"before": before,
			"after": after,
		},
		"chemistry": {
			"python_extension_before": python_chemistry_before,
			"python_extension_after": python_chemistry_after,
			"rust_adapter_before": rust_chemistry_before,
			"rust_adapter_after": rust_chemistry_after,
		},
		"replacement_proof": replacement_proof(
			original_sha256, replacement_sha256, installed_adapter, expected_abi_version,
			replacement_build_type,
		),
		"process_boundary": {
			"python_probe": {
				"fresh_process": True,
				"scrubbed_loader": True,
				"direct_extension_chemistry": True,
			},
			"rust_semantic_probe": {
				"fresh_process": True,
				"explicit_adapter_path": True,
				"scrubbed_loader": True,
			},
		},
	}


#============================================
def publish_evidence(output_parent: Path, evidence: dict[str, object]) -> Path:
	"""Atomically replace the stable, ignored acceptance record after success."""
	directory = output_parent / "evidence"
	directory.mkdir(parents=True, exist_ok=True)
	target = directory / "native-wheel-e2e-receipt.json"
	with tempfile.NamedTemporaryFile(
		mode="w", encoding="utf-8", dir=directory,
		prefix=".native-wheel-e2e-receipt-", suffix=".json", delete=False,
	) as handle:
		handle.write(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
		temporary = Path(handle.name)
	temporary.replace(target)
	return target


#============================================
def run(
	*command: str,
	cwd: Path | None = None,
	env: dict[str, str] | None = None,
	stream_stderr: bool = False,
) -> str:
	"""Run a child while preserving its stdout machine protocol for this runner."""
	print("+", " ".join(command), file=sys.stderr)
	child_environment = scrubbed_environment() if env is None else env.copy()
	child_environment["PYTHONDONTWRITEBYTECODE"] = "1"
	result = subprocess.run(
		command,
		cwd=cwd,
		env=child_environment,
		text=True,
		stdout=subprocess.PIPE,
		stderr=None if stream_stderr else subprocess.PIPE,
		check=False,
	)
	if result.returncode:
		if stream_stderr:
			details = "child stderr was streamed above"
		else:
			details = result.stderr.strip()
		raise E2eError(
			f"command failed ({result.returncode}): {' '.join(command)}\n{details}"
		)
	return result.stdout


#============================================
def scrubbed_environment() -> dict[str, str]:
	environment = os.environ.copy()
	for name in AMBIENT_LIBRARY_VARIABLES:
		environment.pop(name, None)
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	return environment


#============================================
def document_probe(python: Path) -> dict[str, object]:
	"""Exercise Ferrum's public M9 document API in a clean wheel process."""
	output = run(
		# `-I` deliberately ignores environment variables, including
		# PYTHONDONTWRITEBYTECODE. Keep the clean-wheel probe isolated while
		# explicitly prohibiting bytecode output.
		str(python), "-I", "-B", "-c",
		"import importlib.machinery, json, sys, pathlib, ferrum_chem; "
		"source='<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom></molecule></cdml>'; "
		"session=ferrum_chem.DocumentSession.load(source); initial=session.snapshot(); fresh=(session.can_undo, session.can_redo); "
		"changed=session.submit(initial.revision, ferrum_chem.DocumentOperationV1.set_atom_element('a', 'N')).observation.snapshot; committed=(session.can_undo, session.can_redo); "
		"undone=session.undo(changed.revision).observation.snapshot; undone_history=(session.can_undo, session.can_redo); "
		"redone=session.redo(undone.revision).observation.snapshot; redone_history=(session.can_undo, session.can_redo); "
		"path=pathlib.Path(sys.prefix)/'saved.cdml'; saved=session.save_atomic(path, redone.revision); "
		"print(json.dumps({'revision': saved.snapshot.revision, 'dirty': saved.snapshot.is_dirty, 'outcome': saved.outcome.is_confirmed, 'saved': path.read_text()==saved.snapshot.cdml, 'history': [list(fresh), list(committed), list(undone_history), list(redone_history)], 'history_types': [type(value) is bool for facts in (fresh, committed, undone_history, redone_history) for value in facts], 'native_file': ferrum_chem.__file__.endswith(tuple(importlib.machinery.EXTENSION_SUFFIXES)), 'extension_loader': isinstance(ferrum_chem.__spec__.loader, importlib.machinery.ExtensionFileLoader), 'package_shim': hasattr(ferrum_chem, '__path__'), 'bindings_alias': hasattr(ferrum_chem, '_bindings')}))",
		env=scrubbed_environment(),
	)
	value = json.loads(output)
	if value != {
		"revision": 3, "dirty": False, "outcome": True, "saved": True,
		"history": [[False, False], [True, False], [False, True], [True, False]],
		"history_types": [True] * 8,
		"native_file": True, "extension_loader": True, "package_shim": False,
		"bindings_alias": False,
	}:
		raise E2eError(f"public document probe returned an invalid value: {value!r}")
	return value


#============================================
def parse_json_object(output: str, name: str) -> dict[str, object]:
	"""Accept the one-object machine protocol emitted by a semantic probe."""
	lines = output.splitlines()
	if len(lines) != 1:
		raise E2eError(f"{name} must emit exactly one JSON object, got {len(lines)} lines")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise E2eError(f"{name} did not emit valid JSON: {error.msg}") from error
	if not isinstance(value, dict):
		raise E2eError(f"{name} must emit a JSON object: {value!r}")
	return value


#============================================
def expected_semantic_atoms() -> list[dict[str, object]]:
	"""Return the authored facts the native engine must preserve exactly."""
	return [
		{
			"atomic_number": 6,
			"aromatic": True,
			"formal_charge": 0,
			"isotope": None,
			"explicit_hydrogens": None,
		},
		{
			"atomic_number": 6,
			"aromatic": True,
			"formal_charge": None,
			"isotope": 13,
			"explicit_hydrogens": None,
		},
		{
			"atomic_number": 6,
			"aromatic": True,
			"formal_charge": None,
			"isotope": None,
			"explicit_hydrogens": 1,
		},
		*[
			{
				"atomic_number": 6,
				"aromatic": True,
				"formal_charge": None,
				"isotope": None,
				"explicit_hydrogens": None,
			}
			for _ in range(3)
		],
	]


#============================================
def expected_semantic_bonds(order_names: list[str]) -> list[dict[str, object]]:
	"""Return ordered benzene bonds for the Rust semantic-proof protocol."""
	return [
		{
			"start": index,
			"end": (index + 1) % 6,
			"order": order,
			"aromatic": True,
		}
		for index, order in enumerate(order_names)
	]


#============================================
def semantic_probe_fixture(abi_version: int) -> dict[str, object]:
	"""Build a pure-Python fixture for the ABI-4 FCM1 semantic self-test."""
	return {
		"abi_version": abi_version,
		"canonical_smiles": "CCO",
		"atom_count": 3,
		"bond_count": 2,
		"coordinate_count": 3,
	}


#============================================
def assert_fcm1_probe(value: dict[str, object], expected_abi_version: int) -> None:
	"""Require canonical SMILES and an atom-aligned complete FCM1 molecule."""
	expected = semantic_probe_fixture(expected_abi_version)
	if value != expected:
		raise E2eError(f"FCM1 semantic probe returned an invalid value: {value!r}")


#============================================
def assert_semantic_probe(value: dict[str, object], expected_abi_version: int) -> None:
	"""Prove default Rust-engine Kekulization rather than only loader identity."""
	if value.get("abi_version") != expected_abi_version:
		raise E2eError(f"semantic probe reported wrong ABI: {value!r}")
	input_graph = value.get("input")
	output_graph = value.get("output")
	if not isinstance(input_graph, dict) or not isinstance(output_graph, dict):
		raise E2eError(f"semantic probe omitted input or output graphs: {value!r}")
	expected_atoms = expected_semantic_atoms()
	if input_graph.get("atoms") != expected_atoms or output_graph.get("atoms") != expected_atoms:
		raise E2eError(
			"semantic probe did not preserve exact optional charge, isotope, and hydrogen facts"
		)
	input_bonds = input_graph.get("bonds")
	output_bonds = output_graph.get("bonds")
	if not isinstance(input_bonds, list) or not isinstance(output_bonds, list):
		raise E2eError("semantic probe graphs must contain bond arrays")
	if len(input_bonds) != 6 or len(output_bonds) != 6:
		raise E2eError("semantic probe did not retain benzene topology")
	for index, bond in enumerate(input_bonds):
		if bond != expected_semantic_bonds(["aromatic"] * 6)[index]:
			raise E2eError(f"semantic probe input bond {index} is not authored aromatic benzene")
	orders: list[str] = []
	for index, bond in enumerate(output_bonds):
		if not isinstance(bond, dict):
			raise E2eError(f"semantic probe output bond {index} is not an object")
		if bond.get("start") != index or bond.get("end") != (index + 1) % 6:
			raise E2eError(f"semantic probe changed output bond endpoints at index {index}")
		if bond.get("aromatic") is not True:
			raise E2eError(f"semantic probe default unexpectedly cleared aromaticity at bond {index}")
		order = bond.get("order")
		if order not in {"single", "double"}:
			raise E2eError(f"semantic probe did not assign a Kekule order at bond {index}: {order!r}")
		orders.append(order)
	if orders.count("single") != 3 or orders.count("double") != 3:
		message = "semantic probe result is not a three-single/three-double Kekule form"
		raise E2eError(f"{message}: {orders!r}")
	if any(orders[index] == orders[(index + 1) % len(orders)] for index in range(len(orders))):
		raise E2eError(f"semantic probe result does not alternate around benzene: {orders!r}")


#============================================
def direct_python_chemistry_probe(python: Path) -> dict[str, object]:
	"""Prove the installed direct extension owns ABI-4 SMILES parsing."""
	output = run(
		str(python), "-I", "-B", "-c",
		"import importlib.machinery, json, math, pathlib, ferrum_chem; "
		"molecule=ferrum_chem.parse_smiles('CCO'); atoms=molecule.atoms; bonds=molecule.bonds; "
		"standard_inchi=ferrum_chem.molecule_to_inchi(molecule, ferrum_chem.InchiModeV1.standard); "
		"fixed_inchi=ferrum_chem.molecule_to_inchi(molecule, ferrum_chem.InchiModeV1.fixed_hydrogen); "
		"inchi_molecule=ferrum_chem.parse_inchi(standard_inchi); "
		"v2000=ferrum_chem.molecule_to_molblock(molecule, ferrum_chem.MolblockVersionV1.v2000); "
		"v3000=ferrum_chem.molecule_to_molblock(molecule, ferrum_chem.MolblockVersionV1.v3000); "
		"imported_v2000=ferrum_chem.molblock_to_molecule(v2000); imported_v3000=ferrum_chem.molblock_to_molecule(v3000); "
		"record=ferrum_chem.prepare_sdf_record(molecule, 'ethanol', (('source', 'Ferrum'),)); "
		"sdf=ferrum_chem.records_to_sdf((record,), ferrum_chem.MolblockVersionV1.v2000); "
		"imported=ferrum_chem.sdf_to_records(sdf); "
		"print(json.dumps({'module_origin': pathlib.Path(ferrum_chem.__file__).name, "
		"'module_is_direct_extension': ferrum_chem.__file__.endswith(tuple(importlib.machinery.EXTENSION_SUFFIXES)), "
		"'canonical_smiles': molecule.canonical_smiles, "
		"'inchi': {'standard': standard_inchi, 'fixed_is_nonstandard': fixed_inchi.startswith('InChI=1/'), "
		"'key': ferrum_chem.inchi_to_inchi_key(standard_inchi), "
		"'round_trip_smiles': inchi_molecule.canonical_smiles}, "
		"'smarts': ferrum_chem.molecule_to_smarts(molecule), 'atom_count': len(atoms), "
		"'molblock_versions_explicit': 'V2000' in v2000 and 'M  V30 BEGIN CTAB' not in v2000 and 'V3000' in v3000 and 'M  V30 BEGIN CTAB' in v3000, "
		"'molblocks_newline_terminated': v2000.endswith('\\n') and v3000.endswith('\\n'), "
		"'molblock_import_semantics': imported_v2000.canonical_smiles == 'CCO' and imported_v3000.canonical_smiles == 'CCO' and len(imported_v2000.coordinates) == 3 and len(imported_v3000.coordinates) == 3, "
		"'sdf_record_semantic_markers': sdf.startswith('ethanol\\n') and '<source>' in sdf and '\\nFerrum\\n' in sdf and sdf.endswith('$$$$\\n'), "
		"'sdf_import_semantics': len(imported) == 1 and imported[0].title == 'ethanol' and imported[0].molecule.canonical_smiles == 'CCO' and tuple((item.name, item.value) for item in imported[0].properties) == (('source', 'Ferrum'),), "
		"'bond_count': len(bonds), 'coordinate_count': len(molecule.coordinates), "
		"'coordinates_are_finite': all(math.isfinite(point.x) and math.isfinite(point.y) for point in molecule.coordinates), "
		"'coordinates_are_distinct': len({(point.x, point.y) for point in molecule.coordinates}) == len(molecule.coordinates), "
		"'atom_facts': [{'atomic_number': atom.atomic_number, 'aromatic': atom.aromatic, "
		"'formal_charge': atom.formal_charge, 'isotope': atom.isotope, 'explicit_hydrogens': atom.explicit_hydrogens, "
		"'radical_electrons': atom.radical_electrons, 'no_implicit': atom.no_implicit, "
		"'atom_map_number': atom.atom_map_number} for atom in atoms], "
		"'closed_enums': {'atom_chirality_unspecified': all(type(atom.chirality) is ferrum_chem.SmilesAtomChiralityV1 and atom.chirality == ferrum_chem.SmilesAtomChiralityV1.unspecified for atom in atoms), "
		"'bond_order_single': all(type(bond.order) is ferrum_chem.SmilesBondOrderV1 and bond.order == ferrum_chem.SmilesBondOrderV1.single for bond in bonds), "
		"'bond_stereo_none': all(type(bond.stereo) is ferrum_chem.SmilesBondStereoV1 and bond.stereo == ferrum_chem.SmilesBondStereoV1.none for bond in bonds), "
		"'bond_direction_none': all(type(bond.direction) is ferrum_chem.SmilesBondDirectionV1 and bond.direction == ferrum_chem.SmilesBondDirectionV1.none for bond in bonds)}}))",
		env=scrubbed_environment(),
	)
	value = parse_json_object(output, "direct Python chemistry probe")
	assert_direct_python_chemistry_probe(value)
	return value


#============================================
def direct_python_chemistry_fixture() -> dict[str, object]:
	"""Return fixed ABI-4 facts expected from the installed CCO DTO."""
	return {
		"module_is_direct_extension": True,
		"canonical_smiles": "CCO",
		"inchi": {
			"standard": "InChI=1S/C2H6O/c1-2-3/h3H,2H2,1H3",
			"fixed_is_nonstandard": True,
			"key": "LFQSCWFLJHTTHZ-UHFFFAOYSA-N",
			"round_trip_smiles": "CCO",
		},
		"smarts": "[#6]-[#6]-[#8]",
		"molblock_versions_explicit": True,
		"molblocks_newline_terminated": True,
		"molblock_import_semantics": True,
		"sdf_record_semantic_markers": True,
		"sdf_import_semantics": True,
		"atom_count": 3,
		"bond_count": 2,
		"coordinate_count": 3,
		"coordinates_are_finite": True,
		"coordinates_are_distinct": True,
		"atom_facts": [
			{
				"atomic_number": 6,
				"aromatic": False,
				"formal_charge": 0,
				"isotope": None,
				"explicit_hydrogens": 0,
				"radical_electrons": 0,
				"no_implicit": False,
				"atom_map_number": None,
			},
			{
				"atomic_number": 6,
				"aromatic": False,
				"formal_charge": 0,
				"isotope": None,
				"explicit_hydrogens": 0,
				"radical_electrons": 0,
				"no_implicit": False,
				"atom_map_number": None,
			},
			{
				"atomic_number": 8,
				"aromatic": False,
				"formal_charge": 0,
				"isotope": None,
				"explicit_hydrogens": 0,
				"radical_electrons": 0,
				"no_implicit": False,
				"atom_map_number": None,
			},
		],
		"closed_enums": {
			"atom_chirality_unspecified": True,
			"bond_order_single": True,
			"bond_stereo_none": True,
			"bond_direction_none": True,
		},
	}


#============================================
def assert_direct_python_chemistry_probe(value: dict[str, object]) -> None:
	"""Require direct extension SMILES facts and a direct extension origin."""
	module_origin = value.pop("module_origin", None)
	if (
		not isinstance(module_origin, str)
		or not re.fullmatch(r"ferrum_chem[^/]*\.so", module_origin)
	):
		raise E2eError(f"direct Python chemistry probe has an invalid module origin: {module_origin!r}")
	expected = direct_python_chemistry_fixture()
	if value != expected:
		raise E2eError(f"direct Python chemistry probe returned an invalid DTO: {value!r}")
	value["module_origin"] = module_origin


#============================================
def rust_chemistry_probe(
	adapter: Path, output_root: Path, expected_abi_version: int,
) -> dict[str, object]:
	"""Run the safe Rust engine in a fresh, loader-scrubbed process."""
	if not adapter.is_absolute() or not adapter.is_file():
		raise E2eError(f"semantic probe adapter must be an absolute library path: {adapter}")
	environment = scrubbed_environment()
	environment["CARGO_TARGET_DIR"] = str(output_root / "rust-e2e-target")
	value = parse_json_object(
		run(
			"cargo",
			"run",
			"--quiet",
			"--manifest-path",
			str(CHEMISTRY_MANIFEST),
			"--example",
			CHEMISTRY_EXAMPLE,
			"--",
			"--adapter",
			str(adapter),
			env=environment,
		),
		"Rust native chemistry semantic probe",
	)
	assert_fcm1_probe(value, expected_abi_version)
	return value


#============================================
def load_build_tool() -> ModuleType:
	# The tracked builder is dynamically imported for closure validation.  Keep
	# that read-only operation from generating a source-tree __pycache__ even if
	# the caller forgot the environment setting.
	original_dont_write_bytecode = sys.dont_write_bytecode
	original_path = sys.path[:]
	try:
		sys.dont_write_bytecode = True
		sys.path.insert(0, str(BUILD_TOOL.parent.resolve()))
		specification = importlib.util.spec_from_file_location(
			"ferrum_native_wheel_build", BUILD_TOOL,
		)
		if specification is None or specification.loader is None:
			raise E2eError(f"could not load native wheel closure policy from {BUILD_TOOL}")
		module = importlib.util.module_from_spec(specification)
		sys.modules[specification.name] = module
		specification.loader.exec_module(module)
		return module
	finally:
		sys.dont_write_bytecode = original_dont_write_bytecode
		sys.path[:] = original_path


#============================================
# Builder protocol validation


#============================================
def build_contract() -> BuildContract:
	"""Read the native builder's current, typed machine-protocol authority."""
	module = load_build_tool()
	values = {
		"target": getattr(module, "TARGET", None),
		"result_schema": getattr(module, "MACHINE_RESULT_SCHEMA", None),
		"adapter_abi_version": getattr(module, "ADAPTER_ABI_VERSION", None),
	}
	if (
		not isinstance(values["target"], str)
		or not values["target"]
		or not isinstance(values["result_schema"], str)
		or not values["result_schema"]
		or type(values["adapter_abi_version"]) is not int
		or values["adapter_abi_version"] < 1
	):
		raise E2eError(f"native build tool exports an invalid machine contract: {values!r}")
	return BuildContract(
		target=values["target"],
		result_schema=values["result_schema"],
		adapter_abi_version=values["adapter_abi_version"],
	)


#============================================
def parse_artifact_result(
	stdout: str, action: str, output_root: Path, contract: BuildContract,
) -> Path:
	"""Accept exactly one builder JSON artifact record and no progress on stdout."""
	lines = stdout.splitlines()
	if len(lines) != 1:
		raise E2eError(f"builder {action} result must be exactly one JSON line, got {len(lines)} lines")
	try:
		result = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise E2eError(f"builder {action} result is not valid JSON: {error.msg}") from error
	if not isinstance(result, dict):
		raise E2eError(f"builder {action} result must be a JSON object: {result!r}")
	for field in ("schema", "action", "artifact"):
		if field not in result:
			raise E2eError(f"builder {action} result omits required field {field!r}: {result!r}")
	if not isinstance(result["schema"], str) or not isinstance(result["action"], str):
		raise E2eError(f"builder {action} result has non-string protocol fields: {result!r}")
	if not isinstance(result["artifact"], str):
		raise E2eError(f"builder {action} artifact is not a path string: {result!r}")
	if result["schema"] != contract.result_schema or result["action"] != action:
		raise E2eError(f"builder {action} result has the wrong schema or action: {result!r}")
	artifact = Path(result["artifact"])
	if not output_root.is_dir():
		raise E2eError(f"builder {action} output root does not exist: {output_root}")
	if not artifact.is_absolute():
		raise E2eError(f"builder {action} artifact must be an absolute normalized path: {artifact}")
	try:
		normalized_artifact = artifact.resolve(strict=True)
	except FileNotFoundError as error:
		raise E2eError(f"builder {action} reported a missing artifact: {artifact}") from error
	if artifact != normalized_artifact:
		raise E2eError(f"builder {action} artifact must be an absolute normalized path: {artifact}")
	if (
		not normalized_artifact.is_relative_to(output_root.resolve())
		or not normalized_artifact.is_file()
	):
		raise E2eError(f"builder {action} reported a missing or out-of-root artifact: {artifact}")
	return normalized_artifact


#============================================
def assert_wheel_closure(site_packages: Path) -> Path:
	extensions = list(site_packages.glob("ferrum_chem*.so"))
	if len(extensions) != 1:
		raise E2eError(f"expected one native extension in {site_packages}, found {extensions}")
	libs = site_packages / ".dylibs"
	if not (libs / "libferrum_chem.dylib").is_file():
		raise E2eError("wheel does not contain separately replaceable .dylibs/libferrum_chem.dylib")
	try:
		load_build_tool().assert_clean_closure(extensions[0], libs)
	except RuntimeError as error:
		raise E2eError(f"installed wheel fails the native loader closure policy: {error}") from error
	return libs


#============================================
def assert_shipped_typing_metadata(site_packages: Path, python: Path) -> None:
	"""Require the installed direct extension to expose every stubbed class."""
	stub = site_packages / "ferrum_chem.pyi"
	if not stub.is_file() or not (site_packages / "py.typed").is_file():
		raise E2eError("shipping wheel omitted root typing metadata")
	classes = [
		node.name for node in ast.parse(stub.read_text(encoding="utf-8")).body
		if isinstance(node, ast.ClassDef) and not node.name.startswith("_")
	]
	runtime_names = json.loads(run(
		str(python), "-I", "-B", "-c",
		"import json, ferrum_chem; print(json.dumps(sorted(dir(ferrum_chem))))",
		env=scrubbed_environment(),
	))
	if not isinstance(runtime_names, list) or not all(isinstance(name, str) for name in runtime_names):
		raise E2eError("native extension did not report its public names")
	missing = [name for name in classes if name not in runtime_names]
	if missing:
		raise E2eError(f"stubbed public classes are missing from the native extension: {missing}")
	document_session = next(
		(node for node in ast.parse(stub.read_text(encoding="utf-8")).body
		if isinstance(node, ast.ClassDef) and node.name == "DocumentSession"),
		None,
	)
	if document_session is None:
		raise E2eError("shipping wheel typing metadata omitted DocumentSession")
	properties = {
		node.target.id: ast.unparse(node.annotation)
		for node in document_session.body
		if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name)
	}
	if {name: properties.get(name) for name in ("can_undo", "can_redo")} != {
		"can_undo": "bool", "can_redo": "bool",
	}:
		raise E2eError("shipping wheel stub lacks bool DocumentSession history properties")


#============================================
def assert_shipped_wheel_members(wheel: Path) -> None:
	"""Require final wheel contents and RECORD to describe one direct extension."""
	with zipfile.ZipFile(wheel) as archive:
		members = archive.namelist()
		if any(name.startswith("ferrum_chem/") for name in members):
			raise E2eError("shipping wheel retained a nested ferrum_chem package")
		extensions = [name for name in members if re.fullmatch(r"ferrum_chem[^/]*\.so", name)]
		if (
			len(extensions) != 1
			or "ferrum_chem.pyi" not in members
			or "py.typed" not in members
			or "ferrum-operation-v1.schema.json" not in members
		):
			raise E2eError(f"shipping wheel lacks direct extension or typing metadata: {members}")
		try:
			schema = json.loads(archive.read("ferrum-operation-v1.schema.json"))
		except json.JSONDecodeError as error:
			raise E2eError(f"shipping wheel has an invalid operation protocol schema: {error}") from error
		if not isinstance(schema, dict) or not isinstance(schema.get("x-ferrum-roots"), dict):
			raise E2eError("shipping wheel lacks the operation protocol schema roots")
		allowed = {
			*extensions, "ferrum_chem.pyi", "py.typed", "ferrum-operation-v1.schema.json",
			*(name for name in members if ".dist-info/" in name),
			*(f".dylibs/{name}" for name in load_build_tool().MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names),
		}
		unexpected = sorted(set(members).difference(allowed))
		if unexpected:
			raise E2eError(f"shipping wheel contains unexpected members: {unexpected}")
		record = next((name for name in members if name.endswith(".dist-info/RECORD")), None)
		if record is None:
			raise E2eError("shipping wheel omitted RECORD")
		records = [line.rsplit(",", 2) for line in archive.read(record).decode().splitlines()]
		if any(len(entry) != 3 for entry in records) or len({entry[0] for entry in records}) != len(records):
			raise E2eError("shipping wheel RECORD has malformed or duplicate paths")
		expected = set(members)
		if {entry[0] for entry in records} != expected:
			raise E2eError("shipping wheel RECORD does not enumerate every member")
		for name, digest, size in records:
			if name == record:
				if digest or size:
					raise E2eError("shipping wheel RECORD hashes itself")
				continue
			actual = base64.urlsafe_b64encode(hashlib.sha256(archive.read(name)).digest()).rstrip(b"=").decode()
			if digest != f"sha256={actual}" or size != str(len(archive.read(name))):
				raise E2eError(f"shipping wheel RECORD is wrong for {name}")


#============================================
def installed_site_packages(venv: Path) -> Path:
	paths = list((venv / "lib").glob("python*/site-packages"))
	if len(paths) != 1:
		raise E2eError(f"could not identify isolated venv site-packages under {venv}")
	return paths[0]

