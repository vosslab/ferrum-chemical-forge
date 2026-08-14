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
