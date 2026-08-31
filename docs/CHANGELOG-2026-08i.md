# Changelog archive: 2026-08-24

Entries through 2026-08-24 are archived here. For current changes, see
[CHANGELOG.md](CHANGELOG.md). Earlier history is in
[CHANGELOG-2026-08g.md](CHANGELOG-2026-08g.md).

## 2026-08-24

### Additions and New Features

- Exposed Rust-owned compact-group projection facts through PyO3 as
  `MoleculeProjectionV1.compact_groups`. The Qt render bridge now accepts the
  renderer's native group primitive as the closed selectable target kind
  `compact_group`, retaining both the compact-group and parent-molecule document
  IDs without deriving geometry, chemistry, or materialization input in Python.

- Added the visible Chemistry action `Materialize Selected Compact Group`.
  Qt captures only the selected Rust-issued compact-group address and current
  document fence, invokes the generic live operation, installs the committed
  receipt, and selects its Rust-returned focus atom. Closed native refusals
  leave the document unchanged and receive nonmodal recovery feedback.

- Added the public `document.compact-group.materialize.v1` protocol operation and its one named
  `ferrum document command` forwarding route. The closed receipt returns committed CDML and fence,
  source molecule/group IDs, and the replacement focus; five redacted refusal/recovery classes
  preserve stateless retry behavior without exposing chemistry recipes or renderer candidates.

- Added the generic `document.compact-group.materialize.v1` session transition.
  It fences the source revision and digest, reserves durable replacement IDs only
  for commit, re-admits the complete renderer candidate, preserves exterior-bond
  identity, reports the attachment focus, and participates in ordinary undo/redo.

- Added the closed Rust-owned compact materialization recipe catalog. `Me` and
  `NO2` have immutable atom-role, local-geometry, bond, presentation, and
  attachment-role facts; the remaining persisted compact keys explicitly have
  no materialization recipe.

- Added the private typed-CDML compact-group materialization core for attached `Me` and `NO2`.
  It replaces one direct `<compact-group>` with a recipe-owned ordinary atom/bond candidate,
  preserves the exterior bond identity and presentation, reports the attachment focus, and
  re-admits the candidate through typed parsing. Direct legacy `<group>` records now refuse at
  typed admission rather than being retained or translated.

- Routed native source molecules through the V2 generic insertion request so validated
  tetrahedral and E/Z semantics survive SMILES, molblock, and interchange preparation into the
  local document. Shared admission now accepts persistable explicit-hydrogen centers and maps
  native cis/trans facts to durable E/Z descriptors while refusing unsupported stereo policies.

- Retired legacy peptide-template SMILES parsing and compilation. The supported native-17 Qt/PyO3
  insertion now constructs a typed Ferrum peptide structure plan directly before preparation and
  layout; it does not provide a CLI or protocol operation.

- Migrated native InChI preparation to the shared document-preparation boundary and removed the
  obsolete topology-only complete-graph builder. Every active native source now lowers through the
  same validated semantic request before document commit.

- Added durable E/Z carrier-mark depictions. Ferrum now validates, saves, reopens, reports, and
  renders explicit native E/Z marks through the shared Rust artifact path, keeping chemical
  configuration separate from its editable drawing convention.

- Added one generic molecule-insertion request envelope for optional durable stereo semantics.
  The ordinary `MoleculeInsertionV1` remains topology and depiction only; admitted V2 facts now
  serialize as the owned CDML `stereoSemantics` child in the same session transition.

- Completed the V2 semantic persistence contract. A shared document-owned validator canonicalizes
  and checks graph-relative tetrahedral and E/Z descriptors before generic commit, CDML write, and
  CDML load. The generated protocol schema, CDML specification, snapshot receipt, and Qt report
  now represent the same nullable source-order facts; malformed CDML and stale receipts refuse
  instead of becoming durable depictions.

- Added the detached Rust-owned `PreparedDocumentMoleculeV2` proof-of-construction boundary for
  complete graphs. It resolves aromaticity once, admits only typed P0 stereo semantics, returns
  closed preparation refusals before session mutation or identity allocation, and preserves the
  existing `MoleculeInsertionV1` payload for the next generic persistence package. Focused
  `ferrum-document` and `ferrum-chemistry` Cargo receipts cover detached construction and
  residual-aromaticity refusal.

- Added the M4 atom-oxidation V1 evidence receipt. The bounded read-only HCNO
  route now has public Rust corpus coverage and a real staged-runtime Qt E2E;
  M4 remains incomplete while the chemistry-operation catalog continues.

### Behavior or Interface Changes

- Retired residual peptide-template vocabulary from the Qt peptide import flow and
  public installation receipts. Native preparation now shares the source-neutral
  prepared-molecule binding boundary with other insertion routes without depending
  on the SMILES binding module.


- Made `stereoSemantics` a first-class canonical CDML contract. Typed document
  admission refuses malformed, unknown, and graph-invalid semantic descriptors;
  reopening retains typed tetrahedral and E/Z facts, and snapshot-only molecule
  reports expose them in source order.

- Centralized Qt local-document route discovery in the Rust-owned ingress
  registry. CDML and decoded CD-SVG retain their document semantics, CML/CML2
  opens as a clean new document, and CDXML, CD-SVG names, and compressed
  containers have explicit refusal identities.

