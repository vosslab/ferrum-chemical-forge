"""Measure semantic V2000/V3000 parity through isolated RDKit processes."""

# Standard Library
import argparse
import hashlib
import json
import math
import os
import pathlib
import platform
import subprocess


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
FERRUM_CHILD = REPO_ROOT / "devel" / "molblock_parity_ferrum_child.py"
RDKIT_CHILD = REPO_ROOT / "devel" / "molblock_parity_rdkit_child.py"
DEFAULT_REPORT = REPO_ROOT / "docs" / "active_plans" / "reports" / "molblock_codec_v1.json"
EXPECTED_RDKIT_VERSION = "2026.03.5"
CORPUS = (
	{"name": "ethanol", "smiles": "CCO"},
	{"name": "charged_pair", "smiles": "[NH4+].[Cl-]"},
	{"name": "nitrile", "smiles": "C#N"},
	{"name": "benzene", "smiles": "c1ccccc1"},
	{"name": "bond_stereo", "smiles": "F/C=C/F"},
	{
		"name": "isotope_chirality_map",
		"smiles": "[13CH3][C@H](F)[C:9](=O)[O-]",
	},
	{"name": "explicit_methylene", "smiles": "[CH2]"},
)
SOURCE_PATHS = (
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_molblock.cpp",
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_molblock_import.cpp",
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_smarts.cpp",
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_text_response.cpp",
	"packages/ferrum-rust/crates/chemistry/native/include/ferrum_chem_adapter.h",
	"packages/ferrum-rust/crates/chemistry/src/native_engine/graph_wire.rs",
	"packages/ferrum-rust/crates/chemistry/src/native_engine/molblock_wire.rs",
	"packages/ferrum-rust/crates/chemistry/src/native_engine/molblock_import.rs",
	"packages/ferrum-rust/crates/chemistry/src/native_engine/text_response.rs",
	"packages/ferrum-rust/crates/api/python/src/chemistry_binding.rs",
	"packages/ferrum-rust/crates/api/src/molblock_inspection.rs",
	"devel/measure_molblock_codec_parity.py",
	"devel/molblock_parity_ferrum_child.py",
	"devel/molblock_parity_rdkit_child.py",
)


#============================================
class MolblockParityError(RuntimeError):
	"""The molblock differential protocol or result is invalid."""


#============================================
def _sha256(path: pathlib.Path) -> str:
	"""Return the digest of one required regular file."""
	if not path.is_file():
		raise MolblockParityError("required parity input is not a regular file: " + str(path))
	return hashlib.sha256(path.read_bytes()).hexdigest()


#============================================
def _display_path(path: pathlib.Path) -> str:
	"""Return a repository-relative path when possible."""
	resolved = path.resolve()
	try:
		return str(resolved.relative_to(REPO_ROOT))
	except ValueError:
		return str(resolved)


#============================================
def _run_child(python: pathlib.Path, child: pathlib.Path, request: dict) -> dict:
	"""Run one isolated child and require exactly one JSON object."""
	environment = os.environ.copy()
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	result = subprocess.run(
		[str(python), "-I", "-B", str(child)],
		cwd=REPO_ROOT,
		env=environment,
		input=json.dumps(request, allow_nan=False, separators=(",", ":"), sort_keys=True) + "\n",
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)
	if result.returncode:
		raise MolblockParityError(child.name + " failed: " + result.stderr.strip())
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise MolblockParityError(child.name + " must emit exactly one JSON line")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise MolblockParityError(child.name + " emitted invalid JSON") from error
	if not isinstance(value, dict):
		raise MolblockParityError(child.name + " output must be a JSON object")
	return value


#============================================
def _valid_coordinates(value: object, atom_count: int, backend: str) -> list[list[float]]:
	"""Validate one complete finite atom-aligned coordinate sequence."""
	if not isinstance(value, list) or len(value) != atom_count:
		raise MolblockParityError(backend + " returned incomplete coordinates")
	points = []
	for point in value:
		if not isinstance(point, list) or len(point) != 2:
			raise MolblockParityError(backend + " returned a malformed point")
		if any(type(component) not in (int, float) for component in point):
			raise MolblockParityError(backend + " returned a nonnumeric point")
		coordinates = [float(component) for component in point]
		if not all(math.isfinite(component) for component in coordinates):
			raise MolblockParityError(backend + " returned a nonfinite point")
		points.append(coordinates)
	return points


