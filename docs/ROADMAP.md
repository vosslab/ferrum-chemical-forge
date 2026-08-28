# Ferrum roadmap

Ferrum is a pre-alpha chemistry-drawing application with a Rust-owned chemistry,
document, rendering, and admission core plus a PySide6 desktop client. Full-parity
scope and acceptance criteria live in
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md), while
this roadmap records the current delivery checkpoint. The accepted code-level
contracts below do not close the remaining aggregate, human-release, or parity work.
Reconcile broader historical milestone and receipt status before approving the
next milestone. [TODO.md](TODO.md) contains the short dispatch queue for the
next work session.

## Current checkpoint

The 2026-08-27 visual delivery-stabilization slice is implemented and has a
reviewed candidate artifact set. This is not a parity-complete claim.

- The dated build, installed PyO3, Qt, registered E2E, and aggregate-test receipts
  are recorded in [CHANGELOG.md](CHANGELOG.md). The nominal
  `DocumentDisplayRefreshableV1` ABC boundary is delivered and independently
  accepted at code level: production registrants and valid test helpers inherit
  it, structural look-alikes are rejected, and direct forwarding evidence covers
  the delegating adapter.
- `ActionRegistry` destruction retirement is delivered and independently accepted.
  Token-guarded bindings retire the exact destroyed action without removing a
  successor that reuses its stable ID. Permanent regressions cover feature-owned
  retirement/palette dispatch and portable declaration rebinding/dispatch.
- The GUI capture-driver/catalog and command-palette hierarchy/relevance repairs
  are source-reviewed and recaptured. YAML retains placement authority, while the
  searchable palette uses validated breadcrumbs and relevance ordering.
- One transactionally regenerated 13-scene candidate tour passed all semantic
  postconditions and image-by-image agent visual review. The driver uses a
  non-persistent documentation theme and page-contained authored examples; final
  human release sign-off remains distinct.
- Rust now publishes measured molecule content bounds across PyO3. A dedicated
  noninteractive Qt ownership root uses those facts for content fitting while
  ordinary child items retain independent atom and bond selection. CML, CDXML,
  and SDF placement share Rust-owned observed-page centering.
- Four bounded parity slices have progressed without closing their milestones:
  M2.B Rust-owned File/Open catalog, M4.C required Rust-issued report
  identifiers, Rust-projected periodic next-drawing picker, and M6 structural
  keyboard selection. Its tab-owned opaque Rust selection-fence bridge passed
  57 focused Qt/PyO3 tests, its registered no-pointer keyboard E2E exited 0,
  and final review accepted with no P1; M2, M4, M6, and full parity remain open.
- The guidance-format gate, fresh build, complete aggregate suite, registered
  E2E, installed PyO3, full Qt, affected Rust package tests, and strict Clippy
  checks are green. Seven independent post-fix reviews completed; their stale
  contract/ledger, import style, dead projection, and split wheel-ownership
  findings are repaired. Broader historical milestone/receipt reconciliation,
  approved in-progress M5.A implementation, human sign-off, and full parity remain open.

## What works today

- Ordinary `ferrum-qt` starts one Rust-native desktop application. It opens
  admitted CDML, CML, and bounded CDXML inputs, performs typed document edits,
  supports Undo and Redo, saves/reopens CDML, and exports SVG, PDF, and PNG.
- Rust owns durable IDs, chemistry, document mutation, history, render plans,
  admission, provenance, and typed refusal. Qt owns interaction, accessibility,
  transient presentation, and focus.
- YAML owns the File, Edit, Draw, View, Chemistry, Options, and Help menu hierarchy
  and the ribbon projections of registered actions.
- Direct structure drawing, selected-root operations, regular rings, presentation
  vectors, reaction arrows, compact-group attachment/materialization, SMARTS,
  bounded atom oxidation, command-palette discovery, and semantic theme refresh
  have delivered Rust-first slices with focused evidence.
- Historical OASA and BKChem sources remain read-only reference material. They are
  not runtime dependencies or compatibility authorities.

## Resume order

### 1. Preserve delivery stabilization

The source review, focused diagnosis, complete 13-scene recapture, and agent
visual review are complete. The guidance-format, build, aggregate, registered
E2E, installed PyO3, full Qt, affected Rust test, and strict lint gates passed
against the fresh local runtime. The isolated wheel gate and seven independent
post-fix reviews also passed after their actionable findings were repaired.
Obtain final human visual sign-off when preparing a release and reconcile the
broader historical ledger before approving another milestone. The accepted
`ActionRegistry` lifecycle repair and nominal document-display refresh boundary
are completed preconditions, not work to repeat.

