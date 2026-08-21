# Ferrum contract

Ferrum is a Qt Widgets document editor. This contract defines its durable
Rust-native ownership boundary. It follows the active
[ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md) and applies the repository
principles "Fix the design, not the symptom" and "Long-term over short-term."

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
with verified render metrics into `RenderObservationV1`, the separate
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

The Ferrum molecule painter consumes a final `RenderObservationV1`, composed by
the API from one revision-checked `SessionDocumentObservationV1` and verified
render metrics. It does not consume retired atom or bond models or Python XML.
The final observation contains frozen
`DocumentMoleculeRenderPlanV2` values in document root order. Each entry keeps
document-root molecule identity and order separate from the molecule-local atom
and bond order inside its `RenderPlanV2`. `RenderObservationV1` retains its name
because it is the revision-bound document/projection receipt envelope, not a
molecule-plan grammar; its payload contains only V2 molecule plans.

```text
SessionDocumentObservationV1
  snapshot: revision, digest, and dirty state
  projection: typed durable facts with the same revision and digest

RenderObservationV1
  document: SessionDocumentObservationV1
  molecule_plans: tuple[DocumentMoleculeRenderPlanV2, ...]

DocumentMoleculeRenderPlanV2
  molecule: MoleculeRenderRootV1
    id: durable document object key or null
    projection_key: projection-local identity
    source_id: authored CDML ID or null
    source_order: direct document-root position
  plan: RenderPlanV2

RenderPlanV2
  schema: ferrum-render-plan-v2
  provenance:
    revision: exactly document.snapshot.revision
    digest: exactly document.snapshot.digest
  batches: tuple[RenderBatchV2, ...]
  issues: tuple[RenderIssueV1, ...]
```

Each molecule entry becomes one disposable root graphics group whose z order is
the backend-issued molecule `source_order`. Each batch has a durable
`RenderTargetV1` with `record_id.kind`,
`record_id.id`, and `source_order`, plus a declared coordinate space. Ordered
tagged `RenderOperationV2` values contain the established line, mask, text, and
ellipse leaves or a neutral `PathOpV2`. A path is a finite validated stream of
`MoveTo`, `LineTo`, `CubicTo`, and `Close` commands with explicit optional
stroke, fill, and z facts. Scene-space bond batches admit received lines and
paths; atom-local batches retain masks and text. Paint is explicit lowercase
six-digit `Rgb24`. Text runs declare their
supplied origins, exact glyph identifiers, exact glyph origins, script, scale,
size, face, and paint. Paint is document depiction data, not a Qt palette
selection. The plan is complete or reports explicit target issues; Qt does not
infer missing coordinates, hydrogen policy, line width, color, visibility,
font, glyph mapping, or other depiction defaults.

`FerrumRenderProjection` validates the schema, exact revision, root envelope,
and DTO shape before allocating scene objects. It copies received path commands
into `QPainterPath` and paints or hit-tests only received geometry and paint; Qt
does not create, complete, recolor, or reinterpret paths. It builds a complete detached
scene containing molecule groups and every supported presentation-vector root
from the same document observation before replacing scene ownership. Molecule
group children keep molecule-local order; top-level molecule and presentation
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
vendored Telex resource and publishes one centered glyph identifier, its exact
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

### Telex bytes

Ferrum plans use face `ferrum-telex-regular-v1`. The Python extension supplies
the packaged Telex Regular bytes that Rust verified by file type, byte length,
SHA-256, family, and PostScript name. The Qt painter verifies those bytes and
loads them with `QRawFont.loadFromData`.

Qt uses the loaded `QRawFont` only for glyph outlines. Rust has already chosen
all run positions and scales, so Qt must not measure, advance, relayout, select
a system family, or substitute a missing glyph. `QFont("Telex")`,
`QFontDatabase`, `QFontMetrics`, and a font path reopen are outside this
contract. Invalid bytes, font identity, or glyph resolution raises a render
error and preserves the previous projection.

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

`Draw Bond` captures one immutable Next Drawing snapshot at mouse press:
element, normal order, and `DocumentBondPresentationV1`. The personal
`Next presentation:` QSettings client offers Normal, Solid wedge from start
atom, and Hashed wedge from start atom; directed choices use Single. It is
shared by ordinary live windows, and changing it affects only the next press.
Rust maps Normal to `n1`/`n2`/`n3`, SolidWedge to `w1`, and HashedWedge to
`h1`. A directed gesture serializes its press/start atom as the narrow tip and
its release or new atom as the wide base. Rust retains target validation,
duplicate detection, candidate atomicity, provenance, history, selection, and
the accepted render facts. The source-owned `native_directed_bond_preview_v1`
receipt supplies V2 path or line operations; Qt copies those facts into a
disposable preview and does not construct its own wedge geometry. QSettings
preferences never enter CDML, `<standard>`, document state, history,
dirty/save state, or selection. `w2`, `w3`, `h2`, `h3`, and all other styles
remain outside this closed authoring contract.

Selected-bond Properties is a separate editor for one already-projected durable
bond. Its native form offers only Normal, Solid wedge, and Hashed wedge;
directed styles require Single and are the only styles with a wedge-width edit.
Qt submits changes through the existing revision-bound Rust property patch and
replaces from the accepted projection without repairing historical styles or
inferring endpoint direction.

