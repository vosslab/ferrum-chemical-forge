"""Evaluate Ferrum and Python-RDKit molblocks by molecular meaning."""

# Standard Library
import decimal
import hashlib
import json
import math
import pathlib
import sys

# PIP3 modules
import rdkit
import rdkit.rdBase
from rdkit import Chem
from rdkit.Chem import rdDepictor


#============================================
def _fail(message: str) -> None:
	"""Write one diagnostic and terminate without a partial result."""
	print("RDKit molblock evaluator error: " + message, file=sys.stderr)
	raise SystemExit(1)


#============================================
def _request() -> dict:
	"""Read and validate one evaluator request."""
	try:
		value = json.loads(sys.stdin.read())
	except json.JSONDecodeError as error:
		_fail("request is not valid JSON: " + error.msg)
	if not isinstance(value, dict):
		_fail("request must be a JSON object")
	if value.get("schema") != "ferrum-molblock-evaluation-request-v1":
		_fail("request has an unknown schema")
	molecules = value.get("molecules")
	if not isinstance(molecules, list) or not molecules:
		_fail("request needs a nonempty molecule list")
	return value


#============================================
def _coordinates(molecule: Chem.Mol) -> list[list[float]]:
	"""Copy finite x/y coordinates in atom order."""
	conformer = molecule.GetConformer()
	points = []
	for index in range(molecule.GetNumAtoms()):
		point = conformer.GetAtomPosition(index)
		if not math.isfinite(point.x) or not math.isfinite(point.y):
			_fail("molblock contains a nonfinite coordinate")
		points.append([float(point.x), float(point.y)])
	return points


#============================================
def _atom_facts(atom: Chem.Atom) -> dict:
	"""Return discrete atom facts with file-format meaning."""
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
	"""Return discrete bond facts independent of undirected record orientation."""
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
	"""Return the comparison facts intentionally independent of molblock text."""
	Chem.AssignStereochemistry(molecule, cleanIt=True, force=True)
	return {
		"atoms": [_atom_facts(atom) for atom in molecule.GetAtoms()],
		"bonds": [_bond_facts(bond) for bond in molecule.GetBonds()],
		"canonical_smiles": Chem.MolToSmiles(
			molecule, canonical=True, isomericSmiles=True,
		),
	}


#============================================
def _coordinate_tokens(molblock: str, version: str, atom_count: int) -> list[list[str]]:
	"""Extract the written x/y decimals without assuming a global precision."""
	lines = molblock.splitlines()
	if version == "v2000":
		counts = next(
			(index for index, line in enumerate(lines) if "V2000" in line), None,
		)
		if counts is None:
			_fail("V2000 output has no V2000 counts line")
		atom_lines = lines[counts + 1:counts + 1 + atom_count]
		if len(atom_lines) != atom_count:
			_fail("V2000 output has incomplete atom lines")
		return [[line[:10].strip(), line[10:20].strip()] for line in atom_lines]
	if "V3000" not in molblock or "M  V30 BEGIN CTAB" not in molblock:
		_fail("V3000 output lacks its required syntax markers")
	start = lines.index("M  V30 BEGIN ATOM")
	end = lines.index("M  V30 END ATOM")
	atom_lines = lines[start + 1:end]
	if len(atom_lines) != atom_count:
		_fail("V3000 output has incomplete atom lines")
	tokens = [line.split() for line in atom_lines]
	if any(len(fields) < 7 for fields in tokens):
		_fail("V3000 output has a malformed atom line")
	return [[fields[4], fields[5]] for fields in tokens]


#============================================
def _half_quantum(token: str) -> float:
	"""Derive one rounding bound from the actual written decimal token."""
	try:
		value = decimal.Decimal(token)
	except decimal.InvalidOperation:
		_fail("molblock coordinate is not a decimal number")
	if not value.is_finite():
		_fail("molblock coordinate token is not finite")
	quantum = decimal.Decimal(1).scaleb(value.as_tuple().exponent)
	return float(quantum.copy_abs() / 2)


