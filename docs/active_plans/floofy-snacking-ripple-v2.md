# Plan: Ferrum, the CDML Chemical Forge

> Historical draft only. This plan is superseded by
> [ferrum-plan-v3.md](ferrum-plan-v3.md); its paths, commands, and status are not
> current implementation instructions.

## Context

Ferrum is a new CDML-compatible chemical drawing platform with no production-code
overlap with the historical BKChem/OASA implementation. It acknowledges BKChem as
its historical predecessor and retains CDML compatibility, while using
independently developed code throughout.

It ships as two components:

| Component | Role | License |
| --- | --- | --- |
| **Ferrum-Chem** | Reusable chemistry, document, rendering, and format backend | LGPL v3 |
| **Ferrum-Qt** | PySide6 graphical application built on Ferrum-Chem | AGPL v3 |

Ferrum-Qt is independently developed, owned source being renamed and advanced in a
new repository. It is not BKChem code being ported, though it deliberately
preserves CDML compatibility and acknowledges the BKChem lineage. The historical
BKChem/OASA codebase serves as a behavioral and provenance oracle during
development and contributes no production source code.

This distinction is load-bearing throughout the plan. Ferrum preserves **project
heritage** -- CDML as the document format, the behavior users expect, attribution,
and the acknowledgement that Ferrum descends conceptually from BKChem. Ferrum
drops **implementation inheritance**. Compatibility comes from specification and
testing rather than from copied code.

Three things make this the right moment. Ferrum-Qt's source was written
independently and is owned outright, so it can form the frontend without
inheriting historical architecture. The document-authority contracts have
already landed, defining backend-owned persistence, atomic sessions, revisions,
and projection replacement, so the boundary is specified rather than speculative.
And an M0 discovery phase ran to completion, replacing several plausible
assumptions with measurements.

The work happens in a new repository. Nothing is migrated in place.

## Project principles

Decision-making tools for whoever runs this, including a manager who joins later.
When a choice is not covered by a milestone, these resolve it.

- **Chemistry semantics outrank implementation fidelity.** Reproducing correct
  chemical results matters more than reproducing any prior code structure.
- **Evidence outranks assumption.** A design claim that can be measured is
  measured before it is built on.
- **Ferrum-Chem is frontend-agnostic.** It serves Ferrum-Qt today and may serve
  other frontends later. Frontend-specific capability belongs in the consuming
  application. Ferrum-Qt is a frontend by definition and this principle does not
  apply to it.
- **Compatibility exists for users, not for internal details.** CDML documents and
  published formats stay compatible. Internal representations stay free to change.
- **Heritage is preserved; implementation inheritance is not.** Behavior may be
  deliberately compatible. Code is not copied to achieve it.
- **Replace design uncertainty with an experiment before adding complexity.**
- **Every decision leaves an artifact.** A conclusion that lives only in someone's
  memory is treated as not yet made.

## Orientation for a new manager

**Read these first, in this order.**

1. This plan's "Project principles", "Settled decisions", and "The document
   model" -- the architecture and why it is what it is.
2. The Ferrum document and desktop contracts -- what the system guarantees.
3. This plan's "M0 evidence summary" -- what was measured, and the findings that
   change implementation.
4. The repository style, Python style, pytest, and E2E conventions -- what every
   patch is reviewed against.
5. This plan's "Milestone plan", from the phase your work sits in.
6. `docs/provenance/` -- what came from where, and under which terms.

Everything needed to run the project lives in the repository. Where this plan
records a decision, the reason is stated with it.

## Objectives

- Deliver a Rust chemistry and document engine that owns the authoritative CDML
  document and satisfies the landed contracts.
- Deliver Ferrum-Qt on Ferrum namespaces, carrying no production dependency on the
  historical implementation.
- Preserve CDML compatibility so documents users already created continue to open.
- Keep RDKit as the chemistry authority behind one project-owned adapter.
- Ship a self-contained distribution requiring no separately installed chemistry
  runtime beyond supported system libraries.
- Record provenance and licensing precisely enough that redistribution
  obligations are known rather than assumed.

## Design philosophy

The central trade-off: accept a heavier build -- a C++ toolchain and a pinned,
source-built RDKit -- to keep chemistry behavior that measurement showed is the
best available. The symptom motivating this work was slow Python glue; the design
flaw was a frontend holding backend objects and an engine reachable only through
a Python binding.

The rejected alternative is a pure-Rust chemistry core. `chematic` describes its
own depiction as "not publication-quality", its pure-Rust InChI as "approximate",
and its canonical SMILES as unstable on a meaningful fraction of stereo-bearing
molecules. The historical project adopted RDKit because internal implementations
lost to it, so `chematic` leaves this plan.

The exact version, publication date, and quoted figures are recorded in the M0
dependency survey artifact rather than restated here. That matters because M0
found the crates.io registry and the project repository reporting different
current versions on the same day, so the artifact retains the precise source
consulted rather than a number this document would freeze incorrectly.

Four rules govern implementation, each earned during M0.

- **Port depiction utilities, delegate chemistry perception.** A gap becomes a
  Rust port when it is self-contained and arithmetic-only. A gap touching molecule
  modeling, ring perception, valence, aromaticity, or canonical ranking gets an
  exported entry point into RDKit's implementation.
- **Survey the ecosystem for infrastructure, write the chemistry.** Molecule
  layout, bond styling, Haworth projection, and sugar naming are this project's
  work. XML trees, text metrics, record handling, and path math come from
  maintained libraries.
