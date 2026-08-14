"""Measure the build-recorded M5 SMARTS export across process boundaries."""

# Standard Library
import argparse
import hashlib
import json
import os
import pathlib
import subprocess


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
ORACLE_CHILD = REPO_ROOT / "devel" / "smarts_parity_oracle_child.py"
FERRUM_CHILD = REPO_ROOT / "devel" / "smarts_parity_ferrum_child.py"
DEFAULT_REPORT = REPO_ROOT / "docs" / "active_plans" / "reports" / "smarts_codec_v1.json"
EXPECTED_RDKIT_VERSION = "2026.03.5"
CORPUS = (
	{"name": "ethanol", "smiles": "CCO"},
	{"name": "charged_pair", "smiles": "[NH4+].[Cl-]"},
	{"name": "carbonyl", "smiles": "C=O"},
	{"name": "nitrile", "smiles": "C#N"},
	{"name": "benzene", "smiles": "c1ccccc1"},
	{"name": "bond_stereo", "smiles": "F/C=C/F"},
	{
		"name": "isotope_chirality_map",
		"smiles": "[13CH3][C@H](F)[C:9](=O)[O-]",
	},
	{"name": "explicit_methylene", "smiles": "[CH2]"},
)
SOURCE_PATHS = (
	"packages/ferrum-rust/crates/chemistry/native/ferrum_chem_smarts.cpp",
	"packages/ferrum-rust/crates/chemistry/native/include/ferrum_chem_adapter.h",
	"packages/ferrum-rust/crates/chemistry/src/native_engine/graph_wire.rs",
	"packages/ferrum-rust/crates/chemistry/src/native_engine/text_response.rs",
	"packages/ferrum-rust/crates/api/python/src/chemistry_binding.rs",
	"devel/measure_smarts_codec_parity.py",
	"devel/smarts_parity_ferrum_child.py",
	"devel/smarts_parity_oracle_child.py",
)


#============================================
class SmartsParityError(RuntimeError):
	"""The SMARTS differential protocol or result is invalid."""


#============================================
def _sha256(path: pathlib.Path) -> str:
	"""Return the SHA-256 digest of one required regular file."""
	if not path.is_file():
		raise SmartsParityError("required parity input is not a regular file: " + str(path))
	return hashlib.sha256(path.read_bytes()).hexdigest()


#============================================
def _request_text() -> str:
	"""Return the one-line request shared by both children."""
	return json.dumps({
		"molecules": list(CORPUS),
		"schema": "ferrum-smarts-parity-request-v1",
	}, separators=(",", ":"), sort_keys=True) + "\n"


#============================================
def _run_child(python: pathlib.Path, child: pathlib.Path) -> dict:
	"""Run one isolated child and require exactly one JSON object."""
	environment = os.environ.copy()
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	result = subprocess.run(
		[str(python), "-I", "-B", str(child)],
		cwd=REPO_ROOT,
		env=environment,
		input=_request_text(),
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)
	if result.returncode:
		raise SmartsParityError(child.name + " failed: " + result.stderr.strip())
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise SmartsParityError(child.name + " must emit exactly one JSON line")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise SmartsParityError(child.name + " emitted invalid JSON") from error
	if not isinstance(value, dict):
		raise SmartsParityError(child.name + " output must be a JSON object")
	return value


#============================================
def _validate_child(value: dict, backend: str) -> dict:
	"""Validate one complete child response and bind its binary digest."""
	if value.get("schema") != "ferrum-smarts-parity-child-v1":
		raise SmartsParityError(backend + " returned an unknown schema")
	if value.get("backend") != backend:
		raise SmartsParityError(backend + " returned an unexpected backend name")
	if backend == "rdkit-python-wrapper" and value.get("version") != EXPECTED_RDKIT_VERSION:
		raise SmartsParityError(
			"oracle RDKit version must be " + EXPECTED_RDKIT_VERSION,
		)
	if backend == "ferrum-abi4" and value.get("version") != 4:
		raise SmartsParityError("Ferrum child did not report ABI 4")
	binary = value.get("binary")
	if not isinstance(binary, str):
		raise SmartsParityError(backend + " omitted its binary path")
	molecules = value.get("molecules")
	if not isinstance(molecules, list) or len(molecules) != len(CORPUS):
		raise SmartsParityError(backend + " returned the wrong corpus size")
	for expected, actual in zip(CORPUS, molecules, strict=True):
		if not isinstance(actual, dict) or actual.get("name") != expected["name"]:
			raise SmartsParityError(backend + " changed corpus order or identity")
		for field in ("canonical_smiles", "smarts"):
			if not isinstance(actual.get(field), str) or not actual[field]:
				raise SmartsParityError(backend + " returned an invalid " + field)
	return {
		"binary_sha256": _sha256(pathlib.Path(binary)),
		"molecules": molecules,
		"version": value["version"],
	}


