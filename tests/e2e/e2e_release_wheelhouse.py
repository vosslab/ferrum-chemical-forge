"""Prove Ferrum's macOS arm64 release wheels through an offline clean install."""

from __future__ import annotations

# Standard Library
import argparse
import hashlib
import json
import os
import pathlib
import platform
import shutil
import subprocess
import sys
import tempfile
import zipfile


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
RELEASE_BUILDER = REPO_ROOT / "packages/ferrum-rust/tools/build_release_wheelhouse.py"
NATIVE_BUILDER = REPO_ROOT / "packages/ferrum-rust/tools/build_native_wheel.py"
BUILD_RECORD_NAME = "ferrum-release-wheelhouse-build-record.json"
VALIDATION_RECORD_NAME = "ferrum-release-validation-v1.json"
EXPECTED_TARGET = {"platform": "macos", "architecture": "arm64", "python": "3.12"}
LOADER_ENVIRONMENT_NAMES = (
	"DYLD_LIBRARY_PATH",
	"DYLD_FALLBACK_LIBRARY_PATH",
	"DYLD_FRAMEWORK_PATH",
	"DYLD_FALLBACK_FRAMEWORK_PATH",
	"PYTHONHOME",
	"PYTHONPATH",
)


class ReleaseE2eError(RuntimeError):
	"""Raised when the target-specific release route cannot be proved."""


#============================================
def parse_args() -> argparse.Namespace:
	"""Parse the two explicit local wheelhouse inputs.

	Returns:
		Parsed release-root and third-party wheelhouse paths.
	"""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"--release-root", dest="release_root", required=True, type=pathlib.Path,
		help="fresh output root produced by build_release_wheelhouse.py build",
	)
	parser.add_argument(
		"--dependency-wheelhouse", dest="dependency_wheelhouse", required=True,
		type=pathlib.Path,
		help="local third-party wheels for the declared Qt runtime dependencies",
	)
	arguments = parser.parse_args()
	return arguments


#============================================
def scrubbed_environment() -> dict[str, str]:
	"""Return a child environment without ambient Python or macOS loader paths.

	Returns:
		Environment suitable for a clean installed-wheel child process.
	"""
	environment = os.environ.copy()
	for name in LOADER_ENVIRONMENT_NAMES:
		environment.pop(name, None)
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	return environment


#============================================
def run(phase: str, recovery: str, *command: str, env: dict[str, str] | None = None) -> str:
	"""Run one local release command and return its machine-readable stdout.

	Args:
		phase: User-visible release phase being performed.
		recovery: Safe next action after a phase failure.
		*command: Executable and arguments for the local child process.
		env: Optional scrubbed child environment.

	Returns:
		The complete standard output of the successful command.
	"""
	print("+", " ".join(command), file=sys.stderr)
	result = subprocess.run(
		command,
		env=scrubbed_environment() if env is None else env,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)
	if result.returncode:
		details = result.stderr.strip()
		raise ReleaseE2eError(f"{phase} phase failed ({result.returncode}): {details}\nNext: {recovery}")
	output = result.stdout
	return output


#============================================
def json_object(text: str, label: str) -> dict[str, object]:
	"""Decode one JSON object without assigning meaning to serialization details.

	Args:
		text: JSON text emitted by a trusted local child or artifact.
		label: Human-readable source of the JSON text.

	Returns:
		Decoded object.
	"""
	try:
		value = json.loads(text)
	except json.JSONDecodeError as error:
		raise ReleaseE2eError(f"{label} is not valid JSON: {error.msg}") from error
	if not isinstance(value, dict):
		raise ReleaseE2eError(f"{label} must be a JSON object")
	return value


#============================================
def existing_directory(path: pathlib.Path, label: str) -> pathlib.Path:
	"""Resolve a required local directory or raise an actionable phase failure.

	Args:
		path: Directory supplied by the maintainer or builder.
		label: Human-readable boundary name.

	Returns:
		Resolved directory path.
	"""
	resolved = path.resolve()
	if not resolved.is_dir():
		raise ReleaseE2eError(f"{label} is unavailable: {resolved}")
	return resolved


