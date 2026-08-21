## 2026-08-19

### Reaction composer focus and diagnostics

- Strengthened the reaction-composer end-to-end proof to require an isolated installed site for
  both Ferrum wheels. The public-action workflow now verifies both import origins, moves real Qt
  focus to a separately shown external window, hides the Ferrum window normally, proves each
  terminal path leaves CDML untouched and disposable state cleared, then reopens and creates a
  Rust-native reaction.
- Made the modeless Rust-authoritative reaction composer terminally retire when Ferrum loses its
  document-window focus. The cancellation clears only disposable role and renderer-selection
  state, retains authoritative CDML unchanged, refreshes actions, and restores the active canvas
  focus when Ferrum remains active.
- Separated selected usable reaction roots from Rust-issued unavailable-root diagnostics. The Qt
  composer now filters diagnostics to the current selected keys and displays each exact reason,
  recovery token, and recovery instruction outside every selectable role list.
- Added Qt coverage for focus-loss no-mutation retirement and inaccessible vector diagnostics with
  exact recovery guidance, plus the complementary canvas-focus transition that keeps a live form.
- Added the public offscreen reaction-composer E2E: it proves a window-deactivation cancellation
  leaves CDML unchanged and state-free, then reopens the composer, creates a reaction, and reopens
  the native saved CDML through Rust.

### Developer Tests and Notes

- Corrected the Qt reaction composer live-window test teardown to persist its accepted native
  transaction to `tmp_path` before closing the dirty tab. This preserves the real unsaved-change
  policy while preventing an unattended modal close confirmation from hanging focused validation.

- Added an isolated external Rust-consumer proof for `ferrum-document-render` reaction
  capabilities. It compiles the supported opaque begin/prepare/diagnostic/commit flow and uses
  per-attempt Cargo status checks to reject construction, clone, dereference, serialization,
  conversion, preview/receipt imports, and candidate CDML, render-plan, or receipt extraction.

- Added an isolated external Rust-consumer compile proof for the reaction ownership inversion.
  It compiles the documented generic complete-CDML session transaction while separately proving
  that former document reaction request/candidate imports and prepare/commit calls do not compile.

### Catalog renderer-preview bridge

- Added the renderer-selected-root overlay primitive and catalog placement V2 bridge. V2 constructs the renderer-admitted candidate during preview and transfers that exact private receipt to commit, preventing a later compiler pass from diverging from the preview.
- Made V2 hover previews lease-bound: a later renderer-issued preview, explicit release, or gesture cancellation invalidates the earlier opaque candidate without mutating the document. The frozen PyO3 overlay now copies the immutable renderer plan safely, and the Qt palette explicitly retires the gesture on every cancellation route.
- Preserved V2 catalog prepared receipts across every V1 refusal, including foreign, stale, validation, and transaction refusals; only a successful fenced catalog transaction consumes its receipt. Removed the public V1 point-and-segment preview and commit route so Python, CLI, and Qt use the opaque renderer plan exclusively.
- Added installed-wheel Haworth V2 hover coverage that proves the Qt transient group projects every renderer batch, retains filled directed-wedge paths, cancels without mutation, and commits canonical `q1`/`w1` CDML.

### Reaction renderer-preflight bridge

- Added the first modeless Qt `Create Reaction...` composer. It consumes only
  Rust-issued complete-root choices and opaque `create_reaction_v1` commits,
  assigns visible durable role lists without Qt scene-role inference, and
  reselects accepted members from a fresh renderer observation.

- Replaced the reaction bridge's namespace-blind XML tokenizer and lexical `</cdml` splice with one document-owned direct-CDML semantic index and typed detached reaction append. Authoring, deletion protection, and identity reservation now share the same unnamespaced-or-CDML-namespace rule, reject foreign or nested lookalikes without mutation, and accept namespace-prefixed CDML roots.
- Sealed reaction candidate diagnostics across the document, renderer bridge, and API facade. Public `Debug` output now exposes only a reaction ID and capability state, never canonical candidate CDML, XML fragments, or renderer admission data; an external-surface regression proves formatted diagnostics cannot be routed into a generic CDML commit.
- Closed the reaction bridge's raw-candidate bypass. Reaction candidates now retain renderer admission and canonical CDML privately, and the bridge can commit them only through the opaque candidate transaction rather than the generic complete-CDML route. Prepared receipt refusals now preserve their capability until a successful fenced commit consumes it.
- Marked legacy polygon roots as excluded from complete renderer admission, so reaction creation fails before mutation while compatibility CDML remains loadable.
- Moved authored reaction creation behind `ferrum-document-render`'s renderer-preflighted, session-bound, one-use transaction bridge. The document crate now produces only immutable canonical reaction candidates; Python and CLI routes expose typed reaction refusals rather than a raw document mutation path.
- Guarded presentation deletion against reaction-referenced arrows, standalone condition text, and plus signs, preventing dangling durable role references in one atomic refusal.

### Haworth biomolecule catalog

- Added four sealed, Ferrum-authored D-glucose Haworth catalog entries with literal translation-only CDML compilation that preserves directed `n1`, `q1`, and `w1` depiction facts.

### Template catalog palette

- Corrected Ferrum-authored purine template geometry to a centered, non-crossing
  fused five/six-member heterobicycle. Every directly bonded pair is now a
  finite 40-point segment. Catalog tests now table-check every shipped recipe's
  ordered elements, bond endpoints and orders, anchor centroid, emitted
  geometry, and full purine namespace collision reservation through final atom
  and bond identifiers before candidate assembly.

- Corrected catalog-to-authoring-tool handoff ordering in the Ferrum ribbon.
  Catalog placement now retires on each shared checkable QAction's
  `toggled(True)` transition, before that action's existing `triggered` owner
  can install a replacement viewport filter. Direct Draw Plus and compact
  More Tools Draw Bond E2E coverage proves their pointer gestures commit and
  that Escape then cancels only the incoming tool.

- Corrected catalog placement ownership in the Qt Authoring Ribbon. Starting
  the Rust-owned catalog gesture now terminally retires Select Structure with
  every other competing authoring controller, and Select Structure likewise
  replaces catalog placement. Live-window E2E coverage proves that neither
  cancellation nor commit revives the prior structural event filter or action.

- Added an accessible `Insert Template...` palette over the Rust-owned shipped
  catalog. Qt filters immutable Rust summaries, paints only Rust-issued opaque
  placement previews, and commits through the required prepared receipt.

- Corrected template-catalog Qt coverage to construct the ordinary public
  `ferrum-qt` window and its live Authoring Ribbon. Added an installed-wheel
  E2E that triggers the ribbon's `Insert Template...` QAction, cancels once,
  searches and selects Benzene through the actual modal controls, places it by
  click, then moves, undoes, saves, and reopens the Rust-owned document.

- Restored visible template hover previews in the public Ferrum window. The
  catalog placement owner now enables mouse tracking only for its active
  opaque preview and restores the canvas's former setting on cancellation or
  commit.

### Architecture corrections

- Added closed `catalog.list.v1` and `catalog.insert.v1` CLI/protocol routes.
  Listing exposes only immutable Ferrum-owned summary and provenance facts;
  insertion accepts an explicit snapshot/revision/digest fence plus catalog ID
  and finite anchor, then uses the existing opaque renderer-preflighted Rust
  placement capability. No catalog recipe, fragment CDML, or direct commit
  route is exposed through the protocol.

- Added the first Ferrum-authored shipped template catalog slice:
  `system/rings/benzene` has immutable provenance metadata, a native regular
  40-point recipe, opaque placement capabilities, and renderer-preflighted
  one-history CDML insertion. It does not import OASA or BKChem template data.

- Corrected vector commit-recovery wording so a successful authoritative Rust
  refresh invites reselection, while a failed refresh alone says that recovery
  is still required before saving or editing.

- Added mutually exclusive Rust-owned `Draw Line`, `Draw Rectangle`, `Draw Square`,
  `Draw Oval`, and `Draw Circle` Qt tools. Qt carries only opaque vector gesture,
  preview, and renderer-preflight receipt handles; Rust resolves shape constraints,
  appearance, renderability, and the one-use commit. The controller paints only
  Rust-issued overlays and cancels without mutation on Escape, focus loss, tool
  changes, or tab changes.

