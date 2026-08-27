# Changelog archive: 2026-08-25

Entries through 2026-08-25 are archived here. For current changes, see
[CHANGELOG.md](CHANGELOG.md). Earlier history is in
[CHANGELOG-2026-08i.md](CHANGELOG-2026-08i.md), which continues through
[CHANGELOG-2026-08h.md](CHANGELOG-2026-08h.md).

## 2026-08-25

### Additions and New Features

- Added Edit > `Insert Regular Ring...` as the public detached saturated-carbon
  C3-C8 chooser. The existing `Insert Cyclohexane Ring` command is a C6
  shortcut to the same parameterized route. Rust retains the closed ring model,
  `DocumentOperationV1` prepare/commit transition, durable CDML topology and
  geometry, renderer admission, history, and Undo/Redo; Qt retains only the
  chooser and click handoff. Permanent coverage proves C3-C8 action handoff
  and topology, Escape disarm without mutation, occupied nonmutation with an
  armed retry, and accepted-mutation failed-refresh retirement. Save/reopen and
  Undo/Redo are one-time real-Qt evidence plus shared persistent/history
  contract coverage.

- Added the separate API-owned molecular conversion-output registry. It maps
  public output aliases and preferred suffixes to closed chemistry codec keys,
  including canonical CML2 through `cml` and `cml2`; its exhaustive chemistry
  join and collision validation prevent a future codec addition from silently
  bypassing the public output contract. CML1 remains an input-only profile.

- Added Rust-owned canonical CML2 CLI output. Direct CML/CML2-to-CML retains
  validated molecule and atom IDs plus record order without a native runtime;
  other source formats are emitted only when the closed profile can represent
  every admitted fact losslessly. CML remains a bounded CLI interchange codec:
  CDML is still the sole Ferrum document/session/history/Qt local format, and
  Qt CML export is not part of the product route.

- Added the registered public reaction-workflow E2E. It uses visible **Create
  Reaction** and **Reaction Inspector** actions, accessible reaction details
  and strict-validation state, durable semantic role replacement, member
  highlight/nudge, and definition-only deletion that preserves members.
  Expected nested modals are registered before execution; unexpected modals
  fail closed. This is public workflow evidence, not a raw-document, private
  identifier, coordinate, count, timing, pixel, or fixture-catalog test.

- Recorded the Ferrum screenshot geometry contract in
  [HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md): target a 16:10 complete application
  window including the ribbon/menu and status bar, rather than applying the
  aspect ratio to the canvas alone.

- Added [FERRUM_E2E_TESTS.md](FERRUM_E2E_TESTS.md) as the repository-owned
  guide for Ferrum's staged CLI and Qt end-to-end workflows. It defines the
  registered permanent-suite boundary, public-UI and fixture rules, focused
  implementation checks, and automated GUI-tour capture without modifying the
  vendored generic E2E policy or imposing pixel-equivalence tests.

- Implemented the M4 source and static-proof slice for the distinct runtime-free
  `document.molecule.diagnostics.v1` operation and Qt `Check Structure...`
  surface. Its request carries fenced CDML/revision/digest plus durable selected
  direct-root IDs; Rust owns bounded deterministic findings and typed selector
  resource refusal, the module-level PyO3 executor receives owned snapshot data
  from a detached worker, and Qt authenticates tab/fence/roots before showing an
  accessible modeless read-only dialog. The public E2E path is attached `Me` ->
  unexpanded-group finding and materialization recovery -> `Formula: C3H8`.
  Missing `formal_charge` remains unknown source state; no
  `IncompleteAuthoredCharge`, mutation, auto-fix, or canvas navigation is
  delivered. Fresh build and `all_test.sh` evidence records hygiene (`7518`),
  bindings (`277`), Qt (`207` passed and `1` skipped), and all registered CLI
  and GUI E2Es.