- **Match the reference wrapper's defaults, not the C++ defaults.** M0 found
  `canonOrient` defaults `True` in `AllChem.Compute2DCoords` and `false` in C++
  `RDDepict::compute2DCoords`. Every adapter entry point states which default it
  reproduces.
- **Judge inclusion by fitness, not resemblance.** Code is excluded for its origin
  or its unsupported purpose, never because a pattern resembles something the
  historical project used.

- Evidence strategy for uncertain methods: uncertain choices resolve against the
  historical implementation running as an external oracle, on a fixed corpus,
  producing a divergence report per capability.

## Scope

- Create the Ferrum repository, the Ferrum-Chem Rust workspace, and Ferrum-Qt
  under Ferrum namespaces.
- Implement the landed document-authority contracts in Rust.
- Own the complete persistent CDML document, with typed or opaque handling
  assigned per object class.
- Build a differential harness that invokes the historical implementation
  externally, plus a preservation gate that runs on every parity run.
- Implement chemistry over a project-owned RDKit adapter, plus geometry, render
  operations, both render backends, and the domain layers.
- Ship a Python distribution, a headless CLI, and a self-contained wheel.
- Record provenance, attribution, and licensing for every included asset.
- Prove the WebAssembly path with one compiled target.

## Non-goals

- Copy production source from the historical implementation.
- Support, package, or keep operational any Tk frontend. Tk code and resources
  are historical evidence only.
- Build a browser frontend; the WASM work proves feasibility.
- Reimplement chemistry RDKit already provides correctly.
- Change the CDML on-disk format.
- Redesign abstractions in the owned PySide6 application merely because the
  historical project contained something similar.

## Settled decisions

### Ferrum is a new implementation sharing CDML

Ferrum shares the CDML document format and selected user-visible semantics with
the historical BKChem/OASA project, and contains no production code derived from
it. Behavioral compatibility is achieved by specification and differential
testing.

**Licensing is settled.** Because no production code derives from the historical
Python implementation, its GPL-2.0-only terms do not attach. Ferrum-Chem is
**LGPL v3**, chosen so the backend stays reusable by other frontends and
downstream tools. Ferrum-Qt is **AGPL v3**.

The combination is sound: LGPL v3 is GPL v3 plus additional permissions, and
AGPL v3 is GPL v3 compatible, so an AGPL application may link an LGPL library.
Bundled RDKit is BSD-3, permissive and compatible with both.

| Asset | Origin | Obligation |
| --- | --- | --- |
| `straightenDepiction` port | Translated from RDKit C++ (`RDDepictor.cpp`) | BSD-3 attribution; recorded as a derived algorithm |
| Bundled RDKit binaries | Redistributed unmodified | BSD-3 notice, plus notices for its own bundled dependencies |
| Ferrum-Qt source | Independently written, owned outright | AGPL v3 |
| Ferrum-Chem source | Independently written | LGPL v3 |
| Fixtures and reference outputs | Generated by the historical implementation | Treated as generated test data; provenance and redistribution status recorded per corpus rather than by one blanket conclusion |

**LGPL v3 shapes how Ferrum-Chem is packaged**, which is an architectural
consequence rather than a release detail. LGPL v3 section 4 requires a recipient
be able to relink the combined work against a modified library, and it offers two
routes: a suitable shared-library mechanism, or distributing the application
material needed to relink. Static linking is therefore permitted with obligations
rather than prohibited.

Statically linking Ferrum-Chem into the extension would require distributing the
corresponding application material needed to relink against a modified library. To
make compliance and downstream replacement straightforward, Ferrum-Chem instead
ships as a **separately replaceable shared library**, alongside the RDKit dylibs
already bundled that way. Platforms where that mechanism is impractical must
provide and verify the object-code relinking route. M20 verifies whichever route a
platform uses rather than assuming one.

This section records the project's **compliance design**, not legal advice.

M1 produces `docs/provenance/` recording, per included asset, its origin, why it
is present, and its terms, plus the two `LICENSE` files and per-component notices.

### Inclusion is judged by fitness, not resemblance

Three categories, so the rule is mechanical.

**Excluded by origin or purpose:** Tk implementation and resources; historical
Python chemistry backend source; historical branding and user-facing identity;
compatibility shims existing only to support historical imports; packaging and
entry points for retired products; tests asserting historical implementation
details rather than useful Ferrum behavior.

**Evaluated, never prohibited:** class shapes; mutation APIs; serializers;
authority boundaries; command-line entry points; module and package boundaries;
document models; operation names; undo representation. Each is an architectural
decision judged on whether it serves Ferrum's contracts and maintainability.

**Preserved deliberately:** the owned PySide6 code; established user-visible
behavior; CDML compatibility; chemistry and rendering semantics; well-designed
abstractions already present in the desktop application; tests expressing valid
product requirements; fixtures and reference outputs with documented provenance.

For each candidate component: Is it owned and suitable for reuse? Does its
behavior belong in Ferrum? Does its design fit the contracts? Is retaining it
better than rewriting? Can it be renamed and detached from retired dependencies
without a shim? Do its tests describe useful behavior? A component passing these
belongs in the repository even where its form resembles something historical.

### The backend owns the authoritative CDML document

The backend owns the complete persistent CDML document. The frontend communicates
persistent changes exclusively through CDML and owns transient interaction state
and projections. Once the backend accepts a CDML update, its document is the
source of truth.

