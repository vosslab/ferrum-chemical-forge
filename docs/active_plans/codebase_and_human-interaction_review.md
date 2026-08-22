# Ferrum: codebase and human-interaction review, with a convergence plan

## Context

Ferrum replaces the Python `oasa` chemistry backend with a Rust engine
(Ferrum-Chem) and rebrands the BKChem PySide6 frontend as Ferrum-Qt. The
migration is far along: `docs/active_plans/ferrum-plan-v3.md` marks M1a-M18
done, with M19-M22 open. Production source carries no `oasa` import, and
branding is clean.

This review was asked for because two things do not match the stated intent.
First, the shipping `ferrum` CLI is not usable the way OASA was. Second, the
Qt frontend was supposed to differ from `bkchem-qt.app` only by branding and by
`import oasa` -> Rust calls, but the two trees have structurally forked, so
recent BKChem work cannot land.

The review confirms both, and finds a third: the interaction layer has real
gaps - no keyboard shortcuts at all - that the milestone ledger does not track,
because the ledger measures capability plumbing rather than usability.

The repository is pre-production with no users. That is the reason to fix the
design now rather than route around it.

**Decisions taken (user, this session):**

1. Re-converge the frontend structure so BKChem changes port mechanically.
2. Add a verb CLI layered over the frozen protocol core; keep `protocol run`.
3. All four workstreams are near-term.
4. BKChem is **not** a source of truth. The 1000-line limit governs; BKChem's
   3809-line `main_window.py` violates it. Convergence means the trees are
   *more common than not*, not that Ferrum adopts god files.

**Revision r2** answers an external review that asked for dispatchable-task
scale. Four design questions are now resolved with evidence below (convergence
definition, verb-to-protocol mapping, adapter responsibilities, geometry
ownership), one is converted to an explicit spike (worker abstraction), and the
work is expressed as owned tasks rather than workstreams.

---

## Findings

### F1. The CLI is a JSON-RPC endpoint, not a tool

`crates/api/src/cli.rs:17-38` defines one command family:
`ferrum protocol schema` and `ferrum protocol run INPUT [--output OUTPUT]`.

Converting a SMILES string means hand-authoring a JSON envelope naming schema
`ferrum-operation-request-v1`, an opaque `request_id`, and one of four
operations, writing it to a file, then reading an envelope back
(`docs/USAGE.md:25-50`). No piping, no verbs, no default output.

The capability exists: `crates/api/src/` carries SMILES, InChI, SDF, molblock,
and SMARTS codecs, coordinate generation, SVG/PDF/PNG publication, geometry
repair, linear form, Haworth composition, and peptide templates - ~95 modules,
17.8k lines - none reachable from a shell.

The plan's own open question, "CLI subcommand surface", with decision rule
"derive from the batch and export capabilities already enumerated in the M1b
matrix" (`ferrum-plan-v3.md:2458-2462`), was answered at M19 by *retiring* the
provisional command families rather than by designing the surface.

### F2. The frontend fork

Deleted from Ferrum, still present and actively developed in BKChem:

| Area | Files gone |
| --- | --- |
| `actions/` | 21, incl. `action_registry.py`, `menu_builder.py`, `context_menu.py`, `platform_menu.py` |
| `modes/` | 22, incl. `mode_manager.py`, `draw_mode.py`, `edit_mode.py`, `arrow_mode.py`, `vector_mode.py` |
| `models/` | 12, incl. `document_session.py`, `document_object.py`, `atom_model.py` |
| `undo/`, `setup/` | all |
| `widgets/` | `toolbar.py`, `mode_toolbar.py`, `submode_ribbon.py`, `edit_ribbon.py`, `periodic_table.py`, `property_dock.py`, `status_bar.py`, `zoom_controls.py`, `color_picker.py` |
| `io/` | `export.py`, `clipboard_manager.py`, `cdml_document_io.py`, `render_plan.py`, `snapshot_render.py`, `format_bridge.py`, `import_capabilities.py` |
| `dialogs/` | `about_dialog.py`, `preferences_dialog.py`, `text_dialog.py`, `theme_chooser_dialog.py`, `molecule_info_dialog.py`, `drawing_standard_dialog.py` |
| `canvas/items/` | `atom_item.py`, `bond_item.py`, `arrow_item.py`, `text_item.py`, `group_item.py`, `mark_item.py`, `render_ops_painter.py` |
| resources / config | `resources/menus.yaml`, `resources/modes.yaml`, `config/keybindings.py` (300 lines), `geometry.py`, `canvas/view.py` |

