# Design decisions

<!-- VENDORED HEADER: START -->
Record each durable decision about how this code and repository are shaped, once it is settled, with
the reasoning a later reader needs. Guidance Neil Voss states belongs in
[HUMAN_GUIDANCE.md](HUMAN_GUIDANCE.md), dated history in `docs/CHANGELOG.md`, open discussion in
`docs/active_plans/decisions/`. [PROPAGATED HEADER - ENTRIES BELOW ARE YOURS]
<!-- VENDORED HEADER: END -->

Write each decision as a level-three heading with these four fields. `Owner` names the
authoritative code or contract document, rather than a person.

```markdown
### <decision title>

**Decision.** <the durable direction>

**Why.** <the reason it was chosen>

**Consequence.** <the constraint a future change preserves>

**Owner.** <the authoritative code or contract doc>
```

## Software design

### Rust owns the local File/Open catalog

**Decision.** `LocalDocumentOpenCatalogV2` is the sole File/Open discovery and
admission authority. It issues opaque route handles for native CDML, decoded
SVG, and every `DocumentImportNew` interchange descriptor.

**Why.** A split Qt/Python catalog could reselect a parser from a suffix and
drift from Rust admission policy.

**Consequence.** Qt retains and returns the issued handle to one generic
preparation API. File/Open creates or replaces a document; `File > Import SDF`
remains the separate current-drawing insertion workflow.

**Owner.** [QT_CONTRACT.md](QT_CONTRACT.md) and
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

### Rust issues molecule-report identifiers

**Decision.** Every molecule-report record has one required identifiers facet:
the complete canonical-SMILES, Standard InChI, Standard InChIKey trio, or the
closed unavailable reason `unsupported_molecule` or `chemistry_unavailable`.

**Why.** Identifiers are chemistry results, so a Qt fallback, partial field, or
native diagnostic leak would create a second chemistry authority.

**Consequence.** Rust evaluates the trio in dependency order. Resource
exhaustion remains an operation-level `resource_limit` refusal, not a partial
report. Qt presents exactly the issued tagged outcome.

**Owner.** [FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

### Rust projects the periodic picker

**Decision.** Rust owns the bounded next-drawing periodic display catalog:
symbol, display name, grid coordinates, category, and color. Qt projects it
without a Python element catalog.

**Why.** The picker and editable next-atom control must share one chemical
vocabulary while the picker remains a preference control, not document state.

**Consequence.** Accepted picker choices call only the shared drawing-parameter
model. They update the preference and peer clients, never CDML, history,
revision, digest, or structure selection.

**Owner.** [QT_CONTRACT.md](QT_CONTRACT.md) and
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

### Tab owns structural action selection

**Decision.** The delivered M6 bridge makes `FerrumNativeDocumentTab` the owner
of an optional, fenced, Rust-issued structural action selection. The controller
replaces or clears it; the projection provides only visual bounds feedback.

**Why.** Molecule-root projection intentionally has no per-atom or per-bond Qt
identity, so scene selection cannot represent a structural member reliably.

**Consequence.** The bridge validates selection revision and digest against the
installed snapshot, exposes exact Rust targets to actions, and clears them
before successful replacement, refresh failure, cancellation, mode/tab change,
or disposal. Python must not reconstruct target kind or identity from IDs.

**Owner.** [QT_CONTRACT.md](QT_CONTRACT.md) and
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

## Dependencies

## Generated artifacts
