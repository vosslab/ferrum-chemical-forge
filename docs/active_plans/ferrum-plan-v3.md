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

The trade-off this plan accepts: a heavier build -- a C++ toolchain and a
source-built RDKit -- in exchange for chemistry behavior that already works. Each
shipping artifact records and verifies its exact source tag and hash, while the
development policy advances to the latest stable RDKit and compares the prior stable
release when that comparison is meaningful. The
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
| `packages/ferrum-chem-qt.app/` | The retained frontend uses the `ferrum_qt` namespace and Ferrum-Qt product identity. Its one ordinary product window is Rust-native; historical OASA/BKChem material is provenance or isolated-oracle material, not a product route. |
| `pyproject.toml` (Qt app) | `name = "ferrum-qt"`, `license = "AGPL-3.0-only"`, correct Ferrum GitHub URLs, the installed `ferrum-qt` console entry point, and no production OASA declaration. This remains a pre-release product while M17--M22 close its public-contract and release work. |
| OASA coupling | The 445-token/64-import measurement and the former compatibility host are historical evidence. The accepted 2026-08-15 retirement removed the host island and production OASA declarations; ordinary startup, session, rendering, file work, and publication have no OASA fallback. The isolated oracle remains available only where a historical comparison is still useful. |
| `packages/ferrum-rust/` | Seven-crate workspace (api, chemistry, core, document, domain, geometry, render) with the frozen `ferrum protocol schema/run` public CLI and separately owned Rust/Ferrum-Qt-native document, chemistry, rendering, and publication seams. Its scoped `target/` output is ignored; retired provisional root commands are not a public contract. |
| `bkchem_data` symlink | Removed through an escalated, staged `git rm`; package-owned resources resolve without the obsolete link |
| Licensing | Complete canonical offline AGPL v3 and LGPL v3 texts and `docs/PROVENANCE.md` record the intended component boundary; this is not legal advice |
| Scaffolding | `README.md`, `docs/CHANGELOG.md`, and split production/development dependency manifests are populated |
| Hygiene tests | Root suite initially reported 2,967 passed and 200 M1a-scoped failures from empty README/manifests; after accepted metadata, README, and license fixes it reported 3,167 passed; final M1a suite reported 3,186 passed |
| M1b capability evidence | `docs/active_plans/audits/ferrum_qt_capability_matrix.md` remains the historical inventory and the M19 closure ledger. An installed offscreen `ferrum-qt` launch/open receipt established rename behavior; current supported/refused/drop decisions must be reconciled row by row before M19 closes. |
| M1e exclusion evidence | The earlier positive-selector guard is historical evidence of staged migration. The current product boundary has no OASA runtime path; a current-tree/package inventory is disposable release evidence, while M22 still requires a release-artifact audit. |
| M4a packaging evidence | The macOS arm64 native-wheel E2E source-builds the declared RDKit profile with a controlled CMake/LLVM/Rustup environment, installs a minimal wheel into a scrubbed environment, and loads it. The historical M4a stub proved a two-library closure and replacement mechanism. `docs/active_plans/reports/native_wheel_packaging.md` retains that mechanism evidence without confusing it with the later chemistry adapter. |
| M4b adapter evidence | The macOS arm64 source E2E built the GraphMol-only `ferrum-rdkit-graphmol-kekulize-v1` profile into a Ferrum-owned sealed stage, installed ABI 2 and the exact five-library chemistry closure, and ran an aromatic benzene kekulization probe in fresh Rust processes before and after a verified package-relative replacement copy. It deliberately replaces the `Release` wheel adapter with distinct-byte `RelWithDebInfo` output. Both results preserve supplied atom facts and topology and produce alternating single/double bonds. The sealed `ferrum-native-inputs-v2` manifest validates the replacement inputs. `docs/active_plans/reports/native_kekulization.md` records the receipt facts and narrow limits. |
| ABI4/FCM1 superseding wheel evidence | The ABI2/five-library row above remains historical mechanism evidence. The accepted current proof is the direct-extension macOS arm64 wheel `output_native_wheel/inchi-v1-current-20260813/wheelhouse/ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl`, SHA-256 `0f2de3ae9819545846af46efc45cae3eddbfbcabda5a0653f31d2a4ff6e79e6f`. It uses official RDKit 2026.03.5 plus pinned IUPAC InChI 1.07.3 source, ABI4 FCM1, and a measured 18-dylib closure. This is a narrow macOS arm64 packaging proof, not M20 platform coverage or product release evidence. |
| M4c/M4d chemistry evidence | The current fresh direct-extension wheel `output_native_wheel/molblock-import-v1-rdkit-2026035-20260812/` has SHA-256 `13de57cf0d95dc3f1755f14a1ca36350fe4db7dca43e3ab8ead0e3d0e74b3eda`. The coordinate receipt was refreshed against 20 RDKit 2026.03.5 Python-wrapper processes and 20 Ferrum ABI-4 processes; both again had zero internal noise and exact atom-aligned coordinates across six molecules, so the ULP-derived tolerance remains `7.105427357601002e-15`. The direct Rust-native Ferrum Qt route imports, renders, saves, and reopens CCO. This is recorded measurement evidence, not a new gate. |
| M5 SMARTS codec | ABI-4 FCG1 carries a complete frozen molecule graph into the native adapter and bounded FCT1 returns RDKit `MolToSmarts` text. Eight cases match the recorded RDKit 2026.03.5 build exactly. A disposable offline check generated both current and previous stable query sets, parsed both under RDKit 2026.03.4 and 2026.03.5, and found agreement on all 272 chirality-aware query-target outcomes. Exact cross-version text was observed but is not required. OASA explicitly registers SMARTS as export-only, so import is not a replacement gate. |
| M5 molblock codec | ABI-4 carries complete graphs and atom-aligned coordinates in both directions. Strict bounded V2000/V3000 import and export pass seven graph/stereo/charge/isotope cases under current RDKit 2026.03.5 and previous stable 2026.03.4. Coordinate bounds are derived from each emitted decimal token rather than imposed globally. The standalone native Qt route imports one bounded local Molfile through the Rust parser boundary, commits the complete graph against exact document provenance, renders it, and saves/reopens CDML without OASA. Both codec directions pass before and after a distinct adapter replacement. |
| M5 SDF codec | ABI-4 FSD1/FCT1 exports ordered coordinate-bearing records through RDKit `SDWriter`; FSI1 imports bounded UTF-8 SDF through strict `SDMolSupplier` into owned Rust and frozen Python records. Three multi-record cases in explicit V2000 and V3000 pass semantic evaluation under current RDKit 2026.03.5 and previous RDKit 2026.03.4, and Ferrum import agrees with the current evaluator. Acceptance checks chemistry and ordered property values, never SDF bytes. The OASA reference conversion is 2D and uncompressed, so 3D/compressed suppliers are not parity gates. |
| M5 InChI codec | ABI-4 imports Standard and Fixed-H InChI into the complete owned graph, exports both explicit modes, and derives validated 27-character InChIKeys. A disposable five-molecule corpus matched RDKit 2026.03.4 and 2026.03.5 exactly for both modes, keys, and canonical round trips. The current direct wheel uses pinned IUPAC InChI 1.07.3, has SHA-256 `0f2de3ae9819545846af46efc45cae3eddbfbcabda5a0653f31d2a4ff6e79e6f`, and passes installed-extension probes before and after a distinct adapter replacement. Its measured macOS arm64 closure is 18 dylibs. Exact strings are a grounded InChI identifier contract, not a general byte-equivalence gate. |
| M1d preservation evidence | `docs/active_plans/audits/cdml_preservation_coverage.md`, three compact CDML fixtures, and separate-process comparison evidence are established. M1d remains open for real user documents plus no-namespace, future-version, alternate-prefix, and CD-SVG coverage. |
| M6 XML storage | `ferrum-document` stores opaque CDML in `xot` 0.31.2. A one-time three-fixture probe establishes structural, not lexical, retention; DTD input is rejected without an external resolver, and raw source-slice fallback is not adopted. An accepted caller-owned preflight rejects decoded XML by bytes, elements, depth, attributes, and lexical text/CDATA before `xot` allocation. The ordinary local uncompressed-CDML V1 profile now selects one versioned operational envelope for both the native render CLI and ordinary asynchronous desktop Open; CD-SVG and compression remain separate. |
| M7 identity and ordering | `IndexedDocument` derives direct-child records in source order, a declaration `id_index` that also reserves opaque IDs, root-relative element paths, and single-consumption provisional tokens. Fragment bond/vertex `id` references are excluded from declarations and never rewritten. See `docs/active_plans/decisions/document_identity_ordering.md`. |
| M2 core model | `ferrum-core` implements the accepted immutable model, accessors, and presence-sensitive properties. The authoritative M8 document projection now reads every corpus molecule with versioned bond semantics. `docs/active_plans/reports/corpus_molecule_parity.md` records the accepted exact agreements, classified differences, zero unexpected differences, and two independent mutation proofs. |
| M8 typed records | `ferrum-document` implements the accepted assignment as a single-tree typed overlay with context-qualified classes, named lexical fields, unknown-attribute bags, ordered opaque children, and non-demoting diagnostics. It is the sole production CDML reader and projects typed molecules into `ferrum-core`; evidence is in `docs/active_plans/reports/typed_document_records.md`. |
| Reference material | `OTHER_REPOS/` is gitignored, reference-only material that may be removed at any time. It can inform historical contracts and isolated oracle comparisons, but no Ferrum build, test, runtime, or release path may read it. The chemistry-adapter chain obtains RDKit only from the declared hash-verified upstream source profile. |

### 2026-08-12 pre-milestone implementation evidence

The following accepted vertical slices make the native boundary observable without
changing any milestone status in the tracker. They are intentionally smaller than
their corresponding exit criteria.

- The direct PyO3 session exposes exact Rust render observations. The earlier separate
  native-tab route established bounded open/render/edit behavior; the ordinary
  OASA-free `ferrum-qt` route now starts a Rust-owned empty document and opens
  uncompressed local CDML through its direct budgeted-file adapter. It can render, change a
  selected atom element, add one Rust-identified free-standing
  atom to a durable molecule, connect exactly two existing atoms with one
  Rust-identified normal single bond by selection or a revision-bound drag gesture,
  extend an existing atom into empty space with one atomic Rust carbon-plus-bond edit,
  move one durable atom through an exact revision-bound Rust point operation, delete one
  durable atom and its typed incident bonds as one history entry, delete one durable typed
  bond while preserving both endpoint atoms, change one selected normal bond among single,
  double, and triple order, edit one selected durable bond's supported order, depiction style,
  center, width, and color facts as one atomic patch, regenerate one ordinary durable molecule's coordinates off the UI
  thread while retaining its current centroid and mean bond length, export one fresh detached
  observation as atomic SVG, PDF, or allocation-bounded 72-DPI PNG, undo, redo, save, reopen, and
  dispose a bounded native document
  path. Focused evidence is in
  `docs/active_plans/reports/native_bond_creation_v1.md` and
  `docs/active_plans/reports/native_atom_move_v1.md` and
  `docs/active_plans/reports/native_atom_deletion_v1.md` and
  `docs/active_plans/reports/native_bond_deletion_v1.md` and
  `docs/active_plans/reports/native_bond_order_v1.md` and
  `docs/active_plans/reports/native_bond_properties_v1.md` and
  `docs/active_plans/reports/native_coordinate_regeneration_v1.md` and
  `docs/active_plans/reports/native_molfile_import_v1.md`.
- The same early standalone route imported a bounded UTF-8 SDF as one complete ordered
  batch. Every supported 2D record becomes a distinct durable molecule; exact title
  and ordered duplicate property facts remain in a preserved Ferrum extension child,
  and one undo removes the complete import. This advances FQ-004 but does not adopt
  the route into the then-ordinary product window or complete M16.
- The standalone route also prepares one InChI string through the packaged ABI-4
  adapter off the Qt thread and commits its handle-free molecule only against the
  captured document revision and digest. InChI-complete hydrogen counts cross into
  CDML; unrepresentable chirality, radicals, maps, and stereo facts remain typed
  failures rather than lossy imports. This advances FQ-008 without claiming the
  ordinary `MainWindow` cutover or a general text-import subsystem.
- These early-route statements are historical adoption evidence, not a description of the
  current window. The remaining M16 work is closure: keep supported native behavior
  documented, refuse unsupported inputs before read, and record intentional product drops.
- The molecule render-plan painter, presentation-vector projection, and
  Rust/PyO3 display-geometry bridge are separate V1 projections. They do not prove
  M11 geometry parity, M12 full render-op/glyph-metric tolerance, M13 backends, or
  M16 adoption of every Qt path.
- The standalone native Qt snapshot route paints a fresh revision/digest-matched
  detached observation to SVG, PDF, or PNG and never exports selected screen
  items. It is an OASA-free frontend output path, not the independent M13
  renderer-neutral backend. Its artifact and visual checks remain disposable evidence;
  semantic render-operation tests remain the permanent gate.
- Telex design metrics use a verified Rust `ttf-parser` route. The Qt painter
  consumes supplied Rust glyph IDs and origins; it does not shape or measure text.
  Direct PNG/PDF sinks landed under M13; a native graphics library still requires
  a separate M20 packaging decision and is not an M12 metrics dependency.
- Historical provisional CLI families exercised native observation, parse/write
  chemistry, document mutation, and atomic publication before M18. M19 retired
  their root-command surface; these underlying private and Ferrum-Qt-native seams
  do not freeze the public CLI contract.
