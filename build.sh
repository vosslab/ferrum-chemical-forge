#!/usr/bin/env bash
# Build the runnable Ferrum developer program inside this checkout.
#
# This command does not publish wheels or install anything. It creates the
# native extension and CLI below build/, then discards its compiler cache so
# repeated builds cannot accumulate abandoned target directories.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly BUILD_ROOT="${REPO_ROOT}/build"
readonly CARGO_TARGET_DIR="${BUILD_ROOT}/.cargo-target"
readonly RUST_ROOT="${REPO_ROOT}/packages/ferrum-rust"
readonly LOCAL_PYTHON_ROOT="${BUILD_ROOT}/runtime/python"
readonly LOCAL_ADAPTER="${LOCAL_PYTHON_ROOT}/.dylibs/libferrum_chem.dylib"
readonly LOCAL_ENGINE_BUNDLE="${BUILD_ROOT}/runtime/engine-v1"
readonly LOCAL_CLI="${BUILD_ROOT}/bin/ferrum"
readonly LOCAL_GUI="${BUILD_ROOT}/bin/ferrum-qt"
readonly LOCAL_RUNTIME_RECEIPT="${RUST_ROOT}/local_runtime_receipt.py"
readonly LEGACY_WHEEL_ROOT="${REPO_ROOT}/output_native_wheel"
readonly MAX_CHECKOUT_KIB=$((20 * 1024 * 1024))
readonly BUILD_LOCK_DIR="${BUILD_ROOT}/.build.lock"
readonly BUILD_LOCK_OWNER="${$}-${RANDOM}-${RANDOM}"
readonly LOCAL_BUILD_CANDIDATE="${BUILD_ROOT}/.ferrum-local-build-${BUILD_LOCK_OWNER}"
readonly CANDIDATE_PYTHON_ROOT="${LOCAL_BUILD_CANDIDATE}/runtime/python"
readonly CANDIDATE_ADAPTER="${CANDIDATE_PYTHON_ROOT}/.dylibs/libferrum_chem.dylib"
readonly CANDIDATE_ENGINE_BUNDLE="${LOCAL_BUILD_CANDIDATE}/runtime/engine-v1"
readonly CANDIDATE_CLI="${LOCAL_BUILD_CANDIDATE}/bin/ferrum"
readonly CANDIDATE_GUI="${LOCAL_BUILD_CANDIDATE}/bin/ferrum-qt"
BUILD_LOCK_HELD=0


#============================================
fail() {
	printf 'build error: %s\n' "$1" >&2
	exit 1
}


#============================================
usage() {
	cat <<'USAGE'
Usage: ./build.sh

Builds the local Ferrum program under build/:
  build/bin/ferrum       Rust CLI
  build/bin/ferrum-qt    Qt application launcher
  build/runtime/python/  local ferrum_chem extension

The build neither publishes wheels nor installs packages. Run ./all_test.sh
afterward to test this local runtime.
USAGE
}


#============================================
cleanup_transient_build_state() {
	rm -rf "${CARGO_TARGET_DIR}" "${LOCAL_BUILD_CANDIDATE}" \
		"${BUILD_ROOT}/runtime"/.native-engine-*
}


#============================================
retire_legacy_wheel_root() {
	# This root belonged to the retired wheel-publication build and is never an
	# input or output of the local runtime build. Removing the fixed checkout
	# path before measuring the checkout prevents abandoned wheel builds from
	# blocking the supported local build.
	rm -rf -- "${LEGACY_WHEEL_ROOT}"
}


#============================================
acquire_build_lock() {
	local owner="unknown"
	mkdir -p "${BUILD_ROOT}"
	if ! mkdir "${BUILD_LOCK_DIR}" 2>/dev/null; then
		if [[ -r "${BUILD_LOCK_DIR}/pid" ]]; then
			owner="$(<"${BUILD_LOCK_DIR}/pid")"
		fi
		fail "another ./build.sh invocation owns ${BUILD_ROOT} (PID ${owner}); wait or inspect the stale ${BUILD_LOCK_DIR}"
	fi
	BUILD_LOCK_HELD=1
	if ! printf '%s\n' "$$" >"${BUILD_LOCK_DIR}/pid"; then
		rmdir "${BUILD_LOCK_DIR}" || printf 'build error: cannot release failed build lock %s\n' "${BUILD_LOCK_DIR}" >&2
		BUILD_LOCK_HELD=0
		fail "cannot record local build ownership in ${BUILD_LOCK_DIR}"
	fi
	if ! printf '%s\n' "${BUILD_LOCK_OWNER}" >"${BUILD_LOCK_DIR}/owner"; then
		rm -f "${BUILD_LOCK_DIR}/pid"
		rmdir "${BUILD_LOCK_DIR}" || printf 'build error: cannot release failed build lock %s\n' "${BUILD_LOCK_DIR}" >&2
		BUILD_LOCK_HELD=0
		fail "cannot record local build owner token in ${BUILD_LOCK_DIR}"
	fi
}


