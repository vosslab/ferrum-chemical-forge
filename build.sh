#!/usr/bin/env bash
# Build Ferrum's local developer artifacts without installing packages.
#
# The native wheel is produced only by the source-verified builder. Its fresh
# output root retains the wheel, receipt, and matching CLI engine bundle.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly RUST_ROOT="${REPO_ROOT}/packages/ferrum-rust"
readonly NATIVE_WHEEL_BUILDER="${RUST_ROOT}/tools/build_native_wheel.py"
readonly QT_PACKAGE_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app"
readonly BUILD_ROOT="${REPO_ROOT}/build"
readonly WHEELHOUSE="${BUILD_ROOT}/wheelhouse"
readonly BIN_DIRECTORY="${BUILD_ROOT}/bin"
readonly NATIVE_OUTPUT_PARENT="${REPO_ROOT}/output_native_wheel"

BUILT_CLI=false
BUILT_NATIVE_WHEEL=""
BUILT_NATIVE_OUTPUT_ROOT=""
BUILT_NATIVE_ENGINE_BUNDLE=""
BUILT_QT_WHEEL=""
NATIVE_INPUT_FLAG=""
NATIVE_INPUT_ROOT=""
SHOW_HELP=false
BUILD_TARGETS=()

usage() {
	cat <<'EOF'
Usage: ./build.sh [all|cli|native|qt]... [native-input option]

Build local Ferrum developer artifacts without installing them.

Targets:
  all     Build the CLI and both Python wheels (default; requires a native input).
  cli     Build the release-mode `ferrum` CLI in build/bin/.
  native  Build the source-verified `ferrum-chem` PyO3 wheel and matching engine bundle.
  qt      Build the `ferrum-qt` PySide6 wheel without dependencies.

Native input (required exactly once when `all` or `native` is selected):
  --native-sealed-input-root PATH
          Reuse one builder-validated native input root without downloading sources.
  --native-source-archive-root PATH
          Build from one explicit local directory of pinned source archives.

The native builder writes one fresh root below output_native_wheel/. Its JSON artifact
record selects the wheel for this invocation; build.sh never selects a shared or newest wheel.
For the fuller offline release workflow, use packages/ferrum-rust/tools/build_release_wheelhouse.py.
EOF
}

require_command() {
	local command_name="$1"
	if ! command -v "${command_name}" >/dev/null 2>&1; then
		printf 'build error: required command not found: %s\n' "${command_name}" >&2
		exit 1
	fi
}

newest_wheel() {
	local package_prefix="$1"
	local candidate
	local newest=""
	for candidate in "${WHEELHOUSE}/${package_prefix}-"*.whl; do
		if [[ ! -f "${candidate}" ]]; then
			continue
		fi
		if [[ -z "${newest}" || "${candidate}" -nt "${newest}" ]]; then
			newest="${candidate}"
		fi
	done
	if [[ -z "${newest}" ]]; then
		printf 'build error: no %s wheel was produced in %s\n' \
			"${package_prefix}" "${WHEELHOUSE}" >&2
		return 1
	fi
	printf '%s' "${newest}"
}

native_target_requested() {
	local target
	for target in "${BUILD_TARGETS[@]}"; do
		if [[ "${target}" == all || "${target}" == native ]]; then
			return 0
		fi
	done
	return 1
}

