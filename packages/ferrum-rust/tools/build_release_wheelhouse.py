#!/usr/bin/env python3
"""Build the bounded macOS arm64 Ferrum Python release wheelhouse.

This maintainer command assembles the two first-party Python wheels without
resolving third-party dependencies.  A separate release E2E command installs
them using the recorded offline dependency wheelhouse and then returns a
validation record.  The ``receipt`` command publishes the combined release
receipt only after that validation succeeds.
"""

from __future__ import annotations

# Standard Library
import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import tomllib
import zipfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
RUST_PACKAGE_ROOT = Path(__file__).resolve().parents[1]
QT_PACKAGE_ROOT = REPO_ROOT / "packages" / "ferrum-chem-qt.app"
NATIVE_BUILDER = RUST_PACKAGE_ROOT / "tools" / "build_native_wheel.py"
BUILD_RECORD_NAME = "ferrum-release-wheelhouse-build-record.json"
RELEASE_RECEIPT_NAME = "ferrum-release-package-receipt.json"
BUILD_SCHEMA = "ferrum-release-wheelhouse-build-v1"
VALIDATION_SCHEMA = "ferrum-release-validation-v1"
RECEIPT_SCHEMA = "ferrum-release-package-receipt-v1"
TARGET = {"platform": "macos", "architecture": "arm64", "python": "3.12"}


class ReleaseBuildError(RuntimeError):
	"""An actionable failure in the release wheelhouse boundary."""


#============================================
def sha256(path: Path) -> str:
	"""Return one artifact digest used as receipt provenance.

	Args:
		path: Regular file whose digest identifies the observed artifact.

	Returns:
		Lowercase hexadecimal SHA-256 digest.
	"""
	digest = hashlib.sha256()
	with path.open("rb") as source:
		for block in iter(lambda: source.read(1024 * 1024), b""):
			digest.update(block)
	value = digest.hexdigest()
	return value


#============================================
def output_root_path(value: str) -> Path:
	"""Accept a fresh ignored output-root path inside this checkout.

	Args:
		value: User-supplied output-root text.

	Returns:
		Resolved checkout-local output root.
	"""
	path = Path(value).expanduser().resolve()
	try:
		relative = path.relative_to(REPO_ROOT)
	except ValueError as error:
		raise argparse.ArgumentTypeError("--output-root must be inside this checkout") from error
	if not relative.parts or not relative.parts[0].startswith("output"):
		raise argparse.ArgumentTypeError(
			"--output-root must be below a checkout-root ignored output* directory"
		)
	if path.is_relative_to(REPO_ROOT / "OTHER_REPOS"):
		raise argparse.ArgumentTypeError("--output-root must not be inside OTHER_REPOS")
	return path


#============================================
def input_directory_path(value: str) -> Path:
	"""Accept one explicit read-only input directory outside reference sources.

	Args:
		value: User-supplied input directory text.

	Returns:
		Resolved existing directory.
	"""
	path = Path(value).expanduser().resolve()
	if not path.is_dir():
		raise argparse.ArgumentTypeError(f"input directory does not exist: {path}")
	if path.is_relative_to(REPO_ROOT / "OTHER_REPOS"):
		raise argparse.ArgumentTypeError("input directory must not be inside OTHER_REPOS")
	return path


#============================================
def existing_file_path(value: str) -> Path:
	"""Accept one explicit existing JSON validation record.

	Args:
		value: User-supplied validation-record text.

	Returns:
		Resolved existing regular file.
	"""
	path = Path(value).expanduser().resolve()
	if not path.is_file():
		raise argparse.ArgumentTypeError(f"--validation-record is not a file: {path}")
	return path


#============================================
def require_target_host() -> None:
	"""Reject hosts outside the explicitly admitted initial release target."""
	if platform.system() != "Darwin" or platform.machine() != "arm64":
		raise ReleaseBuildError(
			"unsupported host phase: this release route supports only macOS arm64"
		)
	if sys.version_info[:2] != (3, 12):
		raise ReleaseBuildError(
			"unsupported Python phase: run through source_me.sh with Python 3.12"
		)


