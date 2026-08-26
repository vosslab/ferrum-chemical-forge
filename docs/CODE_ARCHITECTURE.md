# Code architecture

## Overview

Ferrum Chemical Forge is a pre-alpha CDML chemical-drawing system with two
separately licensed components:

- Ferrum-Chem is the LGPL Rust workspace. It owns CDML documents, chemistry
  boundaries, geometry, rendering operations, local runtime assembly, and the `ferrum` command-line tool.
- Ferrum is the AGPL PySide6 desktop application. It has one packaged,
  Rust-native product route and presents Ferrum-owned documents through Qt.

The canonical full-parity ledger is
[active_plans/active/FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md) is a subordinate
historical implementation record whose `V3-M*` milestones do not close parity.
Historical
OASA and BKChem material is isolated provenance and accepted migration evidence.
Ferrum's Rust engine is the only runtime chemistry backend. Provenance material
is outside the Ferrum-Chem and Ferrum product runtime, dependency declaration,
packaging path, and normal test suite. The adopted
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

### Bounded molecular interchange

CDML is Ferrum's sole document, session, history, and Qt-local format. CML/CML2
is bounded external interchange, not a document format. `ferrum-chemistry` owns
the closed Rust CML2 codec used by CLI import/conversion and Qt File > Open
ingress. The separate API
`ConversionOutputRegistryV1` owns conversion output aliases, profiles, and
preferred suffixes; `InterchangeFormatDescriptorV1` remains the distinct
import and Qt-ingress registry.

The `ferrum convert --to cml|cml2` route emits canonical CML2. `cml1` remains
input-only. Direct CML/CML2 conversion preserves validated source molecule and
atom IDs and record order without an engine runtime; conversions from other
formats refuse any facts that the closed CML2 profile cannot represent
losslessly. Qt File > Open immediately admits valid CML/CML2 into a clean native
CDML tab. It does not export CML or adopt CML as an in-memory document format.

Selected-molecule read-only work has two distinct contracts. The existing
`document.molecule.report.v1` produces its multi-root report receipt. M4
`document.molecule.diagnostics.v1` is narrower: `ferrum-domain` owns the
closed source-representation finding classification, `ferrum-document` resolves
the fenced selected direct roots, and `ferrum-api` owns deterministic bounded
diagnostic DTOs and typed resource refusal. Its snapshot carries CDML, delivery
revision, digest, and durable selected-root IDs. It has no mutation, renderer
admission, external corpus, or chemistry-runtime ownership.

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

The diagnostics executor is a module-level owned-snapshot PyO3 function. Qt
captures CDML, revision, digest, and durable selected-root IDs on the UI thread,
then a detached worker calls that executor with owned values only.
`PyDocumentSession` is unsendable and is never used by the worker.

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

`Check Structure...` is a separate modeless accessible read-only surface for
`document.molecule.diagnostics.v1`. Before presenting a worker result, Qt
authenticates the current tab, revision/digest fence, and selected direct roots.
It renders Rust-owned findings and recovery wording without mutation, auto-fix,
canvas navigation, or selection changes.

Compact-group authoring has two deliberate ownership boundaries. For attached
authoring, Rust owns the closed catalog, selected-atom availability, capacity
and attachment-geometry admission, deferred durable-ID allocation,
complete-render admission, and the atomic history transition. For free
placement, Qt maps the release through its current view snapping; PyO3 accepts
the resulting finite coordinates; Rust validates the typed `Point3V1` and
candidate geometry, then owns anchor/orientation, durable IDs, renderer
admission, and history. Qt does not create a typed snap contract, so Rust does
not claim to prove that an accepted coordinate came from Qt snapping. Qt owns
two distinct accessible chooser workflows: the direct-root `Place Compact
Group...` Me-only chooser and the generic attached `Attach Compact Group...`
chooser for all nine delivered keys: `Me`, `NO2`, `Et`, `OMe`, `CH2OH`,
`Carboxyl`, `Cyano`, `AcylChloride`, and `Phenyl`. Qt also owns one-shot pointer handoffs and
presentation of committed receipts or typed refusals. The private PyO3 bridge
is only the local Qt implementation seam; it does not create a public
attachment contract.
Compact-group materialization is a separate operation after explicit group
selection.

