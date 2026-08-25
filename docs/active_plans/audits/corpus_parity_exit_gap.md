# Corpus parity exit-gap analysis

Status update, 2026-08-11: all six tasks below are complete and M2 has exited. This
document remains the historical gap snapshot that defined the work; current evidence
and classifications are in
[corpus_molecule_parity.md](../reports/corpus_molecule_parity.md).
M8 has since retired the disposable reader described below. Every corpus path,
CDML loader, OASA command, and E2E runner named below is historical inline
semantic evidence only. None is a current runtime input, supported workflow, or
active proof obligation.

Read-only gap analysis for milestone M2 (core model) in
[../ferrum-plan-v3.md](../ferrum-plan-v3.md), lines 398-408. Every claim below carries a file and
line reference. No production file was changed to produce this report.

## Scope and method

Evidence sources:

- `packages/ferrum-rust/crates/core/src/lib.rs` (1632 lines), read for public API shape.
- [../decisions/ferrum_core_model.md](../decisions/ferrum_core_model.md), the accepted model
  specification.
- `tests/e2e/e2e_oracle_molecule_core.py` and `tests/e2e/oracle/e2e_oasa_molecule_core_child.py`,
  the M1d harness.
- The three corpus files under `tests/e2e/corpus/`, inventoried by
  [cdml_preservation_coverage.md](cdml_preservation_coverage.md).
- OASA's chemistry-import reader, `OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml.py` and
  `OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_writer.py` (gitignored reference material,
  cited by path rather than link).

## Deliverable status

| M2 deliverable | Status | Evidence |
| --- | --- | --- |
| Atoms | Implemented | `crates/core/src/lib.rs:431` struct, `:448` constructor |
| Bonds | Implemented | `crates/core/src/lib.rs:758` struct, `:772` constructor |
| Molecules | Implemented | `crates/core/src/lib.rs:1028`, `:1042`, `:1140` |
| Stable identifiers | Implemented | `crates/core/src/lib.rs:188`, `:222`, `:79` |
| Error types | Implemented | `crates/core/src/lib.rs:995`, 14 variants |
| Model specification | Implemented, one stale section | [../decisions/ferrum_core_model.md](../decisions/ferrum_core_model.md) lines 72-84 |
| `proptest` round-trip properties | Partial | `crates/core/src/lib.rs:1628-1630` |
| Read access to carried fields | Partial | accessors absent for six carried fields |
| Corpus molecule loading | Absent | no code path exists |
| Oracle field agreement | Partial | one narrow non-CDML capability |

### Partial deliverable one, proptest coverage

The `proptest!` block at `crates/core/src/lib.rs:1628` holds exactly two properties.
`nonidentical_idless_identity_is_reorder_independent` (`:1629`) is an identity-stability property
with no serialization. `endpoint_and_source_absence_round_trip` (`:1630`) is the only serde round
trip, and it asserts two facts: bond `source_id` stays absent and the start endpoint stays
non-atom. Both vary a single `i64` coordinate. No property varies charge, isotope, explicit
hydrogens, valence, multiplicity, free sites, bond order, bond style, or the aromatic flag. The
plan says "round-trip properties" plural; one narrow round trip over one scalar is thin evidence
for a model whose central claim is that optional fields preserve source absence.

### Partial deliverable two, unreadable carried fields

Several fields the specification lists as carried have no public read accessor, so nothing --
including a comparison harness -- can observe them through the public API.

| Carried field | Constructor parameter | Read accessor |
| --- | --- | --- |
| Atom position | `crates/core/src/lib.rs:451` | none |
| Atom isotope | `crates/core/src/lib.rs:453` | none |
| Atom valence | `crates/core/src/lib.rs:455` | none |
| Bond aromatic flag | `crates/core/src/lib.rs:779` | none |
| Molecule name | `crates/core/src/lib.rs:1042` | none |
| Group, text, query vertices | `crates/core/src/lib.rs:1042` | none |

Confirmed by searching the crate for `pub fn position`, `pub fn isotope`, `pub fn valence`,
`pub fn name`, `pub fn groups`, `pub fn texts`, and `pub fn queries`: zero matches. Present atom
accessors are `identity`, `source_id`, `element`, `formal_charge`, `explicit_hydrogens`,
`multiplicity`, `free_sites` (`:575-609`); molecule accessors are `identity`, `source_id`,
`atoms`, `bonds` (`:1120-1140`). These fields are reachable only through the internal Serde
representation, which [../decisions/ferrum_core_model.md](../decisions/ferrum_core_model.md)
lines 5-9 scopes to internal persistence and testing. M2 cannot demonstrate field agreement for a
field nothing can read.

## Historical corpus loading gap

At the time of this audit, no loading path existed. Nothing in the repository then turned a
corpus CDML file into a `ferrum-core` `Molecule`.

- Searching every `.rs` file under `packages/ferrum-rust/crates` for `cdml` returns only doc
  comments plus the document crate's opaque XML code. The single core-crate hit is a comment at
  `crates/core/src/lib.rs:57`.
