"""Publication and source-closure fixtures used only by the builder self-test runner."""

from __future__ import annotations

import contextlib
import io
import json
import os
import shutil
import tempfile
import types
import unittest.mock
import zipfile
from pathlib import Path

import wheel_lib.native_wheel_publication as native_wheel_publication
from wheel_lib.native_wheel_receipt import (
	NativeReceiptError,
	_tree_digest_record,
	_tree_relative_path_key,
	directory_tree_sha256,
)


#============================================
def _reject(api: types.ModuleType, action: object, label: str) -> None:
	"""Require one deliberately invalid fixture to fail through the builder API."""
	try:
		action()
	except (api.NativeBuildError, ValueError):
		return
	raise api.NativeBuildError(f"native profile self-test accepted {label}")


#============================================
def _run_publication_wrapper_fixtures(api: types.ModuleType) -> None:
	"""Exercise facade-bound publication helpers with deterministic collaborators."""
	with tempfile.TemporaryDirectory() as temporary:
		root = Path(temporary)
		output_root = root / "output"
		output_root.mkdir()
		wheel = root / "fixture.whl"
		with zipfile.ZipFile(wheel, "w") as contents:
			contents.writestr("ferrum_chem.so", b"extension")
			contents.writestr(".dylibs/libRDKitGraphMol.1.dylib", b"library")
		validated_members: list[list[str]] = []
		def validate_members(names: list[str]) -> None:
			if names != ["ferrum_chem.so", ".dylibs/libRDKitGraphMol.1.dylib"]:
				raise api.NativeBuildError("invalid members")
			validated_members.append(names)
		def extract_members(contents: zipfile.ZipFile, destination: Path) -> None:
			api.safe_extract_zip_members(contents, destination)
		def assert_closure(extension: Path, libraries: Path) -> None:
			if extension.read_bytes() != b"extension" or not (libraries / "libRDKitGraphMol.1.dylib").is_file():
				raise api.NativeBuildError("wheel closure fixture lost its packaged native payload")
		api.native_wheel_publication.audit_wheel_closure(
			wheel, output_root, validate_members, extract_members, assert_closure,
			api.NativeBuildError,
		)
		if validated_members != [["ferrum_chem.so", ".dylibs/libRDKitGraphMol.1.dylib"]]:
			raise api.NativeBuildError("wheel closure fixture did not receive the packaged member list")
		malformed_wheel = root / "malformed.whl"
		with zipfile.ZipFile(malformed_wheel, "w") as contents:
			contents.writestr("unexpected.txt", b"not a native wheel")
		_reject(
			api,
			lambda: api.native_wheel_publication.audit_wheel_closure(
				malformed_wheel, root / "malformed-output", validate_members,
				extract_members, assert_closure, api.NativeBuildError,
			),
			"malformed wheel closure",
		)
		adapter = root / api.ADAPTER_NAME
		graphmol = root / "libRDKitGraphMol.1.dylib"
		adapter.write_bytes(b"adapter")
		graphmol.write_bytes(b"graphmol")
		destination = output_root / "ferrum-engine-bundle"
		def rewrite_closure(source_adapter: Path, source_graphmol: Path, target: Path) -> None:
			(target / source_adapter.name).write_bytes(source_adapter.read_bytes())
			(target / source_graphmol.name).write_bytes(source_graphmol.read_bytes())
		def assert_bundle_closure(bundle: Path) -> None:
			if not (bundle / graphmol.name).is_file():
				raise api.NativeBuildError("engine bundle fixture lost the rewritten dependency")
		if api.native_wheel_publication.build_engine_bundle(
			output_root, adapter, graphmol, destination, rewrite_closure, assert_bundle_closure,
			api.BUNDLE_MANIFEST_NAME, api.BUNDLE_SCHEMA, api.ADAPTER_ABI_VERSION,
			api.ADAPTER_NAME, api.sha256, api.NativeBuildError,
		) != destination.resolve():
			raise api.NativeBuildError("engine bundle facade did not return its destination")
		api.validate_publication_engine_bundle(destination)
		wheel_bundle = root / "bundle-fixture.whl"
		with zipfile.ZipFile(wheel_bundle, "w") as contents:
			for member in destination.iterdir():
				contents.writestr(f"ferrum-engine-bundle/{member.name}", member.read_bytes())
		api.native_wheel_publication.validate_wheel_engine_bundle(
			wheel_bundle, destination, api.BUNDLE_MANIFEST_NAME, api.BUNDLE_SCHEMA,
			api.executable_bundle_target(), api.ADAPTER_ABI_VERSION, api.ADAPTER_NAME, api.sha256,
		)
		with zipfile.ZipFile(wheel_bundle, "w") as contents:
			contents.writestr("ferrum-engine-bundle/ferrum-engine-bundle-v1.json", b"altered")
		_reject(
			api,
			lambda: api.native_wheel_publication.validate_wheel_engine_bundle(
				wheel_bundle, destination, api.BUNDLE_MANIFEST_NAME, api.BUNDLE_SCHEMA,
				api.executable_bundle_target(), api.ADAPTER_ABI_VERSION, api.ADAPTER_NAME, api.sha256,
			),
			"wheel engine bundle differing from its CLI bundle",
		)
		(destination / api.ADAPTER_NAME).write_bytes(b"altered adapter")
		_reject(api, lambda: api.validate_publication_engine_bundle(destination), "invalid engine bundle adapter")
		_reject(api, lambda: api.native_wheel_publication.build_engine_bundle(
			output_root, adapter, graphmol, destination, rewrite_closure, assert_bundle_closure,
			api.BUNDLE_MANIFEST_NAME, api.BUNDLE_SCHEMA, api.ADAPTER_ABI_VERSION,
			api.ADAPTER_NAME, api.sha256, api.NativeBuildError,
		), "existing engine-bundle destination")
		_reject(api, lambda: api.native_wheel_publication.build_engine_bundle(
			output_root, adapter, graphmol, root / "escaped-bundle", rewrite_closure,
			assert_bundle_closure, api.BUNDLE_MANIFEST_NAME, api.BUNDLE_SCHEMA,
			api.ADAPTER_ABI_VERSION, api.ADAPTER_NAME, api.sha256, api.NativeBuildError,
		), "escaping engine-bundle destination")
		artifact = root / "artifact.whl"
		artifact.write_bytes(b"artifact")
		stdout = io.StringIO()
		with contextlib.redirect_stdout(stdout):
			api.emit_artifact_result("wheel", artifact)
		records = stdout.getvalue().splitlines()
		if records != [json.dumps({
			"schema": api.MACHINE_RESULT_SCHEMA,
			"action": "wheel",
			"artifact": str(artifact.resolve()),
		}, sort_keys=True)]:
			raise api.NativeBuildError("artifact emitter fixture did not emit one canonical JSON record")
		_reject(api, lambda: api.emit_artifact_result("wheel", root / "missing.whl"), "missing artifact result")


