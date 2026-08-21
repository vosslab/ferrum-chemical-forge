# M2a CML/CML2 import contract V2

## Decision

M2a adds one Rust-owned, CML/CML2 **import-only** profile for ordinary 2-D
molecule drawings. It admits a closed XML grammar, converts all admitted facts
into one bounded Ferrum import plan, and commits atomically. It adds neither
CML export nor generic XML support nor byte-for-byte CML round-tripping.

This revision replaces the provisional M2a design's permissive grammar,
provisional limits, assumed `FormatCapabilityV1` registry, and ambiguous result
identity. A source is either wholly represented by this profile or refused
before mutation, following **Fix the design, not the symptom**.

## M2a.3a completion

M2a.3a is complete and accepted for the fixed-target CML/CML2
`new_document` import. It implements Rust-owned closed-profile decoding,
private CML-to-document conversion and CDML handoff, atomic document admission,
and exact bounded protocol-envelope admission for both success and refusal.
The CLI publishes an output artifact only after the measured protocol result
commits.

Evidence is recorded in `/private/tmp/ferrum-m2a3a-cml-final-evidence.md` and
the independent acceptance review is
`/private/tmp/ferrum-m2a3a-final-acceptance-review.md`.

This acceptance does not claim full CML, OASA, or BKChem parity. Append to the
current document, durable selectors, live PyO3 or Qt import, export or
conversion, and wider CML semantics remain deferred.

## Scope and ownership

`ferrum-chemistry` owns XML token admission, profile validation, source spans,
and lowering to bounded owned interchange graphs. `ferrum-document` owns the
coordinate transform, persistent identity allocation, candidate construction,
revision/digest fences, and one all-or-nothing commit. `ferrum-api` owns public
DTOs, the format registry contract, CLI presentation, canonical JSON, and
redacted refusals. Live PyO3 receipts and Qt file actions are distinct later
slices, not dependencies of the next stateless open workflow.

No Python/OASA parser, Qt parser, native-adapter XML parser, XML DOM, XPath, or
per-format Qt handler is introduced. `OTHER_REPOS/` remains inventory-only.

## Closed XML profile

### Document admission

The decoder accepts UTF-8 XML only. It strips one optional UTF-8 BOM before byte
budgeting; a non-UTF-8 sequence is `invalid_utf8`. One optional XML declaration
is allowed only as the first token and only as `version="1.0"` with
`encoding="UTF-8"`; it counts toward input bytes but not depth.

Exactly one root is required:

| Profile | Root expanded name | Required binding |
| --- | --- | --- |
| CML1 | `{http://www.xml-cml.org/schema}cml` | default namespace exactly CML1 |
| CML2 | `{http://www.xml-cml.org/schema/cml2/core}cml` | default namespace exactly CML2 |

The root selects the profile for the entire document. A prefixed root,
undeclared prefix, duplicate binding, namespace rebinding, foreign namespace,
`xml:*` attribute, or `xmlns:prefix` is refused. The root has no attributes
other than its required default binding, no non-whitespace text, and one through
`MAX_RECORDS` direct `molecule` children in source order. The only documents are:

```text
cml1-document := cml1-root (cml1-molecule)*
cml2-document := cml2-root (cml2-molecule)*
```

Wrappers, `moleculeList`, nested `cml`, nested `molecule`, sibling semantic
elements, and zero records are refused (`empty_document` for the latter).

### XML lexical behavior

DTD declarations, parameter/general entity declarations, external identifiers,
XInclude, stylesheet instructions, and CDATA are refused. Only the five
predefined XML entities are accepted in attributes; decoded value bytes count
against the attribute budget. No URI is fetched, no entity is input-expanded,
and no arbitrary XML is retained.

Whitespace-only text is allowed between admitted elements. All other text is
`unexpected_xml_text`. Comments and processing instructions are allowed only
between elements or outside the root; they are ignored, unretained, and count
against `MAX_COMMENT_BYTES` and `MAX_PI_BYTES`. A comment or PI inside `atom`,
`bond`, or a scalar builtin is `unexpected_xml_node`. A semantic element or
attribute is never ignored as mere lexical normalization.

