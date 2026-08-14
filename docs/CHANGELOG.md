## 2026-08-14

### Documentation

- Added a repository-local account-switch handoff for the active migration run, including accepted
  linear-form layers, the unaccepted PyO3 checkpoint, fixed architecture decisions, and the exact
  restart sequence. This records status only and does not claim M15, M16, M17, or M18 completion.

- Reconciled current Ferrum identity documentation: ordinary `ferrum-qt` is native first,
  QSettings use `Ferrum` / `Ferrum-Qt`, and templates use `~/.ferrum/templates` without a
  legacy BKChem migration or compatibility promise. Historical provenance and internal
  compatibility identifiers remain distinct from product identity.

### Additions and New Features

- Added provisional `ferrum cdml render {svg,pdf,png}` commands under one
  extensible render namespace rather than a format-specific `render-svg` verb.
  The command admits only uncompressed CDML through the versioned Rust-owned
  `ferrum-local-cdml-ingress-v1` five-dimensional resource profile, observes
  revision zero, composes one shared authenticated complete-page plan, refuses
  any root exclusion, and lowers through the selected native sink. SVG enforces
  a completed-text cap; direct vector PDF performs structural preflight and a
  completed-output check; direct raster PNG requires exact dimensions and an
  explicit background, then checks raw RGBA allocation and encoded output.
  File input retains the exact opened regular descriptor through the shared
  descriptor-relative publisher, so the source path or an observed hard-link
  alias cannot become the artifact destination. Directory-entry-unconfirmed
  publication succeeds with an explicit standard-error warning rather than
  silently discarding the publisher receipt. Standard streams remain bounded
  through `-`. PDF is not SVG embedded in a PDF container. Each backend owns a
  versioned format-specific resource policy, while all three share input,
  complete-plan admission, and safe publication. The local policy decision is
  recorded in `active_plans/reports/local_cdml_render_profile_v1.md`.
  The ordinary OASA-free Ferrum window now uses that same named profile for
  interactive, programmatic, and launch-file Open. A private pre-M18 PyO3
  receipt transfers one Rust-owned session plus its authenticated render
  observation exactly once from an asynchronous worker; Python never reads the
  file, chooses numeric limits, or reparses it. Open queues multiple launch
  paths, invalidates delivery on cancellation/close without claiming native
  preemption, verifies the session/observation pair before scene installation,
  and reports typed source, resource, text, and document failures without an
  OASA fallback. The compatibility host also consumes the same complete receipt
  instead of repeating observation. CD-SVG, compression, same-tab replacement,
  recent-file routing, wire, stable Python, and a frozen public CLI promise
  remain outside this slice.

- Added ordinary OASA-free `Chemistry -> Export Molfile V2000...` and `Export
  Molfile V3000...` for selected atoms or bonds that resolve globally to one
  durable direct-root molecule. Rust authenticates the immutable observation,
  rejects graph facts the native codec cannot preserve before adapter loading,
  and converts CDML/Qt downward-positive y coordinates to the chemistry and
  Molfile upward-positive convention. The immutable receipt retains revision,
  digest, root, explicit syntax, coordinate profile, and the exact native writer
  result; the descriptor-relative artifact publisher writes those bytes without
  path adoption or document mutation. The private PyO3 seam and cancellable Qt
  worker reauthenticate tab, selection, provenance, syntax, and receipt before
  publication. An additional optional ABI-4 capability now carries an authored
  UTF-8 molecule name into the native RDKit writer before serialization, so
  V2000 and V3000 retain that exact first title line without text
  postprocessing. Unnamed molecules continue through the older Molfile
  capability; older ABI-4 adapters remain loadable and report only the optional
  title operation unavailable. This adds no SDF, CLI, wire, stable Python,
  stereo inference, compatibility-host replacement, or OASA fallback.

- Added ordinary OASA-free `Chemistry -> Export SDF Record V2000...` and
  `Export SDF Record V3000...` for one selected durable direct-root molecule.
  Rust authenticates the immutable observation, recovers exact imported SDF
  metadata from the retained typed document, and preserves blank or exact
  titles plus ordered duplicate properties. For an ordinary molecule without
  imported metadata, its authored name becomes the title or the title remains
  blank. The native adapter writes only the titled structural Molfile; Rust
  owns the stable one-record SDF envelope, property blocks, and `$$$$`
  terminator. Ambiguous property names or values are rejected rather than
  serialized lossily. The private PyO3 seam and cancellable Qt worker retain
  revision, digest, root, syntax, selection, and publication fences, and the
  descriptor-relative publisher writes the exact frozen receipt without
  adopting the path or mutating the document. This adds no multi-record export,
  public CLI/wire/stable Python contract, compatibility-host replacement, or
  OASA fallback.

- Added ordinary OASA-free `Edit -> Document Drawing Defaults...` for the seven
  document defaults the current Ferrum renderer consumes: line width, atom-label
  size, line/text color, label background, multiple-bond spacing, wedge width,
  and heteroatom-hydrogen visibility. Rust now owns one unique-field atomic
  drawing-standard patch, creation of an absent canonical `<standard>`, exact
  opaque-content preservation, history, save/reopen behavior, and projection of
  the retained `standard/bond@double-ratio` fact. The private PyO3 and Qt layers
  expose no generic XML or mapping mutation and preserve current selection while
  reinstalling the accepted render observation. Authored font-family mutation and
  double-ratio editing remain outside the ordinary action until the verified-font
  and double-bond render profiles can honor them; personal defaults and applying
  overrides to existing objects also remain open M16 work.

- Added ordinary OASA-free `Edit -> Copy` for the current Rust-native selection.
  A same-molecule atom/bond selection becomes one connected structural fragment;
  selected bonds close over both endpoints, while complete-root metadata stays out
  of that partial fragment. Mixed presentation/structure or multiple-molecule
  selections instead copy complete selected direct roots in document order. Rust
  authenticates the immutable document observation and returns source-bounded
  normalized CDML. A private runtime PyO3 seam and cancellable Qt worker preserve
  revision, digest, tab, selection, and close fences before publishing both the
  Ferrum CDML MIME type and plain text with the retained ownership marker. Failure,
  cancellation, or stale delivery leaves the clipboard and document unchanged.
  Native Cut, Paste, selected-SVG copy, CLI, wire, and stable Python remain open.

- Added ordinary OASA-free `Options -> Theme` to the native-first product
  window. It reuses the retained Ferrum theme chooser and application-owned
  `ThemeManager`; an accepted different theme applies immediately, while cancel
  or reselecting the current theme is a no-op. This does not expose legacy grid,
  drawing-default, shortcut-preference, canvas, toolbar, or session ownership.
  The same ordinary lifecycle now saves window geometry only after every native
  tab actually retires; startup already restores that exact Ferrum preference,
  while a refused dirty or busy close does not publish a completed shutdown.

- Added retained mouse-wheel and keyboard zoom to ordinary Rust-native document tabs. A
  tab-owned `QGraphicsView` subclass applies the existing 1.15-per-notch
  behavior within the retained 10%-1000% display range, preserves the scene
  point under the cursor when scroll ranges permit it, and refreshes the
  existing percentage/status client. The existing View actions now own their
  retained `Ctrl++`, `Ctrl+-`, and `Ctrl+0` accelerators. Unsupported transforms
  refuse the wheel change. These inputs remain disposable display state:
  document, session, history, selection, scene ownership, Rust, PyO3, wire, and
  OASA behavior do not change.

- Added ordinary OASA-free `Export Standard InChI File...` and `Export Fixed-H
  InChI File...`. Rust now returns the existing exact-observation identifier as
  one owned receipt after proving the selected projection root before core graph
  lookup. The private binding retains that receipt, maps invalid Python text into
  the typed InChI error contract, and publishes exactly one newline-terminated
  `.inchi` record through the descriptor-relative Rust artifact writer. Qt
  freezes the chosen root and tab provenance across the destination dialog,
  reauthenticates the delivered receipt, and reports confirmed, unconfirmed,
  rejected, not-started, and possibly-completed outcomes distinctly. The actions
  never adopt the path, change the clipboard, or mutate document state. Existing
  clipboard export and the public InChI DTO/stub contract remain unchanged; the
  file publisher stays runtime-private with no CLI, wire, or OASA fallback.

- Added provisional `ferrum smiles canonicalize --adapter LIBRARY SMILES` so the
  runnable Rust CLI can parse and re-emit one complete graph through the same
  optional canonical-isomeric ABI-4 writer used by native document export. It
  writes exactly one printable SMILES line, requires an explicit safe adapter
  path, and never falls back to OASA or adapter discovery. M17/M18 still own the
  frozen CLI contract.

- Added ordinary OASA-free `Chemistry -> Export SMILES` and `Export SMILES
  File...` for selected atoms or bonds that resolve globally to exactly one
  durable direct-root molecule. Rust
  authenticates the frozen document observation and root before resolving native
  code, converts only the complete supported graph, and uses an optional ABI-4
  RDKit writer capability with the closed canonical-isomeric profile. The immutable
  receipt retains exact revision, digest, root, schema, profile, and one printable
  ASCII SMILES line. Qt shares the existing molecule-export worker owner with
  InChI, reauthenticates the active tab, selection, revision, digest, and receipt
  before copying the result, and shows selectable text; stale, switched, cancelled,
  and unsupported drawing-style outcomes never reach the clipboard. The file action
  reauthenticates the same selection after its destination dialog, then gives the
  immutable receipt to the Rust descriptor-relative artifact publisher. It writes
  exactly one newline-terminated `.smi` record through a private 0600 temporary,
  reports confirmed, unconfirmed, rejected, not-started, and possibly-completed
  outcomes distinctly, and never adopts the path or changes document state. Older ABI-4
  adapters remain loadable and report the optional operation unavailable. The
  runtime-only PyO3 seams stay absent from `.pyi`; this adds no CLI, wire, public
  Python, depiction-stereo inference, multi-record `.smi`, or OASA fallback.

- Added ordinary OASA-free `Chemistry -> Set Molecule Name...` for one direct
  durable molecule selected through any of its atoms or bonds. Qt resolves the
  complete literal child selection to exactly one root and fences the prompt
  against tab, revision, digest, root, and selection changes. Rust authenticates
  that opaque direct root and commits the exact entered text as one transaction;
  whitespace is retained, empty input removes the attribute, and an unchanged
  value adds no history or display refresh. Opaque content, Undo/Redo, Save, and
  reopen remain authoritative. The private runtime PyO3 method stays absent from
  `.pyi`; this adds no generated-name, normalization, OASA fallback, CLI, wire,
  or stable Python contract.

