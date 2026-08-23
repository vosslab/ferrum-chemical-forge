# Ferrum API contract

This reference defines the stateless Ferrum operation-protocol V1 contract. It is the machine
interface behind the human CLI verbs; task-oriented examples are in [USAGE.md](USAGE.md).

## Transport and envelope

`ferrum protocol run` reads one UTF-8 JSON request from a named file or standard input. Its request
schema is `ferrum-operation-request-v1`:

```json
{
  "schema": "ferrum-operation-request-v1",
  "request_id": "caller-chosen-opaque-id",
  "operation": {
    "kind": "document.inspect",
    "document": "<cdml xmlns=\"urn:ferrum:cdml\"/>"
  }
}
```

`request_id` is returned unchanged after admission but has no identity, ordering, persistence, or
authorization meaning. An admitted decodable request returns either
`ferrum-operation-response-v1` or `ferrum-operation-error-v1`. Clients must discriminate on schema,
operation `kind`, and error `category`, never on a diagnostic `message`.

The protocol has a derived UTF-8 transport limit before allocation and JSON parsing. Invalid JSON or
an over-budget transport produces no response envelope. Existing CDML admission and complete-artifact
bounds apply independently after transport admission.

## Operations

The V1 operation set is closed:

| Operation kind | Request fields | Successful outcome |
| --- | --- | --- |
| `document.inspect` | `document` CDML text | `report`, `document_fence` |
| `document.validate` | `document`, `level`: `structural` or `typed` | `level`, `report` |
| `document.rewrite` | `document` | structural `document`, `report` |
| `document.render_artifact` | `document`, `format` | `format`, `media_type`, complete base64 `artifact_base64` |
| `chemistry.convert` | `input.format`, `input.text`, `output_format` | `format`, converted `text`, `record_count` |
| `document.generate_coordinates` | `document` | structural `document`, `regenerated_molecule_count` |
| `presentation.author.v1` | `document`, `expected_revision`, `expected_digest_hex`, typed `authoring` | schema-defined committed document and durable root outcome |
| `catalog.list.v1` | no document input | immutable catalog schema, version, and entry summaries |
| `catalog.insert.v1` | `document`, revision/digest fence, catalog ID, finite anchor | changed `document`, `identifier`, `committed_revision`, `document_fence` |
| `document.molecule.smarts.query.v1` | admitted document and bounded raw or selected SMARTS query | bounded, non-redeemable query summary |

The closed interchange format names are `smiles`, `inchi_standard`, `inchi_fixed_h`,
`molblock_v2000`, `molblock_v3000`, `sdf_v2000`, `sdf_v3000`, and `cdml`. Render formats are `svg`,
`pdf`, and `png_one_pixel_per_point_transparent`.

`chemistry.convert` and `document.generate_coordinates` need an out-of-band trusted chemistry
runtime. Their JSON carries only owned text and closed names: never a filesystem path, library
handle, session, or adapter locator. The default executor returns `chemistry_unavailable` when that
capability is absent.

`document.inspect` admits one snapshot and returns its canonical
`document_fence { expected_revision, expected_digest_hex }` with the report. A caller can carry
that fence into a later request-owned mutation; the human CLI report remains read-only.

`catalog.insert.v1` performs one complete request-owned catalog insertion. On success, it returns
only the changed canonical `document`, created `identifier`, observed `committed_revision`, and a
portable `document_fence` for that returned document. The fence has revision zero because the next
stateless request admits returned CDML into a fresh session; its digest is derived from that exact
returned text. A stale fence is a typed refusal with no success outcome, so a caller retries from
the returned document and fence rather than interpreting partial catalog result data.

`presentation.author.v1` performs one complete request-owned mutation. Its closed authoring
families are Vector, terminal Electron/Retro/Normal arrow, Curved Equilibrium arrow,
Polyline/Polygon path, and explicit-endpoint DirectBond. A request contains finite serializable
geometry and, for DirectBond, durable atom IDs or finite new-atom points. It contains no Qt
pointer, viewport, hit-test, preview, session, capability, or reservation object. The adapter
creates and redeems those short-lived authorities internally, returning only the accepted CDML
snapshot and durable result facts.

