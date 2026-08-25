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
coordinates, SVG/PDF/PNG, ordinary Rust session editing, and protocol-backed
CLI operations.  The frontend inventory has 22 complete workflows, 8 deliberately
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
| M4 | P1 chemistry operations | Add reports, diagnostics, and query | Explain and search structures. |
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

Current M0 implementation establishes one document-owned authority for admitted
construction: `PreparedSessionTransitionV1` receives a semantic request,
prepares it with renderer admission, and performs the one-use atomic commit.
This now includes visual presentation routes, explicit-hydrogen materialization,
`CreateAtomV1`, `CreateBondV1`, `CreateHaworthMoleculeV1`, and attached
cyclohexane. The renderer supplies `DocumentPrecommitOverlayV1` only as an
identifier-free paint value for the relevant UI previews. Route-specific
prepared receipts and migration-history fixture/catalog checks are retired;
wavy and bracket bindings keep their supported semantics. This records an
implemented M0 authority boundary. The authoritative
[m0_complete_render_admission_v1.md](../decisions/m0_complete_render_admission_v1.md)
records M0 closure on 2026-08-24 with fresh aggregate exit evidence.

### M1: P0 direct structure editor

**Depends on:** M0.  **Parallel-plan ready:** yes once the command DTO is frozen.

P0.1 deliver: one Rust-first, revision/digest-fenced direct-bond gesture with
two closed endpoint intents: an existing direct atom in the relevant molecule or
a new carbon at a typed point. Rust admits exactly four forms:
`ExistingExisting`, `ExistingNew`, `NewExisting`, and `NewNew`. `NewNew` is the
intentional blank-canvas authoring route, not a compatibility exception. Its
normal order is the existing visible `Next Drawing` choice: single, double, or
triple. Activating the one existing `Draw Bond` action captures that value, and
admission freezes it for the whole pointer or keyboard gesture; a later
preference change affects only the next gesture. Qt passes only closed
presentation and endpoint UI facts through the existing opaque Rust receipt
flow. Rust owns endpoint resolution, snap policy, chemistry admission,
topology, capacity/valence, IDs, history, and the returned observation. Qt owns
only a disposable copied overlay, action state, and accessible feedback. Begin,
preview, and abandoned admission do not mutate the document or consume identity.

P0.1 covers all normal orders for all four endpoint forms. It does not add an
action, dialog, shortcut, Qt-local chemistry, or a second drawing framework.
Wedges, aromatic bonds, free-form element selection for new endpoints, and
selected-root work remain explicitly deferred; selected-root is P0.2, not an
implicit extension of this package.

P0.1 done when bounded Rust and PyO3 matrix evidence proves each normal order
for `ExistingExisting`, `ExistingNew`, `NewExisting`, and `NewNew`, including
one atomic history transition, undo/redo, and the canonical persistence seam.
Focused Qt behavior proves that the visible Next Drawing value freezes at
admission for pointer and keyboard authoring, while cancellation and typed
refusal remain mutation-free. One isolated, offscreen local-runtime E2E uses
temporary inline input and proves blank-canvas `NewNew` normal-order commit plus
Escape cancellation through the runtime staged by `./build.sh`.
The package uses no committed fixture expansion, network connection, manual
approval, or human GUI gate. Stale revision/digest, malformed, ineligible,
self-loop, duplicate, cross-molecule, unrenderable, and cancelled requests
cannot mutate the document.

P0.2 deliver, only after P0.1 and reliable Rust-issued render hit/containment
and bounds facts: `RenderInteractionSelectionV1` and selected-root
click/marquee/translation contracts. Rust is the canonical authority for
eligible direct roots, including mixed molecule and plus selections in Rust
source order regardless of click or marquee traversal order. It admits their
translation as one revision/digest-fenced atomic operation. Qt passes pointer
and marquee gestures, paints a transient overlay, and installs only the
returned observation. Evidence covers the Rust contract, its PyO3 binding, and
focused Qt behavior, including source-order-independent selection and one
atomic history transition with undo/redo. Nudge/delete, free-space starts,
wedges, and other historical bond styles are separate follow-on contracts, not
implicit P0.1 scope.

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
controller. M3.P2 delivers the separate bounded `CurvedElectronArrowV1` slice:
one `<arrow type="electron">` with exact `start`, `control`, and `end` points.
Rust owns quadratic geometry, cubic lowering, terminal-head derivation, style,
renderer preflight, opaque receipt fencing, history, and atomic commit. Qt
captures three clicks, displays only the returned Rust preview, automatically
commits click 3, and cancels with Escape. Native binding and staged Qt QAction
evidence cover the closed lifecycle.

