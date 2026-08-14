"""Derive the M4c coordinate tolerance from independent process families."""

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
ORACLE_CHILD = REPO_ROOT / "devel" / "coordinate_parity_oracle_child.py"
FERRUM_CHILD = REPO_ROOT / "devel" / "coordinate_parity_ferrum_child.py"
DEFAULT_REPORT = (
	REPO_ROOT / "docs" / "active_plans" / "reports" / "coordinate_parity_v1.json"
)
EXPECTED_RDKIT_VERSION = "2026.03.5"
MINIMUM_REPEATS = 20
CORPUS = (
	{"name": "asymmetric_amide", "smiles": "CC(=O)NCCCl", "asymmetric": True},
	{
		"name": "caffeine",
		"smiles": "Cn1c(=O)c2c(ncn2C)n(C)c1=O",
		"asymmetric": True,
	},
	{
		"name": "ibuprofen",
		"smiles": "CC(C)CC1=CC=C(C=C1)C(C)C(=O)O",
		"asymmetric": True,
	},
	{"name": "branched_octane", "smiles": "CCC(C)CC(C)C", "asymmetric": True},
	{"name": "bridged_ring", "smiles": "C1CC2CCC1C2", "asymmetric": True},
	{"name": "benzene_control", "smiles": "c1ccccc1", "asymmetric": False},
)
SOURCE_PATHS = (
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_adapter.cpp",
	"packages/ferrum-rust/crates/chemistry/native/include/ferrum_chem_adapter.h",
	"packages/ferrum-rust/crates/chemistry/src/native_engine.rs",
	"devel/coordinate_parity_ferrum_child.py",
	"devel/coordinate_parity_oracle_child.py",
	"devel/measure_coordinate_parity.py",
)


#============================================
class CoordinateParityError(RuntimeError):
	"""Raised when the measurement protocol or result is not trustworthy."""


#============================================
def _sha256(path: pathlib.Path) -> str:
	"""Return the SHA-256 digest of one required regular file."""
	if not path.is_file():
		raise CoordinateParityError("required parity input is not a regular file: " + str(path))
	return hashlib.sha256(path.read_bytes()).hexdigest()


#============================================
def _request_text() -> str:
	"""Return the one-line closed request shared by both children."""
	request = {
		"molecules": [
			{"name": record["name"], "smiles": record["smiles"]} for record in CORPUS
		],
		"schema": "ferrum-coordinate-parity-request-v1",
	}
	return json.dumps(request, separators=(",", ":"), sort_keys=True) + "\n"


#============================================
def _run_child(python: pathlib.Path, child: pathlib.Path, request_text: str) -> dict:
	"""Run one isolated child and require exactly one JSON object on stdout."""
	environment = os.environ.copy()
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	result = subprocess.run(
		[str(python), "-I", "-B", str(child)],
		cwd=REPO_ROOT,
		env=environment,
		input=request_text,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)
	if result.returncode != 0:
		raise CoordinateParityError(
			child.name + " failed: " + result.stderr.strip(),
		)
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise CoordinateParityError(child.name + " must emit exactly one JSON line")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise CoordinateParityError(child.name + " emitted invalid JSON") from error
	if not isinstance(value, dict):
		raise CoordinateParityError(child.name + " output must be a JSON object")
	return value