**The persistence invariant**, which decides the architecture on its own:

> If the backend rewrites CDML but only understands molecules, every
> non-molecular object is lost unless the frontend restores it afterward. At that
> point the frontend, not the backend, remains the authoritative document owner.

**The rule that follows:** the backend preserves every persistent CDML object,
whether typed or opaque; the frontend owns the live projection, and persistent
content survives a backend round trip without frontend reconstruction. This is
testable, so it is a gate on every parity run.

| Data | Owner |
| --- | --- |
| Complete CDML document, object order, identifiers, references | Backend |
| Molecules, chemical properties, coordinates as stored in CDML | Backend |
| Arrows, text, plus signs, brackets, vector graphics, reactions, groups | Backend |
| Paper and header data, external data, unknown XML and attributes | Backend |
| File parsing, validation, preservation, serialization | Backend |
| Scene items, selection, focus, hover, handles, gesture previews | Frontend |
| Zoom, viewport, grid visibility, other view-only state | Frontend |
| Dialogs, menus, tools, widget lifetime, worker delivery | Frontend |
| Pixel conversion, and edits until they reach the backend as CDML | Frontend |

**Two channels, one authority.** CDML carries persistent document state both ways.
A separate versioned request/response protocol carries chemistry operations that
are not document mutations. They stay distinct so the operation protocol avoids
becoming a competing representation of the document.

**Whole-document exchange per committed command** was chosen over a fragment
protocol because measurement showed a fragment protocol buys nothing inside the
expected envelope: round-trip cost on real CDML files is 0.34 to 1.83 ms against a
16 ms interactive budget, and the crossover sits near 300 to 600 KB, fifteen to
thirty times larger than documents this project produces. Revisit if real
documents approach 300 KB.

The contracts define transaction semantics normatively: accepted commits are
final and atomic; provisional tokens are consumed by acceptance; recovery after
acceptance is snapshot reprojection only and never candidate resubmission; the
saved canonical baseline is independent of bounded history; Recovery Export writes
an exact snapshot without changing session state; history capacity and performance
limits are implementation choices while semantic behavior is fixed. This plan
implements and validates those rules rather than restating them.

### The chemistry boundary is a project-owned adapter

Ferrum owns a narrow C ABI adapter linking directly to pinned RDKit C++.
MinimalLib informs API shape, ownership conventions, and WASM compatibility rather
than shipping as a dependency. M0 established both halves: no MinimalLib CFFI
library ships with a normal RDKit install, and the project needs its own export
for kekulization regardless.

The `ChemEngine` trait is defined by Ferrum's needs. It exposes an opaque molecule
handle, typed errors, and owned result types, so callers see no pointers, pickle
buffers, or lifetimes. Its structural output is one `MolGraph` struct, which keeps
every RDKit representation an implementation detail.

### Distribution is bundled dynamic linking

Ship dynamically linked RDKit dylibs bundled beside the extension, resolved
through `@loader_path`. Static linkage was attempted and set aside: macOS `ld`
prefers a `.dylib` over a sibling `.a`, so it requires linking archives by
absolute path and resolving RDKit's full transitive closure by hand, for no
benefit. Bundling is what `delocate` and `auditwheel` already automate.

## The document model

Everything inherits the shape of this model, so it is specified here rather than
discovered during implementation.

**One authoritative representation.** The document is a typed tree held in memory.
XML is a serialization of that tree, never an intermediate step within a command.
A command that reads, mutates, and writes goes `XML -> tree -> mutate -> XML`
exactly once.

```
Document
  revision: u64                     monotonic, drives conflict detection
  saved_baseline: CanonicalCdml     pinned, independent of history eviction
  paper, header                     typed, each with an unknown-attribute bag
  objects: Vec<DocObject>           canonical order, preserved
  id_index: Map<Id, usize>          references resolve through this

DocObject
  id, source_order
  payload: Typed(..) | Opaque(..)

Typed payloads
  Molecule | Reaction | Arrow | Text | PlusSign | Bracket
  | VectorGraphic | Group
  each carrying recognized fields plus an unknown-attribute bag
```

Four rules make round trips lossless.

1. **Typed nodes also keep what they do not recognize**, through an
   unknown-attribute bag and an unrecognized-child list, so a recognized element
   with one unfamiliar attribute stays typed and still round-trips.
2. **Opaque nodes preserve their subtree.** The required fidelity is
   **structural**: elements, attributes, namespace identities, ordering, text, and
   values are preserved exactly. Byte-identical re-emission is **not** claimed,
   because M0 measured that tree-based parsing normalizes some lexical detail.
   Raw source-slice retention is the fallback if a case demands byte identity, and
   adopting it requires an experiment proving the behavior first.
3. **Order and identity are data.** Canonical order and stable identifiers are
   part of the model.
4. **References are by identifier**, resolved through `id_index` on load and
   re-emitted as identifiers. A reference inside an opaque node stays textual and
   is never rewritten.

**Typed-versus-opaque policy:**

| Case | Handling |
| --- | --- |
| The backend validates, reorders, references, transforms, or canonicalizes it | Typed |
| Molecules and reactions | Typed, always |
| Arrows, text, plus signs, brackets, vector graphics, groups | Typed; the frontend edits them |
| Elements the backend only stores and re-emits | Opaque |
| Unrecognized elements | Opaque, permanently |
| A future CDML element | Opaque on arrival; promoted when a milestone operates on it |