### CML2 grammar

```text
molecule := molecule(atomArray, bondArray?)
atomArray := atomArray(atom+)
atom := atom[@id, @elementType, @x2, @y2, @formalCharge?, @isotopeNumber?]
bondArray := bondArray(bond*)
bond := bond[@atomRefs2, @order]
```

`molecule`, `atomArray`, and `bondArray` have no text and no attributes other
than the optional unqualified molecule `id`. `atom` and `bond` have no children
or text. Listed attributes are the complete unqualified, single-occurrence
whitelist. `atomRefs2` has exactly two ASCII XML-NCName IDs separated by XML
whitespace. Endpoints are distinct, declared in that record, and form no
repeated undirected pair. `order` is exactly `1`, `2`, `3`, `S`, `D`, or `T`,
with aliases normalizing to `1`, `2`, and `3`.

All CML2 array attributes are refused. In particular, `atomID`, `elementType`,
`x2`, `y2`, `formalCharge`, `isotopeNumber`, `atomRefs2`, `bondID`, and `order`
on an array/container are `array_attribute_unsupported`. M2a never zips,
broadcasts, defaults, or order-reconstructs CML arrays.

### CML1 grammar

```text
molecule := molecule(atomArray, bondArray?)
atomArray := atomArray(atom+)
atom := atom(builtin-id, builtin-element, builtin-x2, builtin-y2,
            builtin-charge?, builtin-isotope?)
bondArray := bondArray(bond*)
bond := bond(builtin-ref, builtin-ref, builtin-order)
```

Each builtin is one direct `builtin` child with exactly unqualified `@builtin`
and one scalar text value. Allowed values, in the exact displayed order, are
`atomId`, `elementType`, `x2`, `y2`, `formalCharge`, `isotopeNumber`, `atomRef`,
and `order`. Containers/atoms/bonds have no other attributes, text, or child
elements. CML1 attributes duplicating builtin facts, CML2-style attributes, and
all CML1 array/container attributes are refused, not merged or prioritized.

The optional molecule `id` is source provenance only: it is a unique nonempty
ASCII NCName. A missing ID is valid and produces no generated public source ID.

### Scalar rules

Atom/molecule IDs are ASCII XML NCNames from 1 to 128 bytes. `elementType` is
one of Ferrum's 118 IUPAC symbols. `formalCharge` is decimal signed integer
`[-8, 8]`; `isotopeNumber` is unsigned decimal `[1, 400]`. Coordinates are
finite decimal literals: no `NaN`, `INF`, hex, unit suffix, list, or whitespace.

Stereochemistry, aromatic/query bonds, hydrogen counts, radicals, labels,
names, properties, formulas, reactions, crystals, polymers, R-groups, electron
flow, presentation, and every unknown child/attribute are
`unrepresented_semantic_fact`.

## Coordinate contract

Existing Ferrum geometry uses finite **drawing units** and y-up geometry, as
documented by `HaworthPoint` in
`packages/ferrum-rust/crates/domain/src/haworth/types.rs`. CML `x2`/`y2` are
unitless source depiction coordinates with y-down source geometry. M2a fixes:

```text
FERRUM_X = 30.0 * CML_X
FERRUM_Y = -30.0 * CML_Y
```

`30.0` drawing units is the canonical Ferrum bond-length unit. The origin is
preserved: `(0, 0)` becomes `(0, 0)`. M2a never centers, translates,
fit-to-pages, autolayouts, infers a bond length, or rescales per record.

Input requires finite `f64` values with `abs(value) <= 100000.0`. Checked
transform arithmetic then requires finite output and `abs(x/y) <= 3000000.0`.
Violations are `coordinate_not_finite` or `coordinate_out_of_range`. The
document candidate validates each transformed point again before allocating
persistent IDs. 3-D-only, missing-2-D, and invalid inputs are never projected.

