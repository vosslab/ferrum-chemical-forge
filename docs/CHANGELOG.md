# Changelog

Earlier history is in `CHANGELOG-2026-08i.md`. Its archive navigation
continues through [CHANGELOG-2026-08h.md](CHANGELOG-2026-08h.md) and
[CHANGELOG-2026-08g.md](CHANGELOG-2026-08g.md).

## 2026-08-26

### Fixes and Maintenance

- Repaired normalized native input modifiers across the controller-owned viewport adapter. Immutable pointer and semantic key/pointer intents now preserve Qt modifiers; native structure selection consumes that intent fact directly, while legacy line, text, shape, atom, and bond endpoints receive the same value through their normalized event adapter. Removed the duplicate window modifier scratch state. Real QTest selection coverage now proves ordinary selection followed by Shift-additive selection retains both Rust-issued targets.

- Restored controller-owned native viewport input lifecycle for active Ferrum document tabs.

- Added the typed `CloseDecision` / `CloseResult` lifecycle seam for Ferrum
  document tabs. Ordinary close now acquires Save, Discard, or Cancel before
  using the shared guarded lifecycle application; deterministic callers may
  explicitly discard without mutating Rust dirty state. The standard
  `main_window` fixture now requires one closed tab and finite progress during
  teardown, so a refused dirty tab fails immediately instead of looping.

- Added focused typed-close lifecycle coverage for successful real-file Save,
  injected Save failure, Cancel-equivalent `KEEP_OPEN`, and invalid-index
  `NO_TAB` outcomes. The tests use Rust-owned dirty state and the existing
  shared window fixture without modal prompts or timing.

- Restored the failure-atomic `FerrumWindowModeSync` activation protocol:
  activate the mode controller, establish the provisional binding and exact
  QAction checks, activate the feature endpoint, then publish success. A
  declined or failing endpoint now cancels feature and controller state before
  publishing inactive state. Real-window tests prove native Add Atom dispatch
  reaches Rust mutation and programmatic bond-to-text-to-structure transitions
  converge QAction identity, dispatch, and Escape cleanup.

- Repaired line and presentation mode dispatch ownership. Each QAction now
  registers and binds its activation, normalized-input endpoint, and
  cancellation immediately beside feature construction; no central
  post-construction action inventory remains. Invalid or lifecycle-escaped
  line intents now fail before document mutation with a feature-specific
  `RuntimeError`. Removed the brittle portability source-text assertion; the
  retained real-window test proves declarative menu/ribbon QAction reuse.

- Repaired authoring-mode dispatch ownership. Atom, structure selection, and
  each line authoring QAction now declare their own normalized activation,
  input-dispatch, and cancellation endpoints beside registry registration.
  `FerrumWindowModeSync` owns the one active lifecycle and native canvas input
  seam; menus and ribbon remain passive clients of the exact QAction.

- Converged Qt active-tool state on per-window `FerrumWindowModeSync` bindings.
  Feature-owned registered QActions retain their handlers, shortcuts,
  accessibility, checked and disabled state; the YAML ribbon is now passive and
  receives typed active/default capability state instead of maintaining a mode
  map or action-ID policy. Removed the shared mode-toolbar bridge, fixed action
  maps, ribbon mode APIs, and private/reverse declarative-resource loading.
  Menu and ribbon declarations now resolve through one public neutral loader
  and combined failure-atomic preflight before visible construction.

- Reconciled compact-group and parity documentation around the delivered
  nine-recipe attached catalog. `FULL_PARITY_RUST_FIRST.md` is now the sole
  full-parity ledger; `ferrum-plan-v3.md` remains a namespaced historical
  implementation record. The missing generic attached CLI/protocol route is
  explicitly the next bounded parity slice. Documentation retains the separate
  manual 16:10 screenshot and keyboard/accessibility evidence gap.

- Restored `Attach Compact Group...` as one registered public QAction with
  declarative `Draw > Compact groups` and `Structure > Groups and templates`
  clients. The routes reuse its feature-owned handler, state, and accessibility
  presentation without a compatibility placement path.

- Repaired the Atom Oxidation State public Qt E2E's fluoride selection by
  clicking an interior point of the live painted selectable glyph rather than
  assuming the atom anchor is a hit target.

