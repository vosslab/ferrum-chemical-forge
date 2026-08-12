"""Prove a macOS arm64 native wheel survives a clean install and ABI relink."""

from __future__ import annotations

# Standard-library imports.
import argparse
import hashlib
import importlib.util
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType


# Local builder imports must remain runtime-only so this direct E2E runner
# cannot create source-tree bytecode merely by being imported or inspected.
REPO_ROOT = Path(__file__).resolve().parents[2]
BUILD_TOOL = REPO_ROOT / "packages/ferrum-rust/tools/build_native_wheel.py"
CHEMISTRY_MANIFEST = REPO_ROOT / "packages/ferrum-rust/crates/chemistry/Cargo.toml"
CHEMISTRY_EXAMPLE = "native_kekulize"
AMBIENT_LIBRARY_VARIABLES = (
	"DYLD_LIBRARY_PATH",
	"DYLD_FALLBACK_LIBRARY_PATH",
	"DYLD_FRAMEWORK_PATH",
	"DYLD_FALLBACK_FRAMEWORK_PATH",
	"PYTHONPATH",
	"PYTHONHOME",
)


#============================================
# Build-contract loading


@dataclass(frozen=True)
class BuildContract:
	"""Machine protocol values exported by the current native build tool."""

	target: str
	result_schema: str
	adapter_abi_version: int


class E2eError(RuntimeError):
	"""An actionable native-wheel proof failure."""


#============================================
# Evidence helpers


#============================================
def sha256(path: Path) -> str:
	digest = hashlib.sha256()
	with path.open("rb") as handle:
		for block in iter(lambda: handle.read(1024 * 1024), b""):
			digest.update(block)
	return digest.hexdigest()


#============================================
def read_build_receipt(output_root: Path) -> dict[str, object]:
	path = output_root / "native-wheel-build-receipt.json"
	if not path.is_file():
		raise E2eError(f"native wheel builder did not publish its receipt: {path}")
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise E2eError(f"native wheel builder receipt is invalid JSON: {error.msg}") from error
	if not isinstance(value, dict):
		raise E2eError("native wheel builder receipt must be a JSON object")
	return value


#============================================
def replacement_proof(
	original_sha256: str, replacement_sha256: str, installed_adapter: Path,
	expected_abi_version: int, replacement_build_type: str,
) -> dict[str, object]:
	"""Validate durable facts that establish the adapter replacement target."""
	for label, digest in (
		("original adapter", original_sha256),
		("replacement adapter", replacement_sha256),
	):
		if not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest):
			raise E2eError(f"{label} SHA-256 is not a lowercase 64-hex digest: {digest!r}")
	if original_sha256 == replacement_sha256:
		raise E2eError(
			"the deliberately different replacement build produced the original adapter bytes"
		)
	if replacement_build_type != "RelWithDebInfo":
		raise E2eError(f"unexpected replacement build type: {replacement_build_type}")
	if installed_adapter.name != "libferrum_chem.dylib":
		raise E2eError(f"replacement target must be the installed adapter library: {installed_adapter}")
	return {
		"library": "libferrum_chem.dylib",
		"package_relative_path": "ferrum_api/.libs/libferrum_chem.dylib",
		"abi_version": expected_abi_version,
		"original_sha256": original_sha256,
		"replacement_sha256": replacement_sha256,
		"replacement_build_type": replacement_build_type,
	}


#============================================
def native_evidence(
	builder_receipt: dict[str, object], wheel: Path, closure_names: list[str],
	before: dict[str, object], after: dict[str, object], original_sha256: str,
	replacement_sha256: str, installed_adapter: Path,
	expected_abi_version: int, chemistry_before: dict[str, object],
	chemistry_after: dict[str, object], replacement_build_type: str,
) -> dict[str, object]:
	"""Assemble the retained reproducibility record without retaining binaries."""
	return {
		"schema": "ferrum-native-wheel-e2e-evidence-v2",
		"builder_receipt": builder_receipt,
		"wheel": {"filename": wheel.name, "sha256": sha256(wheel)},
		"closure": {"names": closure_names},
		"probes": {
			"expected_abi_version": expected_abi_version,
			"before": before,
			"after": after,
		},
		"chemistry": {
			"before": chemistry_before,
			"after": chemistry_after,
		},
		"replacement_proof": replacement_proof(
			original_sha256, replacement_sha256, installed_adapter, expected_abi_version,
			replacement_build_type,
		),
		"process_boundary": {
			"python_probe": {"fresh_process": True, "scrubbed_loader": True},
			"rust_semantic_probe": {
				"fresh_process": True,
				"explicit_adapter_path": True,
				"scrubbed_loader": True,
			},
		},
	}


