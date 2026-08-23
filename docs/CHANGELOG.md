# Changelog

Earlier history is in [CHANGELOG-2026-08d.md](CHANGELOG-2026-08d.md).

## 2026-08-22

### Fixes and Maintenance

- Corrected the atom-chemistry-facts contract to omit an oxidation result that is not yet exposed
  through a public protocol, local CLI, PyO3 binding, or Qt route.

- Made native SMARTS receipt retirement authoritative before Qt overlay cleanup. A failed
  native retirement now retains every local query owner; a failed visual cleanup fails closed
  for refresh rather than partially clearing the transaction.

- Narrowed SMARTS transaction recovery to expected `RuntimeError` failures from native
  retirement calls and Qt visual detachment. Contract drift and programming failures now
  surface instead of being converted into ordinary recovery state.

- Repaired SMARTS reveal recovery so a failed native receipt retirement blocks new queries while
  retaining local ownership and offers an explicit accessible retry through `Clear results`.

- Moved durable presentation-ID allocation to `DocumentSession` transactional
  `PendingCreatePresentationV1` reservations. Terminal Electron, Retro, and
  Curved Normal arrows, Curved Equilibrium arrows, incremental
  Polyline/Polygon paths, and presentation vectors install their ID sequence
  only with the successful document mutation; previews and refused candidates
  receive no durable ID. Renderer and route-local durable counters are retired.
- Replaced three package-local E2E scripts that created current-directory
  artifacts with registered, lease-backed root E2Es for vector authoring and
  template-catalog authoring. The permanent coverage exercises public
  workflows only.
- Preserved explicit terminal SMARTS user outcomes during control refresh. A
  successful clear remains visibly confirmed after the eligibility state is
  recalculated.
- Hardened the shared E2E workspace parent trust boundary before sweep-lock
  acquisition. The helper now accepts only a direct, current-user directory
  with no group or other permissions, so recovery never acts through a
  pre-existing shared or substituted parent.
- Made the Arrow E2E workspace lease return its verified physical directory.
  This keeps its macOS temporary path compatible with Rust publication's
  intentional no-symlink parent policy. Qt now presents a typed rejected
  destination as a save that did not start, rather than one that may have
  completed.
- Established the canonical `ferrum-document` authoring-capability authority
  for every supported live opaque receipt. `DocumentSession` owns one
  allocation-identity `AuthoringCapabilityIssuerV1`; text placement, straight
  presentation, catalog V1/V2 authority, presentation vectors, DirectBond,
  terminal/equilibrium arrows, presentation paths, and reaction
  create/lifecycle/translation now carry its nonserializable
  `AuthoringCapabilityV1`. The `Available -> Claimed -> Consumed` RAII lifecycle
  preserves typed foreign, replay, owner-retry, rollback, and final-handle
  release semantics while keeping catalog preview leases and durable IDs
  separate. Renderer-private gesture capability state and raw bridge-origin
  access are retired. Programmatic DirectBond now receives the same capability
  only after its candidate materializes; its retired independent origin,
  gesture-capability, and counter authorities are removed.
- Moved catalog durable-ID authority fully into `DocumentSession`. System
  catalog recipes lower to `MoleculeInsertionV1`, and the document's opaque
  pending candidate allocates molecule, atom, and bond IDs. Its tentative
  generated-ID sequences install only after a successful transaction, so
  discarded or refused preparation cannot advance durable IDs.
- Preserved presentation-vector and curved-arrow prepared receipts across every
  typed owner-side commit refusal. Their temporary capability claim now consumes
  only after the document transaction succeeds; candidate, fence, digest, and
  transaction errors restore the exact receipt so the owner can correct a
  transient validation condition and retry.
- Made the arrow-authoring E2E lease a lock-backed invocation workspace under
  a private parent. A held advisory lock on its fixed regular marker proves
  ownership; parent sweeping reclaims only abandoned marked regular children
  whose owner lock it can acquire. Normal cleanup preserves a primary scenario
  failure, and marker-acquisition failures remove only the newly allocated
  child. The registered Arrow E2E provides the permanent workflow coverage;
  kill/concurrency proof remains a one-time operating-system lifecycle check.
- Bound the shared Curved Electron, Retro, and Curved Normal terminal-arrow
  gesture and prepared-receipt lifecycle to its opaque originating document
  session. Byte-identical foreign sessions now return typed
  `ForeignSession` / `RefreshAndRestart` outcomes before geometry or candidate
  work; a foreign commit leaves the owner's receipt redeemable and an owner
  commit consumes it only after the transaction succeeds.
- Preserved typed M3.P7 presentation-path resource recovery across the native
  and PyO3 boundary: resource exhaustion now returns `ReduceRequest`, while
  other invalid geometry continues to return `ChangeGeometry`.
- Tightened the native Polyline/Polygon path contract so every repeated vertex,
  including a non-adjacent or closing duplicate, is refused as degenerate before
  the canonical renderer-preflighted gesture lifecycle can prepare a candidate.
  That lifecycle now accepts one opaque scene point at a time, returns
  Rust-derived progress and immutable optional-hover overlays whose persistent
  geometry contains accepted vertices only. Cancellation has its own typed
  `Cancelled` / `DocumentUnchanged` outcome. PyO3 now uses the canonical opaque
  incremental route, and the obsolete full-vector preview bridge is retired.
  `Draw Polyline` and `Draw Polygon` now complete the accepted local Qt route:
  Qt supplies scene conversion, events, and wording, while Rust owns accepted
  geometry, optional-hover overlay appearance, validation, preparation, and
  commit. Qt retains only a transient accepted-press coordinate to de-duplicate
  real and QTest double-click delivery; it is never geometry or validation.
- Hardened the public native-Rust-only neutral direct-bond mutation seam with
  an opaque materialization-session origin fence. Byte-identical foreign
  sessions now return typed `ForeignSession` before mutation; external consumer
  coverage proves successful one-step history transition plus foreign, replay,
  invalid, and unsupported nonmutation behavior. The M3.P6 contracts now
  distinguish this renderer-neutral programmatic seam from the sole public V3
  Qt/Python pointer lifecycle.

### Removals and Deprecations

- Retired Python `DirectBondGestureCategoryV1` and
  `DirectBondGestureRecoveryV1`, including the obsolete Qt admission fallback.
  Invalid `DirectBondSnapPolicyV1` construction now raises `ValueError`; the
  policy remains V3-shared configuration and V1 commit category/recovery names
  remain the V3 commit-result taxonomy.
- Retired the breaking public Rust V2 direct-bond begin/admit/commit lifecycle.
  `ferrum-document` now exposes only neutral materialize/commit mutation
  operations, while the public interactive lifecycle remains V3 in
  `ferrum-document-render`.
- Removed the obsolete generated `docs/GRAPH_REPORT.md` snapshot. Canonical
  Graphify output is `graphify-out/GRAPH_REPORT.md`.

### Behavior or Interface Changes

- Corrected the V1 protocol references and local CLI examples. The contract now
  includes bounded `document.molecule.smarts.query.v1` refusal semantics, names
  `document command presentation.author.v1`, and documents fenced authoring and
  catalog insertion, including the typed stale catalog refusal.
- Corrected the pre-production `catalog.insert.v1` public CLI/protocol chain.
  Success now returns only canonical changed `document`, created `identifier`,
  observed `committed_revision`, and reusable `document_fence`; catalog-specific
  input/next-input revision and digest result fields are retired. The returned
  fence has revision zero and derives its digest from the returned CDML, while a
  stale request is a typed refusal with no partial outcome.
- Made `document.inspect` return the canonical `document_fence` from its
  admitted snapshot: `expected_revision` and `expected_digest_hex`. A caller
  can place those plain facts in a later request-owned
  `presentation.author.v1` operation; human inspection remains report-only.
