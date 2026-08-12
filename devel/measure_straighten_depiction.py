"""One-time isolated RDKit oracle measurement for Ferrum M11.

Run from the repository root with:
    source source_me.sh && python3 devel/measure_straighten_depiction.py

This is intentionally a maintainer experiment, not pytest coverage. It invokes
the dedicated child through the isolated oracle environment; Ferrum's Rust
geometry crate never links to or calls Python RDKit. The emitted JSON records
repeatability for both API branches.
"""

import json
import pathlib
import subprocess

REPOSITORY_ROOT = pathlib.Path(__file__).resolve().parent.parent
RUST_WORKSPACE = REPOSITORY_ROOT / "packages" / "ferrum-rust"
ORACLE_PYTHON = REPOSITORY_ROOT / "tests" / "e2e" / "oracle" / ".venv" / "bin" / "python"
ORACLE_CHILD = REPOSITORY_ROOT / "devel" / "straighten_depiction_oracle_child.py"


def _run_ferrum_probe() -> dict:
	"""Run the Rust probe in a separate process and decode its shared cases."""
	command = ["cargo", "run", "--quiet", "-p", "ferrum-geometry", "--example", "straighten_probe"]
	completed = subprocess.run(command, cwd=RUST_WORKSPACE, check=True, text=True, capture_output=True)
	return json.loads(completed.stdout)


def _run_oracle_probe() -> dict:
	"""Run the RDKit measurement in its isolated historical-oracle environment."""
	if not ORACLE_PYTHON.is_file():
		raise RuntimeError(
			"RDKit oracle environment is unavailable; run devel/setup_oracle_env.sh first"
		)
	# The oracle venv is intentionally isolated from the developer shell.
	command = [str(ORACLE_PYTHON), "-B", str(ORACLE_CHILD)]
	completed = subprocess.run(command, cwd=REPOSITORY_ROOT, check=True, text=True, capture_output=True)
	return json.loads(completed.stdout)


def _maximum_ferrum_difference(oracle: dict, ferrum: dict) -> dict:
	"""Compare both branches of the shared cases, including their applied rotations."""
	ferrum_cases = {case["name"]: case for case in ferrum["cases"]}
	differences = []
	for name, branches in oracle.items():
		for minimize_rotation, result in branches.items():
			oracle_first = result["first"]
			ferrum_branch = ferrum_cases[name]["branches"][minimize_rotation]
			coordinate_difference = max(
				abs(value - ferrum_branch["coordinates"][atom_index][axis])
				for atom_index, coordinate in enumerate(oracle_first["coordinates"])
				for axis, value in enumerate(coordinate)
			)
			rotation_difference = abs(
				oracle_first["rotation_radians"] - ferrum_branch["rotation_radians"]
			)
			differences.append(
				{
					"case": name,
					"minimize_rotation": minimize_rotation,
					"maximum_coordinate_difference": coordinate_difference,
					"rotation_difference_radians": rotation_difference,
				}
			)
	maximum_coordinate_difference = max(item["maximum_coordinate_difference"] for item in differences)
	maximum_rotation_difference = max(item["rotation_difference_radians"] for item in differences)
	return {
		"per_case": differences,
		"maximum_coordinate_difference": maximum_coordinate_difference,
		"maximum_rotation_difference_radians": maximum_rotation_difference,
	}


def main() -> None:
	"""Emit reproducible measurement data on standard output."""
	oracle = _run_oracle_probe()
	measurements = oracle["measurements"]
	ferrum = _run_ferrum_probe()
	comparison = _maximum_ferrum_difference(measurements, ferrum)
	report = {
		"repeats": oracle["repeats"],
		"measurements": measurements,
		"ferrum_comparison": comparison,
	}
	print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
	main()