#============================================
def publish_evidence(output_parent: Path, evidence: dict[str, object]) -> Path:
	"""Atomically replace the stable, ignored acceptance record after success."""
	directory = output_parent / "evidence"
	directory.mkdir(parents=True, exist_ok=True)
	target = directory / "native-wheel-e2e-receipt.json"
	with tempfile.NamedTemporaryFile(
		mode="w", encoding="utf-8", dir=directory,
		prefix=".native-wheel-e2e-receipt-", suffix=".json", delete=False,
	) as handle:
		handle.write(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
		temporary = Path(handle.name)
	temporary.replace(target)
	return target


#============================================
def run(
	*command: str,
	cwd: Path | None = None,
	env: dict[str, str] | None = None,
	stream_stderr: bool = False,
) -> str:
	"""Run a child while preserving its stdout machine protocol for this runner."""
	print("+", " ".join(command), file=sys.stderr)
	child_environment = scrubbed_environment() if env is None else env.copy()
	child_environment["PYTHONDONTWRITEBYTECODE"] = "1"
	result = subprocess.run(
		command,
		cwd=cwd,
		env=child_environment,
		text=True,
		stdout=subprocess.PIPE,
		stderr=None if stream_stderr else subprocess.PIPE,
		check=False,
	)
	if result.returncode:
		if stream_stderr:
			details = "child stderr was streamed above"
		else:
			details = result.stderr.strip()
		raise E2eError(
			f"command failed ({result.returncode}): {' '.join(command)}\n{details}"
		)
	return result.stdout


#============================================
def scrubbed_environment() -> dict[str, str]:
	environment = os.environ.copy()
	for name in AMBIENT_LIBRARY_VARIABLES:
		environment.pop(name, None)
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	return environment


#============================================
def probe(python: Path) -> dict[str, object]:
	output = run(
		str(python), "-I", "-c",
		"import json, ferrum_api._native; "
		"print(json.dumps({'abi_version': ferrum_api._native.probe()}))",
		env=scrubbed_environment(),
	)
	value = json.loads(output)
	if not isinstance(value, dict) or type(value.get("abi_version")) is not int:
		raise E2eError(f"native probe returned an invalid value: {value!r}")
	return {"abi_version": value["abi_version"]}


#============================================
def parse_json_object(output: str, name: str) -> dict[str, object]:
	"""Accept the one-object machine protocol emitted by a semantic probe."""
	lines = output.splitlines()
	if len(lines) != 1:
		raise E2eError(f"{name} must emit exactly one JSON object, got {len(lines)} lines")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise E2eError(f"{name} did not emit valid JSON: {error.msg}") from error
	if not isinstance(value, dict):
		raise E2eError(f"{name} must emit a JSON object: {value!r}")
	return value


#============================================
def expected_semantic_atoms() -> list[dict[str, object]]:
	"""Return the authored facts the native engine must preserve exactly."""
	return [
		{
			"atomic_number": 6,
			"aromatic": True,
			"formal_charge": 0,
			"isotope": None,
			"explicit_hydrogens": None,
		},
		{
			"atomic_number": 6,
			"aromatic": True,
			"formal_charge": None,
			"isotope": 13,
			"explicit_hydrogens": None,
		},
		{
			"atomic_number": 6,
			"aromatic": True,
			"formal_charge": None,
			"isotope": None,
			"explicit_hydrogens": 1,
		},
		*[
			{
				"atomic_number": 6,
				"aromatic": True,
				"formal_charge": None,
				"isotope": None,
				"explicit_hydrogens": None,
			}
			for _ in range(3)
		],
	]


#============================================
def expected_semantic_bonds(order_names: list[str]) -> list[dict[str, object]]:
	"""Return ordered benzene bonds for the Rust semantic-proof protocol."""
	return [
		{
			"start": index,
			"end": (index + 1) % 6,
			"order": order,
			"aromatic": True,
		}
		for index, order in enumerate(order_names)
	]


#============================================
def semantic_probe_fixture(abi_version: int) -> dict[str, object]:
	"""Build a pure-Python fixture for the semantic-protocol self-test."""
	return {
		"abi_version": abi_version,
		"input": {
			"atoms": expected_semantic_atoms(),
			"bonds": expected_semantic_bonds(["aromatic"] * 6),
		},
		"output": {
			"atoms": expected_semantic_atoms(),
			"bonds": expected_semantic_bonds(["single", "double"] * 3),
		},
	}


#============================================
def assert_semantic_probe(value: dict[str, object], expected_abi_version: int) -> None:
	"""Prove default Rust-engine Kekulization rather than only loader identity."""
	if value.get("abi_version") != expected_abi_version:
		raise E2eError(f"semantic probe reported wrong ABI: {value!r}")
	input_graph = value.get("input")
	output_graph = value.get("output")
	if not isinstance(input_graph, dict) or not isinstance(output_graph, dict):
		raise E2eError(f"semantic probe omitted input or output graphs: {value!r}")
	expected_atoms = expected_semantic_atoms()
	if input_graph.get("atoms") != expected_atoms or output_graph.get("atoms") != expected_atoms:
		raise E2eError(
			"semantic probe did not preserve exact optional charge, isotope, and hydrogen facts"
		)
	input_bonds = input_graph.get("bonds")
	output_bonds = output_graph.get("bonds")
	if not isinstance(input_bonds, list) or not isinstance(output_bonds, list):
		raise E2eError("semantic probe graphs must contain bond arrays")
	if len(input_bonds) != 6 or len(output_bonds) != 6:
		raise E2eError("semantic probe did not retain benzene topology")
	for index, bond in enumerate(input_bonds):
		if bond != expected_semantic_bonds(["aromatic"] * 6)[index]:
			raise E2eError(f"semantic probe input bond {index} is not authored aromatic benzene")
	orders: list[str] = []
	for index, bond in enumerate(output_bonds):
		if not isinstance(bond, dict):
			raise E2eError(f"semantic probe output bond {index} is not an object")
		if bond.get("start") != index or bond.get("end") != (index + 1) % 6:
			raise E2eError(f"semantic probe changed output bond endpoints at index {index}")
		if bond.get("aromatic") is not True:
			raise E2eError(f"semantic probe default unexpectedly cleared aromaticity at bond {index}")
		order = bond.get("order")
		if order not in {"single", "double"}:
			raise E2eError(f"semantic probe did not assign a Kekule order at bond {index}: {order!r}")
		orders.append(order)
	if orders.count("single") != 3 or orders.count("double") != 3:
		message = "semantic probe result is not a three-single/three-double Kekule form"
		raise E2eError(f"{message}: {orders!r}")
	if any(orders[index] == orders[(index + 1) % len(orders)] for index in range(len(orders))):
		raise E2eError(f"semantic probe result does not alternate around benzene: {orders!r}")


#============================================
def chemistry_probe(
	adapter: Path, output_root: Path, expected_abi_version: int,
) -> dict[str, object]:
	"""Run the safe Rust engine in a fresh, loader-scrubbed process."""
	if not adapter.is_absolute() or not adapter.is_file():
		raise E2eError(f"semantic probe adapter must be an absolute library path: {adapter}")
	environment = scrubbed_environment()
	environment["CARGO_TARGET_DIR"] = str(output_root / "rust-e2e-target")
	value = parse_json_object(
		run(
			"cargo",
			"run",
			"--quiet",
			"--manifest-path",
			str(CHEMISTRY_MANIFEST),
			"--example",
			CHEMISTRY_EXAMPLE,
			"--",
			"--adapter",
			str(adapter),
			env=environment,
		),
		"Rust native chemistry semantic probe",
	)
	assert_semantic_probe(value, expected_abi_version)
	return value


#============================================
def load_build_tool() -> ModuleType:
	# The tracked builder is dynamically imported for closure validation.  Keep
	# that read-only operation from generating a source-tree __pycache__ even if
	# the caller forgot the environment setting.
	original_dont_write_bytecode = sys.dont_write_bytecode
	original_path = sys.path[:]
	try:
		sys.dont_write_bytecode = True
		sys.path.insert(0, str(BUILD_TOOL.parent.resolve()))
		specification = importlib.util.spec_from_file_location(
			"ferrum_native_wheel_build", BUILD_TOOL,
		)
		if specification is None or specification.loader is None:
			raise E2eError(f"could not load native wheel closure policy from {BUILD_TOOL}")
		module = importlib.util.module_from_spec(specification)
		sys.modules[specification.name] = module
		specification.loader.exec_module(module)
		return module
	finally:
		sys.dont_write_bytecode = original_dont_write_bytecode
		sys.path[:] = original_path


#============================================
# Builder protocol validation


#============================================
def build_contract() -> BuildContract:
	"""Read the native builder's current, typed machine-protocol authority."""
	module = load_build_tool()
	values = {
		"target": getattr(module, "TARGET", None),
		"result_schema": getattr(module, "MACHINE_RESULT_SCHEMA", None),
		"adapter_abi_version": getattr(module, "ADAPTER_ABI_VERSION", None),
	}
	if (
		not isinstance(values["target"], str)
		or not values["target"]
		or not isinstance(values["result_schema"], str)
		or not values["result_schema"]
		or type(values["adapter_abi_version"]) is not int
		or values["adapter_abi_version"] < 1
	):
		raise E2eError(f"native build tool exports an invalid machine contract: {values!r}")
	return BuildContract(
		target=values["target"],
		result_schema=values["result_schema"],
		adapter_abi_version=values["adapter_abi_version"],
	)


#============================================
def parse_artifact_result(
	stdout: str, action: str, output_root: Path, contract: BuildContract,
) -> Path:
	"""Accept exactly one builder JSON artifact record and no progress on stdout."""
	lines = stdout.splitlines()
	if len(lines) != 1:
		raise E2eError(f"builder {action} result must be exactly one JSON line, got {len(lines)} lines")
	try:
		result = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise E2eError(f"builder {action} result is not valid JSON: {error.msg}") from error
	if not isinstance(result, dict):
		raise E2eError(f"builder {action} result must be a JSON object: {result!r}")
	for field in ("schema", "action", "artifact"):
		if field not in result:
			raise E2eError(f"builder {action} result omits required field {field!r}: {result!r}")
	if not isinstance(result["schema"], str) or not isinstance(result["action"], str):
		raise E2eError(f"builder {action} result has non-string protocol fields: {result!r}")
	if not isinstance(result["artifact"], str):
		raise E2eError(f"builder {action} artifact is not a path string: {result!r}")
	if result["schema"] != contract.result_schema or result["action"] != action:
		raise E2eError(f"builder {action} result has the wrong schema or action: {result!r}")
	artifact = Path(result["artifact"])
	if not output_root.is_dir():
		raise E2eError(f"builder {action} output root does not exist: {output_root}")
	if not artifact.is_absolute():
		raise E2eError(f"builder {action} artifact must be an absolute normalized path: {artifact}")
	try:
		normalized_artifact = artifact.resolve(strict=True)
	except FileNotFoundError as error:
		raise E2eError(f"builder {action} reported a missing artifact: {artifact}") from error
	if artifact != normalized_artifact:
		raise E2eError(f"builder {action} artifact must be an absolute normalized path: {artifact}")
	if (
		not normalized_artifact.is_relative_to(output_root.resolve())
		or not normalized_artifact.is_file()
	):
		raise E2eError(f"builder {action} reported a missing or out-of-root artifact: {artifact}")
	return normalized_artifact


#============================================
def command_self_test() -> None:
	"""Prove logs on stdout cannot be mistaken for the artifact result."""
	builder = load_build_tool()
	contract = build_contract()
	semantic_fixture = semantic_probe_fixture(contract.adapter_abi_version)
	assert_semantic_probe(semantic_fixture, contract.adapter_abi_version)
	broken_semantic_fixture = semantic_probe_fixture(contract.adapter_abi_version)
	broken_semantic_fixture["output"] = {
		"atoms": expected_semantic_atoms(),
		"bonds": expected_semantic_bonds(["single", "single", "double"] * 2),
	}
	try:
		assert_semantic_probe(broken_semantic_fixture, contract.adapter_abi_version)
	except E2eError:
		pass
	else:
		raise E2eError("semantic probe self-test accepted non-alternating Kekule bonds")
	builder.validate_wheel_members([
		"ferrum_api/__init__.py",
		"ferrum_api/_native.cpython-312-darwin.so",
		*(
			f"ferrum_api/.libs/{name}"
			for name in sorted(builder.MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names)
		),
	])
	artifact = BUILD_TOOL.resolve()
	root = REPO_ROOT.resolve()
	valid = json.dumps({
		"schema": contract.result_schema,
		"action": "wheel",
		"artifact": str(artifact),
		"future_additive_field": {"supported": True},
	})
	if parse_artifact_result(valid, "wheel", root, contract) != artifact:
		raise E2eError("builder result parser self-test did not preserve a valid artifact")
	for output in (
		f"maturin progress\\n{valid}",
		f"{valid}\\n{valid}",
		json.dumps({"schema": "wrong", "action": "wheel", "artifact": str(artifact)}),
		json.dumps({"schema": contract.result_schema, "action": "wrong", "artifact": str(artifact)}),
		json.dumps({"schema": 42, "action": "wheel", "artifact": str(artifact)}),
		json.dumps({"schema": contract.result_schema, "action": 42, "artifact": str(artifact)}),
		json.dumps({"schema": contract.result_schema, "action": "wheel", "artifact": "relative.whl"}),
		json.dumps({"schema": contract.result_schema, "action": "wheel", "artifact": 42}),
		json.dumps({"schema": contract.result_schema, "action": "wheel"}),
	):
		try:
			parse_artifact_result(output, "wheel", root, contract)
		except E2eError:
			pass
		else:
			raise E2eError("builder result parser self-test accepted noisy or invalid stdout")
	with tempfile.TemporaryDirectory() as temporary:
		directory = Path(temporary)
		wheel = directory / "ferrum_api-test.whl"
		wheel.write_bytes(b"wheel")
		evidence = native_evidence(
			{"profile": "self-test"}, wheel,
			sorted(load_build_tool().MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names),
			{"abi_version": contract.adapter_abi_version},
			{"abi_version": contract.adapter_abi_version},
			"a" * 64,
			"b" * 64,
			directory / "installed" / "libferrum_chem.dylib",
			contract.adapter_abi_version,
			semantic_fixture,
			semantic_fixture,
			"RelWithDebInfo",
		)
		receipt = publish_evidence(directory, evidence)
		if json.loads(receipt.read_text(encoding="utf-8")) != evidence:
			raise E2eError("native evidence self-test did not retain the complete evidence record")
		try:
			replacement_proof(
				"not-a-digest", "b" * 64, directory / "installed" / "libferrum_chem.dylib",
				contract.adapter_abi_version, "RelWithDebInfo",
			)
		except E2eError:
			pass
		else:
			raise E2eError("replacement proof self-test accepted an invalid adapter digest")
		for original, replacement, build_type in (
			("a" * 64, "a" * 64, "RelWithDebInfo"),
			("a" * 64, "b" * 64, "Unsupported"),
		):
			try:
				replacement_proof(
					original,
					replacement,
					directory / "installed" / "libferrum_chem.dylib",
					contract.adapter_abi_version,
					build_type,
				)
			except E2eError:
				pass
			else:
				raise E2eError(
					"replacement proof self-test accepted indistinct bytes or build policy"
				)


#============================================
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


#============================================
def installed_site_packages(venv: Path) -> Path:
	paths = list((venv / "lib").glob("python*/site-packages"))
	if len(paths) != 1:
		raise E2eError(f"could not identify isolated venv site-packages under {venv}")
	return paths[0]


#============================================
# Direct E2E workflow


#============================================
def main() -> int:
	contract = build_contract()
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--target", default=contract.target)
	parser.add_argument(
		"--rdkit-archive",
		help="optional pinned archive; its digest is rechecked by the build tool",
	)
	parser.add_argument(
		"--self-test",
		action="store_true",
		help="validate the builder stdout result parser without building",
	)
	arguments = parser.parse_args()
	if arguments.self_test:
		command_self_test()
		print(json.dumps({
			"schema": "ferrum-native-wheel-e2e-self-test-v1",
			"status": "ok",
		}, sort_keys=True))
		return 0
	if arguments.target != contract.target:
		raise E2eError(
			f"native-wheel proof supports only {contract.target}, not {arguments.target}"
		)
	output_parent = REPO_ROOT / "output_native_wheel"
	output_parent.mkdir(exist_ok=True)
	with tempfile.TemporaryDirectory(prefix="e2e-native-wheel-", dir=output_parent) as temporary:
		output_root = Path(temporary)
		build_command = [
			sys.executable,
			str(BUILD_TOOL),
			"build",
			"--output-root",
			str(output_root),
			"--target",
			arguments.target,
		]
		if arguments.rdkit_archive:
			build_command.extend(("--rdkit-archive", arguments.rdkit_archive))
		wheel = parse_artifact_result(
			run(*build_command, env=scrubbed_environment(), stream_stderr=True),
			"wheel",
			output_root,
			contract,
		)
		builder_receipt = read_build_receipt(output_root)
		venv = output_root / "clean-venv"
		run(sys.executable, "-m", "venv", str(venv))
		python = venv / "bin" / "python"
		run(str(python), "-m", "pip", "install", "--no-deps", str(wheel), env=scrubbed_environment())
		before = probe(python)
		if before != {"abi_version": contract.adapter_abi_version}:
			raise E2eError(f"initial isolated probe was not the wheel ABI: {before}")
		libs = assert_wheel_closure(installed_site_packages(venv))
		closure_names = sorted(path.name for path in libs.glob("*.dylib"))
		chemistry_before = chemistry_probe(
			(libs / "libferrum_chem.dylib").resolve(), output_root,
			contract.adapter_abi_version,
		)
		installed_replacement = libs / "libferrum_chem.dylib"
		original_sha256 = sha256(installed_replacement)
		replacement_root = output_root / "replacement-output"
		replacement = parse_artifact_result(
			run(
				sys.executable,
				str(BUILD_TOOL),
				"adapter",
				"--output-root",
				str(replacement_root),
				"--rdkit-output-root",
				str(output_root),
				"--build-type",
				"RelWithDebInfo",
				env=scrubbed_environment(),
				stream_stderr=True,
			),
			"adapter",
			replacement_root,
			contract,
		)
		replacement_sha256 = sha256(replacement)
		replacement_proof(
			original_sha256, replacement_sha256, installed_replacement.resolve(),
			contract.adapter_abi_version, "RelWithDebInfo",
		)
		shutil.copy2(replacement, installed_replacement)
		if sha256(installed_replacement) != replacement_sha256:
			raise E2eError("replacement adapter copy did not preserve the verified library bytes")
		assert_wheel_closure(installed_site_packages(venv))
		after = probe(python)
		if after != {"abi_version": contract.adapter_abi_version}:
			raise E2eError(f"replaced library was not loaded in a fresh process: {after}")
		chemistry_after = chemistry_probe(
			installed_replacement.resolve(), output_root, contract.adapter_abi_version,
		)
		evidence = native_evidence(
			builder_receipt,
			wheel,
			closure_names,
			before,
			after,
			original_sha256,
			replacement_sha256,
			installed_replacement.resolve(),
			contract.adapter_abi_version,
			chemistry_before,
			chemistry_after,
			"RelWithDebInfo",
		)
		receipt = publish_evidence(output_parent, evidence)
		print(json.dumps({
			"after": after,
			"before": before,
			"chemistry_after": chemistry_after,
			"chemistry_before": chemistry_before,
			"receipt": str(receipt),
		}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except E2eError as error:
		print(f"initial native-wheel E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