### Additions and New Features

- Delivered the ninth and final attached compact-group recipe, `phenyl` /
  `Phenyl` / `Ph`. Its generic Rust materialization contract is one neutral
  six-carbon, alternating normal single/double Kekule cycle with carbon focus;
  the retained exterior normal-single `n1` bond preserves durable identity and
  anchor-side direction while its compact endpoint rewires in both directed
  exterior orientations. Role-addressed native lowering and renderer proof
  establish the exact contract; no aromatic schema or compatibility branch was
  introduced, and aromatic-input `kekulize` is not a gate. Final code/test
  review passed, a fresh build promoted the runtime, and the installed public
  Attach -> chooser -> materialize workflow returned `succeeded` / `updated`,
  reported `C8H10`, and left a usable scene. The installed binding contract
  passed 8/8 and `all_test.sh` exited 0 with 7,633 hygiene, 280 binding, and
  220 Qt tests passed with one skip. All nine attached compact-group recipes
  are delivered; this compact-group recipe milestone is complete, while M4 and
  the Rust/OASA/BKChem parity goal remain incomplete.

- Implemented the eighth attached compact-group recipe, `AcylChloride`:
  the Rust catalog now issues `acyl_chloride` / `AcylChloride` / `COCl` for
  neutral attached `R-C(=O)-Cl`. Generic materialization returns the attachment
  carbon as focus, writes normal double C=O and normal single C-Cl bonds, and
  retains the exterior normal-single bond identity. Approval review passed;
  a fresh promoted build and public installed Qt Attach -> chooser -> materialize
  workflow passed; and `all_test.sh` completed with 7,633 hygiene tests,
  280 installed binding tests, and 220 Qt tests with one skip. Rust catalog and
  session semantics remain the exact topology and directed exterior-identity
  evidence. Qt proves the public receipt and C/O/Cl report composition only.

- Replaced Ferrum's dense icon-only authoring strip with a YAML-authoritative
  task ribbon. Home, Structure, Reactions, Annotate, and View now project
  existing registry-owned actions into labelled task groups with text-plus-icon
  controls and group-local More menus. The menu tree remains independently
  YAML-authoritative; checked state, disabled state, shortcuts, handlers, and
  cancellation remain owned by each existing QAction.

- Replaced the transitional runtime-composed menu route with one strict
  YAML-authoritative menu tree. `menus.yaml` now owns the complete top-level
  order, static action placement, sections, separators, nested menus, and the
  declared Recent Files dynamic-menu position; the registry binds the existing
  feature-owned QAction or QMenu clients and the recursive builder assembles
  the tree once. Unresolved static or dynamic declarations now fail preflight
  rather than being skipped. `menu_layout.py`, `menu_construction.py`, and
  their compatibility bridges were removed.

### Behavior or Interface Changes

- Reorganized the Qt menu bar around drawing tasks: `Edit` begins with History
  and Clipboard, then selected-object editing; `Draw` follows it with labelled
  Drawing Setup, Bonds, Rings, Arrows, Annotations, Geometry, Arrange, and
  Tool groups. The retained `Insert Regular Ring...` and `Transform Complete
  Roots` cascades provide recognition-oriented access to their command
  families. Reaction commands remain in Chemistry and authoritative refresh
  remains in View. The refactor reuses the existing QAction objects, preserving
  their handlers, shortcuts, checked and disabled state, QObject identity, and
  ribbon/context-menu reuse.

- Completed the declarative menu and ribbon cleanup. `menus.yaml` is the sole
  menu-placement authority, while `ribbon_layout.yaml` owns the grouped,
  labelled Home, Structure, Reactions, Annotate, and View ribbon tabs. Both
  surfaces reuse the exact registry-owned QActions by stable semantic ID.
  Attach and Place Compact Group are declarative compact-group clients under
  `Draw > Compact groups`; Chemistry retains materialization. The ribbon's
  owned single-shot overflow timer is teardown-safe, and the dormant
  `modes.yaml`, `ModeToolbar`, and fixed action allowlist are removed.

### Developer Tests and Notes

