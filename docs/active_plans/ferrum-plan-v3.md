# Plan: Ferrum v3, a Rust chemistry backend for Ferrum-Qt

<!--
Authority: this file is the active implementation plan.
Supersedes: docs/active_plans/floofy-snacking-ripple.md (v1),
            docs/active_plans/floofy-snacking-ripple-v2.md (v2)
Revision r2 incorporates external reviewer feedback: first usable increment,
earlier document adoption, source-of-truth hierarchy, packaging proof at M4,
M4 decomposition, preservation coverage inventory, capability matrix at M19,
Python RDKit environment separation, per-scenario performance baselines, and
open questions converted to milestone entry decisions.
-->

## Context

Ferrum is a CDML chemical drawing platform in two parts. **Ferrum-Qt** is the
PySide6 application at `packages/ferrum-chem-qt.app/ferrum_qt/`, renamed from
`bkchem_qt` -- the user's own code, carried forward wholesale and advanced.
**Ferrum-Chem** is a new Rust engine that replaces OASA, the Python chemistry
backend Ferrum does not carry forward.

That distinction is the whole plan. Ferrum-Qt is a rename. Ferrum-Chem is a
rewrite. Prior drafts blurred the two into a single "no production-code overlap"
claim, which the pre-rename baseline contradicted: the 505 tracked frontend files
were byte-identical to the reference tree before Ferrum-specific changes. Treating the Qt app as new code made a mechanical rename
look like an authorship question and left M1 unactionable.

Three revisions distinguish v3 from v2.

**Lineage stated plainly.** Ferrum-Qt is the existing Qt app under a new name. The
exclusion boundary targets `oasa` and Tk, not the frontend.

**Evidence claims matched to available evidence.** v2 leaned on an "M0 discovery
phase" with nine tracked verdicts and specific measurements. Those artifacts are
not in this repository or in `OTHER_REPOS/bkchem-oasa/` -- searching the historical
repo's `docs/`, `tools/`, and `devel/` for `MinimalLib`, `canonOrient`, or
`straightenDepiction` returns nothing. v3 keeps the engineering conclusions as
stated decisions with their reasoning, and drops the apparatus that implies
retrievable reports. Where a decision rests on a measurement that cannot be
reproduced from the repository, the milestone that depends on it re-measures.

**Gates grounded in observed behavior.** v2 required exact float coordinate parity,
exact glyph metrics, and fields matching "exactly" -- while separately conceding
that a rebuild normalizes lexical detail. v3 derives every tolerance from measured
variation rather than asserting a number, and states which comparisons are
genuinely exact because their values are discrete.

## Objectives

- Replace OASA with a Rust chemistry and document engine that owns the
  authoritative CDML document.
- Rename the Qt frontend to Ferrum-Qt and detach it from OASA capability by
  capability, keeping the application runnable throughout.
- Prove the architecture end to end on one real user workflow before building out
  the remaining capability.
- Keep documents users already created opening correctly, verified by round-trip
  preservation against a coverage inventory rather than by byte comparison.
- Keep RDKit as the chemistry authority behind one project-owned adapter.
- Ship a distribution that runs without a separately installed chemistry runtime.
- Record provenance and licensing precisely enough that redistribution obligations
  are known.

## Design philosophy

The trade-off this plan accepts: a heavier build -- a C++ toolchain and a pinned,
source-built RDKit -- in exchange for chemistry behavior that already works. The
rejected alternative is a pure-Rust chemistry core. `chematic` describes its own
depiction as not publication-quality and its pure-Rust InChI as approximate; OASA
adopted RDKit because internal implementations lost to it. Repeating that
experiment is not a good use of the project's time.

Five rules follow, each of which resolves a class of decision without escalation.

- **Port depiction utilities, delegate chemistry perception.** A gap becomes a Rust
  port when it is self-contained and arithmetic-only. A gap touching molecule
  modeling, ring perception, valence, aromaticity, or canonical ranking gets an
  exported entry point into RDKit.
- **Survey the ecosystem for infrastructure, write the chemistry.** Molecule layout,
  bond styling, Haworth projection, and sugar naming are this project's work. XML
  trees, text metrics, and path math come from maintained libraries.
- **Match the reference wrapper's defaults.** RDKit's Python wrapper and its C++ API
  disagree on `canonOrient` (`True` in `AllChem.Compute2DCoords`, `false` in
  `RDDepict::compute2DCoords`). Every adapter entry point states which default it
  reproduces, and M4b re-measures the divergence before relying on it.
- **Derive tolerances, do not invent them.** A gate that names a number the plan
  cannot justify is a gate nobody trusts. Each parity gate first measures the
  oracle's own run-to-run and platform variation, then sets the threshold outside
  that noise floor. This applies the repository's *use the scientific method*
  principle to acceptance criteria.
- **Prove feasibility before hardening an interface.** Where a downstream constraint
  can invalidate an upstream design -- packaging shaping the C ABI, WASM shaping the
  trait -- a small proof runs early rather than a discovery landing late.

Evidence strategy for uncertain methods: uncertain choices resolve against OASA
running as an external oracle on a fixed corpus, producing a divergence report per
capability. The oracle runs in a separate process, because both sides load their own
RDKit and a single process risks duplicate-symbol resolution.

## Scope

- Establish repository identity, licensing, and provenance for both components.
- Rename `bkchem_qt` to the Ferrum-Qt package namespace.
- Build the Rust workspace, the C ABI RDKit adapter, and the `ChemEngine` trait.
- Implement the CDML document model with typed and opaque object handling.
- Build the OASA differential harness, the preservation coverage inventory, and the
  preservation gate.
- Implement geometry, render operations, both render backends, and the domain layers.
- Detach Ferrum-Qt from OASA capability by capability as replacements land.
- Ship a Python distribution, a headless CLI, and a self-contained wheel.
- Prove the WebAssembly path with one compiled target.

## Non-goals

- Carry forward OASA or the Tk frontend (`packages/bkchem-app/`) as production code.
- Rewrite the Qt frontend's architecture because it resembles its own history.
- Build a browser frontend; the WASM work proves feasibility only.
- Reimplement chemistry RDKit already provides correctly.
- Change the CDML on-disk format.
- Guarantee byte-identical output anywhere. Preservation is structural.
- Reproduce historical defects for their own sake; see the source-of-truth hierarchy.

## First usable increment

The smallest workflow that proves the architecture is viable, named here so the
early milestones have one shared target instead of three unrelated slices:

> Open a simple CDML document containing one molecule, generate coordinates for it,
> render it on the Qt canvas from Ferrum-produced render operations, and save it back
> with every persistent object preserved.

Call this the **thin workflow**. It exercises every boundary the architecture rests
on -- document ownership, the chemistry adapter, render operations, and the session
round trip -- on the smallest input that can exercise them.

Cooperating milestones and what each contributes:

| Milestone | Contribution to the thin workflow |
| --- | --- |
| M2 | The molecule the document parses into |
| M4a-M4c | Coordinate generation through the adapter |
| M6, M7 | Load and save the document without losing objects, ids, or order |
| M8 | The typed molecule payload the frontend edits |
| M8a | The narrow session Ferrum-Qt calls to open and save |
| M11, M12 | Geometry and the render ops the canvas draws |

The thin workflow is demonstrated at **M8a** and re-demonstrated at every later
integration milestone. A regression in it blocks progression regardless of which
milestone is nominally in flight. The three vertical slices v2 named (M4, M12, M16)
remain, but they are now checkpoints on one named workflow rather than three
independent claims.

## Source-of-truth hierarchy

The plan uses OASA as an oracle while also intending to improve on it -- deterministic
cycle selection, corrected wrapper defaults, and refusal to reproduce historical
defects. Those goals conflict, so the conflict needs a rule rather than a case-by-case
argument.

When sources disagree about correct behavior, this is the precedence order:

1. **The CDML format specification and the landed contracts.** Where
   `CDML_FORMAT_SPEC.md` and `CDML_BACKEND_TO_FRONTEND_CONTRACT.md` define behavior,
   they win. They are the written agreement.