Ferrum-specific end-to-end authoring and validation guidance is in
[FERRUM_E2E_TESTS.md](FERRUM_E2E_TESTS.md). It distinguishes staged public
workflows from Rust, PyO3, and Qt unit-level responsibilities.

For one selected atom with an exact-current unavailable result, Qt keeps the
existing `Attach Compact Group...` action enabled. Its activation reaches the
existing Rust-owned typed refusal, which Qt presents in the standard accessible
`Action Not Available` dialog with the visible learner message `Me cannot
attach to the selected atom. Select another atom and try again.` Dismissing the
dialog refreshes the selection state, so an eligible atom in the same document
uses the same action to open the guarded chooser. Stale, missing, or
nonmatching availability facts instead keep that action disabled with generic
readiness guidance. No new action, schema, or Qt fallback is involved.

The compact-group deletion slice extends the existing Select Structure/Delete
interaction rather than adding a Qt action. The renderer issues the selected
parent molecule and compact-group `DocumentObjectIdV1` values; Rust lowers only
one exact compact target after proving direct membership. Its detached typed
candidate removes the group and its unique exterior bond, then commits one
history transition. Its public receipt reports removed atom, bond, and
compact-group counts; document-private `PersistentId` values remain internal.
Mixed or multi-group selections are refused before preparation, while replay,
Undo, and Redo use the same Rust session authority.

The packaged application contains one Ferrum-owned Qt route with Rust-backed
document state and chemistry bindings. Unsupported historical file forms
are refused at the native file-admission boundary instead of being routed to a
second host.

### Render and projection ownership

Rust render observations cross into Qt through
[../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_render_projection.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_render_projection.py)
and
[../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_presentation_render_plan.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_presentation_render_plan.py).
Rust supplies the frozen renderer plan as Qt's sole visual scene input. The
same accepted observation fence publishes both that plan and SMARTS results.
Qt creates the disposable graphics-scene projection and manages graphics disposal through
`packages/ferrum-chem-qt.app/ferrum_qt/canvas/graphics_disposal.py`.
Qt is therefore a presentation client, not a second CDML document model.

The Qt authoring window has one per-window active-tool owner:
`ferrum.window_mode_sync.FerrumWindowModeSync`. Each feature registers its exact
registry-owned checkable `QAction`, feature-local normalized mode controller,
context, activation, dispatch, and cancellation endpoints beside action
construction. The controller owns checked state, normalized native input, and
typed active-tool publication; menu and YAML ribbon clients remain passive
clients of that exact action. Packaged menu and ribbon YAML load through the neutral
`declarative_resource_loader` leaf and resolve together through the acyclic
window-resource preflight before either visible surface is assembled.

The renderer-admission target is renderer-mints/document-redeems: `ferrum-render`
mints an opaque proof for a candidate bound to a `ferrum-document` issuer and
sequence identity, and `ferrum-document` privately redeems it during commit.
`PreparedSessionTransitionV1` is the document-owned generic lifecycle for the
admitted visual operations. `ferrum-document-render`, Python, and Qt receive
opaque prepared interaction handles and never receive the proof. The generic
route covers terminal, equilibrium, and straight arrows; paths; vectors; plus;
and explicit-hydrogen materialization.

Presentation creation uses a separate transient contract. Renderer-issued,
identifier-free preview plans replay only through the preview-plan builder and
remain noninteractive Qt scene state. Commit instead returns a durable root
receipt, after which Qt installs a committed Rust observation through the normal
render-plan boundary.