- Added the closed `presentation.author.v1` CLI/protocol command for one
  request-owned presentation mutation. It replaces the vector-only route and
  admits typed Vector, terminal Electron/Retro/Normal, Curved Equilibrium,
  Polyline/Polygon, and explicit-endpoint DirectBond requests. Document-owned
  capabilities and reservations remain internal. Appearance accepts validated
  RGB/bounded-width values; stale, refused, and abandoned presentation
  reservations retain their tentative ID until a successful mutation advances
  the allocator. The nested-modal root text-authoring E2E was retired rather
  than skipped: its synchronous offscreen orchestration is not durable
  permanent-test evidence. Focused rich-text and text-placement contract tests
  remain the authoritative evidence. Redundant package-local E2Es are retired,
  and a public Qt widget regression covers visible SMARTS terminal status after
  control refresh.

- Corrected the in-progress M3.P6 directed-stereobond documentation to the
  public Rust V3 pointer-probe lifecycle: `begin_direct_bond_gesture_v3`,
  `admit_direct_bond_candidate_v3`, and
  `commit_direct_bond_admission_v3`. Qt now contributes only finite scene and
  view facts plus exact hit evidence; Rust owns endpoint resolution, snapping,
  ambiguity, fencing, candidate construction, renderer preflight, and issued
  operations. The resolved V2 endpoint values and lifecycle are internal Rust
  implementation details, and the obsolete raw preview surface is retired.
- Documented the separate typed V3 probe-error and post-resolution
  admission-refusal paths with their closed nonmodal recovery. A valid
  same-existing-atom gesture is `self_loop` / `adjust_endpoint`, not malformed
  pointer input. This bullet records the historical in-progress M3.P6 state;
  the later Developer Tests and Notes entry records its completed closure.

### Developer Tests and Notes

- Rotated the complete 2026-08-20 and older changelog history to
  [CHANGELOG-2026-08d.md](CHANGELOG-2026-08d.md), retaining the two newest
  dated blocks in this active changelog.
- Accepted the bounded M3.P7 Polyline/Polygon slice after a fresh local build
  and `./all_test.sh`; focused public binding and Qt behavior evidence covered
  the declared workflow. Generic splines, broader path grammar, property
  editing, association, and factory semantics remain separate work.
- Closed M3.P6 as supported after a fresh local build and `./all_test.sh`.

### Fixes and Maintenance

- Routed typed V3 direct-bond begin refusals through the same nonmodal Qt
  recovery as admission-time refusals. Mouse and keyboard starts now retire
  transient authoring state and show the closed actionable recovery without a
  V1 fallback.
- Extended M3.P6 V3 directed-wedge durability coverage across ExistingExisting,
  ExistingNew, NewExisting, and NewNew forms. Solid and hashed wedges now prove
  one history transition, undo/redo behavior, and persisted tip-to-base endpoint
  identity categories after reopening.
- Added durable M3.P6 V3 contract coverage for all four directed endpoint forms
  and normal, solid-wedge, and hashed-wedge presentations. Directed `w1` and
  `h1` commits now prove undo, redo, save/reopen retention; stale-digest and
  duplicate-bond post-resolution refusals prove revision, digest, and CDML
  remain unchanged.
- Added public Qt M3.P6 coverage for directed ExistingNew solid and hashed
  wedges: a clicked existing atom remains the CDML tip/start and the released
  blank-canvas endpoint becomes the base/end.
- Corrected the CDML directed-wedge authoring description to identify the
  public V3 pointer-probe lifecycle rather than internal V2 endpoint intents.
  The durable `w1`/`h1` pointer direction remains CDML `start` tip to `end`
  base.
- Corrected `begin_direct_bond_gesture_v3` error translation so V2 document
  and operation failures retain their typed admission or document-gesture
  category instead of being mislabeled as `invalid_hit_evidence`. Pointer
  evidence failures continue to use `DirectBondPointerProbeErrorV3`.
- Corrected Qt Contract wording that described directed direct-bond authoring
  as future work and incorrectly narrowed Bond Properties. The bounded
  Normal/Solid-wedge/Hashed-wedge vocabulary applies to M3.P6 drawing actions;
  the existing properties editor retains its independently supported broader
  Rust-owned style vocabulary.

## 2026-08-21

### Fixes and Maintenance

- Corrected the public direct-bond refusal wording and the renderer-neutral
  candidate documentation. The supported bounded authoring range includes
  normal single/double/triple, solid-wedge, and hashed-wedge presentations;
  renderer preflight has read-only access to the fenced candidate receipt facts
  needed for its target plan while document admission and commit remain owned
  by the V2 lifecycle.
- Replaced the direct-bond V1 lifecycle and obsolete Qt mutation tail with the
  V2-only public `begin -> admit -> commit` receipt flow. `ferrum-document`
  materializes renderer-neutral candidate mutations; admission preflights them
  internally in `ferrum-document-render` and retains the fenced target plan.
- Made V2 admission freeze the exact renderer operations for normal
  single/double/triple, solid-wedge, and hashed-wedge bonds. The native matrix
  proves preflight-to-commit equivalence for every existing/new endpoint form,
  including durable tip-to-base `w1`/`h1` order and both NewNew atom IDs.
- Exposed typed `DirectBondCommitError` categories and recoveries through the
  binding. Qt maps that closed native taxonomy to actionable nonmodal recovery,
  consumes only Rust-issued operations, and has public NewExisting/NewNew and
  visible-action-reset behavior coverage.
- Fixed curved-equilibrium Qt previews to render Rust-issued lanes and
  arrowheads with the same stroke/fill roles as committed arrows, and
  strengthened public authoring/projection coverage.
- Corrected the accepted bounded `CurvedRetroArrowV1` and
  `CurvedNormalReactionArrowV1` capability status across the Rust-first plan,
  Qt capability matrix, and CDML/backend/frontend contract.
- Clarified the Rust-first plan's route-local exclusions: CurvedRetro,
  CurvedNormal, and CurvedEquilibrium remain distinct dedicated contracts and
  do not overload or subsume one another.
- Reworked the arrow-authoring E2E to exercise public New, selected-tab,
  observation, and Undo controls rather than private tab registration and
  snapshot/history seams.

### Local Build and Runtime

- Current acceptance is the runnable staged local runtime under `build/`. Older
  same-day wheel and installed-site receipts below are retained only as historical
  development record and do not describe the supported build or test contract.

- Renamed the extension's internal runtime boundary from wheel terminology to
  the staged local extension model. `PyInit` still captures the admitted module
  path exactly once and derives the sealed module-relative native-library path;
  direct chemistry, protocol, SMARTS, and interchange calls share that one
  immutable runtime authority.

- Retired the obsolete `output_native_wheel/` publication tree and wheel-only
  metadata. `./build.sh` removes that fixed legacy root before its checkout
  budget check; the supported local build owns and stages only `build/`.

- Removed the retired wheel-artifact receipt route from the local native runtime builder. Its
  immutable input manifest and Mach-O closure checks now use local-runtime terminology and a
  versioned input schema, matching `./build.sh`'s local-only contract.

### Additions and New Features
- Updated the in-progress M3.P6 directed-stereobond contract. `Draw Solid
  Wedge Bond` and `Draw Hashed Wedge Bond` are bounded Rust V2 authoring
  actions, while the broader independently supported Bond Properties style
  vocabulary remains unchanged. Generic stereo/CIP or E/Z semantics,
  inference, and stereo import/export expansion remain separate contracts.
  M3.P6 awaits a fresh local build, full `./all_test.sh` receipt, and final
  audit before it is supported.
- Accepted the bounded Rust-owned `CurvedEquilibriumArrowV1` capability.
  Its closed `<arrow type="curved-equilibrium">` record preserves exactly start,
  control, and end; Rust derives both cubic lanes and opposing heads, fences the
  renderer-preflight receipt, and exposes a typed PyO3 lifecycle. It excludes
  `equilibrium2`, variable-point spline semantics, and reaction association.
  The dedicated Qt action now uses a three-click lifecycle and paints only the
  Rust-issued two-lane/head DTO. A fresh local build and `./all_test.sh`
  provided the current validation receipt. Generic spline/head/property
  editing, reaction association, and generic factory semantics remain separate
  future contracts.
