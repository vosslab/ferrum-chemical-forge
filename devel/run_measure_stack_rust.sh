#!/usr/bin/env bash
# Explicit strict-red developer gate for Rust-owned V2 glyph/bond evidence.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
OUTPUT_ROOT="${REPO_ROOT}/output_glyph_alignment/v2"
RUST_ROOT="${REPO_ROOT}/packages/ferrum-rust"

rm -rf -- "${OUTPUT_ROOT}"
(
	cd "${RUST_ROOT}"
	cargo test -p ferrum-document --test atom_label_bond_alignment_corpus \
		glyph_bond_raster_handoff_emits_every_renderable_alignment_case -- --ignored
)

cd "${REPO_ROOT}"
source source_me.sh
python3 -m measure_stack.batch --manifest-root "${OUTPUT_ROOT}" --fail-on-violation
