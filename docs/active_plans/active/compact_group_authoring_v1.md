# Compact group authoring V1 forward plan

## Purpose and scope

Deliver Ferrum-owned compact known-group authoring as a first-class, durable
document capability. The initial catalog target is `Me`, `Et`, `Ph`, `OMe`,
`NO2`, `CN`, `COOH`, `COCl`, and `CH2OH`. A group must display as a readable
abbreviation, survive save/reopen, support durable selection and deletion, and
materialize atomically into ordinary editable atoms and bonds.

This plan advances the OASA/BKChem parity objective through a Rust-owned design.
Historical OASA/BKChem behavior, Python group classes, label-to-SMILES parsing,
publishing, installation, network catalogs, and a broad template palette are
outside this vertical slice. Pre-production Ferrum does not retain a legacy-group
migration path.

## Evidence and current stability

The typed compact-group representation, core-vertex bridge, capacity admission,
placement transaction, and all nine delivered typed materialization recipes for
`Me`, `NO2`, `Et`, `OMe`, `CH2OH`, `Carboxyl`, `Cyano`, `AcylChloride`, and
`Phenyl` establish the current evidence. Phenyl is delivered under the closed
normal-order Kekule contract in `m4_attached_phenyl_v1.md`: native lowering and
target-addressed draw-plan semantics prove both directed exterior orientations;
aromatic-input `kekulize` is not a gate. Its final review, fresh build, public
installed Qt workflow, installed binding contract, and repository-wide suite
are complete. The compact-group recipe milestone is complete, but M4 and full
Rust/OASA/BKChem parity remain incomplete. The current local build remains the
supported runtime authority. Existing reports
establish these completed foundations:

- `DocumentSession` already owns revision/digest fences, durable ID allocation,
  renderer preflight, atomic history, and opaque live authoring receipts.
- Existing catalog placement, generic operation execution, local CLI aliases,
  and the generic PyO3 gateway provide one route from request-owned operations
  to Rust-owned mutations.
- The compact-group projection/render/selection seam and the compact-group
  core-vertex bridge establish the intended first-class group representation:
  a group is observable and can be an atom-to-group bond endpoint without
  becoming a chemistry atom.
- The ordinary attachment-capacity design defines the shared candidate-aware
  admission needed to keep Qt, CLI, and later fragment authoring aligned.
- `document.compact-group.attach.v1` is the delivered public stateless
  attached-group route. Its generic protocol and named document-command
  transports use the same fenced request and typed envelope: an accepted
  outcome and a typed refusal both exit `0`, while nonzero status reports
  usage, input, transport, or output/publication failure.

The complete-render-admission ownership failure exposed by the experiment is
closed. [m0_complete_render_admission_v1.md](../decisions/m0_complete_render_admission_v1.md)
records M0 closure on 2026-08-24: generic preparation and one-use commit now
own complete-render admission. Compact-group materialization is therefore an
active typed-document delivery, not an M0 blocker. The generic protocol, named
CLI route, canonical live-session registration, and Qt compact action are
delivered for typed direct-root `Me`, `NO2`, `Et`, `OMe`, `CH2OH`, `Carboxyl`,
`Cyano`, `AcylChloride`, and `Phenyl` groups. The
internal experiment remains one-time evidence for its implementation limits,
not a route for additional recipes or a legacy alias.

## Frozen design choices

- A compact group is a Rust-owned typed document object, not an atom, Python
  object, XML interpreted by Qt, OASA molecule, parser input, or template
  fallback.
- The catalog is closed, immutable, versioned Rust data. Each record owns an
  exact key, derived display label, atom/bond facts, local coordinates, one
  attachment site, and a single ordinary exterior-bond profile. Labels are not
  mutable chemistry inputs.
- The persisted compact-group identity is typed and durable. Its projection
  carries the key, derived label, finite anchor, attachment data, source order,
  and durable object ID. Rendering, hit testing, and selection derive from that
  projection in Rust.