- Added the local `Place Compact Group...` Me-only canvas workflow. Rust owns the closed
  `PlaceFreeCompactGroupV1` transition: Methyl-only key admission, finite snapped anchor and
  canonical orientation, durable root/group ID allocation, atom-free and bond-free candidate
  construction, complete renderer admission, and one atomic history/reload-persistent commit.
  PyO3 exposes only an opaque session-affine begin/commit/cancel capability with durable commit
  facts. The atom/bond-only precommit-overlay contract intentionally has no free-placement
  overlay; the complete prepared transition is the admission boundary.

- Removed synthetic validation-ID preflight from free compact-group placement.
  The live Rust session now accepts only its own typed, fenced pending
  transition and allocates durable IDs at the committed mutation boundary.

- Added direct-root compact-group materialization. A sole atom-free,
  bond-free free group is replaced in the same molecule by immutable recipe
  atoms and bonds; methyl becomes one explicit carbon. Attached compact-group
  exterior topology remains unchanged. Rust commits the replacement as one
  transition with Undo, Redo, and reopen semantics.

- Added compact-group deletion through the existing Select Structure/Delete and
  Backspace controls. Rust accepts exactly one renderer-issued parent/group
  durable-ID target, proves direct membership, removes only the group and its
  unique exterior bond in one history transition, and returns the combined
  count-only `removed_atom_count`, `removed_bond_count`, and
  `removed_compact_group_count` receipt. The focused public E2E proves visible
  author/select/Delete, the visible count receipt and `Formula: C2H6`, and public
  Undo restoring the compact group. Public materialization of that restored group,
  followed by `Molecule Report...` showing `Formula: C3H8`, proves semantic restoration.

- Added recovery for the existing `Attach Compact Group...` action when a
  Rust-issued availability fact exactly matches the selected atom and reports
  `Me` unavailable. The action remains available so its normal typed refusal
  path can present the accessible `Action Not Available` modal with the learner
  instruction to select another atom and try again. Dismissing the expected
  refusal leaves document and pointer ownership unchanged; the E2E fails closed
  if an unrelated modal appears. Stale, missing, and nonmatching observations
  remain disabled with generic readiness guidance. The same-document E2E then
  selects an eligible target, verifies guarded chooser activation, attaches and
  materializes `Me`, and observes `Formula: C3H8`.

- Added the initial Chemistry `Attach Compact Group...` flow for the Rust-owned attached
  `Me` candidate. The later generic `Me`/`NO2` transaction supersedes its focused chooser
  surface; one guarded canvas release begins, previews, and commits the opaque candidate,
  then installs the authoritative Rust observation and focus selection. Selection, revision,
  digest, durable identifiers, geometry, chemistry, and refusals remain Rust-owned.

- Added the private PyO3 one-use attached-methyl compact-group bridge. It accepts
  only a current revision/digest fence, Rust-issued direct atom ID, and finite
  release point; preview is renderer-owned and identifier-free, while commit returns
  only authoritative fence facts plus durable focus and compact-group IDs. The shared
  Python operation-error boundary maps compact-group ID exhaustion as an existing
  stable operation-validation refusal.

- Added typed live-atom revision and digest fence translation plus direct installed-extension
  coverage for element, position, and deletion mutations using Rust-issued durable IDs.

- Added compiled public coverage for durable live-atom mutations through typed fenced
  constructors, preserving typed refusals and no-mutation behavior for invalid requests.

- Added the dedicated Rust compact-group allocator and the fenced attached-`Me`
  transaction, availability, binding, and Qt authoring flow. Compact selection
  now propagates through the installed render projection, and Molecule Report
  matches returned records to captured durable molecule IDs while retaining
  Rust-issued source ID and source-order facts.

### Behavior or Interface Changes

- Replaced attached-methyl-only authoring with the single Rust-owned
  `AttachCompactGroupV1` transaction. Rust now projects reviewed key-and-label
  choices (`Me` and `NO2` initially), evaluates availability after chooser
  selection, and owns candidate admission through commit or cancel. The private
  PyO3 bridge exposes generic choices, availability, begin, preview, commit,
  and cancel; Qt renders those choices and owns only the accessible chooser and
  one-release capture. Methyl-specific Rust, PyO3, and Qt APIs were removed
  without compatibility aliases.

