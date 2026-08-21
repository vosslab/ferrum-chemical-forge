#!/usr/bin/env bash
# Exercise root build.sh native routing with controlled commands; never compile native sources.

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
	"${TEST_ROOT}/packages/ferrum-chem-qt.app" \
	"${TEST_ROOT}/fixture-input"
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

if [[ "$1" == -B && "$2" == */packages/ferrum-rust/tools/build_native_wheel.py ]]; then
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
	mkdir -p "${FERRUM_TEST_ROOT}/build/native-source-archives/managed"
	: >"${FERRUM_TEST_ROOT}/build/native-source-archives/managed/archive"
	mkdir -p "${engine_bundle}"
	: >"${engine_bundle}/ferrum-engine-bundle-v1.json"
	: >"${physical_output_root}/native-wheel-build-receipt.json"
	case "${FERRUM_TEST_MODE}" in
		good)
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

FERRUM_TEST_DU_MODE=oversize result="$(run_build oversize native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
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
result="$(run_build held_lock native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
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
result="$(run_build stale_lock native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'absent lock owner must recover automatically'
require_contains "${TEST_ROOT}/stale_lock.stderr" 'Recovered stale native build lock'

FERRUM_TEST_MODE=replace_lock result="$(run_build replaced_lock native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'replacement-lock fixture must fail its original build'
[[ -f "${TEST_ROOT}/build/native-build.lock/owner.replacement" ]] || \
	fail 'old-owner cleanup must preserve a replacement lock token'
rm "${TEST_ROOT}/build/native-build.lock/owner.replacement"
rmdir "${TEST_ROOT}/build/native-build.lock"
FERRUM_TEST_MODE=good

result="$(run_build default_native native)"
[[ "${result}" -eq 0 ]] || fail 'native without a selector must delegate managed cache selection to the builder'
require_native_builder_argv default_native '' ''
[[ ! -e "${TEST_ROOT}/build/native-build.lock" ]] || fail 'successful native build must release its lock'

result="$(run_build duplicate native --native-sealed-input-root "${TEST_ROOT}/fixture-input" --native-source-archive-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 2 ]] || fail 'two native selectors must fail'
require_contains "${TEST_ROOT}/duplicate.stderr" 'specify exactly one native input selector'

result="$(run_build non_native cli --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 2 ]] || fail 'selector without native target must fail'
require_contains "${TEST_ROOT}/non_native.stderr" 'valid only with all or native'

rm -rf "${TEST_ROOT}/output_native_wheel"
: >"${TEST_ROOT}/output_native_wheel"
result="$(run_build not_directory native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'non-directory output parent must fail'
require_contains "${TEST_ROOT}/not_directory.stderr" 'must be a directory'
require_absent "${TEST_ROOT}/not_directory.log" 'builder'
rm "${TEST_ROOT}/output_native_wheel"

mkdir -p "${TEST_ROOT}/symlink-target"
ln -s "${TEST_ROOT}/symlink-target" "${TEST_ROOT}/output_native_wheel"
result="$(run_build symlink native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -ne 0 ]] || fail 'symlink output parent must fail'
require_contains "${TEST_ROOT}/symlink.stderr" 'must not be a symbolic link'
require_absent "${TEST_ROOT}/symlink.log" 'builder'
rm "${TEST_ROOT}/output_native_wheel"

FERRUM_TEST_MODE=good result="$(run_build native_good native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'one native selector must build'
require_contains "${TEST_ROOT}/native_good.stdout" "${LOGICAL_TEST_ROOT}/output_native_wheel/current/"
require_absent "${TEST_ROOT}/native_good.stdout" 'build/wheelhouse/ferrum_chem'
require_native_builder_argv native_good --sealed-input-root "${TEST_ROOT}/fixture-input"
[[ -L "${TEST_ROOT}/output_native_wheel/current" ]] || fail 'current must be an atomic publication pointer'