- Completed the native `Chemistry -> Molecule Information...` vertical for one
  or more selected durable atoms or bonds. An optional ABI-4 capability calculates
  isotope- and charge-aware formula, nonduplicating implicit/physical hydrogen
  counts, net charge, average molecular weight, exact mass, and element mass
  contributions with the sealed RDKit engine. Rust authenticates the exact
  observation and direct roots, freezes every graph before adapter use, and returns
  all-or-nothing records in document order plus a checked combined selection. The
  private runtime-only PyO3 seam remains absent from `.pyi`; the ordinary OASA-free
  window owns cancellable worker and stale/close delivery fences and displays a
  selectable, accessible read-only dialog. A fresh RDKit 2026.03.5 wheel and its
  independently rebuilt ABI-4 adapter passed installed-extension, Qt, closure, and
  relink checks; its direct native proof also fixes RDKit's non-obvious ordering
  of multiple labelled non-carbon isotopes. Existing ABI-4 adapters remain
  loadable and report the optional
  operation unavailable. This adds no mutation, OASA fallback, CLI, wire, or stable
  Python contract.

- Added compact native status-bar View controls as accessible clients of the
  existing OASA-free View actions. The visible `-`, current percentage/reset,
  `+`, Page, and Content buttons preserve active-tab display ownership, mirror
  action availability, and show `--` only when display observation is unavailable.
  Percentage observation accepts only exact finite uniform affine transforms;
  it never changes a transform. The reset button remains a keyboard recovery
  action for an unsupported observed transform. This adds no legacy slider,
  wheel or shortcut behavior, document/session/history mutation, Rust, PyO3,
  wire, CDML, backend, or OASA route.

- Added native `File -> Recovery Export Backend CDML...` for an exact current
  Rust backend snapshot. The action stays reachable for a live registered tab
  while its display is pending or native work is busy, freezes tab/revision/digest
  before the destination dialog, and reauthenticates all three before calling the
  existing revision-gated recovery publisher. Both returned snapshot provenances
  must match before feedback. This export never adopts a path, saves, cleans,
  retitles, changes history or selection, refreshes a scene, or changes worker
  state; confirmed, unconfirmed, rejected, not-started, and possibly-completed
  outcomes remain truthfully distinct. No Rust, PyO3, wire, CLI, ingress, or OASA
  route was added.

- Added Rust-only `ferrum_document::artifact_publication_v1`, a generic completed-owned-byte,
  descriptor-relative publisher. An optional retained live regular-source guard refuses
  aliases observed at both final checks only in a trusted, non-mutating destination directory.
  It retains no-follow parent descriptors, creates a private 0600 same-directory temporary,
  writes and file-syncs it, renames it, then directory-syncs it. Its explicit taxonomy
  distinguishes confirmed durability, directory-entry-unconfirmed durability, and
  possibly-published post-rename failure, plus destination-rejection and I/O cleanup
  uncertainty. The existing CDML save route remains an exact adapter, preserving session
  errors and saved-baseline behavior. The generic surface adds no CLI, public Python,
  wire, renderer, ingress, stdout, size-limit, or old-metadata-preservation policy;
  the private M16 `.smi` adapter now consumes it only for an authenticated completed
  SMILES receipt. The later local-CDML V1 policy and native SVG command compose this
  publisher explicitly; the generic publisher itself still chooses no format or
  resource policy.

- Added the ordinary OASA-free native View menu: Zoom In, Zoom Out, Zoom to
  100%, Zoom to Page, and Zoom to Content. Paper framing uses the renderer's
  authoritative scene rectangle; content framing unions installed projection
  roots while excluding the exact paper root. Each Rust-owned tab retains its
  own transform and scroll position. One guarded, retryable initial Page frame
  runs only after a live tab/view/scene passes window, registration, current-tab,
  visibility, and teardown fences. The display-only controls remain available
  while chemistry or rendering is busy and never mutate document, session,
  selection, or scene ownership. This does not adopt legacy `WindowViewMixin` or
  `ChemView`, toolbar/status zoom widgets, mouse-wheel semantics, or a
  pixel/timing gate.

### Fixes and Maintenance

- Restored `RDK_BUILD_THREADSAFE_SSS=ON` to the immutable native-wheel profile and
  preflight validator. The plan and resolved CMake audit already required this fact;
  making it explicit restores exact agreement with the sealed-input manifest instead
  of relying on RDKit's upstream default.

- Fixed verified Telex presentation whitespace to retain its authored advance while
  lowering as an outline-free paint across the private draw stream and SVG, PNG,
  PDF, and composite sinks. Every visible or mismatched scalar still requires its
  exact usable outline; no public DTO, wire, Python, Qt, or API surface changed.

## 2026-08-13

- Added the ordinary native `Chemistry -> Inspect Selected Molecule` slice. For
  exactly one selected durable atom or bond, Qt maps the literal child source ID
  and source order to exactly one direct-root opaque molecule ID, then sends its
  frozen observation to Rust on a cancellable worker. Delivery authenticates the
  worker/tab lifecycle, revision, digest, root ID, projection key, root source ID,
  and root order before a read-only dialog reports only source-backed authored
  name, source ID, atom and bond counts, lexical element inventory, complete-only
  formal charge, and normalized x/y bounds in points. Element inventory is not a
  formula. The private discoverable PyO3 entry remains unsupported and absent from
  the `.pyi`; this adds no molecular formula, mass, valence, oxidation, group,
  fragment, name-generation, linear-form, check, engine, OASA, clipboard,
  mutation, CLI, wire, or stable-Python behavior.

### Additions and New Features

- Added an in-process, paint-only recorder for the opaque authenticated
  whole-document direct-Haworth composite. It retains source-ordered roots,
  actual ordinary and authenticated direct target groups, explicit transforms,
  text/vector paint, styles, and monotonic paint order without emitting issues
  or exclusions. The caller supplies every structural limit, including checked
  fallible copies of root, target, and paint identity text; no default policy,
  API, PyO3, Qt, CLI, wire, or artifact route is introduced.

- Added a Rust-only, detached native-SMILES prepared-builder for the first
  closed direct-glycosidic Haworth profile. It accepts only two vertex-disjoint
  five- or six-member C/O cycles and one unique exterior oxygen bridge, rejects
  unsupported returned adapter facts and every extra/fused/spiro/ambiguous
  topology without normalization, and preserves ordinary placement centroid
  semantics through a receipt-local translation. The result is handle-free and
  does not allocate a session identity, mutate a document, or add PyO3, Qt,
  CLI, wire, renderer, or artifact behavior. A bridge may meet any selected
  ring carbon; the deterministic canonical layout vertex is not a biochemical
  anomeric claim. The UI/composite projection remains a separately reviewed
  dependent slice.

- Added the bounded native-17 peptide-template insertion route to the ordinary
  Ferrum-Qt Rust-owned tab. `Import Supported Peptide Sequence...` submits the
  exact accepted uppercase, no-space text through an API-owned 33,824-byte
  ingress budget to the concrete authenticated `NativeChemEngine`, then commits
  only a still-current prepared insertion through the existing revision/digest
  fence, Rust history, save, and reopen path. The native profile is exactly
  `ACDEFGIKLMNQRSTVY`; H, P, and W produce typed profile errors before native
  library loading, and there is no OASA fallback. The pure-domain historical
  template remains the separate 19-residue `ACDEFGHIKLMNQRSTVWY` profile with P
  rejected. This is neither generic peptide nor OASA parity, and it adds no
  public CLI, wire, or stable Python promise. The native-17 singleton/mixed and
  ANKLE current-artifact checks are disposable implementation evidence, not
  permanent count, timing, or CI thresholds; H/W require a future
  aromatic/explicit-H contract.

- Added M16 Stage A2b's closed direct-Haworth document insertion and Stage B's
  authenticated in-process composition. A1 receipts translate only through one
  finite anchor, persist exact q1/w1/n1 bond facts and ring/bridge roles through
  the prepared-candidate protocol, and retain canonical durable authored facts
  with the accepted operation observation. The API derives the same closed
  render observation without re-observing a session, authenticates the source
  `PersistentId` to one exact projection `DocumentObjectId` and root order, and
  retains the established whole-document plan while suppressing only selected
  bond outcomes. Atom masks and labels, nonselected bonds, and issues remain;
  private recording-sink traversal emits ordinary, q1, and w1 direct drawing
  once. Direct paint and line width follow the accepted drawing standard, and
  `standard/bond@wedge-width` has the source-backed 5px fallback only when it
  is absent. The composite is opaque and non-serde: no public SVG, PNG, or PDF
  overload, nor PyO3, Qt, CLI, or wire route, is added. Sucrose, stereo,
  collision avoidance, and reflow remain unclaimed.
  The same native-only boundary can now re-observe one explicitly selected durable
  molecule at an expected revision after save and reopen. It accepts only the strict
  raw persisted C/O profile, its exact atoms-before-bonds child order, and one of the
  closed 5/5, 5/6, 6/5, or 6/6 ring forms; it reconstructs canonical durable facts and
  composes through the existing authenticated selective Rust path. A hand-authored
  equivalent valid profile is also accepted, so this is not proof of historical A2b
  authorship. No persistent marker is added; M17 owns one only if later required.
  Global `InvalidPresentationFact` suppression remains unchanged.

- Added M16 A2a's read-only Haworth bond-depth projection. Typed CDML bonds now
  expose exact `front` or `back` `haworth_position` facts through the Rust and
  Python document projections; absent facts remain absent, while malformed
  spellings remain retained source data but project as `None` with an
  `invalid_presentation_fact` issue. This adds no authoring, renderer, session,
  Qt, CLI, schema, or XML serialization behavior.

- Added M16 Stage A1's closed direct-glycosidic Haworth authoring receipt in
  `ferrum-domain`. Its one factory accepts only one borrowed source molecule,
  a supplied topology classification, and a positive finite local scale. It
  reconstructs the classification against that exact molecule before copying
  selected C/O elements, rejects stale or mixed classifications, and rejects
  metadata, optional atom chemistry, source bond type/style facts, and extra
  graph facts. The immutable receipt owns canonical selected atoms, local
  coordinates, closed q1/w1/n1 bond facts, finite bounds, and scale for later
  document authoring only; it
  adds no document, renderer, API, PyO3, Qt, CLI, wire, or serialization route.

- Added M14's in-process-only direct-glycosidic Haworth renderer profile in
  `ferrum-render`. It consumes the accepted direct depiction receipt and derives
  one closed molecule-local order: ordinary back ring edges and ring-zero/ring-one
  bridge bonds, then q1 round-cap front strokes, then directed w1 rounded filled
  wedges. Source order survives only as checked `u32` provenance. The local
  profile uses scale-covariant 25% wedge overlap and 35% q1 padding, translates
  the source rounded wedge corners to finite cubic paths, and lowers through the
  private draw stream to structurally validated SVG. This does not alter the V1
  JSON grammar, generic Haworth lowerer, document model, API, PyO3, Qt, CLI, or
  public PNG/PDF routes. Offline semantic tests cover target partition, profile
  facts, and SVG structure without byte, pixel, timing, corpus, or network gates.
  Accepted review and one-time local SVG/raster visual evidence close M14 at this
  renderer boundary. Document authoring, placement, commit/history, CDML persistence,
  and fresh document composition remain the explicit M16 session-authority slice;
  M17 owns only any wire schema and M18 public Python/CLI exposure.

