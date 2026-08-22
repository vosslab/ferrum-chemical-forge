"""Build the disposable local native-engine runtime used by Ferrum development."""

from __future__ import annotations

import json
import os
import platform
import shutil
import tempfile
from pathlib import Path

from engine_lib.native_engine_adapter import build_rdkit, configure_adapter
from engine_lib.native_engine_macho import assert_runtime_library_closure, copy_and_rewrite_runtime_closure
from engine_lib.native_engine_model import ADAPTER_ABI_VERSION, ADAPTER_NAME, NativeBuildError, REPO_ROOT
from engine_lib.native_engine_receipt import sha256


LOCAL_RUNTIME_SCHEMA = "ferrum-local-engine-runtime-v1"
ENGINE_BUNDLE_DIRECTORY_NAME = "engine-v1"
ENGINE_BUNDLE_MANIFEST_NAME = "ferrum-engine-bundle-v1.json"
ENGINE_BUNDLE_SCHEMA = "ferrum-engine-bundle-v1"


#============================================
def runtime_root_path(value: str) -> Path:
	"""Accept one fresh runtime root owned by the repository build directory."""
	path = Path(value).expanduser().resolve()
	build_root = (REPO_ROOT / "build").resolve()
	if path == build_root or not path.is_relative_to(build_root):
		raise ValueError(f"--runtime-root must be below {build_root}: {path}")
	return path


#============================================
def build_local_runtime(runtime_root: Path) -> Path:
	"""Build and atomically stage the private adapter closure for local execution.

	The native sources, CMake output, and RDKit installation all live in one unique
	temporary directory.  Only the loader-relative ``.dylibs`` runtime survives.
	"""
	if platform.system() != "Darwin" or platform.machine() != "arm64":
		raise NativeBuildError(
			"the local native engine currently supports only macOS arm64"
		)
	runtime_root = runtime_root_path(str(runtime_root))
	if runtime_root.exists():
		raise NativeBuildError(
			f"local runtime root already exists; build.sh must provide a fresh path: {runtime_root}"
		)
	runtime_root.parent.mkdir(parents=True, exist_ok=True)
	staging = Path(tempfile.mkdtemp(prefix=".native-engine-", dir=runtime_root.parent))
	try:
		# Passing None materializes every pinned source below the disposable staging
		# root, so no source archive cache survives this local build.
		layout = build_rdkit(staging, None)
		adapter = configure_adapter(staging, layout)
		bundle = staging / ENGINE_BUNDLE_DIRECTORY_NAME
		copy_and_rewrite_runtime_closure(adapter, layout.graphmol_library, bundle)
		assert_runtime_library_closure(bundle)
		_write_engine_bundle_manifest(bundle)
		candidate = staging / "runtime"
		candidate.mkdir()
		(candidate / ".dylibs").symlink_to(f"../{ENGINE_BUNDLE_DIRECTORY_NAME}")
		bundle_destination = runtime_root.parent / ENGINE_BUNDLE_DIRECTORY_NAME
		if bundle_destination.exists() or bundle_destination.is_symlink():
			raise NativeBuildError(
				f"local engine bundle already exists; build.sh must provide a fresh path: {bundle_destination}"
			)
		os.replace(bundle, bundle_destination)
		os.replace(candidate, runtime_root)
		return runtime_root / ".dylibs" / ADAPTER_NAME
	finally:
		shutil.rmtree(staging, ignore_errors=True)


#============================================
def _write_engine_bundle_manifest(bundle: Path) -> None:
	"""Seal one local executable bundle with the Rust-owned fixed schema."""
	members = [
		{"path": member.name, "sha256": sha256(member)}
		for member in sorted(bundle.iterdir())
		if member.is_file() and not member.is_symlink()
	]
	if not members or not any(member["path"] == ADAPTER_NAME for member in members):
		raise NativeBuildError("local engine bundle lacks its Ferrum adapter")
	manifest = {
		"schema": ENGINE_BUNDLE_SCHEMA,
		"target": _executable_bundle_target(),
		"adapter_abi_version": ADAPTER_ABI_VERSION,
		"adapter": ADAPTER_NAME,
		"members": members,
	}
	(bundle / ENGINE_BUNDLE_MANIFEST_NAME).write_text(
		json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
	)


#============================================
def _executable_bundle_target() -> str:
	"""Match Rust's architecture and operating-system target spelling."""
	architecture = {"arm64": "aarch64"}.get(platform.machine(), platform.machine())
	operating_system = {"Darwin": "macos"}.get(platform.system(), platform.system().lower())
	return f"{architecture}-{operating_system}"


#============================================
def emit_runtime_result(runtime_root: Path) -> None:
	"""Build one local runtime and print its sole machine-readable result."""
	adapter = build_local_runtime(runtime_root)
	print(json.dumps({
		"schema": LOCAL_RUNTIME_SCHEMA,
		"runtime_root": str(adapter.parent.parent),
		"adapter": str(adapter),
	}, sort_keys=True))
