# Renderer admission dependency-inversion plan

## Purpose

Every visual document mutation must be admitted by the complete renderer plan
before it can change document state. `ferrum-document` owns that transaction:
it builds the prospective immutable observation, asks `ferrum-render` to admit
it, retains the opaque proof, and redeems that exact proof inside the atomic
commit. `ferrum-document-render`, API, PyO3, CLI, and Qt remain interaction
adapters around opaque prepared document operations.

This active plan records the remaining migration routes. It does not introduce
an alternate renderer, source-string admission, temporary document session, or
publication/install workflow.

## Approved ownership boundary

```
ferrum-document-model
        |
ferrum-document-projection
        |
ferrum-render
        |
ferrum-document
        |
ferrum-document-render
        |
ferrum-api / local CLI / PyO3 / Qt
```

`ferrum-document-projection` owns immutable render-facing DTOs.
`ferrum-render` owns complete-plan construction, Telex verification, geometry,
clipping, bounds, and closed rendering refusals. `ferrum-document` owns
retained CDML traversal, session fences, history, generated IDs, pending
identity, and atomic mutation. Its private `RendererAdmittedPendingV1` binds
the renderer proof to a session-minted issuer and sequence, preventing an
equal-content preparation from redeeming another pending operation's proof.

The renderer has no document-session dependency. The document has no public
raw candidate/proof getter. Outer facades have neither mutation authority nor
an alternate commit path.

## Admission protocol

1. A `DocumentSession::prepare_*` operation constructs its prospective state
   and mints a private pending identity before history or durable generated IDs
   change.
2. The document derives an immutable `DocumentRenderCandidateV1` that includes
   that identity and asks the renderer to construct the real complete plan.
3. The renderer either issues a closed refusal or returns an opaque,
   non-constructible, non-serializable proof bound to the candidate and pending
   identity.
4. The private document pending value retains the candidate, proof, fence,
   generated-ID reservation, and operation-specific commit result.
5. Commit verifies the exact current pending state and redeems the proof
   immediately before the existing atomic transaction. Stale, foreign,
   replayed, refused, and non-consuming document failures preserve document
   content, history, and durable generated IDs.

Private pending-sequence advancement is intentionally not a durable mutation;
it may advance during a refused admission.

## Route ledger

The following visual mutation families are complete. Their document-owned
prepare/commit values admit and redeem `RendererAdmittedPendingV1`; outer
facades retain opaque interaction handles and renderer-issued overlays only.

| Family | Document-owned pending value |
| --- | --- |
| Direct bond | `PendingDirectBondMutationV1` |
| Complete-CDML reactions | `PendingCompleteCdmlMutationV1` |
| Presentation roots and arrows | `PendingCreatePresentationV1` |
| Explicit hydrogen materialization | `PendingHydrogenMaterializationV1` |
| Compact-group materialization | `PendingCompactGroupMaterializationV1` |
| Compact-group placement | `PendingCompactGroupPlacementV1` |
| Catalog molecule placement, including catalog Haworth | `PendingCatalogMoleculePlacementV1` |

The proof is held and redeemed inside `ferrum-document` for every ledger row.
`ferrum-document-render` supplies gesture capture, pointer and hit evidence,
selection, disposable preview orchestration, and user-facing error mapping; it
does not hold or verify the renderer proof.

## Remaining migration inventory

This plan remains active because these visual routes still need classification
and migration to the document-owned admission pattern:

- raw molecule, SMILES, regular-ring, explicit-fragment, batch, and
  interchange insertion;
- direct and standalone Haworth authoring, plus attached cyclohexane;
- primitive atom, bond, bonded-atom, bracket, and wavy construction;
- structural deletion;
- linear-form conversion and other internal or test-reachable mutation
  variants pending classification.

For each route, first establish whether it commits visual state. A visual
commit receives a document-owned pending operation and renderer admission. A
preview-only interaction stays in `ferrum-document-render` without commit
authority. A nonvisual metadata operation records its rationale in this ledger
before it is treated as outside the protocol.

## Implementation sequence

1. Map each remaining route from public entry point through its document commit
   boundary and classify it as visual commit, preview-only interaction, or
   nonvisual metadata.
2. Move one coherent visual family at a time to a private document pending
   value that admits the prospective observation and redeems the exact proof.
3. Retain public request/result and closed refusal behavior while deleting the
   superseded candidate, receipt, temporary-session, or equality bridge.
4. Update this ledger and add durable behavior coverage for accepted commit,
   renderer refusal without durable state change, and stale/foreign/replayed
   pending identity where that boundary is public.
5. Build the local program with `./build.sh`, run `./all_test.sh`, and perform
   an independent architecture review after a coherent migration checkpoint.

## Evidence policy

Permanent tests prove stable user-visible and mutation-boundary behavior. They
cover successful one-time commit, renderer refusal preserving document/history/
durable generated IDs, and public stale or replay refusal where applicable.
They do not assert private holder topology, exact internal call counts, sleep,
network behavior, or pixel equivalence.

One-time implementation checks may compare old and new paths, inspect extreme
geometry, or capture local GUI screenshots. They establish migration evidence
without becoming permanent topology tests.

## Completion criteria

The plan completes when every remaining visual mutation route is either moved
to document-owned renderer admission or recorded as a genuine nonvisual
operation; no public mutation can bypass the complete plan; obsolete bridges
are removed; and current local-build, full-suite, and independent-review
evidence is green.