- `crates/document/src/lib.rs:3-4` states the crate assigns no chemistry meaning, and
  `crates/document/Cargo.toml:14-16` declares only `thiserror` and `xot`. It does not depend on
  `ferrum-core`, so no molecule crosses that boundary.
- `crates/core/Cargo.toml:14-16` declares only `serde` and `thiserror`. The core crate has no XML
  dependency.
- The only executable that builds a `Molecule` is
  `crates/core/examples/oracle_molecule_core.rs:48-97`, fed by a JSON request written by hand at
  `tests/e2e/e2e_oracle_molecule_core.py:54-68`. It reads no file.

Four things must exist that do not exist today.

1. XML entry into the core model: an XML dependency in core, a separate loader depending on both
   `ferrum-document` and `ferrum-core`, or a harness example depending on `xot` plus `ferrum-core`.
2. Coordinate unit handling. The retired authored-document profile carried `cm` units and the
   retired legacy profile carried bare numbers.
   OASA converts with `cm_to_float_coord`
   (`OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml.py:62-68`, factor `72/2.54`).
3. Occurrence assignment for idless records. Every bond in
   `tests/e2e/corpus/legacy_groups_template.cdml:14-15` lacks an `id`, and
   `crates/core/src/lib.rs:481-485` rejects an idless record without a `legacy_occurrence`. The
   loader must assign it, exactly as
   [../decisions/ferrum_core_model.md](../decisions/ferrum_core_model.md) line 45 already assumes
   for a reader that has not been built.
4. Typed non-atom vertices. The retired authored-document profile contains a group, a
   molecule-local text, and a query, and all three bonds reference them.

## Sequencing question

M2 requires "every corpus molecule loads"
([../ferrum-plan-v3.md](../ferrum-plan-v3.md):404). Typed CDML record parsing is M8, which depends
on M7 and M2 ([../ferrum-plan-v3.md](../ferrum-plan-v3.md):508).

Verdict: (b), satisfiable by a minimal M2-scoped loader. This is not a genuine circular dependency.

The two artifacts differ, and the plan's own wording separates them. M8 delivers "typed payloads
for every class present in CDML today ... each with an unknown-attribute bag and an
unrecognized-child list; the typed-versus-opaque assignment table"
([../ferrum-plan-v3.md](../ferrum-plan-v3.md):509-512), exiting on a preservation round trip
(`:513-515`). M2 needs strictly less: a one-way projection of `molecule`, `atom`, `bond`, `point`,
`group`, `text`, and `query` into core records, with no attribute bag, no unrecognized-child list,
no serialization, and no round trip. A one-way projection is a proper subset of M8's contract, so
building it neither pre-empts nor duplicates M8; M8 subsumes and retires it.

Why not the alternatives:

- Option (a), genuine circularity, is wrong on the evidence. M8 requires round-trip preservation
  and the assignment table; M2 requires neither. Declaring circularity would force a plan edit to
  fix a conflict that does not exist.
- Option (c), reinterpreting the exit criterion against the current harness, is unacceptable. The
  harness builds three atoms and two bonds from a JSON literal
  (`tests/e2e/e2e_oracle_molecule_core.py:56-67`) and explicitly excludes "CDML parsing" and
  "coordinates, identifiers, isotope, valence, multiplicity, and free sites" (`:154-159`).
  Accepting it as proof that every corpus molecule loads would let M2 close having never read a
  corpus file, the exact failure mode the corpus exists to prevent. Under **use the scientific
  method**, an exit criterion weakened until existing evidence passes it stops being a test.

Recommended placement: build the loader as a harness-owned Rust example under
`packages/ferrum-rust/crates/core/examples/`, depending on `xot` as a dev-dependency, not as public
`ferrum-core` API. It stays explicitly disposable at M8, cannot be called by production code, and
adds no XML dependency to the shipped core crate. Under **long-term over short-term**, a permanent
parser in `ferrum-core` would create a second CDML reader that M8 must later delete.

### Scope note the loader must record

OASA's chemistry-import reader drops content Ferrum's model carries. In
`OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml_writer.py`, `_read_cdml_molecule_element` accepts
an atom only when its `name` is a periodic symbol or a known group abbreviation and returns `None`
for the whole molecule otherwise (lines 116-146), and it skips any bond whose endpoint is not in
the atom map (lines 177-180). On the retired authored-document profile, OASA yields one
atom and zero bonds while Ferrum represents one atom, three non-atom vertices, and three bonds.
That is a scope difference, not a divergence, and it belongs in the source-of-truth hierarchy
([../ferrum-plan-v3.md](../ferrum-plan-v3.md):153-182) at level 1 or 2, which outranks OASA at
level 5.

## Oracle coverage of carried fields

The current single capability is not sufficient evidence for "each field Ferrum carries agrees with
the oracle". Rust projects only element, formal charge, explicit hydrogens, and bond
start/end/order/type (`crates/core/examples/oracle_molecule_core.rs:98-148`); OASA mirrors that set
(`tests/e2e/oracle/e2e_oasa_molecule_core_child.py:37-57`).

