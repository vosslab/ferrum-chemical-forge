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
| Synthetic metric oracle | `measure_stack.runner` | Green: accepted synthetic scenes pass and named bad scenes reach their declared failure category. |
| Native final ink | `devel/run_measure_stack_rust.sh` | Strict-red: 15 findings across the 12 renderable V2 fixtures. |
| Qt replay | `devel/run_measure_stack_qt.sh --baseline` | Green expected-red infrastructure receipt; frozen categories remain eight detached endpoints and seven target overlaps. |
| Qt strict geometry | `devel/run_measure_stack_qt.sh --strict` | Strict-red until Rust-owned geometry reaches zero findings. |

Each lane writes JSON, per-fixture reports, annotated overlays, contact sheets,
and an aggregate run summary under ignored output directories. The Qt lane uses
an offscreen real `QGraphicsScene` replay; native evidence uses the test-only
8x Rust raster sink.

## Current conclusion

The measurement system is operational and non-circular. It proves the current
renderer is not visually accepted: native evidence reports 11 detached
endpoints, one missing endpoint connection, and three target-label overlaps.
Qt's healthy expected-red receipt prevents a Rust-only geometry change from
silently becoming a presentation baseline change.

Two one-time private outline-support experiments were discarded. Raw convex
support reached 18 native findings; support dilated by the current clearance
kernel reached 10 native findings but produced 17 rebuilt-Qt findings,
including full-label collisions. The remaining correction needs one calibrated
Rust clip model verified by both receipt types, not a threshold change.

## Verification

```bash
source source_me.sh && python3 -m pytest measure_stack/tests -q
# 24 passed

# Produces the native raster layers, reports, and a strict-red exit while
# geometry remains unresolved.
./devel/run_measure_stack_rust.sh

./devel/run_measure_stack_qt.sh --baseline
# PASS: real Qt V2 measurement baseline completed

source source_me.sh && python3 tests/e2e/e2e_atom_label_bond_alignment.py
# {"status":"ok","cases":14}

source source_me.sh && python3 -m measure_stack.batch \
  --manifest-root output_glyph_alignment/v2
# {"fixtures":12,"status":"ok","violations":15}
```

Strict renderer gates intentionally remain nonzero. This report is evidence
that the measurement stack is implemented, not a visual-acceptance claim.
