"""Nonshipping installed-wheel proof for SMARTS response-size admission."""

from __future__ import annotations

import hashlib
import json
import os
import re
from pathlib import Path
import subprocess
import sys
import tempfile
import zipfile

import pytest


LIMIT_ENV = "FERRUM_SMARTS_RESPONSE_SIZE_E2E_LIMIT"
MAX_RESPONSE_BYTES = 1_048_576
CONFIGURATION_ENVIRONMENTS = (
	"FERRUM_RESPONSE_SIZE_E2E_INSTALL_ROOT",
	"FERRUM_RESPONSE_SIZE_E2E_HARNESS_WHEEL",
	"FERRUM_RESPONSE_SIZE_E2E_SHIPPING_WHEEL",
	"FERRUM_RESPONSE_SIZE_E2E_BUNDLE_MANIFEST",
	"FERRUM_RESPONSE_SIZE_E2E_HARNESS_SHA256",
	"FERRUM_RESPONSE_SIZE_E2E_SHIPPING_SHA256",
	"FERRUM_RESPONSE_SIZE_E2E_BUNDLE_MANIFEST_SHA256",
)
CDML = (
	'<cdml><molecule id="m"><atom id="a" name="C">'
	'<point x="10" y="20"/></atom></molecule></cdml>'
)


def _configured_path(name: str) -> Path:
	"""Return one required regular harness provenance path."""
	value = os.environ.get(name)
	if not value:
		raise RuntimeError(f"{name} is required for the response-size PyO3 harness")
	path = Path(value).resolve()
	if not path.is_file() or path.is_symlink():
		raise RuntimeError(f"{name} must name a regular file")
	return path


def _configured_install_root() -> Path:
	"""Return the required isolated installation root or fail closed."""
	value = os.environ.get("FERRUM_RESPONSE_SIZE_E2E_INSTALL_ROOT")
	if not value:
		raise RuntimeError("FERRUM_RESPONSE_SIZE_E2E_INSTALL_ROOT is required")
	root = Path(value).resolve()
	if not root.is_dir() or root.is_symlink():
		raise RuntimeError("FERRUM_RESPONSE_SIZE_E2E_INSTALL_ROOT must be a regular directory")
	return root


def _sha256(path: Path) -> str:
	"""Return the lowercase SHA-256 of one regular provenance file."""
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


def _native_members(archive: zipfile.ZipFile) -> dict[str, bytes]:
	"""Return the exact flat native closure from one wheel archive."""
	members: dict[str, bytes] = {}
	for info in archive.infolist():
		if not info.filename.startswith(".dylibs/") or info.filename == ".dylibs/":
			continue
		name = info.filename.removeprefix(".dylibs/")
		if not name or "/" in name or "\\" in name or name in members:
			raise RuntimeError("response-size wheel has an unsafe native closure")
		members[name] = archive.read(info)
	if not members:
		raise RuntimeError("response-size wheel has no native closure")
	return members


def _regular_wheel_member(archive: zipfile.ZipFile, member: str) -> zipfile.ZipInfo:
	"""Return exactly one regular archive member or reject ambiguous wheel paths."""
	matching = [info for info in archive.infolist() if info.filename == member]
	if len(matching) != 1:
		raise RuntimeError(f"response-size wheel has duplicate or missing member: {member}")
	info = matching[0]
	if info.is_dir() or (info.external_attr >> 16) & 0o170000 == 0o120000:
		raise RuntimeError(f"response-size wheel member is not regular: {member}")
	return info


def _unique_extension_member(archive: zipfile.ZipFile) -> tuple[str, str]:
	"""Return the one Python extension member and its immutable archive digest."""
	candidates = [
		info for info in archive.infolist()
		if Path(info.filename).name.startswith("ferrum_chem")
		and Path(info.filename).suffix in {".so", ".pyd"}
	]
	if len(candidates) != 1:
		raise RuntimeError("response-size wheel needs exactly one ferrum_chem extension member")
	info = _regular_wheel_member(archive, candidates[0].filename)
	digest = hashlib.sha256(archive.read(info)).hexdigest()
	return info.filename, digest


def _require_imported_extension_provenance(wheel: Path, install_root: Path) -> tuple[str, str]:
	"""Bind the imported extension byte-for-byte to one configured wheel member."""
	extension = Path(ferrum_chem.__file__).resolve()
	if extension.is_symlink() or not extension.is_file():
		raise RuntimeError("imported ferrum_chem extension is not regular")
	try:
		member = extension.relative_to(install_root).as_posix()
	except ValueError as error:
		raise RuntimeError("source fallback is forbidden") from error
	with zipfile.ZipFile(wheel) as archive:
		info = _regular_wheel_member(archive, member)
		expected_digest = hashlib.sha256(archive.read(info)).hexdigest()
	actual_digest = _sha256(extension)
	if actual_digest != expected_digest:
		raise RuntimeError("configured harness wheel differs from imported extension")
	return member, expected_digest


