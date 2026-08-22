"""Concrete command handlers for the native-wheel executable."""

from __future__ import annotations

import argparse
import json
import platform
import sys
from pathlib import Path

import wheel_lib.native_wheel_publication as native_wheel_publication
from wheel_lib.native_wheel_macho import assert_packaged_library_closure, copy_and_rewrite_closure, detect_variants
from wheel_lib.native_wheel_packaging import find_maturin, tool_version
from wheel_lib.native_wheel_policy import rust_toolchain_receipt
from wheel_lib.native_wheel_profile import FERRUM_RDKIT_PROFILE, MACOS_ARM64_NATIVE_CLOSURE, TARGET
from wheel_lib.native_wheel_receipt import NativeReceiptError, sha256, write_build_receipt
from wheel_lib.native_wheel_builder_adapter import build_rdkit, configure_adapter
from wheel_lib.native_wheel_builder_model import ADAPTER_ABI_VERSION, ADAPTER_NAME, BUNDLE_MANIFEST_NAME, BUNDLE_SCHEMA, FERRUM_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES, FERRUM_SOURCE_CLOSURE_SCHEMA, FERRUM_WORKTREE_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES, FERRUM_WORKTREE_SOURCE_CLOSURE_EXCLUDED_SUFFIXES, FERRUM_WORKTREE_SOURCE_CLOSURE_SCHEMA, MACHINE_RESULT_SCHEMA, NativeBuildError, emit_artifact_result, executable_bundle_target, rdkit_layout_from_output_root, validate_build_receipt, validate_publication_candidate, validate_publication_engine_bundle
from wheel_lib.native_wheel_builder_sources import archive_root_for_build, reuse_sealed_native_inputs
from wheel_lib.native_wheel_builder_wheel import build_wheel, copy_engine_bundle


def command_build(arguments: argparse.Namespace) -> None:
	if platform.system() != "Darwin" or platform.machine() != "arm64":
		raise NativeBuildError(
			"initial native-wheel currently proves only macOS arm64; run on an arm64 macOS host"
		)
	if arguments.sealed_input_root:
		layout = reuse_sealed_native_inputs(arguments.output_root, arguments.sealed_input_root)
	else:
		layout = build_rdkit(arguments.output_root, archive_root_for_build(arguments))
	rdkit_libraries = sorted(layout.lib_dir.glob("libRDKit*.dylib"))
	if not rdkit_libraries:
		raise NativeBuildError(f"no RDKit dylibs were installed in {layout.lib_dir}")
	variants = detect_variants(rdkit_libraries)
	adapter = configure_adapter(arguments.output_root, layout)
	wheel, source_closure, worktree_source_closure, staged_bundle = build_wheel(
		arguments.output_root, adapter, layout, TARGET
	)
	if arguments.engine_bundle_dir is not None:
		copy_engine_bundle(staged_bundle, arguments.engine_bundle_dir)
		native_wheel_publication.validate_wheel_engine_bundle(
			wheel, arguments.engine_bundle_dir, BUNDLE_MANIFEST_NAME, BUNDLE_SCHEMA,
			executable_bundle_target(), ADAPTER_ABI_VERSION, ADAPTER_NAME, sha256,
		)
	try:
		write_build_receipt(
			arguments.output_root,
			FERRUM_RDKIT_PROFILE,
			ADAPTER_ABI_VERSION,
			layout.cmake_options or FERRUM_RDKIT_PROFILE.cmake_options,
			layout.toolchain or {"native_inputs": "validated-sealed-input-root"},
			layout.provenance_audit or {"native_inputs": "validated-sealed-input-root"},
			{"path": find_maturin(), "version": tool_version(find_maturin())},
			rust_toolchain_receipt(),
			variants,
			worktree_source_closure,
			source_closure,
			wheel,
			MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names,
		)
		validate_build_receipt(
			arguments.output_root / "native-wheel-build-receipt.json", wheel, source_closure,
			worktree_source_closure,
		)
	except NativeReceiptError as error:
		raise NativeBuildError(str(error)) from error
	emit_artifact_result("wheel", wheel)

#============================================
def command_adapter(arguments: argparse.Namespace) -> None:
	layout = rdkit_layout_from_output_root(arguments.rdkit_output_root)
	adapter = configure_adapter(arguments.output_root, layout, "RelWithDebInfo")
	package_libs = arguments.output_root / "replacement-package-libs"
	copy_and_rewrite_closure(adapter, layout.graphmol_library, package_libs)
	assert_packaged_library_closure(package_libs)
	emit_artifact_result("adapter", package_libs / "libferrum_chem.dylib")

