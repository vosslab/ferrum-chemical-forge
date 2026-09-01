# Ferrum contract

Ferrum is a Qt Widgets document editor. This contract defines its durable
Rust-native ownership boundary. It follows the canonical
[active_plans/active/FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md)
and applies the repository principles "Fix the design, not the symptom" and
"Long-term over short-term."

## Current composition

```text
MainWindow
  native CDML and decoded-CD-SVG tabs
    Rust-owned ferrum_chem.DocumentSession and disposable Qt projection
  native File, editing, authoring, history, recovery, and export actions
```

`MainWindow` owns Qt widgets, modal lifecycle, and disposable projection. Rust owns
CDML admission, saving, chemistry/document operations, render plans, and publication.
There is one production editor and one Rust-native session model. Historical CDML namespaces
remain valid document data; they are not product branding.

## Current completed slices

The public `MainWindow` is the bounded Rust-native editor. It opens, renders, saves, reopens,
and closes native tabs with an extension-owned `ferrum_chem.DocumentSession`.
It can change one durably selected atom's element, add one free-standing atom
at an exact scene point in an explicitly chosen durable molecule, and apply
Rust-owned undo and redo. Its reachable actions expose only operations with a complete
Rust implementation; unsupported historical workflows are explicitly refused or dropped.

Current V1 Qt slices also include the exact Rust molecule-plan painter, the
separate presentation-vector projection, and a Rust/PyO3 display-geometry
bridge. These are bounded evidence for their own routes, not completion claims
for full geometry, rendering, editing, or import/export coverage.

The closed CDML paper-name catalog is also Rust-owned and crosses PyO3 as frozen
name and millimetre-dimension values. Qt scene setup and snapshot rendering
consume those issued values. Qt does
not maintain a second paper table or infer missing dimensions. The standalone
native observation resolves orientation and physical dimensions into one finite
scene rectangle at the document origin using 72 points per inch. Qt paints that
rectangle as a noninteractive page decoration behind document roots and may use
the current UI palette for its fill and outline. Preserved malformed paper facts
remain document data; Rust supplies a typed compatibility issue and the A4
portrait display fallback. Normal-window paper/session adoption remains open.

The bounded periodic picker is another Rust projection, not a Qt chemistry
catalog. Rust publishes each picker symbol, display name, grid row/column,
category, and color as frozen facts. Qt uses them for the accessible
**Periodic table...** view beside Next atom and submits accepted symbols only to
the shared drawing-parameter preference model. A choice changes neither CDML,
session revision/digest, history, nor structural selection.

Ordinary native `File -> Export...` never paints the installed view scene or a Qt
projection. Qt captures the active registered tab and its immutable
`SessionDocumentObservationV1`, revision, digest, and opaque local-origin token
before the save chooser. After a selected destination passes the same current/idle
fence, a `QThread` gives only that immutable observation to a private Rust bridge.
Rust derives and renders one complete SVG, vector PDF, or transparent
one-pixel-per-page-point PNG receipt. Qt reauthenticates the same fence on delivery
and asks Rust to publish only a still-current receipt through the descriptor-relative
publisher. A complete-plan exclusion refuses the whole artifact; selection and
hover feedback are absent because they are not document facts.

The `Export PNG (1 pixel per point)...` label describes the Rust page geometry,
not an encoded physical-density metadata promise. A local CDML or decoded CD-SVG
origin remains an opaque Rust-held descriptor token. When present, Rust clones that
live descriptor into publication and rejects the original source or an observed
hard-link alias as a destination. Qt does not compare paths or hold a Python file
object. Cancel, stale delivery, close, and incompatible busy work make no
publication or success claim. Confirmed, directory-entry-unconfirmed, not-started,
rejected-destination, and possibly-published results have distinct recovery wording.
`Recovery Export CDML...` is a separate current-document recovery copy; it does not
replace Save/Save As or artifact export and does not convert unsupported formats.

## Authority and flow

In a Rust-owned tab, one Rust `ferrum-document` session is the authority for
the complete CDML document, durable identifiers and source order, revision and
history, dirty/save baseline, accepted mutations, recovery publication, and
canonical save. `ferrum-chemistry` owns chemistry. `ferrum-render` owns
declarative rendering. The PyO3 module `ferrum_chem` publishes copied, frozen
values and typed errors at this boundary.

Qt owns only disposable scene projection, selection interaction, modal state,
view transform, widgets, and connection lifetime. A Qt scene never becomes
document authority. Python never reconstructs a Rust snapshot by parsing XML,
and Rust never receives a `QGraphicsItem`, Qt object, mutable Python map, or
anonymous render primitive.

```text
gesture/action + expected revision
  -> FerrumSession adapter on the Qt thread
  -> ferrum_chem frozen input/output boundary
  -> Rust session accepts or rejects the operation
  -> immutable observation and typed outcome
  -> detached Qt projection
  -> atomic scene replacement and semantic UI update
```

Each arrow is one direction. Qt submits immutable intent with the observation
revision it observed. Rust returns an accepted observation or a typed failure.
Qt rebuilds its projection from that observation and emits semantic UI events
such as title, dirty, and diagnostics. Sibling widgets
do not call each other's internal methods, and no action retains a Qt item
across a backend call.

For atom insertion, Qt borrows the durable molecule selector from the current
`SessionDocumentObservationV1`, captures its revision and digest, and maps one
viewport click to one exact scene point. Rust resolves that selector against
the current typed document, allocates the persistent atom identifier from the
session-owned sequence, prepares one-use candidate XML including the explicit
point, and commits it through normal history. Caller-provided atom identifiers,
Qt provisional identifiers, hidden snapping, and implicit molecule selection
are outside this contract. Rejected validation does not advance the allocator;
an abandoned successfully prepared candidate leaves a deliberate identifier
gap.

## Session observations

The initial Rust-facing session surface is revision guarded:

- `observe(expected_revision)` returns a frozen snapshot and projection facts.
- A mutation, undo, redo, save, or recovery export receives the expected revision.
- `prepare_create_atom_v1` receives a current durable molecule selector, element,
  and finite position; Rust alone supplies the new persistent atom identifier.
- An accepted result returns the current authoritative observation and outcome.
- A revision conflict, rejected operation, publication failure, or render failure
  is a stable typed error with actionable context.

The document layer owns `SessionDocumentObservationV1`, containing the
revision-bound snapshot and typed document projection. It is the observation
for document and presentation facts. The API layer composes that observation
with verified render metrics into `RenderObservationV2`, the separate
observation for molecule render plans. Neither value replaces the other, and
Qt does not manufacture either from XML or graphics.

