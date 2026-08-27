# Ferrum migration account-switch handoff

> Historical snapshot: this report records the 2026-08-14/15 migration checkpoint.
> Its CDXML refusal statements describe that dated state, not the current product contract.
> See [m2_cdxml_simple_molecule_import_v1.md](../decisions/m2_cdxml_simple_molecule_import_v1.md)
> and [ferrum_qt_capability_matrix.md](../audits/ferrum_qt_capability_matrix.md) for current scope.

## Purpose

This is the restart point for the 2026-08-14 manager run. The worktree is heavily dirty and
user-owned. Do not reset, clean, stage, or rewrite unrelated changes. The historical collaboration
tree is operationally quarantined; the collaboration API has no delete primitive, so use only
fresh task names after the account switch.

The active goal remains the complete Rust replacement of OASA, Ferrum-Qt adoption, CLI and Python
contract freeze, packaging, and final capability closure. The goal is not complete.

## 2026-08-15 M15/M16 closure update

M15 and M16 are accepted and closed at their pre-production boundaries. The retained M15
workflows are strict peptide inspection, the deliberately bounded native peptide-template
insertion profile, the Rust-owned linear-form planner and document transaction, and the native
geometry-repair envelope. They remain narrow, source-backed contracts rather than claims of
generic peptide, template, or OASA parity. Unused compact-sugar parsing/semantic/wire code and
unused biomolecule, functional-group, sugar-name, and catalog-provenance code were removed;
the Ferrum-owned periodic display catalog stays because the ordinary element picker uses it.
Known-group expansion, substructure search, generic sugar entry, system/biomolecule templates,
and functional-group catalogs are explicit pre-production drops unless a separately designed
native workflow adopts them.

The M16 compatibility-host checkpoint rereview is ACCEPT. The explicit
`LegacyCompatibilityMainWindow`, its OASA session/action/mode/worker/codec/projection island,
and both production OASA dependency declarations are retired. `ferrum-qt` opens one ordinary
Rust-native `MainWindow`; historical OASA/BKChem material remains provenance or an isolated
oracle boundary, never a desktop fallback.

The two checkpoint P2 repairs are accepted: native save destination ownership now consults only
the live Rust-native tab map, with the retired `_sessions`/legacy-path branch removed; and the
single retained CDXML refusal test checks actionable converter and `.cdml` recovery plus
nonmutation, without a warning-title snapshot. Neither repair restored a host branch or added a
format matrix, timing gate, or fixture-heavy topology test.

Supported ordinary routes remain bounded native CDML and decoded-CD-SVG Open, Save/Save As,
`Recovery Export CDML...`, accepted editing/history/authoring, bounded chemistry inputs, and
SVG/PDF/transparent-PNG artifact export. CDXML, CML, `.cdsvg`, `.svgz`, and compressed CDML are
pre-read actionable refusals that preserve the active document. PubChem, system/biomolecule
templates, unported legacy modes, repair variants outside the retained envelope, properties, and
clipboard variants are explicit pre-production drops. This does not freeze public Python or CLI
contracts.

Permanent evidence is the existing compact offline semantic coverage for the retained native
workflows, plus one representative CDXML refusal/nonmutation behavior test. Current-tree and
package inventories, OASA-absent build/site launch, and the accepted installed ordinary-window
walkthrough are disposable rebuild evidence. The walkthrough observed ordinary startup without
OASA or compatibility UI, public CDML Open, Radical edit semantic Undo/Redo, retained
Recent/Export/Recovery labels, and real CDXML modal nonmutation. The focused installed subset and
prior artifact walkthrough separately support save/reopen and artifact publication. The offscreen
executor stopped progress during later dialog-backed work without a product error; this is a
harness limitation, not a product gate.

M17, M18, and M19 are independently accepted and complete.
M20 source implementation is accepted for its proposed macOS arm64/CPython 3.12 target, with two
first-party Python wheels, explicit local Cargo/Qt wheelhouses, a scrubbed no-index E2E route, and
an LGPL relink contract. M22 source work is also accepted: the source-release helper understands
the real dual-license archive, the native wheel stages its notice roles, and a predicate inventory
awaits final artifacts. The real target artifact evidence remains pending because the external
wheelhouses are unavailable, so M20 and M22 remain open and no consumer release is claimed. The
Rust `ferrum` executable remains a separate Cargo-installed CLI. M21 is nonblocking WASM contract
proof.

