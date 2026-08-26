# CDML Authoring Gesture Contract

This companion to
[CDML_BACKEND_TO_FRONTEND_CONTRACT.md](CDML_BACKEND_TO_FRONTEND_CONTRACT.md)
defines the bounded Rust-owned authoring and related session contracts.

## Stateless presentation authoring protocol V1

`presentation.author.v1` is the CLI/protocol counterpart for one request-owned
document mutation. Its closed typed variants are Vector, terminal Electron,
Retro, or Normal arrow, Curved Equilibrium arrow, Polyline/Polygon path, and
explicit-endpoint DirectBond. It returns a durable root result and accepted
document. The protocol owns its short-lived `DocumentSession`, capabilities,
and pending reservations; live gesture handles never cross this boundary.

DirectBond uses an existing durable atom ID or a finite new-atom point for each
endpoint. It accepts no Qt pointer state, viewport transform, hit evidence, or
preview. `PresentationAppearanceV1` admits validated RGB and bounded width
values rather than XML/style text. Its document-owned reservation reissues a
tentative presentation ID after an abandoned, stale, or refused candidate;
only a successful commit advances the durable sequence.

## Straight normal-arrow creation gesture V1

`presentation.creation.gesture.v1` is the Rust-owned authoring route for one
direct-root normal Arrow. Begin receives the current revision/digest, the closed
`straight_normal_arrow` kind, a finite scene point, an exact Arrow head-style
value, and a closed snap policy. Preview and commit consume only opaque handles.
Preview returns renderer-issued plan operations and bounds for disposable
frontend painting; it neither allocates an ID nor edits the document. Commit
accepts only the originating session's exact current
gesture/preview pair and returns one durable Arrow selector plus the ordinary
accepted snapshot.

Qt may map pointer coordinates and paint the returned overlay. It must not
calculate Arrow geometry, snapping, default style, identifiers, or CDML. Escape,
focus loss, tab close, tool changes, and handle disposal do not commit. Closed
Rust gesture categories direct recovery: endpoint geometry failures request an
adjusted endpoint, stale/session failures request refresh and restart, and
style-policy failures request a changed tool or style. After acceptance, the
frontend reprojects from the snapshot and restores selection only by the durable
selector through the root-interaction contract.

## Polyline and Polygon creation gesture V1

`presentation.path.gesture.v1` is the Rust-owned authoring route for one
direct-root Polyline or Polygon. Begin receives the current revision/digest and
one closed kind. The native lifecycle is one opaque point-at-a-time transaction:
`begin -> add accepted point -> incremental preview -> prepare -> commit`, with
`cancel` as the no-mutation terminal outcome. Each add receives exactly one
finite scene/PostScript point and returns Rust-derived progress. Incremental
preview receives only the opaque candidate and an optional hover point. Its
Rust-issued overlay always contains exactly accepted vertices; hover is
display-only and cannot become persistent geometry. Polyline requires at least
two points. Polygon requires at least three nondegenerate ordered points. Rust
owns bounded point and extent limits, default stroke/fill policy, and every
geometry refusal.

Prepare accepts only the exact session's gesture and a Rust-issued overlay for
its current accepted vertices. It constructs the complete persistent candidate,
preflights it through the renderer, and returns one opaque, fence-bound receipt.
Commit consumes that receipt once and returns the durable direct-root selector
with the ordinary accepted snapshot. Origin, revision, digest, preview, and
one-use fences remain mandatory. It is the sole persistent mutation:
validation, preflight, stale-session, and receipt failures are typed and
atomic, leaving document content and history unchanged. Cancellation is the
distinct `Cancelled` category with `DocumentUnchanged` recovery.

The PyO3 migration is complete and the retired full-vector preview bridge has
no remaining native or binding route. The ordinary Qt actions are `Draw
Polyline` and `Draw Polygon`. Qt converts its input to scene points, owns event
capture and user-facing wording, displays only the Rust-issued overlay, and
submits only opaque handles. It retains one transient accepted-press coordinate
solely to de-duplicate real and QTest double-click delivery; that coordinate is
not path geometry, validation, progress, or a persistent candidate fact.

