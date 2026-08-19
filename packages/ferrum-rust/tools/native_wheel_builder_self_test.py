"""Pure fixture checks for the native-wheel builder boundary.

This module receives the builder module explicitly so it tests the same public
helpers as the command without importing or executing the builder as a script.
"""

from __future__ import annotations

import argparse
import collections.abc
import hashlib
import io
import json
import os
import stat
import tarfile
import tempfile
import types
import urllib.request
import zipfile
from pathlib import Path

from native_wheel_macho import (
	NativeMachoError,
	deduplicate_paths_by_identity,
	self_test as macho_self_test,
)
from native_wheel_packaging import NOTICE_FILENAMES, NativePackagingError, inject_root_metadata
from native_wheel_policy import NativePolicyError, self_test as policy_self_test
from native_wheel_profile import (
	FERRUM_RDKIT_PROFILE,
	MACOS_ARM64_NATIVE_CLOSURE,
	PinnedSource,
	RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES,
	RdkitCapabilityProfile,
)
from native_wheel_receipt import (
	NativeReceiptError,
	_tree_digest_record,
	_tree_relative_path_key,
	directory_tree_sha256,
	self_test as receipt_self_test,
	sha256,
)


#============================================
def _reject(
	api: types.ModuleType, action: collections.abc.Callable[[], object], label: str
) -> None:
	"""Require one policy-breaking action to fail.

	Args:
		api: The imported builder module that owns the production helpers.
		action: The operation that must reject the deliberately invalid fixture.
		label: A precise description of the invalid fixture.
	"""
	try:
		action()
	except (api.NativeBuildError, NativeMachoError, argparse.ArgumentTypeError, ValueError):
		return
	raise api.NativeBuildError(f"native profile self-test accepted {label}")


#============================================
def _run_policy_fixtures(api: types.ModuleType) -> None:
	"""Run pure policy, receipt, and Mach-O module fixtures."""
	try:
		policy_self_test()
		receipt_self_test(FERRUM_RDKIT_PROFILE)
		macho_self_test()
	except NativePolicyError as error:
		raise api.NativeBuildError(str(error)) from error
	except NativeReceiptError as error:
		raise api.NativeBuildError(str(error)) from error


#============================================
def _run_engine_bundle_fixtures(api: types.ModuleType) -> None:
	"""Verify the fixed engine manifest and narrowly admitted temporary root."""
	with tempfile.TemporaryDirectory() as temporary:
		member = Path(temporary) / api.ADAPTER_NAME
		member.write_bytes(b"adapter")
		manifest = json.loads(api.engine_bundle_manifest(
			[member], api.BUNDLE_SCHEMA, api.ADAPTER_ABI_VERSION, api.ADAPTER_NAME, api.sha256
		))
	if manifest != {
		"schema": api.BUNDLE_SCHEMA,
		"target": api.executable_bundle_target(),
		"adapter_abi_version": api.ADAPTER_ABI_VERSION,
		"adapter": api.ADAPTER_NAME,
		"members": [{"path": api.ADAPTER_NAME, "sha256": hashlib.sha256(b"adapter").hexdigest()}],
	}:
		raise api.NativeBuildError("engine bundle manifest fixture differs from the fixed CLI contract")
	if manifest.get("members") != [{"path": api.ADAPTER_NAME, "sha256": hashlib.sha256(b"adapter").hexdigest()}]:
		raise api.NativeBuildError("engine bundle manifest fixture lacks its closure digest")
	accepted = api.output_path("/private/tmp/ferrum-native-self-test")
	if accepted != Path("/private/tmp/ferrum-native-self-test"):
		raise api.NativeBuildError("engine bundle fixture rejected its admitted temporary output root")
	_reject(
		api,
		lambda: api.output_path("/private/tmp/unrelated-output"),
		"unscoped temporary output root",
	)


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
			raise api.NativeBuildError(
				"tree self-test could not create a portable FIFO fixture"
			) from error
		try:
			directory_tree_sha256(tree_root, "tree self-test")
		except NativeReceiptError:
			pass
		else:
			raise api.NativeBuildError("tree self-test accepted a FIFO special file")


