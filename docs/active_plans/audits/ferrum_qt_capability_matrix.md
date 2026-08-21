# Ferrum-Qt capability matrix

## Current boundary

This is the authoritative current-product ledger. It records what the ordinary
`ferrum-qt` desktop application supports, refuses, drops before production, or
leaves for a later designed contract. It is not a port-all checklist for the
historical BKChem catalog.

The 2026-08-15 retirement removed the explicit compatibility host and its OASA
session, action, mode, worker, codec, and projection island. The ordinary
`MainWindow` is the only desktop route, and production dependency declarations
no longer require OASA. The isolated oracle remains migration evidence only; it
is not a runtime fallback.

The categories have precise meanings:

- **Supported**: a bounded user workflow is owned by Rust/Ferrum plus its thin
  Qt client. A row may name deliberately bounded facts; unlisted variants are
  not implied.
- **Refused with recovery**: the ordinary product declines an input before
  parsing or mutation and provides the named safe next step where one exists.
- **Preproduction drop**: no current product owner exists. Reintroducing it
  requires a new user-facing contract rather than reviving historical code.
- **Future contract**: a useful possible capability whose requirements are not
  yet designed or accepted.

Permanent tests cover compact, deterministic native behavior that can plausibly
regress. Wheel/site rebuilds, installed-window walkthroughs, source and package
inventories, accessibility and visual review, and race observations are
one-time implementation evidence. This ledger creates no byte, pixel, timing,
network, fixture, menu-inventory, or source-count gate.

## Current capability ledger