2. **Documents users already created.** A real document that opens correctly today
   must keep opening correctly, even where it exercises behavior the spec does not
   describe. Compatibility exists for users.
3. **RDKit's behavior.** For anything chemistry-perceptual, RDKit is the authority --
   that is why it is a dependency.
4. **Intended Ferrum behavior.** A deliberate improvement, such as deterministic
   cycle basis selection, outranks OASA's incidental behavior once it is recorded as
   an intended change.
5. **OASA's observed behavior.** The oracle is the tiebreaker for everything the
   levels above leave undefined. It is a description of what happened to be
   implemented, not a specification.

The four-way fixture classification (required compatibility, known defect,
implementation accident, intended change) is how a specific divergence is assigned a
level. **Who decides:** the workstream owner classifies; the `architect` agent
resolves any classification a workstream owner and a `reviewer` disagree on, and
records the decision in the milestone's changelog entry. A divergence classified as
"intended change" carries a one-line statement of which level above OASA justifies it.

## Current state summary

| Area | State |
| --- | --- |
| `packages/ferrum-chem-qt.app/` | Retained frontend now uses the `ferrum_qt` namespace and Ferrum-Qt product identity. Migration-only OASA calls remain until their Rust replacements land. |
| `pyproject.toml` (Qt app) | `name = "ferrum-qt"`, `license = "AGPL-3.0-only"`, correct Ferrum GitHub URLs, and the intentional installed `ferrum-qt` console entry point. The application remains a migration preview with `oasa>=26.08` until its replacement work lands. |
| OASA coupling | 445 `oasa` tokens in 18 production files; 64 direct imports in 16 files. The production tree has zero direct Tk/Tkinter imports and 29 historical Tk/Tcl text hits. |
| `packages/ferrum-rust/` | Seven-crate workspace (api, chemistry, core, document, domain, geometry, render) with a runnable `ferrum` CDML inspection/rewrite CLI; its scoped `target/` output is ignored |
| `bkchem_data` symlink | Removed through an escalated, staged `git rm`; package-owned resources resolve without the obsolete link |
| Licensing | Complete canonical offline AGPL v3 and LGPL v3 texts and `docs/PROVENANCE.md` record the intended component boundary; this is not legal advice |
| Scaffolding | `README.md`, `docs/CHANGELOG.md`, and split production/development dependency manifests are populated |
| Hygiene tests | Root suite initially reported 2,967 passed and 200 M1a-scoped failures from empty README/manifests; after accepted metadata, README, and license fixes it reported 3,167 passed; final M1a suite reported 3,186 passed |
| M1b capability evidence | `docs/active_plans/audits/ferrum_qt_capability_matrix.md` supplies 25 stable rows, including all seven export codecs, durable edits, numbering, and marks. An installed `ferrum-qt` process now starts offscreen, opens `authored_document_forms.cdml` through the existing Qt/OASA-backed native CDML route, writes the controlled receipt, and exits without a traceback. This proves M1b rename/start/open behavior, not Rust-backend adoption or worker-format completion. |
| M1e exclusion evidence | `tests/test_migration_import_exclusion.py` uses the positive Ferrum production selector with an empty active capability set. It excludes `OTHER_REPOS`, proves seeded OASA and Tk imports fail after activation, and does not claim unreplaced migration paths are clean. |
| M4a packaging evidence | The macOS arm64 native-wheel E2E source-builds the declared RDKit profile with a controlled CMake/LLVM/Rustup environment, installs a minimal wheel into a scrubbed environment, and loads it. The historical M4a stub proved a two-library closure and replacement mechanism. `docs/active_plans/reports/native_wheel_packaging.md` retains that mechanism evidence without confusing it with the later chemistry adapter. |
| M4b adapter evidence | The macOS arm64 source E2E built the GraphMol-only `ferrum-rdkit-graphmol-kekulize-v1` profile into a Ferrum-owned sealed stage, installed ABI 2 and the exact five-library chemistry closure, and ran an aromatic benzene kekulization probe in fresh Rust processes before and after a verified package-relative replacement copy. It deliberately replaces the `Release` wheel adapter with distinct-byte `RelWithDebInfo` output. Both results preserve supplied atom facts and topology and produce alternating single/double bonds. The sealed `ferrum-native-inputs-v2` manifest validates the replacement inputs. `docs/active_plans/reports/native_kekulization.md` records the receipt facts and narrow limits. |
| M1d preservation evidence | `docs/active_plans/audits/cdml_preservation_coverage.md`, three compact CDML fixtures, and separate-process comparison evidence are established. M1d remains open for real user documents plus no-namespace, future-version, alternate-prefix, and CD-SVG coverage. |
| M6 XML storage | `ferrum-document` stores opaque CDML in `xot` 0.31.2. A one-time three-fixture probe establishes structural, not lexical, retention; DTD input is rejected without an external resolver, and raw source-slice fallback is not adopted. |
| M7 identity and ordering | `IndexedDocument` derives direct-child records in source order, a declaration `id_index` that also reserves opaque IDs, root-relative element paths, and single-consumption provisional tokens. Fragment bond/vertex `id` references are excluded from declarations and never rewritten. See `docs/active_plans/decisions/document_identity_ordering.md`. |
| M2 core model | `ferrum-core` implements the accepted immutable model, accessors, and presence-sensitive properties. The authoritative M8 document projection now reads every corpus molecule with versioned bond semantics. `docs/active_plans/reports/corpus_molecule_parity.md` records the accepted exact agreements, classified differences, zero unexpected differences, and two independent mutation proofs. |
| M8 typed records | `ferrum-document` implements the accepted assignment as a single-tree typed overlay with context-qualified classes, named lexical fields, unknown-attribute bags, ordered opaque children, and non-demoting diagnostics. It is the sole production CDML reader and projects typed molecules into `ferrum-core`; evidence is in `docs/active_plans/reports/typed_document_records.md`. |
| Reference material | `OTHER_REPOS/` is gitignored, reference-only material that may be removed at any time. It can inform historical contracts and isolated oracle comparisons, but no Ferrum build, test, runtime, or release path may read it. The chemistry-adapter chain obtains RDKit only from the declared hash-verified upstream source profile. |

## Architecture boundaries and ownership

Interface stability is staged rather than blanket, so early milestones are free to
move while later ones can build on settled behavior.

| Boundary | Owner | Stable | Free to change |
| --- | --- | --- | --- |
| `ChemEngine` | WS-B | Internal seam. Behavior, error semantics, `MolGraph` shape, tolerance; frozen at M17 for project stability, not promised to third parties | Which RDKit calls implement it, handle representation |
| C ABI adapter | WS-B | Nothing outside its crate | Entire surface, provided behavior holds and the M4a packaging proof still passes |
| CDML document channel | WS-F | The format, and lossless whole-document round trips | Internal model, typed-versus-opaque assignment |
| Operation protocol | WS-F | Versioned schemas; unknown versions rejected | Encoding, batching, session internals |
| Render operations | WS-C | Serialized op shapes the frontend consumes | Geometry internals, backend implementations |
| Python API | WS-F | Names, argument shapes, exception types, import name | Which crate implements a call |
| CLI | WS-F | Subcommands, flags, exit codes, stream contracts | Formatting beyond the contract |
| Chemistry file formats | WS-B, WS-D | Conformance to each published specification | Which codec path produces them |

Two implementation constraints hold throughout. **The chemistry boundary is
absolute:** one crate links or calls RDKit; every other crate reaches chemistry
through `ChemEngine`. **`ChemEngine` stays small:** anything expressible by
composing existing methods belongs above the trait, and a patch adding a method
states why composition was insufficient.

### Mapping (milestones / workstreams -> components / patches)

