"""Native-wheel publication evidence and sealed payload helpers."""

from __future__ import annotations

import hashlib
import json
import os
import stat
import tempfile
import zipfile
from pathlib import Path

import native_wheel_bundle


class NativePublicationError(ValueError):
	"""A native-wheel publication candidate violates its evidence contract."""


DEVELOPER_WHEEL_PUBLICATION_SCHEMA = "ferrum-developer-wheel-publication-v1"


#============================================
def qt_source_closure(root: Path, sha256: object) -> dict[str, object]:
	"""Return the safe regular-file inventory used for one staged Qt wheel."""
	return ferrum_worktree_source_closure(
		root, "ferrum-qt-source-closure-v1", ("__pycache__", ".pytest_cache", "build"),
		(".pyc",), sha256,
	)


#============================================
def wheel_member_manifest(wheel: Path, sha256: object) -> dict[str, object]:
	"""Hash every safe, unique member of one wheel archive."""
	if wheel.is_symlink() or not wheel.is_file():
		raise NativePublicationError(f"wheel is not a regular file: {wheel}")
	with zipfile.ZipFile(wheel) as archive:
		members: list[dict[str, str]] = []
		seen: set[str] = set()
		for info in archive.infolist():
			name = info.filename
			if not name or name.endswith("/") or name.startswith("/") or ".." in Path(name).parts:
				raise NativePublicationError(f"wheel has an unsafe member path: {name!r}")
			if name in seen:
				raise NativePublicationError(f"wheel has a duplicate member: {name}")
			seen.add(name)
			members.append({"path": name, "sha256": hashlib.sha256(archive.read(info)).hexdigest()})
	payload = json.dumps(members, separators=(",", ":"), sort_keys=True).encode("utf-8")
	return {"members": sorted(members, key=lambda item: item["path"]),
		"fingerprint_sha256": hashlib.sha256(payload).hexdigest()}


