#!/usr/bin/env bash
# Exercise root build.sh pair publication routing with controlled commands; never compile native sources.

set -euo pipefail

readonly SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly REAL_PYTHON="$(command -v python3)"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/ferrum-build-wrapper-XXXXXXXX")"
readonly LOGICAL_TEST_ROOT="$(cd "${TEST_ROOT}" && pwd)"
readonly PHYSICAL_TEST_ROOT="$(cd "${TEST_ROOT}" && pwd -P)"

cleanup() {
	rm -rf "${TEST_ROOT}"
}
trap cleanup EXIT

fail() {
	printf 'build wrapper E2E error: %s\n' "$1" >&2
	exit 1
}

require_contains() {
	local path="$1"
	local expected="$2"
	if ! rg -F --quiet -- "${expected}" "${path}"; then
		fail "expected ${path} to contain: ${expected}"
	fi
}

require_absent() {
	local path="$1"
	local forbidden="$2"
	if [[ ! -e "${path}" ]]; then
		return 0
	fi
	if rg -F --quiet -- "${forbidden}" "${path}"; then
		fail "expected ${path} not to contain: ${forbidden}"
	fi
}

require_native_builder_argv() {
	local name="$1"
	local selector="$2"
	local input_root="$3"
	"${REAL_PYTHON}" -c '
import pathlib
import sys

argument_path = pathlib.Path(sys.argv[1])
expected_parent = pathlib.Path(sys.argv[2]).resolve()
selector = sys.argv[3]
input_root = str(pathlib.Path(sys.argv[4]).resolve())
arguments = argument_path.read_bytes().split(b"\0")
if arguments[-1] != b"":
    raise SystemExit("builder argument record is missing its NUL terminator")
arguments.pop()
actual = [argument.decode("utf-8") for argument in arguments]
expected_length = 5 if not selector else 7
if len(actual) != expected_length:
    raise SystemExit(f"builder argument vector has {len(actual)} entries, expected {expected_length}: {actual!r}")
output_root = pathlib.Path(actual[2])
if not output_root.is_absolute() or output_root.parent.resolve() != expected_parent:
    raise SystemExit(f"builder output root is not fresh beneath its build-owned staging parent: {output_root}")
if not output_root.name.startswith("native-"):
    raise SystemExit(f"builder output root is not an invocation root: {output_root}")
expected = [
    "build",
    "--output-root",
    str(output_root),
    "--engine-bundle-dir",
    str(output_root / "ferrum-engine-bundle"),
]
if selector:
    actual[-1] = str(pathlib.Path(actual[-1]).resolve())
    expected.extend((selector, input_root))
if actual != expected:
    raise SystemExit(f"builder argument vector mismatch: {actual!r} != {expected!r}")
' "${TEST_ROOT}/${name}.log.builder-argv" "${PHYSICAL_TEST_ROOT}/build/native-staging" \
		"${selector}" "${input_root}" || fail "${name} builder argv must match the native wrapper contract"
}

run_build() {
	local name="$1"
	shift
	set +e
	FERRUM_TEST_LOG="${TEST_ROOT}/${name}.log" \
	FERRUM_TEST_MODE="${FERRUM_TEST_MODE:-good}" \
	FERRUM_TEST_DU_MODE="${FERRUM_TEST_DU_MODE:-normal}" \
	FERRUM_TEST_INTERRUPT_SIGNAL="${FERRUM_TEST_INTERRUPT_SIGNAL:-}" \
	FERRUM_TEST_ROOT="${TEST_ROOT}" \
	FERRUM_REAL_PYTHON="${REAL_PYTHON}" \
	PATH="${TEST_ROOT}/fake-bin:${PATH}" \
		"${FERRUM_TEST_BUILD_SCRIPT:-${TEST_ROOT}/build.sh}" "$@" >"${TEST_ROOT}/${name}.stdout" 2>"${TEST_ROOT}/${name}.stderr"
	local result=$?
	set -e
	if [[ "${result}" -ne 0 && "${FERRUM_TEST_DEBUG:-false}" == true ]]; then
		cat "${TEST_ROOT}/${name}.stderr" >&2
	fi
	printf '%s' "${result}"
}

mkdir -p "${TEST_ROOT}/fake-bin" "${TEST_ROOT}/packages/ferrum-rust/tools" \
	"${TEST_ROOT}/packages/ferrum-rust/crates/document/src/session" \
	"${TEST_ROOT}/packages/ferrum-chem-qt.app" \
	"${TEST_ROOT}/fixture-input"
