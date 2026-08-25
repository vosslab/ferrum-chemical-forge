# Changelog archive: 2026-08-13

This archive continues the history before [CHANGELOG.md](CHANGELOG.md). Earlier
history is in [CHANGELOG-2026-08b.md](CHANGELOG-2026-08b.md).

## 2026-08-13

### Additions and New Features

- Added the ordinary native `Chemistry -> Inspect Selected Molecule` route with a revision-bound Rust
  observation and paint-only recorder for authenticated render-plan inspection.
- Added the first closed native-SMILES prepared builder and the bounded native 17 peptide-template
  insertion route; both retain detached preparation, provenance, and one authenticated commit.
- Added direct Haworth/glycosidic migration layers: topology, local layout, fragment and depiction
  receipts, depth projection, direct insertion, and renderer profile. These are explicit M14/M16
  contracts rather than OASA fallback behavior.
- Added a developer-consented API example for explicit performance observation, with no product timing gate.
- Isolated ordinary M16 startup from OASA and established the native-first MainWindow/session lifecycle,
  empty-document baseline, Open CDML, Undo/Redo, atom number, bond-properties, and atom/bond deletion
  routes through Rust-owned document/session boundaries.
- Added M13 renderer foundations: fixed miter behavior, borrowed draw stream, checked neutral vector
  operations, `DocumentRenderPlanV1`, one `RenderObservationV1`, exact direct-root lowering, and
  whole-page composition. SVG, PDF, and PNG snapshot export use this backend path.
- Added bounded ABI-4 InChI import/export/InChIKey, multi-record SDF import, representable InChI native
  projection, and the first five plus clean-geometry Rust-owned Repair actions.
- Added revision-bound selected-atom rotation and whole-root transforms; native Qt consumes prepared
  Rust intents and authoritative projection rather than deriving durable document geometry.
- Added bounded standalone native Text display/editing, supported direct-root vectors/shapes/arrows,
  Wavy lines, rectangular/round brackets, selected presentation deletion, and source-order operations
  to bring forward, send back, or reverse selected slots.
- Added the Rust-owned paper catalog, document properties, physical-page projection, and seven native
  atom-mark toggles. Qt receives supplied geometry and glyph facts rather than interpreting CDML.
- Added the backend-only M10 preservation gate for structural CDML rewrite across the committed corpus.

### Behavior or Interface Changes

- Established the ordinary MainWindow as native first while retaining the compatibility host as an
  explicit OASA session island. Supported routes use `DocumentSession`; unsupported forms report
  typed issues rather than silently substituting legacy behavior.
- Split molecule/document root order from molecule-local order in render observations so molecules and
  direct-root artwork retain authored interleaving. Multi-segment vectors, shapes, arrows, Telex Plus,
  Wavy, and brackets consume Rust-issued geometry and appearance facts.
- Defined fixed M13 stroke, future PNG/PDF admission, and artifact-publication semantics without
  inventing defaults, DPI, allocation, byte, or timing promises.

### Fixes and Maintenance

- Corrected zero-area shape projection, Qt worker delivery through an owned QObject relay, M11/M12
  tracker status, capability-ledger identity claims, and obsolete native-wheel target instructions.
- Simplified `check_rust.sh` while retaining Cargo test and strict Clippy coverage; reduced the RDKit
  build profile to grounded, verified switches and removed fake or redundant CLI choices.
- Removed brittle partial-native-tab, private worker-wiring, parity-wrapper, subprocess-smoke, and
  exact-inventory tests. Kept deterministic behavior-focused coverage and disposable integration probes.
- Rotated 2026-08-11 and 2026-08-03 to `CHANGELOG-2026-08a.md` under the documented archive policy.

### Decisions and Failures

- Closed the pre-production third-party-plugin design as a product drop. Future extensibility requires
  explicit discovery, permission, versioning, lifecycle, and failure-containment ownership.
- M11/M12 remain in progress; M13 backend work is bounded and does not claim final PNG/PDF contracts.
  M16 full-session adoption and compatibility-host cutover remain open.
- Permanent tests use the `PYTEST_STYLE.md` checklist. Current-wheel, OASA-oracle, visual, pixel,
  byte, timing, network, exact-count, and private-wiring checks remain one-time evidence where useful.

### Developer Tests and Notes

- Rust, binding, and native-window semantic tests cover atomicity, provenance, history, selection,
  reopen, preservation, typed refusal, and projection. Wheel and public-window exercises were used as
  disposable integration evidence.
# Unreleased

