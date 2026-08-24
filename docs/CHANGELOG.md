# Changelog

Earlier history is in [CHANGELOG-2026-08e.md](CHANGELOG-2026-08e.md).

## 2026-08-24

### Additions and New Features

- Added the M4 atom-oxidation V1 evidence receipt. The bounded read-only HCNO
  route now has public Rust corpus coverage and a real staged-runtime Qt E2E;
  M4 remains incomplete while the chemistry-operation catalog continues.

### Behavior or Interface Changes

- Completed the M0 generic route-authority migration for curved terminal and
  equilibrium arrows, presentation paths and vectors, and explicit-hydrogen
  materialization. Route code now resolves an opaque generic request; document
  request preparation and one-use commit are the sole mutation authority.
  Generic post-commit outcomes expose created presentation-root facts for Qt
  selection without route-specific prepared or committed receipts. Compact
  group records, projection, and rendering remain supported, while its public
  placement/materialization action surface is removed before an authorized M1
  product route exists.

- Canonicalized document atomic-operation execution around
  `DocumentSession::apply_document_operation_v1`. It is now the sole public
  closed-operation API and always executes through generic request,
  preparation, and commit. Removed `submit` and
  `execute_session_operation_transition_v1` spellings across document, API,
  PyO3, protocol, and Qt instead of preserving alternate public routes.
  `DocumentBondOrderV1` is the sole molecule-insertion bond-order type; the
  `MoleculeInsertionBondOrderV1` compatibility alias is removed.

- Completed the M0 generic reaction authority migration. Reaction creation,
  lifecycle, and translation now resolve opaque generic transition requests;
  generic preparation and generic commit remain the only post-resolution
  authority. Route-specific prepared/committed receipts, prepare/commit
  exports, PyO3 receipt/direct-create surfaces, and raw complete-CDML
  authority are retired without forwarding shims. Successful generic outcomes
  publish reaction IDs only after accepted commit, and generic prepared debug
  output is redacted to lifecycle state. M0 remains incomplete pending
  insertion and materialization authority retirement and exit validation.

- Migrated M0 direct-bond redemption to the generic prepared-transition
  boundary. The V3 pointer, keyboard, and protocol routes now use
  `PreparedSessionTransitionV1` and the sole generic commit authority; the
  route-specific V3 admission/commit holders are retired. The rebuilt runtime
  passed binding (`14/14`) and pointer Qt (`7/7`) focused gates. M0 remains
  incomplete pending the latest aggregate full-suite and external-boundary
  evidence.

- Completed the M0 catalog semantic migration. Catalog placement now resolves
  closed key/anchor intent into a generic document operation and a durable
  post-commit root outcome with `TransitionAuthorizationV1::None`; lowering,
  renderer admission, deferred effects, and commit authority are document
  owned. Retired V1/V2 catalog document-render/API authority and the PyO3 V2
  handle are gone, while the V2 UI lease remains local paint scheduling only.
  Focused Cargo checks and catalog/document/protocol tests passed. M0 remains
  incomplete: an attached-cyclohexane test needs its mutable cancel migration,
  and the live SMARTS-query test still calls the retired raw complete-CDML
  commit surface; neither blocker restores a catalog legacy route.

- Defined `document.molecule.report.v1` as a snapshot-based, read-only
  multi-root receipt: selected root records follow source order, findings follow
  deterministic report-category order, and the aggregate is complete or omitted.

### Fixes and Maintenance

- Routed ordinary Qt atom and selected-atom bond authoring through closed
  `DocumentOperationV1` factories and the current-document operation gateway.
  `DocumentSession::apply_document_operation_v1` now issues their ephemeral
  session-local authoring capability before generic preparation and commit;
  gesture routes retain their distinct receipt-based admission. Qt restores
  selection from durable generic outcomes.

- Aligned the atom-oxidation E2E script with its runner: it is invoked through
  the sourced Python environment, so its misleading direct-execution shebang
  is removed instead of relying on an executable file mode.