#============================================
def _validate_ferrum(value: dict) -> dict:
	"""Validate installed-extension output before passing text to RDKit."""
	if value.get("schema") != "ferrum-molblock-parity-ferrum-v1":
		raise MolblockParityError("Ferrum returned an unknown schema")
	if value.get("backend") != "ferrum-abi4" or value.get("version") != 4:
		raise MolblockParityError("Ferrum returned an invalid backend identity")
	binary = value.get("binary")
	if type(binary) is not str:
		raise MolblockParityError("Ferrum omitted its extension path")
	molecules = value.get("molecules")
	if not isinstance(molecules, list) or len(molecules) != len(CORPUS):
		raise MolblockParityError("Ferrum returned the wrong corpus size")
	validated = []
	for expected, molecule in zip(CORPUS, molecules, strict=True):
		if not isinstance(molecule, dict) or molecule.get("name") != expected["name"]:
			raise MolblockParityError("Ferrum changed corpus order or identity")
		canonical = molecule.get("canonical_smiles")
		blocks = molecule.get("molblocks")
		imports = molecule.get("imports")
		if (
			type(canonical) is not str or not canonical
			or not isinstance(blocks, dict) or not isinstance(imports, dict)
		):
			raise MolblockParityError("Ferrum returned incomplete molecule output")
		if any(type(blocks.get(version)) is not str for version in ("v2000", "v3000")):
			raise MolblockParityError("Ferrum omitted an explicit molblock version")
		atom_count = len(molecule.get("coordinates", []))
		if atom_count < 1:
			raise MolblockParityError("Ferrum returned no coordinates")
		validated_imports = {}
		for version in ("v2000", "v3000"):
			imported = imports.get(version)
			if not isinstance(imported, dict):
				raise MolblockParityError("Ferrum omitted its " + version + " import")
			if imported.get("canonical_smiles") != canonical:
				raise MolblockParityError("Ferrum " + version + " import changed molecule meaning")
			if imported.get("atom_count") != atom_count:
				raise MolblockParityError("Ferrum " + version + " import changed atom count")
			bond_count = imported.get("bond_count")
			if type(bond_count) is not int or bond_count < 0:
				raise MolblockParityError("Ferrum " + version + " import has invalid bonds")
			validated_imports[version] = {
				"atom_count": atom_count,
				"bond_count": bond_count,
				"canonical_smiles": canonical,
				"coordinates": _valid_coordinates(
					imported.get("coordinates"), atom_count, "Ferrum " + version + " import",
				),
			}
		if validated_imports["v2000"]["bond_count"] != validated_imports["v3000"]["bond_count"]:
			raise MolblockParityError("Ferrum molblock versions changed bond count")
		validated.append({
			"canonical_smiles": canonical,
			"coordinates": _valid_coordinates(
				molecule["coordinates"], atom_count, "Ferrum",
			),
			"imports": validated_imports,
			"molblocks": {version: blocks[version] for version in ("v2000", "v3000")},
			"name": expected["name"],
		})
	return {
		"binary_sha256": _sha256(pathlib.Path(binary)),
		"molecules": validated,
	}


#============================================
def _evaluator_request(ferrum: dict) -> dict:
	"""Bind Ferrum's generated text to the closed source corpus."""
	molecules = []
	for source, output in zip(CORPUS, ferrum["molecules"], strict=True):
		molecules.append({
			"coordinates": output["coordinates"],
			"molblocks": output["molblocks"],
			"name": source["name"],
			"smiles": source["smiles"],
		})
	return {
		"molecules": molecules,
		"schema": "ferrum-molblock-evaluation-request-v1",
	}