- The public Rust-only `ferrum_document::artifact_publication_v1` now publishes completed
  caller-owned bytes through retained no-follow parent descriptors. It optionally retains a
  live regular source descriptor and refuses only aliases observed at both final checks under a
  trusted non-mutating output-directory assumption. Its 0600 same-directory temporary is
  written and file-synced before rename, then the held directory is synced; the result records
  confirmed durability, directory-entry-unconfirmed durability, or a possibly-published
  post-rename failure, with separate destination-rejection and I/O cleanup uncertainty. The
  existing CDML save path is its exact adapter and retains its baseline semantics. This is not a
  CLI, stdout, PyO3, wire, renderer, or ingress boundary and assigns no default size policy.
  The later `ferrum-local-cdml-ingress-v1` policy and `cdml render svg` adapter compose it
  explicitly; this does not turn the generic publisher itself into a renderer. M18 remains
  not started.

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
| M1d | Oracle harness and preservation inventory | Pinned OASA harness plus CDML coverage inventory | Harness compares one capability; coverage known | done | `tester` |
| M1e | Exclusion checks | Staged `oasa`/Tk migration guard | Historical staged guard recorded; current product has one native root | done | `tester` |
| M2 | Core model | Atoms, bonds, molecules, identifiers, errors | Corpus molecules load, fields agree with oracle | done | `coder` |
| M3 | Graph and deterministic cycles | `petgraph` plus a project cycle basis | Graph parity green, cycle choice deterministic | done | `coder` |
| M4a | Build and packaging viability | Pinned source build, dependency detection, loadable wheel | The distribution model is proven, not assumed | done | `maintainer` |
| M4b | Adapter semantics | C ABI surface, `ChemEngine`, stated defaults, kekulization | Chemistry reachable through one narrow trait | done | `expert_coder` |
| M4c | Coordinate parity and tolerance | Noise-floor measurement, then the parity gate | A justified coordinate tolerance exists | done | `tester` |
| M4d | Qt chemistry slice | Ferrum-Qt parses SMILES through Ferrum | Frontend consumes the adapter | done | `coder` |
| M5 | Chemistry codecs | SMILES, SMARTS, molblock, SDF, InChI | Codec parity green | done | `expert_coder` |
| M6 | XML storage and opaque retention | `xot` layer, opaque subtrees | Structural preservation proven | done | `coder` |
| M7 | Identity, ordering, references | Stable ids, canonical order, `id_index` | Ids and order survive round trips | done | `coder` |
| M8 | Typed document records | Typed payloads plus unknown-attribute bags | Every class assigned and typed | done | `coder` |
| M8a | Early document session adoption | Narrow load/save session used by Ferrum-Qt | Thin workflow runs end to end | done | `expert_coder` |
| M9 | Document-core semantics | Atomicity, revisions, baseline, Recovery Export | Contract semantics implemented once | done | `expert_coder` |
| M10 | Full-corpus preservation | Integration of the document chain | Preservation gate green over the inventory | done | `tester` |
| M11 | Geometry and straighten port | `kurbo`, `nalgebra`, `straightenDepiction` port | Current-target receipt and public atomic result | done | `coder` |
| M12 | Render ops and glyph metrics | Declarative render-op model and deterministic Telex design metrics | Current-target Qt consumes Ferrum operations and verified Telex metrics | done | `coder` |
| M13 | Render backends | Direct PNG/PDF sinks and SVG over checked render operations | Semantic sink evidence plus disposable artifacts | done | `coder` |
| M14 | Haworth | Spec, layout, fragment layout, renderer | Source-backed semantic behavior | done | `expert_coder` |
| M15 | Domain utilities | Bounded peptide insertion, linear form, and geometry repair; explicit utility drops | Retained workflows have source-backed semantics and unadopted families have a recorded disposition | done | `expert_coder` |
| M16 | Full session boundary and adoption | One Rust-native window plus explicit supported, refused, and dropped routes | One ordinary Rust-native window owns each supported document route; other historical routes are explicitly refused or dropped | done | `expert_coder` |
| M17 | Operation protocol and freeze | Versioned protocol, boundary freeze | Contract frozen | complete | `expert_coder` |
| M18 | Python module and CLI | Bindings, stubs, CLI contract | Callable from Python and shell | complete | `coder` |
| M19 | Integration closure | Capability matrix cleared | Every mapped capability verified | not started | `integrator` |
| M20 | Packaging and platform matrix | Two selected wheels, clean-env install, relink route | Target receipt passes | source accepted; runtime evidence pending | `maintainer` |
| M21 | WASM proof | Project-built MinimalLib WASM against the contract | Contract validated on both platforms | not started | `expert_coder` |
| M22 | Establish as supported product | Release closure and migration documentation | Release artifact and workflows prove the supported boundary | source accepted; runtime and human review pending | `maintainer` |

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
| `pip_requirements.txt` (production) | PySide6, shiboken6, and pyyaml for the retained Qt product. It has no Python RDKit or OASA dependency. | M22 audits the release artifact and workflow rather than assuming source manifests suffice. |
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
- Exit evidence: the historical installed `ferrum-qt` receipt established
  rename/start/open behavior during M1b. It does not define the current product
  boundary or M16/M19 closure; current release evidence is recorded under those
  milestones.
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
  and Tk imports in the production tree, driven by a per-capability list that grows
  as milestones land replacements. Path-classified with an allowlist
  for provenance docs, oracle configuration, and fixture metadata that must name its
  origin accurately.
- Exit criteria: the guard passes for the activated capability paths and fails a
  seeded violation.
