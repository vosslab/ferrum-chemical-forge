"""Native-wheel publication evidence and sealed payload helpers."""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
import tempfile
import zipfile
from pathlib import Path

import wheel_lib.native_wheel_bundle as native_wheel_bundle


class NativePublicationError(ValueError):
	"""A native-wheel publication candidate violates its evidence contract."""


DEVELOPER_WHEEL_PUBLICATION_SCHEMA = "ferrum-developer-wheel-publication-v4"
QT_SOURCE_CLOSURE_SCHEMA = "ferrum-qt-source-closure-v2"
QT_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES = (
	"__pycache__", ".pytest_cache", "build", "ferrum_qt.egg-info",
)
QT_SOURCE_CLOSURE_EXCLUDED_SUFFIXES = (".pyc",)


#============================================
def qt_source_closure(root: Path, sha256: object) -> dict[str, object]:
	"""Return the safe regular-file inventory used for one staged Qt wheel."""
	return ferrum_worktree_source_closure(
		root, QT_SOURCE_CLOSURE_SCHEMA, QT_SOURCE_CLOSURE_EXCLUDED_DIRECTORIES,
		QT_SOURCE_CLOSURE_EXCLUDED_SUFFIXES, sha256,
	)


#============================================
def _closure_relative_path(value: object) -> Path:
	"""Return one safe relative closure path, rejecting forged manifest members."""
	if not isinstance(value, str):
		raise NativePublicationError("Qt source closure has a non-string path")
	path = Path(value)
	if path.is_absolute() or ".." in path.parts or path.as_posix() != value:
		raise NativePublicationError(f"Qt source closure has an unsafe path: {value!r}")
	return path


#============================================
def stage_qt_source_tree(
		worktree_root: Path, destination: Path, admission: dict[str, object], sha256: object,
		) -> dict[str, object]:
	"""Reconstruct fresh Qt staging from exactly one admitted regular-file inventory."""
	worktree_root = worktree_root.resolve(strict=True)
	if not worktree_root.is_dir():
		raise NativePublicationError(f"Qt worktree source root is not a directory: {worktree_root}")
	if destination.exists() or destination.is_symlink():
		raise NativePublicationError(f"Qt staging destination must be absent: {destination}")
	require_matching_worktree_source_closure(
		admission, qt_source_closure(worktree_root, sha256), "before staging Qt wheel",
	)
	files = admission.get("files")
	if not isinstance(files, list):
		raise NativePublicationError("Qt source closure lacks its file inventory")
	destination.mkdir(parents=True)
	seen: set[Path] = set()
	for entry in files:
		if not isinstance(entry, dict):
			raise NativePublicationError("Qt source closure has a non-object file record")
		relative = _closure_relative_path(entry.get("path"))
		if relative in seen:
			raise NativePublicationError(f"Qt source closure has a duplicate path: {relative}")
		seen.add(relative)
		source = worktree_root / relative
		if source.is_symlink() or not source.is_file():
			raise NativePublicationError(f"Qt source closure member is no longer a regular file: {relative}")
		target = destination / relative
		target.parent.mkdir(parents=True, exist_ok=True)
		shutil.copyfile(source, target)
	staged = qt_source_closure(destination, sha256)
	require_matching_worktree_source_closure(admission, staged, "while staging Qt wheel")
	return staged


#============================================
def qt_source_package_manifest(root: Path, sha256: object) -> dict[str, object]:
	"""Hash every admitted package payload file, including non-Python resources."""
	members = [
		{"path": item["path"], "sha256": item["sha256"]}
		for item in qt_source_closure(root, sha256).get("files", [])
		if isinstance(item, dict) and isinstance(item.get("path"), str)
		and isinstance(item.get("sha256"), str) and item["path"].startswith("ferrum_qt/")
	]
	if not members:
		raise NativePublicationError("Qt source closure has no ferrum_qt package payload")
	payload = json.dumps(members, separators=(",", ":"), sort_keys=True).encode("utf-8")
	return {"members": members, "fingerprint_sha256": hashlib.sha256(payload).hexdigest()}


