# Retired Python backend ownership ledger

## Purpose and authority

This is the M15/M16/M22 runtime-ownership transfer ledger. It answers which production
boundary owns each capability while release closure remains incomplete. The detailed
user-facing classification and exit evidence remain in
[`ferrum_qt_capability_matrix.md`](ferrum_qt_capability_matrix.md); this file records
only the shorter ownership decision needed to remove the compatibility island safely.

Two historical upstream evidence streams remain distinct:

- Changes under the former Python chemistry package primarily refined the
  CDML/behavior contract used to specify Ferrum semantics.
- Changes under the read-only frontend reference are product and interface
  improvements. Ferrum should adopt applicable improvements, but any persistent action
  must receive a Rust document/session owner rather than carry its OASA plumbing forward.

The 2026-08-15 compatibility-host retirement checkpoint is accepted. Ordinary
`ferrum-qt` starts the only production window, `MainWindow`, which is Rust-native.
The explicit compatibility host, its session/mode/worker/codec island, and the
production Python-backend declarations are retired. Historical names remain only
where provenance or accepted migration evidence requires them; there is no live
Python chemistry comparison worker or production runtime owner.

## Current production boundary

No production Ferrum module imports the retired Python chemistry backend or
dynamically loads the retired host. The root requirements and Ferrum project
metadata no longer declare it. This is recorded as a retirement result, not
maintained as a source-count gate.

## Milestone disposition

| Milestone | Ledger disposition |
| --- | --- |
| M15, domain utilities | Done: accepted native repair, bounded peptide insertion, linear-form, and Haworth-adjacent domain slices have named Rust owners. Remaining legacy utilities are explicit drops, not incomplete shared ownership. |
| M16, full session adoption | Done: the ordinary Rust-native `MainWindow` owns every retained document route; the compatibility host has no production role. |
| M19, integration closure | Implementation complete, pending independent closure review: supported-row evidence is indexed in the capability matrix; session confinement and serialized GUI-thread mutation are recorded without a timing claim. |
| M22, supported-product release closure | Open: package and release evidence still decide its broader release boundary. |

| Runtime boundary | Current owner | Disposition |
| --- | --- | --- |
| Desktop window and document lifecycle | Rust `DocumentSession` plus ordinary `MainWindow` | Retained product route |
| Native codecs, authoring, publication, and asynchronous clients | Named Rust/native controllers | Retained only within their accepted contracts |
| Legacy session, modes, registry, OASA workers/codecs, projections, and renderer | None | Retired with the compatibility host |
| Historical comparison inputs | Accepted reports only; live Python workers retired | Not a production dependency or desktop fallback |

## Capability ownership decisions

"Native" means the ordinary product route has one Ferrum-owned authority. "Partial" means
the row has a bounded native route while broader work remains unimplemented; it does not
mean a compatibility host remains. Detailed dispositions stay in the capability matrix.