#============================================
def _validate_child(value: dict, backend: str) -> dict:
	"""Validate and normalize one complete child response."""
	if value.get("schema") != "ferrum-coordinate-parity-child-v1":
		raise CoordinateParityError(backend + " returned an unknown schema")
	if value.get("backend") != backend:
		raise CoordinateParityError(backend + " returned an unexpected backend name")
	digest = value.get("binary_sha256")
	if type(digest) is not str or len(digest) != 64:
		raise CoordinateParityError(backend + " omitted its binary digest")
	molecules = value.get("molecules")
	if not isinstance(molecules, list) or len(molecules) != len(CORPUS):
		raise CoordinateParityError(backend + " returned the wrong corpus size")
	normalized = []
	for expected, molecule in zip(CORPUS, molecules, strict=True):
		if not isinstance(molecule, dict) or molecule.get("name") != expected["name"]:
			raise CoordinateParityError(backend + " changed corpus order")
		atom_count = molecule.get("atom_count")
		canonical = molecule.get("canonical_smiles")
		coordinates = molecule.get("coordinates")
		if type(atom_count) is not int or atom_count < 1:
			raise CoordinateParityError(backend + " returned an invalid atom count")
		if type(canonical) is not str or not canonical:
			raise CoordinateParityError(backend + " returned empty canonical SMILES")
		if not isinstance(coordinates, list) or len(coordinates) != atom_count:
			raise CoordinateParityError(backend + " returned incomplete coordinates")
		points = []
		for point in coordinates:
			if not isinstance(point, list) or len(point) != 2:
				raise CoordinateParityError(backend + " returned a malformed coordinate")
			if any(type(component) not in (int, float) for component in point):
				raise CoordinateParityError(backend + " returned a nonnumeric coordinate")
			values = [float(component) for component in point]
			if not all(math.isfinite(component) for component in values):
				raise CoordinateParityError(backend + " returned a nonfinite coordinate")
			points.append(values)
		normalized.append({
			"atom_count": atom_count,
			"canonical_smiles": canonical,
			"coordinates": points,
			"name": expected["name"],
		})
	output = {
		"binary_sha256": digest,
		"molecules": normalized,
	}
	if backend == "rdkit-python-wrapper":
		version = value.get("rdkit_version")
		if version != EXPECTED_RDKIT_VERSION:
			raise CoordinateParityError(
				"oracle RDKit version must be " + EXPECTED_RDKIT_VERSION + ", not " + str(version),
			)
		output["rdkit_version"] = version
	return output


#============================================
def _maximum_delta(first: dict, second: dict) -> float:
	"""Return the largest aligned absolute coordinate delta between two runs."""
	maximum = 0.0
	for left, right in zip(first["molecules"], second["molecules"], strict=True):
		if (
			left["name"] != right["name"]
			or left["atom_count"] != right["atom_count"]
			or left["canonical_smiles"] != right["canonical_smiles"]
			):
			raise CoordinateParityError("coordinate backends disagree on molecule identity")
		for left_point, right_point in zip(
				left["coordinates"], right["coordinates"], strict=True,
				):
			for left_value, right_value in zip(left_point, right_point, strict=True):
				maximum = max(maximum, abs(left_value - right_value))
	return maximum


#============================================
def _maximum_resolution(*runs: dict) -> float:
	"""Return the largest ULP represented by any measured coordinate."""
	return max(
		math.ulp(component)
		for run in runs
		for molecule in run["molecules"]
		for point in molecule["coordinates"]
		for component in point
	)


#============================================
def _display_path(path: pathlib.Path) -> str:
	"""Return a repository-relative path when the artifact belongs to this tree."""
	resolved = path.resolve()
	try:
		return str(resolved.relative_to(REPO_ROOT))
	except ValueError:
		return str(resolved)


