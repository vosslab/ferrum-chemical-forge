"""Build Ferrum's pinned, C++-only RDKit native-wheel packaging proof.

The immutable capability profile below is the source of truth for what Ferrum
asks RDKit to build.  It deliberately does not package RDKit Python, SWIG, or
compiled Boost libraries.  Generated state belongs below one ignored output
root; ``OTHER_REPOS`` is never a source, build, test, or runtime input.
"""

from __future__ import annotations

# Standard library imports.
import argparse
import json
import os
import platform
import re
import stat
import shutil
import subprocess
import sys
import sysconfig
import tarfile
import time
import urllib.request
import urllib.parse
import uuid
import zipfile
from dataclasses import dataclass
from pathlib import Path

# Local builder self-test fixture
import native_wheel_builder_self_test

# Local native-wheel closure
from native_wheel_macho import (
	NativeMachoError,
	assert_clean_closure,
	assert_packaged_library_closure,
	copy_and_rewrite_closure,
	detect_variants,
	otool_dependencies,
)

# Local native-wheel policy
from native_wheel_policy import (
	NativePolicyError,
	apple_sdk,
	audit_cmake_provenance,
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
	RDKIT_URL,
	RdkitCapabilityProfile,
	TARGET,
)
from native_wheel_receipt import (
	NativeReceiptError,
	sha256,
	source_archive_path,
	validate_native_input_manifest,
	write_native_input_manifest,
	write_build_receipt,
)

REPO_ROOT = Path(__file__).resolve().parents[3]
NATIVE_SOURCE = REPO_ROOT / "packages/ferrum-rust/crates/chemistry/native"
PYTHON_SOURCE = REPO_ROOT / "packages/ferrum-rust/crates/api/python"
DOWNLOAD_ATTEMPTS = 3
ADAPTER_BUILD_TYPES = ("Release", "RelWithDebInfo")
ADAPTER_HEADER = NATIVE_SOURCE / "include/ferrum_chem_adapter.h"
ADAPTER_ABI_PATTERN = re.compile(
	r"^\s*#define\s+FERRUM_CHEM_ADAPTER_ABI_VERSION\s+([1-9][0-9]*)U\s*$",
	flags=re.MULTILINE,
)
KEKULIZE_RDKIT_LIBRARIES = (
	"libRDKitGraphMol.1.dylib",
	"libRDKitRDGeometryLib.1.dylib",
	"libRDKitDataStructs.1.dylib",
	"libRDKitRDGeneral.1.dylib",
)


class NativeBuildError(RuntimeError):
	"""An actionable failure in the build or closure contract."""


#============================================
def validated_https_url(url: str, label: str) -> str:
	"""Accept one credential-free HTTPS URL before any request can use it."""
	parsed_url = urllib.parse.urlsplit(url)
	if parsed_url.scheme != "https" or not parsed_url.hostname:
		raise NativeBuildError(f"{label} URL must use HTTPS with a host: {url}")
	if parsed_url.username or parsed_url.password or parsed_url.fragment:
		raise NativeBuildError(f"{label} URL must not contain credentials or a fragment")
	return url


class HttpsOnlyRedirectHandler(urllib.request.HTTPRedirectHandler):
	"""Reject every unsafe redirect before urllib constructs its next request."""

	def redirect_request(
		self,
		request: urllib.request.Request,
		file_pointer: object,
		code: int,
		message: str,
		headers: object,
		new_url: str,
	) -> urllib.request.Request | None:
		validated_https_url(new_url, "redirect")
		redirect = super().redirect_request(
			request,
			file_pointer,
			code,
			message,
			headers,
			new_url,
		)
		return redirect


#============================================
def adapter_abi_version_from_header() -> int:
	"""Read the one public ABI authority without interpreting C++ implementation."""
	try:
		header = ADAPTER_HEADER.read_text(encoding="utf-8")
	except OSError as error:
		raise NativeBuildError(f"cannot read Ferrum-Chem ABI header: {ADAPTER_HEADER}") from error
	versions = ADAPTER_ABI_PATTERN.findall(header)
	if len(versions) != 1:
		raise NativeBuildError(
			"Ferrum-Chem ABI header must define exactly one positive "
			"FERRUM_CHEM_ADAPTER_ABI_VERSION macro"
		)
	return int(versions[0])


ADAPTER_ABI_VERSION = adapter_abi_version_from_header()


