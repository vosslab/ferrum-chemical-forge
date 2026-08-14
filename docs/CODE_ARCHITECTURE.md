# Code architecture

## Overview

Ferrum Chemical Forge is a pre-alpha CDML chemical-drawing system with two
separately licensed components:

- Ferrum-Chem is the LGPL Rust workspace that owns CDML documents, chemistry
  boundaries, geometry, render operations, and the `ferrum` command-line tool.
- Ferrum-Qt is the AGPL PySide6 desktop application. Its normal window remains a
  migration preview, while `ferrum-qt --native` starts a separate, OASA-free Rust
  CDML route.

The active migration contract is
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md). OASA is a
reference-only oracle for bounded comparison work, not a dependency of the Rust
workspace, its CLI, its wheel, or the native Qt route.

The adopted historical behavioral and format references are
[CDML_BACKEND_TO_FRONTEND_CONTRACT.md](CDML_BACKEND_TO_FRONTEND_CONTRACT.md) and
[CDML_FORMAT_SPEC.md](CDML_FORMAT_SPEC.md). Their OASA/BKChem names identify the
source contract; Ferrum-Chem and Ferrum-Qt are the intended corresponding roles.

## Major components

### Rust workspace

[../packages/ferrum-rust/Cargo.toml](../packages/ferrum-rust/Cargo.toml) defines
an edition-2024 workspace with Rust 1.97.1 and seven internal crates:

- [../packages/ferrum-rust/crates/core/](../packages/ferrum-rust/crates/core/)
  provides immutable chemical-domain records and graph-facing data.
- [../packages/ferrum-rust/crates/chemistry/](../packages/ferrum-rust/crates/chemistry/)
  owns the project chemistry engine and the ABI-4 response model.
- [../packages/ferrum-rust/crates/chemistry-sys/](../packages/ferrum-rust/crates/chemistry-sys/)
  loads and validates the native adapter boundary.
- [../packages/ferrum-rust/crates/document/](../packages/ferrum-rust/crates/document/)
  parses CDML, retains opaque content, preserves identity and order, and owns the
  revisioned document session.
- [../packages/ferrum-rust/crates/geometry/](../packages/ferrum-rust/crates/geometry/)
  contains project-owned display geometry.
- [../packages/ferrum-rust/crates/domain/](../packages/ferrum-rust/crates/domain/)
  contains higher-level domain utilities, including Haworth work.
- [../packages/ferrum-rust/crates/render/](../packages/ferrum-rust/crates/render/)
  creates typed render and presentation operations. It uses `ttf-parser` for the
  bundled Telex font metrics rather than a Python or Qt font-metrics authority.

[../packages/ferrum-rust/crates/api/](../packages/ferrum-rust/crates/api/) composes
the crates into the `ferrum` executable. Its CDML commands inspect, validate,
rewrite, extract CD-SVG payloads, and emit a complete render observation. Its
SMILES inspection command requires the caller to name an absolute native adapter.

### Native chemistry extension

[../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/)
builds the direct `ferrum_chem` PyO3 extension. It presents owned document-session,
render-observation, display-geometry, and chemistry DTOs to Python; the Qt native
route imports this module rather than parsing CDML itself.

The native adapter has ABI-4 capability negotiation. Its complete SMILES molecule
response uses the FCM1 wire vocabulary, decoded in
`packages/ferrum-rust/crates/chemistry/src/native_engine/fcm1.rs`.
The native-wheel builder creates the macOS arm64 ABI-4 FCM1 dependency closure and
places it beside the extension; the supported artifact boundary is the packaged
wheel, not a bare Maturin intermediate.

### Ferrum-Qt routes

The package at
[../packages/ferrum-chem-qt.app/ferrum_qt/](../packages/ferrum-chem-qt.app/ferrum_qt/)
contains two deliberately separate routes:

- [../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py)
  is the retained normal application window. It remains isolated from the native
  file route while migration-only OASA-backed capabilities are replaced.
- `packages/ferrum-chem-qt.app/ferrum_qt/native/native_app.py` starts the explicit
  `--native` bounded editor. Its `ferrum_native_main_window.py` imports neither the
  legacy window nor OASA. It hosts only Rust-owned CDML tabs with file actions plus
  Change Element, Add Atom at Point, Undo, Redo, and authoritative Refresh actions.

