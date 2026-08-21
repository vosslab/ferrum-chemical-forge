# Plan: Attach cyclohexane to one atom

## Context

Ferrum already inserts a detached, Rust-owned saturated C6 ring, but clicking an occupied atom
refuses without mutation. This leaves a visible skeletal-drawing gap. The next parity slice adds
one explicit `Attach Cyclohexane Ring` action that attaches one ordinary C6 ring to one existing
atom, while preserving the existing detached Cyclohexane action and the Rust-first document,
preview, history, and Qt ownership boundaries.

The historical OASA catalog identifies cyclohexane as `C1CCCCC1`; it is evidence for the target
chemistry, not an implementation dependency. The current Ferrum detached-ring and direct-bond
gesture seams provide the reusable local evidence.

## Objectives

- Deliver one Qt gesture that attaches an ordinary saturated C6 ring at one eligible existing atom.
- Preserve atomic document mutation, revision fencing, undo/redo, save/reopen, and Rust-issued
  preview geometry.
- Complete every plan gate autonomously with manager/subagent work and deterministic local proof.

## Design philosophy

This plan applies **Keep designs simple**, **Use the scientific method**, and **Design for
adaptability** from [docs/REPO_STYLE.md](../../REPO_STYLE.md): prove the atom-sharing topology with
two small experiments, then add one closed C6 capability rather than a fragment, ring, or template
framework. The rejected alternative is a generic attachment catalog that would introduce policy and
public seams before a second concrete consumer exists.

## Scope

- Define one atom-anchored saturated C6 topology in Rust.
- Retain the existing detached Cyclohexane action and add one explicit attach action for atom hits.
- Add a private live PyO3 receipt and copied Rust preview/result facts.
- Cover semantic behavior with inline/offline tests and one automated root E2E Qt workflow.
- Record the implementation and closure evidence in the active plan and changelog.

## Non-goals

- Implement fused rings or shared-edge/bond attachment.
- Generalize ring, fragment, template, catalog, or attachment abstractions.
- Add CML import expansion, CLI attachment syntax, or a CML route.
- Add checked-in fixtures, a fixture manifest, network checks, sleeps, or regular-test subprocesses.
- Require a person, live service, manual visual sign-off, or interactive debugging step.

## Current state summary

- `ferrum-document` owns detached C3-C8 geometry, candidate construction, IDs, history, and stale
  fencing through `regular_ring_insertion_v1.rs` and `session/molecule_creation.rs`.
- `ferrum-document` already owns revision/digest-fenced private live direct-bond gestures.
- The Qt Cyclohexane action owns pointer capture and paint-only preview; its occupied-atom behavior
  currently refuses and preserves the document.
- The architecture readiness review recommends a C6-only atom-sharing ring: the existing atom is
  one ring vertex, five carbon atoms and six normal single bonds are created, and no ring metadata
  enters CDML.

## Settled V1 chemistry and interaction contract

The capability-survey description of six new atoms, five internal bonds, and one external anchor
bond is superseded for this slice. Ferrum V1 is shared-anchor C6 annulation: the resolved existing
anchor is one ring vertex; the candidate creates exactly five ordinary neutral carbon atoms and
six `n1` bonds, `anchor-r1-r2-r3-r4-r5-anchor`, in the anchor's existing molecule. The anchor
therefore gains exactly two `n1` incidences and no existing record is rewritten. This is neither
an external cyclohexyl substituent nor a fused/shared-edge ring.

An anchor is eligible only at the fenced revision when it is a finite planar direct atom in its
molecule, has element spelling `C`, absent or zero formal charge, no authored nondefault valence
or multiplicity override, and only same-molecule normal single `n1` incident bonds. With absent
explicit hydrogen treated as zero, `incident_n1_count + explicit_hydrogen_count` must be at most
two. The implementation refuses every other target as typed `IneligibleAnchor` before identity,
candidate, history, selection, or preview allocation; it never falls back to an external or
detached topology.

