# File structure

## Top-level layout

```text
ferrum-chemical-forge/
+- AGENTS.md                    Required repository workflow
+- README.md                    Project entry point and current scope
+- docs/                        Durable reference docs and active planning records
+- measure_stack/               Developer-only independent raster measurement library
+- packages/
|  +- ferrum-rust/              Ferrum-Chem Rust workspace
|  `- ferrum-chem-qt.app/       Ferrum PySide6 desktop application
+- tests/                       Repository checks and cross-package E2E coverage
+- devel/                       Maintainer scripts
+- build.sh                     Build the local CLI, extension, and Qt launcher
+- check_rust.sh                Rust formatting, lint, and test route
+- all_test.sh                  Repository aggregate test route
+- source_me.sh                 Python 3.12 environment bootstrap
+- pip_requirements.txt         Product Python dependencies
`- OTHER_REPOS/                 Ignored historical reference material
```

## Key subtrees

### `packages/ferrum-rust/`

- [../packages/ferrum-rust/Cargo.toml](../packages/ferrum-rust/Cargo.toml) is
  the one workspace manifest.
- [../packages/ferrum-rust/crates/core/](../packages/ferrum-rust/crates/core/),
  [../packages/ferrum-rust/crates/chemistry/](../packages/ferrum-rust/crates/chemistry/),
  [../packages/ferrum-rust/crates/domain/](../packages/ferrum-rust/crates/domain/),
  and [../packages/ferrum-rust/crates/geometry/](../packages/ferrum-rust/crates/geometry/)
  hold chemistry, domain, and geometry layers.
- [../packages/ferrum-rust/crates/document/](../packages/ferrum-rust/crates/document/)
  owns session and document behavior. Its `tests/fixtures/`
  directory owns the canonical `atom_label_bond_alignment_cases_v1.json`
  corpus, and `tests/atom_label_bond_alignment_corpus.rs`
  is its Rust contract consumer. Its unversioned
  `src/chemistry/document_molecule_export.rs` is the sole selected-direct-root
  export core; it supersedes format-specific document export owners.
- [../packages/ferrum-rust/crates/document-model/](../packages/ferrum-rust/crates/document-model/),
  [../packages/ferrum-rust/crates/document-projection/](../packages/ferrum-rust/crates/document-projection/),
  [../packages/ferrum-rust/crates/document-render/](../packages/ferrum-rust/crates/document-render/),
  and [../packages/ferrum-rust/crates/render-contract/](../packages/ferrum-rust/crates/render-contract/)
  hold shared records, read-only projection, document-facing render preparation,
  and narrow shared rendering contracts.
- [../packages/ferrum-rust/crates/graph-lowering/](../packages/ferrum-rust/crates/graph-lowering/)
  is the downward-only conversion from capability-free projections to chemistry
  graph facts.
- [../packages/ferrum-rust/crates/render/](../packages/ferrum-rust/crates/render/)
  owns the V4 molecule-plan grammar and renderer geometry. Its
  [src/atom_bond/](../packages/ferrum-rust/crates/render/src/atom_bond/)
  subtree includes private final-ink collision admission, and
  [src/verified_telex_glyph_metrics.rs](../packages/ferrum-rust/crates/render/src/verified_telex_glyph_metrics.rs)
  owns exact Telex measurements.
  Its test-only `src/glyph_bond_raster.rs` sink produces closed V2 raster
  evidence for the independent `measure_stack/` developer lane.
- [../packages/ferrum-rust/crates/api/](../packages/ferrum-rust/crates/api/)
  owns CLI/public composition and the source for PyO3 bindings.
  [../packages/ferrum-rust/crates/api-python/](../packages/ferrum-rust/crates/api-python/)
  owns the extension-wheel crate. [../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/)
  contains the Maturin configuration and Python binding tests.
  `src/protocol/document_molecule_export_v1.rs` adapts the one public singular
  export operation; `src/cli/verbs/document_export.rs` presents it on the CLI.
- [../packages/ferrum-rust/crates/template-catalog/](../packages/ferrum-rust/crates/template-catalog/)
  and [../packages/ferrum-rust/crates/catalog-placement/](../packages/ferrum-rust/crates/catalog-placement/)
  own shipped/user catalog facts and closed catalog-recipe lowering.

### `packages/ferrum-chem-qt.app/`

- [../packages/ferrum-chem-qt.app/pyproject.toml](../packages/ferrum-chem-qt.app/pyproject.toml)
  declares the `ferrum-qt` package and console entry point.