| Capability rows | Ordinary product authority | Current disposition |
| --- | --- | --- |
| FQ-001--003 | Rust `DocumentSession` and ordinary `MainWindow` own startup, document admission, typed projection, Save/Save As, close, recovery, duplicate activation, fenced current-tab replacement, Recent Files, and the bounded decoded CD-SVG Open V1 route. | The retired host owns nothing. CD-SVG export/round trip is dropped; `.cdsvg`, `.svgz`, and compressed CDML are refused by the ordinary window without document mutation. |
| FQ-004--006 | Rust-native molecule codecs and artifact controllers own admitted imports, safe publication, and ordinary `File -> Export...` SVG, PDF, and transparent PNG. | CDXML and CML are refused. CD-SVG export/round trip and a public export API/CLI freeze are pre-production drops, not compatibility routes. |
| FQ-007/007a | Rust document operations and ordinary Qt gesture clients own the accepted atom/bond creation, editing, deletion, projection, rendering, bounded Properties client, `n1`/`n2`/`n3`, `w1`/`h1`, and C6 ring slices. | Unported modes, ring variants, fusion/aromaticity/heteroatom policy, and other bond styles are pre-production drops until a new native contract names an owner. |
| FQ-008--009 | Rust-native SMILES/InChI, coordinate generation, and authenticated insertion own the accepted bounded route. | Unported codecs and preparation workflows are pre-production drops; they have no OASA worker or fallback route. |
| FQ-010 | Rust owns the accepted information, exact-name, explicit-fragment, linear-form, and bounded `Chemistry -> Check Bond Capacity...` slices. | Oxidation, historical group/search tools, generated names, and broader checks are pre-production drops pending a new native contract. |
| FQ-011 | Rust-native document/session repair operations own `normalize-bond-lengths`, `normalize-bond-angles`, `normalize-rings`, `snap-to-hex-grid`, `straighten-bonds`, and clean geometry; accepted rotation, scale, mirror, ordering, and complete-root translation retain their named native owners. | Unported repair, transform, and gesture variants are pre-production drops pending native design and ownership. |
| FQ-012--013 | Rust owns the four-recipe D-glucose route and Direct-Glycosidic Haworth V1. | Verified sucrose, compact sugar catalogs, broader carbohydrate inference/templates, and unported direct-glycosidic variants are pre-production drops. |
| FQ-014 | Rust owns the bounded `ferrum-native-peptide-structure-v1` route, prepared by `prepare_ferrum_peptide_insertion_v1`, for its documented native-17 profile. | System and biomolecule template catalogs, descriptive catalogs, and generic peptide authoring remain pre-production drops; bounded insertion is not OASA parity. |
| FQ-015 | None. | PubChem is an explicit pre-production drop; it has no production transport, rate, error, or fallback owner. |
| FQ-016 | Rust inspection, secure Ferrum user-template catalog refresh, finite-anchor placement, and ordinary native save own their accepted slices. | System and biomolecule template insertion are pre-production drops. |
| FQ-017/017a | Rust document presentation plus the ordinary Qt client own accepted presentation editing, atom numbers, and marks. | Unsupported faces/splines, personal drawing defaults, direct-mark variants, and explicit overrides are pre-production drops. |
| FQ-018 | Rust owns accepted order, scale, mirror, alignment, linear-form, and complete-root translation anchors. | Generic object/property commands outside accepted native slices are pre-production drops. |
| FQ-019 | Rust owns accepted Copy, Cut, Paste, and selected-root SVG slices. | Unported clipboard variants and broader public exposure are pre-production drops. |
| FQ-020 | Ordinary Qt and Rust own accepted view/theme/window state, zoom, toolbars, Editing Tools, the periodic display catalog, QSettings preferences, paper grid, snapping, complete-root moves, and C6 action. | Unported drawing gestures, shortcuts, ring variants, and deferred bond styles are pre-production drops. |
| FQ-021--022 | Rust owns identity/metadata and required built-in registration. | Third-party plugin execution remains an explicit pre-production drop. |
| FQ-023 | Named Rust-native asynchronous clients own the accepted slices, including their cancellation and stale-delivery contracts. | Retired compatibility workers have no replacement facade. New asynchronous work must declare its own Rust owner and confinement contract. |

## Transfer log

Entries before the 2026-08-15 retirement checkpoint preserve the accepted state at
the time of that work. Their compatibility-host references are historical; the
current ownership decision is the table above.

### 2026-08-15: ordinary native artifact-export adoption (accepted)

The ordinary Rust-native product now owns `File -> Export...` for complete current-document
SVG, vector PDF, and transparent PNG at one pixel per Rust page point. Qt captures an immutable
observation plus revision/digest and origin token, fences before chooser/work/delivery, and sends
only the observation to a QThread. Rust prepares the closed receipt, refuses unsupported complete
roots rather than exporting a partial page, then descriptor-relatively publishes a reauthenticated
receipt. A locally admitted CDML or decoded CD-SVG tab retains a Rust-owned descriptor whose direct
or observed hard-link alias cannot become the destination. Qt owns chooser, worker lifetime,
current-tab/close/Open containment, and typed user recovery; it contains no scene-render or
`QSaveFile` fallback. Three compact private-bridge tests are permanent; wheel/site, decoder,
visual/a11y, busy-race, and installed-window checks are disposable evidence. This is neither
CD-SVG export nor a public API/CLI/wire contract.

### 2026-08-15: compatibility-host retirement (checkpoint accepted)