## 2026-08-15 M17/M18 implementation checkpoint

M17/M18 are accepted and complete. Their implementation, checkpoint rereview, and fresh installed
wheel/site evidence establish one closed, stateless JSON V1 with exactly
`document.inspect`, `document.validate`, `document.rewrite`, and
`document.render_artifact`. Requests are independently bounded before CLI transport allocation,
Python copying, and JSON parsing by a derived envelope budget: the established CDML source profile,
worst-case JSON escaping, and a small framing/request-ID allowance. This is resource safety, not a
timing, corpus, pixel, or byte-identity target; CDML admission and base64 completion keep their
separate existing bounds.

The public Python additions are `execute_operation_v1(str) -> str`,
`operation_protocol_schema_v1() -> str`, and categorized `OperationProtocolErrorV1`.
Pre-envelope Python categories are `invalid_json`, `resource_limit`, and
`execution_unavailable`; decodable domain and version refusals are response data. The frozen CLI
family is `ferrum protocol schema` and `ferrum protocol run INPUT [--output OUTPUT]`. It writes
one JSON envelope, keeps diagnostics separate, safely publishes only an explicitly named output,
and uses exit 0 for completed success/refusal, 1 for pre-envelope/confirmed publication failure,
2 for usage, and 3 for possibly-published output. M19 retired provisional root CLI families;
their private and Ferrum-Qt-native seams remain separate from the public command contract.

V1 adds no batch, network, session, receipt, Qt, path-bearing protocol payload, adapter discovery,
chemistry conversion, CD-SVG/compressed input, selection/root export, templates, clipboard,
recovery copy, document mutation, or render-observation operation. Permanent coverage is compact,
offline Rust/Python behavior. Real CLI, generator, wheel/schema-resource, package, and installed
walkthrough checks are E2E or disposable evidence, not timing/count/byte/pixel/network gates.

## Continuation progress

- Ordinary native `File -> Export...` SVG/PDF/PNG adoption is accepted. Qt captures the
  active tab's immutable session observation, revision/digest, and opaque local-origin token;
  after a destination chooser and first current/idle fence, a QThread asks private Rust code to
  create a complete SVG, vector PDF, or transparent one-pixel-per-page-point PNG receipt. A
  second fence authorizes descriptor-relative Rust publication only while the same tab remains
  current. Complete-plan exclusions refuse rather than partially export. Local CDML and decoded
  CD-SVG origins retain a live Rust descriptor, rejecting their original source or an observed
  hard-link alias as a destination. Cancel and stale work are non-mutating; confirmed,
  directory-entry-unconfirmed, not-started/rejected, and possibly-published results retain
  distinct recovery. Open, Open in Current Tab, export, and close have reciprocal busy
  containment. Exactly three private-bridge semantic tests are permanent; wheel/site,
  decoder/signature/dimensions, visual/a11y, busy-race, and installed-window walkthroughs are
  disposable. The retired compatibility export cascade is historical evidence only; it is not a
  supported CD-SVG export/round-trip route. Public PyO3/CLI/wire adoption and M19
  capability-matrix closure are separate.

- Direct-Glycosidic Haworth V1 is accepted. The ordinary native
  `Chemistry -> Insert Direct-Glycosidic Haworth...` dialog starts empty and accepts only
  structural SMILES for two vertex-disjoint five- or six-member C/O rings joined by one
  exterior degree-two oxygen: a neutral, nonaromatic, single-bond-only 11--13-atom graph.
  It is a constrained drawing profile, not sucrose or another named sugar, anomer/linkage/D/L
  assignment, stereochemistry inference, or general SMILES insertion. Rust owns admission,
  graph/one-use receipt, durable drawing facts, IDs, CDML, history, selection, save/reopen,
  and normal V2 rendering. Private PyO3 carries only typed preparation, frozen preview batches,
  and commit; Qt owns accessible text/recovery, captured tab/revision/digest, one shared snap,
  receipt-only preview, and authoritative installation. Occupancy and cancel/stale/busy/tab/close
  containment preserve durable state. SMILES, names, parser coordinates, UI state, and preferences
  never enter CDML/QSettings; no OASA, public `.pyi`, CLI, wire, or composite path is added.
  Compact semantic tests are permanent evidence. A sealed installed site passed the focused
  private/public suite (4 passed), and an independent public walkthrough accepted blank/invalid
  inline accessible recovery, pointer-tool cancellation, occupied retry, selection,
  Escape/tab-switch/close containment, Undo/Redo, save/reopen, and normal V2 receipt-only/no-marker
  installation. Wheel/site mechanics, OASA harness, offscreen focus, screenshots, parser, visual,
  accessibility, occupancy, and walkthrough mechanics are disposable. FQ-013 remains partial:
  legacy direct and verified-sucrose routes are explicit pre-production drops unless a separately
  designed Rust-owned workflow is approved.