| Milestone / Workstream | Component | Review boundary |
| --- | --- | --- |
| M1a, M1d, M1e / WS-G | Repository identity, licensing, harness | Root files, `docs/`, `tests/e2e/` |
| M1b / WS-F | Ferrum-Qt namespace and capability matrix | `packages/ferrum-chem-qt.app/` |
| M1c / WS-A | Rust workspace skeleton | Workspace root |
| M2, M3 / WS-A | Core model and graph | Core crate |
| M4a-M4d, M5 / WS-B | Build viability, adapter, codecs | Chemistry crate, native adapter, build scripts |
| M6-M8, M8a, M9, M10 / WS-D | Document model, early session, XML | Document crate |
| M11-M13 / WS-C | Geometry and render | Geometry and render crates |
| M14, M15 / WS-E | Haworth and domain utilities | Domain crate |
| M16-M19 / WS-F | Session, protocol, desktop integration | API crate, bindings, desktop app |
| M20-M22 / WS-G | Packaging, WASM, closure | Harness, CI, reports |

## Milestone plan

Status values: `not started`, `in progress`, `blocked`, `done`. Keep this table
current -- it is the project's status tracker.

| M | Title | Summary | Goal | Status | Owner |
| --- | --- | --- | --- | --- | --- |
| M1a | Identity and licensing | Two LICENSE files, metadata, scaffolding, symlink repair | Repository states what it is | done | `maintainer` |
| M1b | Ferrum-Qt rename and capability matrix | Namespace rename plus an inventory of what the app does | App starts under new name; capabilities enumerated | done | `coder` |
| M1c | Rust workspace skeleton | Crate layout matching the final architecture | `cargo build` succeeds | done | `coder` |
| M1d | Oracle harness and preservation inventory | Pinned OASA harness plus CDML coverage inventory | Harness compares one capability; coverage known | in progress | `tester` |
| M1e | Exclusion checks | Per-capability `oasa`/Tk import guard | Guard runs with an empty capability list | done | `tester` |
| M2 | Core model | Atoms, bonds, molecules, identifiers, errors | Corpus molecules load, fields agree with oracle | done | `coder` |
| M3 | Graph and deterministic cycles | `petgraph` plus a project cycle basis | Graph parity green, cycle choice deterministic | done | `coder` |
| M4a | Build and packaging viability | Pinned source build, dependency detection, loadable wheel | The distribution model is proven, not assumed | done | `maintainer` |
| M4b | Adapter semantics | C ABI surface, `ChemEngine`, stated defaults, kekulization | Chemistry reachable through one narrow trait | done | `expert_coder` |
| M4c | Coordinate parity and tolerance | Noise-floor measurement, then the parity gate | A justified coordinate tolerance exists | not started | `tester` |
| M4d | Qt chemistry slice | Ferrum-Qt parses SMILES through Ferrum | Frontend consumes the adapter | not started | `coder` |
| M5 | Chemistry codecs | SMILES, SMARTS, molblock, SDF, InChI | Codec parity green | not started | `expert_coder` |
| M6 | XML storage and opaque retention | `xot` layer, opaque subtrees | Structural preservation proven | done | `coder` |
| M7 | Identity, ordering, references | Stable ids, canonical order, `id_index` | Ids and order survive round trips | done | `coder` |
| M8 | Typed document records | Typed payloads plus unknown-attribute bags | Every class assigned and typed | done | `coder` |
| M8a | Early document session adoption | Narrow load/save session used by Ferrum-Qt | Thin workflow runs end to end | not started | `expert_coder` |
| M9 | Document-core semantics | Atomicity, revisions, baseline, Recovery Export | Contract semantics implemented once | not started | `expert_coder` |
| M10 | Full-corpus preservation | Integration of the document chain | Preservation gate green over the inventory | not started | `tester` |
| M11 | Geometry and straighten port | `kurbo`, `nalgebra`, `straightenDepiction` port | Geometry parity green | not started | `coder` |
| M12 | Render ops and glyph metrics | Render-op model, label geometry over `cairo-rs` | Qt draws from Ferrum ops | not started | `coder` |
| M13 | Render backends | Cairo raster and PDF, SVG through `xot` | Backend parity green | not started | `coder` |
| M14 | Haworth | Spec, layout, fragment layout, renderer | Reference output parity | not started | `expert_coder` |
| M15 | Domain utilities | Sugar code, peptides, repair, linear formula | Per-utility parity | not started | `expert_coder` |
| M16 | Full session boundary and adoption | Complete document authority in Ferrum-Qt | Qt opens and saves everything through Ferrum | not started | `expert_coder` |
| M17 | Operation protocol and freeze | Versioned protocol, boundary freeze | Contract frozen | not started | `expert_coder` |
| M18 | Python module and CLI | Bindings, stubs, CLI contract | Callable from Python and shell | not started | `coder` |
| M19 | Integration closure | Capability matrix cleared | Every mapped capability verified | not started | `integrator` |
| M20 | Packaging and platform matrix | Bundled dylibs, clean-env install, relink route | Distribution installs clean everywhere | not started | `maintainer` |
| M21 | WASM proof | Project-built MinimalLib WASM against the contract | Contract validated on both platforms | not started | `expert_coder` |
| M22 | Establish as supported product | Oracle and OASA out of production workflows | OASA dependency removed | not started | `maintainer` |

Completion needs several independent chains, not one critical path: the document
chain (M6-M8, M8a, M9, M10, then M16), the chemistry chain (M4a-M4d, M5), the render
chain (M11-M13), and the domain chain (M14, M15). Haworth is large irreducible domain
logic and can independently delay integration, so it starts as soon as M5 and M13
allow.

### Milestone: M1a identity and licensing

- Depends on: none.
- Deliverables: `LICENSE.LGPL-3.0.md` for Ferrum-Chem alongside the existing AGPL
  file; Qt `pyproject.toml` updated (name, license, homepage, `[project.scripts]`);
  populated `README.md`, first `docs/CHANGELOG.md` entry, and the dependency
  manifests split per the environment table below; repaired `bkchem_data` asset path;
  `docs/PROVENANCE.md`; a first full `pytest tests/` run over the imported tree with
  its output triaged into this milestone's scope.
- Entry criteria: none.
- Exit criteria: `pytest tests/` passes over the imported tree.
- Parallel-plan ready: yes -- licensing, scaffolding, provenance, and hygiene triage
  are independent work packages.

Licensing position, stated once. Ferrum-Chem is LGPL v3 so the backend stays
reusable; Ferrum-Qt is AGPL v3; bundled RDKit is BSD-3. AGPL v3 is GPL v3
compatible and LGPL v3 is GPL v3 plus permissions, so the AGPL application may link
the LGPL library. The Qt source is the user's own work, so relicensing it from its
current `GPL-2.0-only` metadata is a declaration, not a negotiation. Where the plan
touches historical code provenance, cite
`OTHER_REPOS/bkchem-oasa/docs/GPL_FILE_PURPOSES.md` -- which records the historical
tree as mixed GPLv2/LGPLv3 with zero pure GPLv2 files remaining -- rather than
asserting a blanket conclusion. This records compliance design, not legal advice.

LGPL v3 section 4 requires that a recipient be able to relink against a modified
library. Ferrum-Chem therefore ships as a separately replaceable shared library
rather than statically linked into the extension module. M4a proves the mechanism
early; M20 verifies the route each platform actually uses.

**Dependency environments.** Python RDKit is not part of the shipped architecture,
and the manifests must say so or it will quietly become architectural:

| Manifest | Contents | Removed at |
| --- | --- | --- |
| `pip_requirements.txt` (production) | PySide6, shiboken6, and pyyaml for the retained Qt preview. It has no Python RDKit dependency. | M22 governs removal of the Qt package's temporary OASA dependency. |
| `pip_requirements-dev.txt` | pytest, pyflakes, Maturin, and development tooling. The native-wheel builder records the installed, unpinned Maturin version in its receipt. | never |
| `tests/e2e/oracle/pip_requirements.txt` | Pinned OASA and Python RDKit for the isolated historical-oracle process only. | M1d/M22, as applicable |

After M22, production chemistry reaches RDKit only through the bundled native
adapter. Any patch adding Python `rdkit` to a production import path is a design
regression, caught by the M1e exclusion check once the chemistry capability is listed.

### Milestone: M1b Ferrum-Qt rename and capability matrix