The existing detached action remains its current empty-page insertion action. The new, separately
named checkable `Attach Cyclohexane Ring` action has its own keyboard-accessible QAction and status
text, is owned directly by the authoritative line-tool event path, and makes an eligible atom hit.
Shared chrome may derive its status label from that live intent, but C6 never enters the generic
mode-manager vocabulary. Empty-page or ineligible hits make a clear nonmutating
refusal/cancellation. This preserves the detached action's established meaning and avoids an
undocumented context-sensitive mode change; the readiness review supplied the evidence for this
choice, and no contrary behavior evidence exists.

## Architecture boundaries and ownership

Rust document code owns target resolution, snapped pose, finite geometry, valence/topology
admission, generated IDs, candidate history transition, commit, and authoritative selection/result.
The API binding owns opaque receipt identity and only returns copied preview and outcome facts. Qt
owns QAction state, pointer capture, a temporary graphics overlay, status text, cancellation, and
installation of Rust observations. Qt neither calculates topology nor writes CDML or identifiers.

### Mapping (milestones / workstreams -> components / patches)

| Milestone / Workstream | Component | Review boundary |
| --- | --- | --- |
| M1 / WS-A | `ferrum-document` C6 topology experiments and prepared gesture | Rust topology and transaction reviewer |
| M2 / WS-B | `ferrum-api` cooperative internal PyO3 receipt | API boundary reviewer |
| M3 / WS-C | Qt attach action and tab mixin | Qt interaction reviewer |
| M4 / WS-D | Existing focused tests and one root E2E | Test-plan and independent acceptance reviewer |

## Milestone plan

| M | Title | Summary | Goal |
| --- | --- | --- | --- |
| M1 | Implement and probe topology | Add the smallest pure core, then measure its settled contract | A safe atom-sharing C6 geometry/admission kernel |
| M2 | Cross the cooperative internal bridge | Bind one opaque Python receipt over the concrete document seam | Rust-owned C6 authority with a narrow trusted Qt bridge |
| M3 | Deliver the gesture | Add the attach action and lifecycle | Usable attach, cancel, and detached behavior |
| M4 | Prove and close | Run focused checks, one root E2E, review, and records | Autonomous acceptance evidence |

### Milestone: Settle and implement topology

- Depends on: none.
- Deliverables: one private C6 attached-ring pose/admission/candidate seam beside the existing
  document gesture seams; inline probe matrix and experiment receipt in `/private/tmp`.
- Workstreams: WS-A.
- Entry criteria: current detached-ring and direct-bond seams remain available.
- Exit criteria: the private core's two probe results agree with the fixed shared-anchor topology
  and closed anchor-admission invariants. It has no session, identity, history, receipt, or
  cancellation responsibility; WP-A2 owns those stateful obligations.
- Parallel-plan ready: no. Private core and its probes deliberately share one authority boundary.

### Milestone: Cross the cooperative internal bridge

- Depends on: WP-A2, because the binding projects the settled Rust receipt.
- Deliverables: private PyO3 begin/preview/commit/cancel bridge with copied finite vertices and
  typed refusals.
- Workstreams: WS-B.
- Entry criteria: reviewed document receipt contract.
- Exit criteria: the concrete C6 lifecycle remains a public typed internal Rust transaction seam
  required by the `ferrum-api -> ferrum-document` dependency direction. Its opaque private fields
  and document-owned session, revision, digest, anchor, and one-use fencing enforce its transaction
  invariants. Python receives no document IDs, CDML, raw candidate, topology, or serialization
  surface. The trusted Qt controller is the sole intended repository client of the four
  underscore-prefixed bridge calls. This is a cooperative same-process application boundary, not
  isolation from arbitrary Python holding a document session.
- Parallel-plan ready: no. It depends on the settled document capability.

### Milestone: Deliver the gesture

- Depends on: WP-B1, because Qt calls only the private binding.
- Deliverables: a separate atom-hit attach action, unchanged detached Cyclohexane behavior,
  paint-only preview, cancellation, and accessible status wording.
- Workstreams: WS-C.
- Entry criteria: bound opaque receipt methods are accepted.
- Exit criteria: release attaches, Escape/tab change/refusal retires the overlay, and no Qt-side
  mutable topology exists.
- Parallel-plan ready: no. It is a thin integration over the approved binding.

### Milestone: Prove and close

