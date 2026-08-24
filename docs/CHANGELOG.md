# Changelog

Earlier history is in [CHANGELOG-2026-08e.md](CHANGELOG-2026-08e.md).

## 2026-08-23

### Behavior or Interface Changes

- Moved catalog placement onto a document-owned renderer-admitted molecule
  transaction. Catalog entries now resolve only a closed molecule or standalone
  Haworth recipe and preview geometry; the document pending value retains the
  session capability, exact fence, generated-ID reservation, candidate
  observation, renderer proof, and atomic commit authority. The V2 UI facade
  continues to provide its opaque gesture, lease, overlay, and prepared handle.

- Moved direct-bond V3 admission into a document-owned pending transaction.
  Direct semantic endpoints now build the prospective immutable document state
  directly, renderer admission binds that state to the session-minted pending
  identity, and commit redeems the proof immediately before the atomic history
  append. The interaction facade now retains only the opaque pending handle and
  renderer-issued overlay; it no longer constructs candidate CDML, a temporary
  session, a render plan, or a duplicate receipt.

- Moved renderer-admission proof ownership into `ferrum-document` pending visual transactions. Each candidate now binds a document-session issuer and monotonically increasing pending sequence, so equal-content preparations cannot exchange a renderer receipt. Document-render gesture routes retain opaque prepared document handles and no longer construct, verify, restore, or access renderer admission values.

- Moved primitive atom, bond, bonded-atom, bracket, and wavy preparations to
  document-retained renderer admission before candidate visibility or commit.
  Direct and standalone Haworth previews now use that same document-owned
  admission, while Python bindings and Qt replay immutable renderer plans and
  refuse excluded-root candidates before mutation. Attached cyclohexane
  previews likewise replay the admitted plan and verify the exact prospective
  observation before their one-use commit.

- Moved molecule, regular-ring, and interchange import workflows to
  document-owned opaque admitted pendings that Python and Qt redeem; ring
  previews replay renderer-issued plans. Structural deletion now admits its
  exact post-mutation state before issuing the selection token or history
  append, with typed presentation recovery for unrenderable candidates.

### Fixes and Maintenance

- Replaced the obsolete renderer-admission interim plan with a document-owned
  route ledger. The plan now records each migrated pending transaction, names
  the remaining visual mutation families as active work, and clarifies the
  direct-bond and Telex contracts around their actual ownership boundaries.

- Removed the retired complete-render-admission profile/model topology after
  document-owned pending renderer admission replaced its last product route.
  The remaining generated Telex scalar table is now a focused root
  `ferrum-render-contract` module used directly by the renderer, with its
  resource identity and scalar-validation contract preserved.

- Hardened local GUI-launcher creation with an atomic owner-only file contract,
  replacing permissive create-then-chmod behavior. Split presentation-stack
  implementation behind its public facade into semantic construction and
  invariant handling, private wire conversion, and focused behavior tests.

- Moved reaction creation, reaction membership lifecycle, and complete reaction
  translation onto one document-owned renderer-admitted complete-CDML pending
  transaction. Reaction gestures retain their request, selection, membership,
  pointer, and recovery semantics, while the document session now privately
  parses, admits, verifies, and atomically appends each candidate without
  reaction-local renderer proofs, plans, candidate digests, temporary document
  sessions, or the retired detached complete-CDML preflight API.

- Migrated presentation vector/path and curved terminal/equilibrium arrow
  authoring onto the renderer-issued pending receipt. Their prepared handles
  now admit and later verify the immutable document candidate observation,
  preserving retry after document refusal while removing route-local complete
  render contracts, candidate reloads, plan composition, and exclusion scans.

- Replaced the detached complete-document CDML preflight bridge for explicit
  hydrogen materialization and compact-group placement/materialization with a
  renderer-issued opaque receipt bound to an immutable complete candidate.
  The shared pending holder verifies that exact candidate before document
  redemption and restores it after a non-consuming refusal; later visual
  mutation families retain their route-specific preflight receipts pending
  their dedicated migration.