`Edit -> Next Drawing...` is the standard MainWindow-owned labelled route to a
compact client of that same shared application model, including at narrow
window widths. Editing Tools projects the action at low priority; it adds no
second preference or document owner. Escape in the focused client restores its
accepted input and, for an active Draw Bond gesture, composes the shared Cancel
Tool action. Directed armed and press-frozen feedback names Solid or Hashed
wedge with Single, the narrow-tip-to-wide-base direction, and the frozen
element for an empty-space endpoint.

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

The accepted current-source/installed-site walkthrough exercises all four chooser variants,
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
semantic Rust/binding/Qt behaviors are permanent evidence. A sealed installed site passed the
focused private/public suite (4 passed); the independent public walkthrough accepted blank and
invalid inline accessible recovery, pointer-tool cancellation, valid occupied-location retry,
selection, Escape/tab-switch/close containment, Undo/Redo, save/reopen, and normal V2
receipt-only installation without a direct-glycosidic marker. Wheel/site mechanics,
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
order, depiction independence, lifecycle/nonmutation, and accessibility. Fresh wheel/site,
visual, and timing observations remain disposable evidence.

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
Rust/binding/Qt tests are permanent evidence; wheel/site, screenshots, keyboard/accessibility,
visual, corpus, and timing observations remain disposable.

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

## Native Open lifecycle

The local-CDML and decoded-CD-SVG profiles retain the admitted regular descriptor long enough for
Rust to mint an opaque equality-only origin token. The private one-use PyO3 receipt transfers the
authenticated session, render observation, token, and closed source kind together. The token is
live-tab lifecycle state only; it is not CDML, serialized document/session state, history, a
preference, or a cross-process identity. A decoded-CD-SVG receipt contains canonical embedded CDML
only; the SVG wrapper never enters the session or Qt projection.

Qt owns immutable Open intents and their dispositions. Interactive `File > Open...` chooses
`ReplacePristineTarget` only for the explicitly marked, clean revision-zero bootstrap `Untitled`
tab with no origin, content, selection, pending projection, worker, or canvas interaction. Rust
admission and receipt authentication happen before Qt revalidates that target, its revision/digest,
and canvas-idle fence, fully constructs the replacement, and atomically installs it at the target
index. A stale or ineligible target instead receives the ordinary `NewTab` result. A busy target
stays current while that new tab installs in the background, so its visible preview survives until
the user resolves it. Failure, cancellation, shutdown, invalid receipt, and construction failure
preserve all existing tabs. A matching origin token activates the existing tab, including for a
hard-link alias, and disposes the newly admitted receipt.

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
for the selected registered ordinary-native tab with a complete current projection, no close/replacement,
target-owned asynchronous work, or active canvas interaction; while a tool or target-owned operation is
active its guidance is to finish, cancel, or let that work complete. It
captures an immutable target fence, prepares and authenticates the source first, then revalidates the
registered current target before deciding anything destructive. A matching admitted descriptor token activates the
already-open native tab and preserves the requested target. A clean saved populated target is constructed
and swapped atomically at its captured index without a redundant confirmation. A dirty target alone offers
Save (default), Replace, and Cancel: named Save uses native Save, unnamed Save uses native Save As, and a
successful publication must establish and pass a fresh post-save fence before the swap. Stale, busy, close,
cancel, admission, construction, or save failures discard the pending receipt, preserve the target's
selection, tool, preview, and focus, and report an explicit retry; this command never becomes NewTab.
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

The product accepts local CDML, the bounded decoded-CD-SVG profile, and the closed
Rust-owned CML/CML2 simple-molecule profile. CML always converts into a clean new tab and
never replaces the current tab; it has no CML Save baseline or export route. The product
explicitly refuses compressed copies, `.svgz`, `.cdsvg`, `.cdxml`, and unsupported or
incomplete documents without reading, sniffing, or converting them. The recovery copy writes
CDML only; it does not offer format conversion. Broader historical editing modes, template
catalogs, import/export families, and clipboard or presentation workflows are preproduction
drops unless a later slice gives them a complete Rust owner and an explicit user contract.

Historical note: the former mixed-host migration bridge and its retained-session shutdown
path were removed with the second host. They are not current product behavior or test
requirements.

## Verification evidence

Permanent tests stay small and behavior-focused: a supported native operation, typed
refusal with preserved document state, and Rust/PyO3 contracts that can regress independently.
They do not assert private worker wiring, module inventories, timing, fixed action counts,
or visual bytes. Source/dependency inventories, fresh wheel or installed-site checks,
manual walkthroughs, screenshots, accessibility checks, and race observations are useful
one-time implementation evidence, not permanent fast-suite gates.

Before accepting a slice that claims those capabilities, prove its actual user
path: open or create a document, obtain the required revision-bound Ferrum
observation, paint its plan with verified Telex bytes, select durable IDs,
submit its supported operation, replace the projection, and save/reopen through
Rust without losing CDML order, IDs, or opaque content. A bounded slice proves
only the operations and document classes named by its receipt.

Run focused Rust, PyO3, and offscreen Qt behavior tests for the changed slice.
Use a managed end-to-end receipt for installed-wheel and visual evidence. Keep
the fast suite free of timing, private-wiring, fixture-heavy, and exact-count
checks. Follow the repository source-file guidance rather than treating a line
count as a functional acceptance gate.