- Depends on: M1a (package metadata names the new package).
- Deliverables: `git mv` of the package directory and every import rewritten;
  resource and data paths updated; entry point renamed. Plus
  `docs/active_plans/audits/ferrum_qt_capability_matrix.md`, enumerating what the
  application does today -- originally derived from the pre-rename
  `bkchem_qt/actions/`, `modes/`, `dialogs/`, `io/export.py`, and menu registry, now
  located under `ferrum_qt/` -- with a column per capability for its owning
  milestone, its validation artifact, and its classification (supported, known
  defect, unsupported path).
- Entry criteria: M1a metadata landed.
- Exit criteria: the application starts and opens a CDML document; the hygiene suite
  passes over the renamed tree; every enumerated capability is classified and mapped
  to a milestone.
- Exit evidence: an installed `ferrum-qt` process starts offscreen, opens
  `tests/e2e/corpus/authored_document_forms.cdml` through the existing
  Qt/OASA-backed native CDML route, dismisses warnings only inside the controlled
  smoke fence, writes the fixed receipt, and exits cleanly without a traceback. This
  proves M1b rename/start/open behavior, not Rust-backend adoption or worker-format
  completion. The focused lifecycle and CLI suite reports 21 passing tests; the
  package suite reports 918 passed and 1 skipped. Worker-routed non-CDML imports
  remain a later capability-replacement risk, not an M1b failure.
- Parallel-plan ready: yes -- the rename and the matrix are separable, though the
  matrix should be written before the rename obscures the original names.

The capability matrix is the artifact M19 closes against. Building it first means M19
has a finite, checkable exit criterion instead of an unbounded claim.

### Milestone: M1c Rust workspace skeleton

- Depends on: none.
- Deliverables: crate layout under `packages/ferrum-rust/` matching the final
  architecture (core, chemistry, document, geometry, render, domain, api).
- Exit criteria: `cargo build`, `cargo fmt --check`, and
  `cargo clippy -- -D warnings` succeed on empty crates.
- Parallel-plan ready: no -- one owner establishes the structure.

### Milestone: M1d oracle harness and preservation inventory

- Depends on: M1c (harness lives beside the workspace).
- Deliverables: `tests/e2e/e2e_oracle_*.py` runners; OASA pinned at a tag and
  installed into the harness environment only; separate-process comparison; the
  corpus at `tests/e2e/corpus/` with one owning work package. Plus
  `docs/active_plans/audits/cdml_preservation_coverage.md` -- the preservation
  coverage inventory.
- Entry criteria: none beyond M1c.
- Exit criteria: the harness produces a divergence report for one capability and
  exits non-zero on divergence; the coverage inventory is complete and every gap has
  either a corpus document or a recorded reason it cannot be obtained.
- Parallel-plan ready: yes -- harness plumbing and the inventory are independent.

**The preservation coverage inventory** answers a question M10 otherwise assumes:
does the corpus actually contain every object form the preservation gate claims to
protect? Build it from four sources, not from the corpus itself:

1. OASA's serializers and parsers (`cdml_writer.py`, `cdml.py`, `cdml_xml.py`,
   `cdml_bond_io.py`, `cdml_ftext.py`) -- every element and attribute the historical
   implementation can emit.
2. `OTHER_REPOS/bkchem-oasa/docs/CDML_FORMAT_SPEC.md` and `docs/cdml_conformance/` --
   every form the format defines.
3. `docs/reference_outputs/` and any sample or template documents shipped with the
   historical application.
4. Real user documents the user supplies, which are the only source for extension
   content and namespace cases nobody designed.

The inventory lists object form, namespace case, reference pattern, and extension
type, each marked covered or uncovered by the corpus. **M10 cannot exit while an
uncovered form remains unexplained.** This converts "the corpus is complete" from an
assumption into a checkable claim.

### Milestone: M1e exclusion checks

- Depends on: M1b (namespace settled).
- Deliverables: a hygiene test using `file_utils.discover_files` that rejects `oasa`
  and Tk imports in the production tree, driven by a per-capability list that starts
  empty and grows as milestones land replacements. Path-classified with an allowlist
  for provenance docs, oracle configuration, and fixture metadata that must name its
  origin accurately.
- Exit criteria: the guard passes with an empty capability list and fails a seeded
  violation.
- Exit evidence: `tests/test_migration_import_exclusion.py` selects only Ferrum
  production sources with `file_utils.discover_files`, excludes `OTHER_REPOS`, passes
  with its empty active-capability set, and rejects seeded OASA and Tk imports after
  the respective capability activates (4 passing focused tests).
- Parallel-plan ready: no.

### Milestone: M2 core model

- Depends on: M1c.
- Deliverables: atoms, bonds, molecules, stable identifiers, error types; a model
  specification recording which fields are carried, which are computed, and the
  identifier stability guarantee; `proptest` round-trip properties.
- Exit criteria: every corpus molecule loads; each field agrees exactly where the
  oracle is comparable, while direct source facts verify fields it cannot represent.
  Source-absence defaults, dropped fields, and corrections are classified in the model
  spec and report. *Chemistry semantics outrank implementation fidelity* -- an OASA
  bookkeeping field may be dropped once the corpus comparison passes.
- Parallel-plan ready: yes.

### Milestone: M3 graph and deterministic cycles

- Depends on: M2.
- Deliverables: `petgraph` for bridges, articulation points, maximum matching,
  connected components, Dijkstra, Floyd-Warshall, and path connectivity, plus a
  project cycle basis over a spanning tree with fundamental cycles.
- Implementation evidence: graph parity is green and exact cycle and matching outputs
  repeat across 100 calls per fixture. The current reference cycle path was stable in
  this probe. Ferrum's shorter bridged basis is an **intended change** under level 4;
  `docs/active_plans/reports/graph_analysis_parity.md` corrects the older instability
  assumption. The M2 prerequisite and M3 implementation are both green.
- Parallel-plan ready: yes.

### Milestone: M4a build and packaging viability

- Depends on: M1c.
- Deliverables: the pinned RDKit source-build recipe carrying the build facts --
  C++20 (required by RDKit 2026.03.4 headers), detected dependency naming (external
  `libinchi`/`libcoordgen` versus vendored `RDKitInchi`/`RDKitcoordgen`), Boost
  declared separately, `@loader_path` rpaths -- plus a **minimal loadable wheel**: a
  stub extension that links the adapter shell, bundles the dylibs, installs into a
  clean environment, and answers one trivial call.
- Entry criteria: none beyond M1c.
- Exit criteria: the stub wheel installs into a scrubbed environment and runs; the
  bundled Ferrum-Chem library can be replaced with a rebuilt copy and the wheel still
  works, proving the LGPL relink route on the development platform.
- Historical exit evidence: `tests/e2e/e2e_native_wheel.py` passed on macOS arm64
  against the `ferrum-rdkit-cpp-coordgen-inchi-v1` profile. The installed ABI-1 stub
  wheel loaded before and after replacement of `libferrum_chem.dylib`. Its historical
  two-library closure was
  `libferrum_chem.dylib` and `libRDKitRDGeneral.1.dylib`. See
  `docs/active_plans/reports/native_wheel_packaging.md` for that run's sources,
  toolchain, and limits. M4b has since completed the narrow ABI-2 native
  kekulization operation on the same packaging foundation. M4c, M4d, M20, and M22
  remain open for coordinate parity, Qt use, platform coverage, and removal of the
  migration dependency.
- Parallel-plan ready: no -- one owner, one toolchain.

This milestone exists because packaging shapes the C ABI, the wheel layout, the build
system, the supported Python versions, and the feasible platform set. Discovering at
M20 that the linkage model is impractical would invalidate adapter design decisions
made across M4b through M17. The proof is deliberately small: it proves the mechanism,
not the chemistry.

### Milestone: M4b adapter semantics

- Depends on: M2, M4a (the ABI shape must fit a working distribution).
- Deliverables: the C ABI adapter surface, the `ChemEngine` trait, the native
  implementation, a stated reference default per entry point, and the `canonOrient`
  re-measurement.
- Exit criteria: kekulization reaches the state `Kekulize(clearAromaticFlags=False)`
  produces; every entry point documents which reference default it reproduces;
  `MolGraph` is the only structural output type, so no RDKit representation leaks.