- Native decoded CD-SVG Open V1 is implemented at the ordinary Rust-owned Open boundary.
  A requested local `.svg` must be decoded UTF-8 SVG with exactly one canonical embedded CDML
  descendant. Rust contains the wrapper and normalized payload in separate complete envelopes,
  admits only the payload, and exposes a private one-use receipt with an equality-only descriptor
  token and closed source kind. The SVG wrapper is discarded: it contributes no document fact and
  is never rendered, fetched, persisted, or saved. Qt reuses ordinary/bootstrap/current-tab/Recent
  lifecycle ownership, keeps source provenance separate from `file_path`, and opens CD-SVG tabs
  clean and Save-As-only to `.cdml`. A later CDML publication retains the original descriptor token
  for duplicate activation. Compact Rust, binding, and public Qt semantic tests are permanent;
  wheel/site, chooser, accessibility, visual, corpus, and timing probes are disposable. The independent
  checkpoint rereview and final installed public walkthrough are accepted. `.cdsvg`, `.svgz`,
  compression, sniffing, wrapper round trip/export, public API/CLI/wire adoption, OASA fallback,
  and other document classes remain separate work. M19 remains open.

- The bounded FQ-010 `Chemistry -> Check Bond Capacity...` route is accepted. It is a read-only selected-complete-root
  neutral capacity diagnostic, not a chemical-validity, valence, or oxidation-state result.
  Rust owns the H/B/C/N/O/F/Cl/Br/I table, closed grammar, explicit-H plus incident-order
  arithmetic, immutable provenance receipt, and document authentication; private PyO3 transports
  that receipt and Qt owns worker containment plus the selectable dialog. Eligible roots have
  absent/zero formal charge, no aromatic fact, ordinary atoms, and single/double/triple bonds;
  authored `valency`, `multiplicity`, and `free_sites` produce whole-root Not checked outcomes.
  Assessed atoms report Within Capacity or Exceeds Capacity while retaining authored charge and
  explicit-H presence. Bond depiction is ignored. The route changes no CDML, history, selection,
  Properties, QSettings, public Python, CLI, wire, or OASA contract. Compact semantic tests are
  permanent, including a mixed-root public regression. Accepted public real-worker evidence09 covers
  supported/no-excess/finding/Not checked results, authored-presence display for every assessed atom
  in a mixed excess root, multi-root order, depiction independence, lifecycle/nonmutation, and
  accessibility. Fresh wheel/site, visual, OASA, and timing probes remain disposable.

- Explicit Fragment V1 is accepted. `Chemistry -> Create
  Fragment...` creates only a named `type="explicit"` record inside one durable direct-root molecule.
  Rust validates direct ordinary atom/bond membership, closes selected bonds over endpoints, emits
  molecule-source order, allocates a collision-safe tentative ID, and commits one revision-bound
  history transition. Labels are outer-whitespace trimmed, plain, nonblank, and may duplicate;
  disconnected metadata members are valid. Qt captures source tab/revision/digest/root/selection for
  its one-field dialog, preserves selection and chemistry, and provides a read-only View that lists
  exact supported records plus one retained-import-metadata notice. Rust/PyO3 are private ownership;
  V1 adds no OASA, QSettings, public `.pyi`, CLI, or wire surface. Retired-host transfer is not a
  future route. Clipboard, groups, templates, inference, cross-molecule records, type selection,
  delete/rename/member editing, and highlight remain separate future contracts. Compact semantic
  tests are intended permanent evidence;
  fresh wheel/site, screenshots, visual, keyboard/accessibility, corpus, and timing probes are
  disposable. Independent rereview accepted the View-lifecycle and typed-error repairs; the installed
  public walkthrough accepted endpoint closure/source order, duplicate labels, blank retry/Cancel/
  stale containment, retained notice, View lifecycle, undo/redo, save/reopen, and ownership hygiene.

