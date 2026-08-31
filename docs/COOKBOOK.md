# Ferrum cookbook

These end-to-end recipes use the repository-local build. Build first with
`./build.sh`; the runnable programs are `build/bin/ferrum` and
`build/bin/ferrum-qt`. They complement the command reference in
[USAGE.md](USAGE.md) and the admitted-file contract in
[FILE_FORMATS.md](FILE_FORMATS.md).

## Review and render CDML

Use this sequence to inspect an existing CDML drawing, produce a separate
review artifact, and then open the original in Ferrum. It does not save from
the desktop application or rewrite the input.

```bash
./build.sh
build/bin/ferrum validate supplied.cdml --level typed
build/bin/ferrum inspect supplied.cdml --json
build/bin/ferrum render supplied.cdml --output supplied.svg
build/bin/ferrum-qt supplied.cdml
```

`validate` and `inspect` report on the supplied document. `render` publishes a
new artifact only after Ferrum completes the supported render operation. Choose
`supplied.pdf` or `supplied.png` instead when that review format is needed.

In the window, make edits deliberately, then use Save As to choose a CDML
destination. Ferrum structurally re-emits admitted CDML; it does not promise a
byte-for-byte rewrite. See [FILE_FORMATS.md](FILE_FORMATS.md) for the exact
CDML persistence boundary.

## Import a ChemDraw simple molecule

Use `formats` before choosing an interchange path, then create a new CDML
document from a declared source. This recipe uses Ferrum's bounded CDXML
simple-molecule profile. CDXML is input-only: Ferrum creates an editable CDML
document and never treats the source as a future save destination.

```bash
build/bin/ferrum formats
build/bin/ferrum open molecule.cdxml --format cdxml --output molecule.cdml
build/bin/ferrum inspect molecule.cdml --json
build/bin/ferrum-qt molecule.cdml
```

The Rust-owned import must decode the complete source, create one complete
issue-free render plan, and pass the new-document admission before it publishes
`molecule.cdml`. A typed refusal leaves no partial CDML destination or desktop
tab. The desktop File > Open route applies the same contract and always opens
CDXML in a new clean tab; use Save or Save As to choose its CDML destination.

The current profile preserves ordinary, Wavy, Bold, and Dash ChemDraw single
bonds as editable Ferrum presentation. Wavy, Bold, and Dash are fixed-single
styles: a source bond that combines one with a non-single order is refused,
rather than coerced to another bond type. Other CDXML drawing, document-view,
and chemistry features remain outside this bounded profile. Capability limits,
accepted suffixes, and declared losses are defined by
[FILE_FORMATS.md](FILE_FORMATS.md), not inferred from an extension.

## Import a CML simple molecule

Use CML/CML2 simple-molecule import when you need a new editable Ferrum
document from a declared CML source. The source remains input-only; the
published result is CDML.

```bash
build/bin/ferrum formats --json
build/bin/ferrum open molecule.cml --format cml --output molecule.cdml --json
build/bin/ferrum validate molecule.cdml --level typed
build/bin/ferrum-qt molecule.cdml
```

The JSON response summarizes the completed import without embedding the source
or the new document. Ferrum publishes `molecule.cdml` only after decoding,
candidate-document admission, and an issue-free render observation succeed. A
typed refusal leaves no partial CDML output. CML's accepted profile and its
resource limits are defined in [FILE_FORMATS.md](FILE_FORMATS.md).

## Open styled CDXML in the desktop application

Use the normal File > Open command to keep desktop import on the Rust-owned
new-document route. A small source containing all currently preserved external
bond presentations is useful for visual review:

```xml
<CDXML><page><fragment id="presentation-fragment">
  <n id="a" p="0 0"/><n id="b" p="20 0"/>
  <n id="c" p="40 0"/><n id="d" p="60 0"/>
  <b B="a" E="b" Display="Wavy"/>
  <b B="b" E="c" Display="Bold"/>
  <b B="c" E="d" Display="Dash"/>
</fragment></page></CDXML>
```

Save that text as `styled.cdxml`, start `build/bin/ferrum-qt`, and choose
**File > Open**. The new tab shows the three source presentations and remains a CDML
document after its first Save or Save As. This is a bounded interoperability
workflow, not general ChemDraw compatibility; use the current File/Open
catalogue and typed refusals as the authority for unsupported input.

## Inspect interchange semantics before import

Use graph inspection when the question is whether a declared CML or SDF source
contains the semantic facts Ferrum can decode, rather than whether it should
become an editable document. This route emits a bounded decoded-graph summary
and does not construct or publish CDML.

```bash
build/bin/ferrum formats --json
build/bin/ferrum inspect-graph molecule.cml --from cml --json
```

The result identifies the admitted inspection profile, record, atom, and bond
counts, plus declared fact coverage and normalization. Choose `--from sdf` for
a declared SDF source when the trusted chemistry runtime is available. Use
`open` only after the inspection result supports creating a new CDML document.

## Automate a protocol request

Use the generated schema as the contract for an integration, then exchange one
JSON request and one JSON response through the frozen protocol route.

```bash
build/bin/ferrum protocol schema > ferrum-operation-v1.schema.json
build/bin/ferrum protocol run request.json --output response.json
```

`request.json` must be a UTF-8 `ferrum-operation-request-v1` envelope accepted
by the generated schema. `protocol run` publishes the response destination
safely and keeps successful JSON output distinct from diagnostics. Use a named
`document` command only when the integration needs that command's single,
versioned operation route; the generic protocol route remains the complete
automation surface. See [USAGE.md](USAGE.md) for supported named document
commands and [INSTALL.md](INSTALL.md) for the local runtime boundary.
