"""Emit installed Ferrum molblocks and their unrounded source coordinates."""

# Standard Library
import json
import pathlib
import sys

# PIP3 modules
import ferrum_chem


#============================================
def _fail(message: str) -> None:
	"""Write one diagnostic and terminate without a partial result."""
	print("Ferrum molblock error: " + message, file=sys.stderr)
	raise SystemExit(1)


#============================================
def _request() -> dict:
	"""Read and validate the closed parity request."""
	try:
		value = json.loads(sys.stdin.read())
	except json.JSONDecodeError as error:
		_fail("request is not valid JSON: " + error.msg)
	if not isinstance(value, dict):
		_fail("request must be a JSON object")
	if value.get("schema") != "ferrum-molblock-parity-request-v1":
		_fail("request has an unknown schema")
	molecules = value.get("molecules")
	if not isinstance(molecules, list) or not molecules:
		_fail("request needs a nonempty molecule list")
	return value


#============================================
def main() -> int:
	"""Export both explicit molblock versions through the installed extension."""
	request = _request()
	results = []
	for record in request["molecules"]:
		if not isinstance(record, dict):
			_fail("each molecule must be an object")
		name = record.get("name")
		smiles = record.get("smiles")
		if type(name) is not str or type(smiles) is not str:
			_fail("each molecule needs string name and smiles fields")
		molecule = ferrum_chem.parse_smiles(smiles)
		molblocks = {
			"v2000": ferrum_chem.molecule_to_molblock(
				molecule, ferrum_chem.MolblockVersionV1.v2000,
			),
			"v3000": ferrum_chem.molecule_to_molblock(
				molecule, ferrum_chem.MolblockVersionV1.v3000,
			),
		}
		imports = {}
		for version, molblock in molblocks.items():
			imported = ferrum_chem.molblock_to_molecule(molblock)
			imports[version] = {
				"atom_count": len(imported.atoms),
				"bond_count": len(imported.bonds),
				"canonical_smiles": imported.canonical_smiles,
				"coordinates": [[point.x, point.y] for point in imported.coordinates],
			}
		results.append({
			"canonical_smiles": molecule.canonical_smiles,
			"coordinates": [[point.x, point.y] for point in molecule.coordinates],
			"imports": imports,
			"molblocks": molblocks,
			"name": name,
		})
	response = {
		"backend": "ferrum-abi4",
		"binary": str(pathlib.Path(ferrum_chem.__file__).resolve()),
		"molecules": results,
		"schema": "ferrum-molblock-parity-ferrum-v1",
		"version": 4,
	}
	print(json.dumps(response, allow_nan=False, separators=(",", ":"), sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