In their place: `ferrum_qt/native/`, 76 files, 18,831 lines, every file named
`ferrum_native_*`. `main_window.py` went 3809 -> 307 lines - a genuine
improvement, and the version that respects the repo's 1000-line rule.

BKChem commits `f3a6b2f` (2026-08-11) and `f8fd0e6` (2026-08-12) touched
precisely the deleted layer: `modes/arrow_mode.py`, `modes/vector_mode.py`,
`modes/bracket_mode.py`, `modes/edit_mode.py`, `widgets/mode_toolbar.py`
(responsive layout), `widgets/property_dock.py`, `widgets/status_bar.py`,
`widgets/zoom_controls.py`, plus new `models/bracket_pair_selection.py`,
`actions/presentation_property_capture.py`,
`dialogs/geometric_properties_dialog.py`, and a rewrite of
`io/cdml_document_io.py`. None has a landing site in Ferrum.

Drift runs both ways: BKChem deleted `wavy_geometry.py` at `f3a6b2f`; Ferrum
still ships it.

Left on disk: six **empty directories** - `actions/`, `modes/`, `models/`,
`setup/`, `undo/`, `legacy/` - untracked by git, invisible in review. Also
`packages/ferrum-chem-qt.app/build/`, gitignored but present, containing a
stale `oasa_bridge.py`: the only `oasa` module name left anywhere and a false
positive generator for every future audit.

### F3. Interaction gaps the capability ledger cannot see

The capability matrix marks nearly every row Supported. It measures whether a
Rust operation is reachable, not whether a person can work.

- **Zero keyboard shortcuts.** No `QKeySequence`, no `setShortcut`, no
  `StandardKey` anywhere in `ferrum_qt/`. Not Ctrl+S, Ctrl+Z, or Ctrl+O.
  BKChem has `config/keybindings.py` (300 lines) plus wiring in `main_window.py`,
  `actions/context_menu.py`, `actions/platform_menu.py`, `widgets/toolbar.py`.
  For an editor whose core loop is repeated small edits, this is the largest
  usability defect in the product, and no gate detects it.
- **No tab order.** 0 `setTabOrder`; 2 `setFocusPolicy`.
- Partial affordances elsewhere: 45 `setAccessibleName`, 85 `setToolTip` - a
  real foundation, unevenly applied.
- **Refusals are system-centric and terminal:** `"Unsupported Save Format"`
  (`window_native_files.py:183`), `"Native Atom Properties Unavailable"`,
  `"Native Atom Number Unavailable"`
  (`native/ferrum_native_main_window.py:338,369`), `"CDML Document Rejected"` /
  `"The admitted file is not supported typed CDML."`
  (`native/ferrum_native_cdml_open.py:972`). They name the implementation
  boundary, not the user's next step. Two do better - the peptide hint at
  `native/ferrum_native_molecule_imports.py:429` and the compressed-SVG message
  - so the good pattern already exists in-house.
- **Missing standard dialogs.** No About, Preferences, or theme chooser.
  `FQ-020`/`FQ-021` claim these Supported; settings are read/written through
  `config/preferences.py:32-56` with no UI to reach them.
- **Documentation register.** `README.md:6-17` is a single ~150-word sentence.
  `docs/USAGE.md` opens on JSON envelope schemas and transport budgets. Both
  read as evidence ledgers for a reviewer, not instructions for a chemist.

### F4. Boundary discipline

- **No Python-side adapter.** `import ferrum_chem` at ~115 call sites across
  ~55 files, mostly function-local, inside dialog builders, menu handlers,
  canvas painters, and workers. `bridge/` covers 5 concerns. The plan promises
  the Python API boundary stability (`ferrum-plan-v3.md:302`); there is no
  single seam to freeze.