#============================================
def _coordinate_evidence(
	molblock: str, version: str, source: list[list[float]], parsed: list[list[float]],
) -> dict:
	"""Compare coordinates using bounds derived from each emitted token."""
	if len(source) != len(parsed):
		_fail("molblock changed the atom count")
	tokens = _coordinate_tokens(molblock, version, len(source))
	maximum_delta = 0.0
	maximum_bound = 0.0
	for source_point, parsed_point, token_point in zip(
		source, parsed, tokens, strict=True,
	):
		for original, restored, token in zip(
			source_point, parsed_point, token_point, strict=True,
		):
			bound = _half_quantum(token) + max(math.ulp(original), math.ulp(restored))
			delta = abs(original - restored)
			if delta > bound:
				_fail("molblock coordinate exceeds its written decimal precision")
			maximum_delta = max(maximum_delta, delta)
			maximum_bound = max(maximum_bound, bound)
	return {
		"derived_max_abs_bound": maximum_bound,
		"observed_max_abs_delta": maximum_delta,
		"passed": True,
	}


#============================================
def _evaluate_block(molblock: str, version: str, source: list[list[float]]) -> dict:
	"""Parse one block strictly and return semantic and precision evidence."""
	if type(molblock) is not str or not molblock.endswith("\n"):
		_fail("molblock must be newline-terminated text")
	molecule = Chem.MolFromMolBlock(
		molblock, sanitize=True, removeHs=False, strictParsing=True,
	)
	if molecule is None:
		_fail("RDKit rejected a generated " + version + " molblock")
	coordinates = _coordinates(molecule)
	return {
		"coordinates": _coordinate_evidence(molblock, version, source, coordinates),
		"semantic": _semantic_facts(molecule),
		"text_sha256": hashlib.sha256(molblock.encode("utf-8")).hexdigest(),
	}


#============================================
def _source_molecule(smiles: str) -> Chem.Mol:
	"""Parse and depict one source with the adapter's explicit defaults."""
	molecule = Chem.MolFromSmiles(smiles)
	if molecule is None:
		_fail("RDKit rejected source SMILES")
	rdDepictor.Compute2DCoords(
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
	return molecule


#============================================
def _evaluate_record(record: dict) -> dict:
	"""Evaluate both Ferrum blocks and both Python-wrapper writer controls."""
	name = record.get("name")
	smiles = record.get("smiles")
	ferrum_coordinates = record.get("coordinates")
	ferrum_blocks = record.get("molblocks")
	if type(name) is not str or type(smiles) is not str:
		_fail("each evaluator record needs string name and smiles fields")
	if not isinstance(ferrum_coordinates, list) or not isinstance(ferrum_blocks, dict):
		_fail("each evaluator record needs Ferrum coordinates and molblocks")
	source = _source_molecule(smiles)
	oracle_coordinates = _coordinates(source)
	formats = {}
	for version, force_v3000 in (("v2000", False), ("v3000", True)):
		if type(ferrum_blocks.get(version)) is not str:
			_fail("Ferrum omitted " + version)
		oracle_block = Chem.MolToMolBlock(
			source, includeStereo=True, confId=-1, kekulize=True,
			forceV3000=force_v3000,
		)
		ferrum = _evaluate_block(ferrum_blocks[version], version, ferrum_coordinates)
		oracle = _evaluate_block(oracle_block, version, oracle_coordinates)
		formats[version] = {
			"ferrum": ferrum,
			"oracle": oracle,
			"text_exact_observation": ferrum["text_sha256"] == oracle["text_sha256"],
		}
	return {
		"formats": formats,
		"name": name,
		"oracle_coordinates": oracle_coordinates,
		"source_semantic": _semantic_facts(source),
	}


#============================================
def main() -> int:
	"""Evaluate the complete request and emit one inspectable JSON result."""
	request = _request()
	results = [_evaluate_record(record) for record in request["molecules"]]
	binary = pathlib.Path(rdkit.rdBase.__file__).resolve()
	response = {
		"backend": "rdkit-python-wrapper",
		"binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
		"molecules": results,
		"rdkit_version": rdkit.__version__,
		"schema": "ferrum-molblock-evaluation-v1",
	}
	print(json.dumps(response, allow_nan=False, separators=(",", ":"), sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
