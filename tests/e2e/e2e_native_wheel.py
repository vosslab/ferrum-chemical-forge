#!/usr/bin/env python3
"""Prove the M4a macOS arm64 wheel survives a clean install and ABI relink."""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_TOOL = REPO_ROOT / "packages/ferrum-rust/tools/build_native_wheel.py"
DEFAULT_TARGET = "aarch64-apple-darwin"
BUILD_RESULT_SCHEMA = "ferrum-m4a-artifact-v1"
AMBIENT_LIBRARY_VARIABLES = (
	"DYLD_LIBRARY_PATH",
	"DYLD_FALLBACK_LIBRARY_PATH",
	"DYLD_FRAMEWORK_PATH",
	"DYLD_FALLBACK_FRAMEWORK_PATH",
	"PYTHONPATH",
	"PYTHONHOME",
)


class E2eError(RuntimeError):
	"""An actionable M4a proof failure."""


def run(*command: str, cwd: Path | None = None, env: dict[str, str] | None = None) -> str:
	print("+", " ".join(command), file=sys.stderr)
	child_environment = scrubbed_environment() if env is None else env.copy()
	child_environment["PYTHONDONTWRITEBYTECODE"] = "1"
	result = subprocess.run(command, cwd=cwd, env=child_environment, text=True, capture_output=True, check=False)
	if result.returncode:
		raise E2eError(f"command failed ({result.returncode}): {' '.join(command)}\n{result.stderr.strip()}")
	return result.stdout


def scrubbed_environment() -> dict[str, str]:
	environment = os.environ.copy()
	for name in AMBIENT_LIBRARY_VARIABLES:
		environment.pop(name, None)
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	return environment


def probe(python: Path) -> dict[str, object]:
	output = run(
		str(python), "-I", "-c",
		"import json, ferrum_api; print(json.dumps(ferrum_api.probe()))",
		env=scrubbed_environment(),
	)
	value = json.loads(output)
	if not (isinstance(value, list) and len(value) == 2 and isinstance(value[0], int) and isinstance(value[1], str)):
		raise E2eError(f"native probe returned an invalid value: {value!r}")
	return {"abi_version": value[0], "marker": value[1]}


def load_build_tool():
	# The tracked builder is dynamically imported for closure validation.  Keep
	# that read-only operation from generating a source-tree __pycache__ even if
	# the caller forgot the environment setting.
	sys.dont_write_bytecode = True
	specification = importlib.util.spec_from_file_location("ferrum_native_wheel_build", BUILD_TOOL)
	if specification is None or specification.loader is None:
		raise E2eError(f"could not load native wheel closure policy from {BUILD_TOOL}")
	module = importlib.util.module_from_spec(specification)
	sys.modules[specification.name] = module
	specification.loader.exec_module(module)
	return module


def parse_artifact_result(stdout: str, action: str, output_root: Path) -> Path:
	"""Accept exactly one builder JSON artifact record and no progress on stdout."""
	lines = stdout.splitlines()
	if len(lines) != 1:
		raise E2eError(f"builder {action} result must be exactly one JSON line, got {len(lines)} lines")
	try:
		result = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise E2eError(f"builder {action} result is not valid JSON: {error.msg}") from error
	if not isinstance(result, dict) or set(result) != {"schema", "action", "artifact"}:
		raise E2eError(f"builder {action} result has an unexpected schema: {result!r}")
	if result["schema"] != BUILD_RESULT_SCHEMA or result["action"] != action:
		raise E2eError(f"builder {action} result has the wrong schema or action: {result!r}")
	if not isinstance(result["artifact"], str):
		raise E2eError(f"builder {action} artifact is not a path string: {result!r}")
	artifact = Path(result["artifact"])
	if not artifact.is_absolute() or artifact != artifact.resolve():
		raise E2eError(f"builder {action} artifact must be an absolute normalized path: {artifact}")
	if not artifact.is_relative_to(output_root.resolve()) or not artifact.is_file():
		raise E2eError(f"builder {action} reported a missing or out-of-root artifact: {artifact}")
	return artifact