Each `ferrum_native_document_tab.py` owns exactly one Rust `DocumentSession`, one
verified Telex resource, and one
projection controller. It adopts a path and title only after Rust confirms a save
and the replacement render observation installs successfully. Rust resolves durable
molecule selectors and allocates new atom identifiers; Qt supplies only explicit
intent and the exact clicked scene point.

### Render and projection ownership

Rust render observations cross into Qt through `ferrum_render_projection.py` and
`ferrum_presentation_projection.py` in the canvas package.
The render observation carries one document-molecule envelope per plan, so Qt
places a molecule root by its Rust-issued direct-root source order while keeping
atom and bond order local to that group. Each supported presentation vector is
an independent top-level root at its Rust-issued document order, allowing the
render projection to interleave molecule groups and presentation roots without
flattening molecule-local items. Rust supplies every ordered point of a
non-spline polyline or polygon, finite normalized bounds for rectangle, square,
oval, and circle roots, and fully resolved stroke and fill facts. Qt constructs
the corresponding path without reparsing CDML, discarding intermediate bends,
or choosing appearance defaults. Supported normal non-spline arrows additionally
carry a backend-derived shortened axis and filled four-point head polygons;
specialized and spline arrow families remain explicit issues. Qt owns only
disposable scenes and graphics items. A standalone presentation replacement
preflights callback retirement,
moves the complete root set between captured scene states, restores the prior
projection after a recoverable failure, and retains ambiguous native ownership
for explicit later retirement. The shared retirement rules are in
[graphics_retirement.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/graphics_retirement.py).

## Data flow

The native CDML path is the currently demonstrated vertical slice:

```text
CDML file
  -> ferrum-qt --native
  -> FerrumNativeMainWindow
  -> FerrumNativeDocumentTab
  -> ferrum_chem.DocumentSession
  -> Rust document snapshot and render observation
  -> Qt render and presentation projection
  -> one owned graphics-scene projection
  -> save_atomic and confirmed replacement observation
  -> structurally preserved CDML file
```

The CLI path has no Python or OASA runtime:

```text
CDML or SMILES input
  -> ferrum executable
  -> ferrum-api
  -> document and chemistry crates
  -> versioned JSON report or rewritten CDML
```

For SMILES, the caller provides the absolute ABI-4 adapter library. The API crate
does not discover a system chemistry installation.

## Testing and verification

- Workspace crates use Cargo unit and integration tests; the root manifest supports
  `cargo test --workspace --all-targets --locked` from
  [../packages/ferrum-rust/](../packages/ferrum-rust/).
- The wheel boundary is built and checked by
  [../packages/ferrum-rust/tools/build_native_wheel.py](../packages/ferrum-rust/tools/build_native_wheel.py)
  and root native-wheel E2E coverage in
  [../tests/e2e/e2e_native_wheel.py](../tests/e2e/e2e_native_wheel.py).
- Rust document and OASA-oracle comparison inputs live under
  [../tests/e2e/](../tests/e2e/). `OTHER_REPOS/` is ignored and must not become a
  production build, test, runtime, or release dependency.
- Ferrum-Qt native-route and projection checks live with the app in
  [../packages/ferrum-chem-qt.app/tests/](../packages/ferrum-chem-qt.app/tests/),
  including `e2e_native_cdml_file_route.py` and focused Ferrum projection tests.
- Repository hygiene checks, including Markdown-link validation, are in
  [../tests/](../tests/).

## Extension points

- Add chemistry operations behind the narrow Rust engine and ABI adapter contracts;
  keep RDKit linkage confined to the chemistry boundary.
- Add CDML behavior in the document crate, preserving opaque records, stable IDs,
  ordering, revision semantics, and structural output.
- Add geometry and render-operation logic in their Rust crates before adding a Qt
  projection. The desktop projection should consume exact typed observations rather
  than construct a second document model.
- Add native desktop capabilities beneath `packages/ferrum-chem-qt.app/ferrum_qt/native/`
  until the migration plan explicitly retires the separate legacy route.
- Add the corresponding behavior-focused Rust, Qt, or E2E test beside the boundary
  it verifies.

## Known gaps

- Verify full OASA capability replacement before removing the dependency from the
  normal Ferrum-Qt route and its package manifest.
- Complete coordinate tolerance, codec, full-corpus, render-backend, domain, and
  platform-matrix milestones tracked in
  [active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).
- Reconcile the active plan's historical ABI-2/five-library evidence with the
  current ABI-4 FCM1 wheel evidence in the plan's next status update.