`document.molecule.smarts.query.v1` reports bounded source-order match facts for one admitted
document snapshot. Its summary contains no match membership, atom identity, query text, native
state, geometry, or live reveal capability. A response whose canonical public envelope exceeds
the fixed 1 MiB budget is replaced before delivery with the typed
`resource_limit` / `response_size_exceeded` refusal. It carries no partial rows or query result.
The generated [ferrum-operation-v1.schema.json](../packages/ferrum-rust/crates/api/protocol/ferrum-operation-v1.schema.json)
defines the current request and response fields for this closed operation and for
`presentation.author.v1` outcomes.

## Success and error data

A successful envelope has this shape:

```json
{
  "schema": "ferrum-operation-response-v1",
  "request_id": "caller-chosen-opaque-id",
  "outcome": {"kind": "document.inspect", "report": {}}
}
```

A typed refusal has this shape; `request_id` is present only when the envelope admitted it:

```json
{
  "schema": "ferrum-operation-error-v1",
  "request_id": "caller-chosen-opaque-id",
  "error": {
    "category": "chemistry_unavailable",
    "operation": "chemistry.convert",
    "message": "human-readable detail"
  }
}
```

Stable error categories are `invalid_request`, `unsupported_protocol_version`,
`document_admission_failed`, `document_invalid`, `render_unsupported`, `render_failed`,
`chemistry_unavailable`, `conversion_failed`, `conversion_unsupported`,
`coordinate_generation_failed`, `resource_limit`, and `internal_failure`.

When `presentation.author.v1` refuses after envelope admission, its error includes typed
`presentation_author_refusal` facts: an authoring kind, a closed category, and a recovery action.
`refresh_and_restart` means the caller must obtain a current fence and create a new request;
`change_geometry` means finite geometry violates that family's contract;
`adjust_endpoint` means the DirectBond endpoint choice is invalid; and `document_unchanged`
confirms that the supplied document remains the correct retry basis. `change_presentation` and
`report_conflict` describe the remaining typed policy or result-conflict cases. Clients use these
facts rather than parsing diagnostic messages.

When `catalog.insert.v1` refuses after envelope admission, its error includes
`catalog_placement_refusal { category, recovery }`. A stale fence has
`category: "stale_snapshot"` and `recovery: "refresh_and_restart"`; it has no success outcome
or partial document result.

`document.rewrite` provides structural preservation rather than byte identity. A successful render
contains one complete standard-base64 artifact or no artifact; it never exposes a partial result.

## CLI presentation rules

The `inspect`, `validate`, `rewrite`, `render`, `convert`, and `coords` commands construct their
corresponding requests. The named document commands
`document command presentation.author.v1` and `document command catalog.insert.v1` accept one
complete operation JSON object, just as `protocol run` does, so a script can use the fence from
`document.inspect` without an in-process session. `--json` emits the complete envelope. Without
`--json`, inspection and validation print their reports while result-producing commands emit raw
text or artifact bytes. Named output uses safe publication and cannot replace a retained source or
observed hard-link alias.

CLI exit `0` means a completed success or typed refusal. Exit `1` is an input, processing, or
confirmed publication failure; exit `2` is a usage failure; exit `3` means publication may have
occurred but cannot be confirmed. The protocol runner also uses safe publication for a named JSON
response and rejects `--output -`.

## Python boundary

The `ferrum_chem` extension exposes only these protocol functions:

```python
def execute_operation_v1(request_json: str) -> str: ...
def operation_protocol_schema_v1() -> str: ...
```

`OperationProtocolErrorV1(FerrumError)` exposes a stable `category` for invalid JSON,
resource-limit, or execution-unavailable failures that occur before an envelope can exist.
Non-string input follows normal Python type checking. The binding accepts no mapping, bytes, path,
session, receipt, or Qt object.

## Exclusions

V1 excludes batch requests, protocol paths, network access, adapter discovery, CD-SVG or compressed
input, selection or root export, arbitrary template recipes, clipboard, recovery copies, arbitrary
document mutation, and render observation. Its admitted mutations are the closed, fenced
`presentation.author.v1` and `catalog.insert.v1` operations. Desktop editing has its own Rust-owned
live contracts; see [USAGE.md](USAGE.md) and [FILE_FORMATS.md](FILE_FORMATS.md).
