# ABI-4 coordinate parity

## Verdict

M4c passes for the currently supported macOS arm64 native-wheel proof. The recorded
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

The measured direct-extension wheel is
`output_native_wheel/molblock-import-v1-rdkit-2026035-20260812/wheelhouse/` followed by
`ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl`. Its SHA-256 is
`13de57cf0d95dc3f1755f14a1ca36350fe4db7dca43e3ab8ead0e3d0e74b3eda`.

The receipt records the wheel, installed extension, RDKit Python binary, adapter,
public header, Rust decoder, generator, and child-script digests. The repository test
recomputes every source digest, so source drift invalidates this measurement.

## Reproduction

Use an isolated Python 3.12 environment containing exactly `rdkit==2026.3.5` and a
second environment containing the measured Ferrum wheel. From the repository root:

```bash
source source_me.sh
python3 -B devel/measure_coordinate_parity.py \
	--oracle-python <rdkit-2026.03.5-venv>/bin/python \
	--ferrum-python <ferrum-wheel-venv>/bin/python \
	--wheel output_native_wheel/molblock-import-v1-rdkit-2026035-20260812/wheelhouse/\
ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl \
	--repeats 20
```

## Limits

This closes M4c on macOS arm64. M20 still owns additional platform measurements.
The M5 SMARTS export, molblock import/export, and bounded SDF import/export slices are
now green. SMARTS import and InChI remain open. Exact CDML mappings for
chirality, stereo, radicals, atom maps, no-implicit policy, stereo references, and
quadruple bonds remain deliberately rejected writer-contract gaps.
