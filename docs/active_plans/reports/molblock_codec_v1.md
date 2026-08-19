# Molblock Codec V1 Evidence

## Result

The bounded V2000/V3000 import and export slice is green on macOS arm64. It is the
completed molblock part of M5.

Ferrum sends its frozen, handle-free molecule graph and atom-aligned coordinates
through ABI-4 FCB1/FCG1. The native adapter reconstructs the molecule and emits the
explicitly requested RDKit molfile version. The operation is available through:

- `ferrum_chem.molecule_to_molblock(molecule, version)` for an exact frozen
  `SmilesMoleculeV1`; and
- provisional
  `ferrum smiles to-molblock --adapter ABSOLUTE_LIBRARY --format v2000|v3000 SMILES`.

Ferrum also strictly imports one bounded V2000 or V3000 molblock into an owned,
handle-free molecule through `ferrum_chem.molblock_to_molecule(molblock)` and
provisional `ferrum molblock inspect --adapter ABSOLUTE_LIBRARY INPUT`. Import
rejects SDF record separators, multiple CTAB terminators, 3D conformers, embedded NUL,
and oversized input before publishing any Rust or Python value.

The CLI retains the explicit-adapter policy. It does not search the environment,
current directory, executable path, or Python wheel layout.

## Grounded comparison rule

Molblock bytes are not the compatibility gate. Program, comment, timestamp, spacing,
and numeric formatting can vary without changing the molecule. Each generated block
is instead parsed with RDKit's strict parser and normal sanitization, then compared by
chemical meaning:

- canonical isomeric SMILES;
- atom order, atomic number, total hydrogen count, charge, isotope, radical count,
  atom mapping, and tetrahedral chirality;
- normalized undirected bond endpoints, order, aromaticity, and stereo; and
- complete finite coordinates in atom order.

Explicit versus implicit hydrogen storage is deliberately not equated. RDKit can
round-trip `[NH4+]` with the same total hydrogen count and canonical chemistry while
changing which hydrogens are stored as explicit atom metadata. Likewise, reversing an
undirected bond record does not change topology. Those are representation details,
not codec failures.

Coordinate acceptance is derived from each written decimal token. A parsed component
must remain within half that token's decimal quantum plus the source and restored
binary floating-point ULP. This produced maximum bounds near `5.0e-5` for V2000 and
`5.0e-7` for V3000. The largest observed deltas were below those format-derived
bounds. No unrelated pixel, timing, or byte threshold is used.

## Differential corpus

[`molblock_codec_v1.json`](molblock_codec_v1.json) records seven cases:

- ethanol and nitrile;
- disconnected ammonium/chloride;
- aromatic benzene;
- E/Z bond stereo;
- isotope, tetrahedral chirality, formal charge, and atom mapping; and
- explicit methylene with radical electrons.

Both V2000 and V3000 pass strict semantic round trips under the wheel's recorded
RDKit 2026.03.5 build and the previous stable RDKit 2026.03.4 release. The generated
text happened to match the
same-version Python writer in this receipt, but that observation is not an acceptance
requirement.

The source coordinates were independently refreshed for the same wheel across 20
processes. Ferrum and the recorded RDKit build had a maximum absolute delta of `0.0`; the retained
M4c tolerance is `7.105427357601002e-15`, derived from the represented coordinate ULP.

## Installed-wheel and relink proof

The fresh direct-extension wheel is:

`output_native_wheel/molblock-import-v1-rdkit-2026035-20260812/wheelhouse/`
`ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl`

Its SHA-256 is
`13de57cf0d95dc3f1755f14a1ca36350fe4db7dca43e3ab8ead0e3d0e74b3eda`.
The sealed closure contains `libferrum_chem.dylib` and 14 RDKit libraries, 15 total.
The added codec libraries are `libRDKitChemTransforms.1.dylib` and
`libRDKitFileParsers.1.dylib`.

The clean-wheel E2E generated both versions before and after replacement with a
distinct-byte RelWithDebInfo adapter. Original and replacement adapter SHA-256 values,
the exact closure, extension identity, wheel RECORD, and operation outcomes are bound
in the machine-readable receipt.

## M5 relationship

The other M5 codec families are now separately green:

- SMARTS export matches the reference codec's export-only surface and has
  cross-version query-match evidence in [`smarts_codec_v1.md`](smarts_codec_v1.md);
- bounded ordered 2D SDF import and export are covered by
  [`sdf_codec_v1.md`](sdf_codec_v1.md); and
- Standard and Fixed-H InChI plus InChIKey are covered by
  `inchi_codec_v1.md`.

This closes M5 without treating molblock bytes as chemistry and without expanding the
reference codec contract.

## Historical evidence

The isolated Python-RDKit generator and child were retired after this accepted
one-time semantic receipt. The retained corpus, wheel identity, source digests, and
comparison policy are archival evidence rather than a permanent Python-RDKit or CI
dependency. A future codec measurement requires an explicitly scoped Ferrum
release-evidence plan and a fresh accepted receipt.