#============================================
def run_command(
		command: list[str], cwd: Path, phase: str, environment: dict[str, str] | None = None,
		) -> str:
	"""Run one phase command while retaining its stdout as structured evidence.

	Args:
		command: Complete command argv without shell interpolation.
		cwd: Phase working directory.
		phase: Stable phase label used in actionable errors.
		environment: Optional explicit child-process environment.

	Returns:
		Captured command stdout.
	"""
	print("+ " + " ".join(command), file=sys.stderr)
	try:
		result = subprocess.run(
			command, cwd=cwd, env=environment, text=True, capture_output=True, check=False
		)
	except FileNotFoundError as error:
		raise ReleaseBuildError(f"{phase} phase requires unavailable program: {command[0]}") from error
	if result.stderr:
		print(result.stderr, file=sys.stderr, end="")
	if result.returncode:
		raise ReleaseBuildError(f"{phase} phase failed with exit status {result.returncode}")
	return result.stdout


#============================================
def command_version(
		command: list[str], phase: str, environment: dict[str, str] | None = None,
		) -> str:
	"""Return one declared tool version for receipt provenance.

	Args:
		command: Version-query command argv.
		phase: Stable phase label used in actionable errors.
		environment: Optional explicit child-process environment.

	Returns:
		Trimmed declared version output.
	"""
	output = run_command(command, REPO_ROOT, phase, environment)
	value = output.strip()
	if not value:
		raise ReleaseBuildError(f"{phase} phase returned no version text")
	return value


#============================================
def package_metadata(wheel: Path) -> tuple[str, str]:
	"""Read one wheel's distribution name and version from its metadata.

	Args:
		wheel: Candidate wheel archive.

	Returns:
		Normalized distribution name and declared version.
	"""
	try:
		with zipfile.ZipFile(wheel) as archive:
			metadata_names = [
				name for name in archive.namelist() if name.endswith(".dist-info/METADATA")
			]
			if len(metadata_names) != 1:
				raise ReleaseBuildError(
					f"artifact selection phase needs one METADATA record in {wheel.name}"
				)
			contents = archive.read(metadata_names[0]).decode("utf-8")
	except zipfile.BadZipFile as error:
		raise ReleaseBuildError(f"artifact selection phase found invalid wheel: {wheel}") from error
	fields = {}
	for line in contents.splitlines():
		if ": " in line:
			key, value = line.split(": ", 1)
			if key in ("Name", "Version"):
				fields[key] = value
	if "Name" not in fields or "Version" not in fields:
		raise ReleaseBuildError(f"artifact selection phase found incomplete metadata in {wheel.name}")
	name = fields["Name"].lower().replace("_", "-")
	version = fields["Version"]
	return name, version


#============================================
def select_wheel(directory: Path, distribution: str, phase: str) -> Path:
	"""Select the one expected first-party wheel from one isolated phase.

	Args:
		directory: Phase-local wheel directory.
		distribution: Expected normalized distribution name.
		phase: Stable phase label used in actionable errors.

	Returns:
		Resolved selected wheel path.
	"""
	candidates = []
	for wheel in sorted(directory.glob("*.whl")):
		name, _ = package_metadata(wheel)
		if name == distribution:
			candidates.append(wheel.resolve())
	if len(candidates) != 1:
		raise ReleaseBuildError(
			f"{phase} phase requires one {distribution} wheel in {directory}, found {candidates}"
		)
	selected = candidates[0]
	return selected


#============================================
def artifact_record(wheel: Path) -> dict[str, str]:
	"""Describe one selected first-party artifact without treating its digest as a gate.

	Args:
		wheel: Selected wheel archive.

	Returns:
		Filename, distribution version, and observed digest.
	"""
	_, version = package_metadata(wheel)
	record = {"filename": wheel.name, "version": version, "sha256": sha256(wheel)}
	return record


