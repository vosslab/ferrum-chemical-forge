#!/usr/bin/env bash
# Repository verification front door. Build the local Ferrum runtime first.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly PYO3_TEST_ROOT="${REPO_ROOT}/packages/ferrum-rust/crates/api/python/tests"
readonly QT_TEST_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app/tests"
readonly LOCAL_PYTHON_ROOT="${REPO_ROOT}/build/runtime/python"
readonly LOCAL_ADAPTER="${LOCAL_PYTHON_ROOT}/.dylibs/libferrum_chem.dylib"
readonly LOCAL_RUNTIME_RECEIPT="${LOCAL_PYTHON_ROOT}/ferrum-local-runtime-receipt.json"
readonly QT_SOURCE_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app"
readonly LOCAL_CLI="${REPO_ROOT}/build/bin/ferrum"
readonly LOCAL_GUI="${REPO_ROOT}/build/bin/ferrum-qt"


#============================================
run_step() {
	local label="$1"
	shift
	printf '\n=== %s ===\n' "${label}"
	"$@"
}


#============================================
require_local_runtime() {
	if [[ ! -f "${LOCAL_EXTENSION}" || ! -f "${LOCAL_ADAPTER}" || ! -f "${LOCAL_RUNTIME_RECEIPT}" ]]; then
		printf 'all_test error: complete local Ferrum runtime is missing.\n' >&2
		printf '  Extension: %s\n' "${LOCAL_EXTENSION}" >&2
		printf '  Adapter: %s\n' "${LOCAL_ADAPTER}" >&2
		printf '  Receipt: %s\n' "${LOCAL_RUNTIME_RECEIPT}" >&2
		printf 'Run ./build.sh before ./all_test.sh.\n' >&2
		exit 1
	fi
	if ! python3 "${REPO_ROOT}/packages/ferrum-rust/local_runtime_receipt.py" validate \
		--runtime-root "${LOCAL_PYTHON_ROOT}"; then
		printf 'all_test error: local Ferrum runtime is stale or has been modified.\n' >&2
		printf 'Run ./build.sh before ./all_test.sh.\n' >&2
		exit 1
	fi
}


#============================================
require_local_launchers() {
	local chemistry_output
	if [[ ! -x "${LOCAL_CLI}" || ! -x "${LOCAL_GUI}" ]]; then
		printf 'all_test error: local Ferrum launchers are missing or not executable.\n' >&2
		printf '  CLI: %s\n' "${LOCAL_CLI}" >&2
		printf '  GUI: %s\n' "${LOCAL_GUI}" >&2
		exit 1
	fi
	run_step "Local launcher provenance" bash -n "${LOCAL_GUI}"
	run_step "Local CLI smoke" "${LOCAL_CLI}" --version
	chemistry_output="$(printf 'C\n' | "${LOCAL_CLI}" convert - --from smiles --to smiles)"
	if [[ "${chemistry_output}" != "C" ]]; then
		printf 'all_test error: local CLI chemistry smoke returned an unexpected result.\n' >&2
		exit 1
	fi
}


source "${REPO_ROOT}/source_me.sh"
readonly LOCAL_EXTENSION="$(python3 "${REPO_ROOT}/packages/ferrum-rust/local_runtime_receipt.py" \
	extension-path --runtime-root "${LOCAL_PYTHON_ROOT}")"

run_step "Repository hygiene tests" pytest "${REPO_ROOT}/tests/"
require_local_runtime
require_local_launchers
run_step "Local Ferrum CLI E2E tests" bash "${REPO_ROOT}/tests/e2e/run_all.sh"

run_step "Local Ferrum-Chem Python tests" \
	pytest "${PYO3_TEST_ROOT}"
run_step "Ferrum Qt tests" \
	env QT_QPA_PLATFORM=offscreen pytest "${QT_TEST_ROOT}"
