"""Hash-verified native input manifests for Ferrum's local runtime build."""

from __future__ import annotations

# Standard library imports.
import hashlib
import json
import stat
import unicodedata
from pathlib import Path

# Local imports.
from engine_lib.native_engine_profile import (
	MACOS_ARM64_NATIVE_CLOSURE,
	RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES,
	TARGET,
	PinnedSource,
	RdkitCapabilityProfile,
)


NATIVE_INPUT_MANIFEST_FILENAME = "ferrum-native-inputs.json"
NATIVE_INPUT_MANIFEST_SCHEMA = "ferrum-native-inputs-v4"


class NativeReceiptError(RuntimeError):
	"""A native build cannot publish complete reproducibility evidence."""


#============================================
def _profile_source_record(source: PinnedSource) -> dict[str, str]:
	"""Return one source declaration in canonical native-policy form."""
	return {
		"archive_filename": source.archive_filename,
		"name": source.name,
		"sha256": source.sha256,
		"url": source.url,
		"version": source.version,
	}


#============================================
def native_policy_record(profile: RdkitCapabilityProfile) -> dict[str, object]:
	"""Return the complete immutable policy that authorizes native reuse."""
	return {
		"macos_arm64_native_closure": sorted(
			MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names
		),
		"profile": {
			"cmake_options": list(profile.cmake_options),
			"dependencies": [_profile_source_record(source) for source in profile.dependencies],
			"forbidden_native_fragments": list(profile.forbidden_native_fragments),
			"name": profile.name,
			"rdkit": _profile_source_record(profile.rdkit),
		},
		"target": TARGET,
	}


#============================================
def native_policy_sha256(profile: RdkitCapabilityProfile) -> str:
	"""Fingerprint canonical policy bytes so a matching display name is insufficient."""
	canonical = json.dumps(
		native_policy_record(profile), separators=(",", ":"), sort_keys=True
	).encode("utf-8")
	return hashlib.sha256(canonical).hexdigest()


#============================================
def _tree_relative_path_key(relative: str, label: str) -> tuple[str, str]:
	"""Return portable digest and collision keys for one relative tree path."""
	try:
		normalized = unicodedata.normalize("NFC", relative)
		normalized.encode("utf-8", errors="strict")
	except UnicodeError as error:
		raise NativeReceiptError(f"{label} contains a path that cannot be encoded as UTF-8") from error
	return normalized, normalized.casefold()


#============================================
def _tree_digest_record(kind: bytes, relative: str, content_sha256: str = "") -> bytes:
	"""Frame one tree record so arbitrary filename bytes cannot concatenate."""
	path_bytes = relative.encode("utf-8", errors="strict")
	content_bytes = content_sha256.encode("ascii", errors="strict")
	return b"".join((
		kind,
		len(path_bytes).to_bytes(8, "big"),
		path_bytes,
		len(content_bytes).to_bytes(8, "big"),
		content_bytes,
	))


#============================================
def directory_tree_sha256(directory: Path, label: str) -> str:
	"""Hash a regular-file tree with fail-closed type and symlink handling."""
	try:
		root_status = directory.lstat()
	except OSError as error:
		raise NativeReceiptError(f"cannot inspect {label}: {directory}") from error
	if not stat.S_ISDIR(root_status.st_mode) or directory.is_symlink():
		raise NativeReceiptError(f"{label} must be a real directory, not a symlink or special file")
	entries: list[tuple[str, str, Path]] = []
	identity_keys: set[str] = set()
	try:
		paths = tuple(directory.rglob("*"))
	except OSError as error:
		raise NativeReceiptError(f"cannot enumerate {label}: {directory}") from error
	for path in paths:
		relative = path.relative_to(directory).as_posix()
		normalized, identity_key = _tree_relative_path_key(relative, label)
		if identity_key in identity_keys:
			raise NativeReceiptError(
				f"{label} contains case-fold or Unicode-normalization path collision: {relative}"
			)
		identity_keys.add(identity_key)
		entries.append((identity_key, normalized, path))
	digest = hashlib.sha256()
	for _identity_key, relative, path in sorted(entries):
		try:
			entry_status = path.lstat()
		except OSError as error:
			raise NativeReceiptError(f"cannot inspect {label} entry: {relative}") from error
		if stat.S_ISDIR(entry_status.st_mode):
			digest.update(_tree_digest_record(b"D", relative))
			continue
		if stat.S_ISREG(entry_status.st_mode):
			digest.update(_tree_digest_record(b"F", relative, sha256(path)))
			continue
		raise NativeReceiptError(
			f"{label} contains a symlink or unsupported file type: {relative}"
		)
	return digest.hexdigest()