- Replaced the rejected document-owned vector preflight seam with
  `ferrum-document-render`. The bridge now owns vector gesture capabilities,
  candidate CDML, renderer preflight, one-use receipts, and the subsequent
  generic fenced complete-CDML commit. `ferrum-document` exports no
  vector-specific gesture or pending-commit surface; its complete-CDML
  transaction remains a documented generic compatibility operation.
- Hardened the vector bridge receipt: it now binds a stable construction-time
  session origin, gesture nonce, candidate digest, root ID, static preflight,
  and a no-exclusion composed render plan. Renderer exclusions, stale prepared
  receipts, foreign sessions, and replayed capabilities all refuse before the
  generic commit.

### Fixes and Maintenance

- Replaced the platform-owned Reaction Inspector deletion prompt with an accessible Ferrum-owned
  confirmation dialog. It explicitly states that only the reaction definition is removed while
  member roots remain, exposes named Cancel and `Delete reaction definition` controls, and routes
  mutation only after the explicit destructive decision. The installed-wheel Inspector E2E now
  saves every dirty test tab before normal window teardown, preserving the product's unsaved-close
  policy without leaving an unattended close refusal modal.

- Closed reaction-authoring observation diagnostics. Every renderer-admitted direct root now
  becomes either a reaction-member choice or one deduplicated immutable exclusion with a durable
  key, closed reason, and closed recovery. Renderable vectors remain explicitly display-only;
  missing or ambiguous semantic identity and renderer/semantic kind disagreement are distinct
  repair-document diagnostics without CDML or mutation capabilities crossing PyO3.

- Protected reaction-owned presentation records from generic deletion. A shared
  direct-CDML semantic reaction-reference graph now guards arrow, text, and plus
  deletion before detached mutation, for bridge-authored and compatibility-loaded
  reactions. Rejected single and batch requests leave the session unchanged;
  unreferenced multi-root deletion remains one atomic history entry.

- Updated Rust protocol and native CLI catalog coverage for the sealed Haworth
  biomolecule slice. Tests now assert its public Ferrum-owned summary and
  provenance facts, plus missing and contradictory filter results, rather than
  the obsolete empty-biomolecule expectation.

- Corrected `catalog.list.v1` filtering at the Rust protocol boundary. Family
  and category are exact closed identities; normalized query matching considers
  only immutable emitted summary names and IDs. Combined filters intersect and
  no-match listings succeed with an empty summary array.

- Added an installed-wheel PyO3 contract for the Rust-owned shipped-template
  catalog. It proves summary-only immutable catalog metadata, opaque
  renderer-preflighted placement handles, one-revision commit and undo,
  foreign/stale/replay refusal without mutation, and descendant-ID collision
  avoidance without exposing recipe CDML or a direct commit bypass.

- Corrected shipped-template insertion identity allocation. The Rust benzene
  compiler now reserves its molecule, atom, and bond declaration IDs as one
  namespace before detached candidate construction and renderer preflight, so
  opaque preserved literal IDs cannot force a late failed candidate.

- Corrected the native bracket authoring E2E cancellation oracle to compare
  Rust's authoritative pre-gesture snapshot rather than a noncanonical source
  fixture. The proof now demonstrates real active-tool Escape cancellation
  without treating load-time CDML canonicalization as a mutation.

- Strengthened the transient Qt paper-local hex grid with theme-owned lattice tokens,
  cosmetic raster-stable strokes, and durable vertex anchors. Added viewport-pixel coverage
  at 60, 100, and 200 percent in both themes so contrast is proved after Qt composition,
  without changing Rust geometry, preferences, snapping, or document state.

- Repaired Authoring Ribbon handoff from Place User Template. Its real placement
  owner now terminally releases its viewport filter on the QAction's exclusive
  `toggled(False)` transition, before direct or More Tools authoring gestures
  receive canvas input. Template placement no longer disables those replacement
  authoring actions, while non-authoring document commands remain protected.

- Corrected the Authoring Ribbon lifecycle coordinator. Its exclusive group now
  reflects the established live QAction activation sequence without issuing a
  late Cancel or restoring a disconnected checkmark; Place User Template now
  participates in that same visible ribbon and More Tools route.

- Made ambiguous keyboard Draw Bond coordinates a terminal typed refusal. The
  adapter now clears its active gesture and action without mutation, returns
  canvas focus, and directs the author to choose a distinct atom location.

- Repaired keyboard Draw Bond endpoint discovery for implicit-carbon atoms.
  Keyboard cursor commits now resolve exact durable Rust projection coordinates,
  while pointer authoring retains its rendered-item hit testing; one normal bond
  remains one prepared Rust transaction and Escape remains non-mutating.

- Replaced Ferrum's competing main, editing, and mode top-level toolbars with
  one two-row `AuthoringRibbon`. It reuses the live Qt actions and existing
  Rust-backed gesture ownership, keeps grid visibility and snap direct, exposes
  drawing defaults only for atom/bond authoring, and uses a named `More` menu
  at compact widths instead of opaque Qt toolbar overflow.

- Corrected the transient native canvas hex-grid contrast rule. Lines, dot
  outlines, and dot fills now compare against Ferrum's actual themed paper,
  retain passing configured colors, and use bounded adjustment plus an exact
  endpoint guarantee on light and dark palettes. This remains a disposable Qt
  projection and does not alter Rust-owned geometry, snap behavior, or CDML.

- Expanded Ferrum's native catalog from benzene to nine independently authored
  system templates: five carbocyclic rings plus thiophene, furan, pyrrole, and
  purine. The Rust compiler now resolves each closed recipe through the same
  collision-safe, renderer-preflighted transaction and exposes deterministic
  category and semantic search summaries to the CLI and PyO3. Native Haworth
  entries remain intentionally absent until their stereochemical display-token
  receipt can be preserved by the detached catalog candidate contract.

- Rebuilt the ordinary Ferrum command ribbon as a compact single-row toolbar:
  icon-only controls replace text-under-icon padding, the redundant editing title
  is removed, and the editing group no longer forces a second toolbar row.
  Separators, tooltips, and accessible action labels retain command grouping and
  keyboard/screen-reader discoverability.

- Corrected `presentation.vector.create.v1` into an acyclic prepared transaction:
  the document session now holds an exact canonical candidate without mutating
  history or generated IDs, the API composes the complete renderer plan from
  that candidate, and only a no-exclusion preflight may commit it. Its stateless
  CLI response now returns an immutable renderer observation plus explicit
  `input_revision`, `committed_revision`, and next-input revision semantics;
  vector refusals carry closed category and recovery facts.

- Removed the client-visible direct vector commit bypass. Python now receives only
  API-owned gesture, preview, prepared-candidate, and renderer-preflight receipt
  handles; only a matching nonconstructible receipt can commit. The document
  crate retains a raw trusted-Rust composition seam to preserve its no-render
  dependency boundary, but it is no longer bound for client use.

- Added a Rust-owned opaque standalone Text placement session contract. It
  accepts canonical formatted runs, persists one direct-root `<text>` only on
  commit, and exposes typed failure/recovery values through PyO3. Bold and
  italic placement requests are deliberately refused until verified renderer
  faces exist.

- Added Rust-owned direct canvas Plus placement through the presentation-creation
  gesture contract. The revision/digest-fenced opaque handles issue the preview,
  persist one canonical unstyled direct-root `<plus>` with standard-derived
  presentation facts, and power the mutually-exclusive Qt `Draw Plus` tool.
  Its preview now derives its colour and verified glyph bounds from the same
  revision-fenced Rust rendering path as the committed Plus.

- Restored configurable normal Draw Bond behavior for selected atom element,
  bond order, and shared view snapping by using its established prepared Rust
  operation outside the deliberately fixed-carbon P0.1 direct-bond profile.
  Arrow selection restoration now treats any post-commit interaction failure
  as secondary recovery and truthfully reports that the Arrow was added.

- Preserved a Rust-accepted Arrow receipt across a disposable Qt projection
  installation failure. The controller now clears transient selection, invokes
  the exact pending-snapshot refresh path, and reports an added Arrow plus
  display recovery rather than an unchanged drawing.

- Added a Rust-owned, revision/digest-fenced straight normal-arrow creation
  gesture. Its opaque Python handles produce backend-issued preview geometry and
  persist one canonical direct-root arrow with a collision-safe durable ID and
  one history entry.

- Replaced the unsafe direct-child structural-deletion mutation with a Rust
  document planner and session-owned prepared commit. The operation now rejects
  unsupported direct content, malformed topology, and reaction-referenced
  root removal or splitting before mutation; it reports source-order induced
  bond removals, retains connected components canonically, and reserves only
  split-root IDs through the session's collision-safe sequence.