## Cross-layer format registry

M2a introduces `InterchangeFormatRegistryV1`; it does not assume an existing
`FormatCapabilityV1`. `ferrum-api` owns one closed static registry of
`InterchangeFormatDescriptorV1`. Chemistry exposes codec/profile IDs; document
exposes import targets; API validates their exact join at startup and schema
generation. CLI and Qt consume API descriptors only: no duplicate suffix,
alias, filter, capability, or enabled-state tables.

```text
format_id: cml_simple_molecule_import_v1
profile_id: ferrum-cml-simple-molecule-import-profile-v1
input_aliases: [cml, cml1, cml2]
input_suffixes: [.cml]
directions: [document_import_new]
output_suffixes: []
compression: forbidden
semantic_loss_policy: reject_unrepresented_semantics
```

The descriptor has no `decode_for_convert`, `encode`, `export`, or CML output
capability. `ferrum formats`, schema discovery, CLI `--format`, and Qt filters
all derive from it. `convert --to cml`, export, output suffixes, compressed
input, and unknown aliases are typed refusals.

## Public and live contracts

### M2a.0 registry boundary

M2a.0 exports the closed registry, frozen ingress budget, retained 1 MiB
response-budget constant, inbound request DTOs, and redacted refusal triples
only. The retained response value is private budget state for M2a.3 exact-envelope
admission; it is not an M2a.0 response schema, serializer, or publication
claim. M2a.0 has no XML decoder, document transaction,
named operation, durable-selector allocation, public success summary, or
canonical response-envelope admission helper. In particular, it cannot claim a
committed selector or a measured public success response before the
document-owned transaction can make either fact true.

M2a.3 introduces the named stateless operation and its one measured canonical
envelope together with the transaction that allocates selectors after commit.
The 1 MiB response constraint below is the M2a.3 contract, not an M2a.0 public
API or generic callback surface.

### M2a.3a stateless open

M2a.3a implements exactly one named stateless operation:
`document.molecule.interchange.import.v1` with the fixed target
`new_document`. It receives CML text and a CML alias, and receives no current
snapshot, revision, digest, placement, append target, live receipt, or Qt
state. It is a document-import control-plane operation: its protocol success,
including `ferrum protocol run --json`, never returns complete CDML, candidate
CDML, a converted CML artifact, or any other complete document artifact. The
8 MiB candidate exists only inside the Rust-owned admission/commit path; it
cannot cross the 1 MiB JSON response boundary.

M2a.3a success returns only the bounded open summary: fixed target,
source-record count, inserted-record count, revision, digest, profile ID,
source-format ID, and the required loss report. It includes no source identity,
durable selector, atom or bond
ID, record ID, anchor, geometry, complete source, complete candidate, complete
resulting document, or mutable handle. Semantic loss is empty in M2a; only
`source_ids_reallocated` and `lexical_xml_not_retained` normalizations occur.
This is import compatibility, not export, conversion-artifact delivery, or
lexical fidelity.

Append-current-document, durable selector delivery, live PyO3 receipt flow,
and Qt actions remain separate later slices. Their contracts must be specified
against the delivered M2a.3a operation rather than added as optional fields or
alternate behavior here.

Artifact delivery, if later required, must be a separately named and specified
conversion/export operation with its own request, output destination, artifact
budget, and atomic-publication contract. It must not be added as an optional
field, non-JSON side effect, or alternate response mode of
`document.molecule.interchange.import.v1`.

The document crate exposes no CML types or CML transaction capability. It admits
only a generic nonempty `MoleculeInsertionV1` batch as one atomic candidate;
the API-private CML operation owns decode, CML-to-document conversion, the
new-document policy, and CDML handoff. A pending generic batch retains its
next generated-ID cursor and has no issued provisional token until its commit,
so response admission refusal or drop leaves the session unchanged. The
stateless named operation constructs
and measures the exact standard protocol success or refusal envelope before it
commits. Explicit CLI artifact publication occurs only after that commit.