Success means:

- later windows and command palettes see only live registered actions;
- the default run emits all 13 files only after every scene succeeds from a
  1440x900 logical window and each raster is 16:10, and human review confirms each
  complete, uncropped Ferrum surface;
- the new guidance-format gate and `all_test.sh` reach every phase without
  repository-rule failures;
- permanent tests remain semantic, deterministic, offline, and focused;
- the final independent audit has no unresolved high-impact finding.

### 2. Decide the next parity package

`PARITY-M5.A` is approved and in progress. Its canonical
[Template Catalog V1 decision](active_plans/decisions/m5_template_catalog_v1.md)
establishes an independent catalog boundary while M2 and M4 remain open: it consumes their
delivered generic document admission and fenced placement contracts without expanding either
open corpus. Rust replaces Qt filename-derived catalog authority with one immutable, versioned
shipped-and-user snapshot supplying opaque keys, content identity, provenance, compatibility,
explicit entry/candidate/refusal/file/total-byte limits, bounded lexical admission, and aggregate
refusal occurrences. PyO3 projects those facts read-only and accepts a native-issued expected
document snapshot for placement; Rust owns publication capability/receipt. Qt's modeless dialog,
tab, and window owners provide one `chemistry.template.catalog` task without scanning,
re-admission, payload, plan, or raw OS-error authority.

Do not claim M5.A complete until its Rust, PyO3, Qt, manual native accessibility, and complete
aggregate evidence gates are recorded.

The automated gates are now recorded green: build produced CLI/GUI; focused catalog/API/PyO3/Qt
receipts were 13/164/8/18 passed; the public authoring E2E reported `ok`; workspace test and
strict Clippy exited 0; and the aggregate passed 8,092 hygiene checks, all registered E2Es, 294
installed PyO3, and 344 Qt tests. Three independent final reviews found no P1/P2/P3. Manual native
accessibility, contrast, and focus review plus a fresh real-dialog screenshot/human acceptance
remain open; M5.A and full parity therefore remain in progress.

The next Qt foundation work is the approved two-patch
[Qt Operation Lease Registry](active_plans/decisions/qt_operation_lease_registry.md). Patch 1 is
Template Catalog only: controller plus pure Qt lifecycle registry, both catalog mixins and
`CATALOG_PLACEMENT_BLOCKED` deleted, same-attempt close after synchronous cancellation, no Rust or
PyO3 change, then independent review/full Qt/public E2E. Patch 2 starts only after that acceptance:
Local Document Open proves source-retaining `CANCELLATION_REQUESTED` and truthful delivery
cancellation. No alias, event bus, service locator, or cosmetic wholesale tree move is authorized.

### 3. Continue full parity

- M2 expands graph/interchange only through corpus-backed profile decisions;
  M2.B does not complete its remaining scope.
- M4 completes reports, diagnostics, query, and admitted expansion workflows;
  M4.C does not complete its remaining scope.
- M5.A implements its approved Rust-owned Template Catalog V1 before M5 continues reactions,
  peptide/carbohydrate profiles, named groups, and wider curated catalogs.
- M6 completes native keyboard/accessibility sign-off, help, clipboard,
  logging, output, and application usability across delivered capabilities.
- M7 begins with explicit plugin and service security/product decisions.

## Design boundaries

- Ferrum-Chem owns chemistry and document authority; Ferrum Qt is the interaction
  and presentation client.
- RDKit remains behind one project-owned native adapter.
- CDML is the local editable/save format. Interchange profiles are admitted by
  explicit Rust contracts with loss/refusal reporting.
- YAML places registered actions but does not own chemistry or document semantics.
- `build.sh` builds the repository-local application for testing. Installation,
  publishing, and hosted workflows are separate future decisions.
- Screenshots and keyboard walkthroughs are human-reviewed delivery evidence, not
  pixel, byte, timing, or image-count pytest gates.

## Follow the work

Start with [README.md](../README.md) and [USAGE.md](USAGE.md). Use
[GUI_TOUR.md](GUI_TOUR.md) for the current screenshot workflow and
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md) for
milestone scope, dependencies, and evidence requirements.
