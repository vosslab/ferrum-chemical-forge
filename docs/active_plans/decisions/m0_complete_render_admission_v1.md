# M0 complete-render admission V1

Status: closed on 2026-08-24. The complete-CDML reaction migration and every
remaining M0 authority migration are complete, and the recorded exit evidence
is satisfied.

## M0 closure evidence

M0 closes on 2026-08-24. The fresh aggregate evidence satisfies the permanent
semantic-evidence and final-exit requirements in this record:

- `./build.sh` passed, providing the external Cargo/public-boundary build proof.
- A fresh `./all_test.sh` passed completely: 7,452 hygiene tests, local CLI/E2E
  checks including atom oxidation, 256 Python binding tests, and 392 Qt tests
  with 1 skipped.
- The aggregate suite exercises the required generic semantic routes and
  one-use atomic transition behavior through the native, binding, and Qt
  boundaries. It supersedes the earlier focused receipts and pending aggregate
  evidence described below.

This closure is limited to M0 complete-render admission and authority migration.
It does not claim completion of deferred M1 compact-group delivery, M6-M10 XML
milestones, or any future corpus-preservation gate. Every below statement that
says M0 is incomplete, names a blocker, or treats aggregate evidence as pending
is superseded historical implementation context, not a current requirement or
status claim.

## Scope

This record decides the M0 cross-crate ownership boundary for complete-render
admission. It does not authorize compact-group protocol, CLI, PyO3, Qt, catalog,
or user/API documentation delivery.

## Approved ownership

- `ferrum-render-contract::complete_document_admission_v1` owns only portable,
  versioned contract data: `DocumentCompleteRenderCandidateV1`, the closed
  classification/refusal vocabulary, and portable presentation DTO/value types.
- `ferrum-render` alone constructs and owns opaque
  `AcceptedCompleteRenderV1`,
  `AcceptedCompleteRenderPresentationV1`, and
  `AcceptedCompleteRenderRootV1`. Its public pure
  `admit_complete_document_render_v1` accepts the contract candidate and
  returns the renderer-owned accepted value or the contract-owned typed
  refusal.
- `ferrum-document` alone derives a candidate from the final immutable
  `SessionDocumentObservationV1`, retains the renderer result privately in
  `RendererAdmittedPending` within `PreparedSessionTransitionV1`, and
  re-derives and re-admits it at redemption. It requires equality between the
  retained and newly admitted renderer values before effects or history change.
- `ferrum-render` purely lowers and classifies the detached candidate. It never
  calls `DocumentSession`.
- `PreparedSessionTransitionV1` remains the only public post-preparation commit
  capability. Generic prepare, commit, revalidation, foreign, replay, stale,
  history, and no-op semantics remain intact.
- Route metadata may expose only immutable, contract-owned presentation data.
  The renderer accepted value has private fields and exposes no public
  constructor, candidate, plan, verifier, identity, serialization, or
  session-redemption capability.

## 2026-08-28 complete-realization precision amendment

Direct-root classification remains necessary but is not sufficient admission.
`ferrum-render` composes both the current and candidate complete resolved render
plans and derives their private omission sets. Ordinary authoring admits only
when every candidate root exclusion, plan issue, and member depiction issue was
already present in the current plan. Existing imported diagnostics may remain
or be repaired; introducing or replacing an omission is a typed atomic refusal.
The accepted value retains the exact candidate realization for identifier-free
preview paint, and redemption rederives that realization before commit.

History navigation uses a separate document-private policy. Undo and redo
authenticate and rederive the exact retained target state instead of applying
the ordinary current-to-candidate omission delta. This lets an admitted repair
remain undoable without creating an operation-specific escape hatch from the
authoring invariant. Neither policy is a new public protocol version, and no
compatibility alias or frontend fallback is introduced.

## Visibility amendment

This approved amendment corrects the accepted-value visibility boundary without
changing M0 scope or migration order. Rust privacy is crate-scoped: a
`ferrum-render-contract` constructor callable by `ferrum-render` would also be
callable by downstream crates. `#[doc(hidden)]` only suppresses documentation;
it cannot prevent forged acceptance authority.

