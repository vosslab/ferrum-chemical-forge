# Native bond creation V1

## Scope

This pre-milestone slice lets the standalone OASA-free native Qt window connect
exactly two durable existing atoms in one molecule with a normal single bond, either
from explicit selection or through a revision-bound atom-to-atom drag. Releasing the
same gesture in empty space creates one carbon and its single bond as one complete
Rust candidate and one history entry. It does not claim bond-order/style editing,
gesture-time element choice, move/delete operations, or completion of M8a/M9/M16.

Rust owns endpoint validation, generated bond identity, the detached candidate,
the revision transition, history, projection, and serialization. Python translates
the selected authored atom IDs through the installed immutable Rust projection to
opaque document-object selectors; it does not parse CDML or construct selectors.
The drag path adds only a disposable Qt-local preview. It captures the exact snapshot
revision and digest, resolves both endpoints through the installed Rust projection,
and refuses to commit after any intervening document change.

For an empty-space release, Rust resolves the containing molecule, allocates both
identities together, inserts the atom at the exact finite scene point, adds its normal
single bond, validates the complete projection, and only then issues the token. One
undo removes both records; the frontend cannot expose an intermediate atom.

The Rust API carries a closed single/double/triple persistence vocabulary. The Qt
action deliberately chooses normal single because that is the exact bond form the
current native renderer paints. Other orders remain available to document callers
without being approximated by the Qt route.

## Validation

- `ferrum-document`: 90 tests; strict Clippy and rustdoc passed.
- Rust workspace: fmt, all-target check/test/strict Clippy, rustdoc, and macOS arm64
  all-target check passed.
- Fresh direct-extension wheel binding suite: 38 passed under CPython 3.12 with
  `-I -B`.
- Focused native tab/window suite: 25 passed against the fresh wheel.
- Full Ferrum-Qt suite: 909 passed, 1 skipped.
- Root repository suite: 5,689 passed.
- Public installed-wheel native E2E opened CDML, changed an atom, drove the actual
  atom-to-atom pointer gesture, created and selected `ferrum-bond-v1-0`, retired the
  preview, used undo/redo, imported CCO, saved/reopened, retained opaque XML, and
  reported `oasa_imported=false`. It then drove a second drag into empty space and
  saved/reopened `ferrum-atom-v1-0` with `ferrum-bond-v1-1`.

The retained focused wheel is 3,477,354 bytes with SHA-256
`cc423e245a57ce2e28dbeb3a06960aeb34e5e81fea7f4a342c0a827a91fa591f`.
It reuses the already accepted RDKit 2026.03.5 15-dylib closure; temporary build
targets and virtual environments are not retained.
