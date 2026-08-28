#!/usr/bin/env bash
# Build and import Ferrum's checked-in Maturin wheel in an isolated Python 3.12 venv.

set -euo pipefail

readonly REPO_ROOT="$(git rev-parse --show-toplevel)"
readonly PYPROJECT_ROOT="${REPO_ROOT}/packages/ferrum-rust/crates/api/python"
readonly TEMP_ROOT="${TMPDIR:-/tmp}"
readonly WHEEL_ROOT="$(mktemp -d "${TEMP_ROOT%/}/ferrum-python-wheel.XXXXXX")"
readonly CARGO_TARGET_DIR="${WHEEL_ROOT}/cargo-target"
readonly WHEEL_VENV="${WHEEL_ROOT}/venv"


#============================================
cleanup() {
	rm -rf -- "${WHEEL_ROOT}"
}


#============================================
run_step() {
	local label="$1"
	shift
	printf '\n=== %s ===\n' "${label}"
	"$@"
}


#============================================
build_wheel() {
	cd "${PYPROJECT_ROOT}"
	env CARGO_TARGET_DIR="${CARGO_TARGET_DIR}" \
		maturin build --release --locked --out "${WHEEL_ROOT}"
}


trap cleanup EXIT
source "${REPO_ROOT}/source_me.sh"

run_step "Build isolated Ferrum-Chem wheel" build_wheel

shopt -s nullglob
wheel_candidates=("${WHEEL_ROOT}"/*.whl)
if [[ "${#wheel_candidates[@]}" -ne 1 ]]; then
	printf 'wheel verification error: expected one wheel, found %s\n' \
		"${#wheel_candidates[@]}" >&2
	exit 1
fi
readonly WHEEL_PATH="${wheel_candidates[0]}"

run_step "Create isolated CPython 3.12 environment" python3 -m venv "${WHEEL_VENV}"
source "${WHEEL_VENV}/bin/activate"
unset PYTHONPATH
export PYTHONNOUSERSITE=1
run_step "Install isolated Ferrum-Chem wheel" \
	python3 -m pip install --disable-pip-version-check --no-deps "${WHEEL_PATH}"

printf '\n=== Import isolated Ferrum-Chem wheel ===\n'
python3 - "${WHEEL_VENV}" <<'PY'
from pathlib import Path
import sys

import ferrum_chem
import ferrum_chem.ferrum_chem as native


expected_root = Path(sys.argv[1]).resolve()
native_path = Path(native.__file__).resolve()
if not native_path.is_relative_to(expected_root):
	raise SystemExit(f"wheel verification imported outside isolated venv: {native_path}")
session = ferrum_chem.DocumentSession.create_empty_document_v1()
if session.snapshot().revision != 0:
	raise SystemExit("wheel verification received an invalid empty-document revision")
print(native_path)
PY
