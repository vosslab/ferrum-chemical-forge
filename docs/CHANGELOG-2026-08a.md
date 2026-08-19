## 2026-08-11

### Additions and New Features

- Narrowed the native RDKit policy to the implemented GraphMol kekulization ABI. It
  retains only pinned RDKit, configure-time Catch2, Better Enums required for a
  generated GraphMol header, and header-only Boost. InChI, CoordGen, and MAEParser are
  explicitly disabled and no longer downloaded, materialized, or passed to CMake. The
  fresh source E2E passed with the GraphMol-only profile, a Ferrum-owned sealed stage,
  and the exact five-library closure recorded in its receipt.
- Removed the test-only native build-marker export from Ferrum-Chem before release.
  The ABI-2 surface now contains only its version, chemistry operation, and owned
  response lifecycle. The relink E2E proves replacement through the stable library
  role and package-relative copy, distinct `Release` and `RelWithDebInfo` adapter
  hashes, closure validation, fresh ABI loads, and safe Rust kekulization before and
  after replacement.
- Made the public Ferrum-Chem header the sole numeric authority for bond wire values
  as well as the ABI constants. Rust derives its private wire constants from that
  header, preventing independently maintained C++ and Rust bond encodings.
- Documented the native kekulization boundary: a safe Rust `ChemEngine`,
  Ferrum-owned graph records, the version-2 C ABI, explicit operation defaults, sealed
  native inputs, and semantic relink validation. The durable decision and status
  documents distinguish this narrow completed operation from the later
  coordinate-tolerance milestone.
- Completed the narrow native kekulization milestone on macOS arm64. The source E2E
  installed ABI 2 with its exact five-library closure, then verified alternating
  benzene bond orders, topology, and optional atom facts in fresh Rust processes
  before and after replacing `libferrum_chem.dylib` from a sealed manifest-v2 input
  tree. This does not claim Qt adoption, CDML integration, coordinate parity, broader
  RDKit APIs, cross-platform support, or a release.
- Recorded the one-time isolated RDKit orientation measurement on an asymmetric
  molecule. Explicit `canonOrient=false` and `canonOrient=true` diverge; future
  Ferrum layout will select `true` explicitly. The measurement establishes neither
  a Ferrum layout implementation nor the M4c coordinate-tolerance/parity gate. Its
  accepted receipt is
  [`rdkit_layout_orientation.json`](active_plans/reports/rdkit_layout_orientation.json),
  outside pytest's collection tree.

- Declared CMake and upstream LLVM in the root `Brewfile` so `brew bundle`
  installs the native Ferrum-Chem/RDKit build system and FOSS compiler frontend
  instead of relying on ambient CMake or Apple Clang.
- Declared unpinned Maturin in the root development requirements because the
  native-wheel builder invokes its command directly; compatibility is established
  by the build gate and recorded tool version rather than an installation pin.
- Selected current Boost 1.91.0 headers for the reproducible native profile. Ferrum
  neither builds nor bundles compiled Boost libraries; the source digest prevents
  ambient dependency drift without freezing the developer's installed tools.
- Completed the macOS arm64 native-wheel packaging and LGPL relink proof. The
  hash-verified source build installs a minimal wheel in a scrubbed environment,
  then loads a replacement `libferrum_chem.dylib`; the closure is exactly
  `libferrum_chem.dylib` plus `libRDKitRDGeneral.1.dylib`. The durable status report
  is `docs/active_plans/reports/native_wheel_packaging.md`.
- Added JSON-only native-wheel evidence recording source hashes, CMake/LLVM/Rustup
  provenance, the Apple SDK platform boundary, actual unpinned Maturin version, wheel
  digest, closure, and replacement probe. Successful tests retain no wheel or dylib
  in the repository.
- Added the self-contained `ferrum` Rust CLI with typed `cdml inspect` JSON output,
  structural `cdml rewrite`, explicit stdin/stdout selection, stable exit behavior,
  and command-level integration tests. The executable has no Python or `OTHER_REPOS`
  dependency.
- Added verified source-install and CLI usage guides for the Rust backend. Usage-facing
  commands invoke `ferrum`; no Python module-launch route is documented or shipped.
