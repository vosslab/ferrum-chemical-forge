#!/usr/bin/env bash
# Run the supported local end-to-end checks against the staged build.

set -euo pipefail

# Keep local E2E imports from writing __pycache__ directories.
export PYTHONDONTWRITEBYTECODE=1

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly LOCAL_CLI="${REPO_ROOT}/build/bin/ferrum"
readonly LOCAL_PYTHON_ROOT="${REPO_ROOT}/build/runtime/python"
readonly LOCAL_RUNTIME_RECEIPT="${REPO_ROOT}/packages/ferrum-rust/local_runtime_receipt.py"


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


# Keep E2E-only support imports as caller entries. source_me.sh retains them
# after its Qt-source and sealed-runtime roots without duplicating either root.
export PYTHONPATH="${PYTHONPATH:+${PYTHONPATH}:}${REPO_ROOT}/tests"
source "${REPO_ROOT}/source_me.sh"
export QT_QPA_PLATFORM=offscreen

run_e2e "Ferrum aggregate source-owned runtime provenance E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_source_me_local_runtime.py" --current-environment
run_e2e "Ferrum local build cleanup E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_build_local_runtime_cleanup.py"
require_local_runtime

run_e2e "Ferrum CLI verb E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_ferrum_verb_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI operation protocol E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_ferrum_protocol_v1.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI molecule report E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_document_molecule_report_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI compact-group materialization E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_compact_group_materialization_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI compact-group attachment E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_compact_group_attachment_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI template catalog E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_template_catalog_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum document SDF export E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_document_export_sdf_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum CLI CDXML open E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_cdxml_open_cli.py" --ferrum "${LOCAL_CLI}"
run_e2e "Ferrum Qt SDF import E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_sdf_import.py"
run_e2e "Ferrum Qt peptide sequence import E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_peptide_sequence_import.py"
run_e2e "Ferrum Qt render interaction E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_render_interaction_selection.py"
run_e2e "Ferrum Qt atom oxidation observation E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_atom_oxidation_observation.py"
run_e2e "Ferrum Qt attached-Me authoring and materialization E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_compact_group_author_to_materialize.py"
run_e2e "Ferrum Qt attached-Me unavailable-anchor recovery E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_compact_group_unavailable_anchor_recovery.py"
run_e2e "Ferrum Qt attached-NO2 authoring and materialization E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_attached_no2_materialization.py"
run_e2e "Ferrum Qt attached-OMe authoring and materialization E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_attached_methoxy_materialization.py"
run_e2e "Ferrum Qt attached-Me compact-group deletion and Undo E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_compact_group_delete.py"
run_e2e "Ferrum Qt free-Me compact-group placement and materialization E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_free_compact_group_placement.py"
run_e2e "Ferrum Qt Check Structure diagnostics and compact-group recovery E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_molecule_diagnostics.py"
run_e2e "Ferrum Qt E/Z carrier-mark projection E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_ez_carrier_mark_projection.py"
run_e2e "Ferrum Qt arrow authoring E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_arrow_authoring.py"
run_e2e "Ferrum Qt reaction authoring, inspection, editing, movement, and deletion E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_reaction_workflow.py"
run_e2e "Ferrum Qt presentation vector authoring E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_presentation_vector_authoring.py"
run_e2e "Ferrum Qt template catalog authoring E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_template_catalog_authoring.py"
run_e2e "Ferrum Qt SMARTS partial-result warning E2E" \
	python3 "${REPO_ROOT}/tests/e2e/e2e_smarts_partial_result_warning.py"
