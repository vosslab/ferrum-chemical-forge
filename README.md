# Ferrum Chemical Forge

A pre-alpha chemical-document workspace for chemists and developers, with
Rust-first chemistry, document, and rendering behind a PySide6 drawing editor and CLI.

> Status: Ferrum runs from this checkout on macOS arm64. Its bounded Rust and Qt
> workflows are actively developed; it is not release-ready and does not yet claim
> complete historical-feature parity.

## One document model, two routes

Ferrum makes the document rather than the interface the source of truth. Rust owns
the CDML record, validation, document history, chemistry boundaries, and rendering
plans. The `ferrum` CLI and the `ferrum-qt` editor use that same native model, so a
validated document can move between scripted inspection and interactive authoring
without a second chemistry implementation.

That shared, typed document path is Ferrum's signature promise: a drawing made in
the desktop workspace and a document inspected or rendered at the command line are
governed by the same Rust-owned chemistry and rendering decisions.

- Inspect, validate, rewrite, render, convert, and run versioned document operations
  from the local Rust CLI.
- Open and author the supported CDML drawing slice in the PySide6 editor, including
  save, Undo/Redo, and Rust-produced SVG, PDF, and PNG artifacts.
- Preserve structural CDML content and persistent identity through the supported
  document path; successful rewrite is structural, not byte-for-byte, preservation.
- Refuse unsupported files and document features with typed outcomes rather than
  silently changing the active document.

## The desktop workspace

Ferrum's desktop application is a chemical drawing workspace, not a separate
file-format viewer. The managed visual proof below is supplied from the current
13-scene documentation tour of authoring, catalog, query, reaction, and command-palette
workflows. See [docs/GUI_TOUR.md](docs/GUI_TOUR.md) for the complete candidate tour.
The screenshot documentation pass supplies the current curated images and their
descriptive alt text in this managed block.

<!-- screenshots:begin (managed by screenshot-docs) -->
![Ferrum workspace showing a carbonyl with aligned atom labels and double bond](docs/screenshots/workspace.png)
![Template Catalog showing the selected alpha-D-glucofuranose before placement](docs/screenshots/template_catalog.png)
![SMARTS Query dock reporting one carbon match in the open carbonyl document](docs/screenshots/smarts_result.png)
![ChemDraw XML C-O-N-F document with wavy, bold, and dashed bonds](docs/screenshots/cdxml_open.png)
![Command Palette listing registered reaction commands above a reaction arrow](docs/screenshots/command_palette_reaction.png)
<!-- screenshots:end -->

## Quick start

Ferrum's current local route requires Rust 1.97.1 or newer, Python 3.12, and a
macOS arm64 host. From a checkout, build the CLI, native runtime, and Qt launcher:

```bash
./build.sh
build/bin/ferrum --version
build/bin/ferrum formats
```

The version command begins with `ferrum`; the last command is the first useful result:
it prints the declared input/output format catalog and whether each route needs the
local chemistry runtime. The launchers stay below `build/`; they do not install Ferrum
globally or discover a per-user engine.

To launch the desktop application after the same build:

```bash
build/bin/ferrum-qt
```

Launch the GUI from an active macOS desktop session. Its current workflow and the
one-time screenshot-capture route are in [docs/GUI_TOUR.md](docs/GUI_TOUR.md).

For the full local setup, test route, and runtime layout, use
[docs/INSTALL.md](docs/INSTALL.md).

## A small CLI transformation

Ferrum's conversion route accepts a closed set of declared formats. This example
turns a SMILES record into a V2000 molfile on standard output:

```bash
printf 'CCO\n' | build/bin/ferrum convert - --from smiles --to molblock_v2000
```

The output is a three-atom, two-bond V2000 record. Add `--output ethanol.mol` to
publish the result as a file. See [docs/USAGE.md](docs/USAGE.md) before using
interchange paths in a workflow: CML/CML2, CDXML, CD-SVG, and document artifacts each
have deliberately bounded admission and publication rules.

## What is supported today

CDML is Ferrum's sole document, session, history, and Qt-local format. Desktop
**File > Open** supports native CDML, decoded CD-SVG, bounded CML/CML2 and CDXML
simple-molecule inputs, plus trusted-runtime SDF (`.sdf` and `.sd`) records. Every
interchange File/Open result is a clean new CDML document: it never inserts into or
replaces the current tab, and its first Save or Save As publishes CDML. Ferrum can
export complete supported documents as SVG, PDF, or transparent PNG.

This is an active migration, not a general-purpose chemistry format converter or a
finished desktop replacement. Historical predecessor material is reference evidence
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
- [docs/SECURITY_DECISIONS.md](docs/SECURITY_DECISIONS.md) - restrictive parser,
  dependency, and security-boundary decisions that future changes preserve.
- [docs/QT_CONTRACT.md](docs/QT_CONTRACT.md) - the desktop integration contract and
  supported interaction boundaries; [docs/YAML_FILE_FORMAT.md](docs/YAML_FILE_FORMAT.md)
  explains the maintained menu, ribbon, and theme resource format.
- [FULL_PARITY_RUST_FIRST.md](docs/active_plans/active/FULL_PARITY_RUST_FIRST.md).
  Authoritative migration and parity ledger.

## Provenance and license

Ferrum is the AGPL-3.0-only PySide6 application; Ferrum-Chem is the LGPL-3.0-only
Rust workspace. [docs/PROVENANCE.md](docs/PROVENANCE.md) records their boundary,
lineage, notices, and the limits of the current pre-production claim.