- Replaced the rejected document-side direct-root interaction approximation with an
  acyclic API-owned Rust facade. It derives molecule bounds from the renderer's
  shared primitive-lowering pass and presentation bounds from the exact backend
  render/projection observation. A revision-bound document-layer declaration-ID
  fact includes opaque, unsupported, and preservation-only content, excluding every
  colliding durable root before selection while preserving only recognized direct
  core molecule-fragment member IDREF semantics. The facade exposes typed unrenderable,
  ambiguous-root, and display-only recovery without treating blank canvas as a
  refusal. Opaque session-bound
  handles can atomically translate mixed molecule/presentation selections in one
  document history entry. Opaque translation gestures also capture a closed
  Free or View-hex-grid snap policy and calculate snapped preview deltas in Rust;
  no Qt wiring is claimed.

- Isolated the direct-bond Escape test's temporary drawing preference so its
  element, order, and presentation setup is restored after the test.

- Hardened the Rust direct-bond gesture boundary with private session-origin and
  per-gesture capabilities. Byte-identical sessions, mixed previews, and replayed
  previews now fail without mutation; the private PyO3 seam exposes closed typed
  category and recovery values rather than string-based controller decisions.

- Corrected the active full-parity roadmap's P0 ordering to match the approved
  direct-manipulation design. P0.1 is now explicitly a Rust-first,
  revision/digest-fenced direct normal-bond gesture from an existing direct atom
  to a same-molecule existing or new carbon endpoint, limited to single/double/
  triple bonds. Selected-root selection, marquee, and translation are P0.2 and
  remain gated on reliable Rust-issued render hit/containment/bounds facts; the
  roadmap does not claim either contract is implemented.

- Repaired the installed-wheel native CDML file-route E2E to await the public
  asynchronous local-open completion signal, require a successful idle result before
  inspecting the tab, and cancel/drain a failed route before its Qt host is disposed.

### Additions and New Features

- Added a read-only Rust/PyO3 reaction-authoring choice observation. It joins
  namespace-aware direct-root semantics with the current renderer-issued root
  bounds, marks existing reaction members unavailable, retains display-only,
  unrenderable, and ambiguous diagnostics, and fences immutable choices to the
  originating session revision and digest without exposing CDML or a commit path.

- Added a native offscreen bracket-pair E2E that proves actual rectangular
  drag, Escape cancellation, complete-pair movement, undo, Save As, and
  asynchronous reopen through the public Ferrum file route.

- Made `build.sh` print exact next steps for the artifacts it just created:
  direct CLI help and inspection commands plus a no-dependency local-wheel
  install command followed by `ferrum-qt` for the GUI.

- Hardened the Rust-owned presentation vector contract so its frozen preview now
  carries the exact standard-resolved stroke and fill that the canonical direct
  root persists. Newly authored vectors explicitly write `line_color`, `width`,
  and closed-shape `area_color`, keeping preview and saved appearance identical
  without frontend style authority.

- Added the versioned `presentation.vector.create.v1` operation-protocol route.
  It accepts only a complete CDML snapshot, exact revision/digest fence, closed
  vector kind, finite endpoints, and the `effective_drawing_standard` policy;
  it returns one durable ID plus the canonical resulting document.
  The CLI exposes the same route as
  `ferrum document command presentation.vector.create.v1 REQUEST.json`.
  Its shape field is `vector_kind`, reserving the protocol envelope's `kind`
  discriminator for the operation name.

- Added the Rust-owned `presentation.vector.create.v1` gesture foundation for
  canonical direct-root Line, Rectangle, Square, Oval, and Circle authoring.
  Its opaque revision-fenced handles normalize square/circle drags in Rust,
  issue disposable exact overlays, preserve unrelated CDML roots, and accept
  one durable root in one history transition only on commit.

- Added the Rust-owned Qt `Insert Text` tool. A valid page click opens an
  accessible constrained dialog with selected baseline `Text` content and
  supported baseline/subscript/superscript controls, then previews and commits
  exactly one opaque Rust Text placement. Cancellation, failed placement, and
  display recovery leave CDML authority with the Rust session; created Text can
  be selected, moved, undone, saved, and reopened through the native path.
  Its cancellation paths now always disarm the visible tool, while post-commit
  selection failures preserve the accepted Text and explicitly require display
  recovery rather than claiming ordinary selectable success.

- Added the Qt `Draw Arrow` mode for Rust-owned straight normal-arrow drag
  authoring. Qt retains only opaque gesture handles and paints the exact
  backend-issued preview; accepted arrows are reselected through the existing
  durable root-selection contract, while Escape, focus loss, tool changes, and
  typed refusal retire the preview without mutating CDML.

- Added a dedicated Rust-owned P0.3 Select Structure controller. It resolves
  direct atom and normal-bond clicks, Shift toggles, and full-containment
  marquees through the structural interaction facade, then uses opaque
  Delete/Backspace commits for one-history atomic deletion. Qt draws only
  backend-issued bounds and treats display-only paths as typed recovery.

- Wired the P0.2 Ferrum Move Complete Roots canvas path to the accepted
  render-interaction facade. Point and marquee selection, drag translation,
  selection overlays, and arrow-key movement now submit opaque Rust-issued
  handles; Qt retains only durable transient selections and Rust-issued bounds.
  Excluded durable roots use the named Rust query/refusal path, while blank
  canvas remains an ordinary clear/select gesture.

- Added offscreen P0.2 coverage for molecule-root click selection, full-bounds
  marquee selection, keyboard transaction movement, undo, save/reopen, and the
  no-mutation typed recovery route for a known excluded root.

- Added `tests/e2e/e2e_render_interaction_selection.py`, an independent
  offscreen Qt workflow receipt for P0.2 root click/marquee selection, pointer
  movement, keyboard movement, undo, save, and backend reopen.

- Extended the P0.2 render-interaction contract and Qt controller so the one
  Move Complete Roots action authenticates preselected molecule and supported
  presentation roots as one opaque Rust selection. Named-root toggle preserves
  mixed selections without Qt geometry or the retired transform fallback.

- Reauthenticate retained P0.2 selection IDs through Rust named-root queries
  before each new gesture. A root made excluded or ambiguous by reprojection now
  reaches the controller's typed recovery path; blank canvas remains a normal
  selection result.

- Made Move Complete Roots pointer and keyboard gestures honor the existing
  View hex-grid preference through Rust's closed render-interaction snap-policy
  factory. Qt now sends only `Free` or `ViewHexGrid`; it performs no grid
  coordinate or delta calculation.

- Replaced the pre-render P0.2 direct-root seam with a render-evidence-backed
  Rust contract. It provides revision/digest-fenced renderable-root observations,
  full-containment marquee resolution, opaque selection/translation handles,
  typed recovery categories, and one-history-entry commits. Qt integration
  remains a separate follow-up; the PyO3 boundary exposes no editable projection,
  raw XML, or caller-constructed root set.

- Added a root `build.sh` developer entry point. It builds the release-mode
  Rust `ferrum` CLI plus the native Ferrum-Chem and PySide6 Ferrum wheels into
  `build/` without modifying the active Python environment or invoking the
  separately controlled release-wheelhouse workflow.

- Added Rust-authoritative direct normal-bond pointer authoring to Ferrum's
  Draw Bond mode. A drag begins only on an existing atom, asks the Rust session
  for every snapped/topology-checked preview, projects its disposable overlay,
  and atomically commits the checked opaque handle on release. Escape, focus
  loss, tab/tool changes, and ordinary refused endpoints discard the preview
  without changing the drawing.

- Corrected direct normal-bond pointer release so it commits only the exact
  stored Rust gesture and preview, never the legacy bond helpers. Empty-space
  endpoints now always use Rust's fixed carbon semantics, and closed typed
  preview refusals are shown and terminal before release.

- Added the Rust-owned P0.1 direct normal-bond gesture contract. The document
  session now provides revision/digest-fenced begin, pure preview, and atomic
  commit for an existing same-molecule endpoint or a new carbon endpoint,
  restricted to normal single, double, and triple bonds. The PyO3 seam exposes
  opaque gesture and preview handles, frozen overlay/receipt values, and typed
  ordinary refusal categories without giving Qt document or CDML ownership.