#============================================
def _validate_coordinate_evidence(value: object, backend: str) -> dict:
	"""Validate one token-derived coordinate-rounding result."""
	if not isinstance(value, dict) or value.get("passed") is not True:
		raise MolblockParityError(backend + " failed coordinate preservation")
	delta = value.get("observed_max_abs_delta")
	bound = value.get("derived_max_abs_bound")
	if type(delta) not in (int, float) or type(bound) not in (int, float):
		raise MolblockParityError(backend + " omitted coordinate evidence")
	if not math.isfinite(delta) or not math.isfinite(bound) or delta < 0 or bound < 0:
		raise MolblockParityError(backend + " returned invalid coordinate evidence")
	if delta > bound:
		raise MolblockParityError(backend + " exceeds its derived coordinate bound")
	return {
		"derived_max_abs_bound": float(bound),
		"observed_max_abs_delta": float(delta),
	}


#============================================
def _validate_evaluation(value: dict, version: str, ferrum: dict, label: str) -> dict:
	"""Require semantic round trips without comparing variable text bytes."""
	if value.get("schema") != "ferrum-molblock-evaluation-v1":
		raise MolblockParityError(label + " returned an unknown schema")
	if value.get("backend") != "rdkit-python-wrapper":
		raise MolblockParityError(label + " returned an unexpected backend")
	if value.get("rdkit_version") != version:
		raise MolblockParityError(label + " returned RDKit " + str(value.get("rdkit_version")))
	digest = value.get("binary_sha256")
	if type(digest) is not str or len(digest) != 64:
		raise MolblockParityError(label + " omitted its binary digest")
	molecules = value.get("molecules")
	if not isinstance(molecules, list) or len(molecules) != len(CORPUS):
		raise MolblockParityError(label + " returned the wrong corpus size")
	rows = []
	for source, ferrum_record, actual in zip(
		CORPUS, ferrum["molecules"], molecules, strict=True,
	):
		if not isinstance(actual, dict) or actual.get("name") != source["name"]:
			raise MolblockParityError(label + " changed corpus order")
		source_semantic = actual.get("source_semantic")
		formats = actual.get("formats")
		if not isinstance(source_semantic, dict) or not isinstance(formats, dict):
			raise MolblockParityError(label + " omitted semantic evidence")
		format_rows = {}
		for molblock_version in ("v2000", "v3000"):
			comparison = formats.get(molblock_version)
			if not isinstance(comparison, dict):
				raise MolblockParityError(label + " omitted " + molblock_version)
			ferrum_block = comparison.get("ferrum")
			oracle_block = comparison.get("oracle")
			if not isinstance(ferrum_block, dict) or not isinstance(oracle_block, dict):
				raise MolblockParityError(label + " omitted parsed block evidence")
			if ferrum_block.get("semantic") != source_semantic:
				raise MolblockParityError(
					label + " Ferrum " + molblock_version + " changed molecule meaning",
				)
			if oracle_block.get("semantic") != source_semantic:
				raise MolblockParityError(
					label + " oracle " + molblock_version + " changed molecule meaning",
				)
			format_rows[molblock_version] = {
				"ferrum_coordinates": _validate_coordinate_evidence(
					ferrum_block.get("coordinates"), label + " Ferrum " + molblock_version,
				),
				"ferrum_import_round_trip": True,
				"oracle_coordinates": _validate_coordinate_evidence(
					oracle_block.get("coordinates"), label + " oracle " + molblock_version,
				),
				"semantic_round_trip": True,
				"text_exact_observation": comparison.get("text_exact_observation") is True,
			}
		oracle_coordinates = _valid_coordinates(
			actual.get("oracle_coordinates"), len(ferrum_record["coordinates"]), label,
		)
		rows.append({
			"canonical_smiles": source_semantic.get("canonical_smiles"),
			"formats": format_rows,
			"name": source["name"],
			"oracle_coordinates": oracle_coordinates,
		})
	return {
		"binary_sha256": digest,
		"molecules": rows,
		"rdkit_version": version,
	}