The renderer therefore mints its own private-field accepted value beside the
classification it performs. `ferrum-render-contract` remains a portable
candidate, vocabulary, and presentation-data boundary. There is no deprecated
or compatibility forwarder for removed contract accepted types, constructors,
or receipt/proof wrappers.

Commit eligibility is proved without a public verifier: document-private
`RendererAdmittedPending` retains the original opaque renderer value; commit
re-derives the exact candidate from the saved immutable observation and pending
identity, calls the same pure renderer admission function, and requires
equality of the accepted values. A refusal or unequal value reaches the typed
admission/refusal path before any effect or history mutation. The accepted value
may provide only deliberately lossy immutable presentation extraction; it never
exposes construction, candidate binding, verification, or redemption facts.

## Closed classification

The V1 vocabulary is closed and versioned:

```text
CandidateDerivationFailureV1
  SuppressedObservation | InconsistentObservation | InvalidRootIdentity
  MissingRequiredRenderFact | ResourceLimit

CompleteRenderRootClassV1
  VisualMolecule | VisualText | VisualVector
  AllowedNonvisual(AllowedNonvisualRootReasonV1)
  Refused(RefusedRootReasonV1)

AllowedNonvisualRootReasonV1
  // no V1 variants: empty allowlist

RefusedRootReasonV1
  UnsupportedRootKind | InvalidGeometry | MissingVerifiedLayout
  ProfileExcluded | MissingRequiredPrimitive

CompleteRenderAdmissionRefusalV1
  CandidateDerivation(CandidateDerivationFailureV1)
  RootRefused { root: durable identity, class: CompleteRenderRootClassV1 }
  CandidateMismatch
```

- Document derivation maps frozen observation, projection, and composition
  failures once to `CandidateDerivationFailureV1`.
- Renderer lowering assigns each root exactly one class and requires accepted
  primitive representation for each visual class.
- A refused root returns `RootRefused` with its `Refused(...)` class; failed
  redemption returns `CandidateMismatch`.
- API, CLI, PyO3, and Qt may map the category to bounded recovery or display
  data, but may not change its class or inspect renderer prose.
- Renderer diagnostics are bounded adapter-local prose derived after this
  boundary and are never parsed across it.

## Nonvisual policy

V1 has an empty valid-nonvisual allowlist. A Molecule, Text, Plus, Arrow,
Polyline, Wavy, Bracket, or geometric root without its required primitive is
`RootRefused`, never silently accepted. A future exception requires evidence of
one persisted root kind, its durable semantics, why no primitive is intended,
and why it remains valid; it then adds one explicit allowlist reason and a
permanent behavior test.

## Public-surface removal

Remove or make crate-private, with no deprecated forwarding shims:

- Contract exports of `AcceptedCompleteRenderV1` and equivalent accepted
  receipt/proof wrappers, including every
  `from_renderer_classification_v1` constructor and any `#[doc(hidden)]`
  construction bridge.
- Public accepted-value `verify_*`, candidate/identity, renderer-plan,
  serialize/deserialize, and document-redeem operations.
- Raw `ferrum-render` candidate, pending-identity, receipt, verification, and
  plan-access exports, including `admit_document_render_candidate_v1`.
- Public complete-CDML prepare/commit methods and
  `PendingCompleteCdmlMutationV1` as external admission machinery.
- Public direct-bond bridge prepare/commit and
  `DirectBondRendererAdmissionBridgeV1`.
- Public compact-group experiment prepare/commit adapters and `document-render`
  materialization re-exports.
- Route-specific pending, proof, lease, and receipt wrappers for catalog,
  admitted molecule/interchange insertion, worker, and materialization routes.

Retain semantic route request/result DTOs only where they carry intent or
results, never raw candidate, proof, or receipt authority.

## Staged migration

1. Add contract types, pure renderer profile, document-private retention, exact
   revalidation, and generic-core tests.
2. Migrate complete-CDML mutation and reaction lifecycle, gesture, and
   translation; make authoring compilers private adapters and remove public raw
   CDML prepare/commit.
