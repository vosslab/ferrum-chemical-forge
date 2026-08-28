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
- [../packages/ferrum-rust/crates/template-catalog/](../packages/ferrum-rust/crates/template-catalog/)
  owns bounded shipped and user-template catalog snapshots, immutable identity,
  typed refusals, and Rust-side catalog application.
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
- [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/template_catalog_dialog.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/template_catalog_dialog.py),
  [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/template_catalog_controller.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/template_catalog_controller.py),
  and [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/operation_leases.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/operation_leases.py)
  form the implemented Template Catalog Patch 1 slice: dialog presentation;
  controller-owned dialog, pointer, and payload context; and pure Qt
  registered-tab lifecycle identity/state. The explicit native placement port
  is [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/document_tab.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/document_tab.py).
- [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_contract.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_contract.py),
  [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_composition.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_composition.py),
  [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_controller.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_controller.py),
  and [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_delivery.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_delivery.py)
  form the Local Document Open Patch 2 boundary. They own immutable facts,
  host callbacks, and publication/commit receipts; concrete window composition;
  request, queue, action, lease, startup, and terminal orchestration; then
  per-intent staged worker facts, named queued receiver, fence checks,
  candidate construction, receipt validation, and presentation. The lifecycle
  module is the sole host-publication and registered replacement-commit owner:
  it returns a receipt before separate post-commit display work. Rust remains
  local admission authority. The deleted `local_document_open.py` mixin and
  unused `native_app.py` have no replacement compatibility module;
  [../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py)
  is the sole product startup composition.
- [../packages/ferrum-chem-qt.app/ferrum_qt/canvas/](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/)
  contains the Rust-observation projections and Qt graphics lifetime helpers.
- [../packages/ferrum-chem-qt.app/ferrum_qt/dialogs/](../packages/ferrum-chem-qt.app/ferrum_qt/dialogs/),
  [../packages/ferrum-chem-qt.app/ferrum_qt/themes/](../packages/ferrum-chem-qt.app/ferrum_qt/themes/),
  and [../packages/ferrum-chem-qt.app/ferrum_qt/resources/](../packages/ferrum-chem-qt.app/ferrum_qt/resources/)
  hold shared Qt dialogs, appearance support, and packaged assets.
- [../packages/ferrum-chem-qt.app/tests/](../packages/ferrum-chem-qt.app/tests/)
  contains focused application behavior tests and native route E2E tests.

The application tree contains Ferrum-owned Qt presentation, Rust-backed
chemistry bindings, and their current tests and resources. The ordinary package
is the supported application route.

### Qt structure direction

The present `ferrum_qt/ferrum/` layout remains a flat set of vertical feature
modules. This is deliberate while parity work continues: the Rust workspace,
PyO3 extension boundary, Qt support subpackages, action registry, and YAML
declarations are already meaningful ownership boundaries.

Patch 1 replaced the Template Catalog's ordered mixin pair with an explicit
controller and a single `OperationLeaseRegistry` for busy, close, cancellation,
and action-refresh ownership. The lifecycle mixin owns one explicit close
adapter shared by both window hosts, while `tests/e2e/ferrum_qt_e2e.py` owns
the common `close_e2e_main_window` explicit-DISCARD teardown. Patch 2 applies
the same atomic replacement to Local Document Open: the contract,
composition, controller, and delivery modules replace the mixin without a
compatibility alias. Keep this flat
feature-module structure while parity work continues. Do not create speculative
`application/` or `documents/` folders: the established Rust workspace, PyO3,
Qt support packages, action registry, and YAML declarations already express the
current durable boundaries. Each later migration must still move one tested
family atomically without a general event bus or duplicated state.

Within that slice, the lifecycle boundary is intentionally narrower than the
delivery boundary. It owns registered-tab publication, exact replacement commit,
and `LocalOpenNewTabPublicationReceipt` or `LocalOpenReplacementCommitReceipt`
issuance. Delivery retains its candidate until it validates that receipt, then
uses the separate finish callback for optional activation and display. This
prevents presentation exceptions from making the committed owner ambiguous.

The lifecycle module also owns the completed Patch 1 registered replacement
repair for pristine Local Open. Phase 1 fully integrates and binds the new tab
before publication. Phase 2 can restore the exact old registration after typed
old-unregister/disposal refusal, while a failed provisional tab is completely
retired, unbound, disposed, and stripped of product hooks. Phase 3 commits the
irreversibly disposed old tab without fictional rollback. Shutdown settles the
catalog and retires clean tabs via ordinary registry-aware close; dirty user
close is a presented refusal and deterministic tests discard their own tabs.
Queued presentation callbacks are context-bound to their owning Qt window. This
is the historical Patch 1 lifecycle boundary; Patch 2 separately owns Local
Document Open admission and source-tab lease state.

### `tests/` and `docs/`

- [../tests/e2e/](../tests/e2e/) contains corpus inputs and cross-package
  scenarios.
- `test_local_document_open_controller.py`,
  `test_local_document_open_delivery.py`,
  `test_local_document_open_terminal_replacement.py`, and
  `test_registered_native_tab_lifecycle.py` are the focused Local Document Open
  test owners.
- `e2e_local_document_open_lifecycle.py` is the registered terminal Local
  Document Open E2E in
  [../tests/e2e/run_all.sh](../tests/e2e/run_all.sh).
- [../tests/e2e/reference/](../tests/e2e/reference/) is an optional Python RDKit
  environment for one-time maintainer measurements only.
- [../tests/](../tests/) also contains repository policy and documentation checks.
- [active_plans/](active_plans/) contains migration plans, decisions, audits,
  reports, and workstreams. The active plan is the current status authority.
- [QT_CONTRACT.md](QT_CONTRACT.md), [FILE_FORMATS.md](FILE_FORMATS.md), and
  [USAGE.md](USAGE.md) record the desktop boundary, supported formats, and user
  commands.

## Generated artifacts

- Local developer build products use `build/`. Each retained runnable product
  is one immutable `build/programs/program-*/` root, selected atomically by
  `build/current`. `build/bin/` and `build/runtime/` are stable links through
  that pointer, retaining the CLI, Qt launcher, extension, and sealed runtime
  command paths without independently promoted output trees.
- Every immutable program root has a `.ferrum-runtime.lease` inode. Generated
  CLI and Qt launchers hold its shared lease while running; cleanup preserves a
  lease-held root and reclaims only inactive, non-current owned `program-*`
  roots.
- Cargo intermediates are disposable per-owner subdirectories of `build/`.
  `build/.cargo-target-<opaque-id>/` belongs to one `build.sh` invocation and
  is removed on success, failure, interruption, or the next locked startup.
  `build/.cargo-check-target/` belongs to `check_rust.sh` and is removed after
  its gate finishes.
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
- [active_plans/active/FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md)
  is the canonical Rust-first parity ledger. [active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md)
  retains historical implementation milestones and accepted evidence.
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
- For a future Qt host-composition migration, start with one feature family and
  its close/cancel/action-refresh behavior. Add a typed controller port and
  lease ownership, migrate callers and tests together, then remove its mixin.
- Add focused behavior coverage with the owning application or Rust crate; put
  artifact-dependent scenarios under [../tests/e2e/](../tests/e2e/).
- Add durable documentation in [docs/](.) and active migration records under
  [active_plans/](active_plans/).
