# Changelog

Earlier history is in [CHANGELOG-2026-08j.md](CHANGELOG-2026-08j.md). Its archive navigation
continues through [CHANGELOG-2026-08i.md](CHANGELOG-2026-08i.md) and
[CHANGELOG-2026-08h.md](CHANGELOG-2026-08h.md).

## 2026-08-27

### Additions and New Features

- `LocalDocumentOpenCatalogV2` is sole Rust File/Open authority for native CDML,
  decoded SVG, and registry `DocumentImportNew` issued descriptors/limits.
- Open creates/replaces a document; File > Import SDF remains distinct current-document insertion.
  Evidence: 32 PyO3, 18 Qt File/Open, public SDF-import E2E; M2 remains open.
- M4.C identifiers, Rust periodic picker, and M6 fence-owned selected delivery;
  workers admit selection once, then deliver by captured fence/receipt; 57 Qt/PyO3 pass.
- Added Rust-owned Semantic Render Palette V3. Render operations now carry a
  frozen tagged `RenderPaintV3` value: exact authored RGB, semantic document
  roles, or reserved validated element roles. Headless SVG, PDF, and PNG use
  the Rust export palette only, while PyO3 exposes read-only kind, export RGB,
  role, and element facts for the Qt display-palette owner.
- Added the bounded M6 command-palette V1: portable `Ctrl+K` (`Cmd+K` on native
  macOS) and YAML-owned **View > Commands > Command Palette...** provide a
  registry-derived search over live action labels, help text, and stable IDs.

- Delivered the bounded M2 CDXML simple-molecule input profile through the
  Rust decoder, interchange registry, generic CLI/PyO3 ingress, and existing
  Qt File/Open worker. The profile accepts the exact current vendor external
  DTD marker without resolution, imports unprefixed simple fragments as
  ordered records, reports lexical/view losses, refuses unsupported chemistry
  before publication, and remains input-only with CDML as the local save
  format. The added decision records the exact grammar, security/resource
  boundary, provenance, exclusions, and permanent versus one-time evidence.

- Added Ferrum-owned CDXML E2E guidance. The permanent CLI and Qt gates cover
  public semantic import/refusal behavior with inline temporary inputs; a real
  16:10 outer-window screenshot and keyboard/accessibility walkthrough remain
  release evidence rather than pixel or timing assertions.

- Extended bounded native CDXML with C2 `Charge` and `Isotope` atom facts:
  canonical ASCII charge in `-128..=127` and nonnegative isotope mass in
  `0..=32767`, zero normalization, durable CDML lowering, and typed
  `InvalidScalar` refusal. It adds no client route, source-provenance store, or
  CDXML writer.

### Behavior or Interface Changes

- Added Rust-issued `MoleculeContentBoundsV1` to each frozen molecule render
  entry at the PyO3 boundary. Qt now uses a dedicated noninteractive molecule
  ownership root for content fitting and disposal, while atom and bond render
  items remain ordinary independently selectable children.

- Command Palette results now show a validated YAML-derived primary placement
  breadcrumb, prefer ordinary menu paths over ribbon fallbacks, and rank direct
  label/action-ID intent ahead of help-text or subsequence-only matches while
  preserving stable live-action ties and visible unavailable commands.

- Delivered the approved nominal `DocumentDisplayRefreshableV1` `abc.ABC`
  contract. Production registrants and valid test helpers declare membership,
  structural look-alikes are rejected, and retained refreshables use direct
  modern annotations without importing `typing`.

- Command palette search preserves focus, uses bare Up/Down for result
  selection, Return for exact live-action invocation, and Escape for invoking
  focus restoration; disabled commands stay visible with an unavailable
  explanation and modified arrows retain normal text-field behavior.
- Live shortcut ownership now has an atomic prospective-set preflight for
  startup, user reassignment, and default reset. It refuses collisions before
  preferences, manager bindings, or QAction shortcuts change.

### Fixes and Maintenance

- Repaired format-neutral interchange placement in Rust. Shared CML, CDXML, and
  SDF preparation now observes the session paper and centers imported records on
  that page instead of scaling them around a hard-coded scene origin.