#============================================
def _minimal_rdkit_options(api: types.ModuleType) -> set[str]:
	"""Return the production minimal RDKit options for fixture validation."""
	options = api.minimal_rdkit_options(
		Path("/catch2"),
		Path("/better-enums"),
		Path("/boost-config"),
	)
	return set(options)


#============================================
def _run_profile_configuration_fixtures(api: types.ModuleType) -> None:
	"""Verify the minimal profile retains required constraints and rejects drift."""
	options = _minimal_rdkit_options(api)
	required_options = (
		"-DRDK_INSTALL_STATIC_LIBS=OFF",
		"-DRDK_BUILD_PYTHON_WRAPPERS=OFF",
		"-DRDK_BUILD_CPP_TESTS=OFF",
		"-DRDK_BUILD_INCHI_SUPPORT=ON",
		"-DRDK_BUILD_COORDGEN_SUPPORT=OFF",
		"-DRDK_BUILD_MAEPARSER_SUPPORT=OFF",
		"-DFETCHCONTENT_FULLY_DISCONNECTED=ON",
		"-DRDK_BUILD_CHEMDRAW_SUPPORT=OFF",
		"-DRDK_BUILD_PUBCHEMSHAPE_SUPPORT=OFF",
		"-DRDK_BUILD_DESCRIPTORS3D=OFF",
		"-DRDK_BUILD_MOLINTERCHANGE_SUPPORT=OFF",
		"-DRDK_BUILD_SLN_SUPPORT=OFF",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Python3=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Eigen3=ON",
		"-DRDK_USE_BOOST_SERIALIZATION=OFF",
		"-DRDK_USE_BOOST_IOSTREAMS=OFF",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Catch2=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_TBB=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Inchi=ON",
		"-DINCHI_LIBRARIES=Inchi",
		"-DCATCH_BUILD_TESTING=OFF",
		"-DRDK_BUILD_FREETYPE_SUPPORT=OFF",
		"-DRDK_BUILD_THREADSAFE_SSS=ON",
	)
	for required in required_options:
		if required not in options:
			raise api.NativeBuildError(
				f"minimal RDKit configuration omitted required option: {required}"
			)
	without_python_wrapper = [
		option for option in options if option != "-DRDK_BUILD_PYTHON_WRAPPERS=OFF"
	]
	_reject(
		api,
		lambda: api.validate_rdkit_configuration(without_python_wrapper),
		"Python wrapper-enabled configuration",
	)
	_reject(
		api,
		lambda: api.validate_rdkit_configuration(
			[*options, "-DCMAKE_PREFIX_PATH=/opt/homebrew"]
		),
		"host CMake prefix",
	)
	with tempfile.TemporaryDirectory() as temporary:
		build = Path(temporary)
		resolved = {
			option.removeprefix("-D").split("=", 1)[0]: option.split("=", 1)[1]
			for option in FERRUM_RDKIT_PROFILE.cmake_options
		}
		resolved.update({
			"RDK_BUILD_SWIG_WRAPPERS": "OFF",
			"RDK_BUILD_THREADSAFE_SSS": "ON",
			"RDK_USE_FLEXBISON": "OFF",
		})
		resolved["CMAKE_INSTALL_PREFIX"] = str(build.parent / "rdkit-install")
		(build / "CMakeCache.txt").write_text(
			"".join(f"{key}:STRING={value}\n" for key, value in resolved.items()),
			encoding="utf-8",
		)
		api.validate_resolved_rdkit_configuration(build)
		(build / "CMakeCache.txt").write_text(
			(build / "CMakeCache.txt").read_text(encoding="utf-8").replace(
				"RDK_BUILD_SWIG_WRAPPERS:STRING=OFF",
				"RDK_BUILD_SWIG_WRAPPERS:STRING=ON",
			),
			encoding="utf-8",
		)
		_reject(
			api,
			lambda: api.validate_resolved_rdkit_configuration(build),
			"resolved SWIG wrapper support",
		)