- Added M3's private-`petgraph` analysis view for components, connectivity, bridges,
  articulation points, matching, shortest paths, all-pairs distances, diameter, and
  cycle rank, plus a Ferrum-owned deterministic shortest fundamental-cycle basis.
- Added the self-contained M2 corpus comparison runner, pinned isolated-oracle
  requirements, machine-readable evidence, and classified parity report. All three
  corpus documents load with zero unexpected source-fact differences.
- Added M8's single-tree typed CDML overlay, covering every assigned class with named
  lexical fields, unknown-attribute bags, ordered opaque children, and non-demoting
  cardinality diagnostics. Molecule records project into validated `ferrum-core` data.
- Renamed current decisions, audits, reports, and the corpus projection example by
  durable capability so filenames no longer carry temporary milestone numbers.

### Behavior or Interface Changes

- Removed Python RDKit from the root development environment. The Rust backend uses
  the pinned C++ source build, while Python RDKit remains isolated with the
  historical oracle under `tests/e2e/oracle/pip_requirements.txt`.
- Made Ferrum the sole owner of the packaged Mach-O closure: Maturin packages without
  repairing native libraries, then Ferrum extracts the wheel and enforces the exact
  extension dependency, loader path, library identities, and bundled closure.
- Renamed the retained Qt Python namespace to `ferrum_qt`, updated package discovery,
  imports, dynamic registrar paths, resource lookup, application display strings, and
  the `ferrum-qt` console entry point. Removed the Python module-launch route;
  user-facing execution is the installed console command.
- Kept QSettings names, `~/.bkchem/templates`, clipboard ownership value, provisional
  session tokens, and the smoke-receipt schema as explicit compatibility identifiers.
  Ferrum branding changes product-facing names without orphaning existing preferences,
  templates, document contracts, or integration protocols.

### Fixes and Maintenance

- Sealed reusable native RDKit inputs to the complete canonical Ferrum policy,
  including target closure and all CMake/forbidden-fragment choices, rather than
  accepting a matching profile name. The versioned manifest now fingerprints
  deterministic, fail-closed full trees for the installed RDKit and pinned Boost
  headers, so an adapter rebuild rejects transitive header drift, unsupported file
  types, and escaping or dangling library aliases.
- Corrected the native RDKit header-root contract to use the installed
  `rdkit-install/include/rdkit` directory. The builder, immutable input manifest,
  fixture checks, and CMake adapter now agree on the directory that contains
  `GraphMol/` and `RDGeneral/`, preventing a completed private RDKit build from
  failing during adapter configuration.

- Made the public Ferrum-Chem header the single adapter ABI-version authority. The
  C++ implementation uses that macro, the native-wheel builder derives and passes it,
  and the Rust extension includes a generated constant after build-time validation.
- Split Mach-O discovery, rewriting, and exact closure validation into a focused
  native-wheel module. The long E2E now streams compiler diagnostics while retaining
  one strict, additive-compatible JSON artifact record on standard output.
- Hardened native build provenance by removing every inherited `CMAKE_*` variable and
  supplying required compiler, SDK, and source facts explicitly. Retained evidence
  replaces discarded output-root paths with `${OUTPUT_ROOT}`, rejects ambiguous or
  non-JSON values, and records no transient wheel or source-archive path.
- Aligned the native builder and direct E2E with repository execution policy by
  invoking both through Python 3.12 without executable-script shebangs. Kept the
  package initializer declarative and exercised the private native submodule directly.
- Replaced permissive ZIP extraction with component-bounded regular-file extraction,
  stripped privileged archive mode bits, and rejected duplicate TAR targets before
  extraction. A custom redirect handler validates every hop as credential-free HTTPS
  before the next request, followed by digest verification of the completed download.
- Corrected the native CMake provenance parser so CMake punctuation that normalizes
  to the filesystem root is not treated as a host dependency. All concrete absolute
  paths remain subject to the existing fail-closed allowlist.
- Constrained native CMake and Maturin/Cargo program lookup to the declared CMake,
  LLVM, Rustup, and macOS system-tool directories. The provenance audit now checks
  configured CMake values and selected programs rather than harmless candidate paths
  in CMake's exploratory search log; undeclared selected paths still fail closed.