- Focused declarative-resource, action, ribbon, shared-window, portability,
  widget, lint, ASCII, indentation, typing, and source-limit validation passed
  `3,804` checks under Python 3.12.14 and PySide6 6.11.2. Focused ribbon
  lifecycle, overflow, and hygiene selections also passed. The macOS 16:10
  screenshot and keyboard/accessibility walkthrough remains manual evidence;
  this entry does not claim it passed.

- Pre-repair focused Qt validation passed `35` tests across declarative resources,
  action registration/keybindings, real-window seams, ribbon reuse, property
  clients, and the direct-glycosidic Haworth regression. Required source
  hygiene passed `3,272` checks across pyflakes, indentation, import, import
  requirements, and source-size gates. The approved live 16:10 scanability and
  keyboard/accessibility cognitive walkthrough remains manual HCI evidence; no
  screenshot, pixel, timing, or manual-visual claim is recorded as automated
  proof.

- The post-validation render-time failure-atomicity regression passed `8`
  focused keybinding/registry tests. Independent re-review then passed the
  targeted late-resolution failure, successful-retry, and repeat-assembly
  selection (`3 passed, 5 deselected`). These code-level results do not replace
  the remaining manual 16:10 scanability and keyboard/accessibility walkthrough.

- Implemented the sixth attached compact-group recipe, `Carboxyl`: the Rust
  catalog now issues `carboxyl` / `Carboxyl` / `COOH` with the neutral attached
  `R-C(=O)-OH` recipe. Existing generic materialization retains exterior-bond
  identity and returns the attachment carbon as focus. Fresh build produced the
  local CLI, Qt application, and installed Python runtime; attached bindings
  passed 8/8; and final `all_test.sh` passed 7,637 hygiene checks, all named
  CLI/Qt E2Es, 280 installed binding tests, and 214 Qt tests with one skip.
  One-time installed-Qt evidence selected `carboxyl` / `COOH`, publicly
  hit-selected the rendered group, and materialized it to `succeeded` /
  `updated`. Exact topology/exterior-bond semantics remain Rust permanent
  evidence. The probe-only `FAIL` records absent public topology reporting,
  while acceptance is `PASS`; M4 and full parity remain incomplete.

### Fixes and Maintenance

- Repaired grouped authoring-ribbon overflow so tab visibility changes coalesce into a
  convergent, group-local reconciliation instead of recursively re-entering Qt resize handling.
  Supporting actions now yield deterministically by declared `normal` then `required` priority.
  The ribbon no longer creates a competing QActionGroup or rewrites shared QAction icons; it
  remains a projection of feature-owned action, mode, and cancellation state. Ribbon layout
  rejects duplicate action placement within one tab, and focused tests now cover the real
  Bond-to-tab-switch-to-Escape owner lifecycle without brittle tab/count/index assertions.

- Repaired a P2 YAML-menu builder defect found during independent final review:
  every top-level menu now renders unattached before the first live menu-bar
  insertion. A late client-resolution failure therefore leaves the live menu
  bar unchanged; the successful-assembly marker remains unset for a clean retry,
  while a subsequent successful assembly still rejects duplicate rebuilds.

- Delivered the seventh attached compact-group recipe: `Cyano` / `cyano` /
  `CN`. The neutral attached `R-C#N` materializes with carbon focus, one
  recipe-owned normal triple bond, and retained exterior normal-single identity
  through generic Rust binding/session/Qt transport, with no chemistry-specific
  Qt branch. Semantic Rust tests and review prove the exact topology; the
  current build and public installed Qt Attach -> chooser -> materialize
  workflow passed, while Molecule Report showed `C3H5N` without claiming exact
  topology. `AcylChloride` and `Phenyl` remain before M4 completion.

- Updated the attached `Carboxyl` M4 decision to record its delivered bounded
  scope: neutral `R-C(=O)-OH`, carbon focus, ordinary single/double-bond
  topology, and generic Rust-issued key/label transport. M4 and full parity
  remain incomplete; `AcylChloride`, `Cyano`, and `Phenyl` remain unselected.

- Aligned active architecture and implementation plans with the current
  lifecycle vocabulary: graphics disposal, Rust SMARTS clearing, Qt SMARTS
  invalidation, removal of owned records and build output, and consumed
  prepared transitions. Archive navigation now includes the 2026-08-13 history.