Promotion is additive; demotion does not occur.

**Canonical equivalence** means structural equivalence under a single stated
normalization that every implementation follows: attribute ordering, namespace
serialization, and insignificant whitespace are normalized before comparison.
Byte equality is not the standard.

## M0 evidence summary

Nine tracks with verdicts, measured on macOS arm64 with RDKit 2026.03.4, Python
3.12, Rust 1.97.1.

| Track | Verdict |
| --- | --- |
| D1 chemistry API | MinimalLib exposes `details_json` on nine functions and covers the needed parse and write options. `set_2d_coords` accepts none. `/FixedH` is reachable and produces a non-standard InChI |
| D2 native binding | A project-owned C ABI adapter over RDKit C++ reached **exact** coordinate parity, `0.000e+00` across five molecules |
| D3 kekulize | **No** parse or write route reproduces `Kekulize(clearAromaticFlags=False)`; delegation reaches it |
| D4 straighten port | Portable. `minimizeRotation=true` matches at ~5e-16; the `false` branch, which production uses, needs the verbatim C++ source |
| D5 distribution | A pinned source build yields a self-contained relocatable bundle running under `env -i` |
| D6 WASM | One project-level contract spans both platforms; a **project-built** WASM MinimalLib is required |
| D7 CDML channel | Whole-document exchange, 0.34 to 1.83 ms on real files against a 16 ms budget |
| D8 parity rules | Comparison rule assigned per output class |
| D9 dependencies | Adopt `xot` and `cairo-rs`; decline `sdfrust` and a Rust text stack; adopt `petgraph` plus a project cycle basis |

### Findings that became behavioral requirements

Each carries a permanent automated test so a refactor cannot reintroduce it.

| Requirement | Test | Owner |
| --- | --- | --- |
| Adapter entry points reproduce the reference wrapper's defaults, starting with `canonOrient=true` | Layout an **asymmetric** molecule and assert equality with the oracle; symmetric benzene passes either way | M4 |
| The built artifact resolves libraries with no environment variables | Run the packaged artifact under a scrubbed environment; assert rpaths contain `@loader_path` and no absolute build path | M19 |
| The build succeeds against vendored and external dependency naming | CI matrix configuring both prefixes | M4 |
| Native and WASM agree on the frozen contract | Same request set through both implementations | M20 |

Supporting requirements: the C++ standard stays at C++20 or later while RDKit
headers use `constexpr virtual`, and every coordinate parity case includes at
least one asymmetric molecule.

### Findings that change implementation

1. **`canonOrient` diverges** between RDKit's Python and C++ APIs. With the C++
   default, four of five molecules moved 3 to 11 units while symmetric benzene
   matched at `2e-16`.
2. **Runtime library discovery is solved at link time** via `@loader_path`;
   `DYLD_LIBRARY_PATH` cannot fix it because macOS strips `DYLD_*`.
3. **Dependency naming differs by build style**: external `libinchi` and
   `libcoordgen` versus vendored `RDKitInchi` and `RDKitcoordgen`.
4. **RDKit 2026.03.4 headers require C++20.**
5. **`straighten_depiction` exists in the WASM wrapper and not in
   `cffiwrapper.h`**, so the native path uses the D4 port while the browser path
   uses the built-in.

## Historical oracle and imported evidence

The historical BKChem/OASA repository remains the provenance and behavioral
reference for Ferrum development. It contributes no production runtime,
architectural identity, or supported frontend.

The oracle runs **externally**: the historical repository is pinned at a tag and
installed into the harness environment only. The Ferrum repository contains no
historical backend source. Because the harness compares a Python process using its
own RDKit against Ferrum bundling its own RDKit dylibs, each side runs in a
**separate process** and the harness compares serialized results; loading both
into one process risks duplicate-symbol resolution and mismatched global state.

Imported evidence lives under clearly identified paths and stays out of production
architecture: differential inputs, accepted reference outputs, malformed and
adversarial fixtures, chemistry parity corpora, rendering references, and measured
performance baselines. Each carries provenance metadata naming its origin.

The oracle is a development dependency. Cutover removes it from required
production and release workflows.

## Implementation constraints

- **`ChemEngine` stays small.** Its purpose is isolating RDKit; a trait growing
  toward RDKit's full surface becomes a second copy of it. Anything expressible by
  composing existing methods belongs above the trait, as `straighten_depiction`
  and SDF record splitting already do. A patch adding a method states why
  composition was insufficient.
- **The chemistry boundary is absolute.** One crate links or calls RDKit; others
  reach chemistry through `ChemEngine`.
- **Render operations stay purely declarative.** An op describes what to draw and
  carries no layout decisions, so render parity compares data rather than two
  renderers' interpretations.
- **Domain utilities stay separate modules** with separate corpora and reports.
- **Thread affinity is decided, not discovered.** RDKit is built with
  `RDK_BUILD_THREADSAFE_SSS=ON`; sessions are thread-confined and updates
  serialized per session, matching the desktop contract's GUI-thread and
  worker-relay shape rather than introducing a second concurrency model.

### Staged interface stability

Blanket instability until the freeze would risk late rework across most of the
project, so stability is staged.

| Stage | What is stable | What may change |
| --- | --- | --- |
| M4 | Ownership conventions, error categories, coordinate fidelity | Method set, signatures |
| When a method is first consumed | That method's observable behavior, covered by a test | Its signature, until the freeze |
| M17 freeze | The supported public boundaries: Python API, CLI, document channel, operation protocol | Nothing without a version bump |
| Throughout | -- | Internal adapter details, always |