- Deferred command-line file loading until the primary Qt event loop begins and made
  the existing controlled lifecycle callback retire an active modal before queuing
  shutdown. Smoke success now also requires every requested launch file to finish
  opening, so a callback failure cannot publish a false success receipt. Warnings
  remain unchanged in ordinary application use.
- Documented the contributor-preview Ferrum-Qt source install and its installed
  `ferrum-qt` launch command, while keeping the temporary OASA dependency and pending
  self-contained wheel explicit. Corrected stale pre-rename provenance and plan paths.
- Marked both superseded Ferrum plans as historical-only drafts that must not be used
  for current paths, commands, or status; v3 remains the active authority.
- Exempted the three current implementation-plan documents from the 1,000-line
  source-file gate through the manager-approved exact-path override list. Source
  code and non-plan documentation remain subject to the existing limit.
- Removed generated RDKit, Boost, ICU, compression, and Ferrum-Chem dylibs from source
  and package directories. Both generation targets are now ignored; native closures
  belong in build output and inside the produced wheel, not in Git.
- Replaced the first raw edge-order cycle tree after parity exposed chemically
  unhelpful long cycles. The general stable-BFS scoring policy selects shorter bases
  without recognizing molecule names or special topologies.
- Corrected the M2 loader's bond design: CDML 0.8 `s` and `d` are normal single and
  double bonds, while current `s`, `q`, `l`, and `r` retain distinct depiction meaning.
  The exact source token remains carried separately from its versioned interpretation.
- Corrected the document identity index so fragment bond and vertex `id` references do
  not collide with the declarations they name; opaque `id` values still reserve names.
- Made bare pytest select the documented root `tests/` lane and added recursion
  boundaries for `OTHER_REPOS` and ignored `output*/` build trees; the nested
  `tests/conftest.py` cannot control top-level siblings.
- Represented `external-data`, `display-form`, and `user-data` as typed opaque
  containers. Their identity and document position are now typed facts while their
  attributes and descendants remain uninterpreted preservation payload.
- Strengthened corpus parity by deriving non-atom vertices and bonds independently
  from CDML through `defusedxml`. Removed broad harness exception swallowing, renamed
  the isolated manifest to `pip_requirements.txt`, and added separate atom and non-atom
  mutation probes.
- Removed projection errors made impossible by the document identity index and added
  direct behavior coverage for missing geometry, malformed scalars, unresolved
  endpoints, and core-model rejection.
- Completed M1b with installed-command evidence: the offscreen `ferrum-qt` process
  opens the authored CDML fixture through the existing Qt/OASA-backed native CDML
  route, writes the controlled receipt, exits without a traceback, and leaves
  worker-routed non-CDML imports as a later replacement risk. This proves the
  rename/start/open path, not Rust-backend adoption or worker-format completion.
- Completed M1e with a positive Ferrum production selector, an empty active-capability
  policy, and seeded OASA/Tk rejection proofs. `OTHER_REPOS` is categorically outside
  this production scan.

### Developer Tests and Notes

- The historical M4a native-wheel E2E rebuilt the hash-verified profile after the
  download, extraction, package-init, and ABI hardening. A clean isolated process
  loaded ABI 1 before relinking, and a fresh process loaded ABI 1 after relinking;
  its retained wheel digest is recorded in the packaging report. The later ABI-2
  semantic E2E is the final M4b evidence.
- The Rust workspace passes formatting, checking, Clippy with warnings denied, and
  all-target tests on `aarch64-apple-darwin`: 3 API unit tests, 4 CLI integration
  tests, 25 core tests, and 21 document tests. A disposable source install produced
  `ferrum 26.8.0` and inspected a corpus document without Python or `OTHER_REPOS`.
- The renamed Ferrum-Qt package reports 918 passed and 1 skipped tests. A disposable
  no-dependency install produced the `ferrum-qt` console command and reported
  `Ferrum-Qt 26.08`; no `python -m ferrum_qt` route is shipped.
- An earlier pre-M1e root-hygiene run reported 3,066 passing tests when the two known
  migration gates were selected separately. The undeclared-import gate still reports
  the 64 migration-only OASA imports, and the line-limit gate now reports only the 11
  pre-existing oversized code/test files after plan-document exemption.
