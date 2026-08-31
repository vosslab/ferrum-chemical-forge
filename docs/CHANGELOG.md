# Changelog

Earlier history is in [CHANGELOG-2026-08k.md](CHANGELOG-2026-08k.md), continuing through
[CHANGELOG-2026-08j.md](CHANGELOG-2026-08j.md), [CHANGELOG-2026-08i.md](CHANGELOG-2026-08i.md),
and [CHANGELOG-2026-08h.md](CHANGELOG-2026-08h.md).

## 2026-08-30

### Fixes and Maintenance

- Synchronized shared style guides, tests, and repository support files from the starter template.

### Developer Tests and Notes

- Fixed the first-build bootstrap cycle in `build.sh`: candidate construction
  no longer sources `source_me.sh`, which correctly requires an already
  published local extension. The explicit build-lifecycle E2E now mirrors
  that refusal and proves a clean checkout can publish its first runtime.
- Fixed aggregate hygiene failures in the V2 measurement corpus while keeping
  XML parsing on repository-standard hardened `lxml`; runtime shape checks
  replace `typing.cast`, runnable measurement scripts carry executable modes,
  and BKChem provenance links use their tracked upstream GitHub documents.
- Added `SECURITY_DECISIONS.md` and recorded the Python XML security decision:
  `lxml.etree.XMLParser` is secure at this boundary because every instance
  explicitly disables DTD loading, entity resolution, network access, recovery,
  and huge-tree mode; production CDML parsing remains Rust/`xot` owned.
- Updated PyO3 geometry-operation success fixtures to use renderer-admissible
  40-unit drawing geometry, so rotation and angle normalization exercise their
  semantic boundary without violating document-session overlap admission.
- Added `GLYPH_BOND_MEASUREMENT.md`, a maintainer guide to the independent
  raster evidence boundary, developer lanes, output artifacts, measurement
  statistics, thresholds, and the distinction between a green instrument and
  current red bond-glyph alignment.
- Completed an independent audit of the V2 glyph-bond measurement stack.
  Unified the Rust/Qt raster-manifest basename, made the Rust gate locate the
  repository from Git, documented its final-footprint field, removed an unused
  duplicate single-manifest CLI, and clarified the automated glyph-acceptance
  boundary in architecture, usage, and historical V1 records. The audit also
  leaves the public Rust test-support boundary, duplicated Qt aggregate
  receipt, and permanent-test lane violations as explicit follow-up work.
- Added the V2 glyph-bond measurement evidence report. It records the closed
  pixel-only boundary, native and real-Qt evidence lanes, current strict-red
  receipts, rejected calibration experiments, and commands that distinguish a
  healthy measurement stack from unresolved renderer quality.
- Recorded and removed two one-time Rust-private outline-clip experiments. A
  raw convex core support increased the native strict receipt from 15 to 18,
  while a support dilated by the current rectangle-clearance kernel reached 10
  native findings but rebuilt Qt replay regressed to 17 findings, including
  full-label collisions. The unchanged renderer restores the 15-finding native
  receipt and the frozen Qt expected-red baseline; future calibration requires
  both lanes to improve together.
- Refined Rust-private endpoint footprints to distinguish round endpoint ink
  from wedge-only transverse width and explicit axial extensions. The policy
  now clips against the actual directional final footprint rather than using a
  symmetric wedge-radius approximation. The native 12-fixture strict receipt
  improved from 22 to 15 findings, eliminating all opposed solid/hashed-wedge
  topology and connection findings without changing the independent policy.
  The Haworth-front terminal reserve restores the installed Qt contract and
  reduces Qt replay to eight detached endpoints and seven label overlaps; its
  higher-resolution native raster now reaches the same fifteen strict findings.
  The measurement metric
  gives each final parallel lane equal weight, avoiding antialiasing-area drift
  in a geometrically centered double/triple attachment.
- Added `measure_stack.batch`, the common aggregate receipt for real raster
  evidence. The Rust developer lane now publishes an atomic `run_summary.json`
  alongside every per-fixture JSON report, overlay, and contact sheet, with
  stable failure categories derived only from the independent pixel policy.
  Its current 12-fixture receipt reports 26 strict-policy violations across
  seven renderable fixtures; this is evidence of remaining Rust geometry work,
  not a relaxed baseline.
- Refreshed the frozen Qt expected-red receipt after the parallel-lane
  correction. The actual consumer now reports two centerline misses and seven
  target-overlap findings instead of nine target-overlap findings; all other
  categories are unchanged and strict acceptance remains red.
