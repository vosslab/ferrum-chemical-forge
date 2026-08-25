"""Generate the source-owned launcher for Ferrum's sealed local Qt runtime."""

from __future__ import annotations

import argparse
import os
import stat
from pathlib import Path


_GUI_LAUNCHER = """#!/usr/bin/env bash
# Run the source Qt application against the extension built by ./build.sh.

set -euo pipefail

readonly PROGRAM_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
readonly REPO_ROOT="$(cd "${PROGRAM_ROOT}/../../.." && pwd -P)"
readonly LOCAL_PYTHON_ROOT="${PROGRAM_ROOT}/runtime/python"
readonly QT_SOURCE_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app"
readonly LOCAL_RUNTIME_RECEIPT="${REPO_ROOT}/packages/ferrum-rust/local_runtime_receipt.py"

[[ -f "${REPO_ROOT}/source_me.sh" ]] || {
	printf 'ferrum local repository bootstrap is missing: %s\\n' "${REPO_ROOT}/source_me.sh" >&2
	exit 1
}
source "${REPO_ROOT}/source_me.sh"
python3 "${LOCAL_RUNTIME_RECEIPT}" validate --runtime-root "${LOCAL_PYTHON_ROOT}"
exec python3 -m ferrum_qt "$@"
"""
_OWNER_EXECUTABLE_FILE_MODE = stat.S_IRUSR | stat.S_IWUSR | stat.S_IXUSR


#============================================
def gui_launcher_text() -> str:
	"""Return the complete versioned GUI launcher specification."""
	return _GUI_LAUNCHER


#============================================
def write_gui_launcher(path: Path) -> None:
	"""Write one executable launcher into a fresh candidate program tree."""
	if path.exists() or path.is_symlink():
		raise ValueError(f"local GUI launcher destination already exists: {path}")
	if not path.parent.is_dir():
		raise ValueError(f"local GUI launcher parent is missing: {path.parent}")
	try:
		file_descriptor = os.open(
			path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, _OWNER_EXECUTABLE_FILE_MODE,
		)
	except FileExistsError as error:
		raise ValueError(f"local GUI launcher destination already exists: {path}") from error
	with os.fdopen(file_descriptor, "w", encoding="utf-8") as handle:
		handle.write(gui_launcher_text())


#============================================
def main() -> int:
	"""Write the canonical GUI launcher for one fresh local build candidate."""
	parser = argparse.ArgumentParser(description="Generate Ferrum's local Qt launcher.")
	parser.add_argument("--write-gui", action="store_true")
	parser.add_argument("--launcher-path", required=True, type=Path)
	arguments = parser.parse_args()
	if not arguments.write_gui:
		parser.error("--write-gui is required")
	write_gui_launcher(arguments.launcher_path)
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