The frontend maintains the latest accepted observation appropriate to each
projection. It derives title, dirty indication, and action enablement from the
latest accepted document observation. It derives molecule graphics only from
the matching render observation. A rejected request leaves the current
projection in place. A successful save changes the durable baseline only when
Rust reports the publication result and Qt installs a matching fresh render
observation; the UI does not mark a document saved itself.

## Render-plan painter

The Ferrum molecule painter consumes a final `RenderObservationV2`, composed by
the API from one revision-checked `SessionDocumentObservationV1` and verified
render metrics. It does not consume retired atom or bond models or Python XML.
The final observation contains frozen
`DocumentMoleculeRenderPlanV4` values in document root order. Each entry keeps
document-root molecule identity and order separate from the molecule-local atom
and bond order inside its `RenderPlanV4`.

```text
SessionDocumentObservationV1
  snapshot: revision, digest, and dirty state
  projection: typed durable facts with the same revision and digest

RenderObservationV2
  document: SessionDocumentObservationV1
  molecule_plans: tuple[DocumentMoleculeRenderPlanV4, ...]

DocumentMoleculeRenderPlanV4
  molecule: MoleculeRenderRootV1
    document_object_id: opaque durable document object key
  plan: RenderPlanV4
  bounds: MoleculeContentBoundsV1
    left, top, right, bottom: finite Rust-measured painted bounds
  member_issues: tuple[MoleculeMemberDepictionIssueV1, ...]

RenderPlanV4
  schema: ferrum-render-plan-v4
  provenance:
    revision: exactly document.snapshot.revision
    digest: exactly document.snapshot.digest
  batches: tuple[RenderBatchV4, ...]
  issues: tuple[RenderIssueV1, ...]
```

Each molecule entry becomes one disposable, noninteractive
`FerrumMoleculeRootItem`. It copies only the Rust-issued document-object identity
and measured content bounds, owns its ordinary child items for lifetime and
disposal, and neither paints nor handles selection. Its z order comes from the
matching backend-issued direct-root order. Each batch has a durable
`RenderTargetV1` keyed by opaque `DocumentObjectIdV1`, plus a declared
coordinate space and exactly one closed atom, compact-group, or bond content
payload. Atom content carries renderer-issued molecule-label runs, structural core
run, exact full/core ink bounds, and positive bond-ink clearance. Ordered typed
operations contain the established line, mask, text, ellipse, and path leaves.
A path is a finite validated stream of
`MoveTo`, `LineTo`, `CubicTo`, and `Close` commands with explicit optional
stroke, fill, and z facts. Scene-space bond batches admit received lines and
paths and frozen `BondAttachmentAxisV1` center-to-center connection facts;
atom-local batches retain masks and text. The axis is transport-only: Qt checks
its exact finite endpoints but never paints or hit-tests it. Paint is explicit lowercase
six-digit `Rgb24`. Text runs declare their
supplied origins, exact glyph identifiers, exact glyph origins, script, scale,
size, face, and paint. Paint is document depiction data, not a Qt palette
selection. The plan is complete or reports explicit target issues; Qt does not
infer missing coordinates, hydrogen policy, line width, color, visibility,
font, glyph mapping, or other depiction defaults.

`FerrumRenderProjection` validates the schema, exact revision, root envelope,
DTO shape, and Rust bounds before allocating scene objects. It copies received path commands
into `QPainterPath` and paints or hit-tests only received geometry and paint; Qt
does not create, complete, recolor, or reinterpret paths, and does not turn a
bond attachment axis into a line. It builds a complete detached
scene containing explicit molecule ownership roots and every supported presentation-vector root
from the same document observation before replacing scene ownership. Molecule
root children keep molecule-local order; top-level molecule and presentation
roots use backend-issued document order. Construction failure disposes the
detached roots and leaves the previous projection visible.

A non-spline presentation polyline carries at least two finite Rust-issued
points, and a polygon carries at least three. A `style="wavy"` polyline is a
distinct Wavy projection root but retains the durable CDML polyline target. Qt
connects its exact authored points in order; it does not regenerate or smooth
the wave. Qt closes only polygons. Rectangle, square, oval, and circle roots
carry finite normalized Rust-issued bounds. Every vector carries an explicit
noncosmetic stroke and an explicit RGB or transparent fill decision where fill
applies. Qt does not reduce paths to their endpoints, infer splines, alter
historical box proportions, apply a toolkit simplifier, or choose palette
fallbacks.
Retirement failures remain explicit diagnostics through the existing
retained-record lifecycle.

A supported normal non-spline arrow carries its complete finite source path,
backend-shortened axis path, validated head dimensions, zero to two ordered
four-point head polygons, and explicit noncosmetic stroke. Qt paints those
paths directly and does not choose arrowhead dimensions or shorten the axis.
Unsupported arrow families and spline interpolation remain preserved document
facts with typed presentation issues; Qt does not substitute normal-arrow art.

A fixed-content plus root carries a finite scene anchor and resolved source
appearance in `SessionDocumentObservationV1`, but that source projection does
not perform font layout. The API resolves the supported family to the verified
vendored Atkinson Hyperlegible Next Regular resource and publishes one centered glyph identifier, its exact
origins, explicit foreground and optional background paint, and finite ink
bounds. Qt calls `QRawFont.pathForGlyph` for that supplied identifier and caches
the resulting outline. It does not shape the `+` string, calculate an advance,
measure the font, consult the system font database, or reposition the anchor.
An authored unsupported family produces a typed issue and no plus graphics
item; there is no silent substitution.

The native Plus property form starts from one exact frozen
`PlusProjectionV1`. It may submit only fields its controls can preserve without
coercion: integer size from 4 through 144 and six-digit foreground color. The
Rust operation is revision-bound and source-ID-targeted; it owns family, size,
foreground, and optional background mutation, detached candidate validation,
history, and the post-operation observation. Qt never edits XML, rounds a
fractional source size, or invents an absent background.

The native Arrow property form starts from one exact frozen normal non-spline
`ArrowProjectionV1`. It may submit start/end head flags, six-digit color, and a
line width that its one-decimal 0.5-through-20 control can preserve without
rounding. The Rust operation remains source-ID-targeted and revision-bound, and
also defines the durable spline field. Qt visibly disables spline editing until
that renderer exists; it neither authors an omitted visual fact nor converts a
spline into a normal arrow. Supported vector graphics items participate in the
combined scene's durable selection map so replacement restores Arrow selection
by backend-issued identity.

