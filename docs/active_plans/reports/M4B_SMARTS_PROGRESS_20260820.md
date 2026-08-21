# M4b SMARTS query progress checkpoint

**Status:** accepted on 2026-08-20. M4b is complete as a bounded SMARTS
parity milestone. The broader Rust replacement and Ferrum parity objective
remains active.

## Objective

M4b adds Rust-authoritative SMARTS querying to Ferrum without reintroducing
OASA or allowing Qt, JSON, or raw native wire data to become chemistry or
renderer authorities. The intended public operation is
`document.molecule.smarts.query.v1`; the intended live UI is a nonmodal Qt
query dock with renderer-authoritative reveal highlights.

## Accepted foundations

- The native adapter is ABI 5. Its SMARTS capability and exact exported symbol
  are admitted together, and no safe public Rust API exposes the raw pointer,
  capability bit, FCQ1, or FQM1 transport.
- The native RDKit matcher implements bounded FCQ1/FQM1 handling, parser
  admission before target work, chirality-aware unique matching, deterministic
  rows, and closed refusal details. Native wire and adversarial fixtures were
  independently accepted.
- `ferrum-chemistry-sys` has been retired. Loader, foreign-buffer, ABI, and
  symbol ownership live privately in `ferrum-chemistry`; compiler-derived
  facade tests protect that boundary.
- The public typed Rust chemistry surface now has `ChemEngine::smarts_match`
  with detail-free SMARTS-native failures. A sealed ABI-5 bundle exercised
  match, no-match, cap truncation, and invalid-query behavior through the real
  adapter.
- Qt has an accepted pre-mutation/reprojection retirement seam. It retires a
  transient renderer result before native mutation and plan replacement,
  fails closed if retirement fails, and uses a source-derived inventory to
  detect unclassified direct session calls.

## Sealed native evidence

The independently reviewed ABI-5 artifact is under
`output_native_wheel/native-IbU0xixX`.

- Bundle adapter: `ferrum-engine-bundle/libferrum_chem.dylib`
- Adapter SHA-256:
  `5ada54eb23853d79db1d035a7cded3274f85b54dd6d2cc1b956ce6bba3b11689`
- Wheel SHA-256:
  `90757880f13e72665e72f54d85a99e4539b07f63e872d466201b5bdaa92208e4`
- Bundle manifest SHA-256:
  `40b31382553a641c805ae9984067c679aa6c883b3de84b0fcb60905b59307d26`
- Target: `aarch64-apple-darwin`

The exact ignored adapter integration test passed against that bundle. See
`/private/tmp/ferrum-smarts-sealed-adapter-rereview-20260820.md`.

## Live-query design decisions

- Both raw-SMARTS and selected-molecule forms traverse all direct molecule
  targets in source order. Selection supplies only the query graph.
- Scope greater than 256 direct targets is refused before lowering,
  allocation, or chemistry. A renderer target that cannot be admitted fails
  the live run atomically; it is never silently omitted.
- Public results use a closed `query_origin` fact, not raw or Rust-generated
  SMARTS. Raw/generated SMARTS, CDML, atom identities, native positions,
  anchors, adapter paths, and receipt keys must not appear in JSON, PyO3,
  errors, logs, `Display`, or `Debug`.
- A live result must be bound to one authenticated document observation and
  published renderer plan. It is invalidated by fence, digest, plan-generation,
  mutation, reprojection, tab switch, close, and stale delivery.
- A result row may be revealed once. Receipt lookup and row reservation are
  atomic; foreign, stale, invalid, or replayed redemption never produces
  geometry.
- Stateless CLI/protocol query returns bounded facts only and carries no GUI
  capability. In-process Python cannot prove caller identity; the enforceable
  boundary is nonconstructible, nonserializable opaque state plus no public
  raw redemption surface.

## Rejected approaches

Do not restore any of these designs:

- A `#[doc(hidden)] pub` document-to-render identity transport. It is still
  public to external Rust consumers.
- A source-order or independently recomputed projection join from native atom
  positions to renderer anchors.
- Public renderer/document-render prepared-target, raw position, anchor, or
  graph accessors.
- Predictable integer receipts, whole-run consumption for a single row, or
  public PyO3 receipt/paint DTO classes.
- A cyclic `ferrum-document -> ferrum-render` dependency or moving generic
  interaction state into the pure renderer.

## Final acceptance

The historical in-progress statements below this heading are superseded by
the following accepted evidence.

- The Rust backend, private live bridge, selected-token authority, named raw
  and selected CLI operation, response-size admission, and Qt lifecycle seam
  were independently accepted. See
  `/private/tmp/ferrum-smarts-m4b-final-acceptance-review.md`,
  `/private/tmp/ferrum-smarts-backend-completion-review.md`,
  `/private/tmp/ferrum-smarts-response-size-final-review.md`, and
  `/private/tmp/ferrum-smarts-stateless-selected-protocol-review.md`.
- The final packaged GUI contract passed from isolated CPython 3.12 imports,
  with an isolated engine home and verified 18-member bundle closure. It used
  native wheel SHA-256
  `4d436651d7ae6cc101794815f13dd5c72a0ed894d4d0431957c0e9190e023b31`
  and Qt wheel SHA-256
  `fc1980481e20ef25d936ee9a18253a6c458108d847576e63b9d7d5a002ed5b76`.
- That GUI run passed menu/dock placement, raw and direct-root selected
  queries, cancellation-before-tool handoff, Clear/rerun, keyboard Escape,
  multi-row/replay behavior, mutation/reprojection invalidation, Save As,
  Rust and asynchronous GUI reopen, and two-tab retirement/unbinding. See
  `/private/tmp/ferrum-smarts-qt-dock-final-tabswitch-e2e-20260820.md`.

## Next parity work

M4b acceptance is not a claim of complete OASA replacement, complete
BKChem/Ferrum parity, release readiness, a clean worktree, or process
quiescence. The recommended next parity milestone is M2a CML/CML2 interchange:
Rust-owned parsing, serialization, CLI operations, and usable Qt import/export
flows with the same explicit ownership and artifact evidence standards.

## Primary reports

- `/private/tmp/ferrum-smarts-sealed-adapter-rereview-20260820.md`
- `/private/tmp/ferrum-smarts-qt-premutation-inventory-review-20260820.md`
- `/private/tmp/ferrum-smarts-query-api-backend-v3-review.md`
- `/private/tmp/ferrum-smarts-query-v3-remediation-design.md`
- `/private/tmp/ferrum-smarts-query-api-backend-v4.md`
- `/private/tmp/ferrum-smarts-query-api-backend-v4-review.md`
- `/private/tmp/ferrum-smarts-m4b-final-acceptance-review.md`
- `/private/tmp/ferrum-smarts-qt-dock-final-tabswitch-e2e-20260820.md`

## Worktree caution

This checkpoint does not assert a clean worktree, no live processes, a
packaged release, or completion of the broader Ferrum parity objective.
