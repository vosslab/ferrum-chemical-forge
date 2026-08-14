"""One-time isolated RDKit oracle measurement for Ferrum M11.

Run from the repository root with:
    source source_me.sh && python3 devel/measure_straighten_depiction.py

This is intentionally a maintainer experiment, not pytest coverage. It invokes
the dedicated child through an isolated Python process; Ferrum's Rust geometry
crate never links to or calls Python RDKit. The emitted JSON is a target-specific
receipt, not a numerical release gate.
"""

# Standard Library
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess  # nosec B404
import sys


REPOSITORY_ROOT = pathlib.Path(__file__).resolve().parent.parent
RUST_WORKSPACE = REPOSITORY_ROOT / "packages" / "ferrum-rust"
ORACLE_PYTHON = REPOSITORY_ROOT / "tests" / "e2e" / "oracle" / ".venv" / "bin" / "python"
ORACLE_CHILD = REPOSITORY_ROOT / "devel" / "straighten_depiction_oracle_child.py"
ORACLE_REQUIREMENTS = REPOSITORY_ROOT / "tests" / "e2e" / "oracle" / "pip_requirements.txt"
EXPECTED_RDKIT_VERSION = "2026.03.5"
SOURCE_PATHS = (
	"devel/measure_straighten_depiction.py",
	"devel/straighten_depiction_oracle_child.py",
	"packages/ferrum-rust/Cargo.lock",
	"packages/ferrum-rust/crates/geometry/examples/straighten_probe.rs",
	"packages/ferrum-rust/crates/geometry/src/lib.rs",
	"packages/ferrum-rust/crates/geometry/src/straighten.rs",
)


#============================================
class StraightenMeasurementError(RuntimeError):
	"""Raised when this local receipt cannot identify its measured inputs."""


#============================================
def _sha256(path: pathlib.Path) -> str:
	"""Return the SHA-256 digest of one required regular file."""
	if not path.is_file():
		raise StraightenMeasurementError("required receipt input is not a file: " + str(path))
	return hashlib.sha256(path.read_bytes()).hexdigest()


#============================================
def _git_output(arguments: list[str]) -> str:
	"""Return one read-only Git result or raise a measurement-specific error."""
	git_executable = shutil.which("git")
	if git_executable is None:
		raise StraightenMeasurementError("git is unavailable for source identification")
	# The command contains only fixed, read-only Git arguments from this module.
	completed = subprocess.run(  # nosec B603
		[git_executable, *arguments],  # nosec B603
		cwd=REPOSITORY_ROOT,
		check=False,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
	)
	if completed.returncode != 0:
		raise StraightenMeasurementError("git " + " ".join(arguments) + " failed: " + completed.stderr)
	return completed.stdout


#============================================
def _source_identity() -> dict:
	"""Return read-only source, lockfile, and worktree identity for this receipt."""
	status = _git_output(["status", "--porcelain=v1"])
	return {
		"cargo_lock_sha256": _sha256(RUST_WORKSPACE / "Cargo.lock"),
		"git_head": _git_output(["rev-parse", "HEAD"]).strip(),
		"oracle_requirements": {
			"path": str(ORACLE_REQUIREMENTS.relative_to(REPOSITORY_ROOT)),
			"sha256": _sha256(ORACLE_REQUIREMENTS),
		},
		"source_sha256": {path: _sha256(REPOSITORY_ROOT / path) for path in SOURCE_PATHS},
		"worktree_dirty": bool(status.strip()),
		"worktree_status_sha256": hashlib.sha256(status.encode("utf-8")).hexdigest(),
	}


#============================================
def _oracle_interpreter() -> tuple[pathlib.Path, dict]:
	"""Select the maintained isolated interpreter or describe an honest fallback."""
	if ORACLE_PYTHON.is_file():
		return (
			ORACLE_PYTHON,
			{
				"selection": "dedicated_oracle_venv",
				"selection_reason": "the isolated oracle interpreter is present",
			},
		)
	return (
		pathlib.Path(sys.executable),
		{
			"selection": "active_bootstrap_fallback",
			"selection_reason": "dedicated oracle interpreter is absent: " + str(ORACLE_PYTHON),
		},
	)


def _run_ferrum_probe() -> dict:
	"""Run the Rust probe in a separate process and decode its shared cases."""
	command = [
		"cargo",
		"run",
		"--locked",
		"--offline",
		"--quiet",
		"-p",
		"ferrum-geometry",
		"--example",
		"straighten_probe",
	]
	# This Cargo geometry probe is fixed source-maintainer tooling.
	completed = subprocess.run(  # nosec B603
		command,
		cwd=RUST_WORKSPACE,
		check=False,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
	)
	if completed.returncode != 0:
		raise StraightenMeasurementError("Ferrum probe failed: " + completed.stderr)
	try:
		return json.loads(completed.stdout)
	except json.JSONDecodeError as error:
		raise StraightenMeasurementError("Ferrum probe emitted invalid JSON") from error


def _run_oracle_probe(interpreter: pathlib.Path) -> dict:
	"""Run the RDKit measurement in an isolated oracle process."""
	if not interpreter.is_file():
		raise StraightenMeasurementError("RDKit interpreter is unavailable: " + str(interpreter))
	# The oracle process does not inherit local imports or bytecode settings.
	environment = os.environ.copy()
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	command = [str(interpreter), "-I", "-B", str(ORACLE_CHILD)]
	# Both executable paths were resolved from this repository or the active bootstrap.
	completed = subprocess.run(  # nosec B603
		command,
		cwd=REPOSITORY_ROOT,
		env=environment,
		check=False,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
	)
	if completed.returncode != 0:
		raise StraightenMeasurementError("RDKit oracle failed: " + completed.stderr)
	try:
		return json.loads(completed.stdout)
	except json.JSONDecodeError as error:
		raise StraightenMeasurementError("RDKit oracle emitted invalid JSON") from error