Molecule-plan and presentation-vector facts remain separate V1 projections.
The presentation builder consumes exactly `SessionDocumentObservationV1`; it
does not derive presentation facts from paint operations. Each supported
vector owns one top-level graphics root. The standalone presentation
controller replaces the complete root set transactionally: validate and build a
detached candidate, preflight prior callbacks while their roots are scene-owned,
remove the prior roots, attach the candidate roots, then publish. Retirement
never invokes a preflighted callback twice.

If ordinary recovery succeeds, the prior roots and public projection remain
current and the detached candidate roots are retired. If rollback leaves native Qt
ownership ambiguous, the controller invalidates itself, clears its public
projection, retains the prior and candidate projections as explicit recovery
ownership, and rejects later replacement attempts. It never guesses which
scene-owned graphics to dispose. `BaseException` remains outside ordinary
rollback handling.

`FerrumPlanItem` is one selectable, immovable `QGraphicsObject` per render
batch. It owns immutable batch data, cached paths, and bounds. Its `paint()`,
`boundingRect()`, and `shape()` use the same fixed geometry. Qt adds only local
hover and selection decoration. Zoom, pan, DPI, clipping, and antialiasing are
view concerns, not changes to the plan.

`Scene` coordinates map directly to Qt scene units. For `AtomLocal(anchor)`,
Qt paints a run at `anchor + TextOp.origin + TextRun.origin` exactly once. Lines
use the declared scene-unit width and the renderer's one documented pen policy.
Plan paint never resolves a palette role at paint time. Qt-local hover and
selection decoration may use the current UI palette because it is not document
depiction or render-plan content.

### Molecule-label bytes

Ferrum plans use face
`ferrum-atkinson-hyperlegible-next-regular-2.001`. The Python extension supplies
the packaged Atkinson Hyperlegible Next Regular bytes after Rust verifies file
type, byte length, SHA-256, family, and PostScript name. The Qt painter repeats
those checks and loads the bytes directly with `QRawFont.loadFromData`.

Qt uses the loaded `QRawFont` only to resolve the supplied glyph IDs into
outlines at Rust-issued positions and scales. Rust owns measurement, advance,
layout, family selection, and glyph admission. Invalid bytes, font identity, or
glyph resolution raises a render error and preserves the previous projection.

## Selection and interaction

Selection has two projection-local maps rebuilt after every accepted
observation:

- Qt item to immutable `RenderTargetV1`.
- Durable record ID to current live Qt item.

The projection exposes selected targets and selection by durable targets.
Actions capture durable record IDs and observation revision, release Qt
wrappers, submit the intent, then select from the resulting replacement
projection. An id-less projection-local key may be selected for non-mutating
interaction only; it is never a durable operation target, clipboard identifier,
or save identifier. Issues have no selectable graphics item and appear in the
non-modal diagnostic/status model. Qt never creates durable-looking or
provisional correlation identifiers.

The delivered M6 structural-member action bridge is tab-owned. A
`StructureInteractionSelectionV1` remains a Rust-issued opaque selection
fenced by revision and digest; the document tab accepts it only when that fence
matches the installed snapshot and exposes its immutable target tuple to
selection-aware actions. The structural controller owns replace/clear calls and
the projection draws only bounds feedback. Scene selection remains the fallback
for generic root/presentation actions; the two sources are exclusive and never
merge. The tab clears structural action selection before successful
snapshot/projection replacement, when refresh cannot install a matching
projection, and on tool, tab, or disposal lifecycle exit. A refused mutation
preserves it. Qt must not recreate per-atom items, infer target kind from object
IDs, or rebase a stale selection after mutation. Combined focused Qt/PyO3 bridge
coverage passed 57 tests; the registered no-pointer keyboard E2E exited 0; an
independent final review accepted the implementation with no P1 finding. The
YAML `selected_structure` context uses those shared enabled actions, while the
generic focus owner waits for both menu destruction and modal terminal lifecycle
before returning focus to the viewport.

`Edit -> Change Element...` is available only for exactly one current durable
selected atom. Before opening its modal form and again before submission, Qt
rechecks that one-atom selection and its captured observation revision. Zero,
multiple, stale, busy, closed, or tab-switched selection refuses the request
without a Rust submission or document, history, scene, or selection mutation.
Cancel or Escape closes the form with the same preservation guarantee. The
action and dialog use the visible `Change Element` label; keyboard activation
opens the dialog with focus on its bounded `Element symbol:` input. A
screen-reader announcement reports either the successful element change after
the authoritative replacement installs or a typed refusal with recovery
guidance; it does not announce success for cancellation or a refused request.

`Edit Atom Properties...` is the visible property action for exactly one current
durable selected atom. Its modal dialog exposes the atom's `Charge:` control.
Qt captures the durable atom ID and observation revision, then submits the
requested charge only through the revision-bound Rust property mutation; Qt
does not alter a scene item, cache, or serialized document itself. Cancellation,
Escape, invalid charge input, stale/busy/closed/tab-switched state, or a changed
selection preserve the document and make no success announcement. On acceptance,
Qt installs Rust's authoritative replacement observation before presenting the
new charge. The accepted mutation participates in Rust-owned history and save
state, and a normal Save followed by reopen retains the edited charge. The
canonical Qt menu helper is permanent visible-dialog evidence for this route.

The native vector-properties action is available only for one selected durable
rectangle, square, oval, circle, polygon, or ordinary polyline issued by the
current Rust projection. The detached form may submit width and stroke changes;
closed shapes may additionally submit fill or explicit no-fill. The form rejects
source widths it cannot represent instead of rounding them. Rust owns the closed
three-field request, geometry and target validation, semantic no-op decision,
revision check, detached candidate, and history. A Wavy polyline is not an
ordinary vector target. Its dedicated form may submit only width and line color
that its controls preserve without coercion. Rust owns the two-field request,
exact Wavy target and point-structure validation, semantic no-op decision,
revision check, detached candidate, and history. Qt neither interprets XML nor
regenerates the authored Wavy point path.

`Draw Wavy Line` captures one current observation provenance and two finite
scene endpoints. Its straight drag line is disposable interaction feedback,
not persistent geometry. Rust owns the bounded segment/amplitude policy,
generated durable presentation ID, complete authored point path, default
stroke, detached-candidate validation, revision check, and one-use commit. The
created Wavy target becomes selected only after its matching observation
replaces the scene. Invalid, stale, zero-length, or over-bound intent leaves the
authoritative document and next generated identity unchanged.

