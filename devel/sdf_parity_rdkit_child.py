"""Strictly parse Ferrum SDF and compare chemical and ordered property facts."""

# Standard Library
import hashlib
import io
import json
import math
import pathlib
import sys

# PIP3 modules
import rdkit
import rdkit.rdBase
from rdkit import Chem


#============================================
def _fail(message: str) -> None:
	"""Write one diagnostic and terminate without partial comparison output."""
	print("RDKit SDF evaluator error: " + message, file=sys.stderr)
	raise SystemExit(1)


#============================================
def _request() -> dict:
	"""Read one closed evaluation request."""
	try:
		value = json.loads(sys.stdin.read())
	except json.JSONDecodeError as error:
		_fail("request is not valid JSON: " + error.msg)
	if not isinstance(value, dict) or value.get("schema") != "ferrum-sdf-evaluation-request-v1":
		_fail("request has an unknown schema")
	if not isinstance(value.get("records"), list) or not isinstance(value.get("sdf"), dict):
		_fail("request omits records or SDF text")
	return value


#============================================
def _atom_facts(atom: Chem.Atom) -> dict:
	"""Return discrete atom facts with SDF chemical meaning."""
	return {
		"atom_map_number": atom.GetAtomMapNum(),
		"atomic_number": atom.GetAtomicNum(),
		"chirality": str(atom.GetChiralTag()),
		"formal_charge": atom.GetFormalCharge(),
		"isotope": atom.GetIsotope(),
		"radical_electrons": atom.GetNumRadicalElectrons(),
		"total_hydrogens": atom.GetTotalNumHs(includeNeighbors=True),
	}


#============================================
def _bond_facts(bond: Chem.Bond) -> dict:
	"""Return bond facts independent of undirected record orientation."""
	endpoints = sorted((bond.GetBeginAtomIdx(), bond.GetEndAtomIdx()))
	return {
		"aromatic": bond.GetIsAromatic(),
		"begin": endpoints[0],
		"end": endpoints[1],
		"order": str(bond.GetBondType()),
		"stereo": str(bond.GetStereo()),
	}


#============================================
def _semantic_facts(molecule: Chem.Mol) -> dict:
	"""Return chemistry independent of SDF layout and header text."""
	Chem.AssignStereochemistry(molecule, cleanIt=True, force=True)
	return {
		"atoms": [_atom_facts(atom) for atom in molecule.GetAtoms()],
		"bonds": [_bond_facts(bond) for bond in molecule.GetBonds()],
		"canonical_smiles": Chem.MolToSmiles(
			molecule, canonical=True, isomericSmiles=True,
		),
	}


#============================================
def _finite_coordinates(molecule: Chem.Mol) -> bool:
	"""Require one finite x/y coordinate for every parsed atom."""
	if molecule.GetNumConformers() != 1:
		return False
	conformer = molecule.GetConformer()
	return all(
		math.isfinite(conformer.GetAtomPosition(index).x)
		and math.isfinite(conformer.GetAtomPosition(index).y)
		for index in range(molecule.GetNumAtoms())
	)


#============================================
def _parse_sdf(text: str) -> list[Chem.Mol]:
	"""Parse every record strictly without dropping authored hydrogens."""
	if type(text) is not str or not text.endswith("$$$$\n"):
		_fail("SDF text is not terminated by a record delimiter")
	supplier = Chem.ForwardSDMolSupplier(
		io.BytesIO(text.encode("utf-8")),
		sanitize=True,
		removeHs=False,
		strictParsing=True,
	)
	molecules = list(supplier)
	if any(molecule is None for molecule in molecules):
		_fail("strict supplier rejected an SDF record")
	return molecules


#============================================
def _evaluate_record(expected: dict, molecule: Chem.Mol) -> dict:
	"""Compare one parsed record with its authored source and property sequence."""
	if not isinstance(expected, dict):
		_fail("record expectation must be an object")
	smiles = expected.get("smiles")
	title = expected.get("title")
	properties = expected.get("properties")
	if type(smiles) is not str or type(title) is not str or not isinstance(properties, list):
		_fail("record expectation is incomplete")
	source = Chem.MolFromSmiles(smiles)
	if source is None:
		_fail("RDKit rejected source SMILES")
	if _semantic_facts(molecule) != _semantic_facts(source):
		_fail("SDF round trip changed molecular meaning")
	if not molecule.HasProp("_Name") or molecule.GetProp("_Name") != title:
		_fail("SDF round trip changed the record title")
	expected_names = [property_value[0] for property_value in properties]
	actual_names = list(molecule.GetPropNames(includePrivate=False, includeComputed=False))
	if actual_names != expected_names:
		_fail("SDF round trip changed property order or names")
	for name, value in properties:
		if molecule.GetProp(name) != value:
			_fail("SDF round trip changed property value for " + name)
	if not _finite_coordinates(molecule):
		_fail("SDF round trip omitted finite atom-aligned coordinates")
	return {
		"canonical_smiles": _semantic_facts(molecule)["canonical_smiles"],
		"properties": expected_names,
		"semantic_round_trip": True,
		"title": title,
	}


#============================================
def main() -> int:
	"""Evaluate both explicit formats and emit inspectable semantic evidence."""
	request = _request()
	formats = {}
	for version in ("v2000", "v3000"):
		text = request["sdf"].get(version)
		molecules = _parse_sdf(text)
		if len(molecules) != len(request["records"]):
			_fail("SDF round trip changed record count")
		rows = [
			_evaluate_record(expected, molecule)
			for expected, molecule in zip(request["records"], molecules, strict=True)
		]
		markers = "V2000" if version == "v2000" else "V3000"
		if any(markers not in block for block in text.split("$$$$\n")[:-1]):
			_fail("SDF records do not use the requested molfile syntax")
		formats[version] = {"records": rows, "semantic_round_trip": True}
	binary = pathlib.Path(rdkit.rdBase.__file__).resolve()
	response = {
		"backend": "rdkit-python-wrapper",
		"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
		"formats": formats,
		"rdkit_version": rdkit.__version__,
		"schema": "ferrum-sdf-evaluation-v1",
	}
	print(json.dumps(response, allow_nan=False, separators=(",", ":"), sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