The accepted end-to-end lifecycle is `begin -> add accepted point -> progress
-> optional-hover-preview -> prepare -> commit`, or `cancel`. Accepted points
are durable candidate geometry. Hover is display-only, and Rust derives both
the overlay's geometry and appearance. Escape, focus loss, tab close, tool
change, and gesture disposal cancel without a commit. Qt neither derives nor
serializes geometry, appearance, identifiers, CDML, validation, or a recovery
candidate. After success it discards the overlay, reprojects the returned
snapshot, and restores selection only through the durable root selector.
Geometry failures request adjusted points; stale or session failures request
refresh and restart.

A fresh local build and `./all_test.sh` validate this bounded slice; the dated
changelog records the command and result. The remaining presentation work is
separate: generic splines, variable-point-count grammars beyond these two
tools, path property editing, and association/factory semantics require their
own contracts.

## Directed stereobond creation

Direct-bond gesture is the sole current unversioned in-process Rust/PyO3
pointer capability for one direct covalent bond with a closed drawing
presentation. Its lifecycle is: begin the direct-bond gesture, resolve the
endpoint, prepare the generic `PreparedSessionTransitionV1`, then generically
commit that one-use transition. It is not a serialized or compatibility-bound
V3 contract. No route-specific public admit or commit facade exists. The
visible preview is the generic transition's retained operations; it is not a
separate public raw-preview API.

Qt owns pointer events, viewport-to-scene conversion, and scene-item
attribution. It submits a frozen finite pointer probe with exact `none`, unique,
or ambiguous hit evidence. `ferrum-document-render` resolves that UI probe and
the one-use authoring capability. `ferrum-document` owns the durable
`CreateDirectBondV1` request and generic transition: endpoint forms,
revision/digest fencing, chemistry, identity, renderer admission, history, and
the sole commit. V1 remains only on these durable document, fence,
presentation, snap, and transition values where it is the actual contract
version.
Separately, `ferrum-document` exposes native-Rust-only, interaction-neutral
direct-bond mutation for noninteractive programmatic work. Its input is an
already-resolved durable atom ID or finite
new-atom point, never a pointer probe, viewport transform, hit evidence,
snap/tolerance decision, overlay request, render plan, or issued operation.
Rust validates the revision/digest and session-origin fence, owns chemistry,
identity, renderer-admitted pending mutation, history, and CDML mutation, and
returns typed refusal without mutation. This interaction-neutral seam has no
Qt or PyO3 pointer input and accepts no render-plan input.

The first and second resolved probes respectively create the internal
`ExistingExisting`, `ExistingNew`, `NewExisting`, or `NewNew` form. The M3.P6
authoring actions have the bounded Normal, Solid wedge, and Hashed wedge
vocabulary. Solid and hashed wedges admit only covalent single bonds and write
`w1` and `h1`. The durable pointer start is the CDML `start` tip and pointer
end is the `end` base, so the issued overlay, committed projection, and
renderer share one tip-to-base record.

A malformed pointer probe has a typed closed category and recovery: correct the
input, adjust the endpoint, or refresh and restart. Once both probes resolve,
a document refusal instead preserves the native refusal category and selects
nonmodal recovery of refresh and restart, adjust endpoint, or change
presentation. A valid start/end hit on the same existing atom is the latter
`self_loop` / `adjust_endpoint` refusal, not malformed pointer input. Both
failure families leave document content and history unchanged.

Qt exposes `Draw Solid Wedge Bond` and `Draw Hashed Wedge Bond`, constructs the
two probes, paints only admitted Rust operations, and displays the typed
nonmodal recovery. Escape, focus loss, tab close, tool change, and gesture
disposal cancel the transient gesture without mutation. Existing Bond
Properties retains its independently supported broader bond-style vocabulary;
the M3.P6 drawing actions neither narrow that editor nor create a second
stereobond representation.