- Repaired molecule projection ownership without a `QGraphicsItemGroup` or
  Qt-side geometry authority. The completed detached hierarchy is installed only
  after validation, root bounds come from Rust's lowered draw stream, and member
  selection is no longer swallowed by a grouping item. Removed the unused legacy
  overlay module that still constructed the superseded parallel group projection.

- Scoped PyO3 extension-only link mode to the production extension build instead
  of the workspace dependency graph. Rust binaries and tests now link normally on
  macOS, while the installed module retains the required dynamic extension mode.
  The Maturin project now targets the dedicated `api-python` wheel owner directly.

- Cleared strict Clippy debt across the affected Rust API graph: named conversion
  and materialization input bundles replace argument lists, the protocol success
  payload is boxed, and byte-pair parsing uses the current slice API.

- Repaired the real GUI-tour capture contract. Documentation uses a transient
  light theme without changing `QSettings`, frames content through the public
  view transform, verifies imported CDXML semantics and visibility, prints each
  scene phase, fails fast on unexpected modals, and keeps ring, arrow, and vector
  examples on the visible page.

- Repaired the window-owned Properties action so dock replacement and deferred
  Qt deletion cannot leave a registered command pointing at a destroyed dock
  action. Blank-canvas Select Structure also retires Rust-owned direct-root
  selection before durable arrow and vector documentation captures.

- Repaired the successor-window action-lifecycle regression itself: manually
  deferred-deleted owners are no longer also registered for `pytest-qt` teardown,
  and the isolated one-action Command Palette receives its explicit placement
  projection instead of pretending to be a complete production registry.

- Repaired the cyclohexane-attachment documentation route to use an eligible
  host molecule and the template route to place the real Rust-owned Furan system
  entry with visible catalog provenance.

- Repaired the real GUI-tour driver to use the public Rust-backed **Insert
  Template...** palette, visible family/category/search/result/detail provenance,
  and theme-owned document colors. Scene-specific preparation now preserves the
  intended Add Atom and selected-nitrogen states, stages attached cyclohexane on
  a visible host pair, and uses the normal blank-canvas selection path to retire
  presentation feedback before arrow/vector capture.

- Repaired command-palette placement projection validation for the declared
  `file.recent` dynamic menu. The public lazy loader now carries the
  `ActionRegistry`'s registered dynamic-menu IDs through YAML validation, so a
  standard `MainWindow` palette can construct and refresh against the current
  menu and ribbon resources while unresolved dynamic-menu references remain
  strict failures. Focused regressions also protect ribbon-only breadcrumbs and
  honestly unplaced commands.

- Repaired `ActionRegistry` QAction lifecycle ownership. Every live binding now
  receives an opaque retirement token from its feature-owned QObject's
  `destroyed` signal, so destroyed-window actions retire their exact Qt and
  reverse-identity entries without letting late callbacks remove a successor
  binding that reuses the stable ID. Feature-owned declarations retire with
  their QAction; portable declarations remain available for a later binding.
  Focused regressions now prove both the retired public live-action projection
  and portable declaration rebinding/dispatch behavior, and `pytest-qt` is
  declared as the direct development dependency for those lifecycle fixtures.

- Repaired permanent status-bar View controls at ordinary 1440x900 window size.
  The shared seam no longer hides the Ferrum-owned action client or installs a
  duplicate legacy zoom widget, so Reset zoom, Zoom to Content, minus/plus,
  slider, and the observed zoom value remain visibly exposed through their
  existing actions and YAML-owned command surfaces.

- Repaired the real CDXML GUI screenshot assertion to treat the Rust-issued
  local-document origin token as opaque identity. The staged File/Open route now
  retains token presence while proving ChemDraw provenance through the public
  converted-source description and the editable projected C-O document.

- Repaired the real GUI screenshot driver to bootstrap one application-owned
  `ThemeManager` and pass it explicitly to every disposable `MainWindow`, matching
  production theme initialization while preserving the 13-scene staged capture flow.

- Repaired the document-tab scene-selection bridge lifecycle. The tab now owns
  its exact scene connection, retires it before projection or view disposal,
  restores the prior bridge on replacement refusal, and forwards selection only
  through a live QObject slot.