`Draw Rectangular Bracket` and `Draw Round Bracket` capture one current
observation provenance and the same finite normalized drag box. Their rectangle
is disposable interaction feedback, not persistent geometry. Rust owns the
selected closed style, proportional control points, two generated durable
polyline IDs, effective drawing-standard stroke, explicit pair relationship,
detached candidate, revision check, and one-use commit. Qt selects the two
backend-issued member targets after replacement and never infers pairing from
proximity. A projected round pair uses a distinct closed root kind; Qt sends its
four Rust-issued points through the shared cubic-path builder without parsing
CDML or applying a geometry tolerance.

When both durable sides of one rendered bracket pair are selected, the existing
vector form may submit only a common representable width and line color. Rust
revalidates pair identity, side roles, four-point geometry, retained appearance,
and revision before changing both ordinary polylines in one detached candidate.
Qt does not edit either side independently or persist a separate pair registry.

`Rotate Selected Atoms` captures one or more selected durable atom addresses,
their immutable projected positions, affected bond endpoints, and exact
observation provenance. Qt derives the selection center and paints a dashed
atom-and-bond skeleton above the unchanged authoritative scene while the user
drags. The skeleton is interaction feedback only: immutable render-plan items
do not move, and Qt does not write document coordinates. Escape, stale
provenance, tab change, or teardown retires the skeleton without submission.
Release retires it before one Rust rotation operation; durable atom selection
returns only after the accepted replacement observation installs.

`Move Complete Roots` observes one immutable private Rust
`TopLevelTranslationAnchorV1` receipt at press for the exact complete durable
roots. The receipt canonically retains selectors, source revision/digest, and
the finite lower-left union of authored-coordinate bounds; it is neither CDML
nor document/history/preference state. Qt captures the current `Snap New and
Moved Points to Hex Grid` boolean with it. Enabled moves resolve one delta as
`snap(anchor + raw_delta) - anchor`; disabled moves retain `raw_delta`. The
same finite rigid delta translates the dashed projection-only preview and the
revision/digest-fenced Rust commit. Projected bounds are overlay-only, not
anchor authority. Escape, stale provenance, tab change, or teardown retires
the group without submission. A typed stale category covers a changed selection
set or revision/digest race and reports `Native Move Complete Roots Stale` with
the recovery instruction to select complete roots and drag again. Validation
and nonfinite-input failures remain Error or Unavailable. An accepted replacement
restores durable selection and established undo/save/reopen behavior. Rotation
remains angular input and exact existing-atom joins create no moved coordinate.

`Add Atom at Point` is a one-click mode. The frontend captures one current
durable molecule selector, an explicit element spelling, and the observation
provenance before it waits for the click. A tab change, Escape, pending
authority, or provenance change cancels or rejects the intent without
submission. The generated atom becomes selected only after the matching Rust
observation has replaced the scene. If Rust accepted the mutation but the
render projection could not be installed, the tab enters pending-authority
state: Save, Save As, Change Element, Add Atom, Undo, Redo, and Close are
disabled, Refresh Authoritative View is the recovery action, and the tab cannot
close even if an accepted undo made the Rust document clean.

`Draw Bond` captures one immutable Next Drawing normal-order snapshot at mouse
press. The shared QSettings client offers only Single, Double, and Triple;
Rust maps those values to `n1`, `n2`, and `n3` for all four direct-bond endpoint
forms. New endpoints are carbon in this bounded P0.1 route. Rust retains target
validation, duplicate detection, candidate atomicity, provenance, history,
selection, and accepted render facts; Qt owns only disposable feedback and
preview objects. Personal preferences never enter CDML, `<standard>`, document
state, history, dirty/save state, or selection.

Directed direct-bond authoring uses the sole current unversioned in-process
Rust/PyO3 pointer capability: begin a direct-bond gesture, resolve its endpoint,
prepare the generic `PreparedSessionTransitionV1`, then generically commit it.
Qt owns pointer events, finite viewport-to-scene conversion, and exact
`none`/unique/ambiguous scene-hit evidence; it neither scans projected atoms
nor applies endpoint geometry. `ferrum-document-render` resolves the UI pointer
probe and one-use authoring capability. `ferrum-document` owns the durable
`CreateDirectBondV1` request and generic transition, including endpoint forms,
fencing, candidate construction, renderer admission, and the immutable
operations Qt paints. A probe error and a post-resolution document refusal are
separate typed native outcomes with closed nonmodal recovery. A same-atom
directed gesture is `self_loop` with `adjust_endpoint`, not malformed pointer
input. V1 applies only to durable document, fence, presentation, snap, and
transition values where it is the actual contract version. The separate public
`ferrum-document` neutral seam is native-Rust-only, noninteractive programmatic
mutation with already-resolved durable atom IDs or finite new-atom points; it
accepts no UI facts and has no Qt/PyO3 route.

Selected-bond Properties is a separate editor for one already-projected durable
bond. It retains its independently supported broader Rust-owned style
vocabulary; the bounded Normal/Solid-wedge/Hashed-wedge vocabulary describes
the M3.P6 authoring actions, not this editor. Qt submits changes through the
existing revision-bound Rust property patch and replaces from the accepted
projection without repairing historical styles or inferring endpoint direction.

`Edit -> Next Drawing...` is the standard MainWindow-owned labelled route to a
compact client of that same shared application model, including at narrow
window widths. Editing Tools projects the action at low priority; it adds no
second preference or document owner. Escape in the focused client restores its
accepted input and, for an active Draw Bond gesture, composes the shared Cancel
Tool action. Armed and press-frozen feedback names the selected normal order;
an empty-space endpoint remains carbon in this bounded route.

`Insert Cyclohexane Ring` is one shared Edit and Editing Tools QAction for a
closed native detached-ring outcome. At an empty finite page location, its
press captures the active tab/revision/digest and resolves the centre once with
`snap_authored_scene_point`. Rust's private `DetachedRegularRingInsertionV1`
request supplies the canonical 40-point-side-length C6 flat-top vertices in
y-down coordinates; Qt copies only those exact vertices into a disposable
preview. On release, the tab commits the one-use Rust prepared receipt and
installs its authoritative projection and selected created atoms. An atom hit
reports empty-page guidance. Escape, Cancel Tool, tab changes, teardown, and
stale provenance retire the preview/tool and preserve the current document and
selection. The Rust family can validate detached sizes 3 through 8, while the
UI exposes only cyclohexane. It writes ordinary C atoms, points, and `n1`
bonds, not ring metadata, a template name, an orientation preference, or any
other UI state in CDML. Attachment/fusion, UI size selection, heteroatoms,
aromaticity, orientation/rotation, and preferences require their own contracts.