#============================================
def _fixture_source(
	private_input: Path, name: str, filename: str, version: str = "fixture"
) -> PinnedSource:
	"""Create one deterministic source pin and matching archive fixture."""
	payload = f"ferrum-{name}".encode("ascii")
	archive = private_input / "downloads" / filename
	archive.parent.mkdir(exist_ok=True)
	archive.write_bytes(payload)
	return PinnedSource(name, version, "https://example.invalid/source", sha256(archive), filename)


#============================================
def _write_private_native_inputs(private_input: Path) -> RdkitCapabilityProfile:
	"""Materialize the narrow native-input tree used by manifest fixtures."""
	rdkit_include = private_input / "rdkit-install" / "include" / "rdkit"
	(rdkit_include / "GraphMol" / "Depictor").mkdir(parents=True)
	(rdkit_include / "GraphMol" / "SmilesParse").mkdir()
	(rdkit_include / "RDGeneral").mkdir()
	(private_input / "rdkit-install" / "lib").mkdir()
	boost_include = private_input / "dependencies" / "boost-headers" / "boost_1_91_0" / "boost"
	boost_include.mkdir(parents=True)
	(rdkit_include / "GraphMol" / "MolOps.h").touch()
	(rdkit_include / "GraphMol" / "Depictor" / "RDDepictor.h").touch()
	(rdkit_include / "GraphMol" / "SmilesParse" / "SmilesParse.h").touch()
	(rdkit_include / "GraphMol" / "SmilesParse" / "SmilesWrite.h").touch()
	(rdkit_include / "GraphMol" / "inchi.h").touch()
	(rdkit_include / "RDGeneral" / "types.h").touch()
	(rdkit_include / "GraphMol" / "Transitive.h").write_bytes(b"transitive RDKit header")
	(boost_include / "config.hpp").touch()
	(boost_include / "version.hpp").write_bytes(b"transitive Boost header")
	library_aliases = {
		name: name.replace(".1.dylib", ".2026.03.4.dylib")
		for name in RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES
	}
	for alias_name, target_name in library_aliases.items():
		target = private_input / "rdkit-install" / "lib" / target_name
		target.write_bytes(target_name.encode("ascii"))
		(private_input / "rdkit-install" / "lib" / alias_name).symlink_to(target_name)
	profile = RdkitCapabilityProfile(
		name="ferrum-native-input-fixture",
		rdkit=_fixture_source(private_input, "rdkit", "rdkit.tar.gz"),
		dependencies=(
			_fixture_source(private_input, "boost-headers", "boost.tar.gz", "1.91.0"),
		),
		cmake_options=(),
		forbidden_wheel_fragments=(),
		forbidden_native_fragments=(),
	)
	return profile