parse_arguments() {
	while [[ $# -gt 0 ]]; do
		case "$1" in
			all|cli|native|qt)
				BUILD_TARGETS+=("$1")
				;;
			--native-sealed-input-root|--native-source-archive-root)
				if [[ -n "${NATIVE_INPUT_FLAG}" ]]; then
					printf 'build error: specify exactly one native input selector\n' >&2
					return 2
				fi
				if [[ $# -lt 2 || -z "$2" ]]; then
					printf 'build error: %s requires PATH\n' "$1" >&2
					return 2
				fi
				NATIVE_INPUT_FLAG="$1"
				NATIVE_INPUT_ROOT="$2"
				shift
				;;
			-h|--help|help)
				SHOW_HELP=true
				;;
			*)
				printf 'build error: unknown target or option: %s\n\n' "$1" >&2
				usage >&2
				return 2
				;;
		esac
		shift
	done

	if [[ ${#BUILD_TARGETS[@]} -eq 0 && "${SHOW_HELP}" != true ]]; then
		BUILD_TARGETS=(all)
	fi
	if [[ "${SHOW_HELP}" == true ]]; then
		return 0
	fi
	if native_target_requested; then
		if [[ -z "${NATIVE_INPUT_FLAG}" ]]; then
			printf 'build error: native requires exactly one of --native-sealed-input-root PATH or --native-source-archive-root PATH\n' >&2
			return 2
		fi
	elif [[ -n "${NATIVE_INPUT_FLAG}" ]]; then
		printf 'build error: %s is valid only with all or native\n' "${NATIVE_INPUT_FLAG}" >&2
		return 2
	fi
}

build_cli() {
	printf '%s\n' 'Building Ferrum CLI...'
	CARGO_TARGET_DIR="${BUILD_ROOT}/cargo-target" \
		cargo build --locked --release --manifest-path "${RUST_ROOT}/Cargo.toml" --package ferrum-api
	cp "${BUILD_ROOT}/cargo-target/release/ferrum" "${BIN_DIRECTORY}/ferrum"
	BUILT_CLI=true
	printf 'Built CLI: %s\n' "${BIN_DIRECTORY}/ferrum"
}

prepare_native_output_parent() {
	local physical_parent
	local physical_repo
	local expected_parent

	# Create only the repository's direct child, then require that exact physical
	# directory before mktemp. This also handles macOS's /var -> /private/var alias.
	if [[ -L "${NATIVE_OUTPUT_PARENT}" ]]; then
		printf 'build error: native output parent must not be a symbolic link: %s\n' \
			"${NATIVE_OUTPUT_PARENT}" >&2
		return 1
	fi
	if [[ -e "${NATIVE_OUTPUT_PARENT}" && ! -d "${NATIVE_OUTPUT_PARENT}" ]]; then
		printf 'build error: native output parent must be a directory: %s\n' \
			"${NATIVE_OUTPUT_PARENT}" >&2
		return 1
	fi
	if [[ ! -e "${NATIVE_OUTPUT_PARENT}" ]]; then
		if ! mkdir "${NATIVE_OUTPUT_PARENT}"; then
			printf 'build error: could not create native output parent: %s\n' \
				"${NATIVE_OUTPUT_PARENT}" >&2
			return 1
		fi
	fi

	# Recheck after creation so a filesystem race cannot turn the expected child
	# into a symlink or non-directory between the admission checks and mktemp.
	if [[ -L "${NATIVE_OUTPUT_PARENT}" || ! -d "${NATIVE_OUTPUT_PARENT}" ]]; then
		printf 'build error: native output parent is not the required physical directory: %s\n' \
			"${NATIVE_OUTPUT_PARENT}" >&2
		return 1
	fi
	physical_parent="$(cd "${NATIVE_OUTPUT_PARENT}" && pwd -P)"
	physical_repo="$(cd "${REPO_ROOT}" && pwd -P)"
	expected_parent="${physical_repo}/output_native_wheel"
	if [[ "${physical_parent}" != "${expected_parent}" ]]; then
		printf 'build error: native output parent resolves outside the repository path: %s\n' \
			"${NATIVE_OUTPUT_PARENT}" >&2
		return 1
	fi
}

parse_native_artifact_result() {
	local result="$1"
	printf '%s' "${result}" | "${PYTHON_EXECUTABLE}" -c '
import json
import sys
from pathlib import Path

output_root = Path(sys.argv[1]).resolve()
lines = sys.stdin.read().splitlines()
if len(lines) != 1:
    raise SystemExit("build error: native builder must emit exactly one JSON artifact line")
try:
    record = json.loads(lines[0])
except json.JSONDecodeError as error:
    raise SystemExit(f"build error: native builder emitted invalid JSON: {error.msg}") from error
if not isinstance(record, dict):
    raise SystemExit("build error: native builder artifact result must be a JSON object")
if record.get("schema") != "ferrum-native-wheel-artifact-v1" or record.get("action") != "wheel":
    raise SystemExit("build error: native builder artifact result has the wrong schema or action")
artifact_value = record.get("artifact")
if not isinstance(artifact_value, str):
    raise SystemExit("build error: native builder artifact result has no wheel path")
artifact = Path(artifact_value)
if not artifact.is_absolute():
    raise SystemExit("build error: native builder wheel path must be absolute")
try:
    resolved = artifact.resolve(strict=True)
except FileNotFoundError as error:
    raise SystemExit(f"build error: native builder reported a missing wheel: {artifact}") from error
if artifact != resolved or not resolved.is_relative_to(output_root) or not resolved.is_file():
    raise SystemExit("build error: native builder wheel path is not a regular file beneath its fresh output root")
if resolved.suffix != ".whl":
    raise SystemExit("build error: native builder artifact is not a wheel")
print(resolved)
' "${BUILT_NATIVE_OUTPUT_ROOT}"
}

build_native() {
	local builder_input_flag
	local builder_result
	printf '%s\n' 'Building source-verified Ferrum native Python wheel...'
	prepare_native_output_parent
	BUILT_NATIVE_OUTPUT_ROOT="$(mktemp -d "${NATIVE_OUTPUT_PARENT}/native-XXXXXXXX")"
	BUILT_NATIVE_ENGINE_BUNDLE="${BUILT_NATIVE_OUTPUT_ROOT}/ferrum-engine-bundle"
	case "${NATIVE_INPUT_FLAG}" in
		--native-sealed-input-root)
			builder_input_flag="--sealed-input-root"
			;;
		--native-source-archive-root)
			builder_input_flag="--source-archive-root"
			;;
		*)
			printf 'build error: internal native input selector is invalid: %s\n' \
				"${NATIVE_INPUT_FLAG}" >&2
			return 1
			;;
	esac
	builder_result="$("${PYTHON_EXECUTABLE}" -B "${NATIVE_WHEEL_BUILDER}" build \
		--output-root "${BUILT_NATIVE_OUTPUT_ROOT}" \
		--engine-bundle-dir "${BUILT_NATIVE_ENGINE_BUNDLE}" \
		"${builder_input_flag}" "${NATIVE_INPUT_ROOT}")"
	BUILT_NATIVE_WHEEL="$(parse_native_artifact_result "${builder_result}")"
	if [[ ! -f "${BUILT_NATIVE_ENGINE_BUNDLE}/ferrum-engine-bundle-v1.json" ]]; then
		printf 'build error: native builder did not produce its matching engine bundle: %s\n' \
			"${BUILT_NATIVE_ENGINE_BUNDLE}" >&2
		return 1
	fi
	printf 'Built native wheel: %s\n' "${BUILT_NATIVE_WHEEL}"
	printf 'Built matching engine bundle: %s\n' "${BUILT_NATIVE_ENGINE_BUNDLE}"
}