- Added M14's pure direct-glycosidic Haworth depiction-spec receipt in
  `ferrum-domain`. From one checked fragment it owns exact selected coordinates,
  bounds, identities, canonical ring cycles, and snapshot-local source orders,
  plus closed CDML `q1`/`w1`/`n1` and front/back facts for ring bonds. Each ring
  has one `q1`/front bond, its two canonical-cycle neighbours are directed
  `w1`/front shoulders from outer endpoint to shared-q endpoint, and every remaining
  cycle bond is `n1`/back in canonical endpoint order. The bridge is ordinary and
  remains typed separately with no Haworth role, style, or depth; copied source order
  is snapshot-local provenance, not map, child, or paint order. The receipt has no
  stereochemistry, renderer, serializer, document, session, API, PyO3, Qt, or CLI
  route. Compact offline Cargo semantic tests prove canonicalization, exact roles,
  shoulder direction, provenance, and bridge separation.

- Added M14's pure direct-glycosidic Haworth fragment receipt. It combines the
  validated canonical two-ring/exterior-oxygen topology with finite local layout
  into exact selected atom coordinates, ring and bridge endpoint identities,
  corresponding endpoint geometry, ring depiction semantics, source-order facts,
  and finite bounds. Selected atoms are exactly both ring vertex sets plus the
  bridge oxygen; selected bonds are exactly both ring cycles plus the two bridge
  bonds, partitioned by disjoint ring/bridge maps. Ring substituents remain absent;
  the receipt has no renderer, page, document, PyO3, Qt, CLI, stereochemistry,
  RDKit, or OASA behavior.

- Added a developer-only, consented `ferrum-api` Cargo example for measuring explicitly
  selected CDML/CD-SVG inputs under an operator-chosen read ceiling. It shares the document
  tokenizer's five accounting dimensions and hardened DTD/token failures, validates successful
  raw CDML and normalized CD-SVG payloads through the typed-document boundary, and records only
  constrained participant aliases and metadata. The private manifest rejects unknown and duplicate
  JSON keys; the receipt publisher rejects input/output aliases and uses descriptor-relative atomic
  replacement. It chooses no production admission default, exposes no normal CLI/Python/Qt route,
  and does not retain corpus data in the repository. Documented the consented local workflow in
  [devel/DEVEL_README.md](../devel/DEVEL_README.md): a receipt remains one-time evidence for human
  policy review, while external Open stays disabled.

- Added M14's pure direct-glycosidic local-layout receipt. It transforms the
  accepted canonical two-ring C/O topology into two owned finite Haworth
  depictions: canonical ring zero attaches at `(-scale, 0)`, canonical ring one
  at `(+scale, 0)`, and the degree-two exterior oxygen is at the local origin.
  The normalized adjacent-edge outward direction fixes local placement; bridge
  endpoints remain keyed by their topology-proven bond identities rather than
  graph source order. The geometry validates scale, finite results, and
  nonincident-edge crossings; it does not assign sugar identity, stereochemistry,
  labels, document placement, renderer, PyO3, Qt, CLI, RDKit, or OASA behavior.

- Isolated the ordinary M16 Ferrum-Qt startup closure from OASA. The public
  `MainWindow` and normal `ferrum-qt` route now use the Rust-native host and
  create an empty Ferrum-Chem document without importing the retained legacy
  editor. The complete OASA-backed session/canvas/template/worker lifecycle is
  explicit in `LegacyCompatibilityMainWindow`; generic QObject retirement is
  separately OASA-free. Ordinary external CDML Open now uses the shared direct
  Rust profile described above rather than the old whole-file Python route. The redundant
  `--native` selector was removed because the normal product route is native
  first. OASA remains packaged for the explicit compatibility host, so M22 is
  not claimed and M16 remains open. A one-time current-source Qt integration
  run with the sealed RDKit closure reported 985 passed and 1 skipped; it is
  evidence for this boundary change, not a shipping wheel or permanent test gate.

- Added M14's direct-glycosidic topology receipt in `ferrum-domain`. It classifies
  two supplied canonical five- or six-member C/O rings that are revalidated and
  vertex-disjoint, plus one exterior degree-two oxygen with two selected single,
  non-aromatic bridge bonds to selected ring carbons in different rings. The owned
  result records canonical rings, attachments, bridge, and graph-local source order.
  It does not infer sugar identity or stereochemistry, choose page placement, render,
  mutate a document, or cross a PyO3, Qt, CLI, RDKit, or OASA boundary. A read-only
  OASA topology receipt is one-time profile evidence; compact offline Cargo semantic
  tests remain the permanent check.

- Added M15's Rust-owned historical peptide-template SMILES profile. Given a validated
  `PeptideSequence`, it creates an owned deterministic template for
  `ACDEFGHIKLMNQRSTVWY`, including charged termini. Proline remains valid for sequence
  inspection but reports a typed one-based unsupported-template error. The profile is
  a pure domain utility with no OASA production dependency, FFI, PyO3, Qt, CLI, or
  document mutation. Its small read-only OASA comparison is implementation evidence;
  the retained Cargo tests cover durable template and typed-failure semantics. Future
  peptide work is the separate versioned ingestion, native parsing, and document
  insertion boundary.

- Added M15's first public peptide slice: `inspect_peptide_sequence_v1()` is a
  pure in-process Ferrum API for one strict canonical uppercase one-letter
  sequence. Its owned V1 receipt reports the canonical sequence, supported
  alphabet, count, and ordered one-/three-letter residue facts (including
  proline); its closed errors preserve empty input and the first invalid
  Unicode-scalar position. It creates no termini, structure, molecule, SMILES,
  document mutation, FFI/RDKit/OASA, PyO3, or CLI route. Future external
  ingress remains responsible for its own explicit text-resource policy.

- Started the M16 ordinary-window native-first lifecycle. `MainWindow` now
  creates a revision-zero Ferrum-Chem baseline in a native tab at startup and
  for File > New, without constructing or registering a legacy document
  session. The neutral shell supplies only tab, status, File/Edit/Options/Help,
  and explicit native actions; Save/Save As remain Rust-native, ordinary Open,
  same-tab Open, and programmatic/Recent routing accurately refuse until the
  separately required external-input admission policy exists. Closing the last
  native tab leaves the supported zero-page shell. Legacy UI/import ownership
  is deliberately deferred rather than silently recreated.

- Stabilized that M16 host boundary. The ordinary `MainWindow` remains the
  native-first root: it owns one Rust empty-document tab, has no registered
  legacy sessions, and keeps File > New and the final-tab zero-page lifecycle
  on that boundary. `LegacyCompatibilityMainWindow` is now the explicit
  migration-only owner of the complete retained legacy setup, rather than
  letting a fixture or partial initializer publish a legacy session into the
  neutral root. Permanent offline behavior tests cover the native and legacy
  roots, their isolation, and the zero-page action state. A manager one-time
  source-integration Qt run used the current source extension with a prior
  sealed RDKit closure and reported 988 passed, 1 skipped; it is integration
  evidence only, not a shipping wheel or a newly packaged artifact. External
  Open and its admission policy remain deferred, and the ordinary import
  closure remains OASA-coupled pending the next isolation slice, so M16 is
  still open.

- Added M14's first public Haworth slice: a read-only `ferrum-api` observation
  accepts one revision/digest-bound session observation and one explicit direct
  molecule C/O ring request, then returns only the selected direct-root identity
  and order, finite template-local bounds, and a molecule-local render plan.
  It proves direct-root identity before recursive core resolution; rejects foreign
  selected atoms without exposing other-document membership; preserves typed
  topology/render failures; and has no mutation, page placement, CDML rewrite,
  stereochemical inference, RDKit, OASA, PyO3, Qt, CLI, PNG, or PDF route. CDML
  core projection now marks parsed non-aromatic bond orders explicitly, allowing
  valid source-backed normal single ring edges to satisfy the existing Haworth
  non-aromatic topology invariant.

- Added the Rust-owned `DocumentSession.create_empty_document_v1()` baseline and
  its PyO3 binding. It creates a clean revision-zero canonical-namespace CDML
  root at version 26.07 with no selectable direct roots, without exposing a
  frontend XML template or claiming the nonempty `authored-26.07` profile.

- Made M13's fixed V1 miter limit material at the renderer boundary. Every
  issued vector and molecule stroke now carries the fixed 4.0 bevel-fallback
  ratio through the private draw stream, and SVG writes it explicitly alongside
  the existing butt-cap and miter-join profile. This is not a document property
  or a Qt policy change; it prevents future backend defaults from silently
  changing acute-corner semantics.

- Added M13's private borrowed draw stream inside `ferrum-render`. One fallible
  lowerer now owns ordered page traversal, exclusions, molecule anchor scopes,
  opaque masks and text backgrounds, vector path/ellipse profiles, and verified
  Telex outline conversion. The existing SVG API consumes that stream and still
  structurally validates its final artifact with xot; SVG source-order and
  identity attributes remain serializer-local diagnostics. A compact recording
  sink test covers paint order, anchor scope, cubic/fill profile, omission, and
  sink refusal. This is an internal refactor only: no public draw DTO, PNG/PDF,
  document, API, Qt, PyO3, CLI, parser, or file boundary was added.

- Added Edit > Delete Selected Bond with Ferrum to the ordinary window's explicit
  Rust-native tab. It is enabled only for one selected durable bond and asks the
  revision-bound Rust session to delete that bond as one undoable operation; both
  endpoint atoms remain. The replacement projection clears selection, and typed
  failures remain visible with no OASA or local-scene fallback. Same-tab/recent-file
  Open and legacy tabs remain unchanged, so M16 remains open.

- Added checked renderer-neutral direct-root vector operations for M13. Paths
  retain issued MoveTo, LineTo, CubicTo, and Close commands, including repeated
  vertices and self-intersections; ellipses retain exact center and positive
  radii. Every operation has explicit stroke and/or fill, filled paths must be
  closed, and the fixed V1 vector paint profile makes every issued stroke butt-cap
  and miter-joined and every filled path even-odd. SVG writes those values
  explicitly rather than inheriting backend defaults. The new render-only model
  has no document, parser, API, PyO3, Qt, or file boundary; source-specific
  presentation lowerers remain the next M13 slice.

- Completed M13's renderer-neutral backend boundary. `DocumentRenderPlanV1` now lowers
  once through a private checked draw stream to in-memory `xot` SVG, direct pure-Rust
  PNG, and direct pure-Rust vector PDF. Every successful sink returns the same external
  revision/digest, page, and named-exclusion receipt without embedding Ferrum metadata in
  artifact bytes. PNG requires exact dimensions, explicit background, and caller-owned
  raw-RGBA plus encoded-output caps; PDF requires caller-owned pre-allocation structural
  limits and a post-build nonpublication cap. Both receive explicit butt/miter/4.0 and
  even-odd paint facts. Offline semantic tests and an independent review passed; a
  disposable A4 six-root proof verified 800 x 1131 PNG decoding, one-page PDF structure,
  equal reports, and local recognizability. No byte, pixel, timing, or perceptual
  threshold was introduced. Cairo remains a separate M20 packaging decision.