#============================================
def _run_manifest_rejection_fixtures(
	api: types.ModuleType, private_input: Path, temporary: str
) -> None:
	"""Verify completed input manifests seal all adapter input evidence."""
	api.publish_native_input_manifest(private_input)
	private_layout = api.rdkit_layout_from_output_root(private_input)
	if private_layout.graphmol_library.name != "libRDKitGraphMol.1.dylib":
		raise api.NativeBuildError("adapter input self-test did not retain GraphMol install name")
	if private_layout.graphmol_library.resolve().name != "libRDKitGraphMol.2026.03.4.dylib":
		raise api.NativeBuildError("adapter input self-test did not validate GraphMol target")
	expected_include_dir = private_input / "rdkit-install" / "include" / "rdkit"
	if private_layout.include_dir != expected_include_dir.resolve():
		raise api.NativeBuildError(
			"adapter input self-test did not retain the installed RDKit include root"
		)
	if private_layout.boost_include_dir.name != "boost_1_91_0":
		raise api.NativeBuildError("adapter input self-test did not retain pinned Boost headers")
	manifest_path = private_input / "ferrum-native-inputs.json"
	manifest_record = json.loads(manifest_path.read_text(encoding="utf-8"))
	if manifest_record["schema"] != "ferrum-native-inputs-v3":
		raise api.NativeBuildError("manifest self-test did not publish the sealed schema")
	if len(manifest_record["policy_sha256"]) != 64:
		raise api.NativeBuildError("manifest self-test did not fingerprint its full policy")
	library_records = manifest_record["artifacts"]["libraries"]
	graphmol_record = next(
		(
			record
			for record in library_records
			if record["alias_path"] == "rdkit-install/lib/libRDKitGraphMol.1.dylib"
		),
		None,
	)
	if graphmol_record is None:
		raise api.NativeBuildError("manifest self-test did not retain GraphMol alias path")
	if graphmol_record["resolved_target_path"] != (
		"rdkit-install/lib/libRDKitGraphMol.2026.03.4.dylib"
	):
		raise api.NativeBuildError("manifest self-test did not record GraphMol resolved target")
	manifest_path.unlink()
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"missing completed native input manifest",
	)
	manifest_path.write_text(json.dumps(manifest_record), encoding="utf-8")
	molops = private_input / "rdkit-install" / "include" / "rdkit" / "GraphMol" / "MolOps.h"
	molops.write_bytes(b"tampered")
	_reject(api, lambda: api.rdkit_layout_from_output_root(private_input), "tampered RDKit header")
	molops.write_bytes(b"")
	transitive_rdkit = (
		private_input / "rdkit-install" / "include" / "rdkit" / "GraphMol" / "Transitive.h"
	)
	transitive_rdkit.write_bytes(b"tampered transitive RDKit header")
	manifest_path.write_text(json.dumps(manifest_record), encoding="utf-8")
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"tampered transitive RDKit header",
	)
	transitive_rdkit.write_bytes(b"transitive RDKit header")
	transitive_boost = (
		private_input
		/ "dependencies"
		/ "boost-headers"
		/ "boost_1_91_0"
		/ "boost"
		/ "version.hpp"
	)
	transitive_boost.write_bytes(b"tampered transitive Boost header")
	manifest_path.write_text(json.dumps(manifest_record), encoding="utf-8")
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"tampered transitive Boost header",
	)
	transitive_boost.write_bytes(b"transitive Boost header")
	manifest_path.write_text(json.dumps(manifest_record), encoding="utf-8")
	unexpected_link = (
		private_input / "rdkit-install" / "include" / "rdkit" / "GraphMol" / "Linked.h"
	)
	unexpected_link.symlink_to("Transitive.h")
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"symlink inside pinned RDKit headers",
	)
	unexpected_link.unlink()
	policy_tamper = json.loads(json.dumps(manifest_record))
	policy_tamper["policy"]["profile"]["cmake_options"] = ["-DUNSAFE=ON"]
	manifest_path.write_text(json.dumps(policy_tamper), encoding="utf-8")
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"policy-only native input profile drift",
	)
	path_escape = json.loads(json.dumps(manifest_record))
	path_escape["paths"]["include_dir"] = "../outside"
	manifest_path.write_text(json.dumps(path_escape), encoding="utf-8")
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"native input manifest path escape",
	)
	other_repos = json.loads(json.dumps(manifest_record))
	other_repos["paths"]["include_dir"] = "OTHER_REPOS/rdkit/include"
	manifest_path.write_text(json.dumps(other_repos), encoding="utf-8")
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"native input manifest OTHER_REPOS reference",
	)
	manifest_path.write_text(json.dumps(manifest_record), encoding="utf-8")
	graphmol_alias = private_input / "rdkit-install" / "lib" / "libRDKitGraphMol.1.dylib"
	graphmol_alias.unlink()
	graphmol_alias.symlink_to(Path(temporary) / "outside-graphmol.dylib")
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"RDKit library alias outside the output root",
	)
	graphmol_alias.unlink()
	graphmol_alias.symlink_to("missing-graphmol.dylib")
	_reject(
		api,
		lambda: api.rdkit_layout_from_output_root(private_input),
		"dangling RDKit library alias",
	)


