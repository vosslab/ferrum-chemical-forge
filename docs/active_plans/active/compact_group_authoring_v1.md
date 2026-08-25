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
placement transaction, and limited `Me`/`NO2` materialization experiment have
produced useful evidence. The current local build remains the supported runtime
authority. Existing reports establish these completed foundations:

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

The complete-render-admission ownership failure exposed by the experiment is
closed. [m0_complete_render_admission_v1.md](../decisions/m0_complete_render_admission_v1.md)
records M0 closure on 2026-08-24: generic preparation and one-use commit now
own complete-render admission. Compact-group materialization is therefore an
active typed-document delivery, not an M0 blocker. The generic protocol, named
CLI route, canonical live-session registration, and Qt compact action are
delivered for attached direct-root `Me` and `NO2` groups. The
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
- Materialization is a separate fenced replacement transaction. It retires the
  group, preserves an accepted exterior bond identity, creates recipe atoms and
  internal bonds, preflights the exact candidate, and commits one history entry.
- Group orientation uses a rotation from the catalog attachment-to-local-positive-x
  direction to the exterior atom-to-group-anchor vector, without reflection.
  A free group or zero-length vector uses positive-x.
- V1 bounds are a maximum of 256 graph vertices, 512 bonds, 64 components, and
  one exterior compact-group bond. Refusals leave document, IDs, history,
  selection, and receipt ownership unchanged.

### Proposed M1 public-operation namespace

- When M1 authorizes public compact-group delivery, its only operations will be
  `document.compact-group.place.v1` and
  `document.compact-group.materialize.v1`.
- Their envelopes will be request-owned and generic-dispatcher compatible.
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
evidence. Compact-group protocol, CLI, PyO3, and Qt delivery remain deferred
to M1; do not add unimplemented compact-group symbols to user or API
documentation.

M0 delivered the immutable accepted-only render boundary, shared classifier,
explicit nonvisual-root policy, retirement of raw public candidate routes, and
focused semantic no-bypass evidence. It also moved explicit-hydrogen
materialization and catalog semantic migration through the generic transition.
The remaining compact-group risk is delivery correctness: the typed
replacement transaction must use that completed boundary and the later public
route must preserve the same fenced, one-use mutation ownership.

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

### M3. Preserve the `Me`/`NO2` materialization experiment

Owner: Rust document/session/render.

- Retain the existing attached/free `Me` and attached `NO2` results as a
  focused internal experiment while the typed replacement transaction is
  completed.
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

- Extend the proven insertion/materialization path to the nine reviewed keys,
  without adding free-form aliases or runtime chemistry parsers.
- Implement typed compact-group deletion that atomically removes every incident
  bond when the group is selected; preserve normal bond-only deletion.
- Keep existing full-template catalog placement unchanged. Compact-group
  authoring is a sibling Rust operation, not a new Qt/CDML bypass.
- Maintain closed unavailable/refusal mapping for stale fences, unknown root or
  group, catalog identity mismatch, unsupported/multiple exterior bonds,
  invalid geometry, capacity, resource limits, and unrenderable candidates.

Exit evidence: per-record catalog facts; exact accepted/refused transaction
invariants; atom-only operation refusals on compact-bearing roots; deletion;
undo/redo; and renderer preflight.

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
- Keep materialization scoped to typed attached direct-root `Me` and `NO2`
  groups. Free-form labels, recipes, and legacy aliases remain outside it.
- The generic live PyO3 operation bridge registers materialization beside its
  closest existing live materialization operation. It accepts only the fenced
  request-owned CDML witness and Rust-issued molecule/group identifiers, then
  returns the existing committed transition receipt or typed no-change refusal.
- Bound requests and responses with the shared protocol admission budget. Keep
  diagnostic text redacted; clients consume stable category/recovery facts.

Exit evidence: preserve the delivered schema/protocol/named-CLI materialization
coverage, live-versus-stateless canonical response equivalence, refused
live-session non-mutation, and exact fence behavior.

### M6. Add the usable Qt compact-group workflow

Owner: Qt interaction layer.

Status: materialization action delivered on 2026-08-24. The visible action is
enabled only for one selected typed compact group, sends the current fence and
Rust-issued IDs through the generic live operation, installs the committed
receipt, restores Rust's focus atom, and presents closed typed recovery. The
chooser/attachment workflow remains separate work.

- Add an accessible Chemistry chooser that presents the closed labels and
  concise Rust-defined descriptions. It offers `Attach to Selected Atom` only
  when the Rust availability result admits the current typed selection, and
  always exposes explicit `Place Free` where supported.
- Route the next canvas click only as pointer-to-scene intent. Rust computes
  attachment geometry, chemistry, durable IDs, and committed document state.
- Add `Expand Compact Group`, enabled from public typed selected-group facts.
  It refreshes the authoritative replacement projection and selects the
  returned focus atom.
- Use normal nonmodal typed unavailable/refusal feedback. Qt owns accessible
  wording, chooser state, and transient events; it owns no valence, catalog,
  CDML, or recipe construction.

Exit evidence: a visible selected-group workflow can insert, expand, delete,
and recover from an unavailable anchor through public actions.

### M7. Establish public end-to-end evidence and documentation

Owner: Qt E2E and documentation.

- Add one durable public Qt E2E only after M6: create a document through the
  visible UI, author/select carbon, attach `Me`, verify visible label and typed
  selection, save/reopen, expand, verify ordinary editable structure, then
  undo/redo.
- Add a separate public `NO2` workflow only if it proves a distinct durable
  behavior rather than repeating the same path. Avoid raw CDML, generated IDs,
  private widget/session access, timing assertions, pixel equality, mocks, and
  fixture inventories.
- Document supported catalog vocabulary, compact versus materialized behavior,
  one-attachment limit, and the closed unavailable outcomes in user/API/Qt
  docs. Record the implementation and evidence in the changelog.

## Ownership boundaries

| Concern | Owner |
| --- | --- |
| Catalog facts, labels, attachment metadata | Rust domain |
| Typed compact record, CDML validation, fences, IDs, history, replacement candidate | Rust document/session |
| Structural group endpoint and adjacency | Rust core graph |
| Capacity arithmetic and candidate witness | Rust domain/document chemistry |
| Label/glyph geometry, bond endpoint rendering, hit targets, preflight | Rust renderer |
| Request schema, generic dispatcher, CLI aliases, PyO3 forwarding | Rust API |
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
| Public mutation bypasses complete rendering | Block public operation delivery on M0's opaque accepted admission result and retire raw candidate routes. |
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
3. One permanent public Qt E2E: attach, visible group, selection, Save/Open,
   materialize, Undo/Redo. It remains semantic rather than pixel-based.
4. One-time implementation evidence: differential-oracle comparisons between
   the retired experimental path and M0's profile, `Me`/`NO2` receipts,
   renderer visual inspection, and screenshot capture. These do not become
   routine gates unless they meet the permanent-test criteria.
5. Completion gates: `./build.sh`, `./all_test.sh`, documentation-link/style
   checks through the normal suite, and a fresh independent architecture/code
   audit after M7.

## Completion criteria

M0 establishes complete-render admission as an unbypassable generic visual
mutation prerequisite. It does not authorize a public compact-group operation.
The compact-group vertical slice begins in M1 and completes only when the
reviewed catalog entries are usable through the approved public generic
operation, local CLI route, PyO3 route, and accessible Qt flow; groups remain
durable visible/selectable document objects; materialization and deletion are
atomic; refusal semantics are shared and bounded; public semantic E2E evidence
is green; and the documented local build and full validation suite pass.
Broader group grammar, publishing, and installation remain separate work.
