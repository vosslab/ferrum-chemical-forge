# Plan: Rust-first Ferrum feature parity

## Context

Ferrum is a working Rust-native CDML editor, not yet a full replacement for the
read-only BKChem/OASA reference.  The 2026-08-19 inventories identify 23 absent
Qt workflows and reopened backend gaps in interchange, graph coverage, editor
grammar, chemistry operations, reactions, catalogs, and optional integrations.

This is the forward roadmap for a complete usable Ferrum application and
Rust-first OASA replacement.  It supersedes prior parity-related drops in
`docs/active_plans/ferrum-plan-v3.md` only for that expanded goal.  It neither
claims parity already exists nor restores Python OASA, a Python document model,
or reference code as a runtime dependency.  `OTHER_REPOS/` remains read-only.

## Objectives

- Deliver an ordinary Ferrum window in which users author, revise, inspect,
  exchange, and reopen every supported document grammar element.
- Replace production OASA behavior with Rust contracts and a bounded RDKit
  adapter, never a Python compatibility host or fallback.
- Expose every completed non-library operation through the Ferrum CLI and, for
  desktop workflows, the Qt application.
- Prove parity by durable user behavior and semantics rather than source or menu
  imitation.

## Design philosophy

Follow the repository's design-first philosophy.  Rust owns durable data and
semantics, Qt owns interaction and accessibility, and RDKit provides bounded
perception behind project-owned traits.  A parity workflow is complete only
when it has a typed Rust contract, a usable client, and durable evidence.

## Scope

- Reopen every documented missing frontend workflow and backend gap as
  dependency-ordered Rust-first work.
- Establish one versioned `document.command` request/result vocabulary for
  selection, mutation, preview admission, history, and typed refusal.
- Expand document, chemistry, interchange, render, domain, CLI, and Qt
  contracts needed for declared parity.
- Record semantic corpus and ordinary user-workflow evidence for each claim.

## Non-goals

- Port Python OASA modules, Python session/history state, or Qt graphics items
  as durable authority.
- Load arbitrary third-party plugins or call live network services before their
  separate security/service contracts are approved.
- Promise universal IUPAC, arbitrary carbohydrate interpretation, or lossless
  interchange where the admitted grammar cannot prove it.

## Current state summary

The current narrow core supports CDML lifecycle, bounded graph/codecs,
coordinates, SVG/PDF/PNG, ordinary Rust session editing, and six protocol-backed
CLI verbs.  The frontend inventory has 22 complete workflows, 8 deliberately
different equivalents, and 23 missing workflows; 20 missing workflows require
new Rust contracts.  P0 begins with a direct normal-bond gesture, then adds
selected-root operations only after reliable render hit/bounds facts exist;
history, save, and reopen fence each mutation.  P1/P2 then complete
presentation, chemistry, interchange, catalogs, and ecosystem parity.

## Architecture boundaries and ownership

`ferrum-document` owns typed records, transactions, CDML, history, and
`document.command`.  `ferrum-chemistry` owns exchange graphs, codecs, query
normalization, and adapter traits.  `ferrum-domain` owns reports, catalogs,
reactions, peptide, carbohydrate, and nomenclature values.  `ferrum-render`
owns semantic render plans and lowerings.  `ferrum-api` owns only CLI/protocol
transport.  Rust alone owns IDs, selection eligibility, mutation admission,
loss reports, and persistent render facts.

Qt owns menus, dialogs, tool state, pointer/keyboard capture, transient
previews, settings, and accessibility.  It must never mutate projections or
hold a shadow document.  RDKit supplies sanitization, canonicalization, stereo,
valence, SMARTS, and admitted exchange operations behind Rust traits; it is not
the durable graph or public DTO.

### Mapping (milestones / workstreams -> components / patches)

| Milestone | Components | Review boundary |
| --- | --- | --- |
| M0 | capability ledger, corpus, format descriptors | contract/corpus patch |
| M1 | document, render, PyO3, canvas | direct-authoring patch |
| M2 | chemistry, document, API, file routes | graph/interchange patch |
| M3 | document, render, Qt tools | presentation patch |
| M4 | chemistry, domain, API, chemistry dialogs | chemistry-operation patch |
| M5 | domain, document, template palettes | catalog/reaction patch |
| M6 | Qt actions/help/accessibility | usable-application patch |
| M7 | plugin/service boundaries | security/service design review |