Against the source-field mapping table in
[../decisions/ferrum_core_model.md](../decisions/ferrum_core_model.md) lines 72-84:

| Carried field | Oracle-verified today | Oracle can verify via CDML reader |
| --- | --- | --- |
| Atom element | YES | YES |
| Atom formal charge | YES | YES |
| Atom explicit hydrogens | YES | YES |
| Bond ordered start and end | YES | YES |
| Bond source type | YES | YES, parsed into type plus order |
| Bond order | YES | YES |
| Molecule id | NO | YES |
| Molecule name | NO | YES |
| Atom id | NO | YES |
| Atom name | NO | YES, carried as `symbol` |
| Atom point x, y, z | NO | YES, after the `cm` conversion |
| Atom isotope | NO | YES |
| Atom valence | NO | YES, OASA `valency` |
| Atom multiplicity | NO | YES |
| Atom free sites | NO | YES |
| Bond id | NO | YES |
| Bond style | NO | Partly; OASA keeps a `legacy_bond_type` property, no style enum |
| Bond aromatic flag | NO | NO; OASA carries no aromatic flag here |
| Group, text, query identity | NO | NO; OASA drops non-atom vertices |

Eleven carried fields are verifiable through `oasa.cdml.read_cdml` and are simply not compared yet
(`cdml_writer.py:98-136` and `:182-184`). Three (bond style, aromatic flag, non-atom vertices) have
no oracle counterpart and need a recorded classification in the model specification rather than a
comparison.

The specification's status text is stale: lines 11-12 say M1d must still produce corpus loading,
field comparison, and a divergence report. The harness landed and reports `"status": "match"`, but
the corpus loading and field comparison remain absent, and the plan status table still marks M2 not
started ([../ferrum-plan-v3.md](../ferrum-plan-v3.md):251) while the decision record calls it in
progress.

## Historical completed task sequence

| Step | Task | Outcome | Verification command | Mode |
| --- | --- | --- | --- | --- |
| 1 | Add read accessors | `position`, `isotope`, `valence` on `Atom`; `aromatic` on `Bond`; `name`, `groups`, `texts`, `queries` on `Molecule` | `cargo test --manifest-path packages/ferrum-rust/Cargo.toml --package ferrum-core` | Parallel |
| 2 | Extend proptest coverage | One serde round-trip property varying charge, isotope, explicit hydrogens, valence, multiplicity, free sites, order, style, aromatic | `cargo test --manifest-path packages/ferrum-rust/Cargo.toml --package ferrum-core` | Parallel with step 1 |
| 3 | Add OASA corpus child worker | `tests/e2e/oracle/e2e_oasa_corpus_molecule_child.py` projects each corpus file through `oasa.cdml.read_cdml` into the shared field set | Run the child with a corpus path on stdin, confirm one JSON object | Parallel with step 1 |
| 4 | Build the harness CDML loader | A `ferrum-core` example reading a corpus file with `xot`, assigning idless occurrences and converting `cm` coordinates | `cargo run --manifest-path packages/ferrum-rust/Cargo.toml --package ferrum-core --example <name>` per corpus file | Sequential, needs step 1 |
| 5 | Wire the corpus capability | `tests/e2e/e2e_oracle_corpus_molecule.py` runs both workers per file and writes a divergence report under `docs/active_plans/reports/` | `source source_me.sh && python3 tests/e2e/e2e_oracle_corpus_molecule.py` | Sequential, needs steps 3 and 4 |
| 6 | Record classifications and close | Specification gains divergence classifications, three unverifiable-field reasons, and a corrected status line; plan status flips | `source source_me.sh && pytest tests/test_markdown_links.py tests/test_ascii_compliance.py -q` | Sequential, needs step 5 |

Steps 1, 2, and 3 run concurrently with three owners. Steps 4, 5, and 6 are strictly ordered.

## Historical verification runs

| Command | Result |
| --- | --- |
| `cargo test --manifest-path packages/ferrum-rust/Cargo.toml --package ferrum-core` | 13 passed, 0 failed |
| `source source_me.sh && python3 tests/e2e/e2e_oracle_molecule_core.py` | exit 0, `"status": "match"`, OASA 26.2a1, RDKit 2026.03.4 |
| `source source_me.sh && pytest tests/test_markdown_links.py tests/test_ascii_compliance.py -q` | 366 passed before this document was added |

## Historical residual risks

- The harness loader is a second CDML reader from M2 through M8. Name its deletion in M8's entry
  criteria so it does not drift.
- The corpus holds four molecule-bearing atoms in total, so field agreement across it is weaker
  evidence than "every corpus molecule" implies.
  [cdml_preservation_coverage.md](cdml_preservation_coverage.md) governs corpus completeness for
  M10; M2 inherits whatever gaps remain.
- Bond style and the aromatic flag exit M2 unverified against any oracle, carried on the strength
  of the format specification alone.
- Unverified here: whether `oasa.cdml.read_cdml` parses each corpus file without error. The reader
  was read, not executed against the corpus. Step 3 proves that and may surface a blocker.
