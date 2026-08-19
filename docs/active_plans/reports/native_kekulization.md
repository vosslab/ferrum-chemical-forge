# Native kekulization status

## Status

M4b is complete for the narrow native kekulization operation on macOS arm64. The
source E2E built, installed, exercised, rebuilt, and replaced the adapter using the
sealed input manifest. It proved the same semantic result through fresh Rust processes
before and after replacement. This is not Qt adoption, CDML integration, broader RDKit
API coverage, coordinate parity, cross-platform support, or a product release.

## Scope

Ferrum exposes kekulization through the safe Rust `ChemEngine` seam. The native
implementation converts only Ferrum `MolGraph` records over the versioned C ABI;
RDKit objects do not leave the private C++ adapter. The complete design is recorded
in [chemistry_engine_boundary.md](../decisions/chemistry_engine_boundary.md).

The operation reproduces the stated call equivalent to:

```text
Kekulize(clearAromaticFlags=false, canonical=true, maxBackTracks=100)
```

Those values are explicit rather than inherited from a language wrapper. Input is a
strict aromatic graph. Success preserves atom facts and topology, converts aromatic
bonds to an alternating single/double assignment, and applies aromatic-flag clearing
only when requested.

## Native package design

ABI version 2 uses Ferrum-owned serialized request and response records with an
explicit matching free function. The public C header is the sole numeric authority;
Rust validates the header during its build and the dynamic loader validates the
adapter version before a call.

The version-2 native-input manifest seals the profile and exact materialized native
inputs before a replacement adapter build. It rejects changed headers, library aliases
or targets, profile drift, unsupported file types, path escape, and external source
trees. The installed package contains only the five-library closure listed in the
boundary decision. It contains neither compiled Boost nor a Python RDKit runtime.

## Measured native profile

The measured `ferrum-rdkit-graphmol-kekulize-v1` profile builds only the GraphMol
target required for kekulization into a Ferrum-owned sealed stage; it does not produce
or reuse a general RDKit installation. Its source inputs are RDKit, configure-time
Catch2, Better Enums required to generate the GraphMol enum header, and header-only
Boost. InChI, CoordGen, and MAEParser are disabled and absent from source acquisition,
materialization, and CMake paths. The receipt's 462-file CMake provenance audit passed.

## Acceptance evidence

The source E2E builds the declared native profile, creates a wheel, installs it into a
scrubbed environment, and runs a fresh Rust process against the installed adapter.
The process checks an aromatic benzene graph carrying representative optional atom
facts, including an explicit zero formal charge. It requires the expected alternating
bond orders while preserving topology, atom facts, and the requested aromatic-flag
behavior.

The E2E then rebuilds and replaces only `libferrum_chem.dylib` using the sealed input
manifest and repeats the same fresh-process semantic probe. The final receipt therefore
demonstrates semantic behavior before and after the LGPL relink route, not merely that
an extension can load a test-only adapter symbol.

| Receipt field | Verified value |
| --- | --- |
| Evidence schema | `ferrum-native-wheel-e2e-evidence-v2` |
| Platform | macOS arm64 (`aarch64-apple-darwin`) |
| Wheel | `ferrum_api-26.8.0-cp312-cp312-macosx_11_0_arm64.whl` |
| Wheel SHA-256 | `361bc2a3ca3bc63b4383664af4b427e4a22554cbd223fdc3e574d83da4184558` |
| Adapter ABI | 2 |
| Before and after probes | ABI 2 in fresh isolated processes |
| Replacement proof | Stable role, verified package-relative copy, closure, ABI reload, semantics |
| Adapter builds | Distinct-byte values listed below |
| Packaged closure | Five libraries listed below |

- Wheel adapter: `Release`, SHA-256
  `5735eca9625a98b2cf0e83a9b59b90e8657321cf735318ac2daf3abec76bb1b8`
- Replacement adapter: `RelWithDebInfo`, SHA-256
  `10bce4a574f522f7dfa5ed2d949721ace9712afb32de92b43993f7b5449740c4`

- `libferrum_chem.dylib`
- `libRDKitGraphMol.1.dylib`
- `libRDKitRDGeometryLib.1.dylib`
- `libRDKitDataStructs.1.dylib`
- `libRDKitRDGeneral.1.dylib`

Both semantic probes transformed the six aromatic benzene bonds into three single and
three double bonds in alternating order. They preserved every atom fact supplied by the
probe, including formal charge `0`, isotope `13`, and explicit hydrogen count `1`, as
well as its six-atom/six-bond topology and aromatic flags for the default operation.

The native builder supports both `Release` and `RelWithDebInfo`. The E2E deliberately
builds the wheel adapter as `Release` and its replacement as `RelWithDebInfo`, requiring
their recorded SHA-256 values to differ before it copies the replacement at the stable
package-relative library role. The closure validation, fresh ABI load, and semantic
probe then establish that these distinct compatible bytes, rather than a re-copied
identical library, satisfy the relink route.

The manifest reports schema `ferrum-native-inputs-v2`, policy digest
`82dc109f4c0ed21d000b9aee72ea0e432beff67aa3dd689a6bd9e0d3719789a6`, and tree
digests `fe80508578560ac3f893dba5ce933ada756d975868c685c115c1012f6327372b` for
the RDKit includes and `dfe93ad7351b663832673fa03da52b7adfa612e7b4d5151e84deda492389c5b9`
for Boost headers. It separately hashes the required `GraphMol/MolOps.h` and
`RDGeneral/types.h` headers plus the GraphMol and RDGeneral library aliases and their
resolved targets. This seals the build inputs used for the replacement proof; it does
not make them tracked package payloads.

The generated receipt stays ignored at
`output_native_wheel/evidence/native-wheel-e2e-receipt.json`. It is development
evidence, not a tracked wheel or native library.

## Deliberate limits

- Ferrum-Qt does not consume this operation yet.
- SMILES, SMARTS, molblock, SDF, and InChI belong to the later codec milestone.
- Layout is not implemented here. The `canonOrient` result in
  [rdkit_layout_orientation.json](rdkit_layout_orientation.json) is archived one-time
  evidence selecting `true` for a future layout call; it is deliberately outside
  pytest and has no live Python-RDKit measurement route.
- The coordinate tolerance and parity gate remain M4c work.
- The package proof is currently scoped to macOS arm64 development evidence, not a
  supported cross-platform desktop release.