- **14+ bespoke QThread subclasses**, one per feature, each re-implementing
  prepare-off-thread / commit-on-GUI-thread.
- **Python still owns geometry and presentation rules:**
  `wavy_geometry.py:16-89`, `native/ferrum_native_hex_grid.py:78-104`,
  `canvas/ferrum_spline_path.py`, `bridge/display_geometry.py`,
  `bridge/insertion_placement.py`, `config/geometry_units.py`,
  `bond_presentation.py`.
- **`crates/api` is a flat god-crate:** ~95 files directly in `src/`, 17.8k
  lines, holding chemistry codecs, render artifacts, Haworth, peptides,
  clipboard, document operations. Unit tests sit as `*_tests.rs` siblings in
  `src/` - neither layout `docs/RUST_STYLE.md` section 14 describes.
- **One `unsafe` outside the FFI crate:** `crates/api/src/smiles_inspection.rs`.
  All others are correctly confined to `chemistry-sys`.
- Positive: `thiserror` throughout, no `anyhow` in libraries, no TODO/FIXME
  markers, `ChemEngine` a single trait at `crates/chemistry/src/engine.rs:84`,
  RDKit closure genuinely isolated.

### F5. Process and hygiene

- **No CI.** No `.github/`, no workflows, no hooks. `all_test.sh` and
  `check_rust.sh` exist and are correct; nothing runs them automatically.
- **The fast lane has no behavioral tests.** `pytest tests/` collects 5560
  cases from 16 hygiene/lint modules parametrized over every file. Behavior
  lives in `packages/*/tests` and `tests/e2e/`.
- **Tracker inconsistency.** M19 is `not started` in the table while its body
  says "complete, pending independent closure review"
  (`ferrum-plan-v3.md:2005`). M17/M18 use `complete` where others use `done`.
- **Plan file scale.** 2463 lines; the M16 section alone ~900.
- **Version drift.** `VERSION` `26.08`, Qt `pyproject.toml` `26.08`, api python
  `pyproject.toml` `26.8.0`.
- **File-size limit gamed.** `native/ferrum_native_line_tools.py` is exactly
  999 lines; four more native modules sit at 930-990.
- `docs/TODO.md` missing; `docs/RELATED_PROJECTS.md` thin at 36 lines.

---

## Resolved design decisions

These were open at r1 and are settled here so tasks are dispatchable.

### D1. What convergence means

Convergence is defined by **named portable seams**, not by a similarity
percentage. A seam is portable when a BKChem change to it transfers with a
rename and no redesign.

| Seam | Status | Rule |
| --- | --- | --- |
| `resources/menus.yaml`, `resources/modes.yaml` | Shared | Menu/mode structure is declarative in both trees; a BKChem entry applies verbatim modulo action ids |
| `config/keybindings.py` | Shared | Same key-to-action-id table shape |
| `actions/` action ids and registry API | Shared | Same id vocabulary and registration call shape; bodies differ |
| `modes/` mode ids and lifecycle hooks | Shared | Same mode names, same enter/exit/event hook names |
| `widgets/` public widget API | Shared | Same widget class names and slots (`property_dock`, `status_bar`, `mode_toolbar`, `zoom_controls`, `periodic_table`) |
| `dialogs/` field names and result shapes | Shared | Same dialog names and returned field keys |
| Document/session model | **Ferrum-specific** | Ferrum's lives in Rust; BKChem's `models/` does not transfer |
| Canvas item painting | **Ferrum-specific** | Ferrum paints Rust render plans; BKChem's per-object items do not transfer |
| Chemistry calls | **Ferrum-specific** | `import oasa` has no Ferrum equivalent by design |
| File sizes and decomposition | **Ferrum-specific** | Ferrum's 1000-line limit governs; BKChem god files never transfer |

**Success is defined per seam, not in aggregate**: for each Shared row, a named
BKChem change applies to Ferrum with rename-only edits. `diff -rq` overlap is
**implementation-time measurement only** - recorded in the task receipt, never
a permanent gate, because percentage overlap would reward superficial
similarity and penalize legitimate Ferrum decomposition.