#============================================
def read_build_record(release_root: pathlib.Path) -> tuple[dict[str, object], pathlib.Path]:
	"""Read the release builder's artifact-selection record.

	Args:
		release_root: Fresh output root from the first-party release builder.

	Returns:
		The validated build record and its resolved first-party wheelhouse.
	"""
	path = release_root / BUILD_RECORD_NAME
	if not path.is_file():
		raise ReleaseE2eError(
			"first-party build record is unavailable; run build_release_wheelhouse.py build first"
		)
	record = json_object(path.read_text(encoding="utf-8"), "release build record")
	if record.get("schema") != "ferrum-release-wheelhouse-build-v1":
		raise ReleaseE2eError("release build record has an unsupported schema")
	if record.get("target") != EXPECTED_TARGET:
		raise ReleaseE2eError("release build record is not for macOS arm64 CPython 3.12")
	wheelhouse_value = record.get("wheelhouse")
	if not isinstance(wheelhouse_value, str):
		raise ReleaseE2eError("release build record omits its first-party wheelhouse")
	wheelhouse = existing_directory(pathlib.Path(wheelhouse_value), "first-party wheelhouse")
	artifacts = record.get("artifacts")
	if not isinstance(artifacts, dict):
		raise ReleaseE2eError("release build record omits selected first-party artifacts")
	for name in ("ferrum-chem", "ferrum-qt"):
		artifact = artifacts.get(name)
		if not isinstance(artifact, dict) or not isinstance(artifact.get("filename"), str):
			raise ReleaseE2eError(f"release build record omits selected {name} wheel")
		if not (wheelhouse / artifact["filename"]).is_file():
			raise ReleaseE2eError(f"selected {name} wheel is unavailable in the first-party wheelhouse")
	return record, wheelhouse


#============================================
def sha256(path: pathlib.Path) -> str:
	"""Return an observed artifact identity for the release receipt.

	Args:
		path: Selected first-party wheel.

	Returns:
		Lowercase SHA-256 identity of the supplied artifact.
	"""
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	value = digest.hexdigest()
	return value


#============================================
def selected_wheels(
		build_record: dict[str, object], wheelhouse: pathlib.Path,
		) -> dict[str, dict[str, str]]:
	"""Resolve and identify exactly the two first-party wheels selected by the builder.

	Args:
		build_record: Builder's selected-artifact record.
		wheelhouse: Builder-owned first-party wheel directory.

	Returns:
		Observed filename, absolute path, and identity for each Ferrum distribution.
	"""
	artifacts = build_record["artifacts"]
	if not isinstance(artifacts, dict):
		raise ReleaseE2eError("first-party artifact phase lacks selected wheel records")
	selected = {}
	for name in ("ferrum-chem", "ferrum-qt"):
		artifact = artifacts[name]
		if not isinstance(artifact, dict) or not isinstance(artifact.get("filename"), str):
			raise ReleaseE2eError(f"first-party artifact phase lacks selected {name} wheel")
		path = (wheelhouse / artifact["filename"]).resolve()
		if path.parent != wheelhouse or not path.is_file():
			raise ReleaseE2eError(f"first-party artifact phase cannot read selected {name} wheel")
		selected[name] = {"filename": path.name, "path": str(path), "sha256": sha256(path)}
	return selected


#============================================
def wheel_distribution(wheel: pathlib.Path) -> str | None:
	"""Read an optional normalized distribution name from one wheel metadata record.

	Args:
		wheel: Candidate wheel in the third-party resolver input.

	Returns:
		Normalized distribution name, or None when the candidate has no usable metadata.
	"""
	try:
		with zipfile.ZipFile(wheel) as archive:
			for name in archive.namelist():
				if not name.endswith(".dist-info/METADATA"):
					continue
				for line in archive.read(name).decode("utf-8").splitlines():
					if line.startswith("Name: "):
						return line.removeprefix("Name: ").lower().replace("_", "-")
	except (OSError, UnicodeDecodeError, zipfile.BadZipFile):
		return None
	return None


#============================================
def require_third_party_wheelhouse(wheelhouse: pathlib.Path) -> None:
	"""Reject Ferrum candidates that could shadow the selected first-party wheels.

	Args:
		wheelhouse: Explicit resolver input intended only for third-party dependencies.

	Returns:
		None.
	"""
	for wheel in wheelhouse.glob("*.whl"):
		filename = wheel.name.lower().replace("_", "-")
		name = wheel_distribution(wheel)
		if (
			filename.startswith("ferrum-chem-")
			or filename.startswith("ferrum-qt-")
			or name in {"ferrum-chem", "ferrum-qt"}
		):
			raise ReleaseE2eError(
				"installation phase third-party wheelhouse contains a Ferrum candidate; "
				"remove it so only the builder-selected wheel paths can supply Ferrum"
			)


