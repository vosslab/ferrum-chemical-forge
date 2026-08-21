"""Prove a macOS arm64 native wheel survives a clean install and ABI relink."""

from __future__ import annotations

# Standard-library imports.
import argparse
import contextlib
import json
import shutil
import sys
import tempfile
from pathlib import Path

# Private sibling support.  This runner remains the only public executable.
from _native_wheel_e2e_support import (
	BUILD_TOOL,
	REPO_ROOT,
	BuildContract,
	E2eError,
	assert_direct_python_chemistry_probe,
	assert_fcm1_probe,
	assert_shipped_typing_metadata,
	assert_shipped_wheel_members,
	assert_wheel_closure,
	build_contract,
	direct_python_chemistry_fixture,
	direct_python_chemistry_probe,
	document_probe,
	installed_site_packages,
	load_build_tool,
	native_evidence,
	parse_artifact_result,
	publish_evidence,
	read_build_receipt,
	replacement_proof,
	run,
	rust_chemistry_probe,
	scrubbed_environment,
	semantic_probe_fixture,
	sha256,
)


def command_self_test() -> None:
	"""Prove logs on stdout cannot be mistaken for the artifact result."""
	builder = load_build_tool()
	contract: BuildContract = build_contract()
	direct_python_fixture = {
		"module_origin": "ferrum_chem.cpython-312-darwin.so",
		**direct_python_chemistry_fixture(),
	}
	assert_direct_python_chemistry_probe(direct_python_fixture)
	broken_direct_python_fixture = direct_python_fixture.copy()
	broken_direct_python_fixture["canonical_smiles"] = "OCC"
	try:
		assert_direct_python_chemistry_probe(broken_direct_python_fixture)
	except E2eError:
		pass
	else:
		raise E2eError("direct Python chemistry self-test accepted a noncanonical DTO")
	semantic_fixture = semantic_probe_fixture(contract.adapter_abi_version)
	assert_fcm1_probe(semantic_fixture, contract.adapter_abi_version)
	broken_semantic_fixture = semantic_probe_fixture(contract.adapter_abi_version)
	broken_semantic_fixture["canonical_smiles"] = "OCC"
	try:
		assert_fcm1_probe(broken_semantic_fixture, contract.adapter_abi_version)
	except E2eError:
		pass
	else:
		raise E2eError("semantic probe self-test accepted a noncanonical FCM1 result")
	builder.validate_wheel_members([
		"ferrum_chem.cpython-312-darwin.so",
		"ferrum_chem.pyi",
		"py.typed",
		"ferrum-operation-v1.schema.json",
		*(
			f".dylibs/{name}"
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
		wheel = directory / "ferrum_chem-test.whl"
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
			direct_python_fixture,
			direct_python_fixture,
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


# Direct E2E workflow


#============================================
def main() -> int:
	contract: BuildContract = build_contract()
	parser = argparse.ArgumentParser(description=__doc__)
	build_source = parser.add_mutually_exclusive_group()
	build_source.add_argument(
		"--source-archive-root",
		help="read-only directory containing every selected source archive for a disconnected build",
	)
	build_source.add_argument(
		"--sealed-input-root",
		help="previous builder-validated native inputs copied into this fresh E2E root",
	)
	build_source.add_argument(
		"--existing-build-root",
		help="existing current builder output to audit without rebuilding the wheel",
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
	output_parent = REPO_ROOT / "output_native_wheel"
	output_parent.mkdir(exist_ok=True)
	with contextlib.ExitStack() as resources:
		if arguments.existing_build_root:
			output_root = Path(arguments.existing_build_root).resolve()
			work_root = Path(resources.enter_context(tempfile.TemporaryDirectory(
				prefix="e2e-native-wheel-proof-", dir=output_parent,
			)))
		else:
			output_root = Path(resources.enter_context(tempfile.TemporaryDirectory(
				prefix="e2e-native-wheel-", dir=output_parent,
			)))
			work_root = output_root
		if arguments.existing_build_root:
			wheels = sorted((output_root / "wheelhouse").glob("ferrum_chem-*.whl"))
			if len(wheels) != 1:
				raise E2eError(f"existing build must contain exactly one wheel: {wheels}")
			wheel = wheels[0].resolve()
		else:
			build_command = [
				sys.executable, "-B",
				str(BUILD_TOOL),
				"build",
				"--output-root",
				str(output_root),
			]
			if arguments.source_archive_root:
				build_command.extend(("--source-archive-root", arguments.source_archive_root))
			if arguments.sealed_input_root:
				build_command.extend(("--sealed-input-root", arguments.sealed_input_root))
			wheel = parse_artifact_result(
				run(*build_command, env=scrubbed_environment(), stream_stderr=True),
				"wheel",
				output_root,
				contract,
			)
		builder_receipt = read_build_receipt(output_root)
		assert_shipped_wheel_members(wheel)
		venv = work_root / "clean-venv"
		run(sys.executable, "-B", "-m", "venv", str(venv))
		python = venv / "bin" / "python"
		run(str(python), "-B", "-m", "pip", "install", "--no-deps", str(wheel), env=scrubbed_environment())
		before = document_probe(python)
		python_chemistry_before = direct_python_chemistry_probe(python)
		site_packages = installed_site_packages(venv)
		assert_shipped_typing_metadata(site_packages, python)
		libs = assert_wheel_closure(site_packages)
		closure_names = sorted(path.name for path in libs.glob("*.dylib"))
		rust_chemistry_before = rust_chemistry_probe(
			(libs / "libferrum_chem.dylib").resolve(), work_root,
			contract.adapter_abi_version,
		)
		installed_replacement = libs / "libferrum_chem.dylib"
		original_sha256 = sha256(installed_replacement)
		replacement_root = work_root / "replacement-output"
		replacement = parse_artifact_result(
			run(
				sys.executable, "-B",
				str(BUILD_TOOL),
				"adapter",
				"--output-root",
				str(replacement_root),
				"--rdkit-output-root",
				str(output_root),
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
		after = document_probe(python)
		python_chemistry_after = direct_python_chemistry_probe(python)
		rust_chemistry_after = rust_chemistry_probe(
			installed_replacement.resolve(), work_root, contract.adapter_abi_version,
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
			python_chemistry_before,
			python_chemistry_after,
			rust_chemistry_before,
			rust_chemistry_after,
			"RelWithDebInfo",
		)
		receipt = publish_evidence(output_parent, evidence)
		print(json.dumps({
			"after": after,
			"before": before,
			"python_chemistry_after": python_chemistry_after,
			"python_chemistry_before": python_chemistry_before,
			"rust_chemistry_after": rust_chemistry_after,
			"rust_chemistry_before": rust_chemistry_before,
			"receipt": str(receipt),
		}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except E2eError as error:
		print(f"initial native-wheel E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
