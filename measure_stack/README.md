# Ferrum measurement stack

`measure_stack/` is Ferrum-owned, pixel-only visual-quality measurement code.
It implements the OASA-style feedback method without importing OASA code or
runtime. It consumes only rendered raster layers and fixture CDML/graph
identity. It never reads renderer-issued bounds, attachment axes, clipped
endpoints, clearances, or Qt visual corrections. Rust owns geometry; Qt is an
unmodified plan consumer.

For a practical explanation of the layers, statistics, strict thresholds, and
developer commands, read
[`docs/GLYPH_BOND_MEASUREMENT.md`](../docs/GLYPH_BOND_MEASUREMENT.md).

## Closed V2 evidence

V2, `ferrum-measure-stack-raster-layers-v2`, is the acceptance contract. Each
manifest binds a fixture ID, SHA-256 of its exact CDML, a named fixed capture
profile, graph identity, expected visual relations, and SHA-256 of every PNG
layer. The required layers are the normal composite, per-atom target-core glyph,
per-atom full label, and final per-bond footprint. Paths must stay below the
manifest directory and every layer must use the declared pixel dimensions.

The V2 catalog at `fixtures/v2/fixtures.json` contains:

- 14 authoritative, renderable CDML cases from the Rust alignment corpus.
- Seven synthetic adversarial cases for detached, overlap, centerline, style,
  crop, orphan, and third-label-collision rejection predicates.

The real Qt lane captures only the 14 renderable cases through
`QGraphicsScene.render`. The synthetic runner materializes only the seven
declared negative scenes and proves each reaches its named failure category.
Positive renderer acceptance comes only from real native and Qt pixels; the
runner does not fabricate acceptance evidence for renderable CDML.

V2 is the sole accepted measurement artifact contract. The stack deliberately
rejects V1 manifests rather than silently dropping full-label, fixed-profile,
or content-hash evidence.

## Measurements and policy

`MeasurementPolicy` in `measure.py` is the one threshold authority. It reports
per bond endpoint:

- signed target-core gap, normalized to glyph height and stroke width;
- centerline/perpendicular error to the target core glyph;
- target-label overlap and non-endpoint full-label collision;
- final-footprint coverage, endpoint connectivity, and component health; and
- topology sanity for normal, double, triple, dashed, bold, wavy, solid wedge,
  hashed wedge, Haworth-front stroke, and Haworth-front wedge bonds.

At the fixed-profile scene level it also reports occupancy, margins, crop, and
orphaned declared ink. Reports contain JSON metrics, annotated overlays, and a
contact sheet. A threshold may only be changed against this fixed corpus, never
against a newly generated output.

## Permanent tests versus rebuild evidence

`source source_me.sh && python3 -m pytest measure_stack/tests -q` is the
permanent 17-test lane. It uses inline/generated arrays and `tmp_path`, stays
offline, completes in under one second, and tests manifest rejection, content
hashing, pure metric behavior, parallel optical-clearance rejection, style
classification, framing behavior, and nonfinite JSON refusal. It does not
assert the exact fixture inventory, render
Qt scenes, or publish contact sheets.

The synthetic runner, native Rust raster command, actual Qt capture, baseline,
contact sheets, upstream font/catalog census, and before/after comparisons are
explicit rebuild/developer evidence. Run them when their owned source changes;
keep their exact counts, images, and receipts out of permanent pytest.
Verify the complete local font catalog with:

```bash
source source_me.sh && python3 \
  packages/ferrum-rust/devel/verify_vendored_font_catalog.py
```

## Run the lanes

Run the native synthetic contract baseline into an empty ignored output
directory. It exits zero only when every good fixture is accepted and every
synthetic negative produces its exact expected failure category.

```bash
source source_me.sh && python3 -m measure_stack.runner \
  --output-dir output_measure_stack_runner --fail-on-violation
```

Run the actual Rust final-ink handoff. Its fixed output is
`output_glyph_alignment/v2/`. This strict gate publishes diagnostics for every
emitted manifest and one `run_summary.json` with the fixture-level violation
inventory. It exits nonzero for any finding.

```bash
devel/run_measure_stack_rust.sh
```

After `./build.sh`, run the real offscreen Qt consumer strict gate. It captures
the same 14 authoritative V2 fixtures and exits zero only when every fixture
satisfies the policy. Strict mode is the default.

```bash
devel/run_measure_stack_qt.sh --strict
```

Optional baseline mode proves capture health, fixture provenance, and the
frozen accepted-zero failure classification.

```bash
devel/run_measure_stack_qt.sh --baseline
```

The Qt commands write immutable manifests, reports, overlays, contact sheets,
and `run_summary.json` below `output_measure_stack_qt/`. Baseline mode also
writes `baseline_summary.json`. All output directories are ignored.

## Current interpretation

The contract lane is green when it proves the measuring instrument itself. Each
Qt fixture names a fixed, graph-authored `presentation` profile: profiles are
chosen from the immutable fixture graph's intended normal-scale extent and
never fitted from current rendered ink. This makes viewport composition a real
consumer-visible criterion while allowing compact, ring, and directional
fixtures to use their own declared normal framing. The
real-renderer lanes are intentionally distinct: current Rust and Qt strict
receipts both contain zero violations, and the optional Qt baseline freezes
that accepted-zero state. A future renderer change belongs in Rust glyph
anchors, endpoint clipping/style lowering, or complete-plan admission according
to the failed metric. Qt must not add a visual correction.

The V2 corpus measures Ferrum's selected byte-verified Atkinson Hyperlegible
Next Regular molecule-label face. Rust owns that role, exact tight outline
bounds, placement, and clipping; Qt only replays the issued glyphs. A future
role change updates the one selection before fresh native and Qt corpus
evidence is accepted.

## Limits

- Target-core masks identify the fixture-declared atom character; they do not
  infer chemistry from arbitrary text.
- Raster topology is a robust sanity check, not a proof of every artistic
  feature of curved or hashed ink.
- Capture composition detects crop, occupancy, and isolation; it does not by
  itself judge pedagogical usefulness.