The same transition authority owns generic `CreateAtomV1`, `CreateBondV1`, and
`CreateHaworthMoleculeV1` operations. Attached cyclohexane, direct-bond, and
Haworth UI previews receive only a renderer-issued, identifier-free
`DocumentPrecommitOverlayV1` paint value. Wavy and bracket bindings retain
their established supported behavior; none of these adapters receives a raw
render plan, candidate, renderer proof, or alternate commit authority.

## Data flow

The ordinary desktop path is:

```text
local CDML, decoded CD-SVG, or admitted CML/CML2 input
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

The separate local interchange path is:

```text
bounded molecular interchange input
  -> build/bin/ferrum convert
  -> ConversionOutputRegistryV1
  -> Rust chemistry codec or typed losslessness refusal
  -> completed output or one unsuccessful CLI outcome
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
  normal test dependency; backend-comparison evidence remains recorded in the
  migration reports.

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

## Attached compact-group authoring

Attached compact-group authoring uses one Rust-owned `AttachCompactGroupV1`
transaction. Rust projects supported catalog key-and-derived-label choices; the
delivered set is `Me`, `NO2`, `Et`, `OMe`, `CH2OH`, `Carboxyl`, `Cyano`,
`AcylChloride`, and `Phenyl`. A current anchor observation establishes only
general action readiness; Rust evaluates choice-specific availability after
chooser selection. All delivered keys use the supported normal-single profile.
A future key must receive a row-level chooser availability review before it is
admitted. Rust owns chemistry and geometry admission, renderer admission,
durable identity allocation, history, save/reopen, and typed refusals. The
earlier methyl-specific Rust, PyO3, and Qt APIs have been removed without
compatibility aliases.

The native Python binding remains private and session-affine. It exposes
generic choices, availability, begin, preview, commit, and cancel while keeping
prepared candidates opaque. Qt renders the Rust-projected choices and owns only
the accessible chooser plus the one-release canvas capture; it does not derive
recipes or availability.

`CompactGroupRecipeAtomV1` carries an optional formal charge. The canonical
nitro materialization recipe is `R-[N+](=O)[O-]`, and Rust preserves those atom
charges through history and reopen. Rust tests establish the individual `+1`
nitrogen and `-1` oxygen facts; public Molecule Report evidence intentionally
asserts only the net formal charge.

## Known gaps

- Complete the remaining codec, corpus, render-backend, domain, and platform
  work tracked in
  [active_plans/active/FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).
- Extend local-runtime validation as native adapter contracts evolve.
- Keep free placement limited to `Me` until its expanded contract is designed,
  and deliver the planned generic attached compact-group CLI/protocol route.
## Free compact-group placement

`PlaceFreeCompactGroupV1` is the Rust-owned direct-root compact-group
transition. Qt maps the release through its current view-snapping policy and
PyO3 accepts finite coordinates; Rust validates the resulting typed `Point3V1`
and candidate geometry. Rust admits the closed key (`Methyl` is the current
admitted key; other keys return `UnsupportedCatalogKey`), derives canonical
anchor/orientation, allocates molecule-root and compact-group durable IDs, and
prepares a zero-atom, zero-bond candidate with no capacity witness. Complete
renderer admission occurs before the single atomic history transition and
persistence/reload outcome. No typed snap contract currently proves the origin
of the coordinate once it reaches Rust.

The PyO3 binding keeps the pending transition session-affine and opaque: callers may begin, commit, or cancel and receive durable commit facts, but cannot construct or mutate the prepared candidate. Qt owns the distinct `Place Compact Group...` Me-only chooser and a one-release canvas capture. It does not delegate to attached compact-group authoring or template placement. The renderer's current precommit-overlay target is limited to atoms and bonds, so free placement intentionally has no preview overlay.

Direct-root materialization replaces a sole compact group with its immutable
recipe atoms and bonds in the same molecule. A zero-atom, zero-bond free methyl
root becomes one explicit carbon without an exterior rewrite; attached-group
topology remains unchanged. Rust commits this replacement in one history
transition, so Undo, Redo, and reopen operate on the authoritative replacement.