- Routed Rust-owned Qt geometry repair through the captured document revision,
  so the native atomic-operation contract applies repairs instead of refusing
  them as revision-less requests.

- Routed ordinary Qt document mutations through the document tab's current
  observation gateway; scale and geometry repair retain their explicit
  captured-revision contracts for dialog-safe compare-and-apply behavior.

- Retired the external authored-document and opaque-namespace CDML corpus files. Their compact
  semantic XML now lives inline in `typed_tests.rs`, so documentation and local-build examples no
  longer present test-only paths as runnable inputs.

- Completed the final generic-authority cleanup: retired migration-history
  fixture, source-name, and inventory checks; renderer-issued precommit
  overlays replace raw plans; and generic primitive atom/bond and Haworth
  operations retire route-specific public receipts. Restored the supported
  Wavy/bracket semantic binding methods after an unintended broad removal, and
  removed dead no-payload reaction-preview API/test helpers. M0 remains open
  pending fresh aggregate exit evidence.

- Completed the remaining M0 generic-operation authority cleanup. `CreateAtomV1`,
  `CreateBondV1`, and `CreateHaworthMoleculeV1` now pass through the same
  request, prepare, and commit lifecycle as visual routes. Attached
  cyclohexane, direct-bond, and Haworth previews paint only the renderer-issued
  identifier-free `DocumentPrecommitOverlayV1`; public route-specific prepared
  receipts are retired. Wavy and bracket binding methods retain their existing
  supported semantics.

- Removed migration-history-only fixture/source-name/inventory checks and an
  unreferenced corpus input. Permanent evidence remains inline, deterministic,
  and focused on supported behavior rather than private spellings or fixture
  catalogs. M0 remains open pending fresh aggregate exit evidence.

- Removed route-specific Haworth prepared receipts and preview-plan exposure.
  Standalone and direct-glycosidic Haworth authoring now paint the typed generic
  precommit overlay before generic commit; compact inline binding and visible Qt
  tests exercise the supported transition boundary.

- Removed retired visual-route lifecycle wrappers and their external-Cargo and
  name-absence fixture tests. Durable inline document, renderer, API/PyO3,
  Qt-source, protocol, and E2E behavior evidence remains; the removed tests
  enforced migration history rather than supported product behavior.

- Simplified asynchronous SDF import around the sole semantic completion
  contract, `document_installation_completed`. Removed the internal
  `document_import_retired` observer/state; worker cancellation and teardown
  remain internal through the existing import intents and `deleteLater` path,
  including application shutdown.

- Consolidated Cargo work-area ownership below `build/`. `build.sh` now owns
  disposable compiler work in `build/.cargo-target/` while retaining only the
  runnable `build/bin/` and `build/runtime/` products, and `check_rust.sh`
  owns and cleans `build/.cargo-check-target/`. Package-local Rust and nested
  PyO3 target directories are retired. The cleanup lifecycle check moved from
  fast pytest to the E2E tier, and the SDF E2E is registered; both exercise
  durable runtime behavior rather than a machine-dependent checkout-size cap.

- Removed the historical external Cargo compile-fail fixture catalogs and
  orphaned reaction probe artifacts. They enforced absence of spellings rather
  than a lasting semantic contract, so no permanent name-absence test replaces
  them. Generic-operation tests remain the durable authority evidence.

- Removed the remaining molecule-report external-consumer Cargo harness and
  its empty fixture directories. The harness generated a temporary crate only
  to enumerate private imports, which was migration-history enforcement rather
  than a supported product behavior. Inline Rust, API/PyO3, Qt, and E2E tests
  remain the lasting behavior evidence.

- Removed the remaining synthetic external-consumer Cargo fixtures and their
  source-name/compile-fail harnesses from chemistry and document tests. The
  unreferenced keyboard CDML input is also removed. The documented CDML
  preservation corpus remains because decoder behavior depends on loading
  those real format inputs.