#============================================
def wheel_identities(wheelhouse: pathlib.Path) -> set[tuple[str, str]]:
	"""Describe the current resolver input as filename and observed identity pairs.

	Args:
		wheelhouse: Explicit third-party wheel directory.

	Returns:
		Set of current wheel filename and SHA-256 provenance pairs.
	"""
	identities = set()
	for wheel in wheelhouse.glob("*.whl"):
		if wheel.is_file():
			identities.add((wheel.name, sha256(wheel)))
	return identities


#============================================
def require_recorded_dependency_wheelhouse(
		build_record: dict[str, object], wheelhouse: pathlib.Path,
		) -> None:
	"""Bind the offline resolver input to the dependency provenance selected at build time.

	Args:
		build_record: Builder record containing third-party wheel provenance.
		wheelhouse: Current explicit resolver input directory.

	Returns:
		None.
	"""
	declared = build_record.get("dependency_wheelhouse")
	if not isinstance(declared, dict):
		raise ReleaseE2eError("installation phase build record omits dependency wheel provenance")
	recorded_wheels = declared.get("wheels")
	if not isinstance(recorded_wheels, list):
		raise ReleaseE2eError("installation phase build record has malformed dependency provenance")
	expected = set()
	for record in recorded_wheels:
		if not isinstance(record, dict):
			raise ReleaseE2eError("installation phase build record has malformed dependency wheel record")
		filename = record.get("filename")
		identity = record.get("sha256")
		if not isinstance(filename, str) or not isinstance(identity, str):
			raise ReleaseE2eError("installation phase build record has incomplete dependency wheel record")
		expected.add((filename, identity))
	current = wheel_identities(wheelhouse)
	if current != expected:
		raise ReleaseE2eError(
			"installation phase dependency wheelhouse differs from the build-recorded resolver input\n"
			"Next: restore the recorded dependency wheels or build a fresh release wheelhouse"
		)


#============================================
def require_supported_host() -> None:
	"""Refuse an unsupported host before creating an apparently clean proof.

	Returns:
		None.
	"""
	if platform.system() != "Darwin" or platform.machine() != "arm64":
		raise ReleaseE2eError("release proof supports only a macOS arm64 host")
	if sys.version_info[:2] != (3, 12):
		raise ReleaseE2eError("release proof requires the repository CPython 3.12 environment")


#============================================
def create_venv(work_root: pathlib.Path) -> pathlib.Path:
	"""Create one temporary, unconfigured virtual environment.

	Args:
		work_root: Disposable E2E output location.

	Returns:
		The virtual environment's Python executable.
	"""
	venv = work_root / "clean-venv"
	run(
		"venv", "recreate the release environment with CPython 3.12 and retry",
		sys.executable, "-B", "-m", "venv", str(venv),
	)
	python = venv / "bin" / "python"
	if not python.is_file():
		raise ReleaseE2eError("clean virtual environment omitted its Python executable")
	return python


#============================================
def install_release(
		python: pathlib.Path, selected: dict[str, dict[str, str]], third_party: pathlib.Path,
		) -> None:
	"""Resolve both Ferrum wheels only from explicit local wheelhouses.

	Args:
		python: Clean environment interpreter.
		selected: Builder-selected Ferrum wheel paths and observed identities.
		third_party: Separately provisioned Qt dependency wheelhouse.

	Returns:
		None.
	"""
	run(
		"installation", "supply compatible third-party wheels and rebuild the release wheelhouse",
		str(python), "-B", "-m", "pip", "install", "--no-index",
		"--find-links", str(third_party), selected["ferrum-chem"]["path"],
		selected["ferrum-qt"]["path"],
	)


