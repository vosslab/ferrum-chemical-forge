# File structure

## Top-level layout

```text
ferrum-chemical-forge/
+- AGENTS.md                 Agent and Python-runtime requirements
+- README.md                 Project overview and migration status
+- docs/                     Durable documentation and active planning records
+- packages/
|  +- ferrum-rust/           Ferrum-Chem Rust workspace and local runtime staging
|  `- ferrum-chem-qt.app/    Ferrum PySide6 application and tests
+- tests/                    Repository checks and cross-package E2E coverage
+- devel/                    Maintainer scripts and measurement helpers
+- source_me.sh              Bash environment setup for Python 3.12 commands
+- pip_requirements.txt      Product Python dependency declaration
`- OTHER_REPOS/             Ignored optional reference material
```

## Key subtrees

### `packages/ferrum-rust/`

- [../packages/ferrum-rust/Cargo.toml](../packages/ferrum-rust/Cargo.toml) is the
  Rust workspace manifest and shared version, license, and lint policy.
- [../packages/ferrum-rust/crates/api/](../packages/ferrum-rust/crates/api/)
  owns the `ferrum` executable and public composition layer.
- [../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/)
  builds the `ferrum_chem` PyO3 extension.
- [../packages/ferrum-rust/crates/chemistry/](../packages/ferrum-rust/crates/chemistry/),
  [../packages/ferrum-rust/crates/core/](../packages/ferrum-rust/crates/core/),
  [../packages/ferrum-rust/crates/document/](../packages/ferrum-rust/crates/document/),
  [../packages/ferrum-rust/crates/domain/](../packages/ferrum-rust/crates/domain/),
  [../packages/ferrum-rust/crates/geometry/](../packages/ferrum-rust/crates/geometry/),
  and [../packages/ferrum-rust/crates/render/](../packages/ferrum-rust/crates/render/)
  contain the native chemical, document, domain, geometry, and rendering layers.
- [../packages/ferrum-rust/local_engine_builder.py](../packages/ferrum-rust/local_engine_builder.py)
  and [../packages/ferrum-rust/local_runtime_receipt.py](../packages/ferrum-rust/local_runtime_receipt.py)
  stage and validate the repository-owned local native runtime.
- [../packages/ferrum-rust/devel/](../packages/ferrum-rust/devel/) holds
  maintainer measurement and exploration scripts.

### `packages/ferrum-chem-qt.app/`

- [../packages/ferrum-chem-qt.app/pyproject.toml](../packages/ferrum-chem-qt.app/pyproject.toml)
  declares the `ferrum-qt` package, console command, and runtime dependencies.
- [../packages/ferrum-chem-qt.app/ferrum_qt/cli.py](../packages/ferrum-chem-qt.app/ferrum_qt/cli.py)
  parses the command line and calls the application boundary.
- [../packages/ferrum-chem-qt.app/ferrum_qt/app.py](../packages/ferrum-chem-qt.app/ferrum_qt/app.py)
  owns the QApplication lifecycle and creates the ordinary
  [../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py).
- [../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py)
  is the one ordinary product window and derives from the native main-window
  implementation.
- [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/)
  contains the document-tab, file admission, edit, import and export, template,
  presentation, and view-control modules that implement the Ferrum desktop
  route.
- [../packages/ferrum-chem-qt.app/ferrum_qt/canvas/](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/)
  contains the Rust-observation projections and Qt graphics lifetime helpers.
- [../packages/ferrum-chem-qt.app/ferrum_qt/dialogs/](../packages/ferrum-chem-qt.app/ferrum_qt/dialogs/),
  [../packages/ferrum-chem-qt.app/ferrum_qt/themes/](../packages/ferrum-chem-qt.app/ferrum_qt/themes/),
  and [../packages/ferrum-chem-qt.app/ferrum_qt/resources/](../packages/ferrum-chem-qt.app/ferrum_qt/resources/)
  hold shared Qt dialogs, appearance support, and packaged assets.
- [../packages/ferrum-chem-qt.app/tests/](../packages/ferrum-chem-qt.app/tests/)
  contains focused application behavior tests and native route E2E tests.

The removed compatibility host, session and worker layers, action and mode
families, compatibility codecs, and legacy menu and mode resources have no
replacement subtree. The ordinary package is the supported application route.

### `tests/` and `docs/`

- [../tests/e2e/](../tests/e2e/) contains corpus inputs and cross-package
  scenarios.
- [../tests/e2e/reference/](../tests/e2e/reference/) is an optional Python RDKit
  environment for one-time maintainer measurements only.
- [../tests/](../tests/) also contains repository policy and documentation checks.
- [active_plans/](active_plans/) contains migration plans, decisions, audits,
  reports, and workstreams. The active plan is the current status authority.
- [QT_CONTRACT.md](QT_CONTRACT.md), [FILE_FORMATS.md](FILE_FORMATS.md), and
  [USAGE.md](USAGE.md) record the desktop boundary, supported formats, and user
  commands.

## Generated artifacts

- Local developer build products use `build/`. `build/bin/` and
  `build/runtime/` retain the runnable CLI, Qt launcher, extension, and sealed
  runtime. Cargo intermediates are disposable subdirectories of `build/`:
  `build/.cargo-target/` belongs to `build.sh`, while
  `build/.cargo-check-target/` belongs to `check_rust.sh`.
- Rust packages do not own `target/` output directories. The front-door scripts
  clean their work areas, so `packages/ferrum-rust/target/` and nested PyO3
  target directories are absent after normal completion.
- Python packaging byproducts use `dist/`, `sdist/`, `site/`, and
  `*.egg-info/`, which the root ignore policy excludes.
- `OTHER_REPOS/` is ignored optional reference material. Production code,
  packaging, and release paths do not require it.

## Documentation map

- [../README.md](../README.md) is the project entry point and contributor
  overview.
- [CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md) describes ownership boundaries and
  the Rust-to-Qt data flow.
- [active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md) tracks
  migration milestones and accepted evidence.
- [PROVENANCE.md](PROVENANCE.md) records component lineage and licensing scope.
- [CHANGELOG.md](CHANGELOG.md) records changes for human review.

## Where to add work

- Add Rust source and Cargo tests in the responsible
  [../packages/ferrum-rust/crates/](../packages/ferrum-rust/crates/) crate.
- Add Python bindings in
  [../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/)
  when a typed Rust boundary is ready.
- Add desktop work in the responsible
  [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/)
  module or shared Qt module.
- Add focused behavior coverage with the owning application or Rust crate; put
  artifact-dependent scenarios under [../tests/e2e/](../tests/e2e/).
- Add durable documentation in [docs/](.) and active migration records under
  [active_plans/](active_plans/).
