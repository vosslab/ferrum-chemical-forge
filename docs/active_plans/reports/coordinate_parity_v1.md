# ABI-4 coordinate parity

## Verdict

M4c passed for the historical macOS arm64 wheel proof. The recorded
RDKit 2026.03.5 Python wrapper and the ABI-4 FCM1 wheel produced exactly equal x/y
coordinates for every atom in six molecules. Five cases are asymmetric; benzene is
retained only as a symmetric control.

The machine-readable receipt is
[`coordinate_parity_v1.json`](coordinate_parity_v1.json).

## Measurement

Each backend ran in 20 independent Python processes. Both used explicit RDKit
depiction options: canonical orientation, cleared conformers, forced RDKit depiction,
no random sampling, and no ring templates.

| Measurement | Result |
| --- | ---: |
| Oracle process noise | 0.0 |
| Ferrum process noise | 0.0 |
| Largest Python-wrapper versus Ferrum delta | 0.0 |
| Largest measured coordinate ULP | 8.881784197001252e-16 |
| Derived maximum absolute tolerance | 7.105427357601002e-15 |

The tolerance is `max(4 * observed process noise, 8 * maximum coordinate ULP)`.
This places the gate outside the measured zero process noise without inventing a
decimal threshold unrelated to the represented values.

| Case | Atoms | Asymmetric | Maximum delta |
| --- | ---: | --- | ---: |
| Asymmetric amide | 7 | yes | 0.0 |
| Caffeine | 14 | yes | 0.0 |
| Ibuprofen | 15 | yes | 0.0 |
| Branched octane | 8 | yes | 0.0 |
| Bridged ring | 7 | yes | 0.0 |
| Benzene control | 6 | no | 0.0 |

## Artifact

The measured direct-extension wheel was
`output_native_wheel/molblock-import-v1-rdkit-2026035-20260812/wheelhouse/` followed by
`ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl`. Its SHA-256 is
`13de57cf0d95dc3f1755f14a1ca36350fe4db7dca43e3ab8ead0e3d0e74b3eda`.

The receipt recorded the wheel, installed extension, RDKit Python binary, adapter,
public header, Rust decoder, generator, and child-script digests. The wheel output root
and its measurement harness were retired, so this artifact is not reproducible in the
current checkout. A new measurement must be explicitly scoped; current local builds use
`./build.sh` and its staged runtime.

## Historical evidence

The isolated Python-RDKit generator and child were retired after this accepted
one-time receipt. The recorded wheel, source digests, corpus, process count, and
derived tolerance remain archival evidence rather than a permanent Python-RDKit or
CI dependency. A future platform measurement requires an explicitly scoped Ferrum
release-evidence plan and a fresh accepted receipt.

## Limits

This closes M4c on macOS arm64. M20 still owns additional platform measurements.
The M5 SMARTS export, molblock import/export, and bounded SDF import/export slices are
now green. SMARTS import and InChI remain open. Exact CDML mappings for
chirality, stereo, radicals, atom maps, no-implicit policy, stereo references, and
quadruple bonds remain deliberately rejected writer-contract gaps.