- Depends on: WP-C1, because acceptance observes the actual gesture.
- Deliverables: focused offline tests, one root E2E script, independent review, and a private
  acceptance receipt.
- Workstreams: WS-D.
- Entry criteria: Qt gesture is implemented and focused semantic tests pass.
- Exit criteria: the sole root E2E and all targeted checks pass; independent review accepts; no
  human confirmation is required.
- Parallel-plan ready: no. Evidence must reflect the integrated implementation.

## Workstream breakdown

### Workstream: WS-A document topology

- Goal: select and implement one durable C6 atom-sharing candidate.
- Owner: expert Rust coder.
- Work packages: WP-A1, WP-A2.
- Needs: current regular-ring, direct-bond, session, and document candidate seams.
- Provides: WP-A1's pure geometry/admission facts and WP-A2's concrete pending Rust capability to
  WS-B.
- Review boundary, when modifying the repository: independent Rust reviewer after WP-A2.

### Workstream: WS-B cooperative internal API bridge

- Goal: expose no more than opaque live receipt operations and copied preview/result facts to the
  trusted Qt controller, while retaining all C6 authority in Rust.
- Owner: Rust API coder.
- Work packages: WP-B1.
- Needs: WP-A2 receipt contract.
- Provides: private tab-facing bridge to WS-C.
- Review boundary, when modifying the repository: independent API privacy reviewer.

### Workstream: WS-C Qt gesture

- Goal: add an explicit attach action while retaining the detached action's current behavior.
- Owner: PySide6 coder.
- Work packages: WP-C1.
- Needs: WP-B1 private bridge and current action lifecycle.
- Provides: actual user workflow to WS-D.
- Review boundary, when modifying the repository: independent Qt interaction reviewer.

### Workstream: WS-D proof and closure

- Goal: create the smallest automated evidence set for the shipped behavior.
- Owner: tester.
- Work packages: WP-D1, WP-D2.
- Needs: WP-C1 behavior.
- Provides: acceptance receipt and closure inputs to the manager.
- Review boundary, when modifying the repository: independent acceptance reviewer.

## Work packages

### Work package: WP-A1 implement the private topology hypothesis and probe it

- Owner: expert Rust coder.
- Touch points: private document-core attached-C6 seam beside the direct-bond gesture code; its
  existing inline test module; `/private/tmp/ferrum-cyclohexane-attachment-topology.md`.
- Depends on: none.
- Acceptance criteria: first implement only the smallest crate-private pure hypothesis needed for
  the settled attached-C6 path: deterministic pose from supplied anchor/release facts, typed pose
  and `IneligibleAnchor` admission/refusal, and an immutable C6 geometry candidate primitive. It
  must not resolve a document target, allocate IDs, construct CDML, fence a session, issue a
  receipt, mutate history, or model cancellation. It must not expose a public receipt, API bridge,
  Qt contract, catalog, template,
  attachment framework, or fixture/harness. Then run exactly two deterministic in-process inline
  probes against that production-owned seam. The pose probe uses anchor `(0, 0, 0)`, side length
  `40`, and six release directions at 60-degree intervals; it proves vertex zero is the anchor,
  all points are finite and distinct, all six closed-cycle edges equal authored length within one
  documented tolerance, the center lies one side length toward release, and winding is stable in
  document y-down coordinates. Coincident or nonfinite pointers return a typed pose refusal. The
  admission probe supplies closed anchor facts and proves acceptance for neutral planar carbon with
  zero, one, or two `n1` occupancy; typed `IneligibleAnchor` refusal for three `n1`, `n2`, excess
  explicit hydrogen, charge, non-carbon, or nonplanar facts. It proves no document or session
  behavior; deferred identity allocation, stale/cancel/foreign handling, and real-document
  nonmutation belong exclusively to WP-A2.
- Evidence or review, when useful: record the hypothesis, exact probe measurements, comparison to
  the fixed topology/valence invariants, and outcome in `/private/tmp`. The probes may falsify the
  private implementation, not reopen the selected topology.
- Obvious follow-ons: if a probe falsifies an invariant, revise only the private C6 pose/admission/
  candidate core and rerun the same matrix. Do not add a fallback, special case, generic layer, or
  public bridge until it passes.

