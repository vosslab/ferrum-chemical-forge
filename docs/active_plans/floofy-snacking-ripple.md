# Plan: Ferrum, the CDML Chemical Forge -- replace the Python OASA backend with Rust over a project-owned RDKit adapter

> Historical draft only. This plan is superseded by
> [ferrum-plan-v3.md](ferrum-plan-v3.md); its paths, commands, and status are not
> current implementation instructions.

## Context

OASA is the chemistry backend for this repository: 58 Python modules, roughly
24,000 source lines under `packages/oasa/oasa/`. It owns molecular graph
topology, coordinate generation, chemistry perception, format codecs, geometry,
render-operation generation, Haworth layout, and sugar-code handling. Two
frontends consume it: the legacy Tk `packages/bkchem-app/` (43 files, which
subclass OASA graph classes and reach into `_neighbors` and `_vertices`) and the
PySide6 `packages/bkchem-qt.app/`, which consumes OASA as a function-and-data
library.

Three forces motivate the replacement. The chemistry authority already lives
outside Python, since OASA delegates coordinate generation, kekulization, InChI,
and molfile/SDF/SMARTS/SMILES conversion to RDKit (C++). The graph backend already
moved to Rust once: `packages/oasa/oasa/graph/rx_backend.py` is a 521-line Python
mirror in front of `rustworkx`, which is itself Rust. And the frontend boundary is
the real constraint, with `docs/CDML_BACKEND_TO_FRONTEND_CONTRACT.md` section 6
listing 13 tight-coupling violations where frontend classes inherit from or mutate
OASA internals.

An M0 discovery phase ran to completion before this plan was sized. All nine
investigation tracks produced verdicts from executed experiments rather than
reasoning, and several overturned assumptions the earlier draft treated as
settled. The most consequential: RDKit MinimalLib is **not** the right native
dependency, and the backend owns the whole persistent CDML document rather than
only its molecular subset. This plan is the revision those results produced.

## Project principles

Decision-making tools for whoever runs this, including a manager who joins after
the original one leaves. When a choice is not covered by a milestone, these
resolve it.

- **Chemistry semantics outrank implementation fidelity.** Reproducing RDKit's
  chemical results matters more than reproducing OASA's internal structure. Where
  they conflict, keep the chemistry and change the structure.
- **Evidence outranks assumption.** A design claim that can be measured is
  measured before it is built on. M0 exists because several plausible assumptions
  turned out false.
- **Ferrum is frontend-agnostic.** It serves the Qt app today and any frontend
  later. A capability that only makes sense for one frontend belongs in that
  frontend.
- **Compatibility exists for users, not for internal details.** Saved documents
  and published formats stay compatible. Internal representations stay free to
  change.
- **Replace design uncertainty with an experiment before adding complexity.** A
  bounded spike costs less than an abstraction built to hedge an unknown.
- **Every decision leaves an artifact.** A conclusion that lives only in someone's
  memory is treated as not yet made.

## Orientation for a new manager

This plan is written to be executable without its author.

**Read these first, in this order.** Roughly two hours to working context:

1. This plan's "Project principles" and "Settled decisions" -- the architecture
   and why it is what it is.
2. `docs/CDML_BACKEND_TO_FRONTEND_CONTRACT.md` and `docs/QT_CONTRACT.md` -- the
   boundaries as they exist today. Read them knowing this plan revises both at
   M18; the drafts live in `docs/active_plans/decisions/`.
3. This plan's "M0 evidence summary" -- what was measured, and the five findings
   that change implementation.
4. `docs/REPO_STYLE.md`, `docs/PYTHON_STYLE.md`, `docs/PYTEST_STYLE.md`, and
   `docs/E2E_TESTS.md` -- the conventions every patch is reviewed against.
5. This plan's "Milestone plan" from the phase your work sits in.
6. `packages/oasa/tests/reports/` -- the report format every milestone produces,
   and the precedent from the earlier `rustworkx` migration.

Everything needed to run the project lives in the repository:

- **Why the architecture is what it is**: "Settled decisions" states each choice
  with the measurement behind it, and each becomes a record under
  `docs/active_plans/decisions/` in M1.
- **What was already measured**: "M0 evidence summary" holds the verdicts, and M1
  commits the underlying fixtures and reports as reusable assets.
- **What must not regress**: "Findings that became behavioral requirements" maps
  each to a permanent test and an owning milestone.
- **What each milestone is for**: every milestone states its purpose, its
  dependencies, the artifact it leaves behind, and its exit condition.
- **What is still open**: "Open questions and decisions needed" carries an owner
  and a milestone for every item, so nothing is merely pending.

Where this plan says a decision was made, the reason is stated with it. A
replacement manager should never need to reconstruct intent from commit history.

## Objectives

- Replace all Python OASA capability with Rust that passes differential parity
  against frozen Python OASA on chemistry, coordinates, codecs, CDML, Haworth
  output, and render operations.
- Keep RDKit as the chemistry authority behind one project-owned adapter, so
  chemistry behavior is preserved exactly and the engine stays replaceable.
- Give the frontend a serialized boundary, so it holds backend objects nowhere and
  a future frontend needs no chemistry-layer rework.
- Make the backend the authoritative owner of the persistent CDML document, so
  every saved object survives a backend round trip.
- Ship a self-contained wheel whose runtime dependencies reduce to system
  libraries.

## Design philosophy

The central trade-off: accept a heavier build (a C++ toolchain and a pinned,
source-built RDKit) to keep chemistry behavior OASA already proved is the best
available. This is "Long-term over short-term" and "Fix the design, not the
symptom" from `docs/REPO_STYLE.md`. The symptom is slow Python glue; the design
flaw is that the frontend holds backend objects and the engine is reachable only
through a Python binding.

The rejected alternative is a pure-Rust chemistry core. `chematic` (v0.7.1, first
published 2026-05-27) describes its own depiction as "not publication-quality",
its pure-Rust InChI as "approximate", and its canonical SMILES as unstable on
about 5.5% of stereo-bearing molecules. OASA adopted RDKit because internal
implementations lost to it. `chematic` leaves this plan.

Three rules govern implementation, each earned during M0.

- **Port depiction utilities, delegate chemistry perception.** A gap becomes a
  Rust port when it is self-contained and arithmetic-only, as
  `RDDepict::straightenDepiction()` is. A gap touching molecule modeling, ring
  perception, valence, aromaticity, or canonical ranking gets an exported entry
  point into RDKit's implementation, as `MolOps::Kekulize()` does.
- **Survey the ecosystem for infrastructure, write the chemistry.** Molecule
  layout, bond styling, Haworth projection, and sugar naming are this project's
  work. XML trees, text metrics, record handling, and path math come from
  maintained libraries.
- **Match the Python wrapper's defaults, not the C++ defaults.** M0 found
  `canonOrient` defaults `True` in `AllChem.Compute2DCoords` and `false` in C++
  `RDDepict::compute2DCoords`. Every adapter entry point states which default it
  reproduces.

- Evidence strategy for uncertain methods: uncertain choices resolve against the
  frozen Python oracle on a fixed corpus, using the report pattern the repository
  already used for the `rustworkx` swap (`packages/oasa/tests/reports/`).

## Scope

- Build a Cargo workspace `ferrum` implementing OASA's capability set above a
  project-owned RDKit adapter.
- Own the complete persistent CDML document in the backend, with typed or opaque
  handling assigned per object class.
- Provide a versioned serialized boundary: CDML for persistent document state, a
  request/response protocol for chemistry operations.
- Build a differential parity harness against frozen Python OASA, with a
  preservation gate that runs on every parity run.
- Port geometry, render operations, both render backends, and glyph metrics.
- Port the domain layers: Haworth, sugar code, peptide utilities, repair
  operations, hex grid, linear formula, known groups, substructure search data.
- Ship a Python transport for `packages/bkchem-qt.app/`, a headless CLI, and a
  self-contained wheel.
- Migrate the Qt frontend onto the serialized boundary.
- Prove the WebAssembly path with one compiled smoke target.

## Non-goals

- Build a browser frontend; the WASM work proves feasibility.
- Move the Tk `packages/bkchem-app/` frontend onto the Rust backend; it stays on
  frozen Python OASA under a separate plan.
- Reimplement chemistry RDKit already provides correctly.
- Change the CDML on-disk format.
- Port Python OASA's public class shapes; the serialized boundary replaces them.

## Settled decisions

### The backend owns the authoritative CDML document

