"""Stage the PyO3 wheel project and locate its dedicated packager."""

from __future__ import annotations

import base64
import hashlib
import os
import re
import shutil
import sysconfig
import tarfile
import zipfile
from pathlib import Path

from native_wheel_profile import FERRUM_RDKIT_PROFILE, PinnedSource


class NativePackagingError(RuntimeError):
	"""The isolated wheel-staging boundary could not be prepared."""


NOTICE_FILENAMES = (
	"FERRUM-CHEM-LGPL-3.0.txt",
	"RDKIT-BSD-3-CLAUSE.txt",
	"INCHI-MIT.txt",
	"TELEX-OFL-1.1.txt",
	"THIRD_PARTY_NOTICES.md",
)
INCHI_LICENSE_MEMBER = "INCHI-1-SRC/INCHI_API/libinchi/src/inchi_dll.c"


#============================================
def find_maturin() -> str:
	"""Resolve Maturin from the required Python interpreter, never ambient PATH."""
	scripts = Path(sysconfig.get_path("scripts")).resolve()
	command = (scripts / "maturin").resolve()
	if not command.is_file() or not os.access(command, os.X_OK):
		raise NativePackagingError(
			f"maturin is required in the Python 3.12 scripts directory: {scripts}"
		)
	return str(command)


#============================================
def tool_version(command: str) -> str:
	"""Return a native packager's declared version string."""
	import subprocess

	result = subprocess.run([command, "--version"], text=True, capture_output=True, check=False)
	if result.returncode:
		raise NativePackagingError(
			f"could not determine tool version for {command}: {result.stderr.strip()}"
		)
	return result.stdout.strip()


#============================================
def _source_archive(native_input_root: Path, source: PinnedSource) -> Path:
	"""Return one hash-verified archive already admitted as a native input."""
	archive = native_input_root / "downloads" / source.archive_filename
	if not archive.is_file():
		raise NativePackagingError(f"missing pinned {source.name} source archive: {archive}")
	actual = hashlib.sha256(archive.read_bytes()).hexdigest()
	if actual != source.sha256:
		raise NativePackagingError(
			f"pinned {source.name} source archive hash mismatch: "
			f"expected {source.sha256}, got {actual}"
		)
	return archive


#============================================
def _archive_member(archive: Path, member: str) -> bytes:
	"""Read one exact regular-file member without extracting an upstream archive."""
	try:
		if archive.suffix == ".zip":
			with zipfile.ZipFile(archive) as contents:
				return contents.read(member)
		with tarfile.open(archive) as contents:
			info = contents.getmember(member)
			if not info.isfile():
				raise NativePackagingError(f"pinned archive member is not a file: {member}")
			stream = contents.extractfile(info)
			if stream is None:
				raise NativePackagingError(f"cannot read pinned archive member: {member}")
			return stream.read()
	except (KeyError, tarfile.TarError, zipfile.BadZipFile) as error:
		raise NativePackagingError(f"pinned archive lacks required member: {member}") from error


#============================================
def _inchi_mit_notice(source: bytes) -> bytes:
	"""Return the exact leading InChI license comment after semantic verification."""
	end = source.find(b"*/")
	if end < 0:
		raise NativePackagingError("InChI license source lacks its leading comment delimiter")
	notice = source[:end + 2]
	markers = (
		b"International Chemical Identifier (InChI)",
		b"Software version 1.07",
		b"MIT License",
		b"Copyright (c) 2024 IUPAC and InChI Trust",
		b"Permission is hereby granted, free of charge",
		b"copies or substantial portions of the Software.",
		b"THE SOFTWARE IS PROVIDED \"AS IS\"",
		b"info@inchi-trust.org",
	)
	if not all(marker in notice for marker in markers):
		raise NativePackagingError("InChI license source does not contain the expected MIT notice")
	return notice