This V3 slice does not provide generic stereochemistry, CIP assignment or
inference, E/Z semantics, arbitrary depiction styles or orders, or broader
stereo import/export. Those requirements need separate typed chemistry and
interchange contracts. A fresh local build and `./all_test.sh` provide the
current validation receipt for this capability.

## Curved electron arrow V1

`presentation.electron-arrow.gesture.v1` is the Rust-owned authoring route for
one direct-root curved electron arrow. Its persistent CDML grammar is exactly
one `<arrow type="electron">` with three finite ordered `<point>` children:
`start`, `control`, and `end`. The closed grammar accepts neither extra points
nor another arrow type.

Begin receives the current revision/digest. Preview receives only its opaque
gesture and the three point roles. Rust owns quadratic geometry, its one-time
cubic lowering, terminal-head derivation, bounds, style, identifier allocation,
CDML serialization, renderer preflight, and every geometry refusal. It returns
only a disposable Rust-resolved overlay; no preview mutates the document or
allocates a persistent identifier.

Prepare accepts only the exact gesture/preview pair and returns one opaque,
fence-bound receipt. Commit consumes that receipt once, creates one history
entry, and returns the accepted snapshot and durable root selector. Invalid
points, stale sessions, renderer preflight failure, and receipt misuse are
typed and atomic: document content and history remain unchanged.

Qt captures the three scene points only. The first click records `start`; the
second records `control` and begins the native gesture; preview comes only from
Rust; the third records `end` and automatically commits the prepared receipt.
Escape, focus loss, tab close, tool change, and gesture disposal cancel without
a commit. Geometry refusals retain the appropriate recovery guidance; stale or
session refusals refresh then restart. Qt never derives curve geometry, cubic
controls, arrowhead, style, CDML, identifiers, or a replacement candidate.

## Curved retro arrow V1

`presentation.retro-arrow.gesture.v1` is the Rust-owned authoring route for
one direct-root curved retro arrow. Its persistent CDML grammar is exactly one
`<arrow type="retro">` with three finite ordered `<point>` children: `start`,
`control`, and `end`. The closed grammar accepts neither extra points nor
another arrow type.

Electron and retro arrows share the closed Rust
`CurvedTerminalArrowKindV1 { Electron, Retro, Normal }` model. For each kind,
Rust owns semantic path admission, style, identifier allocation, CDML
serialization, renderer preflight, and every geometry refusal. The renderer
issues the cubic axis, terminal head, and bounds as plan operations for preview
and final painting; the persistent projection retains authored curve facts and
no frontend derives a parallel curve or head.

Begin receives the current revision/digest. Preview receives only its opaque
gesture and the three point roles. Prepare accepts only the exact
gesture/preview pair and returns one opaque, fence-bound receipt. Commit
consumes that receipt once, creates one history entry, and returns the accepted
snapshot and durable root selector. Invalid points, stale sessions, renderer
preflight failure, and receipt misuse are typed and atomic: document content
and history remain unchanged.

Qt exposes the named Curved Retro Arrow action and captures the three scene
points only. The first click records `start`; the second records `control` and
begins the native gesture; Rust returns the preview; the third records `end`
and automatically commits the prepared receipt. Escape, focus loss, tab close,
tool change, and gesture disposal cancel without a commit. Qt refreshes and
restarts after stale or session refusals and never derives curve geometry,
cubic controls, arrowhead, style, CDML, identifiers, or a replacement candidate.

This contract excludes generic spline paths, variable point counts, start
heads, property editing, reaction association, and curved equilibrium arrows.
Each requires a separately named Rust record and interaction contract.

## Curved normal reaction arrow V1

`CurvedNormalReactionArrowV1` is the supported bounded Rust-owned curved-normal
reaction-arrow sibling. Its exact PyO3 route is
`begin_curved_normal_reaction_arrow_gesture_v1`,
`preview_curved_normal_reaction_arrow_gesture_v1`,
`prepare_curved_normal_reaction_arrow_gesture_v1`, and
`commit_curved_normal_reaction_arrow_gesture_v1`. It persists exactly one
direct-root `<arrow type="curved-normal">` with three finite ordered direct
`<point>` children: `start`, `control`, and `end`. The route never overloads
`<arrow type="normal">`.

