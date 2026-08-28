#!/usr/bin/env bash
# Build the runnable Ferrum developer program inside this checkout.
#
# This command does not publish wheels or install anything. It creates the
# native extension and CLI below build/, then discards its compiler cache so
# repeated builds cannot accumulate abandoned target directories.

set -euo pipefail

# Keep local-build helper imports from writing __pycache__ directories.
export PYTHONDONTWRITEBYTECODE=1

readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly BUILD_ROOT="${REPO_ROOT}/build"
readonly RUST_ROOT="${REPO_ROOT}/packages/ferrum-rust"
readonly PROGRAMS_ROOT="${BUILD_ROOT}/programs"
readonly CURRENT_PROGRAM="${BUILD_ROOT}/current"
readonly STABLE_BIN_ROOT="${BUILD_ROOT}/bin"
readonly STABLE_RUNTIME_ROOT="${BUILD_ROOT}/runtime"
readonly LOCAL_PYTHON_ROOT="${CURRENT_PROGRAM}/runtime/python"
readonly LOCAL_CLI="${CURRENT_PROGRAM}/bin/ferrum"
readonly LOCAL_GUI="${CURRENT_PROGRAM}/bin/ferrum-qt"
readonly LOCAL_RUNTIME_RECEIPT="${RUST_ROOT}/local_runtime_receipt.py"
readonly OWNED_WHEEL_OUTPUT_ROOT="${REPO_ROOT}/output_native_wheel"
readonly CLEANABLE_BUILD_CARGO_TARGET="${BUILD_ROOT}/cargo-target"
readonly CLEANABLE_RUST_TARGET="${RUST_ROOT}/target"
readonly MAX_CHECKOUT_KIB=$((20 * 1024 * 1024))
readonly BUILD_LOCK_PATH="${BUILD_ROOT}/.build.lock"
readonly BUILD_LOCK_OWNER="${$}-${RANDOM}-${RANDOM}"
readonly CARGO_TARGET_DIR="${BUILD_ROOT}/.cargo-target-${BUILD_LOCK_OWNER}"
readonly LOCAL_BUILD_CANDIDATE="${PROGRAMS_ROOT}/.staging-${BUILD_LOCK_OWNER}"
readonly CURRENT_POINTER_STAGING="${BUILD_ROOT}/.current-next-${BUILD_LOCK_OWNER}"
readonly CANDIDATE_PYTHON_ROOT="${LOCAL_BUILD_CANDIDATE}/runtime/python"
readonly CANDIDATE_ADAPTER="${CANDIDATE_PYTHON_ROOT}/.dylibs/libferrum_chem.dylib"
readonly CANDIDATE_ENGINE_BUNDLE="${LOCAL_BUILD_CANDIDATE}/runtime/engine-v1"
readonly CANDIDATE_CLI="${LOCAL_BUILD_CANDIDATE}/bin/ferrum"
readonly CANDIDATE_GUI="${LOCAL_BUILD_CANDIDATE}/bin/ferrum-qt"
readonly CANDIDATE_RUNTIME_LEASE="${LOCAL_BUILD_CANDIDATE}/.ferrum-runtime.lease"
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
	# Candidate-only cleanup must never resolve a stable path into a published root.
	rm -rf -- "${CARGO_TARGET_DIR}" "${LOCAL_BUILD_CANDIDATE}" \
		"${CURRENT_POINTER_STAGING}"
}


#============================================
clean_noncurrent_owned_build_state() {
	# These fixed paths are compiler or staging outputs owned by the local build.
	# Clean them while holding the build lock before staging a new program. A
	# direct build/bin or build/runtime directory is replaced by the current
	# program pointer and remains disposable local build output.
	if [[ ! -L "${STABLE_BIN_ROOT}" ]]; then
		rm -rf -- "${STABLE_BIN_ROOT}"
	fi
	if [[ ! -L "${STABLE_RUNTIME_ROOT}" ]]; then
		rm -rf -- "${STABLE_RUNTIME_ROOT}"
	fi
	rm -rf -- "${OWNED_WHEEL_OUTPUT_ROOT}" "${CLEANABLE_BUILD_CARGO_TARGET}" \
		"${CLEANABLE_RUST_TARGET}" \
		"${BUILD_ROOT}"/.cargo-target "${BUILD_ROOT}"/.cargo-target-* \
		"${BUILD_ROOT}"/.current-next-* \
		"${BUILD_ROOT}"/.ferrum-local-build-* \
		"${BUILD_ROOT}"/.previous-local-build-* \
		"${PROGRAMS_ROOT}"/.staging-*
}


