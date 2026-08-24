# Renderer admission dependency-inversion plan

## Purpose

Every visual document mutation must be admitted by the complete renderer plan
before it can change document state. `ferrum-document` owns that transaction:
it builds the prospective immutable observation, asks `ferrum-render` to admit
it, retains the opaque proof, and redeems that exact proof inside the atomic
commit. `ferrum-document-render`, API, PyO3, CLI, and Qt remain interaction
adapters around opaque prepared document operations.

This active plan records the current renderer-admission checkpoint. It covers
the local program built by `./build.sh`; publication, installation, and hosted
workflow work are outside this plan.

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
retained CDML traversal, session fences, history, generated IDs, prepared
transition state, and atomic mutation. Its private admitted-transition state binds
the renderer proof to a session-minted issuer and sequence, preventing an
equal-content preparation from redeeming another operation's proof.

`AdmittedHistoryV1` is private to the admitted-transition core. Route modules
receive immutable current-state access and construct prepared transitions;
only the core appends, replaces, or navigates history. The renderer has no
document-session dependency. The document has no public raw candidate/proof
getter. Outer facades have neither mutation authority nor an alternate commit
path.

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

## Current route ledger

`PreparedSessionTransitionV1` is the document-owned generic lifecycle for
admitted visual operations. It prepares the candidate state, retains the
renderer proof privately, and redeems that proof only during the atomic commit.
Outer facades retain opaque prepared handles and renderer-issued overlays.

| Generic visual family | Current M0 authority |
| --- | --- |
| Terminal, equilibrium, and straight arrows | Generic session transition |
| Presentation paths and vectors | Generic session transition |
| Standard plus | Generic session transition |
| Explicit-hydrogen materialization | Generic session transition |
| Atom, bond, and Haworth construction | Generic session transition |
| Attached cyclohexane | Generic session transition |

`ferrum-document-render` supplies gesture capture, pointer and hit evidence,
selection, disposable preview orchestration, and user-facing error mapping. It
does not hold or verify the renderer proof. Public compact-group authoring is
removed from M0; M1 is the earliest authorized public compact-group surface.

For the admitted visual construction routes, Qt and PyO3 paint only the
renderer-issued `DocumentPrecommitOverlayV1`: an immutable, identifier-free
paint value. It is not a render-plan transport, candidate reconstruction API,
or alternate commit capability. Wavy and bracket methods remain supported at
their existing PyO3 boundary and are not compatibility wrappers.

The refreshed Graphify index provides current navigation evidence for the
route and dependency inventory, including molecule and batch/import routes,
direct and standalone Haworth, attached cyclohexane, primitive and
bracket/wavy construction, structural deletion, and linear-form conversion.
Source inspection, focused behavior tests, and independent review establish
that those visual commits use the admitted-transition design; aggregate graph
counts alone do not prove individual route ownership.

## Historical checkpoint evidence and remaining work

- Qt complete-root translation now uses the renderer-admitted gesture lifecycle
  in `direct_root_interaction_tab.py`. The former `translation.py` adapter,
  raw anchor/preview/stale/session-submit methods in `top_level_transform.py`,
  and line-tool orphan state are retired. Focused Qt coverage passed 11 tests.
- PyO3 no longer exposes `PyTopLevelTranslationAnchorV1`, its raw observation
  method, or `DocumentOperationV1.translate_top_level_roots`. Rust retains its
  internal transform value and snapping anchor; focused staged-binding coverage
  passed 2 tests and the affected Cargo check passed.
- At the earlier 2026-08-23 checkpoint, local `./build.sh` completed and
  `./all_test.sh` recorded 7,492 hygiene tests, 289 binding tests, and 418 Qt
  tests with 1 skipped test; the local CLI and GUI E2Es also passed. This is
  historical checkpoint evidence, not final M0 exit evidence after later
  route migrations.
- Graphify was refreshed after the route work and indexed 19,087 nodes,
  45,770 edges, and 689 communities. These counts support current code-map
  navigation only; the source, tests, and review receipt provide the route
  ownership evidence.
- An earlier independent multi-review audit identified and resolved then-known
  raw-route, raw-anchor facade, DirectBond raw-getter, test-policy, and
  documentation-evidence issues. Later migrations require fresh review.
- Classify any newly discovered visual mutation before it is added to the
  ledger: a visual commit uses document-owned renderer admission; a
  preview-only interaction remains non-mutating; a nonvisual metadata action
  records its rationale here.
- This checkpoint records historical local-build, suite, Graphify, and review
  evidence. Keep the ledger current when a newly discovered visual mutation
  family enters the program, and obtain fresh exit evidence before closing M0.

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

The plan completes when the final M0 checkpoint proves that no public visual
mutation bypasses document-owned renderer admission, obsolete bridges remain
removed, and fresh local-build, full-suite, and independent-review evidence is
green.