The backend owns the complete persistent CDML document. The frontend communicates
persistent changes exclusively through CDML and owns transient interaction state
and Qt projections. Once the backend accepts a CDML update, its document is the
source of truth.

`docs/QT_CONTRACT.md` and `docs/CDML_BACKEND_TO_FRONTEND_CONTRACT.md` currently
record the opposite. Both accurately describe the system that exists, and several
of their boundaries exist because Python OASA cannot represent an arrow, a
bracket, a group pseudo-vertex, or an unrecognized XML node. Those contracts are
revised to match this decision.

**The persistence invariant**, which decides the architecture on its own:

> If the backend rewrites CDML but only understands molecules, every
> non-molecular object is lost unless Qt restores it afterward. At that point Qt,
> not the backend, remains the authoritative document owner.

**The rule that follows:**

> The backend preserves every persistent CDML object, whether typed or opaque. Qt
> owns the live projection, and persistent document content survives a backend
> round trip without frontend reconstruction.

This is testable, so it is a gate on every parity run.

| Data | Owner |
| --- | --- |
| Complete CDML document, object order, identifiers, references | Backend |
| Molecules, chemical properties, coordinates as stored in CDML | Backend |
| Arrows, text, plus signs, brackets, vector graphics, reactions, groups | Backend |
| Paper and header data, external data, unknown XML and attributes | Backend |
| File parsing, validation, preservation, serialization | Backend |
| `QGraphicsItem` instances and scene stacking projection | Frontend |
| Selection, focus, hover, active handles, in-progress gestures | Frontend |
| Zoom, viewport, grid visibility, other view-only state | Frontend |
| Dialogs, menus, tools, Qt object lifetime, worker delivery | Frontend |
| Pixel conversion, and edits until they reach the backend as CDML | Frontend |

**Two channels, one authority.** CDML carries persistent document state both ways.
A separate versioned request/response protocol carries chemistry operations that
are not document mutations: coordinate generation, format conversion,
substructure queries, render-op generation, reference data. They stay distinct so
the operation protocol avoids becoming a second, competing representation of the
document.

**Whole-document exchange per committed command was chosen over a fragment
protocol** because measurement showed a fragment protocol buys nothing inside the
expected operating envelope. Round-trip cost on the repository's real CDML files
is 0.34 to 1.83 ms, well within one measured frame interval on the test display;
that observation is not a universal frame deadline. The crossover
where whole-document exchange would exceed that budget sits near 300 to 600 KB,
roughly fifteen to thirty times larger than any document this project currently
produces. Whole-document exchange also keeps ownership unambiguous, which a
fragment protocol would blur. Revisit if real documents approach 300 KB.

**Document revision semantics.** Whole-document exchange needs stale-update
detection, or a slow worker finishing after a newer edit silently reverts the
document. The backend stamps every document with a monotonic revision. A CDML
update carries the revision it was derived from; the backend applies it when that
matches current and rejects it with a conflict result otherwise, leaving its own
document untouched. Backend edits are serialized per session, so a session applies
one update at a time and revisions advance without gaps.

This matters concretely because `docs/QT_CONTRACT.md` already describes
asynchronous import workers whose results can arrive after the document moved on;
its existing defence is a per-session request token. Revisions generalize that to
every persistent change. The frontend handles a conflict by re-deriving from the
backend's current document rather than retrying blindly, since its own copy is
known stale. M13 specifies the wire shape and the conflict result.

**Reactions** split into a persistent backend record and a Qt projection, because
a reaction carries semantic structure alongside presentation.

Undo stays on `Document.undo_stack` per `docs/QT_CONTRACT.md`. What a command
records to reverse an edit is settled in M13 against the interaction model.

### The chemistry boundary is a project-owned adapter

`ferrum` owns a narrow C ABI adapter linking directly to pinned RDKit C++.
MinimalLib informs API shape, ownership conventions, and WASM compatibility rather
than shipping as a dependency. M0 established both halves of this: no MinimalLib
CFFI library ships with a normal RDKit install, and the project needs its own
export for Kekulize regardless, so recreating MinimalLib underneath the adapter
would add a layer without solving an architectural problem.

The `ChemEngine` trait is defined by OASA's needs. It exposes an opaque molecule
handle, typed errors, and owned result types, so callers see no pointers, pickle
buffers, or lifetimes. Its structural output is one `MolGraph` struct of atoms,
bonds, orders, aromatic flags, charges, isotopes, and coordinates, which keeps
every RDKit representation an implementation detail.

### Distribution is bundled dynamic linking

**Decision: ship dynamically linked RDKit dylibs bundled beside the extension,
resolved through `@loader_path`.** Static linkage is recorded as an evaluated
alternative and set aside.

M0 proved the dynamic route end to end: a bundled artifact ran under a scrubbed
environment with external dependencies reduced to `/usr/lib/libc++.1.dylib` and
`/usr/lib/libSystem.B.dylib`. Static linkage was attempted and set aside because
macOS `ld` prefers a `.dylib` over a sibling `.a`, so it requires linking archives
by absolute path and resolving RDKit's full transitive closure by hand, for no
benefit the bundled route lacks. Bundling is also what `delocate` and `auditwheel`
already automate, so the packaging path uses standard tooling rather than bespoke
link scripting.

Revisit only if a target platform makes `@loader_path`-equivalent resolution
unavailable.

### The backend is named Ferrum, the CDML Chemical Forge

**Decision: the new Rust backend is named Ferrum. The rename covers the new
backend's public identity and nothing else.**

Why the name: *ferrum* is Latin for iron, giving the chemistry reference and the
Rust one, since rust is iron oxide. "The CDML Chemical Forge" states what it does
-- CDML is the document format it owns, and it forges chemistry rather than
merely storing it. A new name is justified because the architecture genuinely
changed: this is a Rust backend over a project-owned RDKit adapter that owns the
whole persistent document, not a port of the Python OASA library.

**Registry collisions, verified.** The bare name is taken in both registries the
project would publish to:

| Registry | `ferrum` status |
| --- | --- |
| crates.io | Taken. A web framework, last published 2018-02-08, ~71 recent downloads. Abandoned but occupying the name |
| PyPI | Taken. A package manager, v0.1, uploaded 2025-10-01, which itself exposes a `ferrum.forge()` call |

Neither blocks the project identity, which is not a registry entry. They constrain
published artifact names only.

**Three distinct concepts, easy to conflate months later.** Names are settled
here rather than left to discovery:

| Concept | Value | Constraint | Status |
| --- | --- | --- | --- |
| Project identity | Ferrum, the CDML Chemical Forge | None; not a registry entry | Decided |
| Repository name | `bkchem-oasa` | Unchanged | Decided |
| Rust workspace | `ferrum` | Local directory name only | Decided |
| Rust crate names | `ferrum-core`, `ferrum-chem`, `ferrum-geom`, `ferrum-render`, `ferrum-cdml`, `ferrum-domain`, `ferrum-api` | Unique on crates.io only if published | Decided; `publish = false` by default |
| Python distribution name | `ferrum-chem` | Globally unique on PyPI | Decided; verified free |
| Python import name | `ferrum` | Unique only within an environment | Decided |
| CLI binary | `ferrum` | Unique only within a `PATH` | Decided |

Two consequences worth stating. The workspace crates carry `publish = false`
unless a specific crate is deliberately released, so the occupied crates.io
`ferrum` name constrains nothing today; a crate proposed for publication gets its
name verified at that point. And because a Python distribution name and its import
name are independent, `pip install ferrum-chem` providing `import ferrum` is
normal. The residual risk is that a user who also installs the unrelated PyPI
`ferrum` collides on `import ferrum`; that is accepted rather than mitigated,
since the two serve unrelated audiences.

**Rename scope, deliberately narrow.** Renaming buys clarity; churn costs
history, links, and review noise. So the rename covers:

| Renamed | Unchanged, and why |
| --- | --- |
| Cargo workspace and crates (`ferrum-*`) | Repository name `bkchem-oasa`: renaming breaks remotes, CI, and cross-document links for no functional gain |
| CLI binary (`ferrum`) | `packages/oasa/`, the frozen Python oracle: it is being retired, so renaming it is pure churn |
| Python distribution and import name | BKChem, the frontend: unchanged by this work |
| New docs (`docs/FERRUM_ARCHITECTURE.md`, `docs/FERRUM_API_CONTRACT.md`) | CDML, the format name: it is in the tagline and stays as-is |
| `README.md` first paragraph, which `docs/REPO_STYLE.md` designates as the GitHub About source | Existing `docs/` filenames not specific to the new backend |