#============================================
initialize_program_topology() {
	mkdir -p "${PROGRAMS_ROOT}"
	if [[ ! -e "${STABLE_BIN_ROOT}" && ! -L "${STABLE_BIN_ROOT}" ]]; then
		ln -s "current/bin" "${STABLE_BIN_ROOT}"
	fi
	if [[ ! -e "${STABLE_RUNTIME_ROOT}" && ! -L "${STABLE_RUNTIME_ROOT}" ]]; then
		ln -s "current/runtime" "${STABLE_RUNTIME_ROOT}"
	fi
	[[ -L "${STABLE_BIN_ROOT}" && "$(readlink "${STABLE_BIN_ROOT}")" == "current/bin" ]] \
		|| fail "local launcher root must resolve through ${CURRENT_PROGRAM}"
	[[ -L "${STABLE_RUNTIME_ROOT}" && "$(readlink "${STABLE_RUNTIME_ROOT}")" == "current/runtime" ]] \
		|| fail "local runtime root must resolve through ${CURRENT_PROGRAM}"
}


#============================================
finish_build() {
	local status="$?"
	if ! cleanup_transient_build_state; then
		status=1
	fi
	exit "${status}"
}


#============================================
require_checkout_budget() {
	local size_kib category_kib
	size_kib="$(du -sk "${REPO_ROOT}" | awk '{print $1}')"
	if (( size_kib > MAX_CHECKOUT_KIB )); then
		printf 'build error: checkout exceeds the 20 GiB build budget (%s).\n' \
			"$(du -sh "${REPO_ROOT}" | awk '{print $1}')" >&2
		printf 'Largest known build-owned categories after fixed-path cleanup:\n' >&2
		for category in "${BUILD_ROOT}" "${OWNED_WHEEL_OUTPUT_ROOT}" \
			"${CLEANABLE_RUST_TARGET}"; do
			if [[ -e "${category}" ]]; then
				category_kib="$(du -sk "${category}" | awk '{print $1}')"
				printf '  %s: %s KiB\n' "${category}" "${category_kib}" >&2
			fi
		done
		printf 'The build cleans only its fixed owned paths; inspect other checkout content separately.\n' >&2
		exit 1
	fi
}


#============================================
write_gui_launcher() {
	(
		cd "${RUST_ROOT}"
		python3 -m engine_lib.local_runtime_launcher \
			--write-gui --launcher-path "${CANDIDATE_GUI}.program"
	)
	write_runtime_lease_launcher "${CANDIDATE_GUI}" "ferrum-qt.program"
}


#============================================
write_runtime_lease_launcher() {
	local launcher_path="$1"
	local program_name="$2"

	cat >"${launcher_path}" <<EOF
#!/usr/bin/env bash
# Hold this immutable program root's shared runtime lease through exec.

set -euo pipefail

readonly PROGRAM_ROOT="\$(cd -P "\$(dirname "\${BASH_SOURCE[0]}")" && cd -P .. && pwd -P)"
readonly RUNTIME_LEASE="\${PROGRAM_ROOT}/.ferrum-runtime.lease"
readonly PROGRAM_EXECUTABLE="\${PROGRAM_ROOT}/bin/${program_name}"

[[ -f "\${RUNTIME_LEASE}" ]] || {
	printf 'ferrum local runtime lease is missing: %s\\n' "\${RUNTIME_LEASE}" >&2
	exit 1
}
[[ -x "\${PROGRAM_EXECUTABLE}" ]] || {
	printf 'ferrum local program executable is missing: %s\\n' "\${PROGRAM_EXECUTABLE}" >&2
	exit 1
}

# Perl retains this descriptor across exec only after FD_CLOEXEC is cleared.
# The exec'd CLI or Python Qt process therefore retains LOCK_SH until it exits.
exec /usr/bin/perl -e '
	use Fcntl qw(:flock F_SETFD);
	open my \$lease, "<", shift
		or die "ferrum local runtime lease cannot open: \$!\\n";
	flock(\$lease, LOCK_SH)
		or die "ferrum local runtime lease cannot lock: \$!\\n";
	fcntl(\$lease, F_SETFD, 0)
		or die "ferrum local runtime lease cannot clear close-on-exec: \$!\\n";
	exec { \$ARGV[0] } @ARGV;
	die "ferrum local program cannot exec: \$!\\n";
' "\${RUNTIME_LEASE}" "\${PROGRAM_EXECUTABLE}" "\$@"
EOF
	chmod 755 "${launcher_path}"
}