build_qt() {
	printf '%s\n' 'Building Ferrum Qt Python wheel...'
	"${PYTHON_EXECUTABLE}" -m pip wheel --no-deps --no-build-isolation \
		--wheel-dir "${WHEELHOUSE}" "${QT_PACKAGE_ROOT}"
	BUILT_QT_WHEEL="$(newest_wheel ferrum_qt)"
	printf 'Built Qt wheel: %s\n' "${BUILT_QT_WHEEL}"
}

show_next_steps() {
	if [[ "${BUILT_CLI}" == true ]]; then
		printf '\nRun the Ferrum CLI:\n'
		printf '  %q --help\n' "${BIN_DIRECTORY}/ferrum"
		printf '  %q inspect drawing.cdml\n' "${BIN_DIRECTORY}/ferrum"
	fi

	if [[ -n "${BUILT_NATIVE_WHEEL}" && "${BUILT_CLI}" == true ]]; then
		printf '\nInstall the matching native engine for this CLI build:\n'
		printf '  %q engine install %q\n' "${BIN_DIRECTORY}/ferrum" "${BUILT_NATIVE_ENGINE_BUNDLE}"
		printf '  %q engine status\n' "${BIN_DIRECTORY}/ferrum"
	fi

	if [[ -n "${BUILT_NATIVE_WHEEL}" && -n "${BUILT_QT_WHEEL}" ]]; then
		printf '\nRun the Ferrum GUI:\n'
		printf '  source source_me.sh && %q -m pip install --force-reinstall --no-deps %q %q\n' \
			"${PYTHON_EXECUTABLE}" "${BUILT_NATIVE_WHEEL}" "${BUILT_QT_WHEEL}"
		printf '  ferrum-qt\n'
		printf '  # PySide6 must already be available in this Python 3.12 environment.\n'
	fi
}

main() {
	cd "${REPO_ROOT}"
	parse_arguments "$@"
	if [[ "${SHOW_HELP}" == true ]]; then
		usage
		return 0
	fi

	# source_me.sh establishes the repository's Python 3.12 execution contract.
	# shellcheck disable=SC1091
	source "${REPO_ROOT}/source_me.sh"
	readonly PYTHON_EXECUTABLE="$(command -v python3)"
	export PYTHON_EXECUTABLE

	local target
	for target in "${BUILD_TARGETS[@]}"; do
		case "${target}" in
			all)
				require_command cargo
				require_command maturin
				"${PYTHON_EXECUTABLE}" -m pip --version >/dev/null
				mkdir -p "${BIN_DIRECTORY}" "${WHEELHOUSE}"
				build_cli
				build_native
				build_qt
				;;
			cli)
				require_command cargo
				mkdir -p "${BIN_DIRECTORY}"
				build_cli
				;;
			native)
				require_command cargo
				require_command maturin
				"${PYTHON_EXECUTABLE}" -m pip --version >/dev/null
				build_native
				;;
			qt)
				"${PYTHON_EXECUTABLE}" -m pip --version >/dev/null
				mkdir -p "${WHEELHOUSE}"
				build_qt
				;;
		esac
	done

	show_next_steps
}

main "$@"