- Added Edit > Delete Selected Atom with Ferrum to the ordinary window's
  explicit Rust-native tab. It is enabled only for one selected durable atom
  and asks the revision-bound Rust session to delete that atom with its
  incident typed bonds as one undoable operation. Selection clears after a
  successful replacement projection; typed failures remain visible and do not
  fall back to OASA or a local scene edit. Same-tab/recent-file Open and every
  legacy tab remain unchanged, so M16 is still open.

- Added the M13 API composition boundary for one final `RenderObservationV1`.
  It derives the finite physical page from the authoritative paper fact, carries
  the exact revision/digest provenance, and traverses molecule and presentation
  direct roots by source order. Verified molecule, plus, and Text operations
  retain their issued identity, anchor, bounds, background, and layout. Roots
  without a render operation become named profile, rejected-projection, or
  `not_yet_lowered` exclusions rather than disappearing or receiving a visual
  approximation. Invalid presentation suppression remains a typed no-plan
  result. This is an in-process Rust API only: it adds no wire, CDML parser,
  file, CLI, PyO3, or Qt route.

- Extended that M13 API boundary with exact Rust-owned lowering for direct-root
  Arrow, Polyline, Wavy, round-bracket, Rectangle, Square, Oval, Circle, and
  Polygon projections. It consumes only frozen projection facts: Arrow axis
  then ordered head subpaths, every issued straight-segment point, the supplied
  round-bracket cubic controls, normalized shape geometry, and explicit resolved
  stroke/fill. Canonical presentation RGB and positive widths cross into the
  renderer through checked conversions with no palette or toolkit fallback.
  Rejected projections and text-profile exclusions remain named outcomes. This
  adds no parser, document mutation, wire, CLI, PyO3, Qt, or file route.

- Added the first renderer-neutral whole-page composition model for M13.
  `DocumentRenderPlanV1` keeps one revision/digest provenance, an explicit
  finite positive page, strict direct-root paint order, opaque durable-or-local
  identities, and named exclusions for roots that have no operation yet. It
  preserves existing molecule plans plus both fixed-plus and multi-run Text
  Telex layouts, including their anchors, bounds, and optional backgrounds.
  The in-memory SVG backend paints only those ordered roots and does not invent
  paths for excluded content. This Rust-only model reads neither CDML nor files,
  and has no Qt, PyO3, CLI, or wire-decoding route; the separate API composer
  now supplies this model from one authoritative observation.

- Added Edit > Clear Atom Number with Ferrum to the ordinary window's explicit
  Rust-native tab. It is available only for one durable selected atom with an
  authored number and removes the complete number/show-number pair through one
  revision-bound Rust operation. Same-tab/recent-file Open and legacy-tab
  handling remain unchanged and OASA-backed.

- Added the first bounded M13 backend slice: `ferrum-render` can now lower one
  validated molecule render plan and an explicit finite viewport to owned,
  in-memory SVG. It preserves batch source order and batch-local paint order,
  maps the closed line/mask/ellipse/text grammar directly, and turns the
  digest-verified bundled Telex glyph IDs into SVG outlines with the explicit
  TrueType-up to Ferrum-scene-down Y conversion. It neither reads nor writes
  files, contacts a network service, chooses a system font, shapes text, nor
  exposes a CLI, Qt, or Python route. That initial molecule-only slice preceded
  the now-landed whole-page composition model; direct PNG/PDF sinks, CD-SVG, and
  publication remain M13 follow-on work.

- Completed M12 render operations and glyph metrics for the currently evidenced macOS
  arm64 Telex/PySide6 reference boundary. Rust converts verified Telex design units
  to scene `f64` values with no extra rounding, emits exact glyph IDs/origins, and
  uses true outline ink bounds for runs and centered plus signs; atom-label clipping
  alone also includes the durable atom origin. A QRawFont 6.11.1 comparison recorded
  exact glyph IDs, ordinary per-run floating-point observations of about `1.78e-15`,
  and a separate `0.0001875` baseline observation. Those values are evidence rather
  than permanent thresholds. A disposable current-wheel Qt proof consumed supplied
  iodine and plus outlines without shaping or measuring. M20 refreshes target evidence;
  M13 remains independent backend work.

- Completed M11 geometry and straightening for the currently evidenced macOS arm64
  release target. Generic graph validation now admits fused and bridged topology while
  single-ring normalization retains its own exact one-cycle rule. The public Rust
  result retains complete ordered y-up positions and a y-up rotation, including a
  no-op rotation, and the document session applies exact revision/digest-bound
  molecule batches atomically while preserving z, opaque content, and history. The
  recomputable locked/offline CPython 3.12/RDKit 2026.03.5 receipt measured a maximum
  coordinate delta of `3.645723512257204e-18`, zero rotation delta, and zero local
  repeat variation. Those measurements are one-time target evidence, not a permanent
  tolerance or CI threshold; M20 renews them for each added release target. PyO3
  exposure remains M18 work.

- Added the document-only M11 whole-depiction straightening boundary. A session
  prepares complete direct-atom-source-order y-up coordinates and the calculated
  y-up rotation for an explicit ordered molecule list, then accepts that exact
  revision/digest-bound result only after revalidating every direct target and
  expected source coordinate. The detached batch commits as one history entry,
  preserves z and opaque content, and leaves stale or invalid work non-mutating.
  This is a Rust document API only; PyO3, Qt, and target-specific parity receipts
  remain separate work.

- Added a bounded M16 bridge in the ordinary `MainWindow`: File > Open CDML with
  Ferrum opens a separate Rust-native tab. Within that tab, one selected durable
  atom can use Change Element with Ferrum or Edit Atom Properties with Ferrum.
  The latter sends a closed, revision-bound Rust patch for representable properties;
  fractional or otherwise unrepresentable source facts fail visibly before mutation,
  and cancelling the dialog is a no-op. Save, Save As, and close stay on the
  Rust-native session. The later native-first cutover supersedes this bridge: default
  `.cdml` Open is now Rust-owned, while the legacy property dock/actions remain in the
  explicit compatibility host and full M16 remains open. Permanent behavior tests cover
  route selection, selection-sensitive action state, lossless failure containment, Rust
  publication, and lifecycle; a real
  current-wheel ordinary-window exercise was disposable implementation evidence.

- Added Edit > Undo with Ferrum and Edit > Redo with Ferrum to that same explicit
  Rust-native tab. Each action asks the revision-checked Rust session to navigate
  its own history and installs the returned authoritative projection. Empty history
  reports the typed native failure without mutation. Switching to a legacy tab
  disables the Ferrum controls and restores the legacy Undo/Redo policy. Same-tab and
  recent-file Open plus existing legacy Edit actions remain unchanged. Permanent offline
  tests cover real element-change history, page transition, and empty-history
  containment; any current-wheel proof remains disposable.

- Added Edit > Set Atom Number with Ferrum... to the same explicit Rust-native tab.
  Exactly one selected durable atom can receive a positive number and explicit
  show-number state through one typed Rust mutation; cancelling the dialog leaves
  the session unchanged. The action does not call or fall back to OASA, while the
  same-tab/recent-file Open routes and every ordinary legacy-tab action remain unchanged. Permanent
  offline behavior tests cover the selection gate, accepted number/visibility update,
  and cancellation; any current-wheel exercise is disposable implementation evidence,
  not a pixel, byte, timing, or private-wiring gate. M16 remains open.

- Added Edit > Edit Bond Properties with Ferrum to the same explicit Rust-native
  tab. The action is available only for one selected durable bond and reuses the
  existing frozen-projection adapter and one revision-bound Rust patch. It exposes
  normal single, double, and triple bond semantics with only renderer-supported width
  and centering combinations. Unsupported source facts fail visibly before mutation,
  cancellation remains a no-op, and accepted work retains the durable selection.
  Permanent offline behavior tests cover the route; a current-extension exercise is
  disposable implementation evidence. Same-tab/recent-file Open and legacy property
  actions remain open or OASA-backed; M16 remains open.

- Completed M5 chemistry-codec parity without expanding the reference contract.
  OASA's registry explicitly defines SMARTS as export-only, so Ferrum does not invent
  a SMARTS importer as migration work. A disposable offline comparison generated
  eight queries under RDKit 2026.03.4 and 2026.03.5, parsed both query sets under both
  releases, and found agreement on all 272 chirality-aware query-target outcomes.
  Exact text happened to match but is not a gate. The existing bounded molblock,
  ordered 2D SDF, and Standard/Fixed-H InChI evidence now closes M5; 3D, compressed
  suppliers, and new query-language features remain separate product decisions.

- Added bounded ABI-4 InChI import, Standard and Fixed-H export, and InChIKey
  derivation through the sole native RDKit adapter, safe Rust, the direct PyO3
  extension, and a minimal explicit-adapter CLI. Inputs are validated before
  native loading, returned buffers become owned Rust values before release, and
  malformed prefixes, embedded NUL, statuses, lengths, and key grammar fail
  closed. A fresh RDKit 2026.03.5 macOS arm64 wheel, SHA-256
  `0f2de3ae9819545846af46efc45cae3eddbfbcabda5a0653f31d2a4ff6e79e6f`,
  passed its 18-dylib closure audit and installed-extension probes before and
  after a distinct adapter replacement. A disposable offline five-molecule
  comparison matched RDKit 2026.03.4 and 2026.03.5 exactly for both InChI modes,
  keys, and canonical round trips. Exactness is required for canonical InChI
  identifiers only; no general byte, pixel, timing, or network gate was added.

- Connected representable InChI molecules to the standalone OASA-free native
  editor. Preparation runs off the Qt thread through the packaged adapter, returns
  one handle-free insertion, and commits only against the captured document
  revision and digest. RDKit's InChI parser supplies a complete explicit-hydrogen
  count for every atom, so its accompanying no-implicit bit is treated only as
  parser-owned state on this route; positive counts are persisted and zero remains
  the CDML default. Ordinary graph imports still reject that bit, and chirality,
  radicals, atom maps, stereo bonds, and other unrepresentable facts remain typed
  failures before mutation. The permanent installed-extension test covers the
  public prepare/commit result. A disposable public-window run imported methane,
  rendered it, retained four hydrogens, advanced one revision, and loaded no OASA;
  no private-worker, pixel, timing, byte, or network test was retained.

- Added Standard and Fixed-H InChI export for one durable molecule in the
  standalone OASA-free native editor. Rust resolves the molecule from one exact
  document observation, rejects drawing-only or unsupported graph facts before
  native loading, and freezes the source revision, digest, molecule identity, and
  closed InChI mode around the existing ABI-4 adapter call. Qt performs native work
  off its UI thread and copies only a still-current result to the clipboard. Focused
  Rust and installed-extension tests retain the semantic graph and provenance
  contract. A disposable public-window run exported methane without mutating the
  document or loading OASA; no private-worker, pixel, timing, byte, or network test
  was retained. Ordinary-window and the other document codec exports remain open
  M16 work.