**`ChemEngine` is an internal Rust seam, not a published API.** Its purpose is
isolating RDKit and letting native and WASM implementations coexist. M17 freezes
its tested semantics so downstream milestones can rely on them, and that freeze is
a project-stability commitment rather than a third-party compatibility promise.
The supported public boundaries are the Python API, the CLI, the CDML document
channel, and the operation protocol. Publishing a Rust API for third-party
consumers would be a separate, later decision with its own versioning rules.

### Automated exclusion checks

A production-tree check rejects imports from historical namespaces, Tk or Tcl
dependencies, retired console entry points, historical product names in
user-facing strings, and copied historical chemistry backend modules. Paths are
classified rather than text-banned, with an allowlist for provenance files,
migration documentation, external-oracle configuration, and fixture metadata that
must name its origin accurately. A blanket text ban would damage provenance.

### Interface stability

| Boundary | Owner | Stable | Free to change |
| --- | --- | --- | --- |
| `ChemEngine` | WS-B | Internal seam. Behavior, error semantics, `MolGraph` shape, tolerance; frozen M17 for project stability, not promised to third parties | Which RDKit calls implement it, handle representation |
| C ABI adapter | WS-B | Nothing outside its crate; internal | Entire surface, provided behavior holds |
| CDML document channel | WS-F | The format, and lossless whole-document round trips | Internal model, typed-versus-opaque assignment |
| Operation protocol | WS-F | Versioned schemas; unknown versions rejected | Encoding, batching, session internals |
| Render operations | WS-C | Serialized op shapes the frontend consumes | Geometry internals, backend implementations |
| Python API | WS-F | Names, argument shapes, exception types, import name | Which crate implements a call |
| CLI | WS-F | Subcommands, flags, exit codes, stream contracts | Formatting beyond the contract |
| Generated outputs | WS-C | The formats and documented options | Emission internals |
| Chemistry file formats | WS-B, WS-D | Conformance to each published specification | Which codec path produces them |
| Codec registry | WS-B | Registration and lookup as an extension point | Registry internals |
| Reference data | WS-B | The data values as chemistry facts | On-disk format, load mechanism |
| Configuration | WS-F | Explicit CLI flags or API arguments only | Internal defaults |

## Milestone plan

Vertical slices keep the desktop application runnable early, so contract problems
surface before the integration milestone rather than during it.

| Phase | M | Title | Goal | Size |
| --- | --- | --- | --- | --- |
| A Foundation | M1 | Repository, identity, provenance, oracle | Desktop starts with no historical imports | large |
| | M2 | Core model | Chemistry fields match the oracle | small |
| | M3 | Graph and deterministic cycles | Graph parity green | medium |
| B Chemistry | M4 | RDKit adapter and `ChemEngine` | **Slice: desktop parses SMILES, gets coordinates** | medium |
| | M5 | Chemistry codecs | Codec parity green | medium |
| C Document | M6 | XML storage and opaque retention | Structural preservation proven | medium |
| | M7 | Identity, ordering, references | Identifiers and order survive | medium |
| | M8 | Typed document records | Every class assigned and typed | large |
| | M9 | Document-core semantics | Atomicity, revisions, baseline, Recovery Export | medium |
| | M10 | Full-corpus preservation integration | **Preservation gate green** | medium |
| D Geometry and render | M11 | Geometry and straighten port | Geometry parity green | medium |
| | M12 | Render operations and glyph metrics | **Slice: desktop draws from Ferrum ops** | large |
| | M13 | Render backends | Cairo and SVG parity | medium |
| E Domain | M14 | Haworth | Reference output parity | large |
| | M15 | Domain utilities | Per-utility parity | medium |
| F Delivery | M16 | Session boundary and adoption | **Slice: Ferrum-Qt opens and saves through Ferrum-Chem** | medium |
| | M17 | Operation protocol and contract freeze | Contract frozen | medium |
| | M18 | Python module and CLI | Callable from Python and shell | medium |
| | M19 | Integration closure | Placeholders removed, behavior confirmed | medium |
| | M20 | Packaging and platform matrix | Distribution installs clean and relinks | medium; starts after M18, closes after M19 |
| | M21 | WASM proof | Contract validated on both platforms | small |
| | M22 | Establish as supported product | Oracle removed from production | small |

**Critical workstreams, not a single path.** Completion requires several
independent chains: the document chain (M6 through M10, then M16), the chemistry
chain (M4, M5), the render chain (M11 through M13), and the domain chain (M14,
M15). Haworth is large irreducible domain logic and can independently delay
integration, so it starts as soon as M5 and M13 allow rather than waiting for the
document chain.

### Phase A: foundation

**M1 repository, identity, provenance, oracle.** Depends on nothing.

Creates the repository and the Ferrum-Chem Rust workspace; establishes the two
`LICENSE` files and per-component notices; brings Ferrum-Qt source in under Ferrum
namespaces, detached from retired dependencies and without compatibility shims;
carries the Ferrum-Qt contracts across, rewritten implementation-neutral; sets up
the historical BKChem/OASA oracle harness with separate-process comparison;
ingests the M0 reference assets; writes the decision records; and enables the
automated exclusion checks.

