# Ferrum roadmap

Ferrum is a pre-alpha chemistry-drawing application with a Rust-owned chemistry,
document, rendering, and admission core plus a PySide6 desktop client. Full-parity
scope and acceptance criteria live in
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md). This
roadmap names the strategic delivery order; [TODO.md](TODO.md) is the short
dispatch queue for the next work session.

## Current checkpoint

Ferrum has a Rust-native desktop and CLI foundation, bounded chemistry and
interchange slices, and an explicitly owned Qt operation-lifecycle foundation.
That is substantial progress, not a feature-parity or release-complete claim.

- Rust owns durable IDs, chemistry, document mutation, history, render plans,
  admission, provenance, typed refusal, and public CLI operation contracts.
  PyO3 transports narrow Rust-issued facts; it does not become a second domain
  model.
- The ordinary `ferrum-qt` application opens admitted CDML, CML, and bounded
  CDXML inputs, edits documents, supports Undo and Redo, saves/reopens CDML,
  and exports SVG, PDF, and PNG.
- YAML owns the registered File, Edit, Draw, View, Chemistry, Options, and Help
  action placement and ribbon projections. Qt owns interaction, accessibility,
  and disposable presentation, never chemistry or durable document facts.
- Historical OASA and BKChem material remains read-only provenance. It is not a
  runtime dependency, compatibility host, or architectural authority.
- The Rust-first direct drawing, selected-root, regular-ring, presentation,
  compact-group, SMARTS, bounded oxidation, command-palette, and semantic-theme
  slices have focused evidence. Their delivery does not close their parent
  milestones or full parity.
- The 13-scene documentation tour has source, semantic, and agent-visual
  evidence. It remains a release candidate until a maintainer performs fresh
  native visual and accessibility acceptance.

## Resume order

### 1. Finish Local Open stabilization

The Local Document Open mixin has been replaced with four explicit modules:
immutable contract facts, callback-only window composition, a source-tab
controller, and per-intent delivery. It retains an exact source-tab
`LOCAL_DOCUMENT_OPEN` / `BLOCK_UNTIL_SETTLED` lease; cancellation waits for the
worker finish, stale requests refuse without reanchoring, and background
completion cannot steal focus.

The current repair makes the host issue explicit receipts at Local Open's
irreversible new-tab-publication and replacement-commit boundaries. Delivery
must transfer candidate ownership at those receipts; later activation or
presentation failure can report recovery but cannot dispose a committed tab or
misreport a completed document transition as a rollback. Pre-commit refusal
must still retire the provisional candidate and preserve the exact source tab
and source lease.

Finish this stabilization slice before accepting it or starting another
high-coupling Qt migration:

- Add fault-injection regressions immediately after new-tab publication and
  immediately after replacement commit, plus pre-commit rollback coverage.
- Confirm exact receipt identity, registered-tab ownership, source-lease
  settlement, truthful `COMPLETED` post-commit outcomes, and no focus theft.
- Run the focused Local Open/lease/CDML tests, native build, registered E2E,
  aggregate suite, Rust workspace tests, strict Clippy, and a fresh independent
  architecture review on the resulting worktree.
- Keep public open/save/reopen, nested dirty-dialog, and post-commit recovery
  in the registered E2E lane; retain deterministic contract tests in pytest.

The old Local Open mixin and unused `native_app.py` are removed. This repair
does not change Rust, PyO3, YAML, or `local_document_open_types.py`, introduce
a Qt/Rust lease bridge, add an event bus or service locator, or claim M5.A,
M5, or full parity complete. Native visual, VoiceOver, contrast, focus-ring,
remote-CI, and release acceptance remain separate gates.

### 2. Complete Template Catalog acceptance

`PARITY-M5.A` [Template Catalog V1](active_plans/decisions/m5_template_catalog_v1.md)
is an approved independent Rust snapshot and placement boundary. It consumes
delivered generic admission and fenced placement contracts without expanding the
still-open M2 graph/interchange corpus or M4 report, diagnostic, query, and
expansion corpus.

Rust owns the immutable shipped-and-user catalog snapshot, opaque key/content
identity, provenance, compatibility, explicit entry/candidate/refusal/file and
total-byte limits, bounded lexical admission, aggregate refusal occurrences,
and publication capability/receipt. PyO3 projects those facts read-only. Qt
owns the modeless dialog, `TemplateCatalogController`, and
`OperationLeaseRegistry` lifecycle; `FerrumNativeDocumentTab` is the sole Rust
mutation port for `chemistry.template.catalog`.

Its automated implementation evidence is recorded in the canonical plan:
build, focused Rust/API/PyO3/Qt checks, public authoring E2E, workspace test,
strict Clippy, and aggregate validation passed at the recorded checkpoint.
M5.A remains open until a fresh real-dialog screenshot and native
accessibility, contrast, and focus review receive human acceptance. It also
remains one M5 slice, not an implied catalog, reaction, or parity closure.

The accepted Patch 1 operation-lease foundation is retained while this work
continues. It provides the pure Qt registry/controller boundary, removes the
catalog mixins and `CATALOG_PLACEMENT_BLOCKED`, and establishes registry-aware
clean-tab close behavior. Its narrow provisional-registration repair is not a
general tab-replacement atomicity contract; Local Open owns its own receipt
transaction.

### 3. Continue full parity

- M1 completes dependable direct-structure editing through its declared P0.1
  normal-bond and P0.2 selected-root contracts.
- M2 expands chemistry graphs and interchange only through corpus-backed Rust
  profile decisions. The delivered File/Open catalog and CDXML slices do not
  close M2.
- M4 completes reports, diagnostics, query, and admitted expansion workflows.
  Bounded attached-group and report-identifier deliveries remain prerequisites,
  not a claim of general chemistry-operation parity.
- M5 continues curated templates, reactions, peptides, carbohydrates, named
  groups, provenance, and deterministic exchange after the bounded catalog
  slice has genuine native acceptance.
- M6 completes usability across delivered capabilities: keyboard authoring,
  accessibility, help, clipboard, logging, output, and native human review.
- M7 starts only with an explicit plugin and service security/product decision;
  no historical OASA/BKChem plugin or service surface is adopted by default.

## Design boundaries

- Ferrum-Chem is the authoritative chemistry and document system. Ferrum Qt is
  its interaction and presentation client.
- RDKit remains behind one project-owned native adapter.
- CDML is the editable local/save format. Every interchange profile enters
  through an explicit Rust contract with typed loss or refusal reporting.
- YAML places registered actions and ribbon clients; it does not own chemistry,
  document semantics, or durable lifecycle state.
- `build.sh` constructs the repository-local application for verification.
  Installation, hosted workflows, publication, and release artifacts have
  separate decisions and evidence.
- Screenshots and keyboard walkthroughs are human-reviewed delivery evidence,
  never pixel, byte, timing, or image-count pytest gates.

## Follow the work

Start with [README.md](../README.md) and [USAGE.md](USAGE.md). Use
[GUI_TOUR.md](GUI_TOUR.md) for the screenshot workflow,
[TODO.md](TODO.md) for the next dispatches, and
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md) for
the authoritative milestone scope, dependencies, and evidence requirements.
