# File structure

## Top-level layout

```text
ferrum-chemical-forge/
+- AGENTS.md                 Agent and Python-runtime requirements
+- README.md                 Project overview and migration status
+- docs/                     Durable documentation and active planning records
+- packages/
|  +- ferrum-rust/           Ferrum-Chem Rust workspace and wheel tooling
|  `- ferrum-chem-qt.app/    Ferrum-Qt PySide6 application and tests
+- tests/                    Repository hygiene and cross-package E2E checks
+- devel/                    Maintainer scripts and measurement helpers
+- source_me.sh              Bash environment setup for Python 3.12 commands
+- pip_requirements.txt      Root Python dependency declaration
`- OTHER_REPOS/             Ignored reference material when present
```

## Key subtrees

### `packages/ferrum-rust/`

- [../packages/ferrum-rust/Cargo.toml](../packages/ferrum-rust/Cargo.toml) is the
  workspace manifest and sets shared Rust edition, version, licensing, and lint
  policy.
- [../packages/ferrum-rust/crates/api/](../packages/ferrum-rust/crates/api/) owns
  the `ferrum` executable and public CLI composition.
- [../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/)
  builds the direct `ferrum_chem` PyO3 extension.
- [../packages/ferrum-rust/crates/chemistry/](../packages/ferrum-rust/crates/chemistry/)
  and [../packages/ferrum-rust/crates/chemistry-sys/](../packages/ferrum-rust/crates/chemistry-sys/)
  contain the ABI-4 native-adapter contract and loader.
- [../packages/ferrum-rust/crates/core/](../packages/ferrum-rust/crates/core/),
  [../packages/ferrum-rust/crates/document/](../packages/ferrum-rust/crates/document/),
  [../packages/ferrum-rust/crates/geometry/](../packages/ferrum-rust/crates/geometry/),
  [../packages/ferrum-rust/crates/domain/](../packages/ferrum-rust/crates/domain/),
  and [../packages/ferrum-rust/crates/render/](../packages/ferrum-rust/crates/render/)
  contain the model, document, geometry, domain, and render-operation layers.
- [../packages/ferrum-rust/tools/](../packages/ferrum-rust/tools/) owns native-wheel
  build, receipt, and Mach-O closure verification utilities.
- [../packages/ferrum-rust/devel/](../packages/ferrum-rust/devel/) holds maintainers'
  measurement and exploration scripts.

### `packages/ferrum-chem-qt.app/`

- [../packages/ferrum-chem-qt.app/pyproject.toml](../packages/ferrum-chem-qt.app/pyproject.toml)
  declares the `ferrum-qt` package and console command.
- [../packages/ferrum-chem-qt.app/ferrum_qt/](../packages/ferrum-chem-qt.app/ferrum_qt/)
  is the retained PySide6 package.
- `packages/ferrum-chem-qt.app/ferrum_qt/native/` is the standalone OASA-free
  native CDML route selected with `ferrum-qt --native`.
- [../packages/ferrum-chem-qt.app/ferrum_qt/canvas/](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/)
  holds render/presentation projection and explicit Qt graphics-retirement logic.
- [../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py)
  remains the separate ordinary migration-preview window.
- [../packages/ferrum-chem-qt.app/tests/](../packages/ferrum-chem-qt.app/tests/)
  contains package-level Qt, native-route, rendering, and projection checks.

### `tests/` and `docs/`

- [../tests/e2e/](../tests/e2e/) contains corpus inputs, oracle runners, and wheel
  E2E scripts.
- [../tests/](../tests/) also contains fast repository policy and documentation
  checks.
- [active_plans/](active_plans/) contains the live migration plan, decisions,
  audits, reports, and workstreams. The active plan remains the status authority.
- `docs/QT_CONTRACT.md`, [INSTALL.md](INSTALL.md), and [USAGE.md](USAGE.md)
  describe the desktop boundary, setup, and commands.

## Generated artifacts

- Cargo outputs use `target/` directories and are ignored by
  [../.gitignore](../.gitignore).
- Wheel builds, closure receipts, and similar generated evidence use `output*`
  directories, also ignored by [../.gitignore](../.gitignore).
- Python build products use `build/`, `dist/`, `sdist/`, `site/`, and
  `*.egg-info/`, which the root ignore policy excludes.
- `OTHER_REPOS/` is ignored reference material. Production code and release paths
  must not rely on it being present.

## Documentation map

- [../README.md](../README.md) gives the project entry point and contributor status.
- `docs/CODE_ARCHITECTURE.md` describes ownership boundaries and the Rust-to-Qt
  data flow.
- [active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md) tracks the
  migration milestones and accepted evidence.
- [PROVENANCE.md](PROVENANCE.md) records component lineage and licensing scope.
- [CHANGELOG.md](CHANGELOG.md) records changes for human review.

## Where to add work

- Add Rust source and Cargo tests in the responsible
  [../packages/ferrum-rust/crates/](../packages/ferrum-rust/crates/) crate.
- Add Python bindings only in
  [../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/)
  when a typed Rust boundary is ready.
- Add native-only desktop work under
  `packages/ferrum-chem-qt.app/ferrum_qt/native/` and its focused package tests.
- Add retained-application changes under
  [../packages/ferrum-chem-qt.app/ferrum_qt/](../packages/ferrum-chem-qt.app/ferrum_qt/)
  without conflating them with the native route.
- Add reusable repository checks under [../tests/](../tests/) and artifact-dependent
  scenarios under [../tests/e2e/](../tests/e2e/).
- Add durable documentation in [docs/](.) and active migration artifacts under
  [active_plans/](active_plans/).
