# Document identity and ordering

## Decision

M7 adds an opaque-CDML index without assigning typed chemistry or reference meaning.
`IndexedDocument` keeps the accepted M6 `XmlDocument` tree unchanged and derives:

- `DocumentRecord` entries in exact direct-child `<cdml>` source order;
- a document-wide `id_index` of declaration `id` fields plus every unqualified XML
  `id` in opaque content;
- root-relative element paths for deterministic diagnostics; and
- document-local provisional tokens that are distinct from persistent IDs and consume once.

The index accepts only the canonical Ferrum CDML namespace. It rejects
blank persistent IDs and duplicate declarations with both structural locations,
including a root and descendant collision. M8 established that `fragment/bond@id` and
`fragment/vertex@id` are IDREF fields, so the index excludes those two recognized
references rather than falsely colliding with their declarations. It never rewrites or
resolves them, `idref`, endpoint-like attributes, or text. An `id` in opaque content
only reserves a collision name, as required for preservation; it gains no typed-record
or reference semantics.

Provisional tokens are unforgeable outside the crate and carry a private, process-local
document-instance component plus a document-local sequence. Every document starts at
sequence zero, but a token issued by one document is rejected by another. The component
is deterministic for the running process and deliberately has no persisted-document
meaning.

## Scope boundary

M7 does not expose typed CDML records, validate references, allocate durable IDs, or
mutate XML. Its narrow fragment-reference exclusion is a declaration-index rule, not a
typed payload API. M8 and later document/session milestones own those behaviors. XML
output remains M6 structural fidelity, not lexical byte identity.