- Added charged `NO2` attached-group materialization using the canonical
  `R-[N+](=O)[O-]` recipe. `CompactGroupRecipeAtomV1` now retains optional
  formal charge, and Rust preserves the individual nitrogen and oxygen charges
  through history and reopen. The registered public E2E attaches and
  materializes `NO2` through visible controls, then verifies editable selection,
  the five-atom/four-bond authored graph, `C2/N1/O2`, `C2H5NO2`, and net formal
  charge `+0`; atom-level charges remain Rust-test evidence rather than a
  public report contract.

- Recorded that the delivered generic attached chooser admits `Me` and `NO2`.
  The other seven persisted catalog keys remain separately scoped pending their
  reviewed recipes, attachment profiles, and row-level chooser availability
  contracts.

- Kept Qt presentation ownership at the canonical render-target, presentation-target, and
  presentation-render-plan modules after removing the unnecessary projection facade.

- Local builds now publish one immutable program root through `build/current`. Stable CLI and Qt
  paths use root-local wrapper/payload pairs with explicit runtime leases, preserving active
  programs across promotion.

### Fixes and Maintenance

- Removed the unreachable `HistoryCapacity` commit refusal and every stale
  Python reference. History-resource exhaustion remains a typed
  preparation-time refusal, where capacity is actually reserved.

- Repaired PyO3 registration of the generic prepared-transition classes through
  the feature-owned binding registry. The clean extension assembly now exposes
  the compiler-exhaustive typed transition/refusal boundary used by regular-ring
  and other document operations without duplicate class registration.

- Regenerated the checked-in operation-protocol schema from the authoritative
  Rust DTOs, restoring the full `ferrum-api` library suite and current
  tetrahedral/E-Z report schema semantics.

- Closed the M2a CML/CML2 import plan as a completed historical import slice.
  The plan now records the separate canonical CML2 output and runtime-free
  CML-to-CML/CDML conversion capability without retroactively expanding M2a's
  original import-only acceptance.

- Reconciled the CML/CDML documentation boundary after the capability audit:
  CDML is the sole native document/session/history/Qt-local format, while CML/CML2
  remains bounded CLI and File > Open interchange that immediately becomes a clean
  native CDML tab. The stale `open --json` success-status wording now records exit `1`
  for completed unsuccessful human-oriented CLI verbs after their one diagnostic or
  envelope; named protocol subcommands retain their separate protocol contract.

- Made completed typed CLI refusals report a nonzero process status without
  duplicating their documented JSON or human-readable output. The shared verb
  boundary now classifies every emitted protocol envelope, including the
  canonical `open --json` CML refusal response, and the native executable exits
  from that result without adding a second diagnostic.

- Corrected the current CML/CML2 command contract: completed unsuccessful
  human and JSON outcomes now exit nonzero after their one diagnostic or
  envelope, replacing the earlier success-status behavior.

- Repaired Reaction Inspector against the current reaction observation
  contract. Its visible strictness state now comes from `reaction.strict`, not
  the removed disposition/union-bounds representation; tab state remains
  owned for the complete nested-modal lifetime, and command handling catches
  the current `ReactionCommandError` boundary.

- Hardened local Python launchers against bytecode-cache drift. `all_test.sh`,
  `build.sh`, and `tests/e2e/run_all.sh` now export the no-bytecode runtime
  behavior, while `source_me.sh` reapplies it after `~/.bashrc` before its
  first Python probe. The authoritative validation commands remain
  `source source_me.sh && pytest tests/` and `./all_test.sh`.

- Completed the durable `document_object_id` migration for projection,
  property, Haworth, render-interaction, molecule report/export, and reaction
  consumers. Ferrum now has one opaque document-object identity contract with
  no source-ID compatibility layer.

- Separated transient presentation-preview replay from committed render-plan
  replay. Arrow authoring now redeems the canonical durable receipt field, and
  its public E2E reports unexpected modal or refusal failures promptly.

- Added `materialize_compact_group` as the precise molecule-diagnostics
  recovery for an unexpanded compact group.

