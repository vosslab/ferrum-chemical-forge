#!/usr/bin/env bash
# Repository-owned Rust verification front door.
#
# This covers the complete Ferrum Rust workspace, including its PyO3 extension
# crate. Native wheel construction and external-adapter E2E remain separate
# because they require platform-specific packaged inputs.

set -euo pipefail

usage() {
	cat <<'USAGE'
Usage: ./check_rust.sh [-h|--help]

Runs the complete local Cargo gate:
  1. rustfmt checks for the Rust workspace
  2. normal workspace compilation, including the PyO3 extension crate
  3. strict Clippy with warnings denied
  4. workspace unit, integration, and doc tests
  5. Rust API documentation builds

This script uses a disposable repository-owned Cargo work area under build/ and
removes it on every exit. It does not build a native wheel, download/build
RDKit, run Python/Qt tests, or claim cross-platform packaging coverage.

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
REPO_ROOT="$SCRIPT_DIR"
RUST_ROOT="$REPO_ROOT/packages/ferrum-rust"
BUILD_ROOT="$REPO_ROOT/build"
CHECK_CARGO_TARGET_DIR="$BUILD_ROOT/.cargo-check-target"

if [ ! -f "$RUST_ROOT/Cargo.toml" ]; then
	echo "ERROR: missing Rust workspace manifest: $RUST_ROOT/Cargo.toml" >&2
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

cleanup_check_cargo_state() {
	rm -rf -- "$CHECK_CARGO_TARGET_DIR"
}

mkdir -p "$BUILD_ROOT"
cleanup_check_cargo_state
trap cleanup_check_cargo_state EXIT
trap 'exit 1' INT TERM HUP
export CARGO_TARGET_DIR="$CHECK_CARGO_TARGET_DIR"

RUST_HOST="$(rustc -vV | sed -n 's/^host: //p')"
echo "Ferrum Rust verification"
echo "Repository: $REPO_ROOT"
echo "Rust host: ${RUST_HOST:-unknown}"
echo "Disposable Cargo work area: $CARGO_TARGET_DIR"

run_step "Workspace formatting" "$RUST_ROOT" \
	cargo fmt --all -- --check
run_step "Workspace check" "$RUST_ROOT" \
	cargo check --workspace --locked
run_step "Workspace strict Clippy" "$RUST_ROOT" \
	cargo clippy --workspace --all-targets --locked -- -D warnings
run_step "Workspace tests" "$RUST_ROOT" \
	cargo test --workspace --locked
run_step "Workspace documentation" "$RUST_ROOT" \
	cargo doc --workspace --no-deps --locked

echo
echo "PASS: Ferrum Cargo checks and Rust tests completed."

exit 0