- Replaced the stale Qt direct-bond preview adapter, which incorrectly expected
  a retired complete render-plan field, with a dedicated identifier-free
  primitive projector. The UI now replays the closed Rust `DirectBondOverlayV3`
  line/path contract in source order and rejects malformed DTOs explicitly.
  The Qt gesture state keeps only frozen press evidence and creates a fresh
  move-only native admission token for each pointer sample, so previews and
  the final release coordinate retain one-consumption semantics.

- Completed the bounded M4 atom-oxidation V1 evidence gate. The generic
  executor semantic corpus remains distinct from the named CLI protocol proof;
  the real Qt workflow now proves source-fenced historical status, source-tab-
  only rerun, and source-tab retirement without a timing race or test-only
  frontend. The shared detached-snapshot admission retains caller revision and
  verified digest as source provenance while its request-local session begins
  at revision zero, preventing a valid rerun from being rejected as stale.

### Decisions and Failures

- Approved the M0 direct-bond post-audit corrective amendment. The generic
  transition boundary becomes the only public authorization validation and
  redemption authority; capability inspection/claim APIs become
  document-private, V3 consumes a non-cloneable gesture, and the copied
  precommit overlay becomes an identifier-free paint value. Generic redemption
  settles its private claim before the infallible final history/effect moves.
  The correction retains no legacy compatibility or direct-bond-specific
  commit path. The generic redemption migration and focused rebuilt-runtime
  checks are complete; M0 still requires aggregate and external-boundary
  evidence.

- Approved the M0 admitted insertion and materialization retirement boundary.
  Molecule and interchange insertion move to closed semantic generic-operation
  requests with post-commit durable outcomes; pending admitted values,
  route-specific prepare/commit methods, and pre-commit Python/API accessors
  retire without compatibility wrappers. Explicit-hydrogen materialization
  follows the same generic boundary. Compact-group materialization remains
  internal-only M0 cleanup, with no public operation, protocol, CLI, PyO3, or
  Qt surface before M1. Implementation and validation remain pending.

- Recorded the approved M0 generic transition-authorization amendment. The
  closed `TransitionAuthorizationV1` requires `None` for existing operations
  and an opaque authoring capability for `CreateDirectBondV1`; generic
  preparation, commit, retirement, and retry own the capability lifecycle.
  Every generic caller migrates through the source-breaking signature without
  forwarders, while no public direct-bond pending or decomposition surface is
  added. M0 remains incomplete pending implementation and required evidence.

- Recorded the approved M0 direct-bond cross-crate and precommit-preview
  amendments. Direct bonds now have a generic `CreateDirectBondV1` request and
  generic post-commit outcome boundary, with generic commit as the sole
  redemption authority. The copied generic presentation may carry only an
  identifier-free renderer-owned precommit overlay. This approval record
  captured the then-pending preview repair; later entries record its
  implementation receipt. M0 remains incomplete pending the remaining M0
  evidence.

- Recorded the approved M0 transition-presentation metadata boundary. A
  document-owned copied immutable DTO has a Retired-only extraction refusal;
  commit authority remains opaque; direct-bond results remain route-private;
  and catalog preview leases remain UI-local. Raw plan, proof, and candidate
  authority remain unavailable. M0 remains incomplete.

- Recorded the completed M0 complete-CDML reaction migration tranche under the
  approved Option A boundary. `CreateReactionV1`,
  `ReplaceReactionMembersV1`, and `DeleteReactionV1` are document-owned
  semantics; translation uses existing `TransformTopLevelRoots`; private CDML
  lowering and generic `PreparedSessionTransitionV1` replace the retired raw
  complete-CDML adapter. M0 remains incomplete because the checkout has
  external fixture source but no Cargo fixture manifest or runner for the
  required positive/negative process-status evidence.

