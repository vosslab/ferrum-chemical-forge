"""Build Ferrum's source-verified, C++-only RDKit native-wheel packaging proof.

The immutable capability profile below is the source of truth for what Ferrum
asks RDKit to build.  It deliberately does not package RDKit Python, SWIG, or
compiled Boost libraries.  Generated state belongs below one ignored output
root; ``OTHER_REPOS`` is never a source, build, test, or runtime input.
"""

from __future__ import annotations

# Standard library imports.
import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import time
import urllib.request
import urllib.parse
import uuid
import zipfile
from dataclasses import dataclass
from pathlib import Path
import native_wheel_builder_self_test
import native_wheel_bundle
import native_wheel_builder_cli
from native_wheel_output_root import engine_bundle_path as admitted_engine_bundle_path
from native_wheel_output_root import output_path as admitted_output_path
from native_wheel_packaging import (
	NativePackagingError,
	find_maturin,
	inject_root_metadata,
	stage_native_notice_bundle,
	stage_python_project,
	tool_version,
	validate_wheel_members as validate_packaged_wheel_members,
)
from native_wheel_adapter_abi import adapter_abi_version_from_header
from native_wheel_download import (
	ArchiveExtractionError,
	HttpsOnlyRedirectHandler,
	safe_extract as extract_tar,
	safe_extract_zip as extract_zip,
	safe_extract_zip_members as extract_zip_members,
	validated_https_url,
)
from native_wheel_macho import (
	NativeMachoError,
	assert_clean_closure,
	assert_packaged_library_closure,
	copy_and_rewrite_closure,
	detect_variants,
	otool_dependencies,
)
from native_wheel_policy import (
	NativePolicyError,
	apple_sdk,
	audit_cmake_provenance,
	cmake_cxx_toolchain_options,
	cmake_toolchain_options,
	homebrew_cmake,
	homebrew_llvm,
	native_tool_environment,
	rust_tool_environment,
	rust_toolchain_receipt,
	toolchain_receipt,
)
from native_wheel_profile import (
	BOOST_VERSION,
	FERRUM_RDKIT_PROFILE,
	MACOS_ARM64_NATIVE_CLOSURE,
MACHINE_RESULT_SCHEMA,
	PinnedSource,
	RDKIT_SHA256,
	RDKIT_TAG,
	RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES,
	RDKIT_URL,
	RdkitCapabilityProfile,
	TARGET,
	minimal_rdkit_options as profile_rdkit_options,
	validate_rdkit_configuration as validate_profile_configuration,
	validate_resolved_rdkit_configuration as validate_profile_cache,
)
from native_wheel_receipt import (
	NativeReceiptError,
	sha256,
	source_archive_path,
	validate_native_input_manifest,
	write_native_input_manifest,
	write_build_receipt,
)

engine_bundle_manifest = native_wheel_bundle.engine_bundle_manifest
executable_bundle_target = native_wheel_bundle.executable_bundle_target
validate_engine_bundle = native_wheel_bundle.validate_engine_bundle
REPO_ROOT = Path(__file__).resolve().parents[3]
RUST_PACKAGE_SOURCE = Path(__file__).resolve().parents[1]
NATIVE_SOURCE = REPO_ROOT / "packages/ferrum-rust/crates/chemistry/native"
DOWNLOAD_ATTEMPTS = 3
ADAPTER_BUILD_TYPES = ("Release", "RelWithDebInfo")
ADAPTER_HEADER = NATIVE_SOURCE / "include/ferrum_chem_adapter.h"
BUNDLE_MANIFEST_NAME = "ferrum-engine-bundle-v1.json"
BUNDLE_SCHEMA = "ferrum-engine-bundle-v1"
ADAPTER_NAME = "libferrum_chem.dylib"
FERRUM_SOURCE_CLOSURE_SCHEMA = "ferrum-wheel-source-closure-v2"
# These are the only builder-owned paths created inside maturin-project after
# stage_python_project() has copied and rewritten the Ferrum source workspace.
# Every other regular staged file remains an admitted authored source input.
FERRUM_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES = (
	"crates/api/wheel_metadata/licenses",
	"crates/api/python/.dylibs",
	"crates/api/python/ferrum_chem/.dylibs",
)

class NativeBuildError(RuntimeError):
	"""An actionable failure in the build or closure contract."""

try:
	ADAPTER_ABI_VERSION = adapter_abi_version_from_header(ADAPTER_HEADER)
except RuntimeError as error:
	raise NativeBuildError(str(error)) from error


@dataclass(frozen=True)
class RdkitLayout:
	input_root: Path
	lib_dir: Path
	include_dir: Path
	boost_include_dir: Path
	graphmol_library: Path
	rdgeneral_library: Path
	depictor_library: Path
	smilesparse_library: Path
	fileparsers_library: Path
	rdinchi_library: Path
	substructmatch_library: Path
	cmake_options: tuple[str, ...]
	toolchain: dict[str, str]
	provenance_audit: dict[str, object]


def run(*command: str, cwd: Path | None = None, env: dict[str, str] | None = None) -> None:
	print("+", " ".join(command), file=sys.stderr)
	try:
		# The command protocol reserves stdout for one final machine result. Route
		# compiler and packager progress to stderr so callers never need to guess
		# whether a path is mixed with a build log.
		subprocess.run(command, cwd=cwd, env=env, stdout=sys.stderr, check=True)
	except FileNotFoundError as error:
		raise NativeBuildError(f"required program is unavailable: {command[0]}") from error
	except subprocess.CalledProcessError as error:
		raise NativeBuildError(f"command failed ({error.returncode}): {' '.join(command)}") from error