The explicit `LegacyCompatibilityMainWindow` and its OASA document session, action registry,
mode graph, workers, codecs, render/projection paths, and production dependency declarations
are removed. One ordinary Rust-native `MainWindow` remains. The retained `Recovery Export CDML...`
copy path is distinct from Save and artifact export. CDXML, CML, `.cdsvg`, `.svgz`, and compressed
CDML are explicitly refused without document mutation; PubChem, compact sugar/descriptive
catalogs, system/biomolecule templates, historical group/search tools, unported legacy modes,
properties, and clipboard variants are pre-production drops. Accepted native repair and bounded
peptide insertion retain their separate Rust owners.

Permanent coverage remains compact native behavior plus one representative CDXML
refusal/nonmutation test. Current-source/package inspection, OASA-absent site launch, and an
installed ordinary-window walkthrough are disposable evidence. The accepted walkthrough observed
ordinary startup without OASA or compatibility UI, public CDML Open, Radical edit semantic
Undo/Redo, retained Recent/Export/Recovery labels, and a real CDXML modal refusal/nonmutation.
Save/reopen and artifact publication retain their separate accepted evidence.

### 2026-08-14: compatibility native-bridge retirement (historical)

Ordinary CLI startup and `MainWindow` were the sole Rust-native document owner. The
explicit `LegacyCompatibilityMainWindow` then owned one OASA-backed legacy-session
island. Its mixed native-tab/file bases, registries, guards, action-policy, explicit native
CDML route, duplicate `... with Ferrum` actions, and cross-owner session-menu/shutdown seams
were retired; `window_native_tabs.py` was removed. `window_native_files.py` stayed with the
ordinary native owner. The legacy island then owned codecs, modes/ribbons, property dock,
clipboard/templates, chemistry workers, import/export, and controlled shutdown.

At that checkpoint, OASA remained a declared dependency and the host remained live. Transfer,
drop, and M19 decisions for retained capabilities were still required before M16/M22 closure. Permanent tests
retain semantic ordinary lifecycle and explicit legacy-session/real-worker-drain behavior;
bridge-only coexistence and frozen catalog tests were removed. Source/action inventories,
wheel/site checks, walkthroughs, screenshots, and timing are disposable; walkthrough
evidence accepted the ordinary OASA-omitted lifecycle and explicit legacy session with real
worker drain without becoming permanent test coverage.

### 2026-08-14: standalone D-glucose Haworth insertion

The ordinary Rust-native product now owns one deliberately closed FQ-012 action:
`Edit -> Insert Haworth Ring...`, also projected into Editing Tools. It admits only
alpha/beta D-glucopyranose and alpha/beta D-glucofuranose. Rust owns each literal C6O6
recipe, the corrected O5-pyranose and O4-furanose closures, finite local geometry, IDs,
CDML, one-use revision/digest receipt, history, selection, and normal Render Plan V2
output. Qt owns a readable form/anomer chooser, one captured shared-snap anchor, and a
receipt-only preview. The `q1` front stroke and directed `w1` shoulders use the ordinary
source-owned V2 cap/layer route; `n1` retains the remaining edges. No chooser preference,
compact code, or UI metadata enters CDML or QSettings in this slice. The then-live legacy
OASA sugar route was separately owned by the compatibility host. Permanent tests are compact
recipe, transaction, render, binding, and ordinary interaction behavior; OASA comparison,
source capture, wheel/site, screenshots, and visual/accessibility review are one-time evidence.
The accepted current-source/installed-site walkthrough placed every recipe through the public
chooser, confirmed snap/preview/commit and occupied/cancel/stale preservation, restored semantic
CDML/history through public tab Undo/Redo with expected revision advancement, and saved/reopened
the inserted molecules. Its focused rewalk confirms that an authoritative atom or bond at the raw
or snapped placement point preserves document/selection and retains the armed intent for a later
empty-page click. It confirmed no Haworth UI metadata and closed cleanly with the general clipboard
lifecycle boundary intact.
Other sugars, attachment/fusion, rotation/reflow, and general stereochemical inference need
new source-backed contracts.

### 2026-08-14: bounded native bond-capacity check (accepted)

