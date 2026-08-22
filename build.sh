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
readonly LEGACY_WHEEL_ROOT="${REPO_ROOT}/output_native_wheel"
readonly MAX_CHECKOUT_KIB=$((20 * 1024 * 1024))
readonly BUILD_LOCK_DIR="${BUILD_ROOT}/.build.lock"
readonly BUILD_LOCK_OWNER="${$}-${RANDOM}-${RANDOM}"
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
	rm -rf "${CARGO_TARGET_DIR}" "${BUILD_ROOT}/runtime"/.native-engine-*
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
	cat >"${LOCAL_GUI}" <<'LAUNCHER'
#!/usr/bin/env bash
# Run the source Qt application against the extension built by ./build.sh.

set -euo pipefail

readonly BUILD_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
readonly REPO_ROOT="$(cd "${BUILD_ROOT}/.." && pwd)"
readonly LOCAL_PYTHON_ROOT="${BUILD_ROOT}/runtime/python"
readonly QT_SOURCE_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app"

	source "${REPO_ROOT}/source_me.sh"
	export PYTHONPATH="${LOCAL_PYTHON_ROOT}:${QT_SOURCE_ROOT}"
	exec python3 -m ferrum_qt "$@"
LAUNCHER
	chmod +x "${LOCAL_GUI}"
}


#============================================
build_local_program() {
	local extension_source local_extension

	retire_legacy_wheel_root
	mkdir -p "${BUILD_ROOT}/bin"
	cleanup_transient_build_state
	require_checkout_budget

	rm -rf "${LOCAL_PYTHON_ROOT}" "${LOCAL_ENGINE_BUNDLE}"
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
	local_extension="$(python3 "${RUST_ROOT}/local_runtime_receipt.py" extension-path \
		--runtime-root "${LOCAL_PYTHON_ROOT}")"
	python3 "${RUST_ROOT}/local_engine_builder.py" \
		--runtime-root "${LOCAL_PYTHON_ROOT}"
	[[ -f "${LOCAL_ADAPTER}" ]] || fail "local engine build did not produce ${LOCAL_ADAPTER}"
	[[ -f "${LOCAL_ENGINE_BUNDLE}/ferrum-engine-bundle-v1.json" ]] || \
		fail "local engine build did not produce its sealed CLI bundle"

	install -m 755 "${CARGO_TARGET_DIR}/release/ferrum" "${LOCAL_CLI}"
	install -m 755 "${extension_source}" "${local_extension}"
	write_gui_launcher
	python3 "${RUST_ROOT}/local_runtime_receipt.py" write \
		--runtime-root "${LOCAL_PYTHON_ROOT}"
	cleanup_transient_build_state
	require_checkout_budget

	printf 'Built local Ferrum program:\n'
	printf '  CLI: %s\n' "${LOCAL_CLI}"
	printf '  GUI: %s\n' "${LOCAL_GUI}"
	printf '  Python runtime: %s\n' "${LOCAL_PYTHON_ROOT}"
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
