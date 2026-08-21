# M4b SMARTS dock checkpoint

**Status:** accepted on 2026-08-20. The modeless SMARTS dock is complete for
M4b. This is not a broader Ferrum parity or release-completion claim.

## Original-goal connection

M4b is one Rust-first Ferrum parity slice. It adds OASA-style SMARTS matching
through the Rust backend, named CLI operation, and usable Qt application. It
does not redefine the broader goal of replacing OASA, removing BKChem/OASA
runtime branding/dependencies, and completing Ferrum parity.

## Accepted M4b backend and delivery foundations

- The ABI-5 sealed native adapter, Rust typed SMARTS matcher, loader ownership,
  and native wire boundary have independent acceptance evidence.
- The backend uses an API-owned published render-plan bridge. Native match
  positions join renderer anchors through the same accepted observation; Qt
  receives only identity-free paint facts.
- Live receipts are session-bound opaque Python objects with per-row one-use
  reservation. Mutation, reprojection, publication, tab lifecycle, and stale
  paths retire them before another visual can be issued.
- `document.molecule.smarts.query.v1` is registered through the named CLI and
  returns bounded, redacted raw- or selected-origin facts. It does not return
  raw/generated SMARTS, CDML, atom identifiers, native positions, anchors, or
  GUI receipts.
- The 1 MiB response admission is measured on the canonical JSON envelope.
  Rust proves the actual 1 MiB boundary. A nonshipping harness wheel exercises
  the public PyO3 delivery branch under a lower injected limit because the
  intentionally bounded normal response cannot reach 1 MiB.
- Fresh isolated artifact evidence is recorded in the reports below. The
  latest response-size shipping artifact is
  `output_native_wheel/native-cExQGGcu`; earlier published-plan Qt evidence
  used `output_native_wheel/native-dfrNqEeu` with a separately rebuilt current
  Qt wheel. Do not claim one combined artifact unless it is rebuilt and run.

## Final dock acceptance

The historical implementation, prerequisite, and stop-condition sections are
superseded. The accepted dock now provides the intended modeless right-side
workflow through the Chemistry menu, raw SMARTS and explicit direct-root canvas
selection, typed recovery, accessible keyboard operation, and receipt-only
result activation.

- The final fresh packaged E2E passed an isolated offscreen GUI run with
  `ferrum engine status` reporting `ready`, no source fallback, and terminal
  status `ok`.
- It covers dock placement, capture persistence, Add Atom ownership handoff,
  invalid raw recovery, direct-root selected query, raw/selected Clear-rerun,
  input/tree/button Escape, multi-row replacement/replay refusal, mutation and
  reprojection cleanup, Save As/Rust/GUI reopen, and bidirectional tab-switch
  retirement and state unbinding.
- The isolated artifact receipt is native ABI-5 wheel SHA-256
  `4d436651d7ae6cc101794815f13dd5c72a0ed894d4d0431957c0e9190e023b31` and
  Qt wheel SHA-256
  `fc1980481e20ef25d936ee9a18253a6c458108d847576e63b9d7d5a002ed5b76`.

The acceptance record is
`/private/tmp/ferrum-smarts-qt-dock-final-tabswitch-e2e-20260820.md`.
Backend/lifecycle acceptance preceding dock implementation is recorded in
`/private/tmp/ferrum-smarts-m4b-final-acceptance-review.md`.

## Next parity work

M4b closes one Rust-first SMARTS parity slice. It does not complete the
replacement of OASA, removal of BKChem/OASA branding and runtime dependencies,
or full Ferrum feature parity. Return to the broader parity ledger with M2a
CML/CML2 interchange as the recommended next milestone.

## Key reports

- `/private/tmp/ferrum-smarts-backend-completion-review.md`
- `/private/tmp/ferrum-smarts-response-size-final-review.md`
- `/private/tmp/ferrum-smarts-qt-postpublication-retirement-completion-review.md`
- `/private/tmp/ferrum-smarts-qt-live-query-packaged-rerun.md`
- `/private/tmp/ferrum-smarts-qt-observe-render-packaged-assertion.md`
- `/private/tmp/ferrum-smarts-qt-dock-hci-design.md`
- `/private/tmp/ferrum-smarts-qt-dock-implementation.md`
- `/private/tmp/ferrum-smarts-live-query-qt-seam-review.md`
- `/private/tmp/ferrum-smarts-qt-multirow-overlay-lifecycle.md`
- `/private/tmp/ferrum-smarts-m4b-final-acceptance-review.md`
- `/private/tmp/ferrum-smarts-qt-dock-final-tabswitch-e2e-20260820.md`

## Worktree caution

This document does not assert a clean worktree, process quiescence, release
readiness, or completion of the broader Ferrum objective.