- The private linear-form PyO3 checkpoint and ordinary native Qt action are independently
  accepted. Resource and allocation failures remain operation-specific private errors.
- Ordinary Ferrum-Qt now owns FQ-016 save, secure scan, explicit refresh, Rust inspection, and
  finite-anchor placement through a dedicated document/API/private-PyO3 contract. Optional Paper
  and Standard records remain inspection context and only the molecule is inserted. Application
  preferences remain QSettings state and never enter CDML.
- FQ-020 now includes application-owned paper-local hex-grid visibility and snapping. The disposable
  overlay uses the bounded Rust geometry bridge and renderer-owned paper rectangle. Checked View actions,
  a shared toolbar client, Preferences checkboxes, and `Ctrl+Shift+G` project QSettings choices to current
  tabs, future tabs, and scene replacement. `FerrumNativeGraphicsView` owns the single finite
  authored-point policy and delegates nearest-lattice math at the existing grid spacing to the Rust bridge.
  Free atoms, template centroid anchors, empty-space bonded-atom endpoints, moved-atom targets, Wavy
  endpoints, and rectangular/round bracket corners use the same resolved point for previews and commits.
  Complete-root translation now observes a private immutable Rust `TopLevelTranslationAnchorV1` at press:
  canonical selectors, exact revision/digest, and a finite lower-left authored-coordinate union anchor.
  The gesture captures the existing snap boolean; enabled resolution is `snap(anchor + raw_delta) - anchor`,
  disabled resolution is `raw_delta`, and the same rigid delta drives preview and Rust commit. Projected
  bounds remain overlay-only. Existing-atom joins remain exact and rotation stays angle input. The receipt
  remains outside CDML, history, and preferences. Selection-set or revision/digest stale/race facts use
  `Native Move Complete Roots Stale` and the recovery instruction to select complete roots and drag again;
  ordinary validation and nonfinite input retain Error or Unavailable. The preference remains outside CDML,
  Rust document ownership, history, save state, and selection.
  A fresh disposable installed-site walkthrough accepted active-drag partial-selection and
  intervening revision/digest paths: each preserved the current authoritative state and selection,
  retired preview/tool, and displayed the stale recovery category.
- The ordinary Rust-native Ferrum root now has a second-row `Editing Tools` toolbar with a stable Qt
  workspace identity, visible category header, and `View -> Editing Tools` visibility control.
  It reuses the exact existing native actions for Add Atom, Draw Bond, Wavy,
  rectangular/round brackets, Move Atom, Rotate Selected Atoms, and Move Complete Roots,
  plus a window-owned shared `Cancel Tool` Escape recovery action. Layout, icons,
  accessibility, theme projection, and overflow are UI clients only; action callbacks,
  gesture intents, prerequisites, document/session/history/selection, Rust mutation,
  snap policy, and transform exclusions remain their current owners. Cancel Tool composes
  established cancellation boundaries and
  preserves document and selection. This ports the useful BKChem-Qt discoverability pattern while
  retaining only the useful historical interface concepts as evidence, not a
  legacy toolbar, mode-schema, or mode-manager product route.
  Permanent semantic tests cover one toolbar bond gesture, Escape preservation, Cancel Tool
  preservation, and remembered user-hidden state. Wide/narrow visual, overflow, icon/grouping/
  accessibility walkthrough, workspace-byte, timing, count, pixel, and installed-wheel UI checks are
  one-time evidence.