- Rotated the complete 2026-08-24 day block into
  `CHANGELOG-2026-08i.md`, retaining the two newest
  day blocks in the active changelog and keeping authored Markdown below the
  source-file limit.

- Renamed current naming policy, local-build cleanup, and Python namespace
  boundary language around their exact responsibilities. The policy now directs
  authors to `cancel`, `consumed`, `dispose`, `invalidate`, `clear`, and
  `remove`; the build cleans obsolete owned state; and the permanent release
  input check reports the Ferrum Python namespace boundary.

- Renamed the live SMARTS lifecycle around its actual ownership boundary.
  Rust now clears published plans or derived receipts; Qt invalidates
  source/revision-bound capabilities before requesting that clear operation,
  then clears its copied dock state. The private PyO3 seam and user recovery
  text now describe clearing the active SMARTS result.

- Renamed generated linear-form cleanup around its actual document mutation.
  Transforms and geometry maintenance now describe removal of invalid
  Ferrum-owned generated records while retaining authored linear forms;
  function and semantic-test names use the same current terminology.

- Excluded `output_native_wheel/` from Graphify's production code map. The
  generated local wheel-build output is now outside architecture indexing,
  preserving source-only Graphify evidence and preventing build artifacts from
  distorting future task boundaries.

- Collapsed direct-bond migration chronology into one pre-production design.
  The in-process Rust/PyO3 pointer gesture, probe, and admission facade now uses
  unversioned modules, types, and methods; private V2 candidate names and public
  V3 handle names were removed without aliases. Durable V1 document/session
  operations remain versioned, and current Qt/API docs now describe the actual
  begin, resolve, generic-prepare, and generic-commit ownership chain.

- Reconciled the completed Ethyl compact-group slice across the active plan,
  usage, architecture, and public API contract. Renamed the production
  attached-choice gate around supported authoring capability, replaced
  serializer-substring Ethyl assertions with typed topology and geometry, and
  added the approved materialize/Undo/Redo/reopen session proof in a dedicated
  sibling test module.

- Made graph-inspection admission an explicit interchange-descriptor fact
  rather than an inference from decoder identity. Human `inspect-graph`
  response overflow now emits one standard-error refusal with no JSON stdout;
  JSON mode retains its bounded versioned refusal envelope.

- Strengthened the injected SDF inspection test to prove ordered typed title
  retention, including a retained empty title, without a native fixture.

- Made `inspect-graph` buffer complete text and JSON responses against its
  resolver-owned profile limit. Response overflow now writes one bounded typed
  `response_size_exceeded` JSON refusal without partial success output.

- Made `inspect-graph` resolve graph-inspection admission before source access.
  Unsupported resolved formats emit the deterministic typed refusal without
  opening a path or consuming standard input; declared CML and SDF profiles use
  their resolver-owned bounded source policies. Strengthened text/JSON
  shared-outcome coverage and documented the public inspection DTOs.

- Replaced the CML decoder's quadratic duplicate-bond scan with normalized
  endpoint-pair set membership. Reversed duplicate endpoints retain the closed
  `DuplicateBond` refusal while decoded bonds remain in source order.

- Made default `inspect-graph` source-molecule-ID text unambiguous: known IDs
  now carry a `known:` tag and escaped quoted value, so literal `unknown` and
  `unsupported` IDs remain distinct from those source-fact states.

- Made the interchange capability catalog copy canonical and display identity
  directly from resolver-owned CML, SDF, native-input, and output descriptors.
  Alias order remains lookup transport data rather than public identity policy.

- Made interchange capability catalog construction fallible and resolver-validated.
  A future input/output descriptor mismatch now reaches the CLI as a typed
  configuration error instead of panicking during `ferrum formats`.

- Repaired `inspect-graph` profile routing, source-limit enforcement, and text
  fact-coverage disclosure. The CML inspection profile now explicitly owns its
  decoder and runtime-free route; unsupported profiles refuse before decoding.

### Additions and New Features

- Added the attached `CH2OH` / `hydroxymethyl` compact-group slice. Rust owns
  the neutral attached `R-CH2-OH` recipe with carbon focus, ordinary single C-O
  topology, durable materialization, history, and reopen. Generic PyO3/Qt
  transport remains key-neutral; free placement remains Me-only. Existing
  public authoring/materialization E2Es already cover the generic workflow, so
  no catalog-specific public E2E was added.

