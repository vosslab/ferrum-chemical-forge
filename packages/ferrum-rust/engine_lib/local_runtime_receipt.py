"""Receipt handling for the disposable local Ferrum runtime."""

from __future__ import annotations

import hashlib
import json
import os
import stat
import subprocess
import sys
import sysconfig
import tempfile
from pathlib import Path

from engine_lib.native_engine_profile import FERRUM_RDKIT_PROFILE
from engine_lib.native_engine_receipt import directory_tree_sha256, native_policy_sha256


LOCAL_RUNTIME_RECEIPT_FILENAME = "ferrum-local-runtime-receipt.json"
LOCAL_RUNTIME_RECEIPT_SCHEMA = "ferrum-local-runtime-receipt-v2"
LOCAL_ENGINE_BUNDLE_DIRECTORY_NAME = "engine-v1"
_CARGO_ROOT = Path(__file__).resolve().parents[1]
_REPO_ROOT = _CARGO_ROOT.parents[1]
_BUILD_PACKAGES = (_CARGO_ROOT / "crates/api", _CARGO_ROOT / "crates/api-python")
_RELEASE_DEPENDENCY_KINDS = frozenset((None, "build"))
_LOCAL_GUI_LAUNCHER_SPECIFICATION = _CARGO_ROOT / "engine_lib/local_runtime_launcher.py"


class LocalRuntimeReceiptError(RuntimeError):
	"""The local runtime receipt cannot prove current build inputs and artifacts."""


#============================================
def _sha256_file(path: Path, label: str) -> str:
	"""Hash one regular local file without following a substitution symlink."""
	try:
		status = path.lstat()
	except OSError as error:
		raise LocalRuntimeReceiptError(f"cannot inspect {label}: {path}") from error
	if path.is_symlink() or not stat.S_ISREG(status.st_mode):
		raise LocalRuntimeReceiptError(f"{label} must be a regular file: {path}")
	digest = hashlib.sha256()
	try:
		with path.open("rb") as handle:
			for chunk in iter(lambda: handle.read(1024 * 1024), b""):
				digest.update(chunk)
	except OSError as error:
		raise LocalRuntimeReceiptError(f"cannot read {label}: {path}") from error
	return digest.hexdigest()


#============================================
def _cargo_metadata() -> dict[str, object]:
	"""Load Cargo's locked release graph before fingerprinting local sources."""
	try:
		result = subprocess.run(
			("cargo", "metadata", "--locked", "--format-version=1"),
			cwd=_CARGO_ROOT, check=False, capture_output=True, text=True,
		)
	except OSError as error:
		raise LocalRuntimeReceiptError("cannot run cargo metadata for local runtime") from error
	if result.returncode != 0:
		message = result.stderr.strip() or "cargo metadata failed without diagnostics"
		raise LocalRuntimeReceiptError(f"cannot resolve local Cargo build graph: {message}")
	try:
		metadata = json.loads(result.stdout)
	except json.JSONDecodeError as error:
		raise LocalRuntimeReceiptError("cargo metadata did not produce JSON") from error
	if not isinstance(metadata, dict):
		raise LocalRuntimeReceiptError("cargo metadata did not produce an object")
	return metadata


