#!/usr/bin/env python3
"""Project one narrow molecule-core carrier through the pinned OASA process."""

# Standard Library
import importlib.metadata
import json
import sys

# PIP3 modules
import rdkit
import oasa.atom_lib
import oasa.bond_lib
import oasa.molecule_lib


#============================================
def project_request(request: dict) -> dict:
	"""Construct real OASA records and return only shared carrier fields."""
	molecule = oasa.molecule_lib.Molecule()
	atoms = []
	for atom_data in request["atoms"]:
		atom = oasa.atom_lib.Atom(
			symbol=atom_data["element"], charge=atom_data["formal_charge"]
		)
		atom.explicit_hydrogens = atom_data["explicit_hydrogens"]
		molecule.add_vertex(atom)
		atoms.append(atom)
	bonds = []
	for bond_data in request["bonds"]:
		bond = oasa.bond_lib.Bond(
			vs=[atoms[bond_data["start"]], atoms[bond_data["end"]]],
			order=bond_data["order"],
			type=bond_data["type"],
		)
		molecule.add_edge(atoms[bond_data["start"]], atoms[bond_data["end"]], e=bond)
		bonds.append(bond)
	projection = {
		"capability": request["capability"],
		"atoms": [
			{
				"index": index,
				"element": atom.symbol,
				"formal_charge": atom.charge,
				"explicit_hydrogens": atom.explicit_hydrogens,
			}
			for index, atom in enumerate(atoms)
		],
		"bonds": [
			{
				"start": atoms.index(bond.vertices[0]),
				"end": atoms.index(bond.vertices[1]),
				"order": bond.order,
				"type": bond.type,
			}
			for bond in bonds
		],
	}
	return projection


#============================================
def main() -> None:
	"""Emit exactly one child protocol object."""
	request_text = sys.stdin.read()
	request = json.loads(request_text)
	if not isinstance(request, dict):
		raise ValueError("request must be a JSON object")
	projection = project_request(request)
	result = {
		"engine": "oasa",
		"facts": {
			"oasa_version": importlib.metadata.version("oasa"),
			"python_version": sys.version.split()[0],
			"rdkit_version": rdkit.__version__,
		},
		"projection": projection,
	}
	print(json.dumps(result, separators=(",", ":"), sort_keys=True))


if __name__ == "__main__":
	main()