`Edit -> Insert Haworth Ring...` is the adjacent shared Edit and Editing Tools
action for exactly four named detached D-glucose recipes: alpha/beta
D-glucopyranose and alpha/beta D-glucofuranose. Its parented modal chooser collects
only ring form and anomer and displays the resulting concrete name. Rust owns the
literal C6O6 recipe, finite local geometry, durable IDs, CDML authoring, history,
selection, candidate validation, revision/digest-bound one-use receipt, and normal
Render Plan V2 output. Qt captures the active tab and the shared snap decision once,
copies only the Rust preview receipt, and replaces its disposable preview with the
authoritative observation after commit. Pyranose closes `O5-C1-C2-C3-C4-C5`; furanose
closes `O4-C1-C2-C3-C4` and continues as `C4-C5-C6`. The front C2-C3 edge is `q1` with
a round-cap V2 front-stroke layer; directed shoulders C1->C2 and C4->C3 are `w1` front
wedges; remaining ring edges are `n1` back edges and exocyclic edges are ordinary
`n1`. These presentation facts are chemical single bonds, not Qt geometry. An authoritative
atom or bond at either the raw click point or its shared snapped point refuses placement,
preserves document and selection, and retains the same armed intent for a later empty-page
click. Escape, Cancel Tool, stale provenance, tab change, and teardown retire the preview and
preserve durable state. The chooser has no
QSettings state and no UI choice enters CDML, history, selection, or document data.
Generic codes/catalogs, other sugars, attachment/fusion, rotation/reflow, and general
stereochemical inference require new contracts.

The accepted current-source/staged-local-runtime walkthrough exercises all four chooser variants,
one shared snap anchor, receipt-derived preview, and one authoritative commit. It verifies
occupied-page, Cancel, Escape, and stale-intent preservation; public tab Undo/Redo restores
semantic CDML and history even though revisions advance; and Save/reopen retains the inserted
molecules. It confirms that CDML retains only durable Haworth presentation facts and that neither
the chooser nor QSettings stores Haworth UI metadata. This walkthrough is disposable integration
evidence, while compact semantic Rust, binding, renderer, and visible product tests remain the
permanent coverage.

`Chemistry -> Insert Direct-Glycosidic Haworth...` is an accepted native V1 client. Its
initially empty, accessible `Structural SMILES` dialog
explains that the tool draws only a limited two-ring C/O profile and does not identify a sugar,
infer stereochemistry, or name a glycosidic linkage. Rust alone parses and accepts exactly two
vertex-disjoint five- or six-member C/O rings plus one exterior degree-two oxygen bridge: 11--13
atoms, neutral nonaromatic single bonds, and no other atoms, bonds, charges, stereo, or source
facts. This is a structural drawing profile, not sucrose, anomer, D/L, linkage, or general-SMILES
recognition. No sample or preset is shipped. After typed Rust preparation, Qt captures the source
tab, revision, digest, and one shared snapped empty-page anchor; it paints only frozen
receipt-derived V2 preview batches, then installs the ordinary Rust observation after one commit.
Raw or snapped durable atom/bond occupancy preserves the document and leaves a current intent
available for an empty location. Cancel, Escape, stale/busy/closed/tab-change state, and failed
delivery retire preview/intent without mutation or redirection. Rust owns graph admission, IDs,
CDML, history, selection, persistence, and normal V2 q1/w1/n1 lowering; private PyO3 owns only
the receipt seam. SMILES, names, parser coordinates, ring choice, preview state, and preferences
remain outside CDML, QSettings, public `.pyi`, CLI, wire, and composite rendering. Compact
semantic Rust/binding/Qt behaviors are permanent evidence. A sealed local runtime passed the
focused private/public suite (4 passed); the independent public walkthrough accepted blank and
invalid inline accessible recovery, pointer-tool cancellation, valid occupied-location retry,
selection, Escape/tab-switch/close containment, Undo/Redo, save/reopen, and normal V2
receipt-only installation without a direct-glycosidic marker. Local runtime mechanics,
offscreen-focus behavior, screenshots, parser, visual, accessibility, and occupancy probes remain
disposable.

`Chemistry -> Check Bond Capacity...` is an accepted ordinary-native, read-only FQ-010
diagnostic for selected complete direct roots. It accepts neutral, nonaromatic ordinary
`H`, `B`, `C`, `N`, `O`, `F`, `Cl`, `Br`, and `I` graphs with absent or zero formal charge,
single/double/triple connectivity, and no authored `valency`, `multiplicity`, or
`free_sites` fact. Rust retains whether formal charge and explicit hydrogen values were
authored, evaluates explicit-H plus incident bond-order demand, and reports each assessed
atom as Within Capacity or Exceeds Capacity. A root outside that grammar receives one
Not checked result rather than a partial atom verdict; bond depiction is ignored. This is
not a chemical-validity, valence, or oxidation-state claim. Rust owns the finite table,
grammar, authenticated receipt, and provenance; the private PyO3 seam transfers it and Qt
owns only the fenced worker and selectable report dialog. The route adds no Properties,
QSettings, CDML, history, selection, public Python, CLI, or wire contract. Compact
Rust/API/private-binding/public-action behavior tests are permanent, including a mixed-root public
regression. The accepted public real-worker walkthrough exercises supported/no-excess/finding/Not checked
reports, every assessed atom's authored charge/H supporting facts in a mixed excess root, direct-root
order, depiction independence, lifecycle/nonmutation, and accessibility. Fresh local-runtime,
visual, and timing observations remain disposable evidence.

`Chemistry -> Atom Oxidation State...` is a read-only Rust protocol client for exactly one
selected durable atom in one direct-root molecule. It is enabled only for a current, eligible
Ferrum tab with one resolved atom selection and no conflicting document operation. Rust alone
evaluates the complete materialized H/C/N/O root under
`formal-electron-assignment-hcno-v1`; Qt sends one fenced public
`document.atom.oxidation.observe.v1` request through `execute_operation_v1` and presents the
typed response without deriving chemistry or adding a mark. The modeless accessible `Atom
Oxidation State` dialog displays the atom and source revision, selectable read-only details, a
visible source-status label, `Run Again`, and `Close`. An accepted observation displays its signed
number and convention; an unavailable observation displays its closed reason and recovery. A
typed refusal is displayed as a failed observation rather than as chemical unavailability.

The dialog is source-bound historical context. If its captured document changes, requires refresh,
or loses its revision/digest fence, the details remain selectable and the source status states that
the result is from an earlier revision. `Run Again` recaptures exactly the original active source
tab's current single-atom selection and fresh fence; it is disabled while that source is inactive,
stale, unavailable, or lacks one eligible atom. Closing the source tab retires the dialog. The
interaction never redirects a result or rerun to another tab and never changes CDML, history,
selection, or renderer state.