3. Migrate direct bond and catalog placement; retain only immutable
   renderer-issued preview DTOs.
4. Migrate explicit-hydrogen and compact-group materialization, admitted
   molecule/interchange insertion, and worker wrappers; retain deferred IDs and
   source-fenced no-change results.
5. Remove obsolete exports and re-exports, then map shared categories in API
   and PyO3 adapters.

Compact-group protocol, CLI, PyO3, and Qt delivery are deferred to M1. No user
or API documentation gains unimplemented compact-group symbols during M0.

## Admitted insertion amendment

This approved amendment governed the M0 removal of admitted
molecule/interchange insertion and materializer bridges. The generic migration
is implemented for those routes and explicit-hydrogen materialization; M0
remains open for its remaining migration tranches and final exit evidence.

- `ferrum-document` gains closed semantic requests
  `InsertMoleculeV1` and `InsertInterchangeRecordBatchV1` as
  `SessionOperationV1` variants. Each request contains validated source intent
  only; it contains no session identity, candidate, planned identity, renderer
  fact, source fence, deferred identifier, or redemption capability.
- Successful generic results carry durable post-commit facts through
  `InsertedMoleculeV1` and `InsertedInterchangeRecordBatchV1` outcome variants.
  Interchange record results preserve exact source order. The enclosing generic
  result remains authoritative for final revision and observation.
- Document-private lowering reserves identities, derives and admits the exact
  candidate, and stages effects through `PreparedSessionTransitionV1`. Generic
  redemption re-derives and re-admits before mutation; identifiers publish only
  after an atomic successful commit.
- Both insertion requests use `TransitionAuthorizationV1::None`. The
  direct-bond capability policy remains limited to direct bond authoring.
- Empty, invalid, impossible, stale, foreign, replayed, or refused requests
  return their typed failure. They do not become a no-change success or expose
  deferred identifiers.

Remove without aliases, forwarding methods, compatibility wrappers, or
re-exports: admitted molecule/interchange pending values, their document
prepare/commit methods, PyO3 pending bindings and pre-commit accessors, and
API import holders retaining those pending values. Adapters retain only the
generic prepared transition and decode identifiers from the successful generic
result.

Explicit-hydrogen materialization migrates to the same generic boundary with a
closed request and post-commit outcome using `TransitionAuthorizationV1::None`.
Its stable stateless/live protocol vocabulary may remain as request/result
data; route-specific document and renderer pending, prepared, and commit
wrappers are removed.

Compact-group materialization is M0 internal cleanup only. Document-private
callers may use generic complete-render transition machinery, but M0 adds no
public compact-group operation, protocol envelope, CLI route, PyO3 symbol, Qt
path, renderer re-export, or public prepare/commit adapter. Public compact
materialization remains deferred to M1 authorization in
[compact_group_authoring_v1.md](../active/compact_group_authoring_v1.md).

The pending implementation must add focused semantic evidence for successful
once-only commits, source-ordered interchange outcomes, state preservation on
failure, inert presentation, generic Python/API use, and existing hydrogen
changed/no-change behavior. Those evidence items are requirements, not
passing-test claims.

## Completed generic reaction authority migration

The approved Stage 2 reaction slice is complete through the Rust,
`ferrum-document-render`, API/PyO3, and Qt interface layers. This is a bounded
implementation result, not M0 exit evidence.

- `CreateReactionV1`, `ReplaceReactionMembersV1`, and `DeleteReactionV1` are
  document-owned `SessionOperationV1` semantics with closed refusal vocabulary.
- `ferrum-document` owns direct-root validation, ordered role membership,
  reactant/product policy, uniqueness and cross-reaction exclusion, strict
  definition checks, deterministic reaction-ID allocation, and private CDML
  lowering.
- Reaction creation, lifecycle, and translation resolvers produce opaque
  `SessionOperationTransitionRequestV1` values. The generic document
  prepare/commit path owns preparation and the sole redemption authority.
- Route-specific reaction prepared/committed receipts and prepare/commit
  exports are removed. `PreparedSessionTransitionV1` remains the only public
  post-preparation capability.
