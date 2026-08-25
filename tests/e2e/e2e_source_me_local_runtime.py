#!/usr/bin/env python3
"""Prove sourcing the bootstrap selects this checkout's staged extension."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile


#============================================
class SourceMeLocalRuntimeE2eError(RuntimeError):
	"""Report one failed sourced local-runtime bootstrap outcome."""


#============================================
def main() -> int:
	"""Source from outside the checkout and require the local extension path."""
	repository = Path(__file__).resolve().parents[2]
	source_script = repository / "source_me.sh"
	expected_root = repository / "build/runtime/python"
	with tempfile.TemporaryDirectory(prefix="ferrum-source-me-runtime-") as directory:
		temporary_root = Path(directory)
		harmless_pythonpath = temporary_root / "harmless-pythonpath"
		harmless_pythonpath.mkdir()
		environment = os.environ | {
			"PYTHONPATH": str(harmless_pythonpath),
			"PYTHONDONTWRITEBYTECODE": "1",
		}
		result = subprocess.run(
			(
				"bash", "-c",
				'source "$1" && python3 -c \'import ferrum_chem; print(ferrum_chem.__file__)\'',
				"bash", str(source_script),
			),
			cwd=temporary_root, env=environment, check=False, capture_output=True, text=True,
		)
		if result.returncode != 0:
			raise SourceMeLocalRuntimeE2eError(
				f"sourcing source_me.sh failed: {result.stderr.strip()}"
			)
		module_lines = result.stdout.splitlines()
		if not module_lines:
			raise SourceMeLocalRuntimeE2eError("sourced Python import did not report ferrum_chem")
		module_path = Path(module_lines[-1]).resolve()
		if not module_path.is_relative_to(expected_root.resolve()):
			raise SourceMeLocalRuntimeE2eError(
				f"ferrum_chem loaded outside the local runtime: {module_path}"
			)
	print('{"schema":"ferrum-source-me-local-runtime-e2e-v1","status":"ok"}')
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except SourceMeLocalRuntimeE2eError as error:
		print(f"source_me local runtime E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