- A compact group is a `NonAtomVertex` only for structural endpoint resolution.
  Atom-only chemistry consumers continue to reject a compact-bearing root under
  their existing closed outcomes; no consumer substitutes a formula or ignores
  the group.
- Attached insertion uses one ordinary normal single bond and a Rust-owned,
  candidate-aware capacity witness. Qt presents Rust-derived availability and
  supplies only selected durable address plus pointer/scene intent.
- Materialization is a separate fenced replacement transaction. It removes the
  group record, preserves an accepted exterior bond identity, creates recipe atoms and
  internal bonds, preflights the exact candidate, and commits one history entry.
- Group orientation uses a rotation from the catalog attachment-to-local-positive-x
  direction to the exterior atom-to-group-anchor vector, without reflection.
  A free group or zero-length vector uses positive-x.
- V1 bounds are a maximum of 256 graph vertices, 512 bonds, 64 components, and
  one exterior compact-group bond. Refusals leave document, IDs, history,
  selection, and receipt ownership unchanged.

### Delivered compact-group operation namespaces

- `document.compact-group.attach.v1` is the delivered public operation for one
  attached closed-catalog group. Its request carries a fenced CDML snapshot, a
  document-owned molecule/anchor pair, a closed catalog key, and finite release
  coordinates. Rust owns authorization, mutation, and the reusable next fence;
  generic protocol execution and the named document command are the two
  transports for the same operation.

- `document.compact-group.materialize.v1` is the delivered public operation for
  typed direct-root `Me`, `NO2`, `Et`, `OMe`, `CH2OH`, `Carboxyl`, `Cyano`,
  `AcylChloride`, and `Phenyl` groups. Its stateless envelope is
  request-owned and generic-dispatcher compatible.
- Free compact-group placement remains the separately planned capability. It
  does not broaden either delivered attached-group route.
- Internal experiment methods are implementation evidence only; they are not
  aliases, alternate public contracts, CLI commands, or PyO3 entry points.
- Publishing and installation are outside this plan. Local builds produce the
  runnable testing application inside this repository.

## Ordered implementation milestones

### M0. Stabilize complete-render admission ownership

Owner: approved cross-crate architecture, then Rust document/render-contract.

Status: closed on 2026-08-24. The authoritative
[m0_complete_render_admission_v1.md](../decisions/m0_complete_render_admission_v1.md)
records the completed generic admission core, route migrations, and exit
evidence. Compact-group protocol, CLI, PyO3, and Qt materialization delivery
are complete; the durable render-target and Rust-issued availability migration
is complete for the delivered materialization workflow. Remaining compact
authoring work is limited to separately scoped catalog and chooser capabilities.

M0 delivered the immutable accepted-only render boundary, shared classifier,
explicit nonvisual-root policy, removal of raw public candidate routes, and
focused semantic no-bypass evidence. It also moved explicit-hydrogen
materialization and catalog semantic migration through the generic transition.
The remaining compact-group risk is delivery correctness: future free placement
must use that completed boundary and preserve the same fenced mutation
ownership as the delivered attachment route.

### M1. Lock the typed group representation and public contract

Owner: Rust document/domain architecture.

- Document the exact request/response envelopes, closed refusal names, and
  recovery facts for the canonical public operation pair in the API contract.
- Keep the immutable catalog in a small Rust domain module. Start with the
  full nine-key vocabulary but expose only the reviewed records through a
  typed catalog interface.
- Ensure parser/writer validation accepts only the Ferrum compact-group
  key/type relationship and derives labels from the key. An unrecognized
  group-like record follows a closed format refusal; it is neither retained as
  a legacy object nor silently reclassified.
- Confirm compact group projection, durable target vocabulary, renderer
  transfer, label/glyph primitive, hit testing, and reopen selection carry the
  same durable identity.

Exit evidence: focused typed round-trip, projection, visible render target,
hit selection, and save/reopen tests. Invalid keys, attachments, and geometry
must refuse before mutation.

