"""Pinned RDKit source archive, extraction, and staging operations."""

from __future__ import annotations

import argparse
import shutil
import sys
import time
import urllib.request
import uuid
import zipfile
from pathlib import Path

import wheel_lib.native_wheel_source_cache as native_wheel_source_cache
from wheel_lib.native_wheel_download import ArchiveExtractionError, HttpsOnlyRedirectHandler, safe_extract as extract_tar, safe_extract_zip as extract_zip, safe_extract_zip_members as extract_zip_members, validated_https_url
from wheel_lib.native_wheel_packaging import NativePackagingError, validate_wheel_members as validate_packaged_wheel_members
from wheel_lib.native_wheel_profile import BOOST_VERSION, FERRUM_RDKIT_PROFILE, RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES, RDKIT_SHA256, RDKIT_TAG, RDKIT_URL, PinnedSource, RdkitCapabilityProfile, minimal_rdkit_options as profile_rdkit_options
from wheel_lib.native_wheel_receipt import NativeReceiptError, sha256, source_archive_path, validate_native_input_manifest, write_native_input_manifest
from wheel_lib.native_wheel_builder_model import DOWNLOAD_ATTEMPTS, NativeBuildError, REPO_ROOT, RdkitLayout, pinned_boost_headers, rdkit_layout_from_output_root, validate_materialized_source


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
	return native_wheel_source_cache.managed_source_archive_cache_root(
		REPO_ROOT, FERRUM_RDKIT_PROFILE
	)


#============================================
def provision_managed_source_archive_cache() -> Path:
	"""Return a complete verified managed archive cache, provisioning only misses."""
	try:
		return native_wheel_source_cache.provision_managed_source_archive_cache(
			REPO_ROOT, FERRUM_RDKIT_PROFILE, verified_archive, download_verified_archive
		)
	except native_wheel_source_cache.NativeSourceCacheError as error:
		raise NativeBuildError(str(error)) from error


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
