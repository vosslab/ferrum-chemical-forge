#!/usr/bin/env bash
# Create the isolated, pinned Python RDKit reference environment.
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
reference_venv="$repo_root/tests/e2e/reference/.venv"
python3 -m venv "$reference_venv"
"$reference_venv/bin/python" -m pip install --upgrade pip
"$reference_venv/bin/python" -m pip install \
	-r "$repo_root/tests/e2e/reference/pip_requirements.txt"