#============================================
def _manifest_relative_path(path: Path, output_root: Path, label: str) -> str:
	"""Return a non-empty, root-contained relative path for native evidence."""
	root = output_root.resolve()
	lexical_root = output_root.absolute()
	lexical = path.absolute()
	try:
		relative = lexical.relative_to(lexical_root)
	except ValueError as error:
		resolved_fallback = path.resolve()
		try:
			relative = resolved_fallback.relative_to(root)
		except ValueError:
			raise NativeReceiptError(
				f"{label} is not below the native output root: {lexical}"
			) from error
	if ".." in relative.parts:
		raise NativeReceiptError(f"{label} contains an unsafe relative path: {relative}")
	resolved = path.resolve()
	if not resolved.is_relative_to(root):
		raise NativeReceiptError(f"{label} escapes the native output root: {resolved}")
	if not relative.parts:
		raise NativeReceiptError(f"{label} must not be the native output root")
	return relative.as_posix()


#============================================
def _manifest_path(output_root: Path, relative: object, label: str) -> Path:
	"""Validate one manifest path while retaining its declared alias identity."""
	if not isinstance(relative, str) or not relative:
		raise NativeReceiptError(f"native input manifest {label} must be a non-empty string")
	path = Path(relative)
	if path.is_absolute() or ".." in path.parts or relative != path.as_posix():
		raise NativeReceiptError(f"native input manifest {label} is not a safe relative path")
	if "OTHER_REPOS" in path.parts:
		raise NativeReceiptError(f"native input manifest {label} must not reference OTHER_REPOS")
	root = output_root.resolve()
	lexical = root / path
	resolved = lexical.resolve()
	if not resolved.is_relative_to(root):
		raise NativeReceiptError(f"native input manifest {label} escapes the output root")
	return lexical


#============================================
def _manifest_source_record(source: PinnedSource, output_root: Path) -> dict[str, str]:
	"""Record immutable source policy and the verified materialized archive bytes."""
	archive = source_archive_path(output_root, source)
	if not archive.is_file():
		raise NativeReceiptError(f"missing materialized source archive for {source.name}: {archive}")
	actual = sha256(archive)
	if actual != source.sha256:
		raise NativeReceiptError(
			f"materialized source archive SHA-256 mismatch for {source.name}: "
			f"expected {source.sha256}, got {actual}"
		)
	return {
		"archive_filename": source.archive_filename,
		"materialized_sha256": actual,
		"name": source.name,
		"sha256": source.sha256,
		"url": source.url,
		"version": source.version,
	}


#============================================
def _expected_native_input_paths(profile: RdkitCapabilityProfile) -> dict[str, str]:
	"""Derive the only reusable input locations from the immutable profile."""
	boost_sources = [source for source in profile.dependencies if source.name == "boost-headers"]
	if len(boost_sources) != 1:
		raise NativeReceiptError("native profile must declare exactly one Boost-header source")
	boost_directory = "boost_" + boost_sources[0].version.replace(".", "_")
	return {
		"boost_include_dir": f"dependencies/boost-headers/{boost_directory}",
		# RDKit's installed public headers retain the package prefix.  The adapter
		# includes <GraphMol/...>, so this must be the directory that contains
		# GraphMol/, rather than the enclosing CMake install include directory.
		"include_dir": "rdkit-install/include/rdkit",
		"rdkit_library_dir": "rdkit-install/lib",
	}