- Defined the in-progress M3.P4 `CurvedNormalReactionArrowV1` contract. It
  accepts only direct-root `<arrow type="curved-normal">` records with exact
  start, control, and end points; Rust owns one cubic terminal-head geometry
  and the opaque lifecycle, while the named Qt action captures three clicks and
  cancels transient state with Escape. Focused native/PyO3 and staged Qt
  evidence remain required before support is claimed. Spline compatibility,
  normal-arrow overloading, configurable heads/properties, reaction
  association, a generic factory, and curved-equilibrium geometry remain
  separately scoped.
- Documented the exact M3.P4 PyO3 gesture route and its type-specific CDML
  profile so generic normal-arrow compatibility fields cannot be mistaken for
  authored curved-normal facts. The capability remains in progress pending its
  focused and staged validation evidence.
- Defined the in-progress M3.P3 `CurvedRetroArrowV1` contract. It uses one
  direct-root `<arrow type="retro">` with exact start, control, and end points,
  the shared Rust `CurvedTerminalArrowKindV1` geometry/preview/commit lifecycle,
  and one named three-click Qt action. Staged Rust, PyO3, and Qt evidence remains
  required before the capability becomes supported; spline, property editing,
  reaction association, and other curved arrow families stay separately scoped.
- Added the public Ferrum `Draw Curved Electron Arrow` action. Qt captures only
  start, control, and end points; the Rust-owned renderer issues the quadratic
  curve and arrowhead overlay, preflights the complete candidate, and commits
  one opaque history receipt. The third click automatically commits; typed
  incomplete-point guidance and Escape leave the document unchanged.
- Added the Rust-owned foundation for multi-point Polyline and Polygon authoring: ordered finite path geometry, bounded point counts, renderer-preflighted fenced commit, and cancellation. Its initial full-vector client bridge is now retired; PyO3 uses the incremental transaction documented above while the Qt input bridge remains in progress.
- Added Rust-owned selected-root multi-record SDF V2000/V3000 export. The local
  `ferrum document export-sdf` command accepts repeated `--molecule-id` values
  and atomically publishes one complete file. Qt's explicit selected-molecule
  actions transfer only current durable-root membership, verify the canonical V2
  receipt, and preserve the established one-record SDF export actions.

- Added registry-owned CML, CML1, and CML2 input support to `ferrum convert`.
  The closed Rust interchange profile now decodes bounded CML before acquiring
  the chemistry engine and preserves typed refusal facts for unsupported input.

- Added Rust-owned v2 Draw Bond admission for existing and blank canvas endpoints, including atomic detached two-carbon molecule creation for blank-to-blank gestures.
- Routed the Qt Draw Bond pointer flow through the shared endpoint classifier at both press and release, preserving Rust-owned geometry, opaque admissions, and durable bond selection.

- Added `ferrum haworth`, a Rust-only direct-glycosidic Haworth SVG vertical slice. It accepts one bounded C/O structural SMILES argument or standard input, parses and lays out the closed profile without an engine bundle, and writes deterministic SVG atomically to a named output or directly to standard output.
- Implemented the P0.1 normal-order authoring lanes without closing the milestone. Rust and PyO3 now prove all six normal single/double/triple combinations across existing and new-carbon endpoints, including pure admission, atomic commit, history, and CDML reopen. Qt Draw Bond freezes the visible Next Drawing normal order at activation and retains that immutable order across sticky drags; it supplies no chemistry. The isolated paired native-plus-Qt wheel E2E remains pending before P0.1 can be accepted.
- Defined the next P0.1 normal-order completion contract. The existing Draw Bond action now has one intended source of normal single/double/triple order: the visible Next Drawing preference, frozen at Rust admission. Qt supplies only closed presentation and endpoint UI facts to the opaque Rust receipt flow; Rust remains sole owner of chemistry, identity, history, and commit. Automated completion requires bounded Rust/PyO3 and focused Qt evidence plus one isolated dual-wheel offscreen E2E with temporary inline input. Wedges, aromatic bonds, free-space starts, heteroatom endpoints, and selected-root P0.2 remain deferred.
- Accepted the bounded direct normal-bond drawing slice. Rust performs pure candidate admission and opaque receipt commit; the Qt Draw Bond client has a fixed normal-single carbon profile. Isolated dual-wheel QAction/viewport evidence proves one commit and Escape cancellation. Closure receipts are `/private/tmp/ferrum-direct-bond-candidate-admission-p1-fix.md`, `/private/tmp/ferrum-direct-bond-qt-receipt-review.md`, `/private/tmp/ferrum-direct-bond-dual-wheel-e2e.md`, and `/private/tmp/ferrum-direct-bond-final-acceptance-review.md`. Wider bond styles/profile, richer drawing interactions, and complete OASA/BKChem parity remain deferred.
- Added one isolated dual-wheel, offscreen Draw Bond E2E. It loads only the supplied native and Qt wheels, verifies both installed module digests, drives the public QAction and viewport gesture, and proves normal C-C authoring plus Escape-after- admission cancellation without a committed fixture or network route.
- Accepted the bounded selected-atom `Change Element` slice in the capability ledger. It commits Rust-owned `set_atom_element`, restores the exact durable selected atom after projection replacement, and recovers one failed presentation installation through the typed one-refresh route. Closure evidence includes the dual-wheel offscreen visual E2E, fresh dual-wheel fail-once regression, `/private/tmp/ferrum-change-element-projection-boundary-fix.md`, and `/private/tmp/ferrum-change-element-final-acceptance-review.md`. A generic atom-property editor and broader direct-bond or presentation workflows remain deferred to separate plans.
- Added the selected-atom `Change Element` vertical slice. The Edit-menu action rechecks one durable selected atom, uses its bounded `Element symbol:` dialog, commits only Rust's `set_atom_element` operation, and restores that exact atom selection after the authoritative replacement scene installs. The isolated local-runtime/offscreen receipt is `packages/ferrum-chem-qt.app/tests/e2e/e2e_native_change_element.py`; it uses temporary inline CDML and verifies public action, capture, undo/redo, save, and Rust reopen without a network or committed fixture.
- Corrected the File/Open and Qt contract documentation: CML/CML2 is a closed Rust-owned new-document input profile, not a refused desktop format.
- Documented the closed Rust-owned CML/CML2 File/Open profile: it converts into a clean new document, preserves source provenance only, and writes CDML on its first Save.
- Corrected Qt interactive CML File/Open to retain the pristine initial tab and install descriptor-backed interchange as a new document; native CDML keeps its established pristine-tab replacement behavior.
- Corrected the isolated Qt CML E2E to use the stable `OPEN_DOCUMENT / INVALID_DOCUMENT` presentation title while retaining its typed and redacted refusal checks.
- Corrected the isolated Qt CML E2E to validate the installed immutable molecular projection rather than CML source identifiers, which the Rust document conversion intentionally reallocates.
- Corrected the isolated Qt CML E2E to assert the public imported-document contract: one clean tab, no CDML save baseline, and the input basename title.
- Replaced the CML new-document route string with an API-issued opaque route handle. Qt now retains the descriptor handle through its existing queued local-open request, while PyO3 accepts only that exact native handle before resolving the closed Rust interchange registry.
- Extended the focused CML route-handle binding proof to cover Python shallow and deep copying, pickling, and low-level object allocation. Each route authority reconstruction path is refused before local-file admission.
- Added the bounded Rust-owned local interchange bridge for ordinary File/Open. The Python extension now exposes immutable eligible-open descriptors and accepts only their opaque route tags; the existing Qt local-open queue carries that tag to Rust and installs the existing one-time admission receipt as a new document. CML/CML2 decoding remains Rust-owned, while current-tab replacement, append/import modes, export, and receipt UI remain unchanged.
- Fixed the attached-cyclohexane E2E source-isolation call matcher so the generated child program compiles its intended method-call regex.
- Fixed the isolated attached-cyclohexane E2E child program's missing `re` standard-library import during source-isolation verification.
- Fixed a generic Qt action-handoff race where a `QMenu` could begin closing before its `QAction` reached Python. The window-owned handoff now records popup terminal lifecycle from `Show`, then attaches the action continuation to that recorded latch so late action dispatch still settles safely.
- Fixed shared Qt popup-latch lifecycle reuse and cleanup. Reopened menus now require their own terminal event before a deferred action can run, and destroyed transient popups release their handoff-owned latch QObject.
- Accepted the bounded Qt CML/CML2 new-document File/Open extension. Interactive File/Open preserves the bootstrap tab and creates one clean CML tab through the Rust-issued native route. The native-wheel E2E receipt is `/private/tmp/ferrum-cml-qt-new-document-e2e-receipt.json` (SHA-256 `0ffbde86632859e355dacb2e7c21a54a9e7d1553d57607dd8f7dd28052b12e33`); the final independent re-review is `/private/tmp/ferrum-cml-qt-new-document-final-rereview.md`. CML append or current-tab replacement, live receipts, export or conversion, a generic importer, and broader semantic profiles remain deferred.
- Added one isolated root E2E for the Rust-owned CML/CML2 new-document route. It installs only the supplied native wheel into a temporary system-site-packages venv, opens inline valid and invalid CML through the ordinary queued Qt `open_file_path` API, and verifies one clean new tab plus one typed redacted refusal without a persistent fixture, dialog, network, or browser.
- Accepted the bounded shared-anchor `Attach Cyclohexane Ring` capability. The final autonomous local Qt workflow covers menu activation, typed atom picking, Rust-owned preview and one commit, save/reopen, undo/redo, Escape, refusal, cancellation, and tab lifecycle using the exact wheel emitted by bare `./build.sh native` (SHA-256 `4a438e34473685700d76ae11d8489b4703bfb9c63b0549c39e278ba6ca221ddf`). Evidence: `/private/tmp/ferrum-cyclohexane-final-e2e-receipt.json`; independent acceptance: `/private/tmp/ferrum-cyclohexane-final-acceptance-rereview.md`. This closes only the C6 atom-attachment slice; broader OASA/BKChem parity, other ring families, fusion, templates, and compatibility work remain deferred.
- Retired the false declarative `attach_cyclohexane` mode projection. `Attach Cyclohexane Ring` remains the existing Edit-menu and ribbon QAction, now owned only by its direct line-tool intent and native lifecycle; shared status reads that intent without claiming generic mode-manager ownership.
- Promoted `Attach Cyclohexane Ring` to one first-class shared pointer mode. Its existing QAction is now the one registry, ribbon, declarative-resource, and line-tool client; selecting it retains the actual C6 drag intent while shared mode presentation reports `attach_cyclohexane`. No second native preparation, commit, or cancellation path was added.

