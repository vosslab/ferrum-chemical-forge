#!/usr/bin/env bash
# Create the isolated, pinned M1d oracle environment.
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
oracle_venv="$repo_root/tests/e2e/oracle/.venv"
python3 -m venv "$oracle_venv"
"$oracle_venv/bin/python" -m pip install --upgrade pip
"$oracle_venv/bin/python" -m pip install -r "$repo_root/tests/e2e/oracle/pip_requirements.txt"