- The ordinary native root now owns the bounded Next Drawing element/order slice. Labelled
  `Next atom:` and `Next bond:` controls retain C/single application/QSettings defaults across
  ordinary windows. The Rust periodic catalog supplies suggestions and conventional spelling, while
  valid ASCII-letter plain and pseudo-atom names remain available to the existing Rust validator.
  Add Atom freezes the element for one click; Draw Bond freezes element/order at mouse press, then
  uses the next effective choices on the next gesture. Existing-atom joins use explicit durable IDs;
  empty-space endpoints create the captured element and a normal single/double/triple CDML bond at
  the shared snap target. Rust owns candidate validation, identity, refusal, history, projection,
  selection restoration, and save/reopen behavior. These personal preferences never enter CDML,
  `<standard>`, Rust document/history/dirty/save state, or selection. The same client now has a
  shared-live-window QSettings `Next presentation:` choice. Rust's closed
  `DocumentBondPresentationV1` writes Normal `n1`/`n2`/`n3`, directed solid-wedge `w1`, or
  directed hashed-wedge `h1`; press/start is the narrow tip and release/new atom is the wide base.
  Draw Bond captures element, order, and presentation at press. The V2 renderer owns filled-wedge
  paths and finite widening hashed lines; Qt consumes its preview and committed operations. Focused
  semantic tests are permanent; the keyboard,
  visual, accessibility, overflow, and installed-wheel walkthrough remains disposable evidence. A
  fresh independent current-source Escape walkthrough is accepted: inactive and Draw Bond-armed
  editor Escape restore the visible/effective O value; the active route composes shared cancellation
  and preserves snapshot/selection. The disposable probe and its cache scan are clean.
  `Edit -> Next Drawing...` is the standard MainWindow-owned compact labelled route to the same
  shared model at narrow widths; it restores accepted input on focused Escape and composes Cancel
  Tool for an active gesture. Directed feedback names the Single presentation, tip-to-base drag,
  and captured empty-space element. Width and toolbar-extension observations remain disposable.
- The ordinary native root now owns one closed detached regular-ring authoring family. Private
  Rust `DetachedRegularRingInsertionV1` admits sizes 3 through 8 at a finite centre with the
  canonical 40-point drawing side length and fixed flat-top clockwise y-down geometry. It writes
  only ordinary C atoms, points, and normal `n1` bonds; a full candidate receives Rust IDs,
  prepared-token provenance, one history transition, and authoritative projection/selection.
  CDML has no ring metadata, template, preference, or UI fact. The product exposes one shared
  `Insert Cyclohexane Ring` Edit/Editing Tools action: an empty-page press resolves the centre
  once with the shared snap policy, displays only exact Rust receipt vertices, and commits that
  receipt on release. An atom hit refuses with empty-page guidance. Escape, Cancel Tool, tab
  lifecycle, and stale revision/digest preservation use the shared gesture fence. Corrected
  radius geometry preserves the source side length, and the Qt route reads the established
  drawing-length source. Compact semantic Rust, private-binding, and native-action behavior
  tests are permanent; wheel/site, screenshots, narrow-window, accessibility, and visual probes
  are disposable. UI sizes 3--5/7--8, fusion/attachment, heteroatoms/aromaticity,
  orientation/rotation, and preferences remain future contracts.
- The ordinary native root now owns one closed FQ-012 standalone Haworth family. `Edit -> Insert
  Haworth Ring...`, also in Editing Tools, offers exactly alpha/beta D-glucopyranose and alpha/beta
  D-glucofuranose with readable form/anomer choices. Rust owns literal C6O6 recipes, finite local
  geometry, durable IDs, CDML, history, selection, a revision/digest-bound one-use receipt, and normal
  Render Plan V2 output. The chemistry correction is durable: pyranose closes
  `O5-C1-C2-C3-C4-C5`; furanose closes `O4-C1-C2-C3-C4` and continues as `C4-C5-C6`, not an O5
  furanose closure. The common front edge is `q1` C2-C3, shoulders are directed `w1` C1->C2 and
  C4->C3, and remaining ring edges are `n1`; the ordinary V2 route carries the needed round-cap and
  front-layer facts. Qt owns only the chooser, a single captured shared-snap anchor, and the exact
  Rust preview. The receipt is private and no code, preference, or UI metadata enters CDML/QSettings.
  Compact recipe/transaction/render/binding/public-action semantics are permanent. IUPAC/PubChem/OASA
  comparison, wheel/site, screenshots, and visual/accessibility evidence are disposable. Independent
  current-source/installed-site evidence is accepted: the public chooser placed all four variants,
  exercised snap/preview/commit, and restored semantic CDML/history through public tab undo/redo with
  expected revision advancement. The focused public rewalk confirms that an authoritative atom or bond
  at the raw or snapped placement point preserves document/selection and retains the armed intent for a
  later empty-page click; Cancel and stale paths also preserve state. Save/reopen retained the inserted
  molecules. It confirmed no Haworth UI metadata in CDML/QSettings and clean clipboard/window teardown.
  Generic codes or catalogs, other sugars, attachment/fusion, rotation/reflow, repeated placement, and
  general stereo inference remain separate work.
