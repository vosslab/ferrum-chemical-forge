#!/usr/bin/env bash
# Create the isolated, pinned M1d oracle environment.
set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
oracle_venv="$repo_root/tests/e2e/oracle/.venv"
python3 -m venv "$oracle_venv"
"$oracle_venv/bin/python" -m pip install --upgrade pip
"$oracle_venv/bin/python" -m pip install \
	"rdkit==2026.3.4" \
	"oasa @ git+https://github.com/vosslab/bkchem-oasa.git@c6876626386e3e739c8e8a9f45c4348b2cdb926a#subdirectory=packages/oasa"
