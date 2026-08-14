# OASA runtime ownership ledger

## Purpose and authority

This is the M16/M22 runtime-ownership transfer ledger. It answers which production
boundary owns each capability while migration remains incomplete. The detailed
user-facing classification and exit evidence remain in
[`ferrum_qt_capability_matrix.md`](ferrum_qt_capability_matrix.md); this file records
only the shorter ownership decision needed to remove the compatibility island safely.

Two upstream evidence streams remain distinct:

- Changes under `bkchem-oasa/packages/oasa` primarily refine the CDML/behavior contract
  and read-only oracle used to specify Ferrum semantics.
- Changes under `bkchem-oasa/packages/bkchem-qt.app` are product and interface
  improvements. Ferrum should adopt applicable improvements, but any persistent action
  must receive a Rust document/session owner rather than carry its OASA plumbing forward.

This ledger does not claim that OASA has been removed. Ordinary `ferrum-qt` starts the
native `MainWindow`; the separately loadable `LegacyCompatibilityMainWindow` remains a
real production migration host and keeps OASA as a declared dependency until its retained
capabilities are transferred, explicitly dropped at M19, or removed with that host.

## Current direct-import boundary

The 2026-08-14 source audit found 26 production Ferrum-Qt modules with direct OASA
imports: 25 import-time modules and the action-time-only `bridge/worker.py`. The ordinary
startup graph imports none of them, but the explicit compatibility host still reaches the
whole island.

| Runtime cluster | Direct modules | Current owner and removal condition |
| --- | ---: | --- |
| Legacy document session | 9 | OASA owns the compatibility host's persistent CDML objects, candidates, revisions, commits, and save lifecycle. Remove with the host only after every retained action has a native or M19 disposition. |
| CDML hydration, projection, and XML | 9 | OASA owns compatibility-host hydration and inspection. Rust already owns ordinary local-CDML admission, typed observation, projection, mutation, and publication. |
| Chemistry and asynchronous actions | 4 | OASA owns remaining compatibility codecs, templates, PubChem, and worker preparations. Replace per capability; preserve delivery/cancellation semantics where work remains asynchronous. |
| Format, export, and clipboard | 4 | OASA owns compatibility-host format/clipboard routes. Rust owns the ordinary native codec/export/Copy/Cut/Paste/selected-SVG slices. |
| Distribution declaration | 1 dependency | `oasa>=26.08` remains in Ferrum-Qt and root requirements until the compatibility host and all supported OASA routes are gone. |

## Capability ownership decisions

“Native” means the ordinary product route has one Ferrum-owned authority. “Partial” means
the ordinary route has accepted native slices while the compatibility host or another
retained route still owns behavior. Detailed omissions are intentionally not duplicated
from the capability matrix.

| Capability rows | Ordinary product authority | Remaining OASA owner / decision |
| --- | --- | --- |
| FQ-001--003 | Native startup, Rust document admission/session, typed projection, Save/Save As, close, and recovery slices | Compatibility host lifecycle, remaining document classes, same-tab/recent/compressed policies; M16 decision required. |
| FQ-004--006 | Partial native imports, molecule codecs, safe publication, and page render backends | Remaining CDXML/CML/CD-SVG and compatibility registry/export routes; support or drop each at M19. |
| FQ-007/007a | Partial native atom/bond creation, editing, deletion, projection, rendering, and a read-only native Properties client | Remaining draw modes, rings, and compatibility-host mutations/dock still need Rust operation/gesture owners or cutover. |
| FQ-008--009 | Partial native SMILES/InChI, coordinate generation, and authenticated insertion | Remaining compatibility codecs/preparation workers stay OASA-owned until individually transferred. |
| FQ-010 | Native information, exact molecule name, and linear-form slices | Checks, oxidation, groups/fragments, generated names, and the compatibility-host linear-form action remain OASA-owned or open. |
| FQ-011 | Partial native transforms, rotation, repairs, clean geometry, and stack ordering | Retained compatibility repair/gesture routes require explicit M16 decisions. |
| FQ-012--014 | Partial native Haworth/glycosidic and bounded peptide routes | Broader sugar/template/biomolecule actions remain compatibility-owned; each needs a source-backed contract. |
| FQ-015 | No native owner | PubChem needs a controlled transport/rate/error contract or an M19 drop decision. |
| FQ-016 | No complete native owner | Template catalog/location is Ferrum-branded, but save/inspect/insert session behavior remains compatibility-owned. |
| FQ-017/017a | Partial native presentation editing; atom numbers/marks complete for ordinary Rust tabs | Unsupported faces/splines/preferences/overrides and retained direct-mark/compatibility routes remain open. |
| FQ-018 | Partial native order/scale/mirror/alignment and generated-linear-form ownership | Remaining generic object/property commands stay compatibility-owned. |
| FQ-019 | Native insertion-valid Copy, recoverable atomic Cut, fresh-identity Paste, and selected-root SVG slices | The compatibility-host clipboard remains legacy-owned until cutover; broader public exposure remains an M18 decision. |
| FQ-020 | Partial native view/theme/window state, including a native-owned continuous zoom slider, shared-action toolbar, and projection-only Properties dock | Treat further BKChem-Qt controls as interface improvements to assess and port; persistent preferences or editing ribbons first need honest native owners. |
| FQ-021--022 | Native identity/metadata and built-in registration boundary | Third-party plugin execution still needs an M19 decision; these rows do not justify keeping OASA. |
| FQ-023 | Native workers exist per accepted slice; no shared OASA-worker replacement facade | Retire each compatibility worker only with its caller. Preserve thread confinement, cancellation, and stale-delivery behavior where the native replacement is asynchronous. |

