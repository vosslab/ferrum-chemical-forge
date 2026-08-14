"""Measure ordered multi-record SDF semantics across isolated RDKit versions."""

# Standard Library
import argparse
import hashlib
import json
import os
import pathlib
import platform
import subprocess


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
FERRUM_CHILD = REPO_ROOT / "devel" / "sdf_parity_ferrum_child.py"
RDKIT_CHILD = REPO_ROOT / "devel" / "sdf_parity_rdkit_child.py"
DEFAULT_REPORT = REPO_ROOT / "docs" / "active_plans" / "reports" / "sdf_codec_v1.json"
BUILD_RDKIT_VERSION = "2026.03.5"
RECORDS = (
	{
		"name": "ethanol",
		"properties": [["source", "Ferrum"], ["note", "line one\nline two"]],
		"smiles": "CCO",
		"title": "ethanol",
	},
	{
		"name": "charged_pair",
		"properties": [["category", "salt"], ["empty", ""]],
		"smiles": "[NH4+].[Cl-]",
		"title": "charged pair",
	},
	{
		"name": "isotope_chirality_map",
		"properties": [["identifier", "mapped chiral"], ["sequence", "third"]],
		"smiles": "[13CH3][C@H](F)[C:9](=O)[O-]",
		"title": "isotope chirality map",
	},
)
SOURCE_PATHS = (
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_sdf.cpp",
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_sdf_import.cpp",
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_molecule_response.cpp",
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_molblock.cpp",
	"packages/ferrum-rust/crates/chemistry/native/include/ferrum_chem_adapter.h",
	"packages/ferrum-rust/crates/chemistry/src/native_engine/sdf_wire.rs",
	"packages/ferrum-rust/crates/chemistry/src/native_engine/sdf_import.rs",
	"packages/ferrum-rust/crates/chemistry/src/sdf.rs",
	"packages/ferrum-rust/crates/api/python/src/chemistry_binding.rs",
	"devel/measure_sdf_codec_parity.py",
	"devel/sdf_parity_ferrum_child.py",
	"devel/sdf_parity_rdkit_child.py",
)


#============================================
class SdfParityError(RuntimeError):
	"""The SDF differential protocol or result is invalid."""


#============================================
def _sha256(path: pathlib.Path) -> str:
	"""Return the digest of one required regular file."""
	if not path.is_file():
		raise SdfParityError("required parity input is not a regular file: " + str(path))
	return hashlib.sha256(path.read_bytes()).hexdigest()


#============================================
def _display_path(path: pathlib.Path) -> str:
	"""Return a repository-relative path when possible."""
	resolved = path.resolve()
	try:
		return str(resolved.relative_to(REPO_ROOT))
	except ValueError:
		return str(resolved)


#============================================
def _run_child(python: pathlib.Path, child: pathlib.Path, request: dict) -> dict:
	"""Run one isolated child and require exactly one JSON object."""
	environment = os.environ.copy()
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	result = subprocess.run(
		[str(python), "-I", "-B", str(child)],
		cwd=REPO_ROOT,
		env=environment,
		input=json.dumps(request, allow_nan=False, separators=(",", ":"), sort_keys=True) + "\n",
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)
	if result.returncode:
		raise SdfParityError(child.name + " failed: " + result.stderr.strip())
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise SdfParityError(child.name + " must emit exactly one JSON line")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise SdfParityError(child.name + " emitted invalid JSON") from error
	if not isinstance(value, dict):
		raise SdfParityError(child.name + " output must be a JSON object")
	return value