#============================================
def dependency_wheel_records(directory: Path) -> list[dict[str, str]]:
	"""Record provisioned third-party wheels without copying them into the release.

	Args:
		directory: Explicit third-party dependency wheelhouse.

	Returns:
		Provenance records for available wheel files.
	"""
	records = []
	for wheel in sorted(directory.glob("*.whl")):
		record = {"filename": wheel.name, "sha256": sha256(wheel)}
		records.append(record)
	return records


#============================================
def toml_project_version(project: Path, phase: str) -> str:
	"""Read one declared project version from its owned pyproject file.

	Args:
		project: Project root containing pyproject.toml.
		phase: Stable phase label used in actionable errors.

	Returns:
		Declared project version.
	"""
	pyproject = project / "pyproject.toml"
	with pyproject.open("rb") as source:
		contents = tomllib.load(source)
	try:
		version = contents["project"]["version"]
	except KeyError as error:
		raise ReleaseBuildError(f"{phase} phase needs [project].version in {pyproject}") from error
	if not isinstance(version, str):
		raise ReleaseBuildError(f"{phase} phase found non-string project version in {pyproject}")
	return version


#============================================
def ensure_fresh_build_root(root: Path) -> None:
	"""Create one fresh root so phase outputs cannot borrow stale artifacts.

	Args:
		root: Requested release output root.
	"""
	if root.exists():
		raise ReleaseBuildError(
			f"release build phase refuses existing output root: {root}; choose a fresh output path"
		)
	root.mkdir(parents=True)


#============================================
def atomic_json(path: Path, record: dict) -> None:
	"""Publish one JSON record only after its complete content is prepared.

	Args:
		path: Destination record path.
		record: JSON-serializable receipt or build record.
	"""
	if path.exists():
		raise ReleaseBuildError(f"receipt phase refuses to overwrite existing record: {path}")
	contents = json.dumps(record, indent=2, sort_keys=True) + "\n"
	with tempfile.NamedTemporaryFile(
		mode="w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
	) as temporary:
		temporary.write(contents)
		temporary_path = Path(temporary.name)
	temporary_path.replace(path)


#============================================
def cargo_environment(cargo_home: Path) -> dict[str, str]:
	"""Return an explicit offline Cargo environment for release construction.

	Args:
		cargo_home: Maintainer-provisioned Cargo cache or vendor-backed home.

	Returns:
		Environment that cannot contact a Cargo registry.
	"""
	environment = os.environ.copy()
	environment["CARGO_HOME"] = str(cargo_home)
	environment["CARGO_NET_OFFLINE"] = "true"
	environment.pop("CARGO_TARGET_DIR", None)
	return environment


#============================================
def active_cargo_lockfile(manifest: Path) -> Path:
	"""Return the lockfile Cargo resolves beside the selected package manifest.

	Args:
		manifest: Shipping PyO3 package manifest passed to Cargo.

	Returns:
		Resolved active package lockfile.
	"""
	lockfile = manifest.parent / "Cargo.lock"
	if not lockfile.is_file():
		raise ReleaseBuildError(f"Cargo dependency preflight lacks active lockfile: {lockfile}")
	return lockfile.resolve()


#============================================
def preflight_cargo_dependencies(cargo_home: Path) -> dict[str, str]:
	"""Prove the declared Cargo home resolves the shipping manifest offline.

	Args:
		cargo_home: Maintainer-provisioned Cargo cache or vendor-backed home.

	Returns:
		Recorded Cargo dependency-source facts.
	"""
	environment = cargo_environment(cargo_home)
	manifest = RUST_PACKAGE_ROOT / "crates" / "api" / "python" / "Cargo.toml"
	lockfile = active_cargo_lockfile(manifest)
	command = [
		"cargo", "metadata", "--offline", "--locked", "--format-version", "1",
		"--manifest-path", str(manifest),
	]
	run_command(
		command,
		RUST_PACKAGE_ROOT,
		"Cargo dependency preflight",
		environment,
	)
	record = {
		"home": str(cargo_home),
		"offline": "true",
		"manifest": str(manifest),
		"lockfile": str(lockfile),
		"lockfile_sha256": sha256(lockfile),
		"preflight": "cargo metadata --offline --locked",
	}
	return record