- [../packages/ferrum-chem-qt.app/ferrum_qt/cli.py](../packages/ferrum-chem-qt.app/ferrum_qt/cli.py)
  and [../packages/ferrum-chem-qt.app/ferrum_qt/app.py](../packages/ferrum-chem-qt.app/ferrum_qt/app.py)
  start the product application and main window.
- [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/)
  is the flat vertical-feature layer for document tabs, actions, authoring,
  Local Open, catalog, presentation, and view controls.
- [../packages/ferrum-chem-qt.app/ferrum_qt/actions/](../packages/ferrum-chem-qt.app/ferrum_qt/actions/)
  owns action clients. `command_catalog.py` is the shared live metadata
  projection for the Command Palette and modeless Command Reference;
  `command_reference.py` owns only the read-only help surface and focus
  lifecycle.
- [../packages/ferrum-chem-qt.app/ferrum_qt/canvas/](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/)
  owns disposable graphics-scene projection. In particular,
  [ferrum_render_projection.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_render_projection.py)
  consumes exact V2 observations and V4 plans, while
  [ferrum_telex.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_telex.py)
  replays the verified bundled face.
- [../packages/ferrum-chem-qt.app/ferrum_qt/dialogs/](../packages/ferrum-chem-qt.app/ferrum_qt/dialogs/),
  [../packages/ferrum-chem-qt.app/ferrum_qt/themes/](../packages/ferrum-chem-qt.app/ferrum_qt/themes/),
  and [../packages/ferrum-chem-qt.app/ferrum_qt/resources/](../packages/ferrum-chem-qt.app/ferrum_qt/resources/)
  own Qt dialogs, display themes, and packaged YAML/image assets.
- [../packages/ferrum-chem-qt.app/tests/](../packages/ferrum-chem-qt.app/tests/)
  contains deterministic focused app behavior tests.

### `tests/`, `devel/`, and `docs/`

- [../tests/](../tests/) contains repository policy checks and cross-package
  coverage. [../tests/e2e/](../tests/e2e/) holds artifact-dependent scenarios,
  including the installed Qt consumer of the shared atom-label alignment corpus
  and the real-window attached-cyclohexane admission workflow.
- [../devel/](../devel/) contains maintainer-only helpers such as changelog and
  release support.
- [active_plans/](active_plans/) holds in-flight plans, audits, reports,
  decisions, and workstreams. [active_plans/active/FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md)
  is the current migration/parity authority.

## Generated artifacts

- `build/` is ignored local output. [../build.sh](../build.sh) produces immutable
  program roots selected through `build/current`; `build/bin/` and
  `build/runtime/` are stable paths through that selection.
- Cargo intermediates are owned by the front-door scripts under `build/`.
  Normal workflows do not retain workspace `target/` directories.
- Python packaging output uses ignored `dist/`, `sdist/`, `site/`, and
  `*.egg-info/` paths.
- `graphify-out/` and `OTHER_REPOS/` are ignored. The former is regenerated
  navigation evidence; the latter is
  reference material, never a product dependency.

## Documentation map

- [../README.md](../README.md) is the newcomer entry point.
- [INSTALL.md](INSTALL.md) and [USAGE.md](USAGE.md) cover local setup and
  supported workflows.
- [CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md) describes ownership and the
  Rust-to-Qt data flow; this file maps paths to responsibilities.
- [FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md), [QT_CONTRACT.md](QT_CONTRACT.md),
  [FILE_FORMATS.md](FILE_FORMATS.md), and [YAML_FILE_FORMAT.md](YAML_FILE_FORMAT.md)
  document public API, desktop, interchange, and packaged-resource contracts.
- [DESIGN_DECISIONS.md](DESIGN_DECISIONS.md) records settled technical
  decisions; [HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md) preserves human owner
  guidance; [CHANGELOG.md](CHANGELOG.md) records implementation history.

## Where to add work

- Add a chemistry, document, or renderer change to the smallest responsible
  Rust crate. Add the durable test beside that crate or to the canonical
  document corpus when the behavior crosses the render boundary.
- Add new V4/PyO3 transport facts in the API binding only after Rust owns a
  closed validated representation. Keep Qt as a typed consumer.
- Add a desktop interaction in its existing vertical feature module, canvas
  component, dialog, action, theme, or resource owner rather than creating a
  parallel chemistry layer.
- Add deterministic cross-package workflows under [../tests/e2e/](../tests/e2e/)
  only when they require built artifacts or multiple packages; keep ordinary
  behavior tests near their Rust or Qt owner.
- Add durable reference documentation in `docs/` and temporary planning or
  audit material to the appropriate `docs/active_plans/` subdirectory.
