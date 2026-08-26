# M4 compact-group materialization V1 decision

## Status

Selected. The generic protocol, named CLI forwarding route, canonical live
PyO3 session registration, and Qt materialization action are delivered on the
completed M0 complete-render-admission boundary. The durable-target and
availability migration is complete: Qt uses the same generic live-session
operation and returned receipt without compact chemistry, geometry, identifier
recovery, or mutation ownership. The cross-cutting identity rule is recorded
in [FERRUM_API_CONTRACT.md](../../FERRUM_API_CONTRACT.md#compact-group-materialization).

## Context

Ferrum already has a typed compact-group representation and an internal
materialization experiment. [m0_complete_render_admission_v1.md](m0_complete_render_admission_v1.md)
records M0 closure on 2026-08-24, so the selected M4 operation builds on the
completed document-owned candidate admission, one-use preparation, atomic
history, and Rust-issued durable identity. The public stateless materialization
route is delivered through the generic protocol and CLI only.

## Objectives

- Define one document-owned operation for materializing a selected compact group.
- Preserve one generic protocol and CLI transport without a special parser.
- Deliver one Qt action only after Rust exposes its typed compact selection and
  generic live-operation receipt surface.

## Design philosophy

Apply **Fix the design, not the symptom** and **Design for adaptability** from
[REPO_STYLE.md](../../REPO_STYLE.md): retain existing document transition
ownership instead of adding catalog, CLI, or frontend-specific mutation paths.

## Scope

- Select `document.compact-group.materialize.v1` as a distinct generic
  operation for one fenced direct-root compact group.
- Reuse the existing document-owned compact-group materialization prepare and
  commit owners, including renderer admission, one-use transition ownership,
  history, generated IDs, and canonical next fence.
- Define Rust-issued fenced selected-target availability, typed materialization
  refusal, generic CLI delivery, and the thin delivered Qt action boundary.

## Non-goals

- Parse formula text or expand arbitrary text-to-chemistry input.
- Support legacy implicit-group CDML or port legacy expansion helpers.
- Expand multiple attachments, batches, or all compact groups in one request.
- Recreate chemistry, selection, direct CDML mutation, or focus inference in Qt.
- Extend the M5 catalog, reaction, catalog-row, or reaction-translation scope.
- Add a legacy alias, free-form group label, formula parser, or recipe input.

## Resolved decisions

### Operation and request

The operation kind is exactly `document.compact-group.materialize.v1`. It is a
closed variant of the existing generic operation protocol and uses the canonical
request/envelope transport. It carries only the request schema and ID, a fenced
document snapshot (`cdml`, expected revision, expected digest), and serialized
`DocumentObjectIdV1` molecule and compact-group target identifiers. Rust parses
and resolves those opaque durable selectors in the admitted snapshot; they are
not labels, catalog keys, paths, geometry, formula text, or frontend values.

The generic executor verifies the fence before preparation, invokes the existing
prepare/commit owners, preserves renderer admission, and commits once. Clients
do not supply generated IDs or prepared transition state.

### Success and selection

Success uses the normal committed-document result with canonical CDML, committed
revision, durable document ID, and next fence. Its materialization receipt names
the source target and exposes `replacement_focus_atom_id`, a durable identifier
that resolves in the committed snapshot. The returned focus is authoritative;
clients do not infer it from geometry, labels, or chemistry.

Rust issues a read-only revision/digest-fenced selected compact-group
availability observation for one durable molecule/group pair. Its closed
outcomes are `Eligible`, `StaleDocumentFence`, `UnknownOrForeignTarget`,
`IneligibleTarget`, and `RendererPreparationRefused`. Qt recognizes only its
local selection state; for a concrete durable pair it derives enablement solely
from `Eligible`. The operation revalidates the same session-owned eligibility
during preparation and commit.

The stateless operation and live session both use `DocumentObjectIdV1` target
selectors. Qt render targets retain visual/render identity separately from
durable object and owner-molecule identity, so no identity conversion or
raw-CDML selection payload can enter the live operation.

### Refusal and recovery

Materialization needs a compact-group-specific typed refusal within the existing
execution failure channel because its recovery differs from catalog placement.
The refusal retains a stable category and recovery rather than message matching.

| Category | Recovery | Durable effect |
| --- | --- | --- |
| `stale_document_fence` | Refresh the selection fact and retry. | Document unchanged. |
| `unknown_or_foreign_target` | Refresh and select a current direct-root group. | Document unchanged. |
| `ineligible_target` | Select an eligible compact group. | Document unchanged. |
| `renderer_preparation_refused` | Preserve the document and show the supplied reason. | Document unchanged. |
| `session_conflict_or_replayed_preparation` | Refresh and restart; never reuse preparation. | Document unchanged. |

Malformed protocol input and resource-limit refusals remain on their existing
typed paths. Recovery data must not expose candidate recipes, pending values, or
source CDML.

### Delivery boundaries

- CLI uses only the existing generic `ferrum protocol run <input>` and `ferrum
  document command document.compact-group.materialize.v1 <input>` forms. No
  `expand-group` parser, known-group flags, or second request shape is approved.
- Qt provides one accessible `Materialize Selected Compact Group` QAction. Its
  handler sends the selected durable Rust-issued target and current fence
  through the generic live operation, installs Rust's result, restores the
  returned durable focus, and presents typed refusal feedback. Its enablement
  comes only from the Rust-issued fenced availability observation; it does not
  implement chemistry.

## Approach

1. Complete the typed document replacement transaction through the established
   complete-render-admission boundary.
2. Add the closed generic protocol request, success receipt, and typed
   refusal/recovery mapping in Rust.
3. Dispatch through the generic executor and existing CLI forms, exercising the
   current prepare/commit owners exactly once. This is complete.
4. Reuse the generic live PyO3 operation receipt for Qt delivery; canonical
   live dispatch now executes the existing compact session transition without a
   compact-specific Python mutation method.
5. Drive the delivered shared Qt action from Rust's fenced availability
   observation and explicit durable render-target identity.

## Verification

- Rust semantic coverage proves representative compact groups, returned durable
  focus resolution, next-fence chaining, availability outcomes, and typed
  no-change refusals.
- Generic protocol and named-CLI coverage submits canonical JSON, consumes the
  next fence, and asserts the operation kind plus one semantic result.
- No public compact-materialization Qt E2E is retained: the former scenario
  used raw CDML setup and a file-dialog mock, so it did not prove public UI
  behavior. Semantic CLI materialization and native fenced-availability
  coverage are the current permanent evidence.
- Add a public Qt E2E only when an approved public UI authoring route can
  create the required compact-group state. It must use visible actions through
  materialization and assert a durable user-visible outcome without raw CDML,
  file-dialog replacement, private controller/session access, raw IDs, timing,
  or pixel equality.
- Live PyO3 coverage proves accepted compact-session installation,
  durable-target validation, and typed refused no-change behavior. Stateless
  response coverage retains its separate admitted-snapshot source-ID contract.

## Risks and blockers

| Risk | Trigger | Mitigation | Owner |
| --- | --- | --- | --- |
| Admission bypass | Public route bypasses the completed generic admission boundary. | Reuse generic preparation and one-use commit without route-specific receipts. | Rust document/render owner |
| Qt chemistry duplication | Action enablement reconstructs target eligibility. | Use only the Rust-issued availability fact. | Qt owner |
| Replay corruption | A prepared transition is reused after a conflict. | Retain one-use typed no-change refusal. | Rust document owner |
| Scope expansion | Formula, legacy CDML, or M5 work enters the operation. | Keep the operation target-specific and fenced. | M4 owner |

## Files to modify

- [FULL_PARITY_RUST_FIRST.md](../active/FULL_PARITY_RUST_FIRST.md): retain M4
  status and link this decision as the compact-materialization authority.
- [USAGE.md](../../USAGE.md) and [FERRUM_API_CONTRACT.md](../../FERRUM_API_CONTRACT.md):
  document the delivered closed public operation without broadening its scope.