#============================================
release_build_lock() {
	if (( BUILD_LOCK_HELD )); then
		if [[ ! -r "${BUILD_LOCK_DIR}/owner" ]] \
			|| [[ "$(<"${BUILD_LOCK_DIR}/owner")" != "${BUILD_LOCK_OWNER}" ]]; then
			printf 'build error: cannot release a build lock whose owner changed: %s\n' \
				"${BUILD_LOCK_DIR}" >&2
			return 1
		fi
		if ! rm "${BUILD_LOCK_DIR}/pid" "${BUILD_LOCK_DIR}/owner"; then
			printf 'build error: cannot remove this build owner records: %s\n' \
				"${BUILD_LOCK_DIR}" >&2
			return 1
		fi
		if ! rmdir "${BUILD_LOCK_DIR}"; then
			printf 'build error: cannot release nonempty build lock for inspection: %s\n' \
				"${BUILD_LOCK_DIR}" >&2
			return 1
		fi
		BUILD_LOCK_HELD=0
	fi
}


#============================================
finish_build() {
	local status="$?"
	if ! cleanup_transient_build_state; then
		status=1
	fi
	if ! release_build_lock; then
		status=1
	fi
	exit "${status}"
}


#============================================
require_checkout_budget() {
	local size_kib
	size_kib="$(du -sk "${REPO_ROOT}" | awk '{print $1}')"
	if (( size_kib > MAX_CHECKOUT_KIB )); then
		printf 'build error: checkout exceeds the 20 GiB build budget (%s).\n' \
			"$(du -sh "${REPO_ROOT}" | awk '{print $1}')" >&2
		printf 'Remove obsolete build outputs before running ./build.sh.\n' >&2
		exit 1
	fi
}


#============================================
write_gui_launcher() {
	(
		cd "${RUST_ROOT}"
		python3 -m engine_lib.local_runtime_launcher \
			--write-gui --launcher-path "${CANDIDATE_GUI}"
	)
}


#============================================
build_local_program() {
	local extension_source local_extension

	retire_legacy_wheel_root
	cleanup_transient_build_state
	require_checkout_budget
	mkdir -p "${LOCAL_BUILD_CANDIDATE}/bin"
	(
		cd "${RUST_ROOT}"
		env CARGO_TARGET_DIR="${CARGO_TARGET_DIR}" \
			cargo build --locked --release --package ferrum-api
		env CARGO_TARGET_DIR="${CARGO_TARGET_DIR}" \
			cargo build --locked --release --package ferrum-api-python
	)
	extension_source="${CARGO_TARGET_DIR}/release/libferrum_chem.dylib"
	[[ -f "${extension_source}" ]] || fail "Cargo did not produce ${extension_source}"

	source "${REPO_ROOT}/source_me.sh"
	local_extension="$(python3 "${LOCAL_RUNTIME_RECEIPT}" extension-path \
		--runtime-root "${CANDIDATE_PYTHON_ROOT}")"
	python3 "${RUST_ROOT}/local_engine_builder.py" \
		--runtime-root "${CANDIDATE_PYTHON_ROOT}"
	[[ -f "${CANDIDATE_ADAPTER}" ]] || fail "local engine build did not produce ${CANDIDATE_ADAPTER}"
	[[ -f "${CANDIDATE_ENGINE_BUNDLE}/ferrum-engine-bundle-v1.json" ]] || \
		fail "local engine build did not produce its sealed CLI bundle"

	install -m 755 "${CARGO_TARGET_DIR}/release/ferrum" "${CANDIDATE_CLI}"
	install -m 755 "${extension_source}" "${local_extension}"
	write_gui_launcher
	python3 "${LOCAL_RUNTIME_RECEIPT}" write \
		--runtime-root "${CANDIDATE_PYTHON_ROOT}"
	python3 "${LOCAL_RUNTIME_RECEIPT}" validate \
		--runtime-root "${CANDIDATE_PYTHON_ROOT}"
	promote_local_program
	cleanup_transient_build_state
	require_checkout_budget

	printf 'Built local Ferrum program:\n'
	printf '  CLI: %s\n' "${LOCAL_CLI}"
	printf '  GUI: %s\n' "${LOCAL_GUI}"
	printf '  Python runtime: %s\n' "${LOCAL_PYTHON_ROOT}"
}


