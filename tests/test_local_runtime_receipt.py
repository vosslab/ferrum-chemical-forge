"""Contract tests for local runtime freshness receipt logic."""

import sysconfig
from pathlib import Path

import engine_lib.local_runtime_receipt as receipt
import pytest


#============================================
def test_local_cargo_source_digest_changes_when_selected_source_changes(
	tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
) -> None:
	"""The local Cargo closure digest detects a selected Rust source edit."""
	package = tmp_path / "crates/api"
	(package / "src").mkdir(parents=True)
	(tmp_path / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
	(tmp_path / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
	(package / "Cargo.toml").write_text("[package]\nname = 'api'\n", encoding="utf-8")
	source = package / "src/lib.rs"
	source.write_text("pub fn version() -> u8 { 1 }\n", encoding="utf-8")
	monkeypatch.setattr(receipt, "_CARGO_ROOT", tmp_path)
	monkeypatch.setattr(receipt, "_REPO_ROOT", tmp_path.parent)
	monkeypatch.setattr(receipt, "_BUILD_PACKAGES", (package,))
	metadata = _cargo_metadata_for(package, [])
	before = receipt.local_cargo_source_sha256(metadata)
	source.write_text("pub fn version() -> u8 { 2 }\n", encoding="utf-8")
	assert receipt.local_cargo_source_sha256(metadata) != before


#============================================
def test_local_cargo_source_digest_ignores_a_dev_only_package_edit(
		tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
	) -> None:
	"""A release receipt excludes sources reachable only through dev dependencies."""
	api = _write_cargo_package(tmp_path, "api", "pub fn api() {}\n")
	dev_helper = _write_cargo_package(tmp_path, "dev-helper", "pub fn fixture() { 1; }\n")
	(tmp_path / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
	(tmp_path / "Cargo.lock").write_text("version = 4\n", encoding="utf-8")
	monkeypatch.setattr(receipt, "_CARGO_ROOT", tmp_path)
	monkeypatch.setattr(receipt, "_REPO_ROOT", tmp_path.parent)
	monkeypatch.setattr(receipt, "_BUILD_PACKAGES", (api,))
	metadata = _cargo_metadata_for(api, [(dev_helper, "dev")])
	before = receipt.local_cargo_source_sha256(metadata)
	(dev_helper / "src/lib.rs").write_text("pub fn fixture() { 2; }\n", encoding="utf-8")
	assert receipt.local_cargo_source_sha256(metadata) == before


#============================================
def test_local_extension_path_uses_the_current_python_abi_suffix(tmp_path: Path) -> None:
	"""The staged extension has the exact filename Python will import first."""
	suffix = sysconfig.get_config_var("EXT_SUFFIX")
	assert type(suffix) is str and suffix.startswith(".")
	assert receipt.local_extension_path(tmp_path) == tmp_path / f"ferrum_chem{suffix}"


#============================================
def test_local_runtime_receipt_rejects_a_changed_staged_artifact(
	tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
) -> None:
	"""A copied or modified launcher cannot pass the local runtime gate."""
	runtime_root = _write_runtime_tree(tmp_path)
	monkeypatch.setattr(receipt, "local_runtime_inputs", lambda: {"input": "v1"})
	receipt.write_local_runtime_receipt(runtime_root)
	(runtime_root.parents[1] / "bin/ferrum").write_bytes(b"cli-v2")
	with pytest.raises(receipt.LocalRuntimeReceiptError, match="does not match"):
		receipt.validate_local_runtime_receipt(runtime_root)


#============================================
def test_local_runtime_receipt_rejects_a_changed_engine_bundle(
		tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
	) -> None:
	"""A modified CLI chemistry closure cannot pass the local runtime gate."""
	runtime_root = _write_runtime_tree(tmp_path)
	monkeypatch.setattr(receipt, "local_runtime_inputs", lambda: {"input": "v1"})
	receipt.write_local_runtime_receipt(runtime_root)
	(runtime_root.parent / "engine-v1/libferrum_chem.dylib").write_bytes(b"adapter-v2")
	with pytest.raises(receipt.LocalRuntimeReceiptError, match="does not match"):
		receipt.validate_local_runtime_receipt(runtime_root)


#============================================
def test_local_runtime_receipt_requires_executable_launchers(
		tmp_path: Path, monkeypatch: pytest.MonkeyPatch,
	) -> None:
	"""Receipt creation refuses a local launcher lacking its executable mode."""
	runtime_root = _write_runtime_tree(tmp_path)
	(runtime_root.parents[1] / "bin/ferrum-qt").chmod(0o644)
	monkeypatch.setattr(receipt, "local_runtime_inputs", lambda: {"input": "v1"})
	with pytest.raises(receipt.LocalRuntimeReceiptError, match="gui must be executable"):
		receipt.write_local_runtime_receipt(runtime_root)


#============================================
def _write_cargo_package(root: Path, name: str, source: str) -> Path:
	"""Create one minimal local Rust package for closure contract tests."""
	package = root / "crates" / name
	(package / "src").mkdir(parents=True)
	(package / "Cargo.toml").write_text(f"[package]\nname = '{name}'\n", encoding="utf-8")
	(package / "src/lib.rs").write_text(source, encoding="utf-8")
	return package


#============================================
def _cargo_metadata_for(
		root: Path, dependencies: list[tuple[Path, str]],
	) -> dict[str, object]:
	"""Model the relevant Cargo resolve edges without running Cargo in unit tests."""
	packages = [root, *(package for package, _ in dependencies)]
	return {
		"packages": [
			{"id": package.name, "manifest_path": str(package / "Cargo.toml")}
			for package in packages
		],
		"resolve": {"nodes": [
			{
				"id": root.name,
				"deps": [
					{"pkg": package.name, "dep_kinds": [{"kind": kind}]}
					for package, kind in dependencies
				],
			},
			*({"id": package.name, "deps": []} for package, _ in dependencies),
		]},
	}


#============================================
def _write_runtime_tree(root: Path) -> Path:
	"""Create the disposable runtime and both launchers the receipt owns."""
	runtime_root = root / "build/runtime/python"
	(runtime_root / ".dylibs").mkdir(parents=True)
	receipt.local_extension_path(runtime_root).write_bytes(b"extension-v1")
	(runtime_root / ".dylibs/libferrum_chem.dylib").write_bytes(b"adapter-v1")
	bin_root = runtime_root.parents[1] / "bin"
	bin_root.mkdir()
	for name in ("ferrum", "ferrum-qt"):
		launcher = bin_root / name
		launcher.write_bytes(f"{name}-v1".encode("ascii"))
		launcher.chmod(0o755)
	engine_bundle = runtime_root.parent / "engine-v1"
	engine_bundle.mkdir()
	(engine_bundle / "libferrum_chem.dylib").write_bytes(b"engine-adapter-v1")
	return runtime_root
