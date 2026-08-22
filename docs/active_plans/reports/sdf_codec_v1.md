# SDF codec evidence

## Result

Ferrum's bounded SDF export and import slice passes semantic round trips through the
current RDKit 2026.03.5 release and the previous 2026.03.4 release. Ferrum's own
strict importer agrees with the current RDKit evaluator on every imported molecule.
The source-bound machine receipt is [sdf_codec_v1.json](sdf_codec_v1.json).

This is the completed bounded 2D SDF part of M5. The read-only OASA conversion copies
only x/y coordinates and exposes plain SDF text/file operations, so three-dimensional
coordinates and compressed suppliers are feature expansion rather than parity gates.
Ferrum already retains more ordered string-property information than that reference
path; arbitrary non-string field typing is outside the codec contract.

## Public path

The implemented value flow is:

```text
frozen SmilesMoleculeV1
    -> frozen ordered SdfRecordV1 values
    -> bounded FSD1 request
    -> RDKit SDWriter
    -> bounded FCT1 text response

bounded UTF-8 SDF text
    -> strict RDKit SDMolSupplier
    -> bounded FSI1 records
    -> safe Rust ImportedSdfRecord values
    -> frozen ImportedSdfRecordV1 values
```

The same safe Rust operation is available through:

- `ferrum_chem.prepare_sdf_record()` and `ferrum_chem.records_to_sdf()`;
- `ferrum_chem.sdf_to_records()`;
- `NativeChemEngine::{records_to_sdf,sdf_to_records}()`;
- provisional `ferrum smiles to-sdf --adapter ABSOLUTE_LIBRARY` for one record; and
- provisional `ferrum sdf inspect --adapter ABSOLUTE_LIBRARY INPUT` for bounded import.

The CLI remains provisional until M17 and M18 freeze the boundary.

## Comparison policy

Acceptance checks chemical and document meaning:

- strict SDF parsing and normal sanitization;
- exact record order and title;
- exact property order, names, and values;
- exact discrete atom, bond, charge, isotope, chirality, and atom-map facts;
- finite atom-aligned coordinates;
- native import agreement with the current RDKit semantic evaluator;
- explicitly requested V2000 or V3000 syntax in every record.

SDF bytes, RDKit header spacing, program lines, and record annotations are not compared.
RDKit can vary those without changing the file's meaning.

## Corpus and versions

The three-record corpus covers:

- ethanol with an ordered property and a multiline value;
- ammonium chloride with charge, a disconnected graph, and an empty value;
- an isotope/chirality/atom-map molecule with two ordered properties.

Both V2000 and V3000 passed under RDKit 2026.03.5 and 2026.03.4 at the time of this
measurement. The old source tag and SHA are archival facts, not a reproducible wheel
recipe in the current checkout. Current development uses the local runtime staged by
`./build.sh`; focused binding coverage proves that import retains repeated SDF property
names as distinct ordered entries.

## Retired wheel evidence

The following macOS arm64 direct-extension wheel was used for the historical
measurement:

```text
output_native_wheel/molblock-import-v1-rdkit-2026035-20260812/wheelhouse/
ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl
```

Its SHA-256 is
`13de57cf0d95dc3f1755f14a1ca36350fe4db7dca43e3ab8ead0e3d0e74b3eda`.
The wheel is 3.3 MB. Its 15-library closure contains `libferrum_chem.dylib` and
14 RDKit libraries, with no RDKit Python package, compiled Boost library, Cairo, or
FreeType dependency.

The writer and reader passed before and after replacement with a separately built
`RelWithDebInfo` adapter. The recorded receipt path and wheel output root were removed
with the retired publication workflow, so neither is present or reproducible now.

## Historical evidence

The isolated Python-RDKit generator and child were retired after this accepted
one-time semantic receipt. The retained corpus, wheel identity, source digests, and
comparison policy are archival evidence rather than a permanent Python-RDKit or CI
dependency. A future codec measurement requires an explicitly scoped Ferrum
release-evidence plan and a fresh accepted receipt.
