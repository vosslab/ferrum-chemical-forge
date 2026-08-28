# M4 molecule diagnostics V1 decision

## Status

Selected M4 read-only slice. This decision defines
`document.molecule.diagnostics.v1` and the visible `Check Structure...` route.
It is intentionally narrower than a general molecular report, chemistry engine,
or repair tool. Delivery remains subject to the evidence gates in
[Verification](#verification).

## Stability assessment

The selected operation is runtime-free: it evaluates one immutable document
snapshot and selected durable direct-root molecule IDs. It neither invokes a
chemistry runtime nor changes the document, so it introduces no lifecycle,
mutation, or external-service instability. The remaining risk is integration
correctness across typed Rust, PyO3, CLI, and Qt boundaries; the staged build,
full suite, and independent review below are the required evidence before this
slice is accepted.

## Context

M4 already delivers the snapshot-based `document.molecule.report.v1` contract.
Its follow-on must tell a user whether the selected authored structure has
bounded, actionable structural findings without turning the frontend into a
second chemistry authority. An unexpanded attached compact group is a reachable
public state that demonstrates this need: the finding explains the condition,
and the separately delivered Rust compact-group materialization operation is
the recovery.

## Objectives

- Define deterministic, bounded structural findings for selected direct-root
  molecules.
- Reuse one typed Rust protocol through named CLI, typed PyO3, and a Qt
  read-only inspection surface.
- Make a reachable public E2E prove both diagnosis and the existing Rust-owned
  recovery, without fixtures, raw CDML, or private identifiers.

## Scope

### Request and result

`document.molecule.diagnostics.v1` accepts a fenced document snapshot and
selected durable direct-root molecule IDs. The snapshot contains CDML, revision,
and digest; durable IDs identify the selected current roots. Rust resolves the
IDs against that snapshot, computes findings without a runtime chemistry call,
and returns a typed read-only receipt.

Finding order is deterministic and the result is bounded. The public request
allows at most 128 selected molecule IDs and at most 2 KiB of selector bytes.
Requests exceeding either bound refuse through the typed resource-limit path;
they do not partially select, truncate silently, or execute a best-effort
analysis.

The initially admitted diagnostic categories are structural facts that the
snapshot itself can establish. A missing authored `formal_charge` remains an
intentional unknown source state, not a declared chemical defect.
`IncompleteAuthoredCharge` is reserved for a later contract and is currently
unreachable; no client or documentation may claim it as delivered behavior.

### Rust and CLI boundary

Rust owns protocol decoding, direct-root resolution, deterministic finding
construction, result limits, typed refusal, and all diagnostic wording carried
as stable public data. The named CLI form is
`ferrum document command document.molecule.diagnostics.v1 <input>` alongside
the generic protocol runner. Its operation-kind guard is shared with the
generic command path and rejects a mismatched kind before parsing or executing
the requested operation.

### PyO3 and Qt boundary

`PyDocumentSession` remains `#[pyclass(unsendable)]`. On the UI thread, Qt
captures an owned CDML/revision/digest snapshot and durable selected root IDs.
A detached worker calls a thread-safe module-level or static PyO3 typed executor
with those owned values only. That executor validates the request and digest and
returns typed data; it performs no JSON serialization round trip and never calls
a session-bound method. Selection is admission-only. When the result reaches the
UI thread, Qt authenticates worker/cancel state, a live active ready tab, exact
revision/digest, and the receipt molecule ID/schema before presenting it.

A later selection change or clear does not discard that valid historical
result. The dialog becomes stale and disables rerun until the original molecule
selection is recaptured. While this read-only worker runs, Select Structure
remains available, but selection mutations such as Delete remain disabled.

Qt exposes one explicit `Check Structure...` action and presents authenticated
findings in a modeless, accessible, read-only dialog. Qt owns only action state,
worker/dialog lifetime, focus, and accessible presentation. It does not infer
chemistry, alter findings, mutate the document, or implement recovery. Each
finding's recovery is explanatory: for an unexpanded attached compact group, the
user returns to the existing Rust-owned materialization workflow.

## Non-goals

- Mutation, auto-fix, or diagnostic-triggered document changes.
- Formula, mass, identifiers, oxidation, SMARTS, or runtime chemistry.
- Known-group expansion itself; diagnostics may point to the existing recovery
  but do not perform it.
- Diagnostic-owned canvas highlighting, navigation, or selection changes.
  User navigation and non-mutating selection remain available; selection
  mutations remain disabled while the worker runs.
- Legacy-ID reconstruction or runtime migration heuristics.
- Publishing, installation, or workflow automation outside the local build.

## Verification

- Rust protocol coverage proves direct-root selection, deterministic bounded
  findings, typed malformed/stale/foreign/resource refusals, and no mutation.
- Named CLI coverage proves the shared pre-execution kind guard and one
  semantic diagnostics result.
- Installed PyO3 coverage proves owned-value typed executor transport, digest
  rejection, and execution without a JSON round trip or session-bound worker
  call.
- Focused Qt coverage proves admission-only selection, fenced receipt delivery,
  stale-result/rerun behavior, navigation, and mutation-action gating.
- One registered public E2E authors an attached compact group through visible
  UI, leaves it unexpanded, runs `Check Structure...`, and uses the existing
  public Rust-owned materialization route to recover. It asserts a durable
  user-visible result, not private IDs, raw CDML, pixel equality, arbitrary
  delays, or a fixture catalog.
- A fresh local build, the complete local suite, and independent code review
  must pass before the slice is marked delivered.

## Follow-on boundary

`RecordOrigin::Legacy` remains source-only migration work. It is the next
foundational follow-on for records that originate in imported historical
content, but it is not part of `document.molecule.diagnostics.v1`. This slice
does not add migration behavior, legacy reconstruction, or compatibility
branches.