@dataclass(frozen=True)
class RdkitLayout:
	input_root: Path
	lib_dir: Path
	include_dir: Path
	boost_include_dir: Path
	graphmol_library: Path
	rdgeneral_library: Path
	cmake_options: tuple[str, ...]
	toolchain: dict[str, str]
	provenance_audit: dict[str, object]


#============================================
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

#============================================
def output_path(value: str) -> Path:
	"""Return one resolved output path accepted by the command-line parser.

	Args:
		value: The user-provided output directory text.

	Returns:
		The resolved output directory path.
	"""
	path = Path(value).expanduser().resolve()
	if path.is_relative_to(REPO_ROOT / "OTHER_REPOS"):
		raise argparse.ArgumentTypeError("--output-root must not be inside OTHER_REPOS")
	try:
		relative = path.relative_to(REPO_ROOT)
	except ValueError as error:
		raise argparse.ArgumentTypeError("--output-root must be inside this checkout") from error
	if not relative.parts or not relative.parts[0].startswith("output"):
		raise argparse.ArgumentTypeError("--output-root must be beneath a root ignored output* directory")
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
	graphmol_library = validate_materialized_alias(
		root / str(paths["graphmol_library"]), root, "RDKit GraphMol library"
	)
	rdgeneral_library = validate_materialized_alias(
		root / str(paths["rdgeneral_library"]), root, "RDKit RDGeneral library"
	)
	boost_include_dir = validate_materialized_source(
		root / str(paths["boost_include_dir"]), root, "Boost include"
	)
	lib_dir = validate_materialized_source(graphmol_library.parent, root, "RDKit library")
	if not (include_dir / "GraphMol" / "MolOps.h").is_file():
		raise NativeBuildError(f"RDKit installation lacks GraphMol headers: {include_dir}")
	if not (include_dir / "RDGeneral" / "types.h").is_file():
		raise NativeBuildError(f"RDKit installation lacks RDGeneral headers: {include_dir}")
	if not lib_dir.is_dir():
		raise NativeBuildError(f"RDKit installation lacks library directory: {lib_dir}")
	return RdkitLayout(
		input_root=root,
		lib_dir=lib_dir,
		include_dir=include_dir,
		boost_include_dir=boost_include_dir,
		graphmol_library=graphmol_library,
		rdgeneral_library=rdgeneral_library,
		cmake_options=(),
		toolchain={},
		provenance_audit={},
	)


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
			required_rdkit_library(lib_dir, "libRDKitGraphMol.1.dylib", output_root),
			required_rdkit_library(lib_dir, "libRDKitRDGeneral.1.dylib", output_root),
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
		except (OSError, NativeBuildError) as error:
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
def safe_extract(archive: Path, destination: Path) -> Path:
	"""Extract one tar archive after rejecting traversal and duplicate entries.

	Args:
		archive: The verified tar archive to unpack.
		destination: The empty directory that receives the archive contents.

	Returns:
		The archive's single top-level source directory.
	"""
	with tarfile.open(archive, "r:gz") as contents:
		members = contents.getmembers()
		seen = set()
		for member in members:
			member_path = (destination / member.name).resolve()
			if not member_path.is_relative_to(destination.resolve()):
				raise NativeBuildError(f"RDKit archive contains an unsafe path: {member.name}")
			if member_path in seen:
				raise NativeBuildError(f"RDKit archive contains a duplicate path: {member.name}")
			seen.add(member_path)
		contents.extractall(destination, members, filter="data")
	children = [path for path in destination.iterdir() if path.is_dir()]
	if len(children) != 1:
		raise NativeBuildError("verified source archive must extract one top-level directory")
	return children[0]

#============================================
def safe_extract_zip(archive: Path, destination: Path) -> Path:
	"""Extract one ZIP archive after rejecting unsafe member structure.

	Args:
		archive: The verified ZIP archive to unpack.
		destination: The empty directory that receives the archive contents.

	Returns:
		The archive's single top-level source directory.
	"""
	with zipfile.ZipFile(archive) as contents:
		safe_extract_zip_members(contents, destination)
	children = [path for path in destination.iterdir() if path.is_dir()]
	if len(children) != 1:
		raise NativeBuildError("verified source archive must extract one top-level directory")
	return children[0]