### Behavior or Interface Changes
- Adopted `urn:ferrum:cdml` as Ferrum's sole ordinary CDML namespace. New
  documents, typed editing, direct semantic indexing, and CD-SVG admission now
  share one strict Rust-owned namespace contract; unqualified and historic
  BKChem roots are rejected while foreign children remain opaque below a
  canonical Ferrum root.

- Reconciled the P0.1 direct-bond contract with the Rust-owned endpoint matrix:
  `ExistingExisting`, `ExistingNew`, `NewExisting`, and blank-canvas `NewNew`.
  New endpoints remain carbon-only; normal single/double/triple orders remain
  selected by Next Drawing, while wedges, aromatic bonds, and free-form element
  selection remain deferred.

- Made `./build.sh` build only the runnable local Ferrum program under `build/`.
  It neither publishes wheels nor installs packages, removes its owned compiler
  cache on every exit, and enforces the 20 GiB checkout limit before and after
  compilation. `./all_test.sh` now consumes that local extension after its
  repository hygiene lane instead of relying on a globally installed binding.

- Aligned CI and contributor documentation with the local-build contract. CI
  now builds `build/` and runs `./all_test.sh` without installing Ferrum
  packages; wheel commands are documented only as caller-owned release work.

### Fixes and Maintenance
- Unified direct CDML and gesture admission for curved-equilibrium geometry
  under one typed finite-coordinate, span, forward-tangent, and 45-degree
  contract. Focused proof now confirms the Rust preview projection equals the
  committed projection.
- Hardened Qt curved-equilibrium completion with explicit Rust observation and
  selection recovery, plus durable focused authoring checks.
- Preserved curved-equilibrium axes as cubic paths during Rust SVG lowering.
- Reclaimed only abandoned marker-owned Arrow E2E workspaces under a short
  parent sweep lock. Live concurrent leases retain their held marker lock, and
  unmarked, linked, or unexpected children remain untouched.

- Made the shared curved terminal-arrow error text family-neutral. Retro PyO3
  callers retain the existing typed category and recovery while receiving
  terminal-arrow diagnostics rather than electron-arrow wording; retired the
  unused electron-only geometry compatibility aliases.

- Split the frozen curved-electron-arrow projection DTO from the general Python
  projection binding, keeping the public registration surface stable while
  separating the specialized display contract.

- Centralized Curved Electron Arrow quadratic lowering and terminal-head geometry
  in the document crate. The persisted projection and Rust live overlay now
  consume one exact cubic/head result; electron roots reject normal-arrow head
  attributes, including `end`, and report their required three-point geometry.
- Kept incomplete Polyline and Polygon gestures armed with non-modal
  cardinality guidance. Typed Rust path failures now retire the transient Qt
  owner and use the shared refusal presenter instead of being treated as an
  incomplete preview.
- Fixed Polyline commit receipts in documents that already contain Polygons. The
  renderer-preflighted receipt now retains the validated root kind alongside its
  generated identifier, so selection always targets the newly committed path.
- Fixed selected-molecule SDF destination normalization by importing its path
  authority, and strengthened the public receipt contract to state and prove
  Rust-canonical document source ordering independent of selection click order.
- Corrected direct-root toggle selections to retain Rust's canonical document
  source order regardless of click order, including the public Python binding.
- Corrected the active P0.2 plan to match the released Rust direct-root
  contract: canonical mixed molecule and plus selections translate in one
  fenced atomic operation, while Qt supplies gestures and renders returned
  observations.
- Fixed Ferrum tab shutdown to retire an active Select Structure pointer owner
  before its viewport is disposed. This keeps event-filter ownership aligned
  with the tab lifecycle for ordinary close and application shutdown.
- Made `./all_test.sh` execute the bounded staged-runtime CLI E2Es after its
  launcher proof. The explicit runner covers human CLI verbs and selected-root
  SDF V2000/V3000 export through `build/bin/ferrum`, without an installation or
  a broad E2E file-discovery surface.
- Added the staged offscreen Qt P0.2 root-selection E2E to the explicit local
  runner. It proves click/marquee selection, shared drag translation, undo, and
  save/reopen through the staged extension, then closes the ordinary window
  cleanly without using a globally installed binding.
- Removed the uncalled persistent native-source archive cache. Local builds now
  materialize pinned source archives only below their disposable staging root,
  which is removed after every build and cannot accumulate orphaned archives.
- Removed completed `chemistry.convert`, `document.generate_coordinates`,
  `ferrum convert`, and `ferrum coords` work from the outstanding TODO list.
- Removed the out-of-scope release-wheelhouse builder, artifact inventory,
  release E2E, and installation documentation. `build.sh` remains Ferrum's
  single supported local build path and produces the runnable CLI and Qt app
  under `build/`.
- Made interactive File/Open retire an armed canvas authoring tool before its
  detached Rust admission begins. An armed pristine canvas still fences the
  source into a separate tab and remains selected, but cannot retain a stale
  event filter after Open completes.
- Fixed ring and other non-bond pointer tools by moving Draw Bond's dependency
  import out of the shared dispatch method, preventing Python from treating the
  `ferrum_qt` package name as an unbound local before that branch runs.
- Removed the obsolete wheel-install SMARTS harness and its private fixture
  chain. Local runtime and public Qt workflow coverage remain the permanent
  validation boundary for Ferrum development.
- Hardened the local runtime receipt around Cargo's normal/build release closure
  and sealed both launchers; `all_test.sh` now proves local launcher provenance
  before binding and Qt validation, while build-lock cleanup retains foreign or
  nonempty locks for inspection.
- Centralized synchronous Qt action refresh ownership. Popup handoff observes
  only its owned popup, main-window refresh coalesces layout callbacks, and
  line-tool cancellation no longer re-enters local-open refresh.
- Replaced direct-bond clients' private endpoint classifier coupling with one
  public opaque endpoint-resolution contract shared by pointer and keyboard
  authoring.
