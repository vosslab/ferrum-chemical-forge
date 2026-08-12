#!/usr/bin/env python3
"""Build the macOS arm64 M4a native-wheel packaging proof.

This tool is intentionally narrow.  It establishes the dynamic-loader and LGPL
replacement route; it does not expose RDKit chemistry to Python.  Every generated
file belongs below an explicit ignored output root.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
import time
import urllib.request
import uuid
import zipfile
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
NATIVE_SOURCE = REPO_ROOT / "packages/ferrum-rust/crates/chemistry/native"
PYTHON_SOURCE = REPO_ROOT / "packages/ferrum-rust/crates/api/python"
RDKIT_TAG = "Release_2026_03_4"
RDKIT_SHA256 = "a8bff65bdf13dd47a01f707f7759dd59124a8742f8c50952c2ceae9523b4fd2b"
RDKIT_URL = f"https://github.com/rdkit/rdkit/archive/refs/tags/{RDKIT_TAG}.tar.gz"
CATCH2_TAG = "v3.4.0"
CATCH2_SHA256 = "122928b814b75717316c71af69bd2b43387643ba076a6ec16e7882bfb2dfacbb"
CATCH2_URL = f"https://github.com/catchorg/Catch2/archive/refs/tags/{CATCH2_TAG}.tar.gz"
BETTER_ENUMS_REVISION = "c35576bed0295689540b39873126129adfa0b4c8"
BETTER_ENUMS_SHA256 = "9b78dcef7f88d1345b6f25335bfbcba5f024b08990c7d6dc605b833f4128b8dd"
BETTER_ENUMS_URL = f"https://github.com/aantron/better-enums/archive/{BETTER_ENUMS_REVISION}.tar.gz"
INCHI_VERSION = "1.07.3"
INCHI_SHA256 = "b42d828b5d645bd60bc43df7e0516215808d92e5a46c28e12b1f4f75dfaae333"
INCHI_URL = f"https://github.com/IUPAC-InChI/InChI/releases/download/v{INCHI_VERSION}/INCHI-1-SRC.zip"
MAEPARSER_VERSION = "1.3.3"
MAEPARSER_SHA256 = "78e7571a779ea4952e752ecef57c62fb26463947e29ef7f4b31b11988d88ca07"
MAEPARSER_URL = f"https://github.com/schrodinger/maeparser/archive/v{MAEPARSER_VERSION}.tar.gz"
COORDGEN_VERSION = "3.0.2"
COORDGEN_SHA256 = "f67697434f7fec03bca150a6d84ea0e8409f6ec49d5aab43badc5833098ff4e3"
COORDGEN_URL = f"https://github.com/schrodinger/coordgenlibs/archive/v{COORDGEN_VERSION}.tar.gz"
BOOST_VERSION = "1.86.0"
BOOST_SHA256 = "2575e74ffc3ef1cd0babac2c1ee8bdb5782a0ee672b1912da40e5b4b591ca01f"
BOOST_URL = f"https://archives.boost.io/release/{BOOST_VERSION}/source/boost_1_86_0.tar.gz"
TARGET = "aarch64-apple-darwin"
SYSTEM_PREFIXES = ("/usr/lib/", "/System/Library/")
DEP_LINE = re.compile(r"^\s*(\S+) \(")
RPATH_LINE = re.compile(r"^\s*path (\S+) \(offset ")
DOWNLOAD_ATTEMPTS = 3
MACHINE_RESULT_SCHEMA = "ferrum-m4a-artifact-v1"


class M4aError(RuntimeError):
	"""An actionable failure in the build or closure contract."""


@dataclass(frozen=True)
class RdkitLayout:
	source: Path
	build: Path
	lib_dir: Path


def run(*command: str, cwd: Path | None = None, env: dict[str, str] | None = None) -> None:
	print("+", " ".join(command), file=sys.stderr)
	try:
		# The command protocol reserves stdout for one final machine result.  Route
		# compiler and packager progress to stderr so callers never need to guess
		# whether a path is mixed with a build log.
		subprocess.run(command, cwd=cwd, env=env, stdout=sys.stderr, check=True)
	except FileNotFoundError as error:
		raise M4aError(f"required program is unavailable: {command[0]}") from error
	except subprocess.CalledProcessError as error:
		raise M4aError(f"command failed ({error.returncode}): {' '.join(command)}") from error


def output_path(value: str) -> Path:
	path = Path(value).expanduser().resolve()
	try:
		path.relative_to(REPO_ROOT)
	except ValueError as error:
		raise argparse.ArgumentTypeError("--output-root must be inside this checkout") from error
	if not path.relative_to(REPO_ROOT).parts[0].startswith("output"):
		raise argparse.ArgumentTypeError("--output-root must be beneath a root ignored output* directory")
	return path


def sha256(path: Path) -> str:
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


def verified_archive(path: Path, expected_sha256: str = RDKIT_SHA256, label: str = "RDKit") -> Path:
	if not path.is_file():
		raise M4aError(f"RDKit archive does not exist: {path}")
	actual = sha256(path)
	if actual != expected_sha256:
		raise M4aError(
			f"{label} archive SHA-256 mismatch for {path}: expected {expected_sha256}, got {actual}"
		)
	return path


def download_verified_archive(destination: Path, url: str, digest: str, label: str) -> Path:
	"""Publish a verified archive atomically, never leaving a partial cache entry."""
	destination.parent.mkdir(parents=True, exist_ok=True)
	if destination.exists():
		return verified_archive(destination, digest, label)
	for attempt in range(1, DOWNLOAD_ATTEMPTS + 1):
		temporary = destination.with_name(f".{destination.name}.{uuid.uuid4().hex}.download")
		try:
			print(f"downloading pinned {label} ({attempt}/{DOWNLOAD_ATTEMPTS}) from {url}", file=sys.stderr)
			with urllib.request.urlopen(url, timeout=60) as response, temporary.open("wb") as output:
				shutil.copyfileobj(response, output)
			verified_archive(temporary, digest, label)
			temporary.replace(destination)
			return destination
		except (OSError, M4aError) as error:
			temporary.unlink(missing_ok=True)
			if attempt == DOWNLOAD_ATTEMPTS:
				raise M4aError(
					f"could not download verified {label} after {DOWNLOAD_ATTEMPTS} attempts: {error}"
				) from error
			time.sleep(attempt)
	raise AssertionError("download retry loop must return or raise")


def download_archive(output_root: Path) -> Path:
	archive = output_root / "downloads" / f"rdkit-{RDKIT_TAG}.tar.gz"
	return download_verified_archive(archive, RDKIT_URL, RDKIT_SHA256, f"RDKit {RDKIT_TAG}")


def safe_extract(archive: Path, destination: Path) -> Path:
	with tarfile.open(archive, "r:gz") as contents:
		members = contents.getmembers()
		for member in members:
			member_path = (destination / member.name).resolve()
			if not member_path.is_relative_to(destination.resolve()):
				raise M4aError(f"RDKit archive contains an unsafe path: {member.name}")
		contents.extractall(destination, members, filter="data")
	children = [path for path in destination.iterdir() if path.is_dir()]
	if len(children) != 1:
		raise M4aError("verified source archive must extract one top-level directory")
	return children[0]


def safe_extract_zip(archive: Path, destination: Path) -> Path:
	with zipfile.ZipFile(archive) as contents:
		for member in contents.infolist():
			member_path = (destination / member.filename).resolve()
			if not member_path.is_relative_to(destination.resolve()):
				raise M4aError(f"verified archive contains an unsafe path: {member.filename}")
		contents.extractall(destination)
	children = [path for path in destination.iterdir() if path.is_dir()]
	if len(children) != 1:
		raise M4aError("verified source archive must extract one top-level directory")
	return children[0]


def download_dependency(output_root: Path, name: str, url: str, digest: str) -> Path:
	archive = output_root / "downloads" / f"{name}.tar.gz"
	download_verified_archive(archive, url, digest, name)
	destination = output_root / "dependencies" / name
	if destination.exists():
		raise M4aError(f"refusing to overwrite existing dependency source: {destination}")
	destination.mkdir(parents=True)
	return safe_extract(archive, destination)


def prepare_source(output_root: Path, archive_argument: str | None) -> Path:
	archive = verified_archive(Path(archive_argument).resolve()) if archive_argument else download_archive(output_root)
	source_parent = output_root / "source"
	source = source_parent / f"rdkit-{RDKIT_TAG}"
	if source.exists():
		raise M4aError(f"refusing to overwrite existing source tree: {source}; choose a fresh output root")
	source_parent.mkdir(parents=True, exist_ok=True)
	source = safe_extract(archive, source_parent)
	if source.name != f"rdkit-{RDKIT_TAG}" or not (source / "CMakeLists.txt").is_file():
		raise M4aError(f"verified archive did not contain rdkit-{RDKIT_TAG}/CMakeLists.txt")
	return source


def copy_retained_source(source: Path, destination: Path, label: str) -> None:
	if destination.exists():
		raise M4aError(f"refusing to overwrite {label} source at {destination}")
	shutil.copytree(source, destination)


def materialize_retained_rdkit_inputs(output_root: Path, source: Path) -> tuple[Path, Path]:
	"""Supply every enabled non-RDKit source before CMake is allowed to configure."""
	catch2 = download_dependency(output_root, "catch2-v3.4.0", CATCH2_URL, CATCH2_SHA256)
	better_enums = download_dependency(
		output_root, f"better-enums-{BETTER_ENUMS_REVISION}", BETTER_ENUMS_URL, BETTER_ENUMS_SHA256
	)
	maeparser = download_dependency(output_root, f"maeparser-{MAEPARSER_VERSION}", MAEPARSER_URL, MAEPARSER_SHA256)
	coordgen = download_dependency(output_root, f"coordgenlibs-{COORDGEN_VERSION}", COORDGEN_URL, COORDGEN_SHA256)
	inchi_archive = output_root / "downloads" / f"inchi-{INCHI_VERSION}.zip"
	download_verified_archive(inchi_archive, INCHI_URL, INCHI_SHA256, f"InChI {INCHI_VERSION}")
	inchi_unpack = output_root / "dependencies" / f"inchi-{INCHI_VERSION}"
	inchi_unpack.mkdir(parents=True)
	inchi = safe_extract_zip(inchi_archive, inchi_unpack)
	copy_retained_source(inchi, source / "External/INCHI-API/src", "InChI")
	copy_retained_source(maeparser, source / "External/CoordGen/maeparser", "MAEParser")
	copy_retained_source(coordgen, source / "External/CoordGen/coordgen", "CoordGen")
	return catch2, better_enums


def build_boost(output_root: Path) -> Path:
	"""Build the dynamically linked Boost subset under the ignored output root."""
	boost = download_dependency(output_root, f"boost-{BOOST_VERSION}", BOOST_URL, BOOST_SHA256)
	install = output_root / "boost-install"
	if install.exists():
		raise M4aError(f"refusing to overwrite Boost install: {install}")
	run("./bootstrap.sh", f"--prefix={install}", cwd=boost)
	run(
		"./b2", "install", "link=shared", "runtime-link=shared", "threading=multi",
		"--with-chrono", "--with-iostreams", "--with-regex", "--with-serialization",
		"--with-system", "--with-thread", cwd=boost,
	)
	if not any(install.joinpath("lib").glob("libboost_*.dylib")):
		raise M4aError(f"pinned Boost build produced no shared libraries under {install / 'lib'}")
	return install


def minimal_rdkit_options(
	install: Path, catch2_source: Path, better_enums_source: Path, boost_root: Path
) -> list[str]:
	"""The M4a future-compatible core: InChI, CoordGen/MAE, and dynamic Boost only."""
	return [
		"-DCMAKE_BUILD_TYPE=Release", "-DCMAKE_CXX_STANDARD=20", "-DCMAKE_CXX_STANDARD_REQUIRED=ON",
		"-DBUILD_SHARED_LIBS=ON", "-DRDK_INSTALL_INTREE=OFF", "-DRDK_INSTALL_STATIC_LIBS=OFF",
		"-DBoost_USE_STATIC_LIBS=OFF",
		"-DRDK_BUILD_PYTHON_WRAPPERS=OFF", "-DRDK_BUILD_SWIG_WRAPPERS=OFF",
		"-DRDK_BUILD_CPP_TESTS=OFF", "-DRDK_BUILD_INCHI_SUPPORT=ON",
		"-DRDK_BUILD_COORDGEN_SUPPORT=ON", "-DRDK_BUILD_MAEPARSER_SUPPORT=ON",
		"-DRDK_BUILD_CHEMDRAW_SUPPORT=OFF", "-DRDK_BUILD_PUBCHEMSHAPE_SUPPORT=OFF",
		"-DRDK_BUILD_DESCRIPTORS3D=OFF", "-DRDK_USE_URF=OFF",
		"-DRDK_BUILD_MOLINTERCHANGE_SUPPORT=OFF", "-DRDK_BUILD_AVALON_SUPPORT=OFF",
		"-DRDK_BUILD_FREESASA_SUPPORT=OFF", "-DRDK_BUILD_YAEHMOP_SUPPORT=OFF",
		"-DRDK_BUILD_XYZ2MOL_SUPPORT=OFF", "-DRDK_BUILD_STRUCTCHECKER_SUPPORT=OFF",
		"-DRDK_BUILD_CONTRIB=OFF", "-DRDK_BUILD_PGSQL=OFF", "-DRDK_BUILD_MINIMAL_LIB=OFF",
		"-DRDK_BUILD_CFFI_LIB=OFF", "-DRDK_BUILD_FUZZ_TARGETS=OFF",
		"-DRDK_BUILD_COMPRESSED_SUPPLIERS=OFF", "-DRDK_BUILD_SLN_SUPPORT=OFF",
		"-DRDK_USE_BOOST_STACKTRACE=OFF", "-DRDK_INSTALL_COMIC_FONTS=OFF",
		"-DFETCHCONTENT_FULLY_DISCONNECTED=ON",
		"-DFETCHCONTENT_UPDATES_DISCONNECTED=ON", "-DCMAKE_DISABLE_FIND_PACKAGE_Catch2=ON",
		"-DCMAKE_FIND_USE_PACKAGE_REGISTRY=FALSE",
		"-DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY=FALSE", f"-DCMAKE_INSTALL_PREFIX={install}",
		f"-DBOOST_ROOT={boost_root}", "-DBoost_NO_SYSTEM_PATHS=ON",
		f"-DFETCHCONTENT_SOURCE_DIR_CATCH2={catch2_source}",
		f"-DFETCHCONTENT_SOURCE_DIR_BETTER_ENUMS={better_enums_source}",
	]


def build_rdkit(output_root: Path, archive_argument: str | None) -> RdkitLayout:
	source = prepare_source(output_root, archive_argument)
	catch2_source, better_enums_source = materialize_retained_rdkit_inputs(output_root, source)
	boost_root = build_boost(output_root)
	build = output_root / "rdkit-build"
	install = output_root / "rdkit-install"
	if build.exists() or install.exists():
		raise M4aError("refusing to overwrite an RDKit build; choose a fresh output root")
	run(
		"cmake", "-S", str(source), "-B", str(build),
		*minimal_rdkit_options(install, catch2_source, better_enums_source, boost_root),
	)
	run("cmake", "--build", str(build), "--parallel")
	run("cmake", "--install", str(build))
	lib_dir = install / "lib"
	if not lib_dir.is_dir():
		raise M4aError(f"RDKit install did not create a library directory: {lib_dir}")
	return RdkitLayout(source, build, lib_dir)


def otool_dependencies(binary: Path) -> list[str]:
	result = subprocess.run(["otool", "-L", str(binary)], text=True, capture_output=True, check=False)
	if result.returncode:
		raise M4aError(f"otool -L failed for {binary}: {result.stderr.strip()}")
	dependencies: list[str] = []
	for line in result.stdout.splitlines()[1:]:
		match = DEP_LINE.match(line)
		if match:
			dependencies.append(match.group(1))
	return dependencies


def otool_rpaths(binary: Path) -> list[str]:
	result = subprocess.run(["otool", "-l", str(binary)], text=True, capture_output=True, check=False)
	if result.returncode:
		raise M4aError(f"otool -l failed for {binary}: {result.stderr.strip()}")
	paths: list[str] = []
	lines = iter(result.stdout.splitlines())
	for line in lines:
		if line.strip() != "cmd LC_RPATH":
			continue
		for detail in lines:
			match = RPATH_LINE.match(detail)
			if match:
				paths.append(match.group(1))
				break
	return paths


def otool_identity(binary: Path) -> str:
	result = subprocess.run(["otool", "-D", str(binary)], text=True, capture_output=True, check=False)
	if result.returncode:
		raise M4aError(f"otool -D failed for {binary}: {result.stderr.strip()}")
	identities = [line.strip() for line in result.stdout.splitlines()[1:] if line.strip()]
	if len(identities) != 1:
		raise M4aError(f"expected exactly one dylib identity for {binary}, found {identities}")
	return identities[0]


def linked_names(libraries: list[Path]) -> set[str]:
	return {
		*(library.name for library in libraries),
		*(Path(dependency).name for library in libraries for dependency in otool_dependencies(library)),
	}


def versioned_dylib_name(name: str, stem: str) -> bool:
	"""Match one dylib basename, including the versioned names RDKit installs."""
	return bool(re.fullmatch(rf"{re.escape(stem)}(?:\.\d+)*\.dylib", name))


def select_variant(names: set[str], label: str, external: str, vendored: str) -> str:
	variants = {
		"external": external,
		"vendored": vendored,
	}
	found = [variant for variant, stem in variants.items() if any(versioned_dylib_name(name, stem) for name in names)]
	if len(found) != 1:
		raise M4aError(
			f"could not determine exactly one {label} variant from linked libraries: "
			f"expected one of {external}, {vendored}; saw {sorted(names)}"
		)
	return found[0]


def detect_variants_from_names(names: set[str]) -> dict[str, str]:
	"""Classify actual RDKit/InChI/CoordGen install names without filesystem I/O."""
	boost = sorted(name for name in names if versioned_dylib_name(name, "libboost_") or name.startswith("libboost_"))
	if not boost:
		raise M4aError(
			"RDKit shared-library closure did not expose separately linked Boost dylibs; "
			"M4a requires Boost to remain an explicit bundled dependency"
		)
	return {
		"inchi": select_variant(names, "InChI", "libinchi", "libRDKitInchi"),
		# The authoritative target is `libRDKitcoordgen.dylib` (lowercase coordgen).
		"coordgen": select_variant(names, "CoordGen", "libcoordgen", "libRDKitcoordgen"),
		"boost": ",".join(boost),
	}


def detect_variants(rdkit_libraries: list[Path]) -> dict[str, str]:
	return detect_variants_from_names(linked_names(rdkit_libraries))


def resolved_dependencies(binary: Path, search_directories: list[Path]) -> list[Path]:
	result: list[Path] = []
	for dependency in otool_dependencies(binary):
		if dependency.startswith(SYSTEM_PREFIXES):
			continue
		if dependency.startswith("@loader_path/"):
			candidate = binary.parent / Path(dependency).name
		elif dependency.startswith("@rpath/"):
			candidate = next((directory / Path(dependency).name for directory in search_directories if (directory / Path(dependency).name).is_file()), None)
			if candidate is None:
				raise M4aError(f"{binary} has an unresolved @rpath dependency: {dependency}")
		elif dependency.startswith("@"):
			raise M4aError(f"{binary} uses an unsupported loader reference: {dependency}")
		else:
			candidate = Path(dependency)
		if not candidate.is_file():
			raise M4aError(f"{binary} depends on missing non-system library: {dependency}")
		result.append(candidate.resolve())
	return result


def closure(seed: list[Path]) -> list[Path]:
	pending = list(seed)
	seen: set[Path] = set()
	search_directories = sorted({library.parent.resolve() for library in seed})
	while pending:
		library = pending.pop()
		if library in seen:
			continue
		seen.add(library)
		dependencies = resolved_dependencies(library, search_directories)
		search_directories.extend(dependency.parent for dependency in dependencies if dependency.parent not in search_directories)
		pending.extend(dependencies)
	return sorted(seen)


def install_name_tool(*arguments: str) -> None:
	run("install_name_tool", *arguments)


def replace_rpaths(binary: Path, expected: str) -> None:
	for rpath in otool_rpaths(binary):
		install_name_tool("-delete_rpath", rpath, str(binary))
	install_name_tool("-add_rpath", expected, str(binary))


def copy_and_rewrite_closure(adapter: Path, rdkit_library: Path | None, package_libs: Path) -> list[Path]:
	package_libs.mkdir(parents=True, exist_ok=True)
	seeds = [adapter]
	if rdkit_library is not None:
		seeds.append(rdkit_library)
	libraries = closure(seeds)
	for library in libraries:
		destination = package_libs / library.name
		if destination.exists():
			raise M4aError(f"duplicate dependency basename in closure: {library.name}")
		shutil.copy2(library, destination)
	for library in package_libs.glob("*.dylib"):
		install_name_tool("-id", f"@loader_path/{library.name}", str(library))
		replace_rpaths(library, "@loader_path")
		for dependency in otool_dependencies(library):
			if dependency.startswith(SYSTEM_PREFIXES):
				continue
			name = Path(dependency).name
			if not (package_libs / name).is_file():
				raise M4aError(f"unbundled non-system dependency remains: {dependency}")
			if dependency != f"@loader_path/{name}":
				install_name_tool("-change", dependency, f"@loader_path/{name}", str(library))
	return sorted(package_libs.glob("*.dylib"))


def validate_exact_rpaths(actual: list[str], expected: list[str], label: str) -> None:
	"""Require the complete LC_RPATH sequence, including multiplicity and order."""
	if actual != expected:
		raise M4aError(f"unexpected LC_RPATH entries for {label}: expected {expected}, got {actual}")


def validate_packaged_dylib_closure(
	identity: str, dependencies: list[str], rpaths: list[str], name: str, packaged_names: set[str]
) -> None:
	if identity != f"@loader_path/{name}":
		raise M4aError(f"packaged dylib has a non-packaged identity: {name} -> {identity}")
	validate_exact_rpaths(rpaths, ["@loader_path"], name)
	for dependency in dependencies:
		if dependency.startswith(SYSTEM_PREFIXES):
			continue
		expected = f"@loader_path/{Path(dependency).name}"
		if dependency != expected or Path(dependency).name not in packaged_names:
			raise M4aError(
				f"packaged dylib has an unbundled or non-loader-relative dependency: {name} -> {dependency}"
			)


def validate_extension_closure(dependencies: list[str], rpaths: list[str], has_adapter: bool) -> None:
	validate_exact_rpaths(rpaths, ["@loader_path/.libs"], "native extension")
	extension_dependencies = [dependency for dependency in dependencies if not dependency.startswith(SYSTEM_PREFIXES)]
	if extension_dependencies != ["@rpath/libferrum_chem.dylib"]:
		raise M4aError(
			"extension must depend only on @rpath/libferrum_chem.dylib outside macOS system libraries; "
			f"got {extension_dependencies}"
		)
	if not has_adapter:
		raise M4aError("extension rpath has no packaged libferrum_chem.dylib target")


def assert_packaged_dylib(binary: Path, package_libs: Path) -> None:
	validate_packaged_dylib_closure(
		otool_identity(binary),
		otool_dependencies(binary),
		otool_rpaths(binary),
		binary.name,
		{library.name for library in package_libs.glob("*.dylib")},
	)


def assert_clean_closure(extension: Path, package_libs: Path) -> None:
	validate_extension_closure(
		otool_dependencies(extension),
		otool_rpaths(extension),
		(package_libs / "libferrum_chem.dylib").is_file(),
	)
	for library in sorted(package_libs.glob("*.dylib")):
		assert_packaged_dylib(library, package_libs)


def configure_adapter(output_root: Path, marker: str, rdkit_library: Path | None) -> Path:
	build = output_root / f"adapter-{marker}-build"
	install = output_root / f"adapter-{marker}-install"
	if build.exists() or install.exists():
		raise M4aError(f"refusing to overwrite adapter output for marker {marker}")
	command = [
		"cmake", "-S", str(NATIVE_SOURCE), "-B", str(build),
		"-DCMAKE_BUILD_TYPE=Release", f"-DFERRUM_CHEM_BUILD_MARKER={marker}",
		f"-DCMAKE_INSTALL_PREFIX={install}",
	]
	if rdkit_library is not None:
		command.append(f"-DFERRUM_CHEM_RDKIT_RDGENERAL={rdkit_library}")
	run(*command)
	run("cmake", "--build", str(build), "--parallel")
	run("cmake", "--install", str(build))
	adapter = install / "lib" / "libferrum_chem.dylib"
	if not adapter.is_file():
		raise M4aError(f"adapter build did not produce {adapter}")
	if rdkit_library is not None and not any(
		Path(item).name.startswith(Path(rdkit_library).name.split(".")[0] + ".")
		for item in otool_dependencies(adapter)
	):
		raise M4aError(
			"adapter did not retain its declared RDKit loader dependency; "
			"the M4a closure must be proven by the linked binary, not CMake metadata"
		)
	return adapter


def find_maturin() -> str:
	command = os.environ.get("M4A_MATURIN") or shutil.which("maturin")
	if command is None:
		raise M4aError("maturin is required; install maturin>=1.8,<2.0 into the Python 3.12 environment")
	return command


def stage_python_project(output_root: Path) -> Path:
	"""Copy the tracked maturin project below output_root before adding build artifacts."""
	output_root = output_root.resolve()
	stage = output_root / "maturin-project"
	if stage.exists():
		raise M4aError(f"refusing to overwrite staged maturin project: {stage}")
	shutil.copytree(
		PYTHON_SOURCE,
		stage,
		ignore=shutil.ignore_patterns(".libs", "target", "__pycache__", "*.pyc"),
	)
	return stage


def audit_wheel_closure(wheel: Path, output_root: Path) -> None:
	"""Inspect the packaged Mach-O files, not the source staging directory."""
	audit_root = output_root / "wheel-closure-audit"
	if audit_root.exists():
		raise M4aError(f"refusing to overwrite wheel closure audit directory: {audit_root}")
	with zipfile.ZipFile(wheel) as contents:
		contents.extractall(audit_root)
	package = audit_root / "ferrum_api"
	extensions = sorted(package.glob("_native*.so"))
	if len(extensions) != 1:
		raise M4aError(f"wheel must contain exactly one native extension, found {extensions}")
	assert_clean_closure(extensions[0], package / ".libs")


def build_wheel(output_root: Path, adapter: Path, rdkit_library: Path | None, target: str) -> Path:
	if sys.version_info[:2] != (3, 12):
		raise M4aError(
			f"M4a requires the Python 3.12 build interpreter, got {sys.version.split()[0]}; "
			"run this tool through source_me.sh"
		)
	output_root = output_root.resolve()
	stage = stage_python_project(output_root)
	package_libs = stage / "ferrum_api" / ".libs"
	copy_and_rewrite_closure(adapter, rdkit_library, package_libs)
	environment = os.environ.copy()
	environment["FERRUM_CHEM_LIB_DIR"] = str(adapter.parent)
	# Maturin invokes Cargo.  Keep all generated state under ignored output root.
	environment["CARGO_TARGET_DIR"] = str(output_root / "maturin-target")
	wheelhouse = output_root / "wheelhouse"
	wheelhouse.mkdir(parents=True, exist_ok=True)
	run(
		find_maturin(), "build", "--release", "--target", target,
		"--interpreter", sys.executable, "--out", str(wheelhouse),
		cwd=stage, env=environment,
	)
	wheels = sorted(wheelhouse.glob("ferrum_api-*.whl"))
	if len(wheels) != 1:
		raise M4aError(f"expected exactly one wheel in {wheelhouse}, found {wheels}")
	audit_wheel_closure(wheels[0], output_root)
	return wheels[0]


def emit_artifact_result(action: str, artifact: Path) -> None:
	"""Emit the sole stdout record for build actions after verifying its target."""
	artifact = artifact.resolve()
	if not artifact.is_file():
		raise M4aError(f"{action} did not produce an artifact: {artifact}")
	print(json.dumps({
		"schema": MACHINE_RESULT_SCHEMA,
		"action": action,
		"artifact": str(artifact),
	}, sort_keys=True))


def command_build(arguments: argparse.Namespace) -> None:
	if platform.system() != "Darwin" or platform.machine() != "arm64":
		raise M4aError("M4a currently proves only macOS arm64; run on an arm64 macOS host")
	if arguments.target != TARGET:
		raise M4aError(f"M4a currently supports only {TARGET}, not {arguments.target}")
	layout = build_rdkit(arguments.output_root, arguments.rdkit_archive)
	rdkit_libraries = sorted(layout.lib_dir.glob("libRDKit*.dylib"))
	if not rdkit_libraries:
		raise M4aError(f"no RDKit dylibs were installed in {layout.lib_dir}")
	variants = detect_variants(rdkit_libraries)
	rdgeneral = next((library for library in rdkit_libraries if "RDGeneral" in library.name), None)
	if rdgeneral is None:
		raise M4aError("RDKit installation lacks libRDKitRDGeneral")
	adapter = configure_adapter(arguments.output_root, "wheel", rdgeneral)
	wheel = build_wheel(arguments.output_root, adapter, rdgeneral, arguments.target)
	(arguments.output_root / "m4a-build-receipt.json").write_text(json.dumps({
		"rdkit_tag": RDKIT_TAG, "rdkit_sha256": RDKIT_SHA256,
		"dependency_variants": variants, "wheel": str(wheel),
	}, indent=2) + "\n", encoding="utf-8")
	emit_artifact_result("wheel", wheel)


def command_adapter(arguments: argparse.Namespace) -> None:
	adapter = configure_adapter(arguments.output_root, arguments.marker, None)
	emit_artifact_result("adapter", adapter)


def command_self_test(_: argparse.Namespace) -> None:
	"""Exercise pure naming policy without a native build or binary fixture."""
	external = detect_variants_from_names({
		"libinchi.1.dylib", "libcoordgen.3.0.2.dylib", "libboost_iostreams.dylib",
	})
	if external["inchi"] != "external" or external["coordgen"] != "external":
		raise M4aError(f"external dependency variant self-test failed: {external}")
	vendored = detect_variants_from_names({
		"libRDKitInchi.1.dylib", "libRDKitcoordgen.1.dylib", "libboost_regex.dylib",
	})
	if vendored["inchi"] != "vendored" or vendored["coordgen"] != "vendored":
		raise M4aError(f"vendored dependency variant self-test failed: {vendored}")
	validate_extension_closure(
		["@rpath/libferrum_chem.dylib", "/usr/lib/libSystem.B.dylib"],
		["@loader_path/.libs"],
		True,
	)
	try:
		validate_extension_closure(
			["@rpath/libferrum_chem.dylib"],
			["@loader_path/.libs", "@loader_path/.libs"],
			True,
		)
	except M4aError:
		pass
	else:
		raise M4aError("native loader closure self-test accepted duplicate LC_RPATH entries")
	validate_packaged_dylib_closure(
		"@loader_path/libferrum_chem.dylib",
		["@loader_path/libRDKitRDGeneral.1.dylib", "/usr/lib/libSystem.B.dylib"],
		["@loader_path"],
		"libferrum_chem.dylib",
		{"libferrum_chem.dylib", "libRDKitRDGeneral.1.dylib"},
	)
	try:
		detect_variants_from_names({"libRDKitCoordGen.1.dylib", "libboost_regex.dylib"})
	except M4aError:
		pass
	else:
		raise M4aError("CoordGen case-sensitive naming self-test did not reject libRDKitCoordGen")
	options = set(minimal_rdkit_options(
		Path("/install"), Path("/catch2"), Path("/better-enums"), Path("/boost")
	))
	for required in (
		"-DCMAKE_CXX_STANDARD=20", "-DRDK_INSTALL_INTREE=OFF", "-DRDK_INSTALL_STATIC_LIBS=OFF",
		"-DRDK_BUILD_INCHI_SUPPORT=ON", "-DRDK_BUILD_COORDGEN_SUPPORT=ON",
		"-DRDK_BUILD_MAEPARSER_SUPPORT=ON", "-DFETCHCONTENT_FULLY_DISCONNECTED=ON",
		"-DRDK_BUILD_CHEMDRAW_SUPPORT=OFF", "-DRDK_BUILD_PUBCHEMSHAPE_SUPPORT=OFF",
		"-DRDK_BUILD_DESCRIPTORS3D=OFF", "-DRDK_USE_URF=OFF",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Catch2=ON", "-DBoost_NO_SYSTEM_PATHS=ON",
	):
		if required not in options:
			raise M4aError(f"minimal RDKit configuration omitted required option: {required}")
	for dependencies, rpaths, has_adapter in (
		(["/opt/homebrew/lib/libferrum_chem.dylib"], ["@loader_path/.libs"], True),
		(["@rpath/libferrum_chem.dylib"], ["/checkout/output"], True),
		(["@rpath/libferrum_chem.dylib"], ["@loader_path/.libs"], False),
	):
		try:
			validate_extension_closure(dependencies, rpaths, has_adapter)
		except M4aError:
			pass
		else:
			raise M4aError("native loader closure self-test accepted a host or unresolved path")
	print("native wheel pure helper checks passed")


def parser() -> argparse.ArgumentParser:
	result = argparse.ArgumentParser(description=__doc__)
	subcommands = result.add_subparsers(dest="command", required=True)
	build = subcommands.add_parser("build", help="verify RDKit, source-build it, then build a wheel")
	build.add_argument("--output-root", required=True, type=output_path)
	build.add_argument("--rdkit-archive", help="existing archive; its pinned SHA-256 is always verified")
	build.add_argument("--target", default=TARGET)
	build.set_defaults(handler=command_build)
	adapter = subcommands.add_parser("adapter", help="build only a marked replacement ABI-compatible adapter")
	adapter.add_argument("--output-root", required=True, type=output_path)
	adapter.add_argument("--marker", required=True, choices=("wheel", "replacement"))
	adapter.set_defaults(handler=command_adapter)
	self_test = subcommands.add_parser("self-test", help="run deterministic native-wheel policy helper checks")
	self_test.set_defaults(handler=command_self_test)
	return result


def main() -> int:
	try:
		arguments = parser().parse_args()
		arguments.handler(arguments)
		return 0
	except M4aError as error:
		print(f"M4a build error: {error}", file=sys.stderr)
		return 1


if __name__ == "__main__":
	raise SystemExit(main())
