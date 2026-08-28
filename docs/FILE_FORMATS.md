# Ferrum file formats

This page defines the implemented file and stream boundaries for Ferrum. The
formats are Rust-owned, versioned contracts; they are not a promise of general
chemical-format conversion. [FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md)
defines the JSON protocol. [USAGE.md](USAGE.md) contains task-oriented commands.

## Format authority

The Rust registries are the authoritative source for accepted names, suffixes,
operations, resource limits, and runtime requirements. The desktop File/Open
catalog joins its native routes with the same interchange descriptors used by
the CLI. The human-readable and JSON forms of `ferrum formats` expose the
current conversion capability snapshot; scripts should query it instead of
maintaining their own format table.

### Implemented ownership boundary

Every route documented on this page is implemented. A format mentioned only in
an active plan or read-only reference tree is not a Ferrum capability. In
particular, `OTHER_REPOS/` is migration evidence only: it is neither packaged
nor consulted while Ferrum opens, converts, saves, imports, or renders a
document.

Rust owns the closed format descriptors, decoder choice, resource admission,
candidate-document transaction, CDML serialization, and complete SVG, PDF, or
PNG artifact construction. PyO3 transports Rust-issued descriptors, prepared
sessions, observations, and artifact receipts; it is not a second decoder,
format registry, or serializer. Qt chooses a user path and a Rust-issued
descriptor, then owns tab and destination lifetime. It does not infer a
format, reconstruct a route from a suffix, parse interchange text, or export
its scene as a document artifact. The exact Python/Qt lifecycle contract is in
[QT_CONTRACT.md](QT_CONTRACT.md).

Every admitted local source is a regular, uncompressed file. Ferrum does not
sniff suffixes, fetch external resources, or implicitly decompress input. A
successful route produces a complete result or a typed refusal; it does not
retain partial document state.

## Desktop document input

The native desktop File/Open catalog has these closed routes:

| Source | Accepted profile | Result | Current-tab policy |
| --- | --- | --- | --- |
| `.cdml` | Native CDML | Complete Ferrum document | Replace a pristine tab or open a new tab |
| `.svg` | Decoded CD-SVG only | Embedded CDML document | Replace a pristine tab or open a new tab |
| `.cml` | CML/CML2 simple molecule | Clean new CDML document | New tab only |
| `.cdxml` | CDXML simple molecule | Clean new CDML document | New tab only |
| `.sdf`, `.sd` | SDF | Clean new CDML document | New tab only |

The CML and CDXML profiles accept at most 1 MiB of source bytes. The SDF
profile requires the installed trusted chemistry runtime. Each interchange
profile rejects source facts that it cannot represent, rather than silently
dropping them. CDXML additionally reports its declared conversion losses in
the stable order `lexical_syntax`, then `document_view_metadata`.

### CDXML simple-molecule depiction profile

The CDXML profile is deliberately narrow. For a nondirected single CDXML bond,
the only nonstereochemical `Display` values admitted as durable bond
presentation are `Wavy`, `Bold`, and `Dash`. The CDXML decoder keeps those
facts in its source-specific, bond-order-aligned carrier; the document adapter
then maps them exactly to the closed CDML presentation tokens `s1`, `b1`, and
`d1`, respectively. The carrier is not a general interchange presentation
field and is not exposed as a second document schema.

| CDXML `Display` | Preconditions | Durable Ferrum presentation | CDML token |
| --- | --- | --- | --- |
| `Wavy` | Single order and no bond direction | Wavy | `s1` |
| `Bold` | Single order and no bond direction | Bold | `b1` |
| `Dash` | Single order and no bond direction | Dashed | `d1` |

`Display` omitted or `Solid` is ordinary presentation. `WedgeBegin` and
`WedgedHashBegin` remain stereochemical directions, not entries in this
presentation table. A selected presentation on a double or triple bond, on a
directed bond, or any other unsupported CDXML semantic fact is a typed refusal;
Ferrum never guesses a substitute style or discards the fact.

The resulting CDML uses the closed authoring vocabulary documented in
[CDML_FORMAT_SPEC.md](CDML_FORMAT_SPEC.md): `n1`, `n2`, `n3`, `w1`, `h1`,
`q1`, `b1`, `d1`, and `s1`. In particular, this profile does not establish
generic compatibility spellings or multi-order Bold, Dashed, or Wavy bonds.

### Interchange publication transaction

All interchange formats use one detached candidate-document transaction. The
candidate is committed privately, then Ferrum requires an issue-free,
unsuppressed Rust render observation at that candidate's exact current
revision. Only then can a CLI response or desktop prepared tab receive the
session. A decode, insertion, or render refusal drops the candidate: there is
no partial tab, partial document, or retained current-tab mutation.

The first Save or Save As for every interchange-imported document publishes
CDML. Import does not establish the original interchange source as a future
save destination. `ferrum open` follows the same new-document policy and
requires an explicit named CDML output, for example:

```bash
build/bin/ferrum open molecule.cdxml --format cdxml --output result.cdml
```

Rust owns the desktop catalog, CML/CDXML/SDF decoder selection, and output
ownership. The public protocol summary remains in
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