- Reaction translation uses the existing `TransformTopLevelRoots` operation;
  no reaction-specific translation operation was added.
- The raw complete-CDML adapter and public
  `PendingCompleteCdmlMutationV1` prepare/commit path are removed without
  forwarding shims. Raw CDML candidate parsing and lowering remain private
  document implementation details.
- Successful generic results publish durable reaction IDs only after the
  accepted generic commit. Generic prepared-transition debug output remains
  redacted to lifecycle state and does not reveal pending authority.
- API/PyO3 exposes generic reaction requests and generic post-commit results;
  it no longer exposes reaction receipts or direct-create authority. Qt
  creation, lifecycle, and translation routes use generic preparation and
  generic commit.

Historical focused evidence recorded 19 passing document admitted-transition
tests, 16 renderer reaction tests, `cargo check -p ferrum-api --features
python-binding`, and the API `reaction_opaque_surface` check. The former
external Cargo compile-fail fixture catalog and orphaned reaction probes were
removed: they checked names and compilation failure rather than a durable
semantic behavior, and no permanent name-absence test replaces them. Qt route
code had passed pyflakes and in-memory compilation; the permanent composer test
uses visible controls but requires a rebuilt native runtime. This historical
evidence predates the later canonical atomic-operation API change and is not a
claim of current assembled-build, binding-runtime, Qt-runtime, aggregate-suite,
or M0-exit validation.

Before closure, M0 awaited removal of admitted molecule/interchange insertion
and explicit-hydrogen/compact-group materialization authority, its required
evidence, and M0 exit validation. Those scopes are now complete.

## Permanent evidence

The minimal permanent evidence proves:

1. Valid frozen input produces immutable accepted presentation data without
   mutation authority.
2. Document and renderer share typed derivation/root failure categories without
   text parsing, and the empty-nonvisual policy refuses a missing primitive.
3. Candidate A cannot redeem candidate B, foreign, replayed, or stale state;
   failed redemption preserves the stable observation and a valid handle redeems
   once.
4. Supported semantic routes reach generic preparation and one-use generic
   commit without exposing route-specific redemption authority.
5. One generic runtime route, one complete-CDML/reaction route, and one direct
   mutation route mutate only after accepted transition admission.

No permanent external Cargo fixture catalog or source-name-absence test is
required for this cleanup. Permanent evidence stays focused on behavior:
renderer tests observe only immutable presentation or typed refusal, and
document tests prove once-only commit plus changed, fresh, foreign, and replayed
candidate refusal before mutation.

## Completed catalog semantic migration

The catalog-placement Stage 3 slice is complete. This records a bounded
implementation result, not M0 exit or cross-crate completion.

- `SessionOperationV1::PlaceCatalogMoleculeV1(CatalogMoleculePlacementV1)` is
  the closed document semantic request. Its generic post-commit outcome carries
  only the catalog key, placement anchor, and durable document-owned root ID.
- Catalog lowering, generated-ID reservation, complete-render admission,
  deferred effects, and outcome staging now occur privately in
  `ferrum-document` before generic transition redemption.
- Catalog placement passes `TransitionAuthorizationV1::None`. The direct-bond
  authoring-capability policy does not expand to catalog placement.
- `ferrum-catalog-placement` resolves closed key/anchor intent into the
  semantic request. It no longer owns a gesture, pending receipt, capability,
  preview, or commit authority.
- Route-specific V1/V2 catalog document-render and API authority, including the
  PyO3 V2 handle binding, is removed without aliases or forwarding wrappers.
  The stateless protocol resolves the closed request and prepares and commits
  through the generic transition; it preserves its caller-digest comparison and
  maps renderer refusal distinctly.
- The V2 UI lease remains a local paint-scheduling boundary only. It does not
  retain document capability, candidate, proof, or commit authority.

Focused implementation evidence records passing `cargo fmt --check`, affected
crate checks including the Python binding feature, the document generic catalog
transition test, the Haworth catalog integration test, and four template-catalog
protocol tests. This is bounded evidence and does not claim M0 completion.

Two shared-M0 feature-test blockers remain outside catalog ownership:

- `attached_cyclohexane_binding.rs` passes an immutable session to its
  now-mutable `cancel` helper.
- `live_document_smarts_query_v1/tests.rs` still calls the removed raw
  `commit_complete_cdml_transaction_v1` mutation surface.

Their owners must migrate those adapters/tests to generic transition authority;
catalog placement must not restore a removed surface to make either test pass.

One-time differential evidence compares the current `Me`/`NO2` placement and
materialization experiment, attached and free, with the new profile. CDML, IDs,
counts, source order, coordinates, recipe geometry, and batch layout are
diagnostic only, not permanent contract tests.

## Evidence and status

The required focused Graphify query found the active complete-render-admission
plan, renderer-admission architecture, and this decision record. This record
uses the approved architecture authority at
`/private/tmp/ferrum_m0_complete_render_architecture.md` as amended by
`/private/tmp/ferrum_m0_visibility_amendment.md`. The decision found no
architectural blocker. Its approved generic render-admission scope is complete;
later product work uses that closed authority rather than reopening M0.

## Transition presentation metadata

The approved generic metadata seam is document-owned and returns only a copied,
immutable display DTO while a prepared transition remains redeemable. Access after
commit or cancellation returns the closed `Consumed` refusal; a
copied DTO remains inert display data and never renews transition authority.

Commit remains opaque document authority. The DTO exposes neither a raw render
plan capable of candidate reconstruction nor any candidate, proof, pending
identity, source fence, accepted value, verifier, deferred effect, or redemption
capability. Direct-bond route results stay route-private, and catalog preview
leases remain UI-local renderer lifecycle state rather than generic transition
metadata.

This decision records the M0 metadata seam and the direct-bond/catalog
migrations that depend on it. It does not expand later product work into new
renderer semantics, generic route-result serialization, or broader product/API
changes.

## Direct-bond M0 amendments

The approved cross-crate amendment at
`/private/tmp/ferrum_m0_direct_bond_cross_crate_amendment.md` replaces the
decomposable direct-bond bridge with the generic semantic
`SessionOperationV1::CreateDirectBondV1(CreateDirectBondV1)` request and the
generic `SessionOperationResultV1` outcome boundary. Preparation accepts the
request only through the generic session-operation transition path; the document
keeps direct-bond compiler, candidate, proof, pending identity, reserved IDs,
and deferred effects private. After a successful atomic commit, the generic
result exposes `SessionOperationOutcomeV1::DirectBondV1` with authoritative
direct-bond facts and the post-commit observation. Existing operations retain
the exhaustive `Standard` outcome.

`commit_session_operation_transition_v1` remains the sole generic redemption
and commit authority. The V3 renderer retains its generic transition and reads
direct-bond facts only from a successful generic result. It receives no
direct-bond-specific preparation holder, bridge, direct commit method,
`into_parts` conversion, or compatibility alias.

The approved precommit-preview amendment at
`/private/tmp/ferrum_m0_direct_bond_preview_amendment.md` adds only an
identifier-free, renderer-owned `DocumentPrecommitOverlayV1` to the copied
generic `PreparedSessionTransitionPresentationV1`. The document derives this
immutable paint value during generic preparation from the admitted complete
plan and its private selected targets. The V3 renderer paints the copied
overlay directly; it neither receives nor reconstructs bond or atom IDs,
candidate/proof data, source fences, pending identities, deferred effects, or
route outcomes. Operations without an admitted precommit subset expose no
overlay.

Graphify evidence places `SessionOperationResultV1` with the generic document
session/transition hub and the preview selector in `ferrum-render`; it supports
this separation but is not migration proof. The identifier-free V3 precommit
primitive projector and move-only pointer-token renewal are implemented. The
previous full `./all_test.sh` receipt passed 7,480 hygiene tests plus the native
and Qt gates. These amendments do not change M0 scope or status: the latest
aggregate full-suite and external Cargo-boundary evidence remain pending, so M0
remains incomplete.

## Generic transition authorization