#============================================
def isolated_qt_environment(
		qt_root: Path, dependency_wheelhouse: Path,
		) -> tuple[Path, dict, dict[str, str]]:
	"""Provision Qt build backends from a dedicated local wheelhouse.

	Args:
		qt_root: Fresh Qt phase root.
		dependency_wheelhouse: Explicit wheels for setuptools, wheel, and their requirements.

	Returns:
		Isolated Python executable, build-dependency facts, and scrubbed environment.
	"""
	environment = os.environ.copy()
	environment.pop("PYTHONHOME", None)
	environment.pop("PYTHONPATH", None)
	for name in tuple(environment):
		if name.startswith("PIP_"):
			environment.pop(name)
	environment["PIP_CONFIG_FILE"] = os.devnull
	venv = qt_root / "build-venv"
	run_command(
		[sys.executable, "-m", "venv", str(venv)], qt_root, "Qt build environment", environment
	)
	python = venv / "bin" / "python"
	if not python.is_file():
		raise ReleaseBuildError("Qt build environment phase did not produce its Python executable")
	run_command(
		[
			str(python), "-m", "pip", "install", "--no-index", "--ignore-installed",
			"--only-binary", ":all:",
			"--find-links", str(dependency_wheelhouse), "setuptools>=77", "wheel",
		],
		qt_root,
		"Qt build dependency provisioning",
		environment,
	)
	output = run_command(
		[
			str(python), "-c",
			"import importlib.metadata,json; print(json.dumps({'setuptools': "
			"importlib.metadata.version('setuptools'), 'wheel': "
			"importlib.metadata.version('wheel')}, sort_keys=True))",
		],
		qt_root,
		"Qt build dependency provisioning",
		environment,
	)
	try:
		versions = json.loads(output)
	except json.JSONDecodeError as error:
		raise ReleaseBuildError(
			"Qt build dependency provisioning emitted invalid version evidence"
		) from error
	if not isinstance(versions, dict):
		raise ReleaseBuildError("Qt build dependency provisioning emitted malformed version evidence")
	record = {
		"wheelhouse": str(dependency_wheelhouse),
		"wheels": dependency_wheel_records(dependency_wheelhouse),
		"python": str(python.resolve()),
		"packages": versions,
	}
	return python, record, environment


#============================================
def native_build_command(arguments: argparse.Namespace, chemistry_root: Path) -> list[str]:
	"""Construct the explicit offline native-builder invocation.

	Args:
		arguments: Parsed release-builder arguments.
		chemistry_root: Fresh child root for the delegated native build.

	Returns:
		Safe argv for the existing native wheel builder.
	"""
	command = [sys.executable, str(NATIVE_BUILDER), "build", "--output-root", str(chemistry_root)]
	if arguments.source_archive_root:
		command.extend(["--source-archive-root", str(arguments.source_archive_root)])
	else:
		command.extend(["--sealed-input-root", str(arguments.sealed_input_root)])
	return command


#============================================
def parse_native_result(output: str) -> Path:
	"""Read the delegated native builder's sole machine artifact result.

	Args:
		output: Captured delegated-builder stdout.

	Returns:
		Resolved chemistry wheel path.
	"""
	lines = [line for line in output.splitlines() if line.strip()]
	if len(lines) != 1:
		raise ReleaseBuildError("chemistry build phase did not emit one machine artifact result")
	try:
		record = json.loads(lines[0])
		artifact = Path(record["artifact"]).resolve()
	except (json.JSONDecodeError, KeyError, TypeError) as error:
		raise ReleaseBuildError(
			"chemistry build phase emitted an invalid machine artifact result"
		) from error
	if not artifact.is_file():
		raise ReleaseBuildError(f"chemistry build phase reported missing wheel: {artifact}")
	return artifact