The ordinary Rust-native route now has an accepted `Chemistry -> Check Bond Capacity...`
diagnostic for selected complete direct roots. Rust owns the finite neutral H/B/C/N/O/F/Cl/Br/I
capacity table, closed grammar, explicit-H plus incident single/double/triple demand,
authenticated immutable receipt, and authored charge/explicit-H provenance. Assessed atoms
receive Within Capacity or Exceeds Capacity; a non-neutral, aromatic, unsupported, or
authored-capacity-fact root receives one Not checked result. Bond depiction is presentation-only
and does not affect demand. Qt owns worker fencing and the selectable read-only dialog through a
private PyO3 seam. There is no OASA, CDML, history, selection, settings, public Python, CLI, or
wire ownership transfer. The diagnostic is not a chemical-validity, general-valence, or
oxidation-state claim. Compact semantic tests are permanent; wheel/site, visual, OASA, and
timing checks are disposable. Accepted evidence09 and checkpoint rereview08 include a public
real-worker walkthrough of supported/no-excess/finding/Not checked outcomes, authored-presence
display for every assessed atom in a mixed excess root, root order, depiction independence,
lifecycle/nonmutation, and accessibility.

### 2026-08-14: complete-root translation anchor

Move Complete Roots now receives a private immutable Rust
`TopLevelTranslationAnchorV1` observation containing canonical complete-root
selectors, exact source revision/digest, and the finite lower-left authored
union anchor. The raw typed-document geometry helper remains crate-private;
the observation never becomes CDML, history, or preference data. Qt captures
the existing `Snap New and Moved Points to Hex Grid` boolean at press. Enabled
moves resolve `snap(anchor + raw_delta) - anchor`; disabled moves retain the
raw delta. The resulting one rigid delta drives the projection-only overlay and
the established revision-fenced Rust transform. Rust remains the owner of
validation, selection, undo, save/reopen, refusal, and recovery. Compact
semantic receipt, binding, and visible gesture tests are permanent; wheel,
overlay, screenshot, accessibility, and cache checks are disposable. Rotation
and exact existing-atom joins stay separate interaction contracts. This changes
no compatibility-host ownership or cutover decision.

### 2026-08-14: Render Plan V2 scene-path foundation

The active molecule-plan boundary now emits and accepts only
`ferrum-render-plan-v2`. Rust owns the neutral validated `MoveTo`, `LineTo`,
`CubicTo`, and `Close` scene-path grammar with explicit stroke, fill, and z;
the common draw stream carries received facts through SVG, PNG, PDF, bounds,
and composite recording. Private PyO3 DTOs and Qt consume the same V2 facts,
and Qt only copies paths into disposable painter geometry. `RenderObservationV1`
remains the revision-bound document/projection receipt envelope, not a legacy
molecule-plan grammar; its payload now contains `DocumentMoleculeRenderPlanV2`.
This establishes the general capacity that later supports wedge/hash depiction,
bond-style creation, and a Next presentation control. Compact semantic tests cover
the V2 boundary. The fresh self-contained wheel/site and broad artifact checks
are disposable evidence, with no byte, pixel, timing, or count gate.

### 2026-08-14: closed native bond-presentation authoring

The ordinary Ferrum-native root now routes one closed Rust-owned creation value,
`DocumentBondPresentationV1`, through the private PyO3 session boundary. Normal
single/double/triple writes `n1`/`n2`/`n3`; SolidWedge writes `w1`; and
HashedWedge writes `h1`. Directed forms preserve gesture order: press/start is
the narrow tip and release or new atom is the wide base. Duplicate detection
remains undirected, while the accepted projection retains the authored order.
The V2 common renderer lowers solid wedges as filled paths and hashed wedges as
finite widening lines, and Qt paints only source-issued preview and committed
operations. `Next presentation:` is a QSettings workflow preference shared by
ordinary live windows and captured with element/order at press; it never enters
CDML, `<standard>`, Rust document state, history, dirty/save state, or selection.
Compact semantic tests remain permanent. OASA probes, wheel builds, visual and
accessibility walkthroughs are disposable evidence. `w2`, `w3`, `h2`, `h3`, and
all other styles remain deferred. This did not change the then-live compatibility-host
cutover.

### 2026-08-14: ordinary linear-form conversion

