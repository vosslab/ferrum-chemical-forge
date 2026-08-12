# Chemistry engine boundary

## Decision

Ferrum owns chemistry through a small safe Rust `ChemEngine` interface.  The only
native implementation is `NativeChemEngine`, which crosses a versioned Ferrum C ABI
to a private C++ adapter linked with RDKit.  No Rust crate outside that boundary,
Python package, or runtime workflow imports or links RDKit directly.

`MolGraph` is the structural exchange type.  It carries Ferrum-owned atoms, bonds,
coordinates, and optional atom facts; it never carries an RDKit object or handle.
The native layer converts the graph at the ABI boundary and returns a new
`MolGraph`, so Rust ownership and error handling remain explicit on both sides.

## Boundary contract

- The public header is the numeric authority for ABI version, status codes, wire
  limits, record widths, atom facts, bond orders, and operation options.
- The ABI exposes chemistry operations and their ownership rules only. Build labels
  are not exported: relink evidence records independently built adapter digests at
  the stable package-relative target, reloads the ABI in a fresh process, and reruns
  the safe Rust semantic operation. The E2E deliberately pairs a `Release` wheel
  adapter with a distinct-byte `RelWithDebInfo` replacement.
- ABI version 2 uses owned request and response byte buffers. The native caller owns
  the returned buffer until it releases it through the matching versioned free
  function.
- The Rust FFI crate loads an explicit library path, validates the adapter ABI before
  invoking an operation, and keeps the library handle and returned-buffer ownership
  local to the native engine.
- `NativeChemEngine` is deliberately not transferable across threads. This avoids
  silently assuming that the loaded native library and its ABI are thread-safe.
- The adapter rejects malformed, truncated, trailing, out-of-range, and
  semantically-invalid records at the boundary. It reports a structured failure;
  exceptions do not cross the C ABI.

## First operation

The first native operation is kekulization. Its Ferrum default is stated explicitly:

| Option | Default |
| --- | --- |
| `clear_aromatic_flags` | `false` |
| `canonical` | `true` |
| `max_backtracks` | `100` |

Before conversion, the operation accepts only the aromatic input representation:
aromatic bonds use the aromatic bond order, carry the aromatic flag, and join
aromatic atoms. After successful kekulization, the same topology and atom facts are
retained while the aromatic bonds become alternating single and double bonds. Whether
aromatic flags remain is controlled only by `clear_aromatic_flags`.

This is a contract for the current operation, not a promise that every future RDKit
entry point shares its defaults. Each new operation states its own defaults and
validates its own output invariants.

## Packaging boundary

The native-wheel input manifest is version 2 and seals the specific installed RDKit
and Boost-header trees used to rebuild the adapter. It records the profile, source
archive facts, required headers, and native-library aliases and digests. A replacement
adapter build must validate that manifest before it reuses the native input tree.

For the kekulization adapter, the expected packaged macOS closure is exactly:

- `libferrum_chem.dylib`
- `libRDKitGraphMol.1.dylib`
- `libRDKitRDGeometryLib.1.dylib`
- `libRDKitDataStructs.1.dylib`
- `libRDKitRDGeneral.1.dylib`

Compiled Boost libraries, Python RDKit, SWIG, Boost.Python, NumPy, and every path
under `OTHER_REPOS` are outside this product boundary. Boost headers are a controlled
build input only; they are not a shipped dynamic dependency.

## Consequences

The adapter is replaceable for LGPL relinking without widening the Rust or Python
surface. Adding a native operation requires an intentional ABI and `ChemEngine`
design review, a stated default, wire validation, semantic tests, and package-closure
review. It may not bypass the boundary for convenience.

Coordinate generation is separate work. The recorded `canonOrient` measurement
selects `true` for a future Ferrum layout operation, but it neither implements that
operation nor establishes a coordinate tolerance. Its one-time isolated oracle tool
is [`devel/rdkit_layout_orientation.py`](../../../devel/rdkit_layout_orientation.py),
not a pytest case.