- Exit evidence: ABI 2, Ferrum-owned `MolGraph`, safe `ChemEngine`, strict
  kekulization records, and the sealed `ferrum-native-inputs-v2` manifest passed the
  GraphMol-only source E2E before and after replacing `libferrum_chem.dylib` with
  deliberate distinct-byte `RelWithDebInfo` output. The explicit defaults are
  `clear_aromatic_flags=false`, `canonical=true`, and `max_backtracks=100`. The receipt
  proves the exact five-library macOS closure, alternating benzene bond orders,
  topology preservation, and supplied atom-fact preservation. See
  `reports/native_kekulization.md` and `decisions/chemistry_engine_boundary.md`.
  The one-time orientation evidence is
  [`reports/rdkit_layout_orientation.json`](reports/rdkit_layout_orientation.json),
  generated outside pytest by
  [`devel/rdkit_layout_orientation.py`](../../devel/rdkit_layout_orientation.py).
  M4b is limited to this native operation. M4c remains responsible for coordinate
  implementation, tolerance derivation, and coordinate parity.
- Parallel-plan ready: yes -- one work package per entry-point group.

### Milestone: M4c coordinate parity and tolerance derivation

- Depends on: M4b.
- Deliverables: the noise-floor measurement described under *Acceptance criteria and
  gates*, then the coordinate parity gate wired into the harness.
- Exit criteria: the derived tolerance is recorded alongside the measured variation
  that produced it, and coordinate parity is green on the corpus. Every coordinate
  case includes at least one asymmetric molecule, because a symmetric molecule passes
  under either `canonOrient` default and proves nothing.
- Parallel-plan ready: no -- the measurement gates the gate.

### Milestone: M4d Qt chemistry slice

- Depends on: M4b, M1b.
- Deliverables: Ferrum-Qt parses a SMILES string and displays generated coordinates
  through Ferrum; the chemistry capability added to the M1e exclusion list.
- Exit criteria: the slice runs in the application; the exclusion check now rejects
  new `oasa` chemistry calls.
- Parallel-plan ready: no.

### Milestone: M5 chemistry codecs

- Depends on: M4b.
- Deliverables: SMILES, SMARTS, molblock V2000 and V3000, SDF, and InChI through the
  adapter, reaching RDKit's own `SDMolSupplier` and `SDWriter` so property ordering
  and escaping match.
- Exit criteria: codec parity green per the comparison rules; non-standard InChI
  output carries the `InChI=1/` prefix.
- Parallel-plan ready: yes -- one work package per format family.

### Milestone: M6 XML storage and opaque retention

- Depends on: M1c.
- Deliverables: the `xot`-based layer and opaque subtree handling, plus an
  experiment record establishing achieved fidelity.
- Exit criteria: a document of unrecognized elements round-trips with elements,
  attributes, namespace identities, ordering, text, and values intact. Fidelity is
  **structural**, not byte-identical -- tree-based parsing normalizes lexical detail.
  Raw source-slice retention is the fallback if a real fixture loses meaning under
  structural preservation; adopting it requires the experiment proving the need first.
- Parallel-plan ready: no -- the experiment gates the design.

### Milestone: M7 identity, ordering, references

- Depends on: M6.
- Deliverables: stable identifiers, canonical order, `id_index` resolution,
  provisional-token consumption semantics.
- Exit criteria: identifiers and order survive round trips, and a reference inside an
  opaque node is demonstrably left untouched.
- Parallel-plan ready: yes.

### Milestone: M8 typed document records

- Depends on: M7, M2.
- Entry evidence: the disposable M2 harness reader was deleted during this milestone.
  The corpus comparison now consumes `ferrum-document`'s typed projection, so there is
  one CDML reader. `tests/test_cdml_reader_inventory.py` passes with only the document
  crate allowed, and `xot` is absent from `crates/core/Cargo.toml`
  `[dev-dependencies]`.
- Deliverables: typed payloads for every class present in CDML today -- molecule,
  reaction, arrow, text, plus sign, the six vector-graphic shapes (`rect`, `square`,
  `oval`, `circle`, `polygon`, `polyline`), and the molecule-scoped `group` vertex --
  each with an unknown-attribute bag and an unrecognized-child list; the
  typed-versus-opaque assignment table, accepted as
  `docs/active_plans/decisions/typed_record_assignment.md`. Bracket and vector
  capabilities keep round-tripping without a class of their own: CDML has no
  `<bracket>` or `<vector>` element, the bracket tool persists its artwork as
  direct-root `<polyline>` records, and vector-graphic content is the six shape
  elements above. Legacy or extension `<bracket>` and `<vector>` content stays
  preserve-only opaque, as the format specification's deferred document concepts
  require.
- Exit criteria: a recognized element carrying one unfamiliar attribute stays typed
  and still round-trips. Promotion from opaque to typed is additive; demotion does
  not occur.
- Parallel-plan ready: yes -- one work package per object class.

### Milestone: M8a early document session adoption

- Depends on: M8, M4d, M12 for the render half of the thin workflow.
- Deliverables: a narrow document session exposing load, snapshot, and save; Ferrum-Qt
  using it for open and save on documents within the typed subset; the thin workflow
  demonstrated end to end.
- Entry criteria: M8's typed molecule payload exists.
- Exit criteria: the thin workflow runs -- open a one-molecule CDML document, generate
  coordinates, render from Ferrum ops, save with every persistent object preserved --
  and is added to the E2E suite so later milestones cannot silently break it.
- Parallel-plan ready: no.

This milestone exists so document semantics get sustained use by the real application
early. Under v2's shape, the document core was designed across M6 through M10 and not
exercised by Ferrum-Qt until M16, which is a long time to design a contract nobody is
calling. M8a deliberately adopts less than the full contract: advanced transaction
semantics stay owned by M9, and full adoption stays at M16. What it buys is feedback.

### Milestone: M9 document-core semantics

- Depends on: M8a (adoption feedback informs the semantics).
- Deliverables: atomic commit, monotonic revisions, conflict detection, bounded
  history, the pinned saved baseline, provisional-token consumption, Recovery Export.
  Tested entirely inside Ferrum-Chem with no binding or frontend involved.
- Exit criteria: an accepted candidate is not replayable; exact-snapshot reprojection
  succeeds without recommit; clean/dirty stays correct after history eviction removes
  the saved revision; Recovery Export writes an exact snapshot leaving path, baseline,
  dirty state, revision, and history unchanged.
- Parallel-plan ready: yes.

### Milestone: M10 full-corpus preservation

- Depends on: M9, M1d (the coverage inventory).
- Deliverables: the preservation gate wired into the parity run.
- Entry criteria: the preservation coverage inventory is complete.
- Exit criteria: a document carrying every object class round-trips with every
  persistent object, id, source order, and attribute intact, verified from backend
  output alone with no frontend reconstruction -- **and** every form in the coverage
  inventory is either exercised by a corpus document or carries a recorded reason it
  is not.
- Parallel-plan ready: no -- this is the integration point.

### Milestone: M11 geometry and straighten port

- Depends on: M2.
- Deliverables: `kurbo` and `nalgebra` over geometry, wedge geometry, transforms, and
  hex grid; the `straightenDepiction` port from RDKit C++ source, recorded in
  `docs/PROVENANCE.md` as a derived algorithm under BSD-3; one primary geometry
  representation and a conversion policy.
- Exit criteria: geometry parity within derived tolerance, covering both
  `minimizeRotation` branches; rotation angle reported per molecule alongside
  coordinates.
- Parallel-plan ready: yes.

### Milestone: M12 render ops and glyph metrics

- Depends on: M11.
- Deliverables: the render-op model and label geometry over `cairo-rs`.
- Exit criteria: text extents match the reference within the derived tolerance in the
  pinned font environment; Ferrum-Qt draws molecules from Ferrum-produced render ops.
  Render ops stay purely declarative -- an op describes what to draw and carries no
  layout decisions, so parity compares data rather than two renderers' opinions.
- Parallel-plan ready: yes.

### Milestone: M13 render backends

- Depends on: M12.
- Deliverables: Cairo raster and PDF output, SVG through `xot`. Kept separate from
  M12 so geometry errors stay distinguishable from renderer errors.
