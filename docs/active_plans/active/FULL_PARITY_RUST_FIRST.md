# Plan: Rust-first Ferrum feature parity

## Context

Ferrum is a working Rust-native CDML editor, not yet a full replacement for the
read-only BKChem/OASA reference.  The 2026-08-19 inventories identify 23 absent
Qt workflows and reopened backend gaps in interchange, graph coverage, editor
grammar, chemistry operations, reactions, catalogs, and optional integrations.

This is the canonical full-parity scope, dependency, and acceptance ledger for
complete usable Ferrum and Rust-first OASA/BKChem parity. Its milestones use the `PARITY-M*`
namespace. [ferrum-plan-v3.md](../ferrum-plan-v3.md) is a subordinate historical
implementation record; its `V3-M*` milestones retain only their completed local
scope and must not be used to sequence or close full parity. This ledger neither
claims parity already exists nor restores Python OASA, a Python document model,
or reference code as a runtime dependency. `OTHER_REPOS/` remains read-only.

[ROADMAP.md](../../ROADMAP.md) records the current delivery checkpoint. Its
`ActionRegistry` lifecycle reconciliation is complete. Reconcile broader
historical milestone and receipt status into this ledger before approving or
closing the next parity milestone; dated receipts below prove only the slices
and runs they name.

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

The reported automated Rust, binding, and installed-Qt receipts establish the
delivered nine-recipe attached Qt/Rust slice. `PARITY-M4.A` additionally
delivers the one generic attached CLI/protocol route. These results do not
establish full Rust/OASA/BKChem parity or the remaining manual 16:10 screenshot
and keyboard/accessibility walkthrough.

The delivered CML directed-depiction extension admits exactly one direct CML1
`builtin="stereo"` W/H or CML2 `<stereo>` W/H child on a single bond. W/H are
authored solid/hashed depictions, not inferred tetrahedral, parity, or E/Z
semantics. `ferrum-chemistry` owns the closed grammar and source order;
`ferrum-document` owns generic direction-only depiction admission without atom
chirality. Unsupported, duplicate, nested, non-single, and other stereo forms
remain typed refusals. The focused chemistry, document, and API-library
receipts are recorded in the active CML contract; they do not close M2, full
parity, or the manual 16:10 keyboard/accessibility walkthrough.

The bounded CDXML simple-molecule import profile is delivered through the
Rust decoder, descriptor registry, generic CLI/PyO3 ingress, and existing Qt
interchange worker. It is an input-only `.cdxml` capability with typed loss,
typed provenance, and no CDXML save baseline or encoder. The exact grammar,
security boundary, resource limits, evidence, and exclusions are frozen in
[m2_cdxml_simple_molecule_import_v1.md](../decisions/m2_cdxml_simple_molecule_import_v1.md).
Its chemistry/API/binding/workspace/build/CLI/Qt receipts support this bounded
claim only. The following delivery-time receipt is retained as historical
evidence and is superseded by the current 2026-08-27 checkpoint below. At that
time, post-audit `./build.sh` exited 0; the registered
`tests/e2e/run_all.sh` exited 0, including CDXML; staged Python bindings passed
281 tests; Qt passed 238 tests with one intentional skip; focused chemistry and
API libraries passed 124 and 117 tests; and `cargo check --workspace` passed.
`./all_test.sh` was not aggregate-green: it recorded 7,759 passes before five
Markdown-link failures stopped later aggregate phases. All five canonical links
targeted the present CDXML decision artifact, which was absent only from the
tracked-file catalog. The skipped later phases were run directly and passed as
listed above. Real 16:10 accessibility evidence, M2, and full parity remain
open.

The current checkpoint also contains four bounded Rust-first slices. M2.B
unifies File/Open discovery and preparation in `LocalDocumentOpenCatalogV2`;
M4.C makes the closed molecule-report identifier outcome required; the periodic
picker projects Rust-issued display facts into the shared next-drawing
preference; and M6 adds active-Select-Structure cursor keyboard interaction.
Focused M2 evidence covers 32 PyO3 tests, 18 Qt File/Open tests, and the public
SDF-import E2E. Focused M4 evidence covers 26 Rust tests, 9 PyO3 tests, 10 Qt
tests, and the report CLI E2E. The periodic-picker suites pass 48 focused tests
plus 5 real-document invariance tests. The delivered M6 tab-owned opaque Rust
selection-fence bridge passed 57 combined focused Qt/PyO3 tests; its registered
no-pointer keyboard E2E exited 0 and independent final review accepted it with
no P1 finding. None of these bounded results closes M2, M4, M6, or full parity.