- Added bounded multi-record SDF import to the standalone OASA-free native window.
  Rust reads at most the ABI-4 byte ceiling plus one sentinel, validates through the
  packaged RDKit adapter, converts every complete supported 2D record into one
  nonoverlapping row centered at the requested insertion point, and commits the
  entire source order as one revision-bound history entry. Exact titles and ordered
  duplicate properties remain attached to their molecules in an opaque Ferrum SDF
  metadata namespace; blank titles are retained instead of invented. The permanent
  tests cover semantic metadata, identity, placement, atomic history, bounded UTF-8
  failure, and the frozen Python boundary. A disposable current-extension public-
  window run imported two records, restored all inserted-atom selections, undid the
  batch in one step, retained opaque CDML, and loaded no OASA. No SDF-byte, XML-byte,
  pixel, private-worker-wiring, timing, or network gate was added.

- Added OASA-free SVG, PDF, and PNG snapshot export to the standalone native
  window. Each export asks Rust for a fresh observation at the displayed
  revision and builds a detached, unselected Qt scene, so hover, selection, and
  stale on-screen geometry cannot become document output. `QSaveFile` publishes
  each artifact atomically and existing symbolic-link destinations are rejected.
  SVG and PDF retain vector painting; PNG uses the document's grounded 72-point-
  per-inch scale and Qt's configured image-allocation limit. A disposable
  current-build exercise produced valid SVG, one-page A4 PDF, and 596 by 842 RGBA
  PNG artifacts, visually confirmed the expected molecule and clean paper, and
  proved unchanged provenance and OASA-free execution. No pixel, byte, timing,
  private-wiring, or network test was retained.

- Extended the Rust-owned whole-root transform boundary. One revision-bound
  operation now translates, aligns, positively scales, or mirrors complete
  durable molecules and supported presentation roots around the aggregate
  selection center. Rust validates every target, scale, persistent coordinate,
  finite result, and complete bracket selection before a detached candidate can
  commit. After a real coordinate change, only an exact narrow backend-generated
  `linear_form` record is checked and retired when the transformed molecule no
  longer satisfies that record; richer, foreign, and malformed fragments remain
  untouched. Ferrum-Qt exposes scale, both mirrors, and six alignments only for
  complete root selections, and the modal scale path retains its pre-dialog
  revision and targets. Permanent tests cover semantic geometry, history,
  history-free identity, metadata ownership, and atomic rejection. A temporary
  direct-extension wheel and offscreen native-tab exercise were removed after
  one-time proof; no byte, pixel, private-wiring, timing, or network gate was
  added. The standalone native window now exposes a checkable Move Complete
  Roots gesture for the same validated complete selections. Qt paints and moves
  only disposable dashed root bounds; Escape retires them without submission,
  while release retires them before one captured-revision Rust translation and
  restores the durable selection after replacement. A disposable current-build
  offscreen exercise proved cancel, commit, authored-resolution coordinates, z
  and opaque-content retention, selection restoration, undo, and OASA-free
  execution; it was removed instead of becoming a private-wiring gate.

- Added a distinct Rust/PyO3 selected-atom rotation boundary matching the CDML
  contract: unique durable molecule/atom pairs, one finite scene-point center,
  and one finite radian angle are validated completely before detached mutation.
  Changed axes use the established 0.001 cm authored precision, z stays intact,
  exact canonical no-ops remain history-free, and invalid narrow generated
  linear-form metadata is retired without deleting richer fragments. Two
  semantic Rust tests and two installed-extension tests cover rotation, undo,
  malformed intent, molecule ownership, and atomic failure. The standalone native
  window now exposes a checkable Rotate Selected Atoms gesture. It derives the
  selection center and affected bond skeleton from the immutable Rust projection,
  paints only a disposable dashed Qt-local preview, retires that preview before one
  revision-bound release commit, and restores durable atom selection after the new
  render observation installs. The authoritative plan items are never moved or used
  as document state. A disposable current-wheel offscreen gesture proved preview,
  commit, authored-precision centroid and length preservation, z retention, undo,
  selection restoration, and OASA-free execution; no permanent private-wiring,
  pixel, coordinate-byte, timing, or network gate was added.

- Connected the first five honest document-level geometry-repair kinds to the
  pure-Rust planner. `normalize-bond-lengths`, `normalize-bond-angles`,
  `normalize-rings`, `snap-to-hex-grid`, and `straighten-bonds` accept unique durable
  direct-root molecule IDs plus an explicit finite-positive scene-point spacing,
  validate every selected molecule and graph before detached mutation, convert
  between CDML y-down and geometry y-up once, preserve z and opaque content,
  use 0.001 cm authored precision, and remain history-free when already
  snapped. `straighten-bonds` moves only degree-one endpoints, preserves each
  nondegenerate terminal length, uses increasing-angle 30-degree tie-breaking,
  and fixes the lexically smaller atom ID for an isolated two-atom component;
  its common-envelope spacing is validated but intentionally unused. The older
  Rust `Straighten` planner still means whole-depiction rotation and was not
  reused. Length normalization keeps ring coordinates fixed, grows attached
  acyclic branches from their ring anchors, and anchors ring-free components at
  highest degree with durable-ID tie-breaking. Ring normalization uses a
  durable-ID canonical walk, preserves ring centroid and first-atom orientation,
  and translates each singly anchored acyclic substituent rigidly. Angle normalization
  keeps ring and anchor-edge coordinates fixed, assigns movable children to distinct
  nearest 60-degree slots in authored bond order, preserves nondegenerate lengths,
  and uses the explicit spacing only for coincident atoms. Incoming and ring directions
  reserve slots, and unsupported multiply anchored or oversubscribed topology fails
  atomically. Clean geometry remains a deliberately separate native-chemistry
  operation described below. A bounded OASA comparison caught and fixed a
  y-up/y-down ring-walk orientation error rather than weakening parity evidence.
  Earlier installed-extension proof caught
  and permanently regressed a missing-target eager-index panic before acceptance.

- Added `clean-geometry` as the sixth Rust-owned native Repair action without
  inventing a second layout algorithm. Ferrum validates every durable bonded
  molecule and supported chemistry fact before any graph crosses the existing
  `ChemEngine` boundary, asks packaged ABI-4 RDKit for fresh coordinates, then
  checks the returned coordinate count and prepares one handle-free
  revision-and-digest-bound batch. One document commit
  applies only changed direct atom x/y coordinates, preserves each source
  centroid, explicit target bond length, z values, opaque XML, durable identity,
  and source order, and rejects a later malformed target without partially
  changing an earlier one. Canonically equal authored coordinates remain
  history-free. Ferrum-Qt runs preparation in the existing cancellable worker,
  commits on the UI thread, and restores durable selection. Permanent tests cover
  semantic placement, atomicity, no-op history, and malformed Python intent. A
  disposable current-extension wheel combined with the previously accepted
  local RDKit closure proved the successful OASA-free Python and public-window
  paths; upstream OASA clean-geometry tests were a one-time semantic oracle.
  No coordinate-byte, pixel, timing, or network equivalence gate was added.

- Removed stale Atom/Bond Properties tests that constructed partial fake native
  tabs, invoked private `MainWindow` handlers, and therefore broke whenever an
  unrelated action-state query was added. The retained permanent tests exercise
  dialog-to-closed-DTO mapping, unsupported-value rejection, absence
  preservation, and real native-tab mutation; disposable current-wheel probes
  cover the public window composition without making its private wiring a gate.
  The standalone native Repair menu now routes all five implemented kinds without
  importing OASA: selected atom/bond identities resolve to their Rust-projected
  molecules, an empty selection means every durable molecule, and length/grid
  repairs ask for explicit positive scene-point or degenerate-vector spacing rather
  than adopting the legacy Qt fallback. Permanent Qt tests cover geometry, selection, all-molecule
  routing, and invalid-input atomicity. Temporary installed-wheel and OASA-free
  public-window probes were removed after passing. A separate all-Qt attempt against
  the development extension was not accepted as evidence: unrelated coordinate-generation
  tests require the sealed RDKit closure, and their warning path reached an existing modal
  Qt failure after those libraries were unavailable.

- Began the M16 full-session adoption with an explicit document-molecule render
  envelope. Each Rust render plan now carries its owning molecule's durable or
  projection-local identity and direct-root source order separately from the
  molecule-local atom and bond order. The frozen PyO3 boundary preserves that
  shape, and Ferrum-Qt places each molecule plan under one disposable root
  graphics group at the backend-issued document position. A semantic test uses
  a presentation polyline between two molecules to prove that root order and
  child order remain distinct. Supported Rust-projected polylines now remain
  independent top-level graphics roots in that same scene, so two molecules and
  multiple polylines interleave by backend-issued document order without
  flattening atom or bond order. Non-spline polylines now retain every authored
  point after the required first two, so multi-segment vectors and rectangular
  bracket roots render from the Rust-issued path instead of being rejected or
  collapsed to their endpoints. Rectangle, square, oval, circle, and polygon
  roots now join that same closed projection with class-aware targets, finite
  normalized bounds or ordered points, explicit stroke/fill provenance, and
  typed invalid-geometry issues. Qt draws only those issued facts, including
  established transparent `area_color="none"`, with no palette or geometry
  fallback. Supported normal, non-spline arrow roots now carry their complete
  source path, shortened axis, four-point head polygons, validated head
  dimensions, and explicit stroke across the frozen boundary. Other arrow
  families and spline arrows remain preserved with typed display issues rather
  than receiving normal-arrow artwork. Fixed-content plus roots now resolve
  anchor, size, foreground, background, and provenance in the document layer;
  the API centers one verified Telex glyph and carries its exact ID, origins,
  and ink bounds to Qt. Qt caches those supplied outlines without shaping,
  advancing, measuring, or substituting a system font. Authored font families
  remain explicit unsupported issues instead of receiving a silent fallback.
  One durably selected Plus can now edit its integer size and foreground through
  the existing visual form and one revision-bound Rust operation; the backend's
  wider four-field patch also supports font family and optional background while
  retaining unknown content and history. Values the integer form cannot preserve
  are rejected instead of rounded. No XML-byte, pixel, or timing equivalence gate
  was added. Source-current direct-wheel runs retained the selected Plus through
  the edit and imported no OASA. One durably selected normal non-spline Arrow can
  now edit its two head flags, representable line width, and color through the
  same revision-bound ownership model. The backend patch also owns spline intent,
  but the native dialog visibly disables that control until spline rendering is
  implemented. Vector presentation items now participate in the combined scene's
  durable selection map instead of being paint-only roots. A current-wheel public
  dialog probe remained implementation evidence rather than a permanent wiring,
  pixel, byte, or timing test. One selected rectangle, square, oval, circle,
  polygon, or ordinary polyline can now edit a form-representable width and
  stroke; closed shapes also edit fill or explicit no-fill. Rust validates the
  three-field patch, durable target, source geometry, revision, and retained
  appearance before committing one detached candidate. Semantic equality is
  history-free, and legacy three-digit colors, `color`, and `background-color`
  remain visible without being rewritten. Specialized `style="wavy"` polylines
  use a separate projection kind while retaining their durable CDML polyline
  identity. Rust publishes the exact authored point path and resolved stroke;
  Qt connects those points without regenerating, smoothing, or approximating
  the wave. One selected durable Wavy root may edit its form-representable width
  and line color through a dedicated two-field, revision-bound Rust patch.
  Draw Wavy Line now sends only two finite scene endpoints and current
  provenance to a prepared Rust operation. Rust bounds the established zigzag
  work, allocates the durable presentation ID, authors the complete point path
  and default stroke, validates the detached candidate, and commits once. The
  straight drag preview is disposable Qt state. Focused permanent tests cover
  semantic state, endpoint/path projection, history, selection, and failure
  atomicity. Real public dialog and drag runs remained disposable current-wheel
  evidence; no byte, pixel, wiring, or timing gate was added. Rectangular and
  round bracket insertion is now also one revision-bound Rust operation. The
  document layer allocates two durable polyline IDs, derives the established
  proportional geometry, materializes the effective drawing-standard stroke,
  and publishes an explicit pair relationship; Qt never guesses pairing by
  proximity. Separate rectangular and round native actions now capture the same
  finite drag box and submit the exact closed style to that Rust operation;
  their rectangle preview remains disposable Qt state. Round pairs cross as an
  explicit closed root kind and Qt constructs each side's cubic path from the
  four Rust-issued points without interpreting CDML or approximating pixels.
  Permanent tests cover semantic geometry, pair identity, history, stale and
  malformed rejection, and durable selection. A public-window drag remained
  one-time current-wheel evidence rather than a permanent wiring, pixel, byte,
  or timing test. Selecting both rendered sides now reuses the existing vector
  form for one common
  width/color patch; Rust revalidates the complete durable pair and updates both
  members atomically while preserving pair selection. Normal MainWindow
  adoption, unsupported Text faces, specialized arrow rendering, and broader bracket
  properties remain open M16 work.