#============================================
def _validate_ferrum(value: dict) -> dict:
	"""Validate installed-extension identity and complete SDF output."""
	if value.get("schema") != "ferrum-sdf-parity-ferrum-v1":
		raise SdfParityError("Ferrum returned an unknown schema")
	if value.get("backend") != "ferrum-abi4" or value.get("version") != 4:
		raise SdfParityError("Ferrum returned an invalid backend identity")
	binary = value.get("binary")
	sdf = value.get("sdf")
	imported = value.get("imported")
	if type(binary) is not str or not isinstance(sdf, dict) or not isinstance(imported, dict):
		raise SdfParityError("Ferrum omitted its binary or SDF output")
	if any(type(sdf.get(version)) is not str for version in ("v2000", "v3000")):
		raise SdfParityError("Ferrum omitted an explicit SDF format")
	for version in ("v2000", "v3000"):
		rows = imported.get(version)
		if not isinstance(rows, list) or len(rows) != len(RECORDS):
			raise SdfParityError("Ferrum import changed record count")
		for expected, actual in zip(RECORDS, rows, strict=True):
			if not isinstance(actual, dict) or actual.get("title") != expected["title"]:
				raise SdfParityError("Ferrum import changed record order or title")
			if actual.get("properties") != expected["properties"]:
				raise SdfParityError("Ferrum import changed property order or value")
			if type(actual.get("canonical_smiles")) is not str:
				raise SdfParityError("Ferrum import omitted canonical molecular meaning")
	return {
		"binary_sha256": _sha256(pathlib.Path(binary)),
		"imported": imported,
		"sdf": sdf,
	}


#============================================
def _validate_import_against_rdkit(ferrum: dict, evaluation: dict) -> None:
	"""Require Ferrum and the selected RDKit evaluator to agree on meaning."""
	for version in ("v2000", "v3000"):
		ferrum_rows = ferrum["imported"][version]
		rdkit_rows = evaluation["formats"][version]["records"]
		for ferrum_row, rdkit_row in zip(ferrum_rows, rdkit_rows, strict=True):
			if ferrum_row["canonical_smiles"] != rdkit_row["canonical_smiles"]:
				raise SdfParityError("Ferrum and RDKit disagree on imported molecular meaning")


#============================================
def _validate_evaluation(value: dict, expected_version: str, label: str) -> dict:
	"""Require semantic and ordered-property round trips for both formats."""
	if value.get("schema") != "ferrum-sdf-evaluation-v1":
		raise SdfParityError(label + " returned an unknown schema")
	if value.get("backend") != "rdkit-python-wrapper":
		raise SdfParityError(label + " returned an unexpected backend")
	if value.get("rdkit_version") != expected_version:
		raise SdfParityError(label + " returned RDKit " + str(value.get("rdkit_version")))
	digest = value.get("binary_sha256")
	formats = value.get("formats")
	if type(digest) is not str or len(digest) != 64 or not isinstance(formats, dict):
		raise SdfParityError(label + " omitted its binary or format evidence")
	for version in ("v2000", "v3000"):
		format_value = formats.get(version)
		if not isinstance(format_value, dict) or format_value.get("semantic_round_trip") is not True:
			raise SdfParityError(label + " failed " + version + " semantic round trip")
		records = format_value.get("records")
		if not isinstance(records, list) or len(records) != len(RECORDS):
			raise SdfParityError(label + " changed record count")
		for expected, actual in zip(RECORDS, records, strict=True):
			if not isinstance(actual, dict) or actual.get("title") != expected["title"]:
				raise SdfParityError(label + " changed record order or title")
			if actual.get("properties") != [pair[0] for pair in expected["properties"]]:
				raise SdfParityError(label + " changed property order")
			if actual.get("semantic_round_trip") is not True:
				raise SdfParityError(label + " changed molecular meaning")
	return {
		"binary_sha256": digest,
		"formats": formats,
		"rdkit_version": expected_version,
	}


#============================================
def _native_e2e(path: pathlib.Path, wheel_sha256: str) -> dict:
	"""Require SDF success before and after a distinct adapter replacement."""
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except (OSError, json.JSONDecodeError) as error:
		raise SdfParityError("native E2E receipt is unreadable") from error
	if value.get("schema") != "ferrum-native-wheel-e2e-evidence-v4":
		raise SdfParityError("native E2E receipt has an unknown schema")
	wheel = value.get("wheel")
	chemistry = value.get("chemistry")
	if not isinstance(wheel, dict) or wheel.get("sha256") != wheel_sha256:
		raise SdfParityError("native E2E receipt describes a different wheel")
	if not isinstance(chemistry, dict):
		raise SdfParityError("native E2E receipt omits chemistry probes")
	for key in ("python_extension_before", "python_extension_after"):
		probe = chemistry.get(key)
		if (
			not isinstance(probe, dict)
			or probe.get("sdf_record_semantic_markers") is not True
			or probe.get("sdf_import_semantics") is not True
		):
			raise SdfParityError("native E2E receipt omits SDF replacement proof")
	return {"adapter_replacement": True, "receipt": _display_path(path)}


