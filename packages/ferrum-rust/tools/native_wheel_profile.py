"""Immutable RDKit capability and native-closure declarations for Ferrum."""

from __future__ import annotations

# Standard library
from dataclasses import dataclass


RDKIT_TAG = "Release_2026_03_4"
RDKIT_SHA256 = "a8bff65bdf13dd47a01f707f7759dd59124a8742f8c50952c2ceae9523b4fd2b"
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
TARGET = "aarch64-apple-darwin"
MACHINE_RESULT_SCHEMA = "ferrum-native-wheel-artifact-v1"


# ============================================================================
# Immutable data models

@dataclass(frozen=True)
class PinnedSource:
	"""One immutable upstream input materialized under the output root."""

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
	name="ferrum-rdkit-graphmol-kekulize-v1",
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
	),
	cmake_options=(
		"-DCMAKE_BUILD_TYPE=Release", "-DCMAKE_CXX_STANDARD=20",
		"-DCMAKE_CXX_STANDARD_REQUIRED=ON",
		"-DBUILD_SHARED_LIBS=ON", "-DRDK_INSTALL_INTREE=OFF", "-DRDK_INSTALL_STATIC_LIBS=OFF",
		"-DRDK_BUILD_PYTHON_WRAPPERS=OFF", "-DRDK_BUILD_SWIG_WRAPPERS=OFF",
		"-DRDK_BUILD_SWIG_JAVA_WRAPPER=OFF",
		"-DRDK_BUILD_SWIG_CSHARP_WRAPPER=OFF",
		"-DRDK_BUILD_DOTNET_CSHARP_TESTS=OFF",
		"-DRDK_BUILD_CPP_TESTS=OFF",
		"-DRDK_INSTALL_PYTHON_TESTS=OFF", "-DRDK_USE_FLEXBISON=OFF",
		"-DRDK_BUILD_THREADSAFE_SSS=ON", "-DRDK_BUILD_FREETYPE_SUPPORT=OFF",
		"-DRDK_BUILD_QT_SUPPORT=OFF", "-DRDK_BUILD_CAIRO_SUPPORT=OFF",
		"-DRDK_BUILD_INCHI_SUPPORT=OFF", "-DRDK_BUILD_COORDGEN_SUPPORT=OFF",
		"-DRDK_BUILD_MAEPARSER_SUPPORT=OFF",
		"-DRDK_USE_BOOST_SERIALIZATION=OFF",
		"-DRDK_USE_BOOST_IOSTREAMS=OFF", "-DRDK_BUILD_CHEMDRAW_SUPPORT=OFF",
		"-DRDK_BUILD_PUBCHEMSHAPE_SUPPORT=OFF",
		"-DRDK_BUILD_DESCRIPTORS3D=OFF",
		"-DRDK_USE_URF=OFF", "-DRDK_BUILD_MOLINTERCHANGE_SUPPORT=OFF",
		"-DRDK_BUILD_AVALON_SUPPORT=OFF", "-DRDK_BUILD_FREESASA_SUPPORT=OFF",
		"-DRDK_BUILD_YAEHMOP_SUPPORT=OFF", "-DRDK_BUILD_XYZ2MOL_SUPPORT=OFF",
		"-DRDK_BUILD_STRUCTCHECKER_SUPPORT=OFF", "-DRDK_BUILD_CONTRIB=OFF",
		"-DRDK_BUILD_PGSQL=OFF", "-DRDK_BUILD_MINIMAL_LIB=OFF", "-DRDK_BUILD_CFFI_LIB=OFF",
		"-DRDK_BUILD_FUZZ_TARGETS=OFF",
		"-DRDK_BUILD_COMPRESSED_SUPPLIERS=OFF",
		"-DRDK_BUILD_SLN_SUPPORT=OFF", "-DRDK_USE_BOOST_STACKTRACE=OFF",
		"-DRDK_INSTALL_COMIC_FONTS=OFF",
		"-DFETCHCONTENT_FULLY_DISCONNECTED=ON",
		"-DFETCHCONTENT_UPDATES_DISCONNECTED=ON", "-DCMAKE_DISABLE_FIND_PACKAGE_Python=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Python3=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Eigen3=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Catch2=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_Inchi=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_INCHI=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_maeparser=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_coordgen=ON",
		"-DCMAKE_DISABLE_FIND_PACKAGE_TBB=ON",
		"-DCMAKE_FIND_USE_PACKAGE_REGISTRY=FALSE",
		"-DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY=FALSE",
		"-DCMAKE_FIND_PACKAGE_PREFER_CONFIG=ON",
		"-DBoost_NO_SYSTEM_PATHS=ON",
		"-DBoost_USE_STATIC_LIBS=ON",
	),
	forbidden_wheel_fragments=("rdkit", "rdbase", "boost_python", "swig"),
	forbidden_native_fragments=("boost", "python", "swig", "rdbase"),
)


MACOS_ARM64_NATIVE_CLOSURE = MacosArm64NativeClosure(
	allowed_non_system_names=frozenset({
		"libferrum_chem.dylib",
		"libRDKitGraphMol.1.dylib",
		"libRDKitRDGeometryLib.1.dylib",
		"libRDKitDataStructs.1.dylib",
		"libRDKitRDGeneral.1.dylib",
	}),
)
