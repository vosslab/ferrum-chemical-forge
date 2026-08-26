# source_me.sh - shell environment for running this repo's Python.
# Usage: source source_me.sh && python3 ...
# This is a bash script sourced into your shell, not run directly.

# Require bash: the checks below and the repo's tab-indented shell style are
# bash-specific. Fail loudly rather than misbehave under another shell.
set | grep -q '^BASH_VERSION=' || echo "use bash for your shell"
set | grep -q '^BASH_VERSION=' || exit 1

# Preserve the caller's Python search path before ~/.bashrc resets it. The local
# runtime remains first after the shell setup, while explicit caller entries
# remain available after it.
FERRUM_CALLER_PYTHONPATH="${PYTHONPATH-}"

# Source ~/.bashrc FIRST, before any repo-specific environment extension below.
# ~/.bashrc applies local shell setup (PATH, etc.) and resets some variables.
source ~/.bashrc

# Python runtime defaults: unbuffered stdout/stderr, and no .pyc/__pycache__
# files written on import. Apply these after ~/.bashrc and before the first
# extension probe so every Python process in this supported environment inherits
# the repository-required values.
export PYTHONUNBUFFERED=1
export PYTHONDONTWRITEBYTECODE=1

# Resolve this checkout from the sourced file, not the caller's working
# directory. Refuse to continue unless this interpreter can import the exact
# compiled extension staged beneath this checkout.
FERRUM_REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
FERRUM_RUNTIME_ROOT="${FERRUM_REPO_ROOT}/build/runtime/python"
FERRUM_QT_SOURCE_ROOT="${FERRUM_REPO_ROOT}/packages/ferrum-chem-qt.app"
FERRUM_EXTENSION_SUFFIX="$(python3 -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX") or "")')"
FERRUM_EXTENSION_PATH="${FERRUM_RUNTIME_ROOT}/ferrum_chem${FERRUM_EXTENSION_SUFFIX}"

if [[ -z "${FERRUM_EXTENSION_SUFFIX}" || ! -f "${FERRUM_EXTENSION_PATH}" ]]; then
	printf 'Ferrum local runtime is unavailable: %s is missing.\n' "${FERRUM_EXTENSION_PATH}" >&2
	printf 'Run ./build.sh from %s, then source source_me.sh again.\n' "${FERRUM_REPO_ROOT}" >&2
	return 1 2>/dev/null || exit 1
fi

# Re-sourcing must retain external caller entries without accumulating this
# checkout's runtime roots. Keep the local extension and Qt application source
# first so an installed ferrum_qt cannot replace the source under review.
FERRUM_FILTERED_CALLER_PYTHONPATH=""
FERRUM_PYTHONPATH_INPUT="${FERRUM_CALLER_PYTHONPATH}:"
while IFS= read -r -d ':' FERRUM_PYTHONPATH_ENTRY; do
	if [[ -z "${FERRUM_PYTHONPATH_ENTRY}" || "${FERRUM_PYTHONPATH_ENTRY}" == "${FERRUM_QT_SOURCE_ROOT}" || "${FERRUM_PYTHONPATH_ENTRY}" == "${FERRUM_RUNTIME_ROOT}" ]]; then
		continue
	fi
	FERRUM_FILTERED_CALLER_PYTHONPATH="${FERRUM_FILTERED_CALLER_PYTHONPATH:+${FERRUM_FILTERED_CALLER_PYTHONPATH}:}${FERRUM_PYTHONPATH_ENTRY}"
done <<< "${FERRUM_PYTHONPATH_INPUT}"
export PYTHONPATH="${FERRUM_QT_SOURCE_ROOT}:${FERRUM_RUNTIME_ROOT}${FERRUM_FILTERED_CALLER_PYTHONPATH:+:${FERRUM_FILTERED_CALLER_PYTHONPATH}}"

if ! python3 -c '
import pathlib
import sys
expected = pathlib.Path(sys.argv[1]).resolve()
import ferrum_chem
actual = pathlib.Path(ferrum_chem.__file__).resolve()
raise SystemExit(0 if actual == expected else 1)
' "${FERRUM_EXTENSION_PATH}"; then
	printf 'Ferrum local runtime is unavailable: the staged extension did not load from %s.\n' \
		"${FERRUM_EXTENSION_PATH}" >&2
	printf 'Run ./build.sh from %s, then source source_me.sh again.\n' "${FERRUM_REPO_ROOT}" >&2
	return 1 2>/dev/null || exit 1
fi

unset FERRUM_CALLER_PYTHONPATH
unset FERRUM_FILTERED_CALLER_PYTHONPATH
unset FERRUM_PYTHONPATH_INPUT
unset FERRUM_PYTHONPATH_ENTRY
unset FERRUM_REPO_ROOT
unset FERRUM_RUNTIME_ROOT
unset FERRUM_QT_SOURCE_ROOT
unset FERRUM_EXTENSION_SUFFIX
unset FERRUM_EXTENSION_PATH

# --- Optional: repo-root import path (disabled by default) -------------------
# Uncomment ONLY if this repo needs its repo-root modules importable when
# commands run from a subdirectory without installing the repo -- most commonly
# a repo-root package imported package-qualified (e.g. `import mypkg.thing`),
# or scripts under tools/ or tests/ that import repo-root modules.
# Must come after sourcing ~/.bashrc, which clears PYTHONPATH.
# Assumes the repo is inside a Git work tree (git rev-parse).
#REPO_ROOT="$(git rev-parse --show-toplevel)"
#export PYTHONPATH="$REPO_ROOT${PYTHONPATH:+:$PYTHONPATH}"
#unset REPO_ROOT