- Repaired direct-root interaction overlays to use one scene-owned
  `QGraphicsPathItem` with one rectangle subpath per Rust-issued bound, including
  the line-tool preview contract. The transient root retains its existing preview
  material, ordering, non-hit-test, refresh-registration, and disposal contracts
  without parented child graphics items.

- Repaired accepted native Text placement dispatch so the existing Rust preview,
  intent update, and completion route execute after dialog acceptance; dialog
  cancellation still disarms without mutating the document.
- Repaired native Text placement to pass only RichTextDialog-owned changed
  root-font values to Rust. Unchanged themed defaults retain their Rust-issued
  paint, without duplicating palette resolution in the Qt pointer.

- Made the Ferrum `MainWindow` constructor require a concrete `ThemeManager`
  before the application shell exists. Every successfully constructed window now
  owns an applied typed document-display palette, so local document Open cannot
  reach a missing-theme display failure after admission.

- Completed the nominal retained-root palette contract for ordinary native
  projections. Molecule plans, paper, and the persistent hex grid now declare
  `DisplayPaletteRefreshable`; the production builder admits paper alongside
  plans, and a live native projection test proves palette material changes in
  place while issued geometry, IDs, and revision remain stable.

- Replaced structural `typing.Protocol` admission for retained presentation
  roots with the nominal `DisplayPaletteRefreshable` contract. Vector, Plus,
  Text, and preview roots now declare the shared refresh ownership directly;
  structural look-alikes remain outside the closed scene boundary.

- Repaired Semantic Render Palette V3 so semantic paint values store only their
  role or element identity. Rust now derives export RGB through
  `DocumentExportPaletteV1`; path/vector/text preview DTOs publish tagged
  paints; built-in Haworth ink is semantic foreground; and the path grammar is
  canonically named V3. SVG, PDF, PNG, Rust, and PyO3 boundary proofs cover
  the repaired contract.

- Closed the Qt retained-root palette contract. Persistent and detached-preview
  roots now require `refresh_display_palette` at registration, controller and
  scene refreshes dispatch through that typed contract, and native runtime
  coverage proves vector, Plus, Text, and preview-root materials update without
  replacing renderer-issued identity or geometry.

- Closed the Qt transient-overlay palette contract. Selection, line/bracket,
  rotation, direct-root, path/vector/text, direct-bond, Haworth, and precommit
  previews now use the tab-owned typed refresh registry and release that
  registration before graphics disposal; refresh replaces only retained Qt
  material and preserves Rust/session and gesture facts.

- Repaired direct-glycosidic Haworth preview disposal to release its tab-owned
  display refresh registration before graphics disposal. The route now has a
  focused lifecycle regression covering registration, ordered release, and a
  later theme refresh.

- Rotated the complete 2026-08-25 day block into
  `CHANGELOG-2026-08j.md`, retaining the two newest day blocks in the active
  changelog and preserving reverse-chronological archive navigation.

### Decisions and Failures

- Rejected a reaction-specific command-palette E2E as redundant. Focused Qt
  tests and the registered reaction E2E already own permanent semantics; native
  shortcut dispatch and accessibility are one-time real 16:10 desktop evidence.

- Rejected the deprecated global PyO3 `extension-module` Cargo feature. Its
  workspace-wide no-libpython behavior broke ordinary Rust test targets; the
  extension build now opts in through its dedicated environment instead.

### Developer Tests and Notes

- Built the complete local CLI, PyO3 extension, pinned native RDKit runtime, and
  PySide6 launcher successfully after the Rust, Cargo, and Qt changes. A focused
  final binding/CDXML/theme/lifecycle run passed all 74 tests.

- `./all_test.sh` exited 0: 8,019 repository-hygiene tests, every registered CLI
  and Qt E2E, 286 installed PyO3 tests, and 342 Qt tests passed with one
  intentional skip.

- `cargo test --workspace` exited 0 after the link-mode repair. Affected-package
  receipts include 117 default API and 132 renderer tests; format checks and
  strict API, renderer, and PyO3-feature Clippy gates also passed. A clean Maturin
  wheel build and isolated CPython 3.12 import passed through the checked-in project.

- Seven independent post-fix reviews completed. The stale Qt contract/ledger,
  dynamic import, dead parallel projection, and split Maturin ownership findings
  were repaired; the test auditor found no fragile changed coverage.