The completed M3 presentation families share the universal document-owned
authoring-capability contract.
Every supported opaque authoring receipt carries the nonserializable
`AuthoringCapabilityV1` issued by its `DocumentSession`'s
`AuthoringCapabilityIssuerV1`, beside route-specific revision/digest and
session fences. This same authority covers text placement, straight
presentation, catalog V1/V2, presentation vectors, DirectBond, terminal and
equilibrium arrows, paths, and reaction create/lifecycle/translation. A foreign
preview or prepare fails publicly as `ForeignSession` with `RefreshAndRestart`
before candidate work; a foreign commit preserves the owner receipt. A claim
moves only from `Available` to `Claimed`, becomes `Consumed` only after the
owner transaction succeeds, and restores availability when an owner-side
failure rolls it back. Capability identity, origin fencing, catalog preview
leases, and durable CDML identifiers remain separate concerns. Catalog recipes
lower to `MoleculeInsertionV1`; the receiving `DocumentSession` alone creates
the opaque pending candidate and its molecule, atom, and bond identities.
Tentative generated-ID sequences install only after its authoritative commit,
so abandoned, discarded, and refused catalog candidates leave durable-ID
allocation unchanged. `CatalogPreviewLeaseV2` remains renderer-local transient
preview-retirement state rather than a document identity or authoring receipt.
The generic `PreparedSessionTransitionV1` lifecycle issues presentation IDs
inside document-owned preparation for terminal Electron, Retro, Curved Normal,
Curved Equilibrium, straight normal/equilibrium arrows, incremental
Polyline/Polygon paths, presentation vectors, and standard plus. IDs install
only with the successful document mutation; previews and abandoned or refused
candidates leave the sequence unchanged. Renderer routes consequently own no
durable presentation counter.

The CLI/protocol route `presentation.author.v1` now replaces the earlier
vector-only operation. It accepts one request-owned document and one typed
authoring request for Vector, terminal Electron/Retro/Normal arrow, Curved
Equilibrium arrow, Polyline/Polygon path, or explicit-endpoint DirectBond.
The adapter creates the short-lived live-session authorization internally,
prepares and commits one generic transition, and returns only accepted document
and durable root facts. DirectBond uses durable atom IDs or finite new-atom
points, not Qt pointer state. `PresentationAppearanceV1` validates RGB and
bounded width values at the document boundary, preventing style text from
changing CDML/XML roots or IDs. A stale, refused, or abandoned prepared
transition leaves the allocator unchanged, so its tentative ID may be reissued.

M3.P3 is the accepted bounded `CurvedRetroArrowV1` sibling slice. It authors one
`<arrow type="retro">` with the same exact `start`, `control`, and `end`
points. Rust uses the closed shared `CurvedTerminalArrowKindV1 { Electron,
Retro }` model so persistent projection, renderer lowering, and transient
preview receive one cubic axis and terminal head. The typed opaque
begin/preview/prepare/commit lifecycle, revision/digest fence, one history
entry, and atomic refusal behavior match M3.P2. Qt provides one named action,
collects three clicks, renders only the returned preview, auto-commits click 3,
and uses Escape to cancel. Focused Rust/document, PyO3, and offscreen
staged-QAction evidence covers commit, cancellation, refusal, history, and
save/reopen; the full local suite accepts this exact capability.

M3.P3 excludes generic spline paths, variable point counts, start heads,
property editing, and reaction association. CurvedNormal and
CurvedEquilibrium retain their distinct dedicated contracts; CurvedRetro does
not overload or subsume either arrow family.

