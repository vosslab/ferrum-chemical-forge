"""One-time isolated RDKit oracle measurement for Ferrum M11.

Run from the repository root with:
    source source_me.sh && python3 devel/measure_straighten_depiction.py

This is intentionally a maintainer experiment, not pytest coverage.  It loads
RDKit only in this Python process; Ferrum's Rust geometry crate never links to
or calls it.  The emitted JSON records repeatability for both API branches.
"""

import json
import math
import pathlib
import subprocess

from rdkit import Chem
from rdkit.Chem import AllChem


CASES = {
	"ten_degree_bond": ((0.0, 0.0), (math.cos(math.radians(10.0)), math.sin(math.radians(10.0)))),
	"fifteen_degree_boundary": ((0.0, 0.0), (math.cos(math.radians(15.0)), math.sin(math.radians(15.0)))),
	"thirty_degree_boundary": ((0.0, 0.0), (math.cos(math.radians(30.0)), math.sin(math.radians(30.0)))),
	"asymmetric_three_bond": ((0.0, 0.0), (1.0, 0.2), (1.7, 1.1), (2.4, 1.35)),
}
REPEATS = 25
REPOSITORY_ROOT = pathlib.Path(__file__).resolve().parent.parent
RUST_WORKSPACE = REPOSITORY_ROOT / "packages" / "ferrum-rust"


def _applied_rotation(
	original: tuple[tuple[float, float], ...],
	rotated: list[list[float]],
) -> float:
	"""Derive the origin-centered rotation from all non-zero input coordinates."""
	dot_product = sum(
		x * rotated[index][0] + y * rotated[index][1]
		for index, (x, y) in enumerate(original)
	)
	cross_product = sum(
		x * rotated[index][1] - y * rotated[index][0]
		for index, (x, y) in enumerate(original)
	)
	if dot_product == 0.0 and cross_product == 0.0:
		raise ValueError("cannot derive rotation from coordinates all located at the origin")
	angle = math.atan2(cross_product, dot_product)
	return angle


def _run_case(points: tuple[tuple[float, float], ...], minimize_rotation: bool) -> dict:
	"""Run RDKit on a simple chain with supplied 2-D coordinates."""
	molecule = Chem.MolFromSmiles("C" * len(points))
	conformer = Chem.Conformer(len(points))
	for index, (x, y) in enumerate(points):
		conformer.SetAtomPosition(index, (x, y, 0.0))
	molecule.RemoveAllConformers()
	molecule.AddConformer(conformer, assignId=True)
	AllChem.StraightenDepiction(molecule, minimizeRotation=minimize_rotation)
	result = molecule.GetConformer()
	coordinates = []
	for index in range(len(points)):
		position = result.GetAtomPosition(index)
		coordinates.append([position.x, position.y])
	rotation_radians = _applied_rotation(points, coordinates)
	return {"coordinates": coordinates, "rotation_radians": rotation_radians}


def _maximum_coordinate_variation(runs: list[dict]) -> float:
	"""Return the maximum component spread across repeated oracle output."""
	first = runs[0]["coordinates"]
	return max(
		abs(value - first[atom_index][axis])
		for run in runs
		for atom_index, coordinate in enumerate(run["coordinates"])
		for axis, value in enumerate(coordinate)
	)


def _run_ferrum_probe() -> dict:
	"""Run the Rust probe in a separate process and decode its shared cases."""
	command = ["cargo", "run", "--quiet", "-p", "ferrum-geometry", "--example", "straighten_probe"]
	completed = subprocess.run(command, cwd=RUST_WORKSPACE, check=True, text=True, capture_output=True)
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
	measurements = {}
	for name, points in CASES.items():
		branches = {}
		for minimize_rotation in (False, True):
			runs = [_run_case(points, minimize_rotation) for _ in range(REPEATS)]
			branches[str(minimize_rotation).lower()] = {
				"first": runs[0],
				"maximum_repeat_coordinate_variation": _maximum_coordinate_variation(runs),
			}
		measurements[name] = branches
	ferrum = _run_ferrum_probe()
	comparison = _maximum_ferrum_difference(measurements, ferrum)
	report = {"repeats": REPEATS, "measurements": measurements, "ferrum_comparison": comparison}
	print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
	main()