#============================================
def write_native_input_manifest(
	output_root: Path,
	profile: RdkitCapabilityProfile,
	include_dir: Path,
	boost_include_dir: Path,
	rdkit_library_dir: Path,
) -> Path:
	"""Publish the immutable, hash-verified inputs permitted for adapter reuse."""
	root = output_root.resolve()
	if not root.is_dir():
		raise NativeReceiptError(f"native output root does not exist: {root}")
	manifest = root / NATIVE_INPUT_MANIFEST_FILENAME
	if manifest.exists():
		raise NativeReceiptError(f"refusing to overwrite native input manifest: {manifest}")
	required_headers = (
		include_dir / "GraphMol" / "MolOps.h",
		include_dir / "GraphMol" / "Depictor" / "RDDepictor.h",
		include_dir / "GraphMol" / "SmilesParse" / "SmilesParse.h",
		include_dir / "GraphMol" / "SmilesParse" / "SmilesWrite.h",
		include_dir / "RDGeneral" / "types.h",
	)
	for header in required_headers:
		if not header.is_file():
			raise NativeReceiptError(f"missing required RDKit header: {header}")
	libraries = tuple(rdkit_library_dir / name for name in RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES)
	for library in libraries:
		if not library.is_file():
			raise NativeReceiptError(f"missing required RDKit library alias: {library}")
		if not library.resolve().is_file():
			raise NativeReceiptError(f"RDKit library alias has no regular-file target: {library}")
	if not (boost_include_dir / "boost" / "config.hpp").is_file():
		raise NativeReceiptError(f"missing required pinned Boost headers: {boost_include_dir}")
	# These full-tree fingerprints prevent an adapter rebuild from silently using
	# a changed transitive header that happens not to be one of the two headers
	# named by the C++ adapter itself.
	tree_digests = {
		"boost_include_dir_sha256": directory_tree_sha256(boost_include_dir, "Boost include root"),
		"rdkit_include_dir_sha256": directory_tree_sha256(include_dir, "RDKit include root"),
	}
	paths = {
		"boost_include_dir": _manifest_relative_path(
			boost_include_dir, root, "Boost include directory"
		),
		"include_dir": _manifest_relative_path(include_dir, root, "RDKit include directory"),
		"rdkit_library_dir": _manifest_relative_path(rdkit_library_dir, root, "RDKit library directory"),
	}
	expected_paths = _expected_native_input_paths(profile)
	if paths != expected_paths:
		raise NativeReceiptError(
			"native input locations drifted from the immutable Ferrum profile: "
			f"expected {expected_paths}, got {paths}"
		)
	record = {
		"artifacts": {
			"headers": [
				{"path": _manifest_relative_path(header, root, "RDKit header"), "sha256": sha256(header)}
				for header in required_headers
			],
			"libraries": [
				{
					"alias_path": _manifest_relative_path(library, root, "RDKit library alias"),
					"resolved_target_path": _manifest_relative_path(
						library.resolve(), root, "RDKit library target"
					),
					"sha256": sha256(library.resolve()),
				}
			for library in libraries
			],
		},
		"paths": paths,
		"policy": native_policy_record(profile),
		"policy_sha256": native_policy_sha256(profile),
		"schema": NATIVE_INPUT_MANIFEST_SCHEMA,
		"sources": [
			_manifest_source_record(source, root)
			for source in (profile.rdkit, *profile.dependencies)
		],
		"tree_digests": tree_digests,
	}
	manifest.write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
	validate_native_input_manifest(root, profile)
	return manifest


