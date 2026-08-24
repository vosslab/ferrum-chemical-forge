# M4 atom oxidation corpus V1

## Result

The bounded evidence gate for `document.atom.oxidation.observe.v1` is complete.
This records one read-only HCNO V1 sub-slice; it does not complete M4 or claim
general oxidation-state chemistry. The governing scope is
[m4_atom_oxidation_v1.md](../decisions/m4_atom_oxidation_v1.md).

## Bounded profile

- Assess one selected durable atom in one selected durable direct-root molecule.
- Fence supplied CDML with its revision and lowercase SHA-256 digest.
- Return one signed `i16` oxidation number only under
  `formal-electron-assignment-hcno-v1`.
- Admit materialized H, C, N, and O with integral charge `-4..=4`, explicit H
  vertices, authored explicit-hydrogen facts of zero, non-aromatic single,
  double, or triple bonds, zero radicals, at most 256 atoms, 512 bonds, and
  64 components.
- Keep `accepted` as the only outcome with `oxidation_number`. An admitted root
  outside this profile completes as closed `unavailable`; invalid input,
  stale fences, invalid durable selections, provenance failures, and resource
  limits remain typed refusals with their established recovery.

The exact wire fields and categories are in
[FERRUM_API_CONTRACT.md](../../FERRUM_API_CONTRACT.md). The operation does not
change CDML, revision, history, selection, renderer state, or atom marks.

## Detached source provenance

The caller-provided revision and verified digest identify the source snapshot
and are retained in every accepted or unavailable receipt. The private helper
admits that CDML once into a request-local detached session, whose revision
begins at zero. These are distinct identities: an operation uses the detached
session for chemistry while it reports caller provenance, and it never compares
the caller revision to the temporary session revision. The generic PyO3
regression covers nonzero caller provenance plus malformed and mismatched digest
refusals so the rerun boundary cannot reintroduce that conflation.

## Executable corpus

[`document_atom_oxidation_corpus.rs`](../../../packages/ferrum-rust/crates/api/tests/document_atom_oxidation_corpus.rs)
is the canonical public Rust semantic corpus. It runs the bounded request
through the generic `execute_operation_v1` transport. Its categories are:

- accepted HCNO observations, including water oxygen at `-2`;
- structurally admitted but unsupported chemistry as one closed unavailable
  outcome;
- stale and invalid durable selection or document-fence typed refusals;
- resource-bound refusal and its stated recovery.

[`document_atom_oxidation_protocol.rs`](../../../packages/ferrum-rust/crates/api/tests/document_atom_oxidation_protocol.rs)
owns the named CLI standard-input, newline-terminated envelope, typed-refusal,
and recovery proof. The corpus is executable semantic evidence, not a second
chemistry oracle.

## Public Qt workflow

[`e2e_atom_oxidation_observation.py`](../../../tests/e2e/e2e_atom_oxidation_observation.py),
registered in [`run_all.sh`](../../../tests/e2e/run_all.sh), drives the staged
runtime through visible ordinary UI actions. It selects a structure, clicks a
real atom, invokes `Chemistry -> Atom Oxidation State...`, and accepts the
water oxygen `-2` result with its convention. It also confirms fluorine is
unavailable with recovery rather than an oxidation value.

The E2E exercises the compiled generic extension and the public modeless
dialog. It proves accepted and unavailable presentation and no-mutation
behavior without a test-only frontend. After one visible source edit, it proves
historical status while preserving the result details; it then proves rerun is
disabled on another tab, re-enabled only on the original selected source tab,
and completes a new observation. Finally, it closes the clean source through
the public tab control and observes dialog retirement. Qt typed-refusal
presentation is not a permanent timing or injection obligation; the
protocol/CLI lane proves typed refusal and recovery.

## Validation receipt

| Command | Result |
| --- | --- |
| `cargo test -p ferrum-api --test document_atom_oxidation_corpus` | Passed: 2 tests. |
| `cargo fmt --all --check` | Passed. |
| `PYTHONDONTWRITEBYTECODE=1 ./all_test.sh` | Passed: 7,492 hygiene checks, local CLI/Qt E2Es including atom oxidation, 292 Python binding tests, and 416 Qt tests with 1 skipped. |

## Limits and next work

This V1 deliberately excludes elements outside HCNO, implicit or aggregate
hydrogens, aromatic, radical, delocalized, coordination, and unsupported-bond
chemistry. Broader coverage requires a separately approved V2 contract and
corpus.

M4 remains incomplete. Select the next chemistry-operation catalog contract
separately; this HCNO V1 sub-slice neither expands chemistry support nor claims
broader OASA/BKChem parity.