- Added the reviewed attached `OMe` compact-group slice. Rust owns the neutral
  `R-O-CH3` recipe with oxygen attachment, candidate/render admission, durable
  materialization, history, and reopen; generic PyO3/Qt choice propagation and
  renderer-issued too-close-release pose normalization require no catalog-specific
  frontend path. Free compact-group placement remains Me-only. The permanent
  public E2E uses visible authoring and Molecule Report semantics rather than
  raw CDML, pixel, timing, or fixture assertions. `all_test.sh` exercises that
  public attached-OMe author/materialize/report contract against the built local
  runtime.

- Added [NAMING_CONVENTIONS.md](NAMING_CONVENTIONS.md) as Ferrum's canonical
  Rust/Python/PyO3 boundary-naming policy. Qt lifecycle APIs now distinguish
  close, cancel, clear, and dispose by their actual ownership transition.
  Reconciled the M4 plan with the
  then-delivered four-key attached `Me`/`NO2`/`Et`/`OMe` surface; the Ferrum
  E2E registry records its visible OMe author/materialize/report contract.

- Added the reviewed `Et` attached compact-group choice and its immutable
  materialization recipe. Existing materialization now expands attached or
  loaded sole-root ethyl groups into two neutral carbons joined by one normal
  single bond, while free placement remains Me-only and unreviewed catalog
  keys retain typed refusal.

- Added decoded-semantic `inspect-graph` support for SDF. The revised V1
  schema reports resolver-owned profile facts, zero-based record identity,
  typed source ID/title/property facts, exact counts, coverage, and
  normalization. SDF decoding uses the established native runtime and bridge;
  bounded presentation refuses before stdout on response overflow.

- Added runtime-free `ferrum inspect-graph INPUT --from cml [--json]` source-graph inspection for the CML/CML2 simple-molecule profile. It reports ordered exact counts, optional source molecule IDs, and explicit source-fact coverage through the versioned operation protocol; SDF refuses without runtime acquisition.

- Added the runtime-free `ferrum formats [--json]` discovery command. Its
  API-owned `ferrum-interchange-capabilities-v1` catalog joins each current
  conversion format once while retaining independent input/output names,
  format/profile IDs, aliases, suffixes, limits, policy, and runtime facts.
  The default is a concise human projection and `--json` emits the versioned
  snapshot; neither path reads a source, starts a conversion, constructs a
  document, or loads the chemistry runtime.

- Completed the API-owned M2.A1 conversion-input capability contract. Every
  enumerable input now exposes its direction, bounded source policy,
  explicitly applicable response bound, compression policy, semantic-loss
  policy, runtime requirement, aliases, suffixes, canonical name, and protocol
  format through the resolver. Native record conversion explicitly records that
  it has no input-owned response envelope, leaving the CML/SDF import bound
  available without inventing a CLI-local default for later `formats` output.

- Added Edit > `Reverse Selected Wedge Direction` for exactly one selected
  direct `w1` or `h1` bond. Rust owns the fenced endpoint swap, candidate
  validation, history, CDML persistence, and atomic refusal; the accepted
  action preserves bond identity, connectivity, wedge style, and selection.
  Durable Rust semantic/history/reopen, binding, and compact Qt
  click/reverse/eligibility/lifecycle coverage protect the recurring contract.
  The full visible Undo/Redo/save/reopen walkthrough remains one-time
  production-shaped integration evidence rather than a duplicate permanent
  E2E.

### Fixes and Maintenance

- Corrected two design defects that made a visibly selected wedge unreliable:
  the Rust interaction observer now issues one renderer-envelope-derived
  structural Bond target instead of a reachable same-bond `DisplayOnly`
  sibling, and the coalesced selection-refresh timer is owned by the tab widget
  whose current page it queries. Qt continues to select by durable object ID;
  a source ID crosses the boundary only to construct the Rust reversal
  operation.
- Named the reversal factory input `source_bond_id` across Rust, PyO3, and Qt,
  and narrowed the Qt refusal boundary to the expected document-tab error so
  unexpected defects remain visible. Removed a private-field-coupled timer
  test that did not exercise the claimed document-tab retirement behavior.

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