`Chemistry -> Molecule Report...` is an ordinary-native, read-only receipt viewer with one
source-bound modeless lifecycle. The dialog may remain visible as historical context when the
author changes tabs, but `Run again` is enabled only while its captured source tab is still the
active live tab and has a complete selected molecule. Rerun recaptures that source tab's current

Each displayed record presents the Rust-issued nested composition DTO exactly when available:
formula, net formal charge, finite average molecular weight and monoisotopic mass in Da, and
canonical isotope-aware element contributions with their counts and mass percentages. The dialog
also displays the required tagged report aggregate exactly as either a complete composition or its
closed Rust omission reason; it does not derive formula, mass, percentages, or aggregate state.
selection; it cannot resolve the current selection from another tab. Before its source tab is
disposed, Qt terminally retires the dialog and its rerun action. Rust remains the sole source of
report facts; Qt neither reveals an inferred atom nor mutates a document while presenting or
retiring the receipt.

For each molecule-report record, Qt treats `stereo_semantics` and
`stereo_depiction` as distinct Rust receipts. The former alone carries chemical
tetrahedral and E/Z configuration. The latter carries Rust-issued editable
directed-bond and E/Z carrier-mark facts. Qt renders the issued marks but never
derives configuration from marks or coordinates, and never invents marks from
configuration.

`Chemistry -> Create Fragment...` and `View Fragments...` are accepted ordinary-native
Explicit Fragment V1 clients. Create captures one live source tab, revision, digest, direct-root
molecule, and durable atom/bond selection before the modal name form. It accepts one nonblank
plain name after trimming outer whitespace, retains duplicate labels, and asks Rust to create only
one explicit durable annotation in that molecule. Rust owns direct-child eligibility, selected-bond
endpoint closure, canonical molecule-source order, collision-safe identity, one-use revision-bound
commit, CDML, history, undo/redo, save/reopen, and the scalar observation. Disconnected selected
members are valid metadata; creating the label changes neither molecule chemistry nor selection.
Qt owns only the dialog, frozen capture, focus/recovery, status, and receipt-based projection
installation. It keeps typed text for safe retry, and Cancel, Escape, stale/revision/tab/close/busy
state, or refusal preserves document and selection and directs the author to select again. View is
read-only: it lists only exact supported explicit records and may show one safe notice that retained
imported metadata cannot be edited here. Delete, rename, highlight, type selection, clipboard,
groups/templates, inference, cross-molecule records, QSettings, public `.pyi`, CLI, and wire
surfaces are outside V1. The private PyO3 seam is runtime plumbing, not a stable Python promise.
The independent rereview accepted the View lifecycle repair (source tab, revision, digest, busy,
refresh, and close retire a modal View without redirecting focus) and the stable typed-error repair
(expected Rust errors retain the Create name/focus without exposing internal reasons). Installed
public evidence accepted endpoint closure/source order, duplicate labels, blank retry/Cancel/stale
containment, retained notice, View lifecycle, undo/redo, and save/reopen. Compact semantic
Rust/binding/Qt tests are permanent evidence; local-runtime, screenshots, keyboard/accessibility,
visual, corpus, and timing observations remain disposable.

## Bond presentation editing

One selected durable Rust bond exposes exactly one closed
`DocumentBondPresentationV1` through `BondProjectionV1.presentation`.  Python
does not combine an order field with a style field, reinterpret a CDML token, or
keep a shadow bond model.  The dialog performs a display-only mapping from that
one value to its controls and returns one replacement presentation intent.

The only authorable choices are normal single, double, and triple (`n1`, `n2`,
`n3`), solid wedge (`w1`), hashed wedge (`h1`), Haworth front edge (`q1`), bold
(`b1`), dashed (`d1`), and wavy (`s1`).  Non-normal presentations are fixed to
single order.  Changing a presentation produces one
`DocumentBondPropertyChangeV1.presentation(...)`; it is never split into order
and style changes.  The Qt adapter explicitly clears optional scalar facts that
the replacement cannot retain: centering is only meaningful for a normal double
bond, bond width for normal double or triple bonds, and wedge width for solid or
hashed wedges.  Rust validates the whole patch atomically and returns the fresh
authoritative observation, so a rejection leaves the document and the installed
projection unchanged.

This is an intentionally closed editing grammar, not a claim that every
foreign source fact is authorable.  Rust may preserve or refuse unsupported
source observations with a typed diagnostic; Qt neither normalizes those facts
nor supplies a Python parser or renderer fallback.  The edited tab rebuilds
only from the returned Rust render observation.  Its source presentation,
geometry, and diagnostics therefore have one owner.

## Workers and UI thread

The first render-plan slice is Qt-thread affine: `ferrum_chem` calls and all
QGraphics construction occur on the Qt main thread. Background render work is
introduced only with an owned-value handoff and a documented main-thread
projection delivery contract.

Native artifact export follows the same lifetime rule without moving a
`DocumentSession` off the UI thread. Its worker receives one owned immutable
observation and creates an owned Rust artifact receipt. Qt fences the tab before
starting that work and again before publication. Export retains the captured tab as
busy; ordinary Open and Open in Current Tab also refuse to start while export is
live, while export refuses when a local Open is active or queued. Close cancels
future delivery and waits for retirement; it does not claim to interrupt Rust work
or publication already begun.

Selected read-only workers capture selection only at admission. Diagnostics and
selected SMILES capture the selected molecule IDs with immutable revision/digest
facts before their worker starts. Delivery authenticates worker/cancel state, a
live active ready tab, exact captured revision/digest, and the receipt molecule
ID/schema; it does not require that the user still has that molecule selected.

A later selection change or clear preserves a valid historical diagnostics
result. Its dialog becomes stale and disables rerun until the original molecule
selection is recaptured. Selected SMILES treats a file dialog as pre-admission
and rechecks selection before starting; post-worker delivery uses only the
captured fence and receipt. During either read-only worker, Select Structure
remains available while selection mutations, including Delete, remain disabled
until the worker finishes.

All singular molecule export clients use the same Rust selected-root export
core: Molfile V2000/V3000, SDF V2000/V3000, canonical SMILES, Standard InChI,
and Fixed-Hydrogen InChI. Qt supplies only the selected direct-root ID and the
captured observation fence, presents a typed refusal, and may ask the Rust
publisher to create a new destination after computation completes. It never
lowers a graph, decides format support, rewrites output, or publishes a partial
result. Coordinate-required formats refuse without valid coordinates; the
shared compact-group lowerer decides representation support. The plural
multi-record SDF export remains a different workflow and must not be routed
through a singular selection.