### D2. Verb-to-protocol mapping

The frozen protocol has exactly four operations
(`crates/api/src/protocol_v1.rs:75-88`): `document.inspect`,
`document.validate`, `document.rewrite`, `document.render_artifact`.

| Verb | Maps to | Protocol work needed |
| --- | --- | --- |
| `ferrum inspect DOC` | `document.inspect` | none |
| `ferrum validate DOC` | `document.validate` (`structural` / `typed`) | none |
| `ferrum render DOC -o OUT.{svg,pdf,png}` | `document.render_artifact` | none |
| `ferrum rewrite DOC -o OUT` | `document.rewrite` | none |
| `ferrum convert IN -o OUT` | **nothing** | new `chemistry.convert` operation |
| `ferrum coords DOC` | **nothing** | new `document.generate_coordinates` operation |

So four verbs are pure CLI work over the frozen core, and two require an
**additive** protocol operation. Additive is compatible with the M17 freeze:
existing schemas and envelopes are untouched, unknown-operation rejection
already exists, and the schema is generated from Rust types. The two new
operations are their own tasks (T5, T6) sequenced before the verbs that need
them (T7b), so no verb reaches past the protocol into the crates.

### D3. Adapter responsibilities

The Python adapter is defined by responsibility, not by an import count, so it
does not become a second god-module.

`ferrum_qt/ferrum/engine.py` owns exactly four things:

1. Importing `ferrum_chem` (the only module that may).
2. Converting Qt/Python inputs into frozen DTOs and back.
3. Mapping `ferrum_chem` exceptions to Ferrum-Qt error values.
4. Enforcing revision/digest fencing on every mutating call.

It does **not** own operation catalogues. Each feature keeps its own
operations module (`ferrum/atom_properties.py`, `ferrum/molecule_imports.py`,
...) that calls `engine` - so `engine.py` stays small and feature code stays
where a reader expects it. The import-hygiene test protects this decision; the
responsibility list above is what review checks.

### D4. Geometry ownership

Each Python-side computation is classified against an owning crate before any
migration, so cleanup does not relocate today's ambiguity into Rust.

| Python source | Owner crate | Reason |
| --- | --- | --- |
| `wavy_geometry.py` zigzag path | `render` | A depiction path; output is a render op |
| `canvas/ferrum_spline_path.py` | `render` | Same: curve geometry consumed only by painting |
| `bond_presentation.py` bond-order labels | `render` | Depiction policy; `depiction_profile_v1` already lives near it |
| `bridge/display_geometry.py` | `render` | Presentation projection; joins existing `presentation_vector_lowering_v1` |
| `config/geometry_units.py` pt/mm | `geometry` | Pure unit arithmetic over the shared coordinate space |
| `bridge/insertion_placement.py` | `document` | Placement respects document provenance and revision |
| `native/ferrum_native_hex_grid.py` | **stays Python** | Grid is application view state (`FQ-020`), not document or render state; migrating it would move view state into the document engine |

The rule that produced the table, for future cases: *if it describes what the
document contains, it belongs to `document`; if it describes how the document
is drawn, `render`; if it is coordinate arithmetic independent of both,
`geometry`; if it is per-user view state, it stays in Qt.*

### D5. Worker abstraction - spike first

The 14+ QThread subclasses are clearly duplicated, but whether one helper or a
small family is the durable design depends on semantics not yet compared. T12
is an investigation task producing a comparison table (cancellation, progress,
result type, error mapping, GUI-thread commit, revision fencing) across all
workers; T13 implements whatever that table justifies. This follows the repo's
*use the scientific method* principle rather than assuming uniformity.

---

## Tasks

Each task is independently completable, with one owner, one outcome, one
verification. Parallel groups can run concurrently.

### Group A - no dependencies, start immediately

**T1. CI workflow** (`maintainer`)
Add `.github/workflows/ci.yml` running `pytest tests/`, `all_test.sh`, and
`check_rust.sh` on push and PR. Do not reimplement any check.
*Verify:* workflow runs green on the current tree.