## Delivered bounded parity slice

### PARITY-M4.A: Generic attached compact-group CLI/protocol [delivered]

**Depends on:** the delivered Rust catalog and `AttachCompactGroupV1` Qt/Rust
slice. **Implementation status:** delivered; validation is recorded below.

One stateless operation, `document.compact-group.attach.v1`, now runs through
both `ferrum protocol run` and `ferrum document command
document.compact-group.attach.v1`. It accepts a fenced CDML snapshot, a durable
molecule/anchor pair, one closed catalog key, and finite release coordinates.
Rust owns catalog and pair-local target admission, chemistry, geometry, renderer
admission, durable IDs, history, and typed no-mutation refusals. API and CLI
adapters load the request and present its envelope only.

The request-scoped session prepares and immediately commits an admitted
attachment. Its versioned receipt returns source facts, target/catalog echoes,
the allocated compact-group ID, committed CDML, and a reusable stateless fence.
It exposes no release, pose, overlay, or pending/session capability. Focused
Rust contract coverage is passed, including the nine-recipe semantic matrix;
the registered public E2E covers one representative success through each
transport and typed stale-fence refusals. The fresh local build and
`./all_test.sh` receipt are delivered. This does not advance M4 or full parity
to complete; the screenshot/manual accessibility gate remains separate.

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