- Slice A of ordinary native CDML Open is accepted. Rust derives an opaque equality-only origin
  token from the admitted regular descriptor and carries it with the session and authenticated
  observation in the private one-use PyO3 receipt. Qt retains the token only for live-tab identity:
  a hard-link alias activates the existing ordinary native tab after admission. Immutable Open
  intents replace only the explicitly marked clean revision-zero bootstrap `Untitled` tab after
  preparation, receipt authentication, revision/digest/canvas-idle revalidation, full replacement
  construction, and atomic swap. Other interactive Opens and all launch paths use `NewTab`.
  Busy source tabs retain current focus and their visible preview while the admitted tab installs in
  the background; normal new tabs activate only when the captured focus remains current. Failed,
  cancelled, shutdown, invalid, stale, and busy delivery paths preserve current authoritative work.
  The token never enters CDML, document/session history, preferences, or a public/cross-process
  contract. Compact semantic receipt/lifecycle tests are permanent; fresh wheel/site, visual,
  route-inventory, timing, and deterministic-race observations are disposable evidence. Slice B
  Recent Files/QSettings and explicit populated-tab replacement with Save/Replace/Cancel recovery
  remain separate work; compatibility-host cutover is complete.
- Slice B native Recent Files is accepted with compact semantic test evidence and an independent
  fresh-window walkthrough. `FerrumNativeRecentFilesV1` owns versioned QSettings-only
  lexical normalized absolute display paths without symlink resolution. Confirmed native Open,
  descriptor-token activation, and Save promote a path; File rebuilds its cascade with parent-qualified
  duplicate labels and full-path help. Recent actions force ordinary `NewTab`; descriptor tokens retain
  live-tab duplicate authority. Rust-confirmed unavailable/nonregular sources offer Keep by default or
  explicit Remove, and Clear changes settings only. Capacity is tunable usable-menu policy rather than
  an acceptance count. Recent state remains outside CDML, history, dirty/save state, selection,
  receipts, diagnostics, and OASA. Preserve accepted Slice A busy/focus behavior. Explicit populated-tab
  replacement/recovery remains a later slice. The ordinary startup seam
  initializes the recent owner after `QMainWindow` construction and before File actions, retaining the
  stable cascade. Missing read/nonregular source-policy errors show `File Not Available` with Keep
  (default) or Remove before their generic typed failure. The fresh-product test clicks actual
  Keep/Remove and a valid entry; offscreen physical Remove-click delivery remains disposable evidence.
- The explicit ordinary-native populated-tab replacement slice is accepted. `File -> Open in Current Tab...` is the exact accessible command
  with `Ctrl+Shift+O`; it is available only for a selected, registered, idle native target and tells the
  author to finish or cancel active canvas or target-owned native work. A shared exact-target operation
  predicate controls action reachability, capture, and revalidation while Open retains its separate intent.
  It captures an immutable target fence, then Rust
  prepares and Qt authenticates the source before any destructive choice. A matching descriptor token
  activates the existing ordinary tab and preserves the target. A clean saved populated target fully
  constructs and atomically swaps in place without redundant confirmation. A dirty target alone offers
  Save (default), Replace, and Cancel after admission: named Save reuses native publication, unnamed Save
  uses Save As, and a fresh post-save fence must pass before swapping. Stale/busy/close/cancel/admission/
  save failure preserves target selection, tool, preview, and focus; the command never silently falls back
  to NewTab. Successful replacement retires the old owner only after new-tab registration and starts with
  the new selection/focus. Recent remains confirmed-only forced-NewTab composition. The worker `finished`
  relay defers during the modal decision so nested Qt delivery cannot retire the current intent prematurely.
  Six compact semantic UI/queue behaviors plus one real public-worker disable/preserve/re-enable outcome
  are permanent. Independent evidence07 accepted a fresh 21-case lifecycle walkthrough, including public
  active-tool and target-work disable/re-enable guidance, stale and visible Cancel preservation/no-NewTab,
  clean/dirty Save/Replace/Cancel/Save As, hard-link activation, ordinary Open, and Recent. Accessibility,
  keyboard, wheel/site, race, source, and visual probes remain disposable.