def _validated_ferrum_cases(oracle: dict, ferrum: dict) -> tuple[list[dict], dict]:
	"""Compare both branches of the shared cases, including their applied rotations."""
	if oracle.get("schema") != "ferrum-straighten-depiction-oracle-child-v1":
		raise StraightenMeasurementError("RDKit oracle returned an unknown schema")
	if oracle.get("rdkit_version") != EXPECTED_RDKIT_VERSION:
		raise StraightenMeasurementError(
			"RDKit oracle version must be " + EXPECTED_RDKIT_VERSION + ", not "
			+ str(oracle.get("rdkit_version"))
		)
	if not isinstance(ferrum.get("cases"), list):
		raise StraightenMeasurementError("Ferrum probe omitted its shared cases")
	ferrum_cases = {}
	for case in ferrum["cases"]:
		if not isinstance(case, dict) or not isinstance(case.get("name"), str):
			raise StraightenMeasurementError("Ferrum probe emitted an invalid case")
		if case["name"] in ferrum_cases:
			raise StraightenMeasurementError("Ferrum probe emitted a duplicate case: " + case["name"])
		if not isinstance(case.get("branches"), dict):
			raise StraightenMeasurementError("Ferrum probe omitted branches for: " + case["name"])
		ferrum_cases[case["name"]] = case
	differences = []
	for name, branches in oracle["measurements"].items():
		if name not in ferrum_cases:
			raise StraightenMeasurementError("Ferrum probe omitted oracle case: " + name)
		for minimize_rotation, result in branches.items():
			if minimize_rotation not in ferrum_cases[name]["branches"]:
				raise StraightenMeasurementError(
					"Ferrum probe omitted oracle branch: " + name + "/" + minimize_rotation
				)
			oracle_first = result["first"]
			ferrum_branch = ferrum_cases[name]["branches"][minimize_rotation]
			coordinates = ferrum_branch.get("coordinates")
			rotation = ferrum_branch.get("rotation_radians")
			if (
				not isinstance(coordinates, list)
				or len(coordinates) != len(oracle_first["coordinates"])
				or not isinstance(rotation, (int, float))
			):
				raise StraightenMeasurementError(
					"Ferrum probe emitted malformed result: " + name + "/" + minimize_rotation
				)
			for coordinate in coordinates:
				if (
					not isinstance(coordinate, list)
					or len(coordinate) != 2
					or not all(isinstance(value, (int, float)) for value in coordinate)
				):
					raise StraightenMeasurementError(
						"Ferrum probe emitted malformed coordinates: "
						+ name
						+ "/"
						+ minimize_rotation
					)
			coordinate_difference = max(
				abs(value - coordinates[atom_index][axis])
				for atom_index, coordinate in enumerate(oracle_first["coordinates"])
				for axis, value in enumerate(coordinate)
			)
			rotation_difference = abs(oracle_first["rotation_radians"] - rotation)
			differences.append(
				{
					"case": name,
					"minimize_rotation": minimize_rotation,
					"maximum_coordinate_difference": coordinate_difference,
					"rotation_difference_radians": rotation_difference,
				}
			)
	if not differences:
		raise StraightenMeasurementError("RDKit oracle omitted all measurements")
	maximum_coordinate_difference = max(item["maximum_coordinate_difference"] for item in differences)
	maximum_rotation_difference = max(item["rotation_difference_radians"] for item in differences)
	return ferrum["cases"], {
		"per_case": differences,
		"maximum_coordinate_difference": maximum_coordinate_difference,
		"maximum_rotation_difference_radians": maximum_rotation_difference,
	}


def main() -> None:
	"""Emit reproducible measurement data on standard output."""
	interpreter, interpreter_selection = _oracle_interpreter()
	oracle = _run_oracle_probe(interpreter)
	measurements = oracle.get("measurements")
	if not isinstance(measurements, dict):
		raise StraightenMeasurementError("RDKit oracle omitted its measurements")
	ferrum = _run_ferrum_probe()
	ferrum_cases, comparison = _validated_ferrum_cases(oracle, ferrum)
	report = {
		"case_corpus_sha256": oracle["case_corpus_sha256"],
		"ferrum_comparison": comparison,
		"ferrum_cases": ferrum_cases,
		"measurement_scope": (
			"One-time local receipt for the current measured release target; it does not "
			"define a cross-platform tolerance or a permanent gate."
		),
		"measurements": measurements,
		"oracle": {
			"interpreter": {
				"path": str(interpreter),
				"resolved_path": str(interpreter.resolve()),
				**interpreter_selection,
			},
			"rdkit_artifacts": oracle["rdkit_artifacts"],
			"rdkit_version": oracle["rdkit_version"],
		},
		"platform": {
			"machine": platform.machine(),
			"platform": platform.platform(),
			"python_implementation": platform.python_implementation(),
			"python_version": platform.python_version(),
			"system": platform.system(),
		},
		"repeats": oracle["repeats"],
		"schema": "ferrum-straighten-depiction-platform-receipt-v1",
		"source_identity": _source_identity(),
	}
	print(json.dumps(report, allow_nan=False, indent=2, sort_keys=True))


if __name__ == "__main__":
	main()