- Kept incoming checkable pointer-tool actions visibly active after handoff, and
  refreshed central command state when authoring arms or retires so in-place
  File/Open is available only for an idle current tab.
- Made live-property observation a closed Available/Unavailable/Stale boundary:
  stale state receives one explicit refresh, while unexpected defects surface
  instead of being misrepresented as ordinary unavailable UI state.
- Routed Reaction Inspector and selected-root SMARTS capture through public
  MainWindow ownership transactions, retiring every pointer tool and returning
  typed expected outcomes without reflective private calls.
- Made `build/bin/ferrum` resolve its sealed sibling chemistry engine under
  `build/runtime/engine-v1`, removed the per-user engine install/status route,
  and added an engine-backed local CLI conversion check to `all_test.sh`.
- Narrowed Reaction Composer and Reaction Inspector recovery to their declared
  native refusal and presentation outcomes, so unexpected UI and programming
  defects surface instead of becoming misleading user refusals.
- Updated shared window chrome to show every active tool, dispatch only stable
  action-registry identities, and retire the superseded property dock.
- Removed release-only native-wheel publication commands, Qt wheel staging, and
  current-pointer selection from the local builder. The retained wheel evidence
  helpers now serve only `build.sh`'s runnable local runtime.
- Removed obsolete toolbar aliases; the supported authoring command surface is
  `AuthoringRibbon`.
- Recovered Rust-accepted reaction creation after an initial Qt projection
  installation failure; composer refreshes authoritative state and reselects
  committed members instead of displaying a false role refusal.
- Closed SDF worker failure delivery to `FerrumNativeMoleculeExportFailure`,
  refusing unexpected signal payloads without exposing arbitrary exception text.
- Completed the public document-tab lifecycle boundary across Ferrum UI clients
  and their test doubles. SMARTS capture, main-window action refresh, reaction
  composition, and other tab consumers now read the stable `is_disposed`
  contract rather than owner-internal disposal state.
- Removed the unreferenced preproduction action and editing-tool toolbar
  modules. The single supported `AuthoringRibbon` remains the window's only
  command surface, and startup coverage now exercises its live New action and
  persisted ribbon/property-dock visibility state.
- Aligned Qt contract tests with the current generic interchange preparation API,
  two-field Next Drawing snapshot, and visible Draw Bond action wording.
- Migrated the Qt SDF insertion bridge to Rust's generic immutable interchange
  record receipt API. The SDF workflow still selects every committed atom from
  the prepared receipt, while SDF-specific Rust binding names remain retired.
- Aligned the native SDF export fixture with the canonical Rust interchange
  record namespace, preserving the real UI and native export path.
- Kept one-shot attached-cyclohexane receipts owned through native commit, so
  failed commits retire their live receipt while successful commits clear it
  without a second retirement. Generic action refresh now leaves persistent
  pointer tools armed after accepted projection refreshes.
- Made the local runtime extension path Python-ABI-specific. `build.sh`, the
  local-runtime receipt, and `all_test.sh` now share one resolver for the
  importable extension filename, so local validation cannot silently use a
  globally installed `ferrum_chem` binding.
- Made root local builds exclusive while preserving nested native-engine
  staging, so a failed concurrent `build.sh` invocation cannot remove another
  compiler's target tree. Move Complete Roots asks Rust whether the press is on
  its already validated complete-root selection, so selected-root drags retain
  the full selection while blank-canvas presses retain Rust-owned marquee
  selection. Rust snaps its native root anchor after the raw drag delta.
- Kept the implicit-carbon picker test on its real authoring boundary: a
  rendered carbon is an existing endpoint, while C6 retains its bounded
  fallback check instead of fabricated projection objects or a retired
  direct-bond picker.
- Made rendered C6 attachment hits translate their public document source ID
  to one installed projection object ID before Rust admission; missing or
  ambiguous projection mappings now refuse rather than sending an invalid
  identity across that boundary.
- Restored the bounded P0.1 Draw Bond contract: its shared Next Drawing client
  offers only normal single/double/triple order and its Rust gesture creates
  carbon at new endpoints. Refused window shutdown now retires transient
  viewport input ownership before preserving an unsaved document. Restored the
  interaction-owned line-preview update used by Move Atom and Wavy gestures.
- Restored canonical periodic-table validation for atom-element operations, so
  invalid symbols are refused through the existing typed API boundary.
- Kept accepted Rust mutations recoverable when projection installation fails:
  Ferrum retains pending authority and raises the documented typed presentation
  error for authoritative refresh.
- Made Haworth insertion treat implicit atom and bond projection locations as
  occupied page space, preserving the empty-page placement contract.
- Retired the unused direct-bond start-picker seam. Main-window authoring now
  verifies the canonical classified-endpoint path after popup handoff settles.
- Kept valid Haworth render publications openable when live SMARTS cannot
  establish its private atom correspondence. The binding now records typed
  `unsupported_document` SMARTS readiness without minting a plan or receipt.
- Routed native plus and Text render targets through the public presentation projection facade, so valid roots install without a private cross-module dependency.
- Made local CML interchange admissions derive their returned render observation
  and CML origin provenance from the committed descriptor/session state, so the
  authenticated receipt agrees with its redeemed snapshot instead of refusing
  as stale or unknown-origin input.
- Made focused Draw Bond pointer checks establish and restore their own normal
  single-bond preference, retire Qt objects through the ordinary deferred
  lifecycle, and cover only real user-visible authoring contracts.
- Added a fail-closed local-runtime freshness receipt. `./build.sh` now records
  the Cargo-resolved source closure, native inputs, and staged artifact hashes;
  `./all_test.sh` rejects stale local artifacts before importing Ferrum.
- Made render-projection disposal retire its detached `QGraphicsScene` after
  its owned roots, with idempotent terminal lifecycle behavior.
- Removed an invalid synthetic direct-bond click from the ribbon handoff test; the test now proves action ownership without leaving a required refusal dialog open.
- Kept Qt drawing-test CDML input local and restored package-qualified access to the document-tab fixture seam.
- Made the retired Python-brand boundary inspect the tracked worktree rather
  than stale staged snapshots. It continues to reject live OASA/BKChem module
  paths, imports, and dependency manifests without coupling code validation to
  intermediate source-control state.

- Removed the publication-only `build.sh` wrapper simulator. It faked Cargo,
  Python, disk space, locks, signals, and publication without compiling Ferrum;
  the local build contract is documented in `docs/LOCAL_BUILD.md` instead.

- Made the blank-canvas Draw Bond acceptance E2E verify Escape through saved
  CDML facts and public history instead of an unstable PNG encoding. The
  check now tolerates only ephemeral cursor repaint while still requiring
  Escape to preserve the committed Rust document.

- Preserved final selected-SMARTS search and clear confirmations after control-state refresh, so
  selected-capture recovery guidance no longer overwrites terminal status.

- Moved selected-SMARTS empty-root admission to native capture issuance. Qt now presents the
  typed `selected_root_empty` recovery and retains a not-ready selected source instead of
  reporting a token as ready until Find refuses it.

- Preserved the visible keyboard scene cursor across successful authoritative Qt
  document-mutation scene replacement, so the next keyboard authoring gesture
  retains the user's current endpoint position.

- Made live-SMARTS publication distinguish an unrenderable accepted document
  from an unpublished plan. Partial render observations now open normally,
  retain no SMARTS plan or receipts, and return the closed
  `unsupported_document` SMARTS reason on later queries.

- Made revoked live-SMARTS receipts consistently return their typed
  `receipt_unavailable` refusal before plan lookup. Binding coverage now asserts
  the public refusal facts and checks opaque representations only for meaningful
  payload leakage without mutating installed native artifacts.

- Made Draw Bond snapshot the selected next-drawing parameters at valid pointer press and clear
  that snapshot when the interaction resets. The blank-canvas wheel E2E now selects and verifies
  public `Single` before authoring, then proves the saved canonical `n1` C-C bond topology through
  durable atom identities and endpoints.

- Added the fast root-build wrapper lifecycle E2E to `./all_test.sh`, so ordinary test runs
  now fail on broken cleanup, lock ownership, disk admission, publication retention, or
  signal handling without requiring retained wheel artifacts.