#============================================
def installed_protocol_probe(python: pathlib.Path) -> dict[str, object]:
	"""Prove extension origin, V1 schema roots, packaged schema, and inspect semantics.

	Args:
		python: Clean environment interpreter.

	Returns:
		Small semantic observation from the isolated installed process.
	"""
	code = (
		"import importlib.machinery, json, pathlib, ferrum_chem; "
		"extension=pathlib.Path(ferrum_chem.__file__); "
		"returned=json.loads(ferrum_chem.operation_protocol_schema_v1()); "
		"packaged=json.loads((extension.parent/'ferrum-operation-v1.schema.json').read_text()); "
		"roots=('request','success_response','error_response'); "
		"request={'schema':'ferrum-operation-request-v1','request_id':'release-e2e-inspect',"
		"'operation':{'kind':'document.inspect','document':'<cdml><molecule id=\"m\">"
		"<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom>"
		"</molecule></cdml>'}}; "
		"response=json.loads(ferrum_chem.execute_operation_v1(json.dumps(request))); "
		"ok=(extension.suffix in importlib.machinery.EXTENSION_SUFFIXES and "
		"all(name in returned.get('x-ferrum-roots',{}) for name in roots) and "
		"all(name in packaged.get('x-ferrum-roots',{}) for name in roots) and "
		"response.get('schema')=='ferrum-operation-response-v1' and "
		"response.get('request_id')=='release-e2e-inspect' and "
		"response.get('outcome',{}).get('kind')=='document.inspect'); "
		"print(json.dumps({'ok':ok,'extension':extension.name,'outcome':"
		"response.get('outcome',{}).get('kind')}))"
	)
	value = json_object(run(
		"installed extension/schema/protocol",
		"inspect the fresh wheel and local dependency wheelhouse, then rerun the release proof",
		str(python), "-I", "-B", "-c", code,
	), "installed extension/schema/protocol phase output; next: inspect the fresh wheel and rerun")
	if value.get("ok") is not True:
		raise ReleaseE2eError(
			"installed extension/schema/protocol phase failed semantic validation\n"
			"Next: inspect the fresh wheel and local dependency wheelhouse, then rerun"
		)
	return value


#============================================
def installed_qt_probe(python: pathlib.Path) -> dict[str, object]:
	"""Prove the installed console entry point and application-owned resource lookup.

	Args:
		python: Clean environment interpreter.

	Returns:
		Small semantic observation from installed Ferrum.
	"""
	entrypoint = python.parent / "ferrum-qt"
	version = run(
		"Qt entry/resource", "rebuild the Ferrum wheel with its declared entry point and resources",
		str(entrypoint), "--version",
	)
	if not version.strip().startswith("Ferrum "):
		raise ReleaseE2eError("installed ferrum-qt --version did not identify Ferrum")
	code = (
		"import json, ferrum_qt.resource_paths; "
		"icon=ferrum_qt.resource_paths.get_resource_path('app_icon.svg'); "
		"theme=ferrum_qt.resource_paths.get_resource_path('themes','light.yaml'); "
		"print(json.dumps({'icon':icon.is_file(),'theme':theme.is_file()}))"
	)
	value = json_object(run(
		"Qt entry/resource", "rebuild the Ferrum wheel with its declared entry point and resources",
		str(python), "-I", "-B", "-c", code,
	), "Qt entry/resource phase output; next: rebuild the Ferrum wheel and rerun")
	if value.get("icon") is not True or value.get("theme") is not True:
		raise ReleaseE2eError(
			"Qt entry/resource phase did not find the installed icon and theme\n"
			"Next: rebuild the Ferrum wheel with its declared resources and rerun"
		)
	return {"entrypoint": "ferrum-qt --version", "resources": value}


#============================================
def installed_adapter(python: pathlib.Path) -> pathlib.Path:
	"""Locate the installed replaceable adapter through the extension's package boundary.

	Args:
		python: Clean environment interpreter.

	Returns:
		The installed package-relative adapter library.
	"""
	code = (
		"import pathlib, ferrum_chem; "
		"print(pathlib.Path(ferrum_chem.__file__).parent/'.dylibs/libferrum_chem.dylib')"
	)
	path = pathlib.Path(run(
		"relink/load", "rebuild the selected chemistry wheel and verify its native closure",
		str(python), "-I", "-B", "-c", code,
	).strip())
	if not path.is_file():
		raise ReleaseE2eError(
			"relink/load phase cannot find the installed replaceable adapter library\n"
			"Next: rebuild the selected chemistry wheel and verify its native closure"
		)
	return path


