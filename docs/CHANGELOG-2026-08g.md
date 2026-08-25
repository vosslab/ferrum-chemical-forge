# Changelog archive: 2026-08-23

This archive continues the history before [CHANGELOG.md](CHANGELOG.md). Earlier
history is in [CHANGELOG-2026-08f.md](CHANGELOG-2026-08f.md).

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
