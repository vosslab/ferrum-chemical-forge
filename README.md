# Ferrum Chemical Forge

Ferrum is a Rust chemical-document tool for inspecting, validating, rewriting, rendering,
converting, and drawing durable CDML records.

> Status: pre-production. `./build.sh` creates the local Rust CLI and PySide6 desktop
> application together with their private native runtime.

<!-- screenshots:begin (managed by screenshot-docs) -->
<!-- screenshots:end -->

## Start with a document

Rust 1.97.1 or newer is required. Build the local application, then inspect an
authored CDML document:

```bash
./build.sh
build/bin/ferrum inspect tests/e2e/corpus/authored_document_forms.cdml
```

`build/bin/ferrum render drawing.cdml --to svg --output drawing.svg` writes a
local SVG artifact.

## Convert a molecule

The local runtime assembled by `build.sh` supplies the native chemistry adapter:

```bash
build/bin/ferrum convert aspirin.smi --to sdf_v2000 --output aspirin.sdf
```

The local launchers select their runtime below `build/` rather than a global
installation. [docs/USAGE.md](docs/USAGE.md) explains the six verbs, stream use, desktop workflow,
and the engine lifecycle.

## Draw a CDML record

Run `ferrum-qt` for the bounded native drawing route. Open a CDML document, use the Atom and Draw
Bond commands to author, use Undo to reverse a change, and Save As to publish CDML. The keyboard
workflow, supported editing slice, and current desktop limits are in [docs/USAGE.md](docs/USAGE.md).

## What Ferrum preserves

Ferrum keeps unknown XML and persistent document identity through its structural CDML path.
Rewriting promises structural preservation, not byte-for-byte identity. Rendering produces complete
SVG, PDF, or transparent PNG artifacts for supported documents. The file profile and desktop import
rules are in [docs/FILE_FORMATS.md](docs/FILE_FORMATS.md).

## Contract and architecture

The six human verbs construct versioned, stateless operation requests. The full envelope schema,
operation payloads, result shapes, error categories, Python boundary, and exclusions are in
[docs/FERRUM_API_CONTRACT.md](docs/FERRUM_API_CONTRACT.md). Architecture and ownership boundaries
are described in [docs/CODE_ARCHITECTURE.md](docs/CODE_ARCHITECTURE.md).

## Local build status

Ferrum is pre-production and currently runs from this checkout. See
[docs/INSTALL.md](docs/INSTALL.md) for the local build path and
[docs/PROVENANCE.md](docs/PROVENANCE.md) for concise source lineage and license provenance.

## License

Ferrum is [AGPL-3.0-only](LICENSE.AGPL-3.0.md). Ferrum-Chem is
[LGPL-3.0-only](LICENSE.LGPL-3.0.md). See [docs/PROVENANCE.md](docs/PROVENANCE.md) for the
license boundary and required notices.
