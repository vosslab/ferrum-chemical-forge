# Ferrum-Qt contract

Ferrum-Qt is a Qt Widgets document editor. This contract defines its durable
ownership boundary while the Ferrum render-plan path replaces the temporary
OASA-backed path capability by capability. It follows the active
[ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md) and applies the repository
principles "Fix the design, not the symptom" and "Long-term over short-term."

## Current composition

The legacy `MainWindow` is the application composition facade. It builds the
canvas, menus, toolbars, status bar, preferences, clipboard, and tabbed
workspace. Its window controller mixins are split into focused `window_*.py`
modules, while each legacy open tab has one legacy `DocumentSession`.

```text
legacy MainWindow (QMainWindow)
  tab workspace and active-tab controller
    legacy DocumentSession, one per legacy tab
      QGraphicsScene + QGraphicsView + mode manager
      current temporary document/projection and import workers
  menus, toolbars, dialogs, clipboard, docks, preferences, status

FerrumNativeMainWindow (standalone QMainWindow bounded editor)
  native CDML tabs only
    Rust-owned ferrum_chem.DocumentSession and disposable Qt projection
  Open, Save, Save As, Close Tab, Change Element, Add Atom at Point,
  Undo, Redo, Refresh Authoritative View, and Quit
```

The public legacy `MainWindow` remains a composition and delegation layer. It
owns widgets, active-tab aliases, connection scopes, window dialogs, and
orderly shutdown. It does not own CDML parsing, canonical saving, chemistry
requests, projection construction, graphics disposal loops, or worker result
policy.

The current source has an intentionally transitional `DocumentSession` path.
It still imports OASA for CDML authority, chemistry, and legacy rendering. This
is not Ferrum adoption and is not a compatibility target. The Rust replacement
deletes that path and its provisional `__bkchem_new__` identifiers when the
corresponding Ferrum capability is complete. Historical CDML namespaces remain
valid document data; they are not product branding.

## Current completed slices

The standalone `FerrumNativeMainWindow` is the public `ferrum-qt --native`
bounded editor for a Rust-owned CDML route. It opens, renders, saves, reopens,
and closes native tabs with an extension-owned `ferrum_chem.DocumentSession`.
It can change one durably selected atom's element, add one free-standing atom
at an exact scene point in an explicitly chosen durable molecule, and apply
Rust-owned undo and redo. Its reachable actions do not import OASA, instantiate
the legacy `MainWindow`, or fall back to a legacy tab. It intentionally exposes
only operations with a complete Rust implementation.

The legacy `MainWindow` remains a migration-only OASA path. It is not made
Rust-owned by the preview, and it continues to contain unreplaced editing,
codec, worker, and application-shell behavior. The two window types are
separate composition roots rather than alternate backends behind one tab or
session interface.

Current V1 Qt slices also include the exact Rust molecule-plan painter, the
separate presentation-vector projection, and a Rust/PyO3 display-geometry
bridge. These are bounded evidence for their own routes, not completion claims
for full geometry, rendering, editing, export, or legacy replacement.

The closed CDML paper-name catalog is also Rust-owned and crosses PyO3 as frozen
name and millimetre-dimension values. Qt scene setup, snapshot rendering, and
the transitional session catalog adapter consume those issued values. Qt does
not maintain a second paper table or infer missing dimensions. The standalone
native observation resolves orientation and physical dimensions into one finite
scene rectangle at the document origin using 72 points per inch. Qt paints that
rectangle as a noninteractive page decoration behind document roots and may use
the current UI palette for its fill and outline. Preserved malformed paper facts
remain document data; Rust supplies a typed compatibility issue and the A4
portrait display fallback. Normal-window paper/session adoption remains open.

Standalone native snapshot export never paints the installed view scene. The tab
asks Rust for a fresh render observation at the displayed revision and builds a
detached, initially unselected projection from the same verified Telex resource.
An exact revision/digest mismatch rejects export. Qt paints that detached scene to
SVG, PDF, or PNG, then terminally retires it. SVG and PDF are vector outputs. PNG
uses the established 72-point-per-inch document scale and must fit Qt's configured
image-allocation limit. `QSaveFile` publishes all formats atomically; an existing
symbolic-link or non-file destination is rejected before painting. This frontend
snapshot route does not claim the independent M13 Cairo/`xot` backend milestone.

## Authority and flow

In a Rust-owned tab, one Rust `ferrum-document` session is the authority for
the complete CDML document, durable identifiers and source order, revision and
history, dirty/save baseline, accepted mutations, recovery publication, and
canonical save. `ferrum-chemistry` owns chemistry. `ferrum-render` owns
declarative rendering. The PyO3 module `ferrum_chem` publishes copied, frozen
values and typed errors at this boundary. The legacy tab route has not reached
this boundary and remains explicitly transitional.

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
such as title, dirty, diagnostic, and worker-retired changes. Sibling widgets
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
render metrics. It does not consume legacy atom or bond models, OASA operations,
or Python XML. The final observation contains frozen
`DocumentMoleculeRenderPlanV1` values in document root order. Each entry keeps
document-root molecule identity and order separate from the molecule-local atom
and bond order inside its `RenderPlanV1`.