`PARITY-M2.B` is a delivered bounded authority repair, not M2 completion.
`LocalDocumentOpenCatalogV2` emits native CDML and decoded SVG followed by the
registry's `DocumentImportNew` formats, each with an opaque Rust-issued handle.
The generic preparation API alone dispatches the route; Qt neither classifies
source kind nor selects a parser from a suffix. File/Open creates a new document
or uses the explicit replacement fact, while **File > Import SDF Records into
Current Drawing...** retains its separate source-read and insertion workflow.
Focused PyO3, Qt File/Open, and visible Import-SDF E2E evidence are recorded in
the current checkpoint. M2 remains open for the declared corpus-backed graph and
interchange scope.

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
Bond` and `Draw Hashed Wedge Bond` use the sole current unversioned in-process
Rust/PyO3 pointer capability: begin a direct-bond gesture, resolve its endpoint,
prepare the generic `PreparedSessionTransitionV1`, then generically commit it.
Qt owns finite viewport-to-scene conversion, pointer events, and exact
`none`/unique/ambiguous hit evidence. `ferrum-document-render` resolves the UI
probe and one-use authoring capability. `ferrum-document` owns the durable
`CreateDirectBondV1` request and generic transition, including every
`ExistingExisting`, `ExistingNew`, `NewExisting`, or `NewNew` endpoint form,
tolerance, ties, hit-ID validation, snap/new selection, fences, candidate
construction, direction, IDs, history, complete renderer preflight, immutable
target-bond operations, durable projection, and rendering. V1 applies only to
durable document, fence, presentation, snap, and transition values where it is
the actual contract version. Separately, `ferrum-document` exposes a native-
Rust-only, renderer-neutral direct-bond mutation seam for noninteractive
programmatic work. Its public endpoint input is already resolved to a durable
atom ID or finite new-atom point; it has no Qt/PyO3 route and accepts no pointer
probe, viewport transform, hit evidence, snap decision, overlay, render plan,
or issued operation. Qt paints only Rust-issued operations.
The authoring actions use the bounded Normal, Solid wedge, and Hashed wedge
vocabulary; solid and hashed actions admit only covalent single `w1` and `h1`
bonds with pointer start as CDML tip and pointer end as base. A pointer-probe
error and a post-resolution document refusal have distinct typed nonmodal recovery;
a valid same-atom attempt is `self_loop` / `adjust_endpoint`.
`UnrenderableCandidate` is `ChangePresentation`. Escape and every typed refusal
remain mutation-free. Existing Bond Properties retains its independently
supported broader bond-style vocabulary; M3.P6 does not narrow that unrelated
editor.

M3.P6 excludes generic stereo/CIP semantics or inference, E/Z semantics,
arbitrary bond styles or orders, and stereo import/export expansion. A fresh
local build and `./all_test.sh` provide the current validation receipt.

M3.P6a is the completed bounded directed-wedge reversal slice. Edit > `Reverse
Selected Wedge Direction` accepts exactly one selected direct `w1` or `h1`
bond. The renderer observation supplies the durable object ID for selection;
Qt supplies the current source ID only to construct the closed Rust operation.
Rust fences and validates that operation, swaps only its ordered endpoints in a
detached candidate, reparses and admits it, and retains durable identity,
unordered connectivity, wedge type, selection, history, CDML persistence, and
atomic typed refusals. A wedge publishes one semantic Bond target whose
selectable envelope derives from its lowered path and line bounds plus the
shared pointer tolerance. Structural child `DisplayOnly` state for the same
bond is removed; unrelated root/reaction exclusion diagnostics remain separate.
The coalescing native-selection refresh timer is a single-shot child of the
`QTabWidget` it queries, so teardown cannot leave a callback that reaches an
invalid tab host.

Permanent evidence is proportionate: deterministic Rust endpoint/history/reopen
semantics, binding contracts, and compact public Qt
click/reverse/eligibility/lifecycle coverage. The broader visible
Undo/Redo/save/reopen workflow is one-time production-shaped evidence, not a
new permanent E2E. This slice adds no generic stereochemistry, source-ID
selection route, renderer-path UI API, or compatibility display-only state.

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

`PARITY-M4.A` is also delivered: `document.compact-group.attach.v1` is the one
public stateless attached-group operation, available through generic protocol
execution and the named document command. Its request carries an admitted CDML
fence, a document-owned molecule/anchor pair, a closed catalog key, and finite
release coordinates. Rust owns target authority, chemistry, geometry, render
admission, durable IDs, and history. Both transports print one typed envelope
and exit `0` for either an accepted outcome or a typed refusal; nonzero status
means usage, input, transport, or output/publication failure. The automated
M4.A gates are closed by the focused Rust evidence, local build, public CLI
E2E, and repository suite. The manual 16:10 and keyboard/accessibility
walkthrough remains open, as do M4 and full parity.

Known-group expansion has delivered the reviewed attached `Me`, `NO2`, `Et`,
`OMe`, `CH2OH`, and `Carboxyl` choices. `Hydroxymethyl` is the fifth attached M4 recipe:
neutral `R-CH2-OH`, carbon focus, attached-only scope, and generic PyO3/Qt
transport with key-neutral accessible refusals. The delivery record is
`docs/active_plans/decisions/m4_attached_hydroxymethyl_v1.md`. The remaining
catalog keys stay separate M4 selections rather than implied capability.
`Carboxyl` is the delivered sixth attached M4 recipe: neutral attached
`R-C(=O)-OH`, carbon focus, and generic Rust-issued key/label transport. Its
delivered contract is recorded in
`docs/active_plans/decisions/m4_attached_carboxyl_v1.md`; no candidate-specific
PyO3 or Qt chemistry branch is introduced. The seventh attached choice, `Cyano`
(`cyano` / `CN`), is delivered under the bounded normal-triple recipe contract
recorded at `docs/active_plans/decisions/m4_attached_cyano_v1.md`: neutral
attached `R-C#N`, carbon focus, and retained exterior identity through generic
Rust materialization. The eighth attached choice, `AcylChloride`
(`acyl_chloride` / `COCl`), is delivered under the bounded neutral
`R-C(=O)-Cl` contract recorded at
`docs/active_plans/decisions/m4_attached_acyl_chloride_v1.md`: carbon focus,
normal C=O/C-Cl topology, retained directed exterior identity, and generic
Rust/binding/Qt transport. The ninth and final attached choice, `Phenyl`
(`phenyl` / `Ph`), is delivered under
`docs/active_plans/decisions/m4_attached_phenyl_v1.md`: generic materialization
creates the neutral six-carbon alternating normal-order Kekule cycle with
carbon focus and preserved directed exterior identity. Role-addressed native
lowering and target-addressed renderer proof cover both exterior orientations;
aromatic-input `kekulize` is not a gate and no aromatic compatibility branch is
introduced. Its final review, fresh build, installed public Qt workflow,
installed binding 8/8, and repository-wide validation are complete. This closes
the compact-group recipe milestone only; M4 remains incomplete.

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