`CurvedTerminalArrowKindV1::Normal` gives the route the same one-time
quadratic-to-cubic lowering and fixed terminal-head construction as the other
closed terminal-arrow kinds. Rust owns bounds, style, identifiers, CDML
serialization, renderer preflight, session-origin and fence validation, and atomic history
commit. Begin/preview/prepare/commit exchange only opaque gesture, preview,
and receipt handles. Geometry, stale/session, renderer-preflight, mismatched,
foreign-session, and replayed-receipt failures are typed; they leave document content and
history unchanged.

Electron, Retro, and Curved Normal use this same terminal-arrow receipt
lifecycle. Each gesture and prepared receipt is bound to the opaque origin of
the `DocumentSession` that began it, in addition to its revision/digest fence.
All supported opaque authoring lifecycles use the document-owned
`AuthoringCapabilityIssuerV1` and `AuthoringCapabilityV1` authority. A
`DocumentSession` owns one opaque issuer; each text placement, straight
presentation, catalog V1/V2, presentation-vector, DirectBond, terminal or
equilibrium arrow, presentation-path, and reaction
create/lifecycle/translation receipt carries a capability issued by that
session. Capability identity is the shared nonserializable allocation, never a
serializable nonce or process-wide counter. A receipt alias may be claimed by
one operation at a time; commit consumes its claim only after the authoritative
document transaction succeeds. A failed owner-side operation drops its
unsettled claim and restores the exact receipt to `Available`, preserving owner
retry semantics. Cancellation and successful commit are terminal, and the
state ends with the final opaque holder. There is no renderer-private
capability, bridge-origin accessor, global consumed-receipt registry, or
tombstone. Durable CDML identifier allocation remains independent. Catalog
recipes lower their atom, bond, geometry, and label facts to
`MoleculeInsertionV1`, then `DocumentSession` allocates the molecule, atom,
and bond identifiers in its opaque pending candidate. Terminal Electron, Retro,
and Curved Normal arrows, Curved Equilibrium arrows, incremental
Polyline/Polygon paths, and presentation vectors likewise receive their durable
presentation identity through the session's transactional
`PendingCreatePresentationV1` reservation. A pending candidate owns tentative
generated-ID sequences; discard or an unsuccessful commit leaves the session
allocator unchanged, and only a successful document transaction installs the
tentative sequences. Previews have no durable ID, and renderer routes own no
durable presentation counter. `CatalogPreviewLeaseV2` is renderer-local
transient preview-retirement state, never authoring or durable-ID authority.
A byte-identical foreign session receives `ForeignSession` with
`RefreshAndRestart` before geometry or candidate work; a foreign commit leaves
the owner's receipt redeemable.

Reaction creation uses the embedded `DocumentSession` issuer. Reaction
membership patch/delete uses the embedded document issuer of its
`RenderInteractionSessionV1`; aggregate translation retains its distinct
renderer-interaction origin only where that route's session fence requires it.
Those origins and the document capability fence callers without becoming
durable identifiers.

The Qt Curved Normal Reaction Arrow action captures only three scene points:
click one records `start`, click two records `control` and begins the native
gesture, and click three supplies `end` and commits the prepared native
receipt. Qt paints only the Rust overlay. Escape, focus loss, tab close, tool
change, and gesture disposal cancel transient capture without a commit.
Geometry refusals retain recovery guidance; stale or session refusals refresh
then restart. Qt never derives a curve, cubic controls, head, style, CDML,
identifier, or replacement candidate.

This supported bounded route excludes spline compatibility, variable point counts,
`start`, `end`, or `shape` facts, configurable heads, property editing,
reaction association, generic arrow factories, and curved equilibrium arrows.
Focused native/PyO3 and staged Qt evidence, followed by the full local suite,
accept this exact capability. They do not claim support for the excluded grammar
or editor workflows.

## Curved equilibrium arrow V1