#============================================
def _compare(oracle: dict, ferrum: dict) -> list[dict]:
	"""Require same-build exact string parity and return inspectable rows."""
	rows = []
	for source, expected, actual in zip(
		CORPUS, oracle["molecules"], ferrum["molecules"], strict=True,
	):
		canonical_equal = expected["canonical_smiles"] == actual["canonical_smiles"]
		smarts_equal = expected["smarts"] == actual["smarts"]
		rows.append({
			"canonical_smiles": expected["canonical_smiles"],
			"canonical_smiles_exact": canonical_equal,
			"input_smiles": source["smiles"],
			"name": source["name"],
			"smarts": expected["smarts"],
			"smarts_exact": smarts_equal,
		})
		if not canonical_equal or not smarts_equal:
			raise SmartsParityError("same-build SMARTS divergence for " + source["name"])
	return rows


#============================================
def _native_e2e_facts(path: pathlib.Path, wheel_sha256: str) -> dict:
	"""Retain the source E2E's closure and distinct-adapter SMARTS proof."""
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except (OSError, json.JSONDecodeError) as error:
		raise SmartsParityError("native E2E receipt is unreadable") from error
	if not isinstance(value, dict) or value.get("schema") != "ferrum-native-wheel-e2e-evidence-v4":
		raise SmartsParityError("native E2E receipt has an unknown schema")
	wheel = value.get("wheel")
	chemistry = value.get("chemistry")
	closure = value.get("closure")
	replacement = value.get("replacement_proof")
	if not all(isinstance(entry, dict) for entry in (wheel, chemistry, closure, replacement)):
		raise SmartsParityError("native E2E receipt omits required evidence")
	if wheel.get("sha256") != wheel_sha256:
		raise SmartsParityError("native E2E receipt describes a different wheel")
	before = chemistry.get("python_extension_before")
	after = chemistry.get("python_extension_after")
	if not isinstance(before, dict) or not isinstance(after, dict):
		raise SmartsParityError("native E2E receipt omits Python chemistry probes")
	if before.get("smarts") != "[#6]-[#6]-[#8]" or after.get("smarts") != before["smarts"]:
		raise SmartsParityError("native E2E receipt did not preserve SMARTS after replacement")
	names = closure.get("names")
	if not isinstance(names, list) or not all(isinstance(name, str) for name in names):
		raise SmartsParityError("native E2E receipt has an invalid closure")
	original = replacement.get("original_sha256")
	rebuilt = replacement.get("replacement_sha256")
	if not all(isinstance(digest, str) and len(digest) == 64 for digest in (original, rebuilt)):
		raise SmartsParityError("native E2E receipt has invalid adapter digests")
	if original == rebuilt:
		raise SmartsParityError("native E2E replacement adapter is not distinct")
	return {
		"closure": names,
		"original_adapter_sha256": original,
		"replacement_adapter_sha256": rebuilt,
		"replacement_build_type": replacement.get("replacement_build_type"),
		"smarts_after_replacement": after["smarts"],
		"smarts_before_replacement": before["smarts"],
	}


#============================================
def main() -> int:
	"""Run the same-build differential and publish its source-bound receipt."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--oracle-python", required=True, type=pathlib.Path)
	parser.add_argument("--ferrum-python", required=True, type=pathlib.Path)
	parser.add_argument("--native-e2e-receipt", required=True, type=pathlib.Path)
	parser.add_argument("--wheel", required=True, type=pathlib.Path)
	parser.add_argument("--report", default=DEFAULT_REPORT, type=pathlib.Path)
	arguments = parser.parse_args()
	for interpreter in (arguments.oracle_python, arguments.ferrum_python):
		if not interpreter.is_file():
			raise SmartsParityError("Python interpreter is not a file: " + str(interpreter))
	oracle = _validate_child(
		_run_child(arguments.oracle_python, ORACLE_CHILD),
		"rdkit-python-wrapper",
	)
	ferrum = _validate_child(
		_run_child(arguments.ferrum_python, FERRUM_CHILD),
		"ferrum-abi4",
	)
	wheel_sha256 = _sha256(arguments.wheel)
	receipt = {
		"artifacts": {
			"ferrum_extension_sha256": ferrum["binary_sha256"],
			"rdkit_python_binary_sha256": oracle["binary_sha256"],
			"wheel": str(arguments.wheel.resolve().relative_to(REPO_ROOT)),
			"wheel_sha256": wheel_sha256,
		},
		"comparison_policy": {
			"cross_version": "semantic query equivalence; not established by this slice",
			"recorded_build": "exact canonical SMILES and SMARTS strings",
		},
		"corpus": _compare(oracle, ferrum),
		"native_wheel_e2e": _native_e2e_facts(
			arguments.native_e2e_receipt,
			wheel_sha256,
		),
		"oracle_version": oracle["version"],
		"schema": "ferrum-smarts-codec-parity-v1",
		"source_sha256": {
			path: _sha256(REPO_ROOT / path) for path in SOURCE_PATHS
		},
	}
	arguments.report.parent.mkdir(parents=True, exist_ok=True)
	arguments.report.write_text(
		json.dumps(receipt, indent=2, sort_keys=True) + "\n",
		encoding="utf-8",
	)
	print(json.dumps({
		"corpus_size": len(receipt["corpus"]),
		"report": str(arguments.report.resolve()),
		"schema": receipt["schema"],
	}, sort_keys=True))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