`PARITY-M4.C` is a delivered bounded report-contract repair, not M4 completion.
Each report record now requires exactly one Rust-issued identifier outcome:
the complete canonical SMILES, Standard InChI, and Standard InChIKey trio, or
one closed unavailable reason. Rust evaluates the trio in dependency order;
resource exhaustion remains an operation-level refusal, while Qt presents and
validates the tagged receipt without chemistry calculation. Focused Rust, PyO3,
Qt, and CLI E2E evidence is recorded at the current checkpoint. The remaining
M4 reports, diagnostics, query, and expansion workflows remain open.

The delivered bounded
[`document.molecule.diagnostics.v1` decision](../decisions/m4_molecule_diagnostics_v1.md)
adds deterministic read-only structural findings for a fenced snapshot and
selected durable direct roots, with named CLI, typed PyO3, and modeless
accessible `Check Structure...` delivery. Selection admits the immutable worker;
delivery authenticates the captured fence and receipt rather than live selection.
It excludes auto-fix and runtime chemistry. The source-only durable-identity migration is a separate
foundational follow-on, not an implicit diagnostics compatibility path.

Immediately after diagnostics, deliver the approved
[source-only record identity V1 decision](../decisions/source_only_record_identity_v1.md).
It is delivered. The source boundary requires exact, nonblank,
document-unique source IDs for persisted structural and recognized direct-root
presentation records and replaces the mixed identity representation with
internal `{ kind, source_id }`. CDML does not currently define a lexical
source-ID grammar, so typed ingress must not impose a guessed `NCName` or other
lexical gate. A future grammar requires an explicit CDML specification that
states its scope and migration policy. Typed ingress allocates or preserves an
independent high-entropy document-scoped `DocumentObjectIdV1` before
`RevisionState` serialization. Ferrum-owned namespaced metadata persists that
opaque selector through save, snapshot, history, undo, redo, and reload;
durable-ID grammar and collision validation protect that separate boundary.

WP-ID-2 is also delivered. Reaction observation and listing consume the closed
durable direct-reaction relation built from one shared retained-tree semantic
builder and a private fallible durable binder. They do not serialize XML,
redecode source, or reverse-map a durable ID to a source ID. The completed
implementation evidence covers retained relation history/reload stability and
post-admission reaction/member identity corruption, while retaining existing
session fences, renderer admission, paint order, and display-only diagnostics.

Persisted `PresentationTargetV1` and render `RenderTarget` consume only the
durable selector. Identifier-free preview values remain a distinct transient
type. Public diagnostics and exclusions are source-free: they use a durable ID
when addressable or the closed numeric `DocumentLocationV1` vocabulary before
allocation. The approved A-E sequence is core source contract, typed
ingress/durable metadata, projection/public locations, persisted/preview
target separation, then render/API/Qt convergence and legacy removal. It
carries no reader, alias, occurrence fallback, fingerprint fallback, hash or
source derivation, decoder path, or legacy serde form.

The delivered projection convergence makes `DocumentObjectIdV1` mandatory on
every persisted atom, bond, and molecule projection. The authoritative
`document_object_index` resolves those durable IDs to current document records;
an unknown ID is an ordinary absent target, while absent or malformed persisted
identity metadata is a distinct typed projection error. Preview-only local
projection keys remain separate and never cross the persisted target boundary.

The identity migration is delivered. The current documentation does not claim
an `./all_test.sh` result for this reconciliation; broader delivery evidence
remains owned by the applicable parity gates.

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