## Command discovery and reference

`ActionRegistry` remains the live owner of registered action identity, label,
help, enabled state, and invocation. Its views join validated `menus.yaml`
placement once into an immutable `CommandCatalogEntry` catalog. Command Palette
and Command Reference consume that catalog rather than maintaining parallel
command lists.

Command Reference is modeless and nonmutating. F1, using Qt's native Help
standard key, and **Help > Command Reference...** open it with the search field
focused. It finds label, help, stable ID, current native shortcut, and YAML
breadcrumb, retains unavailable commands with an explanation, and provides no
activation gesture. Close or Escape restores focus to the invoking control.
The filter, list, status, and close control have explicit accessible names and
descriptions with a defined tab order. Focused Qt tests prove this behavior;
native keyboard dispatch and assistive-technology review remain human desktop
acceptance work, not a claim made by this contract.

## Authoring ribbon presentation

The authoring ribbon is one projection of the existing `ActionRegistry`; it is
not a command owner. Failure-atomic window preflight resolves menus, the
complete ribbon layout, and the exact command-icon catalog before any visible
client is constructed. Each quick-access, tab-group, overflow, menu, and
Command Palette client therefore shares the same live `QAction` identity,
label, help, enabled/checked state, shortcut, icon, and invocation.

`ribbon_layout.yaml` owns persistent quick access, global discovery actions,
task tabs, group and entry order, primary/supporting role, reduction priority,
and semantic accent. The closed theme `ribbon` mapping owns every ribbon color.
`actions/command_icons.py` is the exact theme-aware visual binding for every
ribbon command; chemistry-authoring commands use packaged BKChem artwork while
platform file and view commands use Qt standard icons. The widget tree may
only project these resolved values.

`ribbon_contract.py` owns one component geometry instead of allowing action
text to create arbitrary rectangles. Every primary, singleton supporting, and
popup tile occupies a 72-pixel frame. Supporting pairs form 34-pixel rows with
one 4-pixel internal gap; adjacent components and groups use the declared
8-pixel rhythm. Live text hints snap to bounded 32-pixel width increments.
The closed theme owns one default fill, foreground, border, radius, disabled
state, hover state, checked state, and focus state for every command tile.

The fixed header contains brand, icon-only quick access, a keyboard-reachable
`QTabBar`, and the labelled Command Palette route. A `QStackedWidget` exposes
the selected task page. Each page measures its real width and reduces groups
from expanded to compact to collapsed in reverse declared order. Supporting
commands move to a labelled **More** menu, then all group commands move to one
consistently shaped **More** popup under the still-visible group caption;
neither transition duplicates or loses an action, its full accessible name and
tooltip remain specific to the group, and focus follows a control that remains
exposed.

Every direct and popup control has a visible label or tooltip, accessible name
and description, and strong keyboard focus. Text pairs meet 4.5:1 contrast;
focus, checked-state boundaries, and semantic category rails meet 3:1 in both
shipped themes. Native focus traversal and assistive-technology output remain
human desktop acceptance work.

## Native Open lifecycle

`LocalDocumentOpenCatalogV2` is the sole Rust discovery authority for local
File/Open. It issues opaque handles for native CDML, decoded CD-SVG, and each
`DocumentImportNew` interchange descriptor. Qt retains one issued descriptor
and calls one generic preparation API; it does not inspect a source kind,
reselect an interchange parser from a suffix, or synthesize a fallback
document/render result. Rust admission prepares exactly one revision-bound,
issue-free render observation before a new tab or replacement can be published.
File/Open's descriptor fact chooses current-tab replacement eligibility.
`File > Import SDF Records into Current Drawing...` intentionally remains a
distinct source-read/current-document insertion workflow using the
catalog-issued SDF handle.

The local-CDML, decoded-CD-SVG, CML, and CDXML profiles retain the admitted regular descriptor
long enough for Rust to mint an opaque equality-only origin token. The private one-use PyO3 receipt
transfers the authenticated session, render observation, token, and closed source kind together.
The token is live-tab lifecycle state only; it is not CDML, serialized document/session state,
history, a preference, or a cross-process identity. Decoded CD-SVG retains canonical embedded CDML
only; CML and CDXML retain imported molecule semantics only; source wrappers and interchange text
never enter the session or Qt projection.

Qt owns immutable Open intents and their dispositions. Interactive `File > Open...` chooses
`ReplacePristineTarget` only for the explicitly marked, clean revision-zero bootstrap `Untitled`
tab with no origin, content, selection, pending projection, worker, or canvas interaction. Rust
admission and receipt authentication happen before Qt revalidates that target, its revision/digest,
and canvas-idle fence, fully constructs the replacement from that one issue-free
Rust observation, and atomically installs it at the target index. A stale or
ineligible target instead receives the ordinary `NewTab` result. A busy target
stays current while that new tab installs in the background, so its visible preview survives until
the user resolves it. Cancellation, shutdown, invalid admission receipts, and pre-commit
construction failure preserve all existing tabs. After host resolution commits, later receipt
validation or presentation faults retain the committed tab for recovery; they never roll it back
or dispose it. A matching origin token activates the existing tab, including for a hard-link alias,
and disposes the newly admitted receipt.

For decoded CD-SVG, Qt keeps source display provenance separate from the tab's
`file_path`. The source path and closed receipt kind provide truthful tooltip and
accessible wording, but never enter CDML, history, settings, selection, or Rust session
state. The tab begins clean without a publication baseline, so Save uses the ordinary
CDML Save As flow. Once that publication succeeds, later Save uses its `.cdml`
destination while the original token remains valid for source duplicate activation.
Qt selects this route only from the requested `.svg` suffix; it does not parse, sniff,
decompress, render, or preserve a wrapper. Stable rejected-source categories explain
whether to choose a regular decoded SVG, provide one canonical payload, reduce resource
use, or choose supported embedded CDML; installation failure states that the current tab
is unchanged.

