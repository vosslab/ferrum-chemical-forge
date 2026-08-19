#!/usr/bin/env bash
# Repository-owned Python and Qt verification front door.
#
# Rust has its own verification boundary in check_rust.sh.  This script runs
# only Python test suites, including the installed PyO3 package and Ferrum Qt
# behavior suites; callers must install ferrum-chem before invoking it.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
PYO3_TEST_ROOT="$REPO_ROOT/packages/ferrum-rust/crates/api/python/tests"
QT_TEST_ROOT="$REPO_ROOT/packages/ferrum-chem-qt.app/tests"

if [ ! -d "$PYO3_TEST_ROOT" ]; then
	echo "ERROR: missing PyO3 Python tests: $PYO3_TEST_ROOT" >&2
	exit 1
fi
if [ ! -d "$QT_TEST_ROOT" ]; then
	echo "ERROR: missing Ferrum Qt tests: $QT_TEST_ROOT" >&2
	exit 1
fi

source "$REPO_ROOT/source_me.sh"

run_step() {
	local step_name="$1"
	shift
	echo
	echo "==> ${step_name}"
	"$@"
}

cd "$REPO_ROOT"
run_step "Repository hygiene tests" pytest "$REPO_ROOT/tests/"
run_step "Installed Ferrum-Chem Python tests" pytest "$PYO3_TEST_ROOT"
run_step "Ferrum Qt tests" env QT_QPA_PLATFORM=offscreen pytest "$QT_TEST_ROOT"

echo
echo "PASS: Ferrum Python and Qt tests completed."
