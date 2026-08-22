# Ferrum file formats

This page defines the implemented Ferrum file and stream boundaries. It is a
pre-production format guide, not a promise of general chemical-format conversion.
For commands and installation, see [USAGE.md](USAGE.md) and [INSTALL.md](INSTALL.md).

## CDML documents

The protocol accepts UTF-8 CDML text inside one JSON request. Its one native Qt document
window accepts decoded local `.cdml` and `.svg` inputs through distinct Rust-owned
profiles. A native `.svg` input is CD-SVG only when
it contains one canonical embedded CDML payload; it is not general SVG import.

Ferrum parses CDML into a typed document while retaining the parsed XML structure.
Its rewrite contract preserves:

- Persistent object identifiers and direct-record order.
- Element and attribute names by namespace URI and local name.
- Attribute and text values, namespace context, and ordered children.
- Comments, processing instructions, mixed content, and opaque XML content.

The contract is structural, not lexical. A rewrite can normalize namespace prefixes,
attribute order, whitespace representation, or other serializer details. A successful
rewrite does not promise byte-for-byte identity.

`document.rewrite` is the protocol operation that verifies and structurally emits the
admitted CDML as JSON data. The desktop Save/Save As path publishes CDML through the
Rust-owned session. Neither surface promises byte-for-byte output.

## Native CD-SVG Open

The native desktop Open route accepts a local regular `.svg` file only when decoded
UTF-8 SVG in the SVG namespace contains exactly one descendant `cdml` in the canonical
CDML namespace. It applies independent complete resource envelopes to the wrapper and
the normalized selected payload: 16 MiB UTF-8 bytes, 262,144 elements, depth 64,
1,048,576 attributes, and 8 MiB lexical text or CDATA bytes for each envelope.

Ferrum discards the wrapper after it selects and validates the payload. SVG elements,
scripts, styles, images, references, metadata, geometry, and presentation never become
editable facts; the route does not fetch, render, preserve, or save them. The resulting
tab is clean but has no CDML publication path, so Save opens CDML Save As. A successful
`.cdml` publication establishes the future Save destination and never overwrites the
source wrapper. The original descriptor identity remains available only to activate an
already-open tab, including a hard-link alias.

The desktop route rejects unsafe or malformed wrappers, absent or multiple canonical
payloads, exhausted envelopes, and rejected payloads with stable recovery guidance.
It does not sniff suffixes or decompress input. `.cdsvg`, `.svgz`, compression, wrapper
round trips, and CD-SVG export are outside this V1 desktop boundary. The native window
refuses `.cdsvg`, `.svgz`, and compressed names; it does not offer a second editor or
converter fallback.

## Dropped desktop formats

The pre-production desktop product has one Rust-native document window. It supports
decoded `.cdml`, the bounded decoded `.svg` CD-SVG route described above, and the closed
Rust-owned CML/CML2 simple-molecule profile through File/Open. CML always converts into a
clean new document; its source path is provenance only, and Save writes authoritative CDML.
It refuses `.cdxml`, `.cdsvg`, `.svgz`, and compressed CDML names before reading them,
preserving the active document. Ferrum does not provide a second editor or converter
fallback for these dropped desktop formats. This is an explicit format disposition, not a
claim that historical source or oracle references disappeared.

## Native document artifact export

The ordinary native desktop File menu can publish one complete current document as
SVG, PDF, or transparent PNG. These are rendered from Rust's complete document plan,
not from an imported SVG wrapper or a Qt scene. SVG and PDF are vector artifacts; PNG
uses one output pixel per Rust page point and does not promise physical-density metadata.
Unsupported complete roots refuse the requested artifact rather than producing a partial
document. This bounded desktop route is not general SVG import/export, CD-SVG export,
or a wrapper-round-trip contract.

For locally admitted CDML or decoded CD-SVG, publication retains a Rust-owned descriptor
for the source while the tab is live. The ordinary artifact route rejects that source and
an observed hard-link alias as destinations. It does not compare lexical paths, expose a
source descriptor to Python, or preserve CD-SVG wrapper bytes.

## JSON operation protocol

`ferrum protocol schema` prints the generated protocol schema. `ferrum protocol run`
reads one request JSON file or standard input and writes one JSON success or typed-error
envelope. Request payloads carry owned CDML or molecular-interchange text, never protocol
file paths. The six closed V1 operations include document inspection, validation, rewrite,
artifact rendering, chemistry conversion, and coordinate generation. See
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md) for the complete envelopes, categories,
publication rules, and separate bounds that protect transport, CDML, interchange, and artifacts.

The local native runtime is currently limited to macOS arm64. Its source and
licensing boundary is documented in [PROVENANCE.md](PROVENANCE.md); this is not
a cross-platform release claim.

## Unsupported formats

Ferrum is not a general image, SVG, or compressed-SVG converter. `ferrum convert` accepts only
its closed interchange vocabulary and uses the native runtime created by `build.sh`; it is not a
desktop import fallback. The current scope and remaining migration work are tracked in
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).
