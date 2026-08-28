# Ferrum Chemical Forge

A pre-production chemical-document workspace for chemists and developers, with one
Rust-owned CDML model serving both a command-line tool and a PySide6 drawing editor.

> Status: Ferrum runs from this checkout on macOS arm64. Its bounded Rust and Qt
> workflows are actively developed; it is not release-ready and does not yet claim
> complete BKChem or OASA feature parity.

## One document model, two routes

Ferrum makes the document rather than the interface the source of truth. Rust owns
the CDML record, validation, document history, chemistry boundaries, and rendering
plans. The `ferrum` CLI and the `ferrum-qt` editor use that same native model, so a
validated document can move between scripted inspection and interactive authoring
without a second chemistry implementation.

- Inspect, validate, rewrite, render, convert, and run versioned document operations
  from the local Rust CLI.
- Open and author the supported CDML drawing slice in the PySide6 editor, including
  save, Undo/Redo, and Rust-produced SVG, PDF, and PNG artifacts.
- Preserve structural CDML content and persistent identity through the supported
  document path; successful rewrite is structural, not byte-for-byte, preservation.
- Refuse unsupported files and document features with typed outcomes rather than
  silently changing the active document.

## Ferrum in the workspace

The desktop application is a chemical drawing workspace, not a separate file-format
viewer. The screenshot set below records the current bounded editor routes; the
following capture pass maintains its files, captions, and alt text.

<!-- screenshots:begin (managed by screenshot-docs) -->
<!-- screenshots:end -->

## Quick start

Ferrum's current local route requires Rust 1.97.1 or newer, Python 3.12, and a
macOS arm64 host. From a checkout, build the CLI, native runtime, and Qt launcher:

```bash
./build.sh
build/bin/ferrum --version
build/bin/ferrum formats
```

The last command is a useful first result: it prints the declared input/output
format catalog and whether each route needs the local chemistry runtime. The
launchers stay below `build/`; they do not install Ferrum globally or discover a
per-user engine.

To launch the desktop application after the same build:

```bash
build/bin/ferrum-qt
```

For the full local setup, test route, and runtime layout, use
[docs/INSTALL.md](docs/INSTALL.md).

## A small CLI transformation

Ferrum's conversion route accepts a closed set of declared formats. This example
turns a SMILES record into a V2000 molfile on standard output:

```bash
printf 'CCO\n' | build/bin/ferrum convert - --from smiles --to molblock_v2000
```

The output begins with a three-atom, two-bond V2000 record. Add `--output ethanol.mol`
to publish the result as a file. See [docs/USAGE.md](docs/USAGE.md) before using
interchange paths in a workflow: CML/CML2, CDXML, CD-SVG, and document artifacts
each have deliberately bounded admission and publication rules.

## What is supported today

CDML is Ferrum's sole document, session, history, and Qt-local format. The current
desktop route supports decoded local CDML, bounded CD-SVG and CDXML ingress, and
closed CML/CML2 simple-molecule input. It can save CDML and export complete supported
documents as SVG, PDF, or transparent PNG.

This is an active migration, not a general-purpose chemistry format converter or a
finished desktop replacement. Legacy BKChem and OASA material is reference evidence
outside the product runtime; the current scope, format refusals, and remaining parity
work are explicit in the documents below.

## Documentation routes

Start here for the task at hand:

- [docs/INSTALL.md](docs/INSTALL.md) - requirements, local build, and verification.
- [docs/USAGE.md](docs/USAGE.md) - CLI verbs, editing workflows, and protocol examples.
- [docs/FILE_FORMATS.md](docs/FILE_FORMATS.md) - admitted files, conversion limits,
  and artifact publication rules.
- [docs/FERRUM_API_CONTRACT.md](docs/FERRUM_API_CONTRACT.md) - versioned operation
  envelopes, results, and typed failures.
- [docs/CODE_ARCHITECTURE.md](docs/CODE_ARCHITECTURE.md) - Rust, PyO3, and Qt ownership
  boundaries and data flow.
- [docs/FILE_STRUCTURE.md](docs/FILE_STRUCTURE.md) - repository map and where each kind
  of work belongs.
- [docs/QT_CONTRACT.md](docs/QT_CONTRACT.md) - the desktop integration contract and
  supported interaction boundaries.
- [FULL_PARITY_RUST_FIRST.md](docs/active_plans/active/FULL_PARITY_RUST_FIRST.md).
  Authoritative migration and parity ledger.

## Provenance and license

Ferrum is the AGPL-3.0-only PySide6 application; Ferrum-Chem is the LGPL-3.0-only
Rust workspace. [docs/PROVENANCE.md](docs/PROVENANCE.md) records their boundary,
lineage, notices, and the limits of the current pre-production claim.