**The retained historical names are deliberate, not overlooked.** A future
contributor may read `bkchem-oasa` and `packages/oasa/` as leftovers and try to
tidy them. They stay for stated reasons: the repository name is referenced by
remotes, CI configuration, and cross-document links throughout `docs/`, and
renaming it buys nothing functional; `packages/oasa/` holds the frozen Python
oracle that every differential run compares against until M18 retires it, so
renaming the thing being deleted is churn that also invalidates the parity
reports. Revisit both at M18, when the oracle is archived.

M1 owns executing this rename.

### Decision records

Each settled decision above becomes a durable record under
`docs/active_plans/decisions/` during M1, so later milestones cite the decision
instead of rediscovering it. At minimum: the chemistry-boundary pivot (project
owned C ABI over pinned RDKit C++, with MinimalLib as an API and WASM reference
rather than a shipped native dependency), whole-document backend ownership with
the persistence invariant, the bundled-dynamic-linking distribution model, and the
Ferrum naming decision with its registry findings.

**Every milestone leaves an artifact.** A milestone that only finishes work leaves
the next manager nothing. Each one below names what it deposits in the repository:

| Artifact kind | Produced by |
| --- | --- |
| Decision record | M1, and any milestone that resolves an open question |
| Differential parity report | every milestone, under `packages/oasa/tests/reports/` |
| Regression test for a behavioral requirement | M4, M16, M17 |
| Reference fixture or corpus | M1, M9, M11 |
| Interface specification | M4 (`ChemEngine`), M13 (frozen contract and schema) |
| Benchmark with its baseline | M3, M13, M16 |
| Compatibility or platform matrix | M16 |
| Migration note | M15, M18 |

### Implementation constraints

Design rules that keep the architecture from eroding as code accumulates.

- **`ChemEngine` stays small.** Its purpose is isolating RDKit, and a trait that
  grows toward RDKit's full surface stops being an abstraction and becomes a
  second copy of it. The trait covers what OASA's call sites need; anything
  expressible by composing existing methods above the trait belongs above it, as
  `straighten_depiction` and SDF record splitting already do. A patch adding a
  method states why composition was insufficient, and `architect` reviews trait
  growth at M13's freeze.
- **The chemistry boundary is absolute.** `ferrum-chem` is the only crate that
  links or calls RDKit. Other crates reach chemistry through `ChemEngine`.
- **Render operations stay purely declarative.** An op describes what to draw, and
  it carries no layout decisions. Layout belongs to the code that produces ops, so
  that render parity compares data rather than two renderers' interpretations.
  This is what makes render-op comparison deterministic and keeps renderer
  differences from becoming a permanent debugging surface.
- **Domain utilities stay separate modules.** Sugar code, peptide utilities,
  repair operations, linear formula, and known groups share the M12 milestone for
  scheduling only. They are unrelated to each other and keep separate modules,
  separate corpora, and separate reports.
- **Everything before M13 is unstable.** The `ChemEngine` surface introduced in M4
  and consumed from M5 onward is provisional, and breaking changes are expected
  until M13 freezes it. Reviewers treat pre-M13 signatures as working drafts.

Two hazards that surface during implementation rather than design:

- **Two RDKit copies in one process.** The differential harness runs frozen Python
  OASA, which imports the pip `rdkit`, against `ferrum`, which bundles its own
  RDKit dylibs. Loading both into one process risks duplicate-symbol resolution
  and mismatched global state. The harness therefore runs each side in a separate
  process and compares serialized results, which M1 establishes.
- **Thread affinity is decided, not discovered.** RDKit is built with
  `RDK_BUILD_THREADSAFE_SSS=ON`, and sessions are thread-confined: a session is
  used from one thread at a time, and the backend serializes updates per session.
  `docs/QT_CONTRACT.md` already requires Qt object creation on the GUI thread and
  routes worker results back through a relay, so the Rust side matches that shape
  rather than introducing a second concurrency model. M15 documents which calls
  are safe off the GUI thread.

### Interface stability

Each boundary states its owner, what callers may rely on, and what stays free to
change, so a later manager can refactor without guessing.

| Boundary | Owner | Stable | Free to change |
| --- | --- | --- | --- |
| `ChemEngine` trait | WS-B | Method behavior, error semantics, `MolGraph` shape, coordinate tolerance; frozen in M13 | Which RDKit calls implement it, handle representation, adapter internals |
| C ABI adapter | WS-B | Nothing outside `ferrum-chem`; it is an internal boundary | Entire surface, provided `ChemEngine` behavior holds |
| CDML document channel | WS-F | The CDML format itself, and that whole documents round-trip losslessly | Internal document model, typed-versus-opaque assignment per class |
| Operation protocol | WS-F | Versioned request and response schemas; unknown versions are rejected explicitly | Transport encoding, batching, session internals |
| Render operations | WS-C | The serialized op shapes the frontend consumes | Geometry internals, backend implementations |
| `ferrum-core` model | WS-A | Used only inside the workspace | Freely, no external contract |
| Python API (PyO3 surface) | WS-F | Function names, argument shapes, exception types, and the `import ferrum` name; `.pyi` stubs are generated from it | Which Rust crate implements a call, internal handle types |
| CLI (`ferrum`) | WS-F | Subcommand names, flags, exit codes, stdin and stdout contracts; settled in M14 | Output formatting beyond the documented contract, internal dispatch |
| Generated outputs (SVG, PNG, PDF) | WS-C | The formats themselves and their documented rendering options | Emission internals, and which library produces them |
| Chemistry file formats (SMILES, molblock, SDF, InChI, CML, CDXML) | WS-B, WS-D | Conformance to each published format specification | Which RDKit call or codec path produces them |
| Codec registry | WS-B | Registration and lookup as an extension point, since `codec_registry.register_codec` is how new formats are added today | Registry internals, and the default registration mechanism |
| Reference data (periodic table, known groups, amino acids) | WS-B | The data values themselves as chemistry facts | On-disk format, load mechanism, whether they are compiled in |
| Configuration | WS-F | That behavior is configured explicitly through CLI flags or API arguments, per `docs/PYTHON_STYLE.md`, which rules out invented environment variables | Any internal defaults not part of a documented contract |

## Relationship to the CDML backend authority migration

A separate, already-active effort
(`docs/active_plans/active/cdml_backend_authority_migration_2026-07-27.md`) is
moving persistent authority into the **Python** OASA backend through
`oasa.cdml_document.CDMLDocumentSession`, and migrating the Qt frontend onto that
boundary one action family at a time. Arrow Mode is the first accepted slice.

**That work establishes the semantic contract; this plan substitutes the
implementation behind it.** Ferrum is not obliged to imitate the Python
implementation internally. It is obliged to satisfy the same contract: Qt's
ownership model, the serialized CDML boundary, atomic transaction semantics,
revision behavior, and typed failure categories.

The consequence for sequencing is substantial and reduces this plan's scope:

- Qt stops depending on OASA graph internals **before** Ferrum arrives, so the
  frontend rewrite is not part of this plan.
- The boundary is exercised in production against a working backend before the
  language changes, so its design is validated rather than assumed.
- The Rust migration becomes a backend implementation substitution behind an
  already-exercised contract, rather than a simultaneous frontend and backend
  rewrite.

**Semantics are fixed; incidental choices are not.** Ferrum preserves the
contract's semantic guarantees and typed failure categories. It is free to choose
differently on history capacity, internal storage representation, canonicalization
performance, and transport encoding, because those are implementation choices
rather than contract terms.

**What substitution still requires**, so the Rust integration is not prematurely
treated as free. Ferrum must be verified against the Python reference on:

| Aspect | What to verify |
| --- | --- |
| Canonicalization | Identical canonical CDML for identical input |
| ID allocation | Same durable-ID semantics, same provisional-token rewriting scope, including that matching strings inside opaque XML stay unrewritten |
| Accepted-candidate finality | An accepted candidate is consumed; recovery reprojects from the snapshot and never resubmits the candidate, so no path can double-create an object |
| Opaque XML preservation | Byte-identical retention of unrecognized subtrees |
| Revision behavior | Monotonicity, conflict detection, restore creating new revisions |
| Saved-baseline behavior | Content-based clean/dirty, correct after history eviction removes the saved revision |
| Snapshot reprojection | An exact-snapshot rebuild succeeds without a recommit |
| Recovery Export | Writes an exact snapshot while leaving path, baseline, dirty state, revision, and history unchanged |
| Session state model | Every contract-defined state and permitted transition behaves identically |
| Error mapping | Every typed failure category maps one-to-one |
| Performance | Equal or better at the measured document sizes |
| Transport representation | Native and WASM both satisfy the contract |