`CurvedEquilibriumArrowV1` is the supported Rust-owned route for one
direct-root `<arrow type="curved-equilibrium">`. Its exact PyO3 lifecycle is
`begin_curved_equilibrium_arrow_gesture_v1`,
`preview_curved_equilibrium_arrow_gesture_v1`,
`prepare_curved_equilibrium_arrow_gesture_v1`, and
`commit_curved_equilibrium_arrow_gesture_v1`. The closed record has exactly
three finite ordered direct `<point>` children: `start`, `control`, and `end`.
It is neither an `equilibrium2` spelling nor an overload of a straight
`type="equilibrium"` arrow.

The renderer derives two translated quadratic lanes, lowers each to a cubic,
and issues two opposing terminal heads: one at the lower-lane start and one at
the upper-lane end. Preview and final painting consume those renderer-issued
plan operations; the durable projection retains authored curve facts. Rust owns
geometry admission, bounds, style,
identifier allocation, CDML serialization, renderer preflight, revision/digest
fencing, and atomic history commit.

Begin receives the current revision/digest plus `start` and `control`.
Preview receives the opaque gesture plus `end`; prepare accepts only that exact
gesture/preview pair; commit consumes the opaque prepared receipt once and
returns the accepted snapshot and durable root selector. Invalid non-finite,
short-span, collapsed, excessive, or unsuitable-control geometry, stale or
session state, renderer-preflight failure, mismatched preview, and replayed
receipt return typed refusals. Each leaves document content and history
unchanged.

Qt implements a dedicated `Draw Curved Equilibrium Arrow` action. Its
three-click lifecycle captures the three scene coordinates, paints only the
frozen Rust overlay, and commits only the prepared Rust receipt. It does not
calculate lanes, cubic controls, heads, style, identifiers, CDML, or a
replacement candidate. Escape, focus loss, tab close, tool change, and gesture
disposal cancel transient capture without a commit; geometry recovery asks for
adjusted points, while stale or session recovery refreshes then restarts.

This bounded profile rejects `equilibrium2`, generic or variable-point spline
semantics, `spline`, `start`, `end`, `shape`, `properties`, `association`, and
`factory` facts. Configurable heads and property editing remain separate
contracts. Focused native/PyO3 coverage, the staged Qt workflow, local CLI and
Qt E2Es, and the full local suite accept this exact capability. They do not
claim support for any excluded arrow grammar or editor workflow.

## Restore, history, and saved state

Restore copies a retained accepted snapshot into a new increasing revision; it
does not move the revision counter backward. The immediate pre-restore content
is retained as the one opposite restore target. A later restore replaces that
opposite target, and a normal accepted edit clears it.

The saved canonical-content baseline is independent of revision and undo
history retention. A session begins with its initial canonical content as its
saved baseline. `mark_saved` changes that baseline only after successful
external publication of the exact current snapshot. Clean/dirty compares the
current canonical content with this saved canonical content, not revision
numbers. Therefore restoring saved content is clean even though restore creates
a new revision.

History capacity, eviction mechanics, and performance limits are implementation
choices. They cannot change these observable rules: current content, the saved
canonical baseline, and immediate restore recovery retain their stated
semantics; an evicted older revision fails with the typed unavailable-revision
error; and eviction never changes whether current content is clean or dirty.

## External publication

Ordinary Save for a synchronized frontend session publishes the exact current
immutable backend snapshot, then marks that snapshot saved. A failure before
replacement leaves the target and saved baseline unchanged. A failure after
replacement but before baseline marking is a partial external result: the file
may contain canonical CDML, but the saved baseline remains unchanged.

Recovery Export writes an exact backend snapshot without changing backend or
frontend session state, including the saved baseline, dirty state, revision,
history, selection, or projection provenance. It is an export/recovery action,
not ordinary Save and not evidence that a frontend projection is synchronized.

## Visual artifact export

Visual output captures one immutable backend snapshot exactly once, plus only
durable selection IDs and scalar render options. The renderer returns artifact
bytes or a caller-controlled artifact path together with typed failures and
coverage warnings. It never receives a live frontend scene, document, widget,
or graphics wrapper as persistent input. SVG, PNG, PDF, cropped SVG, and
selected SVG therefore all describe the same captured revision; later scene or
selection changes cannot alter the artifact.

