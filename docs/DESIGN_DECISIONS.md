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

### Semantic label registers and bond-owned attachment corridors

**Decision.** Rust classifies every atom-label run as structural element,
isotope, explicit hydrogen/count, or charge before layout. The structural
element's exact ink center is the atom origin. Each resolved bond style owns an
`AtomLabelAttachmentCorridor` derived from its complete final terminal
footprint: approach direction, occupied transverse interval, optical gap, and
decoration clearance. The label solver chooses the minimum valid translation
within conventional semantic registers, then final-operation admission checks
the emitted painted geometry against every endpoint decoration.

**Why.** Atom labels are structured chemical notation rather than one text box.
The [IUPAC graphical-representation recommendations](https://iupac.qmul.ac.uk/drawing/drawing.html)
place isotope mass at upper left, hydrogen beside the element, charge at upper
right when space permits, and bonds at the atom symbol without impinging on it.
A baseline-only or bounding-box-only layout loses those roles. An abstract bond
axis also omits the real width of parallel lanes, waves, wedges, caps, and miter
joins. Rust already owns label meaning, verified font metrics, and final style
lowering, so one constraint boundary can serve Qt, SVG, PDF, and PNG.

**Consequence.** Isotope candidates remain in the upper-left sector;
hydrogen/count candidates use left or right baseline registers; charge
candidates use upper-right, above, below, or upper-left registers. Candidate
translations are solved analytically from exact glyph ink and all incident
bond corridors, then ordered by minimum movement. Core outline support owns
axial endpoint clipping, while decoration ink constrains placement instead of
detaching every parallel lane. Double and triple bonds use one combined
terminal envelope and a 1.5-stroke renderer target inside the independent
1.75-stroke raster ceiling. Wavy amplitude, terminal caps, axial overhang, and
true angle-derived miter extent are part of final footprint geometry. The
renderer emits a typed issue when no complete placement exists and performs an
exact post-lowering collision check before admitting the plan.

Qt replays issued glyph and bond operations without choosing registers or
recomputing geometry. The closed `RenderObservationV2`/`RenderPlanV4` boundary
publishes exact core/full ink bounds and positive clearance. The installed Qt
E2E and independent pixel lane validate final output rather than the private
solver's intermediate values.

`BondAttachmentAxisV1` is the accompanying frozen semantic fact on every bond
batch. Its endpoints are the uncut structural connection points: an atom uses
its exact core-glyph center and a compact group uses its catalog connection
point. Rust constructs the axis before clipping; final typed operations remain
the only paint geometry. PyO3 transports the axis unchanged and Qt validates
but neither paints nor hit-tests it. Thus a bond has a durable center-to-center
attachment truth while visible ink still respects full-glyph clearance.

The independent pixel policy requires at least 0.60 measured stroke widths of
clearance for double and triple bonds, compared with 0.20 for single-stroke
styles. This stronger parallel threshold makes the optical distinction
executable without supplying renderer geometry to the measurement process.

**Owner.** `packages/ferrum-rust/crates/render/src/glyph_metrics.rs`,
`packages/ferrum-rust/crates/render/src/verified_molecule_label_glyph_metrics.rs`,
`packages/ferrum-rust/crates/render/src/atom_bond/atom.rs`,
`packages/ferrum-rust/crates/render/src/atom_bond/bond.rs`, and
`packages/ferrum-rust/crates/render/src/atom_bond/final_ink_collision.rs`.

### Molecule-label font selection belongs to one Rust role

**Decision.** Ferrum vendors all 92 official version 2.001 outputs in the
Atkinson Hyperlegible Next and Atkinson Hyperlegible Mono families. Each family
has 14 static OTF faces, 14 static TTF faces, two variable TTF faces, 14 static
WOFF2 faces, and two variable WOFF2 faces. The unversioned `FerrumFontEnvironment`
selects proportional Atkinson Hyperlegible Next Regular for the
`molecule_label()` role. PyO3 exposes that role as `molecule_label_font()` and
transports immutable resource facts in the unversioned
`VerifiedMoleculeLabelFont` value.

**Why.** Atom labels such as `Cl`, `Br`, and `13C` are compact chemical tokens,
not tabular data. At 1000 units per em, proportional Regular measures `Cl` at
907 units, `Br` at 962, and `13C` at 1628; Mono measures the same strings at
1264, 1264, and 1896. Proportional Regular therefore preserves readable glyph
distinction without imposing unrelated fixed-width spacing. Vendoring both
complete families keeps later UI, code, and accessibility roles available
without coupling those choices to the molecule renderer.

**Consequence.** `assets/fonts/catalog.json` is the inventory and integrity
authority for every face, both exact upstream revisions, both OFL-1.1 notices,
and every byte length and SHA-256 digest. Rust embeds and verifies only the
selected Regular face for current molecule rendering. Qt receives that exact
resource from Rust and only replays Rust-issued glyph IDs and origins. A future
role selection changes one owner and must regenerate native and Qt evidence;
there are no font-generation aliases or compatibility schemas.

**Owner.** `packages/ferrum-rust/crates/render/src/font_environment.rs`,
`packages/ferrum-rust/crates/render/src/font/molecule_label.rs`,
`packages/ferrum-rust/crates/render/assets/fonts/catalog.json`, and
`packages/ferrum-rust/crates/api/src/python_binding/render_binding.rs`.

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

### One preflighted ribbon contract owns command presentation

**Decision.** The closed ribbon YAML owns quick access, global actions, task
placement, responsive role and priority, and semantic accent. One exact command
icon catalog joins every declared command to packaged BKChem or Qt-standard
artwork before the ribbon becomes visible; the shared `ActionRegistry` remains
the only behavior and state owner.

**Why.** Generic tabs, repeated universal commands, and ad hoc per-window icons
created weak hierarchy and allowed menus, ribbon buttons, and theme changes to
drift. A ribbon is a command projection, not another command system.

**Consequence.** Ferrum presents a persistent quick-access row, task tabs,
color-coded cards, labelled overflow, and theme-aware icons without duplicating
actions. Malformed layout or incomplete icon/theme data refuses the window
atomically. Responsive reduction preserves action identity, reachability, and
keyboard focus.

**Owner.** `packages/ferrum-chem-qt.app/ferrum_qt/ribbon_contract.py`,
`packages/ferrum-chem-qt.app/ferrum_qt/resources/ribbon_layout.yaml`,
`packages/ferrum-chem-qt.app/ferrum_qt/actions/command_icons.py`, and
`packages/ferrum-chem-qt.app/ferrum_qt/ferrum/authoring_ribbon.py`.

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

### Native chemistry proportions use one point-space scale

**Decision.** Ferrum's built-in chemistry presentation uses the same 40-point
bond and 12-point atom-label scale as the local BKChem/OASA Qt reference.
Built-in double- and triple-bond lane separation is four times the resolved
line width; an explicit CDML `bond width` remains authoritative. Ordinary
documentation structures begin as exact SMILES and use the active UI's
40-point canonical bond scale. An oversized depiction receives a typed paper
or view change, not a private molecular scale change.

**Why.** The OASA default lane spacing is 6 px with a 1.5 px stroke, a
four-stroke proportion. Carrying 6 into Ferrum beside its 1-point stroke made
parallel lanes six stroke widths apart. Documentation fixtures separately used
200- and 220-point bonds, which made ordinary atom labels look implausibly
small even though native Ferrum authoring already used 40-point spacing. A
second documentation-only 22-point scale would similarly decouple glyph
spacing from the UI drawing standard rather than solve page framing.

**Consequence.** Default multiple bonds remain legible without a wide empty
channel, native and captured chemistry share one bond-to-label scale, and
authored imported widths remain unchanged. The PubChem YAML is the source for
ordinary documented molecules when it contains the requested CID; the exact
user-provided CID 65146 SMILES remains its source because the YAML's CID 94190
record is a different stereospecific compound. RDKit deterministically derives
coordinates and wedges from isomeric SMILES, Rust uniformly places the graph
and owns document semantics, and Qt replays that issued geometry without a
visual correction. Explicit placement scales remain legitimate for editor and
import behavior; this decision fixes the documentation depiction profile, not
every Ferrum placement globally.

**Owner.** `devel/documentation_biomolecule_sources.py`,
`devel/documentation_biomolecule_geometry.py`,
`packages/ferrum-chem-qt.app/ferrum_qt/config/geometry_units.py`,
`packages/ferrum-rust/crates/chemistry/native/ferrum_chem_adapter.cpp`, and
`packages/ferrum-rust/crates/geometry/src/molecule_placement.rs`.

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