- Regenerated all 13 GUI-tour PNGs transactionally at 1440 by 900. Every scene
  passed its semantic postcondition; image-by-image agent review accepted the
  candidate set as complete, legible, page-contained, and uncropped. Final human
  release sign-off remains separate.

- The complete approved document-display focused suite passed: `28 passed` for
  the palette and Qt theme-toggle files, including direct public-registry proof
  that `DocumentDisplayDelegatingRefreshableV1` forwards one shipped palette to
  its renderer item. The structural look-alike boundary asserts only the
  approved `TypeError` class, not diagnostic prose. Ten exact typing, pyflakes,
  indentation, import-requirements, and source-line nodes cover
  `document_display_refresh.py` and `test_document_display_palette.py`.

- Reconciled the paused delivery checkpoint. `ActionRegistry` lifecycle
  retirement and the nominal document-display refresh boundary are delivered
  and independently accepted at code level. Exact lifecycle receipts remain:
  initial action-registry/command-palette validation `17 passed` plus 8 targeted
  hygiene checks; review-fix validation `18 passed` plus 9 targeted checks; and
  final independent lifecycle review `2 passed, 11 deselected in 0.12s`.
  Document-display focused evidence remains `28 passed` plus 10 exact hygiene
  nodes. The last automated 13-scene run completed, but human visual review
  rejected eight frames; later capture-driver/catalog and command-palette
  hierarchy/relevance repairs have not been recaptured or independently finally
  reviewed. The command-palette repair recorded `21 passed` and `2996 passed`
  focused receipts. The next aggregate run includes the new guidance-document
  format gate. Aggregate green, visual acceptance, post-fix audit, broader
  parity-ledger reconciliation, separate M5A approval, and full parity remain
  open.

- Focused action-registry/keybinding and command-palette Qt validation passed
  after the lifecycle repair, including a deterministic two-window successor
  action-ID reuse regression, public live-view retirement, and portable
  declaration successor rebinding/dispatch. Targeted typing, indentation,
  pyflakes, source-line, and import-requirements checks passed for the changed
  files.

- Paused the stabilization audit after documenting its non-acceptance result in
  [TODO.md](TODO.md) and [ROADMAP.md](ROADMAP.md). Resume with explicit
  `ActionRegistry` destruction retirement, one successor-window command-palette
  regression, remaining lint repair, capture, and full gate; reconcile parity-ledger receipts.

- Completed a six-pass independent stabilization audit; its actionable findings are resolved
  above. The approved in-progress M5.A
  [Template Catalog V1](active_plans/decisions/m5_template_catalog_v1.md) replaces
  Qt filename-derived authority with a bounded immutable Rust snapshot, native
  snapshot-fenced placement, and Rust publication receipt without expanding M2/M4.
  Its next foundation prerequisite is the approved
  [Qt Operation Lease Registry](active_plans/decisions/qt_operation_lease_registry.md): Patch 1
  atomically replaces catalog mixins with a controller and pure Qt lifecycle registry, then Patch 2
  proves Local Open source retention and truthful delivery cancellation. Neither patch changes
  Rust/PyO3 in Patch 1 or closes M5.A, M5, native accessibility/visual acceptance, or full parity.
  The registry decision records the fixed ownership, scope, and verification boundary.

- Pre-repair receipt: paused after the delivery-stabilization checkpoint with the
  exact remaining evidence gap recorded in [ROADMAP.md](ROADMAP.md),
  [TODO.md](TODO.md), and [GUI_TOUR.md](GUI_TOUR.md). The fresh
  local build, 283 installed PyO3 tests, 283 Qt tests with one intentional skip,
  and registered E2E suite passed. The close-out aggregate run recorded 7,922
  passes and two failures: one prohibited `import typing` and one premature tour
  embed for an unrecorded focused screenshot; the embed was removed after the
  receipt. Eleven 1440x900 GUI images exist locally, while the complete 13-scene
  run remains blocked because a later command-palette window enumerates a
  `QAction` destroyed with an earlier window. The later lifecycle entry resolves
  that specific blocker. Full parity and the next M5A catalog package remain
  open.