`PARITY-M5.A` is approved and in progress as the independently deliverable
[Template Catalog V1](../decisions/m5_template_catalog_v1.md). It consumes the
delivered generic document admission, prepared user-template plan, session transition,
durable identity, and fenced placement contracts. It does not expand the still-open M2
graph/interchange corpus or M4 report, diagnostic, query, and expansion corpus. Rust owns the
immutable shipped-and-user snapshot, opaque key/content identity, provenance, compatibility,
explicit entry/candidate/refusal/file/total-byte limits, bounded lexical admission and aggregate
refusal occurrences, descriptor-relative user-directory admission, and typed refusal. PyO3
projects that contract read-only and accepts a native-issued expected document snapshot for the
placement fence. Rust also owns prepared user-template publication capability and receipt. Qt's
modeless task is split among dialog, tab, and window owners; it exposes only
`chemistry.template.catalog`, keeps no scanner, plan, payload, re-admission, or raw OS errors,
and forwards only fenced native authority. Implementation and all acceptance gates remain open.

The automated M5.A receipt is green: `./build.sh` produced CLI and GUI; focused catalog, API,
PyO3, and Qt suites passed 13, 164, 8, and 18 tests; the public authoring E2E schema
`ferrum-template-catalog-authoring-e2e-v2` reported `ok`; workspace test and strict Clippy both
exited 0; and `./all_test.sh` exited 0 with 8,092 hygiene checks, all registered E2Es, 294
installed PyO3, and 344 Qt tests. Three independent final reviews found no P1/P2/P3. Manual
native accessibility/contrast/focus review and a fresh real-dialog screenshot with human acceptance
remain open, so M5.A, M5, and full parity do not advance to complete.

Before more high-coupling concurrent Qt tools, execute the approved
[Qt Operation Lease Registry](../decisions/qt_operation_lease_registry.md) in two dependent
patches. Patch 1 migrates Template Catalog placement to an explicit controller and a pure Qt
lifecycle registry, deletes both catalog mixins and `CATALOG_PLACEMENT_BLOCKED`, continues the same
close attempt after synchronous cancellation, and changes neither Rust nor PyO3. Patch 2 begins
only after Patch 1 review and full Qt/E2E acceptance; it migrates Local Document Open and proves
source-retaining `CANCELLATION_REQUESTED` plus truthful delivery cancellation. Both patches delete
their old family state atomically and add no aliases, event bus, service locator, or wholesale move.

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

M6 has a delivered bounded context-action foundation. `edit.delete_selection`
is the canonical registered QAction for Delete, Backspace, and the
selection-sensitive YAML context menu. The context builder reuses registered
actions and their current enablement instead of creating parallel actions; its
input client owns context placement, invocation, focus recovery, and keyboard
equivalence, but no chemistry, hit testing, selection mutation, or document
mutation. The remaining M6 keyboard, accessibility, help, clipboard, and
application-usability work stays open.

The periodic picker is a delivered bounded next-drawing usability/authority
slice. Rust owns each immutable symbol, display name, grid coordinate, category,
and color; Qt projects those entries in the **Periodic table...** control and
updates only the shared next-drawing preference. The focused 48-test picker
suite and 5-test real-document invariance suite prove that an accepted choice
does not change document revision, digest, history, or structural selection.

Keyboard structural selection is a delivered bounded M6 slice.
In active **Select Structure** mode, Arrow and Shift+Arrow move only the view
cursor; Enter and Shift+Enter use Rust point selection; a no-hit preserves the
prior selection; Escape restores inactive cursor/accessibility state. The
command-palette focus repair is focused-test green. The selected-atom action
bridge is delivered: it places fenced Rust-issued structural action selection on
the document tab, not on per-atom Qt graphics items or a Python identity model.
Combined focused Qt/PyO3 bridge evidence passed 57 tests; the registered
no-pointer keyboard E2E exited 0; and independent final review accepted with no
P1 finding. The selected-structure YAML context uses the shared enabled actions,
and generic modal-focus handoff restores viewport focus only after terminal
modal lifecycle. Human native keyboard/accessibility sign-off remains separate;
M6 and full parity remain open.

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