## Milestone plan

| M | Title | Summary | Goal |
| --- | --- | --- | --- |
| M0 | Parity contract and corpus | Freeze claims, descriptors, and representative inputs | Prevent menu-only parity. |
| M1 | P0 direct structure editor | P0.1 normal-bond gesture, then P0.2 selected-root contracts | Make routine drawing dependable. |
| M2 | P0 graph and interchange | Expand molecular facts and shared codecs | Exchange ordinary chemistry safely. |
| M3 | P1 presentation grammar | Add semantic graphical records and tools | Author reaction/figure content. |
| M4 | P1 chemistry operations | Add reports, diagnostics, query, and closed naming | Explain and search structures. |
| M5 | P1 catalogs and reactions | Add templates, groups, reactions, peptide/carbohydrate | Restore productive vocabulary. |
| M6 | P2 usable application | Finish access, help, clipboard, logging, output | Make all delivered workflows usable. |
| M7 | Separate ecosystems | Design plugins and services before implementation | Avoid unsafe compatibility ports. |

### M0: Parity contract and corpus

**Depends on:** none.  **Parallel-plan ready:** yes; descriptor and corpus lanes
are independent.

Deliver: a matrix mapping each of the 23 missing Qt workflows and all reopened
OASA gaps to one Rust contract, Qt route, CLI route, corpus input, and
`supported`/`refused`/`deferred` disposition; `FormatCapabilityV1`; per-format
loss/refusal and resource-limit policy.

Done when every claim has an owner and acceptance workflow, no route imports
reference code, and every format has explicit loss and safety behavior.

### M1: P0 direct structure editor

**Depends on:** M0.  **Parallel-plan ready:** yes once the command DTO is frozen.

P0.1 deliver: a Rust-first, revision/digest-fenced begin/preview/commit gesture
from an existing direct atom to a same-molecule existing direct atom or a new
carbon endpoint. It supports only normal single, double, and triple bonds; Qt
owns a disposable overlay and tool preferences, while Rust owns endpoint
resolution, snap policy, candidate validation, IDs, history, and the returned
observation. Begin and preview do not mutate the document.

P0.1 done when a user draws each normal order to an existing or new carbon
endpoint, cancellation/refusal leaves the document unchanged, one accepted
release creates exactly one history transition, and undo/redo/save/reopen
preserve the semantic graph. Stale revision/digest, malformed, ineligible,
self-loop, duplicate, cross-molecule, unrenderable, and cancelled requests
cannot mutate the document.

P0.2 deliver, only after P0.1 and reliable Rust-issued render hit/containment
and bounds facts: `SelectionSetV1` and selected-root click/marquee/translation
contracts. Qt may supply geometric candidates and paint an overlay, but Rust
resolves canonical eligible identities and fences translation. Nudge/delete,
mixed selection, free-space starts, wedges, and other historical bond styles
are separate follow-on contracts, not implicit P0.1 scope.

### M2: P0 graph and interchange foundation

**Depends on:** M0; M1 for editor property clients.  **Parallel-plan ready:**
yes; graph and codec lanes meet at frozen DTOs.

Deliver: groups/query atoms, aromatic/stereo/isotope/radical/charge/valence
facts; CML/CML2, CDXML, declared CD-SVG, compressed input, multi-record SDF,
multi-root export, rich clipboard, and one format registry shared by CLI and
Qt; `ferrum formats`, conversion, and graph inspection routes.

Done when corpus import -> typed graph/document -> export -> reimport retains
declared facts or returns an explicit loss report.  XML, compression, clipboard,
and unsupported inputs refuse before mutation or publication.

### M3: P1 presentation grammar

P1.1 has an accepted Rust-first straight normal-arrow gesture backend and Qt
controller. It is intentionally limited to two-point normal arrows; spline,
multi-point, non-normal, reaction-association, and other presentation tools
remain separate parity work.

**Depends on:** M1 and M2.  **Parallel-plan ready:** yes after record DTOs freeze.

Deliver semantic records, transactions, render plans, tools, and dialogs for
brackets; straight/curved/reaction/equilibrium/electron/retro arrows; plus;
rich text; rectangle/square/oval/circle/polyline/polygon; bond alignment; mixed
selection affine transforms; and expanded object properties.