- Added the active Rust-first full-parity roadmap at
  `docs/active_plans/active/FULL_PARITY_RUST_FIRST.md`.  It reopens prior
  parity-related drops only for the expanded goal, sequences the 23 missing Qt
  workflows and backend gaps behind shared contracts, separates the P0 usable
  editor from later parity work, and keeps plugins and PubChem behind distinct
  security and service design gates.

- Added the maintainer-only M22 `build_release_wheelhouse.py closeout` phase. It delegates the
  four explicit final artifacts to the existing inventory verifier and atomically retains the
  verifier's JSON result beside the M20 receipt; it neither creates release artifacts nor claims
  a supported release.

- Moved checked CDML centimetre/scene-point conversion ownership into the Rust
  geometry crate with public finite `CdmlLength` and `ScenePoints` value types.
  The PyO3 surface now delegates without duplicating the 72/2.54 policy, and
  Rust session tests explicitly cover persistent Wavy endpoints, alternation,
  safety bounds, and revision fencing.

- Added human-facing `ferrum inspect`, `validate`, `rewrite`, and `render`
  commands over the frozen operation protocol. The commands accept CDML paths
  or standard input, support human or JSON-envelope output, publish named
  outputs safely, infer render formats from destination suffixes, and teach one
  worked example in every verb's help.

- Added Ferrum's shared dotted action registry and centralized keyboard policy.
  Existing window-owned actions now expose stable IDs for File, Edit, View,
  drawing-mode, and cancellation commands. Standard Qt key sequences cover the
  platform editing workflow, while Ferrum-specific bindings cover zoom, grid,
  atom, bond, and Escape cancellation actions; duplicate bindings fail before
  any action is changed.

- Added a standard About Ferrum dialog with version, engine, license, and
  project-link information. About and Preferences use their platform menu
  roles, the dialog has explicit tab order, and the drawing canvas now has an
  explicit strong-focus and accessible-name contract.

- Documented the four human CLI verbs with path, standard-stream, raw-output,
  JSON-envelope, and safe-publication examples. Added `docs/TODO.md` as the
  concise register for the remaining protocol, desktop-convergence, adapter,
  and release gates, and expanded the related-project map for the binding and
  geometry toolchains.

- Completed the T1-T27 convergence register. The CLI now has six protocol-backed
  verbs: `inspect`, `validate`, `rewrite`, `render`, `convert`, and `coords`.
  `convert` and `coords` use a validated explicit engine bundle and return a
  typed unavailable-engine result when no active bundle exists. The public
  protocol remains pathless and the new API contract documents the envelopes,
  stream behavior, publication rule, and 0/1/2/3 exit statuses.

- Added the repository CI workflow and the final local native-wheel/engine-bundle
  verification route. The workflow calls the existing repository, Python/Qt, and
  Cargo front doors without duplicating their checks. It is present in the tree but
  has not yet run on GitHub Actions.

### Behavior or Interface Changes

- Rebranded the visible PySide6 application from Ferrum-Qt to Ferrum across
  application metadata, window titles, settings identity, command help, and
  current user documentation. User-facing failures no longer describe the
  implementation as native, admitted, or typed CDML; the CDML namespace and
  historical lineage records remain unchanged compatibility and provenance
  facts.

- Renamed `ferrum_qt/native/` to `ferrum_qt/ferrum/` and removed the redundant
  `ferrum_native_` module-name prefixes. Feature modules now access the compiled
  Rust extension through the single lazy `ferrum/engine.py` boundary, so an
  isolated Qt test can substitute an exact private DTO seam without leaking a
  fake module into unrelated tests.

- Renamed the internal legacy peptide-template profile from
  `oasa-compatibility-v1` to `ferrum-legacy-template-v1`. Its deterministic
  supported alphabet and generated structural SMILES are unchanged.

- Completed the requested Rust ownership decomposition. `ferrum-api` now owns
  only CLI presentation, protocol DTO/execution, trusted runtime selection, and
  transport. Chemistry codecs live in `ferrum-chemistry`, document/session/CDML
  operations in `ferrum-document`, scientific preparation in `ferrum-domain`, and
  render plans/artifacts in `ferrum-render`; the eight-crate workspace keeps lower
  crates independent of the delivery facade.

### Removals

- Removed the unused Python `wavy_geometry` helper and its duplicate unit test;
  persistent Wavy geometry is generated only by the revision-bound Rust
  document session.

- Removed the ignored stale desktop build tree, including its obsolete
  `oasa_bridge.py`, and removed empty `legacy`, `models`, `modes`, `setup`, and
  `undo` placeholder directories. The two Python package manifests and root
  `VERSION` now use the same zero-padded `26.08` source spelling.

- Retired the two remaining test-only OASA Python workers, their differential
  runners, and the OASA test dependency. Accepted parity reports remain as
  historical evidence; the optional reference environment now contains only
  Python RDKit for one-time maintainer measurements.

### Fixes and Maintenance

- Canvas Add Atom now requires the exact installed Rust render observation to
  contain a molecule render plan for its durable target. Unsupported rich
  labels and other unrenderable molecule states refuse before any Rust
  preparation or commit, leave the drawing unchanged, and explain how to
  choose another visible molecule or restore a supported label/style.

### Developer Tests and Notes

- Added a dedicated renderable CDML fixture for the keyboard-authoring E2E. The
  workflow now proves the original durable render target, immediate Rust atom
  selection at the requested cursor point, durable bond endpoints, undo, Save
  As, and Rust reopen without relying on preservation-only rich-label fixtures.

- Added focused Qt tests for stable action bindings, conflict-free keyboard
  setup, canvas focus, the About dialog, and the single extension-import
  boundary. Added Rust tests that run all four human verbs from standard input
  and verify their worked help, plus a real CLI E2E that compares each human
  surface to the equivalent protocol outcome and composes standard streams.

- Built a fresh CPython 3.12 arm64 `ferrum-chem` wheel and completed an
  offscreen Ferrum startup/shutdown smoke with a success receipt. The focused
  Qt startup/action/adapter checks, Rust CLI tests, strict Clippy, rustfmt,
  Python indentation, Markdown-link, and Pyflakes gates pass. The complete Qt
  suite reaches an environment-dependent coordinate-generation dialog when the
  separately built chemistry adapter is unavailable, so that run is not
  recorded as a full-suite pass.

- Final local verification passed `all_test.sh`: 5,916 repository tests, 213
  installed Ferrum-Chem binding tests, and 393 Ferrum Qt tests with 1 skipped.
  `check_rust.sh` passed earlier in the same convergence run. A final local
  macOS arm64 native wheel and validated engine bundle exercised the chemistry
  CLI route. These are local implementation receipts, not a remote CI result,
  a cross-platform claim, final release artifacts, or human legal/release
  approval.

- Documented the bytecode hygiene correction: agents must not use `py_compile`
  or `compileall` for validation because those explicit compilers write bytecode
  despite no-bytecode runtime settings. Pytest and AST parsing are the approved
  validation paths.

## 2026-08-15

### Additions and New Features

- Added source-accepted M22 release-closure mechanisms: dual-license source-archive validation,
  a standard native-wheel notice bundle, and predicate artifact inventory. The InChI MIT notice
  comes from the leading license and attribution comment in the exact pinned InChI compiled-source
  header. These mechanisms await final artifacts and human legal/release review; they do not claim
  a supported desktop release.

- Added the source-accepted M20 macOS arm64/CPython 3.12 two-wheel release route. It builds
  `ferrum-chem` and `ferrum-qt` from explicit local Cargo, Qt build-backend, and Qt runtime
  wheelhouses; the retained E2E uses a scrubbed no-index install and the existing LGPL relink
  mechanism. The Rust `ferrum` CLI remains separately Cargo-installed.

- Added the implementation-complete M17/M18 stateless operation protocol V1:
  four closed CDML operations, Rust-generated checked-in schema, a narrow
  `ferrum_chem` JSON boundary, and `ferrum protocol schema/run`. The derived
  request-envelope admission runs before CLI transport allocation, Python input
  copying, and JSON parsing. It is a resource-safety boundary grounded in the
  existing CDML profile and JSON representation, not a performance target.

- Added native `File -> Export...` publication for the current complete
  document: SVG, vector PDF, and transparent PNG at one Rust page point per
  output pixel. Rust prepares the revision/digest-bound observation and
  publishes it safely; Qt owns the chooser and current-tab containment.
  Decoded CD-SVG sources cannot be overwritten through their original wrapper
  or a hard-link alias. This does not add CD-SVG export or wrapper round trips.