### M2. Deliver shared candidate-aware ordinary attachment admission

Owner: Rust domain and document chemistry.

- Implement the closed neutral ordinary-single capacity arithmetic as a domain
  primitive, including explicit-H demand, bond-order demand, authored capacity
  overrides, multiplicity, charge, aromaticity, unsupported order, overflow,
  and finite element profile outcomes.
- Implement document-side fenced selector resolution and candidate-witness
  verification. The admission must prove that the exact detached candidate has
  the expected selected atom, new group, and canonical exterior bond.
- Put shared root/incident bounds in one document chemistry limits module.
- Expose a read-only Rust availability result for UI enablement while repeating
  candidate admission before every commit to remove time-of-check/time-of-use
  dependence.

Exit evidence: valid carbon admission, full-valence refusal, explicit-H demand,
non-neutral and unsupported source facts, foreign/altered witness, bounds, and
no-ID/no-history mutation on every refusal.

### M3. Preserve materialization implementation evidence

Owner: Rust document/session/render.

- Retain the original attached/free `Me` and attached `NO2` experiment as
  one-time implementation evidence. The delivered typed transaction now also
  covers attached `Et` and `OMe`.
- Use it to prove deterministic orientation, exterior-bond preservation, focus
  mapping, one undoable transition, undo/redo, and save/reopen only through
  the new complete-render admission profile.
- Classify current experimental comparisons as one-time implementation evidence.
  Promote only stable document contracts to permanent tests under
  [PYTEST_STYLE.md](../../PYTEST_STYLE.md). If the profile changes the observed
  materialization result, stop catalog expansion and design a replacement
  materialization algorithm before continuing.

### M4. Generalize the reviewed catalog transaction

Owner: Rust document/domain/render.

Status: compact-group deletion is delivered for exactly one selected rendered
compact group. The renderer supplies its parent molecule and compact-group
`DocumentObjectIdV1` values; Rust verifies direct membership before preparation.
The detached typed mutation removes exactly that group and its unique exterior
bond, with no atoms, generated IDs, or component facts. It commits through one
history transition and supports replay refusal, Undo, and Redo. Mixed or
multi-group selections are refused before preparation.

Delivered for all nine supported attached choices: `AttachCompactGroupV1`
is now the one generic Rust transaction for `Me`, `NO2`, `Et`, `OMe`, `CH2OH`,
`Carboxyl`, `Cyano`, `AcylChloride`, and `Phenyl`. Rust projects the catalog key and
derived label, evaluates choice-specific availability, and owns the candidate
through commit or cancel. The private PyO3 binding exposes generic choices,
availability, begin, preview, commit, and cancel. Qt renders the Rust choices
and owns only the accessible chooser and canvas capture. Methyl-specific Rust,
PyO3, and Qt entry points were removed rather than retained as compatibility
layers. `Hydroxymethyl` is the delivered fifth attached recipe. Its neutral
`R-CH2-OH` contract returns carbon focus and retains attached-only scope;
generic PyO3/Qt transport and accessible refusal wording remain key-neutral.
The delivery record is `docs/active_plans/decisions/m4_attached_hydroxymethyl_v1.md`.