def _require_harness_provenance() -> tuple[Path, Path, Path]:
	"""Require an imported nonshipping wheel and a distinct shipping wheel."""
	harness_wheel = _configured_path("FERRUM_RESPONSE_SIZE_E2E_HARNESS_WHEEL")
	shipping_wheel = _configured_path("FERRUM_RESPONSE_SIZE_E2E_SHIPPING_WHEEL")
	bundle_manifest = _configured_path("FERRUM_RESPONSE_SIZE_E2E_BUNDLE_MANIFEST")
	if harness_wheel == shipping_wheel:
		raise RuntimeError("response-size harness wheel must be distinct from shipping wheel")
	for name, path in (("harness", harness_wheel), ("shipping", shipping_wheel)):
		expected = os.environ.get(f"FERRUM_RESPONSE_SIZE_E2E_{name.upper()}_SHA256")
		if not expected or not re.fullmatch(r"[0-9a-f]{64}", expected):
			raise RuntimeError(f"response-size {name} wheel needs immutable SHA-256 provenance")
		if _sha256(path) != expected:
			raise RuntimeError(f"response-size {name} wheel SHA-256 provenance mismatch")
	expected_manifest = os.environ.get("FERRUM_RESPONSE_SIZE_E2E_BUNDLE_MANIFEST_SHA256")
	if not expected_manifest or not re.fullmatch(r"[0-9a-f]{64}", expected_manifest):
		raise RuntimeError("response-size bundle manifest needs immutable SHA-256 provenance")
	if _sha256(bundle_manifest) != expected_manifest:
		raise RuntimeError("response-size bundle manifest SHA-256 provenance mismatch")
	manifest = json.loads(bundle_manifest.read_text(encoding="utf-8"))
	if manifest.get("adapter_abi_version") != 5 or manifest.get("adapter") != "libferrum_chem.dylib":
		raise RuntimeError("response-size bundle manifest is not the ABI-5 adapter closure")
	expected_names = {member["path"] for member in manifest["members"]}
	with zipfile.ZipFile(shipping_wheel) as shipping, zipfile.ZipFile(harness_wheel) as harness:
		shipping_members = _native_members(shipping)
		harness_members = _native_members(harness)
		if set(shipping_members) != expected_names or harness_members != shipping_members:
			raise RuntimeError("response-size harness closure differs from sealed ABI-5 wheel")
	return harness_wheel, shipping_wheel, bundle_manifest


if not any(os.environ.get(name) for name in CONFIGURATION_ENVIRONMENTS):
	pytest.skip(
		"requires an explicitly configured isolated response-size PyO3 harness",
		allow_module_level=True,
	)
if any(not os.environ.get(name) for name in CONFIGURATION_ENVIRONMENTS):
	raise RuntimeError("response-size PyO3 harness configuration is incomplete")

INSTALL_ROOT = _configured_install_root()
HARNESS_WHEEL, SHIPPING_WHEEL, BUNDLE_MANIFEST = _require_harness_provenance()
import ferrum_chem
HARNESS_EXTENSION_MEMBER, HARNESS_EXTENSION_SHA256 = _require_imported_extension_provenance(
	HARNESS_WHEEL,
	INSTALL_ROOT,
)


def _request(query: dict[str, str], request_id: str) -> str:
	"""Create one public raw or selected SMARTS operation request."""
	session = ferrum_chem.DocumentSession.load(CDML)
	snapshot = session.snapshot()
	if query["kind"] == "selected_molecule":
		query = {
			"kind": "selected_molecule",
			"molecule_id": session.observe(snapshot.revision).projection.molecules[0].id,
		}
	return json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": request_id,
		"operation": {
			"kind": "document.molecule.smarts.query.v1",
			"document": {
				"cdml": CDML,
				"expected_revision": snapshot.revision,
				"expected_digest_hex": snapshot.digest,
			},
			"query": query,
			"limits": {"max_matches_per_molecule": 1, "max_total_matches": 1},
		},
	})