- Recorded the font-ownership transition rule: Atkinson Hyperlegible is the
  preferred written-UI/prose candidate and mononoki remains fixed-width-only.
  A molecule-label change requires a versioned Rust font resource and fresh
  native/Qt measurement evidence; Qt may not replace the face locally.
- Corrected the independent wedge-topology predicate to sample terminal widths
  inside the final footprint rather than at fixed fractions of the atom-center
  span. Reversed or label-clipped wedges now measure as connected expanding
  footprints. This removes one false native violation (25 remain) and one
  obsolete Qt expected-red category without changing renderer geometry or the
  endpoint collision/gap policy.
- Corrected double/triple lowering to compute one axial endpoint clip across
  the complete parallel-lane footprint before emitting symmetric lanes. This
  removes an invalid per-lane ownership split; regenerated native evidence
  still has seven strict-red fixtures, so style-specific geometry remains open.
- Added the active glyph-bond visual-quality goal. It defines the owned
  `measure_stack/` artifact, noncircular pixel evidence, normalized target/gap/
  axis/collision thresholds, style and scene predicates, normal-scale Qt QImage
  capture, a versioned visual fixture corpus, deterministic diagnostics, and
  the Rust correction loop. During that intermediate V1 stage, its raster
  script remained a narrow regression lane rather than visual acceptance.
- Finalized the V2 goal record against the implemented hash-bound layer
  contract, 12 authoritative renderable fixtures, seven named synthetic
  negatives, deterministic pixel oracle, and actual offscreen Qt consumer
  capture. It now distinguishes the passing expected-red baseline from the
  still-failing strict Rust/Qt geometry gate, removes manual-review language,
  and reserves green acceptance for a later Rust geometry correction.
- Made the unpublished measurement lane V2-only. Removed the V1 manifest
  reader, loose legacy metric policy, generic Haworth alias, Qt fixture fallback,
  and compatibility launcher; renamed the Rust developer entry point to
  `devel/run_measure_stack_rust.sh`. Product render DTO contracts remain
  unchanged.
- Split the closed V2 capture contract into explicit `presentation` and
  `raw_final_ink` evaluation semantics. Native 8x diagnostic rasters retain
  all pixel-integrity, connection, and chemistry geometry checks without being
  misclassified by a user-viewport framing policy; actual Qt captures remain
  strict fixed-profile presentation evidence. The normal-bond clip policy is
  now also the sole font-clearance owner in test construction, eliminating the
  remaining test-only duplicate factor.
- Calibrated normal, bold, dashed, and wavy endpoint behavior against native
  final-ink evidence. Dashed lowering now serves both clipped endpoints,
  wavy clipping reserves its endpoint cap rather than its later amplitude, and
  V2 measurement distinguishes endpoint attachment from intentional mid-bond
  topology. Parallel double/triple lane-envelope admission remains an active
  Rust geometry item; a shared-zero-offset experiment was rejected because it
  regressed measured decorated diagonal endpoints.
- Refreshed the Qt expected-red category receipt after rebuilding the staged
  local runtime from the Rust clip/style corrections. The baseline remains
  strictly capture-healthy and red for unresolved geometry; only observed
  category counts changed, not the V2 metric policy or fixture catalog.
- Replaced the one-size Qt presentation viewport with graph-authored fixed
  profiles for the 12 visual fixtures. Strict Qt evidence no longer reports
  systematic occupancy or under-framing failures; its remaining reports are
  endpoint/style/capture facts tied to individual Rust render owners.
- Classified measurement verification by execution model. Removed the brittle
  hardcoded profile-count assertion and CLI smoke test from the permanent
  pytest lane; native, Qt, and CLI rebuild evidence remains in explicit
  developer gates.
- Corrected the Rust-private atom endpoint contract so final bond clipping uses
  the exact structural core glyph rather than the full decorated label
  rectangle. Full-label rectangles remain complete-plan collision evidence.
  The regenerated native V2 corpus improved from nine red renderable fixtures
  to seven without relaxing measurement thresholds; parallel lanes and
  stereochemical/Haworth endpoints remain open style-specific work.
- Rebuilt the staged local runtime and refreshed the observed Qt expected-red
  baseline after the core-glyph and directional-decoration corrections. The
  consumer baseline now records eleven detached endpoints, one missing endpoint
  connection, one orphaned core, one style-topology failure, and nine
  target-overlap failures; strict visual acceptance remains red.
- Removed the slow synthetic-runner artifact rebuild from permanent pytest.
  That check exercised real CLI publication, 19 fixture manifests, images, and
  contact sheets, so it remains an explicit developer-oracle action instead of
  violating the repository's fast deterministic pytest lane.
