"""Measure both explicit RDKit 2D orientation modes for the maintainer oracle tool."""

# Standard Library
import json
import sys

# PIP3 modules
import rdkit
from rdkit import Chem
from rdkit.Chem import rdDepictor


CAPABILITY = "rdkit-layout-orientation-v1"
SMILES = "CC(=O)NCCCl"


#============================================
def coordinates(molecule: Chem.Mol) -> list[dict]:
	"""Return atom-indexed, three-dimensional conformer coordinates."""
	conformer = molecule.GetConformer()
	points = []
	for index in range(molecule.GetNumAtoms()):
		position = conformer.GetAtomPosition(index)
		points.append(
			{"index": index, "x": position.x, "y": position.y, "z": position.z}
		)
	return points


#============================================
def measure_orientation(source: Chem.Mol, canon_orient: bool) -> dict:
	"""Lay out one independent copy under the requested explicit orientation."""
	molecule = Chem.Mol(source)
	rdDepictor.Compute2DCoords(molecule, canonOrient=canon_orient)
	measurement = {
		"canon_orient": canon_orient,
		"coordinates": coordinates(molecule),
	}
	return measurement


#============================================
def main() -> None:
	"""Emit one result containing both orientation modes from this process."""
	source = Chem.MolFromSmiles(SMILES)
	if source is None:
		raise RuntimeError("the fixed asymmetric oracle molecule could not be parsed")
	measurements = [
		measure_orientation(source, canon_orient=False),
		measure_orientation(source, canon_orient=True),
	]
	result = {
		"capability": CAPABILITY,
		"facts": {
			"atom_count": source.GetNumAtoms(),
			"python_version": sys.version.split()[0],
			"rdkit_version": rdkit.__version__,
			"smiles": SMILES,
		},
		"measurements": measurements,
	}
	print(json.dumps(result, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
	main()