- Repaired paired developer-wheel provenance and cleanup as one contract: the v4 receipt
  source-closure fences the Qt wheel with the native artifact and requires every delivered
  `ferrum_qt/**` member to match an admitted staged payload byte-for-byte, while allowing
  intentionally unshipped admitted sources and the generated wheel dist-info tree. It also
  supports receipt-bound isolated acceptance plus atomic replacement and signal-cleanup
  fixtures without source-rewriting hacks.

- Split the root build lifecycle into its owned shell module and `build_native_wheel.py`
  into a thin CLI facade with concrete builder modules. No compatibility layer was added:
  root `./build.sh` and the builder CLI commands remain the developer entry points.

- Moved root native-artifact lifecycle ownership into
  `tools/build/native_artifact_lifecycle.sh`, kept `build_native_wheel.py` as a thin
  facade over focused public builder helpers, and extracted private live-SMARTS Rust
  test modules. The responsibility splits preserve existing behavior.

- Made Draw Bond press-time classification and native-begin failures retire the exact
  armed intent before unexpected failures propagate unchanged. Only declared,
  user-correctable native begin categories now use the ordinary refusal UI.

- Bound the blank-canvas Draw Bond dual-wheel E2E to the entire supplied Qt package rather than
  a hand-selected routing list. It now derives every safe regular `ferrum_qt/**/*.py` wheel
  member and requires the isolated installed package to have the exact same members and bytes
  before public UI acceptance runs.

- Made unexpected Draw Bond endpoint, native-admission, and preview-overlay failures retire
  their active gesture before propagating. Typed Rust refusal receipts retain their ordinary
  user-visible recovery path; internal failures are not rewritten as refusals.

- Made Draw Bond release an explicit Rust-admission route before the generic
  existing-origin line guard. A blank-canvas NewNew gesture now redeems its final
  opaque admission exactly once, advances native history, and keeps Draw Bond armed;
  other line tools retain the shared durable-origin requirement.

- Corrected the blank-canvas Draw Bond E2E diagnostics to read the native tab's
  public `current_snapshot` property rather than treating it as a callable.

- Corrected the blank-canvas native-wheel E2E provenance gate to consume the exact staged and
  worktree source-closure receipt schemas. It now verifies non-empty per-file manifests,
  fingerprint digests, and the selected `current` wheel receipt without inventing redundant
  count or aggregate-digest fields or re-reading mutable worktree sources after publication.

- Moved final native-wheel source-closure validation and the atomic `current` replacement into
  one Python-owned publication transaction. The builder now records
  the exact source files admitted by the staging copy policy, verifies the raw staged copy,
  retains that input closure in its receipt, and refuses the `current` swap when a live
  worktree change is detected before final publication. Deterministic helper and wrapper
  fixtures prove a changed `session/direct_bond.rs` preserves the prior selected publication.
  The real `publish-publication` self-test path now proves the same refusal after its staged
  manifest is recorded and before its final live-source comparison; documentation defines
  this as an observed-boundary integrity check rather than an editor lock.

- Renamed the native-wheel publication self-test helper to a descriptive ordinary module name.
  The helper remains private to the self-test runner by its documented responsibility rather
  than resembling an ignored temporary file.

- Replaced fragile source-rewritten wheel-publication interruption fixtures with a real
  publisher atomic-replacement failure fixture and a wrapper-owned signal-cleanup fixture.

- Proved the canonical v2 Draw Bond commit history contract with a Rust blank-canvas
  New-New regression: one accepted C-C gesture creates exactly one undo target, while
  Undo restores blank content and Redo restores the committed molecule.

- Corrected the blank-canvas Draw Bond E2E's endpoint oracle. Its saved-CDML proof
  remains the public semantic blankness check; the test no longer mistakes the
  universal paper and grid projection items returned by Qt hit testing for authored
  molecular content.

- Hardened native-wheel `current` publication replacement with a private mode-0700 source-link stage, exact target validation at the `os.replace` boundary, and a fail-closed pointer state machine. An unexpected source or post-swap `current` state now preserves both the prior known-good payload and validated candidate; the prior payload is retired only after a final exact `current` verification under the cooperating-build lock. Wrapper E2E fixtures cover both races without compiling native sources.

- Restored the native-wheel E2E runner's explicit private `run` helper import after its
  runner/support split, preserving the isolated installed-wheel child-process boundary.

- Split the native-wheel direct E2E into its public runner and a private sibling
  support module. The runner retains CLI parsing, one resource lifecycle, and
  installed-wheel orchestration; the support module owns validators and probes.
  Isolated installed-wheel child commands and proof behavior remain unchanged,
  while the shipped typing-metadata continuation now uses tab indentation.

- Modularized the native-wheel builder's source-closure and publication-integrity responsibilities.
  The stable CLI facade now delegates canonical source manifests, receipt and wheel validation,
  sealed engine-bundle validation, and packaged native-closure assembly to a focused private module.

- Hardened copied engine-bundle ABI admission. The manifest validator now requires an exact JSON
  integer for `adapter_abi_version`, rejecting Boolean and floating-point values before comparing
  the configured ABI; pure builder fixtures cover all four malformed scalar types: Boolean,
  floating-point, string, and null.

- Validated the copied CLI engine bundle against its canonical digest-bound manifest before the
  native publication atomically selects `current`. The shared pure validator rejects malformed,
  altered, missing, symlinked, and extra bundle members; the wrapper fixture proves a copied
  adapter mutation preserves the prior selected publication.

- Defined one canonical native-wheel staged source-subset manifest for receipt creation and
  pre-swap publication validation. It captures the post-rewrite Ferrum source tree, records its
  exact builder-owned notice and `.dylibs` exclusions, admits every other staged regular file,
  and leaves wheelhouse, Cargo output, and the engine bundle outside the source boundary.

- Revalidated each copied native-wheel publication immediately before its atomic `current`
  pointer replacement. The builder now recomputes the completed staged Ferrum source closure
  and checks the copied wheel filename and digest against the copied receipt; wrapper fixtures
  prove copied wheel or receipt mutation leaves the prior publication selected.

- Sealed each native-wheel build to a canonical staged Ferrum source-closure manifest. The
  builder now compares source and staged workspaces before Maturin, retains the fingerprint in
  its receipt, and refuses an artifact whose receipt does not match that closure and wheel.

- Declared the native `DocumentSession.can_undo` and `can_redo` Boolean properties in the shipped
  typing stub. The isolated installed-wheel E2E now proves their fresh, commit, undo, and redo
  transitions while independently auditing the delivered stub surface.

- Replaced the blank-canvas Draw Bond E2E's saved-CDML standard-library XML parsing with the
  approved defused XML parser, retaining its empty-document and C-C graph proof.

- Kept the main-window lifecycle as the sole Undo and Redo QAction refresh owner.  Its
  pending and busy guards now combine with each active tab's Rust-owned history availability
  without later generic action writes overriding those facts.

- Made Undo and Redo action availability an independent Rust-owned session capability. The PyO3
  binding and native Qt tab relay the exact history facts, while the selected-window actions now
  refresh from those facts instead of inferring reachability from document snapshots or content.


- Corrected native-wheel source-closure publication validation to fingerprint the completed
  `maturin-project` staging input, including its deterministic packaging transforms. The wrapper
  now revalidates the copied candidate against that real staging root before atomic publication;
  its synthetic E2E fixture uses the same layout.
- Gated the Python-binding-only local interchange runtime resolver, UTF-8 source reader, and
  local new-document preparation seam behind `python-binding`. CLI and protocol builds retain
  their shared generic interchange admission/preparation paths without compiling unused PyO3
  adapter code.
- Corrected the v2 Draw Bond PyO3 NewExisting receipt contract test: it now proves the terminal
  existing endpoint remains `"b"` and independently proves a new start atom was created.
- Repaired the feature-enabled PyO3 interchange binding test to extract native string and integer
  values before asserting source, receipt, and replay facts.
- Removed unused document test-only molecule ID sequence seams, eliminating their dead-code
  warnings without changing direct-bond admission.