The M2a.3a implementation uses this generic batch seam directly: no CML
pending type, source identifier, CDML candidate, or CML conversion contract
crosses the `ferrum-document` public boundary. The protocol-owned
`DocumentMoleculeInterchangeImportSummaryV1` is the only public import DTO;
the CML operation module, CML response envelope, and CML conversion types are
crate-private. Both the CLI's closed CML
response and the named protocol's standard success/refusal envelope are
measured at the CML boundary before publication or commit.

### Deferred live PyO3 and Qt import

The later live import slice uses the same decoder/candidate semantics against
the tab-private snapshot. It returns only an opaque, nonserializable,
non-debuggable, non-cloneable `LiveCmlPreparedImportReceiptV1`. Rust privately
holds issuer, revision/digest fence, candidate, and mapping. Qt holds copied
count/status facts only; it calls only `commit_live_cml_import_v1(receipt)` or
`retire_live_cml_import_v1(receipt)`.

Commit authenticates issuer and fences before sole publication. Foreign,
replayed, stale, retired, tab-switched, closed, and cancelled receipts refuse
before mutation. A public API neither accepts nor returns a live receipt, and
Qt never uses a public selector for live commit. This precisely separates
durable public identity from live capability identity.

## Budgets and typed refusals

`CmlIngressBudgetV1` is frozen:

| Guard | Maximum | Reason |
| --- | ---: | --- |
| raw UTF-8 input bytes | 1,048,576 | `input_bytes_limit` |
| decoded XML text bytes | 1,048,576 | `xml_text_bytes_limit` |
| XML declaration bytes | 256 | `xml_declaration_limit` |
| comments total bytes | 65,536 | `comment_bytes_limit` |
| PI total bytes | 8,192 | `pi_bytes_limit` |
| XML elements | 50,000 | `xml_element_limit` |
| XML depth | 8 | `xml_depth_limit` |
| attributes per element | 8 | `xml_attribute_limit` |
| attribute value bytes | 256 | `attribute_value_limit` |
| source records | 1,024 | `record_limit` |
| atoms per record | 10,000 | `atoms_per_record_limit` |
| atoms total | 100,000 | `atom_limit` |
| bonds per record | 20,000 | `bonds_per_record_limit` |
| bonds total | 200,000 | `bond_limit` |
| source ID map entries | 101,024 | `source_id_map_limit` |
| scalar identifier bytes | 128 | `identifier_bytes_limit` |
| CDML candidate bytes | 8,388,608 | `candidate_bytes_limit` |
| canonical JSON success/refusal response bytes | 1,048,576 | `response_bytes_limit` |

Every guard and condition maps exactly once:

| Category | Closed reasons | Recovery |
| --- | --- | --- |
| `conversion_failed` | `invalid_utf8`, `invalid_xml`, `invalid_xml_declaration`, `unexpected_xml_text`, `unexpected_xml_node`, `invalid_scalar`, `invalid_coordinate`, `coordinate_not_finite`, `coordinate_out_of_range`, `duplicate_source_id`, `duplicate_atom_id`, `dangling_bond`, `self_bond`, `duplicate_bond`, `invalid_graph`, `empty_document` | `choose_supported_cml` |
| `conversion_unsupported` | `namespace_unsupported`, `root_unsupported`, `profile_mismatch`, `attribute_unsupported`, `array_attribute_unsupported`, `unrepresented_semantic_fact`, `dtd_forbidden`, `entity_forbidden`, `external_resource_forbidden`, `xinclude_forbidden`, `stylesheet_forbidden`, `compression_forbidden`, `format_alias_unsupported`, `direction_unsupported` | `remove_unsupported_features` |
| `resource_limit` | every limit reason in the budget table | `reduce_input` |
| `document_admission_failed` | `candidate_validation_failed`, `serialization_failed`, `internal_failure` | `retry_or_report_problem` |
| `stale_document` | `revision_mismatch`, `digest_mismatch`, `live_receipt_stale`, `live_receipt_unavailable` | `reopen_or_retry` |
| `chemistry_unavailable` | `chemistry_runtime_unavailable` | `install_chemistry_runtime` |