- Refined the Rust-private atom-label clipping model to retain exact Telex
  bounds for each non-core run. Bond endpoints now attach to the structural
  glyph and clip around only decorations that actually lie on their approach
  ray, rather than treating the decorated label's aggregate rectangle as ink.
  The bromine decorated-label representative remains green; isotope and
  phosphorus collisions remain visible strict-red geometry evidence.

## 2026-08-29

### Developer Tests and Notes

- Added Ferrum-owned `devel/glyph_bond_alignment_measurement.py`, a developer-only
  NumPy/OpenCV raster measurement library. It derives centerline drift, signed
  endpoint-label gap, non-endpoint collisions, and composite footprint coverage
  from raster layers plus fixture graph identity only; it never consumes issued
  glyph bounds, attachment axes, clearances, clipped endpoints, or Qt objects.
- Hardened that measurement handoff with a closed, bounded JSON/raster manifest,
  identity and cross-field validation, checked diagnostic/report publication, and
  a deterministic pixel oracle for metrics, collision detection, and generated
  diagnostics. The explicit developer gate no longer depends on manual review.
- Added default-off Rust test support that reuses the ordinary Telex draw stream
  to emit 8x composites, source-identified core-glyph masks, and final bond
  footprints for the extended alignment corpus. The corpus now closes expected
  atom core runs plus source-bond style, display-layer, and operation-shape
  semantics; accepted 12-case baseline measurement has no collisions or gaps.

## 2026-08-28

### Behavior or Interface Changes

- Added frozen `BondAttachmentAxisV1` to every typed bond batch. Rust derives
  its center-to-center structural connection before clipping and transports it
  through PyO3 to Qt for validation only; it is neither painted nor hit-tested,
  so final visible ink continues to honor positive full-glyph clearance.
- Consolidated singular selected-root text export behind
  `document.molecule.export.v1` and one unversioned document export core. The
  protocol, PyO3, Qt, and CLI now share seven closed formats (Molfile V2000/
  V3000, SDF V2000/V3000, canonical SMILES, Standard InChI, and Fixed-Hydrogen
  InChI), typed refusals, preallocation output bounds, and atomic create-new
  CLI publication; plural multi-record SDF export remains separate.
- Added a modeless **Command Reference** on F1 and **Help > Command
  Reference...**. It shares one live metadata-derived command catalog with
  Command Palette, searches shortcut/help/YAML placement facts, reports current
  availability without activation, and restores focus after dismissal.
- Replaced the PyO3 render-observation transport in place with frozen V4/V2 DTOs. Render batches
  now expose one closed atom, compact-group, or bond content value; atom labels carry Rust-issued
  exact Telex bounds and core-run identity, while V3/V1 render-plan compatibility classes are gone.
- Corrected M4b SMARTS ownership: complete projection children now make the non-atom inventory
  explicit, the new `ferrum-graph-lowering` crate lowers capability-free facts only, and the API
  privately joins graph positions to durable IDs from exactly one accepted observation. The public
  current-document raw/selected query remains `document.molecule.smarts.query.v1`; historical
  directory, file, and broad search are not restored.

### Fixes and Maintenance

- Gave each successful CLI verb E2E publication its own absent fixture path so
  the conversion matrix verifies the create-new contract instead of requesting
  a second publication at the earlier SMILES output path.
- Standardized current Python test XML parsing on repository-standard `lxml`,
  with DTD loading, entity resolution, and network access disabled. This
  test-only parser choice is separate from production CDML's Rust `xot`
  ownership; archived changelog references to the one-off `defusedxml` corpus
  harness remain historical execution context.
- Reframed the current parity delivery record around completed slices, immediate
  human desktop review and aggregate rerun, and usability/feature-parity work;
  it explicitly retains P2 directory-sync fault injection as deferred and makes
  no parity or release-readiness claim.
- Migrated the production Qt molecule renderer to exact V2 observations and V4 closed batches.
  Atom labels now replay only their typed mask/text/decorations and declared core run, verify Telex
  full/core ink bounds and coordinate-space receipts, and preserve issued paint/layer order; compact
  groups and bonds consume their own typed operations without generic semantic scans.
- Unified attached compact-group pose admission with final normal-single bond clipping. The public
  V2 renderer facade now owns depiction/font resolution; document sessions pass only projections,
  catalog key, and release point. Internal glyph layout receipts and atom-bond construction are
  crate-private, while the documented `GlyphBounds` compact-group DTO remains public for the
  verified document-render consumer.