### Behavior or Interface Changes

- `ferrum protocol run` returns one JSON success or typed-error envelope for a
  completed request and uses explicit safe named publication only. Its 0/1/2/3
  statuses distinguish completed protocol results, pre-envelope or confirmed
  publication failures, usage errors, and possibly-published output. The shipping
  `ferrum` executable now exposes only `protocol schema` and `protocol run`;
  provisional CDML, SMILES, SDF, molblock, and InChI root command families were
  retired without removing their separately owned Rust or Ferrum-Qt-native seams.

- Replaced the implementation-facing `Export Rust Snapshot` submenu with
  `Export...`, `Export SVG...`, `Export PDF...`, and `Export PNG (1 pixel per
  point)...`. A cancelled chooser is quiet and non-mutating. Open, Open in
  Current Tab, export, and close share reciprocal busy containment. The
  separate `Recovery Export CDML...` action remains a current-document recovery
  copy, distinct from Save/Save As and artifact export.

- Added the packaged Ferrum application icon through the existing resource-path
  loader. Qt keeps its generic fallback only when that packaged icon cannot load.

- Retired the explicit OASA compatibility host, legacy document session, and
  its action/mode/worker/codec/projection test island. Production `oasa`
  dependency declarations are gone, and `ferrum-qt` now opens one ordinary
  Rust-native window. File Open gives suffix-based actionable refusals for
  dropped CDXML, CML, `.cdsvg`, `.svgz`, and compressed CDML inputs without
  changing the active document. The pre-production boundary also drops PubChem
  and unported legacy template, mode, repair, clipboard, and property variants
  rather than retaining an unreachable compatibility shell.

### Decisions and Failures

- The target-matching external Cargo and Qt wheelhouses are not available in this checkout, so the
  real build/install/relink receipt remains pending. M20 and M22 source acceptance is not a
  supported-release claim: source-archive CLI, classified artifact, and human legal/release review
  also remain pending. Build/site inspection, toolchain inventory, clean installation, relink, and
  artifact inventory are E2E or disposable release evidence, not timing, byte/hash, member-count,
  pixel, network, or matrix gates.

- Closed the retained M15 utility boundary around bounded peptide insertion,
  selection-to-linear-form conversion, and geometry repair. Removed unused
  compact-sugar and descriptive catalog/seed code and tests instead of carrying
  unowned legacy behavior.

- Closed M16 at the one-window support/refuse/drop boundary: supported native
  routes share Rust document authority, while historical unsupported routes are
  explicitly refused or treated as pre-production drops. M17 and later
  protocol, packaging, and release work remains open.

- Completed M19's implementation documentation: the capability ledger now indexes
  supported rows to their accepted semantic, E2E, or one-time validation lane;
  refusals and drops remain recorded decisions. The thread-affinity receipt records
  thread-confined sessions and serialized GUI-thread mutation without an arbitrary
  timing or throughput requirement. Independent M19 closure review remains pending.

### Developer Tests and Notes

- M17/M18 retains compact offline Rust and installed-Python semantic coverage.
  The real CLI runner, schema-resource/wheel checks, generator, package build,
  and installed walkthrough are E2E or disposable evidence; no byte, pixel,
  timing, count, network, mock, or fixture-matrix gate was added. The checkpoint
  rereview and fresh wheel/site evidence are accepted; M17/M18 are complete.

- Added three private-bridge behavior tests for artifact provenance refusal,
  guarded local SVG publication, and retained-origin hard-link refusal. Native
  lifecycle and authoring coverage retains one representative CDXML
  refusal/nonmutation check. Wheel/site installation, ordinary-window
  walkthroughs, source/package inspection, visual review, and race probes are
  one-time implementation evidence, not timing, count, fixture, byte, or pixel
  gates.

- Reconciled the active migration, interface, usage, file-format, and OASA
  ownership documentation with the compatibility-host retirement. Historical
  OASA/BKChem provenance and test-only oracle references remain separate from
  the shipped Ferrum runtime.

## 2026-08-14

### Additions and New Features

- Added the accepted native `Chemistry -> Insert
  Direct-Glycosidic Haworth...` V1 checkpoint. It accepts only structural SMILES
  for two disjoint five- or six-member C/O rings joined by one exterior degree-two
  oxygen, with neutral nonaromatic single bonds only. Rust owns admission, the
  one-use graph receipt, durable C/O drawing facts, normal V2 rendering, history,
  selection, and persistence; private PyO3 carries the receipt and Qt owns the
  accessible empty-text dialog and one snapped detached placement. This is not a
  sucrose/name, anomer, linkage, stereochemistry, or general-SMILES feature. The
  slice stores no SMILES, UI state, or preferences in CDML and adds no OASA,
  QSettings, public `.pyi`, CLI, wire, or composite-render contract. Compact
  semantic tests are permanent evidence. A sealed installed site passed the focused
  private/public suite (4 passed), and the independent public walkthrough accepted
  inline blank/invalid recovery, pointer-tool cancellation, occupied retry, selection,
  Escape/tab-switch/close containment, Undo/Redo, save/reopen, and normal V2 receipt-only
  installation without a marker. Wheel/site mechanics, OASA harness, offscreen focus,
  screenshots, visual, parser, accessibility, and occupancy probes remain disposable.

- Added the accepted native `Chemistry -> Create Fragment...` and
  `View Fragments...` Explicit Fragment V1 slice. Rust prepares and commits one
  authenticated, explicit-only annotation inside one durable direct-root molecule;
  selected bonds close over their endpoints and records retain source order. Names are
  trimmed, plain, nonblank labels and may duplicate. Qt captures the source tab,
  selection, revision, digest, and molecule before its one-field dialog, while the
  read-only view exposes only exact supported records and one retained-metadata notice.
  Rust/PyO3 remain private runtime ownership; there is no OASA, QSettings, public `.pyi`,
  CLI, or wire contract. The independent rereview accepted the View-lifecycle and stable-error
  repairs, and the installed public walkthrough accepted endpoint closure/source order, duplicate
  labels, blank retry/Cancel/stale containment, retained notice, View retirement, undo/redo, and
  save/reopen. Compact semantic Rust/private-binding/public-Qt tests are permanent; wheel/site,
  screenshots, keyboard/accessibility, visual, corpus, and timing checks are disposable evidence.

- Added the implemented native decoded CD-SVG Open V1 route. The requested local `.svg` is
  decoded UTF-8 SVG with exactly one canonical embedded CDML payload, held inside independent
  wrapper and payload envelopes. Rust admits only the payload and privately transfers a one-use
  receipt, equality-only descriptor token, and closed source kind through the ordinary Open,
  current-tab, and Recent lifecycle. The wrapper is never rendered, fetched, persisted, or saved.
  Decoded CD-SVG tabs are clean and Save-As-only to `.cdml`; later publication retains the original
  duplicate token. Compact semantic tests are permanent; wheel/site, chooser, accessibility,
  visual, corpus, and timing probes are disposable. The independent checkpoint rereview and installed
  public walkthrough are accepted. Compression, `.svgz`, `.cdsvg`, sniffing, wrapper round trip/export, public
  API/CLI/wire adoption, and OASA fallback remain deferred.

- Added the accepted native `Chemistry -> Check Bond Capacity...` read-only selected-root diagnostic.
  Rust owns its closed neutral H/B/C/N/O/F/Cl/Br/I capacity table, explicit-H plus incident-order
  arithmetic, source provenance, and authenticated receipt; private PyO3 transports the receipt and
  Qt owns worker containment and the selectable report. It reports Within Capacity, Exceeds Capacity,
  or whole-root Not checked without claiming chemical validity, general valence, or oxidation state.
  An accepted public real-worker walkthrough covers supported/no-excess/finding/Not checked results,
  authored-fact display for mixed excess roots, root ordering, depiction-independence, lifecycle,
  nonmutation, and accessibility.

- Added the ordinary native `Editing Tools` toolbar and `Cancel Tool` action. It presents existing
  Rust-owned editing actions through a visible, hideable Qt workspace client; gesture, document,
  selection, history, snap, and recovery ownership remain unchanged.
- Added `Next atom:`, `Next bond:`, and compact `Edit -> Next Drawing...` clients for shared
  application/QSettings drawing defaults. Add Atom and Draw Bond capture their effective values per
  gesture; Rust remains the validator and owner of document facts, identity, history, and projection.
