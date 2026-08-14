"""Immutable RDKit capability and native-closure declarations for Ferrum."""

from __future__ import annotations

# Standard library
from dataclasses import dataclass
from pathlib import Path


# This exact tag and archive digest make one wheel build reproducible. They are
# not a permanent compatibility ceiling: new release builds move to RDKit's
# current stable release and retain the previous stable release as a semantic
# compatibility check.
RDKIT_TAG = "Release_2026_03_5"
RDKIT_SHA256 = "336b3ffd9b691e4bfcdf97d361c01e553de34d2ca85c64a941473e9e2f8b707e"
RDKIT_URL = f"https://github.com/rdkit/rdkit/archive/refs/tags/{RDKIT_TAG}.tar.gz"
CATCH2_TAG = "v3.4.0"
CATCH2_SHA256 = "122928b814b75717316c71af69bd2b43387643ba076a6ec16e7882bfb2dfacbb"
CATCH2_URL = f"https://github.com/catchorg/Catch2/archive/refs/tags/{CATCH2_TAG}.tar.gz"
BETTER_ENUMS_REVISION = "c35576bed0295689540b39873126129adfa0b4c8"
BETTER_ENUMS_SHA256 = "9b78dcef7f88d1345b6f25335bfbcba5f024b08990c7d6dc605b833f4128b8dd"
BETTER_ENUMS_URL = f"https://github.com/aantron/better-enums/archive/{BETTER_ENUMS_REVISION}.tar.gz"
BOOST_VERSION = "1.91.0"
BOOST_SHA256 = "5734305f40a76c30f951c9abd409a45a2a19fb546efe4162119250bbe4d3a463"
BOOST_URL = f"https://archives.boost.io/release/{BOOST_VERSION}/source/boost_1_91_0.tar.gz"
INCHI_VERSION = "1.07.3"
INCHI_SHA256 = "b42d828b5d645bd60bc43df7e0516215808d92e5a46c28e12b1f4f75dfaae333"
INCHI_URL = (
	"https://github.com/IUPAC-InChI/InChI/releases/download/"
	f"v{INCHI_VERSION}/INCHI-1-SRC.zip"
)
TARGET = "aarch64-apple-darwin"
MACHINE_RESULT_SCHEMA = "ferrum-native-wheel-artifact-v1"


# ============================================================================
# Immutable data models

@dataclass(frozen=True)
class PinnedSource:
	"""One immutable upstream input for a single reproducible artifact build."""

	name: str
	version: str
	url: str
	sha256: str
	archive_filename: str


@dataclass(frozen=True)
class RdkitCapabilityProfile:
	"""Stable capabilities separate from the measured platform closure."""

	name: str
	rdkit: PinnedSource
	dependencies: tuple[PinnedSource, ...]
	cmake_options: tuple[str, ...]
	forbidden_wheel_fragments: tuple[str, ...]
	forbidden_native_fragments: tuple[str, ...]


@dataclass(frozen=True)
class MacosArm64NativeClosure:
	"""Measured native closure for one platform, not a chemistry capability."""

	allowed_non_system_names: frozenset[str]


# ============================================================================
# Immutable profile declarations

