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
- ABI version 4 uses owned request and response byte buffers. The native caller owns
  each returned buffer until it releases it through the matching versioned free
  function. FCM1 is the strict molecule response for SMILES, including canonical
  text, complete atom and bond facts, and atom-order-aligned finite coordinates.
- The Rust FFI crate loads an explicit library path, validates the adapter ABI before
  invoking an operation, and keeps the library handle and returned-buffer ownership
  local to the native engine.
- `NativeChemEngine` is deliberately not transferable across threads. This avoids
  silently assuming that the loaded native library and its ABI are thread-safe.
- The adapter rejects malformed, truncated, trailing, out-of-range, and
  semantically-invalid records at the boundary. It reports a structured failure;
  exceptions do not cross the C ABI.

## Current operations

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

ABI 4 also supplies complete SMILES parsing, deterministic 2D generation, and bounded
V2000/V3000 and SDF import/export. The
adapter calls `SmilesToMol`, canonical `MolToSmiles`, and RDKit depiction with:

| Option | Value |
| --- | --- |
| `canonOrient` | `true` |
| `clearConfs` | `true` |
| `forceRDKit` | `true` |
| random samples | none |
| ring templates | disabled |

The adapter advertises only the implemented capability subset: kekulization, SMILES
molecule parsing, 2D generation, SMARTS export, molblock import/export, and SDF
import/export. The loader rejects unknown capability bits and checks the required bit
before every call.

## Packaging boundary

The native-wheel input manifest is version 3 and seals the specific installed RDKit
and Boost-header trees used to rebuild the adapter. It records the profile, source
archive facts, required headers, and native-library aliases and digests. A replacement
adapter build must validate that manifest before it reuses the native input tree.

For the current ABI-4 chemistry adapter, the expected packaged macOS arm64 closure is
exactly 15 libraries:

- `libferrum_chem.dylib`
- `libRDKitAlignment.1.dylib`
- `libRDKitChemTransforms.1.dylib`
- `libRDKitDepictor.1.dylib`
- `libRDKitEigenSolvers.1.dylib`
- `libRDKitFileParsers.1.dylib`
- `libRDKitGenericGroups.1.dylib`
- `libRDKitGraphMol.1.dylib`
- `libRDKitMolAlign.1.dylib`
- `libRDKitMolTransforms.1.dylib`
- `libRDKitRDGeometryLib.1.dylib`
- `libRDKitDataStructs.1.dylib`
- `libRDKitRDGeneral.1.dylib`
- `libRDKitSmilesParse.1.dylib`
- `libRDKitSubstructMatch.1.dylib`

Compiled Boost libraries, Python RDKit, SWIG, Boost.Python, NumPy, and every path
under `OTHER_REPOS` are outside this product boundary. Boost headers are a controlled
build input only; they are not a shipped dynamic dependency.

The source version is rolling rather than a permanent compatibility pin. A release
build selects the latest official stable RDKit tag, records that exact tag and archive
SHA-256 for reproducibility, and compares codec semantics with the previous stable
release. An older tag is not retained merely because a cached build exists.

## Consequences

The adapter is replaceable for LGPL relinking without widening the Rust or Python
surface. Adding a native operation requires an intentional ABI and `ChemEngine`
design review, a stated default, wire validation, semantic tests, and package-closure
review. It may not bypass the boundary for convenience.

The native operation generates coordinates, and M4c now records exact same-platform
parity plus a ULP-derived tolerance in `../reports/coordinate_parity_v1.md`. M20
retains future platform expansion. The one-time orientation tool remains
[`devel/rdkit_layout_orientation.py`](../../../devel/rdkit_layout_orientation.py),
not a pytest case.

The document insertion writer is narrower than the ABI molecule model. It stores
elements, finite 2D points, nonzero formal charge, isotope, nonzero explicit hydrogen
count, and single/double/triple bonds. Aromatic molecules are explicitly kekulized
before persistence. Chirality, bond stereo and direction, radicals, no-implicit
policy, atom maps, stereo references, unresolved aromaticity, and quadruple bonds
are rejected before session mutation until their exact CDML mappings and round trips
are proven. These are current writer-contract gaps, not claims that CDML itself is
incapable of representing the concepts.
