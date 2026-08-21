#!/usr/bin/env bash
# Exercise root build.sh native routing with controlled commands; never compile native sources.

set -euo pipefail

readonly SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
readonly REAL_PYTHON="$(command -v python3)"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/ferrum-build-wrapper-XXXXXXXX")"
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
	"${REAL_PYTHON}" -c '
import pathlib
import sys

argument_path = pathlib.Path(sys.argv[1])
expected_parent = pathlib.Path(sys.argv[2]).resolve()
selector = sys.argv[3]
input_root = sys.argv[4]
arguments = argument_path.read_bytes().split(b"\0")
if arguments[-1] != b"":
    raise SystemExit("builder argument record is missing its NUL terminator")
arguments.pop()
actual = [argument.decode("utf-8") for argument in arguments]
if len(actual) != 7:
    raise SystemExit(f"builder argument vector has {len(actual)} entries, expected 7: {actual!r}")
output_root = pathlib.Path(actual[2])
if not output_root.is_absolute() or output_root.parent.resolve() != expected_parent:
    raise SystemExit(f"builder output root is not fresh beneath its admitted parent: {output_root}")
if not output_root.name.startswith("native-"):
    raise SystemExit(f"builder output root is not an invocation root: {output_root}")
expected = [
    "build",
    "--output-root",
    str(output_root),
    "--engine-bundle-dir",
    str(output_root / "ferrum-engine-bundle"),
    selector,
    input_root,
]
if actual != expected:
    raise SystemExit(f"builder argument vector mismatch: {actual!r} != {expected!r}")
' "${TEST_ROOT}/${name}.log.builder-argv" "${PHYSICAL_TEST_ROOT}/output_native_wheel" \
		"${selector}" "${TEST_ROOT}/fixture-input" || fail "${name} builder argv must match the native wrapper contract"
}

run_build() {
	local name="$1"
	shift
	set +e
	FERRUM_TEST_LOG="${TEST_ROOT}/${name}.log" \
	FERRUM_TEST_MODE="${FERRUM_TEST_MODE:-good}" \
	FERRUM_REAL_PYTHON="${REAL_PYTHON}" \
	PATH="${TEST_ROOT}/fake-bin:${PATH}" \
		"${TEST_ROOT}/build.sh" "$@" >"${TEST_ROOT}/${name}.stdout" 2>"${TEST_ROOT}/${name}.stderr"
	local result=$?
	set -e
	if [[ "${result}" -ne 0 && "${FERRUM_TEST_DEBUG:-false}" == true ]]; then
		cat "${TEST_ROOT}/${name}.stderr" >&2
	fi
	printf '%s' "${result}"
}

mkdir -p "${TEST_ROOT}/fake-bin" "${TEST_ROOT}/packages/ferrum-rust/tools" \
	"${TEST_ROOT}/packages/ferrum-chem-qt.app" "${TEST_ROOT}/fixture-input"
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
	mkdir -p "${engine_bundle}"
	: >"${engine_bundle}/ferrum-engine-bundle-v1.json"
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
	"${TEST_ROOT}/fake-bin/python3"

result="$(run_build missing native)"
[[ "${result}" -eq 2 ]] || fail 'native without a selector must fail'
require_contains "${TEST_ROOT}/missing.stderr" 'native requires exactly one'

result="$(run_build duplicate native --native-sealed-input-root "${TEST_ROOT}/fixture-input" --native-source-archive-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 2 ]] || fail 'two native selectors must fail'
require_contains "${TEST_ROOT}/duplicate.stderr" 'specify exactly one native input selector'

result="$(run_build non_native cli --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 2 ]] || fail 'selector without native target must fail'
require_contains "${TEST_ROOT}/non_native.stderr" 'valid only with all or native'

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
require_contains "${TEST_ROOT}/native_good.stdout" "${PHYSICAL_TEST_ROOT}/output_native_wheel/native-"
require_absent "${TEST_ROOT}/native_good.stdout" 'build/wheelhouse/ferrum_chem'
require_native_builder_argv native_good --sealed-input-root

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
require_native_builder_argv native_qt --source-archive-root

result="$(run_build all all --native-sealed-input-root "${TEST_ROOT}/fixture-input")"
[[ "${result}" -eq 0 ]] || fail 'all target must build'
require_contains "${TEST_ROOT}/all.stdout" 'Run the Ferrum CLI:'
require_contains "${TEST_ROOT}/all.stdout" 'Install the matching native engine for this CLI build:'
require_contains "${TEST_ROOT}/all.stdout" 'Run the Ferrum GUI:'

printf 'build.sh native wrapper E2E: PASS\n'
