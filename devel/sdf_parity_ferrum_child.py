"""Emit ordered multi-record SDF through the installed Ferrum extension."""

# Standard Library
import json
import math
import pathlib
import sys

# PIP3 modules
import ferrum_chem


#============================================
def _fail(message: str) -> None:
	"""Write one diagnostic and terminate without partial SDF output."""
	print("Ferrum SDF error: " + message, file=sys.stderr)
	raise SystemExit(1)


#============================================
def _request() -> dict:
	"""Read the closed ordered-record request."""
	try:
		value = json.loads(sys.stdin.read())
	except json.JSONDecodeError as error:
		_fail("request is not valid JSON: " + error.msg)
	if not isinstance(value, dict) or value.get("schema") != "ferrum-sdf-parity-request-v1":
		_fail("request has an unknown schema")
	records = value.get("records")
	if not isinstance(records, list) or not records:
		_fail("request needs a nonempty record list")
	return value


#============================================
def _prepare_record(record: dict) -> ferrum_chem.SdfRecordV1:
	"""Build one frozen record while preserving authored property order."""
	if not isinstance(record, dict):
		_fail("each record must be an object")
	smiles = record.get("smiles")
	title = record.get("title")
	properties = record.get("properties")
	if type(smiles) is not str or type(title) is not str or not isinstance(properties, list):
		_fail("each record needs smiles, title, and property fields")
	pairs = []
	for property_value in properties:
		if (
			not isinstance(property_value, list)
			or len(property_value) != 2
			or any(type(value) is not str for value in property_value)
		):
			_fail("each SDF property must be one string pair")
		pairs.append((property_value[0], property_value[1]))
	molecule = ferrum_chem.parse_smiles(smiles)
	return ferrum_chem.prepare_sdf_record(molecule, title, tuple(pairs))


#============================================
def _import_facts(sdf: str, expected: list[dict]) -> list[dict]:
	"""Require native import to retain complete ordered source facts."""
	imported = ferrum_chem.sdf_to_records(sdf)
	if len(imported) != len(expected):
		_fail("native SDF import changed record count")
	rows = []
	for record, source in zip(imported, expected, strict=True):
		properties = [[item.name, item.value] for item in record.properties]
		if record.title != source["title"] or properties != source["properties"]:
			_fail("native SDF import changed ordered text facts")
		coordinates = record.molecule.coordinates
		if not coordinates or any(
			not math.isfinite(point.x) or not math.isfinite(point.y)
			for point in coordinates
		):
			_fail("native SDF import omitted finite atom-aligned coordinates")
		rows.append({
			"canonical_smiles": record.molecule.canonical_smiles,
			"properties": properties,
			"title": record.title,
		})
	return rows


#============================================
def main() -> int:
	"""Export the same ordered record sequence in both explicit formats."""
	request = _request()
	records = tuple(_prepare_record(record) for record in request["records"])
	sdf = {
		"v2000": ferrum_chem.records_to_sdf(
			records, ferrum_chem.MolblockVersionV1.v2000,
		),
		"v3000": ferrum_chem.records_to_sdf(
			records, ferrum_chem.MolblockVersionV1.v3000,
		),
	}
	response = {
		"backend": "ferrum-abi4",
		"binary": str(pathlib.Path(ferrum_chem.__file__).resolve()),
		"imported": {
			version: _import_facts(text, request["records"])
			for version, text in sdf.items()
		},
		"schema": "ferrum-sdf-parity-ferrum-v1",
		"sdf": sdf,
		"version": 4,
	}
	print(json.dumps(response, allow_nan=False, separators=(",", ":"), sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