#============================================
def qt_wheel_package_manifest(wheel: Path, sha256: object) -> dict[str, object]:
	"""Require a wheel to contain only Ferrum package payload and generated dist-info."""
	manifest = wheel_member_manifest(wheel, sha256)
	members: list[dict[str, str]] = []
	dist_info_roots: set[str] = set()
	for member in manifest["members"]:
		path = member["path"]
		if path.startswith("ferrum_qt/"):
			members.append(member)
			continue
		parts = Path(path).parts
		if len(parts) >= 2 and parts[0].startswith("ferrum_qt-") and parts[0].endswith(".dist-info"):
			dist_info_roots.add(parts[0])
			continue
		raise NativePublicationError(f"Qt wheel has an unapproved non-package member: {path}")
	if not members:
		raise NativePublicationError("Qt wheel has no ferrum_qt package payload")
	if len(dist_info_roots) != 1:
		raise NativePublicationError("Qt wheel must contain exactly one generated ferrum_qt dist-info tree")
	payload = json.dumps(members, separators=(",", ":"), sort_keys=True).encode("utf-8")
	return {"members": members, "fingerprint_sha256": hashlib.sha256(payload).hexdigest()}


#============================================
def validate_qt_wheel_package_payload(qt_source_root: Path, qt_wheel: Path, sha256: object) -> dict[str, object]:
	"""Require every published Qt package byte to equal an admitted staged source byte."""
	source_payload = qt_source_package_manifest(qt_source_root, sha256)
	wheel_payload = qt_wheel_package_manifest(qt_wheel, sha256)
	source_members = {item["path"]: item["sha256"] for item in source_payload["members"]}
	for member in wheel_payload["members"]:
		path = member["path"]
		if source_members.get(path) != member["sha256"]:
			raise NativePublicationError(
				f"Qt wheel package payload is not an admitted staged source member: {path}"
			)
	return wheel_payload


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
		engine_bundle: Path, qt_staged_source_closure: dict[str, object],
		qt_worktree_source_closure_admission: dict[str, object],
		qt_worktree_source_closure_final: dict[str, object], qt_delivered_package_payload: dict[str, object],
		sha256: object,
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
		"qt_staged_source_closure": qt_staged_source_closure,
		"qt_worktree_source_closure": {
			"admitted": qt_worktree_source_closure_admission,
			"final": qt_worktree_source_closure_final,
		},
		"qt_delivered_package_payload": qt_delivered_package_payload,
		"qt_wheel_members": wheel_member_manifest(qt_wheel, sha256),
	}