#============================================
def _run_tree_fixtures(api: types.ModuleType) -> None:
	"""Verify path identity and tree-digest rejection behavior."""
	case_key = _tree_relative_path_key("GraphMol/Case.h", "tree self-test")[1]
	if case_key != _tree_relative_path_key("GraphMol/case.h", "tree self-test")[1]:
		raise api.NativeBuildError("tree self-test did not normalize case-fold identities")
	nfc_name = "GraphMol/caf\N{LATIN SMALL LETTER E WITH ACUTE}.h"
	nfd_name = "GraphMol/cafe\N{COMBINING ACUTE ACCENT}.h"
	nfc_key = _tree_relative_path_key(nfc_name, "tree self-test")[1]
	nfd_key = _tree_relative_path_key(nfd_name, "tree self-test")[1]
	if nfc_key != nfd_key:
		raise api.NativeBuildError("tree self-test did not normalize Unicode path identities")
	try:
		_tree_relative_path_key("GraphMol/invalid-\udcff.h", "tree self-test")
	except NativeReceiptError:
		pass
	else:
		raise api.NativeBuildError("tree self-test accepted a non-UTF-8 path")
	first_record = _tree_digest_record(b"F", r"a\\0b", "c")
	second_record = _tree_digest_record(b"F", "a", r"b\\0c")
	if first_record == second_record:
		raise api.NativeBuildError("tree self-test accepted ambiguous literal backslash-zero names")
	with tempfile.TemporaryDirectory() as temporary:
		tree_root = Path(temporary) / "tree"
		tree_root.mkdir()
		fifo = tree_root / "unsupported-fifo"
		try:
			os.mkfifo(fifo)
		except OSError as error:
			raise api.NativeBuildError("tree self-test could not create a portable FIFO fixture") from error
		try:
			directory_tree_sha256(tree_root, "tree self-test")
		except NativeReceiptError:
			pass
		else:
			raise api.NativeBuildError("tree self-test accepted a FIFO special file")