- Completed Rust-owned reaction authoring transitions: dedicated
  create/replace/delete commands redeem through generic transitions, member
  movement uses generic direct-root translation, and authoring choices nest in
  `RenderInteractionObservationV1`.

- Split the direct-root PyO3 binding into cohesive query, DTO, session,
  conversion, and error modules. Tests now parse XML safely, use canonical
  source-owned CDML and durable observed IDs, and acknowledge the stale
  user-template refusal through the accessible event loop.

- Stabilized the public CLI verb E2E around Ferrum's durable document contract.
  Its shared CDML input now persists opaque molecule and atom IDs, so comparing
  `inspect --json` with the equivalent protocol execution authenticates the
  same document instead of comparing two independently allocated identities.

- Made CDML structural rewrite verification independent of XML namespace-alias
  multiplicity. Expanded element and attribute namespaces remain compared,
  while duplicate in-scope bindings for the same URI no longer create a false
  preservation failure after canonical serialization.

- Migrated the public compact-group materialization CLI E2E to the durable
  selector contract. Its inline authored document persists opaque IDs for the
  molecule, atom, compact group, and exterior bond, and the request selects the
  molecule and group by those public document-object IDs.

- Aligned the Qt SMARTS-query refusal map with the Rust binding's durable-target
  vocabulary by consuming `selected_target_not_molecule` directly. Main-window
  construction no longer fails on the retired source-address enum member.

- Completed the installed canvas migration to Rust-owned durable render
  identity. Structural and presentation draw targets now share the sole opaque
  `document_object` target shape; global root Z order comes from validated
  `DocumentProjectionV1.direct_roots`; molecule member issues remain separately
  owned and ordered. PyO3 exposes direct roots and member issues as immutable
  tuples, and Qt no longer reconstructs projection keys, source IDs, structural
  target kinds, render identifiers, or source order.

- Migrated ferrum-document presentation consumers to the durable
  `DocumentObjectIdV1` contract. Persisted presentation selection, clipboard,
  projection, and renderer-admission paths now require their independently
  persisted opaque identity; transient creation previews carry the separate
  identity-free `PresentationPreviewRenderPlanV1`.


- Completed the public unavailable-anchor recovery slice for attached `Me`.
  For an exact-current unavailable selection, Qt keeps the existing `Attach
  Compact Group...` action enabled and presents the standard accessible `Action
  Not Available` dialog. The typed refusal supplies `Me cannot attach to the
  selected atom. Select another atom and try again.` as its explicit primary
  `What happened` message; the standard dialog applies its translated title to
  both the native window title and accessible name. After dismissal, an eligible
  atom in the same document reuses that action to open the guarded chooser.
  Stale, missing, or nonmatching facts remain disabled with generic readiness
  guidance. The shared pointer-action handoff now unchecks an outgoing checkable
  action before it cancels capture, preventing checked/no-owner drift; this
  recovery explicitly rearms Select Structure. Shared
  `FerrumInteractionActionHandoff` now follows real `QEvent` Hide, Show, and
  destruction lifecycle events, removing the arbitrary 250 ms watchdog and
  three-popup limit. Focused real-`QMenu` coverage proves defer-until-hide and
  exactly-once dispatch/check-state ordering. The registered public E2E authors
  saturated CH4 and eligible C-C, observes the accessible dialog title and body,
  dismisses any unexpected modal before reporting, proves queued chooser
  reentrancy handling, then attaches and materializes `Me` to `Formula: C3H8`.
  Its report-phase liveness guard detects a deadlock failure only; it is not a
  product performance threshold.

- Replaced outward structural-deletion receipt identities with atom, bond, and compact-group
  counts. The compact-topology refusal is now a closed redacted category with document-repair
  recovery, while durable removal receipts remain internal to the document session.

- Recorded audit corrections: public deletion receipts expose a count only;
  topology-invalid `repair_document` calls retain their specific refusal;
  accepted bonds publish durable selection; postcommit presentation failures
  report truthfully after the mutation; and mixed selections remain immutable.

