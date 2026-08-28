# Usage

Ferrum works from this checkout's local program. Use the Rust CLI for document
and molecular-interchange work, and the PySide6 application for interactive
CDML drawing. Build first as described in [INSTALL.md](INSTALL.md). The
canonical selected program is `build/current`; `build/bin` is its stable
shorter launcher path.

## Quick start

Build, inspect a CDML document, and render an SVG:

```bash
./build.sh
build/bin/ferrum inspect drawing.cdml
build/bin/ferrum render drawing.cdml --output drawing.svg
```

```bash
build/bin/ferrum-qt
build/bin/ferrum-qt drawing.cdml
```

For repository Python tools or tests, first select the staged extension from
the same local program:

```bash
source source_me.sh && python3 -m pytest
```

## CLI discovery

```bash
build/bin/ferrum --help
build/bin/ferrum formats
build/bin/ferrum formats --json
```

`formats` reports declared input and output eligibility without reading a
source. Use `--json` for the versioned response.

## Common document tasks

Inspect or validate a CDML document. Each accepts a named path or `-`; `--json`
returns the complete operation envelope.

```bash
build/bin/ferrum inspect drawing.cdml
build/bin/ferrum validate drawing.cdml --level typed
build/bin/ferrum inspect drawing.cdml --json
```

Rewrite a structurally admitted document, or render it as SVG, PDF, or
transparent PNG. Ferrum infers a named render format from the output suffix;
use `--to` when writing to standard output or a suffix it cannot infer.

```bash
build/bin/ferrum rewrite drawing.cdml --output normalized.cdml
build/bin/ferrum render drawing.cdml --output drawing.pdf
build/bin/ferrum render drawing.cdml --to png --output drawing.png
```

Structural rewriting preserves admitted CDML structure and identity, not bytes.
See [FILE_FORMATS.md](FILE_FORMATS.md) for the authoritative file contract.

## Molecular interchange

Convert one declared molecular-interchange source through the local program.
Use `--from` for standard input or to override suffix-based detection.

```bash
build/bin/ferrum convert aspirin.smi --to sdf_v2000 --output aspirin.sdf
printf 'CCO\n' | build/bin/ferrum convert - --from smiles --to molblock_v2000 \
  --output ethanol.mol
build/bin/ferrum inspect-graph records.sdf --from sdf --json
```

Create a new CDML document from one declared interchange input with `open`:

```bash
build/bin/ferrum open molecule.cdxml --format cdxml --output molecule.cdml
```

`convert`, `open`, and `inspect-graph` accept only formats declared by
`formats`. CDML is Ferrum's sole editable document and session format. Read
[FILE_FORMATS.md](FILE_FORMATS.md) for accepted profiles, resource boundaries,
and refused formats.

## Desktop drawing

Use File > Open to load an eligible local document and author through the
visible tool and menu commands. Native CDML and decoded CD-SVG can replace a
pristine tab or open a new tab; CML, CDXML, and SDF always create a clean new
CDML tab. Save and Save As publish CDML; export commands produce SVG, PDF, or
transparent PNG. Native editing supports Undo/Redo and typed refusals without
changing the active document.

The bounded CDXML simple-molecule workflow preserves ordinary, wavy, bold, and
dashed single-bond presentation in the new editable document. Use the regular
File > Open dialog for a `.cdxml` file, then Save or Save As to choose its first
CDML destination. [FILE_FORMATS.md](FILE_FORMATS.md) owns the precise admission
grammar, conversion losses, limits, and refused CDXML features.

For an interactive tour, see [GUI_TOUR.md](GUI_TOUR.md). For exact external
operation envelopes and machine-facing result categories, see
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

## Machine protocol

The protocol runner reads one UTF-8 JSON request or standard input and writes one response envelope:

```bash
build/bin/ferrum protocol schema
build/bin/ferrum protocol run request.json
build/bin/ferrum protocol run request.json --output response.json
```

Request payloads carry document or molecular-interchange text, not input file
paths. The generated schema and closed request and response contracts are in
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

## Inputs and outputs

- CDML is the native editable document input for `inspect`, `validate`,
  `rewrite`, `render`, and `coords`.
- Declared molecular-interchange inputs are discovered with `formats` and used
  by `convert`, `open`, or `inspect-graph` according to their eligibility.
- `render` writes complete SVG, PDF, or transparent PNG artifacts.
- `open` writes a new CDML file; the desktop application's normal save path is
  also CDML.
- `protocol run` accepts JSON and writes one JSON response envelope.

## Known gaps

- Verify a release-grade desktop workflow with human visual and accessibility review.
- Verify format-specific examples against the current `formats --json` catalog before adding names.