- Restored the direct document-domain snap-policy import for feature-enabled direct-bond binding
  tests after the Python-binding module split left the test facade without that type.
- Replaced the private interchange-import summary's positional argument list with one named,
  ownership-preserving facts struct, keeping its protocol DTO and public behavior unchanged.
- Removed stale Python-binding split imports and unreachable local-interchange warning paths.
- Corrected the presentation-creation gesture PyO3 preview boundary to translate native gesture
  errors before the fallible Python-object conversion, preserving its typed public exception.
- Corrected the local interchange PyO3 text-reader refusal conversion so an issued descriptor's typed, redacted source refusal becomes one Python error instead of a nested `PyResult` mapping. The existing binding coverage now exercises a missing SDF through the issued opaque route handle.
- Split native-wheel builder publication, source-closure, and tree-digest self-test fixtures into
  a private helper module. The wheel-closure fixture now delegates ZIP extraction through the
  builder's existing validated member-extraction contract instead of using `extractall`.

- Restored the presentation-creation gesture test mutability contract: canonical-arrow and
  equilibrium commit tests now mutably own their sessions, while the below-span refusal test no
  longer declares an unused mutable session.
- Recorded the durable design decision in [HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md): correct the
  responsible design, make narrow intended boundaries explicit, invest in adaptability when it
  pays for itself, and avoid refinement that does not improve correctness or user value.
- Recorded the periodic `all_test.sh` drift-review practice in [HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md).
  Full runs now explicitly complement focused validation by surfacing fragile or overcomplicated
  permanent tests.
- Kept the canvas projection facade small while preserving its deliberate public item imports
  through `__all__`, so the extracted builder remains modular without silently dropping callers.
- Split the Rust CLI command schema into `cli/commands.rs`; `cli/mod.rs` now remains a compact
  parsing, dispatch, error, and test owner without deleting command behavior or Haworth routing.
- Kept current-tab Open tests self-contained after their size-driven split. Each durable
  behavior now owns its inline Qt setup rather than importing another test module or retaining
  shared fixture infrastructure.
- Restored the generic interchange source-admission `Result` return after a split left its
  descriptor-input `match` outside the function boundary. File, stdin, and request-text
  admission again use the one closed source-policy result.
- Documented the historical native-wheel lifecycle in the local-build authority,
  [LOCAL_BUILD.md](LOCAL_BUILD.md). The current contract retains bounded staging,
  20 GiB admission, locking, and signal cleanup while the runnable local build
  replaces publication and installation.
- Closed the native-build interruption disk leak. `build.sh native` now records only its exact
  invocation staging and hidden publication paths, then uses one idempotent cleanup lifecycle for
  success, failure, `TERM`, `INT`, and `HUP`. It removes the managed archive cache and retired or
  unpublished payloads without following untrusted paths, preserves the valid `current` pointer,
  and releases the acquisition-owned lock last. The wrapper E2E now injects all three signals on
  both sides of atomic publication and proves bounded postconditions.
- Hardened native build lifecycle ownership and publication. `build.sh native` now assigns each
  physical lock directory an acquisition-unique owner token, reclaims only demonstrably dead owners,
  and never removes a replacement lock. It publishes a fully validated immutable payload through one
  atomic `output_native_wheel/current` symlink replacement, preserving a valid prior or new artifact
  across interruption while retaining the 20 GiB admission gate and staged cache cleanup.
- Reworked `build.sh native` into a transactional single-publication flow. Native compilation now
  occurs only in `build/native-staging/`; after validating the builder artifact, receipt, and
  matching engine bundle, the wrapper atomically replaces lightweight
  `output_native_wheel/current/`. Native preflight removes legacy `native-*` output plus build-owned
  staging and managed source archives before consuming space. Explicit sealed-input and source-
  archive roots are never removed, and a failed build retains the prior `current/` publication.
- Added a pre-compiler 20 GiB checkout admission gate to every non-help `build.sh` target. Native
  targets reclaim their owned stale state before measurement; an over-budget checkout prints its
  `du -sh` size and an actionable remediation without starting Cargo or the native builder.
- Added a repository disk-budget pytest that measures the full checkout with
  ``du -sk`` and fails above 20 GiB, preventing generated build outputs from
  silently exhausting the developer volume.
- Repaired Qt test-module split hygiene by keeping drawing-only CDML inputs
  with their drawing coverage, importing the shared editable input once, and
  normalizing the native-editing test indentation.