#============================================
def staged_cargo_lockfile_record(chemistry_root: Path, dependencies: dict) -> dict[str, str]:
	"""Confirm Maturin staged the same active lockfile admitted by preflight.

	Args:
		chemistry_root: Completed delegated native-build phase root.
		dependencies: Cargo dependency record created before delegation.

	Returns:
		Staged lockfile provenance record.
	"""
	lockfile = chemistry_root / "maturin-project" / "crates" / "api" / "python" / "Cargo.lock"
	if not lockfile.is_file():
		raise ReleaseBuildError("chemistry build phase did not stage the active Cargo lockfile")
	digest = sha256(lockfile)
	if digest != dependencies["lockfile_sha256"]:
		raise ReleaseBuildError(
			"chemistry build phase staged a Cargo lockfile different from the offline preflight"
		)
	record = {"path": str(lockfile.resolve()), "sha256": digest}
	return record


#============================================
def build_qt_wheel(python: Path, qt_root: Path, environment: dict[str, str]) -> Path:
	"""Build one Qt wheel without an index, dependencies, or build isolation.

	Args:
		python: Isolated Python provisioned with recorded Qt build backends.
		qt_root: Fresh child root for Qt package output.
		environment: Scrubbed environment used for the isolated build process.

	Returns:
		Selected Ferrum wheel.
	"""
	wheelhouse = qt_root / "wheelhouse"
	wheelhouse.mkdir(parents=True)
	command = [
		str(python), "-m", "pip", "wheel", "--no-index", "--no-deps", "--no-build-isolation",
		"--wheel-dir", str(wheelhouse), str(QT_PACKAGE_ROOT),
	]
	run_command(command, qt_root, "Qt wheel build", environment)
	wheel = select_wheel(wheelhouse, "ferrum-qt", "Qt wheel build")
	return wheel