**T2. Repository hygiene sweep** (`maintainer`)
Delete the six empty directories (`ferrum_qt/actions`, `modes`, `models`,
`setup`, `undo`, `legacy`) and the stale `packages/ferrum-chem-qt.app/build/`
tree. Sync `VERSION`, `packages/ferrum-chem-qt.app/pyproject.toml`, and
`packages/ferrum-rust/crates/api/python/pyproject.toml` to one CalVer string.
Add `docs/TODO.md`; expand `docs/RELATED_PROJECTS.md`.
*Verify:* `pytest tests/`; no `oasa` filename anywhere on disk.

**T3. Plan-file restructure** (`planner`)
Split `ferrum-plan-v3.md`: keep tracker plus milestone definitions; move
accumulated evidence prose (M16's ~900 lines especially) into
`docs/active_plans/reports/`. Normalize `complete` -> `done`; reconcile M19's
table row with its body.
*Verify:* `pytest tests/test_markdown_links.py`; tracker table fits one screen.

**T4. Stray `unsafe` closure** (`coder`)
`crates/api/src/smiles_inspection.rs`: move the `unsafe` behind the
`chemistry-sys` wrapper, or add a `// SAFETY:` comment plus a recorded
exemption.
*Verify:* `check_rust.sh`; `unsafe` appears only in `chemistry-sys` or carries
a SAFETY comment.

**T12. Worker semantics spike** (`reviewer`)
Compare all 14+ QThread subclasses on cancellation, progress reporting, result
type, error mapping, GUI-thread commit, and revision/digest fencing. Produce
`docs/active_plans/reports/qt_worker_semantics.md` with a table and a
recommendation: one helper, or a named small family.
*Verify:* every worker appears in the table with file:line.

### Group B - CLI (depends on nothing in Group A)

**T5. `chemistry.convert` protocol operation** (`expert_coder`)
Add an additive V1 operation converting between SMILES, InChI, molblock, SDF,
and CDML through the existing codecs. Regenerate the checked-in schema.
*Verify:* Rust semantic tests per direction; unknown-operation rejection still
passes; existing four operations' envelopes unchanged.

**T6. `document.generate_coordinates` protocol operation** (`expert_coder`)
Add an additive V1 operation regenerating 2D coordinates for a document's
molecules, preserving centroid and mean bond length as the existing native
route does. Regenerate the schema.
*Verify:* Rust semantic test; coordinates satisfy the M4c tolerance already
recorded in `reports/coordinate_parity_v1.md`.

**T7a. Verb CLI over existing operations** (`coder`)
Add `inspect`, `validate`, `render`, `rewrite` to `crates/api/src/cli.rs` plus
a new `crates/api/src/verb_cli/` directory, one file per verb, each well under
999 lines. Every verb constructs a `ferrum-operation-request-v1` and calls the
existing executor - no verb reaches past the protocol. Extension-based format
inference overridable by `--from`/`--to`; `-` means stdin/stdout; exit codes
match the existing 0/1/2/3 contract; human diagnostics to stderr; `--json`
switches stdout to the envelope.
*Depends on:* nothing. *Verify:* T8.

**T7b. Verbs needing new operations** (`coder`)
Add `convert` and `coords` on the same pattern.
*Depends on:* T5, T6. *Verify:* T8.

**T8. Verb CLI E2E** (`tester`)
`tests/e2e/e2e_ferrum_verb_cli.py`: round-trip each verb against
the frozen `protocol run` surface from a staged local `build/bin/ferrum`.
Assert exit codes, stdin/stdout composition, file publication, and semantic
equivalence. The engine verbs prove the executable resolves its adjacent
`build/runtime/engine-v1` closure without a per-user installation.
*Depends on:* T7a, T7b.

**T9. `--help` that teaches** (`coder`)
Every verb's help shows a worked example in user vocabulary, not schema names.
*Depends on:* T7a.

### Group C - frontend structure (T10 gates the rest)

**T10. Rename `native/` -> `ferrum/`** (`coder`)
`git mv packages/ferrum-chem-qt.app/ferrum_qt/native
packages/ferrum-chem-qt.app/ferrum_qt/ferrum`, then `git mv` each file to drop
the redundant `ferrum_native_` prefix (`ferrum/atom_properties.py`, ...).
Update imports. No behavior change.
*Verify:* `pytest packages/ferrum-chem-qt.app/tests`; app starts.

**T11. Single Python adapter** (`expert_coder`)
Create `ferrum_qt/ferrum/engine.py` owning exactly the four responsibilities in
D3. Route all ~115 `import ferrum_chem` sites through it; feature modules keep
their own operations and call `engine`. Add a fast pytest asserting exactly one
module imports `ferrum_chem`.
*Depends on:* T10. *Verify:* new hygiene test; existing Qt tests pass.

**T13. Worker consolidation** (`expert_coder`)
Implement whatever T12's table justifies - one helper or a named family.
*Depends on:* T10, T12.

**T14-T19. Reinstate the shared seams** - one task per seam, each parallel
after T10, each landing declaration + registry + Rust call in its own files,
all under 1000 lines:

| Task | Seam | Files |
| --- | --- | --- |
| T14 | Menu/mode declarations | `resources/menus.yaml`, `resources/modes.yaml` |
| T15 | Action registry | `actions/action_registry.py`, `menu_builder.py`, `context_menu.py`, `platform_menu.py` |
| T16 | Mode manager | `modes/mode_manager.py` plus per-mode files |
| T17 | Widgets | `widgets/property_dock.py`, `status_bar.py`, `mode_toolbar.py`, `zoom_controls.py`, `periodic_table.py` |
| T18 | Keybindings | `config/keybindings.py` |
| T19 | Standard dialogs | `dialogs/about_dialog.py`, `preferences_dialog.py`, `theme_chooser_dialog.py` |

Each verifies by: the seam's declaration drives the running app, existing Qt
tests pass, and a named BKChem change to that seam applies with rename-only
edits (recorded in the task receipt).

**T20. Port BKChem `f3a6b2f` and `f8fd0e6`** (`coder`)
Apply the arrow/vector/bracket/edit mode work, responsive `mode_toolbar`,
`property_dock`, `status_bar`, `zoom_controls`, `bracket_pair_selection`,
`presentation_property_capture`, and `geometric_properties_dialog` changes onto
the restored seams. Delete Ferrum's stale `wavy_geometry.py` (superseded
upstream; math moves to Rust in T22).
*Depends on:* T14-T19. *Verify:* per-feature Qt tests; each ported change named
in the changelog entry.

