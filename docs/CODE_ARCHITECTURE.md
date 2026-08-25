# Code architecture

## Overview

Ferrum Chemical Forge is a pre-alpha CDML chemical-drawing system with two
separately licensed components:

- Ferrum-Chem is the LGPL Rust workspace. It owns CDML documents, chemistry
  boundaries, geometry, rendering operations, local runtime assembly, and the `ferrum` command-line tool.
- Ferrum is the AGPL PySide6 desktop application. It has one packaged,
  Rust-native product route and presents Ferrum-owned documents through Qt.

The active migration contract is
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md). Historical
OASA and BKChem material is retained only as provenance and accepted migration
reports; the live Python backend comparison workers are retired. It is not part
of the Ferrum-Chem or Ferrum product runtime, dependency declaration, packaging
path, or normal test suite. The adopted
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
  owns the project chemistry engine, private adapter loading, and adapter
  response model.
- [../packages/ferrum-rust/crates/document/](../packages/ferrum-rust/crates/document/)
  parses and structurally writes CDML, retains opaque content, preserves
  identity and order, and owns revisioned document sessions.
- [../packages/ferrum-rust/crates/geometry/](../packages/ferrum-rust/crates/geometry/),
  [../packages/ferrum-rust/crates/domain/](../packages/ferrum-rust/crates/domain/),
  and [../packages/ferrum-rust/crates/render/](../packages/ferrum-rust/crates/render/)
  supply lower geometry values, higher-level domain utilities, and renderer-owned
  typed render and presentation operations.
