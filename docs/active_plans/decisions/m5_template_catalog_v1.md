# M5 template catalog V1

## Decision

`PARITY-M5.A` is approved for implementation as one Rust-authoritative, immutable
Template Catalog V1. It promotes the existing Ferrum shipped catalog and user-template
workflow into one versioned catalog and one fenced placement task. This is an
implementation authorization, not a completion claim. The active milestone authority is
[FULL_PARITY_RUST_FIRST.md](../active/FULL_PARITY_RUST_FIRST.md).

M2 and M4 remain open but do not block this slice. M5.A consumes their delivered generic
document admission, prepared user-template plan, session transition, durable identity, and
fenced placement contracts. It neither expands the M2 graph/interchange corpus nor changes
the M4 reports, diagnostics, query, or chemistry-expansion corpus.

## Ownership and dependencies

```text
ferrum-domain: shipped recipe and provenance manifest
        |                         ferrum-document: CDML admission, plan, session mutation
        +-----------------------------------+
                                            v
                  ferrum-template-catalog: snapshot, user-directory admission, selection
                                            |
                                            v
                              ferrum-api: frozen PyO3 DTO and dispatch
                                            |
                                            v
                   Ferrum Qt: task surface, labels, focus, accessibility, intent
```

Add the acyclic `ferrum-template-catalog` workspace crate. It may depend on
`ferrum-domain`, `ferrum-document`, `rustix`, `sha2`, and `thiserror`; `ferrum-api`
depends on it. `ferrum-domain` remains the owner of shipped authored recipes and their
provenance. `ferrum-document` remains the sole owner of CDML admission, budgets,
prepared plans, durable IDs, renderer admission, history, and save/reopen.
`ferrum-catalog-placement` remains the shipped-recipe lowerer. It must not ingest an
untrusted filesystem directory.

Qt is a projection client. It must not scan a directory, derive a key, hash content,
re-admit CDML, retain a plan or payload, or reopen a path. Replace the current Python
user-template scan/key/hash/payload authority; do not create a compatibility adapter.

## Frozen catalog contract

The new crate exposes an immutable, versioned snapshot with Rust-issued entries and typed
refusals. Public types include `TemplateCatalogSnapshotV1`, `TemplateCatalogEntryV1`,
opaque `TemplateCatalogKeyV1`, `TemplateContentIdentityV1`, `TemplateCatalogSourceV1`,
`TemplateCompatibilityV1`, `TemplateCatalogLimitsV1`,
`TemplateCatalogSelectionV1`, `TemplateCatalogRefusalV1`, and
`TemplateCatalogApplyErrorV1`.

- A snapshot has schema `ferrum-template-catalog-v1`, a catalog version, entries, typed
  refusals, and explicit `max_entries`, `max_candidates`, `max_refusals`, `max_file_bytes`,
  and `max_total_bytes` limits. It is immutable; refresh replaces it. A prior selected snapshot
  remains valid because it owns admitted bytes and a prepared plan, never a live path or FD.
- Content identity is SHA-256 over the exact descriptor-admitted bytes before UTF-8 decode and
  document admission. A user key is the opaque, deterministic
  `user-template:v1:<sha256-lowerhex>` value. Identical valid content is one entry with bounded
  source-name aliases; rename is stable and changed bytes have a new key.
- Shipped keys retain their existing Rust-issued stable IDs. Shipped identity is the SHA-256 of
  the canonical Ferrum-authored recipe descriptor, not a Rust debug representation. Each entry
  explicitly carries source,
  provenance, compatibility, and selection facts. A user entry must not inherit Ferrum license
  or review claims.
- Shipped ordering is manifest order; user ordering is key-byte order. Consumers preserve Rust
  ordering. Qt must not invent a category/order policy.
- User compatibility is `ferrum-document-user-template-profile-v1` plus CDML. Shipped entries
  name their closed native recipe profile. No OASA/BKChem branch, runtime parsing, historical
  coordinates, object IDs, or corpus import is authorized.
- `max_file_bytes` equals `document_user_template_budget_v1().max_utf8_bytes`. Candidate,
  entry, refusal, and total-byte limits are explicit snapshot facts. Native reads stop at
  `max_file_bytes + 1`; no unbounded directory, descriptor, or refusal collection is permitted.

## Secure directory admission and refusal

Rust performs descriptor-relative direct-child admission. It opens configured directory
components with `RDONLY|DIRECTORY|NOFOLLOW|CLOEXEC`; a missing leaf is an empty state, while a
configured-directory open failure is snapshot-wide. It accepts only lowercase `.cdml` UTF-8
direct-child names; non-CDML neighbors are ignored by the lexical candidate policy. Rust opens
each candidate relative to the retained directory FD with `RDONLY|NONBLOCK|NOFOLLOW|CLOEXEC`,
verifies that same FD is regular, then bounded-reads, hashes, decodes, and calls
`prepare_user_template_v1` before closing it.