#============================================
def _run_native_input_manifest_fixtures(api: types.ModuleType, root: Path, temporary: str) -> None:
	"""Run manifest success and rejection fixtures using one private build tree."""
	private_input = root / "private-input"
	fixture_profile = _write_private_native_inputs(private_input)
	original_profile = api.FERRUM_RDKIT_PROFILE
	api.FERRUM_RDKIT_PROFILE = fixture_profile
	try:
		_run_manifest_rejection_fixtures(api, private_input, temporary)
	finally:
		api.FERRUM_RDKIT_PROFILE = original_profile


#============================================
def _run_source_and_redirect_fixtures(api: types.ModuleType, root: Path, temporary: str) -> None:
	"""Verify source provenance, archive hashes, and HTTPS redirect enforcement."""
	inside = root / "source"
	inside.mkdir()
	if api.validate_materialized_source(inside, root, "self-test source") != inside.resolve():
		raise api.NativeBuildError("source materialization self-test rejected an output-root input")
	_reject(
		api,
		lambda: api.validate_materialized_source(
			api.REPO_ROOT / "OTHER_REPOS", root, "reference source"
		),
		"OTHER_REPOS source",
	)
	_reject(
		api,
		lambda: api.validate_materialized_source(Path(temporary), root, "outside source"),
		"outside source",
	)
	escaped_source = PinnedSource(
		"escape", "1", "https://example.invalid", "0" * 64, "../../escape.zip"
	)
	_reject(
		api,
		lambda: api.materialized_archive_path(root, escaped_source),
		"archive filename escaping the output root",
	)
	_reject(
		api,
		lambda: api.download_verified_archive(
			root / "insecure.tar.gz",
			"http://example.invalid/source.tar.gz",
			"0" * 64,
			"insecure self-test",
		),
		"non-HTTPS source URL",
	)
	redirect_handler = api.HttpsOnlyRedirectHandler()
	redirect_request = urllib.request.Request("https://example.invalid/start")
	_reject(
		api,
		lambda: redirect_handler.redirect_request(
			redirect_request,
			None,
			302,
			"Found",
			None,
			"http://example.invalid/insecure",
		),
		"HTTPS redirect downgrade",
	)
	first_redirect = redirect_handler.redirect_request(
		redirect_request,
		None,
		302,
		"Found",
		None,
		"https://download.example.invalid/one",
	)
	if first_redirect is None:
		raise api.NativeBuildError("HTTPS redirect self-test did not return a request")
	second_redirect = redirect_handler.redirect_request(
		first_redirect,
		None,
		307,
		"Temporary Redirect",
		None,
		"https://objects.example.invalid/two",
	)
	if second_redirect is None or second_redirect.full_url != "https://objects.example.invalid/two":
		raise api.NativeBuildError("all-HTTPS redirect-chain self-test changed its target")
	archive = root / "source.tar.gz"
	archive.write_bytes(b"ferrum")
	api.verified_archive(archive, sha256(archive), "self-test")
	archive.write_bytes(b"changed")
	_reject(
		api,
		lambda: api.verified_archive(
			archive, hashlib.sha256(b"ferrum").hexdigest(), "self-test"
		),
		"changed archive",
	)