- The M1b focused lifecycle and CLI suite reports 21 passing tests; its independent
  review was accepted. M1e's focused import-exclusion suite reports 4 passing tests,
  and root hygiene reports 3,070 passing tests when the two known migration gates are
  selected separately.
- The native-wheel audit confirms that no `.dylib`, `.so`, or `.dll` is tracked; its
  completion evidence is the macOS arm64 packaging and relinking proof recorded above.
- Twenty-five `ferrum-core` tests pass. Ten fixed topology fixtures match every
  reference discrete graph result except one documented 5/6-to-5/5 bridged cycle-basis
  improvement; exact Ferrum cycle and matching outputs repeat across 100 calls each.
- The corpus comparison records 96 exact agreements, 29 classified differences, and
  zero unexpected differences. Its atom and non-atom mutations each exit 1 with one
  unexpected difference.
- M2, M3, and M8 are complete. M8 deleted the disposable core reader, removed `xot`
  from `ferrum-core`, and left `ferrum-document` as the sole production CDML
  recognition authority.

## 2026-08-03

### Additions and New Features

- Added the seven-crate Ferrum-Chem Rust workspace and accepted its scoped
  `packages/ferrum-rust/target/` ignore.
- Added complete offline canonical AGPL v3 and LGPL v3 texts and a provenance record.
- Added a project README and populated production and development dependency manifests.
- Added the bounded M1d preservation inventory at
  `docs/active_plans/audits/cdml_preservation_coverage.md` and three CDML fixtures:
  `authored_document_forms.cdml`, `legacy_groups_template.cdml`, and
  `opaque_namespace_preservation.cdml`.
- Added the bounded M1b capability matrix at
  `docs/active_plans/audits/ferrum_qt_capability_matrix.md` with 25 stable
  capability rows.
- Added opaque CDML storage with `xot` 0.31.2 and its decision record,
  `docs/active_plans/decisions/xml_storage_fidelity.md`.
- Added the M7 identity and ordering index, `IndexedDocument`, with its decision record,
  `docs/active_plans/decisions/document_identity_ordering.md`. It derives direct-child
  `DocumentRecord` entries in exact source order, a document-wide `id_index` over every
  unqualified XML `id` including opaque content, root-relative element paths for
  diagnostics, and document-local provisional tokens that consume exactly once.

- Added the M8 typed-versus-opaque assignment table,
  `docs/active_plans/decisions/typed_record_assignment.md`. It assigns every CDML
  element class to typed, opaque payload container, or opaque, names the typed fields
  per class, and fixes the unknown-attribute bag, unrecognized-child list, and
  additive-promotion rules.
- Added the M2 exit gap analysis, `docs/active_plans/audits/corpus_parity_exit_gap.md`, with a
  per-deliverable status table, the four missing loader capabilities, an oracle
  coverage table over the carried fields, and six remaining atomic tasks.

### Behavior or Interface Changes

- Identified the retained PySide6 frontend as Ferrum-Qt in package metadata while
  retaining its `bkchem_qt` Python namespace until M1b.
- Set Ferrum-Qt metadata to AGPL-3.0-only, corrected its GitHub project URLs, and
  exposed the temporary `bkchem_qt.cli:main` entry target as `ferrum-qt`.
- Split production and development dependencies: production states the direct
  `shiboken6` policy; development carries `lxml` and oracle-only chemistry tools.
- Recorded all seven current export codecs (`.mol`, `.sdf`, `.smi`, `.cdml`,
  `.cdxml`, `.cdsvg`, `.inchi`), persistent atom/bond edits, atom numbering, and
  seven durable atom-mark types in the M1b capability ledger.

### Fixes and Maintenance

- Replaced empty repository README, license, dependency, changelog, and provenance
  scaffolding with the current M1 record.
- Corrected the M8 deliverable text in `docs/active_plans/ferrum-plan-v3.md`. It named
  bracket and vector graphic as object classes; CDML defines neither element, so the
  deliverable now names the six vector-graphic shapes (`rect`, `square`, `oval`,
  `circle`, `polygon`, `polyline`) and states that bracket artwork persists as
  direct-root `<polyline>` records while `<bracket>` and `<vector>` stay preserve-only
  opaque.