#============================================
def validate_native_input_manifest(
	output_root: Path, profile: RdkitCapabilityProfile
) -> dict[str, object]:
	"""Fail closed unless adapter inputs exactly match Ferrum's completed build record."""
	root = output_root.resolve()
	manifest = root / NATIVE_INPUT_MANIFEST_FILENAME
	if not manifest.is_file() or manifest.is_symlink():
		raise NativeReceiptError(f"missing regular native input manifest: {manifest}")
	try:
		record = json.loads(manifest.read_text(encoding="utf-8"))
	except (OSError, json.JSONDecodeError) as error:
		raise NativeReceiptError(f"cannot parse native input manifest: {manifest}") from error
	if not isinstance(record, dict) or set(record) != {
		"artifacts", "paths", "policy", "policy_sha256", "schema", "sources", "tree_digests"
	}:
		raise NativeReceiptError("native input manifest has an unexpected schema shape")
	if record["schema"] != NATIVE_INPUT_MANIFEST_SCHEMA:
		raise NativeReceiptError("native input manifest schema does not match Ferrum")
	if (
		record["policy"] != native_policy_record(profile)
		or record["policy_sha256"] != native_policy_sha256(profile)
	):
		raise NativeReceiptError("native input manifest policy does not match Ferrum")
	expected_sources = [
		_manifest_source_record(source, root)
		for source in (profile.rdkit, *profile.dependencies)
	]
	if record["sources"] != expected_sources:
		raise NativeReceiptError("native input manifest source evidence does not match Ferrum pins")
	paths = record["paths"]
	if not isinstance(paths, dict) or set(paths) != {
		"boost_include_dir", "include_dir", "rdkit_library_dir"
	}:
		raise NativeReceiptError("native input manifest paths have an unexpected schema shape")
	expected_paths = _expected_native_input_paths(profile)
	if paths != expected_paths:
		raise NativeReceiptError(
			"native input manifest paths do not match the immutable profile: "
			f"expected {expected_paths}, got {paths}"
		)
	resolved_paths = {
		name: _manifest_path(root, value, f"paths.{name}")
		for name, value in paths.items()
	}
	if (
		not resolved_paths["include_dir"].is_dir()
		or not resolved_paths["boost_include_dir"].is_dir()
	):
		raise NativeReceiptError("native input manifest declares a missing include directory")
	if not (resolved_paths["boost_include_dir"] / "boost" / "config.hpp").is_file():
		raise NativeReceiptError("native input manifest Boost headers are incomplete")
	tree_digests = record["tree_digests"]
	if not isinstance(tree_digests, dict) or set(tree_digests) != {
		"boost_include_dir_sha256", "rdkit_include_dir_sha256"
	} or any(not isinstance(value, str) for value in tree_digests.values()):
		raise NativeReceiptError("native input manifest tree digests have an unexpected schema shape")
	if tree_digests != {
		"boost_include_dir_sha256": directory_tree_sha256(
			resolved_paths["boost_include_dir"], "Boost include root"
		),
		"rdkit_include_dir_sha256": directory_tree_sha256(
			resolved_paths["include_dir"], "RDKit include root"
		),
	}:
		raise NativeReceiptError("native input manifest header-tree evidence does not match files")
	artifacts = record["artifacts"]
	if not isinstance(artifacts, dict) or set(artifacts) != {"headers", "libraries"}:
		raise NativeReceiptError("native input manifest artifacts have an unexpected schema shape")
	expected_headers = [
		resolved_paths["include_dir"] / "GraphMol" / "MolOps.h",
		resolved_paths["include_dir"] / "GraphMol" / "Depictor" / "RDDepictor.h",
		resolved_paths["include_dir"] / "GraphMol" / "SmilesParse" / "SmilesParse.h",
		resolved_paths["include_dir"] / "GraphMol" / "SmilesParse" / "SmilesWrite.h",
		resolved_paths["include_dir"] / "RDGeneral" / "types.h",
	]
	expected_header_records = [
		{"path": _manifest_relative_path(header, root, "RDKit header"), "sha256": sha256(header)}
		for header in expected_headers
		if header.is_file()
	]
	if (
		len(expected_header_records) != len(expected_headers)
		or artifacts["headers"] != expected_header_records
	):
		raise NativeReceiptError("native input manifest RDKit header evidence does not match files")
	expected_libraries = [
		resolved_paths["rdkit_library_dir"] / name
		for name in RDKIT_CLOSURE_LIBRARY_INSTALL_NAMES
	]
	expected_library_records = []
	for library in expected_libraries:
		if not library.is_file() or not library.resolve().is_file():
			raise NativeReceiptError("native input manifest declares a missing RDKit library")
		expected_library_records.append({
			"alias_path": _manifest_relative_path(library, root, "RDKit library alias"),
			"resolved_target_path": _manifest_relative_path(
				library.resolve(), root, "RDKit library target"
			),
			"sha256": sha256(library.resolve()),
		})
	if artifacts["libraries"] != expected_library_records:
		raise NativeReceiptError("native input manifest RDKit library evidence does not match files")
	return record


#============================================
def sha256(path: Path) -> str:
	"""Return the SHA-256 digest for one verified artifact."""
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


#============================================
def source_archive_path(output_root: Path, source: PinnedSource) -> Path:
	"""Resolve the one filename declared by the immutable source record."""
	archive = (output_root / "downloads" / source.archive_filename).resolve()
	if not archive.is_relative_to(output_root.resolve()):
		raise NativeReceiptError(f"source archive escapes the output root: {archive}")
	return archive


