"""Emit exact RDKit Python-wrapper coordinates for one M4c request."""

# Standard Library
import hashlib
import json
import pathlib
import sys

# PIP3 modules
import rdkit
import rdkit.rdBase
from rdkit import Chem
from rdkit.Chem import rdDepictor


#============================================
def _request() -> dict:
	"""Read one closed request object from standard input."""
	value = json.loads(sys.stdin.read())
	if value.get("schema") != "ferrum-coordinate-parity-request-v1":
		raise RuntimeError("coordinate oracle received an unknown request schema")
	molecules = value.get("molecules")
	if not isinstance(molecules, list) or not molecules:
		raise RuntimeError("coordinate oracle requires a nonempty molecule list")
	return value


#============================================
def _coordinates(molecule: Chem.Mol, conformer_id: int) -> list[list[float]]:
	"""Return finite x/y coordinates in exact RDKit atom order."""
	conformer = molecule.GetConformer(conformer_id)
	return [
		[float(conformer.GetAtomPosition(index).x), float(conformer.GetAtomPosition(index).y)]
		for index in range(molecule.GetNumAtoms())
	]


#============================================
def main() -> None:
	"""Run one independent explicit-default Python-wrapper measurement."""
	request = _request()
	results = []
	for record in request["molecules"]:
		name = record["name"]
		smiles = record["smiles"]
		if type(name) is not str or type(smiles) is not str:
			raise RuntimeError("coordinate oracle names and SMILES must be strings")
		molecule = Chem.MolFromSmiles(smiles)
		if molecule is None:
			raise RuntimeError("coordinate oracle could not parse " + name)
		conformer_id = rdDepictor.Compute2DCoords(
			molecule,
			canonOrient=True,
			clearConfs=True,
			coordMap={},
			nFlipsPerSample=0,
			nSample=0,
			sampleSeed=0,
			permuteDeg4Nodes=False,
			bondLength=-1.0,
			forceRDKit=True,
			useRingTemplates=False,
		)
		results.append({
			"atom_count": molecule.GetNumAtoms(),
			"canonical_smiles": Chem.MolToSmiles(molecule),
			"coordinates": _coordinates(molecule, conformer_id),
			"name": name,
		})
	binary = pathlib.Path(rdkit.rdBase.__file__).resolve()
	output = {
		"backend": "rdkit-python-wrapper",
		"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
		"molecules": results,
		"rdkit_version": rdkit.__version__,
		"schema": "ferrum-coordinate-parity-child-v1",
	}
	print(json.dumps(output, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
	main()