#============================================
def _run_live_worktree_publication_refusal_fixture(
		api: types.ModuleType, root: Path, source: Path, stage: Path,
		source_closure: dict[str, object], worktree_source_closure: dict[str, object],
		) -> None:
	"""Refuse a final live-source drift without replacing the known-good publication."""
	publication_root = root / "publications"
	publication_root.mkdir()
	prior = publication_root / ".native-publication-prior"
	prior.mkdir()
	prior_payload = prior / "prior-payload.txt"
	prior_payload.write_text("known-good", encoding="utf-8")
	current = publication_root / "current"
	current.symlink_to(prior.name)
	candidate = publication_root / ".native-publication-candidate"
	wheelhouse = candidate / "wheelhouse"
	wheelhouse.mkdir(parents=True)
	wheel = wheelhouse / "ferrum_chem-fixture.whl"
	wheel.write_bytes(b"wheel")
	bundle = candidate / "ferrum-engine-bundle"
	bundle.mkdir()
	adapter = bundle / api.ADAPTER_NAME
	adapter.write_bytes(b"adapter")
	(bundle / api.BUNDLE_MANIFEST_NAME).write_bytes(api.engine_bundle_manifest(
		[adapter], api.BUNDLE_SCHEMA, api.ADAPTER_ABI_VERSION, api.ADAPTER_NAME, api.sha256
	))
	receipt = candidate / "native-wheel-build-receipt.json"
	receipt.write_text(json.dumps({
		"ferrum_source_closure": source_closure,
		"ferrum_worktree_source_closure": worktree_source_closure,
		"wheel": {"filename": wheel.name, "sha256": api.sha256(wheel)},
	}), encoding="utf-8")
	(source / "crates" / "document" / "src" / "session" / "direct_bond.rs").write_text(
		"changed after stage manifest", encoding="utf-8"
	)
	arguments = types.SimpleNamespace(
		candidate_root=candidate, current_pointer=current, receipt=receipt, wheel=wheel,
		staged_source_root=stage, worktree_source_root=source, engine_bundle=bundle,
	)
	_reject(
		api, lambda: api.command_publish_publication(arguments),
		"live worktree mutation at the real publication boundary",
	)
	if not current.is_symlink() or os.readlink(current) != prior.name:
		raise api.NativeBuildError("live worktree mutation replaced the prior current publication")
	if prior_payload.read_text(encoding="utf-8") != "known-good":
		raise api.NativeBuildError("live worktree mutation changed the prior publication payload")