- Approved the M0 renderer-acceptance visibility amendment. Portable candidate,
  refusal, and presentation vocabulary remains in `ferrum-render-contract`,
  while `ferrum-render` owns and privately constructs the opaque accepted
  value. Document-private retention plus pure re-admission and equality now
  proves commit eligibility; hidden constructors and compatibility forwarders
  are rejected. M0 remains incomplete.

- Approved and recorded the M0 complete-render-admission V1 architecture. The
  generic `PreparedSessionTransitionV1` remains the only public commit
  capability; the empty nonvisual allowlist, closed refusal vocabulary, staged
  retirement without forwarding shims, and minimal permanent evidence now
  govern implementation. M0 remains incomplete, so compact-group protocol,
  CLI, PyO3, and Qt delivery remain blocked.

- Selected the incomplete M4 `document.compact-group.materialize.v1` decision:
  one fenced direct-root group materializes through existing document-owned
  prepare/commit ownership, generic CLI transport, Rust-issued eligibility,
  typed recovery, and a returned replacement focus. Public API/usage docs,
  live PyO3 registration, and the thin Qt action remain deferred pending the
  complete-render-admission stabilization gate and concrete implementation.

- Selected the bounded read-only HCNO V1 closeout for the existing
  `document.atom.oxidation.observe.v1` M4 operation. It retains the generic
  PyO3 executor, named local CLI route, and modeless Qt workflow. The bounded
  sub-slice is complete; M4 remains incomplete while its catalog continues.

- Corrected report documentation to distinguish delivery revision provenance
  from report behavior, ordinary typed failures from response-budget structured
  recovery, and the generic `protocol run` route from a named local report CLI
  command. Recorded completed local validation: `PYTHONDONTWRITEBYTECODE=1
  ./all_test.sh` passed 7,492 hygiene tests, launcher/CLI/Qt E2Es, 290 native
  Python tests, and 414 Qt tests with 1 skip.

### Developer Tests and Notes

- Corrected active M0 and parity-plan records to describe the implemented
  generic `PreparedSessionTransitionV1` lifecycle, preserve M1 as the earliest
  public compact-group surface, and label superseded receipt evidence as
  historical. M0 remains open for its remaining roadmap work and final exit
  evidence.

- Completed source and local-runtime validation for the M0 generic-route work:
  the `ferrum-api` Cargo suite passed 94 unit tests plus its integration
  targets, the local application rebuilt, and the generic visual routes cover
  terminal, equilibrium, and straight arrows; paths; vectors; plus; and
  explicit-hydrogen materialization. M0 remains open for its remaining roadmap
  work and final exit evidence.

- Removed the fragile fast SDF import pytest and stale observer-only
  installation test module. The remaining receipt-based import evidence is one
  real `tests/e2e/e2e_sdf_import.py` flow: it writes inline SDF data to a
  temporary path, calls `start_sdf_import(path)`, and verifies committed
  revision, record count, and source-order names without mocks, polling,
  sleeps, or tunable waits. The repaired staged E2E passed and emitted
  `{"schema": "ferrum-sdf-import-e2e-v1", "status": "ok"}`.

- Corrected the permanent reaction-composer Qt test to drive the style-defined
  visible checkbox indicator and accessible visible `Create Reaction` control,
  replacing internal signal/state manipulation with real UI input. Its inline
  CDML/helper setup needs no committed fixture or mock; the staged native
  runtime test passed (`1 passed`).

- Strengthened the local-runtime receipt staged-extension semantic probe. It
  loads a compact inline canonical `urn:ferrum:cdml` document before certifying
  success and rejects receipts without `canonical_cdml_loads: true`, retaining
  strict Ferrum namespace policy without a global-extension fallback.
  `tests/test_local_runtime_receipt.py` passed 11 tests; this is not a full
  local-build or aggregate-suite receipt.

