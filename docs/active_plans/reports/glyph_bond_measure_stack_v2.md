# Glyph-bond measurement stack V2 evidence

## Scope

`measure_stack/` is Ferrum-owned developer-only raster measurement tooling. It
implements relevant OASA/BKChem measurement ideas without importing their code
or using Qt to calculate geometry. It consumes hash-bound raster layers and
fixture graph identity, rejecting renderer-issued bounds, axes, clearances,
clipped endpoints, transforms, and calculated metrics.

## Evidence lanes

| Lane | Producer | Current result |
| --- | --- | --- |
| Synthetic negative oracle | `measure_stack.runner` | Green: all seven deliberately negative scenes reach their declared failure category. |
| Native final ink | `devel/run_measure_stack_rust.sh` | Green: 14 renderable V2 fixtures, zero violations. |
| Qt strict geometry | `devel/run_measure_stack_qt.sh --strict` | Green: 14 actual-Qt fixtures, zero violations. |
| Qt replay baseline | `devel/run_measure_stack_qt.sh --baseline` | Green capture/provenance receipt with frozen zero failure categories. |

Each lane writes JSON, per-fixture reports, annotated overlays, contact sheets,
and an aggregate run summary under ignored output directories. The Qt lane uses
an offscreen real `QGraphicsScene` replay; native evidence uses the test-only
8x Rust raster sink.

## Current conclusion

The measurement system is operational and non-circular. Human inspection of the
documentation capture showed that the original 0.20-stroke lower bound admitted
a double bond with only 0.44 stroke widths of Qt clearance. The current policy
retains 0.20 for single-stroke styles and requires 0.60 for double/triple bonds.
Rust now resolves semantic label runs against style-owned terminal corridors and
validates exact final operations. Native and actual-Qt evidence each report zero
findings across the 14 renderable fixtures, and the Qt baseline freezes that
accepted-zero state. A separate one-time 10-style by 8-direction actual-Qt
matrix reports zero violations across 160 endpoints.

An explicit anti-circularity check ran the unmodified pre-rebuild `HEAD`
measurement package against copies of the rebuilt captures. It accepted all 14
native fixtures and all 14 Qt fixtures with zero violations. Across the broader
80-scene matrix it rejected only two visually valid dashed bonds whose fixed
samples landed in intentional dash gaps; it reported no glyph-gap, collision,
centerline, double-bond, or triple-bond failure. The retained measurement change
replaces that dashed phase sample with stricter whole-component evidence:
separation, elongation, centerline agreement, and endpoint-spanning coverage.

In the diagonal N=O/O&equiv;N fixture, native parallel gaps are 1.35-1.49 measured
stroke widths. Actual Qt gaps are 0.90-1.08 strokes, or 3.95-4.72 pixels. The
superseded Qt double-bond gaps were 0.44-0.61 strokes, or 1.92-2.70 pixels.

Two one-time private outline-support experiments remain discarded. Raw convex
support reached 18 native findings; support dilated by the rectangle clearance
kernel reached 10 native findings but produced 17 rebuilt-Qt findings,
including full-label collisions. The accepted correction uses the exact
directional outline, final-ink footprint geometry, and distinct optical versus
decoration exclusion gaps in Rust; both real lanes validate it.

## Verification

```bash
source source_me.sh && python3 -m pytest measure_stack/tests -q
# 17 passed

# Produces native raster layers, reports, and a zero-violation strict receipt.
./devel/run_measure_stack_rust.sh

./build.sh
./devel/run_measure_stack_qt.sh --strict
./devel/run_measure_stack_qt.sh --baseline

source source_me.sh && python3 tests/e2e/e2e_atom_label_bond_alignment.py
# {"status":"ok","cases":14}

source source_me.sh && python3 -m measure_stack.batch \
  --manifest-root output_glyph_alignment/v2
# {"fixtures":14,"status":"ok","violations":0}

./all_test.sh
# 8,641 hygiene, 283 installed PyO3, and 444 Qt tests passed;
# every registered CLI/Qt E2E passed.

./check_rust.sh
# Formatting, workspace check, strict Clippy, 169 passing ferrum-render tests,
# one ignored developer receipt, complete workspace tests, doc tests, and Rustdoc passed.
```

This report records automated corpus acceptance for the selected Atkinson
Hyperlegible Next Regular molecule-label face. It does not establish broad
desktop usability acceptance beyond the fixed corpus.

Permanent coverage is limited to the 17 deterministic measurement tests, the
selected default resource/scalar contracts, Rust semantic and renderer tests,
and installed consumer contracts. Upstream comparison of all 92 pinned font
binaries and both licenses, catalog/file census, proportional-versus-monospace
width measurement, native and Qt raster publication, contact-sheet inspection,
and the rebuild are implementation evidence rather than additional permanent
tests. The catalog/file census is reproducible through
`packages/ferrum-rust/devel/verify_vendored_font_catalog.py`. All render-owned,
corpus, installed-consumer, and aggregate gates for this goal are green.