printf original >"${TEST_ROOT}/packages/ferrum-rust/crates/document/src/session/direct_bond.rs"
cp "${SOURCE_ROOT}/build.sh" "${TEST_ROOT}/build.sh"
chmod +x "${TEST_ROOT}/build.sh"
cat >"${TEST_ROOT}/source_me.sh" <<'EOF'
export PYTHONUNBUFFERED=1
export PYTHONDONTWRITEBYTECODE=1
EOF
cat >"${TEST_ROOT}/fake-bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf 'cargo %s\n' "$*" >>"${FERRUM_TEST_LOG}"
mkdir -p "${CARGO_TARGET_DIR}/release"
: >"${CARGO_TARGET_DIR}/release/ferrum"
EOF
cat >"${TEST_ROOT}/fake-bin/maturin" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
cat >"${TEST_ROOT}/fake-bin/du" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

case "$1" in
	-sk)
		if [[ "${FERRUM_TEST_DU_MODE}" == oversize ]]; then
			printf '20971521 %s\n' "$2"
		else
			printf '1 %s\n' "$2"
		fi
		;;
	-sh)
		printf '20G %s\n' "$2"
		;;
	*)
		exit 92
		;;
esac
EOF
cat >"${TEST_ROOT}/fake-bin/python3" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail

if [[ "$1" == -B && "$2" == -c && "$3" == *native_wheel_publication* ]]; then
	printf '{}\n' >"$6"
	exit 0
fi

if [[ "$1" == -B && "$2" == */packages/ferrum-rust/tools/build_native_wheel.py && "$3" == build ]]; then
	shift 2
	printf 'builder %s\n' "$*" >>"${FERRUM_TEST_LOG}"
	printf '%s\0' "$@" >"${FERRUM_TEST_LOG}.builder-argv"
	output_root=""
	engine_bundle=""
	while [[ $# -gt 0 ]]; do
		case "$1" in
			--output-root)
				output_root="$2"
				shift 2
				;;
			--engine-bundle-dir)
				engine_bundle="$2"
				shift 2
				;;
			*)
				shift
				;;
		esac
	done
	physical_output_root="$(cd "${output_root}" && pwd -P)"
	physical_engine_parent="$(cd "$(dirname "${engine_bundle}")" && pwd -P)"
	if [[ "${physical_engine_parent}" != "${physical_output_root}" || \
		"$(basename "${engine_bundle}")" != ferrum-engine-bundle ]]; then
		printf 'builder fixture error: engine bundle must be the canonical child of output root\n' >&2
		exit 93
	fi
	mkdir -p "${FERRUM_TEST_ROOT}/build/native-source-archives/managed"
	: >"${FERRUM_TEST_ROOT}/build/native-source-archives/managed/archive"
	mkdir -p "${physical_output_root}/maturin-project"
	mkdir -p "${engine_bundle}"
	printf adapter >"${engine_bundle}/libferrum_chem.dylib"
	: >"${engine_bundle}/ferrum-engine-bundle-v1.json"
	: >"${physical_output_root}/native-wheel-build-receipt.json"
	case "${FERRUM_TEST_MODE}" in
		good|worktree_changed_at_publication_boundary)
			wheel="${physical_output_root}/ferrum_chem-test.whl"
			: >"${wheel}"
			printf '{"schema":"ferrum-native-wheel-artifact-v1","action":"wheel","artifact":"%s"}\n' "${wheel}"
			;;
		malformed)
			printf 'not json\n'
			;;
		out_of_root)
			wheel="$(dirname "${physical_output_root}")/escaped.whl"
			: >"${wheel}"
			printf '{"schema":"ferrum-native-wheel-artifact-v1","action":"wheel","artifact":"%s"}\n' "${wheel}"
			;;
		non_wheel)
			wheel="${physical_output_root}/not-a-wheel.txt"
			: >"${wheel}"
			printf '{"schema":"ferrum-native-wheel-artifact-v1","action":"wheel","artifact":"%s"}\n' "${wheel}"
			;;
		failure)
			exit 91
			;;
		worktree_changed)
			printf changed >"${FERRUM_TEST_ROOT}/packages/ferrum-rust/crates/document/src/session/direct_bond.rs"
			wheel="${physical_output_root}/ferrum_chem-test.whl"
			: >"${wheel}"
			printf '{"schema":"ferrum-native-wheel-artifact-v1","action":"wheel","artifact":"%s"}\n' "${wheel}"
			;;
		replace_lock)
			rm -rf "${FERRUM_TEST_ROOT}/build/native-build.lock"
			mkdir "${FERRUM_TEST_ROOT}/build/native-build.lock"
			printf 'pid=1\n' >"${FERRUM_TEST_ROOT}/build/native-build.lock/owner.replacement"
			exit 91
			;;
		*)
			exit 91
			;;
	esac
	exit 0
