# Native atom number V1

## Outcome

The standalone OASA-free native window can assign, hide, show, and clear a
persistent atom number for one durably selected direct atom. Ferrum-Chem owns
the CDML mutation, revision, history, projection, and render facts. Ferrum-Qt
collects two scalar values and installs the accepted observation.

This is a bounded FQ-017a replacement slice. Chemical atom marks remain a
separate open capability, and the legacy application route still uses its
historical session.

## Closed operation

The Rust operation names one durable direct-root molecule ID and one direct
atom ID. Assignment requires a positive `u64` and an explicit boolean
visibility value. Clear supplies neither value. An accepted assignment changes
only `number` and `show_number`; clear removes both attributes.

The operation is revision-bound through `DocumentSession.submit`. Rust builds
and reparses a detached candidate before appending history. A matching pair and
an empty clear are semantic no-ops. A stale revision, malformed pair, missing or
mismatched target, or direct legacy `<mark type="atom_number">` produces a
typed failure without changing the snapshot.

## Rendering and Qt ownership

The immutable atom projection exposes a validated positive number and authored
visibility separately. A visible number becomes a second atom-local Rust text
operation with verified Telex glyph IDs and origins, explicit size `9`, RGB
`0000c8`, z value `40`, and local origin `(8, -12)`. A hidden number stays in
the document projection but produces no number operation. Invalid authored
number or visibility text creates a presentation issue; Qt never repairs,
shapes, advances, recolors, or otherwise infers the label.

The native Qt form uses a text field so it does not inherit an arbitrary
spin-box ceiling. It accepts the complete positive `u64` protocol range and
rejects signs, whitespace, leading zeroes, fractions, and overflow before
submission. Set and Clear actions require one durable selected atom. The
accepted atom remains selected after scene replacement.

## Security review

Trust boundary: Python supplies two IDs, one integer, and one boolean to a
closed PyO3 factory. PyO3 requires an exact Python integer that is not a boolean,
an exact boolean, and the Rust `u64` range. Rust revalidates the positive/clear
pair and resolves the atom only as a direct child of the named direct-root
molecule.

Validation owner: Ferrum-Chem owns target eligibility, legacy-mark compatibility,
the detached XML mutation, candidate reparse, revision check, and history append.
Ferrum-Qt owns only form validity and disposable selection.

Resource limits: the request has a fixed two-scalar payload and performs one
bounded direct-root molecule/atom search over the already admitted retained
document. It introduces no new external parser, compression, network, or FFI
path. Existing explicit CDML admission budgets remain the input-size boundary.

Expected failures: malformed Python types fail before operation construction;
invalid pairs and targets fail before mutation; stale revisions fail before
candidate preparation; projection or rendering failure retains the accepted
result in the native tab's existing refresh-required state.

Adversarial coverage includes boolean-as-integer, zero, negative, overflow,
mixed nullable Rust pairs, molecule/atom mismatch, stale revision, direct legacy
number mark, alternate canonical namespace prefix, and malformed authored
presentation text.

## Evidence

- Five focused document tests cover assignment, hidden visibility, clear,
  no-op, undo/redo, target mismatch, legacy compatibility, namespace handling,
  and opaque-content retention.
- Render and API tests prove the visible operation and hidden/invalid behavior
  semantically without pixel comparison.
- A fresh direct-extension wheel passed the installed binding test and the
  public OASA-free Qt E2E. The E2E assigned, hid, cleared, undid/redid,
  saved/reopened, retained opaque content and selection, and observed explicit
  Rust number glyph facts.
- The fresh-wheel receipt reported `clean=true`, `number=42`,
  `show_number=true`, and `oasa_imported=false`.

The temporary wheel and test environment were deleted after the proof. This
report intentionally does not claim a shipping artifact hash or release proof.