- Historical note (superseded on 2026-08-15): this handoff previously described an explicit
  compatibility host and OASA insertion session. That host, its session/action/mode/worker/codec/
  projection island, and both production OASA dependency declarations are now retired. There is
  one ordinary Rust-native `MainWindow`; `window_native_files.py`, the arrow and geometric
  dialogs, user-template components, themes, and icon resources remain ordinary native owners.
  Unsupported historical families are explicit pre-production drops or actionable refusals, not
  a compatibility fallback. The permanent suite retains compact native semantic behavior and one
  representative CDXML refusal/nonmutation behavior test. Source/package scans, OASA-absent
  installed-site launch, and walkthroughs are disposable evidence. Evidence12's offscreen runner
  stopped reporting progress during later dialog-backed work without a traceback or product error;
  that harness limitation establishes neither a product defect nor a gate. The focused installed
  subset and prior artifact walkthrough separately cover save/reopen and one artifact publication.
- Active molecule-plan production and acceptance now use only `ferrum-render-plan-v2`.
  `RenderPlanV2`, `RenderBatchV2`, `RenderOperationV2`, and
  `DocumentMoleculeRenderPlanV2` carry a neutral finite `MoveTo`/`LineTo`/`CubicTo`/
  `Close` path with explicit stroke, fill, and z. Rust owns validation and common-stream
  lowering for SVG, PNG, PDF, bounds, and composite recording; private PyO3 and Qt consume
  received facts, and Qt only copies and paints them. `RenderObservationV1` remains the
  revision-bound document/projection receipt envelope, not a legacy molecule-plan schema.
  Semantic Rust, PyO3, and Qt tests cover the V2 boundary. The final self-contained wheel/site
  and broad visual/artifact checks are disposable evidence, with no byte, pixel, timing, or
  count gate. The subsequent Rust-first slice now uses that foundation for ordinary `w1`/`h1`
  depiction, bond-style creation, and the Next presentation UI; `w2`/`w3`/`h2`/`h3` and all other
  styles remain separate contracts.

## Decisions fixed in this run

- M19 retires provisional root CLI families. M17/M18 freeze `ferrum protocol schema/run` as the
  public command contract; native desktop and private-extension seams keep their own bounded
  ownership rather than becoming CLI aliases.
- M16's single-document-authority classification is closed: retained supported document paths use
  Rust `DocumentSession`, and historical routes have an explicit supported, refusal, or
  pre-production-drop disposition. M17/M18 now own the subsequent interface freeze.
- The compatibility host is retired. Keep unsupported historical workflows as explicit
  refusals or drops until a separately designed Rust-owned workflow is approved.
- Pre-release persistence intentionally uses `Ferrum` / `Ferrum-Qt` QSettings and
  `~/.ferrum/templates`. There is no BKChem preference or template migration promise because the
  product has no production users. Historical provenance and real internal compatibility IDs stay.
- Native `linear-form.convert` is accepted. Its named `linear-form-direction-v1` contract starts at
  the lower durable source-order endpoint, an intentional persistent-CDML divergence from OASA's
  `(x, y, id)` endpoint ordering. The completed FQ-020 point-policy slice makes new and moved authored
  points consistent without treating joins, angular rotation, or rigid root translation as the same input.
- The native Haworth furanose correction is source-backed rather than legacy-compatible: use the O4
  closure and C4-C5-C6 exocyclic chain. IUPAC's carbohydrate nomenclature and Blue Book define the
  ring-size/anomeric terminology, and PubChem corroborates D-glucofuranose as a D-glucose furanose
  form. The references are recorded in the V3 plan; OASA remains disposable semantic evidence only.

## Accepted work

The following linear-form layers are implemented and independently accepted:

1. Pure `ferrum-domain` planner: source-order direction, simple-path validation, fixed 10-point
   geometry, exterior-component translation, selected-hydrogen facts, and typed resource errors.
2. Collision-safe fragment ID allocator: opaque-ID checks, typed exhaustion, and tentative copied
   sequence state which does not advance the session until commit.
3. Typed-document adapter: direct-root extraction, canonical metadata repair/new-ID classification,
   reverse-owned record repair, mark and atom movement, z/opaque preservation, and fallible
   retirement/writer paths.
4. `DocumentSession` transaction: one-use pending receipt, precomputed observation, fallible token
   issuance, allocation-free post-consumption commit tail, deferred ID installation, no-op behavior,
   history, undo/redo, and save/reopen semantics.