fi

if [[ "$1" == -B && "$2" == */packages/ferrum-rust/tools/build_native_wheel.py && "$3" == publish-publication ]]; then
	shift 3
	staged_source_root=""
	wheel=""
	receipt=""
	engine_bundle=""
	worktree_source_root=""
	candidate_root=""
	current_pointer=""
	qt_wheel=""
	qt_source_root=""
	qt_source_closure=""
	pair_receipt=""
	while [[ $# -gt 0 ]]; do
		case "$1" in
			--candidate-root)
				candidate_root="$2"
				shift 2
				;;
			--current-pointer)
				current_pointer="$2"
				shift 2
				;;
			--staged-source-root)
				staged_source_root="$2"
				shift 2
				;;
			--worktree-source-root)
				worktree_source_root="$2"
				shift 2
				;;
			--wheel)
				wheel="$2"
				shift 2
				;;
			--receipt)
				receipt="$2"
				shift 2
				;;
			--engine-bundle)
				engine_bundle="$2"
				shift 2
				;;
			--qt-wheel)
				qt_wheel="$2"
				shift 2
				;;
			--qt-source-root)
				qt_source_root="$2"
				shift 2
				;;
			--qt-source-closure)
				qt_source_closure="$2"
				shift 2
				;;
			--pair-receipt)
				pair_receipt="$2"
				shift 2
				;;
			*)
				printf 'builder fixture error: unsupported publish-publication option: %s\n' "$1" >&2
				exit 99
				;;
		esac
	done
	[[ "$(basename "${staged_source_root}")" == maturin-project && -d "${staged_source_root}" && -f "${wheel}" && -f "${receipt}" && -d "${engine_bundle}" && -d "${candidate_root}" && "$(dirname "${candidate_root}")" == "$(dirname "${current_pointer}")" && -f "${worktree_source_root}/crates/document/src/session/direct_bond.rs" ]] || exit 94
	[[ ! -s "${wheel}" && ! -s "${receipt}" ]] || exit 95
	[[ "$(cat "${engine_bundle}/libferrum_chem.dylib")" == adapter ]] || exit 96
	if [[ -n "${qt_wheel}${qt_source_root}${qt_source_closure}${pair_receipt}" ]]; then
		[[ -n "${qt_wheel}" && -n "${qt_source_root}" && -n "${qt_source_closure}" && -n "${pair_receipt}" && -f "${qt_wheel}" && -d "${qt_source_root}" && -f "${qt_source_closure}" && "$(dirname "${qt_wheel}")" == "${candidate_root}/wheelhouse" && "$(dirname "${pair_receipt}")" == "${candidate_root}" ]] || exit 100
		[[ "$(cat "${qt_source_closure}")" == '{}' ]] || exit 101
		: >"${pair_receipt}"
	fi
	if [[ "${FERRUM_TEST_MODE}" == worktree_changed_at_publication_boundary ]]; then
		printf changed >"${worktree_source_root}/crates/document/src/session/direct_bond.rs"
	fi
	if [[ "${FERRUM_TEST_MODE}" == worktree_changed || "${FERRUM_TEST_MODE}" == worktree_changed_at_publication_boundary ]]; then
		[[ "$(cat "${worktree_source_root}/crates/document/src/session/direct_bond.rs")" == original ]] || exit 97
	fi
	if [[ -n "${FERRUM_TEST_INTERRUPT_SIGNAL}" ]]; then
		kill "-${FERRUM_TEST_INTERRUPT_SIGNAL}" "${PPID}"
		while kill -0 "${PPID}" 2>/dev/null; do
			sleep 0.01
		done
		exit 98
	fi
	"${FERRUM_REAL_PYTHON}" -c '
import os
import pathlib
import sys

