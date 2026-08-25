#!/usr/bin/env bash
# Run the supported local end-to-end checks against the staged build.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly LOCAL_CLI="${REPO_ROOT}/build/bin/ferrum"
readonly LOCAL_PYTHON_ROOT="${REPO_ROOT}/build/runtime/python"
readonly LOCAL_RUNTIME_RECEIPT="${REPO_ROOT}/packages/ferrum-rust/local_runtime_receipt.py"
readonly QT_SOURCE_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app"
readonly LOCAL_PYTHONPATH="${LOCAL_PYTHON_ROOT}:${QT_SOURCE_ROOT}:${REPO_ROOT}/tests"


#============================================
run_e2e() {
	local label="$1"
	shift
	printf '\n=== %s ===\n' "${label}"
	"$@"
}


#============================================
require_local_runtime() {
	if ! python3 "${LOCAL_RUNTIME_RECEIPT}" validate \
		--runtime-root "${LOCAL_PYTHON_ROOT}"; then
		printf 'local E2E error: the staged Ferrum runtime is missing, stale, or modified.\n' >&2
		printf 'Run ./build.sh before this local E2E runner.\n' >&2
		exit 1
	fi
}


source "${REPO_ROOT}/source_me.sh"
export PYTHONPATH="${LOCAL_PYTHONPATH}"
export QT_QPA_PLATFORM=offscreen

run_e2e "Ferrum sourced local runtime E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_source_me_local_runtime.py"
run_e2e "Ferrum local build cleanup E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_build_local_runtime_cleanup.py"
require_local_runtime

run_e2e "Ferrum CLI verb E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_ferrum_verb_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI operation protocol E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_ferrum_protocol_v1.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI compact-group materialization E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_compact_group_materialization_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI template catalog E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_template_catalog_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum document SDF export E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_document_export_sdf_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum Qt SDF import E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_sdf_import.py"
run_e2e "Ferrum Qt peptide sequence import E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_peptide_sequence_import.py"
run_e2e "Ferrum Qt render interaction E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_render_interaction_selection.py"
run_e2e "Ferrum Qt atom oxidation observation E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_atom_oxidation_observation.py"
run_e2e "Ferrum Qt E/Z carrier-mark projection E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_ez_carrier_mark_projection.py"
run_e2e "Ferrum Qt arrow authoring E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_arrow_authoring.py"
run_e2e "Ferrum Qt presentation vector authoring E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_presentation_vector_authoring.py"
run_e2e "Ferrum Qt template catalog authoring E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_template_catalog_authoring.py"
run_e2e "Ferrum Qt compact-group materialization E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_compact_group_materialization_qt.py"