## Transfer log

### 2026-08-14: ordinary linear-form conversion

`Chemistry -> Convert selection to linear form` now belongs to the ordinary Rust-owned
tab. Qt translates selected projected atoms/bonds to one authenticated opaque root and a
source-ordered atom tuple; Rust owns `linear-form-direction-v1`, geometry, metadata,
resources, history, and refusal. The runtime-only PyO3 seam has no public stub, CLI, serde,
or wire promise. The compatibility host's OASA request/commit route remains until the host
cutover, so the direct-import count and dependency declaration do not change in this slice.

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

### 2026-08-14: native Properties projection client

The newer BKChem-Qt Properties dock is adopted without its OASA document reference,
direct edit callbacks, or local undo stack. Each native tab supplies one immutable
inspection receipt only when its installed Rust document projection and disposable scene
share revision/digest. The dock derives display text from those frozen facts and reuses
the already-owned atom/bond edit actions. Pending authoritative refresh produces an
explicit unavailable state rather than stale properties. A compact permanent test covers
active-tab selection behavior; the wide/narrow screenshots and action/widget inspection
were disposable checks rather than width, pixel, count, or wiring gates. The compatibility
host retains its separate OASA property dock until host cutover.

### 2026-08-14: insertion-valid native Paste

Ordinary `Edit -> Paste` now transfers from clipboard transport into one Rust-owned
document transaction. Rust owns the closed fragment grammar, named resource profile,
fresh persistent identifiers, exact declared-ID reference remapping, one group
translation, complete-candidate validation, history, and inserted-root receipts. The
private worker-safe PyO3 plan remains outside `.pyi`; Qt owns only one UI-thread clipboard
capture, cancellable preparation, current-tab/revision/digest delivery, authoritative
scene installation, and projected selection. The compatibility host retains its separate
OASA-backed Cut/Paste route until host cutover, so the direct-import count and dependency
declaration do not change.

### 2026-08-14: recoverable native Cut

Ordinary `Edit -> Cut` now composes one insertion-valid Copy fragment with an exact
source-authenticated Rust deletion plan. Structural Cut owns atom/bond topology cleanup,
generated-linear-form retirement, complete-root cleanup, projection validation, and one history
transition. Presentation Cut reuses the complete direct-root deletion owner. Qt owns worker
scheduling, current-tab/selection delivery, clipboard publication, and scene installation. It
publishes first, then commits, so a recoverable commit refusal produces a usable Copy result and
leaves the source unchanged. Mixed or multi-molecule complete-root Copy fallback has an explicit
Cut refusal with no ambiguous partial deletion. The private PyO3 seam remains outside the public
stub/CLI/wire surface. The compatibility host retains its separate OASA-backed clipboard route
until host cutover, so the direct-import count and dependency declaration stay unchanged.

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
private PyO3 entry remains outside `.pyi`, CLI, serde, and wire surfaces. The compatibility
host retains its OASA-backed selected-SVG route until cutover, so the direct-import count and
dependency declaration do not change.

## Closure gate

M16/M19/M22 may close this ledger only when every supported row names one Rust session or
non-document native owner, every other row has a recorded supported/known-defect/drop
decision, the compatibility host and orphaned callers are removed, production source has
zero OASA imports or dynamic loads, and a clean release install proves no OASA, Python
RDKit, or Tk runtime dependency. Oracle-only use remains outside production and must stay
explicitly isolated.
