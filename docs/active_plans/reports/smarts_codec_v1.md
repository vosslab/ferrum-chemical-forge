# SMARTS Codec V1 Evidence

## Result

The bounded SMARTS export slice is green on macOS arm64 against the wheel's recorded
RDKit 2026.03.5 build and has equivalent query behavior under the previous stable
RDKit 2026.03.4 release. This completes the SMARTS part of M5.

Ferrum parses SMILES into its frozen, handle-free molecule DTO, serializes the complete
graph through ABI-4 FCG1, reconstructs an RDKit molecule inside the C++ adapter, and
returns `MolToSmarts` output through the bounded FCT1 text response. The same operation
is exposed as:

- `ferrum_chem.molecule_to_smarts(molecule)` for an exact frozen
  `SmilesMoleculeV1`; and
- provisional `ferrum smiles to-smarts --adapter ABSOLUTE_LIBRARY SMILES`.

The CLI retains the existing explicit-adapter policy: the adapter must be an absolute,
regular, non-symlink file. It does not search the environment, current directory, or
Python wheel layout.

## Grounded comparison rule

SMARTS text is exact only within one recorded RDKit build. That rule is justified
because both sides invoke the same deterministic RDKit writer over equivalent discrete
graph facts. Canonical atom ranking may change in another RDKit release, so the
cross-version check compares query matches rather than bytes.

This slice does not compare coordinates: SMARTS is a graph/query representation and the
FCG1 request deliberately omits depiction coordinates. It also sets no time or memory
budget. Those would be unrelated to codec correctness and lack a measured baseline.

## Differential corpus

[`smarts_codec_v1.json`](smarts_codec_v1.json) records eight same-build cases:

- single, double, triple, and aromatic bonds;
- disconnected charged atoms;
- directional bond stereo;
- isotope, tetrahedral chirality, formal charge, and atom mapping; and
- an explicit methylene case that verifies Ferrum matches RDKit's own SMARTS
  normalization even when the output does not spell every input fact.

All eight canonical SMILES strings and all eight SMARTS strings matched exactly. The
receipt binds the Python RDKit binary, installed Ferrum extension, wheel, implementation
sources, and test drivers by SHA-256.

A separate disposable, offline compatibility check generated the same eight queries
under RDKit 2026.03.4 and 2026.03.5. Both releases parsed both query sets and evaluated
them with chirality enabled against 17 positive and distinguishing target molecules.
The releases agreed on all 272 query-target truth values, and every query matched its
source molecule. The SMARTS strings happened to be equal across these two releases,
but that observation is not the compatibility requirement.

## Installed-wheel and relink proof

The current direct-extension wheel is:

`output_native_wheel/inchi-v1-current-20260813/wheelhouse/`
`ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl`

Its SHA-256 is
`0f2de3ae9819545846af46efc45cae3eddbfbcabda5a0653f31d2a4ff6e79e6f`.
The current wheel has the measured 18-library closure recorded by the InChI slice.
SMARTS itself does not depend on molfile or InChI formatting.

The source E2E installed the direct wheel into a clean Python 3.12 environment and
obtained `[#6]-[#6]-[#8]` for CCO. It then replaced the packaged Release adapter with a
distinct-byte RelWithDebInfo adapter and obtained the same result again. The original
and replacement adapter digests are recorded in the JSON receipt.

The installed binding suite includes exact-class input enforcement. A duck-typed
Python object cannot be passed in place of the frozen Ferrum molecule.

## OASA parity boundary

The read-only OASA registry intentionally describes SMARTS as export-only, exposes
only molecule-to-text and molecule-to-file operations, and tests that the codec does
not read files. A SMARTS importer is therefore a new query-language feature rather
than replacement parity. It is not an M5 requirement.

Bounded V2000/V3000 molblock, ordered 2D SDF, and Standard/Fixed-H InChI behavior are
separately green in [`molblock_codec_v1.md`](molblock_codec_v1.md),
[`sdf_codec_v1.md`](sdf_codec_v1.md), and `inchi_codec_v1.md`.
Those formats use their own grounded comparison rules rather than SMARTS bytes.

## Reproduction

Build the wheel from its exact hash-verified source archives, run the native-wheel
E2E, install Python RDKit 2026.03.5 in an isolated environment, then run the retained
same-build comparison:

```bash
source source_me.sh
python3 -B devel/measure_smarts_codec_parity.py \
	--oracle-python <rdkit-2026.03.5-venv>/bin/python \
	--ferrum-python <fresh-ferrum-wheel-venv>/bin/python \
	--native-e2e-receipt output_native_wheel/evidence/native-wheel-e2e-receipt.json \
	--wheel <fresh-smarts-wheel.whl>
```

The cross-version check was one-time release evidence rather than a permanent test.
It generated the eight report queries independently under RDKit 2026.03.4 and
2026.03.5, parsed both query sets under both releases, and compared chirality-aware
substructure-match truth across the 17-target corpus. It used no network, timing, or
exact cross-version text assertion.
