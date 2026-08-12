# Corpus molecule parity report

## Verdict

M2 is complete. Every molecule in the three committed CDML corpus files loads into
the validated `ferrum-core` model. The separate-process comparison records 96 exact
source-fact agreements, 29 classified differences, and zero unexpected differences.
One mutation changes an atom element; another removes a non-atom vertex. Each exits 1
with exactly one unexpected difference, proving both comparison routes detect drift.

The machine-readable evidence is
[`corpus_molecule_parity.json`](corpus_molecule_parity.json). It contains both
normalized outputs, the oracle and Ferrum version facts, direct source facts, every
exact field path, and every classification.

## Self-contained boundary

The authoritative Rust reader, comparison runner, corpus, and report are Ferrum files. They
do not read, import, package, or locate `OTHER_REPOS/`. The comparison runner invokes an
isolated environment at `tests/e2e/oracle/.venv/`, which is ignored and recreated from
the pinned
[`pip_requirements.txt`](../../../tests/e2e/oracle/pip_requirements.txt).

For this completion run, OASA 26.8 was copied into that isolated environment from the
read-only reference checkout and RDKit 2026.03.5 came from the repository Python 3.12
environment. The installed copy no longer imports from the checkout. A fresh machine
can install the same pinned published packages; deleting the checkout does not change
Ferrum source, build, package, runtime, or permanent test inputs.

## Exact agreements

All source facts that both models represent agree exactly:

- Molecule order, ID, and source-name presence.
- Atom order, ID, element spelling, authored optional scalars, and coordinates.
- `cm` coordinates converted by the historical CDML factor `72/2.54`; bare coordinates
  remain PostScript points; omitted z becomes zero under the format contract.
- Atom-only bond ID, ordered endpoints, exact source-type token, and every bond order
  except the separately classified legacy correction.
- Corpus molecule and atom counts.
- Group, molecule-local text, and query identities plus the ordered endpoints, type,
  and identity of every bond involving them. These fields are read independently from
  CDML with the Python standard library because the chemistry-only oracle drops them.

## Classified differences

| Class | Count | Authority and treatment |
| --- | ---: | --- |
| Intended source presence | 20 | CDML omits formal charge, explicit hydrogen count, valence, multiplicity, or free sites. Ferrum retains `None`; the chemistry reader computes element defaults. The source document outranks the computed default. |
| Intended core scope | 1 | Ferrum retains three source-verified bonds involving group, molecule-local text, and query vertices. The chemistry reader drops all three because it has no corresponding graph classes. |
| Oracle-unrepresented non-atom vertex | 3 | Direct CDML facts verify the group, text, and query identities required by their bond references. The chemistry reader has no comparable records. |
| Unverifiable bond fields | 4 | Ferrum carries a typed style and aromatic source-presence value for two atom-only bonds. The oracle projection has no independent counterpart. |
| Intended format correction | 1 | In CDML 0.8, single-letter `d` means normal double. The current historical reader reports order 1; Ferrum follows the versioned format contract and reports order 2. |

The exact source token remains `d` even when its versioned meaning is corrected. This
keeps compatibility data and interpreted semantics separate.

## Authoritative reader boundary

M8 retired the dev-only `ferrum-core` loader. The comparison example now consumes the
production `ferrum-document` typed overlay and its validated core projection. The
reader-inventory test permits only the document crate to recognize the CDML namespace
in Rust production code. The E2E runner separately reads a deliberately small set of
direct source facts with `xml.etree.ElementTree`; it is comparison evidence, not a
runtime parser or a document model.

## Reproduction

Create the ignored oracle environment once, then run the comparison:

```bash
source source_me.sh && python3 -m venv tests/e2e/oracle/.venv
tests/e2e/oracle/.venv/bin/python -m pip install \
  -r tests/e2e/oracle/pip_requirements.txt
source source_me.sh && python3 tests/e2e/e2e_oracle_corpus_molecule.py
```

Accepted result:

```text
status: match-with-classified-differences
unexpected_difference_count: 0
```

These diagnostics must each exit 1 with one unexpected difference:

```bash
source source_me.sh && python3 tests/e2e/e2e_oracle_corpus_molecule.py \
  --mutate-ferrum --report /tmp/ferrum-corpus-atom-mutation.json
source source_me.sh && python3 tests/e2e/e2e_oracle_corpus_molecule.py \
  --mutate-ferrum-non-atom \
  --report /tmp/ferrum-corpus-non-atom-mutation.json
```