#============================================
def _run_archive_fixtures(api: types.ModuleType, root: Path) -> None:
	"""Verify native-library identity and safe archive extraction behavior."""
	library = root / "libRDKitRDGeneral.2026.03.4.dylib"
	library.write_bytes(b"native")
	alias = root / "libRDKitRDGeneral.1.dylib"
	alias.symlink_to(library.name)
	if deduplicate_paths_by_identity([library, alias], [alias]) != [alias]:
		raise api.NativeBuildError(
			"native closure identity de-duplication lost the selected install-name alias"
		)
	valid_zip = root / "valid.zip"
	valid_member = zipfile.ZipInfo("source/file.txt")
	valid_member.create_system = 3
	valid_member.external_attr = (stat.S_IFREG | 0o4755) << 16
	with zipfile.ZipFile(valid_zip, "w") as contents:
		contents.writestr(valid_member, "ferrum")
	valid_zip_destination = root / "valid-zip"
	valid_zip_destination.mkdir()
	extracted_zip = api.safe_extract_zip(valid_zip, valid_zip_destination)
	extracted_file = extracted_zip / "file.txt"
	if extracted_file.read_text(encoding="utf-8") != "ferrum":
		raise api.NativeBuildError("safe ZIP extraction self-test lost regular-file content")
	if extracted_file.stat().st_mode & (stat.S_ISUID | stat.S_ISGID | stat.S_ISVTX):
		raise api.NativeBuildError("safe ZIP extraction retained privileged mode bits")
	unsafe_zip = root / "unsafe.zip"
	with zipfile.ZipFile(unsafe_zip, "w") as contents:
		contents.writestr("../escape.txt", "escape")
	unsafe_zip_destination = root / "unsafe-zip"
	unsafe_zip_destination.mkdir()
	_reject(
		api,
		lambda: api.safe_extract_zip(unsafe_zip, unsafe_zip_destination),
		"ZIP path traversal",
	)
	link_zip = root / "link.zip"
	link_member = zipfile.ZipInfo("source/link")
	link_member.create_system = 3
	link_member.external_attr = (stat.S_IFLNK | 0o777) << 16
	with zipfile.ZipFile(link_zip, "w") as contents:
		contents.writestr(link_member, "../../escape")
	link_zip_destination = root / "link-zip"
	link_zip_destination.mkdir()
	_reject(api, lambda: api.safe_extract_zip(link_zip, link_zip_destination), "ZIP symbolic link")
	duplicate_tar = root / "duplicate.tar.gz"
	with tarfile.open(duplicate_tar, "w:gz") as contents:
		for payload in (b"first", b"second"):
			member = tarfile.TarInfo("source/file.txt")
			member.size = len(payload)
			contents.addfile(member, io.BytesIO(payload))
	duplicate_tar_destination = root / "duplicate-tar"
	duplicate_tar_destination.mkdir()
	_reject(
		api,
		lambda: api.safe_extract(duplicate_tar, duplicate_tar_destination),
		"duplicate TAR target",
	)


#============================================
def _run_wheel_member_fixtures(api: types.ModuleType) -> None:
	"""Verify wheel-member validation accepts only Ferrum's native closure."""
	valid_wheel = [
		"ferrum_chem.cpython-312-darwin.so",
		"ferrum_chem.pyi",
		"py.typed",
		"ferrum-operation-v1.schema.json",
		*(
			f".dylibs/{name}"
			for name in sorted(MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names)
		),
	]
	api.validate_wheel_members(valid_wheel)
	_reject(
		api,
		lambda: api.validate_wheel_members(
			[member for member in valid_wheel if member != "ferrum-operation-v1.schema.json"]
		),
		"missing operation protocol schema",
	)
	_reject(
		api,
		lambda: api.validate_wheel_members([*valid_wheel, "ferrum_chem/__init__.py"]),
		"nested Ferrum package shim",
	)
	_reject(
		api,
		lambda: api.validate_wheel_members([*valid_wheel, "unexpected.txt"]),
		"unexpected wheel payload",
	)
	_reject(
		api,
		lambda: api.validate_wheel_members([*valid_wheel, "rdkit/Chem/__init__.py"]),
		"RDKit Python wheel content",
	)
	_reject(
		api,
		lambda: api.validate_wheel_members([*valid_wheel, ".dylibs/libboost_python312.dylib"]),
		"Boost.Python wheel content",
	)
	_reject(
		api,
		lambda: api.validate_wheel_members([*valid_wheel, ".dylibs/libunexpected.dylib"]),
		"extra native closure member",
	)
	_reject(
		api,
		lambda: api.validate_wheel_members([*valid_wheel, "ferrum_chem_helper.so"]),
		"unexpected native extension",
	)
	_reject(
		api,
		lambda: api.validate_wheel_members([*valid_wheel, ".dylibs/libRDKitRDGeneral.1.dylib/extra"]),
		"nested allowed-library prefix",
	)