def output_path(value: str) -> Path:
	"""Return one parser-admitted builder output root."""
	return admitted_output_path(value, REPO_ROOT)


#============================================
def engine_bundle_path(value: str) -> Path:
	"""Return one resolved child destination for the admitted output root."""
	return admitted_engine_bundle_path(value)


#============================================
def archive_root_path(value: str) -> Path:
	"""Accept one read-only directory of exact hash-pinned source archives."""
	path = Path(value).expanduser().resolve()
	if path.is_relative_to(REPO_ROOT / "OTHER_REPOS") or not path.is_dir():
		raise argparse.ArgumentTypeError("--source-archive-root must be a directory outside OTHER_REPOS")
	return path

#============================================
def validate_materialized_source(path: Path, output_root: Path, label: str) -> Path:
	"""Reject an accidental reference-tree or host path before CMake sees it."""
	resolved = path.resolve()
	root = output_root.resolve()
	if resolved.is_relative_to(REPO_ROOT / "OTHER_REPOS"):
		raise NativeBuildError(f"{label} must not resolve inside OTHER_REPOS: {resolved}")
	if not resolved.is_relative_to(root):
		raise NativeBuildError(f"{label} must be materialized below output root {root}: {resolved}")
	return resolved

#============================================
def validate_materialized_alias(path: Path, output_root: Path, label: str) -> Path:
	"""Validate a private symlink target while retaining its declared alias path."""
	root = output_root.resolve()
	lexical = Path(os.path.abspath(path))
	if not lexical.is_relative_to(root):
		raise NativeBuildError(f"{label} must be materialized below output root {root}: {lexical}")
	resolved = lexical.resolve()
	if resolved.is_relative_to(REPO_ROOT / "OTHER_REPOS"):
		raise NativeBuildError(f"{label} must not resolve inside OTHER_REPOS: {resolved}")
	if not resolved.is_relative_to(root):
		raise NativeBuildError(f"{label} must resolve below output root {root}: {resolved}")
	return lexical

#============================================
def ferrum_source_closure(root: Path) -> dict[str, object]:
	"""Return the canonical staged Ferrum source-subset manifest."""
	root = root.resolve()
	if not root.is_dir():
		raise NativeBuildError(f"Ferrum source closure root is not a directory: {root}")
	excluded_directories = frozenset(FERRUM_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES)
	files: list[dict[str, str]] = []
	for directory, names, filenames in os.walk(root):
		directory_path = Path(directory)
		accepted_names: list[str] = []
		for name in sorted(names):
			path = directory_path / name
			relative = path.relative_to(root).as_posix()
			if path.is_symlink():
				raise NativeBuildError(f"Ferrum source closure requires real directories: {path}")
			if relative in excluded_directories:
				continue
			accepted_names.append(name)
		names[:] = accepted_names
		for filename in sorted(filenames):
			path = directory_path / filename
			if not path.is_file() or path.is_symlink():
				raise NativeBuildError(f"Ferrum source closure requires regular files: {path}")
			files.append({
				"path": path.relative_to(root).as_posix(),
				"sha256": sha256(path),
			})
	payload = json.dumps({
		"excluded_directories": list(FERRUM_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES),
		"files": files,
		"schema": FERRUM_SOURCE_CLOSURE_SCHEMA,
	}, separators=(",", ":"), sort_keys=True).encode("utf-8")
	return {
		"excluded_directories": list(FERRUM_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES),
		"files": files,
		"fingerprint_sha256": hashlib.sha256(payload).hexdigest(),
		"schema": FERRUM_SOURCE_CLOSURE_SCHEMA,
	}

#============================================
def stage_ferrum_python_project(output_root: Path) -> Path:
	"""Stage Ferrum's deterministic source workspace for Maturin."""
	return stage_python_project(output_root, RUST_PACKAGE_SOURCE)