#============================================
def _cargo_package_closure(metadata: dict[str, object] | None = None) -> tuple[Path, ...]:
	"""Return Cargo's normal/build local package closure for staged release targets."""
	metadata = _cargo_metadata() if metadata is None else metadata
	packages_value = metadata.get("packages")
	resolve_value = metadata.get("resolve")
	if not isinstance(packages_value, list) or not isinstance(resolve_value, dict):
		raise LocalRuntimeReceiptError("cargo metadata lacks packages or resolve graph")
	packages_by_id: dict[str, dict[str, object]] = {}
	for package in packages_value:
		if isinstance(package, dict) and isinstance(package.get("id"), str):
			packages_by_id[package["id"]] = package
	nodes_value = resolve_value.get("nodes")
	if not isinstance(nodes_value, list):
		raise LocalRuntimeReceiptError("cargo metadata resolve graph lacks nodes")
	nodes_by_id = {
		node["id"]: node for node in nodes_value
		if isinstance(node, dict) and isinstance(node.get("id"), str)
	}
	build_manifest_paths = {(package / "Cargo.toml").resolve() for package in _BUILD_PACKAGES}
	pending = [
		package_id for package_id, package in packages_by_id.items()
		if _package_manifest_path(package) in build_manifest_paths
	]
	if len(pending) != len(build_manifest_paths):
		raise LocalRuntimeReceiptError("cargo metadata does not contain every local build package")
	closure: set[Path] = set()
	seen: set[str] = set()
	while pending:
		package_id = pending.pop()
		if package_id in seen:
			continue
		seen.add(package_id)
		package = packages_by_id.get(package_id)
		node = nodes_by_id.get(package_id)
		if package is None or node is None:
			raise LocalRuntimeReceiptError("cargo metadata graph references an unknown package")
		package_path = _package_manifest_path(package).parent
		if not package_path.is_relative_to(_CARGO_ROOT):
			continue
		closure.add(package_path)
		dependencies = node.get("deps")
		if not isinstance(dependencies, list):
			raise LocalRuntimeReceiptError("cargo metadata package node lacks dependencies")
		for dependency in dependencies:
			if not isinstance(dependency, dict) or not isinstance(dependency.get("pkg"), str):
				raise LocalRuntimeReceiptError("cargo metadata dependency is malformed")
			kinds = dependency.get("dep_kinds")
			if not isinstance(kinds, list):
				raise LocalRuntimeReceiptError("cargo metadata dependency lacks kinds")
			if any(
				isinstance(kind, dict) and kind.get("kind") in _RELEASE_DEPENDENCY_KINDS
				for kind in kinds
			):
				pending.append(dependency["pkg"])
	return tuple(sorted(closure))


#============================================
def _package_manifest_path(package: dict[str, object]) -> Path:
	"""Return one Cargo metadata package's checked manifest path."""
	manifest_path = package.get("manifest_path")
	if not isinstance(manifest_path, str):
		raise LocalRuntimeReceiptError("cargo metadata package lacks manifest path")
	path = Path(manifest_path).resolve()
	if path.name != "Cargo.toml" or not path.is_file():
		raise LocalRuntimeReceiptError(f"cargo metadata package has invalid manifest: {path}")
	return path


#============================================
def _closure_files(metadata: dict[str, object] | None = None) -> tuple[Path, ...]:
	"""Return the exact local Rust inputs selected for Ferrum's release packages."""
	files = {_CARGO_ROOT / "Cargo.toml", _CARGO_ROOT / "Cargo.lock"}
	for package in _cargo_package_closure(metadata):
		files.add(package / "Cargo.toml")
		build_script = package / "build.rs"
		if build_script.is_file():
			files.add(build_script)
		source_root = package / "src"
		if not source_root.is_dir():
			raise LocalRuntimeReceiptError(f"local Cargo package lacks src directory: {package}")
		files.update(path for path in source_root.rglob("*") if path.is_file())
	return tuple(sorted(files))


#============================================
def local_cargo_source_sha256(metadata: dict[str, object] | None = None) -> str:
	"""Fingerprint the local Cargo-resolved release source closure."""
	return _files_sha256(_closure_files(metadata), "local Cargo source")


#============================================
def _files_sha256(paths: tuple[Path, ...], label: str) -> str:
	"""Hash a sorted source collection with unambiguous relative-path framing."""
	digest = hashlib.sha256()
	for path in paths:
		relative = path.relative_to(_REPO_ROOT).as_posix().encode("utf-8")
		content = _sha256_file(path, label).encode("ascii")
		digest.update(len(relative).to_bytes(8, "big"))
		digest.update(relative)
		digest.update(content)
	return digest.hexdigest()


#============================================
def local_runtime_inputs() -> dict[str, str]:
	"""Return stable identities for every local input that shapes this runtime."""
	native_source = _CARGO_ROOT / "crates/chemistry/native"
	builder_files = tuple(sorted((
		*_CARGO_ROOT.joinpath("engine_lib").glob("*.py"),
		_CARGO_ROOT / "local_engine_builder.py",
	)))
	return {
		"cargo_source_sha256": local_cargo_source_sha256(),
		"launcher_specification_sha256": local_launcher_specification_sha256(),
		"native_builder_sha256": _files_sha256(builder_files, "local native builder source"),
		"native_policy_sha256": native_policy_sha256(FERRUM_RDKIT_PROFILE),
		"native_source_sha256": directory_tree_sha256(native_source, "Ferrum native source"),
	}