- Corrected the M2 status in the plan's milestone table from `not started` to
  `in progress`, and added M2 and M8 evidence rows to the current state summary.
- Corrected the stale scope-and-status paragraph in
  `docs/active_plans/decisions/ferrum_core_model.md`. It still attributed corpus
  loading, pinned OASA field comparison, and a divergence report to M1d; the M1d
  oracle harness has landed and reports `"status": "match"`, and the remaining work
  is now scheduled as six atomic M2 steps tracked in
  `docs/active_plans/audits/corpus_parity_exit_gap.md`.
- Fixed the pyflakes and shebang/executable-bit hygiene failures on
  `tests/e2e/e2e_oracle_molecule_core.py` and
  `tests/e2e/oracle/e2e_oasa_molecule_core_child.py`: removed the unused `sys`
  import from the parent harness and set the executable bit on both files, since
  each carries a shebang and an `if __name__ == '__main__':` guard and is invoked
  directly. Both files existed untracked for a while; the repo hygiene suite only
  discovers git-tracked files, so these pre-existing defects surfaced only once
  the files were staged for M1d.

### Removals and Deprecations

- Recorded M22 as the removal gate for migration-only OASA and its Python RDKit
  dependency from the production environment.

### Decisions and Failures

- Retained Ferrum-Qt as the existing frontend and established Ferrum-Chem as the new
  Rust backend; RDKit remains the chemistry authority behind a project-owned adapter.
- The licensing and provenance records describe implementation intent and are not
  legal advice.
- M1a removed the broken `bkchem_data` symlink through an escalated, staged `git rm`;
  package-owned resources now resolve. Independent review accepted M1a after this
  correction.
- Independent content re-review accepted the M1d inventory and compact corpus package.
  M1d remains in progress pending its separate-process oracle harness and divergence
  report.
- Independent content re-review accepted the M1b matrix. Its source measurement is
  445 `oasa` tokens in 18 production files, including 64 direct imports in 16 files;
  it found zero direct Tk/Tkinter imports and 29 historical Tk/Tcl text hits.
- Persistence identifiers remain explicitly unresolved, with retaining existing values
  recommended for M1b. PubChem has no assigned owner, and third-party plugins remain
  an unsupported path.
- M1b remains in progress pending the namespace `git mv`, identity and migration
  decisions, and application start/open gates.
- Independent review accepted M6's structural storage boundary. The one-time
  three-corpus probe preserves expanded element and attribute namespaces, values,
  child order, mixed text and tails, comments, and processing instructions.
- `xot` rejects DTD input and the M6 entry point has no external-entity or network
  resolver. Raw source-slice fallback is not adopted because the current corpus
  retains structural meaning.
- M7 indexes identity without assigning meaning. An `id` inside opaque content only
  reserves a collision name; it gains no typed-record or reference semantics, and
  `idref`, endpoint-like attributes, and text are never resolved or rewritten. Typed
  records, reference validation, and durable ID allocation wait for M8 and later.
- M7 provisional tokens are unforgeable outside the crate: each carries a private,
  process-local document-instance component plus a document-local sequence, so a token
  issued by one document is rejected by another even when the sequences match. The
  instance component is deterministic within a process and has no persisted meaning.
- The M7 index accepts both the canonical CDML namespace and legacy no-namespace CDML,
  and rejects blank and duplicate persistent IDs while reporting both structural
  locations, including a root-versus-descendant collision.
- Cleanup during M7 close-out removed the `include_current_id` parameter from
  `index_element`; every call site passed `true`, so the branch was dead weight that
  implied an unindexed-element mode the design does not have.

- M8 keys a typed class by parent context plus expanded name rather than by local name.
  The reference attribute registry `CDML_CORE_ATTRIBUTE_NAMES` keys by local name only,
  which collapses `arrow@length` (a `standard` default) with `arrow@idref` (a reaction
  role) and merges the `standard`, `molecule`, and `fragment` senses of `<bond>` into
  one entry. Copying that collapse would have given several classes attributes they
  cannot legally carry.
- CDML has no `<bracket>` record. `CDML_CORE_ELEMENT_NAMES` lists 40 names and contains
  neither `bracket` nor `vector`; the bracket tool issues four `new_polyline` calls and
  creates no bracket object; and the backend contract commits a rectangular bracket as
  two direct top-level polyline records with no wrapper semantics. Bracket artwork is
  therefore assigned to the `polyline` row, and the plan's "vector graphic" class maps
  to the six shape elements.