#============================================
def validate_build_receipt(receipt: Path, wheel: Path, source_closure: dict[str, object]) -> None:
	"""Refuse a publication candidate unless its durable source and wheel evidence agree."""
	try:
		value = json.loads(receipt.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise NativeBuildError(f"native build receipt is invalid JSON: {error.msg}") from error
	if not isinstance(value, dict) or value.get("ferrum_source_closure") != source_closure:
		raise NativeBuildError("native build receipt lacks the admitted Ferrum source closure")
	wheel_record = value.get("wheel")
	if not isinstance(wheel_record, dict) or wheel_record != {
		"filename": wheel.name, "sha256": sha256(wheel),
	}:
		raise NativeBuildError("native build receipt does not match the final wheel")

#============================================
def validate_publication_candidate(receipt: Path, wheel: Path, staged_source_root: Path) -> None:
	"""Revalidate copied publication evidence against the completed staged workspace."""
	if receipt.is_symlink() or not receipt.is_file():
		raise NativeBuildError(f"native publication receipt is not a regular file: {receipt}")
	if wheel.is_symlink() or not wheel.is_file():
		raise NativeBuildError(f"native publication wheel is not a regular file: {wheel}")
	source_closure = ferrum_source_closure(staged_source_root)
	validate_build_receipt(receipt, wheel, source_closure)


#============================================
def validate_publication_engine_bundle(bundle: Path) -> None:
	"""Revalidate one copied CLI engine bundle against its sealed manifest."""
	try:
		validate_engine_bundle(
			bundle, BUNDLE_MANIFEST_NAME, BUNDLE_SCHEMA, executable_bundle_target(),
			ADAPTER_ABI_VERSION, ADAPTER_NAME, sha256,
		)
	except native_wheel_bundle.NativeEngineBundleError as error:
		raise NativeBuildError(str(error)) from error

#============================================
def required_rdkit_library(lib_dir: Path, name: str, input_root: Path) -> Path:
	"""Return one exact RDKit install-name alias from a declared private install."""
	library = validate_materialized_alias(lib_dir / name, input_root, name)
	if not library.is_file():
		raise NativeBuildError(f"RDKit installation lacks required library: {library}")
	return library

#============================================
def pinned_boost_headers(input_root: Path) -> Path:
	"""Locate one pinned header tree materialized with this RDKit build."""
	dependency_root = validate_materialized_source(
		input_root / "dependencies" / "boost-headers", input_root, "Boost dependency root"
	)
	if not dependency_root.is_dir():
		raise NativeBuildError(f"RDKit build lacks pinned Boost dependency root: {dependency_root}")
	candidates = [
		path for path in dependency_root.iterdir()
		if path.is_dir() and (path / "boost" / "config.hpp").is_file()
	]
	if len(candidates) != 1:
		raise NativeBuildError(
		"RDKit build must retain exactly one pinned Boost header root below "
		f"{dependency_root}, found {candidates}"
		)
	return validate_materialized_source(candidates[0], input_root, "Boost header root")

#============================================
def rdkit_layout_from_output_root(input_root: Path) -> RdkitLayout:
	"""Derive the adapter's complete native inputs from one prior private build."""
	root = input_root.resolve()
	if root.is_relative_to(REPO_ROOT / "OTHER_REPOS"):
		raise NativeBuildError(f"RDKit native input root must not be inside OTHER_REPOS: {root}")
	if not root.is_dir():
		raise NativeBuildError(f"RDKit native input root does not exist: {root}")
	try:
		manifest = validate_native_input_manifest(root, FERRUM_RDKIT_PROFILE)
	except NativeReceiptError as error:
		raise NativeBuildError(str(error)) from error
	paths = manifest["paths"]
	if not isinstance(paths, dict):
		raise NativeBuildError("validated native input manifest lacks path data")
	include_dir = validate_materialized_source(
		root / str(paths["include_dir"]), root, "RDKit include"
	)
	lib_dir = validate_materialized_source(
		root / str(paths["rdkit_library_dir"]), root, "RDKit library"
	)
	graphmol_library = required_rdkit_library(lib_dir, "libRDKitGraphMol.1.dylib", root)
	rdgeneral_library = required_rdkit_library(lib_dir, "libRDKitRDGeneral.1.dylib", root)
	depictor_library = required_rdkit_library(lib_dir, "libRDKitDepictor.1.dylib", root)
	smilesparse_library = required_rdkit_library(lib_dir, "libRDKitSmilesParse.1.dylib", root)
	fileparsers_library = required_rdkit_library(lib_dir, "libRDKitFileParsers.1.dylib", root)
	rdinchi_library = required_rdkit_library(lib_dir, "libRDKitRDInchiLib.1.dylib", root)
	substructmatch_library = required_rdkit_library(
		lib_dir, "libRDKitSubstructMatch.1.dylib", root
	)
	boost_include_dir = validate_materialized_source(
		root / str(paths["boost_include_dir"]), root, "Boost include"
	)
	if not (include_dir / "GraphMol" / "MolOps.h").is_file():
		raise NativeBuildError(f"RDKit installation lacks GraphMol headers: {include_dir}")
	if not (include_dir / "RDGeneral" / "types.h").is_file():
		raise NativeBuildError(f"RDKit installation lacks RDGeneral headers: {include_dir}")
	if not (include_dir / "GraphMol" / "inchi.h").is_file():
		raise NativeBuildError(f"RDKit installation lacks its pinned InChI wrapper header: {include_dir}")
	if not lib_dir.is_dir():
		raise NativeBuildError(f"RDKit installation lacks library directory: {lib_dir}")
	return RdkitLayout(
		input_root=root,
		lib_dir=lib_dir,
		include_dir=include_dir,
		boost_include_dir=boost_include_dir,
		graphmol_library=graphmol_library,
		rdgeneral_library=rdgeneral_library,
		depictor_library=depictor_library,
		smilesparse_library=smilesparse_library,
		fileparsers_library=fileparsers_library,
		rdinchi_library=rdinchi_library,
		substructmatch_library=substructmatch_library,
		cmake_options=(),
		toolchain={},
		provenance_audit={},
	)


#============================================
def reuse_sealed_native_inputs(output_root: Path, sealed_root: Path) -> RdkitLayout:
	"""Copy one manifest-validated native input set into a fresh output root."""
	sealed_root = sealed_root.resolve()
	if sealed_root.is_relative_to(REPO_ROOT / "OTHER_REPOS"):
		raise NativeBuildError(f"sealed native input root must not be inside OTHER_REPOS: {sealed_root}")
	if not sealed_root.is_dir():
		raise NativeBuildError(f"sealed native input root does not exist: {sealed_root}")
	try:
		validate_native_input_manifest(sealed_root, FERRUM_RDKIT_PROFILE)
	except NativeReceiptError as error:
		raise NativeBuildError(f"sealed native input root is not reusable: {error}") from error
	for relative in ("downloads", "rdkit-install", "dependencies/boost-headers"):
		source = sealed_root / relative
		destination = output_root / relative
		if not source.is_dir() or destination.exists():
			raise NativeBuildError(f"cannot materialize sealed native input: {relative}")
		shutil.copytree(source, destination, symlinks=False)
	manifest = sealed_root / "ferrum-native-inputs.json"
	if not manifest.is_file():
		raise NativeBuildError("sealed native input root has no manifest")
	shutil.copy2(manifest, output_root / manifest.name)
	return rdkit_layout_from_output_root(output_root)


#============================================
def publish_native_input_manifest(output_root: Path) -> None:
	"""Record then immediately verify every private RDKit adapter input."""
	include_dir = validate_materialized_source(
		output_root / "rdkit-install" / "include" / "rdkit", output_root, "RDKit include root"
	)
	lib_dir = validate_materialized_source(
		output_root / "rdkit-install" / "lib", output_root, "RDKit library"
	)
	try:
		write_native_input_manifest(
			output_root,
			FERRUM_RDKIT_PROFILE,
			include_dir,
			pinned_boost_headers(output_root),
			lib_dir,
		)
	except NativeReceiptError as error:
		raise NativeBuildError(str(error)) from error

#============================================
def verified_archive(path: Path, expected_sha256: str = RDKIT_SHA256, label: str = "RDKit") -> Path:
	"""Return one regular archive after its exact SHA-256 digest matches.

	Args:
		path: The archive file to verify.
		expected_sha256: The immutable digest required for this source.
		label: The human-readable source name used in failures.

	Returns:
		The verified archive path.
	"""
	if not path.is_file():
		raise NativeBuildError(f"RDKit archive does not exist: {path}")
	actual = sha256(path)
	if actual != expected_sha256:
		raise NativeBuildError(
			f"{label} archive SHA-256 mismatch for {path}: expected {expected_sha256}, got {actual}"
		)
	return path

#============================================
def materialized_archive_path(output_root: Path, source: PinnedSource) -> Path:
	"""Map source-record policy failures into the build tool's error contract."""
	try:
		return source_archive_path(output_root, source)
	except NativeReceiptError as error:
		raise NativeBuildError(str(error)) from error

#============================================
def download_verified_archive(destination: Path, url: str, digest: str, label: str) -> Path:
	"""Publish a verified archive atomically, never leaving a partial cache entry."""
	validated_https_url(url, f"{label} source")
	if destination.is_symlink():
		raise NativeBuildError(f"{label} archive must not be a symbolic link: {destination}")
	destination.parent.mkdir(parents=True, exist_ok=True)
	if destination.exists():
		return verified_archive(destination, digest, label)
	for attempt in range(1, DOWNLOAD_ATTEMPTS + 1):
		temporary = destination.with_name(f".{destination.name}.{uuid.uuid4().hex}.download")
		try:
			print(f"downloading pinned {label} ({attempt}/{DOWNLOAD_ATTEMPTS}) from {url}", file=sys.stderr)
			opener = urllib.request.OpenerDirector()
			opener.add_handler(urllib.request.UnknownHandler())
			opener.add_handler(urllib.request.HTTPDefaultErrorHandler())
			opener.add_handler(HttpsOnlyRedirectHandler())
			opener.add_handler(urllib.request.HTTPSHandler())
			opener.add_handler(urllib.request.HTTPErrorProcessor())
			with opener.open(url, timeout=60) as response, temporary.open("wb") as output:
				validated_https_url(response.geturl(), f"{label} final response")
				shutil.copyfileobj(response, output)
			verified_archive(temporary, digest, label)
			temporary.replace(destination)
			return destination
		except (OSError, ValueError, NativeBuildError) as error:
			temporary.unlink(missing_ok=True)
			if attempt == DOWNLOAD_ATTEMPTS:
				raise NativeBuildError(
					f"could not download verified {label} after {DOWNLOAD_ATTEMPTS} attempts: {error}"
				) from error
			time.sleep(attempt)
	raise NativeBuildError("download retry loop ended without returning or raising")

#============================================
def download_archive(output_root: Path) -> Path:
	archive = materialized_archive_path(output_root, FERRUM_RDKIT_PROFILE.rdkit)
	return download_verified_archive(archive, RDKIT_URL, RDKIT_SHA256, f"RDKit {RDKIT_TAG}")


#============================================
def managed_source_archive_cache_root() -> Path:
	"""Return the profile-scoped generated archive cache owned by this builder."""
	return REPO_ROOT / "build" / "native-source-archives" / FERRUM_RDKIT_PROFILE.name


#============================================
def provision_managed_source_archive_cache() -> Path:
	"""Return a complete verified managed archive cache, provisioning only misses."""
	cache_root = managed_source_archive_cache_root()
	if cache_root.is_symlink():
		raise NativeBuildError(f"managed native archive cache must not be a symbolic link: {cache_root}")
	if cache_root.exists() and not cache_root.is_dir():
		raise NativeBuildError(f"managed native archive cache must be a directory: {cache_root}")
	physical_repo_root = REPO_ROOT.resolve()
	physical_cache_root = cache_root.resolve()
	if not physical_cache_root.is_relative_to(physical_repo_root):
		raise NativeBuildError(
			f"managed native archive cache resolves outside the repository: {cache_root}"
		)
	if "OTHER_REPOS" in physical_cache_root.parts:
		raise NativeBuildError(
			f"managed native archive cache must not resolve into OTHER_REPOS: {cache_root}"
		)
	cache_root.mkdir(parents=True, exist_ok=True)
	for source in (FERRUM_RDKIT_PROFILE.rdkit, *FERRUM_RDKIT_PROFILE.dependencies):
		destination = cache_root / source.archive_filename
		if destination.exists() or destination.is_symlink():
			verified_archive(destination, source.sha256, source.name)
		else:
			download_verified_archive(destination, source.url, source.sha256, source.name)
	return cache_root


#============================================
def archive_root_for_build(arguments: argparse.Namespace) -> Path:
	"""Select an explicit strict archive root or the builder-owned managed cache."""
	if arguments.source_archive_root is not None:
		return arguments.source_archive_root
	return provision_managed_source_archive_cache()


#============================================
def safe_extract(archive: Path, destination: Path) -> Path:
	"""Map safe tar extraction into the builder's stable error contract."""
	try:
		result = extract_tar(archive, destination)
	except ArchiveExtractionError as error:
		raise NativeBuildError(str(error)) from error
	return result


#============================================
def safe_extract_zip(archive: Path, destination: Path) -> Path:
	"""Map safe ZIP extraction into the builder's stable error contract."""
	try:
		result = extract_zip(archive, destination)
	except ArchiveExtractionError as error:
		raise NativeBuildError(str(error)) from error
	return result


#============================================
def safe_extract_zip_members(contents: zipfile.ZipFile, destination: Path) -> None:
	"""Map ZIP-member extraction into the builder's stable error contract."""
	try:
		extract_zip_members(contents, destination)
	except ArchiveExtractionError as error:
		raise NativeBuildError(str(error)) from error


#============================================
def validate_wheel_members(members: list[str], profile: RdkitCapabilityProfile = FERRUM_RDKIT_PROFILE) -> None:
	"""Map packaged wheel-member validation into the builder error contract."""
	try:
		validate_packaged_wheel_members(members, profile)
	except NativePackagingError as error:
		raise NativeBuildError(str(error)) from error

#============================================
def materialize_source_archive(
	output_root: Path, source_input: PinnedSource, archive_root: Path | None,
) -> Path:
	"""Copy one verified offline archive or download its declared HTTPS source."""
	archive = materialized_archive_path(output_root, source_input)
	if archive_root is None:
		return download_verified_archive(archive, source_input.url, source_input.sha256, source_input.name)
	provided = archive_root / source_input.archive_filename
	if not provided.is_file():
		raise NativeBuildError(f"offline archive root lacks {source_input.archive_filename}")
	verified_archive(provided, source_input.sha256, source_input.name)
	archive.parent.mkdir(parents=True, exist_ok=True)
	if archive.exists():
		raise NativeBuildError(f"refusing to overwrite materialized archive: {archive}")
	shutil.copy2(provided, archive)
	return verified_archive(archive, source_input.sha256, source_input.name)


#============================================
def download_dependency(output_root: Path, source_input: PinnedSource, archive_root: Path | None) -> Path:
	name = source_input.name
	archive = materialize_source_archive(output_root, source_input, archive_root)
	destination = output_root / "dependencies" / name
	if destination.exists():
		raise NativeBuildError(f"refusing to overwrite existing dependency source: {destination}")
	destination.mkdir(parents=True)
	if archive.name.endswith(".tar.gz"):
		extracted = safe_extract(archive, destination)
	elif archive.suffix == ".zip":
		extracted = safe_extract_zip(archive, destination)
	else:
		raise NativeBuildError(f"unsupported pinned archive format: {archive.name}")
	return validate_materialized_source(extracted, output_root, f"{name} source")

#============================================
def prepare_source(output_root: Path, archive_root: Path | None) -> Path:
	archive = materialize_source_archive(output_root, FERRUM_RDKIT_PROFILE.rdkit, archive_root)
	validate_materialized_source(archive, output_root, "RDKit archive")
	source_parent = output_root / "source"
	source = source_parent / f"rdkit-{RDKIT_TAG}"
	if source.exists():
		raise NativeBuildError(
			f"refusing to overwrite existing source tree: {source}; choose a fresh output root"
		)
	source_parent.mkdir(parents=True, exist_ok=True)
	source = safe_extract(archive, source_parent)
	if source.name != f"rdkit-{RDKIT_TAG}" or not (source / "CMakeLists.txt").is_file():
		raise NativeBuildError(f"verified archive did not contain rdkit-{RDKIT_TAG}/CMakeLists.txt")
	return validate_materialized_source(source, output_root, "RDKit source")

#============================================
def materialize_retained_rdkit_inputs(
	output_root: Path, archive_root: Path | None,
) -> tuple[Path, Path, Path, Path]:
	"""Supply every hash-pinned, offline configure input for the native profile."""
	inputs = {item.name: item for item in FERRUM_RDKIT_PROFILE.dependencies}
	catch2 = download_dependency(output_root, inputs["catch2"], archive_root)
	better_enums = download_dependency(output_root, inputs["better-enums"], archive_root)
	boost_headers = download_dependency(output_root, inputs["boost-headers"], archive_root)
	inchi_source = download_dependency(output_root, inputs["inchi-source"], archive_root)
	return catch2, better_enums, boost_headers, inchi_source


#============================================
def install_pinned_inchi_source(rdkit_source: Path, inchi_source: Path) -> None:
	"""Install the verified InChI source where RDKit checks before any download."""
	source = validate_materialized_source(
		inchi_source, rdkit_source.parent.parent, "InChI source",
	)
	if not (source / "INCHI_BASE" / "src" / "ichican2.c").is_file():
		raise NativeBuildError(f"pinned InChI archive lacks INCHI_BASE sources: {source}")
	destination = rdkit_source / "External" / "INCHI-API" / "src"
	if destination.exists():
		raise NativeBuildError(f"refusing to overwrite RDKit InChI source: {destination}")
	shutil.copytree(source, destination, symlinks=False)


#============================================
def materialize_boost_headers_config(output_root: Path, boost_headers: Path) -> Path:
	"""Expose only pinned Boost headers and a local CMake config, never Boost dylibs."""
	validate_materialized_source(boost_headers, output_root, "Boost headers")
	include = boost_headers / "boost"
	if not include.is_dir():
		raise NativeBuildError(f"pinned Boost archive lacks headers: {include}")
	config_dir = output_root / "boost-headers-config" / "lib" / "cmake" / f"Boost-{BOOST_VERSION}"
	config_dir.mkdir(parents=True)
	config = config_dir / "BoostConfig.cmake"
	config.write_text(
		"if(NOT TARGET Boost::boost)\n"
		"  add_library(Boost::boost INTERFACE IMPORTED)\n"
		f"  set_property(TARGET Boost::boost PROPERTY INTERFACE_INCLUDE_DIRECTORIES {boost_headers})\n"
		"endif()\nset(Boost_FOUND TRUE)\nset(BOOST_FOUND TRUE)\n",
		encoding="utf-8",
	)
	(config_dir / "BoostConfigVersion.cmake").write_text(
		f"set(PACKAGE_VERSION {BOOST_VERSION})\n"
		"set(PACKAGE_VERSION_COMPATIBLE TRUE)\n"
		"set(PACKAGE_VERSION_EXACT TRUE)\n",
		encoding="utf-8",
	)
	return validate_materialized_source(config_dir, output_root, "Boost CMake config")

#============================================
def minimal_rdkit_options(
	catch2_source: Path,
	better_enums_source: Path,
	boost_config: Path,
) -> list[str]:
	"""Map profile validation into the builder's public error contract."""
	try:
		return profile_rdkit_options(catch2_source, better_enums_source, boost_config)
	except ValueError as error:
		raise NativeBuildError(str(error)) from error

#============================================
def copy_rdkit_headers(source_root: Path, destination: Path) -> None:
	"""Stage one header tree without silently replacing a prior staged header."""
	for header_root in (source_root / "Code",):
		if not header_root.is_dir():
			raise NativeBuildError(f"RDKit header root is missing: {header_root}")
		for header in header_root.rglob("*"):
			if header.suffix not in (".h", ".hpp", ".inc"):
				continue
			if not header.is_file() or header.is_symlink():
				raise NativeBuildError(f"RDKit header must be a regular file: {header}")
			target = destination / header.relative_to(header_root)
			if target.exists():
				raise NativeBuildError(
				"RDKit generated-header overlay conflicts with an already staged header: "
				f"{target}"
			)
			target.parent.mkdir(parents=True, exist_ok=True)
			shutil.copy2(header, target)


#============================================
def stage_rdkit_inputs(output_root: Path, source: Path, build: Path) -> Path:
	"""Create private headers and the measured ABI-4 RDKit library closure."""
	stage = output_root / "rdkit-install"
	if stage.exists():
		raise NativeBuildError(f"refusing to overwrite RDKit stage: {stage}")
	staging = output_root / f".rdkit-install-{uuid.uuid4().hex}.staging"
	try:
		include = staging / "include" / "rdkit"
		copy_rdkit_headers(source, include)
		# CMake configures headers below build/Code. Add that distinct generated set
		# after source headers, but reject a path collision rather than letting either
		# tree silently change the other's file.
		generated = build / "Code"
		if not generated.is_dir():
			raise NativeBuildError(f"RDKit GraphMol build lacks generated-header root: {generated}")
		copy_rdkit_headers(build, include)
		inchi_header = source / "External" / "INCHI-API" / "inchi.h"
		if not inchi_header.is_file() or inchi_header.is_symlink():
			raise NativeBuildError(f"RDKit InChI wrapper header is missing: {inchi_header}")
		shutil.copy2(inchi_header, include / "GraphMol" / "inchi.h")
		ring_header = (
			source / "External" / "RingFamilies" / "RingDecomposerLib" / "src"
			/ "RingDecomposerLib" / "RingDecomposerLib.h"
		)
		if not ring_header.is_file() or ring_header.is_symlink():
			raise NativeBuildError(f"RDKit ring-decomposer header is missing: {ring_header}")
		shutil.copy2(ring_header, include / "RingDecomposerLib.h")
		lib_dir = staging / "lib"
		lib_dir.mkdir(parents=True)
		for library_name in RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES:
			stem = library_name.removesuffix(".1.dylib")
			candidates = sorted({candidate.resolve() for candidate in (build / "lib").glob(f"{stem}.*.dylib")})
			if len(candidates) != 1:
				raise NativeBuildError(
					"Ferrum RDKit profile did not produce exactly one required library for "
					f"{library_name}: {candidates}"
				)
			shutil.copy2(candidates[0], lib_dir / library_name)
		staging.replace(stage)
	except OSError as error:
		shutil.rmtree(staging, ignore_errors=True)
		raise NativeBuildError(f"could not atomically stage native RDKit inputs: {error}") from error
	except NativeBuildError:
		shutil.rmtree(staging, ignore_errors=True)
		raise
	return stage


#============================================
def build_rdkit(output_root: Path, archive_root: Path | None) -> RdkitLayout:
	source = prepare_source(output_root, archive_root)
	catch2_source, better_enums_source, boost_headers, inchi_source = materialize_retained_rdkit_inputs(
		output_root, archive_root
	)
	install_pinned_inchi_source(source, inchi_source)
	boost_config = materialize_boost_headers_config(output_root, boost_headers)
	build = output_root / "rdkit-build"
	install = output_root / "rdkit-install"
	if build.exists() or install.exists():
		if not (build.is_dir() and install.is_dir()):
			raise NativeBuildError("refusing to overwrite an incomplete RDKit build; choose a fresh output root")
		try:
			llvm_root = homebrew_llvm()
			cmake = homebrew_cmake()
			sdk_root = apple_sdk()
			validate_resolved_rdkit_configuration(build)
			provenance_audit = audit_cmake_provenance(build, output_root, llvm_root, cmake, sdk_root)
		except (NativePolicyError, ValueError) as error:
			raise NativeBuildError(str(error)) from error
		if not (output_root / "ferrum-native-inputs.json").is_file():
			stage_rdkit_inputs(output_root, source, build)
			publish_native_input_manifest(output_root)
		layout = rdkit_layout_from_output_root(output_root)
		return RdkitLayout(
			input_root=layout.input_root,
			lib_dir=layout.lib_dir,
			include_dir=layout.include_dir,
			boost_include_dir=layout.boost_include_dir,
			graphmol_library=layout.graphmol_library,
			rdgeneral_library=layout.rdgeneral_library,
			depictor_library=layout.depictor_library,
			smilesparse_library=layout.smilesparse_library,
			fileparsers_library=layout.fileparsers_library,
			rdinchi_library=layout.rdinchi_library,
			substructmatch_library=layout.substructmatch_library,
			cmake_options=tuple(minimal_rdkit_options(catch2_source, better_enums_source, boost_config)),
			toolchain=toolchain_receipt(llvm_root, cmake, sdk_root),
			provenance_audit=provenance_audit,
		)
	options = minimal_rdkit_options(catch2_source, better_enums_source, boost_config)
	options.append(f"-DCMAKE_INSTALL_PREFIX={install}")
	validate_rdkit_configuration(options)
	try:
		llvm_root = homebrew_llvm()
		cmake = homebrew_cmake()
		sdk_root = apple_sdk()
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	options.extend(cmake_toolchain_options(llvm_root, sdk_root))
	run(
		str(cmake), "-S", str(source), "-B", str(build),
		*options,
		env=native_tool_environment(llvm_root, cmake),
	)
	validate_resolved_rdkit_configuration(build)
	try:
		provenance_audit = audit_cmake_provenance(build, output_root, llvm_root, cmake, sdk_root)
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	run(
		str(cmake), "--build", str(build), "--target", "FileParsers", "RDInchiLib", "--parallel",
		env=native_tool_environment(llvm_root, cmake),
	)
	stage_rdkit_inputs(output_root, source, build)
	publish_native_input_manifest(output_root)
	layout = rdkit_layout_from_output_root(output_root)
	return RdkitLayout(
		input_root=layout.input_root,
		lib_dir=layout.lib_dir,
		include_dir=layout.include_dir,
		boost_include_dir=layout.boost_include_dir,
		graphmol_library=layout.graphmol_library,
		rdgeneral_library=layout.rdgeneral_library,
		depictor_library=layout.depictor_library,
		smilesparse_library=layout.smilesparse_library,
		fileparsers_library=layout.fileparsers_library,
		rdinchi_library=layout.rdinchi_library,
		substructmatch_library=layout.substructmatch_library,
		cmake_options=tuple(options),
		toolchain=toolchain_receipt(llvm_root, cmake, sdk_root),
		provenance_audit=provenance_audit,
	)

#============================================
def validate_rdkit_configuration(options: list[str]) -> None:
	"""Map command-policy validation into the builder's error contract."""
	try:
		validate_profile_configuration(options)
	except ValueError as error:
		raise NativeBuildError(str(error)) from error


#============================================
def validate_resolved_rdkit_configuration(build: Path) -> None:
	"""Map configured-cache validation into the builder's error contract."""
	try:
		validate_profile_cache(build)
	except ValueError as error:
		raise NativeBuildError(str(error)) from error

#============================================
def configure_adapter(
	output_root: Path,
	layout: RdkitLayout,
	build_type: str = "Release",
) -> Path:
	"""Build one real adapter against the declared private RDKit installation."""
	if build_type not in ADAPTER_BUILD_TYPES:
		raise NativeBuildError(f"unsupported adapter build type: {build_type}")
	build = output_root / "adapter-build"
	install = output_root / "adapter-install"
	if build.exists() or install.exists():
		raise NativeBuildError("refusing to overwrite existing adapter build output")
	try:
		llvm_root = homebrew_llvm()
		cmake = homebrew_cmake()
		sdk_root = apple_sdk()
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	command = [
		str(cmake), "-S", str(NATIVE_SOURCE), "-B", str(build),
		f"-DCMAKE_BUILD_TYPE={build_type}",
		f"-DCMAKE_INSTALL_PREFIX={install}",
		f"-DFERRUM_CHEM_RDKIT_INCLUDE_DIR={layout.include_dir}",
		f"-DFERRUM_CHEM_BOOST_INCLUDE_DIR={layout.boost_include_dir}",
		f"-DFERRUM_CHEM_RDKIT_GRAPHMOL={layout.graphmol_library}",
		f"-DFERRUM_CHEM_RDKIT_RDGENERAL={layout.rdgeneral_library}",
		f"-DFERRUM_CHEM_RDKIT_DEPICTOR={layout.depictor_library}",
		f"-DFERRUM_CHEM_RDKIT_SMILESPARSE={layout.smilesparse_library}",
		f"-DFERRUM_CHEM_RDKIT_FILEPARSERS={layout.fileparsers_library}",
		f"-DFERRUM_CHEM_RDKIT_RDINCHI={layout.rdinchi_library}",
		f"-DFERRUM_CHEM_RDKIT_SUBSTRUCTMATCH={layout.substructmatch_library}",
	]
	command.extend(cmake_cxx_toolchain_options(llvm_root, sdk_root))
	run(*command, env=native_tool_environment(llvm_root, cmake))
	try:
		audit_cmake_provenance(
			build, output_root, llvm_root, cmake, sdk_root,
			source_roots=(NATIVE_SOURCE, layout.input_root),
		)
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	run(str(cmake), "--build", str(build), "--parallel", env=native_tool_environment(llvm_root, cmake))
	run(str(cmake), "--install", str(build), env=native_tool_environment(llvm_root, cmake))
	adapter = install / "lib" / "libferrum_chem.dylib"
	if not adapter.is_file():
		raise NativeBuildError(f"adapter build did not produce {adapter}")
	linked_names = {Path(item).name for item in otool_dependencies(adapter)}
	for library in (layout.graphmol_library, layout.rdgeneral_library,
			layout.depictor_library, layout.smilesparse_library, layout.fileparsers_library,
			layout.rdinchi_library, layout.substructmatch_library):
		if library.name not in linked_names:
			raise NativeBuildError(
				"adapter did not retain its declared RDKit loader dependency; "
				f"missing {library.name} from {sorted(linked_names)}"
			)
	return adapter
#============================================
def audit_wheel_closure(wheel: Path, output_root: Path) -> None:
	"""Inspect the packaged Mach-O files, not the source staging directory."""
	audit_root = output_root / "wheel-closure-audit"
	if audit_root.exists():
		raise NativeBuildError(f"refusing to overwrite wheel closure audit directory: {audit_root}")
	with zipfile.ZipFile(wheel) as contents:
		validate_wheel_members(contents.namelist())
		safe_extract_zip_members(contents, audit_root)
		package = audit_root
		extensions = sorted(package.glob("ferrum_chem*.so"))
		if not extensions:
			package = audit_root / "ferrum_chem"
			extensions = sorted(package.glob("ferrum_chem*.so"))
	if len(extensions) != 1:
		raise NativeBuildError(f"wheel must contain exactly one native extension, found {extensions}")
	package_libs = audit_root / ".dylibs"
	if not package_libs.is_dir():
		package_libs = package / ".dylibs"
	assert_clean_closure(extensions[0], package_libs)

#============================================
def build_wheel(
	output_root: Path, adapter: Path, layout: RdkitLayout, target: str,
) -> tuple[Path, dict[str, object]]:
	if sys.version_info[:2] != (3, 12):
		raise NativeBuildError(
			f"native wheel requires the Python 3.12 build interpreter, got {sys.version.split()[0]}; "
			"run this tool through source_me.sh"
		)
	output_root = output_root.resolve()
	stage = stage_ferrum_python_project(output_root)
	stage_native_notice_bundle(stage, RUST_PACKAGE_SOURCE, layout.input_root)
	package_libs = stage / "ferrum_chem" / ".dylibs" if (stage / "ferrum_chem").is_dir() else stage / ".dylibs"
	copy_and_rewrite_closure(adapter, layout.graphmol_library, package_libs)
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
	return wheels[0], source_closure

#============================================
def build_engine_bundle(output_root: Path, adapter: Path, layout: RdkitLayout, destination: Path) -> Path:
	"""Publish the same rewritten native closure in Ferrum's CLI bundle layout."""
	root = output_root.resolve()
	destination = destination.resolve()
	if not destination.is_relative_to(root):
		raise NativeBuildError("--engine-bundle-dir must be beneath --output-root")
	if destination.exists():
		raise NativeBuildError(f"refusing to overwrite existing engine bundle: {destination}")
	destination.mkdir(parents=True)
	copy_and_rewrite_closure(adapter, layout.graphmol_library, destination)
	assert_packaged_library_closure(destination)
	manifest = destination / BUNDLE_MANIFEST_NAME
	manifest.write_bytes(engine_bundle_manifest(
		sorted(destination.glob("*.dylib")), BUNDLE_SCHEMA, ADAPTER_ABI_VERSION, ADAPTER_NAME, sha256
	))
	if not (destination / ADAPTER_NAME).is_file():
		raise NativeBuildError(f"engine bundle lacks required adapter: {destination / ADAPTER_NAME}")
	return destination

#============================================
def emit_artifact_result(action: str, artifact: Path) -> None:
	"""Emit the sole stdout record for build actions after verifying its target."""
	artifact = artifact.resolve()
	if not artifact.is_file():
		raise NativeBuildError(f"{action} did not produce an artifact: {artifact}")
	print(json.dumps({
		"schema": MACHINE_RESULT_SCHEMA,
		"action": action,
		"artifact": str(artifact),
	}, sort_keys=True))

#============================================
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
	wheel, source_closure = build_wheel(arguments.output_root, adapter, layout, TARGET)
	if arguments.engine_bundle_dir is not None:
		build_engine_bundle(arguments.output_root, adapter, layout, arguments.engine_bundle_dir)
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
			source_closure,
			wheel,
			MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names,
		)
		validate_build_receipt(
			arguments.output_root / "native-wheel-build-receipt.json", wheel, source_closure
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
def command_self_test(_: argparse.Namespace) -> None:
	"""Exercise policy helpers without a native build or host-native fixture."""
	native_wheel_builder_self_test.run(sys.modules[__name__])
	print("native wheel pure helper checks passed")

#============================================
def command_validate_publication(arguments: argparse.Namespace) -> None:
	"""Validate one copied publication candidate without rebuilding native sources."""
	validate_publication_candidate(arguments.receipt, arguments.wheel, arguments.staged_source_root)
	validate_publication_engine_bundle(arguments.engine_bundle)
	print(json.dumps({
		"action": "validate-publication",
		"schema": MACHINE_RESULT_SCHEMA,
		"validated": True,
	}, sort_keys=True))

#============================================
def main() -> int:
	"""Parse one command and execute its selected native-wheel operation.

	Returns:
		Zero after the selected command completes successfully.
	"""
	try:
		arguments = native_wheel_builder_cli.parser(
			command_build,
			command_adapter,
			command_self_test,
			command_validate_publication,
			output_path,
			engine_bundle_path,
			archive_root_path,
		).parse_args()
		arguments.handler(arguments)
		return 0
	except (NativeBuildError, NativeMachoError, NativePackagingError) as error:
		print(f"initial native-wheel build error: {error}", file=sys.stderr)
		return 1

if __name__ == "__main__":
	raise SystemExit(main())