#============================================
def stage_native_notice_bundle(
	project: Path, package_source: Path, native_input_root: Path
) -> Path:
	"""Stage source-backed native-wheel notices beside authored wheel metadata."""
	metadata = project.parent / "wheel_metadata"
	notices = metadata / "licenses"
	if notices.exists():
		raise NativePackagingError(f"refusing to overwrite staged native notices: {notices}")
	repo_root = package_source.resolve().parents[1]
	lgpl = repo_root / "LICENSE.LGPL-3.0.md"
	telex = package_source / "crates" / "render" / "assets" / "licenses" / "Telex-OFL-1.1.txt"
	index = metadata / "THIRD_PARTY_NOTICES.md"
	for source in (lgpl, telex, index):
		if not source.is_file():
			raise NativePackagingError(f"missing authored wheel notice source: {source}")
	rdkit = _archive_member(
		_source_archive(native_input_root, FERRUM_RDKIT_PROFILE.rdkit),
		"rdkit-Release_2026_03_5/license.txt",
	)
	inchi_source = next(
		(source for source in FERRUM_RDKIT_PROFILE.dependencies if source.name == "inchi-source"),
		None,
	)
	if inchi_source is None:
		raise NativePackagingError("native profile lacks its pinned InChI source")
	inchi = _inchi_mit_notice(
		_archive_member(_source_archive(native_input_root, inchi_source), INCHI_LICENSE_MEMBER)
	)
	notices.mkdir()
	contents = {
		"FERRUM-CHEM-LGPL-3.0.txt": lgpl.read_bytes(),
		"RDKIT-BSD-3-CLAUSE.txt": rdkit,
		"INCHI-MIT.txt": inchi,
		"TELEX-OFL-1.1.txt": telex.read_bytes(),
		"THIRD_PARTY_NOTICES.md": index.read_bytes(),
	}
	for name in NOTICE_FILENAMES:
		notices.joinpath(name).write_bytes(contents[name])
	return notices


#============================================
def stage_python_project(output_root: Path, package_source: Path) -> Path:
	"""Copy the Rust workspace and configure its generated native-wheel layout."""
	stage = output_root.resolve() / "maturin-project"
	if stage.exists():
		raise NativePackagingError(f"refusing to overwrite staged maturin project: {stage}")
	shutil.copytree(
		package_source,
		stage,
		ignore=shutil.ignore_patterns(".libs", "target", "__pycache__", "*.pyc"),
	)
	project = stage / "crates" / "api" / "python"
	pyproject = project / "pyproject.toml"
	contents = pyproject.read_text(encoding="utf-8")
	if "include =" in contents:
		raise NativePackagingError("source PyO3 project must reserve wheel contents for the shipping builder")
	pyproject.write_text(contents + '\ninclude = [".dylibs/*"]\n', encoding="utf-8")
	# Root typing metadata lives outside Maturin's project discovery path. The
	# shipping rewriter installs it after Maturin has emitted its intermediate.
	metadata = project.parent / "wheel_metadata"
	for name in ("ferrum_chem.pyi", "py.typed"):
		if not (metadata / name).is_file():
			raise NativePackagingError(f"missing authored wheel metadata: {metadata / name}")
	schema = project.parent / "protocol" / "ferrum-operation-v1.schema.json"
	if not schema.is_file():
		raise NativePackagingError(f"missing generated operation protocol schema: {schema}")
	build_script = project / "build.rs"
	build_contents = build_script.read_text(encoding="utf-8")
	needle = "pyo3_build_config::add_extension_module_link_args();"
	if needle not in build_contents:
		raise NativePackagingError("staged PyO3 build script lacks its extension-linking hook")
	build_script.write_text(
		build_contents.replace(
			needle,
			needle + '\n    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path/.dylibs");',
		),
		encoding="utf-8",
	)
	return project


