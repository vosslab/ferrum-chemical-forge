# Native Molecule Coordinate Regeneration V1

## Outcome

The standalone OASA-free native editor can regenerate one durable ordinary molecule's 2D
coordinates through the packaged ABI-4 chemistry engine. Native chemistry runs outside the Qt UI
thread. The UI-thread document session accepts the complete result as one revision and history
entry only while its source revision and digest remain current.

The provisional Rust CLI exposes the same document operation as
`ferrum cdml generate-coordinates`. It requires an explicit ABI-4 adapter, exact authored molecule
ID, input CDML, and output destination. File output uses Ferrum's descriptor-relative atomic
publisher; `-` keeps the ordinary standard-input or standard-output stream contract.

This is pre-milestone evidence toward M8a. It does not claim that the legacy normal Ferrum-Qt
editor has adopted the Rust session or that every persistent object class can be regenerated.

## Grounded placement policy

Ferrum sends RDKit a coordinate-free graph, so coordinate generation cannot silently reuse the
old drawing. The destination placement derives only from current document facts:

- the arithmetic mean of the existing atom points remains the destination centroid;
- a bonded molecule keeps its current mean bond length;
- a bondless molecule is translated to its current centroid without inventing a scale; and
- each atom's authored finite `z` value is retained because this operation replaces only 2D
  depiction coordinates.

The tests use ordinary floating-point comparison tolerances around those mathematical invariants.
They do not require pixel equivalence, byte-equivalent CDML, or equality with an unrelated toolkit
rendering.

## Fail-closed boundary

V1 requires a nonempty durable molecule made only of ordinary atoms and normal single, double, or
triple bonds. It rejects groups, molecule-text/query vertices, unsupported atom facts, non-atom
bond endpoints, drawing-specific bond styles, absent/unsupported orders, zero current bonded scale,
and invalid native responses. No unsupported fact is silently omitted.

The prepared value carries its source revision, SHA-256 document digest, durable molecule selector,
and complete source-ordered positions. A session revalidates all provenance and the exact direct
typed-atom count before one detached candidate can enter history.

## Evidence

- Rust workspace format, all-target check, tests, strict Clippy, rustdoc, and macOS arm64 all-target
  check pass.
- Focused Rust tests prove coordinate-free engine input, retained centroid/scale/`z`, one undo
  entry, digest rejection across equal revisions, and unsupported-style rejection.
- Command-level tests prove usage exit status, source selection before dynamic loading, relative
  adapter rejection, clean stdout on failure, and an opt-in verified-adapter CDML round trip. The
  real-adapter test checks finite non-collinear output plus retained centroid, mean bond length,
  and `z`; it does not compare serialized bytes or pixels.
- Nested PyO3 format, test, strict Clippy, rustdoc, and arm64 checks pass; the installed direct
  wheel binding suite reports 44 passed.
- The focused native Qt suites report 41 passed against the installed wheel.
- The full Qt suite reports 919 passed and 1 skipped; the root repository suite reports 5685
  passed, and focused hygiene reports 2966 passed.
- The public installed-wheel native E2E reports schema `ferrum-native-cdml-route-e2e-v9`, generated
  coordinates for `molecule-1`, clean save/reopen, opaque-root retention, and
  `oasa_imported: false`.

The focused macOS arm64 direct wheel is
`output_native_wheel/coordinate-regeneration-v1-rdkit-2026035-20260812/wheelhouse/`
`ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl`, with SHA-256
`f5f86b46ada762c1bb7663b32fe8e69d83a5795d869f44bab3a8cd96b395b4e2`.
