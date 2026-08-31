# Glyph-bond measurement guide

## Purpose and outcome

Ferrum measures the pixels that its renderer and Qt consumer actually produce.
It does not reuse renderer-issued glyph bounds, attachment axes, clipped bond
endpoints, or clearance values. That separation answers the useful question:
does the visible bond ink meet the visible atom glyph correctly?

There are two independent outcomes:

- A green measurement instrument means the manifest contract, pixel metrics,
  diagnostic images, and known-bad synthetic cases work correctly.
- Green alignment means every real Rust and Qt fixture satisfies the strict
  policy. Ferrum's 2026-08-31 V2 evidence is green in both real lanes.

Rust owns the geometry to correct. Qt only replays the issued render plan and
is captured as a real consumer; it must not add a visual correction.

## Input evidence

Each V2 manifest binds the exact fixture CDML, graph identity, fixed capture
profile, and SHA-256 hashes for these rendered raster layers:

| Layer | What it establishes |
| --- | --- |
| Normal composite | The complete image users see. |
| Target-core glyph | The actual atom-character ink at each endpoint. |
| Full label | Core glyph plus isotope, hydrogen, charge, and other decoration ink. |
| Final bond footprint | The actual final ink belonging to one bond after style lowering. |

The fixed capture profile prevents a current drawing from improving its own
score merely by zooming, cropping, or changing the viewport. The V2 reader
also validates manifest shape, file paths, layer dimensions, and hashes before
measuring pixels.

The V2 suffix versions this serialized corpus contract. Capture-profile and
fixture IDs are unversioned present-tense identities; changing an internal
rectangle replaces its definition rather than retaining parallel generations.

## How the measurements work

For each bond endpoint, the library recovers the approach direction from the
centers of the two target-core glyph masks. It then samples only the final bond
footprint near that endpoint. For double and triple bonds, every visible lane
has equal weight, so antialiasing cannot shift the answer toward a thicker lane.

The library compares the recovered footprint with target-core and full-label
pixels. It also checks the complete scene, declared layer coverage, connected
components, and style-specific topology. The algorithms are in
[`measure_stack/measure.py`](../measure_stack/measure.py); threshold values are
the one `MeasurementPolicy` authority in that file.

## Run the lanes

First run the deterministic contract oracle. It creates controlled good and bad
pixel scenes, then proves that every named bad fixture reaches its named failure
category. This verifies the measuring instrument; it does not assess Ferrum's
renderer.

```bash
source source_me.sh && python3 -m measure_stack.runner \
  --output-dir output_measure_stack_runner --fail-on-violation
```

Run the native Rust producer to capture the renderer's test-only 8x final-ink
layers. This strict gate publishes diagnostics and exits nonzero for any
violation:

```bash
devel/run_measure_stack_rust.sh
```

Build, then capture the real Qt consumer at normal display scale. Strict mode is
the default and passes only when bond-glyph alignment is good. Optional baseline
mode additionally freezes the accepted zero-finding classification.

```bash
./build.sh
devel/run_measure_stack_qt.sh --strict
devel/run_measure_stack_qt.sh --baseline
```

All lanes write ignored output directories containing V2 manifests, JSON
reports, per-case overlays, contact sheets, and `run_summary.json`. Use the
contact sheet to locate a reported fixture, then use its JSON report to see the
specific endpoint and metric that failed.

## Endpoint statistics

