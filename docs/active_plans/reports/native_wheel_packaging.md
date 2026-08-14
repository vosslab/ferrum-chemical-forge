# Native wheel packaging proof

## Status

The M4a native-wheel packaging proof is complete for macOS arm64. It established the
development-platform distribution mechanism with ABI 1 and a two-library stub closure.
The table below is that historical M4a evidence. The current M4b receipt supersedes it
for the native chemistry adapter: ABI 2, a five-library closure, and semantic
kekulization before and after relinking are recorded in
[native_kekulization.md](native_kekulization.md). Neither proof establishes product
readiness or cross-platform support.

## Reproduce

From the repository root on macOS arm64, run:

```bash
source source_me.sh && PYTHONDONTWRITEBYTECODE=1 \
  python3 tests/e2e/e2e_native_wheel.py
```

On success, standard output is one JSON object with the initial and replacement
probe values and the evidence-receipt path. The test retains only the ignored local
generated JSON record at
`output_native_wheel/evidence/native-wheel-e2e-receipt.json`; it does not retain a
wheel, dylib, or other native build payload in the repository.

## Historical M4a result

The original M4a run of `tests/e2e/e2e_native_wheel.py` built the then-current broader
profile from hash-verified upstream sources,
installed the wheel in a scrubbed environment, and ran its adapter probe twice.

| Check | Result |
| --- | --- |
| Platform | macOS arm64 |
| Wheel | `ferrum_api-26.8.0-cp312-cp312-macosx_11_0_arm64.whl` |
| Wheel SHA-256 | `ea32bf843d5a8d85f2e51c4dde4d84fa1f7b50c0e6abb575a54c0e29ca73dea0` |
| ABI version | 1 |
| Initial/replacement probe | ABI 1 in separate fresh processes |
| Replacement library | `libferrum_chem.dylib` |
| Native closure | `libferrum_chem.dylib`, `libRDKitRDGeneral.1.dylib` |

The historical replacement probe proved that the installed extension found the
separately replaceable Ferrum-Chem library through its packaged loader configuration.
It did not exercise a chemical operation. Its test-only marker was removed before the
ABI-2 chemistry adapter was adopted; the current proof compares adapter digests and
reruns safe Rust chemistry instead.

The public Ferrum-Chem C header remains the single ABI-version authority. The C++
adapter returns that declared value, the builder derives it for Cargo, and Rust includes
the generated constant. A mismatch fails during the build or load boundary instead of
becoming a second hand-maintained version fact.

## Controlled native inputs

- RDKit `Release_2026_03_4`, SHA-256
  `a8bff65bdf13dd47a01f707f7759dd59124a8742f8c50952c2ceae9523b4fd2b`.
- Boost 1.91.0 headers, SHA-256
  `5734305f40a76c30f951c9abd409a45a2a19fb546efe4162119250bbe4d3a463`.
- CMake 4.4.2 and Homebrew LLVM/Clang 22.1.8 provide the FOSS build frontend.
- Rustup provides Cargo 1.97.1 and Rust 1.97.1.
- The Apple SDK and system linker are recorded as macOS platform inputs, not bundled
  dependencies.
- Maturin is intentionally unpinned. The successful receipt records Maturin 1.14.1.

The historical CMake profile disables Python RDKit, SWIG wrappers, compiled Boost
components, and unused feature families. The current GraphMol-kekulize profile is now
measured in a fresh source E2E: it builds only GraphMol into a Ferrum-owned sealed
stage, keeps only RDKit, configure-time Catch2, Better Enums, and header-only Boost,
and excludes InChI, CoordGen, and MAEParser from source acquisition and CMake paths.
Its ABI 2 semantic and relink evidence is in
[native_kekulization.md](native_kekulization.md). That E2E deliberately replaces the
wheel's `Release` adapter with a distinct-byte `RelWithDebInfo` adapter, then verifies
the same closure, ABI, and semantic result through fresh processes.
Downloads are hash verified, and every redirect hop is credential-free HTTPS. ZIP
extraction accepts only component-bounded regular files and directories; links,
special files, privileged mode bits, duplicate targets, and traversal paths fail
closed. Tar extraction uses Python's data filter plus component-boundary and
duplicate-target checks before extraction begins.
The current GraphMol-kekulize provenance audit examined 462 generated CMake files
and passed with only the declared build, toolchain, SDK, and macOS system locations
allowed.

## Evidence retention

The successful E2E publishes only
`output_native_wheel/evidence/native-wheel-e2e-receipt.json`. It contains the source
digests, toolchain and CMake provenance, wheel filename and digest, exact closure, and
before/after replacement probes. Paths below the discarded build root are represented
by the stable `${OUTPUT_ROOT}` placeholder rather than dead temporary paths. It is
ignored local generated evidence. The wheel and native libraries are temporary test
artifacts; they are not tracked repository files or a release payload.

## Deliberate limits

- The M4a historical result makes no chemistry API semantics, coordinate output, or
  parity claim. M4b adds only native kekulization semantics; its limits are stated in
  [native_kekulization.md](native_kekulization.md).
- Ferrum-Qt does not consume this backend yet.
- This does not qualify any platform beyond macOS arm64.
- This is not a desktop distribution or release.
- The temporary OASA dependency remains in the Qt migration preview.

The next chemistry step is coordinate parity and tolerance derivation. The platform
matrix and distributable desktop closure remain later work.