The approved 2026-08-24 authorization amendment adds the closed,
document-owned `TransitionAuthorizationV1` input to generic session-operation
preparation. Its V1 cases are `None` and
`AuthoringCapability(AuthoringCapabilityV1)`. `None` is required for existing
generic operations; `SessionOperationV1::CreateDirectBondV1` requires the
moved opaque authoring capability. This is a source-breaking generic signature
change: every caller migrates in the same change, with no overload, defaulting
helper, `Option<AuthoringCapabilityV1>` adapter, compatibility forwarder, or
route-specific wrapper.

Generic preparation alone validates the requirement and issuer, claims a valid
capability, and privately retains only `AuthoringCapabilityClaimV1` in
`PreparedSessionTransitionV1`. Generic commit alone consumes that claim during
the sole atomic redemption. Generic terminal cancellation and consumption and
incomplete-preparation cleanup own the remaining lifecycle through one
document-private helper: a nonterminal refusal leaves the opaque prepared
transition available for its valid generic retry, while terminal paths consume
or restore only under generic control. No caller, renderer, or semantic route
may separately claim, release, consume, cancel, or reanimate the authorization.

`direct_bond_v3_lifecycle` passes its existing capability by value only through
the generic authorization input with its existing semantic request. It retains
route-local gesture/fence handling, copied presentation use, and the generic
prepared handle. Direct-bond compilation, renderer admission, generated IDs,
candidate, proof, pending identity, outcome staging, claim lifecycle, and
commit authority remain document-private generic ownership. The direct-bond
route exposes successful facts only through the existing generic
`SessionOperationResultV1` outcome.

The amendment authorizes neither a direct-bond-specific prepare, claim, or
commit path nor public pending/decomposition access. In particular, no public
holder, bridge, direct commit method, retained-claim accessor, `into_parts`
conversion, raw candidate/proof, renderer acceptance constructor, alias, or
forwarder may remain. `commit_session_operation_transition_v1` remains the
only public redemption and commit authority.

The required Graphify gate queries generic preparation, authoring-capability
placement, and the `direct_bond_v3_lifecycle` path. It is routing evidence
only: it confirms the intended generic preparation/execution seam and the
current direct-bond lifecycle seam, but does not prove implementation or
migration. Required permanent evidence includes `None` for a representative
existing operation; typed direct-bond refusal for `None`, foreign, and consumed
capabilities before mutation; generic claim/commit/retry/consumption ownership;
all four V3 endpoint forms through generic prepare and commit. This approval
text records the then-pending implementation state;
the post-audit corrective amendment below supersedes it for implementation
status. M0 remains incomplete pending the required evidence.

## Direct-bond post-audit corrective amendment

The approved 2026-08-24 post-audit correction replaces the affected details of
the authorization and precommit-preview amendments. The generic redemption
migration is implemented: the direct V3, keyboard, and protocol routes now
prepare through `PreparedSessionTransitionV1` and redeem only through generic
commit. A rebuilt runtime passed the focused binding (`14/14`) and pointer Qt
(`7/7`) gates. It closes four observed authority leaks:

- public capability inspection and claim APIs exposed issuer, availability,
  comparison, claim, and consumption authority outside `ferrum-document`;
- cloneable V3 gestures could retain an alias to a capability after admission;
- the copied precommit overlay wrapped a render plan and could reveal targets
  or identifiers rather than carrying paint-only data; and
- successful generic redemption could finalize history/effects before settling
  its private authorization claim.

`AuthoringCapabilityV1` remains the sole public opaque input receipt. Its
issuer, state, inspection, comparison, claim, consumption, claim type, and
access errors become document-private. `DocumentSession` provides only the
non-inspecting `issue_authoring_capability_v1` factory for a live session.
`prepare_session_operation_transition_v1` is the sole public operation that
validates `TransitionAuthorizationV1`; it privately claims a valid receipt and
retains only that private claim in the opaque prepared transition. There is no
public issuer accessor, retained-claim accessor, constructor, verifier,
decomposer, cancellation handle, or alternate preparation authority.