- Command-palette focused Qt/keybinding evidence: `32 passed in 1.56s`.
  This bounded evidence does not claim aggregate `all_test.sh` success, GUI
  screenshot proof, full M6 usability, or complete feature parity.

- Registered the existing CDXML CLI E2E in the permanent E2E runner. Corrected
  current product guidance and the capability ledger to describe bounded
  input-only CDXML Open, CDML output/save behavior, descriptor operation
  eligibility, canonical loss ordering, and remaining refusal boundaries.
  Historical migration reports now identify their dated CDXML refusal state and
  link to the current decision and capability ledger.

- Recorded final bounded-CDXML receipts: post-audit `./build.sh` exited 0;
  the registered `tests/e2e/run_all.sh` exited 0, including CDXML; staged
  Python bindings passed 281 tests; Qt passed 238 tests with one intentional
  skip; focused chemistry and API libraries passed 124 and 117 tests; and
  `cargo check --workspace` passed. `./all_test.sh` is not aggregate-green:
  it reported 7,759 passes, then stopped at five Markdown-link failures. Each
  canonical link targets the present CDXML decision artifact but is absent
  from the tracked-file catalog, so later aggregate phases did not run through
  that script. The later phases were run directly and passed as recorded
  above. M2, full Rust/OASA/BKChem parity, and real 16:10 GUI/accessibility
  evidence remain open.

- Recorded C2-focused receipts: formatting succeeded; CDXML-focused chemistry passed 17 tests; the built public CLI E2E succeeded; scalar-contract tests passed 3 tests; and the chemistry library passed 127 tests. These do not make the aggregate suite green or close M2, parity, or GUI evidence.

## 2026-08-26

### Additions and New Features

- Reconciled the delivered M4.A and bounded M6 foundations across the active
  plan, usage, API contract, and architecture documentation. Molecule Report
  now documents `ferrum document command document.molecule.report.v1 <input>`
  as a thin positional adapter over the generic protocol request and envelope.
  Persisted atom, bond, and molecule projections require `DocumentObjectIdV1`;
  the authoritative `document_object_index` now distinguishes unknown targets
  from invalid persisted identity metadata, while preview-local keys remain
  separate. Context actions reuse registered QActions through YAML placement,
  with canonical `edit.delete_selection` shared by keyboard and context input.
  M4, full parity, and manual 16:10/accessibility evidence remain open. This
  milestone retains the recorded successful `build.sh` and `all_test.sh` receipt
  of 7,775 hygiene tests, all registered CLI/Qt E2Es, 280 binding tests, and
  229 Qt tests with one intentional skip.

- Delivered `PARITY-M4.A`: `document.compact-group.attach.v1` now runs through
  both `ferrum protocol run` and `ferrum document command`. The request supplies
  fenced CDML, a document-owned molecule/anchor pair, a closed catalog key, and
  finite release coordinates. Rust owns pair-local target authority, chemistry,
  geometry, renderer admission, durable IDs, and history; adapters only load
  requests and present envelopes. The versioned receipt returns source/target
  facts, the allocated group ID, committed CDML, and a reusable stateless fence,
  while intentionally omitting release, pose, overlay, and pending-session
  facts. Multi-reviewer repair moved shared selector/digest parsing from the
  materialization-owned module to neutral crate-private
  `document_request_parse_v1`, used by the feature-gated live-document caller;
  renamed the misleading `Result<()>` chemistry predicate to
  `require_attached_compact_group_chemistry_support_v1`; and rewrote attachment
  tests around durable IDs and chemistry roles instead of ordinal, count, raw
  `type="n3"`, or repeated availability-inventory assertions. The public E2E
  uses Methyl through both transports, and the CLI/active-plan documentation
  now agrees. The source-line gate passed 1,280 checks; focused attachment tests
  passed 17 tests; the full `ferrum-api` library passed 104 tests; and `build.sh`
  passed, including the feature-gated PyO3 path. Final
  `source source_me.sh && ./all_test.sh` validation passed 7,775 repository
  hygiene tests; all registered CLI and Qt E2E phases, including
  `ferrum-compact-group-attachment-cli-e2e-v1`; 280 native binding tests; and
  229 Qt tests with one intentional skip. M4, full parity, and manual
  16:10/accessibility evidence remain open.

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

### Fixes and Maintenance