#============================================
def build_replacement(release_root: pathlib.Path, work_root: pathlib.Path) -> pathlib.Path:
	"""Build the independently replaceable adapter from the selected native inputs.

	Args:
		release_root: First-party release root that owns validated native inputs.
		work_root: Disposable output location for the replacement adapter.

	Returns:
		The independently built adapter library.
	"""
	result = json_object(run(
		"relink/load", "rebuild the native adapter from the selected chemistry-build inputs",
		sys.executable, "-B", str(NATIVE_BUILDER), "adapter",
		"--output-root", str(work_root / "replacement-adapter"),
		"--rdkit-output-root", str(release_root / "chemistry-build"),
	), "relink/load phase replacement output; next: rebuild the selected native adapter and rerun")
	if result.get("schema") != "ferrum-native-wheel-artifact-v1" or result.get("action") != "adapter":
		raise ReleaseE2eError(
			"relink/load phase replacement builder returned an unsupported artifact record\n"
			"Next: rebuild the native adapter from the selected chemistry-build inputs"
		)
	artifact = result.get("artifact")
	if not isinstance(artifact, str):
		raise ReleaseE2eError(
			"relink/load phase replacement builder omitted its adapter library\n"
			"Next: rebuild the native adapter from the selected chemistry-build inputs"
		)
	path = pathlib.Path(artifact)
	if not path.is_file() or path.name != "libferrum_chem.dylib":
		raise ReleaseE2eError(
			"relink/load phase replacement builder did not produce libferrum_chem.dylib\n"
			"Next: rebuild the native adapter from the selected chemistry-build inputs"
		)
	return path


#============================================
def write_validation_record(
		release_root: pathlib.Path, selected: dict[str, dict[str, str]], observations: dict[str, object],
		) -> pathlib.Path:
	"""Publish the E2E outcome atomically for the release builder's receipt command.

	Args:
		release_root: First-party release output root.
		selected: Actual wheel identities supplied to the clean installer.
		observations: Small installed behavior observations.

	Returns:
		The atomically published validation record.
	"""
	observed_artifacts = {}
	for name in ("ferrum-chem", "ferrum-qt"):
		artifact = selected[name]
		observed_artifacts[name] = {"filename": artifact["filename"], "sha256": artifact["sha256"]}
	value = {
		"schema": "ferrum-release-validation-v1",
		"target": EXPECTED_TARGET,
		"outcome": "success",
		"artifacts": observed_artifacts,
		"observations": observations,
	}
	with tempfile.NamedTemporaryFile(
		mode="w", encoding="utf-8", dir=release_root,
		prefix=".ferrum-release-validation-", suffix=".json", delete=False,
	) as handle:
		handle.write(json.dumps(value, indent=2, sort_keys=True) + "\n")
		temporary = pathlib.Path(handle.name)
	target = release_root / VALIDATION_RECORD_NAME
	temporary.replace(target)
	return target


#============================================
def main() -> int:
	"""Run the one target-specific offline install and relink proof.

	Returns:
		Zero after the builder has accepted the successful validation record.
	"""
	arguments = parse_args()
	require_supported_host()
	release_root = existing_directory(arguments.release_root, "release output root")
	third_party = existing_directory(arguments.dependency_wheelhouse, "third-party wheelhouse")
	build_record, first_party = read_build_record(release_root)
	selected = selected_wheels(build_record, first_party)
	require_third_party_wheelhouse(third_party)
	declared_dependency = build_record.get("dependency_wheelhouse")
	if not isinstance(declared_dependency, dict) or declared_dependency.get("path") != str(third_party):
		raise ReleaseE2eError("release build record does not identify the supplied third-party wheelhouse")
	require_recorded_dependency_wheelhouse(build_record, third_party)
	with tempfile.TemporaryDirectory(prefix="ferrum-release-e2e-") as temporary:
		work_root = pathlib.Path(temporary)
		python = create_venv(work_root)
		install_release(python, selected, third_party)
		before = installed_protocol_probe(python)
		qt = installed_qt_probe(python)
		adapter = installed_adapter(python)
		replacement = build_replacement(release_root, work_root)
		shutil.copy2(replacement, adapter)
		after = installed_protocol_probe(python)
		observations = {"protocol_before_relink": before, "qt": qt, "protocol_after_relink": after}
		validation = write_validation_record(release_root, selected, observations)
		receipt = run(
			"receipt", "inspect the build and validation records, then rebuild the release output if identities differ",
			sys.executable, "-B", str(RELEASE_BUILDER), "receipt",
			"--output-root", str(release_root), "--validation-record", str(validation),
		)
		result = json_object(
			receipt, "receipt phase output; next: inspect artifact identities and rebuild if they differ"
		)
		print(json.dumps(result, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except ReleaseE2eError as error:
		print(f"release E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