- Corrected compact-group deletion to accept renderer-issued durable molecule
  and compact-group object IDs, validate current parent-child containment, and
  lower once to direct CDML IDs for detached mutation. Raw source IDs no longer
  select the public deletion API.

- Corrected attached-methyl compact-group admission so catalog, selector, geometry, and
  chemistry-capacity validation complete before tentative group/bond allocation. Accepted
  commits now expose typed focus and compact-group IDs while canceled and refused attempts
  leave durable sequences and history unchanged.

- Corrected Qt SMARTS results to show the accessible learner warning when Rust's canonical
  `total_match_budget_reached` traversal fact reports unexamined molecules.

- Added the deterministic public Qt SMARTS E2E for both partial and complete result runs, and
  registered it in the aggregate E2E runner.

- Centralized Qt import provenance in `source_me.sh`: repository `ferrum_qt` source precedes the
  sealed runtime and retained caller paths. Aggregate and generated GUI launchers preserve that
  order, while provenance E2Es prevent site-packages substitution.

- Made `source_me.sh`'s canonical Qt/runtime path construction safe for macOS Bash 3.2 and
  `set -u` callers with unset or empty `PYTHONPATH`, after a real build exposed empty-array
  expansion; repeated sourcing remains idempotent.

- Repaired local atomic-build promotion with owner-unique pointer staging, a parent-owned
  close-on-exec lock, per-owner Cargo targets, and locked cleanup of inactive orphan roots.

- Corrected compact durable and stateless documentation/test-policy wording, and aligned the
  local-build lifecycle runner manifest with its behavior-level E2E coverage.

- Sealed local-runtime Receipt V4 wrapper and payload bytes with an owner-executable predicate.
  Corrected generated CLI argument forwarding, and reclaimed only stopped or malformed
  non-current owned `program-*` roots without mutating the published root.

- Pruned allocator- and render-cardinality assertions from bracket tests, retaining only
  durable public behavior through commit, undo, and redo.

- Removed the dead Molecule Information Qt, Rust, PyO3, and test stack, plus
  fragile dialog pytest coverage. Audit cleanup corrected stale compact-group
  prerequisites, receipt wording, and documentation claims.

- Corrected the operation-protocol response-budget name and completed the Rustdoc contract for
  the prepared Wavy insertion binding without changing the response schema or numeric limit.

- Aligned the active architecture, file-layout, and atomic-promotion plan with the V4
  current-root topology and distinct CLI-versus-Qt runtime bundles.

### Removals and Deprecations

- Removed the unused Qt presentation-projection re-export facade and fragile lifecycle tests
  that enforced implementation history rather than durable behavior.

- Removed confirmed dead pre-production configuration and build constants, plus material-tree
  Python bytecode artifacts and the remaining EOF whitespace.

### Decisions and Failures

- Recorded the delivered public Qt attached `Me`/`NO2` author-to-materialize
  slice.
  Rust retains catalog, availability, chemistry, geometry, deferred durable
  IDs, render admission, and atomic commit; Qt retains the accessible chooser,
  one-release pointer handoff, and receipt/refusal presentation. The PyO3
  bridge is private implementation detail. Free placement, the other seven
  catalog keys, and broader full-plan gates remain incomplete.

- Recovered the restored changelog after a truncation failure, retaining every historical bullet
  while consolidating the 2026-08-24 categories into the canonical order.

### Developer Tests and Notes

- Added permanent typed Ethyl materialization coverage: direct-root candidate
  serialization is reparsed before asserting two neutral carbon atoms and their
  normal single bond; attached Ethyl now proves commit, Undo, Redo, and reopen
  semantics without duplicating the generic operation protocol test.

- Local evidence includes the installed-extension live-atom boundary review, the atomic-build
  lifecycle E2E, the focused Qt presentation-target pytest, and the final aggregate suite.

- Focused live-atom, bracket-fence, local-runtime receipt, Markdown, ASCII, and indentation
  checks provide component evidence. The private Qt lifecycle pytest was a one-time implementation
  check and is removed; it is not permanent evidence.