- Added bounded standalone Text display to the OASA-free native scene. Rust now
  projects direct-root Text identity, anchor, resolved font/background facts,
  multiline character data, and the closed `b`/`i`/`sub`/`sup` formatted-text
  grammar. The fragment decoder rejects declarations, custom entities,
  namespaces, attributes, comments, processing instructions, unknown tags, and
  contradictory script styles; whole-document resource policy remains owned by
  the explicit CDML ingress budget rather than an invented Text-only limit. The
  render API lays out supported regular Telex text into exact glyph IDs,
  positions, scripts, paints, and bounds. Qt caches only those supplied glyph
  outlines, so it performs no shaping, measuring, system-font lookup, or XML
  interpretation. Authored font families, bold/italic faces, and missing glyphs
  remain typed target issues rather than substitutions. Permanent tests cover
  semantic runs, malformed-input rejection, provenance, frozen DTOs, and durable
  selection; a fresh wheel install and offscreen scene exercise remained
  disposable rebuild evidence. No XML-byte, pixel, timing, or network gate was
  added. One selected durable Text can now replace its complete baseline,
  subscript, and superscript run sequence, integer size, and foreground colour
  through one revision-bound Rust patch. The detached mutation preserves
  unrelated namespaced content and participates in normal undo/redo history.
  The native form visibly disables bold, italic, and font-family controls
  because the current verified renderer cannot preserve those choices; the
  backend grammar retains those closed facts for later renderer expansion.
  Permanent tests cover semantic run mutation, history, preservation,
  malformed and stale rejection, exact frozen DTOs, and durable selection. A
  fresh direct-wheel install and public-window action launch remained
  disposable rebuild evidence. Unsupported face families remain open M16 work.
  A complete selected set of durable direct-root presentation objects can now be
  deleted through one exact kind-and-authored-ID Rust operation. The document
  layer resolves every target before mutation, revalidates direct-root ownership
  and revision, rejects wrong kinds and duplicate targets, and deletes both
  members of an authoritative bracket pair atomically while refusing a partial
  pair. Permanent tests cover semantic removal, history, preservation, and
  atomic rejection; wheel installation and public action launch remain
  disposable evidence rather than permanent wiring tests.
  Bring to Front, Send to Back, and Reverse Selected Slots now use the same
  exact Rust selector boundary. Front/back operations retain selected source
  order, slot reversal changes only selected element slots, non-element root
  content stays in place, and bracket members move only as a complete pair.
  Permanent tests cover semantic order, history, preservation, selection, and
  atomic failure rather than XML bytes, pixels, wiring, or timing.

- Moved the fixed CDML paper-name catalog and millimetre dimensions into
  `ferrum-document` and exposed it as frozen PyO3 values. Live Qt scene setup,
  snapshot rendering, and the transitional session's catalog query now consume
  that Rust-issued table instead of asking OASA to interpret paper names. The
  permanent Rust tests cover catalog invariants and representative exact
  lookups; a complete comparison against the read-only OASA oracle remained a
  one-time rebuild check. The first direct core paper and viewport now also
  cross the observation boundary as revision- and digest-bound frozen facts,
  including valid standard-backed creation defaults without materializing an
  absent paper. The standalone native editor reuses the existing intent-only
  Document Properties form and submits its seven explicit fields as one closed,
  revision-bound Rust operation. Rust validates the complete patch, edits a
  detached retained tree, preserves foreign content, later paper records, and
  root order, and participates in ordinary undo/redo history. Permanent tests
  cover those semantic ownership and atomicity rules. A fresh direct-wheel
  offscreen edit/undo/redo run remained disposable implementation evidence, so
  this change adds no XML-byte, raster-pixel, private-wiring, or timing gate.
  That observation now also resolves the oriented physical page to the same
  72-point-per-inch scene units as document geometry. The native projection
  paints one noninteractive palette-local page at the backend-issued rectangle;
  malformed preserved paper facts receive a typed issue and the established A4
  portrait display fallback. A current-wheel offscreen image was inspected and
  deleted as one-time implementation evidence rather than retained as a pixel
  golden. Normal-window session adoption remains open M16 work.

- Added seven Rust-owned atom-mark toggles to the standalone OASA-free native
  editor: circled plus/minus, radical, biradical, electron pair, dotted electron
  pair, and p orbital. Ferrum-Chem owns direct-atom targeting, chemistry-scalar
  deltas, source-order ordinals, atomic history, and semantic render primitives;
  Ferrum-Qt consumes frozen projections and cached line/ellipse geometry. The
  selected durable atom survives each reprojection, and an exact same-type
  ordinal chooser can remove one duplicate mark without inventing persistent
  IDs. The chooser is the native replacement for legacy canvas selection because
  ID-less marks are atom-owned facts rather than independent durable objects. An
  explicit local-wheel E2E now saves and reopens all seven kinds, checks their
  semantic render operations, retains opaque XML, and imports no OASA. FQ-017a
  is complete for the standalone native route; normal-window adoption remains
  part of the broader M16/M22 cutover.

- Added the backend-only M10 preservation gate at
  `tests/e2e/e2e_cdml_preservation.py`. It discovers every committed CDML corpus
  document and runs the public structural rewrite check without byte comparison,
  Qt reconstruction, OASA, network access, timing thresholds, or checked-in JSON
  goldens. The current corpus passes, completing M1d, M8a, M9, and M10 while the
  full normal-window session cutover remains M16 work.

### Fixes

- Clarified the open M13 contracts before PNG/PDF implementation. The shared V1
  stroke profile now names its existing fixed 4.0 miter bevel fallback alongside
  butt caps and even-odd fill. A future PNG route requires exact caller-owned
  dimensions and background, preflights raw RGBA admission, and caps logical
  encoded bytes through direct streaming. A future PDF completed-artifact cap
  rejects publication only after build; it is not a memory or allocation claim.
  Neither route has a default or DPI policy, and their pure-Rust dependencies
  remain implementation-time lock-review decisions. M13 remains in progress.

- Corrected direct-root rectangle, square, oval, and circle projection to reject
  finite zero-width or zero-height bounds as the existing typed shape-geometry
  issue. CDML remains preserved; only lowerable presentation roots change, so
  every current and future projection consumer receives the same no-root result.

- Corrected the M11 and M12 tracker status to `in progress` based on landed geometry,
  straighten, declarative render-op, Telex, PyO3, and Qt evidence. The tracker records
  their remaining measured receipts without treating the native Qt snapshot export as
  M13 backend completion.

- Corrected the native coordinate worker refactor to route queued results through
  an owned `QObject` relay; PySide6 does not reliably register decorated slots
  inherited from a plain mixin. Removed obsolete `--target aarch64-apple-darwin`
  arguments from three native-wheel instructions because the current proof has a
  fixed profile and exposes no target option.

- Corrected the Ferrum-Qt capability ledger to recognize that About, CLI version,
  release metadata, license, and repository identity already use the self-contained
  Ferrum boundary. The newer BKChem/OASA presentation-authority commits were also
  checked as a read-only migration oracle: their frontend-only responsive and
  accessibility changes are already present in Ferrum, while their OASA-owned
  appearance resolution is intentionally superseded by the Rust projection path.

- Simplified the root `check_rust.sh` front door without weakening its baseline.
  Ordinary `cargo test` now owns unit, integration, and doc tests in one pass;
  `--all-targets` remains only on strict Clippy, where it finds lint failures in
  tests and examples. The human-facing command is just `./check_rust.sh`, and a
  platform `--target` is reserved for actual platform qualification rather than
  every local edit.

- Reduced the pinned RDKit profile to 24 deliberate CMake switches, then added
  only two grounded InChI hermeticity declarations for the pinned local target.
  Thread-safe substructure search now relies on RDKit's current default and is
  checked in the resolved cache, so the current profile has 25 fixed switches. Removed
  wrapper child switches already controlled by their parent, current default-off
  features unrelated to the two built targets, an obsolete `RDK_USE_URF` name,
  and redundant discovery hints. The builder now verifies the resolved CMake
  cache plus the existing provenance and final native-closure gates, so the
  shorter command does not weaken host, network, Python, or compiled-Boost
  isolation. The maintainer CLI also removed its fake target choice, partial
  RDKit-only archive override, and adapter build-type knob. It now exposes only
  the output locations and the real choice between verified offline archives,
  sealed native inputs, or hash-verified downloads. The C++-only replacement
  adapter also no longer receives the unused C compiler setting; RDKit retains
  it because the pinned InChI implementation is C.

- Rotated the 2026-08-11 and 2026-08-03 day blocks into
  `docs/CHANGELOG-2026-08a.md`, retaining the two newest day
  blocks in the active changelog as required by repository policy.

- Removed permanent pytest wrappers for one-time parity receipts, a subprocess
  module-entry smoke check, a hard-coded CDML reader filename inventory, and five
  private worker-wiring checks. Their useful rebuild evidence remains in focused
  development reports or E2E tools. Permanent tests remain deterministic, local,
  and behavior-facing; byte, raster-pixel, and arbitrary startup-time equivalence
  are not general acceptance gates.

