"""Stage the PyO3 wheel project and locate its dedicated packager."""

from __future__ import annotations

import base64
import hashlib
import os
import re
import shutil
import sysconfig
import zipfile
from pathlib import Path


class NativePackagingError(RuntimeError):
	"""The isolated wheel-staging boundary could not be prepared."""


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
def inject_root_metadata(wheel: Path, project: Path) -> None:
	"""Add root typing metadata and regenerate the wheel's canonical RECORD."""
	metadata = project.parent / "wheel_metadata"
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
	if len([name for name in entries if name.startswith("ferrum_chem") and name.endswith(".so")]) != 1:
		raise NativePackagingError("wheel does not contain exactly one root ferrum_chem extension")
	record = next(name for name in entries if name.endswith(".dist-info/WHEEL")).replace("WHEEL", "RECORD")
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