make_interrupted_build() {
	local name="$1"
	local interruption="$2"
	local signal_name="$3"
	"${REAL_PYTHON}" -c '
import pathlib
import sys

source = pathlib.Path(sys.argv[1])
destination = pathlib.Path(sys.argv[2])
interruption = sys.argv[3]
text = source.read_text(encoding="utf-8")
needle = "\tmv -f \"${temporary_pointer}\" \"${NATIVE_CURRENT_OUTPUT}\""
if interruption == "before":
    replacement = f"\tkill -{sys.argv[4]} $$\n" + needle
else:
    replacement = needle + f"\n\tkill -{sys.argv[4]} $$"
if text.count(needle) != 1:
    raise SystemExit("interruption fixture could not find the pointer replacement boundary")
destination.write_text(text.replace(needle, replacement), encoding="utf-8")
' "${TEST_ROOT}/build.sh" "${TEST_ROOT}/${name}.sh" "${interruption}" "${signal_name}" || fail 'could not create interruption fixture'
	chmod +x "${TEST_ROOT}/${name}.sh"
}

assert_interrupted_native_cleanup() {
	local phase="$1"
	local signal_name="$2"
	local expected_status="$3"
	local name="interrupt_${phase}_${signal_name}"
	local current_target
	local candidate

	make_interrupted_build "${name}" "${phase}" "${signal_name}"
	FERRUM_TEST_BUILD_SCRIPT="${TEST_ROOT}/${name}.sh" \
		result="$(run_build "${name}" native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
	[[ "${result}" -eq "${expected_status}" ]] || fail "${signal_name} ${phase} pointer replacement must interrupt the native build"
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
	[[ ! -e "${TEST_ROOT}/build/native-source-archives" ]] || fail 'interruption must remove the managed archive cache'
}

for signal_name in TERM INT HUP; do
	case "${signal_name}" in
		TERM) signal_status=143 ;;
		INT) signal_status=130 ;;
		HUP) signal_status=129 ;;
	esac
	assert_interrupted_native_cleanup before "${signal_name}" "${signal_status}"
	assert_interrupted_native_cleanup after "${signal_name}" "${signal_status}"
done
FERRUM_TEST_BUILD_SCRIPT=""

mkdir -p "${TEST_ROOT}/output_native_wheel/native-stale" \
	"${TEST_ROOT}/output_native_wheel/.current-stale" \
	"${TEST_ROOT}/build/native-staging/native-stale" \
	"${TEST_ROOT}/build/native-source-archives/stale"
: >"${TEST_ROOT}/output_native_wheel/native-stale/obsolete"
: >"${TEST_ROOT}/output_native_wheel/.current-stale/obsolete"
: >"${TEST_ROOT}/build/native-staging/native-stale/obsolete"
: >"${TEST_ROOT}/build/native-source-archives/stale/obsolete"
result="$(run_build native_cleanup native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
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
FERRUM_TEST_MODE=failure result="$(run_build native_failure native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
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
	FERRUM_TEST_MODE="${mode}" result="$(run_build "receipt_${mode}" native --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
	[[ "${result}" -ne 0 ]] || fail "${mode} receipt must fail"
	done
FERRUM_TEST_MODE=good

result="$(run_build cli cli)"
[[ "${result}" -eq 0 ]] || fail 'cli target must build'
require_contains "${TEST_ROOT}/cli.stdout" 'Run the Ferrum CLI:'
require_absent "${TEST_ROOT}/cli.stdout" 'Run the Ferrum GUI:'
require_contains "${TEST_ROOT}/cli.log" '--locked --release'

result="$(run_build qt qt)"
[[ "${result}" -eq 0 ]] || fail 'qt target must build'
require_absent "${TEST_ROOT}/qt.stdout" 'Run the Ferrum GUI:'

result="$(run_build native_qt native qt --native-source-archive-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'native qt targets must build'
require_contains "${TEST_ROOT}/native_qt.stdout" 'Run the Ferrum GUI:'
require_absent "${TEST_ROOT}/native_qt.stdout" 'Run the Ferrum CLI:'
require_native_builder_argv native_qt --source-archive-root "${TEST_ROOT}/fixture-input"

result="$(run_build default_all)"
[[ "${result}" -eq 0 ]] || fail 'bare all target must delegate managed cache selection to the builder'
require_contains "${TEST_ROOT}/default_all.stdout" 'Run the Ferrum CLI:'
require_contains "${TEST_ROOT}/default_all.stdout" 'Install the matching native engine for this CLI build:'
require_contains "${TEST_ROOT}/default_all.stdout" 'Run the Ferrum GUI:'
require_native_builder_argv default_all '' ''

printf 'build.sh native wrapper E2E: PASS\n'