#============================================
def _metadata_with_license_files(metadata: bytes, license_paths: tuple[str, ...]) -> bytes:
	"""Add PEP 639 license-file headers without replacing existing metadata."""
	try:
		text = metadata.decode("utf-8")
	except UnicodeDecodeError as error:
		raise NativePackagingError("wheel METADATA is not UTF-8") from error
	headers, separator, body = text.partition("\n\n")
	if not separator:
		raise NativePackagingError("wheel METADATA lacks a header/body separator")
	if any(line.startswith("License-File:") for line in headers.splitlines()):
		raise NativePackagingError("wheel METADATA already declares license files")
	license_headers = "\n".join(f"License-File: {path}" for path in license_paths)
	return f"{headers}\n{license_headers}{separator}{body}".encode("utf-8")


#============================================
def inject_root_metadata(wheel: Path, project: Path) -> None:
	"""Add metadata, notices, and a regenerated canonical wheel RECORD."""
	metadata = project.parent / "wheel_metadata"
	notices = metadata / "licenses"
	for name in NOTICE_FILENAMES:
		if not (notices / name).is_file():
			raise NativePackagingError(f"missing staged native-wheel notice: {notices / name}")
	entries: dict[str, bytes] = {}
	with zipfile.ZipFile(wheel) as archive:
		for info in archive.infolist():
			if info.is_dir() or info.filename.endswith(".dist-info/RECORD"):
				continue
			if info.filename == "ferrum_chem/__init__.py":
				continue
			if re.fullmatch(r"ferrum_chem/ferrum_chem[^/]*\.so", info.filename):
				name = info.filename.removeprefix("ferrum_chem/")
			elif info.filename.startswith("ferrum_chem/"):
				raise NativePackagingError("Maturin emitted unexpected nested ferrum_chem content")
			else:
				name = info.filename
			if name in entries:
				raise NativePackagingError(f"Maturin emitted duplicate wheel member: {name}")
			entries[name] = archive.read(info.filename)
	entries["ferrum_chem.pyi"] = (metadata / "ferrum_chem.pyi").read_bytes()
	entries["py.typed"] = (metadata / "py.typed").read_bytes()
	entries["ferrum-operation-v1.schema.json"] = (
		project.parent / "protocol" / "ferrum-operation-v1.schema.json"
	).read_bytes()
	if len([name for name in entries if name.startswith("ferrum_chem") and name.endswith(".so")]) != 1:
		raise NativePackagingError("wheel does not contain exactly one root ferrum_chem extension")
	wheel_metadata = [name for name in entries if name.endswith(".dist-info/WHEEL")]
	if len(wheel_metadata) != 1:
		raise NativePackagingError("wheel must contain one dist-info WHEEL metadata member")
	dist_info = wheel_metadata[0].removesuffix("/WHEEL")
	metadata_path = f"{dist_info}/METADATA"
	if metadata_path not in entries:
		raise NativePackagingError("wheel lacks its dist-info METADATA member")
	license_paths = tuple(f"licenses/{name}" for name in NOTICE_FILENAMES)
	entries[metadata_path] = _metadata_with_license_files(entries[metadata_path], license_paths)
	for name in NOTICE_FILENAMES:
		entries[f"{dist_info}/licenses/{name}"] = (notices / name).read_bytes()
	record = wheel_metadata[0].replace("WHEEL", "RECORD")
	lines = []
	for name, content in sorted(entries.items()):
		digest = base64.urlsafe_b64encode(hashlib.sha256(content).digest()).rstrip(b"=").decode()
		lines.append(f"{name},sha256={digest},{len(content)}")
	lines.append(f"{record},,")
	entries[record] = ("\n".join(lines) + "\n").encode()
	temporary = wheel.with_name(f".{wheel.name}.rewrite")
	with zipfile.ZipFile(temporary, "w", compression=zipfile.ZIP_DEFLATED) as archive:
		for name, content in sorted(entries.items()):
			archive.writestr(name, content)
	temporary.replace(wheel)