- Closed P0.1 implicit direct-bond endpoint classification. Draw Bond now bridges only a uniquely resolved existing atom source identity; empty space remains the sole new-carbon route, while ambiguous, missing, or invalid identity facts refuse before admission and cannot create a carbon atom.
- Corrected PyO3 reaction-enum identity so the Python binding exposes the canonical Rust reaction role enum rather than a parallel value representation.
- Regenerated the checked-in SMARTS protocol schema from its owning generator after the redacted DTO contract changed, preventing hand-maintained schema drift.
- Added the M4b compiler-derived Rustdoc public-surface oracle. It verifies the approved typed `ferrum-chemistry` facade, including doc-hidden, macro, include, and reexport paths, so raw adapter authority cannot become public unnoticed.
- Added release-boundary coverage proving an in-radius direct-bond projection atom with no usable source identity cancels before either endpoint bridge or Rust admission.
- Repaired repository-hygiene source splits: script-local native-wheel helpers are recognized as local imports, extracted Qt modules retain their required imports and mixin aliases, and shared CDML test utilities remain owned by the ordinary-open module.
- Rotated the changelog into `docs/CHANGELOG-2026-08c.md`, retaining the two newest complete day blocks and compacting retained Markdown entries without deleting any history.<br>- Made implicit-carbon start selection one private, bounded projection fallback for Attach Cyclohexane Ring and Draw Bond. Rendered hits still win; only one nonempty projection `atom.id` within six device pixels is eligible, while ties and farther points refuse. Direct-bond endpoints use the same bounded fallback: only a unique valid source identity becomes an existing endpoint; ambiguous or invalid nearby identities cancel without creating a new atom.
- Corrected implicit-carbon start identity handling: shared picker geometry now carries validated projection object and source identifiers. C6 continues to use its object ID, while Draw Bond receives the source ID required by its Qt-to-Rust bridge; incomplete identity facts fail closed.
- Corrected the Qt Draw Bond action's accessibility metadata to use QAction's text, tooltip, status-tip, and What's This surfaces rather than QWidget-only accessibility methods. Added a native-session-free action construction test.
- Narrowed Qt Draw Bond receipt redemption recovery to closed typed native commit-error categories. Unexpected commit, projection-install, and programming exceptions now propagate instead of being shown as ordinary drawing refusals.
- Replaced Qt Draw Bond's pre-admission preview/commit pair with the Rust-issued opaque `DirectBondAdmissionV1` receipt. Qt now draws only an admitted copied overlay and redeems that exact receipt once; direct normal-bond chemistry, identifiers, and commit authority remain in Rust.
- Corrected direct-bond candidate admission exceptions to expose frozen closed category and recovery values for foreign and stale receipt attempts. Added registered PyO3 coverage for opaque receipts, typed foreign/stale recovery, and legacy `preview_mismatch` conflict reporting; Rust admission tests now prove refused and abandoned candidates do not advance generated IDs or issue provisional tokens before commit.
- Corrected the direct Draw Bond action to use its closed Qt contract regardless of saved Next atom and Next bond preferences. It now creates only normal single carbon bonds, exposes the documented accessible name and description, and cancels an in-progress Rust gesture when Qt loses mouse capture or hides the viewport.
- Extended the direct-bond native refusal vocabulary with typed neutral-capacity and unsupported-chemistry admission categories. The public Rust gesture still represents normal single, double, and triple bonds; the Qt controller selects only the normal-single profile.
- Corrected the Qt contract for `Change Element`: only one current durable selected atom is eligible; invalid selection, cancellation, Escape, and tab changes preserve state without submission. The contract now also specifies the action/dialog labels, keyboard focus route, and screen-reader success or refusal behavior.
- Tightened Change Element acceptance recovery. The installed-wheel E2E now proves that `ferrum_qt.ferrum.main_window` is installed from the supplied Qt wheel and matches its wheel-member SHA-256; it continues to prove the native extension provenance independently. One failed projection installation now has focused Change Element coverage for one successful refresh, exact durable selected-atom recovery, and exactly one Rust revision change. Unexpected action or refresh failures now propagate instead of being presented as an ordinary edit refusal.
- Corrected the isolated Change Element installed-wheel E2E startup and failure cleanup. It now waits for the public local-CDML completion signal before inspecting its native tab, and cancels then drains a pending admission worker before closing the Qt host.
- Corrected the isolated Change Element installed-wheel E2E to require explicit native and Qt wheels, install both with isolated dependency resolution, and verify the loaded root extension digest against the supplied native wheel.
- Corrected Change Element so a Rust-accepted mutation with a failed Qt projection follows the authoritative-refresh recovery path instead of being reported as refused. Its isolated installed-wheel E2E now lives under the package `tests/e2e/` fence, outside fast pytest collection.
- Removed the Qt-local `.cml` suffix gate after a Rust-admitted CML receipt. Rust descriptor admission remains the sole interchange suffix authority while Qt retains CML provenance and CDML Save behavior.
- Accepted Rust-admitted CML files as local imported sources in the Qt tab provenance model. CML now preserves its input identity without becoming a loaded-CDML save baseline; Save and Save As publish authoritative CDML.
- Retired armed Ferrum pointer tools before Qt Undo or Redo advances Rust history, preventing stale checked actions after a revision transition.<br>- Corrected the attached-cyclohexane root E2E to invoke the window-owned Undo and Redo QActions. The check now covers the production history lifecycle that retires C6 state, rather than bypassing it through direct tab calls.
- Hardened the focused direct-bond history test to observe the retired intent and unchecked action at the actual Rust Undo and Redo transition boundary.
- Added passive phase fences to the attached-cyclohexane root E2E menu discovery. The harness now snapshots the shared action, C6 intent, mode, and Qt action trace before traversal, after traversal, after action-group lookup, and after popup close; an unexpected dispatch reports its exact phase and event delta without resetting product state.
- Narrowed the attached-cyclohexane root E2E startup guard to the attach QAction, its presentation mode, and line intent. Checked dock and view preferences remain passive diagnostic facts, rather than incorrectly preventing an otherwise unarmed attach workflow.
- Extended the attached-cyclohexane root E2E's initial unarmed-action precondition with passive state facts. An unexpected checked attach action now records the presentation mode, line intent lifecycle, checked Qt actions, and action-registry identifiers before refusing the run; the test does not alter product state or command dispatch.
- Replaced the attached-cyclohexane root E2E's offscreen popup-row pointer dispatch with the shared visible `QAction` activation contract. The harness now proves menu/action/group identity and opens then closes the real menu before `trigger()` drives Qt's checked-action transition. This isolates the offscreen QPA popup hit-testing limitation without bypassing the product command, mode, picker, bridge, or lifecycle path.
- Latched terminal popup completion in the shared canvas-action handoff. A popup destroyed after its terminal Qt signal now permits exactly its already-authorized queued action; destruction before a terminal signal still cancels the action.
- Replaced recursive popup polling in the shared canvas-action handoff with a window-owned Qt continuation. Deferred tools now observe popup lifetime, action/window destruction, bounded replacement popups, and one watchdog; failures use the ordinary typed-refusal route and cannot escape a Qt slot.
- Deferred each registered canvas-action handoff until a transient Qt popup has fully closed. The capture guard and its exact handler still run together; ordinary no-popup actions remain synchronous. This removes popup teardown's race with newly armed tools for every shared canvas action, including C6.
- Repaired the common pointer-tool popup focus handoff. A viewport `FocusOut` now settles for one Qt turn and retires only the same still-unfocused intent, while the same intent reclaims focus after popup teardown. Genuine focus loss still cancels, and a stale callback cannot retire a replacement tool.
- Added the focused symmetric popup-focus regression: a stale queued focus restoration cannot reclaim the viewport or alter a replacement pointer tool.
- Corrected the attached-cyclohexane root E2E's ownership oracle to match the one approved Qt route: only `line_tools.py` may call the public tab bridge, and only `attached_cyclohexane_tab.py` may call the private session bridge. The E2E now also proves that the menu's shared checkable QAction sets the `attach_cyclohexane` presentation mode before its existing line-tool drag.
- Removed the inert `AttachCyclohexaneMode` drag controller and its synthetic dispatcher mapping. The existing QAction, registry identity, ribbon, and `ATTACH_CYCLOHEXANE_RING` presentation map remain; the production line-tool event filter is the sole C6 pointer and native-commit path. Focused Qt coverage now proves shared QAction and presentation state instead of a controller-only drag simulation.
- Restored the PyO3 reaction-binding split by explicitly resolving its private methods and support
  modules as sibling files, while retaining one public facade and registration surface.

- Hardened native wheel publication against a post-rename pointer race.  If
  `output_native_wheel/current` no longer selects the just-published candidate,
  `build.sh` now fails without overwriting the observed pointer; it preserves
  the prior and candidate payloads for inspection.  The wrapper E2E injects an
  external replacement inside the Python helper's post-replace verification
  window and proves all known payloads remain intact.
- Restored the direct-bond PyO3 binding facade's private sibling support-module registration after its modular split, so the feature build resolves the extracted implementation from the shared `python_binding` parent rather than a nonexistent nested path.

### Developer Tests and Notes
- Scoped the paired-publication Qt-closure mutation fixture to its one build invocation.
  Later successful-build assertions now exercise the ordinary copied build contract regardless
  of test ordering.

- Enriched only the existing blank-canvas Draw Bond E2E history failure with
  non-mutating Rust-owned availability, snapshot, and QAction facts, so a failed
  installed-wheel acceptance run distinguishes native history publication from Qt action state.

- Extended the deterministic native-wheel builder self-test through the stable facade. It now
  exercises wheel-closure auditing, sealed engine-bundle construction and validation, destination
  refusal, and the single JSON artifact-emitter protocol with local fake payloads only.

- Extended deterministic history availability coverage across Rust, PyO3, and Qt. The tests now
  prove branch discard, active-tab-only action state, and temporary pending or busy action gates
  without altering the authoritative Rust history cursor.

- Corrected the blank-canvas Draw Bond dual-wheel E2E baseline. Each disposable public New tab
  now saves and parses as independently blank CDML; the second tab reports Undo and Redo state
  separately before drawing, then establishes its own same-view visual lifecycle baseline.

- Corrected the native-wheel builder CLI staging contract. `--output-root` remains the only
  independently admitted fresh `build/native-staging/native-*` root; the matching
  `--engine-bundle-dir` is now parsed as its child payload and remains subject to the builder's
  strict containment check. The focused helper self-test and `build.sh native` wrapper E2E prove
  this exact handoff without relaxing retired-output rejection or publication cleanup.

- Added a standalone offline dual-wheel Qt E2E for Draw Bond on a blank canvas.  It drives only public QActions, the visible viewport, and the Save As dialog; saves and parses both independent New documents as empty CDML before drawing; proves sticky QAction arming across gestures, native preview and Escape retirement through viewport captures; checks a single undo/redo history transition; parses the resulting temporary CDML for two carbon atoms and one bond; and hashes both installed wheel payloads before exercising the UI.
- Audited and rewired the final combined SMARTS artifact-pair release/acceptance E2E to invoke the explicit
  sealed native-wheel SMARTS proof rather than copying a removed source-test
  harness.  Its one receipt now retains isolated CLI, PyO3, and real Qt
  evidence without shared-artifact mutation or environment-gated test inputs,
  and follows the public Qt open-queue completion signal.
- Moved the sealed live-SMARTS wheel harness out of ordinary pytest into an
  explicit local E2E command.  It now creates and removes its isolated venv,
  verifies the supplied native wheel and derived ABI-5 bundle bytes, exercises
  opaque and typed-refusal FFI contracts, and never mutates published artifacts.
- Extended the isolated blank-canvas Direct Bond wheel proof to byte-verify
  the installed preview and line-tool interaction modules, so its active
  authoring route is proven to originate from the supplied Qt wheel.