The C3-C8 detached regular-ring slice is also complete. `Insert Regular
Ring...` exposes the closed Rust `RegularRingSizeV1` family through one Qt
chooser, and the retained C6 shortcut invokes that same parameterized action.
The generic `DocumentOperationV1` transition retains CDML, renderer, history,
Undo/Redo, and typed refusal ownership; invalid input, Escape, occupied
placement, stale state, renderer refusal, and session conflict remain
mutation-free. Permanent Rust, binding, and real-Qt coverage proves C3-C8
action handoff and topology, Escape disarm without mutation, occupied-click
nonmutation with an armed retry, and retirement after an accepted mutation
cannot refresh its presentation. One-time real-Qt evidence demonstrated
save/reopen and Undo/Redo; the generic persistent-document and history
contracts remain the permanent evidence for those shared behaviors. The
unreachable `HistoryCapacity` commit refusal and all stale Python references
were removed; real history-resource exhaustion remains a preparation-time
typed refusal. The PyO3 prepared-transition classes register through the
feature registry, restoring the one generic public receipt lifecycle for the
ring and other typed authoring operations. This completion does not promote
ring fusion, non-carbon rings, aromaticity, arbitrary ring geometry, or
free-form polygon authoring.

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

The selected `Carboxyl` compact-group slice likewise has a completed bounded
validation record: the fresh local build produced the CLI, Qt application, and installed Python
runtime; attached bindings passed 8/8; and `all_test.sh` passed 7,637 hygiene
checks, all named CLI/Qt E2Es, 280 installed binding tests, and 214 Qt tests
with one skip. One-time installed-Qt evidence proves Rust-issued `carboxyl` /
`COOH` choice transport, public rendered-group hit selection, and terminal
materialization `succeeded` / `updated`. Exact recipe topology and exterior
bond semantics remain the Rust permanent-test responsibility. The raw probe's
narrow `FAIL` reflects only the absent public Molecule Report topology view;
the acceptance adjudication is `PASS`, and that view is future parity scope.
AcylChloride is the delivered eighth attached recipe: approval is `PASS`, a
fresh build promoted the installed runtime, public Qt evidence proves Attach /
chooser / materialize and C/O/Cl composition, and `all_test.sh` passed. Exact
topology and directed exterior identity remain Rust semantic evidence, not a
claim inferred from the public Molecule Report. `Phenyl` is the delivered ninth
attached recipe: final review passed; a fresh build promoted the installed
runtime; public Attach -> chooser -> materialize completed `succeeded` /
`updated`, reported `C8H10`, and retained a usable scene; the installed binding
contract passed 8/8; and `all_test.sh` exited 0 with 7,633 hygiene, 280 binding,
and 220 Qt tests passed with one skip. Exact normal-order Kekule topology,
carbon focus, both directed exterior orientations, native lowering, and renderer
semantics remain Rust proof. All nine attached compact-group recipes are
delivered, but M4 and full Rust/OASA/BKChem parity remain incomplete.

The M0 statement about document-private compact cleanup is superseded for
materialization by the selected
[m4_compact_group_materialization_v1.md](../decisions/m4_compact_group_materialization_v1.md):
the generic protocol, named CLI route, live-session PyO3 registration, and Qt
action are delivered. This does not advance M5 catalog or reaction work.

The delivered M4.A attachment route is maintained as the sole generic
stateless attached-group route. New compact-group command surfaces require a
separate design decision. The remaining compact-group capability in this plan
is broader free compact-group placement, rather than another attachment alias
or a parallel request contract.

The subsequent confirmed queue includes the approved in-progress M5.A Template Catalog V1,
then M2 interchange/graph completion, remaining M5 catalogs and reactions, and M6
usable-application work. The completed catalog protocol prerequisite and M5.A decision do not
advance M5 beyond their remaining parity work.
These milestones stay open until their declared parity claims have contract,
client, corpus, and workflow evidence; this document does not claim complete
OASA/BKChem parity.

For the delivered CML W/H extension, the two former regression-coverage items
are closed. The public protocol regression proves duplicate direct W/H
declarations return `InvalidScalar` without a conversion outcome. The public
`MolBond::directed` contract and `MolGraph::new` make modeled direction/order
compatibility graph-owned; a mutated FCM1 `Double` plus `BEGINWEDGE` or
`BEGINDASH` is rejected as `MalformedNativeResponse`. Focused receipts record
110 chemistry-library tests, 503 document-library tests, and `cargo check
--workspace` passing. This closes neither M2 interchange/graph completion,
full Rust/OASA/BKChem parity, nor the manual 16:10 keyboard/accessibility
walkthrough.

