# M4 atom oxidation V1

## Decision

M4 selects the existing `document.atom.oxidation.observe.v1` operation for a
bounded read-only closeout. This decision records scope and evidence only; it
does not declare M4 or oxidation complete. The active milestone authority is
[FULL_PARITY_RUST_FIRST.md](../active/FULL_PARITY_RUST_FIRST.md).

## Supported profile

- Assess one selected durable atom in one selected durable direct-root molecule.
- Fence the supplied CDML with its revision and lowercase SHA-256 digest.
- Return one signed `i16` oxidation number under
  `formal-electron-assignment-hcno-v1`.
- Admit materialized H, C, N, and O with integral charge `-4..=4`, explicit H
  vertices, zero authored explicit-hydrogen facts, non-aromatic single/double/
  triple bonds, zero radicals, at most 256 atoms, 512 bonds, and 64 components.

`accepted` is the only outcome containing `oxidation_number`. A structurally
admitted root outside this profile completes as `unavailable` with one closed
reason. Invalid input, stale fences, invalid direct-root/atom selection,
provenance failure, and resource limits remain typed refusals with their
existing recovery; they are not unavailable chemistry values. The exact V1
wire fields and closed categories remain in
[FERRUM_API_CONTRACT.md](../../FERRUM_API_CONTRACT.md).

## Ownership and boundary

| Owner | Responsibility |
| --- | --- |
| Chemistry | HCNO admission and oxidation reduction |
| Document | Direct-root/atom identity, snapshot fence, graph lowering, and refusal mapping |
| Protocol and CLI | Versioned execution dispatch and `ferrum document-atom-oxidation-observe --request <path|->` |
| PyO3 | Existing generic `execute_operation_v1` JSON bridge only |
| Qt | Frozen request, detached worker, source-fenced modeless result dialog, retry and recovery wording |

The generic PyO3 bridge is the sole public transport. No oxidation-specific
binding, live-session shortcut, document mutation, history entry, renderer
mark, SMARTS change, or known-group expansion is approved. The ordinary Qt
workflow remains the modeless `Chemistry -> Atom Oxidation State...` route
specified in [QT_CONTRACT.md](../../QT_CONTRACT.md).

## Completion gate

Oxidation remains incomplete until the protocol/CLI and Qt evidence lanes pass:

- The canonical executable HCNO corpus in
  [`document_atom_oxidation_corpus.rs`](../../../packages/ferrum-rust/crates/api/tests/document_atom_oxidation_corpus.rs)
  runs through the generic `execute_operation_v1` transport and proves its
  semantic corpus. The named CLI stdin, newline, envelope, typed-refusal, and
  recovery proof belongs to
  [`document_atom_oxidation_protocol.rs`](../../../packages/ferrum-rust/crates/api/tests/document_atom_oxidation_protocol.rs).
- A focused real Qt workflow against the compiled generic extension proves
  accepted and unavailable presentation, source-fenced historical status,
  source-tab-only rerun eligibility, source-tab retirement, and no mutation.
  Qt typed-refusal presentation is not a permanent timing or injection
  obligation; typed refusal and recovery remain proven in the protocol/CLI
  lane.

The corpus record belongs in
`docs/active_plans/reports/m4_atom_oxidation_corpus_v1.md`; it is human-facing
evidence linked to the executable protocol test, not a second oracle. Tests
use deterministic event processing and actual visible UI workflows where the
harness supports them.

## Exclusions and next dependency

This V1 does not claim general oxidation chemistry. Elements outside HCNO,
implicit or aggregate hydrogens, aromatic, radical, delocalized,
coordination, or unsupported-bond chemistry remain unavailable or refused.
Any broader chemistry requires a separately approved V2 corpus and contract.

After this evidence gate, the next M4 dependency is the remaining
chemistry-operation catalog work, beginning with its own selected bounded
contract. Known-group expansion remains a separate mutating M4/M5 decision.