current = pathlib.Path(sys.argv[1])
candidate = sys.argv[2]
temporary = current.parent / ".fixture-current"
temporary.unlink(missing_ok=True)
temporary.symlink_to(candidate)
os.replace(temporary, current)
if not current.is_symlink() or os.readlink(current) != candidate:
    raise SystemExit("fixture current pointer did not select the validated candidate")
' "${current_pointer}" "$(basename "${candidate_root}")" || exit 98
	printf '{"action":"publish-publication","schema":"ferrum-native-wheel-artifact-v1","published":true}\n'
	exit 0
fi

if [[ "$1" == -m && "$2" == pip ]]; then
	if [[ "$3" == wheel ]]; then
		while [[ $# -gt 0 ]]; do
			if [[ "$1" == --wheel-dir ]]; then
				mkdir -p "$2"
				: >"$2/ferrum_qt-test.whl"
				break
			fi
			shift
		done
	fi
	exit 0
fi

exec "${FERRUM_REAL_PYTHON}" "$@"
EOF
chmod +x "${TEST_ROOT}/fake-bin/cargo" "${TEST_ROOT}/fake-bin/maturin" \
	"${TEST_ROOT}/fake-bin/python3" "${TEST_ROOT}/fake-bin/du"

FERRUM_TEST_DU_MODE=oversize result="$(run_build oversize wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'over-budget checkout must refuse before native build'
require_contains "${TEST_ROOT}/oversize.stderr" 'checkout exceeds the 20 GiB build budget'
require_contains "${TEST_ROOT}/oversize.stderr" 'Remediation:'
require_absent "${TEST_ROOT}/oversize.log" 'builder'
require_absent "${TEST_ROOT}/oversize.log" 'cargo'
FERRUM_TEST_DU_MODE=normal

mkdir -p "${TEST_ROOT}/build/native-build.lock" \
	"${TEST_ROOT}/build/native-staging/native-held" \
	"${TEST_ROOT}/output_native_wheel/.current-new-held" \
	"${TEST_ROOT}/output_native_wheel/current"
printf 'pid=%s\n' "$$" >"${TEST_ROOT}/build/native-build.lock/owner.live"
: >"${TEST_ROOT}/build/native-staging/native-held/sentinel"
: >"${TEST_ROOT}/output_native_wheel/.current-new-held/sentinel"
: >"${TEST_ROOT}/output_native_wheel/current/sentinel"
result="$(run_build held_lock wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'held native-build lock must reject a second native build'
require_contains "${TEST_ROOT}/held_lock.stderr" 'another native build holds the repository lock'
require_absent "${TEST_ROOT}/held_lock.log" 'builder'
require_absent "${TEST_ROOT}/held_lock.log" 'cargo'
[[ -f "${TEST_ROOT}/build/native-staging/native-held/sentinel" ]] || fail 'held lock must preserve live staging'
[[ -f "${TEST_ROOT}/output_native_wheel/.current-new-held/sentinel" ]] || fail 'held lock must preserve live candidate'
[[ -f "${TEST_ROOT}/output_native_wheel/current/sentinel" ]] || fail 'held lock must preserve current publication'
rm "${TEST_ROOT}/build/native-build.lock/owner.live"
rmdir "${TEST_ROOT}/build/native-build.lock"
rm -rf "${TEST_ROOT}/output_native_wheel"

mkdir -p "${TEST_ROOT}/build/native-build.lock"
printf 'pid=999999\n' >"${TEST_ROOT}/build/native-build.lock/owner.stale"
result="$(run_build stale_lock wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'absent lock owner must recover automatically'
require_contains "${TEST_ROOT}/stale_lock.stderr" 'Recovered stale native build lock'

FERRUM_TEST_MODE=replace_lock result="$(run_build replaced_lock wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'replacement-lock fixture must fail its original build'
[[ -f "${TEST_ROOT}/build/native-build.lock/owner.replacement" ]] || \
	fail 'old-owner cleanup must preserve a replacement lock token'
rm "${TEST_ROOT}/build/native-build.lock/owner.replacement"
rmdir "${TEST_ROOT}/build/native-build.lock"
FERRUM_TEST_MODE=good

result="$(run_build default_wheels wheels)"
[[ "${result}" -eq 0 ]] || fail 'wheels without a selector must delegate managed cache selection to the builder'
require_native_builder_argv default_wheels '' ''
[[ ! -e "${TEST_ROOT}/build/native-build.lock" ]] || fail 'successful native build must release its lock'

result="$(run_build duplicate wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input" --native-source-archive-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 2 ]] || fail 'two native selectors must fail'
require_contains "${TEST_ROOT}/duplicate.stderr" 'specify exactly one native input selector'