- Hardened same-fence SMARTS publication so stale raw queries refuse execution
  until the renderer plan is republished. Arrow Properties tests now assert
  public semantic and selection behavior, and Electron, Retro, and Curved
  Normal terminal families document intentional renderer visual equivalence
  while retaining behavior coverage.

- Moved normal, equilibrium, curved-terminal, and curved-equilibrium arrow
  previews onto one pure renderer plan lowerer. Document gestures retain only
  semantic authoring/fence state, PyO3 delivers frozen renderer plans, and Qt
  replays that plan without arrow-specific axes, head polygons, or cubics.

- Clarified renderer ownership in the architecture, authoring contract, active
  admission plan, and Rust API comments: semantic arrows retain authored
  facts, while renderer-issued plans supply visual geometry. The generic
  receipt migration and renderer-owned preview lowerer remain explicit pending
  P0 work.

- Completed the semantic-arrow rendering repair: projections reject only
  collapsed spans while `ferrum-render` derives and scales short-arrow
  geometry. Qt now replays renderer-issued presentation plans as its sole
  visual scene source, Arrow Properties reads semantic arrow facts, and
  same-fence Python rendering publication activates SMARTS queries from the
  exact accepted observation.

- Retargeted arrow-property and curved-arrow Python binding tests to immutable
  semantic projection policies and same-fence renderer-plan topology, removing
  assertions against retired document-side display geometry.

- Updated the arrow-authoring E2E to assert the fenced renderer presentation plan's
  curved shaft and terminal-head operations instead of retired document-side display geometry.

- Made arrow projection semantic-only: its lower DTO now retains authored
  points, family, head policy, and stroke while refusing collapsed source
  spans. `ferrum-render` derives normal, equilibrium, and curved display
  geometry, including interactive terminal-arrow previews; document and PyO3
  consumers now expose semantic policy or replay renderer-issued plans. The
  retired document-private complete-render profile and its redundant model
  dependencies are removed from `ferrum-document`.

- Repaired Qt renderer-plan replay to accept the current document-owned render
  observation schema and retain its revision/digest fence. Presentation target
  validation now has one lower shared module, so renderer-plan plus/text items
  import without a circular presentation-facade initialization path.

- Routed Qt presentation painting through the frozen renderer-owned
  `PresentationRenderPlanV1` beside the existing complete render observation.
  The canvas replays validated path, ellipse, stroke, fill, plus, and text
  operations without rebuilding presentation geometry from semantic DTOs.

- Made renderer-owned presentation plans publish the fixed
  `ferrum-presentation-render-plan-v1` schema through Rust and frozen PyO3
  delivery. Callers cannot supply or alter the plan grammar, so Qt can reject
  plans outside the exact renderer-issued contract before scene construction.

- Retargeted Python render observations, local document ingress, and live
  SMARTS publication to document-owned session rendering. Added frozen Python
  delivery of fenced renderer-owned presentation plans with direct-root bounds
  and issued vector, plus, and text operations.

- Inverted the document rendering dependency: `ferrum-render` now resolves only
  immutable lower projection DTOs and emits plans/bytes, while
  `ferrum-document::rendering` owns session provenance, complete-plan policy,
  selected-root SVG identity, native artifact preparation, and publication.
  Interaction and API callers now acquire document-owned render observations.

- Moved the immutable outer document projection, its snapshot provenance, and general projection issues into `ferrum-document-projection`. The document crate now adapts typed CDML privately and the pure renderer depiction profile consumes the lower aggregate directly; aggregate construction refuses presentation provenance from another snapshot.

- Corrected renderer presentation-plan bounds for the shared finite scalar API, imported pure arrow vector types directly from `ferrum-document-projection`, and kept the renderer refusal test at the reachable lower-DTO boundary.

