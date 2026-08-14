# Native atom deletion V1

## Scope

This pre-milestone slice lets the standalone OASA-free native Qt window delete one
durably identified atom. Rust removes the atom and every direct typed molecule bond
whose start or end identity names that atom. The candidate is installed as one
revision and one history entry, so undo restores the atom and all incident bonds
together and redo removes them together.

The document layer owns identity validation, structural mutation, reparse,
projection, history, serialization, and preservation. Qt supplies only the currently
selected durable atom identity. Reference-looking content in opaque XML remains
untouched because it is preservation data rather than a typed bond endpoint.
An unknown atom is rejected without changing revision, history, or document bytes.

This slice does not claim bond deletion, deletion of arbitrary presentation or
document objects, bond-order/style editing, complete reaction-reference handling,
complete native editing, or M8a/M9/M16 completion.

## Validation

- `ferrum-document`: 94 tests passed, including exact incident-bond removal,
  unknown-identity rejection, one-entry undo/redo, and opaque-content preservation.
- Fresh direct-extension binding suite: 40 passed under CPython 3.12 with `-I -B`.
- Focused native tab/window suite: 29 passed against the fresh wheel.
- Full Ferrum-Qt suite: 913 passed, 1 skipped.
- Root repository suite: 5,685 passed; the function-typing, indentation, and Bandit
  policy subset reported 1,209 passed.
- Public installed-wheel native E2E deleted `atom-c` and its incident generated bond,
  preserved a separate bond, undid and redid the operation, saved and reopened the
  result, retained opaque XML, and reported `oasa_imported=false`.

The retained focused wheel is 3,483,175 bytes with SHA-256
`94b1c57278c73b909929b4f6c8ea10a0f69d0586d8d01f5ef617bc16a460b46f`.
All 15 native libraries are byte-identical to the accepted atom-move wheel's RDKit
2026.03.5 closure; this document-mutation slice does not repeat the release-only
native relinking proof.