- Updated the Python document-operation factories to submit ordinary topology-only molecule
  insertions through the generic `MoleculeInsertionRequestV1` envelope. Existing Python callers
  retain their prior molecule insertion behavior while typed stereo semantics remain explicit.

- `ferrum open --json` now emits the canonical `ferrum-operation-response-v1` or
  `ferrum-operation-error-v1` interchange envelope. An admitted malformed CML source is a
  redacted typed refusal on standard output with exit `1`, no standard-error diagnostic, and no
  published CDML artifact; the human CLI presentation remains one diagnostic with exit `1`.

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

- Registered `document.compact-group.materialize.v1` in the canonical generic
  live-document session dispatcher. The existing PyO3 live bridge now applies
  the owned fenced compact-group transition and returns its committed receipt
  or typed no-change refusal without a compact-specific Python route.

- Allowed molecule-root render groups to pass pointer events to their child
  items. Compact-group, atom, and bond labels now enter the existing canonical
  durable selection state through their own rendered item under the supported
  offscreen Qt runtime.

- Clicking a Rust-issued compact-group label now enters the same canonical durable Qt selection
  state as atoms and bonds. The selected target retains both compact-group and parent-molecule
  document IDs for the existing materialization action; presentation selection remains unchanged.


- Corrected the CDML, usage, Qt, and backend-to-frontend contracts to list the
  persisted `stereoDepictions` child and keep E/Z configuration separate from
  Rust-issued editable carrier-mark depiction facts.

- Corrected the compact-group planning and public-contract records to recognize
  the delivered generic protocol and named CLI route for
  `document.compact-group.materialize.v1`; PyO3 live registration and the Qt
  compact action remain deferred. Added the closed native-17 peptide sequence
  import decision and its outstanding visible-UI E2E gate.


- Hardened the Qt molecule-report stereo receipt boundary so malformed non-string enum values
	are refused without raising a container-membership `TypeError`. Moved the full native E/Z
	carrier-mark persistence and render check out of the fast Python binding lane into a staged
	real-Qt E2E that follows the Rust report and render geometry into the visible canvas projection;
	the check loads generated CDML as its stable window baseline and drains posted native-window
	teardown events before it exits.

- Corrected E/Z carrier-mark admission and rendering for conjugated systems. One directional
  single-bond carrier can now remain associated with multiple distinct double-bond descriptors,
  and the renderer preserves each association as a deterministic separate operation.

- Split the depiction-profile resolver into a focused private module. The public profile and
  resolution receipt remain stable while atom, bond, metrics, and E/Z carrier-mark lowering now
  have one dedicated implementation boundary.

- Repaired the Qt Molecule Report receipt boundary after the typed Rust stereo descriptor
  expansion. The dedicated P0 stereo receipt module validates and displays Rust-issued facts,
  so a valid report now reaches its modeless dialog instead of the invalid-receipt path.

- Repaired the shared Qt local-ingress descriptor state so File/Open and SDF import
  retain their respective Rust-issued descriptor sets and opaque route handles. The
  staged visible SDF-import E2E now checks the initialized handle contract before
  it starts its import worker.

- Corrected the M0 closure receipt to record the fresh aggregate result exactly:
  392 Qt tests passed and 1 skipped. The closure decision now labels all earlier
  incomplete/blocker language as superseded implementation history.

- Completed local-runtime/property-dialog parity evidence. `source_me.sh` now
  selects only the staged native runtime and fails closed when it is unavailable;
  the outside-checkout E2E confirms that selection. The canonical Qt menu-test
  helper drives the visible `Edit Atom Properties...` dialog and proves that an
  atom charge edit is Rust-owned and persists after save/reopen.

- Repaired the visible Qt atom and bond property-dialog parity test so it arms
  real modal acceptance only after the user-visible menu opens, then locates
  the `Charge:` and `Order:` controls through each rendered form layout.

- Repaired the sourced local Python runtime selector: `source_me.sh` now
  derives its checkout from `BASH_SOURCE` and prioritizes the freshly staged
  `build/runtime/python` extension over globally installed `ferrum_chem`.

- Corrected sourced-shell ownership in `source_me.sh`: the local runtime
  selector derives `BASH_SOURCE` inline, so it no longer assigns or unsets a
  caller-owned `REPO_ROOT` variable.

- Closed M0 complete-render admission after fresh `./build.sh` and complete
  `./all_test.sh` evidence: 7,452 hygiene tests, local CLI/E2E checks including
  atom oxidation, 256 Python binding tests, and 392 Qt tests with 1 skipped.
  Reclassified retired CDML profiles as historical inline semantic evidence;
  removed stale active-corpus, E2E-runner, and M10 gate claims.

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

- Completed the Rust CLI format-contract E2E slice in
  [tests/e2e/e2e_ferrum_verb_cli.py](../tests/e2e/e2e_ferrum_verb_cli.py):
  representative closed conversions across SMILES, InChI, molfile, and CDML;
  malformed coordinate-input refusal without an output artifact; and Haworth
  SVG artifact publication plus invalid-SMILES refusal without an artifact.

- Added one real-window Qt regression that opens the visible Atom and Bond
  Properties dialogs, accepts one durable edit in each, and verifies the saved
  document after reopening.

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
