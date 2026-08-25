# Changelog archive

Newer history is in [CHANGELOG.md](CHANGELOG.md).
Earlier history is in [CHANGELOG-2026-08e.md](CHANGELOG-2026-08e.md).

## 2026-08-22

### Fixes and Maintenance

- Added read-only atom oxidation observation through the public
  `document.atom.oxidation.observe.v1` protocol operation, generic PyO3 protocol gateway, and
  local `ferrum document-atom-oxidation-observe --request -` command. The modeless accessible
  Qt `Atom Oxidation State` dialog is source-fenced, presents typed accepted/unavailable/refusal
  results, and never changes the document.

- Made the manual GUI-tour template catalog image self-evident. Its targeted capture now shows
  Ferrum's selected `Reusable oxygen-ring template` chooser alongside the completed placement,
  and `--scene template_catalog` refreshes only that fixed documentation asset.

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

- Added `./capture_gui_screenshots.sh` and [GUI_TOUR.md](GUI_TOUR.md), a manually invoked,
  non-gating ten-scene Ferrum Qt documentation-capture workflow. It stages all verified
  real-window PNGs before publishing them, prefers `easy-screenshot` when available, and
  uses Qt's same-window capture fallback without Screen Recording access. The inspected
  real visible capture pass publishes the managed README embeds.

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