- [../packages/ferrum-rust/crates/api/](../packages/ferrum-rust/crates/api/)
  composes those crates into the `ferrum` executable and its document-native
  artifact publication boundary. Its `protocol_v1` module owns the closed,
  stateless operation request/response DTOs, generated schema, and pure
  owned-value executor described in [USAGE.md](USAGE.md#machine-protocol).

Selected-molecule diagnostics are a read-only boundary: `ferrum-domain` owns
the closed source-representation finding classification, `ferrum-document`
defines the report record boundary for selected direct-root molecules, and
`ferrum-api` exposes `document.molecule.report.v1`. A supplied snapshot carries
CDML, delivery revision, and digest. The route is read-only and has no mutation,
renderer admission, external corpus, or chemistry-engine ownership.

### Native Python extension

[../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/)
builds the direct `ferrum_chem` PyO3 extension. It exposes typed document
sessions, fenced render observations, renderer-issued presentation plans,
chemistry DTOs, and native artifact preparation and publication to Python. Qt
uses this extension rather than parsing a product document itself.

The extension's V1 public automation additions are deliberately narrower:
`execute_operation_v1`, `operation_protocol_schema_v1`, and
`OperationProtocolErrorV1`. They exchange request/response JSON only; the
existing broad extension namespace remains the Ferrum integration surface,
not a blanket third-party API promise.

Native chemistry adapters remain private to their owning Rust and Ferrum
workflows. The local Rust CLI consumes the sealed engine bundle at
`build/runtime/engine-v1/`. `build/runtime/python/` is the `ferrum_chem` Qt and
local-Python extension runtime, not the CLI runtime; neither route has adapter
argument or discovery behavior.

### Ferrum application

[../packages/ferrum-chem-qt.app/pyproject.toml](../packages/ferrum-chem-qt.app/pyproject.toml)
declares one `ferrum-qt` console command. Its product startup chain is:

```text
build/bin/ferrum-qt
  -> ferrum_qt.cli
  -> ferrum_qt.app.main
  -> ferrum_qt.main_window.MainWindow
  -> FerrumNativeMainWindow
```

[../packages/ferrum-chem-qt.app/ferrum_qt/app.py](../packages/ferrum-chem-qt.app/ferrum_qt/app.py)
owns Qt application lifecycle, launch-file admission, controlled smoke support,
and clean shutdown. [../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/main_window.py)
is the ordinary product window and subclasses
[../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/main_window.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/main_window.py).

The [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/)
submodules organize document tabs and publication, local CDML/CD-SVG admission,
editing tools and properties, molecule import and export, user templates,
recovery export, and view controls. A document tab owns one Rust
`DocumentSession`; Rust confirms document changes and replacement render
observations before Qt adopts persistent document state.

The native `Molecule Report...` information surface presents a frozen,
multi-root diagnostic receipt. It renders typed records, aggregate outcome,
findings, and any supplied recovery facts; it does not classify chemistry or
mutate document state. Qt authenticates each returned record against the
captured durable molecule ID. Source ID and source order remain Rust response
facts, rather than Qt-authored identity or ordering.

The attached compact-group slice has one deliberate ownership boundary. Rust
owns the closed catalog, selected-atom availability, capacity and attachment
geometry admission, deferred durable ID allocation, complete-render admission,
and the atomic history transition. Qt owns the accessible `Me` chooser, the
one-shot pointer handoff, and presentation of the committed receipt or typed
refusal. The private PyO3 bridge is only the local Qt implementation seam; it
does not create a public attachment contract. Compact-group materialization is
a separate operation after explicit group selection.

For one selected unavailable atom, Qt presents Rust's advisory availability
result by disabling the existing `Attach Compact Group...` action and assigning
the same learner recovery text to its status tip, tool tip, and What's This:
`Me cannot attach to the selected atom. Select another atom and try again.` A
later eligible selection refreshes that existing action to enabled. Pre-chooser
revalidation still belongs to Rust: a changed selection receives the existing
typed nonmodal refusal and action refresh, not a new action, schema, or Qt
fallback.

The compact-group deletion slice extends the existing Select Structure/Delete
interaction rather than adding a Qt action. The renderer issues the selected
parent molecule and compact-group `DocumentObjectIdV1` values; Rust lowers only
one exact compact target after proving direct membership. Its detached typed
candidate removes the group and its unique exterior bond, then commits one
history transition. Its public receipt reports removed atom, bond, and
compact-group counts; document-private `PersistentId` values remain internal.
Mixed or multi-group selections are refused before preparation, while replay,
Undo, and Redo use the same Rust session authority.

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
Rust supplies the frozen renderer plan as Qt's sole visual scene input. The
same accepted observation fence publishes both that plan and SMARTS results.
Qt creates the disposable graphics-scene projection and manages graphics retirement through
[../packages/ferrum-chem-qt.app/ferrum_qt/canvas/graphics_retirement.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/graphics_retirement.py).
Qt is therefore a presentation client, not a second CDML document model.

The renderer-admission target is renderer-mints/document-redeems: `ferrum-render`
mints an opaque proof for a candidate bound to a `ferrum-document` issuer and
sequence identity, and `ferrum-document` privately redeems it during commit.
`PreparedSessionTransitionV1` is the document-owned generic lifecycle for the
admitted visual operations. `ferrum-document-render`, Python, and Qt receive
opaque prepared interaction handles and never receive the proof. The generic
route covers terminal, equilibrium, and straight arrows; paths; vectors; plus;
and explicit-hydrogen materialization.

The same transition authority owns generic `CreateAtomV1`, `CreateBondV1`, and
`CreateHaworthMoleculeV1` operations. Attached cyclohexane, direct-bond, and
Haworth UI previews receive only a renderer-issued, identifier-free
`DocumentPrecommitOverlayV1` paint value. Wavy and bracket bindings retain
their established supported behavior; none of these adapters receives a raw
render plan, candidate, renderer proof, or alternate commit authority.

## Data flow

The ordinary desktop path is:

```text
local CDML or decoded CD-SVG input
  -> build/bin/ferrum-qt
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
  -> build/bin/ferrum protocol
  -> ferrum-api
  -> protocol_v1 owned-value executor
  -> CDML/document or complete-artifact operation
  -> one JSON success or typed-error envelope
```

The matching `ferrum_chem.execute_operation_v1` binding uses the same owned-value
executor. Direct Rust and Ferrum-native library calls remain separate private
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
- [../build.sh](../build.sh) assembles the local CLI, Qt launcher, extension,
  and dynamic-library closure below `build/`; [../all_test.sh](../all_test.sh)
  runs repository hygiene and product suites against that runtime.
- Ferrum behavior tests live in
  [../packages/ferrum-chem-qt.app/tests/](../packages/ferrum-chem-qt.app/tests/).
  They exercise the ordinary product window and focused native boundaries.
- Repository documentation and policy checks live in [../tests/](../tests/).
- [../tests/e2e/reference/](../tests/e2e/reference/) is an optional Python RDKit
  environment for one-time maintainer measurements. It is not a product or
  normal test dependency; the retired backend comparison survives only as
  recorded migration evidence.

## Extension points

- Add chemical operations behind the Rust engine and explicit adapter contracts.
- Add CDML behavior in the document crate, preserving opaque records, stable
  identifiers, order, revision semantics, and structural output.
- Add geometry and rendering logic in Rust before adding its Qt projection.
- Add desktop capability in the corresponding
  [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/)
  submodule, with Qt as the typed Rust client.
- Add a focused, deterministic behavior test at the boundary it protects; use
  disposable local evidence for package and visual checks that do not warrant a
  permanent suite test.

## Known gaps

- Complete the remaining codec, corpus, render-backend, domain, and platform
  work tracked in [active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).
- Extend local-runtime validation as native adapter contracts evolve.