#============================================
def safe_extract_zip_members(contents: zipfile.ZipFile, destination: Path) -> None:
	"""Extract regular ZIP members without traversal, links, or duplicate targets."""
	root = destination.resolve()
	seen = set()
	for member in contents.infolist():
		target = (destination / member.filename).resolve()
		if not target.is_relative_to(root):
			raise NativeBuildError(f"verified archive contains an unsafe path: {member.filename}")
		if target in seen:
			raise NativeBuildError(f"verified archive contains a duplicate path: {member.filename}")
		seen.add(target)
		mode = member.external_attr >> 16
		file_type = stat.S_IFMT(mode)
		if member.is_dir():
			if file_type not in (0, stat.S_IFDIR):
				raise NativeBuildError(f"verified archive contains an invalid directory: {member.filename}")
			target.mkdir(parents=True, exist_ok=True)
			continue
		if file_type not in (0, stat.S_IFREG):
			raise NativeBuildError(f"verified archive contains a non-regular file: {member.filename}")
		target.parent.mkdir(parents=True, exist_ok=True)
		with contents.open(member) as source, target.open("xb") as output:
			shutil.copyfileobj(source, output)
		# Preserve ordinary rwx bits, never setuid, setgid, or sticky archive bits.
		permissions = mode & 0o777
		if permissions:
			target.chmod(permissions)

#============================================
def download_dependency(output_root: Path, source_input: PinnedSource) -> Path:
	name, url, digest = source_input.name, source_input.url, source_input.sha256
	archive = materialized_archive_path(output_root, source_input)
	download_verified_archive(archive, url, digest, name)
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
def prepare_source(output_root: Path, archive_argument: str | None) -> Path:
	if archive_argument:
		external_archive = verified_archive(Path(archive_argument).resolve())
		if external_archive.is_relative_to(REPO_ROOT / "OTHER_REPOS"):
			raise NativeBuildError(f"RDKit archive must not resolve inside OTHER_REPOS: {external_archive}")
		archive = materialized_archive_path(output_root, FERRUM_RDKIT_PROFILE.rdkit)
		archive.parent.mkdir(parents=True, exist_ok=True)
		if archive.exists():
			raise NativeBuildError(f"refusing to overwrite materialized RDKit archive: {archive}")
		shutil.copy2(external_archive, archive)
		verified_archive(archive)
	else:
		archive = download_archive(output_root)
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
def materialize_retained_rdkit_inputs(output_root: Path) -> tuple[Path, Path, Path]:
	"""Supply only configure-time sources required by the kekulize profile."""
	inputs = {item.name: item for item in FERRUM_RDKIT_PROFILE.dependencies}
	catch2 = download_dependency(output_root, inputs["catch2"])
	better_enums = download_dependency(output_root, inputs["better-enums"])
	boost_headers = download_dependency(output_root, inputs["boost-headers"])
	return catch2, better_enums, boost_headers

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
	install: Path,
	catch2_source: Path,
	better_enums_source: Path,
	boost_config: Path,
) -> list[str]:
	"""Return the complete normalized policy for the immutable Ferrum profile."""
	options = list(FERRUM_RDKIT_PROFILE.cmake_options)
	options.extend((
		f"-DCMAKE_INSTALL_PREFIX={install}", f"-DBoost_DIR={boost_config}",
		f"-DCMAKE_PREFIX_PATH={boost_config.parent.parent.parent}",
		f"-DFETCHCONTENT_SOURCE_DIR_CATCH2={catch2_source}",
		f"-DFETCHCONTENT_SOURCE_DIR_BETTER_ENUMS={better_enums_source}",
		"-DCATCH_BUILD_TESTING=OFF",
	))
	validate_rdkit_configuration(options)
	return options

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
def stage_kekulize_rdkit_inputs(output_root: Path, source: Path, build: Path) -> Path:
	"""Create the exact private headers and four dylibs required by kekulization."""
	stage = output_root / "rdkit-install"
	if stage.exists():
		raise NativeBuildError(f"refusing to overwrite RDKit stage: {stage}")
	include = stage / "include" / "rdkit"
	copy_rdkit_headers(source, include)
	# CMake configures headers below build/Code. Add that distinct generated set
	# after source headers, but reject a path collision rather than letting either
	# tree silently change the other's file.
	generated = build / "Code"
	if not generated.is_dir():
		raise NativeBuildError(f"RDKit GraphMol build lacks generated-header root: {generated}")
	copy_rdkit_headers(build, stage / "include" / "rdkit")
	lib_dir = stage / "lib"
	lib_dir.mkdir(parents=True)
	for library_name in KEKULIZE_RDKIT_LIBRARIES:
		built_library = build / "lib" / library_name
		if not built_library.is_file():
			raise NativeBuildError(
				"GraphMol target did not produce required kekulize library: "
				f"{built_library}"
			)
		shutil.copy2(built_library, lib_dir / library_name)
	return stage