Artifacts: `docs/provenance/` inventorying every included file by origin, reason,
and terms; decision records for the chemistry-boundary pivot, whole-document
ownership, the distribution model, and licensing; the M0 assets as
version-controlled references.

Exits when Ferrum-Qt starts from the new repository importing no historical
package, and the exclusion checks pass. Capability is expected to be thin at this
point; the module graph should already reflect the final architecture.
Parallel-plan ready: no, one owner establishes the structure.

**M2 core model.** Depends on M1. Atoms, bonds, molecules, stable identifiers,
error types. Artifact: a model specification recording which fields are carried,
which are computed, and the identifier stability guarantee, plus `proptest`
round-trip properties. Exits when every corpus molecule loads and its per-atom and
per-bond fields match the oracle exactly.

Decision criterion: *chemistry semantics outrank implementation fidelity*. A field
the historical implementation carried for bookkeeping may be dropped provided the
corpus comparison passes. Parallel-plan ready: yes.

**M3 graph and deterministic cycles.** Depends on M2. Adopts `petgraph` for
`bridges`, `articulation_points`, `matching::maximum_matching`,
`connected_components`, `dijkstra`, `floyd_warshall`, `has_path_connecting`, and
adds a project cycle basis over a spanning tree plus fundamental cycles. Exits
when graph parity is green and cycle selection is deterministic, improving on the
historical `rustworkx` path which varies by run. Parallel-plan ready: yes.

### Phase B: chemistry

**M4 RDKit adapter and `ChemEngine`.** Depends on M2. Delivers the C ABI adapter,
its build script, the native implementation, and the pinned source-build recipe,
carrying the M0 build facts: C++20, detected dependency naming, Boost declared
separately, `@loader_path` rpaths, and a stated reference default per entry point.

**Vertical slice:** Ferrum-Qt can parse a SMILES string and display generated
coordinates through Ferrum. Exits when coordinate parity is exact on the corpus,
kekulization reaches the target state, and the slice runs.
Parallel-plan ready: yes.

**M5 chemistry codecs.** Depends on M4. SMILES, SMARTS, molblock V2000 and V3000,
SDF, and InChI through the adapter, reaching RDKit's own `SDMolSupplier` and
`SDWriter` so property ordering and escaping match. InChI asserts the `InChI=1/`
prefix when non-standard output is requested. Parallel-plan ready: yes.

### Phase C: document

Split into independently verifiable units. Only M10 satisfies the complete
invariant, but each stage leaves a durable artifact and a focused gate.

**M6 XML storage and opaque retention.** Depends on M1. Delivers the `xot`-based
layer and opaque subtree handling. Artifact: an experiment record establishing
achieved fidelity -- structural by default, with byte-identical retention adopted
only if proven. Exits when a document of unrecognized elements round-trips with
elements, attributes, namespace identities, ordering, text, and values intact.

**M7 identity, ordering, references.** Depends on M6. Stable identifiers, canonical
order, `id_index` resolution, provisional-token consumption semantics. Exits when
identifiers and order survive round trips and a reference inside an opaque node is
demonstrably left untouched.

**M8 typed document records.** Depends on M7, M2. Typed payloads for every class
present in CDML today, each with unknown-attribute bags. Artifact: the
typed-versus-opaque assignment table. Largest document milestone.

**M9 document-core semantics.** Depends on M8. Owns the authoritative document
behavior, tested entirely inside Ferrum-Chem with no binding or frontend involved:
atomic commit, monotonic revisions, conflict detection, bounded history, the
pinned saved baseline, provisional-token consumption, and Recovery Export.

Exits when: an accepted candidate is not replayable; exact-snapshot reprojection
succeeds without recommit; clean/dirty stays correct after history eviction
removes the saved revision; and Recovery Export writes an exact snapshot leaving
path, baseline, dirty state, revision, and history unchanged. These semantics are
implemented once, here.

**M10 full-corpus preservation integration.** Depends on M9. Exits when the
preservation gate is green across the corpus for a document carrying every object
class. Parallel-plan ready: no, this is the integration point.

### Phase D: geometry and render

**M11 geometry and straighten port.** Depends on M2. `kurbo` and `nalgebra` over
geometry, wedge geometry, transforms, and hex grid, plus the `straightenDepiction`
port from verbatim RDKit source, recorded in `docs/provenance/` as a derived
algorithm under BSD-3. Reports rotation angle per molecule alongside coordinates
and covers both `minimizeRotation` branches. Establishes one primary geometry
representation and a conversion policy.

**M12 render operations and glyph metrics.** Depends on M11. The render-op model
and label geometry over `cairo-rs`, which M0 measured reproducing reference text
extents exactly across 78 samples. **Vertical slice:** Ferrum-Qt draws molecules
from Ferrum-produced render ops.

**M13 render backends.** Depends on M12. Cairo raster and PDF output, SVG through
`xot`. Kept separate so geometry errors stay distinguishable from renderer errors.

### Phase E: domain

**M14 Haworth.** Depends on M5, M13. Spec, layout, fragment layout, renderer.
Large irreducible domain logic. Exits against the reference outputs.

**M15 domain utilities.** Depends on M5. Sugar code, peptide utilities, repair
operations, linear formula, known groups, substructure search data. Artifact: one
differential report per utility against its own corpus.

### Phase F: delivery

