# Code architecture

## Overview

Ferrum Chemical Forge is a pre-alpha CDML chemical-drawing system with two
separately licensed components:

- Ferrum-Chem is the LGPL Rust workspace. It owns CDML documents, chemistry
  boundaries, geometry, rendering operations, native artifact publication, and
  the `ferrum` command-line tool.
- Ferrum-Qt is the AGPL PySide6 desktop application. It has one packaged,
  Rust-native product route and presents Ferrum-owned documents through Qt.

The active migration contract is
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md). Historical
OASA and BKChem material is retained only as isolated provenance and oracle
input; it is not part of the Ferrum-Chem or Ferrum-Qt product runtime,
dependency declaration, packaging path, or normal test suite. The adopted
historical format and behavior references are
[CDML_BACKEND_TO_FRONTEND_CONTRACT.md](CDML_BACKEND_TO_FRONTEND_CONTRACT.md) and
[CDML_FORMAT_SPEC.md](CDML_FORMAT_SPEC.md).

## Major components

### Rust workspace

[../packages/ferrum-rust/Cargo.toml](../packages/ferrum-rust/Cargo.toml) defines
the edition-2024 workspace. Its crates divide responsibility as follows:

- [../packages/ferrum-rust/crates/core/](../packages/ferrum-rust/crates/core/)
  provides chemical-domain records and graph-facing data.
- [../packages/ferrum-rust/crates/chemistry/](../packages/ferrum-rust/crates/chemistry/)
  owns the project chemistry engine and adapter response model.
- [../packages/ferrum-rust/crates/chemistry-sys/](../packages/ferrum-rust/crates/chemistry-sys/)
  validates and loads the explicit native chemistry adapter boundary.
- [../packages/ferrum-rust/crates/document/](../packages/ferrum-rust/crates/document/)
  parses and structurally writes CDML, retains opaque content, preserves
  identity and order, and owns revisioned document sessions.
- [../packages/ferrum-rust/crates/geometry/](../packages/ferrum-rust/crates/geometry/),
  [../packages/ferrum-rust/crates/domain/](../packages/ferrum-rust/crates/domain/),
  and [../packages/ferrum-rust/crates/render/](../packages/ferrum-rust/crates/render/)
  supply display geometry, higher-level domain utilities, and typed render and
  presentation operations.