- Exit criteria: SVG structural equivalence; raster within the perceptual threshold
  derived in this milestone.
- Parallel-plan ready: yes -- one work package per backend.

### Milestone: M14 Haworth

- Depends on: M5, M13.
- Deliverables: spec, layout, fragment layout, renderer.
- Exit criteria: output matches the reference outputs under the render-op and SVG
  comparison rules.
- Parallel-plan ready: yes.

### Milestone: M15 domain utilities

- Depends on: M5.
- Deliverables: sugar code, peptide utilities, repair operations, linear formula,
  known groups, substructure search data; one differential report per utility.
- Exit criteria: per-utility parity green. Domain utilities stay separate modules
  with separate corpora and reports.
- Parallel-plan ready: yes -- one work package per utility.

### Milestone: M16 full session boundary and adoption

- Depends on: M10.
- Deliverables: the complete document core exposed through the supported session
  boundary; every remaining Ferrum-Qt document path replaced; typed failure categories
  mapped across; the pre-migration performance baselines recorded (see
  *Performance expectations*).
- Exit criteria: Ferrum-Qt opens and saves every document class through Ferrum-Chem.
  This milestone adds no document semantics -- M9 owns those and this publishes them.
- Parallel-plan ready: no.

### Milestone: M17 operation protocol and boundary freeze

- Depends on: M16.
- Deliverables: the versioned request/response protocol with a schema generated from
  Rust types; the freeze of the supported public boundaries (Python API, CLI,
  document channel, operation protocol) and of `ChemEngine`'s tested semantics.
- Exit criteria: schema generated and checked in; unknown protocol versions rejected.
  The `ChemEngine` freeze is an internal stability commitment so native and WASM stay
  aligned, not a third-party compatibility promise.
- Parallel-plan ready: no.

### Milestone: M18 Python module and CLI

- Depends on: M17.
- Pre-milestone proof: the self-contained `ferrum cdml inspect` and
  `ferrum cdml rewrite` shell interface is implemented and tested. This proves that
  the Rust backend is directly runnable, but remains non-frozen until M17; it neither
  starts formal M18 delivery nor satisfies the still-pending Python binding work.
- Deliverables: bindings, generated `.pyi` stubs, and a CLI contract fixing
  subcommands, flags, exit codes, and stream behavior, derived from the Qt app's
  existing batch and export capabilities in the M1b matrix.
- Exit criteria: CLI round trips succeed under `tests/e2e/`.
- Parallel-plan ready: yes.

### Milestone: M19 integration closure

- Depends on: M18, M14, M15.
- Deliverables: remaining placeholders removed; thread-affinity confirmed (RDKit
  built with `RDK_BUILD_THREADSAFE_SSS=ON`, sessions thread-confined, updates
  serialized per session, matching the desktop contract rather than introducing a
  second concurrency model).
- Entry criteria: the M1b capability matrix exists and every row is mapped.
- Exit criteria: **every capability in the M1b matrix classified as supported has its
  named validation artifact passing.** Rows classified known defect or unsupported
  path carry a recorded decision -- reproduced, fixed, or dropped -- rather than
  silently passing. Per-scenario performance is no worse than the M16 baselines.
- Parallel-plan ready: yes -- capability verification parallelizes by matrix row.

Replacing "every capability the Qt app performed" with the matrix makes this exit
criterion finite and checkable. Earlier vertical slices already connected the desktop,
so this milestone verifies rather than performs first integration.

### Milestone: M20 packaging and platform matrix

- Depends on: M18, M4a (which already proved the mechanism on one platform). Closes
  after M19.
- Deliverables: the full distribution with bundled dylibs; a clean-environment install
  test; a named build and validation route per platform; LGPL v3 relink verification
  per platform.
- Exit criteria: per platform, the matrix records whether the bundle builds, installs
  clean, answers one chemistry request under a scrubbed environment, and permits
  relinking. Runtime library discovery is solved at link time through `@loader_path`;
  `DYLD_LIBRARY_PATH` cannot fix it because macOS strips `DYLD_*`.
- Parallel-plan ready: yes -- one work package per platform.

### Milestone: M21 WASM proof

- Depends on: M4b, M17.
- Deliverables: a project-built MinimalLib WASM carrying the project's exports,
  validated against the frozen contract.
- Exit criteria: the same request set produces equivalent results through both
  implementations. Build the same RDKit version on both targets; where that is
  impractical, the gate is exact-within-version and invariant-across-version.
  Note that `straighten_depiction` exists in the WASM wrapper but not in
  `cffiwrapper.h`, so the native path uses the M11 port while the browser path uses
  the built-in. If satisfying the contract on WASM would require a browser-shaped
  concept in `ChemEngine`, record the divergence and leave the trait alone.
- Parallel-plan ready: no.

### Milestone: M22 establish as supported product

- Depends on: M19, M20.
- Deliverables: OASA removed from required production and release workflows; `oasa`
  and Python `rdkit` dropped from `pip_requirements.txt` and the Qt `pyproject.toml`;
  migration documentation published; the historical project marked superseded.
- Exit criteria: the release artifact contains no OASA or Tk runtime and no Python
  RDKit dependency, while retaining intentional attribution, provenance, migration
  references, and CDML heritage acknowledgement. An About-box acknowledgement of
  BKChem lineage is correct and expected; an `oasa` import is not.
- Parallel-plan ready: no.

## Workstream breakdown

### Workstream: WS-A core model and graph

- Goal: deterministic core model and graph behavior.
- Owner: `coder`.
- Work packages: M1c, M2, M3.
- Needs: nothing.
- Provides: the core model every other workstream builds on.
- Review boundary: core crate.

### Workstream: WS-B chemistry adapter and codecs

- Goal: isolate RDKit behind one narrow trait, on a distribution model proven to work.
- Owner: `expert_coder`, with `maintainer` owning M4a.
- Work packages: M4a, M4b, M4c, M4d, M5.
- Needs: the core model from WS-A.
- Provides: `ChemEngine` and a proven packaging mechanism.
- Review boundary: chemistry crate, native adapter, build scripts.

### Workstream: WS-C geometry and render

- Goal: declarative render operations and both backends.
- Owner: `coder`.
- Work packages: M11, M12, M13.
- Needs: the core model from WS-A.
- Provides: render ops the frontend consumes, including the render half of the thin
  workflow.
- Review boundary: geometry and render crates.

### Workstream: WS-D document model

- Goal: lossless CDML ownership, exercised by the real application early.
- Owner: `coder`, with `expert_coder` on M8a and M9.
- Work packages: M6, M7, M8, M8a, M9, M10.
- Needs: the crate skeleton; typed molecule payloads need WS-A; M8a needs render ops
  from WS-C and chemistry from WS-B.
- Provides: the authoritative document and the thin workflow.
- Review boundary: document crate.

### Workstream: WS-E domain capability

- Goal: Haworth and the domain utilities.
- Owner: `expert_coder`.
- Work packages: M14, M15.
- Needs: codecs from WS-B, render backends from WS-C.
- Provides: domain capability.
- Review boundary: domain crate.

### Workstream: WS-F session, protocol, desktop

- Goal: the frontend contract, the capability inventory, and the Qt adoption path.
- Owner: `expert_coder`, with `coder` on M1b.
- Work packages: M1b, M16, M17, M18, M19.
- Needs: the document core from WS-D.
- Provides: the supported public boundaries and the capability matrix.
- Review boundary: API crate, bindings, desktop app.

### Workstream: WS-G harness, packaging, closure

- Goal: the gate each milestone exits through.
- Owner: `tester`, with `maintainer` for packaging and licensing.
- Work packages: M1a, M1d, M1e, M4c, M10, M20, M21, M22.
- Needs: whichever capability it is gating.
- Provides: divergence reports, the coverage inventory, the preservation gate, the
  distribution.
- Review boundary: harness, CI, reports.

## Acceptance criteria and gates

- **Per-patch gate:** `cargo clippy -- -D warnings`, `cargo fmt --check`,
  `cargo test`, and `pytest tests/` pass; new behavior carries a test; the
  exclusion check passes.