| Statistic | Meaning | Strict policy |
| --- | --- | --- |
| `signed_gap_px` | Along-axis distance from target core-glyph edge to the nearest bond footprint. Negative means bond ink overlaps the full label. | Reported in pixels and normalized forms. |
| `signed_gap_strokes` | Gap divided by measured bond stroke width. | At least 0.20 strokes for single-stroke styles and 0.60 for double/triple bonds; no more than 1.75. |
| `signed_gap_glyph_height` | Gap divided by target-core glyph height. | No more than 0.22 glyph heights. |
| `perpendicular_error_px` | Sideways displacement of the local endpoint footprint from the core-to-core axis. | Reported in pixels and normalized form. |
| `perpendicular_error_glyph_height` | Sideways error divided by target-core glyph height. | No more than 0.12 glyph heights. |
| `own_core_collision_pixels` | Final bond pixels that overlap its own target core glyph. | Exactly 0. |
| `own_full_label_collision_pixels` | Final bond pixels that overlap its own complete label. | Exactly 0. |
| `third_full_label_collision_pixels` | Final bond pixels that overlap another atom's full label. | Exactly 0. |
| `final_footprint_coverage` | Fraction of declared final bond-footprint pixels visible in the normal composite. | At least 0.995. |
| `connectivity_components` | Eight-connected regions in one bond footprint. | Interpreted with style topology. |

Positive gap means a visible separation. A gap below the minimum indicates a
touching or detached-looking endpoint; a gap above the maximum indicates an
obvious detached bond. A negative gap is an overlap and is always bad.

The target-core glyph is deliberately narrower than the full label. Attachment
is judged against the chemical atom character, while collision is judged against
all visible label ink. That distinction lets a bond meet `Br` at its `B` core
without accepting a collision with an isotope or charge decoration.

## Scene and style statistics

| Statistic | Meaning | Strict policy |
| --- | --- | --- |
| `occupancy_fraction` | Fraction of the fixed capture image containing composite foreground ink. | 0.015 to 0.33 for presentation captures. |
| `minimum_margin_fraction` | Smallest blank border around visible composite ink. | At least 0.025. |
| `axis_occupancy_fraction` | Visible bounding-box share on the horizontal and vertical axes. | Reported independently so elongated chemistry is not mistaken for a tiny scene. |
| `dominant_axis_occupancy_fraction` | Larger of the two axis occupancies. | At least 0.10 for presentation captures. |
| `unexplained_foreground_fraction` | Composite ink absent from every declared glyph or bond layer. | At most 0.005. |
| `missing_declared_fraction` | Declared glyph or bond pixels absent from the composite. | At most 0.005. |
| `orphaned_atom_cores` | Connected atoms whose target-core glyph is not connected as expected. | None. |

Style topology checks use final pixels rather than abstract style names. They
confirm lane counts for double/triple bonds, connected endpoint ink, expected
wavy turns, wedge widening, hashed segmentation, and distinct Haworth-front
forms. These are robust visual sanity checks, not a claim that raster analysis
can prove every artistic quality of every path.

## Reading current results

Strict green means that every real fixture satisfies the pixel policy. Baseline
green means that capture is healthy and its frozen accepted-zero receipt has not
changed. A strict receipt with any violation means: **the measurements work,
and bond-glyph alignment is outside the accepted specification.**

Use the failed category to select the Rust owner:

- Systematic sideways error: private glyph optical anchors.
- Gap, attachment, coverage, or style topology: endpoint clipping and style
  lowering.
- Third-label collision: complete-plan admission.

Never resolve a failed metric by adding molecule-specific or Qt-specific visual
offsets. The fixed corpus and strict policy remain unchanged while a proposed
Rust correction is measured before and after.

## Scope and limits

This is a developer and E2E evidence lane, not a permanent fast pytest suite or
a product API. It covers Ferrum's fixed corpus and supported bond styles; it
does not establish SVG, PDF, or PNG backend parity, infer arbitrary chemistry
from text, or substitute for every broader desktop usability evaluation.

V2 measures Ferrum's byte-verified Atkinson Hyperlegible Next Regular
molecule-label face. Rust owns its tight curve bounds, placement, and clipping;
a future role change requires fresh native and Qt evidence under the same
pixel-only policy.

For contract details and current evidence, see
[`active_plans/active/glyph_bond_visual_quality_goal.md`](active_plans/active/glyph_bond_visual_quality_goal.md)
and [`measure_stack/README.md`](../measure_stack/README.md).