#============================================
def _run_notice_injection_fixture(api: types.ModuleType) -> None:
	"""Verify the wheel rewrite links each staged notice role through metadata."""
	with tempfile.TemporaryDirectory() as temporary:
		root = Path(temporary)
		project = root / "crates" / "api" / "python"
		project.mkdir(parents=True)
		metadata = project.parent / "wheel_metadata"
		notices = metadata / "licenses"
		notices.mkdir(parents=True)
		(metadata / "ferrum_chem.pyi").write_text("", encoding="utf-8")
		(metadata / "py.typed").write_text("", encoding="utf-8")
		(project.parent / "protocol").mkdir()
		(project.parent / "protocol" / "ferrum-operation-v1.schema.json").write_text(
			"{}\n", encoding="utf-8"
		)
		for name in NOTICE_FILENAMES:
			(notices / name).write_text(f"fixture {name}\n", encoding="utf-8")
		wheel = root / "ferrum_chem-0-py3-none-any.whl"
		with zipfile.ZipFile(wheel, "w") as archive:
			archive.writestr("ferrum_chem/ferrum_chem.cpython-312-darwin.so", b"extension")
			archive.writestr("ferrum_chem-0.dist-info/WHEEL", "Wheel-Version: 1.0\n\n")
			# Maturin may emit header-only core metadata with no description body.
			archive.writestr("ferrum_chem-0.dist-info/METADATA", "Name: ferrum-chem\n")
		inject_root_metadata(wheel, project)
		with zipfile.ZipFile(wheel) as archive:
			members = set(archive.namelist())
			prefix = "ferrum_chem-0.dist-info/licenses/"
			for name in NOTICE_FILENAMES:
				member = prefix + name
				if member not in members:
					raise api.NativeBuildError(f"notice fixture omitted required role: {name}")
			metadata_text = archive.read("ferrum_chem-0.dist-info/METADATA").decode("utf-8")
			for name in NOTICE_FILENAMES:
				if f"License-File: licenses/{name}" not in metadata_text:
					raise api.NativeBuildError(f"notice fixture omitted metadata link: {name}")
			if "ferrum_chem-0.dist-info/RECORD" not in members:
				raise api.NativeBuildError("notice fixture did not regenerate RECORD")
		(notices / NOTICE_FILENAMES[0]).unlink()
		try:
			inject_root_metadata(wheel, project)
		except NativePackagingError:
			pass
		else:
			raise api.NativeBuildError("notice fixture accepted a missing required notice")


