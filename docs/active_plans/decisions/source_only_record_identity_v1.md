# Source-only record identity V1 decision

## Status

Approved architecture decision. This is a pre-production foundational migration
after `document.molecule.diagnostics.v1` source-contract acceptance. It replaces
the mixed source/legacy identity design; it is not a compatibility migration.

## Stability evidence

The architect handoff identifies `DocumentObjectIdV1` as the cross-cutting
durable selector boundary. Graphify records its definition at
`packages/ferrum-rust/crates/document-projection/src/identity.rs:L9`, with degree
295. Its affected paths include `PresentationTargetV1`, render `RenderTarget`,
selection and session lowering, chemistry/export requests, and PyO3 bindings.
The decision therefore changes one identity contract before consumer migration;
it does not redesign rendering, diagnostics, or commands.

## Decision

### Internal source identity

Every persisted structural record and every recognized direct-root presentation
record admitted through typed ingress has one explicit, nonblank, unique source
ID. `RecordId` is internal only:

```rust
RecordId { kind, source_id }
```

Typed ingress validates canonical source-ID grammar and document-local
uniqueness before projection. It assigns or preserves the record's durable
identity before `RevisionState` serialization. Raw XML location context exists
only until it becomes typed admission data.

`LegacyFingerprint`, `RecordOrigin`, `from_legacy`, `legacy_occurrence`, legacy
serde forms, occurrence allocation, hash-derived identity, source-derived
identity, and decoder compatibility paths are removed. The historic BKChem
namespace remains rejected.

### Durable document identity

`DocumentObjectIdV1` is independently allocated as a high-entropy,
document-scoped opaque selector. It is persisted in Ferrum-owned namespaced
metadata, validated for canonical grammar and collision, and remains stable
through save, snapshot, history, undo, redo, and reload. It is never a hash of,
or derivation from, a `RecordId`, source ID, decoder position, or XML content.

For persisted records, `PresentationTargetV1` and render `RenderTarget` carry
only `DocumentObjectIdV1`. Preview targeting is a separate identifier-free
value and cannot stand in for a persisted target.

Public diagnostics and exclusions are source-free. An addressable result uses a
durable document ID; a result produced before allocation uses the closed numeric
`DocumentLocationV1` vocabulary. No public result exposes a source locator.

## Consequences

- Core graph records use admitted source IDs only; document identity is not a
  graph-derived fallback.
- Document ownership allocates, preserves, serializes, and collision-checks
  durable selectors independently from source records.
- Render, CLI, PyO3, and Qt interact only through durable targets after a
  record exists. Precommit overlays remain identifier-free paint values.
- Diagnostics can report admissible input failures without an invented record
  identity, using only `DocumentLocationV1` where an address is unavailable.
- `OTHER_REPOS/` remains read-only and supplies no runtime identity decoder.

## Migration phases

### A. Core source contract

Replace the mixed `RecordId` representation and serde with `{ kind, source_id
}`; delete fingerprint/origin/occurrence construction and persistence.

### B. Typed ingress and durable metadata

Require canonical, unique source IDs for structural and recognized direct-root
presentation records. Allocate or preserve independently generated durable IDs,
validate their grammar/collisions, and install them before `RevisionState`
serialization.

### C. Projection and public locations

Project source-only core records. Convert public diagnostics and exclusions to
durable IDs when addressable, otherwise to closed `DocumentLocationV1` values.

### D. Persisted and preview targets

Make `PresentationTargetV1` and `RenderTarget` durable-selector-only for
persisted records. Keep preview values distinct and identifier-free.

### E. Consumer convergence and removal

Move render lowering, CLI, PyO3, and Qt selection/mutation to durable targets.
Remove source-locator public paths and every legacy acceptance path.

## Evidence and sequencing

- Focused core and typed-ingress evidence proves canonical grammar, nonblank and
  unique source IDs, source-only serde, durable-ID allocation/preservation, and
  collision refusal.
- Document persistence evidence proves durable selectors survive save, snapshot,
  history, undo, redo, and reload independently of source records.
- Projection/render/API evidence proves persisted targets use durable selectors,
  previews carry no identifier, and diagnostics never expose source IDs.
- The migration may begin after diagnostics source-contract acceptance. Installed
  binding, registered E2E, fresh build, and `./all_test.sh` proof wait for the
  repository inventory to include the new authored artifacts; this is delivery
  sequencing, not a reason to defer the identity migration.
