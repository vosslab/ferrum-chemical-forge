#!/usr/bin/env python3
"""Prove sourcing the bootstrap selects this checkout's local Python sources."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile


#============================================
class SourceMeLocalRuntimeE2eError(RuntimeError):
	"""Report one failed sourced local-runtime bootstrap outcome."""


#============================================
def _require_ordered_provenance(
	repository: Path, pythonpath: str, ferrum_chem_file: str, ferrum_qt_file: str,
) -> None:
	"""Require source-owned path order and real local module provenance."""
	expected_runtime_root = repository / "build/runtime/python"
	expected_qt_root = repository / "packages/ferrum-chem-qt.app"
	pythonpath_entries = pythonpath.split(":")
	if pythonpath_entries[:2] != [str(expected_qt_root), str(expected_runtime_root)]:
		raise SourceMeLocalRuntimeE2eError(
			"source_me.sh did not keep Qt source first and the sealed runtime second"
		)
	ferrum_chem_path = Path(ferrum_chem_file).resolve()
	ferrum_qt_path = Path(ferrum_qt_file).resolve()
	if not ferrum_chem_path.is_relative_to(expected_runtime_root.resolve()):
		raise SourceMeLocalRuntimeE2eError(
			f"ferrum_chem loaded outside the local runtime: {ferrum_chem_path}"
		)
	if not ferrum_qt_path.is_relative_to(expected_qt_root.resolve()):
		raise SourceMeLocalRuntimeE2eError(
			f"ferrum_qt loaded outside the repository Qt source: {ferrum_qt_path}"
		)


#============================================
def _require_sourced_child_provenance(
	repository: Path, caller_pythonpath: str | None, shell_options: str,
) -> None:
	"""Source from outside the checkout and require local module provenance."""
	source_script = repository / "source_me.sh"
	with tempfile.TemporaryDirectory(prefix="ferrum-source-me-runtime-") as directory:
		temporary_root = Path(directory)
		environment = os.environ | {
			"PYTHONDONTWRITEBYTECODE": "1",
		}
		if caller_pythonpath is None:
			environment.pop("PYTHONPATH", None)
		else:
			environment["PYTHONPATH"] = caller_pythonpath
		result = subprocess.run(
			(
				"bash", shell_options,
				"source \"$1\" && python3 -c "
				"'import os, ferrum_chem, ferrum_qt; "
				"print(os.environ[\"PYTHONPATH\"]); "
				"print(ferrum_chem.__file__); print(ferrum_qt.__file__)'",
				"bash", str(source_script),
			),
			cwd=temporary_root, env=environment, check=False, capture_output=True, text=True,
		)
		if result.returncode != 0:
			raise SourceMeLocalRuntimeE2eError(
				f"sourcing source_me.sh failed: {result.stderr.strip()}"
			)
		module_lines = result.stdout.splitlines()
		if len(module_lines) < 3:
			raise SourceMeLocalRuntimeE2eError("sourced Python imports did not report module paths")
		_require_ordered_provenance(repository, *module_lines[-3:])


#============================================
def _require_current_provenance(repository: Path) -> None:
	"""Require the aggregate runner process to retain source-owned provenance."""
	import ferrum_chem
	import ferrum_qt
	_require_ordered_provenance(
		repository, os.environ["PYTHONPATH"], ferrum_chem.__file__, ferrum_qt.__file__,
	)


#============================================
def main() -> int:
	"""Prove direct bootstrap or aggregate-process source provenance."""
	repository = Path(__file__).resolve().parents[2]
	if sys.argv[1:] == ["--current-environment"]:
		_require_current_provenance(repository)
	else:
		with tempfile.TemporaryDirectory(prefix="ferrum-source-me-caller-") as directory:
			caller_pythonpath = str(Path(directory))
			_require_sourced_child_provenance(repository, caller_pythonpath, "-c")
		_require_sourced_child_provenance(repository, None, "-uc")
	print('{"schema":"ferrum-source-me-local-runtime-e2e-v1","status":"ok"}')
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except SourceMeLocalRuntimeE2eError as error:
		print(f"source_me local runtime E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