FERRUM_RDKIT_PROFILE = RdkitCapabilityProfile(
	name="ferrum-rdkit-smiles-depict-fileparsers-inchi-fcm1-v2",
	rdkit=PinnedSource(
		"rdkit", RDKIT_TAG, RDKIT_URL, RDKIT_SHA256, f"rdkit-{RDKIT_TAG}.tar.gz",
	),
	dependencies=(
		# RDKit configures Catch2 even with its C++ tests disabled. Keep that
		# configure-time dependency local, hash-pinned, and offline.
		PinnedSource("catch2", CATCH2_TAG, CATCH2_URL, CATCH2_SHA256, "catch2.tar.gz"),
		# GraphMol's configuration generates its enum header through this pinned
		# source. It is a configure input, not an optional future capability.
		PinnedSource(
			"better-enums", BETTER_ENUMS_REVISION, BETTER_ENUMS_URL, BETTER_ENUMS_SHA256,
			"better-enums.tar.gz",
		),
		# RDKit's non-Python root always finds Boost. This profile supplies headers
		# only and forbids every compiled Boost dependency from its closure.
		PinnedSource(
			"boost-headers", BOOST_VERSION, BOOST_URL, BOOST_SHA256,
			"boost-headers.tar.gz",
		),
		# RDKit's InChI wrapper otherwise downloads this archive during CMake.
		# Materialize the exact upstream release before configuration instead.
		PinnedSource(
			"inchi-source", INCHI_VERSION, INCHI_URL, INCHI_SHA256,
			"inchi-1.07.3-source.zip",
		),
	),
	cmake_options=(
		# Build only the shared C++ libraries consumed by Ferrum.
		"-DCMAKE_BUILD_TYPE=Release", "-DBUILD_SHARED_LIBS=ON",
		"-DRDK_INSTALL_STATIC_LIBS=OFF", "-DRDK_BUILD_PYTHON_WRAPPERS=OFF",
		"-DRDK_BUILD_CPP_TESTS=OFF",
		# Preserve the one required optional capability and disable unused drawing.
		"-DRDK_BUILD_FREETYPE_SUPPORT=OFF",
		"-DRDK_BUILD_INCHI_SUPPORT=ON", "-DRDK_BUILD_COORDGEN_SUPPORT=OFF",
		"-DRDK_BUILD_MAEPARSER_SUPPORT=OFF",
		# These upstream-default-on features change FileParsers or its closure.
		"-DRDK_USE_BOOST_SERIALIZATION=OFF",
		"-DRDK_USE_BOOST_IOSTREAMS=OFF", "-DRDK_BUILD_CHEMDRAW_SUPPORT=OFF",
		"-DRDK_BUILD_PUBCHEMSHAPE_SUPPORT=OFF",
		"-DRDK_BUILD_DESCRIPTORS3D=OFF",
		"-DRDK_BUILD_MOLINTERCHANGE_SUPPORT=OFF", "-DRDK_BUILD_SLN_SUPPORT=OFF",
		# Resolve only the pinned sources supplied by the builder.
		"-DFETCHCONTENT_FULLY_DISCONNECTED=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Python3=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Eigen3=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Catch2=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_TBB=ON",
		# The pinned source tree supplies the InChI target. Do not let RDKit's
		# FindInchi module retain a Homebrew/system library in configured state.
		"-DCMAKE_DISABLE_FIND_PACKAGE_Inchi=ON", "-DINCHI_LIBRARIES=Inchi",
		"-DCMAKE_FIND_USE_PACKAGE_REGISTRY=FALSE",
		"-DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY=FALSE",
	),
	forbidden_wheel_fragments=("rdkit", "rdbase", "boost_python", "swig"),
	forbidden_native_fragments=("boost", "python", "swig", "rdbase"),
)


def minimal_rdkit_options(
	catch2_source: Path,
	better_enums_source: Path,
	boost_config: Path,
) -> list[str]:
	"""Return the normalized CMake interface for the immutable profile."""
	options = list(FERRUM_RDKIT_PROFILE.cmake_options)
	options.extend((
		f"-DBoost_DIR={boost_config}",
		f"-DFETCHCONTENT_SOURCE_DIR_CATCH2={catch2_source}",
		f"-DFETCHCONTENT_SOURCE_DIR_BETTER_ENUMS={better_enums_source}",
		"-DCATCH_BUILD_TESTING=OFF",
	))
	validate_rdkit_configuration(options)
	return options