- Made the renderer-issued normal-single clip policy the sole owner of font-derived clearance for
  attached compact-group pose admission and final lowering. Every authorable compact-group family
  now proves its short-ray precommit exterior-bond overlay commits to the same catalog key.
- Split the PyO3 render transport into a small registration facade, generic primitive converter,
  closed V4/V2 render-plan converter, and typed content owner without changing Python class names,
  schemas, or frozen DTO behavior.
- Repaired the PyO3 V4 transport contract: batches now retain renderer-issued paint order, atom,
  compact-group, and bond payloads retain closed typed operations, and generic replay is derived
  only from those typed facts. Render issues also retain their serialized paint order, and the
  private live render-publication seam now truthfully identifies its V2 observation contract.
- Recorded the owner-observed atom-label/bond-line alignment defect and the prior OASA emphasis on
  rigorous alignment tests in `HUMAN_GUIDANCE.md`; technical interpretation remains outside the
  human-authored guidance record.
- Added the one schema-closed `atom_label_bond_alignment_cases_v1` JSON corpus at the authoritative
  document-to-V4 observation seam. Its twelve semantic rows prove emitted label/bond content,
  ordered operations, isotope core-run semantics, and target-specific third-label refusal without
  coordinates, computed bounds, or pixel snapshots.
- Published each atom label's validated positive `bond_ink_clearance` in the V4/V2 PyO3 transport.
  The artifact-dependent installed Qt consumer now expands exact full Telex ink by that issued gap
  and proves final bond ink remains disjoint. The shared consumer and real-window attached-ring
  gesture moved from deterministic pytest into the registered E2E lane, with a bounded Open wait;
  focused Qt pytest retains one behavioral projection check instead of private paint snapshots.
- Corrected the compact-group deletion E2E's canvas-tool contract: selecting a scene point now
  activates Select Structure only when it is not already checked, so a successful deletion that
  intentionally leaves the tool active is followed by a real atom selection instead of toggling the
  tool off. The public delete, report, undo, and materialize workflow is green again.
- Migrated the installed E/Z carrier-mark projection E2E from the removed generic batch operation
  bag to the exact closed `BondRenderBatchV1.typed_operations` payload, preserving a real Rust-to-Qt
  geometry assertion at the typed transport boundary.
- Recalibrated installed authoring, clipboard, template, regular-ring, and geometry-repair fixtures
  to Ferrum's native 40-point molecular scale. The tests now exercise renderer-admissible geometry
  instead of depending on 1--13 point bonds or gestures that crossed an unrelated atom label; the
  stronger final-ink admission contract remains intact.
- Moved the real-Qt existing/new stereo-bond gestures off an unrelated oxygen atom. Final-ink
  admission now remains exercised without opening an unobserved modal refusal during `mouseMove`,
  eliminating a deterministic aggregate-suite stall while preserving the public gesture route.
- Reconciled the active parity plan and render-metrics/CDXML decision records with the completed
  local aggregate: 8,297 hygiene tests, every registered CLI/Qt E2E, 299 installed PyO3 tests, and
  437 Qt tests pass. Human real-window/accessibility acceptance, remote CI, release, and full parity
  remain explicitly open.
- Strengthened generic renderer admission from root classification to a complete-render omission
  delta. Ordinary authoring can retain or repair existing imported diagnostics but cannot add a new
  root exclusion, plan issue, or member depiction issue; undo/redo instead authenticate an exact
  retained history target so repairs remain reversible without an authoring bypass.
- Corrected the catalog protocol regression to exercise that same omission-delta policy: stale and
  unknown requests still refuse, while an unrelated insertion may retain an already-authored text
  exclusion and must preserve both the prior text and the newly inserted catalog object.
- Repositioned the alpha-D-glucopyranose anomeric oxygen in the Rust-owned standalone Haworth
  recipe so its C1--O1 bond clears the non-endpoint C2 label under final-ink admission. The catalog
  integration now commits and verifies all four native D-glucose Haworth recipes, rather than one.
- Made the unversioned domain `LinearFormBondLength::NATIVE` the sole owner of Ferrum-generated
  linear-form spacing. Planning and exact CDML metadata now use 40 PostScript points; the duplicated
  10-point writable grammar and its inherently unrenderable generated geometry are gone.
- Renamed private renderer-admission modules, receipts, and errors in place without compatibility
  aliases; version suffixes remain reserved for durable serialized, public, cross-language, or
  cross-crate contracts.
- Restored the repository's authored-file size boundary by extracting immutable screenshot scene
  data, closed render-observation binding tests, and the delivered M6 command-palette detail into
  focused owner files. The canonical capture entrypoint, binding suite, and parity ledger remain
  below 1,000 lines without exemptions or behavior changes.
