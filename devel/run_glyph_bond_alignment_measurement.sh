#!/usr/bin/env bash
# Explicit developer gate for Rust-owned glyph/bond raster evidence.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUTPUT_ROOT="${REPO_ROOT}/output_glyph_alignment"
RUST_ROOT="${REPO_ROOT}/packages/ferrum-rust"

rm -rf -- "${OUTPUT_ROOT}"
(
	cd "${RUST_ROOT}"
	cargo test -p ferrum-document --test atom_label_bond_alignment_corpus \
		glyph_bond_raster_handoff_emits_every_renderable_alignment_case -- --ignored
)

manifest_count=0
while IFS= read -r manifest; do
	case_directory="${manifest%/glyph_bond_raster_manifest.json}"
	(
		cd "${REPO_ROOT}"
		source source_me.sh
		python3 devel/glyph_bond_alignment_measurement.py \
			--manifest "${manifest}" \
			--output-dir "${case_directory}/measurement" \
			--fail-on-violation
	)
	for artifact in alignment_metrics.json alignment_contact_sheet.png; do
		if [[ ! -s "${case_directory}/measurement/${artifact}" ]]; then
			echo "glyph alignment gate error: missing ${artifact}" >&2
			exit 1
		fi
	done
	manifest_count=$((manifest_count + 1))
done < <(find "${OUTPUT_ROOT}" -name glyph_bond_raster_manifest.json -type f | sort)

if [[ "${manifest_count}" -eq 0 ]]; then
	echo "glyph alignment gate error: Rust harness emitted no manifests" >&2
	exit 1
fi

printf 'PASS: glyph-bond alignment measurement completed for %s renderable cases.\n' \
	"${manifest_count}"
