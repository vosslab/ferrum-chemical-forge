# Completed milestone evidence

This report holds the accumulated implementation evidence removed from the active
tracker on 2026-08-19. It is an evidence index, not a second plan: the active
[Ferrum v3 plan](../ferrum-plan-v3.md) owns status, dependencies, and next
decisions. References below point to the focused receipts and contracts that retain
the detailed commands, fixtures, measurements, and review findings.

## Product and repository foundation

- **M1a-M1e:** Ferrum is an AGPL PySide6 application; Ferrum-Chem is an LGPL
  Rust backend. Production manifests contain neither OASA nor Python RDKit. The
  repository has canonical offline license texts, provenance, split production and
  development dependency manifests, a repaired resource layout, and a historical
  capability matrix. Historical OASA material is isolated reference material, not
  a runtime dependency. The retired live Python comparison workers remain history,
  not product code.
- **M1d/M6-M10:** CDML preservation is structural rather than byte-level. The
  current reader uses `xot`, rejects DTD input without external resolution, applies
  caller-owned byte/node/depth/attribute/text limits before parsing, and retains an
  opaque/typed single-tree overlay. `IndexedDocument` protects declaration IDs,
  source order, root-relative paths, opaque IDs, and one-use provisional tokens;
  fragment bond/vertex references are never mistaken for declarations. See the
  [preservation inventory](../audits/cdml_preservation_coverage.md),
  [document identity decision](../decisions/document_identity_ordering.md),
  [typed-record receipt](typed_document_records.md), and
  [preservation gate](cdml_preservation_gate.md).
- **M2/M3:** `ferrum-core` owns immutable molecule facts and
  presence-sensitive properties. The document projection reads every corpus
  molecule using versioned bond semantics. Corpus agreement and the classified
  differences are in [corpus molecule parity](corpus_molecule_parity.md).
  Ferrum owns a deterministic fundamental-cycle policy; its shorter bridged basis
  is an intended change, not an oracle regression. See
  [graph analysis parity](graph_analysis_parity.md).

## Chemistry and packaging evidence

- **M4a:** A macOS arm64 source-built native-wheel route proved a package-relative,
  replaceable chemistry library. The initial two-library ABI-1 proof is historical;
  its surviving engine-boundary evidence is in
  [native_kekulization.md](native_kekulization.md).
- **M4b-M4d:** The accepted adapter is a narrow `ChemEngine` boundary with owned
  `MolGraph` output. ABI-2 established GraphMol kekulization with stated defaults;
  ABI-4 carries complete graphs. The current direct extension is FCM1/ABI-4 against
  RDKit 2026.03.5 and IUPAC InChI 1.07.3. The narrow five-library kekulization proof
  remains in [native kekulization](native_kekulization.md); the current package
  closure is a macOS arm64 observation, not cross-platform release proof. The
  coordinate parity receipt used twenty fresh Python-wrapper and twenty fresh
  extension processes on six molecules, observed zero internal/cross-backend noise,
  and derived `7.105427357601002e-15` from ULP spacing. See
  [coordinate parity](coordinate_parity_v1.md) and the
  [engine-boundary decision](../decisions/chemistry_engine_boundary.md).
- **M5:** SMILES, SMARTS, molblock, SDF, and InChI use complete owned graph DTOs.
  SMARTS is export-only because historical OASA did not advertise import. Molblock
  V2000/V3000, ordered multi-record SDF, Standard/Fixed-H InChI, and validated
  InChIKey paths have semantic tests against RDKit 2026.03.5 and previous stable
  2026.03.4. Exact strings are required only for deterministic identifier outputs;
  molblock/SDF comparison is semantic and coordinate bounds are derived from emitted
  decimal tokens. These facts demonstrate current adapter behavior, not a promise
  that every legacy text format is supported.

## Geometry, rendering, and domain evidence

- **M11:** The Rust geometry layer owns y-up `RepairOutcome`, durable identity
  ordering, atomic revision/digest-fenced application, and the BSD-3-derived
  `straightenDepiction` algorithm. The one-time macOS arm64 receipt observed a
  `3.645723512257204e-18` maximum coordinate delta and zero rotation delta; it is
  evidence, not a permanent CI threshold. See
  [geometry straighten parity](geometry_straighten_parity.md).
- **M12/M13:** Rust issues declarative checked render operations and Telex design
  metrics; Qt copies supplied paths/glyph facts rather than reshaping labels. The
  render crate lowers supported plans through one checked draw stream to SVG, pure
  Rust PNG, and vector PDF. It rejects invalid/non-finite/over-limit output before
  publication and reports named exclusions. Exact glyph IDs/origins are a closed
  contract, while Qt metric and raster observations are platform evidence only.
  See [render metrics](render_ops_glyph_metrics_v1.md) and
  [render backend evidence](m13_svg_plan_backend_v1.md).
- **M14:** The Haworth work is intentionally bounded infrastructure: selected
  single-ring and direct-glycosidic topology/layout/fragment/depiction observations
  with no document authoring, page composition, stereochemical inference, or public
  importer. The direct renderer uses explicit `q1`/`w1`/`n1` depiction facts. The
  CDML profile is documented in the format specification; a future authoring route
  needs its own session contract.
- **M15:** Retained utilities are bounded peptide sequence insertion through
  `prepare_ferrum_peptide_insertion_v1`, linear-form
  conversion, Clean Geometry, and five geometry repairs. Compact sugar parsing,
  known-group catalogs, substructure search, oxidation, generated names, and broad
  biomolecule catalogs are intentional pre-production drops, not partial fallbacks.
  Unsupported peptide/profile facts fail explicitly before native work.

## Frozen public boundary evidence

- **M17:** Protocol V1 is a stateless, versioned JSON boundary with generated schema,
  independent transport admission, closed request/error envelopes, and no paths in
  payloads. Its initially frozen operations are `document.inspect`,
  `document.validate`, `document.rewrite`, and `document.render_artifact`; unknown
  versions are rejected. Resource limits are allocation-safety controls, not timing
  thresholds.
- **M18:** `ferrum_chem` exposes `execute_operation_v1`,
  `operation_protocol_schema_v1`, and categorized `OperationProtocolErrorV1`.
  `ferrum protocol schema` and `ferrum protocol run` preserve one JSON stream and
  explicit safe publication. The human verbs `inspect`, `validate`, `rewrite`, and
  `render` construct protocol requests rather than becoming a second backend.
  Provisional direct-chemistry root commands are retired. The Python/CLI contract is
  frozen only where its schema and checked stubs say it is; private native Qt DTOs do
  not become third-party API.

## Current boundary evidence

The desktop package now has Ferrum product identity, an implementation-facing
`ferrum_qt/ferrum/` directory, one lazy compiled-extension adapter, shared action
identifiers/keybindings, an About route, and focused accessibility/focus seams. Real
verb E2E compares protocol and CLI semantic output across file/stdin/stdout routes.
This proves an important convergence slice but does **not** close M19: chemistry
conversion/coordinate operations, portable frontend seams, complete adapter review,
capability-ledger closure, worker consolidation, and release evidence are deliberately
tracked as open work in the active plan.