#============================================
def _measure(arguments: argparse.Namespace) -> dict:
	"""Run both process families and derive one explicit tolerance."""
	# A venv interpreter is normally a symlink. Resolving it would discard the
	# environment identity and accidentally run the system package set.
	oracle_python = pathlib.Path(os.path.abspath(arguments.oracle_python))
	ferrum_python = pathlib.Path(os.path.abspath(arguments.ferrum_python))
	wheel = arguments.wheel.resolve()
	for interpreter in (oracle_python, ferrum_python):
		if not interpreter.is_file():
			raise CoordinateParityError("Python interpreter is missing: " + str(interpreter))
	if arguments.repeats < MINIMUM_REPEATS:
		raise CoordinateParityError(
			"coordinate parity requires at least " + str(MINIMUM_REPEATS) + " process repeats",
		)
	request_text = _request_text()
	oracle_runs = [
		_validate_child(
			_run_child(oracle_python, ORACLE_CHILD, request_text),
			"rdkit-python-wrapper",
		)
		for _iteration in range(arguments.repeats)
	]
	ferrum_runs = [
		_validate_child(
			_run_child(ferrum_python, FERRUM_CHILD, request_text),
			"ferrum-abi4-fcm1",
		)
		for _iteration in range(arguments.repeats)
	]
	oracle_noise = max(_maximum_delta(oracle_runs[0], run) for run in oracle_runs[1:])
	ferrum_noise = max(_maximum_delta(ferrum_runs[0], run) for run in ferrum_runs[1:])
	cross_delta = _maximum_delta(oracle_runs[0], ferrum_runs[0])
	resolution = _maximum_resolution(oracle_runs[0], ferrum_runs[0])
	observed_noise = max(oracle_noise, ferrum_noise)
	tolerance = max(observed_noise * 4.0, resolution * 8.0)
	baselines = []
	for definition, oracle, ferrum in zip(
			CORPUS,
			oracle_runs[0]["molecules"],
			ferrum_runs[0]["molecules"],
			strict=True,
			):
		maximum = 0.0
		for left_point, right_point in zip(
				oracle["coordinates"], ferrum["coordinates"], strict=True,
				):
			for left_value, right_value in zip(left_point, right_point, strict=True):
				maximum = max(maximum, abs(left_value - right_value))
		baselines.append({
			"asymmetric": definition["asymmetric"],
			"atom_count": oracle["atom_count"],
			"canonical_smiles": oracle["canonical_smiles"],
			"ferrum_coordinates": ferrum["coordinates"],
			"max_abs_delta": maximum,
			"name": definition["name"],
			"oracle_coordinates": oracle["coordinates"],
			"smiles": definition["smiles"],
		})
	return {
		"artifacts": {
			"ferrum_extension_sha256": ferrum_runs[0]["binary_sha256"],
			"rdkit_python_binary_sha256": oracle_runs[0]["binary_sha256"],
			"wheel": _display_path(wheel),
			"wheel_sha256": _sha256(wheel),
		},
		"baselines": baselines,
		"measurement": {
			"coordinate_resolution_max_ulp": resolution,
			"cross_backend_max_abs_delta": cross_delta,
			"ferrum_process_noise_max_abs": ferrum_noise,
			"oracle_process_noise_max_abs": oracle_noise,
			"parity_passed": cross_delta <= tolerance,
			"tolerance_formula": "max(4 * observed process noise, 8 * maximum coordinate ULP)",
			"tolerance_max_abs": tolerance,
		},
		"platform": {
			"machine": platform.machine(),
			"system": platform.system(),
		},
		"rdkit_version": EXPECTED_RDKIT_VERSION,
		"repeats_per_backend": arguments.repeats,
		"schema": "ferrum-coordinate-parity-v1",
		"scope": (
			"RDKit 2026.03.5 Python wrapper versus the current ABI-4 FCM1 "
			"macOS arm64 wheel; M20 retains future cross-platform expansion"
		),
		"source_sha256": {
			path: _sha256(REPO_ROOT / path) for path in SOURCE_PATHS
		},
		"status": "measured" if cross_delta <= tolerance else "parity-failed",
	}


#============================================
def main() -> None:
	"""Parse explicit artifact inputs, write the receipt, and report its summary."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--oracle-python", type=pathlib.Path, required=True)
	parser.add_argument("--ferrum-python", type=pathlib.Path, required=True)
	parser.add_argument("--wheel", type=pathlib.Path, required=True)
	parser.add_argument("--output", type=pathlib.Path, default=DEFAULT_REPORT)
	parser.add_argument("--repeats", type=int, default=MINIMUM_REPEATS)
	arguments = parser.parse_args()
	report = _measure(arguments)
	arguments.output.parent.mkdir(parents=True, exist_ok=True)
	arguments.output.write_text(
		json.dumps(report, allow_nan=False, indent=2, sort_keys=True) + "\n",
		encoding="ascii",
	)
	print(json.dumps({
		"cross_backend_max_abs_delta": report["measurement"]["cross_backend_max_abs_delta"],
		"output": _display_path(arguments.output),
		"schema": report["schema"],
		"status": report["status"],
		"tolerance_max_abs": report["measurement"]["tolerance_max_abs"],
	}, separators=(",", ":"), sort_keys=True))
	if report["status"] != "measured":
		raise SystemExit(1)


if __name__ == "__main__":
	main()
