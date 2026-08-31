# Glyph-bond visual-quality goal

## Goal

Ferrum must render bonds that read as connected chemical drawings: each
endpoint serves its intended atom-character glyph, leaves a small visible gap,
avoids every non-endpoint full label, and retains the topology of its declared
bond style. Rust owns placement, optical anchoring, clipping, final-footprint
lowering, complete-plan admission, and typed refusals. Qt is a consumer of the
issued V4/V2 plan and must not apply visual corrections.

This is a Ferrum-owned implementation of measurement ideas documented in the
read-only BKChem/OASA provenance records
[GLYPH_ALIGNMENT_METRICS.md](https://github.com/vosslab/bkchem/blob/main/docs/GLYPH_ALIGNMENT_METRICS.md)
and
[GLYPH_ALIGNMENT_TECHNIQUE_SUMMARY.md](https://github.com/vosslab/bkchem/blob/main/docs/GLYPH_ALIGNMENT_TECHNIQUE_SUMMARY.md).
Ferrum neither imports nor copies OASA runtime or source.

## Evidence boundary

The measurement stack derives attachment, gap, collision, topology, and scene
facts from pixels. Fixture data supplies only immutable CDML, source identities,
bond styles, fixed capture profiles, and expected relationships. The V2 reader
rejects renderer-issued `GlyphBounds`, `BondAttachmentAxisV1`, clearance,
endpoints, transforms, coordinates, and calculated metrics.

Every V2 capture declares whether it is `presentation` evidence (a fixed
user-facing viewport) or `raw_final_ink` evidence (a fixed diagnostic raster).
Presentation captures enforce framing, occupancy, and crop rules; raw final-ink
captures enforce the same layer-integrity and endpoint rules without treating
their deliberately spacious diagnostic canvas as a user viewport. Every V2
capture contains a hash-bound manifest, final composite, core-glyph
mask, full-label mask, and final bond-footprint mask. The core mask selects the
attachment target; the full label, including decorations and masks, owns all
collision checks. The composite comes from the actual rendering path rather
than reconstruction from isolated masks.

## Owned artifacts

| Owner | Artifact | Responsibility |
| --- | --- | --- |
| Fixture authority | `measure_stack/fixtures/v2/fixtures.json` | Fixed 480x360 normal-document `presentation` profile, CDML/graph identity, approved relations, and named negative cases. |
| Measurement library | `measure_stack/contracts.py`, `measure.py`, `diagnostics.py` | Closed V2 manifest validation, hash/path bounds, pixel-only metrics, JSON reports, overlays, and contact sheets. |
| Deterministic fixture oracle | `measure_stack/runner.py` | Materializes declared synthetic pixels and proves every named bad case reaches its named failure category. It is an oracle test, not a renderer-quality claim. |
| Rust producer | `packages/ferrum-rust/crates/render/src/glyph_bond_raster.rs` | Test-only 8x `raw_final_ink` raster layers and V2 manifest handoff for the Rust alignment corpus. It exposes no product API. |
| Qt consumer capture | `measure_stack/qt_scene_capture.py`, `tests/e2e/e2e_measure_stack_qt.py` | Fixed-profile offscreen `QGraphicsScene.render()` capture of actual projection molecule roots, actual full labels, and actual final bond items for renderable rows. |
| Developer entry points | `devel/run_measure_stack_rust.sh`, `devel/run_measure_stack_qt.sh`, `measure_stack/batch.py` | Native and real-Qt strict evidence plus an optional frozen accepted-zero Qt baseline, each with an aggregate JSON receipt. |

The two named developer gates are the production-evidence entry points.
`measure_stack.runner` and `measure_stack.batch` are supporting developer
tools for the contract oracle and aggregate receipt. There is no compatibility
launcher, V1 artifact reader, or duplicate single-manifest CLI. The earlier V1
report remains historical regression evidence, not acceptance evidence.

## Frozen corpus

The V2 catalog currently mirrors all 12 renderable rows in the authoritative
Rust alignment corpus and adds seven schema-valid synthetic negative fixtures.
It covers all eight endpoint directions; C, N, O, S, P, F, Cl, Br, and I;
isotope, hydrogen, charge, and mask decorations; ordinary, double, triple,
dashed, bold, wavy, solid-wedge, hashed-wedge, Haworth-front stroke, and
Haworth-front wedge footprints; a crowded third-label near miss; and named
detachment, target-overlap, centerline, style-topology, crop, orphan, and
third-label-collision failures.

The synthetic rows are measured by the deterministic pixel oracle. Qt replay
captures only renderable rows. Typed refusals, including non-endpoint-label
admission, remain assertions in the existing Rust semantic corpus and contract
checks rather than fabricated Qt pixel fixtures.

Measurement-contract changes require a new schema version. Visual corrections
update the one canonical live fixture definition and preserve before/after run
reports as migration evidence instead of creating chronological fixture IDs.

## Quantitative acceptance

`MeasurementPolicy` is the current threshold authority. It is fixed before a
run and a current rendering cannot tune it.

| Criterion | Required result |
| --- | --- |
| Intended label gap | 0.20-1.75 final stroke widths for single-stroke styles, 0.60-1.75 for double/triple bonds, and no more than 0.22 target-glyph heights |
| Serving axis | Perpendicular error no more than 0.12 target-glyph heights |
| Target and third-label collision | Zero final-footprint pixels in the target full label or every non-endpoint full label |
| Footprint evidence | At least 99.5 percent final-footprint coverage in the composite; no missing declared component |
| Style topology | Visible lane/component/carrier predicate passes for every supported style |
| Presentation scene | 1.5-33 percent pixel occupancy, at least 10 percent dominant-axis occupancy, at least 2.5 percent minimum margin, no crop, no unexplained foreground, and no orphaned core |
| Raw final-ink scene | No unexplained/missing declared ink, no missed endpoint neighborhood, and no orphaned core; diagnostic-canvas framing is not evaluated |

Nonfinite endpoint values are explicit violations and serialize as JSON `null`;
they never become an accepted numeric result. Threshold changes require a new
versioned before/after report for the frozen corpus and an owner rationale.

## Automated evidence lanes

All required review is automated and artifact-backed; no interactive desktop,
screenshot operator, or manual approval is a gate.

```bash
# V2 metric/oracle lane: approved synthetic fixtures pass and every deliberate
# bad fixture proves its named rejection.
source source_me.sh && python3 -m measure_stack.runner \
  --output-dir output_measure_stack_runner

# Rust final-ink evidence: strict by design and publishes a report and contact
# sheet for every renderable case.
./devel/run_measure_stack_rust.sh

# Build the local runtime first, then capture actual Qt consumer pixels.
./build.sh
./devel/run_measure_stack_qt.sh --strict

# Optional accepted-zero capture/provenance baseline.
./devel/run_measure_stack_qt.sh --baseline
```

The Qt baseline exits zero only when renderable-row capture infrastructure is
healthy and the frozen accepted-zero failure categories match. The synthetic
runner separately proves third-label collision rejection, while Rust semantic
and contract checks prove typed refusal. This prevents an accidentally relaxed
predicate or capture regression from being accepted as progress. Strict mode
exits nonzero for any visual-quality violation. Both lanes write ignored evidence below
`output_glyph_alignment/`, `output_measure_stack_runner/`, or
`output_measure_stack_qt/`.

Focused `measure_stack/tests/` tests use inline or generated arrays and
`tmp_path` files to prove manifest rejection, hash binding, metric behavior,
Haworth-form distinction, synthetic-negative classification, dominant-axis
framing, and nonfinite JSON refusal. The explicit synthetic runner owns exact
fixture coverage and diagnostic publication; the installed Qt E2E owns actual
consumer capture. Existing Rust corpus tests remain semantic contract checks;
`./check_rust.sh` and `./all_test.sh` remain aggregate gates.

### Permanent versus implementation evidence

Permanent pytest coverage is limited to deterministic manifest, metric, and
synthetic-negative behavior. The synthetic runner's real manifest/image/contact
sheet publication, native raster command, real CLI, actual Qt capture,
before/after comparisons, and aggregate repository gates are explicit
developer/E2E evidence for this rebuild rather than fragile permanent tests.

## Current truth and correction record

Ferrum's 2026-08-31 V2 glyph-bond geometry is accepted by the unchanged pixel
policy. The deterministic oracle reports 19 fixtures and zero violations; the
native final-ink and actual Qt lanes each report 12 renderable fixtures and zero
violations. The prior native and Qt receipts each contained 15 findings.

The correction remains Rust-owned. Exact quadratic/cubic outline support for
the current verified reference face replaces rectangular corner clipping;
core optical clearance is distinct from mask and decoration exclusion; and
style lowering models actual endpoint caps, transverse widths, axial overhang,
and axial retreat. Parallel terminals use the full occupied ink interval rather
than distance from the attachment axis, preserving the same clearance contract
for future asymmetric lane placement. A sole rightward bond relocates explicit
hydrogen/count ink to the left while preserving isotope and charge conventions.
The wide-endpoint gap floor is normalized to final endpoint width, so
Haworth-front ink retains the same minimum visible separation as ordinary
strokes.

The Qt evidence correction is capture-only: fixed-profile rendering now uses
one isotropic scale and centered letterboxing instead of stretching the scene.
Three tightened source rectangles replace their superseded definitions under
one unversioned live profile ID each. Neither Qt nor the measurement library
adds a visual offset. Human review of the documentation capture exposed a
parallel-bond gap that the original 0.20-stroke minimum admitted, so the
independent policy now requires 0.60 strokes for double and triple bonds.

Two one-time 2026-08-30 outline experiments remain rejected evidence: a raw
convex support plus radial clearance made the native receipt worse (18
findings), while an outline dilated by the rectangle clearance improved native
evidence to 10 but regressed rebuilt Qt replay to 17 findings, including
full-label collisions. The delivered exact directional support model is green
in both real lanes rather than accepting a native-only count reduction.

Failures select a Rust owner instead of a molecule-, sugar-, screenshot-, or
Qt-specific offset:

| Failed evidence | Rust owner | Permitted correction |
| --- | --- | --- |
| Systematic target-axis drift | Private `GlyphMetrics` / `AtomLabelAttachmentGeometry` | Verified molecule-label font hash, tight outline bounds, and glyph-class optical-anchor calibration |
| Gap, target overlap, or serving-lane error | `NormalBondEndpointClipPolicy` and style lowering | Clip-policy or final-footprint correction |
| Parallel-lane disagreement | `atom_bond/bond/ink.rs` parallel-terminal envelope | One combined lane-envelope clip, validated against full-label pixels before symmetric lane emission |
| Third-label crossing or crowding | Complete-plan admission | Typed refusal or globally valid placement decision |
| Qt-only mismatch | Qt replay contract test | Consumer defect correction without visual offsets |
| Crop/composition defect | Fixture capture profile | Fixed-profile correction, never molecule paint offsets |

Each correction must add a focused fixture when appropriate, record the
before/after V2 JSON summaries and contact sheets, and run the frozen corpus
through both native and Qt lanes.

### Font transition rule

Ferrum selects the byte-verified proportional Atkinson Hyperlegible Next
Regular face for molecule labels. The repository also vendors all 92 official
Next and Mono version 2.001 static OTF, static and variable TTF, and static and
variable WOFF2 outputs under one integrity catalog, but no other face is active
in the molecule renderer. If Ferrum changes that role later, Rust
updates the one selected resource and scalar-admission table, issues new exact
outline metrics, recalibrates the current V2 corpus, and reruns native and Qt
strict evidence. The independent `measure_stack/` remains unchanged because it
consumes pixels rather than font internals.

## Completion criteria

This goal is complete only when:

1. the V2 manifests, full-layer inventory, fixture catalog, test-only Rust
   producer, and deterministic actual-Qt capture remain reproducible;
2. the synthetic oracle accepts every approved row and rejects every named bad
   row under the fixed current policy;
3. native Rust strict measurement and actual Qt strict replay have zero visual
   violations for every accepted renderable fixture, while typed-refusal
   contract cases remain green in their Rust semantic lane;
4. all reports, overlays, contact sheets, and run summaries are published and
   validated automatically; and
5. focused tests, Rust corpus checks, Qt E2E, `./check_rust.sh`, and
   `./all_test.sh` pass, with any unrelated pre-existing failure documented
   separately.

The 2026-08-31 evidence satisfies the instrument, native geometry, actual Qt,
diagnostic-publication, selected-font recalibration, and accepted-zero baseline
criteria. Aggregate gate receipts are recorded in the changelog.
