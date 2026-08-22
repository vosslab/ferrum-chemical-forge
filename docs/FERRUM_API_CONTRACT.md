# Ferrum API contract

This reference defines the stateless Ferrum operation-protocol V1 contract. It is the machine
interface behind the six human CLI verbs; task-oriented examples are in [USAGE.md](USAGE.md).

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
| `document.inspect` | `document` CDML text | `report` |
| `document.validate` | `document`, `level`: `structural` or `typed` | `level`, `report` |
| `document.rewrite` | `document` | structural `document`, `report` |
| `document.render_artifact` | `document`, `format` | `format`, `media_type`, complete base64 `artifact_base64` |
| `chemistry.convert` | `input.format`, `input.text`, `output_format` | `format`, converted `text`, `record_count` |
| `document.generate_coordinates` | `document` | structural `document`, `regenerated_molecule_count` |

The closed interchange format names are `smiles`, `inchi_standard`, `inchi_fixed_h`,
`molblock_v2000`, `molblock_v3000`, `sdf_v2000`, `sdf_v3000`, and `cdml`. Render formats are `svg`,
`pdf`, and `png_one_pixel_per_point_transparent`.

`chemistry.convert` and `document.generate_coordinates` need an out-of-band trusted chemistry
runtime. Their JSON carries only owned text and closed names: never a filesystem path, library
handle, session, or adapter locator. The default executor returns `chemistry_unavailable` when that
capability is absent.

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

`document.rewrite` provides structural preservation rather than byte identity. A successful render
contains one complete standard-base64 artifact or no artifact; it never exposes a partial result.

## CLI presentation rules

The `inspect`, `validate`, `rewrite`, `render`, `convert`, and `coords` commands only construct
these requests. `--json` emits the complete envelope. Without `--json`, inspection and validation
print their reports while result-producing commands emit raw text or artifact bytes. Named output
uses safe publication and cannot replace a retained source or observed hard-link alias.

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
input, selection or root export, templates, clipboard, recovery copies, document mutation, and
render observation. Desktop editing has its own Rust-owned contracts; see [USAGE.md](USAGE.md) and
[FILE_FORMATS.md](FILE_FORMATS.md).