#============================================
def _run_smiles_depict_stage_fixture(api: types.ModuleType, root: Path) -> None:
	"""Verify closure staging preserves headers and every declared dylib alias."""
	source = root / "source"
	(source / "Code" / "GraphMol").mkdir(parents=True)
	(source / "Code" / "RDGeneral").mkdir()
	(source / "External" / "INCHI-API").mkdir(parents=True)
	ring_source = (
		source / "External" / "RingFamilies" / "RingDecomposerLib" / "src"
		/ "RingDecomposerLib"
	)
	ring_source.mkdir(parents=True)
	(source / "Code" / "GraphMol" / "MolOps.h").write_text("source", encoding="utf-8")
	(source / "Code" / "RDGeneral" / "types.h").write_text("source", encoding="utf-8")
	(source / "External" / "INCHI-API" / "inchi.h").write_text("inchi", encoding="utf-8")
	(ring_source / "RingDecomposerLib.h").write_text("ring", encoding="utf-8")
	build = root / "rdkit-build"
	(build / "Code" / "RDGeneral").mkdir(parents=True)
	(build / "Code" / "RDGeneral" / "RDKitBuildInfo.h").write_text(
		"generated", encoding="utf-8"
	)
	lib_dir = build / "lib"
	lib_dir.mkdir()
	for name in api.RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES:
		(lib_dir / name).write_bytes(name.encode("ascii"))
	stage = api.stage_rdkit_inputs(root, source, build)
	graphmol_header = stage / "include" / "rdkit" / "GraphMol" / "MolOps.h"
	if graphmol_header.read_text(encoding="utf-8") != "source":
		raise api.NativeBuildError("GraphMol stage fixture lost source header")
	build_info = stage / "include" / "rdkit" / "RDGeneral" / "RDKitBuildInfo.h"
	if build_info.read_text(encoding="utf-8") != "generated":
		raise api.NativeBuildError("GraphMol stage fixture lost generated header")
	if (stage / "include" / "rdkit" / "RingDecomposerLib.h").read_text(
		encoding="utf-8"
	) != "ring":
		raise api.NativeBuildError("GraphMol stage fixture lost ring-decomposer header")
	if {path.name for path in (stage / "lib").iterdir()} != set(
		api.RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES
	):
		raise api.NativeBuildError("GraphMol stage fixture retained an unexpected native library")
	with tempfile.TemporaryDirectory() as temporary:
		conflicting_root = Path(temporary) / "output-conflicting-stage"
		conflicting_root.mkdir()
		conflicting_source = conflicting_root / "source"
		(conflicting_source / "Code" / "GraphMol").mkdir(parents=True)
		(conflicting_source / "Code" / "GraphMol" / "Duplicate.h").touch()
		conflicting_build = conflicting_root / "rdkit-build"
		(conflicting_build / "Code" / "GraphMol").mkdir(parents=True)
		(conflicting_build / "Code" / "GraphMol" / "Duplicate.h").touch()
		conflicting_libraries = conflicting_build / "lib"
		conflicting_libraries.mkdir()
		for name in api.RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES:
			(conflicting_libraries / name).touch()
		_reject(
			api,
			lambda: api.stage_rdkit_inputs(
				conflicting_root, conflicting_source, conflicting_build
			),
			"source/generated RDKit header collision",
		)
	_reject(
		api,
		lambda: api.stage_rdkit_inputs(root, source, build),
		"existing GraphMol stage",
	)


#============================================
def run(api: types.ModuleType) -> None:
	"""Exercise every pure builder-policy fixture through its supplied API.

	Args:
		api: The imported builder module that owns the production helpers.
	"""
	_run_policy_fixtures(api)
	_run_engine_bundle_fixtures(api)
	_run_tree_fixtures(api)
	_run_profile_configuration_fixtures(api)
	with tempfile.TemporaryDirectory() as temporary:
		root = Path(temporary) / "output-native"
		root.mkdir()
		_run_native_input_manifest_fixtures(api, root, temporary)
		_run_source_and_redirect_fixtures(api, root, temporary)
		_run_archive_fixtures(api, root)
		stage_root = Path(temporary) / "output-stage"
		stage_root.mkdir()
		_run_smiles_depict_stage_fixture(api, stage_root)
	_run_wheel_member_fixtures(api)
	_run_notice_injection_fixture(api)
