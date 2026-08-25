# Ferrum core model decision

## Scope and status

M2 supplies an immutable, chemistry-independent graph in `ferrum-core`. It carries
typed vertices, ordered bonds, source-field presence, and validated identity. It does
not parse or preserve CDML XML, call RDKit, render, hold PySide6 objects, or define a
wire ABI. Its Serde implementation is internal persistence/testing only; M17 owns the
versioned external DTO and operation protocol.

M2 is complete. The dev-only projection loads every committed corpus molecule, and the
separate-process comparison reports 81 exact source-fact agreements, 29 classified
differences, and zero unexpected differences. The comparison, classifications, and
intentional mutation proof are recorded in
`docs/active_plans/reports/corpus_molecule_parity.md`.

## Structural identity

### Supersession

The approved [source_only_record_identity_v1.md](source_only_record_identity_v1.md)
supersedes this document's former optional `source_id`, `RecordOrigin`, legacy
fingerprint, and idless-occurrence design. Those mechanisms are no longer
valid contracts or migration options.

Every typed persisted structural record and recognized direct-root presentation
record has a canonical, nonblank, document-unique source ID. Core `RecordId` is
internal only: `{ kind, source_id }`. Construction and serialization reject
missing, malformed, duplicate, or kind-mismatched source identity.

`DocumentObjectIdV1` is a separate, high-entropy, document-scoped opaque
selector. Ferrum-owned namespaced metadata persists it, validates its canonical
grammar and collisions, and preserves it through save, snapshot, history,
undo, redo, and reload. It is neither a source-ID, hash, fingerprint, decoder,
nor graph derivation. Typed ingress allocates or preserves it before
`RevisionState` serialization.

Persisted `PresentationTargetV1` and `RenderTarget` use only this durable
selector. Identifier-free preview targets are separate transient values. Public
diagnostics and exclusions use durable IDs when addressable and the closed
numeric `DocumentLocationV1` vocabulary before allocation; source IDs remain
internal and never appear in public results.

## Graph and mutation invariants

- `Molecule::graph()` builds an immutable analysis view whose `petgraph` indexes stay
  private. Public results contain stable Ferrum identities and owned errors.
- Components, paths, bridges, articulation points, matching, Dijkstra distances,
  Floyd-Warshall distances, diameter, cycle rank, and a deterministic fundamental-cycle
  basis are implemented. `deterministic_graph_analysis.md` owns their ordering policy.
- `VertexRef` is `Atom`, `Group`, molecule-local `Text`, or `Query`. Construction,
  validation, and deserialization reject a variant whose `RecordId.kind` disagrees.
  Bond endpoints preserve their type and order; chemistry code can explicitly require
  atom endpoints.
- Each endpoint resolves to a matching local vertex and cannot self-link.
- Internal source `RecordId` values and durable document selectors are unique in
  their respective document scopes.
- M2 mutation is replacement-only. `Atom::replace_source_fields`,
  `Bond::replace_source_fields`, and `Molecule::replace_records` are bounded
  immutable replacements that retain identity anchors; full document/session edit
  APIs wait for their contract.
- `Position` is finite. Present elements and source bond type tokens cannot be blank.
  Present multiplicity cannot be zero.

## Source field mapping

| Source field | Treatment | Presence policy or reason |
| --- | --- | --- |
| Molecule id and name | Carried | ID is required canonical source identity; name remains an optional source fact. |
| Atom id, name, point x/y, OASA z | Carried | ID is required canonical source identity; name remains optional and finite position is required in this core atom shape. |
| Charge, isotope, explicit H, valency, multiplicity, free sites | Carried | `Option` preserves absent versus default source state. |
| Atom periodic facts, formula, mass, valence perception | Computed | Chemistry adapter/RDKit owns derivation. |
| Group, text, query identity | Carried minimally | Required canonical source identity for typed endpoint resolution; document layer owns payload. |
| Bond id, start/end, source type | Carried | ID is required canonical source identity; source type may preserve absence and endpoints stay ordered and typed. |
| Bond order, style, aromatic flag | Carried optionally | No missing source type/order/style is normalized into an authored default. |
| Bond depiction attributes and atom presentation | Dropped | M6-M8 document and M12 rendering own them. |
| Stereo, cycles, bond length | Computed or deferred | Chemistry/geometry contracts own derivation. |
| Molecule `template` and `fragment` children | Dropped | Molecule-scoped IDREF metadata. `template` names an attachment atom and up to two attachment bonds; `fragment` names a substructure through `bond` and `vertex` IDREF children. Neither declares a vertex or a bond, so the loaded graph is identical with or without them. M8 types both, as `molecule/template` and `molecule/fragment`. |
| Molecule `display-form` and `user-data` children | Dropped | Documented preservation-only containers whose entire subtree is uninterpreted payload, including descendants that use canonical CDML names. They contribute no graph member and no carried scalar. M8 assigns both as opaque payload containers; M6 owns their survival. |
| Foreign-namespace attributes on a molecule, atom, or bond | Dropped | An attribute outside the CDML namespace is vendor payload, never core vocabulary, and an attribute cannot introduce a vertex or a bond. M8 stores it in that record's unknown-attribute bag with its lexical QName, expanded name, literal value, and in-scope namespaces. |
| Atom `local_extension` | Dropped | The one unqualified atom attribute outside the documented atom grammar. M8 pins it as the example proving an unfamiliar attribute leaves a record typed. It is named here individually rather than by an unqualified catch-all, so a misspelled carried attribute such as `valancy` stays an error instead of vanishing. |

## Validation evidence

- Serde uses private wire records and validates admitted source identities plus
  independently allocated durable selectors before accepting Position, Atom,
  Bond, vertex, or Molecule values. It neither derives nor rehydrates a legacy
  identity.
- Evidence rejects missing, malformed, duplicate, or spoofed source identity;
  kind-mismatched endpoint variants; malformed/colliding durable selectors; and
  nonfinite positions. It preserves durable selectors independently through
  revision-state persistence while retaining absent non-identity source fields.
- One property varies every carried optional atom scalar plus bond order, style, and
  aromatic presence through internal serialization without collapsing absence into a
  default.
- The M2 dev loader reads all three committed documents, assigns idless occurrences,
  converts coordinate units, resolves all four vertex kinds, applies versioned bond
  semantics, and rejects unassigned molecule content.
- The corpus gate catches an intentional atom-element mutation. Exact shared source
  facts agree; source-default, non-atom, style/aromatic, and legacy `d` differences are
  classified in the parity report rather than hidden by normalization.
