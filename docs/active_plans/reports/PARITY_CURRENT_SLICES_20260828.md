# Current parity-slice delivery record

The canonical scope and status ledger is
[FULL_PARITY_RUST_FIRST.md](../active/FULL_PARITY_RUST_FIRST.md). This record
does not claim full Rust/OASA/BKChem parity, human desktop acceptance, CI, or
release readiness.

## PROGRESS

- Every `BondRenderBatchV1` now transports frozen `BondAttachmentAxisV1` from
  exact structural connection point to connection point. Atom anchors are
  glyph-core centers. Rust, PyO3, and Qt preserve the axis; Qt validates but
  never paints or hit-tests it. Typed final operations remain the sole visible
  ink and retain positive full-glyph clearance. The shared 12-case Rust-to-Qt
  E2E is the durable alignment transport/clearance oracle.
- `document.molecule.export.v1` has one selected-direct-root core and seven
  closed representations: Molfile V2000/V3000, SDF V2000/V3000, canonical
  SMILES, Standard InChI, and Fixed-Hydrogen InChI. Typed refusals distinguish
  snapshot, root, representation, runtime, and output-limit failures. CLI
  publication is atomic create-new after computation; plural multi-record SDF
  remains distinct. ABI-6 native writers receive bounded output limits before
  allocation.
- Modeless Command Reference shares one live `CommandCatalogEntry` projection
  with Command Palette. F1 or **Help > Command Reference...** searches label,
  help, ID, shortcut, breadcrumb, and availability without invoking actions;
  focus restoration and accessibility metadata are explicit.
- CLI/protocol E2Es now use artifact fixtures that preserve their create-new
  publication oracle. Current Python test XML parsing is standardized on
  `lxml`; `defusedxml` is absent from current source/tests.

## TODO

1. Perform human real-window screenshot, visual, keyboard, contrast, and
   assistive-technology review. Include bond/glyph center attachment and the
   Command Reference focus/accessibility path.
2. Rerun `./all_test.sh` at the next integration session. The last full run,
   before the final lxml-only cleanup, exited 0: 8,302 hygiene tests, every
   registered CLI/Qt E2E, 283 installed PyO3 tests, and 444 Qt tests. After
   that cleanup: 21 affected PyO3 tests, 3 affected E2Es, all 8,302 hygiene
   tests, zero `defusedxml`, Markdown links, ASCII, and `git diff --check`
   passed. The aggregate was deliberately not rerun for this wrap-up.
3. Keep P2 directory-sync fault injection explicitly deferred; it is neither
   an immediate usability/parity deliverable nor a release claim.

## ROADMAP

1. Complete ordinary M1/M2 editing and interchange workflows.
2. Complete M4 chemistry operations, then M5 catalogs and reactions.
3. Complete M6 keyboard authoring, help, clipboard, logging, and output
   usability.
4. Finish CI, packaging, release evidence, and the full parity acceptance
   ledger only after the product workflows and human review are complete.

Historical OASA/BKChem material remains read-only provenance under
`OTHER_REPOS/`, never a production dependency or product brand.