#============================================
def build_rdkit(output_root: Path, archive_argument: str | None) -> RdkitLayout:
	source = prepare_source(output_root, archive_argument)
	catch2_source, better_enums_source, boost_headers = materialize_retained_rdkit_inputs(output_root)
	boost_config = materialize_boost_headers_config(output_root, boost_headers)
	build = output_root / "rdkit-build"
	install = output_root / "rdkit-install"
	if build.exists() or install.exists():
		raise NativeBuildError("refusing to overwrite an RDKit build; choose a fresh output root")
	options = minimal_rdkit_options(install, catch2_source, better_enums_source, boost_config)
	validate_rdkit_configuration(options, output_root)
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
	try:
		provenance_audit = audit_cmake_provenance(build, output_root, llvm_root, cmake, sdk_root)
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	run(
		str(cmake), "--build", str(build), "--target", "GraphMol", "--parallel",
		env=native_tool_environment(llvm_root, cmake),
	)
	stage_kekulize_rdkit_inputs(output_root, source, build)
	publish_native_input_manifest(output_root)
	layout = rdkit_layout_from_output_root(output_root)
	return RdkitLayout(
		input_root=layout.input_root,
		lib_dir=layout.lib_dir,
		include_dir=layout.include_dir,
		boost_include_dir=layout.boost_include_dir,
		graphmol_library=layout.graphmol_library,
		rdgeneral_library=layout.rdgeneral_library,
		cmake_options=tuple(options),
		toolchain=toolchain_receipt(llvm_root, cmake, sdk_root),
		provenance_audit=provenance_audit,
	)

#============================================
def validate_rdkit_configuration(options: list[str], output_root: Path | None = None) -> None:
	"""Fail closed if a caller weakens the profile's non-discovery policy."""
	values = {option.split("=", 1)[0]: option.split("=", 1)[1] for option in options if "=" in option}
	required = {
		"-DRDK_BUILD_PYTHON_WRAPPERS": "OFF", "-DRDK_BUILD_SWIG_WRAPPERS": "OFF",
		"-DRDK_BUILD_INCHI_SUPPORT": "OFF", "-DRDK_BUILD_COORDGEN_SUPPORT": "OFF",
		"-DRDK_BUILD_MAEPARSER_SUPPORT": "OFF", "-DRDK_USE_BOOST_SERIALIZATION": "OFF",
		"-DRDK_USE_BOOST_IOSTREAMS": "OFF", "-DCMAKE_DISABLE_FIND_PACKAGE_Python": "ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Python3": "ON", "-DCMAKE_DISABLE_FIND_PACKAGE_Eigen3": "ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Catch2": "ON", "-DCMAKE_DISABLE_FIND_PACKAGE_maeparser": "ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_coordgen": "ON", "-DCMAKE_DISABLE_FIND_PACKAGE_TBB": "ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Inchi": "ON", "-DCMAKE_DISABLE_FIND_PACKAGE_INCHI": "ON",
		"-DRDK_BUILD_FREETYPE_SUPPORT": "OFF", "-DRDK_INSTALL_PYTHON_TESTS": "OFF",
		"-DRDK_USE_FLEXBISON": "OFF", "-DRDK_BUILD_THREADSAFE_SSS": "ON",
		"-DCATCH_BUILD_TESTING": "OFF",
		"-DFETCHCONTENT_FULLY_DISCONNECTED": "ON", "-DFETCHCONTENT_UPDATES_DISCONNECTED": "ON",
		"-DCMAKE_FIND_USE_PACKAGE_REGISTRY": "FALSE",
		"-DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY": "FALSE", "-DBoost_NO_SYSTEM_PATHS": "ON",
	}
	for option, expected in required.items():
		if values.get(option) != expected:
			raise NativeBuildError(f"Ferrum RDKit profile requires {option}={expected}")
	for forbidden in (
		"-DINCHI_INCLUDE_DIR", "-DINCHI_LIBRARY",
		"-DMAEPARSER_DIR", "-DCOORDGEN_DIR", "-DMAEPARSER_FORCE_BUILD",
		"-DCOORDGEN_FORCE_BUILD",
	):
		if forbidden in values:
			raise NativeBuildError(
				f"Ferrum kekulize profile must not configure future chemistry input: {forbidden}"
			)
	for option in options:
		if "/opt/homebrew" in option or "OTHER_REPOS" in option:
			raise NativeBuildError(
				f"Ferrum RDKit profile forbids host/reference path in CMake option: {option}"
			)

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
	]
	command.extend(cmake_toolchain_options(llvm_root, sdk_root))
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
	for library in (layout.graphmol_library, layout.rdgeneral_library):
		if library.name not in linked_names:
			raise NativeBuildError(
				"adapter did not retain its declared RDKit loader dependency; "
				f"missing {library.name} from {sorted(linked_names)}"
			)
	return adapter