* Added opaque direct-bond candidate admission receipts. Rust now validates complete existing
  and snapped new-endpoint candidates without reserving identifiers, tokens, or history, then
  redeems the receipt through one fenced atomic commit. The legacy preview API remains compatible,
  including foreign/stale precedence and `preview_mismatch` / `report_conflict`; PyO3 adds opaque
  admission handles with frozen semantic-refusal and commit-category surfaces.

* Registered Ferrum pointer actions with each owning `QMenu` before insertion,
  so direct menu lifecycle signals now bracket action handoff even when an
  offscreen popup Show event does not reach the application filter.

* Added the active autonomous plan for attaching one cyclohexane ring to one
  existing atom. It uses two bounded Rust topology experiments before selecting
  the closed C6 contract, then document, private PyO3, Qt, focused-proof, and
  one E2E packages. Ring fusion, bond attachment, generic templates, broad CML
  work, fixtures, and human/manual gates remain explicitly deferred.

* Registered Rust/PyO3 receipt-lifetime coverage for the private live SMARTS bridge. The focused
  API test now proves receipt-only retirement refuses the old opaque receipt while retaining the
  published plan for raw and selected reruns, and proves full retirement reports the closed
  `plan_not_published` reason.

* Added one-shot canvas capture for selected-molecule SMARTS queries. Qt now
  immediately converts a renderer point selection into a tab-private opaque
  Rust token, discards the generic selection, and runs selected SMARTS only
  from that token. Focus, Escape, right-click, tab, dock, and authoring-tool
  handoffs retire the temporary capture.

* Corrected the modeless Qt SMARTS dock so selected-molecule execution uses a
  one-shot tab-private capture/run boundary, errors accept only closed category
  enums, replay reaches native receipt refusal, and Escape first removes only
  transient highlight before retiring the full result run. Retirement failures
  now preserve and block the visible state rather than being swallowed.

* Strengthened the final installed-wheel SMARTS combined E2E receipt with exact
  named-CLI result semantics, manifest/wheel/installed native-closure equality,
  guarded reprojection retirement, and live-query Save As/Rust reopen/async GUI
  reopen coverage.

* Replaced the Qt SMARTS multi-row mock bridge with an isolated installed-wheel
  proof. It now exercises real PyO3 row redemption, one-use replay refusal,
  overlay replacement, and native receipt invalidation after rollback failure.

* Added installed-wheel SMARTS evidence for the guarded Qt
  `tab._session.observe_render(...)` proxy: it now proves that proxy publication
  returns a fresh valid observation while retiring an unread live receipt.

* Removed the obsolete public SMARTS query-origin DTO and added sealed-boundary
  coverage for the stateless protocol/schema, named CLI command, and live
  PyO3 receipt lifecycle. Custom typed Rust chemistry engines can now construct
  validated owned SMARTS match facts without receiving native adapter or wire access.

* Tightened the M4b typed SMARTS result construction contract. Custom engine
  rows now validate against the caller's target graph and requested cap before
  they become owned facts, including target bounds and fixed query-row arity.
  The private live-query PyO3 receipt test now refuses source-extension runs
  unless `FERRUM_SMARTS_SEALED_WHEEL_ROOT` identifies a fresh installed ABI-5
  bundle manifest.

* Routed every Ferrum Qt document render-plan publication through the private
  API-owned live SMARTS transaction. Initial construction, admitted async Open,
  refresh, accepted mutation, Undo/Redo, and Save now retire the Qt transient
  visual before private Rust retirement and plan publication; a missing or
  failing private boundary fails closed through typed tab recovery.

* Moved all authored reaction candidate construction, validation, complete-render admission, and one-use receipt commit authority from `ferrum-document` into `ferrum-document-render`. The document session now exposes no reaction-specific candidate or commit route: authored reactions use the renderer bridge and commit only through the generic complete-CDML compatibility transaction, while compatibility-loaded reaction records and their structural deletion guards remain lossless.

* Added the Rust-owned `reaction.create.v1` aggregate. It atomically creates a canonical direct-root reaction with ordered reactants, products, one arrow, optional conditions, and optional pluses. The bounded route validates durable direct-root kinds, rejects duplicate/cross-role/cross-reaction members, allocates collision-safe `rxn-N` IDs, preserves compatibility-loaded malformed records, and now prevents structural deletion of all referenced reaction roots.

* Corrected the compact Ferrum authoring ribbon's visible action language. Select Structure and all
  five vector tools now use required, shipped Ferrum icons while retaining their existing live
  QAction command owners, compact More Tools entries, accessibility metadata, and lifecycle.
  Narrow bond authoring now presents the readable `Next atom/bond defaults.` hint while assistive
  technology retains the full next-operation instruction.

