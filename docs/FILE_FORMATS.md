# Ferrum file formats

This page defines the implemented Ferrum file and stream boundaries. It is a
pre-alpha compatibility guide, not a promise of general chemical-format conversion.
For commands and installation, see [USAGE.md](USAGE.md) and [INSTALL.md](INSTALL.md).

## CDML documents

Ferrum's Rust document commands accept UTF-8 CDML XML from a file or standard input
(`-`). The native Qt bounded editor accepts `.cdml` files only.

Ferrum parses CDML into a typed document while retaining the parsed XML structure.
Its rewrite contract preserves:

- Persistent object identifiers and direct-record order.
- Element and attribute names by namespace URI and local name.
- Attribute and text values, namespace context, and ordered children.
- Comments, processing instructions, mixed content, and opaque XML content.

The contract is structural, not lexical. A rewrite can normalize namespace prefixes,
attribute order, whitespace representation, or other serializer details. A successful
rewrite does not promise byte-for-byte identity.

Use `ferrum cdml rewrite INPUT --check` to verify this structural contract without
creating an output file. `ferrum cdml rewrite INPUT --output OUTPUT` publishes a named
output through `DocumentSession.save_atomic`. With `--output -`, the rewritten CDML is
written to standard output and has normal stream semantics.

## CD-SVG extraction

`ferrum cdml extract-cdsvg INPUT --output OUTPUT` accepts decoded UTF-8 SVG XML, not
compressed `.svgz`. The SVG root must use the SVG namespace and contain exactly one
descendant `cdml` element in the canonical CDML namespace.

Ferrum retains that embedded CDML subtree, structurally serializes and reparses it,
then publishes the verified CDML through the same named-file or standard-output
boundary as `rewrite`. The SVG wrapper is presentation data only: Ferrum does not
infer editable document state from rendered SVG elements.

## JSON report streams

The CLI sends successful machine reports to standard output and operational failures
to standard error. Argument errors use exit status 2; an accepted command that cannot
read, process, or publish data uses exit status 1.

| Command | Success stream contract |
| --- | --- |
| `cdml inspect` | `ferrum-cdml-inspection-v1` JSON by default; text is not a parsing contract. |
| `cdml validate` | `ferrum-cdml-validation-v1` JSON by default; `--typed` requires core facts. |
| `cdml rewrite --check` | One versioned JSON preservation report. |
| `cdml render-observation` | One complete `ferrum-render-observation-v1` JSON line. |
| `smiles inspect` | One newline-terminated `ferrum-smiles-inspection-v1` JSON object. |
| `molblock inspect` | One newline-terminated `ferrum-molblock-inspection-v1` JSON object. |
| `sdf inspect` | One newline-terminated `ferrum-sdf-inspection-v1` JSON object. |

`render-observation` represents one initial revision-zero CDML session observation,
its matching document digest, the fixed depiction profile, and complete molecule
plans or an explicit failure. It accepts no output-format flag.

`--format text` is available only for `cdml inspect` and `cdml validate`.

## Native chemistry boundary

`ferrum smiles inspect --adapter ABSOLUTE_LIBRARY SMILES` is a provisional native
adapter route, not a file-format converter. The supplied library path must be
absolute, regular, and non-symbolic-link; Ferrum performs no library discovery.

The current adapter boundary is ABI-4. Its native wire vocabularies are internal to
Ferrum's adapter loader and chemistry crate. Users consume the versioned reports or
text formats rather than those bytes.

`ferrum sdf inspect --adapter ABSOLUTE_LIBRARY INPUT` accepts bounded UTF-8 SDF from
a file or standard input (`-`). Its `ferrum-sdf-inspection-v1` report preserves record
order, titles, ordered repeated properties, complete molecule facts, and atom-aligned
finite 2D coordinates. Three-dimensional coordinate import is not yet supported.

`ferrum molblock inspect --adapter ABSOLUTE_LIBRARY INPUT` accepts exactly one bounded
UTF-8 V2000 or V3000 molblock from a file or standard input. Its
`ferrum-molblock-inspection-v1` report contains complete owned molecule facts and
finite atom-aligned 2D coordinates. SDF separators, multiple terminators, and 3D
conformers are rejected rather than guessed.

The accepted ABI-4 FCM1 wheel evidence is limited to macOS arm64. Its packaging and
licensing boundary is documented in [PROVENANCE.md](PROVENANCE.md); it is not a
cross-platform release claim.

## Unsupported formats

Ferrum does not currently provide general import or export coverage for arbitrary
molfile/SDF variants, SMILES files, image formats, or compressed SVG. Its current
molblock and SDF commands are bounded provisional slices. The retained legacy Qt
editor may have separate migration-only capabilities, but they are not part of this
Rust-owned format contract. The current scope and remaining migration work are tracked
in [active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).