### Work package: WP-A2 implement atomic atom-sharing C6 capability

- Owner: expert Rust coder.
- Touch points: `packages/ferrum-rust/crates/document/src/`; existing inline Rust test module.
- Depends on: WP-A1, because its production-owned probes validate the pose/admission/candidate
  contract before the receipt is completed.
- Acceptance criteria: implement exactly one concrete `PendingAttachedCyclohexaneV1` capability,
  not a generic attachment/template/ring framework. It resolves a direct target atom in its owning
  molecule and consumes WP-A1's immutable geometry/admission candidate. One selected/validated
  existing atom becomes one ring vertex; the candidate adds five ordinary carbon atoms and six
  `n1` bonds in that same molecule; preview and commit use identical Rust geometry.

  The selected deferred-ID discipline is the existing molecule-batch pattern applied only to this
  C6 operation: after revision/digest and target admission, copy the generated-ID sequences,
  reserve exactly five atom IDs and six bond IDs in the copy, construct and validate the complete
  candidate, and retain the tentative sequences with the pending capability. Prepare issues no
  token and changes no session state. Commit rechecks session origin and the stored revision/digest
  before issuing and consuming its token, then installs the copied sequences and appends one
  revision/history transition. Retire drops the candidate and tentative sequences without a fence
  check, allowing stale previews to be cancelled safely. Successful commit consumes the pending
  capability. Sequential per-atom/per-bond commits, preallocation in live session state, detached
  molecule writers, and a generic prepared-attachment abstraction are explicitly rejected.

  Inline literal-CDML proof owned by this package snapshots CDML, revision, digest, history,
  selection, and next generated IDs. It proves accepted attachment preserves prior records and
  molecule count while adding five C and six `n1` bonds, raising anchor incidence by two, and
  making one transition; it proves unknown/non-direct/ineligible targets, stale revision/digest,
  invalid release, and near-exhausted atom or bond allocation leave every snapshot fact unchanged;
  it proves cancel, foreign commit/retire, stale commit, and replay are nonmutating and leave an
  owner receipt committable where applicable.
- Evidence or review, when useful: run only focused document crate checks and independent review.
- Obvious follow-ons: expose the exact receipt through WP-B1; defer every other ring shape.

### Work package: WP-B1 bind the cooperative internal receipt

- Owner: Rust API coder.
- Touch points: `packages/ferrum-rust/crates/api/src/python_binding/` and existing API tests.
- Depends on: WP-A2, because the binding may not invent an alternate candidate or pose.
- Acceptance criteria: retain exactly four undocumented underscore-prefixed PyO3 calls on the
  existing trusted `PyDocumentSession` controller: begin, preview, commit, and cancel. This is an
  application-internal same-process bridge, not a claim that arbitrary Python holding that session
  cannot invoke a callable method. Caller inputs are durable target ID, current fence, raw pointer
  facts, and approved snap policy; outputs are copied finite preview geometry and typed
  identity-free outcome facts. The Rust-issued pending value is nonconstructible, opaque,
  methodless, uncloneable, unsendable, and has no pickle/reduction path. Do not register
  `PyPendingAttachedCyclohexaneV1` with the extension module: specifically omit
  `module.add_class::<PyPendingAttachedCyclohexaneV1>()`, while proving the four underscore
  methods can return and consume the Rust-issued opaque object without registration. It exposes no document
  IDs, CDML, raw candidate, topology, fence, or serialization fields. Rust alone retains candidate
  identity, target admission, C6 topology, session origin, revision/digest/anchor fencing, atomic
  commit, and one-use retirement. Foreign session, stale revision, stale digest, replayed,
  retired, unknown/ineligible anchor, and invalid-release uses refuse before mutation.

  Do not add a generic attachment API, Python constructor, module-level
  function, attachment service/protocol, selector, CDML result, generated-ID output, or topology
  serialization surface. The concrete C6 document lifecycle is a public typed internal Rust
  transaction seam, not an access-control boundary: the current `ferrum-api -> ferrum-document`
  dependency direction requires it for the API to retain and redeem the pending receipt. Its fields
  stay opaque and its lifecycle keeps document-owned admission, deferred identities, fences, and
  atomic commit. Remove `#[doc(hidden)]` from this supported public seam or document its
  internal-Rust status in normal rustdoc; hiding it would be misleading. Do not introduce a
  compiled shim, caller-class check, stack inspection, capability wrapper, or process isolation
  merely to claim stronger Python authority. Do not add a generic prepared-edit framework: it has
  no second consumer. Defer a lower-layer Python session-host migration until a second independently
  justified private document operation needs retained cross-language state and a cycle-free
  prototype shows that migration reduces coupling.
