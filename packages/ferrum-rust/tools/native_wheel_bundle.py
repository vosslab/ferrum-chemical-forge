"""Small sealed-engine bundle helpers shared by Ferrum's native builder."""

from __future__ import annotations

import json
import platform
from pathlib import Path


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
