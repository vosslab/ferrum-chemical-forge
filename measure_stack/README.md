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

- 12 authoritative, renderable CDML cases from the Rust alignment corpus.
- Seven synthetic adversarial cases for detached, overlap, centerline, style,
  crop, orphan, and third-label-collision rejection predicates.

The real Qt lane captures only the 12 renderable cases through
`QGraphicsScene.render`. The synthetic runner is a native measurement-contract
test: it materializes deliberately controlled pixels to prove each predicate
accepts the positive corpus and rejects its named failure mode. It is not a
renderer-quality claim.

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

## Run the lanes

Run the native synthetic contract baseline into an empty ignored output
directory. It exits zero only when every good fixture is accepted and every
synthetic negative produces its exact expected failure category.

```bash
source source_me.sh && python3 -m measure_stack.runner \
  --output-dir output_measure_stack_runner --fail-on-violation
```

Run the actual Rust final-ink handoff. Its fixed output is
`output_glyph_alignment/v2/`. This is a strict gate, and it currently exits
nonzero because it records real renderer geometry defects; it still publishes
diagnostics for every emitted manifest and one `run_summary.json` with the
fixture-level violation inventory.

```bash
devel/run_measure_stack_rust.sh
```

After `./build.sh`, run the real offscreen Qt consumer baseline. It captures the
same 12 authoritative V2 fixtures and exits zero only when capture is healthy
and the frozen known-red failure classification is unchanged.

```bash
devel/run_measure_stack_qt.sh --baseline
```

Run strict-red explicitly when working on Rust geometry. It deliberately exits
nonzero until all visual-quality violations are eliminated; do not add it to a
normal aggregate while the frozen baseline remains red.

```bash
devel/run_measure_stack_qt.sh --strict
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
real-renderer lanes are intentionally distinct: current Rust/Qt geometry is
expected red, and the frozen Qt baseline preserves that evidence rather than
mistaking it for acceptance. The next renderer change belongs in Rust glyph
anchors, endpoint clipping/style lowering, or complete-plan admission according
to the failed metric. Qt must not add a visual correction.

## Limits

- Target-core masks identify the fixture-declared atom character; they do not
  infer chemistry from arbitrary text.
- Raster topology is a robust sanity check, not a proof of every artistic
  feature of curved or hashed ink.
- Capture composition detects crop, occupancy, and isolation; it does not by
  itself judge pedagogical usefulness.