- Evidence or review, when useful: use the existing inline API binding suite and embedded Python
  interpreter to prove the extension module has no `PendingAttachedCyclohexaneV1` attribute, no
  module-level attachment constructor, function, or service, and no CDML, topology, identifier,
  pickle, or reduction path; a minted pending object rejects construction and look-alike extraction;
  and the
  actual bound calls enforce every stated fence/refusal invariant without state mutation. These
  tests prove Python opacity and document transaction invariants, not that arbitrary same-process
  Python is barred from underscore calls. A Rust API contract check proves the documented public
  C6 seam remains concrete, has opaque private state, and has not grown an additional generic
  attachment capability. No fixture, subprocess, or second root E2E is added.
- Obvious follow-ons: WP-C1 consumes the bridge through one private tab controller without Qt
  topology math or a claim of access control.

### Work package: WP-C1 add the explicit attach action

- Owner: PySide6 coder.
- Touch points: existing `line_tools.py`, ring helper/tab mixin, and existing regular-ring test
  module in `packages/ferrum-chem-qt.app/`.
- Depends on: WP-B1, because all attachment work passes through the opaque receipt.
- Acceptance criteria: a separately named, checkable `Attach Cyclohexane Ring` QAction with a
  keyboard-accessible activation creates the direct C6 line intent while the generic mode manager
  remains inactive and the existing
  line-tool event filter retains sole pointer ownership; the existing Cyclohexane QAction retains its
  detached empty-page behavior unchanged. In attach mode an eligible atom hit starts the receipt,
  move paints only copied Rust preview facts, and release commits once. Escape, tab transition,
  document change, empty-page hit, ineligible anchor, and receipt refusal retire preview and
  preserve state. Status text explicitly identifies attach mode, unavailable target, and retry.
- Evidence or review, when useful: extend the existing offscreen Qt behavior test with inline CDML;
  prove tab change, dispose, Escape, refusal, and document change retire the local pending value.
  Add one literal, offline, source-local repository-discipline oracle for exactly
  `_begin_attach_cyclohexane_v1`, `_preview_attach_cyclohexane_v1`,
  `_commit_attach_cyclohexane_v1`, and `_cancel_attach_cyclohexane_v1`: it scans only the Qt
  production and Qt test source trees, excluding Rust binding definitions and tests. Every Qt
  production occurrence must be in the one named private tab-controller module, and the only
  permitted Qt test occurrence is the dedicated Qt behavior test module. Update this same oracle
  when that controller is added.
  It proves repository usage discipline, not access control. No generic scanner/framework, new
  fixture directory, or test harness.
- Obvious follow-ons: WP-D1 proves integrated package behavior.

### Work package: WP-D1 run focused offline proof

- Owner: tester.
- Touch points: existing inline Rust tests, existing API binding tests, and existing Qt regular-ring
  test module.
- Depends on: WP-C1, because proofs target the finished gesture.
- Acceptance criteria: compact semantic tests prove C6 graph/pose, atom target validation,
  atomicity/history/undo-reopen, stale and cancellation nonmutation, actual-bound-bridge opacity
  and receipt invariants, and offscreen action preview/commit/cancel/refusal. The API proof covers
  nonconstructibility, no serialization/reduction, no public Python attachment surface beyond the
  cooperative bridge, and session/revision/digest/anchor/one-use fencing without claiming arbitrary
  same-process Python cannot call a cooperative bridge. The Rust API proof permits the concrete
  typed document seam, verifies its opaque-state and transaction invariants, and rejects an added
  generic attachment capability. The Qt proof covers the private controller's lifecycle and
  repository call-site discipline. All input documents are inline and all tests are offline,
  deterministic, and free of sleep/network/subprocess dependence.