- Clarified the M19 acceptance wording so the capability behavior remains required
  without freezing every currently named pytest artifact. Each candidate test must
  satisfy the [PYTEST_STYLE.md](PYTEST_STYLE.md) permanent-test checklist; fragile
  wiring, exact-count, fixture-heavy, networked, subprocess, and timing checks are
  deleted, relocated to E2E, or retained only as disposable implementation evidence.

## 2026-08-12

### Additions and New Features

- Added persistent atom numbering to the standalone OASA-free native editor.
  One durably selected direct atom can receive a positive number with explicit
  show/hide state or have both fields cleared through one revision-bound Rust
  operation. The Rust projection and render plan now carry the number and its
  verified Telex glyphs; Qt neither parses CDML nor invents font, color, offset,
  or visibility defaults. Assignment, hidden storage, clear, undo/redo,
  save/reopen, legacy-number-mark rejection, and opaque-content retention pass
  in a fresh direct-wheel E2E without importing OASA. Chemical atom marks remain
  an open FQ-017a capability.

- Added root `check_rust.sh` as the memorable Cargo verification front door.
  It runs rustfmt, all-target checks, strict Clippy, tests, doctests, and API
  documentation for the main eight-crate workspace and the standalone PyO3
  extension workspace. It reuses the bounded Cargo cache and deliberately keeps
  native-wheel, RDKit source-build, and Python/Qt end-to-end proof in their
  separate platform-specific gates.

- Added bounded local V2000/V3000 Molfile import to the standalone OASA-free
  native window. Rust verifies the opened regular-file handle, reads no more
  than the existing ABI-4 operation limit plus one sentinel byte, rejects
  invalid UTF-8 and molblock input before native loading, converts the complete
  RDKit graph into a handle-free insertion, and commits it only against the
  captured document revision and digest. The public Qt action runs preparation
  off the UI thread, renders the accepted result, and saves/reopens CDML without
  giving Qt parser or mutation authority. The focused source-bound direct wheel
  SHA-256 is
  `aefe9789f582b710c047bd1e570df424a510220886c0c3a2f2eb74c8fcab5232`;
  it reuses the accepted RDKit 2026.03.5 native closure and is not a new release
  or platform-matrix receipt.

- Adopted the paired historical CDML backend/frontend contract and format specification
  from `vosslab/bkchem-oasa` as explicit Ferrum reference documents. Each copy records
  its source commit and SHA-256, retains historical OASA/BKChem terminology, maps the
  intended Ferrum-Chem/Ferrum-Qt roles, and requires deliberate upstream reconciliation
  for future local changes. The adoption is not a completion claim for every operation.

- Added strict bounded V2000/V3000 molblock import through official RDKit 2026.03.5,
  ABI-4, owned Rust molecule values, frozen PyO3 DTOs, and provisional
  `ferrum molblock inspect --adapter ABSOLUTE_LIBRARY INPUT`. Seven chemistry cases
  pass semantic import/export under current RDKit 2026.03.5 and previous stable
  2026.03.4; bytes, timestamps, and formatting are observations rather than gates.
  The fresh 3.3 MB direct wheel SHA-256 is
  `13de57cf0d95dc3f1755f14a1ca36350fe4db7dca43e3ab8ead0e3d0e74b3eda`.

- Accepted the macOS arm64 ABI-4 FCM1 Ferrum-Chem direct-extension wheel. Its sealed
  package-owned closure contains `libferrum_chem.dylib` and 12 RDKit dylibs (13 total),
  and the accepted wheel SHA-256 is
  `7928bbf082578f1325cc21dbae4aa90171e22a6fe6f7ea1aa6df42f662e5dc44`.
  The reviewed wheel has no Python binding shim and no Cairo or FreeType native linkage.
- Added accepted Rust session, document/projection, render-observation, and PyO3 DTO
  slices. The Qt molecule painter consumes exact Rust render plans and verified Telex
  glyph data, including Rust-supplied glyph IDs and origins.
- Added the standalone `ferrum-qt --native` CDML preview route. Its native window and
  document tab create a direct `ferrum_chem.DocumentSession`, use the Rust observation
  and render path, and open, save, reopen, and close the bounded native CDML tab without
  importing OASA.
- Added a bounded Rust-owned native edit loop: change one durably selected atom element,
  add one free-standing atom at an exact scene point in an explicitly selected durable
  molecule, connect exactly two selected atoms in one molecule with a normal single bond,
  or drag from one existing atom to another through a disposable Qt preview; then apply Rust
  undo/redo and save/reopen the result without importing OASA. Bond creation allocates its
  durable ID in Rust, rejects self/cross-molecule/duplicate edges before mutation, and uses
  the same revision-bound one-use candidate contract as atom insertion. The gesture refuses
  stale document provenance and never gives Qt chemical-document ownership.
  Releasing that gesture in empty space now creates one carbon and its single bond as a single
  fully projected Rust candidate and one undo entry; no intermediate free-standing atom is
  published. The focused 3.5 MB direct wheel SHA-256 is
  `cc423e245a57ce2e28dbeb3a06960aeb34e5e81fea7f4a342c0a827a91fa591f`.
- Added a bounded Rust-owned native atom-properties edit. One durably selected atom can receive
  element, charge, valence, isotope, multiplicity, label visibility, hydrogen visibility, label
  font size, and label colour as one validated revision with undo/redo. The PyO3 API accepts exact
  frozen values only, and the shared Qt dialog remains visual-only; optional authored facts clear
  rather than storing UI defaults, opaque CDML material remains preserved, and unrepresentable
  dialog values fail visibly rather than being clamped. Requests longer than the nine-field closed
  grammar are rejected before value extraction, and stale requests retain the authoritative
  snapshot unchanged. This is pre-milestone native-preview evidence, not completion of M8a, M9,
  M16, or OASA removal.
- Added an explicit caller-owned XML admission-budget preflight in `ferrum-document`. It can
  reject decoded XML by UTF-8 bytes, element count, nesting depth, attribute count, and lexical
  text/CDATA bytes before the retained `xot` tree is built, while preserving the existing DTD and
  entity rejection policy. CD-SVG can apply independent caller policies to the original lexical
  SVG wrapper and to the selected CDML subtree after structural serialization. No production
  limit or ingress switch is claimed yet: the tiny tracked corpus does not justify a compatibility
  ceiling, so deployment policy waits for representative user-document measurements.
- Added a budgeted `TypedDocument` construction seam and an admitted-document session
  constructor. The backend can now initialize the same clean revision-zero history state from
  one already budget-validated retained tree, without an unbounded second source parse; ingress
  policy remains explicit and caller-owned.
- Added Rust-owned native bond-properties editing for one durably selected bond. One closed
  seven-field operation can update order, depiction style, center intent, line width, signed bond
  width, wedge width, and color with one revision and undo/redo. The PyO3 boundary rejects
  non-exact tuples and more than seven entries before value extraction; Rust validates the whole
  patch, including the documented `q1`-only Haworth-front constraint, and retains endpoints, IDs,
  opaque content, namespace meaning, and source order. The
  visual-only Qt dialog rejects source widths it cannot represent exactly instead of clamping or
  rounding them; its native route visibly limits style to Normal and disables unavailable wedge
  width before submission. Negative signed lane width now becomes one durable target-owned
  unsupported render issue rather than a silent omission or an absolute-value fallback. This remains
  pre-milestone native-preview evidence, not completion of M8a, M9, M16, or OASA removal.
- Added Rust-owned native atom movement. Move Atom captures revision/digest and the
  pointer-to-atom offset, shows only a disposable Qt preview, and commits one finite
  `Point3V1` through the document session. The operation is no-op aware, undoable,
  redoable, and preserved by save/reopen without OASA. The focused 3.5 MB direct wheel
  SHA-256 is `ae873cdbbdc39e571eb685e76af1551e08bc682b43c669fefdd2a9e6d10f2f4f`;
  its 15 native libraries are exact reuse of the accepted RDKit 2026.03.5 closure.
- Added Rust-owned native atom deletion. Delete Selected Atom submits one durable atom
  identity; the document layer removes that atom and every direct typed incident bond as
  one revision and one undo entry while retaining opaque reference-looking XML. The public
  installed-wheel route proves delete, undo, redo, save, and reopen without OASA. The
  focused 3.5 MB direct wheel SHA-256 is
  `94b1c57278c73b909929b4f6c8ea10a0f69d0586d8d01f5ef617bc16a460b46f`;
  its 15 native libraries exactly reuse the accepted RDKit 2026.03.5 closure.
- Added Rust-owned native bond deletion. Delete Selected Bond submits one durable bond
  identity; the document layer removes exactly that direct typed molecule bond as one
  revision and undo entry while preserving both endpoint atoms and opaque XML. The
  installed-wheel route proves delete, undo, redo, save, and reopen without OASA. The
  focused 3.5 MB direct wheel SHA-256 is
  `c41a19f2c5f8fd0d21429b900df0b1615324732c04753029c701e16276bb18a6`.
- Added Rust-owned normal bond-order editing. Change Selected Bond Order submits one durable
  bond identity and a closed single, double, or triple value; Rust owns no-op detection,
  revision, history, projection, and saved `n1`/`n2`/`n3` CDML. Normal double and triple bonds
  now render as explicit parallel lines. CDML `line_width` controls stroke thickness while the
  independent `bond_width` controls lane spacing, replacing an arbitrary stroke multiplier.
  The focused 3.5 MB direct wheel SHA-256 is
  `bbf93e5fafb805327a34eb0beba303c59a0bc519837522ee3e359fefc96ef411`.
- Added Rust-owned coordinate regeneration for one existing ordinary durable molecule. The
  packaged ABI-4 chemistry worker receives a coordinate-free graph, and Ferrum places its
  atom-aligned result at the molecule's existing centroid and existing mean bond length. A frozen
  revision- and digest-bound update commits every atom point as one history entry; stale results,
  unsupported pseudo-vertices/facts/styles, and unusable existing scale fail without partial
  mutation. The public native action runs off the Qt thread and passes undo/redo/save/reopen with
  no OASA import. The provisional `ferrum cdml generate-coordinates` command exposes the same
  operation through an exact authored molecule ID, explicit adapter path, and atomic file or
  standard-output publication. Its command-level proof checks retained placement semantics rather
  than CDML bytes or pixels. The focused 3.5 MB direct wheel SHA-256 is
  `f5f86b46ada762c1bb7663b32fe8e69d83a5795d869f44bab3a8cd96b395b4e2`.
- Split native Draw Bond and Move Atom pointer capture into a focused line-tools mixin.
  The Qt main window remains the tab/action host, while the mixin owns only revision-bound
  pointer intent and disposable preview retirement; neither layer gains document ownership.