| ID | Capability | Disposition and owner | Current boundary |
| --- | --- | --- | --- |
| FQ-001 | Application and tabs | Supported: `ferrum_qt` ordinary `MainWindow` with Rust document sessions. | Create, close, bootstrap, Recent, and explicitly targeted current-tab Open retain tab and stale/busy containment. |
| FQ-002 | CDML and decoded CD-SVG files | Supported: Rust local-document ingress/publication with native Qt file clients. | Open/save/Save As/close/Recent/recovery support bounded CDML; a decoded CD-SVG `.svg` becomes a clean Save-As-only CDML document while retaining source provenance. |
| FQ-002 update | Unsupported local document formats | Refused with recovery: suffix-only native Qt refusal before document work. | CDXML, CML, `.cdsvg`, `.svgz`, compressed CDML, and compressed SVG do not parse, decompress, mutate a tab, or invoke a fallback. `Recovery Export CDML...` writes a recovery copy of the current supported document; it is not a converter. |
| FQ-003 | CDML projection and opaque facts | Supported: Rust typed document/session/projection boundary. | The native path owns typed facts, rendering observations, and bounded opaque preservation. Lexical XML fidelity is not promised. |
| FQ-004 | Chemistry import | Supported: Rust chemistry/document import plus native Qt clients. | SMILES, representable InChI, V2000/V3000 Molfile, and bounded 2D SDF import through one authenticated native operation. Unsupported chemistry fails visibly without partial insertion. |
| FQ-005 | Molecule file export | Supported: Rust chemistry/document export plus safe native publication. | Selected supported direct roots export as Molfile, one-record SDF, SMILES, or Standard/Fixed-H InChI without mutating the document. Multi-record and unrepresentable variants need a future contract. |
| FQ-006 | Whole-document artifact export | Supported: Rust Render Plan V2 backends and descriptor-relative publisher with a native Qt client. | `Export...` produces complete SVG, vector PDF, or transparent PNG at one pixel per Rust page point. Immutable provenance fences, source/known-hardlink protection, quiet cancel, and complete-plan refusal are part of the behavior. |
| FQ-007 | Basic drawing and rings | Supported: Rust candidate/session/render ownership with native gesture clients. | The ordinary editor authors supported atoms, normal and directed bonds, and the bounded detached regular-ring family exposed as Cyclohexane. Attachment, fusion, arbitrary ring controls, and new presentation grammars are future contracts. |
| FQ-007a | Atom and bond edits | Supported in accepted bounded slices: Rust revision-bound patches with native action and dialog clients. | `Change Element` accepts exactly one durable selected atom and commits only Rust-owned `set_atom_element`; its replacement projection restores that exact durable selection after one typed refresh recovery. Atom numbers, deletes, history, Undo/Redo, and save/reopen remain native. A source fact outside an accepted grammar is refused without local scene editing. A generic atom-property editor and broader direct-bond or presentation workflows require separate plans. |
| FQ-007b | Direct normal-bond drawing | Supported in one accepted bounded Qt profile: Rust pure admission plus opaque receipt commit, with a fixed normal-single carbon client. | The Qt `Draw Bond` QAction and viewport gesture admit and redeem one Rust-issued receipt for one C-C normal-single bond; Escape after admission leaves no commit. Rust owns candidate chemistry, identity allocation, fencing, history, and the sole commit. Wider bond styles/profile, richer drawing interactions, and complete OASA/BKChem parity remain future work. |
| FQ-008 | SMILES and InChI | Supported: Rust chemistry boundary with native import/export clients. | The supported graph subset reads SMILES/InChI and copies or publishes canonical SMILES and Standard/Fixed-H InChI. Stereo and other unrepresentable facts are refused. |
| FQ-009 | Coordinates and parsed insertion | Supported: Rust chemistry preparation and authenticated document insertion. | Parsed supported graphs receive native coordinates and one atomic insertion; placement respects document provenance. |
| FQ-010 | Chemistry tools and fragments | Supported in bounded slices: Rust inspection, naming, linear-form, bond-capacity, and explicit-fragment contracts. | These operations use accepted direct-root and document facts. Compact sugar notation, sugar-name inference, known-group expansion, substructure search, oxidation inference, generated names, and broader checks are preproduction drops; their dormant catalog/code families were retired. |
| FQ-011 | Geometry repair and bounded rotation | Supported in bounded Rust operations. | Clean Geometry plus Snap to Hex Grid, Straighten Terminal Bonds, Normalize Bond Lengths, Normalize Bond Angles, and Normalize Ring Geometry commit native plans atomically. The separately owned Rotate Selected Atoms route is also native. Fused or multicycle normalization, heuristic cleanup, bond edits, stereochemical inference, transform stacks, historical Rotate mode, and unported gestures are preproduction drops. |
| FQ-012 | Haworth insertion | Supported: Rust recipe/candidate/session/render ownership with a native Qt chooser. | The product inserts the four explicit alpha/beta D-glucopyranose and D-glucofuranose recipes at a shared snapped anchor. Broader carbohydrate inference, attachment, and preferences are future contracts. |
| FQ-013 | Direct glycosidic Haworth insertion | Supported in its closed structural slice: Rust composition and native Qt dialog. | It accepts only the documented neutral two-ring, one-exterior-oxygen topology and preserves state on invalid, stale, occupied, cancelled, or closed deliveries. Named-sugar, anomer, linkage, D/L, and stereochemical inference are future contracts. |
| FQ-014 | Biomolecule templates and peptides | Supported only for strict native peptide-template insertion. | `Import Supported Peptide Sequence...` accepts the documented unmodified uppercase native-17 sequence profile and commits one revision/digest-fenced Rust insertion; H/P/W are refused before engine work. System and biomolecule template catalogs, broader peptide authoring, termini policy UI, public Python/CLI exposure, and structure/mass/pI claims are preproduction drops. |
| FQ-015 | PubChem lookup and insertion | Preproduction drop. | The product has no live network lookup route or fallback. A future service integration needs its own reliability, identity, and failure contract. |
| FQ-016 | User templates | Supported: Rust template admission/insertion with a native Qt catalog client. | Ferrum-owned user templates support bounded save, inspect, refresh, and placement with fresh identities and safe catalog confinement. System-template compatibility is not implied. |
| FQ-017 | Presentation objects and drawing defaults | Supported in bounded Rust-rendered slices. | Supported text, plus, normal arrow, wavy, bracket, geometric, vector, paper, and drawing-default operations retain native facts, history, and rendering. Unsupported faces, spline/specialized arrows, and unaccepted editing variants are future contracts. |
| FQ-017a | Atom numbering and marks | Supported: Rust document/render ownership with native action clients. | Number visibility and the accepted chemical-mark toggles persist through history and save/reopen. New mark families or selection semantics require a future contract. |
| FQ-018 | Object transforms and ordering | Supported in bounded Rust operations. | Direct-root scale, mirrors, ordering, alignment, generated linear form, and complete-root translation retain native validation, history, and refusal behavior. Broader transform semantics remain future work. |
| FQ-019 | Clipboard and selected SVG | Supported: Rust fragment/session/render ownership with native Qt MIME clients. | Copy, Cut, Paste, and selected SVG use closed fragment/root grammars, authenticated provenance, fresh identities, and mutation-safe failure handling. New public clipboard or artifact APIs are future contracts. |
| FQ-020 | View and personal UI state | Supported: ordinary Qt view clients over native document/render state. | Bounded zoom, page/content framing, grid visibility/snap, tab view state, theme, toolbars, Next Drawing, properties observation, preferences, and workspace state remain application state rather than CDML or Rust document state. Broad historical mode/view configuration is a preproduction drop. |
| FQ-021 | Product metadata and help | Supported: Ferrum Qt release boundary. | About, CLI, version lookup, product naming, licensing, and lineage acknowledgement are Ferrum-owned. Historical backend branding is provenance, not a runtime claim. |
| FQ-022 | Built-in action registration and plugins | Supported for explicit built-ins; third-party execution is a preproduction drop. | The ordinary menus register shipped native actions. An extension system must begin with a new discovery, permissions, lifecycle, compatibility, and failure-containment design. |
| FQ-023 | Compatibility bridge workers | Preproduction drop. | The OASA bridge worker family was retired with the compatibility host. Native asynchronous work remains only where a current native workflow needs it; a replacement bridge is not planned. |

