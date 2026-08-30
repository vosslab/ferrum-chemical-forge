# Glyph-bond alignment measurement V1

## Ownership and purpose

Rust remains the owner of glyph placement, clipping, clearance, and refusal.
Qt remains a consumer of the closed V4/V2 render contract. This developer lane
does not alter either boundary. It applies an OASA/BKChem-inspired *method*—
measure the rendered result independently—using Ferrum-owned code and never
imports their runtime or source.

`devel/glyph_bond_alignment_measurement.py` accepts only a closed raster-layer
manifest, its normal composite, target core-glyph masks, final bond-footprint
masks, and fixture graph identity. The library does not accept `GlyphBounds`,
`BondAttachmentAxisV1`, clearance, or emitted clipped endpoint values.

The manifest is an untrusted developer-file boundary. The reader accepts only
its exact JSON object shape, rejects duplicate JSON keys and nonstandard JSON
numbers, requires bounded ASCII fixture IDs, and validates every bond-to-mask
relationship before it reads raster pixels. Layer paths remain below the
manifest directory, are distinct regular files with allowlisted raster suffixes,
and have per-file, total-file, layer-count, and decoded-pixel limits. These are
the implementation and documentation controls for ASVS V1.5.2, V2.1.1,
V2.1.2, V2.1.3, V2.2.1, and V2.2.3. The tool checks both JSON and diagnostic
image writes and refuses an unwritable output target.

## Current developer interface

The manifest schema is `ferrum-glyph-bond-raster-layers-v1`. It names adjacent
PNG layers and identifies every bond by its fixture source ID, endpoint atom
IDs, and style. The tool writes `alignment_metrics.json`, four annotated layer
overlays, and a four-panel per-case contact sheet under ignored
`output_glyph_alignment/`:

```text
source source_me.sh && python3 devel/glyph_bond_alignment_measurement.py --self-test
source source_me.sh && python3 devel/glyph_bond_alignment_measurement.py \
  --manifest path/to/raster_layers.json --fail-on-violation
./devel/run_glyph_bond_alignment_measurement.sh
```

Its per-endpoint metrics are inferred from actual target-core and final-bond
pixels: centerline/perpendicular error, signed target-label gap, and
non-endpoint label collision. Per-bond metrics record final-footprint presence
and composite coverage. Threshold failure is an explicit developer/E2E gate,
not a pytest lane.

## Deterministic acceptance boundary

The present commit establishes the locally implemented measurement library and
its deterministic pixel/metric snapshot plus the first 12-renderable-case
baseline. The observed maximum attachment-locus error is 20.602 pixels at the
fixed 8x raster scale, on an approved asymmetric double-bond footprint; the
supported wavy footprint reaches 4.772 pixels. These values are intentional
style footprint offsets in final ink, not label collisions or unapproved
renderer drift; the version-one maximum is therefore 24 pixels. The baseline
otherwise has full footprint coverage, positive intended-label gaps, and zero
non-endpoint core-glyph collisions. The threshold is a developer gate policy,
not renderer geometry or a published DTO value.
`--self-test` uses fixed normal, double, triple, dashed, bold, wavy, wedge,
hashed, and Haworth-front footprints. It requires exact footprint coverage,
one-pixel label gaps, a present local attachment-medial-line measurement,
third-label collision detection, and readable annotated outputs. This is the
automated acceptance gate for the measurement implementation; it replaces the
former manual-review dependency.

The default-off Rust test support now emits 8x layers for every renderable
semantic-corpus row. The ignored Rust harness deliberately writes those
developer artifacts only when explicitly invoked, so ordinary fast tests do not
mutate the checkout. Threshold changes require the same corpus's recorded
before/after JSON and the deterministic oracle to pass. The gate rejects
nonfinite, overlapping, or below-minimum intended-label gaps, and its local
attachment locus is derived from final footprint pixels instead of a
renderer-issued endpoint. No threshold or optical-anchor correction is accepted
from a number alone.

Known limit: typed refusal rows have no final bond footprint by design; their
existing Rust and installed-Qt contract tests prove omission and typed refusal,
while this pixel lane measures only accepted bond ink.