Launch and queued paths always use `NewTab`; they never consume a bootstrap page. `File -> Recent
Files` is the versioned
`FerrumNativeRecentFilesV1` QSettings-only personal client. It stores lexical
normalized absolute display paths without symlink resolution, deduplicates those display keys, and
uses descriptor-token equality only for the stronger live-tab duplicate rule. Confirmed native Open,
token activation, and Save promote an entry; failed, cancelled, and unconfirmed work does not.
Recent selection always submits a forced `NewTab` intent through the ordinary coordinator. File
rebuilds its Recent Files cascade after updates and when shown; duplicate basenames add parent context
while the full path remains the tooltip, status, and accessible description. Its default usable menu
capacity is tunable local presentation policy, not a document contract or exact-count test. A
Rust-confirmed unavailable/nonregular source presents `File Not Available` with `Keep` by default or
explicit `Remove from Recent Files` before the generic typed failure; cancellation and other failures
retain the entry. `Clear Recent Files` clears settings only.
`File -> Open in Current Tab...` is the separate, explicit ordinary-native replacement command. Its
accessible text is `Open in Current Tab...` and its standard shortcut is `Ctrl+Shift+O`. It is enabled only
for the selected registered ordinary-native tab with a complete current
projection, no close/replacement, target-owned asynchronous work, or active
canvas interaction; while a tool or target-owned operation is active its
guidance is to finish, cancel, or let that work complete. It
captures an immutable target fence, prepares and authenticates the source first, then revalidates the
registered current target before deciding anything destructive. A matching admitted descriptor token activates the
already-open native tab and preserves the requested target. A clean saved populated target is constructed
and swapped atomically at its captured index without a redundant confirmation. A dirty target alone offers
Save (default), Replace, and Cancel: named Save uses native Save, unnamed Save uses native Save As, and a
successful publication must establish and pass a fresh post-save fence before the swap. Stale, busy,
close, cancel, admission, pre-commit construction, issue-bearing render receipt,
or save failures discard the pending receipt, preserve the target's selection,
tool, preview, and focus, and report an explicit retry without document
mutation; this command never becomes NewTab. A receipt-validation or presentation fault after irreversible replacement
retains the committed new tab and reports recovery instead of restoring the disposed target.
At a successful swap the old owner retires only after the complete incoming tab registers, and the new tab
starts with its own clean selection and focus. A queued worker `finished` signal is deferred while this
modal recovery decision is active, so it cannot retire the explicit intent inside the nested event loop.

Recent state never enters CDML, standard metadata, Rust sessions/history, dirty/save state, selection,
receipts, or diagnostics. Recent Files remains confirmed-only composition through forced `NewTab`;
ordinary Open and Recent semantics are unchanged by explicit current-tab replacement.

## Visible state model

| State | Session/backend fact | Qt presentation and permitted action |
| --- | --- | --- |
| Loading | A current open or projection request is pending. | Keep the prior complete projection visible when present; show loading status and disable only actions that require the pending result. |
| Empty | The accepted Rust document has no renderable targets. | Show the empty canvas and normal creation affordances; do not invent a document object or render default. |
| Valid | An accepted observation has complete supported batches. | Show the rebuilt projection, durable selection, current title, and dirty state from Rust. |
| Invalid | Rust reports rejected document content, projection failure, or render issues. | Retain the last valid projection when replacement fails; show the typed diagnostic and selectable targets only where a batch exists. |
| Busy | A current worker or submission is active. | Show progress/status; stale delivery is discarded by request token and session identity. |
| Success | Rust accepts an operation, load, save, or recovery publication. | Replace from the returned observation and report the typed outcome; only confirmed publication updates saved UI state. |
| Failure | Rust rejects an operation or publication/projection fails. | Preserve the current accepted observation and projection; report the typed error with a recovery action when one exists. |

## Native product boundary

The public command and `MainWindow` own the complete Rust-native document lifecycle:
Open, Open in Current Tab, save/reopen, cancellation and stale fences, recovery export,
ordinary artifact export, and shutdown. There is no second editor, alternate session
model, compatibility tab, or action-policy switch.

Before a tab closes or the application shuts down, `MainWindow` retires every active
canvas pointer owner for that tab, including `Select Structure`, before the tab disposes
its viewport. The focused Qt behavior test
`test_close_selected_structure_tab_retires_the_active_pointer_tool` is the permanent
evidence for this ownership invariant; staged-local-runtime walkthroughs remain
disposable integration evidence.

The product accepts local CDML, the bounded decoded-CD-SVG profile, closed Rust-owned CML/CML2,
and bounded input-only CDXML simple-molecule import. CML and CDXML always create clean new tabs
and never replace the current tab; neither has a source Save baseline or export route. The first
Save or Save As publishes CDML. The product explicitly refuses CDX, unsupported CDXML chemistry
or presentation, namespaces, compressed copies, `.svgz`, `.cdsvg`, and unsupported or incomplete
documents without mutation. The recovery copy writes CDML only; it does not offer format conversion.
Broader historical editing modes, template catalogs, import/export families, and clipboard or
presentation workflows are preproduction drops unless a later slice gives them a complete Rust
owner and an explicit user contract. See [FILE_FORMATS.md](FILE_FORMATS.md) for the concise
current disposition.

Historical note: the former mixed-host migration bridge and its retained-session shutdown
path were removed with the second host. They are not current product behavior or test
requirements.

## Verification evidence

Permanent tests stay small and behavior-focused: a supported native operation, typed
refusal with preserved document state, and Rust/PyO3 contracts that can regress independently.
They do not assert private worker wiring, module inventories, timing, fixed action counts,
or visual bytes. Source/dependency inventories, fresh staged-runtime checks,
manual walkthroughs, screenshots, accessibility checks, and race observations are useful
one-time implementation evidence, not permanent fast-suite gates.

Before accepting a slice that claims those capabilities, prove its actual user
path: open or create a document, obtain the required revision-bound Ferrum
observation, paint its plan with verified molecule-label bytes, select durable IDs,
submit its supported operation, replace the projection, and save/reopen through
Rust without losing CDML order, IDs, or opaque content. A bounded slice proves
only the operations and document classes named by its receipt.

Run focused Rust, PyO3, and offscreen Qt behavior tests for the changed slice.
Use a managed end-to-end receipt for staged-local-runtime and visual evidence. Keep
the fast suite free of timing, private-wiring, fixture-heavy, and exact-count
checks. Follow the repository source-file guidance rather than treating a line
count as a functional acceptance gate.

The focused tests establish this boundary: the dialog maps the one Rust
presentation value, emits the one presentation patch with required clears, and
the generic Open path accepts styled CDXML only through a clean Rust render
receipt.  The real Qt window end-to-end check additionally covers Wavy, Bold,
and Dashed import plus Current Tab refusal without mutation.  It is stronger
than an offscreen unit test, but it is still local automated evidence.  Fresh
human review of the captured window, accessibility review, CI execution on its
supported runners, and release packaging are separate acceptance gates; none is
implied by a green local test run.