This list is the acceptance content of the milestone that swaps the backend.

**Normative definitions live in the contracts, not here.** Transaction semantics,
provisional-token consumption, saved-baseline independence, and Recovery Export
behavior belong in `docs/CDML_BACKEND_TO_FRONTEND_CONTRACT.md`; the frontend
session states, their permitted operations and transitions, Save versus Recovery
Export eligibility, projection-failure recovery, and the prohibition on candidate
resubmission belong in `docs/QT_CONTRACT.md`. This plan carries the work and the
validation, and links to those sections rather than restating them, so milestone
identifiers and temporary sequencing stay out of the durable architecture.

## The document model

Everything in this project inherits the shape of this model, so it is specified
here rather than discovered in M9. If it is clean, the rest follows; if it is
messy, every layer above it carries that mess.

**One authoritative representation.** The document is a typed tree held in memory.
XML is a serialization of that tree and never an intermediate step within a
command. A command that reads, mutates, and writes goes
`XML -> tree -> mutate -> XML` exactly once. Round-tripping through XML in the
middle of an operation is what accumulates formatting drift, so the model forbids
it.

```
Document
  revision: u64                     monotonic, drives conflict detection
  paper: PaperModel                 typed, with an unknown-attribute bag
  header: HeaderModel               typed, with an unknown-attribute bag
  objects: Vec<DocObject>           canonical top-level order, preserved
  id_index: Map<Id, usize>          references resolve through this

DocObject
  id: Id                            stable across load, mutate, save
  source_order: usize               original position, preserved on rewrite
  payload: Typed(..) | Opaque(RawXml)

Typed payloads
  Molecule | Reaction | Arrow | Text | PlusSign | Bracket
  | VectorGraphic | Group
  each carrying its recognized fields plus an unknown-attribute bag
```

Four rules make round trips lossless.

1. **Typed nodes also keep what they do not recognize.** Every typed payload
   carries an unknown-attribute bag and, where the element has children, an
   unrecognized-child list. A recognized element with one unfamiliar attribute
   stays typed and still round-trips exactly.
2. **Opaque nodes retain their subtree verbatim**, including namespace prefixes
   and formatting, and re-emit unchanged.
3. **Order and identity are data.** Canonical top-level order and stable
   identifiers are part of the model, because
   `docs/QT_CONTRACT.md` already makes `Document.objects` order-significant and
   the frontend's object-stack actions reorder it.
4. **References are by identifier**, resolved through `id_index` on load and
   re-emitted as identifiers. A reference into an opaque node stays textual and is
   never rewritten.

**Typed-versus-opaque policy**, so the choice is a rule rather than a per-element
judgement:

| Case | Handling |
| --- | --- |
| The backend validates, reorders, references, transforms, or canonicalizes it | Typed |
| Molecules and reactions | Typed, always; chemistry operates on them |
| Arrows, text, plus signs, brackets, vector graphics, groups | Typed; the frontend edits them, so they need stable fields and identifiers |
| Elements the backend only stores and re-emits | Opaque |
| Unrecognized elements | Opaque, permanently |
| A future CDML element | Opaque on arrival; promoted to typed when a milestone needs to operate on it |

Promotion from opaque to typed is a normal, additive change. Demotion does not
occur: once an element is typed, its fields are part of the round-trip contract.
M9 records the assignment for every class present in CDML today.

**Why this matters for M9's size.** M9 is large because owning persistence is the
actual hard problem here, not because it is poorly split. Splitting it further
would create artificial milestones that each deliver a partial document model, and
a partial document model cannot pass the preservation gate.

## M0 evidence summary

Nine tracks, all with verdicts, measured on macOS arm64 with RDKit 2026.03.4,
Python 3.12, and Rust 1.97.1.

| Track | Verdict |
| --- | --- |
| D1 chemistry API | MinimalLib exposes `details_json` on nine functions and covers OASA's parse and write options. `set_2d_coords` accepts no options. `/FixedH` is reachable, and it produces a non-standard InChI that OASA uses by default |
| D2 native binding | A project-owned C ABI adapter over RDKit C++ reached **exact** coordinate parity, `0.000e+00` across five molecules |
| D3 kekulize | **No** parse or write route reproduces `Kekulize(clearAromaticFlags=False)`; delegation through the adapter reaches it on benzene, naphthalene, and pyrrole |
| D4 straighten port | Portable. The `minimizeRotation=true` branch matches at ~5e-16; the `false` branch, which OASA uses, needs the verbatim C++ source |
| D5 distribution | A pinned source build yields a self-contained relocatable bundle running under `env -i` |
| D6 WASM | One project-level contract spans both platforms; a **project-built** WASM MinimalLib is required |
| D7 CDML channel | Whole-document exchange, measured at 0.34 to 1.83 ms on real files; compare against the target display's measured frame interval |
| D8 parity rules | Comparison rule assigned per output class, several measured |
| D9 dependencies | Adopt `xot` and `cairo-rs`; decline `sdfrust` and a Rust text stack; adopt `petgraph` plus a project cycle basis |

### Findings that became behavioral requirements

Four M0 findings are requirements rather than history, so each carries a permanent
automated test that keeps a future refactor from reintroducing it. These are
listed here and owned by the milestone named in each row.

| Requirement | Test | Home | Owner |
| --- | --- | --- | --- |
| Adapter entry points reproduce the Python wrapper's defaults, starting with `canonOrient=true` | Layout an asymmetric molecule through the adapter and assert equality with the oracle; benzene alone passes either way, so the case uses octane or caffeine | Rust unit test in `ferrum-chem` plus a corpus case | M4 |
| The built artifact resolves its libraries with no environment variables | Run the packaged artifact under a scrubbed environment and assert one chemistry call succeeds; assert the rpath list contains `@loader_path` and no absolute build path | `tests/e2e/` runner | M16 |
| The build succeeds against both vendored and external dependency naming | Configure against a source-built prefix (`RDKitInchi`, `RDKitcoordgen`) and a system prefix (`inchi`, `coordgen`), asserting both link | CI matrix job | M4 |
| Native and WASM implementations agree on the frozen contract | Run the same request set through both implementations and assert identical project-level results | `tests/e2e/` runner | M17 |

Two supporting requirements follow from the same evidence: the C++ standard stays
at C++20 or later while RDKit headers use `constexpr virtual`, and every
coordinate parity case includes at least one asymmetric molecule.

### Findings that change implementation

1. **`canonOrient` diverges between RDKit's Python and C++ APIs.** With the C++
   default, four of five molecules moved 3 to 11 units while symmetric benzene
   still matched at `2e-16`. Every adapter entry point states which Python default
   it reproduces, and every parity case includes an asymmetric molecule.
2. **Runtime library discovery is solved at link time.** A dynamically linked
   binary failed with an unresolved `@rpath`, and `DYLD_LIBRARY_PATH` cannot fix it
   because macOS strips `DYLD_*`. `@loader_path` rpaths are the mechanism.
3. **Dependency naming differs by build style.** A system RDKit links external
   `libinchi` and `libcoordgen`; a source build vendors them as `RDKitInchi` and
   `RDKitcoordgen`. The build detects rather than assumes. Boost is external in
   both and is declared separately.
4. **RDKit 2026.03.4 headers require C++20** (`constexpr virtual` in
   `Geometry/point.h`).
5. **`straighten_depiction` exists in the WASM wrapper and not in
   `cffiwrapper.h`.** The native path uses the D4 Rust port; the browser path uses
   the built-in. WASM MinimalLib is 2025.03.4 against the native 2026.03.4, so
   pinning covers both targets.

## Current state summary

| Area | Modules | Lines | External coupling |
| --- | --- | --- | --- |
| Haworth renderer and layout | 9 | ~4,260 | cairo (indirect) |
| Render ops, render_lib, outputs | 14 | ~4,180 | pycairo, XML |
| Chemistry and RDKit glue | 7 | ~2,990 | rdkit |
| Graph backend | 7 | ~1,520 | rustworkx |
| Geometry and transforms | 5 | ~1,690 | none |
| CDML, CML, CDXML, XML plumbing | 8 | ~1,600 | defusedxml |
| Sugar code and names | 4 | ~1,060 | none |
| Reference data | 4 | ~600 | pyyaml |
| Remaining utilities | tail | ~2,100 | mixed |

