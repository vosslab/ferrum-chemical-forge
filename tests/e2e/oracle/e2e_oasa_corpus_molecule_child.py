#!/usr/bin/env python3
"""Project one corpus CDML file through the pinned OASA chemistry-import reader.

Process contract: read one JSON request object on stdin, write exactly one JSON
object on stdout. The request carries a capability name and a corpus file path.
The reply mirrors the sibling worker e2e_oasa_molecule_core_child.py: an engine
name, pinned interpreter facts, and a projection.

Scope: this worker reports exactly what oasa.cdml.read_cdml produces. That
reader accepts an atom only when its CDML name is a periodic symbol or a known
group abbreviation, and it skips a bond whose endpoint is missing from the atom
map, so a molecule projected here can hold far fewer records than its source
file. That difference is reported, never repaired.

Coordinate unit: PostScript points. OASA converts a CDML `cm` coordinate with
the factor 72/2.54 and passes a bare number through unchanged, so every emitted
point is already in points. Emitted coordinates are rounded to 6 decimal places
so downstream exact-equality comparison sees a stable representation.

Ordering: molecules keep their read_cdml yield order and atoms keep their
molecule vertex order. OASA stores edges in a set, so bonds are sorted by
ordered endpoint index, then bond id, type, and order, to make output stable
across runs.
"""

# Standard Library
import json
import sys
import importlib.metadata

# PIP3 modules
import rdkit
import oasa.cdml


#============================================
def optional_attribute(record: object, attribute_name: str) -> object:
	"""Return an OASA attribute that exists only when CDML supplied it.

	OASA sets `id` on a molecule, atom, or bond only when the source element
	carried one, and absence is a fact the comparison needs to see.

	Args:
		record: An OASA molecule, atom, or bond.
		attribute_name: Name of the optional attribute.

	Returns:
		The attribute value, or None when OASA never set it.
	"""
	if not hasattr(record, attribute_name):
		return None
	value = getattr(record, attribute_name)
	return value


#============================================
def round_coordinate(value: object) -> float:
	"""Return one point coordinate rounded to a stable decimal representation."""
	rounded_value = round(float(value), 6)
	return rounded_value


#============================================
def project_atom(atom: object, index: int) -> dict:
	"""Return the shared carrier fields OASA holds for one atom."""
	projected_atom = {
		"index": index,
		"id": optional_attribute(atom, "id"),
		"symbol": atom.symbol,
		"element": atom.symbol,
		"formal_charge": atom.charge,
		"explicit_hydrogens": atom.explicit_hydrogens,
		"isotope": atom.isotope,
		"valence": atom.valency,
		"multiplicity": atom.multiplicity,
		"free_sites": atom.free_sites,
		"x": round_coordinate(atom.x),
		"y": round_coordinate(atom.y),
		"z": round_coordinate(atom.z),
	}
	return projected_atom


#============================================
def project_bond(bond: object, atom_index_by_object_id: dict) -> dict:
	"""Return the shared carrier fields OASA holds for one bond."""
	start_atom = bond.vertices[0]
	end_atom = bond.vertices[1]
	projected_bond = {
		"id": optional_attribute(bond, "id"),
		"start": atom_index_by_object_id[id(start_atom)],
		"end": atom_index_by_object_id[id(end_atom)],
		"type": bond.type,
		"order": bond.order,
	}
	return projected_bond


#============================================
def bond_sort_key(projected_bond: dict) -> tuple:
	"""Return a total ordering key so set-stored bonds emit deterministically."""
	bond_id = projected_bond["id"]
	if bond_id is None:
		bond_id = ""
	sort_key = (
		projected_bond["start"],
		projected_bond["end"],
		bond_id,
		projected_bond["type"],
		projected_bond["order"],
	)
	return sort_key


#============================================
def project_molecule(molecule: object, index: int) -> dict:
	"""Return one molecule projection with ordered atoms and sorted bonds."""
	atoms = list(molecule.vertices)
	# map Python object identity to emitted index so bonds can name endpoints
	atom_index_by_object_id = {}
	for atom_index, atom in enumerate(atoms):
		atom_index_by_object_id[id(atom)] = atom_index
	projected_atoms = []
	for atom_index, atom in enumerate(atoms):
		projected_atoms.append(project_atom(atom, atom_index))
	projected_bonds = []
	for bond in molecule.edges:
		projected_bonds.append(project_bond(bond, atom_index_by_object_id))
	projected_bonds.sort(key=bond_sort_key)
	projected_molecule = {
		"index": index,
		"id": optional_attribute(molecule, "id"),
		"name": optional_attribute(molecule, "name"),
		"atoms": projected_atoms,
		"bonds": projected_bonds,
	}
	return projected_molecule


#============================================
def project_request(request: dict) -> dict:
	"""Read the requested corpus file and project every molecule OASA yields."""
	corpus_path = request["corpus_path"]
	with open(corpus_path, "rb") as corpus_handle:
		corpus_bytes = corpus_handle.read()
	molecules = list(oasa.cdml.read_cdml(corpus_bytes))
	projected_molecules = []
	for molecule_index, molecule in enumerate(molecules):
		projected_molecules.append(project_molecule(molecule, molecule_index))
	projection = {
		"capability": request["capability"],
		"corpus_path": corpus_path,
		"coordinate_unit": "postscript_point",
		"molecules": projected_molecules,
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