`DirectBondGestureV3` is consumed by V3 admission and its lifecycle payload is
not cloneable. The lifecycle moves its opaque capability into the generic
authorization input and supplies pointer-derived semantic intent only. V3 has
no availability, issuer, claim, release, consume, candidate, proof,
generated-identity, target-selection, or commit authority. Capability replay,
foreign-session, and missing-authorization behavior belong exclusively to the
generic transition boundary.

`DocumentPrecommitOverlayV1` and its direct-bond counterpart become
renderer-owned, closed, identifier-free paint values. Their public observation
may expose only immutable paint primitives and paint attributes; it may not
expose a render plan, batch, target, record/persistent identifier, selector,
candidate, proof, document reference, or conversion back to any such value.
The renderer alone lowers document-private selected targets during preparation;
the generic presentation copies the resulting paint-only value for V3 to draw.

For an authorized changed transition, all fallible validation occurs while the
prepared transition and private claim remain live. A nonterminal owner refusal
leaves both redeemable for retry; a foreign attempt leaves them untouched. At
the sole private redemption point, generic commit consumes the claim before the
infallible, already-preflighted history/effect finalization and consumption.
Explicit cancellation and `Drop` use one private terminal helper that consumes a
retained claim exactly once; only cleanup of preparation that fails before
return restores local availability.

This correction retains no compatibility aliases, forwarders, legacy gesture
entry points, direct-bond-specific preparation/commit paths, raw overlay-plan
accessors, or public capability lifecycle surfaces. M0 remains incomplete
pending the latest aggregate full-suite, specified semantic, and external
public-boundary evidence.

## Completed canonical atomic-operation cleanup

The document operation boundary now has one public closed-operation entry:
`DocumentSession::apply_document_operation_v1`. It accepts document-owned
semantic operation intent and executes solely through the generic request,
prepare, and commit lifecycle. It is the canonical atomic-operation surface;
there is no second public execution spelling.

The former public `submit` and
`execute_session_operation_transition_v1` spellings are removed together from
`ferrum-document`, the API/PyO3 layer, protocol adapters, and Qt callers.
Removal is source-breaking by design: this pre-production repository has no
compatibility population that justifies aliases, wrappers, or forwarding paths.
`DocumentBondOrderV1` is likewise the canonical molecule-insertion bond-order
vocabulary; `MoleculeInsertionBondOrderV1` is removed rather than retained as
an alias.

The historical external Cargo compile-fail fixture catalog and orphaned
reaction probe artifacts are removed. They verified absence of particular
source names, not a durable user-visible or semantic behavior, and therefore
fail the permanent-test policy. No replacement process-status or name-absence
fixture is authorized. Permanent evidence exercises semantic requests through
the single generic lifecycle, verifies atomic success and typed failure
preservation, and keeps test setup inline unless a policy-approved real-file
dependency is being tested.

Build cleanup lifecycle evidence belongs in the E2E tier rather than fast
pytest, and SDF import E2E is registered in that tier. Current evidence covers
the `ferrum-api` Cargo suite (94 unit tests plus its integration targets) and a
rebuilt local application. M0 remains incomplete pending its remaining roadmap
work and final exit evidence.

## Completed generic operation authority migration

The M0 authority cleanup now also covers document construction operations that
previously had route-specific prepared receipts. `CreateAtomV1`, `CreateBondV1`,
and `CreateHaworthMoleculeV1` resolve through the generic request, preparation,
and one-use commit lifecycle owned by `PreparedSessionTransitionV1`. No
route-specific public prepare, commit, candidate, proof, or receipt surface
remains for those operations.

Attached cyclohexane, direct-bond, and Haworth interaction adapters paint the
renderer-issued, identifier-free `DocumentPrecommitOverlayV1`. The overlay is a
copied paint value, not a raw render plan or a reconstruction capability. Wavy
and bracket PyO3 methods retain their existing supported semantics; this
migration does not rename, emulate, or broaden those operations.

The cleanup removes external Cargo fixture catalogs, source-name/inventory
checks, and an unreferenced corpus input because they measured migration history
rather than a supported contract. Durable tests stay inline where possible and
exercise semantic behavior, typed refusals, and one-use atomicity. This is an
implementation checkpoint, not M0 exit evidence; fresh aggregate validation is
still required before the milestone can close.