`Chemistry -> Convert selection to linear form` now belongs to the ordinary Rust-owned
tab. Qt translates selected projected atoms/bonds to one authenticated opaque root and a
source-ordered atom tuple; Rust owns `linear-form-direction-v1`, geometry, metadata,
resources, history, and refusal. The runtime-only PyO3 seam has no public stub, CLI, serde,
or wire promise. The then-live compatibility host retained its OASA request/commit route;
the later retirement checkpoint supersedes that temporary boundary.

### 2026-08-14: continuous native zoom control

The newer BKChem-Qt status-bar slider is adopted as an interface improvement, not as a
legacy compatibility obligation. Ferrum's status widget only projects the active view and
emits an integer request; the native graphics view owns the 10%-1000% contract, absolute
transform, stable center anchor, invalidation events, and refresh signal. No OASA import,
document mutation, compatibility owner, or persistent preference was added.

### 2026-08-14: ordinary shared-action toolbar

The newer BKChem-Qt frequent-action toolbar is adopted as an interface improvement, but
its separately created callbacks and document-facing state are not. The ordinary window
projects existing native New, Open, Save, history, clipboard, and zoom actions through one
labeled toolbar. Their established menu, Rust session, clipboard worker, and tab-owned
view boundaries remain authoritative. Qt owns only layout, native overflow, platform icon
presentation, and a View-menu visibility action. Permanent tests cover a real document
command and the user visibility choice; disposable wide/narrow screenshots checked the
layout without creating breakpoint, pixel, timing, exact-action-list, or icon-list gates.

### 2026-08-14: ordinary native Editing Tools client

The ordinary Ferrum-native root now adds a second-row `Editing Tools` QToolBar with a stable Qt
workspace identity, visible category header, and View visibility control. It projects the exact
native Add Atom, Draw Bond, Wavy, rectangular-bracket, round-bracket, Move Atom, Rotate
Selected Atoms, and Move Complete Roots actions, plus one window-owned shared `Cancel Tool`
Escape recovery action. Layout, icons,
accessibility, theme projection, and overflow remain UI-client concerns. Existing action callbacks,
gesture intents, prerequisites, document/session/history/selection, Rust mutation, snap policy, and
transform exclusions retain their present owners. Cancel Tool composes existing cancellation boundaries
while preserving document and selection. This ports BKChem-Qt discoverability without importing legacy
`ModeToolbar`, `SubModeRibbon`, `EditRibbon`, `ModeManager`, or mode YAML as product owners; those
remain interface evidence. Permanent tests cover one real toolbar bond gesture, Escape preservation,
distinct Cancel Tool preservation, and remembered user-hidden toolbar state. Wide/narrow visual,
overflow, icon/grouping/accessibility walkthrough, workspace-byte, timing, count, pixel, and
installed-wheel UI checks are one-time evidence.

### 2026-08-14: native Next Drawing parameters

The ordinary Ferrum-native root now supplies `Next atom:` and `Next bond:` in the same Editing Tools
toolbar. They are application/QSettings preferences with defaults C and single, shared by ordinary
native windows and kept outside CDML, `<standard>`, Rust document state, history, dirty/save state,
and selection. The Rust periodic catalog supplies suggestions and canonical familiar spelling, while
valid ASCII-letter plain or pseudo-atom names remain admissible for the existing Rust candidate
validator. Add Atom freezes its element for one click; Draw Bond freezes element and order at mouse
press. Existing-atom joins pass explicit durable IDs and preserve the accepted selection on refusal;
empty-space endpoints create the frozen element and normal single, double, or triple authored CDML
bond in one Rust-owned operation at the shared snap target. Rust retains validation, identity,
atomic history, projection, and save/reopen authority. Independent current-source keyboard evidence
accepts inactive and Draw Bond-armed Next atom Escape recovery: it restores the visible/effective
value, uses shared cancellation when active, and preserves snapshot/selection. Small preference and
native behavior tests remain permanent; keyboard, visual, accessibility, overflow, and installed-wheel
checks remain disposable evidence. The later closed native presentation slice adds the separate
`Next presentation:` QSettings client and Rust-owned `n1`/`n2`/`n3`/`w1`/`h1` grammar recorded
above; it did not change the element/order preference boundary or the then-live
compatibility host.

### 2026-08-14: native Properties projection client