#============================================
def find_maturin() -> str:
	"""Resolve Maturin from the required Python interpreter, never ambient PATH."""
	scripts = Path(sysconfig.get_path("scripts")).resolve()
	command = (scripts / "maturin").resolve()
	if not command.is_file() or not os.access(command, os.X_OK):
		raise NativeBuildError(
			f"maturin is required in the Python 3.12 scripts directory: {scripts}"
		)
	return str(command)

#============================================
def tool_version(command: str) -> str:
	result = subprocess.run([command, "--version"], text=True, capture_output=True, check=False)
	if result.returncode:
		raise NativeBuildError(f"could not determine tool version for {command}: {result.stderr.strip()}")
	return result.stdout.strip()

#============================================
def stage_python_project(output_root: Path) -> Path:
	"""Copy the tracked maturin project below output_root before adding build artifacts."""
	output_root = output_root.resolve()
	stage = output_root / "maturin-project"
	if stage.exists():
		raise NativeBuildError(f"refusing to overwrite staged maturin project: {stage}")
	shutil.copytree(
		PYTHON_SOURCE,
		stage,
		ignore=shutil.ignore_patterns(".libs", "target", "__pycache__", "*.pyc"),
	)
	return stage

#============================================
def validate_wheel_members(
	members: list[str],
	profile: RdkitCapabilityProfile = FERRUM_RDKIT_PROFILE,
) -> None:
	"""Reject Python-RDKit/SWIG payloads even when the loader does not use them."""
	native_extensions = [
		member for member in members if re.fullmatch(r"ferrum_api/_native[^/]*\.so", member)
	]
	if len(native_extensions) != 1:
		raise NativeBuildError(
			f"wheel must contain one Ferrum native extension, found {native_extensions}"
		)
	native_prefix = "ferrum_api/.libs/"
	native_members = {
		member.removeprefix(native_prefix)
		for member in members
		if member.startswith(native_prefix) and member.lower().endswith(".dylib")
	}
	expected_native = MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names
	if native_members != expected_native:
		raise NativeBuildError(
			"wheel native members differ from the frozen platform closure: "
			f"expected {sorted(expected_native)}, got {sorted(native_members)}"
		)
	for member in members:
		if member in native_extensions:
			continue
		if member.startswith(native_prefix) and member.removeprefix(native_prefix) in expected_native:
			continue
		lower = member.lower()
		if any(fragment in lower for fragment in profile.forbidden_wheel_fragments):
			raise NativeBuildError(f"wheel contains forbidden RDKit/Python wrapper content: {member}")
		if lower.endswith((".so", ".dylib", ".pyd")):
			raise NativeBuildError(f"wheel contains an unexpected native extension: {member}")

#============================================
def audit_wheel_closure(wheel: Path, output_root: Path) -> None:
	"""Inspect the packaged Mach-O files, not the source staging directory."""
	audit_root = output_root / "wheel-closure-audit"
	if audit_root.exists():
		raise NativeBuildError(f"refusing to overwrite wheel closure audit directory: {audit_root}")
	with zipfile.ZipFile(wheel) as contents:
		validate_wheel_members(contents.namelist())
		safe_extract_zip_members(contents, audit_root)
	package = audit_root / "ferrum_api"
	extensions = sorted(package.glob("_native*.so"))
	if len(extensions) != 1:
		raise NativeBuildError(f"wheel must contain exactly one native extension, found {extensions}")
	assert_clean_closure(extensions[0], package / ".libs")