M3.P4 is the accepted bounded `CurvedNormalReactionArrowV1` sibling slice. It
authors one direct-root `<arrow type="curved-normal">` with exactly `start`,
`control`, and `end` points. It does not overload `<arrow type="normal">`.
Rust owns the one quadratic-to-cubic axis, terminal-head geometry, renderer
preflight, typed opaque begin/preview/prepare/commit lifecycle,
revision/digest fence, one history entry, and atomic refusal behavior. Qt
offers one named three-click action, paints only the native preview, commits
only the native receipt on click three, and clears its transient capture with
Escape. Focused Rust/document and PyO3 semantic evidence, an offscreen
staged-QAction workflow for commit, cancellation, refusal, history, and
save/reopen, and the full local suite accept this exact capability.

M3.P4 excludes spline compatibility, variable point counts, configurable or
start heads, property editing, reaction association, and a generic arrow
factory. CurvedEquilibrium retains its distinct dedicated two-lane geometry
contract; CurvedNormal does not overload or subsume it. CurvedRetro likewise
remains a separate terminal-arrow contract.

M3.P5 is the accepted bounded `CurvedEquilibriumArrowV1` slice. It authors one
direct-root `<arrow type="curved-equilibrium">` with exact `start`, `control`,
and `end` points. Rust owns two translated quadratic lanes, their one-time
cubic lowering, two opposing heads, bounds, renderer preflight, the typed
opaque begin/preview/prepare/commit lifecycle, revision/digest fence, one
history entry, and atomic refusal behavior. The PyO3 overlay carries only
Rust-issued `lower_axis`, `upper_axis`, `lower_head`, and `upper_head` facts.
Qt integration captures the three coordinates, paints that frozen DTO, commits
only the prepared receipt, and cancels transient capture with Escape.

M3.P5 rejects `equilibrium2`, generic spline or variable-point semantics,
`spline`, `start`, `end`, `shape`, `properties`, `association`, and `factory`
facts. Normal-arrow head facts, configurable heads, property editing, reaction
association, and generic arrow factories remain separate future contracts.
Focused Rust/document and PyO3 evidence, the staged offscreen Qt workflow for
commit, cancellation, refusal, history, and save/reopen, local CLI and Qt
E2Es, and the current full local suite provide the validation receipt.

M3.P6 is the supported bounded directed stereobond slice. `Draw Solid Wedge
Bond` and `Draw Hashed Wedge Bond` use the public Rust V3
`begin_direct_bond_gesture_v3 -> admit_direct_bond_candidate_v3 ->
commit_direct_bond_admission_v3` lifecycle. Qt submits finite scene points,
viewport-to-scene mappings, and exact `none`/unique/ambiguous hit evidence;
Rust resolves every endpoint into `ExistingExisting`, `ExistingNew`,
`NewExisting`, or `NewNew`. The V2 gesture lifecycle is retired and its
resolved values are Rust-internal. V1 document, fence, presentation, snap, and
commit values remain the current V3 commit taxonomy. Separately, `ferrum-document` exposes a
native-Rust-only, renderer-neutral direct-bond mutation seam for noninteractive
programmatic work. Its public endpoint input is already resolved to a durable
atom ID or finite new-atom point; it has no Qt/PyO3 route and accepts no pointer
probe, viewport transform, hit evidence, snap decision, overlay, render plan,
or issued operation. Rust owns tolerance, ties, hit-ID validation,
snap/new selection, fences, renderer-neutral candidate construction, direction,
IDs, history, complete renderer preflight, immutable target-bond operations,
durable projection, and rendering. Qt paints only admitted Rust operations.
The authoring actions use the bounded Normal, Solid wedge, and Hashed wedge
vocabulary; solid and hashed actions admit only covalent single `w1` and `h1`
bonds with pointer start as CDML tip and pointer end as base. A V3 probe error
and a post-resolution admission refusal have distinct typed nonmodal recovery;
a valid same-atom attempt is `self_loop` / `adjust_endpoint`.
`UnrenderableCandidate` is `ChangePresentation`. Escape and every typed refusal
remain mutation-free. Existing Bond Properties retains its independently
supported broader bond-style vocabulary; M3.P6 does not narrow that unrelated
editor.

M3.P6 excludes generic stereo/CIP semantics or inference, E/Z semantics,
arbitrary bond styles or orders, and stereo import/export expansion. A fresh
local build and `./all_test.sh` provide the current validation receipt.