- Added closed ordinary bond presentation: Normal emits `n1`/`n2`/`n3`, and directed Single
  Solid/Hashed wedge emits `w1`/`h1` with press/start as tip and release/new endpoint as base.
  `Next presentation:` is an application preference captured at press; `w2`/`w3`/`h2`/`h3` and
  other styles remain deferred. Selected-bond Properties now supports the admitted styles.
- Added the active `ferrum-render-plan-v2` molecule-plan grammar. Neutral finite paths carry
  `MoveTo`, `LineTo`, `CubicTo`, `Close`, explicit stroke/fill/z, and flow through Rust's SVG,
  PNG, PDF, bounds, composite, private-PyO3, and Qt consumers. `RenderObservationV1` remains the
  document/projection receipt envelope and `DocumentRenderPlanV1` the distinct full-document grammar.
- Added a closed Rust-owned detached regular-ring V1 substrate for finite-centre C 3--8 normal
  single rings using a 40-point flat-top clockwise y-down geometry. The exposed action is only
  `Insert Cyclohexane Ring`; its empty-page gesture captures the shared snap centre and commits the
  prepared C6 receipt atomically. Fusion, attachment, heteroatoms, aromaticity, rotation, and
  preferences remain future contracts.
- Added native `Edit -> Insert Haworth Ring...` for four explicit detached D-glucose recipes:
  alpha/beta D-glucopyranose and alpha/beta D-glucofuranose. Rust owns literal C6O6 chemistry,
  finite local geometry, IDs, CDML, history, selection, the one-use prepared receipt, and the
  normal render plan; Qt owns the readable form/anomer chooser, one captured snap anchor, and the
  receipt-derived preview. Pyranose uses `O5-C1-C2-C3-C4-C5`; furanose correctly uses
  `O4-C1-C2-C3-C4` with the `C4-C5-C6` chain. The closed front depiction uses `q1` C2-C3,
  directed `w1` shoulders, and `n1` remaining edges. Generic codes/catalogs, other sugars,
  attachment/fusion/rotation/reflow, and general stereochemistry remain separate contracts.
- Extended normal whole-document Render Plan V2 lowering for declared Haworth front facts. A
  front `q1` becomes a round-cap front-stroke path and a directed front `w1` becomes a rounded
  filled front-wedge path; the emitted V2 cap and display layer flow through Rust, PyO3, and Qt.
- Added Slice A of ordinary native CDML Open. Rust mints an opaque descriptor-derived origin token
  in the private one-use admission receipt; Qt uses immutable Open intents and token equality to
  activate an already-open native tab, including a hard-link alias. Interactive Open replaces only
  the marked, clean bootstrap `Untitled` page after detached admission and atomic installation;
  populated, dirty, busy, stale, cancelled, and failed paths preserve existing work or use a new tab.
  Queued launch paths always use new-tab intents.
- Added native `File -> Recent Files` as the versioned
  `FerrumNativeRecentFilesV1` personal QSettings client. It stores lexical normalized absolute display
  paths without resolving symlinks, promotes only after confirmed native Open/token activation or Save,
  and routes selections through the ordinary forced-`NewTab` coordinator. Colliding basenames gain
  parent context, full paths remain available to the user, stale entries offer default Keep or explicit
  Remove, and Clear changes settings only. The ordinary startup seam creates the owner after
  `QMainWindow` construction and before File actions, retaining one stable File cascade.
- Added the explicit ordinary-native `File -> Open in Current Tab...` route with `Ctrl+Shift+O`.
  It prepares and authenticates the source before it revalidates an immutable current-tab fence;
  matching descriptor tokens activate the existing tab without replacing the requested target. Clean
  saved populated tabs swap atomically. Dirty targets receive Save (default), Replace, and Cancel;
  named Save publishes through the native path, unnamed Save uses Save As, and a fresh post-save fence
  is required before installation. The shared exact-target asynchronous-work predicate disables the route
  during relevant native work and refreshes it at terminal delivery. Failure, cancellation, stale/busy/close state, invalid admission,
  or save failure preserves the target and never becomes a surprise NewTab. Recent composition remains
  confirmed-only/forced-NewTab, outside CDML, QSettings beyond its existing client, OASA, and legacy
  ownership. Worker finalization defers across the modal recovery decision.
- Added the private immutable `TopLevelTranslationAnchorV1` receipt for complete-root drags.
  It records canonical complete roots, revision/digest, and the lower-left authored union anchor;
  enabled snapping resolves `snap(anchor + raw_delta) - anchor`, disabled snapping keeps raw delta.
  Stale selection/provenance paths preserve state and direct the author to select complete roots again.
- Added ordinary Rust-native user templates in `~/.ferrum/templates`: bounded one-molecule CDML,
  inspection-only Paper/Standard context, fresh identities/reference remapping, centroid placement,
  atomic history, secure catalog refresh, and safe Save As User Template publication.
- Added native Preferences for theme, workspace restoration, paper-local grid visibility, and authored
  point snapping. QSettings choices project to tabs and views; the renderer-owned paper rectangle
  and Rust lattice bridge supply the disposable overlay. These personal values remain outside CDML,
  document/history/save state, and selection.
- Added ordinary Rust-native Copy as SVG, Cut, Paste, and frequent-action toolbar routes. Rust owns
  authenticated source/fragment contracts, selection/root resolution, atomic mutations, and
  authoritative observation; Qt owns bounded worker and clipboard clients.
- Added ordinary native chemistry and document routes: Convert selection to linear form; Molfile,
  SDF, SMILES, Standard/Fixed-H InChI, and recovery-CDML export; molecule naming/information;
  drawing defaults; Copy; theme; and accessible status/zoom controls. Provisional
  `ferrum cdml render {svg,pdf,png}` and `ferrum smiles canonicalize` remain explicitly bounded.
- Added the M16/M22 OASA runtime ownership ledger and migration account-switch handoff. They
  distinguish CDML/oracle evidence from BKChem-Qt interface improvements and record the completed
  compatibility-host retirement boundary.

### Behavior or Interface Changes

- Adopted native-first Ferrum identity, `Ferrum` / `Ferrum-Qt` QSettings, and the pre-release
  template location without a legacy BKChem migration promise. Historical provenance stays separate.
- Aligned the FQ-020 label to `Snap New and Moved Points to Hex Grid`; shared point policy covers
  authored placement while exact joins, rotation, and complete-root translation retain distinct inputs.
- Moved growing native-tab/PyO3 registrations to feature owners, adopted a read-only Properties
  projection client and continuous zoom/status presentation, and retained narrow-window behavior as
  Qt client behavior rather than a fixed visual specification.

### Fixes and Maintenance

- Haworth insertion now asks the installed native projection whether either a durable atom or a
  durable bond occupies the raw or snapped placement point. An occupied location leaves the
  document and selection unchanged and keeps the one-use chooser intent ready for an empty-page
  click.

- Disconnected each native window's QApplication clipboard relay at Qt teardown
  and fenced late clipboard notifications behind the live native tab host. This
  keeps clipboard changes from reaching disposed window widgets while preserving
  normal Paste refreshes for active windows.

- Repaired the native Haworth chooser and preview lifecycle. The global Next Drawing filter now
  passes non-key events through, the parented chooser uses normal modal event delivery, and a
  preview retires through the graphics owner before authoritative scene replacement. The same
  native-window clipboard teardown boundary now prevents deferred clipboard notifications from
  reaching disposed Qt widgets; this is a general lifecycle correction, not Haworth-only logic.

- Corrected ring circumradius to preserve the source side length, root-drag stale recovery, focused
  Escape restoration for Next Drawing, and the FQ-016 template-location documentation.
- Removed stale plugin placeholder ownership and brittle tests; retained callable manifest registration
  and required-action failures. The extension system remains a future explicit contract.
- Restored a type-check-only native-tab import, removed inert typing imports, corrected Telex whitespace,
  and restored the intended RDKit threadsafe-substructure profile setting.
- Rotated the complete 2026-08-12 block to `CHANGELOG-2026-08b.md` while retaining these two newest
  day blocks in the active log.

### Decisions and Failures

- Direct-Glycosidic Haworth V1 opens with an empty structural-SMILES field and
  ships no sample or named preset. The earlier illustrative string had substituent
  atoms outside the closed bare two-ring profile. A verified neutral unnamed example
  needs a separate disposable native-parser probe before it can become help text.
  M16/M19 retain the separate legacy/OASA direct-glycosidic and verified-sucrose
  actions until this checkpoint has independent acceptance and each retained path
  has its own transfer or disposition.