#============================================
def main() -> int:
	"""Run the semantic differential and publish its source-bound receipt."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--oracle-python", required=True, type=pathlib.Path)
	parser.add_argument("--cross-version-python", type=pathlib.Path)
	parser.add_argument("--ferrum-python", required=True, type=pathlib.Path)
	parser.add_argument("--wheel", required=True, type=pathlib.Path)
	parser.add_argument("--native-e2e-receipt", required=True, type=pathlib.Path)
	parser.add_argument("--report", default=DEFAULT_REPORT, type=pathlib.Path)
	arguments = parser.parse_args()
	interpreters = [arguments.oracle_python, arguments.ferrum_python]
	if arguments.cross_version_python is not None:
		interpreters.append(arguments.cross_version_python)
	if any(not pathlib.Path(os.path.abspath(path)).is_file() for path in interpreters):
		raise SdfParityError("every Python interpreter must be an existing file")
	wheel = arguments.wheel.resolve()
	wheel_sha256 = _sha256(wheel)
	request = {"records": list(RECORDS), "schema": "ferrum-sdf-parity-request-v1"}
	ferrum = _validate_ferrum(_run_child(arguments.ferrum_python, FERRUM_CHILD, request))
	evaluation_request = {
		"records": list(RECORDS),
		"schema": "ferrum-sdf-evaluation-request-v1",
		"sdf": ferrum["sdf"],
	}
	current = _validate_evaluation(
		_run_child(arguments.oracle_python, RDKIT_CHILD, evaluation_request),
		BUILD_RDKIT_VERSION,
		"current build RDKit",
	)
	_validate_import_against_rdkit(ferrum, current)
	evaluations = [current]
	if arguments.cross_version_python is not None:
		cross_value = _run_child(
			arguments.cross_version_python, RDKIT_CHILD, evaluation_request,
		)
		cross_version = cross_value.get("rdkit_version")
		if type(cross_version) is not str or cross_version == BUILD_RDKIT_VERSION:
			raise SdfParityError("cross-version evaluator is not a distinct RDKit version")
		evaluations.append(
			_validate_evaluation(cross_value, cross_version, "cross-version RDKit"),
		)
	receipt = {
		"artifacts": {
			"ferrum_extension_sha256": ferrum["binary_sha256"],
			"wheel": _display_path(wheel),
			"wheel_sha256": wheel_sha256,
		},
		"comparison_policy": {
			"molecule": "exact discrete chemistry after strict parse and sanitization",
			"properties": "exact authored record order, title, property order, and values",
			"text": "not compared; legal writer formatting is outside the contract",
		},
		"evaluations": evaluations,
		"ferrum_import": ferrum["imported"],
		"native_wheel_e2e": _native_e2e(arguments.native_e2e_receipt, wheel_sha256),
		"build_rdkit_version": BUILD_RDKIT_VERSION,
		"compatibility_policy": "refresh the build to current stable and retain previous stable",
		"platform": {"machine": platform.machine(), "system": platform.system()},
		"records": list(RECORDS),
		"schema": "ferrum-sdf-codec-parity-v1",
		"source_sha256": {path: _sha256(REPO_ROOT / path) for path in SOURCE_PATHS},
		"status": "semantic-parity",
	}
	arguments.report.parent.mkdir(parents=True, exist_ok=True)
	arguments.report.write_text(
		json.dumps(receipt, allow_nan=False, indent=2, sort_keys=True) + "\n",
		encoding="ascii",
	)
	print(json.dumps({
		"report": _display_path(arguments.report),
		"schema": receipt["schema"],
		"status": receipt["status"],
		"versions": [evaluation["rdkit_version"] for evaluation in evaluations],
	}, separators=(",", ":"), sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