def command_self_test() -> None:
	"""Prove logs on stdout cannot be mistaken for the artifact result."""
	artifact = BUILD_TOOL.resolve()
	root = REPO_ROOT.resolve()
	valid = json.dumps({"schema": BUILD_RESULT_SCHEMA, "action": "wheel", "artifact": str(artifact)})
	if parse_artifact_result(valid, "wheel", root) != artifact:
		raise E2eError("builder result parser self-test did not preserve a valid artifact")
	for output in (
		f"maturin progress\\n{valid}",
		f"{valid}\\n{valid}",
		json.dumps({"schema": "wrong", "action": "wheel", "artifact": str(artifact)}),
		json.dumps({"schema": BUILD_RESULT_SCHEMA, "action": "wheel", "artifact": "relative.whl"}),
	):
		try:
			parse_artifact_result(output, "wheel", root)
		except E2eError:
			pass
		else:
			raise E2eError("builder result parser self-test accepted noisy or invalid stdout")


def assert_wheel_closure(site_packages: Path) -> Path:
	package = site_packages / "ferrum_api"
	extensions = list(package.glob("_native*.so"))
	if len(extensions) != 1:
		raise E2eError(f"expected one native extension in {package}, found {extensions}")
	libs = package / ".libs"
	if not (libs / "libferrum_chem.dylib").is_file():
		raise E2eError("wheel does not contain separately replaceable .libs/libferrum_chem.dylib")
	try:
		load_build_tool().assert_clean_closure(extensions[0], libs)
	except RuntimeError as error:
		raise E2eError(f"installed wheel fails the native loader closure policy: {error}") from error
	return libs


def installed_site_packages(venv: Path) -> Path:
	paths = list((venv / "lib").glob("python*/site-packages"))
	if len(paths) != 1:
		raise E2eError(f"could not identify isolated venv site-packages under {venv}")
	return paths[0]


def main() -> int:
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--target", default=DEFAULT_TARGET)
	parser.add_argument("--rdkit-archive", help="optional pinned archive; its digest is rechecked by the build tool")
	parser.add_argument("--self-test", action="store_true", help="validate the builder stdout result parser without building")
	arguments = parser.parse_args()
	if arguments.self_test:
		command_self_test()
		print(json.dumps({"schema": "ferrum-m4a-e2e-self-test-v1", "status": "ok"}, sort_keys=True))
		return 0
	if arguments.target != DEFAULT_TARGET:
		raise E2eError(f"M4a supports only {DEFAULT_TARGET}, not {arguments.target}")
	output_parent = REPO_ROOT / "output_m4a"
	output_parent.mkdir(exist_ok=True)
	with tempfile.TemporaryDirectory(prefix="e2e-native-wheel-", dir=output_parent) as temporary:
		output_root = Path(temporary)
		build_command = [sys.executable, str(BUILD_TOOL), "build", "--output-root", str(output_root), "--target", arguments.target]
		if arguments.rdkit_archive:
			build_command.extend(("--rdkit-archive", arguments.rdkit_archive))
		wheel = parse_artifact_result(run(*build_command, env=scrubbed_environment()), "wheel", output_root)
		venv = output_root / "clean-venv"
		run(sys.executable, "-m", "venv", str(venv))
		python = venv / "bin" / "python"
		run(str(python), "-m", "pip", "install", "--no-deps", str(wheel), env=scrubbed_environment())
		before = probe(python)
		if before != {"abi_version": 1, "marker": "wheel"}:
			raise E2eError(f"initial isolated probe was not the wheel ABI: {before}")
		libs = assert_wheel_closure(installed_site_packages(venv))
		replacement_root = output_root / "replacement-output"
		replacement = parse_artifact_result(run(
			sys.executable, str(BUILD_TOOL), "adapter", "--output-root", str(replacement_root), "--marker", "replacement",
			env=scrubbed_environment(),
		), "adapter", replacement_root)
		shutil.copy2(replacement, libs / "libferrum_chem.dylib")
		after = probe(python)
		if after != {"abi_version": 1, "marker": "replacement"}:
			raise E2eError(f"replaced library was not loaded in a fresh process: {after}")
		print(json.dumps({"wheel": wheel.name, "before": before, "after": after}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except E2eError as error:
		print(f"M4a E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
