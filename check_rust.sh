#!/usr/bin/env bash
# Repository-owned Rust verification front door.
#
# This covers the main Ferrum Rust workspace and the separately packaged PyO3
# workspace. Native wheel construction and external-adapter E2E remain separate
# because they require platform-specific packaged inputs.

set -euo pipefail

usage() {
	cat <<'USAGE'
Usage: ./check_rust.sh [-h|--help]

Runs the complete local Cargo gate:
  1. rustfmt checks for both Rust workspaces
  2. normal workspace compilation
  3. strict Clippy with warnings denied
  4. workspace unit, integration, and doc tests
  5. Rust API documentation builds
  6. standalone PyO3 extension compilation, Clippy, tests, and docs

This script reuses Cargo's bounded target cache. It does not run cargo clean,
build a native wheel, download/build RDKit, run Python/Qt tests, or claim
cross-platform packaging coverage.

  -h, --help  Print this help and exit 0.
USAGE
}

while [ "$#" -gt 0 ]; do
	case "$1" in
		-h|--help)
			usage
			exit 0
			;;
		*)
			echo "ERROR: unknown flag: $1" >&2
			usage >&2
			exit 2
			;;
	esac
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
RUST_ROOT="$REPO_ROOT/packages/ferrum-rust"
PYO3_ROOT="$RUST_ROOT/crates/api/python"

if [ ! -f "$RUST_ROOT/Cargo.toml" ]; then
	echo "ERROR: missing Rust workspace manifest: $RUST_ROOT/Cargo.toml" >&2
	exit 1
fi
if [ ! -f "$PYO3_ROOT/Cargo.toml" ]; then
	echo "ERROR: missing PyO3 workspace manifest: $PYO3_ROOT/Cargo.toml" >&2
	exit 1
fi
if ! command -v cargo >/dev/null 2>&1; then
	echo "ERROR: cargo not found on PATH." >&2
	echo "Install the Rust toolchain required by packages/ferrum-rust/Cargo.toml." >&2
	exit 1
fi
if ! command -v rustc >/dev/null 2>&1; then
	echo "ERROR: rustc not found on PATH." >&2
	exit 1
fi
if ! cargo fmt --version >/dev/null 2>&1; then
	echo "ERROR: rustfmt is unavailable. Run: rustup component add rustfmt" >&2
	exit 1
fi
if ! cargo clippy --version >/dev/null 2>&1; then
	echo "ERROR: Clippy is unavailable. Run: rustup component add clippy" >&2
	exit 1
fi

run_step() {
	local step_name="$1"
	local workdir="$2"
	shift 2
	echo
	echo "==> ${step_name}"
	(
		cd "$workdir"
		"$@"
	)
}

RUST_HOST="$(rustc -vV | sed -n 's/^host: //p')"
echo "Ferrum Rust verification"
echo "Repository: $REPO_ROOT"
echo "Rust host: ${RUST_HOST:-unknown}"

run_step "Main workspace formatting" "$RUST_ROOT" \
	cargo fmt --all -- --check
run_step "PyO3 workspace formatting" "$PYO3_ROOT" \
	cargo fmt --all -- --check
run_step "Main workspace check" "$RUST_ROOT" \
	cargo check --workspace --locked
run_step "Main workspace strict Clippy" "$RUST_ROOT" \
	cargo clippy --workspace --all-targets --locked -- -D warnings
run_step "Main workspace tests" "$RUST_ROOT" \
	cargo test --workspace --locked
run_step "Main workspace documentation" "$RUST_ROOT" \
	cargo doc --workspace --no-deps --locked
run_step "PyO3 workspace check" "$PYO3_ROOT" \
	cargo check --locked
run_step "PyO3 workspace strict Clippy" "$PYO3_ROOT" \
	cargo clippy --all-targets --locked -- -D warnings
run_step "PyO3 workspace tests" "$PYO3_ROOT" \
	cargo test --locked
run_step "PyO3 workspace documentation" "$PYO3_ROOT" \
	cargo doc --no-deps --locked

echo
echo "PASS: Ferrum Cargo checks and Rust tests completed."

exit 0