- Exit evidence: `tests/test_migration_import_exclusion.py` was a staged positive
  selector while native adoption was incremental. It is historical evidence, not a
  claim that a separate desktop route remains or a substitute for the M22
  release-artifact audit.
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
- Deliverables: a versioned and hash-verified RDKit source-build recipe carrying the
  build facts. Each wheel records one exact reproducible source, while new wheel
  builds move to the current stable RDKit release and check semantic compatibility
  with the previous stable release. The recorded build facts include
  C++20 (required by current RDKit headers), detected dependency naming (external
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
  kekulization operation on the same packaging foundation. M20 and M22 remain open
  for broader platform coverage and removal of the migration dependency.
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
  M4b is limited to this native operation. M4c now supplies the separate coordinate
  implementation, tolerance derivation, and coordinate parity evidence.
- Parallel-plan ready: yes -- one work package per entry-point group.

### Milestone: M4c coordinate parity and tolerance derivation

- Depends on: M4b.
- Deliverables: the noise-floor measurement described under *Acceptance criteria and
  gates*, then the coordinate parity gate wired into the harness.
- Exit criteria: the derived tolerance is recorded alongside the measured variation
  that produced it, and coordinate parity is green on the corpus. Every coordinate
  case includes at least one asymmetric molecule, because a symmetric molecule passes
  under either `canonOrient` default and proves nothing.
- Exit evidence: `reports/coordinate_parity_v1.md` and its source-bound JSON receipt
  compare 20 independent RDKit 2026.03.5
  Python-wrapper processes with 20 fresh ABI-4 wheel processes. Both process noise
  maxima and the cross-backend maximum delta are 0.0 across six molecules, five of
  them asymmetric. The tolerance is derived as four times observed process noise or
  eight times the largest measured coordinate ULP, whichever is larger:
  `7.105427357601002e-15`. The committed test recomputes every measurement-source
  digest. M20 retains expansion beyond the current macOS arm64 platform.
- Parallel-plan ready: no -- the measurement gates the gate.

### Milestone: M4d Qt chemistry slice

- Depends on: M4b, M1b.
- Deliverables: Ferrum-Qt parses a SMILES string and displays generated coordinates
  through Ferrum; the chemistry capability added to the M1e exclusion list.
- Exit criteria: the slice runs in the application; the exclusion check now rejects
  new `oasa` chemistry calls.
- Exit evidence: the standalone `FerrumNativeMainWindow` starts a cancellable
  background Rust SMILES preparation, accepts only the frozen extension DTO, commits
  it through one revision-bound `DocumentSession` molecule transaction, and installs
  the resulting render observation on the Qt thread. The fresh macOS arm64 wheel at
  `output_native_wheel/smiles-insertion-v1-20260812-v2/` has SHA-256
  `a901132f29fa3cd33c2516004be8bdf7fbe9272066d7cb6ab2b8b82b82caaaff` and an
  audited 13-library ABI-4 closure. `e2e_native_cdml_file_route.py` proves public CCO
  import, display, atomic save, reopen, opaque-root retention, and zero OASA imports.
  The chemistry capability is active in the M1e guard for the native route.
  The V1 insertion writer deliberately rejects chirality, bond stereo/direction,
  radicals, no-implicit policy, atom maps, stereo references, and quadruple bonds
  until exact CDML round trips are defined; this is a writer-contract gap, not a
  claim that CDML cannot represent those concepts.
- Parallel-plan ready: no.

### Milestone: M5 chemistry codecs

- Depends on: M4b.
- Deliverables: SMILES, SMARTS, molblock V2000 and V3000, SDF, and InChI through the
  adapter, reaching RDKit's own `SDMolSupplier` and `SDWriter` so property ordering
  and escaping match.
- Exit criteria: codec parity green per the comparison rules; non-standard InChI
  output carries the `InChI=1/` prefix.
- Implementation evidence: the SMARTS export slice is green in
  `reports/smarts_codec_v1.md`. ABI-4 FCG1/FCT1, safe Rust, the frozen PyO3 DTO,
  and the provisional explicit-adapter CLI match the recorded RDKit 2026.03.5 build
  exactly on eight discrete graph cases and survive a distinct adapter replacement.
  RDKit 2026.03.4 and 2026.03.5 also agree on 272 chirality-aware query-target
  outcomes. The read-only OASA registry defines SMARTS as export-only, so importing
  SMARTS would be a new query feature rather than parity work.
- The explicit V2000/V3000 import and export slice is green in
  `reports/molblock_codec_v1.md`. Seven molecules pass strict semantic reparse under
  current RDKit 2026.03.5 and previous stable 2026.03.4. Coordinates are bounded by the precision actually
  written in each format; molblock bytes, comments, spacing, and headers are observed
  but are not acceptance gates. The clean 15-library wheel and both operations pass
  before and after a distinct adapter replacement.
- Ordered multi-record SDF export and bounded 2D import are green in
  [reports/sdf_codec_v1.md](reports/sdf_codec_v1.md).
  Titles, record order, property order and values, and discrete chemistry survive
  strict V2000/V3000 reparse under current and previous stable RDKit releases. Exact
  SDF bytes are outside the contract. Import retains duplicate property names as
  distinct ordered values and rejects 3D conformers until the Rust model owns 3D.
- The standalone native window now consumes that import boundary directly. Rust
  bounds local-file bytes before UTF-8/native parsing, preserves every record and
  exact ordered metadata, lays multiple molecules out without overlap, and prepares
  one revision-bound document batch. The Qt worker carries only one frozen batch;
  it does not parse SDF or interpret CDML. Semantic record/data retention and atomic
  history are permanent gates; the public-window worker run is disposable evidence.
  The OASA conversion copies x/y only and exposes plain text/file SDF, so 3D and
  compressed suppliers are not parity requirements.
- Standard and nonstandard Fixed-H InChI import/export plus InChIKey are green in
  `reports/inchi_codec_v1.md`. The safe Rust engine,
  direct PyO3 extension, and explicit-adapter CLI share the same ABI-4 boundary.
  A disposable five-molecule corpus matched current RDKit 2026.03.5 and previous
  stable 2026.03.4 exactly for the canonical identifiers and semantic round trips.
- M5 is complete at the declared OASA-compatible codec boundary. Future SMARTS query
  import, 3D chemistry, compressed suppliers, or additional data typing require their
  own user-facing capability decisions rather than being smuggled into parity gates.
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
  one authoritative CDML reader, and `xot` is absent from `crates/core/Cargo.toml`
  `[dev-dependencies]`. The former permanent source-path allowlist was retired on
  2026-08-12 after it misclassified legitimate `cfg(test)` and API-ingress literals;
  parser ownership is now reviewed at the crate/API boundary and exercised through
  document-ingress behavior rather than inferred from which files spell a URI.
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

The accepted standalone `--native` route now exercises load, existing-molecule coordinate
generation, Rust render plans, save/reopen, opaque-root preservation, a closed atomic
atom-properties patch, persistent atom-number assign/show/hide/clear, and all seven closed
atom-mark toggles. The property slice
targets one durable atom ID and carries all nine authored dialog facts through exact frozen
binding values; optional facts can be cleared without authoring dialog defaults. The number
slice targets one direct atom through its durable molecule/atom pair and renders its explicit
Rust-owned Telex glyph operation. Mark toggles use the same durable pair and render explicit
Rust-owned line and ellipse operations. A projection-backed chooser can remove one ID-less mark
by its exact same-type ordinal. That chooser is the accepted replacement for legacy direct-canvas
selection: an atom mark has no durable identity of its own, so Qt must not invent one. An explicit
local-wheel E2E saves and reopens all seven kinds while retaining opaque XML and avoiding OASA.
Opaque CDML material remains retained. This completed the bounded M8a thin workflow;
it predates the single-window host retirement and does not claim M16/M19/M22 closure.
Its gates remain semantic rather than byte-, pixel-, or arbitrary timing-equivalence
requirements.

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

The accepted backend-only gate is `tests/e2e/e2e_cdml_preservation.py`. It discovers
the committed corpus and runs the public structural rewrite contract without byte
comparison or frontend reconstruction. See
[`cdml_preservation_gate.md`](reports/cdml_preservation_gate.md).

### Milestone: M11 geometry and straighten port

- Depends on: M2.
- Status: done for the currently evidenced macOS arm64 release target. Generic graph
  topology validation no longer globally excludes fused or bridged graphs; only
  single-ring normalization owns the exact one-independent-cycle boundary.
- The public Rust `RepairOutcome` reports complete durable-identity-ordered y-up
  positions and the y-up applied rotation in radians, including a reported zero
  rotation for a valid no-op. `DocumentSession` prepares exact revision/digest-bound,
  caller-ordered targets and applies them atomically after structural selector
  preflight, preserving z, opaque content, and history semantics.
- Deliverables: `kurbo` and `nalgebra` over geometry, wedge geometry, transforms, and
  hex grid; the `straightenDepiction` port from RDKit C++ source, recorded in
  `docs/PROVENANCE.md` as a derived algorithm under BSD-3; one primary geometry
  representation and a conversion policy.
- Exit evidence: both `minimizeRotation` branches have semantic Cargo behavior tests,
  full public results, and an atomic document boundary. The recomputable, locked,
  offline macOS arm64 CPython 3.12/RDKit 2026.03.5 receipt reports a maximum coordinate
  delta of `3.645723512257204e-18`, rotation delta of `0`, and local repeat variation
  of `0`. See [`geometry_straighten_parity.md`](reports/geometry_straighten_parity.md).
- Permanent evidence: semantic Cargo behavior tests. The receipt is one-time evidence,
  not a CI tolerance or pass threshold. M20 refreshes equivalent receipts for each
  added release target. PyO3 exposure belongs to M18, not this exit. M11 adds no byte,
  pixel, timing, or network gate.
- Parallel-plan ready: no.

### Milestone: M12 render ops and glyph metrics

- Depends on: M11.
- Status: done for the currently evidenced macOS arm64 Telex/PySide6 reference
  boundary. Declarative operations, verified Telex glyph IDs, frozen PyO3
  observations, and Qt drawing without reshaping are landed. This does not establish
  cross-platform coverage, pixel or byte equivalence, a permanent numeric threshold,
  or an M13 backend.
- Deliverables: a declarative render-op model and deterministic Telex design metrics
  from Rust `ttf-parser`; Qt consumes supplied glyph IDs and origins rather than
  selecting fonts, shaping, or measuring text. Direct PNG/PDF sinks are later M13 work;
  a native graphics library requires a separate M20 packaging decision.
- Exit evidence: the closed Telex corpus agrees with a QRawFont 6.11.1 design-metric
  reference on macOS arm64. Exact glyph IDs/origins agree; the largest per-run `f64`
  representation observation is about `1.78e-15` scene units. Qt baseline
  descent/height differs by `0.0001875` scene units and remains an observation, not a
  tolerance or pass threshold. A disposable current-wheel proof consumes supplied
  iodine and plus outlines through `QRawFont.pathForGlyph`. The M12 report records
  the complete boundary and its limits.
- Render-op policy: Rust converts verified `ttf-parser` design units directly to
  scene `f64` values with no extra rounding. True outline ink bounds serve runs and
  centered plus signs; only atom-label clipping adds the atom origin to the outline
  union. Ordered typed DTO semantics preserve schema, provenance, targets/order,
  variants, exact discrete facts, and finite `f64` values through round-trip JSON.
  JSON spelling is not a renderer contract.
- Permanent evidence: semantic Cargo model and projection tests. Corpus measurements
  and current-wheel receipts are one-time development or E2E evidence, not arbitrary
  byte, pixel, timing, or GUI-wiring gates. M20 refreshes target evidence when the
  release matrix grows.
- Parallel-plan ready: no.

### Milestone: M13 render backends

- Depends on: M12.
- Status: done. `ferrum-render` lowers each supported `DocumentRenderPlanV1` once through
  its private checked `DrawSinkV1` stream to owned in-memory `xot` SVG, direct pure-Rust
  PNG, and direct pure-Rust vector PDF. The common external receipt carries the exact
  revision/digest provenance, full page rectangle, and source-order named exclusions;
  Ferrum provenance is not embedded in output artifact bytes. Supported molecule,
  fixed-plus, Text, Arrow, Polyline, Wavy, round-bracket, Rectangle, Square, Oval,
  Circle, and Polygon roots retain issued geometry, paint, order, and Telex outlines.
  Profile, rejected-projection, and `not_yet_lowered` roots remain named exclusions.
  Invalid projection, non-finite conversion, overflow, and sink failure are typed and
  yield no partial artifact/receipt.
- Active molecule-plan production and consumption use only `ferrum-render-plan-v2`.
  `RenderPlanV2`, `RenderBatchV2`, `RenderOperationV2`, and
  `DocumentMoleculeRenderPlanV2` carry a neutral validated scene path grammar:
  `MoveTo`, `LineTo`, `CubicTo`, and `Close`, with explicit optional stroke, fill,
  and z. The common draw stream feeds SVG, PNG, PDF, bounds, and composite
  recording; private PyO3 DTOs and Qt consume the same received facts. The
  pre-existing `DocumentRenderPlanV1` remains the distinct whole-document plan
  grammar. `RenderObservationV1` remains the stable document/projection receipt
  envelope and contains V2 molecule plans, so its suffix is not a compatibility
  alias. This foundational capacity does not yet author or depict ordinary `w1` or
  `h1` bonds and does not add Next presentation UI.
- Permanent evidence covers finite closed-path admission and rejection, shared
  SVG/PDF/bounds lowering, PDF structural accounting, frozen PyO3 DTOs, and Qt
  source-path copying/hit behavior. Fresh self-contained wheel/site checks and
  broad visual or artifact inspection are disposable implementation evidence;
  acceptance has no byte, pixel, timing, or operation-count gate.
- Output boundary: PNG requires exact nonzero caller-owned dimensions, an explicit
  transparent or RGB background, and raw-RGBA and encoded-artifact caps. The raw cap is
  admitted before pixmap allocation and a bounded writer enforces the encoded cap.
  PDF requires caller-selected structural limits for plan traversal, draw-path commands,
  and exclusion-report UTF-8 before `pdf-writer` allocation, plus a nonzero post-build
  completed-artifact nonpublication cap. The latter is not an allocation or process-
  memory bound. Every V1 sink receives explicit butt caps, miter joins with fixed 4.0
  bevel fallback, and even-odd fill; no sink inherits those appearance defaults.
- Dependencies: the lock-reviewed in-process sink surface is `tiny-skia`/`tiny-skia-path`
  (BSD-3-Clause), `png` and `pdf-writer` (MIT OR Apache-2.0), with no build scripts or
  native-graphics linkage. Their source/licensing and internal-unsafe boundary are in
  [docs/PROVENANCE.md](../PROVENANCE.md). A native graphics library remains a separate M20
  packaging decision.
- Permanent evidence: focused offline Cargo tests cover common lowering/order/provenance,
  named exclusions, explicit paint, semantic resource failures, and SVG structure. The
  2026-08-13 independent review passed 70 `ferrum-render` tests, fmt, clippy, docs, and
  locked-offline macOS arm64 checking. A disposable current-source proof used one A4 page
  with six recognizable supported roots, produced an 800 x 1131 PNG and one-page PDF,
  verified equal SVG/PNG/PDF receipts, PNG decoding, and `qpdf`/`pdfinfo`, then visually
  inspected both raster outputs. It reported no exclusions. These checks establish
  semantic structure, dimensions, and recognizability only; they do not create byte,
  pixel, timing, or perceptual thresholds.
- M14's M13 dependency is satisfied. M13 does not supply a file, CLI, PyO3, Qt, CD-SVG,
  RDKit, or cross-platform-pixel-equivalence route; those belong to later boundaries.
- Parallel-plan ready: no.

### Milestone: M14 Haworth

- Depends on: M5, M13.
- Status: done. The first public slice is the separate generic single-ring,
  read-only, revision/digest-bound `ferrum-api` observation of one selected direct
  molecule. Its caller supplies the exact five- or six-member C/O cycle IDs,
  anomeric atom, scale, paint, and width. After proving the selected direct root
  before core resolution, Ferrum returns the durable root identity and document-root
  order, finite template-local bounds, and a molecule-local Haworth plan. The plan's
  selected bond order is molecule-local; it is not a claim about document stacking
  order or a public entry to the two-ring direct-glycosidic renderer below.
- The accepted `ferrum-domain` direct-glycosidic topology slice classifies two
  supplied, revalidated, vertex-disjoint five- or six-member C/O ring receipts plus
  one exterior degree-two oxygen and its two selected single, non-aromatic bridge
  bonds. Each bond attaches the bridge to a selected carbon in a different ring. Its
  owned receipt retains canonical ring facts, the two ring attachments, bridge
  identity, and snapshot-local source-order records.
- The accepted pure-domain local layout consumes that topology and a finite positive
  scale. Canonical ring zero has its attachment at `(-scale, 0)`, canonical ring one
  at `(+scale, 0)`, and the exterior oxygen at `(0, 0)`; the normalized adjacent-edge
  outward direction fixes each local rigid transform. Bridge endpoint pairs are keyed
  by the selected attachment-bond identity, not graph source order. The accepted
  fragment lowering owns exactly the two ring vertex sets plus bridge oxygen and
  exactly the two ring cycles plus bridge bonds. Its disjoint ring/bridge bond maps,
  endpoint identities, and endpoint geometry partition those selected facts exactly;
  ring substituents are absent.
- The accepted pure-domain `DirectGlycosidicHaworthDepictionSpecV1` lowers one
  checked fragment to owned depiction facts. Per canonical ring it assigns exactly
  one `q1`/front bond, its two canonical-cycle neighbours as directed `w1`/front
  shoulders from the outer endpoint to the shared-q endpoint, and every remaining
  cycle bond as `n1`/back. The `q1` and `n1` endpoint pairs retain canonical cycle
  order. The bridge remains an ordinary separately typed bond with no Haworth role,
  style, or depth. Snapshot-local source order is copied only as provenance, never
  as map, child, or paint order.
- The accepted `ferrum-render` direct renderer consumes the depiction spec through a
  crate-private draw stream and emits local structural SVG in one closed order:
  ordinary ring and bridge bonds, then round-cap `q1` strokes, then directed rounded
  filled `w1` wedges. It is an in-process local renderer, not a document or page
  compositor; it adds no public PNG/PDF, API, PyO3, Qt, CLI, transport, or file route.
  Offline semantic and SVG-structure tests cover its bounded rendering behavior.
- These domain receipts deliberately own no placement on a page, document or
  session, API, transport behavior, PyO3, Qt, CLI, parser, stereochemistry, RDKit,
  or OASA behavior. In particular, the depiction spec neither authors nor serializes
  CDML; it only uses the closed vocabulary defined in
  [CDML_FORMAT_SPEC.md](../CDML_FORMAT_SPEC.md#direct-glycosidic-haworth-profile).
  These receipts remain topology/layout/fragment/depiction infrastructure, not a
  native glycosidic insertion, a named sucrose preset, or a drawing convention.
- This observation deliberately has no mutation, page placement or complete-document
  replacement, source-to-template transform, CDML rewrite, stereochemistry or sugar
  inference, PyO3, Qt, CLI, RDKit, or OASA route. It cannot convert to an M13
  `DocumentRenderPlanV1`; a later composition boundary must choose a page/placement
  contract and declare complete coverage and omissions. Haworth topology currently
  consumes only core atom elements and bond topology/order, not CDML coordinates,
  style, `haworth_position`, labels, charges, isotopes, hydrogens, wedges/hashes, or
  carbohydrate naming/stereochemistry.
- Core projection now records a known parsed aromatic order as `Some(true)` and every
  other known order as `Some(false)`; unknown order remains unknown. This corrects
  the normal non-aromatic fact required by the existing ring validator without
  inferring chemistry from an unknown order.
- Permanent evidence is compact offline Cargo semantic testing of provenance/root
  identity, local bond order, finite bounds and line batches, canonical
  rotation/reversal, stale or invalid selections, typed topology failures,
  direct-glycosidic ring/bridge classification, canonical local placement, exact
  selected atom/bond partition, endpoint geometry, depiction roles/direction, and
  snapshot non-mutation. There is no one-time pixel, coordinate, timing, or byte
  claim for the depiction spec. The one-time read-only OASA topology receipt
  establishes the two-ring exterior-oxygen profile and its boundaries; it is not a
  permanent dependency or byte, pixel, coordinate, timing, network, RDKit, or OASA
  parity gate.
- Deliverables: spec, layout, fragment layout, renderer.
- Exit criteria: source-backed topology, explicit depiction-spec, fragment layout, and
  the bounded local direct renderer are accepted at the M14 owning boundary. One
  disposable visual review records recognizable two-ring chemical meaning and the
  `q1`/`w1` drawing distinction; it establishes no byte, pixel, coordinate, timing, or
  perceptual threshold. Direct-glycosidic document authoring, page placement, session
  commit/history, CDML persistence, and whole-document composition are deferred to an
  explicit M16 session-authority slice.
- Parallel-plan ready: yes.

### Milestone: M15 domain utilities

- Depends on: M5.
- Status: done. Ferrum retains only three bounded M15 user workflows: supported
  peptide-template insertion, linear-form conversion, and Clean Geometry plus
  the five native geometry repairs. The remaining historical utility families
  have an explicit pre-production drop rather than a compatibility promise.
  The first public utility is the pure in-process
  `ferrum-api::inspect_peptide_sequence_v1(&str)` boundary. It accepts one strict
  canonical uppercase one-letter sequence and returns an owned V1 receipt with its
  canonical sequence, supported alphabet, `u64` residue count, and N-to-C ordered
  one-/three-letter residue facts. Empty input and the first unsupported Unicode scalar
  map to closed public errors without losing the one-based scalar position. Proline is
  a supported residue.
- Scope: this inspection does not normalize input or create termini, a structure,
  molecule, SMILES, mass, pI, document operation, external-input reader, FFI/RDKit/OASA
  call, PyO3 binding, or CLI command. A future CLI, Python, clipboard, document, or
  network ingress owns its explicit text-resource policy before calling this allocated
  in-process API; M17/M18 own the versioned protocol and public CLI/Python contracts.
- The second accepted peptide slice is the pure-domain
  `build_legacy_peptide_template_smiles_v1(&PeptideSequence)` compatibility profile.
  It produces owned deterministic historical template SMILES for `ACDEFGHIKLMNQRSTVWY`,
  including charged termini, and reports proline as a typed one-based unsupported
  template residue while keeping proline valid for sequence inspection. The bounded
  native insertion integration is now a separate, deliberately narrower profile:
  `ferrum-native-peptide-template-insertion-v1` accepts exactly
  `ACDEFGIKLMNQRSTVY`. It admits exact unmodified uppercase/no-space text through an
  API-owned 33,824-byte budget, rejects H/P/W as typed profile failures before the
  legacy builder, native path resolution, or library load, and uses the concrete
  authenticated `NativeChemEngine` only after preflight. It produces an ordinary
  revision/digest-fenced insertion; it neither changes the pure-domain 19-residue
  compatibility profile nor claims generic peptide or OASA parity. H/W remain
  excluded until a future aromatic/explicit-H contract is designed.
- The accepted linear-form slice is the pure
  `ferrum_domain::linear_form::plan_linear_form_v1` planner plus one Rust-owned
  document/session/API transaction and an ordinary-native Qt action. Its named
  `linear-form-direction-v1` contract deliberately starts a multi-atom path at the
  lower durable direct-child source-order endpoint, rather than reproducing OASA's
  process-local coordinate/object-identity tie break. Rust alone owns the induced
  simple-path decision, fixed 10-point replacements, uniquely anchored exterior
  translations, explicit hydrogen visibility, exact generated fragment grammar,
  allocation, atomic history, no-op classification, and later semantic retirement.
  Qt maps current durable atoms/bonds to one opaque direct root, expands bond endpoints,
  submits the source-ordered atom tuple synchronously, installs only a changed
  authoritative observation, and restores accepted atom selection. The private PyO3
  seam and typed error remain absent from `.pyi`, CLI, serde, and wire contracts.
  Compact domain/document/API/binding/Qt semantic tests and a disposable OASA direction
  oracle establish the contract. This retires the ordinary native linear-form gap only;
  known-group expansion, substructure search data, and broader chemistry checks
  are recorded below as pre-production drops.
- Permanent evidence is compact offline Cargo semantic testing of inspection facts,
  template grammar, distinctive residue behavior, typed syntax/profile failures, and
  the bounded native operation. A small read-only OASA comparison plus current-artifact
  singleton, mixed-sequence, and ANKLE checks are one-time implementation evidence only.
  No fixture corpus, subprocess, network, native adapter, OASA/RDKit, timing, or opaque
  exact-size/count test is retained as a permanent gate.
- Deliverables: an explicit disposition for the remaining sugar code, repair,
  known-group, and substructure-search families. A newly adopted utility needs a
  source-backed native contract; an unadopted family is an honest pre-production drop
  with no menu, CLI, Python, or OASA fallback.
- Exit criteria: each adopted utility has source-backed semantic acceptance at its
  owning boundary; each unadopted family has one recorded product disposition. No
  differential corpus or permanent test is required solely to justify a drop.
- Closure: the compact-sugar parser, descriptive sugar-name catalog, known-group
  catalog, and biomolecule/system-template catalogs were removed as unshipped
  code. Substructure search, oxidation, generated names, and broader chemistry
  utilities are pre-production drops. A future workflow must introduce its own
  product contract; M17/M18 decide any public protocol, Python, or CLI surface.
  Cargo checks, source scans, and installed workflow observations were one-time
  migration evidence, not permanent count, timing, equivalence, or packaging gates.
- Parallel-plan ready: yes -- one work package per utility.

### Milestone: M16 full session boundary and adoption

- Depends on: M10.
- Deliverables: the complete document core exposed through the supported session
  boundary; one ordinary native window for each supported route; user-facing typed
  refusals for unsupported inputs; and documented pre-production drops. Scenario
  observations may inform later regression policy but do not supply an arbitrary gate.
- Exit criteria: one Rust-native `MainWindow` owns every supported document route;
  every historical route is supported, explicitly refused, or explicitly dropped; and
  independent current-tree review finds no dangling product host/worker/codec owner.
  This milestone adds no document semantics -- M9 owns those and this publishes them.

#### 2026-08-15 compatibility-host retirement checkpoint

The compatibility-host checkpoint rereview is ACCEPT. The explicit OASA host and its
session, action/mode/worker/codec/projection island are removed with the production OASA
dependency declarations. The public desktop has one ordinary Rust-native `MainWindow`.
Supported document classes remain bounded native contracts; CDXML, CML, `.cdsvg`, `.svgz`,
and compressed CDML are clear pre-read refusals, while PubChem and unported legacy
templates, modes, repairs, properties, and clipboard variants are explicit pre-production drops.
`Recovery Export CDML...` remains a current-document recovery copy, not a converter.

The retained permanent suite is native semantic behavior plus one representative actionable
CDXML refusal/nonmutation test. Source/package scans, an OASA-absent build/site launch, and
the accepted installed ordinary-window walkthrough are disposable integration evidence. The
walkthrough directly observed startup without OASA or compatibility UI, public CDML Open, Radical
edit semantic Undo/Redo, retained Recent/Export/Recovery labels, and real CDXML modal nonmutation.
Save/reopen and artifact publication remain separately accepted evidence. This checkpoint neither
freezes M17/M18 interfaces nor, by itself, closed M16/M19/M22.
- The ordinary native root now has one deliberately narrow standalone Haworth insertion route for
  four explicit D-glucose recipes: alpha/beta D-glucopyranose and alpha/beta
  D-glucofuranose. Rust owns the literal recipe and finite 40-point local geometry, complete
  candidate/ID allocation, CDML authoring, revision/digest-bound one-use receipt, atomic history,
  selection, and ordinary whole-document V2 rendering. The chemistry records C6O6 and twelve
  single heavy-atom bonds. Pyranose closes `O5-C1-C2-C3-C4-C5`; furanose closes
  `O4-C1-C2-C3-C4` and its exocyclic chain is `C4-C5-C6`, correcting the earlier mistaken O5
  furanose closure. The source-backed terminology and anomeric orientation follow
  [IUPAC carbohydrate nomenclature](https://iupac.qmul.ac.uk/2carb/noGreek/05.html),
  [IUPAC Blue Book P-10](https://iupac.qmul.ac.uk/BlueBook/P10.html), and
  [PubChem D-glucofuranose](https://pubchem.ncbi.nlm.nih.gov/compound/D-Glucofuranose).
  Qt owns only the readable form/anomer chooser, one captured shared-snap anchor, and an exact
  receipt-derived preview. The front C2-C3 `q1` and directed C1->C2/C4->C3 `w1` shoulders use the
  normal V2 round-cap/front-layer and filled-front-wedge lowering; remaining ring edges are `n1`
  back edges. The private receipt is absent from public Python, CLI, serde, and wire surfaces.
  QSettings and CDML receive no Haworth UI state. Compact offline recipe/transaction/render tests
  and visible product behavior are permanent; source captures, OASA semantic comparison,
  wheel/site runs, screenshots, and visual/accessibility walkthroughs are one-time evidence.
  Generic codes/catalogs, other sugars, fusion/attachment, rotation/reflow, repeated placement,
  and general stereochemical inference remain explicit future contracts.
  An accepted current-source/installed-site walkthrough exercised all four visible chooser choices,
  O4/O5 chemistry, snap/preview/commit, and no UI metadata in CDML or QSettings. An authoritative
  atom or bond at the raw or snapped location preserves document/selection and retains the armed
  intent for a later empty-page click; Cancel and stale paths preserve state. Public tab undo/redo
  semantically restores history and save/reopen remains valid. Revision advancement on undo/redo is
  expected history behavior. This is disposable integration evidence, not a permanent timing, pixel,
  byte, coordinate, or count requirement.
- Direct-glycosidic Haworth authoring is a separate M16 session-authority slice. It
  accepts complete, explicitly authored no-substituent atom/bond facts and one finite
  caller-selected placement transform; it derives the closed `q1`/`w1`/`n1` and
  Haworth-depth facts, validates the complete candidate, and inserts the accepted new
  molecule once through one revision-bound document operation. The accepted receipt
  retains canonical durable authored facts and the accepted observation. Its sole
  in-process API composition derives the closed render observation from that embedded
  observation, without session re-observation, then authenticates source `PersistentId`
  through the exact projection `DocumentObjectId` and root order. The opaque non-serde
  composite retains the established whole-document plan, suppresses only selected bond
  outcomes, preserves atom masks/labels, nonselected bonds, and issues, and injects the
  ordinary/q1/w1 direct drawing once in private recording-sink traversal. Direct paint
  and ordinary width resolve from the accepted drawing standard; `standard/bond@wedge-width`
  does likewise, with the source-backed 5px fallback only when absent. This adds no public
  SVG, PNG, or PDF overload, Python, CLI, Qt, or wire route. It does not reflow existing
  roots or claim collision avoidance, carbohydrate recognition, stereochemistry inference,
  generic exterior-component transforms, or a named sucrose preset. M17 owns any needed
  wire schema; M18 owns public Python/CLI. Compact offline Cargo semantic tests cover
  authentication, selective preservation, one injection point, and typed sink refusal;
  no one-time visual proof is a permanent M16 requirement.
- The same closed slice now supports durable re-observation only for an explicitly
  selected molecule at an expected revision. After save and reopen, it recovers one
  strict raw persisted C/O profile with exact atoms-before-bonds child order and one
  closed 5/5, 5/6, 6/5, or 6/6 ring form. It reconstructs canonical durable facts and
  uses the existing authenticated selective Rust composition path. A hand-authored
  equivalent valid profile is accepted, so this observation cannot prove historical
  A2b authorship. No persistent marker is added; M17 owns one only if a later product
  requirement needs it. Existing global `InvalidPresentationFact` suppression remains
  unchanged. Permanent Cargo tests cover save/reopen reconstruction, accepted forms,
  profile rejection, stale/foreign/non-molecule non-mutation, and authenticated
  composition. No visual check is required or retained for this native-only seam.
- The accepted checkpoint adds an ordinary, private native route:
  `Chemistry -> Insert Direct-Glycosidic Haworth...`. Its empty structural-SMILES
  dialog and source-owned closure admit only two vertex-disjoint five- or six-member
  C/O rings bridged by one exterior degree-two oxygen: a neutral nonaromatic
  single-bond graph with 11--13 atoms and no other source facts. Rust owns the closed
  graph, one-use receipt, finite authored drawing, IDs, CDML, history, selection, and
  normal Render Plan V2 observation; private PyO3 exposes only typed preparation,
  receipt-derived preview batches, and commit. Qt captures source tab/revision/digest,
  resolves one shared snapped empty anchor, paints the frozen preview, and installs the
  authoritative observation. Occupancy, cancel, stale, tab/close, busy, and failure
  boundaries preserve durable state. This does not claim sucrose, any named sugar,
  anomer/linkage/D/L/stereochemical inference, arbitrary SMILES, attachment/fusion,
  repeated placement, preferences, public Python/CLI/wire, OASA ownership, or a
  special composite render path. SMILES and UI state never enter CDML or QSettings.
  Compact semantic Rust/private-binding/Qt behavior is permanent evidence;
  parser/OASA probes, fresh wheel/site, screenshots, accessibility, visual, occupancy,
  and installed-product walkthrough mechanics are disposable. The sealed installed site passed
  the focused private/public suite (4 passed). Its independent public walkthrough accepted blank
  and invalid inline accessible recovery, pointer-tool cancellation, occupied retry, selection,
  Escape/tab-switch/close containment, Undo/Redo, save/reopen, and normal V2 receipt-only
  installation without a durable feature marker. FQ-013 remains partial for its other
  explicit deferrals; M16/M19 status does not otherwise advance.
- The ordinary native File menu now adopts the existing Rust document-artifact path as
  `Export...`, with SVG, vector PDF, and transparent PNG at one output pixel per Rust page point.
  Qt captures one active immutable `SessionDocumentObservationV1`, revision/digest, and opaque
  local-origin token before its chooser, then rechecks the same current/idle fence before a QThread
  sends only the observation to private Rust preparation. Rust derives the complete document plan,
  refuses an excluded root rather than writing a partial artifact, and returns owned bytes plus
  closed provenance. Qt reauthenticates the delivery before invoking descriptor-relative Rust
  publication. A local CDML or decoded CD-SVG tab retains a live Rust source descriptor; the
  publisher rejects its original source or an observed hard-link alias without lexical path
  comparison or a Python-held file. The PNG label names page geometry, not an encoded-density
  metadata promise. Cancel, stale delivery, close, and conflicting native work make no publication
  claim; confirmed, directory-entry-unconfirmed, not-started/rejected, and possibly-published
  outcomes retain distinct recovery. The ordinary Qt route contains no detached scene, Qt writer,
  `QSaveFile`, or OASA fallback. Open, Open in Current Tab, export, and close use reciprocal busy
  containment. Three compact private-bridge behavior tests are permanent: provenance refusal with
  no session mutation, guarded local publication with no session mutation, and retained-origin
  hard-link refusal. Fresh wheel/site, decoder/signature/dimension, visual/a11y, busy-race, and
installed-window walkthrough checks remain disposable. This does not adopt CD-SVG export/round
trip or public Python/CLI/wire contracts; those require an explicit M16/M19 disposition, so
the compatibility-host checkpoint remained in progress.
- Parallel-plan ready: no.

The following pre-closure material records the shared-scene prerequisite: every
API/PyO3 molecule render entry carries the owning document-root molecule identity
and source order around the existing molecule-local render plan. Ferrum-Qt owns one
disposable graphics root per molecule, places that root at the backend-issued
document position, and keeps atom/bond source order local to the root. This removes
the former flat-scene ambiguity. Supported Rust-projected polylines now remain
independent top-level roots and share that scene ordering with molecule groups;
multiple presentation roots no longer sit behind one aggregate Qt group. The same
path now retains every ordered point after the required first two, enabling
multi-segment vectors and rectangular bracket roots without endpoint-only
approximation. Rectangles, squares, ovals, circles, and polygons now use the same
class-aware root projection with finite normalized bounds or ordered points and
explicit stroke/fill provenance; Qt adds no palette, shape, or appearance
fallback. Semantic geometry, appearance, scene ownership, and order are the
permanent gates, not XML bytes, pixels, or an arbitrary timing threshold.
Normal non-spline arrows now cross the same boundary with a backend-derived
shortened axis, validated head dimensions, filled head polygons, and explicit
stroke; specialized or spline arrow families remain typed issues rather than
visual fallbacks. Fixed-content plus roots now resolve their authored anchor and
appearance in the document projection, then cross an API-owned verified-Telex
layout as an exact glyph identifier, supplied origins, explicit paint, and
centered ink bounds. Qt paints those cached outlines without shaping, advancing,
measuring, or selecting a fallback font; an authored family becomes a typed
unsupported issue. The bounded native edit now sends one selected direct-root
Plus through a closed revision-bound Rust patch. Rust owns the unique family,
integer-size, foreground, and optional-background changes, detached candidate,
history, and authoritative result. The current Qt form exposes only integer size
and foreground because those controls preserve the projected facts exactly;
fractional source sizes are rejected instead of rounded. Selection returns by
durable document identity after the replacement render installs. Permanent gates
are semantic and offline; the public-dialog current-wheel exercise remains a
one-time probe rather than a pixel, XML-byte, wiring, or timing test. The same
durable selection path now includes supported vector presentation items. One
selected normal non-spline Arrow may edit start/end heads, a form-representable
width, and color through a closed five-field Rust patch. Rust also owns spline
intent, but the native dialog disables that control until spline rendering is
available, so the UI cannot author a fact it would then omit. This does not yet
claim specialized arrow rendering, ordinary MainWindow routing, or the complete
capability-matrix cutover. Direct-root Text now has a bounded display path: Rust
owns its anchor, identity, resolved appearance, multiline character data, and
closed rich-run grammar; the renderer supplies exact regular-Telex glyph IDs,
positions, scripts, paints, and bounds; Qt only caches and paints those facts.
Font-family requests, bold/italic faces, missing glyphs, and malformed rich
fragments remain typed issues rather than fallbacks. Permanent tests cover
semantic runs, failure containment, frozen DTOs, and durable selection. The
current-wheel install and offscreen scene exercise remain one-time rebuild
evidence, not XML-byte, pixel, timing, or wiring gates. One selected durable
Text now edits its complete baseline/subscript/superscript run sequence,
integer size, and foreground colour through one closed revision-bound Rust
patch. The detached candidate retains unrelated namespaced content and normal
history semantics. Qt visibly disables bold, italic, and font-family controls
until the verified renderer supports those faces, so the native form cannot
author an immediately omitted object. Permanent gates remain semantic and
offline; the wheel install and public-dialog launch are one-time evidence.
A complete selected set of durable direct-root presentation objects can now be
deleted through one closed record-kind operation. Qt resolves every authenticated
rendered identity to a Rust-projected authored selector; Rust resolves and
revalidates every selector, exact kind, direct-root owner, bracket relationship,
and expected revision before committing. A complete bracket pair deletes in the
same history entry, while partial pairs and duplicates are rejected atomically.
Permanent gates cover semantic removal, history, preservation, wrong-kind and
stale rejection; the wheel install and public-action launch remain one-time
evidence rather than wiring or timing gates.
The same exact selector boundary now owns Bring to Front, Send to Back, and
Reverse Selected Slots. Rust retains selected source order for front/back moves,
reverses only selected element slots, preserves non-element root slots, and
requires complete bracket-pair selection. Wrong-kind, partial-pair, duplicate,
stale, and semantic no-op requests are atomic. Permanent tests inspect source
order, history, preservation, and durable selection; the public action launch is
disposable evidence.
That boundary now also owns translation, positive scaling, both axis mirrors,
and the six top-level alignments for complete durable molecules and supported
presentation roots. Scaling and mirrors use one aggregate selection-center
pivot. Rust validates the exact kind, direct-root identity, complete bracket
selection, persistent coordinate grammar, scale factors, finite result, and
expected revision before committing one detached candidate. After a real move,
Rust recognizes only the exact narrow backend-generated `linear_form` grammar
and retires it only when the transformed molecule no longer satisfies its path;
richer, foreign, malformed, and historical fragments are preserved. Native Qt
recognizes a molecule only when every durable atom is selected, restores the
disposable atom/root selection after replacement, and keeps scale's targets and
revision fixed while its modal form is open. Permanent gates cover semantic
geometry, history, identity no-op, narrow metadata ownership, and whole-request
rejection. Temporary wheel and offscreen native-tab runs remain one-time rebuild
evidence. The native rotation interaction now has the distinct preview semantics
described below; the implemented repair boundaries follow it.
The standalone native window also exposes complete-root translation as a
revision-bound pointer gesture. At press, a private immutable Rust
`TopLevelTranslationAnchorV1` receipt captures canonical complete selectors,
source revision/digest, and the finite lower-left union of authored-coordinate
bounds. The raw typed-document geometry helper is crate-private; the receipt
is observation-only and stays outside CDML, history, and preferences. The
captured View snap boolean resolves enabled movement as
`snap(anchor + raw_delta) - anchor`, and disabled movement as `raw_delta`.
That one finite rigid delta drives both projection-only dashed feedback and the
existing Rust translation; projected bounds remain non-authoritative. Escape,
stale provenance, tab change, and teardown retire the preview without
submission. Accepted replacement restores durable selection and established
undo/save/reopen behavior. Compact semantic receipt and public gesture tests
are permanent; current-wheel, overlay, screenshot, accessibility, and cache
evidence is disposable. Rotation and exact existing-atom joins remain separate
input contracts.
Selected-atom rotation now has that distinct Rust document/session/PyO3
operation: durable molecule/atom pairs, a scene-point center, and a radian angle
are validated before one detached commit. It preserves z, uses the documented
0.001 cm authored precision, and applies the same narrow generated-form validity
rule. The standalone native window now captures only durable atom identities and
immutable projection positions, derives the selection center, and paints a dashed
Qt-local atom/bond skeleton while the pointer moves. It retires the skeleton before
submitting one still-current Rust operation on release, restores durable selection
only after replacement, and cancels without persistence on Esc, stale provenance,
or tab teardown. It never moves immutable render-plan items or mutates a Qt document
model. Historical Rotate-mode behavior was not retained; the supported native rotation
contract is the product route.
The document/session/PyO3 repair envelope now implements
`normalize-bond-lengths`, `normalize-bond-angles`, `normalize-rings`, `snap-to-hex-grid`, and
`straighten-bonds`, delegating
to pure-Rust planners after every
selected durable direct-root molecule has entered one validated coordinate
graph. It converts CDML y-down coordinates to geometry y-up and back once,
preserves z and opaque content, writes only changed direct atom x/y axes at the
documented authored precision, and treats an unchanged result as a no-op.
Terminal straightening moves only degree-one endpoints, preserves nondegenerate
lengths, uses increasing-angle 30-degree ties, and anchors an isolated pair by
lexical durable atom ID. Its common spacing value is validated but unused. The
older domain `Straighten` operation remains a distinct whole-depiction rotation.
Length normalization fixes ring coordinates, grows acyclic substituent trees
from their ring anchors, and selects ring-free roots by highest degree with a
durable-ID tie. Original nondegenerate directions are retained and coincident
eligible bonds use the upstream eastward fallback. Ring normalization uses a
canonical durable-ID walk,
preserves centroid and first-atom authored orientation across the y-down/y-up
boundary, and rigidly translates each singly anchored acyclic component. Angle
normalization fixes ring atoms and the first anchored edge, assigns movable
children to distinct nearest 60-degree slots in authored bond order, preserves
nondegenerate lengths, and uses explicit spacing only for coincident atoms.
Incoming and fixed-ring directions reserve slots; ambiguous multiple anchors and
exhausted slots fail atomically. `clean-geometry` is the sixth implemented kind,
but deliberately does not use the pure-Rust local-repair planner. Ferrum validates
every selected bonded molecule and supported graph fact before crossing the narrow
`ChemEngine` boundary, requests fresh coordinates from packaged ABI-4 RDKit, and
returns one handle-free revision-and-digest-bound multi-molecule batch. The document
session validates the whole batch again and commits only changed direct x/y axes in
one history entry while retaining source centroids, explicit target bond length, z,
opaque content, identity, and source order. The standalone native Repair menu maps
selected durable atom/bond identities to their Rust-projected molecules, treats an
empty selection as all durable molecules, asks the user for grounded explicit
spacing, and uses the existing cancellable coordinate worker for clean geometry. It
does not import OASA, reconstruct CDML in Qt, or persist a fallback spacing.
The same selection path now exposes
one detached vector-appearance form for rectangles, squares, ovals, circles,
polygons, and ordinary polylines. Rust owns its unique width, stroke, and
shape-only fill changes, validates retained geometry, and commits one detached
candidate. Semantic equality is history-free, and the permanent gates inspect
resolved facts, preservation, selection, and atomic failures rather than XML
bytes, pixels, wiring, or timing. Specialized Wavy polylines now cross as a
distinct projection kind while retaining their durable polyline identity. Rust
publishes their exact authored point path and resolved stroke, and Qt connects
those points without regenerating or smoothing the wave. The dedicated native
form sends only representable width and line-color changes through a closed
two-field Rust patch; durable selection returns after reprojection. Permanent
gates inspect authored-point retention, semantic appearance, selection, and
atomic failure. A real public-dialog current-wheel run remains one-time evidence,
not a pixel, XML-byte, wiring, or timing test. Wavy creation is now also
Rust-owned: Qt submits current provenance and two finite endpoints, while a
prepared operation applies the established bounded zigzag policy, allocates the
durable presentation ID, authors the full point path/default stroke, validates
the detached candidate, and commits once. Qt's straight drag preview is
disposable. Permanent gates cover semantic projection, history, durable
selection, and atomic rejection; a real public-window drag remains one-time
current-wheel evidence.

Bracket creation now follows the same ownership boundary. One prepared Rust
operation accepts an exact revision, a closed rectangular/round style, and four
finite normalized bounds values. It allocates two durable polyline identities,
derives the established proportional control points, materializes the effective
drawing-standard stroke, validates the detached candidate, and commits the pair
once. The projection publishes pair ID, ordered member IDs, style, and common
appearance, so Qt never reconstructs persistent pairing from spatial proximity.
Separate rectangular and round actions send the exact closed style and a finite
normalized drag box through that same operation. Round pairs cross as a distinct
closed root kind, and Qt constructs each cubic side from the four Rust-issued
points without parsing CDML or applying a pixel tolerance.
Selecting both rendered members exposes one common width/color edit through the
existing vector form. Rust revalidates the exact pair, member geometry, retained
appearance, and revision before updating both ordinary polylines in one detached
candidate. Permanent gates inspect semantic geometry, identity, appearance,
history, selection, cubic path kind, and atomic malformed/stale rejection.
Public-window drag/edit and source-current wheel builds remain one-time
implementation evidence, not pixel, XML-byte, private-wiring, network, or timing
tests.

The fixed CDML paper-name catalog and its millimetre dimensions are now owned by
`ferrum-document` and cross PyO3 as frozen values. Qt scene setup, snapshot
rendering, and the transitional session catalog query consume that table without
an OASA catalog lookup. Permanent tests cover structural invariants and meaningful
lookup behavior; the complete table comparison to the read-only OASA oracle is
one-time rebuild evidence. The first direct core paper and viewport now join the
same Rust document observation with revision and digest provenance. The
standalone native editor maps the existing intent-only form to one seven-field,
revision-bound Rust operation; detached mutation preserves opaque content, later
paper records, source order, and history. Stable tests cover semantics and
atomicity. A source-current direct-wheel offscreen edit/undo/redo run remains
one-time implementation evidence rather than a byte, pixel, wiring, or timing
gate. The observation now resolves the oriented physical page into the same
72-point-per-inch scene coordinates as document geometry, including a typed
compatibility issue and A4 portrait display fallback for malformed preserved
paper facts. The native scene owns one noninteractive palette-local page behind
the document roots. Its inspected offscreen image remained disposable evidence,
not a permanent pixel golden. This does not claim normal-window session adoption.
One durable molecule can now cross an exact-observation Standard or Fixed-H InChI
export boundary. Rust proves a direct projection root before core lookup and
validates the complete graph before loading the
packaged ABI-4 adapter, then returns the canonical identifier with the frozen source
revision, digest, molecule identity, and closed mode. The ordinary native window
runs only the adapter work off the UI thread and discards a result after document,
tab, or provenance drift. Permanent tests cover graph conversion, unsupported-fact
rejection before FFI, mode routing, and frozen provenance. The current-wheel public
window and clipboard exercise was disposable evidence, not a private-worker, byte,
pixel, timing, or network gate. The ordinary window also exposes explicit Standard
and Fixed-H `.inchi` file actions. Each freezes the chosen root and source state
across its destination dialog, reauthenticates the owned native receipt, and
publishes exactly one newline-terminated identifier through the descriptor-relative
Rust artifact writer. It never adopts the path or changes the clipboard, session,
history, scene, or selection; publication uncertainty remains typed and visible.
The runtime-private file seam adds no CLI, wire, or stable Python contract. Other
document codecs need an explicit M16/M19 support, refusal, or drop decision.
The ordinary native root now also replaces the historical compatibility
`_gen_smiles` behavior for an atom or bond selection that maps globally to exactly
one durable direct-root molecule. Rust authenticates the exact observation and
opaque root, rejects every unsupported source graph fact before native loading,
and sends the complete frozen graph through the optional ABI-4 SMILES-writer
capability. That adapter owns the explicit canonical-isomeric RDKit profile and
returns one bounded printable ASCII line; the immutable receipt retains its source
revision, digest, molecule identity, schema, and profile. Qt shares one cancellable
molecule-export intent with InChI and requires the same active registered tab plus
the same selected root, revision, digest, and receipt before copying or displaying
selectable text. Its sibling `Export SMILES File...` action freezes those facts
before the destination dialog, reauthenticates them after it, and publishes the
immutable receipt as exactly one newline-terminated `.smi` record through the
descriptor-relative Rust artifact publisher. Confirmed, directory-entry-unconfirmed,
rejected, not-started, and possibly-completed outcomes remain distinct. Stale,
switched, cancelled, mismatched, and typed failure outcomes do not reach the
clipboard or file publisher, and neither route changes the session, adopted path,
history, scene, or selection. Older ABI-4 adapters remain loadable and report the optional writer
unavailable. Permanent Rust, installed-binding, and ordinary-window tests cover
the supported result, pre-native refusals, and lifecycle containment; the rebuilt
adapter call is disposable implementation evidence. This is not multi-record
`.smi`, depiction-stereo inference, a public PyO3/stub, CLI, wire route, or an
OASA fallback. The remaining file-codec ledger stays open M16/M18 work.
The same ordinary selected-root authority now exports explicit V2000 or V3000
Molfile syntax. Rust authenticates the immutable observation and direct root,
freezes the complete supported graph with atom-aligned coordinates, and rejects
unrepresentable graph or drawing facts before locating the packaged adapter. The
document-to-chemistry boundary owns the inverse of insertion placement: CDML and
Qt downward-positive y coordinates become upward-positive Molfile coordinates.
An authored molecule name is retained through a separate optional ABI-4
title-aware operation and installed on the native molecule before RDKit writes
either syntax; Ferrum neither blanks it nor edits the returned text. Unnamed
molecules continue through the older optional Molfile operation, and an older
ABI-4 adapter reports only the title-aware operation unavailable. The native
writer result is retained byte-for-byte in a receipt with revision, digest,
root, schema, coordinate profile, syntax, and optional exact title. Qt freezes and rechecks the
same selected root across the destination dialog and accepts only an exact
still-current receipt before invoking the descriptor-relative Rust publisher.
Confirmed, directory-entry-unconfirmed, rejected, not-started, and
possibly-completed outcomes remain distinct; neither syntax adopts the path or
changes session, history, scene, selection, or clipboard. Permanent Rust,
installed-binding, and ordinary-window tests cover both syntaxes, coordinate
direction, exact publication, unsupported-fact refusal, stale selection,
cancellation, and nonmutation. The private PyO3 operation remains absent from the
stub and adds no SDF, public CLI/wire/Python,
stereochemical inference, or OASA fallback. The remaining codec ledger stays open
M16/M18 work.
That selected-root authority now also exports one explicit V2000 or V3000 SDF
record. The document layer recognizes only the exact Ferrum SDF-import
namespace and encoding, then returns a fallibly owned blank or exact title and
ordered properties without collapsing duplicate names. Imported metadata is
authoritative; an ordinary molecule instead uses its authored name or a blank
title. The native adapter owns structural Molfile generation and the optional
exact title operation, while Rust chemistry owns the stable property grammar,
one-record envelope, and `$$$$` terminator. Ambiguous property names, carriage
returns, blank value lines, trailing newlines, and a value line equal to the
record terminator are rejected rather than serialized with a different
meaning. Qt freezes and rechecks the same root, revision, digest, syntax, and
selection across the dialog and worker, then publishes the immutable receipt
through the descriptor-relative Rust publisher. Permanent Rust,
installed-binding, and ordinary-window semantic tests cover duplicate imported
metadata, ordinary and blank titles, both syntaxes, exact publication,
staleness, cancellation, re-import, and nonmutation. The private PyO3 operation
is absent from the stub. This is not multi-record selected export, a public
Python/CLI/wire contract. There is no OASA fallback; remaining codec/adoption decisions
stay open M16/M18 work.
The ordinary `MainWindow` now begins as the native-first product root. It creates one
Rust-owned empty-document tab, retains no second session model, and keeps
`File -> New` plus the final-tab zero-page lifecycle on the Rust boundary. `File -> Open`,
programmatic Open, and command-line launch paths now admit uncompressed `.cdml` files
through the same named local-CDML V1 profile as the native render CLI. One private
pre-M18 PyO3 receipt owns the admitted Rust session and its authenticated render
observation; an asynchronous Qt worker prepares both without a Python whole-file read,
then the UI consumes the pair once, verifies their revision and digest, and installs the
scene. Multiple launch paths queue in accepted order. Cancellation and close invalidate
delivery without pretending to preempt an in-flight Rust read, while typed source,
resource, UTF-8, document, and render failures preserve the current document. Slice A now
adds an opaque descriptor-derived origin token to the private one-use local-CDML receipt.
Qt carries it only as a live-tab identity, compares it only after admission, and therefore
activates an existing ordinary native tab for the same regular file, including a hard-link
alias. Immutable Open intents replace only the marked, clean, revision-zero bootstrap
`Untitled` tab after detached receipt authentication, revision/digest/canvas-idle
revalidation, complete new-tab construction, and atomic replacement. Every other
interactive Open and every launch path uses a new tab. A busy source tab remains current
while its admitted tab installs in the background, preserving its active preview until the
user resolves it. The token is not CDML, serialized session/history state, a preference, or
a cross-process identity contract. Permanent Rust/PyO3/Qt tests prove receipt ownership,
bootstrap replacement, meaningful/busy-page preservation, launch routing, and hard-link
activation; wheel/site, visual, route-inventory, timing, and delivery-race observations are
disposable evidence. Slice B adds `FerrumNativeRecentFilesV1`, the versioned QSettings-only MRU of
lexical normalized absolute display paths with no symlink resolution. Confirmed native Open, live-token
activation, and Save promote it; Recent selection forces the same `NewTab` intent, descriptor tokens
remain the stronger live-tab duplicate rule, and Clear changes settings only. Colliding basenames show
parent context with full-path help. Rust-confirmed unavailable/nonregular entries offer default Keep
or explicit removal; cancellation and other failures preserve the entry. Capacity is a tunable usable-menu
policy rather than an exact-count gate. Recent data never enters CDML, standard metadata, Rust
session/history, dirty/save state, selection, receipts, diagnostics, or OASA. The fresh ordinary-window
walkthrough accepts its File cascade and default Keep/explicit Remove recovery before the generic typed
failure. Explicit populated-tab replacement is now `File -> Open in Current Tab...` (`Ctrl+Shift+O`),
not an ordinary-Open expansion. It takes an immutable target fence, admits/authenticates first, activates a
descriptor-token duplicate without touching the target, and atomically swaps a clean saved current native
tab only after immediate revalidation. The same shared target-owned asynchronous-work predicate governs
action reachability, capture, and revalidation, while Open keeps its separate intent. A dirty target receives Save (default), Replace, and Cancel after
admission; named Save uses native publication, unnamed Save uses Save As, and a fresh post-save fence must
pass before the swap. Stale/busy/close/cancel/admission/save failures preserve target selection, tool,
preview, and focus and never fall back to NewTab. Worker finalization defers during the modal decision so a
nested event loop cannot retire the active intent early. Six compact Qt behaviors cover clean swap, dirty
Replace, named and unnamed Save, Cancel/failure preservation, duplicate activation, and one real public
worker disable/preserve/re-enable outcome; keyboard,
accessibility, wheel/site, race, source, and visual probes remain disposable. Independent evidence07
accepted the fresh 20/20 lifecycle, including public active-tool disable/re-enable guidance, stale and
Cancel containment, clean/dirty recovery, hard-link activation, ordinary Open, and Recent composition.
The implemented decoded CD-SVG Open V1 now shares that coordinator for a requested `.svg`.
It admits only a decoded UTF-8 regular SVG with exactly one canonical embedded CDML descendant,
using independent named wrapper and payload envelopes. Rust retains only the validated CDML
payload and mints the same equality-only descriptor token plus a private source kind. Qt retains
source display provenance separately from `file_path`, installs a clean Save-As-only CDML tab,
and preserves the original token after a later CDML publication. The wrapper is never rendered,
fetched, retained, written, or allowed to contribute editable facts. Compact Rust, binding, and
public Qt semantic tests are permanent. Fresh wheel/site, chooser, accessibility, visual, corpus,
and timing observations are disposable evidence; the independent checkpoint rereview and installed
public walkthrough are accepted. `.cdsvg`, `.svgz`, compression, sniffing, wrapper round trip/export, public API/CLI/wire,
and OASA fallback are not product routes. The accepted 2026-08-15 retirement removed
the explicit compatibility host, its legacy session/action/worker/codec/projection
island, and the production OASA declarations. `ferrum_qt.main_window` and
`ferrum_qt.app` now expose the one ordinary Rust-native `MainWindow`; there is no
selector, fallback editor, or second session owner. The host-only mode, renderer,
template, clipboard, property, and codec families are pre-production drops unless a
future product contract deliberately adopts a bounded replacement. The native-only
test boundary retains compact supported behavior and one representative refusal/
nonmutation case; source/package inventories, wheel/site runs, walkthroughs,
screenshots, race checks, and timing observations are disposable evidence.
The ordinary native `MainWindow` also offers `File -> Recovery Export CDML...` for an
exact current Rust document snapshot. It remains reachable for a live registered native
tab while its disposable projection is pending or native work is busy.
Before the destination dialog it freezes tab identity, revision, and digest; afterward it
reauthenticates all three before the existing revision-gated Rust publisher runs. Both
receipt snapshots must corroborate that frozen provenance. Confirmed, directory-entry
unconfirmed, possibly-completed, not-started, and rejected outcomes remain distinct, and
the operation never adopts a path or changes title, baseline, dirty state, history,
selection, scene, or worker state. This is neither ordinary Save/Open nor an external
ingress route: it adds no Rust, PyO3, wire, CLI, or OASA boundary. Compact permanent
semantic tests cover provenance, receipt truthfulness, pending/busy reachability, and
nonmutation; no screenshot, timing, count, or private-worker gate is retained. FQ-002 and
FQ-002 remained partial for plan/ledger reconciliation and closure review, not because ordinary
external CDML lacks a Rust boundary.
The ordinary native tab now also has `Import Supported Peptide Sequence...`. Its prompt
passes exact accepted uppercase/no-space text unchanged to the strict native-17 profile
`ACDEFGIKLMNQRSTVY`; H/P/W fail as typed profile errors before library load. Rust owns
the 33,824-byte ingress budget, preflight, concrete authenticated `NativeChemEngine`, and
frozen insertion; Qt retains only worker scheduling and the existing revision/digest,
cancellation, active-tab-close, window-close, history, save, and reopen fences. A
successful result commits as one ordinary Rust-owned molecule with no OASA mutation or
fallback. This is not generic peptide construction, historical OASA parity, or a public
CLI, wire, or stable Python contract; at that implementation point M15 and M16
were still in progress. Current-artifact
checks of all advertised singletons, one mixed sequence, and ANKLE are disposable
implementation evidence, not permanent CI, count, or timing thresholds. H/W require a
future aromatic/explicit-H contract.
The ordinary native `Chemistry -> Molecule Information...` action is a read-only FQ-010
subset. It accepts one or more selected durable atoms or bonds, never a molecule-root
click. Qt globally matches every literal child source ID and nested source order to one
direct-root projection molecule, refuses ambiguous or unsupported selection, deduplicates
roots, and passes only their existing opaque IDs to Rust. Rust authenticates the frozen
observation and direct roots, retains each accepted source-fact receipt, freezes every
complete graph before resolving native code, and executes one all-or-nothing optional
ABI-4 RDKit composition operation. Per-root results remain in document order; two or more
roots also receive one checked aggregate. The selectable, accessible dialog reports
authored name, source ID, atom/bond counts, lexical inventory, complete-only authored
charge, normalized x/y bounds, isotope- and charge-aware formula, perceived atom counts,
net formal charge, average molecular weight, monoisotopic mass, and average-mass element
percentages. It preserves the document, selection, scene, and history. Qt authenticates
worker/tab lifecycle, revision, digest, root IDs, projection keys, source IDs, and root
orders before display; stale, cancelled, inactive, and closing deliveries remain silent.
The private runtime PyO3 operation is unsupported Qt plumbing and remains absent from
`.pyi`; there is no OASA fallback, mutation, clipboard, CLI, wire, or stable Python
contract. Existing ABI-4 adapters load normally but report this optional capability
unavailable. Permanent Rust, PyO3, and Qt semantic tests cover exact source/engine facts,
multi-root ordering and aggregation, ambiguity refusal, lifecycle containment, and
nonmutation. A sealed RDKit 2026.03.5 wheel plus independently rebuilt adapter passed
installed-extension, ordinary-window, closure, and relink checks as disposable evidence.
Formula and mass do not claim valence analysis, oxidation, group/fragment recognition,
generated names, linear forms, or chemistry checks. Ordinary external CDML Open is now
wired through the shared local-CDML V1 admission profile independently of this feature.
The accepted ordinary-native `Chemistry -> Check Bond Capacity...` route is a deliberately
narrow read-only FQ-010 diagnostic, rather than an extension of Molecule Information. It maps
selected durable atoms or bonds through the same complete direct-root resolver, freezes the
observation/revision/digest/root provenance, and has Rust evaluate only ordinary neutral
nonaromatic H/B/C/N/O/F/Cl/Br/I roots with absent or zero formal charge, no authored
`valency`, `multiplicity`, or `free_sites`, and single/double/triple connectivity. Rust retains
whether formal charge and explicit hydrogen values were authored, computes explicit-H plus
incident bond-order demand, and returns Within Capacity or Exceeds Capacity per atom. A root
outside the grammar receives one Not checked outcome and no partial atom findings; retained bond
depiction is ignored. This is not a general valence or chemical-validity result: IUPAC defines
[valence](https://goldbook.iupac.org/terms/view/V06588) more broadly, and
[oxidation state](https://goldbook.iupac.org/terms/view/O04365/1000) uses a separate
electron-assignment model. Rust owns the closed table, grammar, arithmetic, receipt, and
provenance; a private PyO3 seam transports immutable DTOs and Qt owns only worker containment
and a selectable read-only dialog. The route has no Properties, QSettings, CDML, history,
selection, OASA, public Python, CLI, or wire contract. Compact semantic Rust/API/binding/public
action tests are permanent; fresh wheel/site, visual, OASA, and timing probes are disposable.
Independent evidence09 and checkpoint rereview08 accept the route. Their public real-worker
walkthrough covers supported/no-excess/finding/Not checked reports, authored charge/H display for
every assessed atom in a mixed excess root, document-root order, depiction independence,
lifecycle/nonmutation, and accessibility. This evidence is behavior-facing; fresh wheel/site,
visual, OASA, and timing probes remain disposable.
The ordinary native root now also exposes the bounded FQ-010 mutation
`Chemistry -> Set Molecule Name...`. One or more selected durable atoms or bonds must map
globally and unambiguously to one direct projection root. Qt freezes the tab, revision,
digest, opaque root, and exact child selection across the prompt. Rust reauthenticates that
observation and root before applying one atomic typed operation. The entered text is stored
exactly, including whitespace; empty input removes `molecule@name`, and an unchanged value
creates neither history nor a Qt reprojection. Opaque XML and ordinary Undo/Redo,
Save/reopen behavior remain intact. Its private runtime PyO3 operation stays absent from
`.pyi`; there is no OASA fallback, name generation or normalization, CLI, wire, or stable
Python contract.
The ordinary native root now also owns `Chemistry -> Convert selection to
linear form`. It accepts only a current nonempty durable atom/bond selection that maps
globally and unambiguously to one direct projection root. Selected bonds contribute both
projected atom endpoints; Qt deduplicates and orders the final atom IDs by direct child
source order, then reauthenticates the active tab, revision, digest, opaque root, and exact
expanded selection immediately before its synchronous private PyO3 call. Rust applies
`linear-form-direction-v1`, whose lower source-order endpoint deliberately replaces the
legacy coordinate/object-identity direction rule, and owns path refusal, fixed 10-point
geometry, one-anchor exterior translation, hydrogen visibility, exact generated metadata,
allocation, one history entry, and no-op classification. A changed result installs one
authoritative observation before restoring atom selection; a canonical repeat leaves the
scene and selection untouched. Typed refusal is visible and nonmutating. This route has no
worker, dialog, local Qt undo, OASA fallback, stable Python stub, CLI, serde, or wire surface.
FQ-010 remains partial because broader chemistry checks, oxidation,
groups/fragments, and generated names need an explicit support or drop decision; the
bounded Bond Capacity Check is accepted.
The accepted Explicit Fragment V1 slice narrows that open area to one durable annotation
inside one existing direct-root molecule. Rust creates only `type="explicit"` records after a
revision/digest/root/selection-bound one-use prepare/commit. A nonblank trimmed name may duplicate;
selected direct bonds add their direct endpoints, while all retained members write in molecule source
order. Disconnected member sets are allowed because this is named metadata rather than clipboard
extraction or connected-subgraph recognition. The read-only observation exposes exact supported
records plus a scalar retained-unsupported-metadata notice, preserving richer, foreign, implicit,
linear, and malformed metadata untouched. Qt has only Create/View dialog and lifecycle ownership;
Rust owns ID allocation, CDML, history, and authoritative facts through a private PyO3 seam. No
OASA, QSettings, public Python, CLI, wire, type chooser, delete/rename/member editing/highlight,
groups, templates, inference, or cross-molecule authoring enters this slice. Compact semantic
behavior tests are proposed permanent evidence; installed wheel/site, visual, keyboard/accessibility,
corpus, and timing probes remain disposable. Independent rereview accepted the View-lifecycle and
typed-error repairs; installed public evidence accepted closure/order, duplicates, blank retry,
Cancel/stale containment, retained notice, View lifecycle, undo/redo, and save/reopen.
The ordinary native root now also has a bounded FQ-020 View menu with Zoom In,
Zoom Out, Zoom to 100%, Zoom to Page, and Zoom to Content. Page uses the renderer-owned
paper `sceneRect`; Content derives finite positive bounds from installed projection roots
while excluding only the exact paper root, then falls back to Page. Each native tab retains
its own view transform and scroll position. Every window show and current-tab transition
may request one initial Page frame for an unframed current tab; a queued callback fits only
after exact window/tab/view visibility, registration, current-tab identity, live-scene, and
teardown checks, and a failed delivery clears only its pending request for a later retry.
The controls remain display-only and available while a registered active scene exists,
including pending rendering or chemistry work. They do not read or mutate the document,
session, selection, projection ownership, or worker state. Tab and accepted-window teardown
cancel pending frames before disposal. A permanent right-side status client reuses those exact
five actions through visible `-`, current percentage/reset, `+`, Page, and Content buttons and
adds an accessible continuous 10%-1000% absolute-zoom slider adopted from the newer BKChem-Qt
interface. The status widget only projects action/view state and emits requests; the tab-owned
native graphics view owns exact uniform-transform observation, the bounded zoom contract,
center preservation, and transform-change notification. Consecutive slider changes retain one
stable scene-center anchor without cumulative quantization drift, while real scrolling, resize,
scene replacement, wheel or action zoom, fit, and reset rebase it. Unsupported transforms
disable the slider but leave reset keyboard-reachable. Its visible percentage is rounded to the
same whole value as the integer slider rather than presenting false extra precision. Compact
permanent behavior tests and
adjacent native integration checks provide
semantic evidence without making their counts thresholds. One offscreen widget-tree,
accessibility, and visual run is disposable implementation evidence, not a pixel or timing
gate. The tab-owned native graphics view also adopts the retained vertical-wheel behavior:
one standard notch changes scale by 1.15 within the existing 10%-1000% display range,
keeps the cursor's scene point fixed when scroll ranges permit it, and refreshes the same
status client. The existing Zoom In, Zoom Out, and Zoom to 100% actions also own the retained
`Ctrl++`, `Ctrl+-`, and `Ctrl+0` accelerators. Unsupported transforms refuse the wheel change.
The ordinary window also adopts the newer BKChem-Qt frequent-action toolbar as an interface
improvement and keeps command/document ownership in the existing native actions. One non-movable top
toolbar projects the already-owned New, Open, Save, Undo, Redo, Cut, Copy, Paste, Zoom Out, Zoom to
100%, and Zoom In actions with visible labels, platform icons and standard file/history
shortcuts. It adds no command callback, history, selection, document, or enabled-state owner;
menu, worker, session, and view owners continue to drive the same shared actions. Qt's native
overflow handles narrow windows without an invented breakpoint, and `View -> Main Toolbar`
lets the user hide the client. Permanent coverage exercises a real New command and visibility
choice. Exact action/icon lists, direct shortcut-property inspection, and the wide/narrow
offscreen screenshots were disposable implementation checks rather than permanent wiring,
pixel, width, or timing gates.
The ordinary native root also has a second-row `Editing Tools` toolbar with a stable Qt
workspace identity, visible category header, and `View -> Editing Tools` visibility control.
It adopts the useful BKChem-Qt tool discoverability pattern through honest native owners:
the toolbar projects
the exact established Add Atom, Draw Bond, Draw Wavy Line, Draw Rectangular Bracket,
Draw Round Bracket, Move Atom, Rotate Selected Atoms, and Move Complete Roots actions, plus
one window-owned shared `Cancel Tool` Escape recovery action. Layout, icons, accessibility,
theme projection, native overflow, and visibility are UI clients only. Existing action
callbacks, gesture intents, prerequisites, document/session/history/selection, Rust
mutation, authored-point snap policy, and transform exclusions remain in their current
owners. Cancel Tool composes the existing cancellation boundaries and preserves document
and selection. Compact permanent tests exercise one real toolbar bond gesture, Escape
preservation, distinct Cancel Tool preservation, and restored user-hidden toolbar state.
Wide/narrow screenshots, overflow/icon/grouping/accessibility walkthroughs, workspace
bytes, timing, counts, pixels, and installed-wheel UI smoke are one-time evidence. Legacy
`ModeToolbar`, `SubModeRibbon`, `EditRibbon`, `ModeManager`, and mode YAML remain interface
evidence rather than ordinary product owners. The same toolbar now exposes an always-available
`Next Drawing` client with labelled `Next atom:`, `Next bond:`, and `Next presentation:` controls. It keeps C and
single as application/QSettings defaults, offers the Rust periodic catalog as conventional
spelling suggestions, and accepts valid ASCII-letter plain or pseudo-atom names without making
the picker a closed chemistry vocabulary. Add Atom freezes the selected element for its click;
Draw Bond freezes element, order, and presentation at mouse press. Rust's closed
`DocumentBondPresentationV1` writes Normal `n1`/`n2`/`n3`, SolidWedge `w1`, or HashedWedge `h1`.
For both directed forms, press/start is the narrow tip and release or inserted atom is the wide
base. The V2 renderer owns the filled-wedge path and finite widening hashed-line operations that
both native preview and committed projection consume. Existing-atom joins remain exact; an
empty-space endpoint creates the frozen element and presentation at the shared snap point. Preferences
do not enter CDML or `<standard>`, Rust document state, history, dirty/save state, or selection.
Compact preference, cross-window, projection, history/save-reopen, stale, and cancellation
behavior tests are permanent. An independent current-source keyboard walkthrough accepts Escape
recovery: it restores the visible/effective next element and, during Draw Bond, composes the
shared cancellation action without changing snapshot or selection. Keyboard, visual,
accessibility, overflow, and installed-wheel walkthroughs are disposable evidence. `w2`, `w3`,
`h2`, `h3`, and all other bond styles remain separate Rust-first contracts; broader drawing controls
and shortcut preferences remain open.
The ordinary root now also exposes one closed native ring outcome, `Insert Cyclohexane Ring`.
Its private Rust `DetachedRegularRingInsertionV1` family admits detached saturated-carbon
normal-single rings of sizes 3 through 8 at a finite centre, but the UI submits only C6. The
operation uses the established 40-point drawing side length and fixed flat-top clockwise geometry
in y-down document coordinates. Rust constructs the complete ordinary molecule, assigns IDs,
prepares and authenticates the candidate, owns history and projection, and returns the exact
vertices for Qt's disposable preview. The action captures the shared snap-resolved centre once;
release commits the same receipt. An atom hit is a normal empty-page refusal. Escape, Cancel Tool,
tab lifecycle, and stale provenance preserve the document and prior selection. Corrected
circumradius geometry now preserves the supplied side length. CDML contains only the resulting
ordinary molecule, C atoms, points, and `n1` bonds; no ring metadata, template, orientation,
preference, or UI state is serialized. Compact semantic Rust, private-binding, and visible native
action tests are permanent. Wheel/site, screenshots, narrow-width, accessibility, and visual
walkthroughs remain one-time evidence, not pixel, byte, count, or timing gates. UI sizes 3--5 and
7--8, fusion/attachment, heteroatoms/aromaticity, orientation/rotation, and preferences need
separate contracts.
The newer BKChem-Qt Properties dock is likewise adopted as an interface improvement without
its OASA document model, direct mutations, or local undo stack. A native tab exposes one
frozen current-revision inspection receipt containing the installed Rust document projection
and its disposable selection under matching revision/digest through the dedicated
`ferrum_native_property_observation.py` boundary. The dock derives readable empty,
document, atom, bond, mixed-selection, drawing, and refresh-required views from that receipt.
Its atom and bond buttons are only clients of the existing window-owned native edit actions,
so their established validators, Rust transactions, enabled state, selection restoration, and
history remain authoritative. The dock is movable, hideable through `View -> Properties`, and
adds no fixed-width requirement. One permanent test checks that inspected durable atom facts
follow the active document tab. Literal field lists, action identity, panel indices, widget
counts, widths, and the wide/narrow offscreen screenshots remain disposable implementation
evidence rather than suite gates.
The retired legacy view/mixin/toolbar owners are outside the ordinary product. The ordinary
window now exposes one native
`Options -> Preferences...` surface for the settings its application boundary actually owns:
theme and whether to restore the workspace on a future launch. An accepted theme change uses the
application `ThemeManager`; cancellation preserves the application and document. Workspace
restoration stores QMainWindow geometry plus toolbar and Properties-panel state only after a fully
accepted shutdown. Clearing the choice removes those stored values and keeps later shutdowns from
recreating them. All preference data stays in `Ferrum` / `Ferrum-Qt` QSettings; document drawing
defaults remain an explicit CDML edit and personal UI preferences never become CDML or Rust history.
The ordinary native view now also owns paper-local hex-grid visibility. One disposable overlay uses
the existing bounded Rust geometry bridge and the exact renderer-owned paper rectangle, stays below
document content, follows packaged palette colors, and survives authoritative scene replacement.
Application-owned QSettings choices drive the shared View actions, toolbar clients, and Preferences
checkboxes for grid visibility and snapping. `Ctrl+Shift+G` also toggles the checked snap action.
`FerrumNativeGraphicsView` owns one finite authored-point policy: while enabled it delegates
nearest-lattice math at the same grid spacing to the existing Rust display-geometry bridge, and while
disabled it preserves the finite point. Free atoms, template centroid anchors, empty-space bonded-atom
endpoints, moved-atom targets, Wavy endpoints, and rectangular/round bracket corners all resolve through
that policy. Their previews and commits use the same resolved point. Existing-atom joins remain exact;
rotation is angle input; and complete-root translation now captures a durable Rust anchor receipt while
keeping the snap preference outside the receipt. These application preferences never enter CDML, Rust document
ownership, history, save state, or selection. Permanent coverage keeps semantic enabled/disabled placement
and propagation behavior, accepted/cancelled application behavior, grid state across projection replacement,
document nonmutation, and visible workspace restoration. Offscreen dialog/grid/gesture previews and
current-source wheel review are disposable evidence rather than pixel, size, exact-field, timing, or
private-wiring gates. FQ-020 remains partial for other drawing-gesture and shortcut
preferences and the remaining source-backed window/view capability decisions. There is no retained
Ferrum-Qt or current read-only upstream full-screen action to adopt.
The ordinary native root now also owns bounded FQ-019 `Edit -> Copy`, `Edit -> Cut`, and
`Edit -> Paste` slices. For an
exact current same-molecule atom/bond selection, Rust emits one connected structural
fragment and closes every selected bond over both endpoint atoms; source molecule
metadata and its exact generated linear form remain complete-root facts and are not
copied into the partial fragment. Mixed presentation/structure or multiple-molecule
selection instead emits each complete selected direct root in document order. The
operation authenticates the immutable admitted observation, projection, revision, and
digest, and refuses output larger than the normalized source document. Its private
runtime PyO3 seam stays absent from `.pyi`. Qt owns only literal-scene-to-opaque-target
mapping, cancellable worker scheduling, exact tab/revision/digest/selection delivery
fences, and publication of the Ferrum CDML MIME type plus plain text with the retained
ownership marker. Failure, cancellation, inactive-tab delivery, and teardown preserve
the old clipboard and document. Paste gives persistent meaning to that copied envelope:
`ferrum-document-clipboard-paste-v1` admits only a closed set of durable direct roots
under the same named local-CDML resource profile, allocates fresh collision-safe IDs for
every persistent declaration, remaps exact attribute references to declared IDs, applies
one explicit scene-space translation to the whole inserted group, validates the complete
candidate, and commits it as one authenticated history entry. Its private PyO3 boundary
prepares an immutable handle-free plan off the Qt event thread and returns only the
committed observation plus inserted-root identity receipts. Qt captures preferred custom
MIME or plausible complete-CDML plain text once, fences delivery by the active destination
revision and digest, installs the authoritative result, and selects inserted artwork or
the durable children of inserted molecules. Malformed, unsupported, undecodable, stale,
cancelled, or resource-failed input is nonmutating; any failure after Rust accepts the edit
enters the existing authoritative-refresh state. Acceptance is semantic--fresh identities,
preserved exact references and opaque content, one group translation, atomic history,
selection, Undo/Redo, and reopen--not XML-byte, coordinate-spelling, pixel, timing, or
arbitrary-count equivalence. Cut reuses the same insertion-valid extraction grammar and adds
one source-revision deletion plan. Rust validates the full candidate during worker preparation,
then reauthenticates the current revision, digest, selectors, topology, and projection while
committing one history transition. A structural Cut removes selected bonds and selected atoms
with their incident bonds, retires invalid exact generated-linear-form metadata, and removes the
complete copied molecule root when every atom is selected. A presentation Cut removes complete
selected direct roots, including bracket-pair completeness through the established Rust deletion
contract. Mixed presentation/structure and multiple-molecule selections retain their complete-root
Copy meaning and receive a clear Cut refusal because partial child deletion would disagree with
the copied roots. Qt publishes the admitted fragment before requesting the deletion. A commit
refusal therefore leaves the source unchanged and the fragment available as a usable Copy result;
an accepted edit whose scene installation fails enters authoritative refresh. Permanent coverage
keeps the Rust transaction/refusal semantics, private installed-binding contract, successful public
action, and Copy-fallback safety case. The current temporary wheel is disposable rebuild evidence,
not a shipping artifact or byte, pixel, timing, action-count, or worker-wiring gate.
Selected-SVG copy now has its own read-only Rust contract rather than reusing fragment
extraction or Qt scene cropping. Exact durable atom, bond, or molecule selection retains
the complete authenticated molecule render root; presentation selection retains exact direct
roots. Rust filters one final render plan, refuses any selected profile exclusion, measures
conservative content bounds from the same lowered paths, verified glyph outlines, masks,
ellipses, transforms, and stroke profiles consumed by the artifact sinks, and emits a bounded
content-fitted SVG receipt with revision, digest, canonical selected objects, and retained-root
identity. Unrelated excluded roots do not prevent copying a complete selected subset. The private
PyO3 boundary performs native rendering off the UI thread and remains outside `.pyi`, CLI, serde,
and wire contracts. Qt owns current scene selection mapping, cancellable scheduling,
tab/revision/digest/selection delivery fences, and final `image/svg+xml` plus plain-text clipboard
publication with the existing ownership marker. Failure and stale delivery preserve the existing
clipboard and document. Permanent checks use semantic root/provenance/nonmutation and relative
content-range assertions; the installed wheel rebuild is one-time evidence rather than a byte,
pixel, exact-viewport, timing, or count gate. FQ-019 remains partial for the
remaining clipboard disposition; a broader public Python/CLI/wire surface is an M18
decision, not a prerequisite for the ordinary product action.
The ordinary native root now also owns document drawing defaults for the seven fields
the current Ferrum renderer consumes: line width, atom-label size, line/text color,
label background, multiple-bond spacing, wedge width, and heteroatom-hydrogen
visibility. Rust owns one unique-field patch, creates an absent direct core `standard`
in canonical header order, preserves unrelated attributes, child content, later
standards, and durable selection, and carries the accepted edit through history and
save/reopen. The projection now also retains a valid `standard/bond@double-ratio`, and
the Rust patch grammar covers that source fact plus font family, but the private PyO3
and ordinary Qt mutation route deliberately do not expose either field: the current
renderer rejects authored font families outside its verified Telex resource and does
not yet shorten double-bond lanes from that ratio. Personal application preferences
and materializing standard values as explicit overrides on existing objects are also
separate contracts. This adds no public stub, CLI, wire, external ingress, or OASA
fallback; FQ-017 remains partial.
The developer-only `measure_cdml_manifest` Cargo example now makes a consented local
corpus measurement repeatable without creating a product ingress route. Its explicit,
untracked manifest names only operator-selected CDML or CD-SVG samples and an
operator-chosen read ceiling that bounds the measurement run. Its receipt retains
participant-chosen aliases, declared metadata, five shared XML-accounting dimensions,
format/stratum coverage, and stable non-content failures; it retains no paths, filenames,
document text, snippets, or hashes. A successful measurement means the raw CDML or
normalized CD-SVG payload also reached the current typed-document boundary. This is
one-time evidence, not a permanent corpus fixture, an admission default, or an external
Open capability. The project owner later delegated the operational policy choice and asked
for a long-lived adaptable boundary rather than a provisional command. The resulting
`ferrum-local-cdml-ingress-v1` profile is an engineering resource envelope, not a corpus
compatibility claim; its exact limits and change rule are recorded in
`reports/local_cdml_render_profile_v1.md`. CD-SVG and compression remain closed without their
own profiles. Ordinary Qt Open now calls that exact budgeted Rust file route through a
one-use session-and-observation receipt; same-tab replacement and recent-file routing remain
separate product decisions.
The same native tab now offers Set Atom Number with Ferrum... for exactly one selected
durable atom. It sends one typed positive number plus explicit show-number state to
Rust, publishes the returned authoritative projection, and leaves the session
unchanged when its dialog is cancelled. The action has no OASA mutation or fallback.
Clear Atom Number with Ferrum is a separate explicit action for one selected durable
atom with an authored number. A hidden number remains authored after Set, whereas Clear
removes the complete durable number/show-number pair in one revision-bound Rust
operation, republishes the projection, retains selection, and then disables the clear
action. It has no OASA mutation or fallback.
Delete Selected Atom with Ferrum is separately available for exactly one selected durable
atom. Rust validates the durable target and current revision, then atomically removes that
atom and its directly typed incident bonds in one revision-bound operation. The returned
projection clears selection and disables the action; Undo with Ferrum restores the prior
Rust-owned topology. A typed failure remains visible without an OASA or local-scene-edit
fallback. This does not adopt a bond-deletion bundle; same-tab/recent-file Open remains
unchanged.
Delete Selected Bond with Ferrum is a separate explicit action for exactly one selected
durable bond. Rust validates that target and current revision, removes exactly the selected
bond in one revision-bound operation while retaining both endpoint atoms, and publishes the
replacement projection with cleared selection and disabled action. Undo with Ferrum restores
the bond through Rust history. It has no shortcut, does not bundle atom deletion, and leaves
typed failures visible without OASA or local-scene fallback. Same-tab/recent-file Open remains
unchanged.
Edit Bond Properties with Ferrum is available for exactly one selected durable bond.
It reuses the frozen-projection BondDialog adapter to submit one revision-bound Rust
patch, publishes the returned projection, and retains the durable bond selection.
Only normal single, double, and triple bond semantics plus renderer-supported width and
centering combinations are exposed. Unrepresentable source facts fail visibly without
mutation, and cancellation is a no-op. This native path does not call or fall back to OASA.
The ordinary FQ-016 authority transfer uses `~/.ferrum/templates` as intentional Ferrum
application state, with no BKChem directory migration promise. One saved template remains a
complete bounded CDML document for inspection context, but eligibility requires exactly one
direct molecule; optional paper and standard records provide context and are not inserted.
Rust owns template admission, display-name extraction, finite atom-centroid derivation, fresh
durable identity allocation, internal-reference remapping, authored-scale translation from the
centroid to one requested scene anchor, complete-candidate validation, and atomic session history.
Qt owns secure catalog enumeration/confinement, visible refresh and malformed-neighbor reporting,
one placement intent, and tab/revision/digest delivery fences. Save As Template validates one
eligible Rust snapshot and uses the safe publisher within the configured catalog. This is a
dedicated template operation: clipboard Paste accepts a broader root grammar and applies a fixed
group displacement, so it is not the template insertion contract. Compact permanent evidence
covers eligibility, context separation, centroid placement, fresh identity/reference behavior,
atomicity, save/refresh/place behavior, cancellation, and stale containment. The runtime-only
PyO3 seam remains outside `.pyi`, CLI, serde, and wire contracts. FQ-016 remains
partial only for the documented template-capability disposition. Wheel rebuilds and UI
walkthroughs remain one-time evidence rather than XML-byte, pixel, timing,
catalog-count, or worker-wiring gates.
Permanent offline behavior tests cover route choice, selection state, lossless failure
containment, real native edit-history navigation, page transition, native publication,
lifecycle, accepted atom-number/show-number mutation and clear-pair removal, public-action
atom deletion with native Undo restoration, public-action bond deletion with retained endpoint
atoms and native Undo restoration, and the accepted/cancelled/lossless native-bond route.
Current-extension ordinary-window interactions are disposable implementation proof,
not byte, pixel, timing, network, or private-wiring gates. The default ordinary Open route
is Rust-owned. Its Properties dock reads only the installed Rust projection and reuses
existing native actions. The former compatibility dock/actions were retired; remaining
window/view capability decisions remain explicit FQ/M19 decisions.

#### 2026-08-15 M16 closure

M16 is done. The ordinary product has one Rust-native `MainWindow`; it owns the
supported bounded CDML and decoded CD-SVG document routes, native editing/history,
safe save/reopen, and native SVG/PDF/PNG publication. The retired OASA host and its
session, action, mode, worker, codec, and projection island are not a product route.

Unsupported local CDXML, CML, `.cdsvg`, `.svgz`, and compressed-CDML inputs receive
an actionable pre-read refusal. PubChem, compact-sugar interchange, descriptive
sugar/group/biomolecule catalogs, broad legacy templates, unported modes and repairs,
substructure search, oxidation, generated names, broad utilities, and unadopted
clipboard/window variants are intentional pre-production drops. `Recovery Export
CDML...` remains a current-document recovery copy, not a converter.

Permanent coverage remains compact offline semantic behavior, including one
representative CDXML refusal/nonmutation test; it does not preserve a host-topology
test. Installed launches, source and package scans, wheel/site checks, walkthroughs,
visual inspections, races, and scenario measurements are disposable implementation
evidence. M17/M18 subsequently completed the public protocol and CLI/Python
surface; M19, M20, and M22 remain open, and M21 is nonblocking. M16 completion
itself was not release-artifact proof.

### Milestone: M17 operation protocol and boundary freeze

- Depends on: M16.
- 2026-08-15 implementation checkpoint: the Rust-owned stateless JSON V1 now
  admits exactly `document.inspect`, `document.validate`, `document.rewrite`,
  and `document.render_artifact`; it has a generated checked-in schema, closed
  request/error envelopes, independent request transport admission before JSON
  parsing, established CDML admission, and derived checked base64 completion.
  The transport boundary derives from the established CDML source profile plus
  worst-case JSON escaping and a small V1 framing/request-ID allowance. It is
  an allocation-safety contract, not a timing, corpus, byte-identity, or pixel
  gate. The independent checkpoint rereview and fresh wheel/site evidence are
  accepted; M17 is complete.
- Deliverables: the versioned request/response protocol with a schema generated from
  Rust types; the freeze of the document channel, operation protocol, and
  `ChemEngine`'s tested semantics; and approval of the Python API and CLI contract
  surfaces that M18 implements. M17 does not itself expose a Python or CLI route.
- Exit criteria: schema generated and checked in; unknown protocol versions rejected.
  The `ChemEngine` freeze is an internal stability commitment so native and WASM stay
  aligned, not a third-party compatibility promise.
- Parallel-plan ready: no.

### Milestone: M18 Python module and CLI

- Depends on: M17.
- 2026-08-15 implementation checkpoint: `ferrum_chem` exposes only
  `execute_operation_v1(str) -> str`, `operation_protocol_schema_v1() -> str`,
  and categorized `OperationProtocolErrorV1`. Pre-envelope categories are
  `invalid_json`, `resource_limit`, and `execution_unavailable`; decodable
  domain/version refusals are JSON envelope data. `ferrum protocol schema` and
  `ferrum protocol run INPUT [--output OUTPUT]` preserve one JSON stream,
  explicit safe named publication, and the documented 0/1/2/3 exit meanings.
  M19 retired the provisional root command families; their underlying private and
  Ferrum-Qt-native seams are not new public CLI promises. No batch, network,
  session, Qt, adapter, path-bearing protocol payload, or render-observation
  operation is added. Compact Rust/Python semantic coverage is permanent;
  real CLI, wheel/schema-resource, generator, package build, and walkthrough
  checks are E2E or disposable evidence. The independent checkpoint rereview
  and fresh wheel/site evidence are accepted; M18 is complete.
- Historical pre-milestone proof: provisional direct CLI families demonstrated the
  Rust backend before the public contract freeze. M19 retired their root-command
  parsing and tests; private/native desktop seams remain owned by their existing
  contracts. The direct extension also supplies typed PyO3 DTOs for bounded native
  Qt slices. This history neither expands the frozen protocol nor establishes a
  third-party API. Native artifact publication continues to use its versioned local
  CDML ingress profile and complete-plan rule.
- Deliverables: bindings, generated `.pyi` stubs, and a CLI contract fixing
  subcommands, flags, exit codes, and stream behavior, derived from the Qt app's
  existing batch and export capabilities in the M1b matrix.
- Exit criteria: CLI round trips succeed under `tests/e2e/`.
- Parallel-plan ready: yes.

### Milestone: M19 integration closure

- Depends on: M18, M14, M15.
- 2026-08-15 implementation checkpoint: complete, pending independent closure review.
  The shipping CLI now contains only the frozen protocol family; the capability ledger
  indexes every supported row to its accepted semantic, E2E, or one-time validation
  lane, while refusals, drops, and future contracts remain decisions rather than
  untested parity obligations. The thread-affinity receipt records thread-confined
  sessions and GUI-thread serialized mutations with authenticated detached worker
  handoffs. It makes no timing or throughput claim.
- Deliverables: remaining placeholders removed; thread-affinity confirmed (RDKit
  built with `RDK_BUILD_THREADSAFE_SSS=ON`, sessions thread-confined, updates
  serialized per session, matching the desktop contract rather than introducing a
  second concurrency model).
- Entry criteria: the M1b capability matrix exists and every row is mapped.
- Exit criteria: **every capability in the M1b matrix classified as supported has its
  required behavior passing in the appropriate validation lane.** The artifact names
  currently listed in the matrix are evidence routes, not promises to retain a fragile
  test. Before M19 closes, each named pytest is checked against
  [PYTEST_STYLE.md](../PYTEST_STYLE.md); incidental wiring, exact-count, fixture-heavy, networked, or
  otherwise brittle cases are deleted or replaced by a semantic Rust test, fast
  offline pytest, or explicit E2E route. Rows classified known defect or unsupported
  path carry a recorded decision -- reproduced, fixed, or dropped -- rather than
  silently passing. Performance observations inform later deployment decisions when a
  user-visible regression is investigated; this plan sets no arbitrary timing threshold.
- Parallel-plan ready: yes -- capability verification parallelizes by matrix row.

Replacing "every capability the Qt app performed" with the matrix makes this exit
criterion finite and checkable. Earlier vertical slices already connected the desktop,
so this milestone verifies rather than performs first integration.

FQ-022's third-party-plugin path is a recorded pre-production drop. The former plugin
registrar and menu/action cascade were retired with the compatibility host; the ordinary
native product does not advertise an extension route. A future extension system begins as
a new designed capability with discovery, permissions, version negotiation, lifecycle,
and failure containment. No registrar inventory or placeholder wiring test is retained.

### Milestone: M20 packaging and platform matrix

- Depends on: M18, M4a (which already proved the mechanism on one platform). Closes
  after M19.
- 2026-08-15 source checkpoint: accepted for the one initial macOS arm64/CPython 3.12
  target. `tools/build_release_wheelhouse.py` builds the two selected first-party Python
  wheels, `ferrum-chem` and `ferrum-qt`, from explicit local Cargo, Qt build-backend, and
  Qt runtime wheelhouse inputs. The Rust `ferrum` command remains separately Cargo-installed.
  The real offline/scrubbed build, clean-install, installed-resource, and LGPL relink receipt
  is pending until those external target-matching inputs are provisioned. This is not a
  supported-product declaration or a commitment to other platforms.
- Deliverables: a target-specific two-wheel distribution route with bundled chemistry dylibs;
  one coherent clean-environment E2E observation; a named build/validation route; and LGPL v3
  relink verification for each admitted target.
- Exit criteria: for each admitted target, recorded local inputs build the selected two wheels,
  a scrubbed no-index install answers one installed chemistry request and loads Ferrum-Qt-owned
  resources, and the same chemistry behavior works after relinking. Runtime library discovery is
  solved at link time through `@loader_path`;
  `DYLD_LIBRARY_PATH` cannot fix it because macOS strips `DYLD_*`.
- Parallel-plan ready: yes -- one work package per platform.

### Milestone: M21 WASM proof

- Depends on: M4b, M17.
- Deliverables: a project-built MinimalLib WASM carrying the project's exports,
  validated against the frozen contract.
- Exit criteria: the same request set produces equivalent results through both
  implementations under the comparison rule for each output class. Build the same
  RDKit version on both targets where practical; otherwise compare discrete fields
  within one version and the documented semantic or geometric invariants across
  versions.
  Note that `straighten_depiction` exists in the WASM wrapper but not in
  `cffiwrapper.h`, so the native path uses the M11 port while the browser path uses
  the built-in. If satisfying the contract on WASM would require a browser-shaped
  concept in `ChemEngine`, record the divergence and leave the trait alone.
- Parallel-plan ready: no.

### Milestone: M22 establish as supported product

- Depends on: M19, M20.
- 2026-08-15 source checkpoint: accepted. `devel/make_release.py` now verifies the actual
  dual-license source-release contract rather than inventing a generic root license. The native
  wheel builder prepares its standard distribution-metadata notice bundle, and
  `tools/release_artifact_inventory.py` classifies final artifacts by semantic roles. The native
  InChI MIT notice is extracted from the leading license/attribution comment in the exact pinned
  `INCHI-1-SRC/INCHI_API/libinchi/src/inchi_dll.c` archive member. These source mechanisms do not
  yet establish a supported platform or published artifact.
- Deliverables: release-artifact proof that OASA, Tk, and Python RDKit are absent from
  production and release workflows; accurate migration documentation; the dual-license source
  archive; native and Qt notice/license roles; and the historical project marked superseded where
  appropriate. Production OASA declarations were removed at the 2026-08-15 host-retirement
  checkpoint, but that source fact is not release proof.
- Exit criteria: after M19 closes and M20 provides a real macOS arm64/CPython 3.12 receipt, a
  committed-release source archive, final wheels, source-archive Cargo CLI run, classified
  inventory, and human legal/release review establish the one supported boundary. The artifact
  contains no OASA or Tk runtime and no Python RDKit dependency, while retaining intentional
  native RDKit closure, notice, attribution, provenance, migration references, and CDML heritage
  acknowledgement. An About-box acknowledgement of BKChem lineage is correct and expected; an
  `oasa` import is not. M21 remains nonblocking.
- Parallel-plan ready: no.

The retired OASA host is not an exception to this global gate: M22 remains open until
the release artifact, install/migration documentation, and required release workflows
prove the declared supported boundary.

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

- **Per-patch gate:** the affected Rust and Python baselines pass, including
  `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo test`, and
  `pytest tests/` where those lanes are affected; the exclusion check passes.
  Durable logic receives the smallest stable semantic test that satisfies
  [PYTEST_STYLE.md](../PYTEST_STYLE.md). Rebuild receipts, public-window probes, and artifact
  inspections remain E2E or one-time evidence instead of becoming permanent tests.
- **Thin-workflow gate:** from M8a onward, the thin workflow E2E case passes. It runs
  on every milestone exit, not only in the milestone that introduced it.
- **Milestone-exit gate:** the touched capability's differential report shows no new
  divergence, run through `tests/e2e/`. This is deliberately not a per-patch gate --
  the oracle is a subprocess with its own RDKit and belongs in the slow lane.
- **Preservation gate:** every committed CDML corpus document round-trips intact on every
  parity run, against the coverage inventory. This is the one strict gate, because it
  is a structural, binary, checkable data-loss boundary rather than byte equivalence.
- **Independent review gate:** a `reviewer` agent audits M4a, M5, M10, M13, and M16
  without having implemented them.
- **Security-by-boundary gate:** each new external-input, FFI, parser, or package work package
  records its trust boundary, validation owner, resource limits, typed malformed-input failure,
  and adversarial tests before acceptance. CDML parsing continues to reject DTD input and permits
  no external entities, entity resolution, network access, recovery mode, or opt-in huge-tree mode.
  Those parser settings are not a substitute for measured byte, node, depth, attribute, and text
  budgets; the shared ingestion boundary records that gap until representative user documents
  justify a versioned deployment policy. M22 confirms the released product has retained these
  controls; it does not defer their first use.

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
| 2D coordinates | Within tolerance derived at M4c against the wheel's recorded RDKit build; geometric invariants across versions | Floating-point layout varies with RDKit version and build |
| Atom and bond fields | Every field Ferrum carries agrees with the oracle; dropped fields listed in the model spec | Discrete values, but Ferrum may carry fewer |
| Bond orders and aromatic flags | Exact | Discrete enumerations |
| InChI and InChIKey | Exact string; `InChI=1/` prefix asserted for non-standard output | Deterministic string output |
| Canonical SMILES and SMARTS | Exact within one recorded build; semantic round trip across versions | Canonical ranking can change between RDKit releases |
| Molblock and SDF | Semantic equivalence | Headers carry program and timestamp lines |
| CDML and SVG | Structural equivalence under one stated normalization: attribute ordering, namespace serialization, and insignificant whitespace normalized before comparison | Byte equality is not achievable through a tree parser |
| Text and glyph metrics | Current macOS arm64 QRawFont design-metric receipt: exact glyph IDs/origins; run values are ordinary `f64` observations, not a CI threshold | Closed Telex bytes and design units are stable, while Qt baseline metrics and future targets may differ |
| Render ops | Ordered typed DTO semantics with exact discrete facts and finite `f64` values carried by round-trip JSON; no extra rounding | Declarative data has a stated schema/provenance contract, not JSON-byte or renderer equivalence |
| Raster output | Requested dimensions and semantic render-plan structure; disposable local visual inspection may assess recognizability | Anti-aliasing differs across backends; M13 established no pixel or perceptual threshold |
| Straighten port | One-time current-target receipt for both `minimizeRotation` branches; M20 repeats it for each added release target | Floating-point trigonometry varies by target and build; no CI threshold is justified by one receipt |
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

M16 records a distribution for each scenario, including warmup policy, repetition
count, machine, corpus, and relevant background conditions. M19 derives each
regression threshold from that distribution rather than requiring a new run to beat
one baseline sample. Interactive scenarios are also compared with the measured frame
interval of the target display; the familiar roughly 16 ms interval is relevant only
when testing a 60 Hz display and is not a universal application requirement. Opening,
saving, and export are never judged against a frame budget. Whole-document CDML
exchange remains the working design while its measured committed-edit latency stays
appropriate for documents the project actually handles; larger documents or a
measured interaction regression trigger a protocol review rather than an invented
document-size cutoff.

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
properties, exact checks for closed render-operation fields, and structural or
chemical-semantic checks for CDML and Haworth output. A snapshot or golden is suitable
only when canonical spelling is itself part of a closed contract; writer bytes and
raster pixels are not general acceptance contracts. Most Ferrum-Chem verification
lands here. This lane is invisible to `pytest tests/` and is named as its own
per-patch gate.

**`pytest tests/` -- permanent, fast, offline.** Only checks passing the
`docs/PYTEST_STYLE.md` checklist: deterministic, inline inputs, `tmp_path` only,
small enough for the measured permanent-suite budget, no network, no subprocess.
No individual test receives an arbitrary duration limit. For this project that means CDML
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

The 2026-08-12 enforcement audit removed permanent pytest wrappers around the
coordinate, Molfile, SDF, and SMARTS measurement receipts; the reports remain durable
evidence and the repeatable measurement programs remain maintainer tools under
`devel/`. Four package-local native scripts named `e2e_native_*.py` remain placement
debt: preserve their installed-wheel behavior coverage, but move them with Git history
to `tests/e2e/e2e_*.py` before treating their location as conforming. Do not duplicate
them into the permanent pytest lane.

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

Ordinary `ferrum-qt` is Rust-native. The explicit OASA compatibility host and its
production dependency declarations were retired at the accepted 2026-08-15 checkpoint.
Historical OASA/BKChem material remains only as provenance or the separately isolated oracle
boundary. Unsupported historical workflows are refused or recorded as pre-production drops;
they do not activate a compatibility editor. No milestone is expected to leave the retained
application unrunnable.

Pre-release persistence uses `Ferrum` / `Ferrum-Qt` QSettings and
`~/.ferrum/templates`. Ferrum makes no BKChem preference or template migration or
compatibility promise. Historical lineage, provenance, and internal compatibility
identifiers remain separately documented where they retain a real consumer contract.

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
| Coordinates drift across RDKit versions | Medium: parity reports lose signal | An update lands without a cross-version receipt | WS-G | Build against the latest stable RDKit, record the exact artifact source, and compare measured invariants with the prior stable release |
| OASA code enters the production tree | Medium: provenance and licensing claims become false | A convenience import slips past review | WS-G | Per-capability exclusion checks, path-classified with a provenance allowlist |
| Imported Qt files fail the repository hygiene suite | Medium: M1 stalls on unrelated lint | `pytest tests/` has never run against the 505 imported files | WS-G | Run the suite during M1a and treat the output as M1a scope |

## Rollout and release checklist

- [ ] Native and future WASM artifacts record exact RDKit source provenance while
  routine development tracks the latest stable release and checks the prior stable
  release where a semantic comparison is useful.
- [x] Thin workflow passing continuously since M8a.
- [ ] Milestone differential reports committed.
- [ ] Every parity tolerance derived from a recorded measurement, not asserted.
- [ ] Accepted-difference list published, each entry naming its source-of-truth level.
- [x] Preservation coverage inventory complete; preservation gate green against it.
- [ ] M1b capability matrix fully closed at M19.
- [ ] Scenario observations recorded only where they will inform a real M19/M20
  deployment decision; no fixed timing target is a release gate.
- [ ] Release-artifact inventory confirms the declared OASA/Tk/Python-RDKit boundary.
- [ ] `docs/PROVENANCE.md` complete and accurate.
- [ ] Installation documentation covers the Rust toolchain, C++20 compiler, and RDKit
      build prerequisites.
- [x] `oasa` and Python `rdkit` removed from production manifests.
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