The parser applies byte/lexical limits before decoded text allocation,
structural limits before descending/appending, graph limits before insertion,
and candidate/result limits before publication/delivery. `response_bytes_limit`
is measured over the canonical JSON bytes of every stateless success and
refusal before it is emitted. Success construction must reserve/measure the
summary before publication; a response that cannot fit is
`resource_limit/response_bytes_limit` and leaves the import uncommitted. Every
refusal discards the candidate. It creates no partial
document/history/selection/output or externally visible generated ID.

## CLI and Qt behavior

`ferrum protocol schema` and `ferrum protocol run` stay authoritative. M2a.3a
adds this one thin presentation:

```text
ferrum open INPUT --format cml --output NEW.cdml [--json]
```

`open` builds a new CDML document and writes only after complete Rust import
success. Stdin requires `--format`; only `.cml` infers this profile. The
explicit `--output` file is the only CLI document delivery; it is atomically
published only after canonical-envelope admission and Rust import both succeed.
Both `--json` and non-JSON success write no complete CDML, candidate, source
CML, or converted artifact to stdout or stderr. `--json` emits exactly the
bounded protocol summary. Non-JSON emits a human-readable bounded success
summary (target, imported-record count, revision/digest, profile, and explicit
output path) and bounded progress/status only; it performs no hidden copy or
artifact stream. Typed refusal follows the existing exit-status convention and
emits only the redacted bounded refusal. M2a.3a neither exports CML nor expands
`convert`; a future artifact-producing conversion route must be separately
named and specified rather than tunneled through open.

## Phase-local proof gates

M2a adds proof only when the corresponding behavior exists. Regular Rust and
Python tests are offline and in-process. They use inline CML text, or a test's
temporary directory when a decoder/document boundary requires a file. M2a has
no shared CML corpus, manifests, hash inventories, generated inputs, network
access, or subprocess tests without explicit human approval and a durable
parser consumer that inline input cannot represent.

1. M2a.0 proves the registry joins its owning crates, unknown aliases receive a
   typed refusal, request/refusal DTOs remain closed, and the frozen budget
   maintains the relationships required by its guards. It has no decoder,
   document admission, canonical response-envelope admission, CLI route, or Qt
   action.
2. M2a.1 introduces decoder tests as compact, table-driven Rust tests with
   inline XML. Each test distinguishes an admitted grammar rule, a typed
   refusal, or the coordinate transform. A durable interoperability corpus
   requires explicit human approval after a parser consumer exists.
3. M2a.2 adds in-memory document tests: one successful batch and one late
   refusal that proves atomicity. It does not inventory implementation-owned
   identifiers, history storage, or intermediate state.
4. M2a.3a adds one real CLI workflow only after the fixed-target stateless open
   operation exists. It belongs in root `tests/e2e/`, creates its CML input in
   a temporary directory, has an explicit runner, and remains offline. It
   proves `ferrum open INPUT --format cml --output NEW.cdml [--json]`; exact
   canonical response admission occurs before document/output publication
   through the actual operation rather than a hook.
5. Append, durable-selector delivery, live PyO3 receipts, and Qt actions each
   receive a separately scoped later proof gate only when that behavior is
   implemented. They do not expand M2a.3a's E2E into a fixture matrix.

Disposable build and wheel provenance belongs in `/private/tmp` and the active
plan report, not in `devel/`, fixtures, or ordinary tests.

## Implementation order

1. Add registry, DTO, refusal, and budget definitions without UI.
2. Implement the Rust tokenizer/decoder with compact inline contract tests.
3. Build the new-document transaction from the admitted batch.
4. Add the fixed-target stateless protocol and one `ferrum open` CLI workflow.
5. Specify durable selectors, live receipts, and Qt in later distinct slices
   when their user-facing workflow is next.