- M2 needs only a one-way CDML-to-core projection, so its exit criterion and M8 are not
  circular. M8 delivers round-trip preservation plus the assignment table; M2 requires
  neither, and M8 subsumes and retires the harness loader. Reinterpreting the exit
  criterion against the existing JSON-fed harness was rejected: it would let M2 close
  having never read a corpus file.
- The M2 corpus loader is authorized as a disposable harness example at
  `packages/ferrum-rust/crates/core/examples/m2_corpus_cdml_loader.rs`, owned by the
  harness, depending on `ferrum-core` plus `xot` as a dev-dependency only. It parses
  CDML itself from a corpus file path; no Python may interpret CDML for the Ferrum side.
- Loading is total rather than best-effort: inside a `<molecule>` subtree an unhandled
  element or attribute is an error unless it appears in the deferred set, which is
  exactly the Dropped and Computed rows of the core model specification. A best-effort
  loader could pass by ignoring what it does not understand, the exact failure the
  corpus exists to catch.
- A Python-side projection was rejected. Beyond not proving that Ferrum loads the
  corpus, a projector written by consulting OASA's own reader would silently agree with
  the oracle and make the comparison partly self-confirming. It also could not avoid the
  Rust work, since idless occurrence assignment keys off `ferrum-core`'s internal
  canonical fingerprint, and it would make the two-readers problem permanent in the
  language M22 exists to remove.
- The removal control is mechanical rather than prose: `tests/test_cdml_reader_inventory.py`
  asserts the set of Rust files containing the CDML namespace URI equals a hard-coded
  allowlist, so a third reader fails the suite immediately. M8's entry criteria now carry
  the loader's scheduled deletion.
- Verified rather than assumed: library source cannot use a dev-dependency
  (`error[E0433]`), `cargo test` compiles examples so the loader cannot rot, a dependent
  crate cannot name an example, and `xot` is already in the workspace lockfile via
  `ferrum-document`, so the dev-dependency adds zero new third-party code.
- M2 remains open on three measured gaps: no code path turns a corpus CDML file into a
  `ferrum-core` `Molecule`, six carried fields (atom position, isotope, valence, bond
  aromatic flag, molecule name, and non-atom vertices) have no public read accessor,
  and the oracle compares 6 of the 19 carried fields.

### Developer Tests and Notes

- The initial root suite reported 2,967 passed tests and 200 M1a-scoped failures
  caused by empty README and manifest scaffolding. After the accepted metadata,
  README, and license fixes, the root suite reported 3,167 passed tests; the final
  M1a root suite reported 3,186 passed tests.
- The Rust workspace completed `cargo fmt --check`, `cargo build`,
  `cargo clippy -- -D warnings`, and `cargo test`.
- The M1d package passed ASCII, XML parsing, ftext, query, reference, and namespace
  checks; `git diff --check` also passed.
- M6 passed its exact gates: parse, serialize, reparse, and structural comparison of
  all three corpus fixtures; document-crate formatting, build, Clippy, and tests;
  and independent review. Serialization normalizes the XML declaration, top-level
  newline placement, CDATA/entity spelling, prefixes, attribute order, quote style,
  and original whitespace spelling rather than promising lexical preservation.
- Gotcha: the repo hygiene tests discover git-tracked files only, so a newly created
  untracked document is not checked at all. The count stayed at 366 passed after two
  new documents were added and rose to 370 only once they were staged. Stage new files
  with `git add` before treating a green
  `pytest tests/test_markdown_links.py tests/test_ascii_compliance.py` run as evidence
  that they pass; otherwise the verification is silently vacuous.
- M7 passed its exit gates on the Rust workspace: `cargo fmt --check`,
  `cargo clippy --all-targets -- -D warnings`, and `cargo test`, with eight
  `ferrum-document` tests green. Two of those tests carry the milestone exit criteria
  directly: one proves direct source order and identity paths survive a round trip,
  and one proves a reference-looking value inside an opaque node is reserved but left
  byte-for-byte alone.