#============================================
def command_validate_publication(arguments: argparse.Namespace) -> None:
	"""Validate one copied publication candidate without rebuilding native sources."""
	validate_publication_candidate(
		arguments.receipt, arguments.wheel, arguments.staged_source_root,
		arguments.worktree_source_root,
	)
	validate_publication_engine_bundle(arguments.engine_bundle)
	native_wheel_publication.validate_wheel_engine_bundle(
		arguments.wheel, arguments.engine_bundle, BUNDLE_MANIFEST_NAME, BUNDLE_SCHEMA,
		executable_bundle_target(), ADAPTER_ABI_VERSION, ADAPTER_NAME, sha256,
	)
	print(json.dumps({
		"action": "validate-publication",
		"schema": MACHINE_RESULT_SCHEMA,
		"validated": True,
	}, sort_keys=True))

#============================================
def command_publish_publication(arguments: argparse.Namespace) -> None:
	"""Validate one evidenced native-plus-Qt pair and atomically select it."""
	try:
		qt_evidence = (
			getattr(arguments, "qt_wheel", None), getattr(arguments, "qt_source_root", None),
			getattr(arguments, "qt_source_closure", None),
			getattr(arguments, "qt_worktree_source_root", None),
			getattr(arguments, "qt_worktree_source_closure", None),
			getattr(arguments, "pair_receipt", None),
		)
		if any(value is None for value in qt_evidence):
			raise NativeBuildError("developer pair publication requires every Qt evidence argument")
		native_wheel_publication.publish_current_publication(
			arguments.candidate_root, arguments.current_pointer, arguments.receipt, arguments.wheel,
			arguments.staged_source_root, FERRUM_SOURCE_CLOSURE_SCHEMA,
			FERRUM_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES, arguments.worktree_source_root,
			FERRUM_WORKTREE_SOURCE_CLOSURE_SCHEMA,
			FERRUM_WORKTREE_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES,
			FERRUM_WORKTREE_SOURCE_CLOSURE_EXCLUDED_SUFFIXES, arguments.engine_bundle,
			BUNDLE_MANIFEST_NAME, BUNDLE_SCHEMA, executable_bundle_target(), ADAPTER_ABI_VERSION,
			ADAPTER_NAME, sha256,
			*qt_evidence,
		)
	except native_wheel_publication.NativePublicationError as error:
		raise NativeBuildError(str(error)) from error
	print(json.dumps({
		"action": "publish-publication",
		"schema": MACHINE_RESULT_SCHEMA,
		"published": True,
	}, sort_keys=True))

#============================================
def command_parse_artifact_result(arguments: argparse.Namespace) -> None:
	"""Validate one streamed native-wheel build result and print its wheel path."""
	output_root = arguments.output_root.resolve(strict=True)
	lines = sys.stdin.read().splitlines()
	if len(lines) != 1:
		raise NativeBuildError("native builder must emit exactly one JSON artifact line")
	try:
		record = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise NativeBuildError(f"native builder emitted invalid JSON: {error.msg}") from error
	if not isinstance(record, dict):
		raise NativeBuildError("native builder artifact result must be a JSON object")
	if record.get("schema") != MACHINE_RESULT_SCHEMA or record.get("action") != "wheel":
		raise NativeBuildError("native builder artifact result has the wrong schema or action")
	artifact_value = record.get("artifact")
	if not isinstance(artifact_value, str):
		raise NativeBuildError("native builder artifact result has no wheel path")
	artifact = Path(artifact_value)
	if not artifact.is_absolute():
		raise NativeBuildError("native builder wheel path must be absolute")
	try:
		resolved = artifact.resolve(strict=True)
	except FileNotFoundError as error:
		raise NativeBuildError(f"native builder reported a missing wheel: {artifact}") from error
	if artifact != resolved or not resolved.is_relative_to(output_root) or not resolved.is_file():
		raise NativeBuildError(
			"native builder wheel path is not a regular file beneath its fresh output root"
		)
	if resolved.suffix != ".whl":
		raise NativeBuildError("native builder artifact is not a wheel")
	print(resolved)

#============================================
def command_record_qt_worktree_source_closure(arguments: argparse.Namespace) -> None:
	"""Record the admitted Qt worktree source closure before staging its wheel."""
	closure = native_wheel_publication.qt_source_closure(arguments.worktree_source_root, sha256)
	arguments.closure_path.write_text(
		json.dumps(closure, sort_keys=True, separators=(",", ":")), encoding="utf-8"
	)

#============================================
def command_stage_qt_source_tree(arguments: argparse.Namespace) -> None:
	"""Stage exactly one recorded Qt worktree source closure and record its closure."""
	admission = native_wheel_publication.load_qt_source_closure(arguments.admission_path)
	closure = native_wheel_publication.stage_qt_source_tree(
		arguments.worktree_source_root, arguments.destination, admission, sha256,
	)
	arguments.closure_path.write_text(
		json.dumps(closure, sort_keys=True, separators=(",", ":")), encoding="utf-8"
	)