- Rebuilt the native runtime and recaptured all 13 Qt documentation scenes. The attached-ring scene
  now requires and visibly shows all seven bonds, including the original host C--O bond; an
  independent image review accepted label/bond alignment and visible count consistency, while human
  visual and accessibility acceptance remain open.
- Replaced the rejected document-owned SMARTS graph/identity shadow and eager observation lowering.
  Live reveal uses an entropy-backed, one-use API-private receipt: reservation precedes later
  validation, every post-reservation outcome consumes the receipt, lifecycle invalidation clears it,
  and unavailable entropy fails closed without a deterministic issuer. Private SMARTS helper and
  capability PyClasses are unregistered; internal types are unversioned, while durable closed error
  enums retain `V1`.

- Closed Local Open host ownership with a one-use resolution: pre-commit refusal returns the
  candidate only after rollback, while unresolved returns or exceptions retain possibly committed
  candidates. Extracted the Qt host transaction from the generic lifecycle module and added direct
  publication/replacement return-and-raise regressions for the previously unsafe pre-resolution window;
  55 focused tests and the 8,218-hygiene/294-PyO3/412-Qt aggregate gate pass.
- Reconciled plan authority, five-module ownership, and the sole Rust workspace/PyO3 package boundary;
  human accessibility, remote CI, release, M5.A, and full parity remain open.
- Corrected public rustdoc prose that linked a private lowering context and named a removed render
  constructor; documentation now describes the actual projection-owned resolver boundary.
- Refreshed the repository docset, added the authoritative packaged UI YAML contract, and regenerated
  the independently reviewed 13-scene GUI tour; docs now describe Rust-first Local Open and the one-workspace
  packaging boundary, while human accessibility, remote CI, release, M5.A, and full parity remain open.

### Developer Tests and Notes

- M4b Patch 3 automated acceptance is green: `./check_rust.sh`, `./build.sh`, public Rustdoc/Python
  isolation, packaged raw/selected CLI, installed PyO3 receipt lifecycle/module isolation, and the
  six-test offscreen Qt lifecycle lane. Real-window/human visual, CI, and release gates remain open;
  this does not close M4 or full parity.

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

- Reworked Template Catalog Patch 1 as a pure Qt registry/controller with a shared lifecycle close
  adapter and three-phase pristine replacement, removing the catalog mixins. `FerrumNativeDocumentTab`
  remains the Rust mutation port; M5.A and native human acceptance stay open.
- Reworked Local Document Open into contract, composition, controller, and delivery modules with a named
  queued relay, transactional cleanup, QWidget-owned dirty-dialog rechecks, and distinct failed versus
  post-commit completed outcomes. The old mixin and unused `native_app.py` are removed; Rust, PyO3, YAML,
  and local-open types are unchanged. Publication-ownership review and current full-gate evidence remain pending.
- Repaired format-neutral interchange placement in Rust: shared CML, CDXML, and SDF preparation
  observes the session paper and centers imported records there rather than at a hard-coded scene origin.
- Repaired molecule projection ownership without a `QGraphicsItemGroup` or Qt-side geometry authority.
  The completed detached hierarchy installs after validation, Rust owns root bounds, and member selection
  is preserved. Removed the unused legacy overlay that constructed the superseded parallel projection.

- Scoped PyO3 extension-only link mode to the production extension build instead
  of the workspace dependency graph. Rust binaries and tests now link normally on
  macOS, while the installed module retains the required dynamic extension mode.
  The Maturin project now targets the dedicated `crates/api-python/` wheel crate directly.

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

- Paused the stabilization audit after documenting non-acceptance in [TODO.md](TODO.md) and
  [ROADMAP.md](ROADMAP.md); resume with the recorded repairs, capture, and full gate.

- Completed a six-pass independent stabilization audit; its actionable findings are resolved
  above. The in-progress M5.A [Template Catalog V1](active_plans/decisions/m5_template_catalog_v1.md)
  replaces Qt filename authority with a bounded Rust snapshot and fenced placement without
  expanding M2/M4. M5.A, M5, native accessibility/visual acceptance, and full parity remain open.

- Pre-repair receipt: paused after the delivery-stabilization checkpoint with the exact remaining
  evidence gap in [ROADMAP.md](ROADMAP.md), [TODO.md](TODO.md), and [GUI_TOUR.md](GUI_TOUR.md).
  The fresh local build, 283 installed PyO3 tests, 283 Qt tests with one intentional skip, and
  registered E2E suite passed. The close-out aggregate run recorded 7,922
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