The newer BKChem-Qt Properties dock is adopted without its OASA document reference,
direct edit callbacks, or local undo stack. Each native tab supplies one immutable
inspection receipt only when its installed Rust document projection and disposable scene
share revision/digest. The dock derives display text from those frozen facts and reuses
the already-owned atom/bond edit actions. Pending authoritative refresh produces an
explicit unavailable state rather than stale properties. A compact permanent test covers
active-tab selection behavior; the wide/narrow screenshots and action/widget inspection
were disposable checks rather than width, pixel, count, or wiring gates. The then-live
compatibility host retained its separate OASA property dock before retirement.

### 2026-08-14: insertion-valid native Paste

Ordinary `Edit -> Paste` now transfers from clipboard transport into one Rust-owned
document transaction. Rust owns the closed fragment grammar, named resource profile,
fresh persistent identifiers, exact declared-ID reference remapping, one group
translation, complete-candidate validation, history, and inserted-root receipts. The
private worker-safe PyO3 plan remains outside `.pyi`; Qt owns only one UI-thread clipboard
capture, cancellable preparation, current-tab/revision/digest delivery, authoritative
scene installation, and projected selection. The then-live compatibility host retained its
separate OASA-backed Cut/Paste route before retirement.

### 2026-08-14: recoverable native Cut

Ordinary `Edit -> Cut` now composes one insertion-valid Copy fragment with an exact
source-authenticated Rust deletion plan. Structural Cut owns atom/bond topology cleanup,
generated-linear-form retirement, complete-root cleanup, projection validation, and one history
transition. Presentation Cut reuses the complete direct-root deletion owner. Qt owns worker
scheduling, current-tab/selection delivery, clipboard publication, and scene installation. It
publishes first, then commits, so a recoverable commit refusal produces a usable Copy result and
leaves the source unchanged. Mixed or multi-molecule complete-root Copy fallback has an explicit
Cut refusal with no ambiguous partial deletion. The private PyO3 seam remains outside the public
stub/CLI/wire surface. The then-live compatibility host retained its separate OASA-backed
clipboard route before retirement.

### 2026-08-14: native selected-root SVG

Ordinary `Edit -> Copy as SVG` now composes a selected subset of the authenticated Rust
document render plan. Atom, bond, and durable molecule selectors retain complete molecule
roots; presentation selectors retain their exact direct roots. Rust owns selected-root
resolution, profile-exclusion refusal, conservative content measurement over the shared
lowered draw stream, the fitted viewport, bounded SVG generation, and the immutable receipt.
Qt owns only disposable scene-selection mapping, cancellable worker delivery, current
tab/revision/digest/selection fences, and final clipboard MIME publication. Permanent tests
retain semantic selection, provenance, nonmutation, and failure containment; the fresh wheel
build is one-time evidence rather than a byte, pixel, exact-bounds, timing, or count gate. The
private PyO3 entry remains outside `.pyi`, CLI, serde, and wire surfaces. The then-live
compatibility host retained its OASA-backed selected-SVG route before retirement.

### 2026-08-14: application-only native Preferences

The ordinary product replaces its standalone Theme command with a focused
`Options -> Preferences...` surface. The application theme manager retains theme
authority, while QSettings owns the user's workspace-restoration, grid-visibility, and
grid-snap choices plus Qt's opaque window, toolbar, and Properties-panel layout state. The grid is
one disposable paper-local projection built from the existing Rust display-geometry bridge;
the renderer remains authoritative for the paper rectangle. Its shared View, toolbar, and
Preferences clients apply each choice to current tabs, later tabs, and replacement scenes. The checked
snap action also has `Ctrl+Shift+G` and a shared toolbar client. `FerrumNativeGraphicsView` owns the
single finite authored-point policy, delegating nearest-lattice math at the existing grid spacing to the
Rust display-geometry bridge. Free atoms, template centroid anchors, empty-space bonded-atom endpoints,
moved-atom targets, Wavy endpoints, and rectangular/round bracket corners use it for both preview and
commit. Existing-atom joins remain exact; rotation stays angle input; and complete-root translation now
uses a private Rust-authored anchor receipt plus a press-captured snap policy.
A fully accepted shutdown captures workspace state only when requested; clearing that choice
removes the stored workspace. CDML, Rust document ownership, save state, selection, and history remain
unchanged. The legacy Preferences dialog remained with the then-live compatibility host because its
gesture and shortcut settings depended on legacy owners. Permanent tests cover semantic snap
enabled/disabled placement and preference propagation, public accepted/cancelled behavior, grid state
across a native edit, and visible restoration. Offscreen visual and gesture-preview inspections plus a
current-source wheel review are one-time evidence rather than pixel, size, field-count, timing, or
private-wiring gates.