#============================================
def build_release(arguments: argparse.Namespace) -> None:
	"""Build the two first-party wheels into one release-local wheelhouse.

	Args:
		arguments: Parsed build command arguments.
	"""
	require_target_host()
	ensure_fresh_build_root(arguments.output_root)
	chemistry_root = arguments.output_root / "chemistry-build"
	qt_root = arguments.output_root / "qt-build"
	wheelhouse = arguments.output_root / "wheelhouse"
	cargo_dependencies = preflight_cargo_dependencies(arguments.cargo_home)
	cargo_build_environment = cargo_environment(arguments.cargo_home)
	chemistry_output = run_command(
		native_build_command(arguments, chemistry_root),
		RUST_PACKAGE_ROOT,
		"chemistry build",
		cargo_build_environment,
	)
	chemistry_wheel = parse_native_result(chemistry_output)
	cargo_dependencies["staged_lockfile"] = staged_cargo_lockfile_record(
		chemistry_root, cargo_dependencies
	)
	selected_chemistry = select_wheel(
		chemistry_wheel.parent, "ferrum-chem", "chemistry artifact selection"
	)
	qt_python, qt_build_dependencies, qt_build_environment = isolated_qt_environment(
		qt_root, arguments.qt_build_dependency_wheelhouse
	)
	qt_wheel = build_qt_wheel(qt_python, qt_root, qt_build_environment)
	wheelhouse.mkdir()
	selected_wheels = {"ferrum-chem": selected_chemistry, "ferrum-qt": qt_wheel}
	for wheel in selected_wheels.values():
		shutil.copy2(wheel, wheelhouse / wheel.name)
	artifacts = {
		name: artifact_record(wheelhouse / wheel.name) for name, wheel in selected_wheels.items()
	}
	chemistry_project = RUST_PACKAGE_ROOT / "crates" / "api" / "python"
	chemistry_version = toml_project_version(chemistry_project, "chemistry")
	qt_version = toml_project_version(QT_PACKAGE_ROOT, "Qt")
	if artifacts["ferrum-chem"]["version"] != chemistry_version:
		raise ReleaseBuildError(
			"chemistry artifact selection phase found a wheel version outside pyproject"
		)
	if artifacts["ferrum-qt"]["version"] != qt_version:
		raise ReleaseBuildError("Qt artifact selection phase found a wheel version outside pyproject")
	native_receipt = chemistry_root / "native-wheel-build-receipt.json"
	if not native_receipt.is_file():
		raise ReleaseBuildError("chemistry build phase did not publish its native receipt")
	record = {
		"schema": BUILD_SCHEMA,
		"target": TARGET,
		"wheelhouse": str(wheelhouse.resolve()),
		"artifacts": artifacts,
		"dependency_wheelhouse": {
			"path": str(arguments.dependency_wheelhouse),
			"wheels": dependency_wheel_records(arguments.dependency_wheelhouse),
		},
		"cargo_dependencies": cargo_dependencies,
		"qt_build_dependencies": qt_build_dependencies,
		"source_versions": {"ferrum-chem": chemistry_version, "ferrum-qt": qt_version},
		"source": {
			"chemistry_pyproject_sha256": sha256(chemistry_project / "pyproject.toml"),
			"qt_pyproject_sha256": sha256(QT_PACKAGE_ROOT / "pyproject.toml"),
			"native_receipt_sha256": sha256(native_receipt),
		},
		"toolchain": {
			"python": sys.version.split()[0],
			"pip": command_version([sys.executable, "-m", "pip", "--version"], "toolchain"),
			"rustc": command_version(["rustc", "--version"], "toolchain"),
			"cargo": command_version(
				["cargo", "--version"], "toolchain", cargo_build_environment
			),
		},
		"phases": {
			"chemistry": {
				"root": str(chemistry_root.resolve()),
				"native_receipt": str(native_receipt.resolve()),
			},
			"qt": {"root": str(qt_root.resolve())},
		},
	}
	build_record = arguments.output_root / BUILD_RECORD_NAME
	atomic_json(build_record, record)
	result = {"schema": BUILD_SCHEMA, "action": "build", "record": str(build_record.resolve())}
	print(json.dumps(result, sort_keys=True))


#============================================
def read_json_record(path: Path, phase: str) -> dict:
	"""Load one object-shaped JSON record with a phase-specific failure.

	Args:
		path: JSON record path.
		phase: Stable phase label used in actionable errors.

	Returns:
		Decoded object record.
	"""
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise ReleaseBuildError(f"{phase} phase found invalid JSON: {path}") from error
	if not isinstance(value, dict):
		raise ReleaseBuildError(f"{phase} phase needs a JSON object: {path}")
	return value


#============================================
def validate_artifact_match(build: dict, validation: dict) -> None:
	"""Require validation to attest to the exact selected first-party artifacts.

	Args:
		build: Durable builder record.
		validation: Delegated E2E validation record.
	"""
	if validation.get("schema") != VALIDATION_SCHEMA:
		raise ReleaseBuildError("receipt phase needs ferrum-release-validation-v1 input")
	if validation.get("outcome") != "success":
		raise ReleaseBuildError("receipt phase received a validation outcome that is not success")
	if validation.get("target") != build["target"]:
		raise ReleaseBuildError("receipt phase validation target differs from the built target")
	validated_artifacts = validation.get("artifacts")
	if not isinstance(validated_artifacts, dict):
		raise ReleaseBuildError("receipt phase validation record lacks artifact observations")
	for name, artifact in build["artifacts"].items():
		observed = validated_artifacts.get(name)
		if not isinstance(observed, dict):
			raise ReleaseBuildError(f"receipt phase validation record lacks {name} observation")
		if (
			observed.get("filename") != artifact["filename"]
			or observed.get("sha256") != artifact["sha256"]
		):
			raise ReleaseBuildError(f"receipt phase validation artifact differs for {name}")
	if not isinstance(validation.get("observations"), dict):
		raise ReleaseBuildError("receipt phase validation record lacks semantic observations")


