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

### Core-element anchors and full-ink bond clearance

**Decision.** Atom-label layout derives unversioned
`AtomLabelAttachmentGeometry` from the typed structural element run. Its exact
ink center is the local atom origin. `LaidOutAtomLabel` separately retains the
complete visible-ink bounds of the element, hydrogens, isotopes, and charge
annotations. Every atom-bond render request supplies an explicit positive
`BondInkClearance`; lowering clips against full label ink expanded by that
clearance and the final style-specific painted footprint.

**Why.** Centering a whole decorated label or attaching an axis to its text
baseline makes bond placement depend on incidental hydrogen and charge text.
Clipping only an abstract axis leaves visible stroke caps, bold widths, waves,
and Haworth extensions able to enter glyph ink. Structural element identity and
font measurements are already Rust-owned facts, so the renderer can establish
one geometry contract before Qt, SVG, PDF, or PNG consume its plan.

**Consequence.** The renderer uses one y-down script-baseline calculation,
centers the core-element ink for every admitted label, and treats full visible
ink as exclusion geometry. It reserves the resolved gap plus each style's
transverse radius and any axial overhang, refusing an unrenderable target rather
than emitting intersecting or partial ink. Qt replays the issued glyph and bond
operations without choosing a text anchor, recomputing glyph bounds, or relaxing
clearance. The closed `RenderObservationV2`/`RenderPlanV4` boundary publishes
the exact core/full ink bounds and positive clearance. The installed Qt E2E
replays the same Rust-owned corpus and proves that the expanded exclusion does
not intersect issued bond ink.

`BondAttachmentAxisV1` is the accompanying frozen semantic fact on every bond
batch. Its endpoints are the uncut structural connection points: an atom uses
its exact core-glyph center and a compact group uses its catalog connection
point. Rust constructs the axis before clipping; final typed operations remain
the only paint geometry. PyO3 transports the axis unchanged and Qt validates
but neither paints nor hit-tests it. Thus a bond has a durable center-to-center
attachment truth while visible ink still respects full-glyph clearance.

**Owner.** `packages/ferrum-rust/crates/render/src/glyph_metrics.rs`,
`packages/ferrum-rust/crates/render/src/verified_telex_glyph_metrics.rs`, and
`packages/ferrum-rust/crates/render/src/atom_bond/`.

### One selected-root export owns all singular text representations

**Decision.** One public `document.molecule.export.v1` operation owns export
of one authenticated direct molecule root as Molfile V2000/V3000, SDF
V2000/V3000, canonical SMILES, Standard InChI, or Fixed-Hydrogen InChI. Its
unversioned document core prepares the selected root once and contains no
runtime, path, publication, protocol, PyO3, or Qt state. The protocol returns
UTF-8 text; the named CLI may atomically create a new output file only after
that computation succeeds. The existing plural `document export-sdf` workflow
remains a separate multi-record operation.

**Why.** Separate format-specific routes had repeated root authentication,
graph preparation, native-runtime access, refusal mapping, and publication
logic. A file path is presentation concern, while root identity and chemistry
representation are document concerns.

**Consequence.** The exact request snapshot, direct-root ID, closed format
enum, source revision/digest, and typed refusal travel through protocol, PyO3,
Qt, and CLI without a second exporter. Coordinate-required formats require
coordinates; graph-only formats do not. The compact-group graph lowerer is the
single representation-support gate. Native writers receive the 128 KiB export
text ceiling before allocation, and the envelope is bounded again before
delivery. CLI publication reuses descriptor-relative atomic create-new
publication and rejects aliases or unsafe destinations.

**Owner.** `packages/ferrum-rust/crates/document/src/chemistry/document_molecule_export.rs`
and `packages/ferrum-rust/crates/api/src/protocol/document_molecule_export_v1.rs`.

### One live command catalog serves discovery surfaces

**Decision.** The unversioned Qt `CommandCatalogEntry` is the one immutable
presentation projection of a live `ActionRegistry` action joined with its
already validated YAML menu placement. Both Command Palette and the modeless
Command Reference consume that catalog; neither owns command metadata or
activation policy.

**Why.** Duplicating labels, shortcuts, help text, availability, or menu paths
would make help drift from the action a user can actually invoke.

**Consequence.** F1 and **Help > Command Reference...** open a nonmutating
surface that searches the live label, help, ID, native shortcut, and breadcrumb;
it reports unavailable commands rather than hiding them. Opening focuses the
filter, closing or Escape restores the invoking focus, and accessible names,
descriptions, and tab order remain explicit. The reference has no activation
route; Command Palette remains the action-invoking client.

**Owner.** `packages/ferrum-chem-qt.app/ferrum_qt/actions/command_catalog.py`
and `packages/ferrum-chem-qt.app/ferrum_qt/actions/command_reference.py`.

### Complete rendering is an atomic authoring invariant

**Decision.** Generic authoring compares the complete resolved candidate render
with the current resolved render. A candidate may retain or remove an existing
omission, but may not introduce a new root exclusion, plan issue, or member
depiction issue. The opaque admission value retains the exact candidate
realization and generic commit rederives it before mutation. A separate private
history policy authenticates an exact retained undo/redo target rather than
applying the current-to-target authoring delta.

**Why.** Root-level classification alone allowed a generated overlay to look
valid while an existing host bond disappeared behind a newly inserted atom
label. Requiring every document to be completely clean would make imported
diagnostic content impossible to repair or undo honestly.

**Consequence.** Ordinary operations are atomic across authored state and
visible state: an attached ring cannot commit if it suppresses the host C--O
bond. Imported diagnostics may be retained or repaired, and a repair remains
undoable, but no operation-specific bypass or Qt fallback may admit new missing
ink.

**Owner.** `packages/ferrum-rust/crates/render/src/complete_document_admission_v1.rs`
and `packages/ferrum-rust/crates/document/src/session/renderer_admitted_pending.rs`.

### Native linear-form spacing has one domain owner

**Decision.** The unversioned `LinearFormBondLength::NATIVE` value owns the
40-PostScript-point bond length for Ferrum-generated linear forms. The planner
uses it for coordinates and the document adapter writes and recognizes exactly
`<property name="bond_length" value="40" type="IntType"/>`.

**Why.** The previous duplicated 10-point constants produced generated
hydrogen-bearing forms whose labels and bonds could not be rendered completely.
Spacing is a durable construction choice, not a renderer exception or fixture
scale.

**Consequence.** There is no alternative writable 10-point grammar, layout
fallback, or admission bypass. Differently shaped imported forms remain
preservation-only, while every Ferrum-generated form uses the same domain value
through planning, metadata, validation, history, and save/reopen.

**Owner.** `packages/ferrum-rust/crates/domain/src/linear_form/types.rs` and
`packages/ferrum-rust/crates/document/src/typed_linear_form_metadata.rs`.

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
