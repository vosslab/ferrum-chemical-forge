"""Small sealed-engine bundle helpers shared by Ferrum's native builder."""

from __future__ import annotations

import json
import platform
from pathlib import Path


class NativeEngineBundleError(ValueError):
	"""A sealed engine-bundle manifest or payload violates its fixed contract."""


def executable_bundle_target() -> str:
	"""Return Rust's fixed executable target spelling for the local host."""
	architecture = {"arm64": "aarch64"}.get(platform.machine(), platform.machine())
	operating_system = {"Darwin": "macos"}.get(platform.system(), platform.system().lower())
	return f"{architecture}-{operating_system}"


def engine_bundle_manifest(
		members: list[Path], schema: str, adapter_abi_version: int, adapter_name: str,
		sha256: object,
		) -> bytes:
	"""Return the exact digest-bound manifest accepted by the Rust installer."""
	return (json.dumps({
		"schema": schema,
		"target": executable_bundle_target(),
		"adapter_abi_version": adapter_abi_version,
		"adapter": adapter_name,
		"members": [
			{"path": member.name, "sha256": sha256(member)}
			for member in sorted(members)
		],
	}, indent=2, sort_keys=True) + "\n").encode("utf-8")


#============================================
def validate_engine_bundle(
		bundle: Path, manifest_name: str, schema: str, target: str,
		adapter_abi_version: int, adapter_name: str, sha256: object,
		) -> None:
	"""Require one copied engine bundle to match its canonical member manifest."""
	if bundle.is_symlink() or not bundle.is_dir():
		raise NativeEngineBundleError(f"engine bundle is not a regular directory: {bundle}")
	manifest_path = bundle / manifest_name
	if manifest_path.is_symlink() or not manifest_path.is_file():
		raise NativeEngineBundleError(f"engine bundle manifest is not a regular file: {manifest_path}")
	try:
		manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
	except json.JSONDecodeError as error:
		raise NativeEngineBundleError(f"engine bundle manifest is invalid JSON: {error.msg}") from error
	if not isinstance(manifest, dict) or set(manifest) != {
		"schema", "target", "adapter_abi_version", "adapter", "members",
	}:
		raise NativeEngineBundleError("engine bundle manifest has an invalid schema")
	if (
		manifest["schema"] != schema or manifest["target"] != target
		or type(manifest["adapter_abi_version"]) is not int
		or manifest["adapter_abi_version"] != adapter_abi_version
		or manifest["adapter"] != adapter_name
	):
		raise NativeEngineBundleError("engine bundle manifest does not match the local CLI contract")
	members = manifest["members"]
	if not isinstance(members, list) or not members:
		raise NativeEngineBundleError("engine bundle manifest has no members")
	expected_names = {manifest_name}
	for member in members:
		if not isinstance(member, dict) or set(member) != {"path", "sha256"}:
			raise NativeEngineBundleError("engine bundle manifest has an invalid member")
		name = member["path"]
		digest = member["sha256"]
		if (
			not isinstance(name, str) or Path(name).name != name or name in {"", ".", ".."}
			or not isinstance(digest, str) or len(digest) != 64
			or any(character not in "0123456789abcdef" for character in digest)
			or name in expected_names
		):
			raise NativeEngineBundleError("engine bundle manifest has an unsafe member")
		expected_names.add(name)
		path = bundle / name
		if path.is_symlink() or not path.is_file():
			raise NativeEngineBundleError(f"engine bundle member is not a regular file: {path}")
		if sha256(path) != digest:
			raise NativeEngineBundleError(f"engine bundle member digest mismatch: {name}")
	actual_names = {path.name for path in bundle.iterdir()}
	if actual_names != expected_names:
		raise NativeEngineBundleError("engine bundle contains unexpected or missing members")