A swap before open is refused; a swap after open cannot alter admitted bytes. Placement never
reopens a source path. Candidates are selected in bounded lexical order; refusals are bounded,
and a repeated limit category carries an aggregate `occurrences` fact. Snapshot-wide errors cover
configured-directory `directory_open_failed` and allocation. Entry-local categories cover
directory symlink/non-directory, non-UTF8 filename, candidate symlink/nonregular/open/read/size/
limit, UTF-8/document admission, and duplicate content. These scan/admission outcomes are the
complete `TemplateCatalogRefusalCategoryV1` domain. Placement and application failures are
separate typed `TemplateCatalogApplyErrorV1` outcomes: selection, document, renderer, and session
recovery is mapped once at PyO3/Qt. Neither error domain is a free-form exception category.

This enforces ASVS 5.1-5.3 safe file/input handling. ASVS 1.5 is met by one-way crate ownership
and one PyO3 conversion. ASVS 2.1-2.3 are met because only Rust issues selections and existing
revision/digest fences authorize mutation. For ASVS 15.3, diagnostic logging may contain a
stable category, opaque key, and basename only; it must not record CDML payload or arbitrary
full paths.

## PyO3, unified task, and accessibility

PyO3 exposes a frozen snapshot and the single selection route:

```text
snapshot_template_catalog_v1(directory) -> snapshot
session.place_template_catalog_entry_v1(snapshot, key, expected_document_snapshot, x, y)
```

The Python object privately retains native selection data but exposes no plan, descriptor, path,
raw CDML, recipe, or parser. The native-issued expected document snapshot carries the revision and
digest mutation fence; Python cannot construct equivalent authority. The binding validates/copies
input once, calls Rust while detached, and converts native errors once. Native dispatch uses the
existing shipped placement or user-plan route according to the private selection variant.

Qt provides one searchable **Template Catalog** task with visible `Built-in` and `My templates`
sources. It owns labels, layout, source/facet/search controls, provenance and compatibility
wording, focus, and pointer/keyboard handoff. It projects typed outcomes and uses one registered
action, `chemistry.template.catalog`, across YAML menu and authoring clients. `Save Current as
Template` remains the publication route and Refresh remains task-local. The placement intent is
exactly `(tab, viewport, expected_document_snapshot, snapshot, key)`.

Rust owns a prepared user-template publication capability and receipt. Qt supplies presentation
only: it cannot retain CDML, re-admit a saved template, scan the directory, or surface raw OS
errors. This decision's former dialog/tab/window mixin split is superseded by the later
[Qt Operation Lease Registry](qt_operation_lease_registry.md): dialog presentation,
an explicit controller, pure Qt lifecycle registry, and the `document_tab.py` native placement
port now own their distinct responsibilities without compatibility modules.

The task must preserve useful healthy entries beside malformed neighbours; distinguish empty,
no-match, loading, partial-failure, unavailable, save-refusal, stale, and accepted-display
recovery states; and never automatically retry a refused mutation. Its keyboard order is source,
facets, search, results, details, secondary actions, Place, Cancel. Enter arms only an admitted
selection; Escape closes or disarms as appropriate. Armed placement moves focus to the canvas and
announces `Click canvas to place; Escape cancels`; cancellation/refusal restores a predictable
catalog focus target. Controls need names, descriptions, roles, text-equivalent state, measured
4.5:1 text and 3:1 control/focus contrast, and screen-reader announcements for refresh, armed,
cancellation, and refusal states.

## Scope, exclusions, and evidence

M5.A promotes all currently shipped Ferrum entries but adds no new chemistry corpus. It excludes
legacy OASA dictionaries, GPL-only payloads, historical branding, filename/CDML-object identity,
attachment/fusion, plugin/external catalogs, public CLI local-directory protocol, peptides,
carbohydrates, atom-mapped reaction import/export, SMARTS/SMIRKS, and publication-write policy.

Implementation is in progress until all gates pass:

1. Rust proves order, key/content invariants, provenance/compatibility/limits, deduplication,
   secure direct-child admission, refusal, immutable old snapshot, selection, and no mutation on
   stale/unknown/refusal; it also proves shipped/user insert, undo/redo, and save/reopen.
2. Installed PyO3 proves frozen DTO fields, typed outcomes, key round-trip, and no payload/plan
   exposure.
3. Qt uses real registered actions and visible controls for shipped/user browse, eligible save and
   immediate visibility, malformed neighbour, empty/no match, refresh, cancellation, armed
   Escape/tool/tab/focus retirement, stale recovery, and accepted display recovery.
4. A manual native walkthrough records keyboard focus, visible focus, Enter/Escape, accessible
   names/roles/states, high contrast, recovery language, and an updated real-dialog screenshot.
5. `cargo fmt --all -- --check`, `cargo test --workspace`, strict Clippy, clean PyO3
   build/import, relevant Qt tests, and `./all_test.sh` pass. Permanent tests stay semantic,
   deterministic, offline, and non-pixel-based.

Current automated receipt: `./build.sh` produced CLI and GUI; focused catalog, API, PyO3, and Qt
tests passed 13, 164, 8, and 18; the public authoring E2E schema
`ferrum-template-catalog-authoring-e2e-v2` reported `ok`; `cargo test --workspace` and strict
workspace Clippy exited 0; and `./all_test.sh` passed 8,092 hygiene checks, all registered E2Es,
294 installed PyO3, and 344 Qt tests. Three independent final reviews found no P1/P2/P3. This
closes no manual gate: native accessibility, contrast, focus, fresh real-dialog screenshot, and
human acceptance remain required before M5.A can complete.