#============================================
def local_launcher_specification_sha256() -> str:
	"""Fingerprint the source-owned GUI launcher contract before it is generated."""
	return _sha256_file(
		_LOCAL_GUI_LAUNCHER_SPECIFICATION, "local GUI launcher specification"
	)


#============================================
def local_extension_filename() -> str:
	"""Return the current interpreter's exact importable Ferrum extension name."""
	suffix = sysconfig.get_config_var("EXT_SUFFIX")
	if type(suffix) is not str or not suffix.startswith("."):
		raise LocalRuntimeReceiptError("Python does not provide a usable extension suffix")
	return f"ferrum_chem{suffix}"


#============================================
def local_extension_path(runtime_root: Path) -> Path:
	"""Return the ABI-specific extension path staged under one local runtime."""
	return runtime_root.resolve() / local_extension_filename()


#============================================
def _local_launcher_paths(runtime_root: Path) -> dict[str, Path]:
	"""Return the launchers that consume one checked local runtime tree."""
	build_root = runtime_root.resolve().parents[1]
	return {
		"cli": build_root / "bin/ferrum",
		"gui": build_root / "bin/ferrum-qt",
	}


#============================================
def _local_engine_bundle_path(runtime_root: Path) -> Path:
	"""Return the sealed engine closure paired with one local executable."""
	return runtime_root.resolve().parent / LOCAL_ENGINE_BUNDLE_DIRECTORY_NAME


#============================================
def _sha256_executable(path: Path, label: str) -> str:
	"""Hash one regular executable launcher without following a symlink."""
	try:
		mode = path.lstat().st_mode
	except OSError as error:
		raise LocalRuntimeReceiptError(f"cannot inspect {label}: {path}") from error
	if not mode & stat.S_IXUSR:
		raise LocalRuntimeReceiptError(f"{label} must be executable: {path}")
	return _sha256_file(path, label)


#============================================
def _runtime_artifacts(runtime_root: Path) -> dict[str, Path]:
	root = runtime_root.resolve()
	return {
		"adapter": root / ".dylibs/libferrum_chem.dylib",
		"extension": local_extension_path(root),
		**_local_launcher_paths(root),
	}


#============================================
def _receipt_record(runtime_root: Path) -> dict[str, object]:
	artifacts = _runtime_artifacts(runtime_root)
	return {
		"artifacts": {
			name: (_sha256_executable(path, name) if name in {"cli", "gui"}
				else _sha256_file(path, name))
			for name, path in artifacts.items()
		} | {"engine_bundle": directory_tree_sha256(
			_local_engine_bundle_path(runtime_root), "local engine bundle"
		)},
		"inputs": local_runtime_inputs(),
		"schema": LOCAL_RUNTIME_RECEIPT_SCHEMA,
	}


#============================================
def write_local_runtime_receipt(runtime_root: Path) -> Path:
	"""Atomically publish the receipt after every local runtime artifact is staged."""
	runtime_root = runtime_root.resolve()
	if not runtime_root.is_dir():
		raise LocalRuntimeReceiptError(f"local runtime root is missing: {runtime_root}")
	receipt = runtime_root / LOCAL_RUNTIME_RECEIPT_FILENAME
	if receipt.exists() or receipt.is_symlink():
		raise LocalRuntimeReceiptError(f"local runtime receipt already exists: {receipt}")
	record = _receipt_record(runtime_root)
	try:
		with tempfile.NamedTemporaryFile(
			mode="w", encoding="utf-8", prefix=".runtime-receipt-", dir=runtime_root,
			delete=False,
		) as handle:
			handle.write(json.dumps(record, indent=2, sort_keys=True) + "\n")
			temporary = Path(handle.name)
		os.replace(temporary, receipt)
	except OSError as error:
		raise LocalRuntimeReceiptError(f"cannot write local runtime receipt: {receipt}") from error
	return receipt


