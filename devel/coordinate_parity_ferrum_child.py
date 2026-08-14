"""Emit exact Ferrum ABI-4 coordinates for one M4c request."""

# Standard Library
import hashlib
import json
import pathlib
import sys

# PIP3 modules
import ferrum_chem


#============================================
def _request() -> dict:
	"""Read one closed request object from standard input."""
	value = json.loads(sys.stdin.read())
	if value.get("schema") != "ferrum-coordinate-parity-request-v1":
		raise RuntimeError("Ferrum coordinate child received an unknown request schema")
	molecules = value.get("molecules")
	if not isinstance(molecules, list) or not molecules:
		raise RuntimeError("Ferrum coordinate child requires a nonempty molecule list")
	return value


#============================================
def main() -> None:
	"""Run one independent installed-extension measurement."""
	request = _request()
	results = []
	for record in request["molecules"]:
		name = record["name"]
		smiles = record["smiles"]
		if type(name) is not str or type(smiles) is not str:
			raise RuntimeError("Ferrum coordinate names and SMILES must be strings")
		molecule = ferrum_chem.parse_smiles(smiles)
		results.append({
			"atom_count": len(molecule.atoms),
			"canonical_smiles": molecule.canonical_smiles,
			"coordinates": [[point.x, point.y] for point in molecule.coordinates],
			"name": name,
		})
	binary = pathlib.Path(ferrum_chem.__file__).resolve()
	output = {
		"backend": "ferrum-abi4-fcm1",
		"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
		"molecules": results,
		"schema": "ferrum-coordinate-parity-child-v1",
	}
	print(json.dumps(output, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
	main()