#============================================
def load_qt_source_closure(closure_path: Path) -> dict[str, object]:
	"""Load one canonical Qt source closure from a regular evidence file."""
	if closure_path.is_symlink() or not closure_path.is_file():
		raise NativePublicationError("Qt source closure is not a regular evidence file")
	try:
		expected = json.loads(closure_path.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise NativePublicationError(f"Qt source closure is invalid JSON: {error.msg}") from error
	if not isinstance(expected, dict):
		raise NativePublicationError("Qt source closure is not an object")
	return expected


#============================================
def require_qt_source_closure(
		qt_source_root: Path, closure_path: Path, sha256: object, label: str,
		) -> dict[str, object]:
	"""Require one source tree to match its recorded canonical Qt closure."""
	expected = load_qt_source_closure(closure_path)
	actual = qt_source_closure(qt_source_root, sha256)
	require_matching_worktree_source_closure(expected, actual, label)
	return actual


#============================================
def validate_developer_pair(
		candidate_root: Path, native_wheel: Path, qt_wheel: Path, native_receipt: Path,
		engine_bundle: Path, qt_source_root: Path, qt_source_closure_path: Path,
		qt_worktree_source_root: Path, qt_worktree_source_closure_path: Path,
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
	qt_staged_source_closure = require_qt_source_closure(
		qt_source_root, qt_source_closure_path, sha256, "after Qt wheel build",
	)
	qt_worktree_source_closure_admission = load_qt_source_closure(qt_worktree_source_closure_path)
	require_matching_worktree_source_closure(
		qt_worktree_source_closure_admission, qt_staged_source_closure, "while staging Qt wheel",
	)
	qt_worktree_source_closure_final = require_qt_source_closure(
		qt_worktree_source_root, qt_worktree_source_closure_path, sha256, "before publication",
	)
	qt_delivered_package_payload = validate_qt_wheel_package_payload(qt_source_root, qt_wheel, sha256)
	expected = developer_pair_receipt(
		candidate_root, native_wheel, qt_wheel, native_receipt, engine_bundle,
		qt_staged_source_closure, qt_worktree_source_closure_admission,
		qt_worktree_source_closure_final, qt_delivered_package_payload, sha256,
	)
	try:
		actual = json.loads(pair_receipt.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise NativePublicationError(f"developer pair receipt is invalid JSON: {error.msg}") from error
	if actual != expected:
		raise NativePublicationError("developer pair receipt does not match the complete candidate")


#============================================
def write_developer_pair_receipt(
		candidate_root: Path, native_wheel: Path, qt_wheel: Path, native_receipt: Path,
		engine_bundle: Path, qt_source_root: Path, qt_source_closure_path: Path,
		qt_worktree_source_root: Path, qt_worktree_source_closure_path: Path,
		pair_receipt: Path, sha256: object,
		) -> None:
	"""Seal the pair receipt from the final live Qt worktree observation."""
	if pair_receipt.is_symlink() or pair_receipt.exists():
		raise NativePublicationError("developer pair receipt must be a fresh regular candidate file")
	if not pair_receipt.parent.resolve(strict=True).is_relative_to(candidate_root):
		raise NativePublicationError("developer pair receipt is outside its candidate")
	qt_staged_source_closure = require_qt_source_closure(
		qt_source_root, qt_source_closure_path, sha256, "after Qt wheel build",
	)
	qt_worktree_source_closure_admission = load_qt_source_closure(qt_worktree_source_closure_path)
	require_matching_worktree_source_closure(
		qt_worktree_source_closure_admission, qt_staged_source_closure, "while staging Qt wheel",
	)
	qt_worktree_source_closure_final = require_qt_source_closure(
		qt_worktree_source_root, qt_worktree_source_closure_path, sha256, "before publication",
	)
	qt_delivered_package_payload = validate_qt_wheel_package_payload(qt_source_root, qt_wheel, sha256)
	pair_receipt.write_text(json.dumps(developer_pair_receipt(
		candidate_root, native_wheel, qt_wheel, native_receipt, engine_bundle,
		qt_staged_source_closure, qt_worktree_source_closure_admission,
		qt_worktree_source_closure_final, qt_delivered_package_payload, sha256,
	), sort_keys=True, separators=(",", ":")), encoding="utf-8")


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
		adapter_abi_version: int, adapter_name: str, sha256: object, qt_wheel: Path,
		qt_source_root: Path, qt_source_closure_path: Path, qt_worktree_source_root: Path,
		qt_worktree_source_closure_path: Path, pair_receipt: Path,
		) -> None:
	"""Validate one candidate then atomically select it as the current publication."""
	candidate_root = candidate_root.resolve(strict=True)
	publication_parent = current_pointer.parent.resolve(strict=True)
	if candidate_root.parent != publication_parent:
		raise NativePublicationError("native publication candidate is not a current-pointer sibling")
	if any(value is None for value in (
			qt_wheel, qt_source_root, qt_source_closure_path, qt_worktree_source_root,
			qt_worktree_source_closure_path, pair_receipt,
		)):
		raise NativePublicationError("developer pair publication requires every Qt evidence input")
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
	validate_wheel_engine_bundle(
		wheel, engine_bundle, manifest_name, bundle_schema, bundle_target,
		adapter_abi_version, adapter_name, sha256,
	)
	write_developer_pair_receipt(
		candidate_root, wheel, qt_wheel, receipt, engine_bundle, qt_source_root,
		qt_source_closure_path, qt_worktree_source_root,
		qt_worktree_source_closure_path, pair_receipt, sha256,
	)
	validate_developer_pair(
		candidate_root, wheel, qt_wheel, receipt, engine_bundle, qt_source_root,
		qt_source_closure_path, qt_worktree_source_root,
		qt_worktree_source_closure_path, pair_receipt, sha256,
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
def validate_wheel_engine_bundle(
		wheel: Path, bundle: Path, manifest_name: str, schema: str, target: str,
		adapter_abi_version: int, adapter_name: str, sha256: object,
		) -> None:
	"""Require the installed-wheel bundle payload to equal its CLI bundle byte-for-byte."""
	validate_engine_bundle(
		bundle, manifest_name, schema, target, adapter_abi_version, adapter_name, sha256,
	)
	expected = {f"ferrum-engine-bundle/{path.name}": path for path in bundle.iterdir()}
	try:
		with zipfile.ZipFile(wheel) as archive:
			actual = {
				info.filename: info for info in archive.infolist()
				if info.filename.startswith("ferrum-engine-bundle/") and not info.is_dir()
			}
			if set(actual) != set(expected):
				raise NativePublicationError("wheel sealed engine bundle members differ from its CLI bundle")
			for name, source in expected.items():
				info = actual[name]
				if stat.S_IFMT(info.external_attr >> 16) == stat.S_IFLNK:
					raise NativePublicationError(f"wheel sealed engine bundle member is a symbolic link: {name}")
				if archive.read(info) != source.read_bytes():
					raise NativePublicationError(f"wheel sealed engine bundle member differs from its CLI bundle: {name}")
	except zipfile.BadZipFile as error:
		raise NativePublicationError(f"wheel is not a readable archive: {wheel}") from error


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