result="$(run_build non_native cli --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 2 ]] || fail 'selector without native target must fail'
require_contains "${TEST_ROOT}/non_native.stderr" 'valid only with all or wheels'

rm -rf "${TEST_ROOT}/output_native_wheel"
: >"${TEST_ROOT}/output_native_wheel"
result="$(run_build not_directory wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'non-directory output parent must fail'
require_contains "${TEST_ROOT}/not_directory.stderr" 'must be a directory'
require_absent "${TEST_ROOT}/not_directory.log" 'builder'
rm "${TEST_ROOT}/output_native_wheel"

mkdir -p "${TEST_ROOT}/symlink-target"
ln -s "${TEST_ROOT}/symlink-target" "${TEST_ROOT}/output_native_wheel"
result="$(run_build symlink wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'symlink output parent must fail'
require_contains "${TEST_ROOT}/symlink.stderr" 'must not be a symbolic link'
require_absent "${TEST_ROOT}/symlink.log" 'builder'
rm "${TEST_ROOT}/output_native_wheel"

FERRUM_TEST_MODE=good result="$(run_build native_good wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'one native selector must build'
require_contains "${TEST_ROOT}/native_good.stdout" "${LOGICAL_TEST_ROOT}/output_native_wheel/current/"
require_absent "${TEST_ROOT}/native_good.stdout" 'build/wheelhouse/ferrum_chem'
require_native_builder_argv native_good --sealed-input-root "${TEST_ROOT}/fixture-input"
[[ -L "${TEST_ROOT}/output_native_wheel/current" ]] || fail 'current must be an atomic publication pointer'

old_publication_name='.native-publication-old'
old_publication_root="${TEST_ROOT}/output_native_wheel/${old_publication_name}"
rm -f "${TEST_ROOT}/output_native_wheel/current"
mkdir -p "${old_publication_root}"
: >"${old_publication_root}/old-payload-sentinel"
ln -s '.native-publication-stale' "${old_publication_root}/.current-pointer-leftover"
ln -s "${old_publication_name}" "${TEST_ROOT}/output_native_wheel/current"
result="$(run_build replace_existing_current wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'native build must replace an existing current publication pointer'
new_publication_name="$(readlink "${TEST_ROOT}/output_native_wheel/current")"
[[ "${new_publication_name}" != "${old_publication_name}" ]] || \
	fail 'native build must not report success while current still selects the old payload'
[[ -f "${TEST_ROOT}/output_native_wheel/current/wheelhouse/ferrum_chem-test.whl" ]] || \
	fail 'replaced current pointer must select the newly validated wheel payload'
[[ ! -e "${old_publication_root}" ]] || \
	fail 'successful publication replacement must retire the prior immutable payload'
for pointer in "${TEST_ROOT}/output_native_wheel/${new_publication_name}"/.current-pointer-*; do
	[[ ! -e "${pointer}" && ! -L "${pointer}" ]] || \
		fail 'native current replacement must not leave a temporary pointer inside a publication payload'
done

old_publication_name="$(readlink "${TEST_ROOT}/output_native_wheel/current")"
FERRUM_TEST_MODE=worktree_changed result="$(run_build worktree_changed wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'worktree source mutation after staging must refuse publication'
require_contains "${TEST_ROOT}/worktree_changed.stderr" 'paired wheel publication failed'
[[ "$(readlink "${TEST_ROOT}/output_native_wheel/current")" == "${old_publication_name}" ]] || \
	fail 'worktree source mutation must preserve the prior current publication'
printf original >"${TEST_ROOT}/packages/ferrum-rust/crates/document/src/session/direct_bond.rs"
FERRUM_TEST_MODE=good

make_mutated_publication_build() {
	local name="$1"
	"${REAL_PYTHON}" -c '
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
destination = pathlib.Path(sys.argv[2])
text = source.read_text(encoding="utf-8")
needle = "\tPREVIOUS_NATIVE_PUBLICATION_ROOT=\"$(native_current_publication_root)\" || return 1\n\tif ! \"${PYTHON_EXECUTABLE}\" -B \"${NATIVE_WHEEL_BUILDER}\" publish-publication \\\n\t\t--candidate-root \"${publication_root}\" --current-pointer \"${NATIVE_CURRENT_OUTPUT}\" \\\n"
mutation = "\tprintf changed >\"${BUILT_QT_SOURCE_CLOSURE}\"\n"
if text.count(needle) != 1:
    raise SystemExit("paired publication mutation fixture could not find validation boundary")
destination.write_text(text.replace(needle, mutation + needle), encoding="utf-8")
' "${TEST_ROOT}/build.sh" "${TEST_ROOT}/${name}.sh" || fail 'could not create paired publication mutation fixture'
	chmod +x "${TEST_ROOT}/${name}.sh"
}

