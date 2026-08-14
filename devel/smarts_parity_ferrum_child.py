"""Emit installed Ferrum ABI-4 SMARTS facts for one closed request."""

# Standard Library
import json
import pathlib
import sys

# Third Party
import ferrum_chem


#============================================
def _fail(message: str) -> None:
	"""Write one diagnostic and terminate without a partial result."""
	print("Ferrum SMARTS error: " + message, file=sys.stderr)
	raise SystemExit(1)


#============================================
def main() -> int:
	"""Read one request and emit one installed-extension response."""
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
		molecule = ferrum_chem.parse_smiles(smiles)
		results.append({
			"canonical_smiles": molecule.canonical_smiles,
			"name": name,
			"smarts": ferrum_chem.molecule_to_smarts(molecule),
		})
	response = {
		"backend": "ferrum-abi4",
		"binary": str(pathlib.Path(ferrum_chem.__file__).resolve()),
		"molecules": results,
		"schema": "ferrum-smarts-parity-child-v1",
		"version": 4,
	}
	print(json.dumps(response, separators=(",", ":"), sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