M3.P7 is the supported bounded Polyline/Polygon incremental-authoring slice.
Its native contract is one opaque Rust-owned point-at-a-time transaction:
`begin -> add accepted point -> progress -> optional-hover-preview -> prepare
-> commit`, or `cancel`. Accepted points are durable candidate geometry; hover
never persists. Rust derives overlay geometry and appearance, validates path
cardinality and geometry, preflights the renderer, and enforces
origin/revision/digest/preview/one-use fences. Cancellation is a typed
`DocumentUnchanged` outcome.

M3.P7 uses the same universal document-owned authoring capability as every
other supported opaque authoring lifecycle. Its gesture, preview, and prepared
aliases hold one `AuthoringCapabilityV1` allocation issued by the owning
`DocumentSession`; `Available -> Claimed -> Consumed` supplies one-use
redemption without a serializable nonce, process-wide allocator, or
consumed-capability registry. Dropping an unsettled owner claim restores the
receipt, foreign callers are fenced before candidate work, catalog preview
leases remain preview-only supersession state, and durable CDML identifiers
remain independent.

PyO3 exposes this canonical opaque lifecycle and the full-vector preview bridge
is retired. Qt exposes `Draw Polyline` and `Draw Polygon`, converts points to
the scene, captures events, and presents user wording. Its only retained local
coordinate is the transient accepted press used to de-duplicate real and QTest
double-click delivery; Qt does not use it for geometry, validation, progress,
or persistent state. Focused public binding and Qt behavior evidence, followed
by a fresh local build and `./all_test.sh`, provide the current receipt.

M3.P7 does not claim generic splines, variable-point-count grammar beyond the
two named tools, path property editing, association semantics, or a generic
presentation factory. Those are separate future contracts.

**Depends on:** M1 and M2.  **Parallel-plan ready:** yes after record DTOs freeze.

Deliver semantic records, transactions, render plans, tools, and dialogs for
brackets; straight/curved/reaction/equilibrium/electron/retro arrows; plus;
rich text; rectangle/square/oval/circle/polyline/polygon; bond alignment; mixed
selection affine transforms; and expanded object properties.

Done when a reaction scheme with molecules, arrow, plus, text, brackets, and
vectors survives edit, stacking, save/reopen, rich copy/paste, and SVG/PDF/PNG.
Permanent vector-authoring coverage uses the registered root E2E lane and its
lease-backed workspace, exercising only public Qt and Rust workflows rather
than package-local scripts that write artifacts in the current directory.
Text authoring uses focused rich-text and text-placement contract evidence.
Its former root E2E was intentionally retired under the pytest permanent-test
policy because nested synchronous modal orchestration is offscreen-platform
fragile; it is neither skipped evidence nor a remaining required E2E lane.

### M4: P1 chemistry operation catalog

The compact-materialization portion of M4 is governed by the selected
[m4_compact_group_materialization_v1.md](../decisions/m4_compact_group_materialization_v1.md).
Its generic protocol, named CLI route, live-session PyO3 registration, and
visible Qt action are complete. The action consumes only Rust-issued durable
target IDs and a current document fence; it does not recover targets from raw
CDML or source IDs.

**Depends on:** M2.  **Parallel-plan ready:** yes; report and query lanes share
DTO conventions only.

Deliver `MoleculeReportV1` (formula, exact/average mass, composition, charge,
identifiers, aromatic/stereo/valence status); diagnostic findings/recovery;
oxidation; SMARTS; and known-group expansion, each with CLI and Qt
information/check/find surfaces.

The evidence-selected first slice is `document.molecule.report.v1`: a read-only,
snapshot-based report for one or more selected direct-root molecules. Its request
is `snapshot { cdml, revision, digest_hex }` plus `molecule_ids`; source order
governs returned root records, while findings use deterministic report-category
order. The completed receipt preserves the source revision and verified digest,
and its aggregate is complete or omitted. Unaddressable source locations remain
typed report outcomes; the slice adds no mutation, local CLI verb, chemistry
engine, external corpus, or installation/publishing workflow.

Ferrum preserves authored molecule display names. A future generated naming
product requires separate corpus, provenance, interaction, and refusal scope;
it is not part of this M4 delivery plan.

