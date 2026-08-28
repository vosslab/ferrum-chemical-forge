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

The first Save or Save As for every interchange-imported document publishes
CDML. Import does not establish the original interchange source as a future
save destination. `ferrum open` follows the same new-document policy and
requires an explicit named CDML output, for example:

```bash
ferrum open molecule.cdxml --format cdxml --output result.cdml
```

Rust owns the desktop catalog, CML/CDXML/SDF decoder selection, and output
ownership. The public protocol summary remains in
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

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
[ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).