- Added a pure renderer-owned presentation-stack plan that preserves immutable
  targets and source order while issuing vector or verified text operations
  with renderer-calculated finite painted bounds.

- Moved immutable paper-layout and complete presentation-stack values, including
  payloads and bracket pairs, to canonical `ferrum-document-projection`
  ownership. `ferrum-document` retains typed-CDML projection adapters that
  resolve retained facts and emit lower presentation issue values, while its
  facade re-exports the exact lower DTOs.

- Restored the lower immutable presentation-stack constructor as the sole
  admission boundary for round-bracket root and pair consistency.

- Closed immutable presentation-stack construction through public lower-crate
  refusals for duplicate identities, invalid paths, root-kind mismatches, and
  round-bracket disagreement. The document facade now re-exports the exact
  bracket-style and stack-error types; redundant JSON-mutation and duplicate
  save/reopen mechanics no longer obscure those durable contracts.

- Restored the document-side typed-CDML arrow projection adapter after immutable
  arrow values moved into `ferrum-document-projection`. The document crate again
  resolves retained facts and emits closed projection issues while its facade
  re-exports the exact lower DTO types.

- Moved immutable paper-layout projection values into
  `ferrum-document-projection`. `ferrum-document` now owns only the typed-CDML
  adapter, catalog/default resolution, and paper mutation intent while retaining
  facade re-exports.

- Rotated complete 2026-08-21 history into
  [CHANGELOG-2026-08e.md](CHANGELOG-2026-08e.md), retaining the two newest
  date blocks in this active changelog and one unique home for every date.

- Made snapshot-derived projection and presentation identity failures propagate
  as typed refusals. Direct-bond now distinguishes missing endpoints from
  malformed projection facts.

- Corrected the active renderer-admission plan to distinguish the completed
  first immutable projection DTO extraction and document facade re-exports from
  the remaining paper/presentation/issue/aggregate extraction, renderer import
  inversion, wrapper relocation, and pure-plan proof work.

- Hardened lower molecule-projection construction to validate child order and
  identity. Projection tests retain behavioral contracts without enforcing a
  brittle JSON wire shape; renderer dependency inversion remains in progress.

- Completed the immutable molecule-projection DTO extraction: the lower crate
  now owns atom, mark, bond, endpoint, Haworth, and molecule values; typed-CDML
  traversal and diagnostics remain document-owned, and public document types
  remain re-exports.

- Added `ferrum-document-projection` as the canonical owner of immutable
  presentation-style DTOs. `ferrum-document` supplies migration re-exports
  while session observation and traversal remain document-owned.

- Moved immutable identity, finite geometry, and compact-group projection DTOs
  into canonical `ferrum-document-projection` ownership. Typed-CDML traversal
  and adaptation remain in `ferrum-document`, and facade paths remain re-exported.

- Hardened local `build.sh` promotion as a contained rollback transaction.
  Disposable candidates now remain below `build/`; failed candidate promotion
  or final receipt validation restores the prior runtime and launchers, removes
  transient candidate/recovery data after successful rollback, and retains a
  named recovery location in the error when restoration cannot complete.

- Sealed local build promotion around a V2 runtime receipt that binds the
  canonical Qt launcher source. Each candidate local program is validated
  before promotion, failures remove only disposable candidate/staging data,
  and the previous sealed runtime remains available.

- Made Qt command help identify the public local command as `ferrum-qt`,
  including when the launcher executes the package parser through `__main__`.

- Made the public molecule-import retirement interval an explicit close-lifecycle
  boundary. A close retry now remains nonblocking until Qt confirms the exact
  worker destruction acknowledgement, so parent disposal cannot suppress
  `document_import_retired`; focused public Qt coverage exercises the
  finished-before-destroyed close attempt.

