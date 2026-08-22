"""Models and immutable paths for repository-local native-engine construction."""

from __future__ import annotations

import argparse
import os
from dataclasses import dataclass
from pathlib import Path

from engine_lib.native_engine_adapter_abi import adapter_abi_version_from_header
from engine_lib.native_engine_profile import FERRUM_RDKIT_PROFILE
from engine_lib.native_engine_receipt import NativeReceiptError, validate_native_input_manifest

REPO_ROOT = Path(__file__).resolve().parents[3]
NATIVE_SOURCE = REPO_ROOT / "packages/ferrum-rust/crates/chemistry/native"
DOWNLOAD_ATTEMPTS = 3
ADAPTER_BUILD_TYPES = ("Release", "RelWithDebInfo")
ADAPTER_HEADER = NATIVE_SOURCE / "include/ferrum_chem_adapter.h"
ADAPTER_NAME = "libferrum_chem.dylib"

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