@pytest.mark.parametrize("query", [
	{"kind": "smarts", "value": "[#6]"},
	{"kind": "selected_molecule", "molecule_id": "m"},
], ids=["raw", "selected"])
def test_public_pyo3_smarts_response_admission_is_exact_and_redacted(query: dict[str, str]) -> None:
	"""Public PyO3 preserves success at its measured bound and redacts one-byte overrun."""
	assert _require_imported_extension_provenance(HARNESS_WHEEL, INSTALL_ROOT) == (
		HARNESS_EXTENSION_MEMBER,
		HARNESS_EXTENSION_SHA256,
	)
	request_id = f"response-size-pyo3-{query['kind']}"
	request = _request(query, request_id)
	os.environ.pop(LIMIT_ENV, None)
	baseline = ferrum_chem.execute_operation_v1(request)
	baseline_bytes = baseline.encode("utf-8")
	assert len(baseline_bytes) <= MAX_RESPONSE_BYTES
	assert json.loads(baseline)["outcome"]["kind"] == "document.molecule.smarts.query.v1"

	os.environ[LIMIT_ENV] = str(len(baseline_bytes))
	exact = ferrum_chem.execute_operation_v1(request)
	assert exact == baseline

	os.environ[LIMIT_ENV] = str(len(baseline_bytes) - 1)
	overrun = ferrum_chem.execute_operation_v1(request)
	refusal = json.loads(overrun)
	assert refusal == {
		"schema": "ferrum-operation-error-v1",
		"request_id": request_id,
		"error": {
			"category": "resource_limit",
			"operation": "document.molecule.smarts.query.v1",
			"message": "response_size_exceeded",
			"resource_limit_reason": "response_size_exceeded",
		},
	}
	for forbidden in ("[#6]", CDML, "molecules", "receipt", "record_id", "position", "adapter", ".dylibs"):
		assert forbidden not in overrun


def test_shipping_wheel_does_not_contain_the_nonshipping_limit_hook() -> None:
	"""The ordinary distribution contains no test-harness feature or environment hook."""
	shipping_bytes = SHIPPING_WHEEL.read_bytes()
	assert LIMIT_ENV.encode("ascii") not in shipping_bytes
	assert b"response-size-e2e-harness" not in shipping_bytes


def test_shipping_wheel_ignores_limit_environment_in_separate_interpreter() -> None:
	"""A separately installed normal wheel ignores the nonshipping limit environment."""
	with zipfile.ZipFile(SHIPPING_WHEEL) as archive:
		extension_member, extension_digest = _unique_extension_member(archive)
	with tempfile.TemporaryDirectory(prefix="ferrum-response-size-shipping-") as temporary:
		install_root = Path(temporary) / "site-packages"
		install = subprocess.run(
			[
				sys.executable,
				"-m",
				"pip",
				"install",
				"--no-deps",
				"--target",
				str(install_root),
				str(SHIPPING_WHEEL),
			],
			capture_output=True,
			check=False,
			text=True,
		)
		if install.returncode != 0:
			raise RuntimeError(f"shipping wheel installation failed: {install.stderr}")
		child_environment = os.environ.copy()
		child_environment.pop("PYTHONPATH", None)
		child_environment.pop("PYTHONHOME", None)
		for name in CONFIGURATION_ENVIRONMENTS:
			child_environment.pop(name, None)
		child_environment[LIMIT_ENV] = "1"
		child_program = "\n".join([
			"import hashlib",
			"import json",
			"import sys",
			"from pathlib import Path",
			"install_root = Path(sys.argv[1]).resolve()",
			"expected_member = sys.argv[2]",
			"expected_digest = sys.argv[3]",
			"sys.path.insert(0, str(install_root))",
			"import ferrum_chem",
			"extension = Path(ferrum_chem.__file__).resolve()",
			"if extension.is_symlink() or not extension.is_file(): raise RuntimeError('shipping extension is not regular')",
			"if extension.relative_to(install_root).as_posix() != expected_member: raise RuntimeError('shipping extension path mismatch')",
			"if hashlib.sha256(extension.read_bytes()).hexdigest() != expected_digest: raise RuntimeError('shipping extension digest mismatch')",
			"cdml = '<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule></cdml>'",
			"session = ferrum_chem.DocumentSession.load(cdml)",
			"snapshot = session.snapshot()",
			"request = {'schema': 'ferrum-operation-request-v1', 'request_id': 'shipping-limit-env-ignored', 'operation': {'kind': 'document.molecule.smarts.query.v1', 'document': {'cdml': cdml, 'expected_revision': snapshot.revision, 'expected_digest_hex': snapshot.digest}, 'query': {'kind': 'smarts', 'value': '[#6]'}, 'limits': {'max_matches_per_molecule': 1, 'max_total_matches': 1}}}",
			"response = json.loads(ferrum_chem.execute_operation_v1(json.dumps(request)))",
			"if response['outcome']['kind'] != 'document.molecule.smarts.query.v1': raise RuntimeError('shipping wheel honored nonshipping limit environment')",
		])
		child = subprocess.run(
			[
				sys.executable,
				"-I",
				"-c",
				child_program,
				str(install_root),
				extension_member,
				extension_digest,
			],
			capture_output=True,
			check=False,
			text=True,
			env=child_environment,
		)
		if child.returncode != 0:
			raise RuntimeError(f"isolated shipping-wheel execution failed: {child.stderr}")