Done when a reaction scheme with molecules, arrow, plus, text, brackets, and
vectors survives edit, stacking, save/reopen, rich copy/paste, and SVG/PDF/PNG.

### M4: P1 chemistry operation catalog

**Depends on:** M2.  **Parallel-plan ready:** yes; report, query, and closed
nomenclature lanes share DTO conventions only.

Deliver `MoleculeReportV1` (formula, exact/average mass, composition, charge,
identifiers, aromatic/stereo/valence status); diagnostic findings/recovery;
oxidation; SMARTS; known-group expansion; and a closed structure-name grammar,
each with CLI and Qt information/check/find surfaces.

Done when users can describe, validate, search, and where admitted name/expand
structures; ambiguous/unavailable/resource-bounded calls return typed outcomes,
never guesses.

### M5: P1 catalogs and reactions

**Depends on:** M1, M2, M4.  **Parallel-plan ready:** yes after immutable catalog
manifest and attachment DTOs are frozen.

Deliver a versioned provenance-bearing template manifest; system and biomolecule
palettes; user-template toolbar; reaction roots/import/export/templates;
declarative carbohydrate schemas; expanded peptide/residue/termini profiles;
and named-group reference-data contracts.

Done when curated templates, peptides, carbohydrates, and atom-mapped reactions
can preview, attach, undo/redo, save/reopen, and exchange deterministically.

### M6: P2 usable application

**Depends on:** M1-M5.  **Parallel-plan ready:** yes once action contracts exist.

Deliver selection-sensitive context menus with keyboard equivalents; complete
keyboard authoring; accessible names/roles/descriptions/focus/high-contrast
behavior; help generated from action metadata; logging controls; rich clipboard
interoperability; 3D/transform/stereo policy; and PostScript implementation or
explicit evidence-based deprecation.

Done when a keyboard-only user creates, edits, inspects, exports, and reopens a
reaction scheme, with useful accessible state and a clear result/refusal for
every enabled action.

### M7: Separately designed ecosystems

**Depends on:** M0 and M4.  **Parallel-plan ready:** no; security/product policy
is serial.

Before any implementation, approve a plugin decision covering signing,
permissions, process/WASM isolation, RPC, compatibility, failure containment,
and document authority.  Separately approve a compound-service decision covering
provider terms, privacy, rate limits, cache/provenance, cancellation, offline
recovery, and import confirmation.  A sandboxed plugin and fake provider prove
the approved contracts; neither is a menu-port task.

## Workstream breakdown

| Workstream | Owner | Work packages | Interface |
| --- | --- | --- | --- |
| WS-A contracts | Rust engineer | command/selection, records, format registry | versioned DTOs/errors |
| WS-B chemistry/domain | chemistry engineer | reports/query, catalogs/reactions, peptide/carbohydrate/naming | owned domain values |
| WS-C rendering | render engineer | presentation records, plans, artifacts | checked render plans |
| WS-D interaction | Qt engineer | gesture client, tools/dialogs, help/accessibility | immutable observations/results |
| WS-E evidence | test engineer | corpus, semantics, CLI/Qt E2E | stable acceptance evidence |

## Work packages

| ID | Owner | Depends on | Outcome |
| --- | --- | --- | --- |
| WP-A1 | Rust engineer | M0 | Versioned command/selection/preview/refusal boundary. |
| WP-D1 | Qt engineer | WP-A1 | Direct tools with transient previews only. |
| WP-E1 | test engineer | M0 | Corpus loss/refusal classification. |
| WP-A2 | chemistry engineer | WP-E1 | Expanded graph and presentation schemas. |
| WP-A3 | Rust engineer | WP-E1 | Bounded shared format registry. |
| WP-C1 | render engineer | WP-A2 | Semantic rendering for admitted records. |
| WP-B1 | chemistry engineer | WP-A2 | Reports, diagnostics, query, oxidation, naming. |
| WP-B2 | domain engineer | WP-A2 | Catalog, group, reaction, peptide, carbohydrate values. |
| WP-D2 | Qt engineer | WP-A1, WP-C1, WP-B1, WP-B2 | Dialogs, palettes, action enablement. |
| WP-D3 | accessibility engineer | WP-D2 | Keyboard, help, focus, high contrast. |
| WP-E2 | test engineer | implemented contracts | Semantic regressions and corpus reports. |
| WP-E3 | Qt/CLI test engineer | WP-D2, WP-D3 | Real ordinary user workflows. |