- Evidence or review, when useful: targeted command results go only in the private acceptance report.
- Obvious follow-ons: WP-D2 runs the one whole-workflow proof after focused checks pass.

### Work package: WP-D2 run one root E2E and close

- Owner: tester.
- Touch points: one `tests/e2e/e2e_cyclohexane_attachment.py`, docs, and `/private/tmp` receipt.
- Depends on: WP-D1, to avoid using an E2E loop for unit diagnosis.
- Acceptance criteria: the sole root E2E is an offscreen local-Qt harness using an inline CDML
  document and a fresh exact native `ferrum_chem` wheel. Its manager-owned prerequisite invokes
  `packages/ferrum-rust/tools/build_native_wheel.py build --output-root <fresh-root>
  --sealed-input-root <validated-root>` with an explicit fresh output root and a manager-selected
  manifest-validated sealed local input root; the builder JSON supplies the exact wheel path, never
  timestamp discovery or a network download. The E2E creates a temporary
  `--system-site-packages` venv, installs that exact wheel with `pip install --no-deps`, starts an
  isolated offscreen child with only the current-checkout Qt package root on `sys.path`, and rejects
  a checkout `ferrum_chem` import. It proves the Edit-menu hierarchy exposes exactly one visible,
  enabled, checkable, initially unchecked `Attach Cyclohexane Ring` QAction with the window's
  shared action and exclusive-group ownership. It opens and closes the owning visible `QMenu` with
  the Qt event queue drained, then calls the canonical shared `QAction.trigger()` only after popup
  teardown and proves the action is checked, the direct C6 line intent exists, and generic mode
  state remains `None` before
  using QTest to perform one attach, save/reopen,
  undo/redo, cancel, and stale/ineligible refusal without a network or person. It emits sorted JSON
  and writes a compact receipt only under `/private/tmp`, not fixtures or `devel/`; it is executed
  directly with `source source_me.sh && python3`, never normal pytest. Offscreen QPA cannot reliably
  hit-test a native popup action row, so this local E2E proves real menu ownership and uses the
  QAction's platform-independent activation contract; desktop popup pointer behavior remains a
  separate platform-level concern.
- Failure diagnostics: the manager pipeline reports selected sealed-input candidates and manifest
  errors; build failures retain stderr and output root; bootstrap failures report interpreter,
  PySide6 version, wheel path/digest, and `ferrum_chem.__file__`; interaction failures report Qt
  platform/version, deterministic probe state, and child stderr. The responsible owner repairs the
  owning layer and reruns its focused proof, not a retry or E2E fallback.
- Evidence or review, when useful: independent reviewer reads the diff, plan, focused results, E2E
  result, and receipt before the manager marks M4 closed.
- Obvious follow-ons: update plan status/changelog and archive only after acceptance.

## Acceptance criteria and gates

- Experiment gate: WP-A1 reports both pure results for the already-settled shared-anchor C6
  contract before WP-A2 begins; a failure revises only its pose or admission implementation and
  repeats. WP-A2 independently proves all document/session/identity behavior using the concrete
  pending capability.
- Per-patch gate: each owner runs only the package's focused offline checks and records command,
  result, changed files, and residual risk in its `/private/tmp` report.
- Integration gate: WP-D1 passes before the sole root E2E is created or run.
- Independent review gate: separate fresh reviewers accept WP-A2, WP-B1, WP-C1, and the combined
  M4 evidence; a finding returns to a new implementation owner, followed by re-review.
- Closure gate: manager verifies every package acceptance criterion and receipt without a human,
  network, live service, or manual UI gate.

## Test and verification strategy

- Keep Rust semantic cases inline with the ownership module and use compact literal documents.
- Keep PySide6 tests in their existing module, offscreen, deterministic, and behavior-focused.
- Add no permanent fixtures, fixture manifest, reusable debug runtime, regular subprocess test, or
  network test.
- Use exactly one root whole-system E2E after the Qt workflow exists; keep it under `tests/e2e/`,
  execute it directly with `source source_me.sh && python3`, and use inline synthetic transitions.