**M16 session boundary and adoption.** Depends on M10. Exposes the completed
document core through the supported session boundary and replaces Ferrum-Qt's
temporary document paths with that boundary. It adds no document semantics: M9
owns those, and this milestone publishes them and maps typed failure categories
across the boundary. **Vertical slice:** Ferrum-Qt opens and saves documents
through Ferrum-Chem.

**M17 operation protocol and boundary freeze.** Depends on M16. The versioned
request/response protocol with a schema generated from Rust types, and the freeze
of the supported public boundaries: Python API, CLI, document channel, and
operation protocol.

Also freezes `ChemEngine`'s tested semantics -- per method, observable behavior,
handle ownership and lifetime, error semantics and recoverability, coordinate
fidelity and tolerance, and `MolGraph` serialization -- as an internal stability
commitment so the native and WASM implementations stay aligned. That freeze is not
a third-party compatibility promise.

**M18 Python module and CLI.** Depends on M17. Artifacts: generated `.pyi` stubs
and a CLI contract fixing subcommands, flags, exit codes, and stream behavior.

**M19 integration closure.** Depends on M18, M14, M15. Removes remaining
placeholders and confirms complete behavior. Because earlier slices already
connected the desktop, this milestone verifies rather than performs the first
meaningful integration. Includes thread-affinity confirmation. Parallel-plan
ready: no.

**M20 packaging and platform matrix.** Starts after M18 and **closes after M19**,
so packaging work runs in parallel while final acceptance still requires the
complete integrated application. The distribution with
bundled dylibs, a clean-environment install test, and a named build and validation
route per platform. M0 proved the mechanism on macOS arm64; every other platform
needs its own evidence.

Also verifies **LGPL v3 relink compliance**: Ferrum-Chem ships as a separately
replaceable shared library rather than statically linked into the extension
module, and the matrix records that a recipient can substitute a modified
Ferrum-Chem on each supported platform. Artifact: the platform matrix recording,
per platform, whether the bundle builds, installs clean, answers one chemistry
request, and permits relinking.

**M21 WASM proof.** Depends on M4, M17. A project-built MinimalLib WASM carrying
the project's exports, validated against the frozen contract. **Version policy:**
M0 measured native 2026.03.4 against WASM 2025.03.4. M21 builds the same RDKit
version on both targets; where that is impractical, the gate is exact-within-version
and invariant-across-version, matching the policy used elsewhere, rather than
identical results across differing versions.

Decision criterion: *Ferrum-Chem is frontend-agnostic*. If satisfying the contract
on WASM would require a browser-shaped concept in the trait, record the divergence
and leave the trait alone.

**M22 establish as supported product.** Depends on M19, M20. Establishes the
Ferrum repository as the supported product, removes the historical oracle from
required production and release workflows, marks the historical project as
superseded, and publishes migration documentation.

Proves the release artifact contains no historical runtime, retired product
identity, or accidental historical namespace, **while retaining intentional
attribution, provenance, migration references, and CDML heritage
acknowledgements**. The distinction the plan draws between heritage and
implementation inheritance holds at the release gate: an About-box acknowledgement
of BKChem lineage is correct and expected, a historical import is not.
Parallel-plan ready: no.

## Workstream breakdown

| WS | Goal | Owner | Provides | Review boundary |
| --- | --- | --- | --- | --- |
| WS-A | Model and graph with deterministic behavior | `coder` | Core model | Core crate |
| WS-B | Chemistry adapter and codecs | `expert_coder` | `ChemEngine` | Chemistry crate, native adapter, build scripts |
| WS-C | Geometry and render backends | `coder` | Render ops | Geometry and render crates |
| WS-D | Document model and XML formats | `coder` | Document model | Document crate |
| WS-E | Haworth and domain utilities | `expert_coder` | Domain capability | Domain crate |
| WS-F | Session, protocol, desktop integration | `expert_coder` | Frontend contract | API crate, bindings, desktop app |
| WS-G | Oracle, benchmarks, packaging, closure | `tester` | The gate each milestone exits through | Harness, CI, reports |

## Acceptance criteria and gates

- Per-patch: `cargo clippy -- -D warnings` and `cargo fmt --check` pass; new
  behavior carries a test; the touched capability's differential report shows no
  new divergence; exclusion checks pass.
- Preservation: the CDML invariant fixture round-trips intact on every parity run.
- Integration: each milestone's differential report is committed, with every
  divergence resolved or recorded as an accepted difference.
- Independent review: a `reviewer` agent audits M5, M10, M13, and M16 without
  having implemented them.

**Accepted differences** record: tolerance, rationale, affected corpus or
capability, confirmation that the difference is neither document loss nor
chemistry-semantic drift, the deciding owner, and whether it is permanent or
carries a review condition.

**Evidence over checkboxes.** Every completed milestone records the exact commands
run, the report paths, the fixtures used, and reviewer findings, so status is
verifiable rather than trusted.

**Human involvement.** Repository rules reserve `git commit` to humans; that is
release governance and sits outside technical completion. Every technical gate is
satisfiable by agents: fixture inclusion follows the objective classification in
"Inclusion is judged by fitness", verified by an automated check and a `reviewer`
agent rather than by human sign-off.

## Test and verification strategy

- Fast unit and integration tests stay offline and deterministic, use inline
  inputs and temporary directories, and finish well under a second.
- E2E runners carry the slow work: full-corpus differential sweeps, real RDKit
  execution, distribution builds, CLI round trips, and image comparison.
- Rust unit tests sit beside the code; snapshot tests cover render-op batches,
  CDML output, and Haworth output.