#============================================
def _coordinate_receipt(path: pathlib.Path, wheel_sha256: str) -> dict:
	"""Load the measured raw-coordinate tolerance for this exact wheel."""
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except (OSError, json.JSONDecodeError) as error:
		raise MolblockParityError("coordinate receipt is unreadable") from error
	if value.get("schema") != "ferrum-coordinate-parity-v1" or value.get("status") != "measured":
		raise MolblockParityError("coordinate receipt is not accepted measurement evidence")
	if value.get("rdkit_version") != EXPECTED_RDKIT_VERSION:
		raise MolblockParityError("coordinate receipt uses a different RDKit version")
	artifacts = value.get("artifacts")
	measurement = value.get("measurement")
	if not isinstance(artifacts, dict) or not isinstance(measurement, dict):
		raise MolblockParityError("coordinate receipt omits required evidence")
	if artifacts.get("wheel_sha256") != wheel_sha256:
		raise MolblockParityError("coordinate receipt describes a different wheel")
	tolerance = measurement.get("tolerance_max_abs")
	if type(tolerance) not in (int, float) or not math.isfinite(tolerance) or tolerance < 0:
		raise MolblockParityError("coordinate receipt has an invalid tolerance")
	return {
		"raw_coordinate_tolerance_max_abs": float(tolerance),
		"receipt": _display_path(path),
	}


#============================================
def _maximum_raw_coordinate_delta(ferrum: dict, oracle: dict) -> float:
	"""Compare pre-writer coordinates in exact atom order."""
	maximum = 0.0
	for left, right in zip(ferrum["molecules"], oracle["molecules"], strict=True):
		for left_point, right_point in zip(
			left["coordinates"], right["oracle_coordinates"], strict=True,
		):
			for left_value, right_value in zip(left_point, right_point, strict=True):
				maximum = max(maximum, abs(left_value - right_value))
	return maximum


#============================================
def _native_e2e(path: pathlib.Path, wheel_sha256: str) -> dict:
	"""Require the operation before and after a distinct packaged-adapter rebuild."""
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except (OSError, json.JSONDecodeError) as error:
		raise MolblockParityError("native E2E receipt is unreadable") from error
	if value.get("schema") != "ferrum-native-wheel-e2e-evidence-v4":
		raise MolblockParityError("native E2E receipt has an unknown schema")
	wheel = value.get("wheel")
	chemistry = value.get("chemistry")
	closure = value.get("closure")
	replacement = value.get("replacement_proof")
	if not all(isinstance(entry, dict) for entry in (wheel, chemistry, closure, replacement)):
		raise MolblockParityError("native E2E receipt omits required evidence")
	if wheel.get("sha256") != wheel_sha256:
		raise MolblockParityError("native E2E receipt describes a different wheel")
	before = chemistry.get("python_extension_before")
	after = chemistry.get("python_extension_after")
	if not isinstance(before, dict) or not isinstance(after, dict):
		raise MolblockParityError("native E2E receipt omits Python probes")
	for probe in (before, after):
		if probe.get("molblock_versions_explicit") is not True:
			raise MolblockParityError("native E2E did not prove both explicit molblock versions")
		if probe.get("molblocks_newline_terminated") is not True:
			raise MolblockParityError("native E2E returned truncated molblock text")
		if probe.get("molblock_import_semantics") is not True:
			raise MolblockParityError("native E2E did not prove molblock import semantics")
	original = replacement.get("original_sha256")
	rebuilt = replacement.get("replacement_sha256")
	if type(original) is not str or type(rebuilt) is not str or original == rebuilt:
		raise MolblockParityError("native E2E replacement adapter is not distinct")
	names = closure.get("names")
	if not isinstance(names, list) or "libRDKitFileParsers.1.dylib" not in names:
		raise MolblockParityError("native E2E closure omits FileParsers")
	return {
		"closure": names,
		"molblock_after_replacement": True,
		"molblock_before_replacement": True,
		"original_adapter_sha256": original,
		"replacement_adapter_sha256": rebuilt,
	}


#============================================
def _report_rows(pinned: dict) -> list[dict]:
	"""Drop raw coordinates while retaining inspectable comparison outcomes."""
	return [
		{
			"canonical_smiles": record["canonical_smiles"],
			"formats": record["formats"],
			"name": record["name"],
		}
		for record in pinned["molecules"]
	]