#============================================
def _run_live_qt_worktree_publication_refusal_fixture(api: types.ModuleType) -> None:
	"""Refuse a Qt edit after staging without changing the selected wheel pair."""
	with tempfile.TemporaryDirectory() as temporary:
		root = Path(temporary)
		publication_root = root / "publications"
		publication_root.mkdir()
		prior = publication_root / ".native-publication-prior"
		prior.mkdir()
		prior_payload = prior / "prior-payload.txt"
		prior_payload.write_text("known-good", encoding="utf-8")
		current = publication_root / "current"
		current.symlink_to(prior.name)
		candidate = publication_root / ".native-publication-candidate"
		wheelhouse = candidate / "wheelhouse"
		wheelhouse.mkdir(parents=True)
		native_wheel = wheelhouse / "ferrum_chem-fixture.whl"
		native_wheel.write_bytes(b"native wheel")
		native_source = root / "native-source"
		native_input = native_source / "crates" / "document" / "src" / "session" / "direct_bond.rs"
		native_input.parent.mkdir(parents=True)
		native_input.write_text("admitted native source", encoding="utf-8")
		native_stage = root / "native-stage"
		shutil.copytree(native_source, native_stage)
		native_source_closure = api.ferrum_source_closure(native_stage)
		native_worktree_source_closure = api.ferrum_worktree_source_closure(native_source)
		bundle = candidate / "ferrum-engine-bundle"
		bundle.mkdir()
		adapter = bundle / api.ADAPTER_NAME
		adapter.write_bytes(b"adapter")
		(bundle / api.BUNDLE_MANIFEST_NAME).write_bytes(api.engine_bundle_manifest(
			[adapter], api.BUNDLE_SCHEMA, api.ADAPTER_ABI_VERSION, api.ADAPTER_NAME, api.sha256
		))
		native_receipt = candidate / "native-wheel-build-receipt.json"
		native_receipt.write_text(json.dumps({
			"ferrum_source_closure": native_source_closure,
			"ferrum_worktree_source_closure": native_worktree_source_closure,
			"wheel": {"filename": native_wheel.name, "sha256": api.sha256(native_wheel)},
		}), encoding="utf-8")
		qt_worktree = root / "qt-worktree"
		qt_module = qt_worktree / "ferrum_qt" / "__init__.py"
		qt_module.parent.mkdir(parents=True)
		qt_module.write_text("admitted Qt source\n", encoding="utf-8")
		(qt_worktree / "ferrum_qt" / "unshipped.txt").write_text("source-only\n", encoding="utf-8")
		(qt_worktree / "build" / "lib" / "ferrum_qt").mkdir(parents=True)
		(qt_worktree / "build" / "lib" / "ferrum_qt" / "__init__.py").write_text(
			"stale build output\n", encoding="utf-8"
		)
		(qt_worktree / "ferrum_qt.egg-info").mkdir()
		(qt_worktree / "ferrum_qt.egg-info" / "SOURCES.txt").write_text("generated\n", encoding="utf-8")
		qt_stage = root / "qt-stage"
		qt_worktree_closure = api.native_wheel_publication.qt_source_closure(qt_worktree, api.sha256)
		qt_staged_closure = api.native_wheel_publication.stage_qt_source_tree(
			qt_worktree, qt_stage, qt_worktree_closure, api.sha256,
		)
		if (qt_stage / "build").exists() or (qt_stage / "ferrum_qt.egg-info").exists():
			raise api.NativeBuildError("Qt closure staging retained excluded generated build content")
		if (qt_stage / "ferrum_qt" / "__init__.py").read_text(encoding="utf-8") != "admitted Qt source\n":
			raise api.NativeBuildError("Qt closure staging did not retain the admitted package source")
		qt_staged_closure_path = root / "qt-staged-source-closure.json"
		qt_staged_closure_path.write_text(json.dumps(qt_staged_closure), encoding="utf-8")
		qt_worktree_closure_path = root / "qt-worktree-source-closure.json"
		qt_worktree_closure_path.write_text(json.dumps(qt_worktree_closure), encoding="utf-8")
		qt_wheel = wheelhouse / "ferrum_qt-fixture.whl"
		with zipfile.ZipFile(qt_wheel, "w") as contents:
			contents.writestr("ferrum_qt/__init__.py", b"admitted Qt source\n")
			contents.writestr("ferrum_qt-fixture.dist-info/METADATA", b"metadata\n")
		delivered_payload = api.native_wheel_publication.validate_qt_wheel_package_payload(
			qt_stage, qt_wheel, api.sha256,
		)
		if [member["path"] for member in delivered_payload["members"]] != ["ferrum_qt/__init__.py"]:
			raise api.NativeBuildError("Qt payload fixture did not allow admitted source omitted from wheel")
		qt_module.write_text("changed after Qt staging\n", encoding="utf-8")
		arguments = types.SimpleNamespace(
			candidate_root=candidate, current_pointer=current, receipt=native_receipt,
			wheel=native_wheel, staged_source_root=native_stage,
			worktree_source_root=native_source, engine_bundle=bundle, qt_wheel=qt_wheel,
			qt_source_root=qt_stage, qt_source_closure=qt_staged_closure_path,
			qt_worktree_source_root=qt_worktree,
			qt_worktree_source_closure=qt_worktree_closure_path,
			pair_receipt=candidate / "developer-wheel-publication-receipt.json",
		)
		_reject(
			api, lambda: api.command_publish_publication(arguments),
			"live Qt worktree mutation at the real publication boundary",
		)
		if not current.is_symlink() or os.readlink(current) != prior.name:
			raise api.NativeBuildError("live Qt mutation replaced the prior current publication")
		if prior_payload.read_text(encoding="utf-8") != "known-good":
			raise api.NativeBuildError("live Qt mutation changed the prior publication payload")
		qt_module.write_text("admitted Qt source\n", encoding="utf-8")
		qt_wheel.unlink()
		with zipfile.ZipFile(qt_wheel, "w") as contents:
			contents.writestr("ferrum_qt/__init__.py", b"stale wheel payload\n")
			contents.writestr("ferrum_qt-fixture.dist-info/METADATA", b"metadata\n")
		_reject(
			api, lambda: api.command_publish_publication(arguments),
			"Qt wheel package payload that differs from admitted staged source",
		)
		if not current.is_symlink() or os.readlink(current) != prior.name:
			raise api.NativeBuildError("mismatched Qt wheel replaced the prior current publication")


