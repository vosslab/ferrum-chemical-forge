# Ferrum core model decision

## Scope and status

M2 supplies an immutable, chemistry-independent graph in `ferrum-core`. It carries
typed vertices, ordered bonds, source-field presence, and validated identity. It does
not parse or preserve CDML XML, call RDKit, render, hold PySide6 objects, or define a
wire ABI. Its Serde implementation is internal persistence/testing only; M17 owns the
versioned external DTO and operation protocol.

M2 remains in progress. The M1d oracle harness has landed: `tests/e2e/e2e_oracle_molecule_core.py`
runs a separate-process comparison against pinned OASA 26.2a1 and RDKit 2026.03.4 and reports
`"status": "match"`. Corpus loading and pinned OASA field comparison across the corpus still do
not exist. The remainder is no longer M1d's; it is scheduled as six atomic M2 steps in
[../audits/m2_exit_gap.md](../audits/m2_exit_gap.md), which tracks the remaining work.

## Structural identity

- `source_id: Option<Identifier>` preserves exact CDML `@id` presence. The core
  never fabricates a source ID.
- `RecordId` is structural: `RecordKind` plus `RecordOrigin::Source(exact_id)` or
  `RecordOrigin::Legacy { fingerprint, occurrence }`. Public constructors cannot
  create arbitrary string identities.
- Source-backed records derive their identity from the exact source ID. Validated
  construction and deserialization reject a source/kind/origin mismatch.
- Legacy fingerprints use `ferrum-core-legacy-v1`, record kind, and length-prefixed
  UTF-8 fields. The format is unambiguous, deterministic across processes/platforms,
  and versioned for future incompatible changes.
- Internal snapshot rehydration parses the complete encoding, checks its version,
  embedded kind, field lengths, full consumption, and record-kind field shape before
  accepting it. This is structural integrity checking, not a cryptographic security
  boundary and not proof of historical semantic provenance after an edit.
- Atom fingerprints include all carried scalar presence and values plus coordinate
  bits. Bond fingerprints include source-ID presence, ordered typed endpoint identity,
  source-type presence/value, optional observed order/style, and aromatic presence.
- A fingerprint seeds a legacy identity only at load/construction time. It is not
  recomputed to rename an existing idless record after a replacement edit. Source ID
  identities are reload-stable; idless identities are typed document/session anchors.
- Internal Serde rehydration validates record kind, origin, source-ID/occurrence
  consistency, scalar invariants, endpoint kinds, and graph resolution while retaining
  an already-issued legacy anchor. It does not reseed that anchor from edited fields.

## Idless duplicate policy

An idless record adds an occurrence only among records with an identical canonical
fingerprint. It is not a general collection position: nonidentical records retain the
same identity when reordered. The CDML reader assigns the occurrence while loading,
and internal persistence retains it, so exact duplicates stay distinct during a live
session and through an internal snapshot round trip. For idless group/text/query
vertices, this typed occurrence is the minimal document-provided anchor until M6-M8
can derive one from the typed or opaque CDML node; it is never an arbitrary raw string.

Source-only reload has an unavoidable limitation: exact duplicate idless records have
no source fact that distinguishes them. Their assigned occurrences are deterministic
within the loaded sequence but cannot preserve user meaning across a reorder of those
indistinguishable duplicates. The core therefore supports representable documents
without pretending that source XML contains an absent identity. M6 can improve the
document-node anchor, but cannot invent semantic distinction absent from source data.

## Graph and mutation invariants

- `VertexRef` is `Atom`, `Group`, molecule-local `Text`, or `Query`. Construction,
  validation, and deserialization reject a variant whose `RecordId.kind` disagrees.
  Bond endpoints preserve their type and order; chemistry code can explicitly require
  atom endpoints.
- Each endpoint resolves to a matching local vertex and cannot self-link.
- Internal identities and present molecule-local source IDs are unique.
- M2 mutation is replacement-only. `Atom::replace_source_fields`,
  `Bond::replace_source_fields`, and `Molecule::replace_records` are bounded
  immutable replacements that retain identity anchors; full document/session edit
  APIs wait for their contract.
- `Position` is finite. Present elements and source bond type tokens cannot be blank.
  Present multiplicity cannot be zero.

## Source field mapping

| Source field | Treatment | Presence policy or reason |
| --- | --- | --- |
| Molecule id and name | Carried | Both remain optional source facts. |
| Atom id, name, point x/y, OASA z | Carried | ID/name preserve absence; finite position is required in this core atom shape. |
| Charge, isotope, explicit H, valency, multiplicity, free sites | Carried | `Option` preserves absent versus default source state. |
| Atom periodic facts, formula, mass, valence perception | Computed | Chemistry adapter/RDKit owns derivation. |
| Group, text, query identity | Carried minimally | Required for typed endpoint resolution; document layer owns payload. |
| Bond id, start/end, source type | Carried | ID/type preserve source absence; endpoints stay ordered and typed. |
| Bond order, style, aromatic flag | Carried optionally | No missing source type/order/style is normalized into an authored default. |
| Bond depiction attributes and atom presentation | Dropped | M6-M8 document and M12 rendering own them. |
| Stereo, cycles, bond length | Computed or deferred | Chemistry/geometry contracts own derivation. |
| Molecule `template` and `fragment` children | Dropped | Molecule-scoped IDREF metadata. `template` names an attachment atom and up to two attachment bonds; `fragment` names a substructure through `bond` and `vertex` IDREF children. Neither declares a vertex or a bond, so the loaded graph is identical with or without them. M8 types both, as `molecule/template` and `molecule/fragment`. |
| Molecule `display-form` and `user-data` children | Dropped | Documented preservation-only containers whose entire subtree is uninterpreted payload, including descendants that use canonical CDML names. They contribute no graph member and no carried scalar. M8 assigns both as opaque payload containers; M6 owns their survival. |
| Foreign-namespace attributes on a molecule, atom, or bond | Dropped | An attribute outside the CDML namespace is vendor payload, never core vocabulary, and an attribute cannot introduce a vertex or a bond. M8 stores it in that record's unknown-attribute bag with its lexical QName, expanded name, literal value, and in-scope namespaces. |
| Atom `local_extension` | Dropped | The one unqualified atom attribute outside the documented atom grammar. M8 pins it as the example proving an unfamiliar attribute leaves a record typed. It is named here individually rather than by an unqualified catch-all, so a misspelled carried attribute such as `valancy` stays an error instead of vanishing. |

## Validation evidence

- Serde uses private wire records and structurally validates issued identities and
  anchors before accepting Position, Atom, Bond, vertex, or Molecule values. It only
  derives a fresh identity at initial construction, never while rehydrating an edited
  legacy record.
- Tests reject spoofed source identity, kind-mismatched endpoint variants, and
  nonfinite position; distinguish delimiter inputs in canonical fingerprints; preserve
  absent bond type/order/style; prove child-set/reorder stability for idless molecules;
  retain exact duplicate idless occurrences in a session; and exercise typed
  endpoint/source-absence round trips.
