#!/usr/bin/env bash
# Build Ferrum's local developer artifacts without installing packages.
#
# The native wheel is produced only by the source-verified builder. Compiler
# state is staged below build/; output_native_wheel/ retains only one published
# wheel, receipt, and matching CLI engine bundle.

set -euo pipefail

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly RUST_ROOT="${REPO_ROOT}/packages/ferrum-rust"
readonly NATIVE_WHEEL_BUILDER="${RUST_ROOT}/tools/build_native_wheel.py"
readonly QT_PACKAGE_ROOT="${REPO_ROOT}/packages/ferrum-chem-qt.app"
readonly BUILD_ROOT="${REPO_ROOT}/build"
readonly WHEELHOUSE="${BUILD_ROOT}/wheelhouse"
readonly BIN_DIRECTORY="${BUILD_ROOT}/bin"
readonly NATIVE_OUTPUT_PARENT="${REPO_ROOT}/output_native_wheel"
readonly NATIVE_CURRENT_OUTPUT="${NATIVE_OUTPUT_PARENT}/current"
readonly NATIVE_STAGING_PARENT="${BUILD_ROOT}/native-staging"
readonly NATIVE_MANAGED_ARCHIVE_PARENT="${BUILD_ROOT}/native-source-archives"
readonly NATIVE_BUILD_LOCK="${BUILD_ROOT}/native-build.lock"
readonly CHECKOUT_DISK_BUDGET_KIB=$((20 * 1024 * 1024))

BUILT_CLI=false
BUILT_NATIVE_WHEEL=""
BUILT_NATIVE_OUTPUT_ROOT=""
BUILT_NATIVE_ENGINE_BUNDLE=""
BUILT_QT_WHEEL=""
NATIVE_INPUT_FLAG=""
NATIVE_INPUT_ROOT=""
SHOW_HELP=false
BUILD_TARGETS=()
NATIVE_BUILD_LOCK_HELD=false
NATIVE_BUILD_LOCK_TOKEN=""
ACTIVE_NATIVE_STAGING_ROOT=""
ACTIVE_NATIVE_PUBLICATION_ROOT=""
PREVIOUS_NATIVE_PUBLICATION_ROOT=""
ACTIVE_NATIVE_PUBLICATION_PUBLISHED=false
NATIVE_BUILD_CLEANUP_RUNNING=false