**T21. Convergence receipt** (`tester`)
One-time `diff -rq` measurement plus a per-seam portability check, written to
`docs/active_plans/reports/frontend_convergence.md`. **Not** a permanent gate.
*Depends on:* T20.

### Group D - interaction (T18 gates T23)

**T22. Geometry migration** (`expert_coder`)
Move each computation to the crate named in D4. `hex_grid` deliberately stays
in Python.
*Depends on:* T10. *Verify:* `check_rust.sh`; render output semantically
unchanged on the corpus.

**T23. Keyboard workflow** (`coder`)
Wire `config/keybindings.py` through the action registry; use
`QKeySequence.StandardKey` for New/Open/Save/SaveAs/Undo/Redo/Cut/Copy/Paste/
Quit. Every menu item gets a shortcut or a recorded exemption. Add explicit
`setTabOrder` in every dialog and a canvas focus policy.
*Depends on:* T15, T18. *Verify:* T25.

**T24. Refusal rewrite** (`coder`)
Convert refusals to "what happened / why / what now" in user vocabulary. Start
with `ferrum/main_window_support.py`'s shared helper (~34 call sites), then the
per-feature strings in F3. Model: the peptide hint at
`ferrum/molecule_imports.py:429`.
*Verify:* no user-visible string contains "native", "admitted", or "typed
CDML"; every refusal names a next step.