```text
SessionDocumentObservationV1
  snapshot: revision, digest, and dirty state
  projection: typed durable facts with the same revision and digest

RenderObservationV1
  document: SessionDocumentObservationV1
  molecule_plans: tuple[DocumentMoleculeRenderPlanV1, ...]

DocumentMoleculeRenderPlanV1
  molecule: MoleculeRenderRootV1
    id: durable document object key or null
    projection_key: projection-local identity
    source_id: authored CDML ID or null
    source_order: direct document-root position
  plan: RenderPlanV1

RenderPlanV1
  schema: ferrum-render-plan-v1
  provenance:
    revision: exactly document.snapshot.revision
    digest: exactly document.snapshot.digest
  batches: tuple[RenderBatchV1, ...]
  issues: tuple[RenderIssueV1, ...]
```

Each molecule entry becomes one disposable root graphics group whose z order is
the backend-issued molecule `source_order`. Each batch has a durable
`RenderTargetV1` with `record_id.kind`,
`record_id.id`, and `source_order`, plus a declared coordinate space. Ordered
tagged operations contain a `LineOpV1`, `MaskOpV1`, or `TextOpV1` payload.
Scene-space bond batches contain lines; atom-local batches contain masks and
text. Paint is explicit lowercase six-digit `Rgb24`. Text runs declare their
supplied origins, exact glyph identifiers, exact glyph origins, script, scale,
size, face, and paint. Paint is document depiction data, not a Qt palette
selection. The plan is complete or reports explicit target issues; Qt does not
infer missing coordinates, hydrogen policy, line width, color, visibility,
font, glyph mapping, or other depiction defaults.

`FerrumRenderProjection` validates the schema, exact revision, root envelope,
and DTO shape before allocating scene objects. It builds a complete detached
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

`Move Complete Roots` captures the exact current Rust selectors for complete
durable roots, their durable selection keys, immutable projected scene bounds,
and observation provenance. Qt paints dashed local rectangles and translates
only their shared disposable preview group while the pointer moves. Escape,
stale provenance, tab change, or teardown retires that group without submission.
Release retires it before one captured-revision Rust translation; the accepted
replacement observation restores the same durable selection. Qt never moves an
installed render-plan or presentation-root item and never derives persistent
coordinates from the preview rectangles.

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

## Workers and UI thread

The first render-plan slice is Qt-thread affine: `ferrum_chem` calls and all
QGraphics construction occur on the Qt main thread. Background render work is
introduced only with an owned-value handoff and a documented main-thread
projection delivery contract.

Current import workers remain asynchronous. A worker has one owning session
while live. Its queued relay delivers once on the UI thread only if its session
and request token are still current. A stale, cancelled, or closed-tab result
cannot mutate widgets or projection state. Closing or replacing a tab requests
interruption, transfers a still-running worker to the window's terminal owner,
and releases its Qt wrapper only after `finished`. Shutdown waits for that
retirement path before it reports completion.

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

## Replacement path

The cutover proceeds capability by capability, with no fallback or OASA-shaped
adapter:

1. Publish frozen revision-bound Rust observations, typed projection facts, and
   the smallest durable operation through `ferrum_chem`.
2. Build and verify the isolated RenderPlanV1/Telex Qt painter and projection
   controller using those values.
3. Route the completed vertical slice through the tab session, then remove its
   OASA producer, legacy atom/bond rendering bridge, provisional IDs, and tests.
4. Repeat for each document, chemistry, presentation, and export capability
   until no production Qt source imports OASA or exposes BKChem branding.

This contract does not claim that the current temporary path has been replaced,
that unsupported document features render, or that full edit, save, export, and
codec coverage is complete. In particular, free-standing atom insertion does
not provide bond creation, Draw-mode equivalence, atom movement or deletion,
coordinate generation, or general molecule import. It defines the durable path
that each completed slice must follow.

## Verification evidence

Before accepting a slice that claims those capabilities, prove its actual user
path: open or create a document, obtain the required revision-bound Ferrum
observation, paint its plan with verified Telex bytes, select durable IDs,
submit its supported operation, replace the projection, and save/reopen through
Rust without losing CDML order, IDs, or opaque content. A bounded slice proves
only the operations and document classes named by its receipt.

Run focused Rust, PyO3, and offscreen Qt behavior tests for the changed slice.
Use a managed end-to-end receipt for installed-wheel and visual evidence. Keep
the fast suite free of timing, private-wiring, fixture-heavy, or exact-count
checks. The source-file limit remains below 1,000 physical lines for every
authored implementation, test, and durable documentation file.