- **Thin-workflow gate:** from M8a onward, the thin workflow E2E case passes. It runs
  on every milestone exit, not only in the milestone that introduced it.
- **Milestone-exit gate:** the touched capability's differential report shows no new
  divergence, run through `tests/e2e/`. This is deliberately not a per-patch gate --
  the oracle is a subprocess with its own RDKit and belongs in the slow lane.
- **Preservation gate:** the CDML invariant fixture round-trips intact on every
  parity run, against the coverage inventory. This is the one strict gate, because it
  is binary and checkable.
- **Independent review gate:** a `reviewer` agent audits M4a, M5, M10, M13, and M16
  without having implemented them.

### How each tolerance is derived

No gate in this plan names a threshold the plan cannot justify. Each parity gate
follows the same procedure, owned by the milestone that first needs it:

1. Run the oracle repeatedly on the corpus and record its own run-to-run variation.
2. Run it across the supported platforms and record cross-platform variation.
3. Set the gate threshold outside that measured noise floor, and record the measured
   variation alongside the chosen number.
4. When variation is zero because the values are discrete, state exact equality and
   say why it is exact.

A gate whose threshold has not been derived yet is written as "derived at M<n>"
rather than filled with a placeholder number.

### Comparison rules by output class

| Output class | Rule | Basis |
| --- | --- | --- |
| 2D coordinates | Within tolerance derived at M4c against the pinned RDKit version; geometric invariants across versions | Floating-point layout varies with RDKit version and build |
| Atom and bond fields | Every field Ferrum carries agrees with the oracle; dropped fields listed in the model spec | Discrete values, but Ferrum may carry fewer |
| Bond orders and aromatic flags | Exact | Discrete enumerations |
| InChI and InChIKey | Exact string; `InChI=1/` prefix asserted for non-standard output | Deterministic string output |
| Canonical SMILES and SMARTS | Exact within a pinned version; semantic round trip across versions | Canonical ranking can change between RDKit releases |
| Molblock and SDF | Semantic equivalence | Headers carry program and timestamp lines |
| CDML and SVG | Structural equivalence under one stated normalization: attribute ordering, namespace serialization, and insignificant whitespace normalized before comparison | Byte equality is not achievable through a tree parser |
| Text and glyph metrics | Within tolerance derived at M12, in the pinned font environment | Font rasterization varies by version and platform |
| Render ops | Exact after the rounding documented at M12 | Declarative data with stated precision |
| Raster output | Perceptual threshold with the algorithm and value derived at M13, pinned font environment | Anti-aliasing differs across backends |
| Straighten port | Within tolerance derived at M11, both `minimizeRotation` branches | Floating-point trigonometry |
| Cycle basis | Deterministic across runs; divergence from OASA classified intended change | The current historical reference was stable in the 100-call probe; Ferrum owns a deterministic shorter-basis policy independent of dependency traversal |

### Performance expectations

The plan sets no absolute latency or memory targets. A number like "loads in under
400 ms" would be invented rather than measured, and would fail or pass for reasons
unrelated to this work. A single whole-application baseline is also too coarse -- it
can hide a large editing-latency regression behind an unchanged startup time. So the
baseline is per scenario.

Record these at M16, on the current application, on one machine, over the corpus:

| Scenario | What it protects | Frame-budget relevant |
| --- | --- | --- |
| Application startup to usable canvas | Launch cost of the new backend and bundled dylibs | no |
| Open a typical document | Parse and projection cost | no |
| Open the largest corpus document | Scaling behavior | no |
| Draw one bond (gesture to repaint) | Interactive editing latency | yes |
| Drag a selection (per-frame preview) | Interactive latency under continuous input | yes |
| Redraw after a committed edit | Round-trip plus reprojection cost | yes |
| Save a typical document | Serialization cost | no |
| Save the largest corpus document | Serialization scaling | no |
| Export SVG and PNG | Render backend cost | no |

The M19 gate is "no worse than the recorded baseline, per scenario, on the same
machine and corpus." The roughly 16 ms frame budget applies only to the three
scenarios marked frame-budget relevant, because it is a property of display refresh
and has nothing to say about opening or saving a file. Whole-document CDML exchange
was chosen over a fragment protocol on the understanding that a committed-edit round
trip fits inside that budget for documents of the size this project produces; M8a
measures it on the real corpus as soon as the session exists, and the choice is
revisited if documents grow past a few hundred kilobytes.

**Accepted differences** record: tolerance, rationale, affected corpus or capability,
which source-of-truth level justifies the difference, confirmation that it is neither
document loss nor chemistry-semantic drift, the deciding owner, and whether it is
permanent or carries a review condition.

**Evidence over checkboxes.** Every completed milestone records the exact commands
run, the report paths, the fixtures used, and reviewer findings.

**Human involvement.** Repository rules reserve `git commit` to humans; that is
release governance and sits outside technical completion. Every technical gate is
satisfiable by agents.

## Test and verification strategy

Four homes, one rule each. Placing a check in the wrong home is how a fast suite
becomes slow and how a one-time proof becomes permanent maintenance.

**`cargo test` -- Rust, fast.** Unit tests beside the code, `proptest` round-trip
properties, snapshot tests for render-op batches, CDML output, and Haworth output.
Most Ferrum-Chem verification lands here. This lane is invisible to `pytest tests/`
and is named as its own per-patch gate.

**`pytest tests/` -- permanent, fast, offline.** Only checks passing the
`docs/PYTEST_STYLE.md` checklist: deterministic, inline inputs, `tmp_path` only,
well under a second, no network, no subprocess. For this project that means CDML
typed and opaque round trips on inline XML strings; id, order, and reference
preservation on an inline document; the `oasa`/Tk exclusion lint written as a hygiene
test using `file_utils.discover_files`; and pure geometry helpers.

**`tests/e2e/e2e_*.py` -- slow, real.** Everything touching real RDKit, the oracle
subprocess, built artifacts, or the corpus: the thin-workflow case, differential
sweeps, the packaging and scrubbed-environment install checks, CLI round trips, the
WASM contract run, per-scenario performance runs, and image comparison. Never
`test_*.py` under `tests/e2e/` -- `collect_ignore` would skip it silently and the
name would lie.

**One-time proofs -- not permanent tests.** Checks that prove the rebuild during a
milestone and then stop earning their keep: the M6 opaque-fidelity experiment, the
M4b `canonOrient` divergence measurement, and the tolerance-derivation runs described
above. Record the result in the milestone's changelog entry, then either delete the
script or keep it under `devel/` as a maintainer tool. Do not let them accrete into
`tests/`.

Reclassifying the behavioral requirements v2 listed as permanent tests: the
scrubbed-environment rpath check is E2E; the dependency-naming build matrix is CI
configuration rather than a test; the WASM cross-check is E2E; the asymmetric-molecule
layout check is a `cargo test` or E2E case depending on whether it needs RDKit loaded.
None is a `pytest tests/` case.

### Corpus and fixture placement

The corpus, reference outputs, and adversarial fixtures live under
`tests/e2e/corpus/` as E2E inputs, owned by one work package in M1d. They are never
pytest fixtures: repository rules treat committed `tests/fixtures/` as shared
infrastructure needing explicit sign-off, and this plan asks for that sign-off once,
here, rather than letting each milestone add a directory. A pytest case needing a
document writes an inline string to `tmp_path`.

Corpus composition is driven by the preservation coverage inventory rather than by
what happens to be convenient, and includes adversarial cases: malformed files, very
large structures, unusual valence, query atoms, disconnected records, zero-bond
molecules, and extreme labels. Every coordinate case includes an asymmetric molecule.

Fixtures are classified as required compatibility, known defect, implementation
accident, or intended change **before** implementation is shaped around them, using
the source-of-truth hierarchy. This prevents the plan's most expensive failure mode:
reproducing a historical bug because a fixture defaulted to required-compatibility.

## Migration and compatibility policy

Ferrum-Qt keeps calling installed `oasa` until Ferrum-Chem replaces each capability.
That is the migration, not a violation of the no-shims rule:

- `oasa` and its transitive Python `rdkit` are declared in `pip_requirements.txt` and
  the Qt `pyproject.toml` as migration-window dependencies, with M22 named as the
  removal gate in a comment beside each.
