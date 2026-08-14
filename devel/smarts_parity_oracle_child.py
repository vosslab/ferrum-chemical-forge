"""Emit pinned Python-RDKit SMARTS facts for one closed request."""

# Standard Library
import json
import pathlib
import sys

# Third Party
from rdkit import Chem
from rdkit import rdBase


#============================================
def _fail(message: str) -> None:
	"""Write one diagnostic and terminate without a partial result."""
	print("SMARTS oracle error: " + message, file=sys.stderr)
	raise SystemExit(1)


#============================================
def main() -> int:
	"""Read one request and emit one pinned-oracle response."""
	try:
		request = json.loads(sys.stdin.read())
	except json.JSONDecodeError as error:
		_fail("request is not valid JSON: " + error.msg)
	if not isinstance(request, dict):
		_fail("request must be a JSON object")
	if request.get("schema") != "ferrum-smarts-parity-request-v1":
		_fail("request has an unknown schema")
	molecules = request.get("molecules")
	if not isinstance(molecules, list):
		_fail("request molecules must be a list")
	results = []
	for entry in molecules:
		if not isinstance(entry, dict):
			_fail("each molecule must be a JSON object")
		name = entry.get("name")
		smiles = entry.get("smiles")
		if not isinstance(name, str) or not isinstance(smiles, str):
			_fail("each molecule needs string name and smiles fields")
		molecule = Chem.MolFromSmiles(smiles)
		if molecule is None:
			_fail("RDKit rejected " + name)
		results.append({
			"canonical_smiles": Chem.MolToSmiles(molecule),
			"name": name,
			"smarts": Chem.MolToSmarts(molecule),
		})
	response = {
		"backend": "rdkit-python-wrapper",
		"binary": str(pathlib.Path(rdBase.__file__).resolve()),
		"molecules": results,
		"schema": "ferrum-smarts-parity-child-v1",
		"version": rdBase.rdkitVersion,
	}
	print(json.dumps(response, separators=(",", ":"), sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
