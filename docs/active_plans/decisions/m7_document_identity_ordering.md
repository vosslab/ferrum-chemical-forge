# M7 document identity and ordering

## Decision

M7 adds an opaque-CDML index without assigning typed chemistry or reference meaning.
`IndexedDocument` keeps the accepted M6 `XmlDocument` tree unchanged and derives:

- `DocumentRecord` entries in exact direct-child `<cdml>` source order;
- a document-wide `id_index` of every unqualified XML `id`, including the root and
  opaque content;
- root-relative element paths for deterministic diagnostics; and
- document-local provisional tokens that are distinct from persistent IDs and consume once.

The index accepts the canonical CDML namespace and legacy no-namespace CDML. It rejects
blank persistent IDs and duplicate IDs with both structural locations, including a root
and descendant collision. It never rewrites or resolves `idref`, endpoint-like
attributes, or text. An `id` in opaque content only reserves a collision name, as
required for preservation; it gains no typed-record or reference semantics.

Provisional tokens are unforgeable outside the crate and carry a private, process-local
document-instance component plus a document-local sequence. Every document starts at
sequence zero, but a token issued by one document is rejected by another. The component
is deterministic for the running process and deliberately has no persisted-document
meaning.

## Scope boundary

M7 does not parse typed CDML records, classify known versus unknown elements, validate
references, allocate durable IDs, or mutate XML. Those behaviors wait for M8 and later
document/session milestones. XML output remains M6 structural fidelity, not lexical
byte identity.