#============================================
restore_saved_local_program() {
	local previous_root="$1"
	local restore_runtime="$2"
	local restore_bin="$3"
	local recovery_failed=0

	if (( restore_runtime )); then
		if ! rm -rf "${BUILD_ROOT}/runtime"; then
			printf 'build error: cannot clear candidate runtime during recovery: %s\n' \
				"${BUILD_ROOT}/runtime" >&2
			recovery_failed=1
		elif ! mv "${previous_root}/runtime" "${BUILD_ROOT}/runtime"; then
			printf 'build error: cannot restore prior runtime from %s\n' \
				"${previous_root}/runtime" >&2
			recovery_failed=1
		fi
	fi
	if (( restore_bin )); then
		if ! rm -rf "${BUILD_ROOT}/bin"; then
			printf 'build error: cannot clear candidate launchers during recovery: %s\n' \
				"${BUILD_ROOT}/bin" >&2
			recovery_failed=1
		elif ! mv "${previous_root}/bin" "${BUILD_ROOT}/bin"; then
			printf 'build error: cannot restore prior launchers from %s\n' \
				"${previous_root}/bin" >&2
			recovery_failed=1
		fi
	fi
	if (( recovery_failed )); then
		printf 'build error: local build recovery is incomplete; recover retained components from %s\n' \
			"${previous_root}" >&2
		return 1
	fi
	if ! rm -rf "${previous_root}"; then
		printf 'build error: restored the prior local program but cannot remove recovery path: %s\n' \
			"${previous_root}" >&2
		return 1
	fi
}


#============================================
promote_local_program() {
	local previous_root="${BUILD_ROOT}/.previous-local-build-${BUILD_LOCK_OWNER}"
	local saved_runtime=0
	local saved_bin=0
	[[ ! -e "${previous_root}" && ! -L "${previous_root}" ]] || \
		fail "local build recovery path already exists: ${previous_root}"
	if ! mkdir "${previous_root}"; then
		fail "cannot create local build recovery path: ${previous_root}"
	fi
	if [[ -e "${BUILD_ROOT}/runtime" || -L "${BUILD_ROOT}/runtime" ]]; then
		if ! mv "${BUILD_ROOT}/runtime" "${previous_root}/runtime"; then
			rm -rf "${previous_root}"
			fail "cannot save the existing local runtime for promotion"
		fi
		saved_runtime=1
	fi
	if [[ -e "${BUILD_ROOT}/bin" || -L "${BUILD_ROOT}/bin" ]]; then
		if ! mv "${BUILD_ROOT}/bin" "${previous_root}/bin"; then
			if ! restore_saved_local_program "${previous_root}" "${saved_runtime}" "${saved_bin}"; then
				fail "cannot save the existing local launchers for promotion; recovery is incomplete"
			fi
			fail "cannot save the existing local launchers for promotion; restored the prior local program"
		fi
		saved_bin=1
	fi
	if ! mv "${LOCAL_BUILD_CANDIDATE}/runtime" "${BUILD_ROOT}/runtime"; then
		if ! restore_saved_local_program "${previous_root}" "${saved_runtime}" "${saved_bin}"; then
			fail "cannot promote the sealed local runtime; recovery is incomplete"
		fi
		fail "cannot promote the sealed local runtime; restored the prior local program"
	fi
	if ! mv "${LOCAL_BUILD_CANDIDATE}/bin" "${BUILD_ROOT}/bin"; then
		if ! restore_saved_local_program "${previous_root}" "${saved_runtime}" "${saved_bin}"; then
			fail "cannot promote the sealed local launchers; recovery is incomplete"
		fi
		fail "cannot promote the sealed local launchers; restored the prior local program"
	fi
	if ! python3 "${LOCAL_RUNTIME_RECEIPT}" validate \
		--runtime-root "${LOCAL_PYTHON_ROOT}"; then
		if ! restore_saved_local_program "${previous_root}" "${saved_runtime}" "${saved_bin}"; then
			fail "promoted local runtime failed its receipt validation; recovery is incomplete"
		fi
		fail "promoted local runtime failed its receipt validation; restored the prior local program"
	fi
	rm -rf "${previous_root}" "${LOCAL_BUILD_CANDIDATE}"
}


case "${1:-}" in
	"")
		acquire_build_lock
		trap finish_build EXIT
		trap 'exit 1' INT TERM HUP
		build_local_program
		;;
	-h|--help)
		usage
		;;
	*)
		usage >&2
		fail "expected no arguments"
		;;
esac
