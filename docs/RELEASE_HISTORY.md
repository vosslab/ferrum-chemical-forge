# Release history

Ferrum has not published a consumer release. `26.08` is the synchronized
pre-alpha development version, not a shipped release number. A release entry
is added only after a human approves the release gate and creates its `v*` tag.

## Unreleased pre-alpha work

### Highlights

- Ferrum is becoming a Rust-first chemical editor: Rust owns the document,
  chemistry, rendering, and cross-language contracts, while the PySide6 client
  consumes typed native observations.
- The local developer build produces the `ferrum` CLI and `ferrum-qt` desktop
  application under `build/`. It is not an installation or consumer-release
  workflow.
- Native CDML editing, bounded interchange and export routes, typed CLI
  operations, and a closed PyO3/Qt render-observation path are under active
  development.
- The current renderer selects proportional Atkinson Hyperlegible Next Regular
  and issues exact tight-outline label geometry, including a centered
  element-core run and full visible-ink exclusion for bond clipping.
  The alignment corpus covers isotope labels, decorations, bond styles, and
  refusal when a final bond would cross another label.

### Notable fixes

- The Qt molecule renderer now consumes closed V4/V2 native observations and
  preserves native paint order rather than inferring generic drawing meaning.
- Attached compact-group placement and normal-single-bond clipping now use one
  renderer-owned clearance policy.
- The active migration removed the OASA compatibility host from the production
  desktop route; read-only historical material remains only as provenance or
  an oracle for parity work.

### Compatibility notes

- There is no supported upgrade path, released artifact, or compatibility
  promise. Existing behavior may change as the pre-production contracts are
  strengthened.
- Full OASA/BKChem parity, human accessibility review, remote CI, release
  verification, and the remaining parity milestones are open. See
  [active_plans/active/FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

### Validation

- Current work records focused Rust, PyO3, and offscreen Qt evidence in
  [CHANGELOG.md](CHANGELOG.md). Those receipts are development evidence, not
  release acceptance.
