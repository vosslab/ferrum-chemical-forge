#!/usr/bin/env bash
# Run the supported local end-to-end checks against the staged build.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly LOCAL_CLI="${REPO_ROOT}/build/bin/ferrum"
readonly LOCAL_PYTHON_ROOT="${REPO_ROOT}/build/runtime/python"
readonly QT_SOURCE_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app"


#============================================
run_e2e() {
	local label="$1"
	shift
	printf '\n=== %s ===\n' "${label}"
	"$@"
}


#============================================
require_local_cli() {
	if [[ ! -x "${LOCAL_CLI}" ]]; then
		printf 'local CLI E2E error: staged Ferrum CLI is missing or not executable: %s\n' \
			"${LOCAL_CLI}" >&2
		printf 'Run ./build.sh before this local CLI E2E runner.\n' >&2
		exit 1
	fi
}


############################################
require_local_qt_runtime() {
	if ! compgen -G "${LOCAL_PYTHON_ROOT}/ferrum_chem.*" > /dev/null; then
		printf 'local Qt E2E error: staged Ferrum extension is missing: %s\n' \
			"${LOCAL_PYTHON_ROOT}" >&2
		printf 'Run ./build.sh before this local E2E runner.\n' >&2
		exit 1
	fi
}


source "${REPO_ROOT}/source_me.sh"
require_local_cli
require_local_qt_runtime

run_e2e "Ferrum CLI verb E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_ferrum_verb_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum document SDF export E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_document_export_sdf_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum Qt render interaction E2E" \
	env PYTHONPATH="${LOCAL_PYTHON_ROOT}:${QT_SOURCE_ROOT}" QT_QPA_PLATFORM=offscreen \
	python3 "${REPO_ROOT}/tests/e2e/e2e_render_interaction_selection.py"
run_e2e "Ferrum Qt arrow authoring E2E" \
	env PYTHONPATH="${LOCAL_PYTHON_ROOT}:${QT_SOURCE_ROOT}" QT_QPA_PLATFORM=offscreen \
	python3 "${REPO_ROOT}/tests/e2e/e2e_arrow_authoring.py"