usage() {
	cat <<'EOF'
Usage: ./build.sh [all|cli|native|qt]... [native-input option]

Build local Ferrum developer artifacts without installing them.

Targets:
  all     Build the CLI and both Python wheels (default).
  cli     Build the release-mode `ferrum` CLI in build/bin/.
  native  Build the source-verified `ferrum-chem` PyO3 wheel and matching engine bundle.
  qt      Build the `ferrum-qt` PySide6 wheel without dependencies.

Native input (a profile-scoped managed cache is the default for `all` or `native`):
  --native-sealed-input-root PATH
          Reuse one builder-validated native input root without downloading sources.
  --native-source-archive-root PATH
          Use one explicit local directory of pinned source archives.

With no native input option, the native builder manages hash-pinned archives below
build/native-source-archives/ for the current invocation. It fetches only missing
pinned archives, then build.sh reclaims that managed cache when the invocation exits.
Use an explicit source-archive root for reusable offline inputs.

The native builder compiles in a build-owned staging root, then build.sh validates and atomically
publishes only the wheel, receipt, and matching engine bundle in output_native_wheel/current/.
Before each native build it removes obsolete generated native output and caches; explicit native
input roots are never removed.
For the fuller offline release workflow, use packages/ferrum-rust/tools/build_release_wheelhouse.py.
See docs/NATIVE_WHEEL_BUILD.md for the native wheel lifecycle and artifact contract.
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
	if ! native_target_requested && [[ -n "${NATIVE_INPUT_FLAG}" ]]; then
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

prepare_native_staging_parent() {
	local physical_parent
	local physical_repo
	local expected_parent

	if [[ -L "${BUILD_ROOT}" ]]; then
		printf 'build error: build root must not be a symbolic link: %s\n' "${BUILD_ROOT}" >&2
		return 1
	fi
	if [[ -e "${BUILD_ROOT}" && ! -d "${BUILD_ROOT}" ]]; then
		printf 'build error: build root must be a directory: %s\n' "${BUILD_ROOT}" >&2
		return 1
	fi
	mkdir -p "${BUILD_ROOT}"
	if [[ -L "${NATIVE_STAGING_PARENT}" ]]; then
		printf 'build error: native staging parent must not be a symbolic link: %s\n' \
			"${NATIVE_STAGING_PARENT}" >&2
		return 1
	fi
	if [[ -e "${NATIVE_STAGING_PARENT}" && ! -d "${NATIVE_STAGING_PARENT}" ]]; then
		printf 'build error: native staging parent must be a directory: %s\n' \
			"${NATIVE_STAGING_PARENT}" >&2
		return 1
	fi
	mkdir -p "${NATIVE_STAGING_PARENT}"
	physical_parent="$(cd "${NATIVE_STAGING_PARENT}" && pwd -P)"
	physical_repo="$(cd "${REPO_ROOT}" && pwd -P)"
	expected_parent="${physical_repo}/build/native-staging"
	if [[ "${physical_parent}" != "${expected_parent}" ]]; then
		printf 'build error: native staging parent resolves outside the repository path: %s\n' \
			"${NATIVE_STAGING_PARENT}" >&2
		return 1
	fi
}

prepare_native_build_lock_parent() {
	local physical_build_root
	local physical_repo
	local expected_build_root

	if [[ -L "${BUILD_ROOT}" ]]; then
		printf 'build error: build root must not be a symbolic link: %s\n' "${BUILD_ROOT}" >&2
		return 1
	fi
	if [[ -e "${BUILD_ROOT}" && ! -d "${BUILD_ROOT}" ]]; then
		printf 'build error: build root must be a directory: %s\n' "${BUILD_ROOT}" >&2
		return 1
	fi
	mkdir -p "${BUILD_ROOT}"
	physical_build_root="$(cd "${BUILD_ROOT}" && pwd -P)"
	physical_repo="$(cd "${REPO_ROOT}" && pwd -P)"
	expected_build_root="${physical_repo}/build"
	if [[ "${physical_build_root}" != "${expected_build_root}" ]]; then
		printf 'build error: build root resolves outside the repository path: %s\n' "${BUILD_ROOT}" >&2
		return 1
	fi
}

release_native_build_lock() {
	local physical_lock
	local expected_lock
	local recorded_pid

	[[ "${NATIVE_BUILD_LOCK_HELD}" == true ]] || return 0
	NATIVE_BUILD_LOCK_HELD=false
	if [[ -L "${NATIVE_BUILD_LOCK}" || ! -d "${NATIVE_BUILD_LOCK}" ]]; then
		printf 'build error: native build lock changed before release: %s\n' "${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	physical_lock="$(cd "${NATIVE_BUILD_LOCK}" && pwd -P)"
	expected_lock="$(cd "${BUILD_ROOT}" && pwd -P)/native-build.lock"
	if [[ "${physical_lock}" != "${expected_lock}" ]]; then
		printf 'build error: native build lock resolves outside the repository path: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	if [[ -z "${NATIVE_BUILD_LOCK_TOKEN}" || -L "${NATIVE_BUILD_LOCK_TOKEN}" || \
		! -f "${NATIVE_BUILD_LOCK_TOKEN}" ]]; then
		printf 'build error: native build lock ownership was replaced before release; leaving it intact: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	recorded_pid="$(cat "${NATIVE_BUILD_LOCK_TOKEN}")"
	if [[ "${recorded_pid}" != "pid=$$" ]]; then
		printf 'build error: native build lock owner token does not belong to this process; leaving it intact: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	rm -f -- "${NATIVE_BUILD_LOCK_TOKEN}"
	if ! rmdir -- "${NATIVE_BUILD_LOCK}"; then
		printf 'build error: native build lock contains unexpected state at release: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
}

recover_stale_native_build_lock() {
	local physical_lock
	local expected_lock
	local owner_token
	local owner_count
	local recorded_pid

	if [[ -L "${NATIVE_BUILD_LOCK}" || ! -d "${NATIVE_BUILD_LOCK}" ]]; then
		printf 'build error: native build lock is not a physical directory: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	physical_lock="$(cd "${NATIVE_BUILD_LOCK}" && pwd -P)"
	expected_lock="$(cd "${BUILD_ROOT}" && pwd -P)/native-build.lock"
	if [[ "${physical_lock}" != "${expected_lock}" ]]; then
		printf 'build error: native build lock resolves outside the repository path: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	owner_count="$(find "${NATIVE_BUILD_LOCK}" -maxdepth 1 -type f -name 'owner.*' | wc -l | tr -d ' ')"
	if [[ "${owner_count}" != 1 ]]; then
		printf 'build error: native build lock has unknown owner metadata; refusing unsafe recovery: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	owner_token="$(find "${NATIVE_BUILD_LOCK}" -maxdepth 1 -type f -name 'owner.*' -print -quit)"
	if [[ -L "${owner_token}" || ! -f "${owner_token}" ]]; then
		printf 'build error: native build lock owner token is not a regular file: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	recorded_pid="$(cat "${owner_token}")"
	if [[ ! "${recorded_pid}" =~ ^pid=[1-9][0-9]*$ ]]; then
		printf 'build error: native build lock has unknown owner metadata; refusing unsafe recovery: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	if kill -0 "${recorded_pid#pid=}" 2>/dev/null || \
		ps -p "${recorded_pid#pid=}" >/dev/null 2>&1; then
		printf 'build error: another native build holds the repository lock; wait for it to finish and retry.\n' >&2
		return 1
	fi
	# The token name is acquisition-unique. If a new owner replaces this directory,
	# this unlink can affect only the vanished stale token, never its new token.
	rm -f -- "${owner_token}"
	if ! rmdir -- "${NATIVE_BUILD_LOCK}"; then
		printf 'build error: stale native build lock changed during recovery; leaving it intact: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	printf 'Recovered stale native build lock from absent process %s.\n' "${recorded_pid#pid=}" >&2
}

native_current_publication_root() {
	local physical_parent
	local current_target
	local current_target_path

	[[ -e "${NATIVE_CURRENT_OUTPUT}" || -L "${NATIVE_CURRENT_OUTPUT}" ]] || return 0
	if [[ ! -L "${NATIVE_CURRENT_OUTPUT}" ]]; then
		printf 'build error: native current publication must be a symbolic link: %s\n' \
			"${NATIVE_CURRENT_OUTPUT}" >&2
		return 1
	fi
	physical_parent="$(cd "${NATIVE_OUTPUT_PARENT}" && pwd -P)"
	current_target="$(readlink "${NATIVE_CURRENT_OUTPUT}")"
	if [[ "${current_target}" != .native-publication-* || "${current_target}" == */* ]]; then
		printf 'build error: native current publication has an invalid target: %s\n' \
			"${NATIVE_CURRENT_OUTPUT}" >&2
		return 1
	fi
	current_target_path="${NATIVE_OUTPUT_PARENT}/${current_target}"
	if [[ -L "${current_target_path}" || ! -d "${current_target_path}" || \
		"$(cd "${current_target_path}" && pwd -P)" != "${physical_parent}/${current_target}" ]]; then
		printf 'build error: native current publication target is not a physical child: %s\n' \
			"${NATIVE_CURRENT_OUTPUT}" >&2
		return 1
	fi
	printf '%s' "${physical_parent}/${current_target}"
}

remove_native_publication_root() {
	local publication_root="$1"
	local physical_parent
	local physical_root
	local current_root

	[[ -n "${publication_root}" && -e "${publication_root}" ]] || return 0
	if [[ -L "${publication_root}" || ! -d "${publication_root}" ]]; then
		printf 'build error: native publication candidate is not a physical directory: %s\n' \
			"${publication_root}" >&2
		return 1
	fi
	physical_parent="$(cd "${NATIVE_OUTPUT_PARENT}" && pwd -P)"
	physical_root="$(cd "${publication_root}" && pwd -P)"
	if [[ "$(dirname "${physical_root}")" != "${physical_parent}" || \
		"$(basename "${physical_root}")" != .native-publication-* ]]; then
		printf 'build error: refusing to remove unexpected native publication path: %s\n' \
			"${publication_root}" >&2
		return 1
	fi
	current_root="$(native_current_publication_root)" || return 1
	if [[ "${physical_root}" == "${current_root}" ]]; then
		return 0
	fi
	rm -rf -- "${publication_root}"
}

cleanup_active_native_build() {
	local cleanup_failed=false

	[[ "${NATIVE_BUILD_CLEANUP_RUNNING}" == false ]] || return 0
	NATIVE_BUILD_CLEANUP_RUNNING=true
	if ! remove_native_publication_root "${ACTIVE_NATIVE_PUBLICATION_ROOT}"; then
		cleanup_failed=true
	fi
	if ! remove_native_publication_root "${PREVIOUS_NATIVE_PUBLICATION_ROOT}"; then
		cleanup_failed=true
	fi
	if ! remove_native_staging_root "${ACTIVE_NATIVE_STAGING_ROOT}"; then
		cleanup_failed=true
	fi
	if ! remove_build_owned_tree "${NATIVE_MANAGED_ARCHIVE_PARENT}" \
		"$(cd "${BUILD_ROOT}" && pwd -P)/native-source-archives"; then
		cleanup_failed=true
	fi
	ACTIVE_NATIVE_STAGING_ROOT=""
	ACTIVE_NATIVE_PUBLICATION_ROOT=""
	PREVIOUS_NATIVE_PUBLICATION_ROOT=""
	ACTIVE_NATIVE_PUBLICATION_PUBLISHED=false
	if ! release_native_build_lock; then
		cleanup_failed=true
	fi
	NATIVE_BUILD_CLEANUP_RUNNING=false
	[[ "${cleanup_failed}" == false ]]
}

native_build_exit_cleanup() {
	local status="$?"

	trap - EXIT INT TERM HUP
	cleanup_active_native_build || true
	exit "${status}"
}

native_build_signal_cleanup() {
	local signal_status="$1"

	trap - EXIT INT TERM HUP
	cleanup_active_native_build || true
	exit "${signal_status}"
}

acquire_native_build_lock() {
	prepare_native_build_lock_parent
	if [[ -L "${NATIVE_BUILD_LOCK}" ]]; then
		printf 'build error: native build lock must not be a symbolic link: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	if ! mkdir "${NATIVE_BUILD_LOCK}" 2>/dev/null; then
		if ! recover_stale_native_build_lock || ! mkdir "${NATIVE_BUILD_LOCK}" 2>/dev/null; then
			printf '%s\n' 'build error: another native build holds the repository lock; wait for it to finish and retry.' >&2
			return 1
		fi
	fi
	if [[ -L "${NATIVE_BUILD_LOCK}" || ! -d "${NATIVE_BUILD_LOCK}" ]]; then
		printf 'build error: native build lock changed during acquisition: %s\n' \
			"${NATIVE_BUILD_LOCK}" >&2
		return 1
	fi
	NATIVE_BUILD_LOCK_TOKEN="$(mktemp "${NATIVE_BUILD_LOCK}/owner.XXXXXXXX")"
	printf 'pid=%s\n' "$$" >"${NATIVE_BUILD_LOCK_TOKEN}"
	NATIVE_BUILD_LOCK_HELD=true
	trap 'native_build_exit_cleanup' EXIT
	trap 'native_build_signal_cleanup 130' INT
	trap 'native_build_signal_cleanup 143' TERM
	trap 'native_build_signal_cleanup 129' HUP
}

remove_build_owned_tree() {
	local path="$1"
	local expected="$2"
	local physical_path

	if [[ -L "${path}" ]]; then
		rm -f -- "${path}"
		return
	fi
	if [[ ! -e "${path}" ]]; then
		return
	fi
	if [[ ! -d "${path}" ]]; then
		printf 'build error: managed native path is not a directory: %s\n' "${path}" >&2
		return 1
	fi
	physical_path="$(cd "${path}" && pwd -P)"
	if [[ "${physical_path}" != "${expected}" ]]; then
		printf 'build error: managed native path resolves outside its repository-owned location: %s\n' \
			"${path}" >&2
		return 1
	fi
	rm -rf -- "${path}"
}

remove_native_staging_root() {
	local staging_root="$1"
	local physical_parent
	local physical_root

	[[ -n "${staging_root}" && -e "${staging_root}" ]] || return
	physical_parent="$(cd "${NATIVE_STAGING_PARENT}" && pwd -P)"
	if [[ -L "${staging_root}" ]]; then
		rm -f -- "${staging_root}"
		return
	fi
	physical_root="$(cd "${staging_root}" && pwd -P)"
	if [[ "$(dirname "${physical_root}")" != "${physical_parent}" || \
		"$(basename "${physical_root}")" != native-* ]]; then
		printf 'build error: refusing to remove unexpected native staging path: %s\n' \
			"${staging_root}" >&2
		return 1
	fi
	rm -rf -- "${staging_root}"
}

remove_native_publication_worktrees() {
	local candidate
	local physical_parent
	local physical_candidate
	local current_target=""
	local current_target_path

	physical_parent="$(cd "${NATIVE_OUTPUT_PARENT}" && pwd -P)"
	if [[ -e "${NATIVE_CURRENT_OUTPUT}" || -L "${NATIVE_CURRENT_OUTPUT}" ]]; then
		if [[ ! -L "${NATIVE_CURRENT_OUTPUT}" ]]; then
			printf 'build error: native current publication must be a symbolic link: %s\n' \
				"${NATIVE_CURRENT_OUTPUT}" >&2
			return 1
		fi
		current_target="$(readlink "${NATIVE_CURRENT_OUTPUT}")"
		if [[ "${current_target}" != .native-publication-* || "${current_target}" == */* ]]; then
			printf 'build error: native current publication has an invalid target: %s\n' \
				"${NATIVE_CURRENT_OUTPUT}" >&2
			return 1
		fi
		current_target_path="${NATIVE_OUTPUT_PARENT}/${current_target}"
		if [[ -L "${current_target_path}" || ! -d "${current_target_path}" || \
			"$(cd "${current_target_path}" && pwd -P)" != "${physical_parent}/${current_target}" ]]; then
			printf 'build error: native current publication target is not a physical child: %s\n' \
				"${NATIVE_CURRENT_OUTPUT}" >&2
			return 1
		fi
	fi
	for candidate in "${NATIVE_OUTPUT_PARENT}"/native-* "${NATIVE_OUTPUT_PARENT}"/.current-* \
		"${NATIVE_OUTPUT_PARENT}"/.native-publication-*; do
		[[ -e "${candidate}" || -L "${candidate}" ]] || continue
		if [[ "$(basename "${candidate}")" == "${current_target}" ]]; then
			continue
		fi
		if [[ -L "${candidate}" ]]; then
			rm -f -- "${candidate}"
			continue
		fi
		if [[ ! -d "${candidate}" ]]; then
			printf 'build error: native publication is not a directory: %s\n' "${candidate}" >&2
			return 1
		fi
		physical_candidate="$(cd "${candidate}" && pwd -P)"
		if [[ "$(dirname "${physical_candidate}")" != "${physical_parent}" ]]; then
			printf 'build error: native publication worktree resolves outside its repository-owned parent: %s\n' \
				"${candidate}" >&2
			return 1
		fi
		rm -rf -- "${candidate}"
	done
}

prepare_native_build() {
	prepare_native_output_parent
	if ! remove_native_publication_worktrees || \
		! remove_build_owned_tree "${NATIVE_MANAGED_ARCHIVE_PARENT}" \
			"$(cd "${BUILD_ROOT}" && pwd -P)/native-source-archives" || \
		! remove_build_owned_tree "${NATIVE_STAGING_PARENT}" \
			"$(cd "${BUILD_ROOT}" && pwd -P)/native-staging"; then
		printf 'build error: native build preflight cleanup did not complete\n' >&2
		return 1
	fi
}

enforce_checkout_disk_budget() {
	local usage_kib
	local usage_human

	usage_kib="$(du -sk "${REPO_ROOT}" | awk 'NR == 1 { print $1 }')"
	if [[ ! "${usage_kib}" =~ ^[0-9]+$ ]]; then
		printf 'build error: could not measure checkout disk usage with du -sk\n' >&2
		return 1
	fi
	if (( usage_kib <= CHECKOUT_DISK_BUDGET_KIB )); then
		return
	fi
	usage_human="$(du -sh "${REPO_ROOT}")"
	printf 'build error: checkout exceeds the 20 GiB build budget: %s\n' "${usage_human}" >&2
	printf '%s\n' 'Remediation: remove non-source files outside output_native_wheel/current/, then rerun build.sh.' >&2
	return 1
}

install_native_publication() {
	local candidate_root="$1"
	local candidate_name
	local temporary_pointer

	candidate_name="$(basename "${candidate_root}")"
	if [[ "${candidate_name}" != .native-publication-* ]]; then
		printf 'build error: native publication candidate has an invalid name: %s\n' \
			"${candidate_root}" >&2
		return 1
	fi
	PREVIOUS_NATIVE_PUBLICATION_ROOT="$(native_current_publication_root)" || return 1
	# Rename one prepared symlink over the old pointer. Readers therefore resolve
	# either the complete old payload or the complete new payload, never a gap.
	temporary_pointer="$(mktemp "${NATIVE_OUTPUT_PARENT}/.current-pointer-XXXXXXXX")"
	rm -f -- "${temporary_pointer}"
	ln -s "${candidate_name}" "${temporary_pointer}"
	mv -f "${temporary_pointer}" "${NATIVE_CURRENT_OUTPUT}"
	ACTIVE_NATIVE_PUBLICATION_PUBLISHED=true
}

publish_native_artifacts() {
	local staging_root="$1"
	local wheel="$2"
	local engine_bundle="$3"
	local publication_root="$4"
	local receipt="${staging_root}/native-wheel-build-receipt.json"
	local publication_wheel="${publication_root}/wheelhouse/$(basename "${wheel}")"

	if [[ ! -f "${receipt}" || -L "${receipt}" ]]; then
		printf 'build error: native builder did not produce a regular build receipt: %s\n' \
			"${receipt}" >&2
		return 1
	fi
	if [[ ! -d "${engine_bundle}" || -L "${engine_bundle}" || \
		! -f "${engine_bundle}/ferrum-engine-bundle-v1.json" || \
		-L "${engine_bundle}/ferrum-engine-bundle-v1.json" ]]; then
		printf 'build error: native builder did not produce its matching regular engine bundle: %s\n' \
			"${engine_bundle}" >&2
		return 1
	fi
	mkdir -p "${publication_root}/wheelhouse"
	cp "${wheel}" "${publication_wheel}"
	cp "${receipt}" "${publication_root}/native-wheel-build-receipt.json"
	cp -R "${engine_bundle}" "${publication_root}/ferrum-engine-bundle"
	if [[ ! -f "${publication_wheel}" || \
		! -f "${publication_root}/ferrum-engine-bundle/ferrum-engine-bundle-v1.json" ]]; then
		printf 'build error: native publication is incomplete: %s\n' "${publication_root}" >&2
		return 1
	fi
	printf '%s' "${publication_wheel}"
}

validate_native_publication() {
	local staging_root="$1"
	local publication_wheel="$2"
	local publication_root="$3"
	local publication_receipt="${publication_root}/native-wheel-build-receipt.json"
	local publication_engine_bundle="${publication_root}/ferrum-engine-bundle"
	local staged_source_root="${staging_root}/maturin-project"

	if ! "${PYTHON_EXECUTABLE}" -B "${NATIVE_WHEEL_BUILDER}" validate-publication \
		--staged-source-root "${staged_source_root}" \
		--wheel "${publication_wheel}" \
		--receipt "${publication_receipt}" \
		--engine-bundle "${publication_engine_bundle}" >/dev/null; then
		printf 'build error: copied native publication failed receipt, wheel, source-closure, or engine-bundle validation\n' >&2
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
	local builder_input_root
	local builder_result
	local staging_root=""
	local staging_engine_bundle=""
	local publication_root=""
	local published_wheel=""
	printf '%s\n' 'Building source-verified Ferrum native Python wheel...'
	case "${NATIVE_INPUT_FLAG}" in
		--native-sealed-input-root)
			builder_input_flag="--sealed-input-root"
			builder_input_root="${NATIVE_INPUT_ROOT}"
			;;
		--native-source-archive-root)
			builder_input_flag="--source-archive-root"
			builder_input_root="${NATIVE_INPUT_ROOT}"
			;;
		"")
			;;
		*)
			printf 'build error: internal native input selector is invalid: %s\n' \
				"${NATIVE_INPUT_FLAG}" >&2
			return 1
			;;
	esac
	prepare_native_staging_parent
	staging_root="$(mktemp -d "${NATIVE_STAGING_PARENT}/native-XXXXXXXX")"
	ACTIVE_NATIVE_STAGING_ROOT="${staging_root}"
	BUILT_NATIVE_OUTPUT_ROOT="${staging_root}"
	staging_engine_bundle="${staging_root}/ferrum-engine-bundle"
	local -a builder_arguments=(build \
		--output-root "${staging_root}" \
		--engine-bundle-dir "${staging_engine_bundle}")
	if [[ -n "${builder_input_flag}" ]]; then
		builder_arguments+=("${builder_input_flag}" "${builder_input_root}")
	fi
	if ! builder_result="$("${PYTHON_EXECUTABLE}" -B "${NATIVE_WHEEL_BUILDER}" "${builder_arguments[@]}")"; then
		return 1
	fi
	if ! BUILT_NATIVE_WHEEL="$(parse_native_artifact_result "${builder_result}")"; then
		return 1
	fi
	publication_root="$(mktemp -d "${NATIVE_OUTPUT_PARENT}/.native-publication-XXXXXXXX")"
	ACTIVE_NATIVE_PUBLICATION_ROOT="${publication_root}"
	if ! published_wheel="$(publish_native_artifacts "${staging_root}" "${BUILT_NATIVE_WHEEL}" \
		"${staging_engine_bundle}" "${publication_root}")"; then
		return 1
	fi
	if ! validate_native_publication "${staging_root}" "${published_wheel}" "${publication_root}"; then
		return 1
	fi
	if ! install_native_publication "${publication_root}"; then
		return 1
	fi
	BUILT_NATIVE_OUTPUT_ROOT="${NATIVE_CURRENT_OUTPUT}"
	BUILT_NATIVE_WHEEL="${NATIVE_CURRENT_OUTPUT}/wheelhouse/$(basename "${published_wheel}")"
	BUILT_NATIVE_ENGINE_BUNDLE="${NATIVE_CURRENT_OUTPUT}/ferrum-engine-bundle"
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
	if native_target_requested; then
		acquire_native_build_lock
		prepare_native_build
	fi
	enforce_checkout_disk_budget

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