previous_target="$(readlink "${TEST_ROOT}/output_native_wheel/current")"
make_mutated_publication_build mutated_qt_source_closure
FERRUM_TEST_BUILD_SCRIPT="${TEST_ROOT}/mutated_qt_source_closure.sh" \
	result="$(run_build mutated_qt_source_closure wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'mutated Qt source closure must fail paired publication validation'
[[ "$(readlink "${TEST_ROOT}/output_native_wheel/current")" == "${previous_target}" ]] || \
	fail 'mutated Qt source closure must leave the prior current publication selected'

previous_target="$(readlink "${TEST_ROOT}/output_native_wheel/current")"
FERRUM_TEST_MODE=worktree_changed_at_publication_boundary \
	result="$(run_build worktree_changed_at_publication_boundary wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'source mutation at the former validation-to-publish boundary must refuse publication'
require_contains "${TEST_ROOT}/worktree_changed_at_publication_boundary.stderr" 'failed validation or atomic selection'
[[ "$(readlink "${TEST_ROOT}/output_native_wheel/current")" == "${previous_target}" ]] || \
	fail 'source mutation at the former validation-to-publish boundary must preserve current'
printf original >"${TEST_ROOT}/packages/ferrum-rust/crates/document/src/session/direct_bond.rs"
FERRUM_TEST_MODE=good

assert_interrupted_native_cleanup() {
	local signal_name="$1"
	local expected_status="$2"
	local name="interrupt_publisher_${signal_name}"
	local current_target
	local candidate

	FERRUM_TEST_INTERRUPT_SIGNAL="${signal_name}" \
		result="$(run_build "${name}" wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
	[[ "${result}" -eq "${expected_status}" ]] || fail "${signal_name} publisher interruption must interrupt the native build"
	[[ -L "${TEST_ROOT}/output_native_wheel/current" ]] || fail 'interruption must retain the canonical publication pointer'
	[[ -f "${TEST_ROOT}/output_native_wheel/current/wheelhouse/ferrum_chem-test.whl" ]] || \
		fail 'interruption must retain a complete publication'
	current_target="$(readlink "${TEST_ROOT}/output_native_wheel/current")"
	for candidate in "${TEST_ROOT}/output_native_wheel"/.native-publication-*; do
		[[ -e "${candidate}" ]] || continue
		[[ "$(basename "${candidate}")" == "${current_target}" ]] || \
			fail 'interruption must remove unpublished and retired publication payloads'
	done
	for candidate in "${TEST_ROOT}/build/native-staging"/native-*; do
		[[ ! -e "${candidate}" ]] || fail 'interruption must remove active native staging'
	done
	for candidate in "${TEST_ROOT}/build/qt-staging"/qt-*; do
		[[ ! -e "${candidate}" ]] || fail 'interruption must remove active Qt staging'
	done
	[[ ! -e "${TEST_ROOT}/build/native-source-archives" ]] || fail 'interruption must remove the managed archive cache'
	[[ ! -e "${TEST_ROOT}/build/native-build.lock" ]] || fail 'interruption must release the native build lock'
	for candidate in "${TEST_ROOT}/output_native_wheel"/.native-pointer-stage-*; do
		[[ ! -e "${candidate}" ]] || fail 'interruption must remove temporary pointer stages'
	done
}

for signal_name in TERM INT HUP; do
	case "${signal_name}" in
		TERM) signal_status=143 ;;
		INT) signal_status=130 ;;
		HUP) signal_status=129 ;;
	esac
	assert_interrupted_native_cleanup "${signal_name}" "${signal_status}"
done

mkdir -p "${TEST_ROOT}/output_native_wheel/native-stale" \
	"${TEST_ROOT}/output_native_wheel/.current-stale" \
	"${TEST_ROOT}/build/native-staging/native-stale" \
	"${TEST_ROOT}/build/native-source-archives/stale"