- The M1e exclusion check runs from a per-capability list that starts empty. When a
  milestone lands a replacement, that milestone adds its capability to the list, and
  the check then rejects new `oasa` calls in that area. M4d is the first such addition.
- No milestone is expected to leave the application unrunnable.

CDML compatibility is preserved by specification and differential testing. The
on-disk format does not change.

## Risk register

| Risk | Impact | Trigger | Owner | Mitigation |
| --- | --- | --- | --- | --- |
| A non-molecular object is dropped on round trip | High: documents lose annotations, surfacing after a save | The fixture omits a class, or the gate is skipped | WS-D | The preservation gate runs on every parity run against the M1d coverage inventory, and treats backend output alone as the verdict |
| The corpus silently under-covers CDML | High: the preservation gate passes while real documents break | Corpus assembled from convenient samples | WS-G | M1d builds the coverage inventory from OASA's serializers, the format spec, reference outputs, and real user documents; M10 cannot exit with an unexplained gap |
| The distribution model proves impractical late | High: adapter and wheel design decisions across M4b-M17 are invalidated | Packaging left until M20 | WS-B | M4a proves a loadable, relinkable wheel before the ABI hardens |
| Reference-versus-C++ default divergence changes output | High: every depiction shifts while symmetric molecules pass | An entry point is written without checking the reference default | WS-B | Each entry point states its reproduced default; every coordinate case includes an asymmetric molecule |
| Opaque fidelity is weaker than a case requires | High: a document class loses lexical detail | M6 finds structural preservation insufficient for a real fixture | WS-D | M6 establishes achieved fidelity by experiment before M8 depends on it; raw source-slice retention is the fallback |
| Parity rules encode historical defects as specification | High: the rebuild reproduces bugs | Fixtures default to required-compatibility | WS-G | Four-way classification against the source-of-truth hierarchy, with `architect` resolving disputes, before implementation |
| A gate is set to an invented number and quietly skipped | High: the gate protects nothing while appearing to | A tolerance is filled in without the derivation run | WS-G | Tolerances are written as "derived at M<n>" until measured; a gate cannot be marked green with an underived threshold |
| Document semantics designed without a caller | Medium: the contract fits no real usage and is reworked at M16 | Full adoption deferred to M16 | WS-D | M8a adopts a narrow session after M8 and demonstrates the thin workflow |
| Python RDKit becomes an architectural dependency | Medium: the shipped product needs a separate chemistry runtime | The oracle dependency drifts into a production import | WS-G | Manifests separate production from dev; the exclusion check covers chemistry from M4d; M22 exits on its removal |
| The Qt rename leaves partial namespaces | Medium: imports break in rarely-exercised dialogs | The rename is done by search-and-replace without starting the app | WS-F | M1b exits on a running application plus the hygiene suite over the renamed tree |
| Interactive latency regresses behind an unchanged startup time | Medium: the app feels slower while the gate passes | Performance measured only as one whole-application number | WS-F | Per-scenario baselines recorded at M16; M19 compares each scenario separately |
| Haworth delays integration | Medium: large irreducible domain logic on its own chain | M14 starts late | WS-E | Start as soon as M5 and M13 allow, independent of the document chain |
| Coordinates drift across RDKit versions | Medium: parity reports lose signal | A version bump lands mid-project | WS-G | Pin RDKit for oracle, native, and WASM; compare invariants across versions |
| OASA code enters the production tree | Medium: provenance and licensing claims become false | A convenience import slips past review | WS-G | Per-capability exclusion checks, path-classified with a provenance allowlist |
| Imported Qt files fail the repository hygiene suite | Medium: M1 stalls on unrelated lint | `pytest tests/` has never run against the 505 imported files | WS-G | Run the suite during M1a and treat the output as M1a scope |

## Rollout and release checklist

- [ ] RDKit version pinned for oracle, native, and WASM.
- [ ] Thin workflow passing continuously since M8a.
- [ ] Milestone differential reports committed.
- [ ] Every parity tolerance derived from a recorded measurement, not asserted.
- [ ] Accepted-difference list published, each entry naming its source-of-truth level.
- [ ] Preservation coverage inventory complete; preservation gate green against it.
- [ ] M1b capability matrix fully closed at M19.
- [ ] Per-scenario performance no worse than the M16 baselines.
- [ ] Exclusion checks passing on the production tree with the full capability list.
- [ ] `docs/PROVENANCE.md` complete and accurate.
- [ ] Installation documentation covers the Rust toolchain, C++20 compiler, and RDKit
      build prerequisites.
- [ ] `oasa` and Python `rdkit` removed from production manifests.
- [ ] Oracle removed from required production workflows.

## Documentation close-out requirements

- Active plan: keep the milestone status table current; the `Status` and `Owner`
  columns are the tracker.
- `docs/CHANGELOG.md`: entries as work lands, including findings that overturn an
  assumption, every derived tolerance with its measurement, and every
  source-of-truth classification dispute the `architect` resolved.
- Provenance: `docs/PROVENANCE.md` records origin, reason, and terms per included
  asset, including the RDKit-derived `straightenDepiction` algorithm and bundled
  binary notices.
- Audits: `docs/active_plans/audits/ferrum_qt_capability_matrix.md` and
  `docs/active_plans/audits/cdml_preservation_coverage.md`, both kept current as
  milestones close their rows.
- Durable docs, named per `docs/REPO_STYLE.md`: `docs/CODE_ARCHITECTURE.md` (crate
  layout and the `ChemEngine` trait), `docs/FILE_STRUCTURE.md`, and
  `docs/FERRUM_API_CONTRACT.md` (generated from the schema). Carry the contracts
  across from `OTHER_REPOS/bkchem-oasa/docs/`:
  `CDML_BACKEND_TO_FRONTEND_CONTRACT.md`, `QT_CONTRACT.md`, `CDML_FORMAT_SPEC.md`,
  rewritten implementation-neutral.
- Migration documentation for users opening historical documents.
- Plan file hygiene: this file remains the active authority. The two superseded drafts
  are retained in place as explicitly historical records until a separate archival
  cleanup is authorized.

## Open questions and decisions needed

Each question names the latest point at which it can be answered without rework, and
that point is an **entry condition** for the first milestone that would otherwise
commit the architecture. None blocks starting M1a.

- **Whether Ferrum-Chem and Ferrum-Qt share one repository.**
  - Decision owner: `architect`. Latest safe point: entry to M1a, because licensing
    file placement and packaging layout both depend on it.
  - Evidence and decision rule: the current tree already answers this in favor of one
    repository with per-component `LICENSE` files; confirm the LGPL boundary is
    enforceable by directory and packaging alone, or split. Either satisfies the
    licensing decision.
- **Supported platforms.**
  - Decision owner: `maintainer`. Latest safe point: entry to M4a, because the RDKit
    build recipe, the dependency-detection logic, and the feasible linkage model are
    all per platform.
  - Evidence and decision rule: name the candidate set at M4a and prove one of them
    end to end there; M20 builds the bundle per remaining candidate and keeps the ones
    that install clean and permit relinking.
- **Undo representation: inverse command, snapshot, or serialized delta.**
  - Decision owner: `expert_coder`. Latest safe point: entry to M9, because bounded
    history and the pinned saved baseline are shaped by the representation. This is
    earlier than v2's M16.
  - Evidence and decision rule: decide against the interaction model plus the
    whole-document snapshot cost measured at M8a on the real corpus.
- **Opaque-node fidelity: structural, or raw source-slice retention.**
  - Decision owner: `coder`. Latest safe point: entry to M8, because typed records
    are built on the retention model.
  - Evidence and decision rule: run the M6 experiment against the real corpus; adopt
    source-slice retention only if a real fixture loses meaning under structural
    preservation.
- **CLI subcommand surface.**
  - Decision owner: `planner`. Latest safe point: entry to M18. Genuinely late-binding;
    nothing upstream depends on it.
  - Evidence and decision rule: derive from the batch and export capabilities already
    enumerated in the M1b matrix rather than designing fresh.