`packages/oasa/oasa/render_ops.py` already exposes `ops_to_json_dict` and
`ops_to_json_text`, so a serialized render boundary has working precedent. The
coupling to remove is Qt's construction of `oasa.atom_lib.Atom`,
`oasa.bond_lib.Bond`, and `oasa.molecule_lib.Molecule`, plus its calls into
private helpers (`render_ops._text_segments`, `_segment_font_size`,
`_segment_baseline_state`, `_baseline_offset_em`,
`codec_registry._ensure_defaults_registered`).

Existing assets to reuse: 104 test files under `packages/`, 19 under `tests/`,
parity tests (`test_graph_parity.py`, `test_renderer_pipeline_parity.py`,
`test_cdml_roundtrip_oasa.py`), `benchmark_graph_algorithms.py`, reference outputs
per `docs/REFERENCE_OUTPUTS.md`, and four real CDML files under
`packages/bkchem-app/bkchem_data/templates/`.

## Architecture boundaries and ownership

```
  PySide6 Qt frontend (Python)          future browser frontend (JS/TS)
   scene items, selection, gestures,     same transient-state role
   viewport, dialogs, Qt lifetime
            |                                        |
            |  CDML document channel (authoritative) |
            |  + versioned operation protocol        |
            v                                        v
  +--------------------------------------------------------------+
  |                    ferrum (Rust)                            |
  |  persistent CDML document, molecular model, geometry,        |
  |  render ops, Haworth, sugar code, codecs, reference data     |
  +--------------------------------------------------------------+
            |                    ChemEngine trait
            v                                        v
  project-owned C ABI adapter          project-built MinimalLib WASM
            |                                        |
            v                                        v
                 RDKit (C++ engine, pinned, source-built)
```

`ferrum-chem` is the one crate that links RDKit. Every chemistry call routes through
the `ChemEngine` trait, so a gap becomes a trait method with two implementations.

### Mapping (milestones / workstreams -> components / patches)

| Milestone | Component | Review boundary |
| --- | --- | --- |
| M1 | `ferrum/harness/`, CI | Harness and gate only |
| M2, M3 | `ferrum-core` | Crate boundary |
| M4, M5 | `ferrum-chem`, `ferrum/native/` | Sole RDKit linker |
| M6 | `ferrum-geom` | Crate boundary |
| M7, M8 | `ferrum-render` | Crate boundary |
| M9, M10 | `ferrum-cdml` | Crate boundary |
| M11, M12 | `ferrum-domain` | Crate boundary |
| M13, M14 | `ferrum-api`, `ferrum/bindings/` | API contract review |
| M15 | `packages/bkchem-qt.app/` | Needs `docs/QT_CONTRACT.md` review |
| M16, M17, M18 | packaging, WASM, cutover | Release gate |

## Milestone plan

Sized from M0 evidence. Milestones that combine independently risky systems are
split, following "Atomic task decomposition" in `docs/REPO_STYLE.md`; milestones
whose risk M0 retired stay whole.

The size column reflects M0 measurement, not estimate-by-feel. Milestones whose
risk M0 retired are marked *reduced*; whole-document ownership made M9 *grown*.

| Phase | M | Title | Goal | Size |
| --- | --- | --- | --- | --- |
| A Foundation | M1 | Workspace, harness, gate, decision records | Parity measurable | medium |
| | M2 | Core model | Atoms, bonds, molecules, ids, errors | small |
| | M3 | Graph algorithms and deterministic cycles | Graph parity green | medium |
| B Chemistry | M4 | RDKit adapter and `ChemEngine` | Exact coordinate parity in tree | medium, *reduced* -- M0 built and measured this spike |
| | M5 | Chemistry codecs | SMILES, molblock, SDF, SMARTS, InChI parity | medium, *reduced* -- SDF and property handling come from RDKit itself |
| C Geometry and render | M6 | Geometry and straighten port | Geometry parity green | medium |
| | M7 | Render operations and glyph metrics | Render-op parity green | large, *reduced* -- Cairo parity removes the metrics risk |
| | M8 | Render backends | Cairo and SVG output parity | medium |
| D Documents | M9 | Full-document CDML | Preservation gate green | **large, grown** -- the backend now owns arrows, text, brackets, reactions, groups, paper state, and unknown XML |
| | M10 | Foreign XML codecs | CML, CDXML, CDSVG | medium |
| E Domain | M11 | Haworth | Reference output parity | large, irreducible domain logic |
| | M12 | Domain utilities | Sugar code, peptide, repair, formula | medium |
| F Delivery | M13 | Contract implementation, protocol, freeze | Contract satisfied in Rust | large |
| | M14 | PyO3 module and CLI | Callable from Python and shell | medium |
| | M15 | Backend substitution | Qt runs on Ferrum, behavior unchanged | medium, *reduced* -- the Qt migration belongs to the CDML backend authority plan |
| | M16 | Packaging and platform matrix | Wheel installs clean | medium, *reduced* -- mechanism proven on macOS arm64 |
| | M17 | WASM proof | Contract validated on both platforms | small |
| | M18 | Cutover | Python OASA retired from the Qt path | small |

Sequencing consequence: **M9 and M13 are the critical path.** M9 builds the
document model everything above it inherits, and M13 satisfies an externally
defined contract whose semantics are fixed. Start M9 as early as its dependencies
allow (M2 and M6).

M15 dropped from large to medium because the CDML backend authority migration
moves Qt onto the contract first. That reduction is real but conditional: it holds
only while that migration completes each persistent action family. Check its
status before dispatching M15, since an unmigrated family arrives as frontend work
this plan did not budget.

Every milestone exits when its differential report shows no unresolved divergence
under the D8 comparison rules and the preservation gate passes.

### Phase A: foundation

**M1 workspace, oracle harness, preservation gate, decision records.** Depends on
nothing. Delivers the Cargo workspace, CI building pinned RDKit from source, the
frozen Python OASA tag, the corpus, the differential runner, and the
preservation-gate fixture carrying every CDML object class.

Three additional deliverables come from M0 rather than from new work.

- **Ingest the M0 artifacts as version-controlled reference assets**, so later
  milestones reuse the measurements instead of regenerating them: the coordinate
  oracle for the five reference molecules, the CDML preservation fixture, the
  cairo text-extents reference (78 samples), the CDML timing baseline, and the
  comparison reports. These become engineering evidence rather than spike output.
- **Write the decision records** listed under "Decision records".
- **Apply the rename** recorded under "The backend is named Ferrum", across the
  workspace, crates, CLI, Python distribution and import names, new docs, and the
  `README.md` first paragraph.

Exits when the runner reproduces a deliberately injected divergence, the gate
catches a deliberately dropped object, and the decision records and naming
position are published. Parallel-plan ready: no, one owner defines the report
format first.

**M2 core model.** Depends on M1. Delivers `ferrum-core` atoms, bonds, molecules,
stable identifiers, and the error hierarchy replacing `oasa_exceptions.py`.

Artifact: a model specification recording which chemistry fields the model
carries, which are computed rather than stored, and the identifier stability
guarantee, plus `proptest` round-trip properties over construction and mutation.
Exits when every molecule in the corpus loads into the model and its per-atom and
per-bond fields -- symbol, charge, isotope, multiplicity, valency, bond order,
aromatic flag -- match the oracle exactly. This gives M2 an independently
reviewable outcome rather than an assertion that the code exists.

Decision criterion for this milestone: *chemistry semantics outrank implementation
fidelity*. Where OASA's Python model carries a field for internal bookkeeping
rather than chemical meaning, the Rust model may drop it, provided the corpus
comparison above still passes. Reproducing OASA's structure is explicitly not a
goal. Parallel-plan ready: yes.

**M3 graph algorithms and deterministic cycles.** Depends on M2. Adopts
`petgraph` for `bridges`, `articulation_points`, `matching::maximum_matching`,
`connected_components`, `dijkstra`, `floyd_warshall`, and `has_path_connecting`,
and adds a project cycle basis over a spanning tree plus fundamental cycles. Exits
when graph parity is green and cycle selection is deterministic, which improves on
`rx_backend.py` line 190 where the current `rustworkx` path varies by run.
Parallel-plan ready: yes.