- Added a public terminal document-import retirement signal after Qt confirms
  queued worker destruction, releases the import-retirement owner, and restores
  action availability. Kept it separate from the earlier success-only
  installation receipt, and reduced the receipt tests to one distinct SDF
  accessibility mapping plus the normal close-cancellation lifecycle.

- Routed application-window shutdown through the existing molecule-import
  cancellation lifecycle before tab disposal. A close request now invalidates
  live import delivery through a nonblocking status notification, lets the
  worker reach its terminal cleanup boundary, and accepts the subsequent clean
  close without retaining a Qt worker. Corrected the public receipt test to
  assert that this first, cancellation-owning close request is rejected.

- Split the complete-render admission contract tests into focused molecule,
  visual-root, proof-identity, and Telex/text modules while retaining the
  existing contract assertions and shared typed builders.

- Kept the local-runtime receipt command wrapper aligned with its complete
  staged-extension validation gate, removing its stale unused low-level import.

- Kept the `ferrum-document` strict test build warning-free by placing its
  hydrogen candidate-ID helper before the module-local tests and using idiomatic
  boolean assertions in arrow history coverage.

- Clarified the compact-group placement candidate boundary with one typed
  generated-identity and authored-state input, and removed redundant borrows
  from hydrogen-bearing selection.

- Moved complete-render compact-group catalog identity, attachment-site semantics,
  and atom-symbol grammar into `ferrum-document-model`. Accepted profiles now bind
  a closed catalog key instead of a Telex-valid label string; lower admission and
  renderer lowerers share the exact catalog and uppercase-plus-zero-to-two-lowercase
  atom-symbol predicate.

- Strengthened V2 complete-render molecule admission so accepted profiles retain
  visible atom state, persistent mark geometry, compact-group label/attachment/
  orientation facts, and exact supported Haworth-front bond variants. The lower
  contract now refuses hidden atoms, Wavy/unsupported bonds, coincident endpoints,
  and unsupported compact-group exterior topology before a candidate can be
  admitted; profile identity binds the new renderer-relevant facts.

- Closed the remaining molecule render-admission gaps: only exact single wedge
  and single hashed-wedge variants are admitted; resolved positive bond-lane
  spacing is retained in the immutable profile identity; and compact labels use
  the Telex scalar-capability contract before a candidate proof is issued.

- Repaired closed Telex glyph-capability verification so its `OnceLock` returns
  owned results and its scalar traversal skips non-scalar surrogate values. The
  packaged asset/digest check and fail-closed cached verification remain intact.

- Made `./build.sh` prove that a fresh isolated Python import resolves the
  exact staged `ferrum_chem` extension and its current `DocumentSession`
  history surface before declaring the local CLI and Qt runtime ready.

- Made public document-installation receipts report the exact installed SDF
  record count in their accessible summary. Added focused public Qt coverage
  for every successful import route, batch-count semantics, and
  cancellation/disposal receipt suppression.

### Developer Tests and Notes

- Added [renderer_admission_dependency_inversion.md](active_plans/active/renderer_admission_dependency_inversion.md),
  the active plan for moving complete-render admission below document mutation
  through a typed immutable projection and renderer-owned opaque receipt.

- Revised [docs/active_plans/active/compact_group_authoring_v1.md](active_plans/active/compact_group_authoring_v1.md)
  to block public compact-group delivery on the unimplemented document-owned
  complete-render admission profile. The active plan now records the
  accepted-only DTO, typed shared classifier, explicit nonvisual-root policy,
  raw-candidate-route retirement, permanent no-bypass contracts, and the
  `Me`/`NO2` differential oracle as one-time evidence.

- Added [docs/active_plans/active/compact_group_authoring_v1.md](active_plans/active/compact_group_authoring_v1.md), the evidence-based forward plan for Rust-owned compact known-group authoring. It records the typed group, candidate-capacity, `Me`/`NO2` experiment, public-operation, Qt, and validation gates while keeping legacy compatibility and publishing outside this slice.

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