#============================================
def _run_failed_current_pointer_replacement_fixture(
		api: types.ModuleType, root: Path, source: Path, stage: Path,
		source_closure: dict[str, object], worktree_source_closure: dict[str, object],
		) -> None:
	"""Keep the selected publication intact when the real atomic replacement fails."""
	publication_root = root / "replacement-publications"
	publication_root.mkdir()
	prior = publication_root / ".native-publication-prior"
	prior.mkdir()
	prior_payload = prior / "prior-payload.txt"
	prior_payload.write_text("known-good", encoding="utf-8")
	current = publication_root / "current"
	current.symlink_to(prior.name)
	candidate = publication_root / ".native-publication-candidate"
	wheelhouse = candidate / "wheelhouse"
	wheelhouse.mkdir(parents=True)
	wheel = wheelhouse / "ferrum_chem-fixture.whl"
	wheel.write_bytes(b"wheel")
	bundle = candidate / "ferrum-engine-bundle"
	bundle.mkdir()
	adapter = bundle / api.ADAPTER_NAME
	adapter.write_bytes(b"adapter")
	(bundle / api.BUNDLE_MANIFEST_NAME).write_bytes(api.engine_bundle_manifest(
		[adapter], api.BUNDLE_SCHEMA, api.ADAPTER_ABI_VERSION, api.ADAPTER_NAME, api.sha256
	))
	receipt = candidate / "native-wheel-build-receipt.json"
	receipt.write_text(json.dumps({
		"ferrum_source_closure": source_closure,
		"ferrum_worktree_source_closure": worktree_source_closure,
		"wheel": {"filename": wheel.name, "sha256": api.sha256(wheel)},
	}), encoding="utf-8")
	qt_worktree = root / "replacement-qt-worktree"
	qt_module = qt_worktree / "ferrum_qt" / "__init__.py"
	qt_module.parent.mkdir(parents=True)
	qt_module.write_text("pair fixture\n", encoding="utf-8")
	qt_stage = root / "replacement-qt-stage"
	qt_worktree_closure = api.native_wheel_publication.qt_source_closure(qt_worktree, api.sha256)
	qt_staged_closure = api.native_wheel_publication.stage_qt_source_tree(
		qt_worktree, qt_stage, qt_worktree_closure, api.sha256,
	)
	qt_staged_closure_path = root / "replacement-qt-staged-source-closure.json"
	qt_staged_closure_path.write_text(json.dumps(qt_staged_closure), encoding="utf-8")
	qt_worktree_closure_path = root / "replacement-qt-worktree-source-closure.json"
	qt_worktree_closure_path.write_text(json.dumps(qt_worktree_closure), encoding="utf-8")
	qt_wheel = wheelhouse / "ferrum_qt-fixture.whl"
	with zipfile.ZipFile(qt_wheel, "w") as contents:
		contents.writestr("ferrum_qt/__init__.py", b"pair fixture\n")
		contents.writestr("ferrum_qt-fixture.dist-info/METADATA", b"metadata\n")
	pair_receipt = candidate / "developer-wheel-publication-receipt.json"
	publication_arguments = (
		candidate, current, receipt, wheel, stage, api.FERRUM_SOURCE_CLOSURE_SCHEMA,
		api.FERRUM_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES, source,
		api.FERRUM_WORKTREE_SOURCE_CLOSURE_SCHEMA,
		api.FERRUM_WORKTREE_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES,
		api.FERRUM_WORKTREE_SOURCE_CLOSURE_EXCLUDED_SUFFIXES, bundle,
		api.BUNDLE_MANIFEST_NAME, api.BUNDLE_SCHEMA, api.executable_bundle_target(),
		api.ADAPTER_ABI_VERSION, api.ADAPTER_NAME, api.sha256,
	)
	_reject(
		api, lambda: native_wheel_publication.publish_current_publication(
			*publication_arguments, None, None, None, None, None, None,
		),
		"native-only current publication",
	)
	if not current.is_symlink() or os.readlink(current) != prior.name:
		raise api.NativeBuildError("native-only publication changed the selected publication")
	with unittest.mock.patch.object(
		native_wheel_publication.os, "replace", side_effect=OSError("fixture replacement failure")
	):
		try:
			native_wheel_publication.publish_current_publication(
				*publication_arguments, qt_wheel, qt_stage, qt_staged_closure_path,
				qt_worktree, qt_worktree_closure_path, pair_receipt,
			)
		except native_wheel_publication.NativePublicationError:
			pass
		else:
			raise api.NativeBuildError("failed current replacement did not raise NativePublicationError")
	if not current.is_symlink() or os.readlink(current) != prior.name:
		raise api.NativeBuildError("failed current replacement changed the selected publication")
	if current.resolve() == candidate:
		raise api.NativeBuildError("failed current replacement selected the candidate publication")
	if prior_payload.read_text(encoding="utf-8") != "known-good":
		raise api.NativeBuildError("failed current replacement changed the prior publication payload")
	if list(publication_root.glob(".native-pointer-stage-*")):
		raise api.NativeBuildError("failed current replacement left a temporary pointer stage")