## Acceptance criteria and gates

P0 is complete only when M1 and P0-relevant M2 routes prove direct drawing,
selection, cancellation, history, and save/reopen in the ordinary Qt window.
P1 families need a Rust operation, CLI route, Qt workflow, and durable round
trip.  P2 interactions need keyboard reachability, accessible state, and clear
refusal language.

Complete parity evidence is: no unclassified reference workflow; an owned Rust
contract for every admitted behavior; CLI and Qt paths for non-library work;
semantic corpus results against RDKit where representable; undo/redo/save/reopen
for mutations; render/export proof for presentation; and manual accessibility
evidence.  OASA is at most an offline migration oracle, never a dependency.

## Test and verification strategy

Use fast deterministic tests for parsers, DTO admission, graph/render invariants,
and typed errors.  Run actual CLI and Qt workflows in `tests/e2e/`, not the fast
pytest lane.  Compare molecular semantics rather than writer bytes or
version-sensitive display strings.  Each patch requires independent review by a
non-author and evidence that covers its declared capability and refusal path.

## Migration and compatibility policy

Product source, packaging, runtime messages, and imports use Ferrum branding,
not OASA or BKChem.  Historical names, CDML namespace facts, fixture provenance,
and read-only citations remain permitted compatibility/provenance records.  A
changed legacy behavior is either a named Ferrum contract or an explicit refusal.

## Risk register

| Risk | Trigger | Mitigation | Owner |
| --- | --- | --- | --- |
| Menu-only parity | UI ships without durable evidence | Capability-to-contract-to-E2E ledger | integrator |
| Qt shadow model | Tool edits projection/invents chemistry | Rust command-boundary review | Qt lead |
| RDKit drift | Upgrade changes chemistry output | Version-record and normalize semantics | chemistry lead |
| Silent interchange loss | Foreign fact is discarded | Loss report or pre-mutation refusal | codec lead |
| Catalog provenance gap | Asset lacks source/license | Block manifest admission | domain lead |
| Unsafe integrations | Plugin/network menu port proposed | M7 decision and offline contracts | maintainer |

## Documentation close-out requirements

Every completed milestone updates the capability matrix, CLI/Qt usage,
architecture notes, changelog, and concise evidence report.  Each format,
profile, catalog, and refusal code documents scope, source, limits, and recovery.

## Patch plan and reporting format

The first executable P0 patch is **P0.1: Rust-first direct normal-bond gesture**.
Add a small `direct_bond_gesture_v1` contract with explicit revision/digest
fence, immutable begin/preview handles, typed refusal categories, captured snap
policy, and a commit that delegates to existing prepared bond or bonded-carbon
insertion primitives. The accepted start is an existing direct atom; the end
is an existing direct atom in that molecule or a new carbon endpoint. Expose
only normal single, double, and triple presentations. Begin/preview allocate
no IDs and change no session state; a valid release creates one history entry.
Add Rust and PyO3 tests for both endpoint forms, all three orders, fences,
refusals, cancellation-as-drop, undo/redo, and reopen. The next patch adds the
small Qt pointer controller and disposable overlay with a real end-to-end
workflow; interactive handles remain desktop-only, not CLI/protocol values.

**P0.2 follows P0.1, not in parallel with it.** Add selected-root
selection/marquee/translation only after render publishes reliable eligible-root
hit, containment, and bounds facts. `SelectionSetV1` is revision/digest-fenced
and canonicalized by Rust; Qt has neither a shadow selection set nor transform
authority. Its Qt E2E is click/marquee select, drag, undo, save, reopen.

Implementation reports state: `Milestone`, `Work package`, `Contracts`, `User
workflow`, `Changed files`, `Evidence run`, `Known limits`, and `Next dependency`.

## Open questions and decisions needed

- Set exact CDXML/CML profiles from corpus evidence before promising universal
  interchange.
- Decide PostScript support versus explicit deprecation after user-value evidence.
- Decide reaction SMARTS/SMIRKS adapter scope before reaction import/export.
- Decide plugin isolation technology only after permission/lifecycle approval.
