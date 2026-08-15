# Use Ferrum

Ferrum provides a Rust `ferrum` command-line tool and one ordinary `ferrum-qt`
Rust-native drawing application. The command-line tool works without Python.
Ferrum-Qt is a bounded pre-production editor; see [INSTALL.md](INSTALL.md) for
its verified setup and platform limits.

## Quick start

Inspect the generated protocol schema:

```bash
ferrum protocol schema
```

Start a new drawing window, or open an uncompressed drawing on launch:

```bash
ferrum-qt
ferrum-qt drawing.cdml
```

Use `ferrum --help` or `ferrum-qt --help` for the installed command help.

## Operation protocol V1

The shipping `ferrum` command exposes one stateless, machine-readable CDML
operation protocol:

```bash
ferrum protocol schema
ferrum protocol run request.json
ferrum protocol run request.json --output response.json
```

`request.json` is one UTF-8 JSON object. Its `schema` is
`ferrum-operation-request-v1`, it carries an opaque `request_id`, and its
operation is one of `document.inspect`, `document.validate`,
`document.rewrite`, or `document.render_artifact`. See
[the protocol section below](#protocol-contract) for the request shape, Python
boundary, result/error schemas, and exclusions.

`run` emits one success or typed protocol-error JSON envelope on standard
output. A completed typed refusal exits 0; input or pre-envelope failure exits
1; usage failure exits 2; and an output publication that may have occurred but
cannot be confirmed exits 3. A named `--output` uses safe publication and
cannot replace the pathname request source or its observed hard-link alias.
`--output -` is a usage error. The protocol does not batch requests, infer an
output, write raw artifacts beside JSON, use Qt, or access a network.

### Protocol contract

The request schema is `ferrum-operation-request-v1`. A successful response
uses `ferrum-operation-response-v1`; a decodable refusal uses
`ferrum-operation-error-v1`. Ferrum echoes an admitted request ID unchanged,
but it has no identity, ordering, persistence, or authorization meaning.
Clients use schema, error category, and operation as discriminators, never a
diagnostic message.

`document.validate` accepts `structural` or `typed`. Artifact formats are
`svg`, `pdf`, and `png_one_pixel_per_point_transparent`; a response contains a
complete base64 artifact and its media type or a typed refusal, never a partial
artifact. Rewrite is structural preservation, not byte identity.

The envelope has a derived UTF-8 transport boundary before CLI input is
allocated, Python input is copied, or JSON is parsed. It derives from the
existing uncompressed CDML profile, worst-case JSON escaping, and a small V1
framing/request-ID allowance. This is allocation safety, not a latency, corpus,
pixel, or performance requirement. CDML admission and base64 completion retain
their separate existing bounds. Stable response categories are
`invalid_request`, `unsupported_protocol_version`,
`document_admission_failed`, `document_invalid`, `render_unsupported`,
`render_failed`, `resource_limit`, and `internal_failure`. Invalid JSON and an
over-budget transport have no response envelope.

The `ferrum_chem` extension adds only:

```python
def execute_operation_v1(request_json: str) -> str: ...
def operation_protocol_schema_v1() -> str: ...

class OperationProtocolErrorV1(FerrumError):
    category: str
```

A decodable domain or version refusal is returned JSON data. Before an envelope
can exist, `OperationProtocolErrorV1.category` is `invalid_json`,
`resource_limit`, or `execution_unavailable`; non-string input uses Python's
normal type error. This API accepts no mapping, bytes, path, session, receipt,
or Qt object. It excludes batch/multi-request transport, protocol paths,
network, adapter discovery, chemistry conversion, CD-SVG/compressed input,
selection/root export, templates, clipboard, recovery copies, document
mutation, and render observation. Existing direct extension values remain the
Ferrum-Qt integration surface, not a public CLI contract.

Five compact offline Rust semantic cases and two installed-Python semantic
cases are permanent coverage. The real CLI runner, generator, wheel/schema
resource check, package build, and installed walkthrough are E2E or one-time
evidence, not byte, pixel, timing, count, network, mock, or fixture-matrix
gates.

## Ferrum-Qt

`ferrum-qt` is the sole desktop product command. It opens an uncompressed local
`.cdml` document or a decoded local `.svg` containing exactly one canonical embedded
CDML payload. File admission and rendering occur through Rust-owned profiles; Python
does not parse the source or choose its resource limits.

Use File > New for another empty document. File > Open creates a native tab after
admission succeeds; File > Open in Current Tab... replaces the selected tab only
after admission and the required save or replacement choice succeed. Save and Save
As publish CDML through Rust. The Recent Files menu reuses the same native open route.

For a decoded CD-SVG source, Ferrum opens only the embedded CDML. Its wrapper is not
rendered, preserved, or rewritten. Save therefore uses CDML Save As and never
overwrites the source SVG wrapper.

### Supported drawing work

The bounded editor supports Rust-owned document changes including atom and bond
editing, bounded molecule import, supported peptide and ring insertion, selected
molecule inspection, coordinate work, geometry tools, presentation and text edits,
Undo/Redo, Save/Save As, and native artifact export. Available actions enable only
when their selection and document requirements are met; visible refusal leaves the
document unchanged.

File > Export... creates one complete current document as:

- Export SVG...
- Export PDF...
- Export PNG (1 pixel per point)...

SVG and PDF are vector output. PNG has a transparent background and one output pixel
per Rust page point; this describes page geometry, not a print-DPI metadata promise.
Export does not include selection or hover feedback. Cancelling an export destination,
or a document change before publication, leaves the document unchanged.

File > Recovery Export CDML... writes a recovery copy of the current CDML. It does
not replace the save target, change unsaved state, export another format, or convert a
file.

### Refused formats and drops

Ferrum-Qt refuses `.cdxml`, `.cml`, `.cdsvg`, `.svgz`, and compressed CDML names
before reading them. The current document remains unchanged; use the source application
or a converter to produce an uncompressed `.cdml` drawing. Ferrum does not sniff
suffixes, decompress input, export CD-SVG, or preserve a CD-SVG wrapper.

Ferrum is not yet a complete general-purpose chemical-drawing editor or a
cross-platform desktop distribution. Workflows outside the supported Rust-owned route,
including broad legacy-format conversion and unbounded chemistry import, are explicit
pre-production drops rather than alternate desktop paths.

## Package-release evidence

M20 and M22 have accepted source implementation for one proposed initial target, macOS arm64
with CPython 3.12. The route builds exactly two first-party Python wheels: `ferrum-chem` and
`ferrum-qt`. The Rust `ferrum` CLI is deliberately separate and remains installed through Cargo;
installing either Python wheel does not provide it.

The release route is a maintainer-only E2E procedure. It needs a separately provisioned offline
Cargo home, native source input, Qt build-backend wheelhouse, and Qt runtime dependency wheelhouse.
The build and installation use scrubbed environments and `--no-index`; they do not use an index,
an editable checkout, an ambient Python package, or a loader-path workaround. The installed proof
checks one admitted protocol operation and its schema resource, the `ferrum-qt` entry point and
owned resources, then the same chemistry observation after the target-specific LGPL relink route.

The external macOS arm64/CPython 3.12 wheelhouses are currently unavailable, so this remains
pending runtime evidence rather than a supported consumer release. The final M22 classifier also
requires the two final wheels, a committed source archive, and the M20 receipt; human legal and
release review remain required. Build/site inspection, toolchain inventory, clean installation,
relink, source-archive CLI, and artifact-inventory observations are E2E or disposable release
evidence; they are not permanent pytest, timing, byte, hash, member-count, pixel, network, or
matrix gates. For the exact maintainer commands and input roles, see
[INSTALL.md](INSTALL.md#m20-package-release-proof).

## Known gaps

- TODO: complete the real macOS arm64/CPython 3.12 offline package/relink evidence and publish a
  supported consumer installer only after it succeeds.
- TODO: qualify additional desktop platforms before documenting them as supported.
- TODO: expand drawing and chemistry workflows through separately reviewed Rust-owned
  contracts.