#============================================
def developer_pair_receipt(
		candidate_root: Path, native_wheel: Path, qt_wheel: Path, native_receipt: Path,
		engine_bundle: Path, qt_source_closure_value: dict[str, object], sha256: object,
		) -> dict[str, object]:
	"""Bind both developer wheels and their independently verified source evidence."""
	try:
		native_value = json.loads(native_receipt.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise NativePublicationError(f"native build receipt is invalid JSON: {error.msg}") from error
	if not isinstance(native_value, dict):
		raise NativePublicationError("native build receipt is not an object")
	closure = native_value.get("ferrum_source_closure")
	if not isinstance(closure, dict) or not isinstance(closure.get("fingerprint_sha256"), str):
		raise NativePublicationError("native build receipt lacks its canonical source closure")
	engine_manifest = engine_bundle / "ferrum-engine-bundle-v1.json"
	if engine_manifest.is_symlink() or not engine_manifest.is_file():
		raise NativePublicationError("native engine bundle lacks its regular manifest")
	return {
		"schema": DEVELOPER_WHEEL_PUBLICATION_SCHEMA,
		"native_wheel": {"filename": native_wheel.name, "sha256": sha256(native_wheel)},
		"qt_wheel": {"filename": qt_wheel.name, "sha256": sha256(qt_wheel)},
		"native_receipt": {"filename": native_receipt.name, "sha256": sha256(native_receipt)},
		"engine_bundle_manifest_sha256": sha256(engine_manifest),
		"native_source_closure_sha256": closure["fingerprint_sha256"],
		"qt_source_closure": qt_source_closure_value,
		"qt_wheel_members": wheel_member_manifest(qt_wheel, sha256),
	}


#============================================
def require_qt_source_closure(
		qt_source_root: Path, closure_path: Path, sha256: object,
		) -> dict[str, object]:
	"""Load a pre-wheel Qt input closure and reject post-wheel staged-source drift."""
	if closure_path.is_symlink() or not closure_path.is_file():
		raise NativePublicationError("Qt source closure is not a regular staged file")
	try:
		expected = json.loads(closure_path.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise NativePublicationError(f"Qt source closure is invalid JSON: {error.msg}") from error
	actual = qt_source_closure(qt_source_root, sha256)
	if expected != actual:
		raise NativePublicationError("staged Qt source changed after its wheel input closure")
	return actual


#============================================
def validate_developer_pair(
		candidate_root: Path, native_wheel: Path, qt_wheel: Path, native_receipt: Path,
		engine_bundle: Path, qt_source_root: Path, qt_source_closure_path: Path,
		pair_receipt: Path, sha256: object,
		) -> None:
	"""Require a complete pair receipt and artifacts physically below one candidate."""
	for path, label in ((native_wheel, "native wheel"), (qt_wheel, "Qt wheel"),
			(native_receipt, "native receipt"), (engine_bundle, "engine bundle"),
			(pair_receipt, "pair receipt")):
		if path.is_symlink() or not path.exists() or not path.resolve().is_relative_to(candidate_root):
			raise NativePublicationError(f"developer publication {label} is not a regular candidate member")
	if not qt_wheel.name.startswith("ferrum_qt-") or qt_wheel.suffix != ".whl":
		raise NativePublicationError(f"developer publication has an invalid Qt wheel: {qt_wheel}")
	qt_closure = require_qt_source_closure(qt_source_root, qt_source_closure_path, sha256)
	expected = developer_pair_receipt(
		candidate_root, native_wheel, qt_wheel, native_receipt, engine_bundle, qt_closure, sha256
	)
	try:
		actual = json.loads(pair_receipt.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise NativePublicationError(f"developer pair receipt is invalid JSON: {error.msg}") from error
	if actual != expected:
		raise NativePublicationError("developer pair receipt does not match the complete candidate")


#============================================
def ferrum_source_closure(
		root: Path, schema: str, excluded_directories: tuple[str, ...], sha256: object,
		) -> dict[str, object]:
	"""Return the canonical staged Ferrum source-subset manifest."""
	root = root.resolve()
	if not root.is_dir():
		raise NativePublicationError(f"Ferrum source closure root is not a directory: {root}")
	excluded = frozenset(excluded_directories)
	files: list[dict[str, str]] = []
	for directory, names, filenames in os.walk(root):
		directory_path = Path(directory)
		accepted_names: list[str] = []
		for name in sorted(names):
			path = directory_path / name
			relative = path.relative_to(root).as_posix()
			if path.is_symlink():
				raise NativePublicationError(
					f"Ferrum source closure requires real directories: {path}"
				)
			if relative in excluded:
				continue
			accepted_names.append(name)
		names[:] = accepted_names
		for filename in sorted(filenames):
			path = directory_path / filename
			if not path.is_file() or path.is_symlink():
				raise NativePublicationError(f"Ferrum source closure requires regular files: {path}")
			files.append({"path": path.relative_to(root).as_posix(), "sha256": sha256(path)})
	payload = json.dumps({
		"excluded_directories": list(excluded_directories),
		"files": files,
		"schema": schema,
	}, separators=(",", ":"), sort_keys=True).encode("utf-8")
	return {
		"excluded_directories": list(excluded_directories),
		"files": files,
		"fingerprint_sha256": hashlib.sha256(payload).hexdigest(),
		"schema": schema,
	}


#============================================
def ferrum_worktree_source_closure(
		root: Path, schema: str, excluded_directories: tuple[str, ...],
		excluded_suffixes: tuple[str, ...], sha256: object,
		) -> dict[str, object]:
	"""Return exactly the regular workspace files admitted by the copy policy."""
	root = root.resolve()
	if not root.is_dir():
		raise NativePublicationError(f"Ferrum worktree source root is not a directory: {root}")
	excluded = frozenset(excluded_directories)
	files: list[dict[str, str]] = []
	for directory, names, filenames in os.walk(root):
		directory_path = Path(directory)
		accepted_names: list[str] = []
		for name in sorted(names):
			path = directory_path / name
			relative = path.relative_to(root).as_posix()
			if path.is_symlink():
				raise NativePublicationError(
					f"Ferrum worktree source closure requires real directories: {path}"
				)
			if name not in excluded and relative not in excluded:
				accepted_names.append(name)
		names[:] = accepted_names
		for filename in sorted(filenames):
			path = directory_path / filename
			if not path.is_file() or path.is_symlink():
				raise NativePublicationError(
					f"Ferrum worktree source closure requires regular files: {path}"
				)
			if path.suffix in excluded_suffixes:
				continue
			files.append({"path": path.relative_to(root).as_posix(), "sha256": sha256(path)})
	payload = json.dumps({
		"excluded_directories": list(excluded_directories),
		"excluded_suffixes": list(excluded_suffixes), "files": files, "schema": schema,
	}, separators=(",", ":"), sort_keys=True).encode("utf-8")
	return {
		"excluded_directories": list(excluded_directories),
		"excluded_suffixes": list(excluded_suffixes), "files": files,
		"fingerprint_sha256": hashlib.sha256(payload).hexdigest(), "schema": schema,
	}


#============================================
def require_matching_worktree_source_closure(
		expected: dict[str, object], actual: dict[str, object], label: str,
		) -> None:
	"""Reject input drift while reporting only a bounded set of relative paths."""
	if expected == actual:
		return
	expected_files = {item["path"]: item["sha256"] for item in expected.get("files", [])
			if isinstance(item, dict) and isinstance(item.get("path"), str)}
	actual_files = {item["path"]: item["sha256"] for item in actual.get("files", [])
			if isinstance(item, dict) and isinstance(item.get("path"), str)}
	changed = sorted(path for path in set(expected_files).union(actual_files)
		if expected_files.get(path) != actual_files.get(path))
	paths = ", ".join(changed[:3]) if changed else "manifest metadata"
	raise NativePublicationError(
		f"Ferrum worktree source changed {label}; changed paths: {paths}; "
		f"expected {expected.get('fingerprint_sha256')}, got {actual.get('fingerprint_sha256')}"
	)


#============================================
def validate_build_receipt(
		receipt: Path, wheel: Path, source_closure: dict[str, object],
		worktree_source_closure: dict[str, object], sha256: object,
		) -> None:
	"""Require the durable receipt to bind the canonical source and final wheel."""
	try:
		value = json.loads(receipt.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise NativePublicationError(f"native build receipt is invalid JSON: {error.msg}") from error
	if not isinstance(value, dict) or value.get("ferrum_source_closure") != source_closure:
		raise NativePublicationError("native build receipt lacks the admitted Ferrum source closure")
	receipt_worktree_source_closure = value.get("ferrum_worktree_source_closure")
	if receipt_worktree_source_closure != worktree_source_closure:
		if isinstance(receipt_worktree_source_closure, dict):
			require_matching_worktree_source_closure(
				receipt_worktree_source_closure, worktree_source_closure, "before publication",
			)
		raise NativePublicationError("native build receipt lacks the admitted Ferrum worktree source closure")
	wheel_record = value.get("wheel")
	if not isinstance(wheel_record, dict) or wheel_record != {
		"filename": wheel.name, "sha256": sha256(wheel),
	}:
		raise NativePublicationError("native build receipt does not match the final wheel")


#============================================
def validate_publication_candidate(
		receipt: Path, wheel: Path, staged_source_root: Path, schema: str,
		excluded_directories: tuple[str, ...], worktree_source_root: Path,
		worktree_schema: str, worktree_excluded_directories: tuple[str, ...],
		worktree_excluded_suffixes: tuple[str, ...], sha256: object,
		) -> None:
	"""Revalidate copied publication evidence against the completed staged workspace."""
	if receipt.is_symlink() or not receipt.is_file():
		raise NativePublicationError(f"native publication receipt is not a regular file: {receipt}")
	if wheel.is_symlink() or not wheel.is_file():
		raise NativePublicationError(f"native publication wheel is not a regular file: {wheel}")
	source_closure = ferrum_source_closure(
		staged_source_root, schema, excluded_directories, sha256
	)
	worktree_source_closure = ferrum_worktree_source_closure(
		worktree_source_root, worktree_schema, worktree_excluded_directories,
		worktree_excluded_suffixes, sha256,
	)
	validate_build_receipt(receipt, wheel, source_closure, worktree_source_closure, sha256)


#============================================
def publish_current_publication(
		candidate_root: Path, current_pointer: Path, receipt: Path, wheel: Path,
		staged_source_root: Path, schema: str, excluded_directories: tuple[str, ...],
		worktree_source_root: Path, worktree_schema: str,
		worktree_excluded_directories: tuple[str, ...],
		worktree_excluded_suffixes: tuple[str, ...], engine_bundle: Path,
		manifest_name: str, bundle_schema: str, bundle_target: str,
		adapter_abi_version: int, adapter_name: str, sha256: object,
		qt_wheel: Path | None = None, qt_source_root: Path | None = None,
		qt_source_closure_path: Path | None = None,
		pair_receipt: Path | None = None,
		) -> None:
	"""Validate one candidate then atomically select it as the current publication."""
	candidate_root = candidate_root.resolve(strict=True)
	publication_parent = current_pointer.parent.resolve(strict=True)
	if candidate_root.parent != publication_parent:
		raise NativePublicationError("native publication candidate is not a current-pointer sibling")
	candidate_name = candidate_root.name
	if not candidate_name.startswith(".native-publication-"):
		raise NativePublicationError(f"native publication candidate has an invalid name: {candidate_root}")
	for path, label in ((receipt, "receipt"), (wheel, "wheel"), (engine_bundle, "engine bundle")):
		if not path.resolve(strict=True).is_relative_to(candidate_root):
			raise NativePublicationError(f"native publication {label} is outside its candidate: {path}")
	validate_publication_candidate(
		receipt, wheel, staged_source_root, schema, excluded_directories, worktree_source_root,
		worktree_schema, worktree_excluded_directories, worktree_excluded_suffixes, sha256,
	)
	validate_engine_bundle(
		engine_bundle, manifest_name, bundle_schema, bundle_target, adapter_abi_version,
		adapter_name, sha256,
	)
	if any(value is not None for value in (qt_wheel, qt_source_root, qt_source_closure_path, pair_receipt)):
		if qt_wheel is None or qt_source_root is None or qt_source_closure_path is None or pair_receipt is None:
			raise NativePublicationError("developer pair publication requires Qt wheel, source root, closure, and receipt")
		validate_developer_pair(
			candidate_root, wheel, qt_wheel, receipt, engine_bundle, qt_source_root,
			qt_source_closure_path, pair_receipt, sha256
		)
	# The private stage prevents another cooperating build from substituting the source link
	# between its exact validation and os.replace().  Ordinary source edits are instead
	# represented by the closure validation immediately above.
	pointer_stage = Path(tempfile.mkdtemp(prefix=".native-pointer-stage-", dir=publication_parent))
	temporary_pointer = pointer_stage / "current"
	try:
		os.chmod(pointer_stage, stat.S_IRWXU)
		os.symlink(candidate_name, temporary_pointer)
		if not temporary_pointer.is_symlink() or os.readlink(temporary_pointer) != candidate_name:
			raise NativePublicationError("temporary current pointer does not select the validated candidate")
		try:
			os.replace(temporary_pointer, current_pointer)
		except OSError as error:
			raise NativePublicationError(
				f"could not atomically replace native current publication: {error}"
			) from error
		if not current_pointer.is_symlink() or os.readlink(current_pointer) != candidate_name:
			raise NativePublicationError("native current publication did not resolve to the validated candidate")
	finally:
		if temporary_pointer.is_symlink():
			temporary_pointer.unlink()
		pointer_stage.rmdir()


#============================================
def validate_engine_bundle(
		bundle: Path, manifest_name: str, schema: str, target: str, adapter_abi_version: int,
		adapter_name: str, sha256: object,
		) -> None:
	"""Revalidate one copied CLI engine bundle against its sealed manifest."""
	try:
		native_wheel_bundle.validate_engine_bundle(
			bundle, manifest_name, schema, target, adapter_abi_version, adapter_name, sha256
		)
	except native_wheel_bundle.NativeEngineBundleError as error:
		raise NativePublicationError(str(error)) from error


#============================================
def audit_wheel_closure(
		wheel: Path, output_root: Path, validate_wheel_members: object,
		safe_extract_zip_members: object, assert_clean_closure: object,
		error_type: type[Exception],
		) -> None:
	"""Inspect the packaged Mach-O files, not the source staging directory."""
	audit_root = output_root / "wheel-closure-audit"
	if audit_root.exists():
		raise error_type(f"refusing to overwrite wheel closure audit directory: {audit_root}")
	with zipfile.ZipFile(wheel) as contents:
		validate_wheel_members(contents.namelist())
		safe_extract_zip_members(contents, audit_root)
		package = audit_root
		extensions = sorted(package.glob("ferrum_chem*.so"))
		if not extensions:
			package = audit_root / "ferrum_chem"
			extensions = sorted(package.glob("ferrum_chem*.so"))
	if len(extensions) != 1:
		raise error_type(f"wheel must contain exactly one native extension, found {extensions}")
	package_libs = audit_root / ".dylibs"
	if not package_libs.is_dir():
		package_libs = package / ".dylibs"
	assert_clean_closure(extensions[0], package_libs)


#============================================
def build_engine_bundle(
		output_root: Path, adapter: Path, graphmol_library: Path, destination: Path,
		copy_and_rewrite_closure: object, assert_packaged_library_closure: object,
		manifest_name: str, schema: str, adapter_abi_version: int, adapter_name: str,
		sha256: object, error_type: type[Exception],
		) -> Path:
	"""Publish the rewritten native closure in Ferrum's CLI bundle layout."""
	root = output_root.resolve()
	destination = destination.resolve()
	if not destination.is_relative_to(root):
		raise error_type("--engine-bundle-dir must be beneath --output-root")
	if destination.exists():
		raise error_type(f"refusing to overwrite existing engine bundle: {destination}")
	destination.mkdir(parents=True)
	copy_and_rewrite_closure(adapter, graphmol_library, destination)
	assert_packaged_library_closure(destination)
	manifest = destination / manifest_name
	manifest.write_bytes(native_wheel_bundle.engine_bundle_manifest(
		sorted(destination.glob("*.dylib")), schema, adapter_abi_version, adapter_name, sha256
	))
	if not (destination / adapter_name).is_file():
		raise error_type(f"engine bundle lacks required adapter: {destination / adapter_name}")
	return destination


#============================================
def emit_artifact_result(action: str, artifact: Path, schema: str, error_type: type[Exception]) -> None:
	"""Emit the sole stdout record for a completed native publication action."""
	artifact = artifact.resolve()
	if not artifact.is_file():
		raise error_type(f"{action} did not produce an artifact: {artifact}")
	print(json.dumps({"schema": schema, "action": action, "artifact": str(artifact)}, sort_keys=True))


#============================================
def artifact_emitter(schema: str, error_type: type[Exception]) -> object:
	"""Bind publication-result constants into the builder's stable emitter shape."""
	def emit(action: str, artifact: Path) -> None:
		emit_artifact_result(action, artifact, schema, error_type)
	return emit