- Bond Capacity Check preserves authored charge and explicit-hydrogen presence, ignores bond depiction,
  and accepts only the declared neutral nonaromatic ordinary-atom grammar. It adds no CDML, history,
  selection, Properties, QSettings, OASA, public Python, CLI, or wire surface. Compact semantic tests
  remain permanent, including a mixed-root public regression; fresh wheel/site, visual, OASA, and
  timing probes are disposable evidence.

- FQ-016, FQ-020, M16, M17, M18, M19, and M22 remain partial where later contracts remain. The
  active renderer foundation does not by itself claim wedge/hash authoring.
- Recent Files capacity is a local usable-menu policy, not a fixed acceptance gate. Explicit
  populated-tab replacement is a distinct target-fenced command. Origin tokens are private live-tab
  receipt facts, never CDML, session/history state, or a cross-process identity promise. Compact
  current-tab behavior coverage is permanent; a one-time lifecycle walkthrough confirmed active-tool
  and target-work disable/re-enable
  guidance, stale/Cancel containment, clean/dirty recovery, hard-link activation, ordinary Open, and
  Recent routes.
  Its accessibility, keyboard, wheel/site, race, source, and visual probes remain disposable evidence.
- Slice B startup reachability and its visible stale-file recovery were remediated and accepted by a
  fresh ordinary-window walkthrough. The permanent product test clicks actual Keep/Remove actions;
  the offscreen physical Remove-click limitation is recorded as disposable evidence only.
- Permanent coverage remains compact, local, and semantic. Keyboard, visual, accessibility, overflow,
  wheel/site, screenshot, artifact, byte, pixel, timing, count, and private-wiring observations are
  disposable implementation evidence unless a behavior-facing test earns permanent status.
- Haworth permanent coverage checks the four recipe graphs, O4/O5 closure distinction, one-use
  Rust transaction, semantic q/w/n depiction, private binding behavior, and the visible native
  chooser-to-placement path. IUPAC/PubChem source captures, OASA comparisons, wheel/site builds,
  screenshots, and visual/accessibility walkthroughs are one-time evidence. The accepted installed
  walkthrough drove all four chooser variants, snap/preview/commit, occupied/cancel/stale
  preservation, public tab undo/redo, and save/reopen; its focused public bond-midpoint rewalk
  confirmed that the still-armed intent accepts a subsequent empty-page click. Revision advancement
  during history is expected, while semantic CDML and document history are restored.
- The active changelog exceeded the repository's source-file limit after retaining two unusually large
  current-day blocks; its history has been consolidated without splitting or moving either day.

### Developer Tests and Notes

- Semantic Rust, private-PyO3, and Qt tests cover the admitted template, point/snap, drawing,
  transform-anchor, render-plan, bond-presentation, regular-ring, Open-receipt/lifecycle, and Recent
  Files promotion/removal boundaries. Fresh wheel/site, menu/accessibility, visual, route-inventory,
  screenshot, and timing observations remain disposable implementation evidence rather than release gates.
- The completed compatibility-host retirement retains semantic ordinary
  startup/Open-save-reopen/cancel/stale/shutdown coverage. Retired bridge coexistence, legacy-session,
  and frozen action-catalog tests were not replaced; importer scans, action absence, walkthroughs,
  wheel/site, screenshots, and timing remain disposable evidence rather than permanent tests.

## 2026-08-13

### Additions and New Features

- Added the ordinary native `Chemistry -> Inspect Selected Molecule` route with a revision-bound Rust
  observation and paint-only recorder for authenticated render-plan inspection.
- Added the first closed native-SMILES prepared builder and the bounded native 17 peptide-template
  insertion route; both retain detached preparation, provenance, and one authenticated commit.
- Added direct Haworth/glycosidic migration layers: topology, local layout, fragment and depiction
  receipts, depth projection, direct insertion, and renderer profile. These are explicit M14/M16
  contracts rather than OASA fallback behavior.
- Added a developer-consented API example for explicit performance observation, with no product timing gate.
- Isolated ordinary M16 startup from OASA and established the native-first MainWindow/session lifecycle,
  empty-document baseline, Open CDML, Undo/Redo, atom number, bond-properties, and atom/bond deletion
  routes through Rust-owned document/session boundaries.
- Added M13 renderer foundations: fixed miter behavior, borrowed draw stream, checked neutral vector
  operations, `DocumentRenderPlanV1`, one `RenderObservationV1`, exact direct-root lowering, and
  whole-page composition. SVG, PDF, and PNG snapshot export use this backend path.
- Added bounded ABI-4 InChI import/export/InChIKey, multi-record SDF import, representable InChI native
  projection, and the first five plus clean-geometry Rust-owned Repair actions.
- Added revision-bound selected-atom rotation and whole-root transforms; native Qt consumes prepared
  Rust intents and authoritative projection rather than deriving durable document geometry.
- Added bounded standalone native Text display/editing, supported direct-root vectors/shapes/arrows,
  Wavy lines, rectangular/round brackets, selected presentation deletion, and source-order operations
  to bring forward, send back, or reverse selected slots.
- Added the Rust-owned paper catalog, document properties, physical-page projection, and seven native
  atom-mark toggles. Qt receives supplied geometry and glyph facts rather than interpreting CDML.
- Added the backend-only M10 preservation gate for structural CDML rewrite across the committed corpus.

### Behavior or Interface Changes

- Established the ordinary MainWindow as native first while retaining the compatibility host as an
  explicit OASA session island. Supported routes use `DocumentSession`; unsupported forms report
  typed issues rather than silently substituting legacy behavior.
- Split molecule/document root order from molecule-local order in render observations so molecules and
  direct-root artwork retain authored interleaving. Multi-segment vectors, shapes, arrows, Telex Plus,
  Wavy, and brackets consume Rust-issued geometry and appearance facts.
- Defined fixed M13 stroke, future PNG/PDF admission, and artifact-publication semantics without
  inventing defaults, DPI, allocation, byte, or timing promises.

### Fixes and Maintenance

- Corrected zero-area shape projection, Qt worker delivery through an owned QObject relay, M11/M12
  tracker status, capability-ledger identity claims, and obsolete native-wheel target instructions.
- Simplified `check_rust.sh` while retaining Cargo test and strict Clippy coverage; reduced the RDKit
  build profile to grounded, verified switches and removed fake or redundant CLI choices.
- Removed brittle partial-native-tab, private worker-wiring, parity-wrapper, subprocess-smoke, and
  exact-inventory tests. Kept deterministic behavior-focused coverage and disposable integration probes.
- Rotated 2026-08-11 and 2026-08-03 to `CHANGELOG-2026-08a.md` under the documented archive policy.

### Decisions and Failures

- Closed the pre-production third-party-plugin design as a product drop. Future extensibility requires
  explicit discovery, permission, versioning, lifecycle, and failure-containment ownership.
- M11/M12 remain in progress; M13 backend work is bounded and does not claim final PNG/PDF contracts.
  M16 full-session adoption and compatibility-host cutover remain open.
- Permanent tests use the `PYTEST_STYLE.md` checklist. Current-wheel, OASA-oracle, visual, pixel,
  byte, timing, network, exact-count, and private-wiring checks remain one-time evidence where useful.

### Developer Tests and Notes

- Rust, binding, and native-window semantic tests cover atomicity, provenance, history, selection,
  reopen, preservation, typed refusal, and projection. Wheel and public-window exercises were used as
  disposable integration evidence.
# Unreleased

* Added opaque direct-bond candidate admission receipts. Rust now validates complete existing
  and snapped new-endpoint candidates without reserving identifiers, tokens, or history, then
  redeems the receipt through one fenced atomic commit. The legacy preview API remains compatible,
  including foreign/stale precedence and `preview_mismatch` / `report_conflict`; PyO3 adds opaque
  admission handles with frozen semantic-refusal and commit-category surfaces.

* Registered Ferrum pointer actions with each owning `QMenu` before insertion,
  so direct menu lifecycle signals now bracket action handoff even when an
  offscreen popup Show event does not reach the application filter.

* Added the active autonomous plan for attaching one cyclohexane ring to one
  existing atom. It uses two bounded Rust topology experiments before selecting
  the closed C6 contract, then document, private PyO3, Qt, focused-proof, and
  one E2E packages. Ring fusion, bond attachment, generic templates, broad CML
  work, fixtures, and human/manual gates remain explicitly deferred.