- [../packages/ferrum-rust/crates/api/](../packages/ferrum-rust/crates/api/)
  composes those crates into the `ferrum` executable and its document-native
  artifact publication boundary. Its `protocol_v1` module owns the closed,
  stateless operation request/response DTOs, generated schema, and pure
  owned-value executor described in [USAGE.md](USAGE.md#operation-protocol-v1).

### Native Python extension

[../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/)
builds the direct `ferrum_chem` PyO3 extension. It exposes typed document
sessions, render observations, display geometry, chemistry DTOs, and native
artifact preparation and publication to Python. Qt uses this extension rather
than parsing a product document itself.

The extension's V1 public automation additions are deliberately narrower:
`execute_operation_v1`, `operation_protocol_schema_v1`, and
`OperationProtocolErrorV1`. They exchange request/response JSON only; the
existing broad extension namespace remains the Ferrum-Qt integration surface,
not a blanket third-party API promise.

Native chemistry adapters remain private to their owning Rust and Ferrum-Qt
workflows; the shipping CLI has no adapter argument or discovery behavior. The
native-wheel tooling in
[../packages/ferrum-rust/tools/](../packages/ferrum-rust/tools/) verifies the
packaged extension closure; it does not make a cross-platform desktop-release
claim.

### Ferrum-Qt application

[../packages/ferrum-chem-qt.app/pyproject.toml](../packages/ferrum-chem-qt.app/pyproject.toml)
declares one `ferrum-qt` console command. Its product startup chain is:

```text
ferrum-qt
  -> ferrum_qt.cli
  -> ferrum_qt.app.main
  -> ferrum_qt.main_window.MainWindow
  -> FerrumNativeMainWindow
```

[../packages/ferrum-chem-qt.app/ferrum_qt/app.py](../packages/ferrum-chem-qt.app/ferrum_qt/app.py)
owns Qt application lifecycle, launch-file admission, controlled smoke support,
and clean shutdown. [../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py)
is the ordinary product window and subclasses
[../packages/ferrum-chem-qt.app/ferrum_qt/native/ferrum_native_main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/native/ferrum_native_main_window.py).

The [../packages/ferrum-chem-qt.app/ferrum_qt/native/](../packages/ferrum-chem-qt.app/ferrum_qt/native/)
submodules organize document tabs and publication, local CDML/CD-SVG admission,
editing tools and properties, molecule import and export, user templates,
recovery export, and view controls. A document tab owns one Rust
`DocumentSession`; Rust confirms document changes and replacement render
observations before Qt adopts persistent document state.

The old compatibility host, its session and worker layers, legacy action and
mode families, compatibility codecs, and their menu and mode resources have
been removed from the packaged application. Unsupported historical file forms
are refused at the native file-admission boundary instead of being routed to a
second host.

### Render and projection ownership

Rust render observations cross into Qt through
[../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_render_projection.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_render_projection.py)
and
[../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_presentation_projection.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_presentation_projection.py).
Rust supplies ordered geometry and resolved presentation facts. Qt creates the
disposable graphics-scene projection and manages graphics retirement through
[../packages/ferrum-chem-qt.app/ferrum_qt/canvas/graphics_retirement.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/graphics_retirement.py).
Qt is therefore a presentation client, not a second CDML document model.

## Data flow

The ordinary desktop path is:

```text
local CDML or decoded CD-SVG input
  -> ferrum-qt
  -> app.MainWindow
  -> FerrumNativeMainWindow and FerrumNativeDocumentTab
  -> ferrum_chem.DocumentSession
  -> Rust document observation and render observation
  -> Qt projection
  -> confirmed native save or native artifact publication
```

This route handles the bounded Ferrum desktop behavior recorded in
[QT_CONTRACT.md](QT_CONTRACT.md) and [FILE_FORMATS.md](FILE_FORMATS.md). It
keeps document authority in Rust while preserving the user's window, selection,
and view state through confirmed updates.

The public CLI path has no Python UI runtime, session, adapter, or Qt state:

```text
one bounded JSON request
  -> ferrum protocol
  -> ferrum-api
  -> protocol_v1 owned-value executor
  -> CDML/document or complete-artifact operation
  -> one JSON success or typed-error envelope
```

The matching `ferrum_chem.execute_operation_v1` binding uses the same owned-value
executor. Direct Rust and Ferrum-Qt-native library calls remain separate private
integration seams; they do not extend the public CLI transport.

Desktop sessions are thread-confined. Each ordinary native tab owns one unsendable
PyO3 session on the Qt GUI thread; mutations return through that owner and are serialized
by the GUI event queue. Worker paths prepare owned, immutable observations or detached
one-use receipts, then the GUI owner rechecks its tab, revision, digest, and intent before
accepting a result. This preserves one document authority without making a throughput or
timing promise.

Its envelope is bounded before transport allocation/copy/parsing, then uses the
existing CDML and artifact-completion resource policies. This is a safety
boundary, not a performance target.

## Testing and verification

- Workspace crates use Cargo unit and integration tests from
  [../packages/ferrum-rust/](../packages/ferrum-rust/).
- The wheel boundary is built by
  [../packages/ferrum-rust/tools/build_native_wheel.py](../packages/ferrum-rust/tools/build_native_wheel.py)
  and has root E2E coverage in
  [../tests/e2e/e2e_native_wheel.py](../tests/e2e/e2e_native_wheel.py).
- Ferrum-Qt behavior tests live in
  [../packages/ferrum-chem-qt.app/tests/](../packages/ferrum-chem-qt.app/tests/).
  They exercise the ordinary product window and focused native boundaries.
- Repository documentation and policy checks live in [../tests/](../tests/).
- [../tests/e2e/oracle/](../tests/e2e/oracle/) is an isolated differential
  comparison environment. Its optional historical dependencies and ignored
  external references are not product requirements.

## Extension points

- Add chemical operations behind the Rust engine and explicit adapter contracts.
- Add CDML behavior in the document crate, preserving opaque records, stable
  identifiers, order, revision semantics, and structural output.
- Add geometry and rendering logic in Rust before adding its Qt projection.
- Add desktop capability in the corresponding
  [../packages/ferrum-chem-qt.app/ferrum_qt/native/](../packages/ferrum-chem-qt.app/ferrum_qt/native/)
  submodule, with Qt as the typed Rust client.
- Add a focused, deterministic behavior test at the boundary it protects; use
  disposable local evidence for package and visual checks that do not warrant a
  permanent suite test.

## Known gaps

- Complete the remaining codec, corpus, render-backend, domain, and platform
  work tracked in [active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).
- Reconcile historical ABI evidence with the current adapter and wheel evidence
  when the active plan next records that boundary.
