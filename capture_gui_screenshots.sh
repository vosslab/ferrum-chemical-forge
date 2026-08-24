#!/usr/bin/env bash
# Capture the documented Ferrum Qt GUI tour from a locally built application.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly LOCAL_RUNTIME="${REPO_ROOT}/build/runtime/python"
readonly QT_SOURCE_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app"


#============================================
require_local_runtime() {
	"${REPO_ROOT}/build/bin/ferrum" --help > /dev/null
	if ! compgen -G "${LOCAL_RUNTIME}/ferrum_chem.*" > /dev/null; then
		printf 'GUI screenshot capture error: staged Ferrum extension is missing: %s\n' \
			"${LOCAL_RUNTIME}" >&2
		printf 'Run ./build.sh before capture_gui_screenshots.sh.\n' >&2
		exit 1
	fi
}


#============================================
main() {
	source "${REPO_ROOT}/source_me.sh"
	export PYTHONDONTWRITEBYTECODE=1
	export PYTHONPATH="${LOCAL_RUNTIME}:${QT_SOURCE_ROOT}${PYTHONPATH:+:${PYTHONPATH}}"
	require_local_runtime
	exec python3 -B "${REPO_ROOT}/devel/capture_gui_screenshots.py" "$@"
}


main "$@"