#============================================
build_local_program() {
	local extension_source local_extension

	clean_noncurrent_owned_build_state
	initialize_program_topology
	cleanup_unreachable_programs
	cleanup_transient_build_state
	require_checkout_budget
	mkdir -p "${LOCAL_BUILD_CANDIDATE}/bin"
	: >"${CANDIDATE_RUNTIME_LEASE}"
	(
		cd "${RUST_ROOT}"
		env CARGO_TARGET_DIR="${CARGO_TARGET_DIR}" \
			cargo build --locked --release --package ferrum-api
		# Extension-only link mode must not leak into workspace tests or binaries.
		env CARGO_TARGET_DIR="${CARGO_TARGET_DIR}" PYO3_BUILD_EXTENSION_MODULE=1 \
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

	install -m 755 "${CARGO_TARGET_DIR}/release/ferrum" "${CANDIDATE_CLI}.program"
	write_runtime_lease_launcher "${CANDIDATE_CLI}" "ferrum.program"
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
promote_local_program() {
	local program_root="${PROGRAMS_ROOT}/program-${BUILD_LOCK_OWNER}"

	[[ ! -e "${program_root}" && ! -L "${program_root}" ]] || \
		fail "local program root already exists: ${program_root}"
	mv "${LOCAL_BUILD_CANDIDATE}" "${program_root}"
	if [[ "${FERRUM_BUILD_KILL_AFTER_PROGRAM_RENAME:-}" == "1" ]]; then
		# Test-only crash checkpoint: the program root is now immutable but still
		# unreachable through current. The E2E kills this process group here.
		printf 'program-rename-ready\n' >&2
		while :; do sleep 1; done
	fi
	ln -s "programs/$(basename "${program_root}")" "${CURRENT_POINTER_STAGING}"
	if [[ "${FERRUM_BUILD_FAIL_AFTER_POINTER_STAGE:-}" == "1" ]]; then
		fail "injected interruption after current-pointer staging"
	fi
	python3 -c 'import os, sys; os.replace(sys.argv[1], sys.argv[2])' \
		"${CURRENT_POINTER_STAGING}" "${CURRENT_PROGRAM}"
	cleanup_inactive_programs "${program_root}"
}


#============================================
program_root_lease_is_exclusive() {
	local program_root="$1"

	/usr/bin/perl -e '
		use Errno qw(EAGAIN EWOULDBLOCK);
		use Fcntl qw(:flock);
		open my $lease, "<", shift @ARGV or exit 2;
		flock($lease, LOCK_EX | LOCK_NB) or
			exit($! == EAGAIN || $! == EWOULDBLOCK ? 1 : 2);
	' "${program_root}/.ferrum-runtime.lease"
}


#============================================
cleanup_inactive_program_root() {
	local program_root="$1"
	local runtime_lease="${program_root}/.ferrum-runtime.lease"
	local lease_status

	if [[ ! -f "${runtime_lease}" || -L "${runtime_lease}" || ! -r "${runtime_lease}" ]]; then
		printf 'Removing malformed local program without a readable regular runtime lease: %s\n' \
			"${program_root}" >&2
		rm -rf -- "${program_root}"
		return
	fi
	if program_root_lease_is_exclusive "${program_root}"; then
		rm -rf -- "${program_root}"
		return
	else
		lease_status="$?"
	fi
	if [[ "${lease_status}" == "1" ]]; then
		printf 'Retaining lease-held local program: %s\n' "${program_root}" >&2
		return
	fi
	printf 'Retaining indeterminate local program lease: %s\n' "${program_root}" >&2
}


#============================================
cleanup_unreachable_programs() {
	local selected_program=""
	local retained_program

	if [[ -e "${CURRENT_PROGRAM}" || -L "${CURRENT_PROGRAM}" ]]; then
		[[ -d "${CURRENT_PROGRAM}" ]] || \
			fail "current local program does not resolve to a directory: ${CURRENT_PROGRAM}"
		selected_program="$(cd "${CURRENT_PROGRAM}" && pwd -P)"
	fi
	for retained_program in "${PROGRAMS_ROOT}"/program-*; do
		[[ -d "${retained_program}" && ! -L "${retained_program}" ]] || continue
		[[ "$(cd "${retained_program}" && pwd -P)" == "${selected_program}" ]] && continue
		cleanup_inactive_program_root "${retained_program}"
	done
}


#============================================
cleanup_inactive_programs() {
	local selected_program="$1"
	local retained_program

	for retained_program in "${PROGRAMS_ROOT}"/program-*; do
		[[ "${retained_program}" == "${selected_program}" ]] && continue
		[[ -d "${retained_program}" && ! -L "${retained_program}" ]] || continue
		cleanup_inactive_program_root "${retained_program}"
	done
}


case "${1:-}" in
	"")
		mkdir -p "${BUILD_ROOT}"
		: >>"${BUILD_LOCK_PATH}"
		exec /usr/bin/perl -e '
			use Fcntl qw(:flock F_SETFD FD_CLOEXEC);
			open my $lock, ">>", shift @ARGV
				or die "build error: cannot open the local build lock: $!\\n";
			flock($lock, LOCK_EX | LOCK_NB)
				or die "build error: another ./build.sh invocation owns the local build; wait for it to finish\\n";
			fcntl($lock, F_SETFD, FD_CLOEXEC)
				or die "build error: cannot isolate the local build lock: $!\\n";
			my $child = fork;
			defined $child or die "build error: cannot enter the locked local build lifecycle: $!\\n";
			if ($child == 0) {
				exec @ARGV or die "build error: cannot enter the locked local build lifecycle: $!\\n";
			}
			$SIG{INT} = $SIG{TERM} = $SIG{HUP} = sub {
				kill "TERM", $child;
				exit 1;
			};
			waitpid($child, 0);
			exit($? >> 8);
		' "${BUILD_LOCK_PATH}" "${BASH_SOURCE[0]}" --locked
		;;
	--locked)
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
