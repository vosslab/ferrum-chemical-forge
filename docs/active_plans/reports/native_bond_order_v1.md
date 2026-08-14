# Native bond order V1

## Scope

This pre-milestone slice lets the standalone OASA-free native Qt window change one
durably selected normal bond among single, double, and triple order. The frontend
submits only the durable bond identity and a closed Rust enum value. Rust validates
the target, recognizes a no-op, replaces only the typed bond order, builds the next
observation before publishing it, and owns revision, undo/redo history, and saved
`n1`, `n2`, or `n3` CDML.

The render contract now distinguishes two CDML facts that were previously conflated:

- `line_width` is the thickness of each painted line;
- `bond_width` is the spacing used to place parallel bond lanes.

A local bond value overrides `standard/bond@width`, whose historical CDML default is
6 scene points when neither value is authored. A negative local `bond_width` retains
its exact spelling in the authoritative document and contributes its magnitude to
the current centered-lane renderer. Centered double lanes use half the spacing on
each side. Triple outer lanes use the established CDML depiction factor of 0.7. These
are source-format rendering semantics, not pixel-equivalence or screenshot gates.

This slice does not claim ring-side double-bond placement, asymmetric shortening,
wedge, hashed, dashed, aromatic, or wavy styles, choosing an order in the Draw Bond
gesture, deletion of other object classes, or completion of M8a, M9, M12, or M16.

## Validation

- The Rust workspace passes formatting, all-target checking and tests, strict Clippy,
  rustdoc, and the macOS arm64 all-target check. `ferrum-document` reports 98 passing
  tests and `ferrum-render` reports 32.
- Renderer behavior tests prove stroke width and authored lane spacing vary
  independently, parallel lanes remain symmetric and bounded, and label clipping is
  required only where a lane intersects a label.
- The fresh direct-extension binding suite passes 42 tests under CPython 3.12 with
  `-I -B`; the focused native tab/window suite passes 33 tests.
- The complete Ferrum-Qt suite passes 917 tests with one existing skip.
- The public installed-wheel native E2E changes a generated bond to double, observes
  two Rust line operations, undoes and redoes the change, continues native editing,
  saves and reopens the document, retains opaque XML, and reports
  `oasa_imported=false`.

The retained focused wheel is 3,492,456 bytes with SHA-256
`bbf93e5fafb805327a34eb0beba303c59a0bc519837522ee3e359fefc96ef411`.
All 15 native libraries are byte-identical to the accepted RDKit 2026.03.5 closure;
this document/render slice does not rebuild RDKit or repeat release-only relinking.