#============================================
def _run_ferrum_source_closure_fixtures(api: types.ModuleType) -> None:
	"""Prove the canonical staged source subset admits no authored-source drift."""
	with tempfile.TemporaryDirectory() as temporary:
		root = Path(temporary)
		source = root / "source"
		output_root = root / "output"
		binding = source / "crates" / "api" / "src" / "python_binding" / "document_session_binding.rs"
		stub = source / "crates" / "api" / "wheel_metadata" / "ferrum_chem.pyi"
		binding.parent.mkdir(parents=True)
		stub.parent.mkdir(parents=True)
		binding.write_text("getter", encoding="utf-8")
		admission = source / "crates" / "document" / "src" / "session" / "direct_bond.rs"
		admission.parent.mkdir(parents=True)
		admission.write_text("admission", encoding="utf-8")
		stub.write_text("class DocumentSession: ...\n", encoding="utf-8")
		(stub.parent / "py.typed").write_text("", encoding="utf-8")
		project = source / "crates" / "api" / "python"
		project.mkdir()
		(project / "pyproject.toml").write_text("[build-system]\n", encoding="utf-8")
		build_script = source / "crates" / "api" / "build.rs"
		build_script.write_text("pyo3_build_config::add_extension_module_link_args();\n", encoding="utf-8")
		protocol = source / "crates" / "api" / "protocol"
		protocol.mkdir()
		(protocol / "ferrum-operation-v1.schema.json").write_text("{}", encoding="utf-8")
		(source / "Cargo.lock").write_text("lock", encoding="utf-8")
		worktree_source_closure = api.ferrum_worktree_source_closure(source)
		stage_project = api.stage_python_project(
			output_root, source,
			lambda copied: api.require_matching_worktree_source_closure(
				worktree_source_closure, api.ferrum_worktree_source_closure(copied),
				"while staging fixture",
			),
		)
		stage = stage_project.parents[2]
		staged_binding = stage / "crates" / "api" / "src" / "python_binding" / "document_session_binding.rs"
		source_closure = api.ferrum_source_closure(stage)
		if source_closure == api.ferrum_source_closure(source):
			raise api.NativeBuildError("source closure fixture did not capture Maturin staging transforms")
		authored_target = stage / "target"
		authored_target.mkdir()
		(authored_target / "authored-source.txt").write_text("must be admitted", encoding="utf-8")
		if source_closure == api.ferrum_source_closure(stage):
			raise api.NativeBuildError("source subset silently ignored an authored staged directory")
		shutil.rmtree(authored_target)
		notices = stage / "crates" / "api" / "wheel_metadata" / "licenses"
		notices.mkdir()
		(notices / "RDKIT-BSD-3-CLAUSE.txt").write_text("generated notice", encoding="utf-8")
		package_libs = stage / "crates" / "api" / "python" / ".dylibs"
		package_libs.mkdir()
		(package_libs / "libferrum_chem.dylib").write_bytes(b"generated library")
		nested_package_libs = stage / "crates" / "api" / "python" / "ferrum_chem" / ".dylibs"
		nested_package_libs.mkdir(parents=True)
		nested_library = nested_package_libs / "libferrum_chem.dylib"
		nested_library.write_bytes(b"generated package library")
		if source_closure != api.ferrum_source_closure(stage):
			raise api.NativeBuildError("source subset included builder-generated staging payloads")
		nested_library.write_bytes(b"changed generated package library")
		if source_closure != api.ferrum_source_closure(stage):
			raise api.NativeBuildError("source subset included nested builder-generated staging payloads")
		wheel = root / "wheel.whl"
		wheel.write_bytes(b"wheel")
		receipt = root / "native-wheel-build-receipt.json"
		receipt.write_text(json.dumps({
			"ferrum_source_closure": source_closure,
			"ferrum_worktree_source_closure": worktree_source_closure,
			"wheel": {"filename": wheel.name, "sha256": api.sha256(wheel)},
		}), encoding="utf-8")
		api.validate_build_receipt(receipt, wheel, source_closure, worktree_source_closure)
		api.validate_publication_candidate(receipt, wheel, stage, source)
		_run_failed_current_pointer_replacement_fixture(
			api, root, source, stage, source_closure, worktree_source_closure,
		)
		_run_live_worktree_publication_refusal_fixture(
			api, root, source, stage, source_closure, worktree_source_closure,
		)
		admission.write_text("changed admission", encoding="utf-8")
		try:
			api.validate_publication_candidate(receipt, wheel, stage, source)
		except api.NativeBuildError as error:
			if "crates/document/src/session/direct_bond.rs" not in str(error):
				raise api.NativeBuildError(
					"worktree mutation refusal omitted the changed relative source path"
				) from error
		else:
			raise api.NativeBuildError(
				"publication validation accepted a worktree source mutation after staging"
			)
		staged_binding.write_text("changed getter", encoding="utf-8")
		_reject(api, lambda: api.validate_publication_candidate(receipt, wheel, stage, source), "changed staged authored source")
		staged_binding.write_text("getter", encoding="utf-8")
		wheel.write_bytes(b"changed wheel")
		_reject(api, lambda: api.validate_publication_candidate(receipt, wheel, stage, source), "changed publication wheel")
		wheel.write_bytes(b"wheel")
		receipt.write_text(json.dumps({"ferrum_source_closure": {"schema": "changed"}, "wheel": {"filename": wheel.name, "sha256": api.sha256(wheel)}}), encoding="utf-8")
		_reject(api, lambda: api.validate_publication_candidate(receipt, wheel, stage, source), "changed publication receipt")
		receipt.write_text(json.dumps({"wheel": {"filename": wheel.name, "sha256": api.sha256(wheel)}}), encoding="utf-8")
		_reject(api, lambda: api.validate_build_receipt(receipt, wheel, source_closure, worktree_source_closure), "receipt without source closure")


#============================================
def run(api: types.ModuleType) -> None:
	"""Run publication, wheel-closure, and staged-source proof fixtures."""
	_run_publication_wrapper_fixtures(api)
	_run_tree_fixtures(api)
	_run_ferrum_source_closure_fixtures(api)
	_run_live_qt_worktree_publication_refusal_fixture(api)