def validate_rdkit_configuration(options: list[str]) -> None:
	"""Reject a command that weakens the profile's dependency policy."""
	values = {option.split("=", 1)[0]: option.split("=", 1)[1] for option in options if "=" in option}
	required = {
		"-DRDK_BUILD_PYTHON_WRAPPERS": "OFF",
		"-DRDK_BUILD_INCHI_SUPPORT": "ON", "-DRDK_BUILD_COORDGEN_SUPPORT": "OFF",
		"-DRDK_BUILD_MAEPARSER_SUPPORT": "OFF", "-DRDK_USE_BOOST_SERIALIZATION": "OFF",
		"-DRDK_USE_BOOST_IOSTREAMS": "OFF", "-DCMAKE_DISABLE_FIND_PACKAGE_Python3": "ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Eigen3": "ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Catch2": "ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_TBB": "ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Inchi": "ON", "-DINCHI_LIBRARIES": "Inchi",
		"-DRDK_BUILD_FREETYPE_SUPPORT": "OFF",
		"-DCATCH_BUILD_TESTING": "OFF", "-DFETCHCONTENT_FULLY_DISCONNECTED": "ON",
		"-DCMAKE_FIND_USE_PACKAGE_REGISTRY": "FALSE",
		"-DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY": "FALSE",
	}
	for option, expected in required.items():
		if values.get(option) != expected:
			raise ValueError(f"Ferrum RDKit profile requires {option}={expected}")
	for forbidden in (
		"-DINCHI_INCLUDE_DIR", "-DINCHI_LIBRARY", "-DMAEPARSER_DIR", "-DCOORDGEN_DIR",
		"-DMAEPARSER_FORCE_BUILD",
	):
		if forbidden in values:
			raise ValueError(
				f"Ferrum ABI-4 profile must not configure undeclared chemistry input: {forbidden}"
			)
	for option in options:
		if "/opt/homebrew" in option or "OTHER_REPOS" in option:
			raise ValueError(
				f"Ferrum RDKit profile forbids host/reference path in CMake option: {option}"
			)


def validate_resolved_rdkit_configuration(build: Path) -> None:
	"""Verify CMake honored the profile and relied-on pinned-source defaults."""
	cache = build / "CMakeCache.txt"
	if not cache.is_file():
		raise ValueError(f"RDKit configure did not produce {cache}")
	values = {
		line.partition("=")[0].partition(":")[0]: line.partition("=")[2]
		for line in cache.read_text(encoding="utf-8").splitlines()
		if "=" in line and not line.startswith(("//", "#"))
	}
	expected = {
		option.removeprefix("-D").split("=", 1)[0]: option.split("=", 1)[1]
		for option in FERRUM_RDKIT_PROFILE.cmake_options
	}
	expected.update({
		"RDK_BUILD_SWIG_WRAPPERS": "OFF",
		"RDK_BUILD_THREADSAFE_SSS": "ON",
		"RDK_USE_FLEXBISON": "OFF",
	})
	expected["CMAKE_INSTALL_PREFIX"] = str(build.parent / "rdkit-install")
	for key, required in expected.items():
		if values.get(key) != required:
			raise ValueError(
				f"resolved RDKit configuration requires {key}={required}, got {values.get(key)!r}"
			)


MACOS_ARM64_NATIVE_CLOSURE = MacosArm64NativeClosure(
	allowed_non_system_names=frozenset({
		"libferrum_chem.dylib",
		"libRDKitAlignment.1.dylib",
		"libRDKitGraphMol.1.dylib",
		"libRDKitRDGeometryLib.1.dylib",
		"libRDKitDataStructs.1.dylib",
		"libRDKitRDGeneral.1.dylib",
		"libRDKitSmilesParse.1.dylib",
		"libRDKitDepictor.1.dylib",
		"libRDKitChemTransforms.1.dylib",
		"libRDKitFileParsers.1.dylib",
		"libRDKitRDInchiLib.1.dylib",
		"libRDKitInchi.1.dylib",
		"libRDKitEigenSolvers.1.dylib",
		"libRDKitGenericGroups.1.dylib",
		"libRDKitMolAlign.1.dylib",
		"libRDKitMolTransforms.1.dylib",
		"libRDKitRingDecomposerLib.1.dylib",
		"libRDKitSubstructMatch.1.dylib",
	}),
)


RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES = tuple(sorted(
	name for name in MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names
	if name != "libferrum_chem.dylib"
))
