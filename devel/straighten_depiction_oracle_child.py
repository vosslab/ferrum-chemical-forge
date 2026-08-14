"""Measure RDKit depiction straightening inside the isolated oracle environment."""

# Standard Library
import hashlib
import json
import math
import pathlib
import sys

# PIP3 modules
import rdkit
import rdkit.rdBase
from rdkit import Chem
from rdkit.Chem import AllChem


CASES = {
	"ten_degree_bond": ((0.0, 0.0), (math.cos(math.radians(10.0)), math.sin(math.radians(10.0)))),
	"fifteen_degree_boundary": ((0.0, 0.0), (math.cos(math.radians(15.0)), math.sin(math.radians(15.0)))),
	"thirty_degree_boundary": ((0.0, 0.0), (math.cos(math.radians(30.0)), math.sin(math.radians(30.0)))),
	"asymmetric_three_bond": ((0.0, 0.0), (1.0, 0.2), (1.7, 1.1), (2.4, 1.35)),
}
REPEATS = 25


#============================================
def case_corpus_sha256() -> str:
	"""Return the digest of the exact fixed geometry corpus."""
	encoded = json.dumps(CASES, separators=(",", ":"), sort_keys=True).encode("ascii")
	return hashlib.sha256(encoded).hexdigest()


def applied_rotation(
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


def run_case(points: tuple[tuple[float, float], ...], minimize_rotation: bool) -> dict:
	"""Run RDKit on a simple chain with supplied two-dimensional coordinates."""
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
	rotation_radians = applied_rotation(points, coordinates)
	return {"coordinates": coordinates, "rotation_radians": rotation_radians}


def maximum_coordinate_variation(runs: list[dict]) -> float:
	"""Return the maximum component spread across repeated oracle output."""
	first = runs[0]["coordinates"]
	variation = max(
		abs(value - first[atom_index][axis])
		for run in runs
		for atom_index, coordinate in enumerate(run["coordinates"])
		for axis, value in enumerate(coordinate)
	)
	return variation


#============================================
def maximum_rotation_variation(runs: list[dict]) -> float:
	"""Return the largest applied-angle change from the first local run."""
	first = runs[0]["rotation_radians"]
	return max(abs(run["rotation_radians"] - first) for run in runs)


#============================================
def rdkit_artifacts() -> list[dict]:
	"""Identify loaded RDKit files without scanning the package installation."""
	artifacts = []
	for module_name, module in sorted(sys.modules.items()):
		if module_name != "rdkit" and not module_name.startswith("rdkit."):
			continue
		module_path = getattr(module, "__file__", None)
		if not isinstance(module_path, str):
			artifacts.append({"module": module_name, "status": "absent"})
			continue
		try:
			path = pathlib.Path(module_path).resolve()
		except OSError as error:
			artifacts.append(
				{"module": module_name, "path": module_path, "status": "unresolvable: " + str(error)}
			)
			continue
		if not path.is_file():
			artifacts.append({"module": module_name, "path": str(path), "status": "not_a_file"})
			continue
		artifacts.append(
			{
				"module": module_name,
				"path": str(path),
				"sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
				"status": "hashed",
			}
		)
	return artifacts


def main() -> None:
	"""Emit repeated RDKit results for each shared Ferrum geometry case."""
	measurements = {}
	for name, points in CASES.items():
		branches = {}
		for minimize_rotation in (False, True):
			runs = [run_case(points, minimize_rotation) for _ in range(REPEATS)]
			branches[str(minimize_rotation).lower()] = {
				"first": runs[0],
				"maximum_repeat_coordinate_variation": maximum_coordinate_variation(runs),
				"maximum_repeat_rotation_variation_radians": maximum_rotation_variation(runs),
			}
		measurements[name] = branches
	result = {
		"case_corpus_sha256": case_corpus_sha256(),
		"measurements": measurements,
		"rdkit_artifacts": rdkit_artifacts(),
		"rdkit_version": rdkit.__version__,
		"repeats": REPEATS,
		"schema": "ferrum-straighten-depiction-oracle-child-v1",
	}
	print(json.dumps(result, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
	main()