#============================================
def publish_receipt(arguments: argparse.Namespace) -> None:
	"""Publish the complete receipt after a delegated successful validation record.

	Args:
		arguments: Parsed receipt command arguments.
	"""
	build_path = arguments.output_root / BUILD_RECORD_NAME
	if not build_path.is_file():
		raise ReleaseBuildError(f"receipt phase needs completed build record: {build_path}")
	build = read_json_record(build_path, "receipt")
	if build.get("schema") != BUILD_SCHEMA:
		raise ReleaseBuildError("receipt phase found an unrecognized build record")
	validation = read_json_record(arguments.validation_record, "receipt")
	validate_artifact_match(build, validation)
	receipt = {
		"schema": RECEIPT_SCHEMA,
		"target": build["target"],
		"artifacts": build["artifacts"],
		"source_versions": build["source_versions"],
		"source": build["source"],
		"toolchain": build["toolchain"],
		"dependency_wheelhouse": build["dependency_wheelhouse"],
		"cargo_dependencies": build["cargo_dependencies"],
		"qt_build_dependencies": build["qt_build_dependencies"],
		"native_build": build["phases"]["chemistry"],
		"validation": validation,
	}
	path = arguments.output_root / RELEASE_RECEIPT_NAME
	atomic_json(path, receipt)
	result = {"schema": RECEIPT_SCHEMA, "action": "receipt", "record": str(path.resolve())}
	print(json.dumps(result, sort_keys=True))


#============================================
def parser() -> argparse.ArgumentParser:
	"""Create the maintainer release-wheelhouse command interface.

	Returns:
		Configured command parser.
	"""
	result = argparse.ArgumentParser(description=__doc__)
	subcommands = result.add_subparsers(dest="command", required=True)
	build = subcommands.add_parser(
		"build", help="build two first-party wheels into a release wheelhouse"
	)
	build.add_argument("--output-root", required=True, type=output_root_path)
	sources = build.add_mutually_exclusive_group(required=True)
	sources.add_argument("--source-archive-root", type=input_directory_path)
	sources.add_argument("--sealed-input-root", type=input_directory_path)
	build.add_argument(
		"--cargo-home",
		dest="cargo_home",
		required=True,
		type=input_directory_path,
		help="provisioned Cargo cache or vendor-backed home verified with cargo --offline",
	)
	build.add_argument("--dependency-wheelhouse", required=True, type=input_directory_path)
	build.add_argument(
		"--qt-build-dependency-wheelhouse",
		dest="qt_build_dependency_wheelhouse",
		required=True,
		type=input_directory_path,
		help="local setuptools/wheel build-backend wheelhouse for an isolated Qt build venv",
	)
	build.set_defaults(handler=build_release)
	receipt = subcommands.add_parser(
		"receipt", help="publish receipt after successful release validation"
	)
	receipt.add_argument("--output-root", required=True, type=output_root_path)
	receipt.add_argument("--validation-record", required=True, type=existing_file_path)
	receipt.set_defaults(handler=publish_receipt)
	return result


#============================================
def main() -> int:
	"""Run one selected release wheelhouse phase.

	Returns:
		Zero after success, otherwise one after an actionable failure.
	"""
	try:
		arguments = parser().parse_args()
		arguments.handler(arguments)
		return 0
	except ReleaseBuildError as error:
		print(f"release wheelhouse error: {error}", file=sys.stderr)
		return 1


if __name__ == "__main__":
	raise SystemExit(main())
