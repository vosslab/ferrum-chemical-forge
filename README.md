# Ferrum Chemical Forge

Ferrum is a Rust chemical-document tool for inspecting, validating, rewriting, rendering,
converting, and drawing durable CDML records.

> Status: pre-production. The command-line document verbs work from a Cargo installation. The
> desktop editor and engine-backed chemistry verbs have explicit platform and bundle prerequisites.

<!-- screenshots:begin (managed by screenshot-docs) -->
<!-- screenshots:end -->

## Start with a document

Rust 1.97.1 or newer is required. From a source checkout, this non-engine example inspects an
authored CDML document without installing a chemistry engine:

```bash
cargo run --manifest-path packages/ferrum-rust/Cargo.toml --locked --quiet \
  --bin ferrum -- inspect tests/e2e/corpus/authored_document_forms.cdml
```

After installing the CLI with the command in [docs/INSTALL.md](docs/INSTALL.md), the same work is:

```bash
ferrum inspect drawing.cdml
ferrum render drawing.cdml --to svg --output drawing.svg
```

## Convert a molecule

`convert` and `coords` use an explicitly installed native chemistry engine bundle. First obtain a
bundle built for this Ferrum executable and host from the release process, then install it once:

```bash
ferrum engine install /path/to/ferrum-engine-bundle
ferrum engine status
ferrum convert aspirin.smi --to sdf_v2000 --output aspirin.sdf
```

`ferrum engine status` prints `ready` only for a valid active bundle. Without one, `convert` and
`coords` return a typed `chemistry_unavailable` refusal and leave requested output files untouched.
Ferrum never searches the current directory, `PATH`, Python environments, or adapter variables for
an engine. [docs/USAGE.md](docs/USAGE.md) explains the six verbs, stream use, desktop workflow,
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

## Installation and status

Ferrum currently has a proposed macOS arm64 CPython 3.12 desktop release route, not a supported
consumer desktop distribution. The Rust CLI is installed independently from the Python wheels. See
[docs/INSTALL.md](docs/INSTALL.md) for setup and [docs/PROVENANCE.md](docs/PROVENANCE.md) for
concise source lineage and license provenance.

## License

Ferrum is [AGPL-3.0-only](LICENSE.AGPL-3.0.md). Ferrum-Chem is
[LGPL-3.0-only](LICENSE.LGPL-3.0.md). See [docs/PROVENANCE.md](docs/PROVENANCE.md) for the
license boundary and required notices.