- Added a direct `InvalidIdentity` reaction protocol-mapper regression. It locks the stable
  invalid-document envelope plus `InvalidRequest` / `RefreshAndRestart` recovery contract
  without constructing an otherwise impossible post-admission corrupt session.

- Split direct CDML reaction semantics from fallible durable identity binding. Reaction list and
  observation now surface corrupted retained reaction/member identities through the existing
  `InvalidIdentity` refusal instead of silently omitting durable relations.

- Reconciled the source-only identity ledger after WP-ID-2. Authored CDML source IDs are exact,
  nonblank, and document-unique; no lexical source-ID grammar or guessed `NCName` gate is
  currently approved. Durable reaction observation/listing uses the retained-tree relation and
  private fallible binder without an XML/redecode or source-ID reverse bridge. Full parity,
  CDXML, and manual 16:10/accessibility work remain open.

- Moved reaction-member observation and list construction onto the retained
  durable direct-reaction relation. Selection now resolves members by persisted
  document object ID while preserving existing session fences, renderer admission,
  paint order, and display-only diagnostics.

- Moved modeled directional-bond compatibility into `MolGraph::new`: native
  W/H and E/Z carrier directions now require non-aromatic single bonds at the
  shared graph boundary, with FCM1 ingress regression coverage. Removed the
  unreachable document-only directed-depiction refusal and its API mapper.

- Added the public fallible `MolBond::directed` contract for the four modeled
  directional non-aromatic single bonds. CML lowering now enters the owned
  model through that constructor and preserves the existing redacted import
  refusal boundary; native ABI decoding retains its crate-private path for
  unsupported native facts. Focused chemistry and document tests cover direct
  construction, CML lowering, and producer-neutral ordered wedge/hash admission.

- Reconciled the active CML grammar with the delivered direct W/H slice. CML1
  accepts one final direct `builtin="stereo"` and CML2 accepts one direct
  `<stereo>` child only on a single bond; `W` and `H` retain endpoint-ordered
  authored solid/hashed depiction without fabricating tetrahedral, parity, or
  E/Z semantics. `MolBond::directed` and `MolGraph::new` own modeled
  direction/order compatibility; a mutated FCM1 invalid native direction is
  `MalformedNativeResponse`, and a public duplicate-W/H request is atomically
  `InvalidScalar` with no conversion outcome. Focused receipts record 110
  chemistry and 503 document tests plus `cargo check --workspace`; M2, full
  parity, and manual 16:10/accessibility proof remain open.

- Bound the permanent native selection-context focus test to the deterministic
  keyboard context routes (`Menu` and `Shift+F10`) that Qt's offscreen backend
  can model. The right-button route remains covered by the registered-action
  deletion contract; one-time real macOS Cocoa evidence separately confirmed
  focus restoration for all three public routes.

- Corrected M4.A documentation drift. Usage and active plans now describe the
  delivered stateless attached-group contract consistently: one generic
  operation with generic-protocol and named-document transports, document-owned
  molecule/anchor authority, typed envelopes that exit `0` for accepted and
  refused outcomes, and nonzero status reserved for operational failure. The
  plans keep free compact-group placement, manual 16:10/accessibility evidence,
  M4, and full parity open.

- Removed the superseded local action refresh from explicit `Cancel Tool`.
  Completed mode-state publication now drives the single window-owned
  capability refresh, avoiding a duplicate recomputation while preserving
  cancellation and synchronization.

- Connected completed active-tool publication to the window's centralized action
  availability refresh. Escape and other completed tool transitions now
  immediately recompute local-document capabilities such as `Open in Current
  Tab...` through their existing owner.

- Corrected the README and provenance links to the existing AGPL and LGPL
  license texts. Ferrum remains AGPL-3.0-only and Ferrum-Chem remains
  LGPL-3.0-only.

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
  clients. Every pointer action now prepares an immutable one-shot command
  before cancelling the prior tool; Attach freezes Rust selection facts before
  cleanup, re-fences later chooser acceptance, and preserves its exact refusal request.

- Repaired the Atom Oxidation State public Qt E2E's fluoride selection by
  clicking an interior point of the live painted selectable glyph rather than
  assuming the atom anchor is a hit target.

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