- Write generated experiment, review, and E2E receipts only under `/private/tmp`.

## Migration and compatibility policy

The change is pre-production and Rust-first. It adds no OASA or BKChem runtime fallback, branding,
compatibility adapter, or persistent ring/template metadata. Ordinary CDML remains the persisted
format: after save/reopen the result is an ordinary connected carbon cycle plus its attachment bond.

## Risk register

| Risk | Impact | Trigger | Owner | Mitigation |
| --- | --- | --- | --- | --- |
| Pose is confusing or overlaps anchor geometry | Incorrect visible chemistry | WP-A1 geometry results fail | expert Rust coder | Revise the single C6 pose rule before implementation |
| Candidate leaks partial state | Corrupt history or IDs | Failed refusal test | expert Rust coder | Build/validate candidate before one prepared commit |
| Binding widens document authority | Fragile cross-layer API | Generic attachment capability appears | API reviewer | Keep the concrete typed seam, opaque receipt, and one Qt controller |
| Qt recreates topology | Divergent preview/commit | Qt geometry differs from Rust preview | Qt reviewer | Paint copied Rust vertices only |
| Scope creeps toward templates/fusion | Delayed durable delivery | Extra ring policies appear | manager | Reject outside-scope work and record deferred item |
| E2E becomes a diagnostic loop | Slow brittle proof | Focused tests fail or E2E needs retries | tester | Repair owning semantic test first; retain one final E2E |

## Rollout and release checklist

- [x] Run WP-A1 and record the selected topology.
- [x] Complete, review, and re-review document, API, and Qt packages in dependency order.
- [x] Run focused semantic checks and exactly one root E2E.
- [x] Capture the autonomous acceptance report in `/private/tmp`.
- [x] Update the active plan status and [docs/CHANGELOG.md](../../CHANGELOG.md).
- [ ] Move the plan to archive only after the manager closes this accepted slice.

## Documentation close-out requirements

- Active plan / progress tracker: this plan receives package status and links to private receipts.
- docs/CHANGELOG.md entry: record the closed C6 attachment behavior and explicitly deferred family.
- Archive / closure notes: record focused and E2E commands, results, review decision, and any
  remaining parity work before moving the plan.

## Completion record

The bounded shared-anchor C6 attachment slice is accepted on 2026-08-21. Final autonomous
evidence is `/private/tmp/ferrum-cyclohexane-final-e2e-receipt.json`; it used the exact native
wheel emitted by bare `./build.sh native`, with SHA-256
`4a438e34473685700d76ae11d8489b4703bfb9c63b0549c39e278ba6ca221ddf`. The managed build-cache
path replaces the earlier explicit sealed-root prerequisite, so normal builds require no
operator-selected input root.

The final local offscreen Qt workflow covered unarmed startup, Edit-menu action identity and
post-popup canonical action activation, typed atom picking, Rust-owned preview and one commit,
save/reopen, undo/redo, Escape, refusal, cancellation, and tab lifecycle. Independent acceptance
is recorded in `/private/tmp/ferrum-cyclohexane-final-acceptance-rereview.md`.

This closes only the atom-attached cyclohexane capability. Broader OASA replacement, BKChem UI
parity, additional ring families, fusion, templates, and compatibility work remain deferred to
separately scoped milestones.

## Patch plan and reporting format

- Patch 1: WP-A1 pure experiment receipt and WP-A2 concrete deferred-ID document capability.
- Patch 2: WP-B1 private API bridge.
- Patch 3: WP-C1 Qt action integration.
- Patch 4: WP-D1 focused proof, WP-D2 one E2E, independent acceptance, and documentation closure.
- Every owner writes one full report under a manager-assigned collision-safe `/private/tmp` path
  with assumptions, decisions, changed files, commands/results, and residual risk. The chat handoff
  contains only status, path, summary, validation status, and blockers.

## Deferred follow-up

- Consider a separately planned selected-bond fusion slice only after this atom-attachment receipt
  has accepted implementation evidence.
- Continue broader OASA and BKChem parity through separately scoped, capability-led milestones.