: >"${TEST_ROOT}/output_native_wheel/native-stale/obsolete"
: >"${TEST_ROOT}/output_native_wheel/.current-stale/obsolete"
: >"${TEST_ROOT}/build/native-staging/native-stale/obsolete"
: >"${TEST_ROOT}/build/native-source-archives/stale/obsolete"
result="$(run_build native_cleanup wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'successful native build must publish and clean managed native state'
[[ -f "${TEST_ROOT}/output_native_wheel/current/wheelhouse/ferrum_chem-test.whl" ]] || fail 'successful native build must publish the wheel under current'
[[ -f "${TEST_ROOT}/output_native_wheel/current/native-wheel-build-receipt.json" ]] || fail 'successful native build must publish the receipt under current'
[[ -f "${TEST_ROOT}/output_native_wheel/current/ferrum-engine-bundle/ferrum-engine-bundle-v1.json" ]] || fail 'successful native build must publish the engine bundle under current'
[[ ! -e "${TEST_ROOT}/output_native_wheel/native-stale" ]] || fail 'successful native build must prune stale publications'
[[ ! -e "${TEST_ROOT}/output_native_wheel/.current-stale" ]] || fail 'successful native build must prune stale publication worktrees'
for candidate in "${TEST_ROOT}/build/native-staging"/native-*; do
	[[ ! -e "${candidate}" ]] || fail 'successful native build must remove native staging'
done
[[ ! -e "${TEST_ROOT}/build/native-source-archives" ]] || fail 'successful native build must remove managed archive cache'
require_native_builder_argv native_cleanup --sealed-input-root "${TEST_ROOT}/fixture-input"

mkdir -p "${TEST_ROOT}/build/native-staging/native-unrelated"
: >"${TEST_ROOT}/build/native-staging/native-unrelated/retained"
FERRUM_TEST_MODE=failure result="$(run_build native_failure wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'native builder failure must fail the build'
[[ -f "${TEST_ROOT}/output_native_wheel/current/wheelhouse/ferrum_chem-test.whl" ]] || fail 'failed build must preserve current publication'
[[ ! -e "${TEST_ROOT}/build/native-staging/native-unrelated" ]] || fail 'preflight must remove stale native staging'
failure_staging_root="$("${REAL_PYTHON}" -c '
import pathlib
import sys

arguments = pathlib.Path(sys.argv[1]).read_bytes().split(b"\0")
arguments.pop()
print(arguments[2].decode("utf-8"))
' "${TEST_ROOT}/native_failure.log.builder-argv")"
[[ ! -e "${failure_staging_root}" ]] || fail 'failed build must remove its current staging root'
FERRUM_TEST_MODE=good

for mode in malformed out_of_root non_wheel; do
	FERRUM_TEST_MODE="${mode}" result="$(run_build "receipt_${mode}" wheels --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
	[[ "${result}" -ne 0 ]] || fail "${mode} receipt must fail"
	done
FERRUM_TEST_MODE=good

result="$(run_build cli cli)"
[[ "${result}" -eq 0 ]] || fail 'cli target must build'
require_contains "${TEST_ROOT}/cli.stdout" 'Run the Ferrum CLI:'
require_absent "${TEST_ROOT}/cli.stdout" 'Run the Ferrum GUI:'
require_contains "${TEST_ROOT}/cli.log" '--locked --release'

result="$(run_build wheels wheels --native-source-archive-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'wheels target must atomically build a paired publication'
require_contains "${TEST_ROOT}/wheels.stdout" 'Run the Ferrum GUI:'
require_absent "${TEST_ROOT}/wheels.stdout" 'Run the Ferrum CLI:'
require_native_builder_argv wheels --source-archive-root "${TEST_ROOT}/fixture-input"
[[ -f "${TEST_ROOT}/output_native_wheel/current/developer-wheel-publication-receipt.json" ]] || \
	fail 'paired publication must consume its Qt inputs and write its paired receipt'

result="$(run_build default_all)"
[[ "${result}" -eq 0 ]] || fail 'bare all target must delegate managed cache selection to the builder'
require_contains "${TEST_ROOT}/default_all.stdout" 'Run the Ferrum CLI:'
require_contains "${TEST_ROOT}/default_all.stdout" 'Install the matching native engine for this CLI build:'
require_contains "${TEST_ROOT}/default_all.stdout" 'Run the Ferrum GUI:'
require_native_builder_argv default_all '' ''

printf 'build.sh native wrapper E2E: PASS\n'