### Phase B: chemistry

**M4 RDKit adapter and `ChemEngine`.** Depends on M2. Delivers the C ABI adapter,
its build script, the native `ChemEngine` implementation, and the pinned
source-build recipe. Carries the M0 build facts: C++20, detected dependency
naming, Boost declared separately, `@loader_path` rpaths, and a stated Python
default per entry point. Exits when coordinate parity is exact on the corpus and
kekulize reaches the target state. M0 proved each of these in a spike, so this
milestone productionizes rather than discovers. Parallel-plan ready: yes.

**M5 chemistry codecs.** Depends on M4. Delivers SMILES, SMARTS, molblock V2000
and V3000, SDF, and InChI through the adapter, reaching RDKit's own
`SDMolSupplier` and `SDWriter` so property ordering and escaping match. InChI
asserts the `InChI=1/` prefix when non-standard output is requested. Exits on
codec parity under the D8 rules. Parallel-plan ready: yes.

### Phase C: geometry and render

**M6 geometry and straighten port.** Depends on M2. Delivers `ferrum-geom` over
`kurbo` and `nalgebra`, covering `geometry.py`, `low_level_geometry.py`,
`wedge_geometry.py`, transforms, and `hex_grid.py`, plus the
`straightenDepiction` port from verbatim
`Code/GraphMol/Depictor/RDDepictor.cpp`. The port reports rotation angle per
molecule alongside coordinates and covers both `minimizeRotation` branches, since
M0 matched the `true` branch and missed the `false` branch that OASA uses.
Establishes one primary geometry representation and a conversion policy so
`kurbo` and `nalgebra` avoid adding conversion code. Parallel-plan ready: yes.

**M7 render operations and glyph metrics.** Depends on M6. Delivers the render-op
model, `render_lib` equivalents, and label geometry over `cairo-rs`, which M0
measured as exactly reproducing `pycairo` `text_extents` across 78 samples. Exits
when `ops_to_json_dict` output matches the oracle after rounding.
Parallel-plan ready: yes.

**M8 render backends.** Depends on M7. Delivers Cairo raster and PDF output and
SVG emission through `xot`. Kept separate from M7 so geometry errors stay
distinguishable from renderer errors. Parallel-plan ready: yes.

### Phase D: documents

**M9 full-document CDML.** Depends on M2, M6. Implements "The document model":
the typed tree, the unknown-attribute bags on typed payloads, opaque retention of
unrecognized nodes, canonical order, stable identifiers, identifier-resolved
references, and the document revision counter. Uses `xot`, which M0 measured
preserving every object class including unknown elements and attributes.

Artifacts: the typed-versus-opaque assignment for every class present in CDML
today, and the preservation gate green on the full corpus. Exits when a document
carrying every object class round-trips with order, identifiers, unknown
attributes, and unrecognized subtrees intact.

This is the largest milestone in the plan and deliberately unsplit: every layer
above inherits this model's shape, and a partial document model cannot pass the
preservation gate. Parallel-plan ready: yes.

**M10 foreign XML codecs.** Depends on M9. Delivers CML, CML2, CDXML, and CDSVG.
Separated from M9 because CDXML carries format-specific presentation semantics
whose supported subset needs identifying on its own. Parallel-plan ready: yes.

### Phase E: domain

**M11 Haworth.** Depends on M5, M8. Delivers spec, layout, fragment layout, and
renderer, roughly 4,260 lines of irreducible domain logic. Exits against the
reference SVG and PNG in `docs/REFERENCE_OUTPUTS.md`. Stands alone because its
size and risk are unrelated to the other domain utilities. Parallel-plan ready:
yes.

**M12 domain utilities.** Depends on M5. Delivers sugar code and SMILES
conversion, peptide utilities, repair operations, linear formula, known groups,
and substructure search data. Artifact: one differential report per utility, each
against its own corpus, so a failure names the utility rather than the milestone.
Exits when every utility's report is clean under its D8 rule. Parallel-plan ready:
yes.

### Phase F: delivery

**M13 contract implementation, operation protocol, and `ChemEngine` freeze.**
Depends on M9. Implements the CDML authority contract in Rust rather than
designing it: `docs/CDML_BACKEND_TO_FRONTEND_CONTRACT.md` already defines the
session operations, transaction atomicity, revision and saved-baseline behavior,
provisional-token consumption, and typed failure categories, with the Python
`oasa.cdml_document.CDMLDocumentSession` as the working reference. Ferrum
provides an implementation satisfying that contract.

Also delivers the versioned operation protocol for chemistry work that is not a
document mutation, with a schema generated from Rust types.

Validation specific to this milestone, drawn from the substitution table above:
an accepted candidate is never replayable; exact-snapshot reprojection succeeds
without a recommit; the saved baseline still drives clean and dirty after history
eviction removes the saved revision; Recovery Export writes an exact snapshot
while leaving path, baseline, dirty state, revision, and history unchanged; and
every contract-defined session state and transition behaves identically to the
reference.

Also **freezes the `ChemEngine` compatibility contract**, so the native and WASM
implementations evolve independently without drifting. The frozen contract states,
per method: observable behavior, handle ownership and lifetime, error semantics
and which conditions are recoverable, coordinate fidelity and its tolerance, and
the serialization shape of `MolGraph`. M0 established that one contract can span
both platforms; freezing it is what keeps that true as each side changes.
Parallel-plan ready: yes.

Also **freezes the `ChemEngine` compatibility contract**, so the native and WASM
implementations evolve independently without drifting. The frozen contract states,
per method: observable behavior, handle ownership and lifetime, error semantics
and which conditions are recoverable, coordinate fidelity and its tolerance, and
the serialization shape of `MolGraph`. M0 established that one contract can span
both platforms; freezing it is what keeps that true as each side changes.
Parallel-plan ready: yes.

**M14 PyO3 module and CLI.** Depends on M13. Delivers the Python extension
(distribution `ferrum-chem`, import `ferrum`) and the headless `ferrum` binary.
Artifacts: generated `.pyi` stubs, and a CLI contract document fixing subcommand
names, flags, exit codes, and stdin and stdout behavior, which is what the
interface stability table promises callers. Also decides the relationship to
`packages/oasa/oasa_cli.py` and `packages/oasa/chemical_convert.py`. Exits when
`tests/e2e/` runs CLI round trips against the contract. Parallel-plan ready: yes.

**M15 backend substitution.** Depends on M14, M10, M11, M12, and on the CDML
backend authority migration having moved Qt onto the contract.

Scope reduced substantially. The frontend rewrite belongs to the other plan: by
the time this milestone runs, Qt already holds no backend graph objects, already
sends persistent changes as complete CDML, and already rebuilds projections from
canonical responses. This milestone points Qt at the Rust implementation instead
of the Python one and confirms nothing observable changed.

Delivers the dependency swap, thread-affinity confirmation for PyO3 alongside Qt
workers, RDKit, and Cairo, and a behavioral comparison run. Exits when Qt imports
`oasa` nowhere, `pytest packages/bkchem-qt.app/tests/` passes unchanged against
the Rust backend, behavior matches
`docs/active_plans/audits/BKCHEM_QT_ACTION_PARITY_2026-07-27.md`, and the full
substitution table verifies clean.

Risk note: this milestone is only small if the other plan's migration actually
completes for every persistent action family. Any family still on the
transitional Qt-local route arrives here as unmigrated frontend work, so its
status is checked before this milestone is dispatched rather than discovered
during it. Parallel-plan ready: no, the frontend stays runnable throughout.

**M16 packaging and platform matrix.** Depends on M14. Delivers the wheel with
bundled dylibs, a clean-environment install test, the Python floor reconciled
against `requires-python = ">=3.10"` before naming an `abi3` target, and a named
build and validation route per platform the project builds for. M0 proved the
mechanism on macOS arm64; every other platform is unproven and needs its own
build-and-run evidence. Artifact: the platform matrix recording, per platform,
whether the bundle builds, installs into a clean environment, and answers one
chemistry request. Parallel-plan ready: yes.

**M17 WASM proof.** Depends on M4, M13. Delivers a project-built MinimalLib WASM
carrying the project's exports, and validates it against the frozen `ChemEngine`
contract rather than against an ad-hoc request: the same request set runs through
both implementations and must produce identical project-level results. Separated
from cutover so a negative result costs nothing.

