# Native bond deletion V1

## Scope

This pre-milestone slice lets the standalone OASA-free native Qt window delete one
durably identified typed bond. Rust validates the identity, removes exactly that
direct molecule bond, reparses the retained document, and installs one revision and
one history entry. Neither endpoint atom is changed. Undo restores the same bond and
redo removes it again.

The Qt tab selects a current durable render target and submits only its bond identity.
It does not edit XML, endpoints, topology, or history. Opaque XML is preservation
content and cannot be targeted by this operation even when it contains a matching
reference-looking value. An unknown bond is rejected without changing revision,
history, or document content.

The native pointer-tool code was also separated from the window host. Disposable Draw
Bond and Move Atom pointer capture now live in a 379-line mixin, while the native main
window is 668 lines and remains responsible for actions, tabs, and status presentation.

This slice does not claim bond-order/style editing, deletion of molecule,
presentation, reaction, or opaque records, complete native editing, or M8a/M9/M16
completion.

## Validation

- `ferrum-document`: 96 tests passed, including selected-bond-only deletion,
  endpoint-atom preservation, opaque-content preservation, unknown-identity rejection,
  and one-entry undo/redo.
- Fresh direct-extension binding suite: 41 passed under CPython 3.12 with `-I -B`.
- Focused native tab/window suite: 31 passed against the fresh wheel.
- Full Ferrum-Qt suite: 915 passed, 1 skipped.
- Root repository suite: 5,685 passed; the function-typing, indentation, Bandit,
  and source-size policy subset reported 1,867 passed.
- Public installed-wheel native E2E deleted a selected generated bond while retaining
  both endpoint atoms, undid and redid the operation, saved and reopened the result,
  retained opaque XML, and reported `oasa_imported=false`.

The retained focused wheel is 3,488,056 bytes with SHA-256
`c41a19f2c5f8fd0d21429b900df0b1615324732c04753029c701e16276bb18a6`.
All 15 native libraries are byte-identical to the accepted atom-deletion wheel's RDKit
2026.03.5 closure; this document-mutation slice does not repeat the release-only
native relinking proof.
