"""Output-root admission for Ferrum's native-wheel builder."""

from __future__ import annotations

import argparse
from pathlib import Path


#============================================
def engine_bundle_path(value: str) -> Path:
	"""Resolve one engine-bundle destination for later output-root containment checks.

	An engine bundle is a child payload, not an independently admitted build root.
	``build_engine_bundle`` verifies that this resolved path is strictly beneath
	the already-admitted ``--output-root`` before it creates anything.
	"""
	return Path(value).expanduser().resolve()


#============================================
def output_path(value: str, repo_root: Path) -> Path:
	"""Resolve one builder output root while rejecting retired developer layouts.

	The developer wrapper owns the sole rotating native build path below
	``build/native-staging`` and publishes only ``output_native_wheel/current``.
	Independent release tooling may continue to use other checkout ``output*``
	roots.
	"""
	path = Path(value).expanduser().resolve()
	if path.is_relative_to(repo_root / "OTHER_REPOS"):
		raise argparse.ArgumentTypeError("--output-root must not be inside OTHER_REPOS")
	try:
		relative = path.relative_to(repo_root)
	except ValueError:
		relative = None
	if relative is not None:
		if (
			len(relative.parts) >= 2
			and relative.parts[0] == "output_native_wheel"
			and relative.parts[1].startswith("native-")
		):
			raise argparse.ArgumentTypeError(
				"--output-root must not use retired output_native_wheel/native-* roots; "
				"use build.sh native"
			)
		if relative.parts and relative.parts[0].startswith("output"):
			return path
		if (
			relative.parts[:2] == ("build", "native-staging")
			and len(relative.parts) == 3
			and relative.parts[2].startswith("native-")
		):
			return path
	else:
		temporary_root = Path("/private/tmp")
		try:
			temporary_relative = path.relative_to(temporary_root)
		except ValueError:
			temporary_relative = None
		if (
			temporary_relative is not None
			and temporary_relative.parts
			and temporary_relative.parts[0].startswith("ferrum-native-")
		):
			return path
	raise argparse.ArgumentTypeError(
		"--output-root must be beneath a checkout output* directory, build/native-staging/native-*, or "
		"/private/tmp/ferrum-native-*"
	)