- Comparison rules by output class:

| Output class | Rule |
| --- | --- |
| 2D coordinates | Exact within a pinned RDKit version; geometric invariants across versions |
| Bond orders and aromatic flags | Exact |
| Text and glyph metrics | Exact |
| InChI and InChIKey | Exact string, with the `InChI=1/` prefix asserted for non-standard output |
| CDML and SVG | Structural equivalence under the stated normalization |
| Canonical SMILES and SMARTS | Exact within a pinned version; semantic round trip across versions |
| Molblock and SDF | Semantic equivalence, since headers carry program and timestamp lines |
| Raster output | Perceptual threshold with a named algorithm and pinned font environment |
| Render ops | Exact after the documented rounding |

- The corpus includes adversarial cases: malformed files, very large structures,
  unusual valence, query atoms, disconnected records, zero-bond molecules, extreme
  labels. Every coordinate case includes an asymmetric molecule.
- Fixtures are classified as required compatibility, known defect, implementation
  accident, or intended change before implementation is shaped around them.

## Risk register

M0 narrowed several risks rather than eliminating categories. Each entry states
what the evidence covered.

| Risk | Status | Impact | Trigger | Owner | Mitigation |
| --- | --- | --- | --- | --- | --- |
| Coordinate generation is unreachable from Rust | **Narrowed by D2** | -- | -- | -- | Exact parity on five molecules, one RDKit version. Chemistry parity beyond coordinates and kekulization stays unproven until M5 |
| Kekulization needs reimplementation | **Closed by D3** | -- | -- | -- | Delegation measured reaching the target state |
| RDKit cannot be built and bundled relocatably | **Narrowed by D5** | -- | -- | -- | Proven on macOS arm64 only |
| Cairo metrics cannot be reproduced | **Narrowed by D9** | -- | -- | -- | Exact across 78 samples in the measured font environment |
| A non-molecular object is dropped on round trip | Live | High: documents lose annotations, surfacing after a save | The fixture omits a class, or the gate is skipped | WS-D | The gate runs on every parity run and treats backend output alone as the verdict |
| A reference-versus-C++ default divergence changes output | Live | High: every depiction shifts while symmetric molecules pass | An entry point is written without checking the reference default | WS-B | Each entry point states its reproduced default; every coordinate case includes an asymmetric molecule |
| Opaque fidelity is weaker than a case requires | Live | High: a document class loses lexical detail | M6 finds structural preservation insufficient for a real fixture | WS-D | M6 establishes achieved fidelity by experiment before M8 depends on it; raw source-slice retention is the fallback |
| Parity rules encode historical defects as specification | Live | High: the new implementation reproduces bugs | Fixtures default to required-compatibility | WS-G | Four-way classification with a named owner, before implementation |
| Haworth delays integration | Live | Medium: large irreducible domain logic on its own chain | M14 starts late | WS-E | Start as soon as M5 and M13 allow, independent of the document chain |
| Coordinates drift across RDKit versions | Live | Medium: parity reports lose signal | A version bump lands mid-project | WS-G | Pin RDKit for oracle, native, and WASM; compare invariants across versions |
| Historical code enters the production tree | Live | Medium: provenance and licensing claims become false | A convenience import slips past review | WS-G | Automated exclusion checks on the production tree, path-classified with a provenance allowlist |

## Completion checklist

Technical completion, distinct from release process.

- [ ] RDKit version pinned for oracle, native, and WASM.
- [ ] Milestone differential reports committed.
- [ ] Accepted-difference list published with full rationale per entry.
- [ ] Preservation gate green.
- [ ] Behavioral-requirement regression tests present and passing.
- [ ] Exclusion checks passing on the production tree.
- [ ] `docs/provenance/` complete and accurate.
- [ ] Installation documentation covers the Rust toolchain, C++20 compiler, and
      RDKit build prerequisites.
- [ ] External oracle removed from required production workflows.

## Documentation close-out requirements

- Active plan: keep the milestone status table current.
- Changelog: entries as work lands, with findings that overturn an assumption
  recorded alongside additions.
- Provenance: `docs/provenance/` records origin, reason, and terms per included
  asset, including the RDKit-derived `straightenDepiction` algorithm and bundled
  binary notices.
- Durable documents, named consistently: `docs/FERRUM_ARCHITECTURE.md` (crate
  layout and the `ChemEngine` trait) and `docs/FERRUM_API_CONTRACT.md` (generated
  from the schema). The document and desktop contracts carry across rewritten
  implementation-neutral.
- Migration documentation for users opening historical documents.

## Open questions and decisions needed

Each carries an owner and a milestone. An item with neither is a defect in this
plan, not a deferral.

- Whether Ferrum-Chem and Ferrum-Qt share one repository with per-component
  `LICENSE` files or split into two. Owner: `architect`, in M1. A split enforces
  the LGPL boundary structurally; one repository is simpler during the build-out.
  Either satisfies the licensing decision.
- Which Ferrum-Qt components move as-is versus get reworked. Owner: `architect`,
  in M1, applying the six-question inclusion test per component.
- Opaque-node fidelity: structural, or raw source-slice retention. Owner: `coder`,
  in M6, decided by experiment.
- Supported platforms. Owner: `maintainer`, in M20.
- CLI subcommand surface. Owner: `planner`, in M18.
- Undo representation: inverse command, snapshot, or serialized delta. Owner:
  `expert_coder`, in M16 against the interaction model.
