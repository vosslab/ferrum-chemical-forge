#!/usr/bin/env bash
# Run the real-Qt V2 measurement gate in strict or accepted-baseline mode.

set -euo pipefail


#============================================
usage() {
	cat <<'EOF'
Usage: devel/run_measure_stack_qt.sh [--baseline|--strict]

Capture every authored V2 glyph-bond fixture through the real offscreen Qt
projection and write immutable layer manifests, reports, overlays, contact
sheets, and a run summary below output_measure_stack_qt/.

Modes:
  --strict    Fail for every visual-quality violation. This is the default
              developer evidence lane.
  --baseline  Require healthy Qt capture plus the frozen accepted-zero renderer
              failure-category receipt.

Prerequisites:
  Run ./build.sh first. The script validates the staged local Python runtime
  before it captures pixels. PySide6, NumPy, and OpenCV must be installed in
  the source_me.sh Python environment.
EOF
}


#============================================
mode="strict"
case "${1:---strict}" in
	--baseline)
		mode="baseline"
		;;
	--strict)
		mode="strict"
		;;
	-h|--help)
		usage
		exit 0
	;;
	*)
		usage >&2
		exit 2
	;;
esac
if [[ "$#" -gt 1 ]]; then
	usage >&2
	exit 2
fi

readonly REPO_ROOT="$(git rev-parse --show-toplevel)"
readonly OUTPUT_ROOT="${REPO_ROOT}/output_measure_stack_qt"
readonly RUNTIME_RECEIPT="${REPO_ROOT}/packages/ferrum-rust/local_runtime_receipt.py"
readonly RUNTIME_ROOT="${REPO_ROOT}/build/runtime/python"

if [[ ! -f "${REPO_ROOT}/measure_stack/fixtures/v2/fixtures.json" ]]; then
	echo "Qt measure stack error: V2 fixture catalog is missing" >&2
	exit 1
fi

source "${REPO_ROOT}/source_me.sh"
python3 "${RUNTIME_RECEIPT}" validate --runtime-root "${RUNTIME_ROOT}"

rm -rf -- "${OUTPUT_ROOT}"
export PYTHONPATH="${PYTHONPATH:+${PYTHONPATH}:}${REPO_ROOT}/tests"
export QT_QPA_PLATFORM=offscreen

arguments=(--output-dir "${OUTPUT_ROOT}")
if [[ "${mode}" == "baseline" ]]; then
	arguments+=(--baseline)
else
	arguments+=(--fail-on-violation)
fi
python3 "${REPO_ROOT}/tests/e2e/e2e_measure_stack_qt.py" "${arguments[@]}"

for artifact in run_summary.json; do
	if [[ ! -s "${OUTPUT_ROOT}/${artifact}" ]]; then
		echo "Qt measure stack error: missing ${artifact}" >&2
		exit 1
	fi
done
if [[ "${mode}" == "baseline" && ! -s "${OUTPUT_ROOT}/baseline_summary.json" ]]; then
	echo "Qt measure stack error: missing baseline_summary.json" >&2
	exit 1
fi

printf 'PASS: real Qt V2 measurement %s completed. Evidence: %s\n' "${mode}" "${OUTPUT_ROOT}"