**T25. Keyboard-workflow E2E** (`tester`)
`tests/e2e/e2e_keyboard_workflow.py`: drive one complete drawing task using
**only** key events - open a document, add an atom, add a bond, undo, save -
and assert the resulting document. This proves the failure in F3 is corrected,
which a metadata check alone cannot. Plus a fast structural pytest: every
`QAction` has text and a shortcut or exemption; every dialog sets tab order;
every interactive widget has an accessible name.
*Depends on:* T23.

**T26. Documentation register** (`planner`)
`README.md` leads with what the tool does plus one worked example.
`docs/USAGE.md` opens with `ferrum convert`, `ferrum render`, and a drawing
walkthrough; the protocol envelope reference moves to
`docs/FERRUM_API_CONTRACT.md` (already named at `ferrum-plan-v3.md:2417`).
Precise boundary language is kept, below the task content.
*Depends on:* T8, T9. *Verify:* `pytest tests/test_readme_first_paragraph.py`,
`test_markdown_links.py`.

### Group E - Rust structure

**T27. Decompose `crates/api`** (`expert_coder`)
Move chemistry codecs to `chemistry`, artifact publication to `render`,
Haworth/peptide to `domain`, document operations to `document`. What remains in
`api` is protocol, CLI, and PyO3 bindings. Introduce module subdirectories
instead of ~95 flat files; move `*_tests.rs` into `#[cfg(test)]` modules or
`tests/` per `docs/RUST_STYLE.md` section 14; drop `_v1` filename suffixes
where the type already carries the version.
*Depends on:* T5, T6, T7a (avoids conflicting edits to the same crate).
*Verify:* `check_rust.sh`; `ferrum protocol run` and every verb behave
identically before and after.

---

## Sequencing

- **Wave 1, fully parallel:** T1, T2, T3, T4, T12, T5, T6, T10.
- **Wave 2:** T7a, T7b (after T5/T6), T11, T13, T14-T19, T22 - all parallel
  after their gates.
- **Wave 3:** T8, T9, T20, T23, T24.
- **Wave 4:** T21, T25, T26, T27.

T10 gates only the Qt tasks; the entire CLI group, the Rust group, CI, hygiene,
plan restructuring, and the worker spike proceed independently of it, so the
rename does not serialize the project.

Milestone alignment: Groups B, C, and D belong to M19's closure - capability
rows claimed Supported without a usable interaction path are not closed. Group
C convergence should land before M22 declares a supported boundary.

---

## Reuse rather than rebuild

- `crates/api/src/protocol_v1.rs` executor - every verb routes through it
- `crates/chemistry/src/engine.rs:84` `ChemEngine` - already the single
  chemistry seam; no new trait
- `tests/file_utils.py` `discover_files` + `REPO_HYGIENE_FILTERS` - the
  established pattern for the adapter-boundary and accessibility checks
- `all_test.sh`, `check_rust.sh` - CI calls these, does not reimplement them
- `devel/changelog_lib.py` and friends - changelog updates
- BKChem's `menus.yaml` / `modes.yaml` schema and `keybindings.py` table shape
  - proven declarative shapes to adopt, decomposed, not copied

## Verification summary

| Concern | Check | Kind |
| --- | --- | --- |
| Verb CLI | staged `build/bin/ferrum` semantic equality with `protocol run`, including its adjacent local engine runtime | permanent E2E |
| Adapter boundary | one module imports `ferrum_chem` | permanent fast pytest |
| Keyboard workflow | key-events-only drawing task completes | permanent E2E |
| Accessibility structure | action/shortcut, tab order, accessible name | permanent fast pytest |
| Convergence | per-seam portability plus one `diff -rq` measurement | one-time receipt |
| Rust | `check_rust.sh` after each Rust task | permanent gate |
| Repo | `pytest tests/`; the 1000-line limit keeps decomposition honest | permanent gate |

## Repo conventions

- Every change gets a `docs/CHANGELOG.md` entry under the right subsection.
- All moves use `git mv`.
- Only humans commit.
- Rust is rustfmt-formatted 4-space; Python stays tab-indented.
- New source files stay under 1000 lines - the forcing function for the
  frontend decomposition, not an obstacle to it.
