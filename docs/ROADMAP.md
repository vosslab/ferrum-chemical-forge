# Ferrum roadmap

Ferrum is a pre-alpha chemistry-drawing application with a Rust-owned chemistry,
document, rendering, and admission core plus a PySide6 desktop client. Full-parity
scope and acceptance criteria live in
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md), while
this roadmap records the current delivery checkpoint. The accepted code-level
contracts below do not close the remaining visual, aggregate, or parity work.
Reconcile broader historical milestone and receipt status before approving the
next milestone. [TODO.md](TODO.md) contains the short dispatch queue for the
next work session.

## Paused checkpoint

Work is intentionally paused after the 2026-08-27 delivery-stabilization slice.
The pause is not a parity-complete claim.

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
- The GUI capture-driver/catalog repair and command-palette hierarchy/relevance
  repair are implemented with focused evidence, but each remains pending
  independent final review. The latter repairs dynamic-menu placement validation,
  YAML breadcrumbs, and relevance without relaxing YAML placement ownership.
- The last automated 13-scene capture completed, but human visual review rejected
  eight frames. The subsequent repairs have not been recaptured or visually
  accepted. No accepted complete tour is published from that run.
- The new `tests/test_guidance_doc_format.py` gate joins the next aggregate run.
  Aggregate-green status, the post-fix audit, broader historical milestone/receipt
  reconciliation, the separate unapproved M5A decision, and full parity remain
  open.

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

### 1. Close delivery stabilization

First independently review the two in-flight repairs: GUI capture-driver/catalog,
then command-palette hierarchy/relevance. Next run focused recaptures and the
complete 13-scene capture, followed by human visual review and publication of an
accepted artifact set. Then run the new guidance-format gate, complete
build/all-test/E2E/PyO3/Qt validation, and the post-fix audit against one staged
runtime. The accepted `ActionRegistry` lifecycle repair and nominal
document-display refresh boundary are completed preconditions, not work to repeat.

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

The proposed `PARITY-M5.A` package first needs an approved canonical-plan contract
or explicit decision showing that its catalog ownership boundary can proceed while
M2 and M4 remain open. The current architecture proposal is a candidate, not an
implementation authorization.

If approved, it replaces Qt filename-derived template-catalog authority with one
immutable, versioned Rust manifest. Rust supplies stable opaque keys, content
identity, provenance, compatibility, limits, admission, and refusal. PyO3 projects
those facts read-only, while the existing Qt palette retains search, labels,
accessibility, and interaction. Otherwise, resume with the highest-value package
whose declared dependencies are already satisfied.

### 3. Continue full parity

- M2 expands graph/interchange only through corpus-backed profile decisions.
- M4 completes reports, diagnostics, query, and admitted expansion workflows.
- M5 continues reactions, peptide/carbohydrate profiles, named groups, and wider
  curated catalogs after an approved Rust-owned catalog contract is sequenced
  through the canonical dependency decision.
- M6 completes keyboard-only authoring, accessibility, help, clipboard, logging,
  output, and application usability across delivered capabilities.
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