The canonical `NO2` materialization recipe is `R-[N+](=O)[O-]`.
`CompactGroupRecipeAtomV1` retains optional formal charge, and Rust tests cover
the `+1` nitrogen and `-1` oxygen through materialization, history, and reopen.
The canonical `Et` recipe is two neutral carbons joined by one normal single
bond. Independent recipe/session review, row-level chooser availability, the
generic binding seam, and a production-shaped visible Qt walkthrough cover
attachment and materialization; typed session evidence owns Undo, Redo, and
reopen without a duplicate GUI lifecycle fixture.
The public E2E proves the net-zero report and editable graph, not private
per-atom charge storage. The canonical `OMe` recipe is neutral `R-O-CH3`, with
the exterior bond and returned focus on oxygen. Renderer-owned attached-pose
normalization handles too-close releases through the existing generic path.
The decision at `docs/active_plans/decisions/m4_attached_methoxy_v1.md` records
the approved scope and semantic evidence. `Carboxyl` is the delivered sixth
attached recipe: neutral attached `R-C(=O)-OH`, carbon focus, and generic
Rust-issued key/label transport. Its delivered contract is recorded in
`docs/active_plans/decisions/m4_attached_carboxyl_v1.md`. `Cyano` is the
delivered seventh attached recipe under
`docs/active_plans/decisions/m4_attached_cyano_v1.md`: neutral attached
`R-C#N`, carbon focus, a canonical recipe-owned normal triple bond, retained
exterior identity, and generic Rust-issued key/label transport. `AcylChloride`
is the delivered eighth attached recipe under
`docs/active_plans/decisions/m4_attached_acyl_chloride_v1.md`: neutral attached
`R-C(=O)-Cl`, carbon focus, canonical normal double/single topology, retained
exterior identity, and generic Rust-issued key/label transport. Its approval is
`PASS`; fresh build, installed Qt, and repository-wide validation are complete.
Rust supplies exact topology and directed exterior-identity evidence; Qt supplies
the public chooser/materialization receipt and C/O/Cl composition evidence.
`Phenyl` is the delivered ninth and final attached recipe under
`docs/active_plans/decisions/m4_attached_phenyl_v1.md`: `phenyl` / `Phenyl` /
`Ph` materializes through the generic route as a neutral six-carbon alternating
normal-order Kekule cycle with carbon focus. Both directed exterior orientations,
role-addressed native lowering, and target-addressed renderer semantics are
proved without aromatic schema or compatibility logic; aromatic-input `kekulize`
is not a gate. Final review, fresh build, installed public Qt workflow, binding
8/8, and the repository suite are complete. This closes the compact-group recipe
milestone only; full Rust/OASA/BKChem parity remains incomplete. The delivered
generic attached CLI/protocol operation is recorded in
[FULL_PARITY_RUST_FIRST.md](FULL_PARITY_RUST_FIRST.md#parity-m4a-generic-attached-compact-group-cliprotocol-delivered).

Carboxyl has completed its current validation gate: a fresh build produced the
local CLI, Qt application, and installed Python runtime; the attached binding
suite passed 8/8; and final `all_test.sh` passed 7,637 hygiene checks, all
named CLI/Qt E2Es, 280 installed binding tests, and 214 Qt tests with one skip.
The one-time installed-Qt workflow selected the Rust-issued `carboxyl` / `COOH`
choice, hit-selected the rendered group publicly, and materialized it to
terminal `succeeded` / `updated`. Exact topology and exterior-bond preservation
remain Rust/session permanent evidence. The probe receipt's narrow `FAIL`
means only that public Molecule Report cannot independently reveal private bond
topology; the acceptance adjudication is `PASS`, and a public topology report
is future parity work rather than a sixth-recipe blocker. AcylChloride is the
delivered eighth attached recipe: approval is `PASS`, and its fresh build,
installed Qt, and repository-wide validation gates are closed. `Phenyl` is the
delivered ninth recipe: its final review passed; its fresh build, installed Qt,
binding 8/8, and repository-wide validation gates are closed. All nine attached
compact-group recipes are delivered, completing this recipe milestone without
completing M4.

- Preserve the delivered one-group deletion contract. Broader selection forms
  remain refused rather than being lowered into a synthetic batch operation.
- Keep existing full-template catalog placement unchanged. Compact-group
  authoring is a sibling Rust operation, not a new Qt/CDML bypass.
- Maintain closed unavailable/refusal mapping for stale fences, unknown root or
  group, catalog identity mismatch, unsupported/multiple exterior bonds,
  invalid geometry, capacity, resource limits, and unrenderable candidates.

Exit evidence: per-record catalog facts; exact accepted/refused transaction
invariants; atom-only operation refusals on compact-bearing roots; the delivered
one-group deletion/Undo/Redo receipt; and renderer preflight.

### M5. Complete remaining compact delivery adapters

Owner: Rust API and binding adapters.

Status: the read-only `MoleculeProjectionV1.compact_groups` PyO3 child DTO and
the native Qt render-target prerequisite are complete. Every rendered compact
group now has the closed Qt target kind `compact_group`, its Rust-issued group
document ID, and its parent molecule document ID. This remains a passive
projection/render bridge: it does not expose recipes, infer chemistry, or
perform materialization.

- `document.compact-group.materialize.v1` is already in the single
  `ferrum-operation-request-v1` schema, generic dispatcher, `protocol run`,
  and named document command. Do not reschedule or duplicate that route.
- Keep materialization scoped to typed direct-root `Me`, `NO2`, `Et`, `OMe`,
  `CH2OH`, `Carboxyl`, `Cyano`, `AcylChloride`, and `Phenyl` groups.
  Free-form labels, recipes, and legacy aliases remain outside it.
- The generic live PyO3 operation bridge registers materialization beside its
  closest existing live materialization operation. It accepts only the current
  session fence and Rust-issued durable molecule/group IDs, then returns the
  existing committed transition receipt or typed no-change refusal. The
  stateless protocol separately retains its admitted-CDML source-ID contract.
- Bound requests and responses with the shared protocol admission budget. Keep
  diagnostic text redacted; clients consume stable category/recovery facts.

Exit evidence: preserve the delivered schema/protocol/named-CLI materialization
coverage, live-versus-stateless canonical response equivalence, refused
live-session non-mutation, and exact fence behavior.

### M6. Add the usable Qt compact-group workflow

Owner: Qt interaction layer.

Status: delivered for the public generic attached nine-recipe workflow and
unavailable-anchor recovery on 2026-08-25.

The existing `Attach Compact Group...` action follows the current Rust-issued
general-readiness fact for the selected durable atom. A fact whose revision,
digest, and anchor exactly match the live document keeps the action enabled
when the selected anchor is ready for the reviewed normal-single attachment
profile. It then opens the visible chooser, which exposes `Me`, `NO2`, `Et`,
`OMe`, `CH2OH`, `Carboxyl`, `Cyano`, `AcylChloride`, `Phenyl`, and `Attach to
Selected Atom`; one canvas release supplies pointer intent only.
Rust evaluates choice-specific availability only after chooser selection. The
all nine delivered keys share the supported normal-single attachment profile.
An exact-current unavailable target is the standard
accessible `Action Not Available` modal exception, using the existing typed
refusal path and the selected Rust-issued label or label-neutral wording.
Dismissing that refusal leaves no
false pointer ownership or document mutation. A stale, missing, or nonmatching
fact keeps the action disabled with generic readiness guidance; Qt must not
infer or present chemical unavailability from a fact that is not exactly current.

Rust performs availability, capacity and geometry admission, durable group and
bond ID allocation, complete-render admission, and the one atomic history
transition. Qt installs the committed receipt and does not carry raw CDML or
source IDs. The private PyO3 bridge remains an implementation detail of this
local Qt route, not a public attachment API. The chooser has a reentrancy guard
so a duplicate activation cannot replace an already-open chooser.

The delivered public E2E creates methane and an eligible C-C target in one
visible document, selects the saturated methane carbon, invokes the enabled
action, reads and dismisses the accessible refusal, then uses Select Structure
in that same document to select the eligible carbon and complete the normal
chooser flow. It exercises a queued duplicate activation while the chooser is
open, materializes the resulting group, and reads the visible Molecule Report
result `Formula: C3H8`. It does not use private bridge access, raw IDs, raw
CDML, mocks, pixel equality, or fixed-duration waits. Zero-delay event-queue
deferrals are permitted to express ordering; a generous phase-scoped liveness
guard may fail a hung E2E but is not a product performance gate.

- Keep Rust responsible for availability, catalog chemistry, attachment
  geometry, durable ID allocation, render admission, and the atomic commit.
  Qt supplies chooser state and pointer intent; it does not reconstruct
  chemistry, issue IDs, admit a render, or stage an independent mutation.
- Keep the PyO3 bridge private to the local Qt application. Do not add a CLI
  command, stateless protocol route, public binding contract, or public catalog
  expansion for this slice.
- Preserve `document.compact-group.materialize.v1` and its existing public
  delivery route unchanged. This attachment slice does not reschedule,
  duplicate, or broaden materialization.
- Keep permanent coverage to durable contracts: the fenced private lifecycle,
  Rust-issued inputs and outputs, closed refusals, renderer-admitted preview,
  cancel/refusal non-mutation, and one atomic committed transition. Classify
  attached-cyclohexane demonstrations, renderer inspection, and exploratory
  timing or visual probes as one-time evidence, not routine gates.
- The accessible `Draw > Compact groups` chooser is delivered for `Me`, `NO2`,
  `Et`, `OMe`, `CH2OH`, `Carboxyl`, `Cyano`, `AcylChloride`, `Phenyl`, and
  `Attach to Selected Atom`. Rust owns the post-selection availability decision.
- Keep free placement in its separate `Place Compact Group...` chooser. It is
  delivered for `Me`; the remaining reviewed catalog keys stay separate work.
- For an exact-current unavailable selected atom, keep the existing `Attach
  Compact Group...` action enabled. Its activation presents the standard,
  accessible `Action Not Available` typed refusal with the exact primary
  selected Rust-issued label or label-neutral wording. Dismissing it and using
  public Select Structure in the same
  document to select an eligible atom opens the ordinary chooser. A stale,
  missing, or nonmatching availability observation instead leaves the action
  generically disabled. The Rust availability taxonomy remains advisory; Qt
  does not add a fallback or new action.
- Guard chooser reentrancy so a duplicate queued activation cannot replace an
  open chooser or create a second mutation route. If selection changes before
  commit, preserve the existing typed refusal and refresh the existing action;
  this does not alter the schema.
- The existing Select Structure tool delivers compact deletion without a new
  Qt action: select exactly one group and press `Delete` or `Backspace`. Qt
  forwards the renderer-issued parent/group durable IDs and presents the
  authoritative combined receipt; it does not derive membership or mutation
  effects.
- The delivered nine-key chooser does not imply admission of later keys. Any
  later key requires its own catalog, availability/refusal, and public semantic
  evidence review. Existing generic public workflow evidence does not require a
  per-key E2E.
- Retain `Materialize Selected Compact Group` as the delivered action. It uses
  only a Rust-issued fenced availability observation for enablement, refreshes
  the authoritative replacement projection, and selects the returned durable
  focus atom.
- Use normal nonmodal typed feedback where the route remains nonmodal. The
  exact-current unavailable attachment outcome is the standard accessible
  `Action Not Available` modal exception. Qt owns accessible wording, chooser
  state, and transient events; it owns no valence, catalog, CDML, or recipe
  construction.

Exit evidence delivered: the generic attached nine-recipe transaction,
visible C-C to compact-group attachment, compact selection,
deletion through the existing `Delete` control with a group-aware receipt,
public Formula `C2H6`, and public Undo that restores the compact group. The
restored group then materializes successfully through the public route, and
`Molecule Report...` returns public Formula `C3H8`. The registered public
unavailable-anchor E2E authors saturated CH4 and eligible C-C, invokes the
enabled existing action on the exact-current unavailable selection, reads and
dismisses its accessible typed refusal, then uses Select Structure in that same
document to attach and materialize `Me` to prove `Formula: C3H8`. It also
proves guarded chooser reentrancy. The delivered `Carboxyl` catalog/session
slice has focused Rust evidence; its one-time installed-Qt smoke, fresh build,
and full `all_test.sh` evidence remain pending. Free `Me` placement is separately
delivered; Cyano is the delivered seventh attached recipe. AcylChloride is the
delivered eighth attached recipe, with `PASS` approval and closed Rust review,
fresh-build, installed-Qt, and repository-wide validation gates. `Phenyl` is
the delivered ninth recipe: both exterior orientations are proved through
role-addressed native graph lowering and target-addressed renderer semantics;
aromatic-input `kekulize` is not a gate. Its final review, fresh build, public
installed Qt workflow, binding 8/8, and repository-wide suite are complete.
All nine attached compact-group recipes are delivered; this recipe milestone is
complete, while M4 remains incomplete.

### M7. Establish public end-to-end evidence and documentation

Owner: Qt E2E and documentation.

Status: the former raw-CDML/mock compact-group E2E has been removed. The
replacement public E2E is delivered and registered as
`tests/e2e/e2e_compact_group_author_to_materialize.py`; it proves the visible
C-C to `Me` to materialized `Formula: C3H8` workflow.

`tests/e2e/e2e_compact_group_delete.py` is also registered. It creates and
authors `Me` through visible Qt controls, explicitly selects the group, presses
the existing `Delete` control, verifies the group-aware committed receipt and
public `Formula: C2H6`, then uses public Undo to restore the compact group.
It materializes that restored group through the public route and verifies
`Molecule Report...` returns public `Formula: C3H8`.

`tests/e2e/e2e_compact_group_unavailable_anchor_recovery.py` is registered. It
uses public Draw Bond gestures to author saturated CH4 and eligible C-C,
invokes the enabled existing action on the exact-current unavailable atom,
reads and dismisses the accessible standard typed refusal, then uses Select
Structure in the same document to complete the ordinary chooser flow. It
queues one duplicate activation while the chooser is open to prove guarded
reentrancy, then attaches and materializes `Me` to prove public `Formula:
C3H8`.

`tests/e2e/e2e_free_compact_group_placement.py` is registered. It uses
the visible `Place Compact Group...` action, the Me-only chooser, and one canvas
release, then reads `Authored graph: 1 atoms, 0 bonds` and `Formula: CH4` from
the public Molecule Report. These observations distinguish the explicit
authored representation from the compact-group chemistry.

- Retain public attached-choice E2Es only where each one proves a distinct
  durable behavior. The delivered `Me`, `NO2`, `Et`, and `OMe` workflows use visible controls
  and event-driven report completion, without raw CDML, private
  controller/session access, raw IDs, mocks, timing, or pixels.
- Retain the public compact-group deletion E2E as the durable ownership gate:
  it must use visible selection, `Delete`, public report, and public Undo rather
  than raw CDML, generated IDs, private bridge/session access, mocks, timing,
  or pixel comparisons.
- Retain the public unavailable-anchor recovery E2E. It must use public Draw
  Bond gestures and accessible UI only, with no raw CDML, private session or
  bridge access, pixels, network, mocks, or fixtures. `QTimer.singleShot(0)`
  is permitted solely to order modal dismissal or duplicate activation through
  the event queue; it is not a timing assertion. One phase-scoped liveness
  guard may terminate a deadlocked E2E and is not product-performance
  acceptance.
- Retain the public free-Me placement E2E as the durable direct-root workflow
  gate. It must use the visible action, chooser, canvas release, and public
  report, including the authored-graph and formula observations, rather than
  raw CDML, private bridge/session access, IDs, fixtures, timing, pixels, or
  byte equality. `CH2OH` adds no public E2E because this generic workflow is
  already covered by its durable Rust semantics and existing public workflow evidence.
- Keep user/API documentation aligned with the delivered nine-recipe
  materialization boundary, compact versus materialized behavior, and closed
  Rust availability outcomes. The generic attached CLI/protocol route remains
  one operation with two transports, not a recipe-command family.

## Ownership boundaries

| Concern | Owner |
| --- | --- |
| Catalog facts, labels, attachment metadata | Rust domain |
| Typed compact record, CDML validation, fences, IDs, history, replacement candidate | Rust document/session |
| Structural group endpoint and adjacency | Rust core graph |
| Capacity arithmetic and candidate witness | Rust domain/document chemistry |
| Label/glyph geometry, bond endpoint rendering, hit targets, preflight | Rust renderer |
| Request schema, generic dispatcher, CLI aliases, PyO3 forwarding | Rust API |
| Private M6 generic attached-group lifecycle and atomic transition | Rust document/session via PyO3 |
| Chooser, accessibility, pointer conversion, visible feedback | Qt |
| Durable public workflow evidence | Qt E2E |

## Risks and controls

| Risk | Control |
| --- | --- |
| A label becomes a second chemistry model | Derive labels from one closed Rust key and prohibit free-form parser input. |
| Group UI drifts from CLI capacity behavior | Use one document availability/admission contract and repeat it on the exact candidate. |
| A group endpoint becomes a pseudo-atom | Retain `NonAtomVertex` structural role and explicit atom-only consumer refusals. |
| Materialization breaks durable references | Preserve permitted exterior-bond identity and return an explicit focus mapping. |
| Renderer admits an unselectable group | Make typed projection, render target, hit test, and durable selection one prerequisite gate. |
| Public mutation bypasses complete rendering | Block public operation delivery on M0's opaque accepted admission result and remove raw candidate routes. |
| Broad catalog work masks a flawed transaction | Keep `Me`/`NO2` as a measured experiment with an explicit retain-or-redesign decision. |
| Tests create a private parallel workflow | Restrict permanent E2E evidence to public UI semantics and classify one-time probes separately. |

## Validation tiers

1. Focused permanent Rust contracts: complete-render admission ownership,
   accepted-only DTO, shared classification, nonvisual-root policy, no-bypass
   behavior, typed persistence, catalog data, capacity, candidate integrity,
   transaction atomicity, core bridge, render/hit/selection, undo/redo, and
   protocol equivalence.
2. Focused permanent Qt public behavior: accessibility/action enablement,
   authoritative refresh, and only public visible workflow assertions.
3. The registered permanent compact-materialization Qt E2E covers public
   authoring, explicit compact selection, materialization, and the visible
   `Formula: C3H8` report result.
4. The M6 private generic attached-group lifecycle receives permanent focused
   contract coverage for fenced begin/preview/commit/cancel behavior,
   post-selection availability, and atomic no-mutation refusals.
   Attached-cyclohexane demonstrations, renderer inspection, and exploratory
   timing or visual probes remain one-time evidence; they do not become routine
   gates or substitute for public Qt workflow evidence.
5. One-time implementation evidence: differential-oracle comparisons between
   the removed experimental path and M0's profile, `Me`/`NO2` receipts,
   renderer visual inspection, and screenshot capture. These do not become
   routine gates unless they meet the permanent-test criteria.
6. Completion gates: `./build.sh`, `./all_test.sh`, documentation-link/style
   checks through the normal suite, and a fresh independent architecture/code
   audit after M7.

## Completion criteria

M0 establishes complete-render admission as an unbypassable generic visual
mutation prerequisite. It does not authorize a public compact-group operation.
The compact-group vertical slice begins in M1 and completes only when the
reviewed catalog entries are usable through the approved Rust transaction,
PyO3 route, and accessible Qt flow; groups remain
durable visible/selectable document objects; materialization and deletion are
atomic; refusal semantics are shared and bounded; public semantic E2E evidence
is green; the generic attached nine-key slice has passed its public Qt
author-to-materialize evidence and independent review; and the documented local
build and full validation suite pass. The compact-group recipe milestone is
complete, while full parity remains pending. The delivered generic CLI/protocol
operation retains one representative public route E2E while the nine-recipe
semantic matrix stays in focused Rust tests.
Broader group grammar, publishing, and installation remain separate work.
## Free methyl placement reference

The delivered free-`Me` operation, its public E2E evidence, and its remaining
scope are authoritative in M6, M7, and the completion criteria above.