## Conversion is not desktop File/Open

`ferrum formats` describes the Rust conversion-capability registry. Its
conversion inputs and targets are not an additional desktop File/Open list.
For example, the CLI can declare molecular record formats such as SMILES,
InChI, and MDL molfile for `ferrum convert`, but the desktop opens only the
five suffix routes in the preceding table.

The three interchange descriptors deliberately have different roles:

| Interchange source | New-document import | `ferrum convert` | Conversion output |
| --- | --- | --- | --- |
| CML/CML2 | Yes | Yes | Canonical CML2 |
| CDXML simple molecule | Yes | No | None |
| SDF | Yes | Yes | SDF V2000 or V3000 |

`ferrum open` selects only a `document_import_new` descriptor and always
publishes a new CDML document. `ferrum convert` selects an input and an output
from its separate registries; named input suffixes may select an input format,
and `--from` declares one explicitly. Neither command turns a general
interchange format into a desktop current-tab import route.

## Native CDML and CD-SVG

Native CDML is UTF-8 XML in Ferrum's CDML namespace. The local V1 profile
admits at most 16 MiB of UTF-8 source, 262,144 elements, depth 64, 1,048,576
attributes, and 8 MiB of combined text or CDATA bytes. Ferrum parses CDML into
a typed document while retaining the XML structure. A successful rewrite
preserves document facts, namespace URIs and local names, ordered children,
comments, processing instructions, mixed content, and opaque XML content. It
does not promise byte-for-byte serialization identity.

A local `.svg` route is CD-SVG only when a UTF-8 SVG-namespace wrapper contains
exactly one canonical embedded CDML descendant. The wrapper and selected,
normalized payload each receive the full native CDML resource envelope. Ferrum
discards the wrapper after selecting its CDML payload. SVG geometry, scripts,
styles, images, references, metadata, and presentation do not become editable
document facts and are never preserved for a later save.

A CD-SVG tab is clean but initially has no CDML publication destination, so its
first Save opens CDML Save As. Ferrum retains the source descriptor only to
avoid publishing CDML or an artifact over the opened wrapper or a detected
hard-link alias. It does not expose that descriptor to Python or round-trip
wrapper bytes.

`.cdsvg`, `.svgz`, compressed input, multiple payloads, missing payloads, and
general SVG import are outside this V1 boundary. The profile fixes separate
wrapper and payload limits so that a wrapper cannot spend the payload budget.

## Rendered document artifacts

The desktop File menu and `ferrum render` publish a complete current CDML
document as one of these artifact formats:

| Artifact | Representation |
| --- | --- |
| SVG | Complete vector document artifact |
| PDF | Complete vector document artifact |
| PNG | Transparent raster artifact with one output pixel per Rust page point |

Artifacts come from Rust's complete document render plan, not from a Qt scene
or an imported SVG wrapper. Unsupported complete roots refuse the requested
artifact rather than emitting a partial file. Artifact SVG is not CD-SVG export
and does not become general SVG import.

`ferrum render` infers `svg`, `pdf`, or `png` from a named output suffix, or
requires the corresponding `--to` value when suffix inference is unavailable.
The `document.render_artifact` protocol outcome carries the complete artifact
as standard base64 JSON data; it does not accept an output path in the request.

## CLI conversion and export

`ferrum convert` is a molecular-interchange route, separate from desktop
document import. Its accepted source and target combinations are the current
Rust capability catalog reported by `ferrum formats`; conversion never uses a
Qt document or an imported SVG wrapper. CDXML is document-import-only, so
`ferrum convert --from cdxml` refuses before reading the source and directs the
caller to `ferrum open`.

`ferrum inspect-graph` accepts the declared CML or SDF inspection profile and
returns a bounded decoded-semantic graph summary without constructing a
document. CML inspection is runtime-free; SDF inspection requires the trusted
native runtime and reports native normalization rather than source-byte
fidelity.

`ferrum document export-sdf` writes a complete, atomically published multi-root
SDF artifact from selected direct CDML molecules. It requires a named output
and explicit `v2000` or `v3000` record syntax. This is a selected-document
export, not evidence that every desktop input format has a matching exporter.

## JSON streams

`ferrum protocol schema` prints the generated V1 JSON schema. `ferrum protocol
run` reads one UTF-8 request JSON object from a named file or standard input and
writes one complete success or typed-error envelope. Its fixed transport bound
applies before JSON parsing; CDML, interchange, artifact, and response bounds
apply independently afterward.

Protocol requests contain owned text and closed format names. They never carry
filesystem paths, library handles, live sessions, Qt objects, retained source
descriptors, or adapter locators. The protocol's operation set is closed but
evolves independently of this file guide; use the generated schema and
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md) for exact request and response
members.

## Refused format families

Ferrum is not a general image converter, general SVG editor, compressed-SVG
reader, CD-SVG exporter, or wrapper round-trip tool. CDX binary input, `.cdsvg`,
`.svgz`, compressed containers, and CDXML outside the simple-molecule profile
are refused without changing the current document. The current migration scope
and its remaining work are tracked in
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).