- Added the completed M4d native SMILES insertion path. ABI-4 FCM1 chemistry produces
  a frozen handle-free molecule, Rust places it and allocates all durable molecule,
  atom, and bond identifiers, and the native Qt window prepares it off the UI thread
  before one revision-bound document commit. The fresh direct-extension macOS arm64
  wheel SHA-256 is
  `a901132f29fa3cd33c2516004be8bdf7fbe9272066d7cb6ab2b8b82b82caaaff`.
- Added the first M5 codec slice: a frozen Ferrum molecule can be exported through
  ABI-4 FCG1/FCT1 as RDKit SMARTS from Rust, `ferrum_chem.molecule_to_smarts`, and
  provisional `ferrum smiles to-smarts --adapter ABSOLUTE_LIBRARY SMILES`. Eight
  graph cases match the recorded RDKit 2026.03.4 build exactly. The current source-bound
  direct-extension wheel SHA-256 is
  `4b9f8e97629bdfa32f3ed0734c0f046a2118d8f64286a7128f759408f83bb650`,
  and installed Python SMARTS export passes before and after a distinct adapter
  replacement.
- Added the M5 V2000/V3000 export slice through ABI-4 FCB1/FCG1, safe Rust,
  frozen PyO3 values, and provisional explicit-adapter CLI output. Seven molecules
  pass strict semantic reparse under RDKit 2026.03.4 and 2026.03.3. The fresh
  direct-extension wheel SHA-256 is
  `4b9f8e97629bdfa32f3ed0734c0f046a2118d8f64286a7128f759408f83bb650`;
  its sealed closure contains the Ferrum adapter plus 14 RDKit dylibs, and both
  formats pass before and after a distinct adapter replacement.
- Added ordered multi-record SDF export through ABI-4 FSD1/FCT1, safe Rust,
  frozen PyO3 records, and provisional explicit-adapter CLI output. V2000 and
  V3000 records preserve title, record order, property order and values, and
  discrete chemistry under current RDKit 2026.03.4 and previous stable RDKit
  2026.03.3. Acceptance is semantic rather than byte-based. The current 3.3 MB
  direct wheel SHA-256 is
  `ebdb8de6dd779561472d0cd8f6bb3e395ca3838724053e831f81b743171026c4`.
- Added bounded 2D SDF import through strict RDKit `SDMolSupplier`, ABI-4 FSI1,
  safe owned Rust records, and frozen `ferrum_chem.ImportedSdfRecordV1` values.
  Import preserves record order, titles, ordered duplicate property names and
  values, and complete atom-aligned molecules. Provisional
  `ferrum sdf inspect --adapter ABSOLUTE_LIBRARY INPUT` exposes the same bounded
  import as one `ferrum-sdf-inspection-v1` JSON report. The current and previous
  stable RDKit semantic receipt remains meaning-based rather than byte-based. The
  fresh 3.3 MB direct wheel SHA-256 is
  `ebdb8de6dd779561472d0cd8f6bb3e395ca3838724053e831f81b743171026c4`.
- Completed the macOS arm64 M4c coordinate-parity gate. Twenty independent recorded
  RDKit 2026.03.4 Python-wrapper processes and 20 Ferrum ABI-4 wheel processes had
  zero internal noise and exact atom-order-aligned coordinates across six molecules,
  five asymmetric. The derived maximum absolute tolerance is
  `7.105427357601002e-15`, based on eight times the largest measured coordinate ULP.
  Refreshed the source-bound receipt after the molblock adapter/header change against
  wheel SHA-256 `4b9f8e97629bdfa32f3ed0734c0f046a2118d8f64286a7128f759408f83bb650`;
  the measured noise and cross-backend delta remain 0.0.
- Added provisional pre-M18 CLI commands: `ferrum cdml render-observation INPUT`, which
  emits one `ferrum-render-observation-v1` JSON document, and
  `ferrum smiles inspect --adapter ABSOLUTE_LIBRARY SMILES`, which emits
  `ferrum-smiles-inspection-v1`. SMILES inspection requires an explicit absolute,
  regular, non-symlink ABI-4 adapter; it performs no adapter discovery.
- Added the closed Rust/PyO3 periodic-picker display catalog: its exact 42
  user-visible elements use eight fixed category/color definitions, and the Ferrum-Qt
  popup now obtains its display facts without importing OASA. The fresh macOS arm64
  wheel SHA-256 is `3b11b82ca854574f73a6760ac46220fbe7fef8c96d04aeda8ce73b0560858313`.

### Behavior or Interface Changes

- Advanced native builds to the latest official stable RDKit release rather than
  retaining an older compatibility ceiling. Each wheel still records one exact tag
  and source-archive digest so that artifact can be reproduced; the prior stable
  release is retained only as a semantic compatibility comparison.

- The accepted wheel now uses ABI-4 FCM1 rather than the historical ABI-2 wheel and
  earlier smaller closures described in the 2026-08-11 entry. This is a macOS arm64
  packaging proof, not a cross-platform consumer release or a closed migration.
- Kept ordinary `MainWindow` CDML actions on the legacy OASA-backed route. The public
  `--native` window is the separate, OASA-free bounded-editor boundary; it does not
  claim bond drawing, full editing, document-class, export, or legacy-editor replacement
  coverage.
- Replaced caller-supplied atom identifiers at the PyO3 boundary with
  `prepare_create_atom_v1`: the caller supplies a current durable molecule selector and
  finite point, while the Rust session resolves the target and allocates the persistent
  atom identifier.
- Activated the migration exclusion guard for chemistry on the standalone native
  route. The separately hosted legacy editor remains outside that bounded ownership
  and is not incorrectly described as OASA-free.
- Made the V1 SMILES insertion writer fail closed for chirality, bond stereo and
  direction, radicals, no-implicit policy, atom maps, stereo references, unresolved
  aromaticity, and quadruple bonds. These remain explicit writer-mapping gaps until
  exact CDML round trips are established; they are not classified as CDML omissions.
- Kept SMARTS comparison scoped to what the evidence supports: exact strings within
  the recorded RDKit 2026.03.4 build, with semantic query equivalence required across
  versions. RDKit is not held at that release: each wheel records an exact tag and
  archive digest for reproducibility, while new builds move to the current stable
  release and retain the previous stable release as a compatibility check.
  SMARTS export does not compare coordinates or impose an unmeasured performance
  threshold because neither is part of the codec contract.
- Molblock acceptance compares strict parsed chemistry, not text bytes. Total
  hydrogens and normalized undirected bond endpoints prevent explicit/implicit
  storage or record direction from becoming false failures. Coordinate bounds come
  from half each emitted decimal quantum plus floating-point ULP; no arbitrary
  decimal, pixel, or timing threshold is imposed.

### Fixes and Maintenance

- Made native action refresh explicit across PySide6 6.11 signal boundaries.
  Programmatic Rust projection replacement now re-enables selection-sensitive
  actions only after pending authority clears, stale pointer gestures survive
  long enough to report their revision conflict, and the SMILES worker test owns
  deletion only after observing completion instead of depending on event-loop
  teardown timing.
- Removed the permanent coordinate, Molfile, SDF, and SMARTS receipt pytest wrappers.
  The reports retain build provenance and scientific observations, and their repeatable
  measurement programs remain under `devel/`; exact RDKit versions, closure counts,
  and one-time measurement results no longer become routine suite requirements.
- Removed a subprocess-only module-entrypoint test and five timeout/private-worker
  wiring tests from the permanent Qt suite. Direct preparation, delivery, revision,
  lifecycle, and visible behavior coverage remains; executable and installed-wheel
  proof belongs to the explicit E2E lane.
- Retired the filename-based CDML namespace-reader inventory test. It had begun
  classifying legitimate `cfg(test)` fixtures and the bounded API-ingress module as
  extra parsers; CDML ownership remains a Rust crate/API boundary backed by parser and
  ingress behavior, not an exact allowlist of source files containing one URI literal.
- Limited local Rust build storage without discarding the useful compiled-dependency
  cache. The main and standalone PyO3 workspaces now share one target directory,
  disable unbounded incremental object caches, and retain line-table debug information
  only for Ferrum code. Sealed wheel builds continue to use their explicit isolated
  target directories.
- Grounded the migration plan's performance gates in measured per-scenario
  distributions and explicit test conditions. Removed single-sample "no worse than"
  and arbitrary per-test duration language; frame timing now follows the target
  display's measured interval instead of assuming a universal 16 ms requirement.
- Hardened the native tab's terminal disposal so its callback retirement does not import
  the legacy document-projection facade. Confirmed save now refreshes and installs the
  authoritative Rust render observation before adopting the new title and file path.
- Accepted the presentation-polyline projection's one-root atomic handoff. Candidate
  build, old callback disposal, root replacement, publication, rollback, and terminal
  invalidation now have explicit ownership rules; ambiguous native ownership fails
  closed instead of publishing a misleading projection.
- Corrected a stale PyO3 binding test to assert the implemented `OnceLock` invariant:
  a captured native function retains its package-owned library origin after a temporary
  `sys.modules["ferrum_chem"]` replacement.
- Made accepted-but-not-projected native mutations fail closed. Only authoritative
  refresh remains enabled, and a pending tab cannot close even when an accepted undo
  returns the Rust document to a clean baseline.
- Cleared the reported Python hygiene findings: native render modules no longer import
  `typing`, the native-file helper has an explicit return annotation, temporary-path
  tests use `tmp_path`, and the native package-builder self-test is consistently
  tab-indented.

### Decisions and Failures

- Removed native `.cdml` interception from the ordinary legacy `MainWindow` after it
  cleared the legacy active session and caused a full-suite cascade. The smallest
  source-of-truth correction preserves the dedicated native window while leaving the
  unfinished legacy architecture unchanged.
- Kept milestone status conservative. The accepted vertical slices do not close
  coordinate parity, broad codec compatibility, full Qt adoption, OASA removal,
  cross-platform support, or the M17-M22 public-release gates.
- Kept M5 in progress. SMARTS export, explicit V2000/V3000 import/export, and bounded
  ordered 2D SDF import/export are green; SMARTS import/broader cross-version
  semantics and InChI remain open.

### Developer Tests and Notes

- The accepted ABI-4 wheel review verified the exact digest, ARM64 Mach-O closure,
  direct extension import, fresh replacement probes, `parse_smiles("CCO")`, and a saved
  Rust `DocumentSession`; it also verified the absence of Cairo, FreeType, Fontconfig,
  HarfBuzz, and libpng linkage.
- Focused real-wheel Qt acceptance covered the exact molecule-plan painter, native tab
  lifecycle, native CDML open/save/reopen/close route, bounded atom edits, and
  presentation-projection rollback paths. After native SMILES insertion, the full
  offscreen Qt suite passed with `902 passed, 1 skipped`; `ferrum-document` passed 84
  tests, and the current root Python suite passed 5,386 tests. The fresh molblock
  sealed-closure wheel passed 32 installed binding tests, including successful CCO
  insertion, explicit V2000/V3000 export, and rejection of unsupported aromatic
  fact-loss cases.