CDXML simple-molecule V1 is no longer a decoder or client-routing queue item.
Its fresh `./all_test.sh`, independent audit, and agent-reviewed real 16:10
File/Open evidence are complete. Human keyboard/accessibility release sign-off
remains separate. CDX binary, reaction/presentation/layout import, namespace
variants, end-directed wedges, and wider chemistry forms remain separate
corpus-backed M2 decisions rather than implied CDXML V1 compatibility.

Implementation reports state: `Milestone`, `Work package`, `Contracts`, `User
workflow`, `Changed files`, `Evidence run`, `Known limits`, and `Next dependency`.

## Open questions and decisions needed

- Set exact CDXML/CML profiles from corpus evidence before promising universal
  interchange.
- Decide PostScript support versus explicit deprecation after user-value evidence.
- Decide reaction SMARTS/SMIRKS adapter scope before reaction import/export.
- Decide plugin isolation technology only after permission/lifecycle approval.
### M6 command palette V1: delivered bounded productivity slice

Ferrum now provides a modeless command palette as a registry-derived command
client. `Ctrl+K` is the portable shortcut policy; Qt renders it as `Cmd+K` on
native macOS. The same action is also available from **View > Commands >
Command Palette...**. The palette searches each live registered action's label,
help text, and stable ID, keeps disabled actions visible with an unavailable
explanation, and invokes the exact selected live `QAction` only after a final
enabled-state check.

The keyboard contract is deliberately narrow: the search field retains focus;
bare Up and Down move result selection; Return activates the selected command;
Escape closes the palette and restores the invoking focus. Modified arrows
remain ordinary text-field input. The action registry remains the sole command
projection and handler owner. `resources/menus.yaml` remains authoritative for
the menu placement.

The keybinding layer validates the complete prospective live shortcut set before
startup setup, user reassignment, or default reset changes preferences, managed
bindings, or an action shortcut. This makes collisions with both managed and
otherwise registered actions atomic failures rather than partial state changes.

Permanent evidence is intentionally compact: focused Qt tests own live search,
disabled-command refusal, exact action activation, bare/modifier arrow behavior,
Escape focus restoration, and atomic live-shortcut collision handling. The
planned reaction-specific palette E2E was rejected as redundant: registered
reaction E2E and focused Qt tests already own its durable semantics. Native
shortcut dispatch and accessibility remain one-time real 16:10 desktop evidence,
not a pixel, timing, or reaction-fixture gate.

Current delivery checkpoint: the independently accepted `ActionRegistry`
token/identity-guarded destruction-retirement repair closes the stale-QAction
defect, and the nominal `DocumentDisplayRefreshableV1` ABC boundary is also
delivered and independently accepted at code level. Permanent lifecycle
regressions cover feature-owned `register_existing()` stable-ID reuse/successor
palette dispatch and portable `register()` plus `bind_qt_action()` destruction,
declaration retention, successor rebinding, and dispatch. The display-refresh
evidence covers nominal membership, structural-look-alike rejection, and direct
delegating-adapter forwarding.

Source review, focused diagnosis, the transactionally staged 13-scene recapture,
and image-by-image agent visual review are complete. The current candidate set
uses a non-persistent documentation theme, visible Rust catalog provenance,
page-contained examples, YAML-owned command breadcrumbs, Rust-measured molecule
bounds, and Rust-owned observed-page centering for interchange imports. Final
human release sign-off remains separate. The guidance-format, fresh build,
complete aggregate, registered E2E, installed PyO3, full Qt, affected Rust test,
strict lint, and isolated wheel gates passed; seven independent post-fix reviews
completed and their actionable findings were repaired. Resume with broader
parity-ledger reconciliation, human release sign-off when preparing a release,
and the later approved, in-progress M5.A decision. That earlier stabilization checkpoint did not
approve M5.A; the current decision does. Neither advances full parity.

This completes one bounded M6 discoverability slice. It does not prove a full
M6 usability program, real desktop visual acceptance, or complete Ferrum parity.