### 2026-08-14: explicit plugin-surface drop

FQ-022 retains explicit built-in registrar loading and required YAML menu preflight. The
pre-release product drops third-party plugin execution because the former empty registrar,
label-inferred import/export slots, and unused optional-action flag provided no extension
contract. Those placeholders are removed. A future extension system needs a new discovery,
permission, versioning, lifecycle, and failure-containment design. The exact registrar-list,
private cascade-set, and optional-plugin wiring tests were deleted under the permanent-test
checklist; callable registration and visible missing-action failure behavior remain covered.

### 2026-08-14: explicit fragment V1 (accepted)

The ordinary Rust tab now has an accepted `Chemistry -> Create Fragment...` /
`View Fragments...` route. Rust owns the one-molecule direct-child admission, selected-bond
endpoint closure, source order, collision-safe ID, candidate, one-use commit, CDML, history,
and safe scalar observation. Qt owns the captured-tab name/list dialogs, status, focus, and
authoritative installation; it has no XML, local model, undo, preferences, or OASA action.
Only explicit records are authored; duplicate names and disconnected metadata members are valid.
The read-only view lists exact supported records and a scalar notice for retained unsupported
metadata. Clipboard/group/template/inference/cross-molecule semantics, delete/rename/edit/highlight,
and public `.pyi`/CLI/wire remain outside V1. Compatibility-host action transfer was superseded
by retirement. Independent
rereview accepted the View-lifecycle and typed-error repairs; installed public evidence accepted
closure/order, duplicate labels, blank retry/Cancel/stale containment, retained notice, View
lifecycle, undo/redo, save/reopen, and OASA/QSettings/public-surface hygiene. Compact semantic tests
are permanent evidence; wheel/site, screenshots, visual, keyboard/accessibility, corpus, and timing
probes remain disposable.

### 2026-08-15: Direct-Glycosidic Haworth V1 (accepted)

The ordinary native root now implements `Chemistry -> Insert Direct-Glycosidic
Haworth...` as a private, Rust-owned structural-SMILES route. It admits only a
neutral nonaromatic single-bond graph of two disjoint five- or six-member C/O rings
and one exterior degree-two oxygen bridge. Rust owns the graph, receipt, ordinary
durable C/O/q1/w1/n1 drawing, CDML, history, selection, persistence, and normal V2
rendering; private PyO3 transports typed receipt/preview/commit data; Qt owns the
empty accessible text form, captured-tab lifecycle, one shared snap anchor, and
receipt-only preview. It adds no SMILES, name, parser coordinate, UI state, or
preference to CDML/QSettings and no OASA, public `.pyi`, CLI, wire, or composite
owner. This is not sucrose, sugar recognition, anomer/linkage/D/L/stereo inference,
or general SMILES insertion. The separate legacy direct-glycosidic and verified-sucrose
actions were OASA-owned before retirement. Compact semantic tests are permanent evidence. A sealed installed
site passed the focused private/public suite (4 passed), and an independent public walkthrough
accepted blank/invalid inline accessible recovery, pointer-tool cancellation, occupied retry,
selection, Escape/tab-switch/close containment, Undo/Redo, save/reopen, and normal V2
receipt-only/no-marker installation. Wheel/site mechanics, OASA harness, offscreen focus,
screenshots, parser, visual, accessibility, occupancy, and walkthrough mechanics are disposable.

## Closure gate

M19 may close this ledger after independent review confirms that every supported row names one Rust session or
non-document native owner, every other row records a supported, known-defect, refusal, or
drop decision. M22 separately requires release-package validation that the ordinary product has
no retired Python chemistry backend, Python RDKit, or Tk runtime dependency. The
live differential workers are retired; accepted reports retain the migration
record. Repository inspection and installed-window walkthroughs may support that
release decision as disposable evidence; they are not permanent source-count gates.