5. Public in-process Rust API: revision/digest/direct-root authentication and immediate prepare to
   commit with closed changed/no-change results. It adds no CLI, wire, serde, or stable Python API.

Primary receipts are under
`/private/tmp/ferrum-manager-20260814-next-migration.B5h9nb/`:

- `design_native_linear_form_v1.d7e2.report.md`
- `review_native_linear_form_v1_design.83af.report.md`
- `linear_form_direction_oracle.5aa1.report.md`
- `implement_linear_form_domain_v1.114c.report.md`
- `review_linear_form_domain_v1.0d8f.report.md`
- `finish_linear_form_document_v1.61f0.report.md`
- `review_fragment_id_allocator_v1.51a2.report.md`
- `implement_linear_form_document_adapter_v1.3c7b.report.md`
- `review_linear_form_document_adapter_v1.1e6d.report.md`
- `implement_linear_form_session_v1.885e.report.md`
- `review_linear_form_session_v1.d3f4.report.md`
- `implement_linear_form_api_v1.9d0a.report.md`
- `review_linear_form_api_v1.287b.report.md`

## Completed checkpoint evidence

- The linear-form private PyO3 resource mapping was remediated and independently accepted. Its
  ordinary Qt action was also independently accepted. Both remain private runtime plumbing outside
  `.pyi`, CLI, wire, and serde surfaces.
- FQ-016 has fresh root-workspace Rust tests, strict Clippy, rustdoc, separate PyO3 checks, and a
  provenance-matched installed wheel. The focused installed-binding/native/compatibility suite
  passes 71 tests with one intentional compatibility-host skip.
- Current-source focused semantic tests pass for the native grid, Preferences, and authored-point snap
  boundary. Offscreen menu/grid/gesture-preview reviews and the wheel walkthrough are disposable
  implementation evidence.
- Repository ASCII, Markdown-link, Pyflakes, and source-limit checks pass 2776 cases. The only
  remaining hygiene failure is the pre-existing over-limit `docs/CHANGELOG.md` day-history file.
- No Python bytecode caches were created anywhere in the repository.
- The Haworth private binding, normal V2 rendering, public ordinary chooser/canvas route, and the
  native clipboard teardown repair have current focused evidence. The independent current-source/
  installed-site Haworth walkthrough is accepted as disposable integration evidence; no visual or wheel
  observation is a permanent gate.

## Next restart sequence

1. Read the repository rules, current capability matrix, OASA ownership ledger, and this handoff.
2. Preserve the dirty worktree as user-owned and verify exact current diffs before adding a slice.
3. FQ-013 Direct-Glycosidic Haworth V1 is accepted; keep its legacy/OASA direct and
   verified-sucrose actions separate. Continue FQ-020 with other drawing-gesture and
   shortcut preferences. Keep directed
   `w2`/`w3`/`h2`/`h3` and all other bond presentations for source-backed render contracts.
   The Rust-authored complete-root anchor contract is complete; continue other drawing-gesture and
   shortcut preferences through equally explicit native owners. The detached regular-ring V1
   substrate exists, while its unexposed sizes, fusion/attachment, heteroatom/aromaticity,
   orientation/rotation, and preference choices still need separate contracts.
4. Complete M19's independent closure review from the accepted M17/M18 protocol boundary. Keep
   isolated oracle provenance separate from product runtime; the retired host is not a fallback.
5. Keep permanent tests compact, offline, semantic, and aligned with `docs/PYTEST_STYLE.md`; use a
   disposable current-wheel walkthrough for visual or installed-source evidence.
6. Keep the matrix and ownership ledger aligned with supported routes, explicit refusals, and
   pre-production drops while retaining M19/M20/M22 status truthfully. M21 remains nonblocking
   WASM contract proof.

## Other audit receipts

The same private report directory contains the current OASA ownership, M15/M16 closure, M17,
CLI, frontend-drift, and packaging audits. Their important conclusions are reflected above. M15
and M16 are closed at their documented pre-production boundaries. M17/M18 are complete; M19
implementation awaits independent closure review; M20 and M22 source mechanisms are accepted but
their target artifacts and human release review remain pending. M21 is nonblocking contract proof. None are
blocked by a live compatibility host. OASA is absent from production dependency declarations;
isolated oracle use remains provenance, not a runtime dependency.