#============================================
def build_wheel(output_root: Path, adapter: Path, layout: RdkitLayout, target: str) -> Path:
	if sys.version_info[:2] != (3, 12):
		raise NativeBuildError(
			f"native wheel requires the Python 3.12 build interpreter, got {sys.version.split()[0]}; "
			"run this tool through source_me.sh"
		)
	output_root = output_root.resolve()
	stage = stage_python_project(output_root)
	package_libs = stage / "ferrum_api" / ".libs"
	copy_and_rewrite_closure(adapter, layout.graphmol_library, package_libs)
	try:
		environment = rust_tool_environment(homebrew_llvm())
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	# Preserve the adapter's @rpath install identity at link time. Ferrum owns the
	# separately staged and rewritten .libs closure and audits it after packaging.
	environment["FERRUM_CHEM_LIB_DIR"] = str(adapter.parent)
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
	wheels = sorted(wheelhouse.glob("ferrum_api-*.whl"))
	if len(wheels) != 1:
		raise NativeBuildError(f"expected exactly one wheel in {wheelhouse}, found {wheels}")
	audit_wheel_closure(wheels[0], output_root)
	return wheels[0]

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
	if arguments.target != TARGET:
		raise NativeBuildError(
			f"initial native-wheel currently supports only {TARGET}, not {arguments.target}"
		)
	layout = build_rdkit(arguments.output_root, arguments.rdkit_archive)
	rdkit_libraries = sorted(layout.lib_dir.glob("libRDKit*.dylib"))
	if not rdkit_libraries:
		raise NativeBuildError(f"no RDKit dylibs were installed in {layout.lib_dir}")
	variants = detect_variants(rdkit_libraries)
	adapter = configure_adapter(arguments.output_root, layout)
	wheel = build_wheel(arguments.output_root, adapter, layout, arguments.target)
	try:
		write_build_receipt(
			arguments.output_root,
			FERRUM_RDKIT_PROFILE,
			ADAPTER_ABI_VERSION,
			layout.cmake_options,
			layout.toolchain,
			layout.provenance_audit,
			{"path": find_maturin(), "version": tool_version(find_maturin())},
			rust_toolchain_receipt(),
			variants,
			wheel,
			MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names,
		)
	except NativeReceiptError as error:
		raise NativeBuildError(str(error)) from error
	emit_artifact_result("wheel", wheel)

#============================================
def command_adapter(arguments: argparse.Namespace) -> None:
	layout = rdkit_layout_from_output_root(arguments.rdkit_output_root)
	adapter = configure_adapter(arguments.output_root, layout, arguments.build_type)
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
def parser() -> argparse.ArgumentParser:
	"""Create the native-wheel command parser and its subcommands.

	Returns:
		The fully configured command-line parser.
	"""
	result = argparse.ArgumentParser(description=__doc__)
	subcommands = result.add_subparsers(dest="command", required=True)
	build = subcommands.add_parser("build", help="verify RDKit, source-build it, then build a wheel")
	build.add_argument("--output-root", required=True, type=output_path)
	build.add_argument(
		"--rdkit-archive", help="existing archive; its pinned SHA-256 is always verified"
	)
	build.add_argument("--target", default=TARGET)
	build.set_defaults(handler=command_build)
	adapter = subcommands.add_parser(
		"adapter", help="build a replacement ABI-compatible adapter from sealed native inputs"
	)
	adapter.add_argument("--output-root", required=True, type=output_path)
	adapter.add_argument(
		"--rdkit-output-root",
		required=True,
		type=output_path,
		help=(
			"completed Ferrum native-build output root containing the private RDKit install "
			"and pinned Boost headers"
		),
	)
	adapter.add_argument(
		"--build-type",
		choices=ADAPTER_BUILD_TYPES,
		default="Release",
		help="CMake build variant for the ABI-compatible replacement adapter",
	)
	adapter.set_defaults(handler=command_adapter)
	self_test = subcommands.add_parser(
		"self-test", help="run deterministic native-wheel policy helper checks"
	)
	self_test.set_defaults(handler=command_self_test)
	return result

#============================================
def main() -> int:
	"""Parse one command and execute its selected native-wheel operation.

	Returns:
		Zero after the selected command completes successfully.
	"""
	try:
		arguments = parser().parse_args()
		arguments.handler(arguments)
		return 0
	except (NativeBuildError, NativeMachoError) as error:
		print(f"initial native-wheel build error: {error}", file=sys.stderr)
		return 1

if __name__ == "__main__":
	raise SystemExit(main())
