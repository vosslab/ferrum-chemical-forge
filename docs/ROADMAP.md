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
- The Rust renderer now owns atom-label/bond attachment end to end: it centers
  the typed core element run at the atom origin, treats every visible label
  mark as final-ink exclusion geometry, clips each bond against its actual
  style-specific ink footprint, and refuses a bond whose final ink would cross
  another atom label. The V4 plan/V2 observation boundary carries those
  renderer-issued facts unchanged; PyO3 and Qt consume the closed typed plan
  rather than reconstructing label geometry or chemistry.
- One schema-closed 14-case corpus covers ordinary, decorated, isotope,
  bond-style, Haworth, near-miss, coincident, and third-label-refusal cases at
  the Rust document-to-observation seam and the installed Qt consumer. The
  independent native and actual-Qt pixel lanes accept all 14 renderable
  cases with zero violations under the strengthened V2 policy. Broader desktop
  usability and accessibility acceptance remains a separate maintainer gate.
- The Rust-first direct drawing, selected-root, regular-ring, presentation,
  compact-group, SMARTS, bounded oxidation, command-palette, and semantic-theme
  slices have focused evidence. The bounded CDXML profile now also imports
  Wavy, Bold, and Dash bond presentation through Rust-owned document and render
  contracts. These deliveries do not close their parent milestones or full
  parity.
- The 13-scene documentation tour has source, semantic, and agent-visual
  evidence. It remains a release candidate until a maintainer performs fresh
  native visual and accessibility acceptance.

## Resume order

### 1. Preserve the accepted renderer alignment contract

The label/bond repair is deliberately a renderer contract, not a Qt placement
tweak. Rust is the sole owner of molecule-label metrics, exact core-run placement,
full-ink exclusion, final bond-ink collision admission, and paint order. The
closed V4 plan and V2 observation now give PyO3 and Qt the exact facts required
to replay that result without independent geometry. The canonical semantic
corpus must remain the permanent guardrail as more label decorations, bond
styles, fonts, and chemistry grammars are added.

The current Atkinson Hyperlegible Next Regular cutover is complete at this
contract boundary. The synthetic oracle makes all seven deliberately negative
scenes reach their named rejection category; native final ink and actual Qt
each accept all 14 renderable fixtures with zero violations. The 17 permanent
measurement tests and 16-case installed semantic alignment E2E are green;
aggregate gate receipts are recorded after each renderer change.

For every later font, label-decoration, bond-style, or capture-profile change,
the Rust-render owner updates the exact font resource/metrics or geometry and
the Qt owner preserves replay-only behavior. Success is zero violations in the
unchanged native and actual-Qt V2 lanes. Validation is `./build.sh`, both strict
measurement scripts, `./check_rust.sh`, and `./all_test.sh`. Broader M6 native
visual and accessibility acceptance may judge legibility and interaction, but
does not replace or reopen this pixel-geometry contract without new evidence.

### 2. Carry M4b SMARTS Patch 3 through external acceptance

The implemented M4b correction makes the projection's non-atom inventory
closed, moves graph construction into the pure downward-only
`ferrum-graph-lowering` crate, and limits graph-plus-durable-ID correspondence
to API-private target construction from one accepted observation. Live Qt
reveal uses entropy-backed one-use receipts that fail closed if entropy is
unavailable. The canonical Rust gate, sealed build, public Rustdoc/Python
isolation oracle, packaged raw/selected CLI, installed PyO3 receipt lifecycle,
and offscreen Qt lifecycle are green. No private SMARTS helper/capability
PyClass is registered; internal types are unversioned, while durable closed
error enums retain `V1`. Real-window/human visual acceptance, CI, and release
evidence remain open; M4 and full parity remain open. The active acceptance record is
[M4B_SMARTS_QUERY.md](active_plans/active/M4B_SMARTS_QUERY.md).

### 3. Carry M2 CDXML presentation through acceptance

The Rust-owned CDXML simple-molecule profile now admits `Display="Wavy"`,
`Display="Bold"`, and `Display="Dash"` only on non-directed single bonds. A
source-specific validated carrier preserves those facts until the document
adapter creates the sole durable `DocumentBondPresentationV1` state. CDML
persists them as `s1`, `b1`, and `d1`; the native renderer creates their clipped
geometry; and generic CLI, PyO3, and Qt new-document admission publish only a
clean rendered candidate. This keeps chemistry, durable document semantics,
rendering, and client presentation in their respective ownership boundaries.

Adder and Dotted are future parity work. Each must begin with a Rust-owned
renderer and document-semantics decision, then receive an interchange profile
only when corpus evidence supports it. They are not source-compatibility aliases
or narrow fallbacks in the delivered CDXML profile.

The implementation does not close M2. Fresh real-window and human visual and
accessibility acceptance, CI, release evidence, the remaining M2 corpus, and
full parity remain open. The authoritative profile and acceptance criteria are
in [M2 CDXML simple-molecule import](active_plans/decisions/m2_cdxml_simple_molecule_import_v1.md).

### 4. Maintain Local Open stabilization evidence

The Local Document Open mixin has been replaced with five explicit modules:
immutable contract facts, callback-only window composition, a source-tab
controller, per-intent delivery, and a Qt host transaction. It retains an exact source-tab
`LOCAL_DOCUMENT_OPEN` / `BLOCK_UNTIL_SETTLED` lease; cancellation waits for the
worker finish, stale requests refuse without reanchoring, and background
completion cannot steal focus.

The current repair makes the host return `None` and close one explicit resolution.
A pre-commit refusal returns candidate ownership only after full rollback; an
irreversible publication or replacement transfers candidate ownership and
`COMPLETED` truth before receipt validation. Any unresolved return or exception
retains the candidate conservatively. Later presentation can report recovery but
cannot rewrite committed truth.

The stabilization acceptance is green: `./build.sh` exited 0; the focused Local
Open suite passed 55 tests; registered
`ferrum-local-document-open-lifecycle-e2e-v1` reported `ok`; and `./all_test.sh`
exited 0 with 8,218 hygiene checks, every registered E2E, 294 PyO3 tests, and
412 Qt tests. The repaired one-workspace `./check_rust.sh` fmt/check/strict
Clippy/test/doc gate exited 0. Final architecture review ACCEPT found no P1/P2/P3.
The independent screenshot review ACCEPTed all 13 frames. Public open/save/reopen,
nested dirty-dialog, and post-commit recovery remain registered E2E behavior.

The old Local Open mixin and unused `native_app.py` are removed. This repair
does not change Rust runtime/semantic APIs, PyO3, YAML, or
`local_document_open_types.py`, introduce
a Qt/Rust lease bridge, add an event bus or service locator, or claim M5.A,
M5, or full parity complete. Human native accessibility, contrast, focus-ring,
remote-CI, and release acceptance remain separate gates.

### 5. Complete Template Catalog acceptance

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

### 6. Continue full parity

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