SMARTS control refresh recomputes action eligibility without erasing an explicit
terminal user outcome. In particular, clearing successful results remains
visibly confirmed as completed after the control state refreshes. A public Qt
widget regression covers that visible terminal-status behavior.

Done when users can describe, validate, search, and where admitted expand
structures; ambiguous, unavailable, and resource-bounded calls return typed
outcomes, never guesses.

### M5: P1 catalogs and reactions

**Depends on:** M1, M2, M4.  **Parallel-plan ready:** yes after immutable catalog
manifest and attachment DTOs are frozen.

The shipped `catalog.list.v1` / `catalog.insert.v1` protocol contract and its
lease-backed public E2E are completed prerequisites for this milestone. They
establish bounded catalog discovery, fenced insertion, and typed stale refusal;
they do not complete M5's palette, reaction, provenance, corpus, or user-workflow
parity work.

Deliver a versioned provenance-bearing template manifest; system and biomolecule
palettes; user-template toolbar; reaction roots/import/export/templates;
declarative carbohydrate schemas; expanded peptide/residue/termini profiles;
and named-group reference-data contracts.

Done when curated templates, peptides, carbohydrates, and atom-mapped reactions
can preview, attach, undo/redo, save/reopen, and exchange deterministically.
Permanent template-catalog authoring coverage likewise runs through the
lease-backed root E2E lane and public workflows, replacing package-local
current-directory artifact scripts.

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

## Current next-work queue

The historical P0.1 and P0.2 patch text is closed: normal-order direct-bond
authoring and selected-root selection/translation have their Rust-owned
contracts and acceptance evidence. The completed M3 presentation slices are
also not a current patch queue. Their shared public CLI route is now
`presentation.author.v1`, which accepts a request-owned fenced document and
typed serializable intent rather than a live receipt.

The selected read-only `document.atom.oxidation.observe.v1` HCNO V1 operation
has completed its bounded evidence gate: generic-executor semantic corpus,
named CLI protocol proof, nonzero source-provenance PyO3 regression, and real
compiled-extension Qt evidence. Its bounded contract is recorded in
[m4_atom_oxidation_v1.md](../decisions/m4_atom_oxidation_v1.md), and the
human-facing receipt is
[m4_atom_oxidation_corpus_v1.md](../reports/m4_atom_oxidation_corpus_v1.md).
The Qt lane proves accepted and unavailable presentation, no mutation,
source-fenced historical status, source-tab-only rerun eligibility, and
source-tab retirement. The shared detached admission retains caller revision
and verified digest as immutable source provenance while its request-local
session begins at revision zero; operations must never compare those identities.
Typed refusal and recovery remain in the protocol/CLI lane. This preserves the
bounded HCNO chemistry, generic PyO3 transport, unchanged SMARTS, and no
known-group expansion. M4 itself remains incomplete.

Continue the M4 chemistry-operation catalog with a separately selected bounded
contract. Each candidate remains supported only after it has a typed refusal
and recovery contract, a durable workflow proof, and an explicit statement of
its limits.

The M0 statement about document-private compact cleanup is superseded for
materialization by the selected
[m4_compact_group_materialization_v1.md](../decisions/m4_compact_group_materialization_v1.md):
the generic protocol, named CLI route, live-session PyO3 registration, and Qt
action are delivered. This does not advance M5 catalog or reaction work.

The subsequent confirmed queue remains M2 interchange/graph completion, M5
catalogs and reactions, and M6 usable-application work. The completed catalog
protocol prerequisite does not advance M5 beyond its remaining parity work.
These milestones stay open until their declared parity claims have contract,
client, corpus, and workflow evidence; this document does not claim complete
OASA/BKChem parity.

Implementation reports state: `Milestone`, `Work package`, `Contracts`, `User
workflow`, `Changed files`, `Evidence run`, `Known limits`, and `Next dependency`.

## Open questions and decisions needed

- Set exact CDXML/CML profiles from corpus evidence before promising universal
  interchange.
- Decide PostScript support versus explicit deprecation after user-value evidence.
- Decide reaction SMARTS/SMIRKS adapter scope before reaction import/export.
- Decide plugin isolation technology only after permission/lifecycle approval.
