# Compact group authoring V1 forward plan

## Purpose and scope

Deliver Ferrum-owned compact known-group authoring as a first-class, durable
document capability. The initial catalog target is `Me`, `Et`, `Ph`, `OMe`,
`NO2`, `CN`, `COOH`, `COCl`, and `CH2OH`. A group must display as a readable
abbreviation, survive save/reopen, support durable selection and deletion, and
materialize atomically into ordinary editable atoms and bonds.

This plan advances the OASA/BKChem parity objective through a Rust-owned design.
Legacy OASA/BKChem compatibility, Python group classes, label-to-SMILES parsing,
legacy group migration, publishing, installation, network catalogs, and a broad
template palette are outside this vertical slice.

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

The experiment also exposed an unresolved foundational failure: public
`DocumentSession` preparation/commit paths can admit compact-group candidates
without the renderer's complete admission. This is a design boundary failure,
not a compact-group algorithm failure. Public compact-group operations, API,
CLI, PyO3, and Qt work remain blocked until M0 supplies a document-owned lower
complete-render admission contract. The internal experiment remains useful
one-time evidence, but is not a public operation or a completion receipt.

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

### Public-operation namespace

- The only public compact-group operations are
  `document.compact_group.place.v1` and
  `document.compact_group.materialize.v1`.
- Their envelopes are request-owned and generic-dispatcher compatible.
- Internal experiment methods are implementation evidence only; they are not
  aliases, alternate public contracts, CLI commands, or PyO3 entry points.
- Publishing and installation are outside this plan. Local builds produce the
  runnable testing application inside this repository.

## Ordered implementation milestones

### M0. Stabilize complete-render admission ownership

Owner: approved cross-crate architecture, then Rust document/render-contract.

- Obtain architect approval for a document-owned lower complete-render admission
  profile before changing the crate boundary. The profile accepts the exact
  immutable candidate that would commit and produces only an accepted-only
  render DTO. It is not a lossy observation DTO, a mutable document view, or a
  renderer callback into a document session.
- Define typed candidate-derivation failures and one shared classifier
  vocabulary in `render-contract`. Document and renderer must classify the
  same accepted/refused candidate facts without string matching or parallel
  taxonomy.
- Record an explicit policy for valid nonvisual roots. The profile must state
  which roots are admitted without visible primitives and why; it must not
  silently treat missing render output as an accepted visual candidate.
- Remove raw candidate-CDML getters and bridge receipts from public session
  mutation surfaces. Public mutation preparation and commit must depend on the
  opaque accepted admission result, so no caller can create or redeem a raw
  candidate that bypasses complete-render admission.
- Use the existing compact-group placement and materialization implementations
  only as differential-oracle inputs while comparing the new profile against
  current renderer behavior. This comparison is one-time implementation
  evidence, not a permanent test or compatibility commitment.
- Add permanent contract, document, renderer, and no-bypass tests. They must
  cover accepted DTO construction, typed derivation/classification failures,
  nonvisual-root policy, exact-candidate binding, and rejection of every
  public raw-preparation or bridge-receipt bypass. The tests should assert
  stable behavior and ownership boundaries, not private layout or timing.

Exit evidence: architect-approved boundary, one immutable accepted-only DTO,
shared classifier taxonomy, explicit nonvisual-root behavior, retired raw
public candidate routes, and focused permanent tests proving a public compact
operation cannot commit outside complete-render admission.

### M1. Lock the typed group representation and public contract

Owner: Rust document/domain architecture.

- Document the exact request/response envelopes, closed refusal names, and
  recovery facts for the canonical public operation pair in the API contract.
- Keep the immutable catalog in a small Rust domain module. Start with the
  full nine-key vocabulary but expose only the reviewed records through a
  typed catalog interface.
- Ensure parser/writer validation accepts only the Ferrum compact-group
  key/type relationship, derives labels from the key, and retains ordinary
  imported legacy groups without reclassifying them.
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
  focused internal experiment while M0 is being completed.
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

### M5. Route approved generic public operations through API, CLI, and PyO3

Owner: Rust API and binding adapters.

- Add `document.compact_group.place.v1` and
  `document.compact_group.materialize.v1` to the single
  `ferrum-operation-request-v1` schema and generic operation dispatcher only
  after M0 has retired public bypasses.
- Add local CLI aliases that delegate to that dispatcher and accept one complete
  request envelope from a path or standard input.
- Use existing `execute_operation_v1` as the sole stateless PyO3 surface and
  the existing live operation receipt for applied resident-session mutations.
- Bound requests and responses with the shared protocol admission budget. Keep
  diagnostic text redacted; clients consume stable category/recovery facts.

Exit evidence: schema round trip, generic protocol/CLI/PyO3 equivalence for
applied and closed outcomes, exact fence behavior, and no parallel CLI or
Python chemistry engine.

### M6. Add the usable Qt compact-group workflow

Owner: Qt interaction layer.

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

This vertical slice is complete when M0 has made complete-render admission an
unbypassable public mutation prerequisite; all nine closed catalog entries are
usable through the canonical generic operations, local CLI aliases, generic
PyO3 route, and accessible Qt flow; groups are durable visible/selectable
document objects; materialization and deletion are atomic; refusal semantics
are shared and bounded; public semantic E2E proof is green; and the documented
local build and full validation suite pass. Broader group grammar, legacy
compatibility, publishing, and installation remain separate future work.