* Corrected the Ferrum authoring ribbon's compact behavior: all live tools now share one
  optional-exclusive active-tool group, tool changes retire an active gesture through the existing
  cancellation authority, and the named accessible `More tools` menu retains the exact QAction
  instances for every authoring tool at narrow widths. Grid and snap controls remain direct.

* Started the renderer ownership inversion with a session-free
  `RenderDocumentModelV1` transfer schema and a one-way conversion from one
  accepted document observation. The model retains immutable render facts,
  source identity/order, molecule topology, presentation records, paper,
  drawing-standard, Telex profile, and diagnostics without CDML nodes, session,
  history, or toolkit ownership. Renderer lowerer migration remains in progress.

* Moved presentation-vector complete-render admission into the Rust document
  session. The session now retains the opaque renderer proof with its prepared
  capability; raw pending candidates, public preflight receipts, and direct raw
  vector commits are no longer available to Rust, Python, CLI, or Qt callers.
  Compatible CDML load and rewrite remain independent from authoring admission.

* Corrected Qt direct-Plus preview paint conversion: renderer RGB wire values
  now receive their one required `#` only at the Qt projection boundary.

* Replaced the direct Plus preview seam with a dedicated opaque Rust API facade.
  It derives the temporary Plus from the exact canonical insertion in a detached
  session and publishes only verified renderer layout, paint, and bounds; Qt
  can no longer submit caller-created appearance or enter Plus through the
  generic presentation-gesture API.

* Corrected Rust structural-interaction geometry: ordinary bond hits and full
  marquee containment now use rendered line stroke width rather than endpoint
  rectangles. Rendered path-only bond depictions remain visible as typed
  `DisplayOnly` structural targets and refuse authoring atomically instead of
  disappearing from interaction selection.
* Added Rust-authoritative direct atom/bond interaction observations and opaque
  atomic structural deletion handles for the Ferrum Qt path.

- Added an interruption checkpoint for M4b SMARTS dock integration, including accepted backend evidence, current Qt work, selected-query requirements, and the live-binding ownership conflict.

* Corrected the SMARTS dock Find-state lifecycle. One centralized Qt refresh rule now reacts to raw text edits, source-mode changes, selected-molecule availability, tab readiness, busy dispatch, and blocked retirement; whitespace-only input cannot begin a query.
- Added the separate `Attach Cyclohexane Ring` Qt action. It captures an eligible atom, paints
  only copied finite Rust C6 preview geometry, commits one opaque receipt on release, and retires
  the receipt and overlay on cancellation or stale/refused gestures without changing the detached
  `Insert Cyclohexane Ring` action.
# Cyclohexane attachment E2E

- Added failure-only, value-safe diagnostics to the isolated Qt CML new-document
  E2E. A failed valid queue completion now records only fixture labels, public
  completion success, scheduled-start acceptance, typed-refusal presentation
  facts, and pending/tab counts; it never includes CML or arbitrary file paths.

- Corrected the isolated C6 E2E's final ineligible-anchor exercise to use the
  real press-drag-release contract, so it reaches the intentional drag-time
  eligibility refusal without relaxing its typed, mutation-free terminal checks.

- Corrected the attached-cyclohexane root E2E to require generic history
  retirement after both undo and redo, then rearm C6 through its canonical
  visible-menu QAction before the Escape-cancel drag.

- Corrected the attached-cyclohexane root E2E to assert the intended sticky
  selected-tool state after a successful commit, then begin its Escape-cancel
  drag directly instead of redundantly toggling the shared QAction.

- Added passive pre-trigger ownership fences around snapshot and coordinate setup
  in the isolated C6 E2E. Each failure now identifies the exact setup phase and
  attach-action signal delta before canonical QAction activation.

- Strengthened the isolated attached-cyclohexane E2E startup invariant. Before
  menu discovery it now requires no line intent, active tool mode, or checked
  authoring action, and its failure payload records the real intent tool plus
  copied start/pending/preview presence and passive attach-action/mode traces.

- Corrected the isolated attached-cyclohexane E2E popup observation window to
  wait through the existing bounded 250 ms popup-handoff watchdog plus one
  queued-turn margin. Its 300 ms event-pumping deadline now exits as soon as
  the shared attach action and attach mode are both active, and records popup
  ownership facts on timeout.

- Corrected the Escape failure diagnostic to include the existing bridge facts,
  so a missing native preview reports its actual state instead of raising a
  diagnostic `TypeError`.

- Added one isolated offscreen root E2E for the native-wheel attached-cyclohexane workflow.

- Corrected the C6-only implicit atom picker to pass Rust-issued opaque atom IDs
  into the attachment bridge rather than authored CDML source IDs.
