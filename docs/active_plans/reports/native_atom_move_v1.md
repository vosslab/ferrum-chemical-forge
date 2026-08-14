# Native atom move V1

## Scope

This pre-milestone slice lets the standalone OASA-free native Qt window move one
durably identified atom by dragging its rendered position. Rust owns the finite
replacement `Point3V1`, revision transition, history, projection, serialization,
undo, and redo. Qt owns pointer capture and one disposable line preview only.

The gesture records the current Rust revision and digest before pointer capture. It
resolves the durable atom and its exact projected scene point, preserves the user's
pointer-to-atom offset, and submits only the final finite point. Any intervening
document change rejects the operation before mutation. A no-op point does not create
a history entry, and an atom without a direct point fails with a typed document error.

This slice does not claim atom or bond deletion, bond-order/style editing, hidden
snapping, general coordinate generation, complete native editing, or M8a/M9/M16
completion.

## Validation

- `ferrum-document`: 92 tests passed, including no-op, undo/redo, exact projection,
  unknown atom, and missing-point behavior.
- Fresh direct-extension binding suite: 39 passed under CPython 3.12 with `-I -B`.
- Focused native tab/window suite: 27 passed against the fresh wheel.
- Full Ferrum-Qt suite: 911 passed, 1 skipped.
- Public installed-wheel native E2E drove the real Move Atom pointer gesture,
  saved/reopened the generated atom at its exact moved position, retained opaque XML,
  and reported `oasa_imported=false`.

The retained focused wheel is 3,480,016 bytes with SHA-256
`ae873cdbbdc39e571eb685e76af1551e08bc682b43c669fefdd2a9e6d10f2f4f`.
All 15 native libraries are byte-identical to the previously accepted RDKit
2026.03.5 closure; this slice does not repeat the release-only relinking proof.