#============================================
def main() -> int:
	"""Run the semantic differential and publish its source-bound receipt."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--oracle-python", required=True, type=pathlib.Path)
	parser.add_argument("--cross-version-python", type=pathlib.Path)
	parser.add_argument("--ferrum-python", required=True, type=pathlib.Path)
	parser.add_argument("--wheel", required=True, type=pathlib.Path)
	parser.add_argument("--coordinate-receipt", required=True, type=pathlib.Path)
	parser.add_argument("--native-e2e-receipt", required=True, type=pathlib.Path)
	parser.add_argument("--report", default=DEFAULT_REPORT, type=pathlib.Path)
	arguments = parser.parse_args()
	interpreters = [arguments.oracle_python, arguments.ferrum_python]
	if arguments.cross_version_python is not None:
		interpreters.append(arguments.cross_version_python)
	if any(not pathlib.Path(os.path.abspath(path)).is_file() for path in interpreters):
		raise MolblockParityError("every Python interpreter must be an existing file")
	wheel = arguments.wheel.resolve()
	wheel_sha256 = _sha256(wheel)
	ferrum_request = {
		"molecules": list(CORPUS),
		"schema": "ferrum-molblock-parity-request-v1",
	}
	ferrum = _validate_ferrum(
		_run_child(arguments.ferrum_python, FERRUM_CHILD, ferrum_request),
	)
	evaluation_request = _evaluator_request(ferrum)
	build_recorded = _validate_evaluation(
		_run_child(arguments.oracle_python, RDKIT_CHILD, evaluation_request),
		EXPECTED_RDKIT_VERSION,
		ferrum,
		"build-recorded RDKit",
	)
	coordinate = _coordinate_receipt(arguments.coordinate_receipt, wheel_sha256)
	raw_delta = _maximum_raw_coordinate_delta(ferrum, build_recorded)
	if raw_delta > coordinate["raw_coordinate_tolerance_max_abs"]:
		raise MolblockParityError("raw coordinates exceed the measured M4c tolerance")
	cross_version = None
	if arguments.cross_version_python is not None:
		cross_value = _run_child(
			arguments.cross_version_python, RDKIT_CHILD, evaluation_request,
		)
		cross_version_name = cross_value.get("rdkit_version")
		if type(cross_version_name) is not str or cross_version_name == EXPECTED_RDKIT_VERSION:
			raise MolblockParityError("cross-version evaluator is not a distinct RDKit version")
		cross = _validate_evaluation(
			cross_value, cross_version_name, ferrum, "cross-version RDKit",
		)
		cross_version = {
			"binary_sha256": cross["binary_sha256"],
			"rdkit_version": cross_version_name,
			"semantic_round_trip": True,
		}
	receipt = {
		"artifacts": {
			"ferrum_extension_sha256": ferrum["binary_sha256"],
			"rdkit_python_binary_sha256": build_recorded["binary_sha256"],
			"wheel": _display_path(wheel),
			"wheel_sha256": wheel_sha256,
		},
		"comparison_policy": {
			"coordinates": (
				"each parsed coordinate stays within half the actual emitted decimal "
				"quantum plus binary ULP; pre-writer coordinates use the measured M4c tolerance"
			),
			"discrete_facts": "exact after strict parse and normal sanitization",
			"headers_and_text": "observed only; never an acceptance gate",
			"molblock": "semantic round trip for explicitly requested V2000 and V3000",
			"molblock_import": (
				"Ferrum imports each emitted version to an owned FCM1 molecule with the same "
				"canonical chemistry and complete finite atom-aligned coordinates"
			),
		},
		"coordinate_source": {
			**coordinate,
			"raw_coordinate_max_abs_delta": raw_delta,
		},
		"corpus": _report_rows(build_recorded),
		"cross_version": cross_version,
		"native_wheel_e2e": _native_e2e(arguments.native_e2e_receipt, wheel_sha256),
		"build_rdkit_version": EXPECTED_RDKIT_VERSION,
		"platform": {"machine": platform.machine(), "system": platform.system()},
		"schema": "ferrum-molblock-codec-parity-v1",
		"source_sha256": {path: _sha256(REPO_ROOT / path) for path in SOURCE_PATHS},
		"status": "semantic-parity",
	}
	arguments.report.parent.mkdir(parents=True, exist_ok=True)
	arguments.report.write_text(
		json.dumps(receipt, allow_nan=False, indent=2, sort_keys=True) + "\n",
		encoding="ascii",
	)
	print(json.dumps({
		"corpus_size": len(receipt["corpus"]),
		"cross_version": None if cross_version is None else cross_version["rdkit_version"],
		"report": _display_path(arguments.report),
		"schema": receipt["schema"],
		"status": receipt["status"],
	}, separators=(",", ":"), sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
