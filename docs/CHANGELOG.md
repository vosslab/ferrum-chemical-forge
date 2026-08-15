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
