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
[GLYPH_ALIGNMENT_METRICS.md](../../../OTHER_REPOS/bkchem-oasa/docs/GLYPH_ALIGNMENT_METRICS.md)
and
[GLYPH_ALIGNMENT_TECHNIQUE_SUMMARY.md](../../../OTHER_REPOS/bkchem-oasa/docs/GLYPH_ALIGNMENT_TECHNIQUE_SUMMARY.md).
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
| Developer entry points | `devel/run_measure_stack_rust.sh`, `devel/run_measure_stack_qt.sh`, `measure_stack/batch.py` | Native strict-red evidence and deterministic Qt expected-red baseline lanes, each with an aggregate JSON receipt. |

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

New renderer behavior must add a fixture version and migration evidence rather
than revise an accepted fixture to match the current output.

## Quantitative acceptance

`MeasurementPolicy` is the current threshold authority. It is fixed before a
run and a current rendering cannot tune it.

| Criterion | Required result |
| --- | --- |
| Intended label gap | 0.20-1.75 final stroke widths and no more than 0.22 target-glyph heights |
| Serving axis | Perpendicular error no more than 0.12 target-glyph heights |
| Target and third-label collision | Zero final-footprint pixels in the target full label or every non-endpoint full label |
| Footprint evidence | At least 99.5 percent final-footprint coverage in the composite; no missing declared component |
| Style topology | Visible lane/component/carrier predicate passes for every supported style |
| Presentation scene | 1.5-33 percent occupancy, 2.5-45 percent margins, no crop, no unexplained foreground, and no orphaned core |
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

# Rust final-ink evidence: strict by design; it currently exits nonzero while
# publishing a report and contact sheet for every renderable case.
./devel/run_measure_stack_rust.sh

# Build the local runtime first, then capture actual Qt consumer pixels.
./build.sh
./devel/run_measure_stack_qt.sh --baseline

# Qt strict mode is deliberately red until Rust geometry satisfies this policy.
./devel/run_measure_stack_qt.sh --strict
```

The Qt baseline exits zero only when renderable-row capture infrastructure is
healthy and frozen expected-red failure-category counts match. The synthetic
runner separately proves third-label collision rejection, while Rust semantic
and contract checks prove typed refusal. This prevents an accidental relaxed
predicate or capture regression from being accepted as progress. Strict mode
exits nonzero for any visual-quality violation. Both lanes write ignored evidence below
`output_glyph_alignment/`, `output_measure_stack_runner/`, or
`output_measure_stack_qt/`.

Focused `measure_stack/tests/` tests prove manifest rejection, hash binding,
metric behavior, Haworth-form distinction, fixture coverage, synthetic negative
classification, and diagnostic publication. Existing Rust corpus tests and the
installed Qt E2E remain consumer/transport contract checks; `./check_rust.sh`
and `./all_test.sh` remain aggregate gates.

### Permanent versus implementation evidence

Permanent pytest coverage is limited to deterministic manifest, metric, and
synthetic-negative behavior. The synthetic runner's real manifest/image/contact
sheet publication, native raster command, real CLI, actual Qt capture,
before/after comparisons, and aggregate repository gates are explicit
developer/E2E evidence for this rebuild rather than fragile permanent tests.

## Current truth and correction loop

The stack and corpus are present, but current Ferrum glyph-bond geometry is
**not accepted** by the V2 visual-quality policy. The native 2026-08-30
regeneration reduced strict-policy findings from 26 to 15 without changing its
pixel thresholds: complete parallel-lane clipping, directional wedge
footprints, and correct terminal wedge topology are now represented in Rust.
Normal, bold, dashed, wavy, and opposed stereochemical wedges are green;
diagonal multi-line bonds, Haworth-front endpoints, and a few decorated/ring
endpoints remain red. Native strict measurement and Qt strict replay are
expected-red evidence, not green acceptance. The Qt baseline freezes that fact
so that future improvement is measured against the same corpus and policy
instead of hidden by threshold drift. No screenshots should be refreshed as
quality evidence until strict geometry acceptance is green.

The next correction is not a global gap-factor change: diagonal normal bonds
remain centerline-correct but detach because the private clip ray exits a
rectangular `GlyphBounds` envelope at its corner. The renderer must promote the
verified core glyph's own outline (or a conservative outline-derived directional
support representation) into private clipping geometry, then repeat the same
native/Qt corpus. The independent measurement stack continues to consume only
rendered pixels and graph identity.

Two one-time 2026-08-30 outline experiments are rejected evidence, not retained
renderer behavior: a raw convex outline plus radial clearance made the native
receipt worse (18 findings), while an outline dilated by the present rectangle
clearance improved native evidence to 10 but regressed rebuilt Qt replay to 17
findings, including full-label collisions. The next implementation must derive
one calibrated support model from the exact Telex layout and validate its
native and Qt output together; a native-only count reduction is insufficient.

Failures select a Rust owner instead of a molecule-, sugar-, screenshot-, or
Qt-specific offset:

| Failed evidence | Rust owner | Permitted correction |
| --- | --- | --- |
| Systematic target-axis drift | Private `GlyphMetrics` / `AtomLabelAttachmentGeometry` | Verified-Telex font-hash and glyph-class optical-anchor calibration |
| Gap, target overlap, or serving-lane error | `NormalBondEndpointClipPolicy` and style lowering | Clip-policy or final-footprint correction |
| Parallel-lane disagreement | `atom_bond/bond.rs` multi-lane lowering | One combined lane-envelope clip, validated against full-label pixels before symmetric lane emission |
| Third-label crossing or crowding | Complete-plan admission | Typed refusal or globally valid placement decision |
| Qt-only mismatch | Qt replay contract test | Consumer defect correction without visual offsets |
| Crop/composition defect | Fixture capture profile | Fixed-profile correction, never molecule paint offsets |

Each correction must add a focused fixture when appropriate, record the
before/after V2 JSON summaries and contact sheets, and run the frozen corpus
through both native and Qt lanes.

### Font transition rule

The current byte-verified Telex face is a legacy renderer resource, not a
permanent design decision. Atkinson Hyperlegible is the preferred candidate
for Ferrum's written UI and document prose because its letterform distinction
is an accessibility feature; mononoki is reserved for fixed-width developer
and source-oriented surfaces. Neither candidate may be substituted by Qt or by
CSS alone. If Ferrum adopts a different molecule-label face, Rust must add a
new versioned, byte-verified font resource and scalar-admission table, issue
new exact outline metrics, regenerate the V2 corpus as a new fixture version,
and rerun native and Qt strict evidence. The independent `measure_stack/`
remains unchanged because it consumes pixels rather than font internals.

## Completion criteria

This goal is complete only when:

1. the V2 manifests, full-layer inventory, fixture catalog, test-only Rust
   producer, and deterministic actual-Qt capture remain reproducible;
2. the synthetic oracle accepts every approved row and rejects every named bad
   row under the unchanged policy;
3. native Rust strict measurement and actual Qt strict replay have zero visual
   violations for every accepted renderable fixture, while typed-refusal
   contract cases remain green in their Rust semantic lane;
4. all reports, overlays, contact sheets, and run summaries are published and
   validated automatically; and
5. focused tests, Rust corpus checks, Qt E2E, `./check_rust.sh`, and
   `./all_test.sh` pass, with any unrelated pre-existing failure documented
   separately.

The current V2 baseline satisfies the infrastructure and expected-red evidence
parts of this goal. The native aggregate receipt currently reports 15 policy
violations across six renderable fixtures; geometry acceptance remains open and
is owned by Rust.
