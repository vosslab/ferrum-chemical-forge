# Native Molfile import V1

## Outcome

The standalone OASA-free `ferrum-qt --native` window can import one local
V2000 or V3000 Molfile into the current Rust-owned CDML session. Python passes
only a file path and an immutable insertion placement. Rust owns the bounded
read, UTF-8 admission, molblock validation, RDKit call, graph conversion,
durable ID allocation, revision check, document mutation, and resulting
projection.

This slice follows the ownership rules in
[CDML_BACKEND_TO_FRONTEND_CONTRACT.md](../../CDML_BACKEND_TO_FRONTEND_CONTRACT.md)
and the untrusted-input posture in
[CDML_FORMAT_SPEC.md](../../CDML_FORMAT_SPEC.md). Ferrum-Qt never parses the
Molfile and never constructs persistent molecule facts.

## Security boundary

- The local Molfile is untrusted input.
- Rust opens the path and verifies the opened handle is a regular file.
- A read-only symbolic link is permitted; authority follows the opened regular
  handle rather than a race-prone pre-open path classification.
- Rust reads at most the ABI-4 molblock operation limit plus one sentinel byte.
  The limit is the existing native operation capacity, not a newly invented
  desktop threshold.
- The byte ceiling is checked before UTF-8 conversion and before RDKit loads.
- Existing molblock lexical validation runs before the native adapter loads.
- RDKit remains reachable only through the narrow `ChemEngine` implementation.
- The complete returned graph is owned Rust data. Unsupported atom, bond,
  coordinate, stereo, or aromatic facts fail before a document candidate exists.
- The prepared insertion is immutable and handle-free. The document session
  accepts it only against the captured revision and digest.
- Malformed input, stale intent, and conversion failure leave the document
  revision, digest, dirty state, and history unchanged.

The path currently rejects nonregular handles but does not claim a stronger
filesystem sandbox. A deployment that needs directory confinement must provide
that policy outside this format operation.

## Qt lifecycle

Molfile and SMILES preparation share one import-intent controller. A small
UI-thread `QObject` relay receives worker result, failure, and completion
signals. It passes the exact emitting worker to the controller, so a stale or
replacement worker cannot release or commit another intent. Cancel invalidates
delivery without claiming to preempt a native library call already executing.
Tabs remain uncloseable until native teardown finishes.

## Grounded verification

Rust tests cover complete graph placement, unsupported facts, the one-byte
limit sentinel, and invalid UTF-8. Binding tests use a real packaged adapter and
cover a valid file, typed invalid UTF-8, exact built-in path type, and a stale
prepared commit with no mutation. Qt tests cover worker conversion, invalid
input with no document mutation, public menu import, render, save, and reopen.
The existing SMILES worker suite runs beside the Molfile suite to protect the
shared lifecycle controller.

The focused direct CPython 3.12 wheel has SHA-256
`aefe9789f582b710c047bd1e570df424a510220886c0c3a2f2eb74c8fcab5232`.
It contains the current root extension and the previously accepted RDKit
2026.03.5 15-library closure. This is a source-bound test artifact, not a new
cross-platform release receipt.
The retained wheel is under
`output_native_wheel/native-molfile-import-v1-20260812/wheelhouse/`; its
disposable 415 MB staging tree and virtual environment were removed after the
installed tests passed.

Acceptance is semantic: atom and bond facts, coordinates, durable session
mutation, revision behavior, and save/reopen are checked. There is no Molfile
byte-equivalence, CDML byte-equivalence, pixel-equivalence, or elapsed-time gate.

## Boundary

This is bounded evidence for M5 codec reuse and native document adoption. It
does not complete M5, M8a, M16, M20, or OASA removal. The normal Ferrum-Qt
window remains on the legacy editor route, and other imported chemistry formats
need their own explicit admission and document-conversion contracts.