## Evidence and follow-up

The M15 utility-disposition closure records the retained strict peptide,
linear-form, geometry-repair, and rotation routes while retiring unowned compact
sugar, catalog, group, and search families. The M16 host-retirement closure
establishes a single ordinary native desktop route, a production package with no
OASA dependency, a representative actionable CDXML refusal/nonmutation behavior
test, and disposable installed-product evidence.
Earlier migration reports that describe a compatibility host, OASA routes, or
their test suite are historical provenance and do not describe current behavior.

### M19 supported-row evidence index

M19 implementation is complete and awaits independent closure review. This index links each
supported row to already accepted evidence, rather than prescribing a new test matrix. The
[current migration handoff](../reports/current_migration_handoff_20260814.md) links the accepted
slice receipts; its named compact semantic tests are durable only where they protect a bounded
behavior. Installed-wheel, walkthrough, visual, package, and thread observations remain one-time
acceptance evidence.

| Supported rows | Existing accepted evidence | Validation lane |
| --- | --- | --- |
| FQ-001 | Native Open, current-tab replacement, Recent, and host-retirement receipts. | Compact native lifecycle semantics; installed ordinary-window walkthrough. |
| FQ-002 and FQ-003 | Local CDML and decoded CD-SVG admission, provenance, save/reopen, and projection receipts. | Rust/binding/native admission semantics; one-time wheel and public Open evidence. |
| FQ-004, FQ-008, and FQ-009 | Native chemistry-import/export and coordinate-preparation receipts. | Rust/document and focused native behavior semantics; accepted worker walkthroughs where applicable. |
| FQ-005 and FQ-006 | Native molecule and complete-artifact publication receipts. | Bounded publication/provenance semantics; one-time artifact and installed-window evidence. |
| FQ-007, FQ-007a, and FQ-007b | Native atom, bond, ring, property, gesture, history, and render receipts. `Change Element` closure: `/private/tmp/ferrum-change-element-projection-boundary-fix.md`, prior dual-wheel offscreen visual E2E, fresh dual-wheel fail-once regression, and `/private/tmp/ferrum-change-element-final-acceptance-review.md`. Direct normal-bond closure: `/private/tmp/ferrum-direct-bond-candidate-admission-p1-fix.md`, `/private/tmp/ferrum-direct-bond-qt-receipt-review.md`, `/private/tmp/ferrum-direct-bond-dual-wheel-e2e.md`, and `/private/tmp/ferrum-direct-bond-final-acceptance-review.md`. | Compact Rust/PyO3/native behavior semantics; the direct normal-bond dual-wheel QAction/viewport proof is one-time installed-wheel evidence for one commit and Escape cancellation without a manual gate. |
| FQ-010 and FQ-011 | Bond-capacity, linear-form, geometry-repair, and rotation closure receipts. | Compact semantic behavior; accepted public worker evidence where it protects the route. |
| FQ-012 and FQ-013 | Haworth and direct-glycosidic insertion receipts. | Recipe/transaction/render/native-action semantics; installed chooser walkthroughs are one-time evidence. |
| FQ-014 | Strict peptide-template insertion closure. | Bounded parser/session/native behavior semantics. |
| FQ-016 | Ferrum-owned user-template receipts. | Admission/insertion/catalog behavior semantics; public walkthrough is one-time evidence. |
| FQ-017 and FQ-017a | Presentation, drawing defaults, atom-number, and mark receipts. | Focused document/render/native-action semantics. |
| FQ-018 | Direct-root transform, ordering, alignment, and translation receipts. | Rust/session semantic behavior. |
| FQ-019 | Clipboard and selected-SVG fragment receipts. | Fragment/provenance/session semantics; worker and UI observations are one-time evidence. |
| FQ-020 and FQ-021 | Qt view, preferences, metadata, help, and release-boundary receipts. | Focused native behavior and ordinary launch evidence. |
| FQ-022 | Explicit built-in registration and no third-party extension contract. | Supported built-in behavior evidence; extension execution is a recorded pre-production drop. |

FQ-002's unsupported-format update is a refusal decision with its representative durable CDXML
nonmutation behavior test. FQ-015 and FQ-023 are pre-production drops; no parity test is owed.
Future-contract exclusions in supported rows remain outside their named lane until a later plan
adopts them.

Future work must start from a named user workflow and an explicit owner. It may
promote a future-contract row or replace a preproduction drop, but it must not
reintroduce a compatibility host or treat the legacy catalog as a required
parity inventory. The active plan remains the detailed slice record:
[ferrum-plan-v3.md](../ferrum-plan-v3.md).

## Historical provenance

Before the 2026-08-15 retirement, this file catalogued a migration-era
compatibility host and OASA-backed routes. Those descriptions explain the
origin of the FQ identifiers only. They are intentionally not current owners,
runtime dependencies, test requirements, or future commitments.
