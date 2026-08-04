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
  `docs/active_plans/decisions/m6_xml_storage_fidelity.md`.
- Added the M7 identity and ordering index, `IndexedDocument`, with its decision record,
  `docs/active_plans/decisions/m7_document_identity_ordering.md`. It derives direct-child
  `DocumentRecord` entries in exact source order, a document-wide `id_index` over every
  unqualified XML `id` including opaque content, root-relative element paths for
  diagnostics, and document-local provisional tokens that consume exactly once.

- Added the M8 typed-versus-opaque assignment table,
  `docs/active_plans/decisions/m8_typed_record_assignment.md`. It assigns every CDML
  element class to typed, opaque payload container, or opaque, names the typed fields
  per class, and fixes the unknown-attribute bag, unrecognized-child list, and
  additive-promotion rules.
- Added the M2 exit gap analysis, `docs/active_plans/audits/m2_exit_gap.md`, with a
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
  `docs/active_plans/audits/m2_exit_gap.md`.
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