#============================================
def validate_local_runtime_receipt(runtime_root: Path) -> None:
	"""Fail closed unless receipt, source closure, native inputs, and artifacts match."""
	runtime_root = runtime_root.resolve()
	receipt = runtime_root / LOCAL_RUNTIME_RECEIPT_FILENAME
	try:
		stored = json.loads(receipt.read_text(encoding="utf-8"))
	except FileNotFoundError as error:
		raise LocalRuntimeReceiptError(f"local runtime receipt is missing: {receipt}") from error
	except (OSError, json.JSONDecodeError) as error:
		raise LocalRuntimeReceiptError(f"local runtime receipt is unreadable: {receipt}") from error
	if not isinstance(stored, dict) or stored.get("schema") != LOCAL_RUNTIME_RECEIPT_SCHEMA:
		raise LocalRuntimeReceiptError(f"local runtime receipt has wrong schema: {receipt}")
	if stored != _receipt_record(runtime_root):
		raise LocalRuntimeReceiptError(
			"local runtime receipt does not match current Cargo/native inputs or staged artifacts"
		)


#============================================
def validate_local_runtime_import(runtime_root: Path) -> None:
	"""Prove Python imports the exact staged extension and its required surface.

	The receipt establishes artifact and source identities. This second gate uses a
	new interpreter with the staged runtime as its only explicit import root, so a
	successful local build cannot silently resolve an installed or stale extension.
	"""
	runtime_root = runtime_root.resolve()
	validate_local_runtime_receipt(runtime_root)
	expected_extension = local_extension_path(runtime_root)
	probe = _run_extension_import_probe(runtime_root, expected_extension)
	module_file = probe.get("module_file")
	if not isinstance(module_file, str):
		raise LocalRuntimeReceiptError("staged extension import did not report its module file")
	try:
		actual_extension = Path(module_file).resolve(strict=True)
	except OSError as error:
		raise LocalRuntimeReceiptError(
			"staged extension import reported an unreadable module file"
		) from error
	if actual_extension != expected_extension:
		raise LocalRuntimeReceiptError(
			"Python imported a different ferrum_chem extension than the staged runtime"
		)
	members = probe.get("document_session_members")
	if not isinstance(members, list) or not all(isinstance(member, str) for member in members):
		raise LocalRuntimeReceiptError(
			"staged extension import did not report its DocumentSession surface"
		)
	missing = sorted({"can_redo", "can_undo"}.difference(members))
	if missing:
		raise LocalRuntimeReceiptError(
			"staged ferrum_chem extension is missing required DocumentSession members: "
			+ ", ".join(missing)
		)
	if probe.get("canonical_cdml_loads") is not True:
		raise LocalRuntimeReceiptError(
			"staged ferrum_chem extension did not confirm canonical Ferrum CDML loading"
		)


#============================================
def _run_extension_import_probe(runtime_root: Path, expected_extension: Path) -> dict[str, object]:
	"""Import the staged extension in an isolated subprocess and return its facts."""
	probe = (
		"import importlib, json, pathlib, sys; "
		"module = importlib.import_module('ferrum_chem'); "
		"session = module.DocumentSession; "
		"session.load('<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\"/>'); "
		"print(json.dumps({'module_file': str(pathlib.Path(module.__file__).resolve()), "
		"'document_session_members': [name for name in ('can_undo', 'can_redo') "
		"if hasattr(session, name)], 'canonical_cdml_loads': True}, sort_keys=True))"
	)
	environment = os.environ.copy()
	environment.pop("PYTHONHOME", None)
	environment["PYTHONPATH"] = str(runtime_root)
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	try:
		result = subprocess.run(
			(sys.executable, "-c", probe, str(expected_extension)),
			cwd=runtime_root,
			env=environment,
			check=False,
			capture_output=True,
			text=True,
		)
	except OSError as error:
		raise LocalRuntimeReceiptError("cannot start Python to import staged ferrum_chem") from error
	if result.returncode != 0:
		diagnostics = result.stderr.strip() or result.stdout.strip()
		raise LocalRuntimeReceiptError(
			"cannot import staged ferrum_chem: "
			+ (diagnostics or "Python exited without diagnostics")
		)
	try:
		payload = json.loads(result.stdout)
	except json.JSONDecodeError as error:
		raise LocalRuntimeReceiptError(
			"staged extension import did not produce an identity receipt"
		) from error
	if not isinstance(payload, dict):
		raise LocalRuntimeReceiptError("staged extension import produced an invalid identity receipt")
	return payload