An unrenderable retained object produces a typed warning. Export never commits,
marks saved, consumes a token, or falls back to a retained frontend projection.
It publishes only after disposable render retirement; failure returns a typed
render failure with any cleanup diagnostic.

## Projection rules

Frontends rebuild all-or-nothing from the accepted snapshot. They may reuse
stable selection IDs, never old persistent objects, XML, or wrappers; failure
requires exact-snapshot reprojection or an unavailable frontend state. Complete
routes accept complete CDML; bounded routes accept documented IDs and scalars,
produce canonical CDML internally, and keep previews transient.

Direct atom-mark observations expose only actionable plain addresses, source
positions, removal ordinals, diagnostics, and finite rendering facts. Raw mark
XML remains backend-owned; legacy `atom_number` is a separate compatibility
diagnostic.
# Structural child selection and deletion

Direct atom/bond selection is a fenced Rust interaction contract.  Qt may send
only finite point or full-containment marquee coordinates plus replace/toggle;
Rust issues all target bounds, target identity, one-molecule selection, and the
atomic deletion receipt.  An atom target absorbs incident bonds during delete;
an explicitly selected bond preserves both endpoint atoms.  Blank canvas is an
ordinary empty selection.  Qt must clear a child selection after any commit,
tab change, focus loss, or new render observation and must not use scene item
selection, XML, or locally-derived bond topology as durable authority.

## Molecule report core V1

`document.molecule.report.v1` is the sole public read-only, revision-and-
digest-fenced molecule-report route. Its request carries bounded complete CDML,
unique durable direct-root molecule IDs, revision zero, and the input digest.
`ferrum-document` remains the authority for retained snapshots, direct-root
projection, typed-core corroboration, and fences. A private `ferrum-api`
protocol enclave owns report graph preparation and the one trusted chemistry
runtime callback. Neither crate exports a prepared report, graph, chemistry
engine, callback, adapter path, or report executor.

Each record always retains authenticated source identity, authored name when
present, root order, atom count, bond count, canonical authored-element counts,
and an authored charge only when every atom supplied a charge. Composition is optional: a
graph that falls outside the closed composition vocabulary remains a successful
record with bounded closed diagnostic findings. Neutral bond capacity is also a
record facet and reports `within_capacity`, `exceeds_capacity`, or
`not_checked`; an excess is a finding, not a request refusal. Combined
composition is emitted only when at least two selected records all have a
complete composition. Ferrum never aggregates a subset.

The public protocol represents each available record composition as one
all-or-none plain DTO: isotope- and
charge-aware formula, net formal charge, average molecular weight in Da,
monoisotopic mass in Da, and canonical formula-ordered isotope-aware element
rows. Each row carries symbol, optional isotope mass number, atom count,
average-mass contribution in Da, and mass percentage. All mass values are
finite. An unavailable record composition is `null` with its closed finding;
the required `aggregate` DTO is either `{"kind":"complete","composition":...}`
or `{"kind":"omitted","reason":...}`. Its reason is closed to
`fewer_than_two_selected` or `incomplete_record_composition`. No partial
formula, subset mass, contradictory complete-and-omitted state, or open reason
string is emitted.

Report findings use Rust-owned closed code, severity, recovery, and location
vocabularies. Optional detail is capped at 256 UTF-8 bytes, each record admits
at most 64 findings, and a storage/capacity breach refuses the whole operation
without mutation. Stale observations, digest mismatches, malformed admitted
CDML, and non-direct/ambiguous selectors remain whole-request refusals. The
protocol and CLI presentation layers serialize only the explicitly mapped
source, capacity, finding, aggregate, and composition facts; they do not repeat
CDML traversal or chemistry arithmetic. The named CLI route is `ferrum document
command document.molecule.report.v1 REQUEST.json`; normal typed refusal paths
remain read-only and never reveal CDML, a native-library location, or native
diagnostic payloads.