Decision criterion for this milestone: *Ferrum is frontend-agnostic*. A capability
needed only by a browser frontend belongs to that frontend, not to Ferrum. If
satisfying the frozen contract on WASM would require a browser-shaped concept in
the trait, record the divergence instead and leave the trait alone.
Parallel-plan ready: yes.

**M18 cutover.** Depends on M15, M16. Retires Python OASA from the Qt dependency
set, lands the revised `docs/CDML_BACKEND_TO_FRONTEND_CONTRACT.md` and
`docs/QT_CONTRACT.md`, and confirms interactive latency inside the measured
budget. Artifacts: a migration note covering what changed for users and how to
open pre-cutover documents, the archived Python oracle labelled as historical
reference, and the decisions this milestone is the landing place for -- whether
Ferrum becomes its own repository, and whether the retained `bkchem-oasa` and
`packages/oasa/` names change now that the oracle is archived. Exits when the
cutover gate passes. Parallel-plan ready: no.

## Workstream breakdown

| WS | Goal | Owner | Provides | Review boundary |
| --- | --- | --- | --- | --- |
| WS-A | Model and graph with deterministic behavior | `coder` | `ferrum-core` | `ferrum/crates/ferrum-core/` |
| WS-B | Chemistry adapter and codecs | `expert_coder` | `ChemEngine` | `ferrum-chem/`, `native/`, build scripts |
| WS-C | Geometry and render backends | `coder` | Render ops | `ferrum-geom/`, `ferrum-render/` |
| WS-D | Full-document CDML and XML formats | `coder` | Document model | `ferrum-cdml/` |
| WS-E | Haworth and domain utilities | `expert_coder` | Domain capability | `ferrum-domain/` |
| WS-F | Serialized boundary and Qt migration | `expert_coder` | Frontend contract | `ferrum-api/`, `bindings/`, Qt app |
| WS-G | Parity oracle, benchmarks, packaging, cutover | `tester` | The gate each milestone exits through | `harness/`, CI, `tests/reports/` |

## Work packages

Work packages are written when their milestone is dispatched, so they reflect
current evidence. Three are specified now because M0 settled their shape.

### Work package: WP-B-adapter

- Owner: `expert_coder`
- Touch points: `ferrum/native/adapter.cpp`, `ferrum/crates/ferrum-chem/`
- Depends on: M2
- Acceptance criteria: the adapter exposes the `ChemEngine` surface over pinned
  RDKit C++; each entry point states which Python-wrapper default it reproduces,
  beginning with `canonOrient=true`; C++ exceptions convert to typed errors at the
  ABI; allocation and free are paired and explicit; the molecule handle stays
  opaque; the build detects vendored versus external `inchi` and `coordgen`,
  compiles as C++20, declares Boost separately, and sets `@loader_path` rpaths;
  coordinate parity is exact on the corpus.
- Evidence or review: `architect` confirms the adapter stays a delegation layer.
- Obvious follow-ons: M17 reuses the same contract for the WASM implementation.

### Work package: WP-B-kekulize

- Owner: `expert_coder`
- Touch points: the adapter plus `ferrum-chem`
- Depends on: WP-B-adapter
- Acceptance criteria: a narrow entry point delegates to
  `RDKit::MolOps::Kekulize()` with `clear_aromatic_flags`, `canonical`, and
  `max_backtracks`; bond orders and retained aromatic flags match
  `rdkit.Chem.Kekulize(mol, clearAromaticFlags=False)` across aromatic, fused,
  charged, and query-atom fixtures; RDKit's implementation remains the single
  source of the algorithm.
- Evidence or review: `architect` confirms the shim stays a delegation as it
  evolves.
- Obvious follow-ons: the same export is compiled into the M17 WASM build.

### Work package: WP-D-preservation gate

- Owner: `tester`
- Touch points: `ferrum/harness/`
- Depends on: M1
- Acceptance criteria: the fixture carries every CDML object class; the gate runs
  on every parity run; a failure names the object class lost or altered; the
  verdict rests on backend output alone, so frontend recoverability leaves it
  unchanged.
- Evidence or review: demonstrate the gate catching a deliberately dropped object.
- Obvious follow-ons: the gate becomes a contract term in the revised
  `docs/CDML_BACKEND_TO_FRONTEND_CONTRACT.md`.

## Cargo dependency inventory

### Required

| Crate | Role | Notes |
| --- | --- | --- |
| Project-owned C ABI adapter | Chemistry boundary | M0 chose this over `rdkit-sys` and the `rdkit` crate, both last published 2024-10-14, neither exposing 2D coordinates, InChI, molblock writing, or kekulization |
| `cc` | Build dependency | Compiles the adapter; requires C++20 |
| `petgraph` | Graph algorithms | Supplies everything except a cycle basis, which is project code |
| `xot` | XML tree | Adopted on measurement: preserves every CDML object class, 1.7 to 3.0 times the Python layer's speed |
| `cairo-rs` | Raster, PDF, glyph metrics | Adopted on measurement: reproduces `pycairo` `text_extents` exactly, 78 of 78 samples |
| `kurbo`, `nalgebra` | Geometry | Affine transforms, Bezier paths, offsets, bounding boxes, intersections, 3D transforms |
| `serde`, `serde_json`, `rmp-serde` | Serialization | Boundary and codecs |
| `schemars` | Schema generation | Generates the operation-protocol schema from Rust types |
| `thiserror` | Library errors | Replaces `oasa_exceptions.py` |
| `clap`, `tracing`, `tracing-subscriber` | CLI and diagnostics | |
| `pyo3`, `maturin` | Python transport | Version target set in M16 |

### Declined, with reasons

| Crate | Reason |
| --- | --- |
| `chematic` | Self-described approximate InChI, non-publication-quality depiction, unstable canonical SMILES |
| `rdkit-sys`, `rdkit` | Stale and missing the operations OASA needs |
| `sdfrust` | The adapter reaches RDKit's own `SDMolSupplier` and `SDWriter` |
| `svg`, `svgdom`, `esvg` | `xot` already covers emission; one XML dependency is preferable |
| `parley`, `cosmic-text`, `rustybuzz` | Cairo parity removed the reason on desktop. Reconsidered in M17, which is the first context where Cairo is unavailable |

### Conditional

`tiny-skia` and `resvg` for WASM raster, `wasm-bindgen` and
`serde-wasm-bindgen` for M17, `serde_norway` or `saphyr` for YAML reference data
since `serde_yaml` was deprecated 2024-03-25, and `criterion`, `insta`,
`proptest`, `approx`, `assert_cmd`, `pretty_assertions` for testing.

Sizing, offered as a planning figure rather than a success measure: roughly 24,000
Python lines map to an order of 14,000 to 16,000 Rust lines, with large reductions
in the graph backend, RDKit glue, XML plumbing, and geometry primitives, and with
Haworth (~4,260 lines) and sugar code (~1,060 lines) porting near 1:1. Backend
ownership of the full CDML document raises the M9 figure above the earlier
estimate. Measure success by parity and delivery.

## Acceptance criteria and gates

- Per-patch gate: `cargo clippy -- -D warnings` and `cargo fmt --check` pass; new
  behavior carries a Rust unit test; the touched capability's differential report
  shows no new divergence.
- Preservation gate: the CDML invariant fixture round-trips intact on every parity
  run.
- Integration gate: each milestone's differential report lands under
  `packages/oasa/tests/reports/`, with every divergence resolved or recorded as an
  accepted difference carrying its tolerance and named decision owner.
- Independent review gate: `reviewer` audits M5, M8, and M9 without having
  implemented them. M15 and M18 need `architect` sign-off against the revised
  ownership contracts.
- Cutover gate: Qt imports `oasa` nowhere; the Qt and repo-wide pytest suites
  pass; interactive latency sits inside the measured budget; wheels build for the
  M16 platform list.

## Test and verification strategy

Testing centers on the frozen Python oracle, with placement by tier.

- Fast pytest under `tests/` and `packages/` stays offline and deterministic, uses
  inline inputs and `tmp_path`, and finishes well under one second, per
  `docs/PYTEST_STYLE.md`. It covers focused logic: parsers, transformations, error
  behavior, round-trip invariants.
- E2E runners under `tests/e2e/` carry the slow work, per `docs/E2E_TESTS.md`:
  full-corpus differential sweeps, real RDKit execution, wheel builds, CLI round
  trips, and image comparison.
- Rust unit tests sit beside the code; `insta` snapshots cover render-op batches,
  CDML output, and Haworth output.