* Registered Rust/PyO3 receipt-lifetime coverage for the private live SMARTS bridge. The focused
  API test now proves receipt-only retirement refuses the old opaque receipt while retaining the
  published plan for raw and selected reruns, and proves full retirement reports the closed
  `plan_not_published` reason.

* Added one-shot canvas capture for selected-molecule SMARTS queries. Qt now
  immediately converts a renderer point selection into a tab-private opaque
  Rust token, discards the generic selection, and runs selected SMARTS only
  from that token. Focus, Escape, right-click, tab, dock, and authoring-tool
  handoffs retire the temporary capture.

* Corrected the modeless Qt SMARTS dock so selected-molecule execution uses a
  one-shot tab-private capture/run boundary, errors accept only closed category
  enums, replay reaches native receipt refusal, and Escape first removes only
  transient highlight before retiring the full result run. Retirement failures
  now preserve and block the visible state rather than being swallowed.

* Strengthened the final installed-wheel SMARTS combined E2E receipt with exact
  named-CLI result semantics, manifest/wheel/installed native-closure equality,
  guarded reprojection retirement, and live-query Save As/Rust reopen/async GUI
  reopen coverage.

* Replaced the Qt SMARTS multi-row mock bridge with an isolated installed-wheel
  proof. It now exercises real PyO3 row redemption, one-use replay refusal,
  overlay replacement, and native receipt invalidation after rollback failure.

* Added installed-wheel SMARTS evidence for the guarded Qt
  `tab._session.observe_render(...)` proxy: it now proves that proxy publication
  returns a fresh valid observation while retiring an unread live receipt.

* Removed the obsolete public SMARTS query-origin DTO and added sealed-boundary
  coverage for the stateless protocol/schema, named CLI command, and live
  PyO3 receipt lifecycle. Custom typed Rust chemistry engines can now construct
  validated owned SMARTS match facts without receiving native adapter or wire access.

* Tightened the M4b typed SMARTS result construction contract. Custom engine
  rows now validate against the caller's target graph and requested cap before
  they become owned facts, including target bounds and fixed query-row arity.
  The private live-query PyO3 receipt test now refuses source-extension runs
  unless `FERRUM_SMARTS_SEALED_WHEEL_ROOT` identifies a fresh installed ABI-5
  bundle manifest.

* Routed every Ferrum Qt document render-plan publication through the private
  API-owned live SMARTS transaction. Initial construction, admitted async Open,
  refresh, accepted mutation, Undo/Redo, and Save now retire the Qt transient
  visual before private Rust retirement and plan publication; a missing or
  failing private boundary fails closed through typed tab recovery.

* Moved all authored reaction candidate construction, validation, complete-render admission, and one-use receipt commit authority from `ferrum-document` into `ferrum-document-render`. The document session now exposes no reaction-specific candidate or commit route: authored reactions use the renderer bridge and commit only through the generic complete-CDML compatibility transaction, while compatibility-loaded reaction records and their structural deletion guards remain lossless.

* Added the Rust-owned `reaction.create.v1` aggregate. It atomically creates a canonical direct-root reaction with ordered reactants, products, one arrow, optional conditions, and optional pluses. The bounded route validates durable direct-root kinds, rejects duplicate/cross-role/cross-reaction members, allocates collision-safe `rxn-N` IDs, preserves compatibility-loaded malformed records, and now prevents structural deletion of all referenced reaction roots.

* Corrected the compact Ferrum authoring ribbon's visible action language. Select Structure and all
  five vector tools now use required, shipped Ferrum icons while retaining their existing live
  QAction command owners, compact More Tools entries, accessibility metadata, and lifecycle.
  Narrow bond authoring now presents the readable `Next atom/bond defaults.` hint while assistive
  technology retains the full next-operation instruction.

* Corrected the Ferrum authoring ribbon's compact behavior: all live tools now share one
  optional-exclusive active-tool group, tool changes retire an active gesture through the existing
  cancellation authority, and the named accessible `More tools` menu retains the exact QAction
  instances for every authoring tool at narrow widths. Grid and snap controls remain direct.

* Started the renderer ownership inversion with a session-free
  `RenderDocumentModelV1` transfer schema and a one-way conversion from one
  accepted document observation. The model retains immutable render facts,
  source identity/order, molecule topology, presentation records, paper,
  drawing-standard, Telex profile, and diagnostics without CDML nodes, session,
  history, or toolkit ownership. Renderer lowerer migration remains in progress.

* Moved presentation-vector complete-render admission into the Rust document
  session. The session now retains the opaque renderer proof with its prepared
  capability; raw pending candidates, public preflight receipts, and direct raw
  vector commits are no longer available to Rust, Python, CLI, or Qt callers.
  Compatible CDML load and rewrite remain independent from authoring admission.

* Corrected Qt direct-Plus preview paint conversion: renderer RGB wire values
  now receive their one required `#` only at the Qt projection boundary.

* Replaced the direct Plus preview seam with a dedicated opaque Rust API facade.
  It derives the temporary Plus from the exact canonical insertion in a detached
  session and publishes only verified renderer layout, paint, and bounds; Qt
  can no longer submit caller-created appearance or enter Plus through the
  generic presentation-gesture API.

* Corrected Rust structural-interaction geometry: ordinary bond hits and full
  marquee containment now use rendered line stroke width rather than endpoint
  rectangles. Rendered path-only bond depictions remain visible as typed
  `DisplayOnly` structural targets and refuse authoring atomically instead of
  disappearing from interaction selection.
* Added Rust-authoritative direct atom/bond interaction observations and opaque
  atomic structural deletion handles for the Ferrum Qt path.

- Added an interruption checkpoint for M4b SMARTS dock integration, including accepted backend evidence, current Qt work, selected-query requirements, and the live-binding ownership conflict.

* Corrected the SMARTS dock Find-state lifecycle. One centralized Qt refresh rule now reacts to raw text edits, source-mode changes, selected-molecule availability, tab readiness, busy dispatch, and blocked retirement; whitespace-only input cannot begin a query.
- Added the separate `Attach Cyclohexane Ring` Qt action. It captures an eligible atom, paints
  only copied finite Rust C6 preview geometry, commits one opaque receipt on release, and retires
  the receipt and overlay on cancellation or stale/refused gestures without changing the detached
  `Insert Cyclohexane Ring` action.
# Cyclohexane attachment E2E

- Added failure-only, value-safe diagnostics to the isolated Qt CML new-document
  E2E. A failed valid queue completion now records only fixture labels, public
  completion success, scheduled-start acceptance, typed-refusal presentation
  facts, and pending/tab counts; it never includes CML or arbitrary file paths.

- Corrected the isolated C6 E2E's final ineligible-anchor exercise to use the
  real press-drag-release contract, so it reaches the intentional drag-time
  eligibility refusal without relaxing its typed, mutation-free terminal checks.

- Corrected the attached-cyclohexane root E2E to require generic history
  retirement after both undo and redo, then rearm C6 through its canonical
  visible-menu QAction before the Escape-cancel drag.

- Corrected the attached-cyclohexane root E2E to assert the intended sticky
  selected-tool state after a successful commit, then begin its Escape-cancel
  drag directly instead of redundantly toggling the shared QAction.

- Added passive pre-trigger ownership fences around snapshot and coordinate setup
  in the isolated C6 E2E. Each failure now identifies the exact setup phase and
  attach-action signal delta before canonical QAction activation.

- Strengthened the isolated attached-cyclohexane E2E startup invariant. Before
  menu discovery it now requires no line intent, active tool mode, or checked
  authoring action, and its failure payload records the real intent tool plus
  copied start/pending/preview presence and passive attach-action/mode traces.

- Corrected the isolated attached-cyclohexane E2E popup observation window to
  wait through the existing bounded 250 ms popup-handoff watchdog plus one
  queued-turn margin. Its 300 ms event-pumping deadline now exits as soon as
  the shared attach action and attach mode are both active, and records popup
  ownership facts on timeout.

- Corrected the Escape failure diagnostic to include the existing bridge facts,
  so a missing native preview reports its actual state instead of raising a
  diagnostic `TypeError`.

- Added one isolated offscreen root E2E for the native-wheel attached-cyclohexane workflow.

- Corrected the C6-only implicit atom picker to pass Rust-issued opaque atom IDs
  into the attachment bridge rather than authored CDML source IDs.
