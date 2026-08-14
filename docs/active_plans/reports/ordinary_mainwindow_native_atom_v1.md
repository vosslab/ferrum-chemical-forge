# Ordinary MainWindow native atom editing slice

## Scope

The ordinary Ferrum-Qt `MainWindow` now offers File > Open CDML with Ferrum. The
explicit route validates and opens one `.cdml` document as a separate Rust-native tab;
it does not replace the ordinary window's default Open route.

Within an active native tab, Edit > Change Element with Ferrum and Edit > Edit Atom
Properties with Ferrum each require exactly one durably selected atom. Rust validates
the element or closed property patch, selection, revision, and mutation, then publishes
the replacement observation. The property route only accepts values the shared dialog
can represent exactly: a fractional source value for an integer-only control, for
example, fails visibly before mutation instead of being rounded or remapped. Cancelling
the dialog is a no-op. Edit > Undo with Ferrum and Edit > Redo with Ferrum navigate
only that tab's revision-checked Rust session history and install the returned
authoritative projection. Empty history reports a typed failure without mutation. File
> Save and Save As use that tab's Rust publication boundary, and closing it retires its
native lifecycle.

Edit > Set Atom Number with Ferrum... also requires exactly one selected durable atom. It
sends a typed positive number and explicit show-number state to Rust, then installs the
returned projection. Cancelling its dialog is a no-op.

Edit > Clear Atom Number with Ferrum is a separate action. It becomes available only for
one selected durable atom with an authored number. Set can leave an authored number
hidden; Clear instead removes the complete durable number/show-number pair through one
revision-bound Rust operation, installs the returned projection, retains selection, and
then disables the action.

Edit > Delete Selected Atom with Ferrum is separately available for exactly one selected
durable atom. Rust validates the durable target and revision, atomically deletes that atom
and its directly typed incident bonds in one revision-bound operation, then installs the
replacement projection. The selection clears with that projection and disables the action;
Undo with Ferrum restores the prior Rust-owned topology. This is an atom deletion route,
not a bond-deletion bundle.

Edit > Delete Selected Bond with Ferrum is separately available for exactly one selected
durable bond. Rust validates the durable target and revision, then removes exactly that
bond in one revision-bound operation while retaining both endpoint atoms. The returned
projection clears selection and disables the action; Undo with Ferrum restores the bond
from Rust history. The action has no shortcut and does not bundle atom deletion.

Edit > Edit Bond Properties with Ferrum requires exactly one selected durable bond. It
uses the existing frozen-projection, capability-limited BondDialog to submit one
revision-bound Rust patch, installs the returned projection, and retains that durable
bond selection. The current native profile exposes normal single, double, and triple
bond semantics with only renderer-supported width and centering combinations. A source
fact that the dialog cannot represent losslessly fails visibly before mutation, and
cancelling is a no-op.

## Ownership and failures

The chooser path and element text are untrusted frontend input. The native boundary
owns CDML validation, durable selection, revision checks, mutation, and publication.
The tab neither aliases the legacy document session nor falls back to it for mutation
or saving. Atom-number set and clear likewise cross only the typed Rust boundary. A
cancelled chooser or non-CDML choice leaves tabs and sessions unchanged. Atom deletion
also crosses only that boundary: typed deletion failures remain visible without a legacy,
OASA, or local-scene fallback.
Bond deletion crosses only that Rust boundary as well: typed failures remain visible
without a legacy, OASA, or local-scene fallback.
Bond-property input also crosses only that Rust boundary; it does not fall back to
OASA when a selected fact or requested form is unsupported.

The ordinary `MainWindow` still imports and hosts the OASA-backed legacy editor. Its
default Open `.cdml` path and legacy property dock/actions remain legacy, and its other
session actions are not part of this slice. Selecting a legacy tab disables the explicit
Ferrum Undo/Redo controls and returns existing legacy Undo/Redo policy to the legacy
session. This is therefore partial M16 adoption, not an OASA-free ordinary window or a
completed desktop migration.

## Evidence

Permanent offline behavior tests cover the explicit route, legacy-route preservation,
selection-sensitive element and properties actions, semantic Rust edit-history undo and
redo, the native-to-legacy page transition, empty-history containment, invalid/cancelled
chooser and property-dialog containment, visible rejection of a lossy property
projection, accepted atom-number/show-number mutation, clear-pair removal with retained
selection, cancelled number-dialog containment, public-action atom deletion with its
incident bond and native Undo restoration, public-action bond deletion with retained
endpoint atoms and native Undo restoration, native Save/Save As dispatch, and tab lifecycle.
They use inline CDML and `tmp_path`; they do not require network access, timing, pixels,
bytes, private-worker wiring, or a real wheel.

The same permanent offline coverage includes native-bond action selection, accepted
representable mutation and selection retention, cancellation, and visible lossless
rejection. A current-extension exercise of that ordinary-window route was disposable
implementation evidence, not a pixel, byte, timing, network, or private-wiring gate.

The final independent review accepted the focused public-action atom deletion coverage:
one selected atom and incident bond disappear, selection/action state clears, and native
Undo restores topology. The implementation's temporary-extension run (26 focused tests)
and static review are disposable evidence, not permanent byte, pixel, timing, network, or
private-wiring gates. Earlier current-wheel ordinary-window and atom-number exercises are
likewise disposable implementation evidence.

## Follow-on work

Complete M16 by moving the remaining ordinary document classes, actions, recovery,
and legacy session ownership to Ferrum-Chem. Track the broader inventory in
[ferrum-plan-v3.md](../ferrum-plan-v3.md) and
[ferrum_qt_capability_matrix.md](../audits/ferrum_qt_capability_matrix.md).