- Recorded the then-current generic reaction authority evidence: 19 document
  admitted-transition tests and 16 renderer reaction tests passed; `cargo
  check -p ferrum-api --features python-binding` and `reaction_opaque_surface`
  passed; and the now-retired external Cargo boundary fixture passed one
  generic reaction consumer flow. Qt route code passed pyflakes and in-memory
  compilation. Later entries supersede this intermediate receipt; it was not a
  Qt runtime or full-suite receipt.

- Recorded the atom-oxidation validation receipt: `cargo test -p ferrum-api
  --test document_atom_oxidation_corpus` passed 2 tests, `cargo fmt --all
  --check` passed, and `PYTHONDONTWRITEBYTECODE=1 ./all_test.sh` passed 7,492
  hygiene checks, local CLI/Qt E2Es including atom oxidation, 290 Python
  binding tests, and 416 Qt tests with 1 skipped. The final receipt passed 292
  Python binding tests and the same hygiene, E2E, and Qt gates.

## 2026-08-23

### Behavior or Interface Changes

- Established the selected-molecule diagnostics boundary around the existing
  read-only `document.molecule.report.v1` route. The Rust report carries fenced
  capacity/composition results and source-ordered findings for text vertices,
  unexpanded group vertices, and explicit zero-order bonds; findings retain
  typed severity, code, recovery, location, and nullable detail.

- Retired generated structure nomenclature as a BKChem/OASA parity obligation.
  Legacy behavior provides authored display names, which Ferrum preserves. A
  future generator now requires its own approved product, corpus, provenance,
  and typed-refusal contract.

- Moved complete-root translation onto the document-owned admitted-transition
  core. Renderer interaction code retains transient gesture, preview, and
  validation work, while `SessionOperationV1::TransformTopLevelRoots` now
  prepares and redeems before atomic history mutation; stale gestures preserve
  CDML, revision, and history.

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

- Consolidated Qt complete-root translation onto the renderer-admitted gesture
  lifecycle and removed the obsolete PyO3 external translation facade. Rust
  retains only its internal transform and snapping primitives; Qt, PyO3, and
  document history now share one admitted prepare-and-commit route.

- Consolidated Qt molecule-plan projection behind the shared overlay module and
  removed duplicate catalog preview lowering. The native frontend now replays
  one renderer-issued complete plan path for catalog and direct authoring
  previews.

- Renamed the remaining DirectBond document/render handoff to
  `DirectBondRendererAdmissionBridgeV1` and tightened it around opaque
  renderer-admitted geometry. The bridge remains document-internal to the
  one-use prepare-and-commit path rather than exposing a raw candidate or plan
  getter to PyO3 or Qt.

- Centralized mutable document history behind renderer-admitted transitions.
  Pending visual operations now retain the renderer-issued proof and immutable
  preview, while history append, generated IDs, stale fences, and retirement
  remain document-owned and atomic.

- Preserved complete renderer batches in DirectBond, C6, and Haworth previews.
  PyO3 and Qt now replay the renderer-owned plan through the shared plan item
  instead of flattening a line/path subset that could reject valid labels or
  masks.

- Closed direct Text and Plus font admission around the bundled Telex face.
  Unbundled authored faces now refuse at document load; persisted drawing
  standards remain source data and renderer admission refuses only the
  unrenderable visual operation without mutation.

- Aligned Python gesture and presentation-stack tests with their public
  contracts: stale fences follow real semantic changes, and impossible deferred
  direct-font rejection paths no longer duplicate the admission policy.

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

- Established a temporary public molecule-import retirement observer for close
  lifecycle experiments. That observer design was later retired in favor of
  the sole semantic `document_installation_completed` receipt; worker teardown
  and action restoration are now internal lifecycle behavior.

- Routed application-window shutdown through molecule-import cancellation
  before tab disposal. The retained design invalidates live import delivery and
  releases workers through the internal import intent and `deleteLater` path;
  it has no public retirement receipt.

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