- Comparison follows the D8 rules:

| Output class | Rule |
| --- | --- |
| 2D coordinates | Exact within a pinned RDKit version; geometric invariants across versions |
| Bond orders and aromatic flags | Exact |
| Text and glyph metrics | Exact |
| InChI and InChIKey | Exact string, with the `InChI=1/` prefix asserted for non-standard output |
| CDML and SVG | Structural equivalence |
| Canonical SMILES and SMARTS | Exact within a pinned version; semantic round trip across versions |
| Molblock and SDF | Semantic equivalence, since headers carry program and timestamp lines |
| Raster output | Perceptual threshold with a named algorithm and pinned font environment |
| Render ops | Exact after the existing rounding in `ops_to_json_dict` |

- The durable corpus is shared test infrastructure and needs explicit user
  sign-off before it is committed, per the fixture policy in
  `docs/PYTEST_STYLE.md`. It lives in a documented data or E2E location, records
  its pinned RDKit, Cairo, font, and locale environment, and includes adversarial
  cases: malformed files, very large structures, unusual valence, query atoms,
  disconnected records, zero-bond molecules, extreme labels. Every coordinate case
  includes an asymmetric molecule, since M0 showed symmetric benzene masking a
  real divergence.
- Fixtures are classified as required historical compatibility, known defect,
  implementation accident, or intended change before the Rust implementation is
  shaped around them, with a named accepted-difference owner.
- Benchmarks use `criterion` against
  `packages/oasa/tests/benchmark_graph_algorithms.py` as the Python baseline.

Targeted runs use the documented form:

```bash
pytest packages/oasa/tests/test_graph_parity.py
pytest tests/test_markdown_links.py
```

## Migration and compatibility policy

- Python OASA freezes at the M1 tag as the oracle. A required production fix
  during the port lands in the frozen package, is mirrored into `ferrum`, and is
  recorded in the divergence report, so the oracle stays versioned rather than
  untouchable.
- CDML on disk stays unchanged. Files written before cutover open after it.
- The serialized boundary is versioned from its first commit; the backend rejects
  an unknown version explicitly.
Release policy, rollback procedure, Tk coexistence, and dependency licensing are
out of scope for this plan.

## Risk register

M0 narrowed several risks rather than eliminating whole categories. Each entry
below states exactly what the evidence covered, so a later manager knows what
remains untested.

| Risk | Status | Impact | Trigger | Owner | Mitigation |
| --- | --- | --- | --- | --- | --- |
| Coordinate generation is unreachable from Rust | **Narrowed by D2** | -- | -- | -- | Exact parity measured on five molecules with one RDKit version. Chemistry parity beyond coordinates and kekulization stays unproven until M5 |
| Kekulize needs reimplementation | **Closed by D3** | -- | -- | -- | Delegation to `MolOps::Kekulize` measured reaching the target state |
| RDKit cannot be built and bundled relocatably | **Narrowed by D5** | -- | -- | -- | Proven on macOS arm64 only; other platforms remain untested |
| Cairo metrics cannot be reproduced | **Narrowed by D9** | -- | -- | -- | Exact across 78 samples in the measured font environment; a different font stack or Cairo version is untested |
| A non-molecular object is dropped on round trip | Live | High: documents lose arrows or annotations, surfacing after a save | The fixture omits a class, or the gate is skipped | WS-D | The gate runs on every parity run and treats backend output alone as the verdict |
| A Python-versus-C++ default divergence changes output | Live | High: every depiction shifts while symmetric molecules still pass | An entry point is written without checking the Python default | WS-B | Each entry point states its reproduced default; every coordinate case includes an asymmetric molecule |
| Parity rules encode current defects as specification | Live | High: the port reproduces bugs | Fixtures are classified as required-compatibility by default | WS-G | Four-way classification with a named owner, done before implementation |
| The Qt document codec rework destabilizes the frontend | Live | High: M15 is the largest single change | Rework begins before M13's contract is stable | WS-F | M15 depends on M14; the frontend stays runnable throughout; `architect` sign-off |
| Coordinates drift across RDKit versions | Live | Medium: parity reports lose signal | A version bump lands mid-port | WS-G | Pin RDKit for oracle, native build, and WASM; compare invariants across versions |
| Browser and native diverge | Live | Medium: a future frontend cannot reach parity | The WASM target is built from the stock package | WS-B | M17 builds a project WASM carrying the project exports; pinning covers 2025.03.4 and 2026.03.4 |
| Contract revisions destabilize in-flight Qt work | Live | Medium | Revisions land before cutover | WS-F | Hold revisions as drafts until M18 |

## Completion checklist

Technical completion, distinct from any release process.

- [ ] RDKit version pinned for oracle, native build, and WASM build.
- [ ] Milestone differential reports committed under
      `packages/oasa/tests/reports/`.
- [ ] Accepted-difference list published, each with tolerance, reason, and owner.
- [ ] Preservation gate green.
- [ ] Behavioral-requirement regression tests present and passing.
- [ ] `docs/INSTALL.md` documents the Rust toolchain, C++20 compiler, and RDKit
      build prerequisites.
- [ ] Revised ownership contracts landed.
- [ ] Frozen Python OASA archived with a note stating it is the historical oracle.

## Documentation close-out requirements

- Active plan: file at
  `docs/active_plans/active/oasa_rust_backend_replacement.md`, keeping the
  milestone status table current. On filing, convert the backticked repository
  paths into relative Markdown links per `docs/MARKDOWN_STYLE.md`.
- `docs/CHANGELOG.md`: entries as work lands, in the dated subsections
  `docs/REPO_STYLE.md` defines. Findings that overturn an assumption belong under
  "Decisions and Failures". Agents write the entries; the user makes the commits.
- M0 evidence: file the discovery reports under
  `docs/active_plans/audits/` and `docs/active_plans/decisions/` in lowercase
  snake_case, so the measurements stay citable. Version-control the reusable
  assets alongside them per M1: the coordinate oracle, the CDML preservation
  fixture, the cairo text-extents reference, and the CDML timing baseline. These
  are project engineering evidence, and later milestones cite rather than
  regenerate them. The durable corpus needs the user sign-off described in
  `docs/PYTEST_STYLE.md` before it is committed.
- Contract revisions: `docs/CDML_BACKEND_TO_FRONTEND_CONTRACT.md` and
  `docs/QT_CONTRACT.md` change under whole-document backend ownership. Draft in
  `docs/active_plans/decisions/` and land at M18. The revisions restate the
  ownership tables along the data-versus-UI split, replace the
  bridge-and-inheritance model in contract sections 5 through 8, update where
  `docs/QT_CONTRACT.md` names the Qt `Document` the persistent-change authority,
  and state the preservation invariant as a contract term.
- New durable docs in SCREAMING_SNAKE_CASE: `docs/RUST_BACKEND_ARCHITECTURE.md`
  and `docs/OASA_API_CONTRACT.md`, the latter generated from the schema.
- Updated: `docs/CODE_ARCHITECTURE.md`, `docs/FILE_STRUCTURE.md`,
  `docs/INSTALL.md`, `docs/USAGE.md`,
  `docs/OASA_MOLECULE_COORDINATE_GENERATION_METHODS.md`,
  `docs/RELATED_PROJECTS.md`, `docs/ROADMAP.md`.

## Open questions and decisions needed

M0 closed the questions that gated architecture. These remain, each decided
before its dependent workstream dispatches rather than during M18.

- Supported platforms and release model. Owner: `maintainer`, in M16. M0 proved
  the mechanism on macOS arm64 only.
- CLI scope against `packages/oasa/oasa_cli.py` and
  `packages/oasa/chemical_convert.py`. Owner: `planner`, in M14.
- Tk coexistence: dependency resolution, and whether Rust-written CDML stays
  readable by the Tk frontend. Owner: `architect`, before WS-F.
- Undo representation: inverse command, snapshot, or serialized delta. Owner:
  `expert_coder`, in M13 against the interaction model.
- Thread affinity across PyO3, Qt workers, RDKit, and Cairo. Owner:
  `expert_coder`, in M15.
- Whether Ferrum becomes its own repository. Owner: `architect`, decided at M18.
  Keeping it in-tree during the port keeps the oracle adjacent for differential
  runs; that reason expires once Python OASA is retired, which is what makes M18
  the decision point rather than an open-ended "someday".

Every item above carries an owner and a milestone. An item with neither is a
defect in this plan, not a deferral.
