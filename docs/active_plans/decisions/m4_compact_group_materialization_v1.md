# M4 compact-group materialization V1 decision

## Status

Selected. The generic protocol, named CLI forwarding route, and canonical live
PyO3 session registration are complete on the completed M0
complete-render-admission boundary. The Qt action uses the existing generic
live-session operation and returned receipt; it adds no compact chemistry,
geometry, identifiers, or mutation ownership.

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
- Define Rust-issued selected-target eligibility, typed materialization refusal,
  generic CLI delivery, and a future thin Qt action boundary.

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
document snapshot (`cdml`, expected revision, expected digest), and opaque Rust-
issued `molecule_id` and `compact_group_id` target identifiers. The identifiers
are not labels, catalog keys, paths, geometry, formula text, or frontend values.

The generic executor verifies the fence before preparation, invokes the existing
prepare/commit owners, preserves renderer admission, and commits once. Clients
do not supply generated IDs or prepared transition state.

### Success and selection

Success uses the normal committed-document result with canonical CDML, committed
revision, durable document ID, and next fence. Its materialization receipt names
the source target and exposes `replacement_focus_atom_id`, a durable identifier
that resolves in the committed snapshot. The returned focus is authoritative;
clients do not infer it from geometry, labels, or chemistry.

Before Qt action work, Rust must issue a read-only revision/digest-fenced selected
compact-group availability fact. It is either `available` with opaque target IDs
or `unavailable` with one closed reason: `no_selection`, `not_direct_root`,
`not_compact_group`, or `not_materializable`. Qt derives enablement solely from
that fact and the operation revalidates it at commit time.

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
  handler sends the selected Rust-issued target and current fence through the
  generic live operation, installs Rust's result, restores the returned focus,
  and presents typed refusal feedback. It does not implement chemistry.

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
5. Add the shared Qt action only after Rust exposes typed compact selection.

## Verification

- Rust semantic cases cover representative compact groups, returned focus
  resolution, next-fence chaining, and one typed no-change refusal.
- Delivered generic protocol and named-CLI coverage submits canonical JSON,
  consumes the next fence, and asserts the operation kind plus one semantic
  result.
- A Qt E2E is required only after compiled delivery exists. It creates state by
  visible UI, invokes the shared action, and proves availability, focus, save/
  reopen, and undo/redo without pixel, timing, raw-ID, or whole-CDML assertions.
- The live PyO3 proof covers accepted compact-session installation, canonical
  stateless-envelope equivalence, and typed refused no-change behavior.

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
