"""Staged Python wheel construction and executable engine bundle assembly."""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

import wheel_lib.native_wheel_publication as native_wheel_publication
from wheel_lib.native_wheel_macho import assert_clean_closure, assert_packaged_library_closure, copy_and_rewrite_closure
from wheel_lib.native_wheel_packaging import find_maturin, inject_root_metadata, stage_native_notice_bundle, stage_python_project
from wheel_lib.native_wheel_policy import NativePolicyError, homebrew_llvm, rust_tool_environment
from wheel_lib.native_wheel_receipt import sha256
from wheel_lib.native_wheel_builder_adapter import run
from wheel_lib.native_wheel_builder_model import ADAPTER_ABI_VERSION, ADAPTER_NAME, BUNDLE_MANIFEST_NAME, BUNDLE_SCHEMA, NativeBuildError, RUST_PACKAGE_SOURCE, RdkitLayout, ferrum_source_closure, ferrum_worktree_source_closure, require_matching_worktree_source_closure
from wheel_lib.native_wheel_builder_sources import safe_extract_zip_members, validate_wheel_members


def audit_wheel_closure(wheel: Path, output_root: Path) -> None:
	"""Inspect the packaged Mach-O files, not the source staging directory."""
	native_wheel_publication.audit_wheel_closure(
		wheel, output_root, validate_wheel_members, safe_extract_zip_members,
		assert_clean_closure, NativeBuildError,
	)


#============================================
def build_wheel(
		output_root: Path, adapter: Path, layout: RdkitLayout, target: str,
		) -> tuple[Path, dict[str, object], dict[str, object], Path]:
	if sys.version_info[:2] != (3, 12):
		raise NativeBuildError(
			f"native wheel requires the Python 3.12 build interpreter, got {sys.version.split()[0]}; "
			"run this tool through source_me.sh"
		)
	output_root = output_root.resolve()
	worktree_source_closure = ferrum_worktree_source_closure(RUST_PACKAGE_SOURCE)
	stage = stage_python_project(
		output_root, RUST_PACKAGE_SOURCE,
		lambda copied: require_matching_worktree_source_closure(
			worktree_source_closure, ferrum_worktree_source_closure(copied), "while staging",
		),
	)
	stage_native_notice_bundle(stage, RUST_PACKAGE_SOURCE, layout.input_root)
	package_libs = stage / "ferrum_chem" / ".dylibs" if (stage / "ferrum_chem").is_dir() else stage / ".dylibs"
	copy_and_rewrite_closure(adapter, layout.graphmol_library, package_libs)
	staged_bundle = stage / "ferrum-engine-bundle"
	build_engine_bundle(output_root, adapter, layout, staged_bundle)
	# Capture the source subset after each deterministic staging rewrite. The
	# manifest explicitly excludes only the builder-owned payload paths.
	source_closure = ferrum_source_closure(stage.parents[2])
	try:
		environment = rust_tool_environment(homebrew_llvm())
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	# The public C header is the only ABI authority. Cargo writes this derived
	# build setting into generated Rust source for the PyO3 boundary.
	environment["FERRUM_CHEM_ADAPTER_ABI_VERSION"] = str(ADAPTER_ABI_VERSION)
	# Maturin invokes Cargo.  Keep all generated state under ignored output root.
	environment["CARGO_TARGET_DIR"] = str(output_root / "maturin-target")
	wheelhouse = output_root / "wheelhouse"
	wheelhouse.mkdir(parents=True, exist_ok=True)
	run(
		find_maturin(), "build", "--release", "--target", target,
		"--auditwheel", "skip",
		"--interpreter", sys.executable, "--out", str(wheelhouse),
		cwd=stage, env=environment,
	)
	wheels = sorted(wheelhouse.glob("ferrum_chem-*.whl"))
	if len(wheels) != 1:
		raise NativeBuildError(f"expected exactly one wheel in {wheelhouse}, found {wheels}")
	inject_root_metadata(wheels[0], stage)
	audit_wheel_closure(wheels[0], output_root)
	return wheels[0], source_closure, worktree_source_closure, staged_bundle

#============================================
def build_engine_bundle(output_root: Path, adapter: Path, layout: RdkitLayout, destination: Path) -> Path:
	"""Publish the same rewritten native closure in Ferrum's CLI bundle layout."""
	return native_wheel_publication.build_engine_bundle(
		output_root, adapter, layout.graphmol_library, destination, copy_and_rewrite_closure,
		assert_packaged_library_closure, BUNDLE_MANIFEST_NAME, BUNDLE_SCHEMA, ADAPTER_ABI_VERSION,
		ADAPTER_NAME, sha256, NativeBuildError,
	)


#============================================
def copy_engine_bundle(source: Path, destination: Path) -> Path:
	"""Copy the wheel's sealed bundle unchanged for the matching CLI installation."""
	if source.is_symlink() or not source.is_dir():
		raise NativeBuildError(f"staged engine bundle is not a regular directory: {source}")
	if destination.exists() or destination.is_symlink():
		raise NativeBuildError(f"refusing to overwrite existing engine bundle: {destination}")
	shutil.copytree(source, destination)
	return destination
