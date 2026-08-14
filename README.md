# Ferrum Chemical Forge

A pre-alpha chemical drawing platform for scientists and educators, pairing a Rust CDML
engine with a PySide6 interface and an OASA-free bounded native CDML editor.

> Status: pre-alpha. The standalone `ferrum-qt --native` route is an accepted macOS arm64
> bounded editor: it opens, renders, changes an atom element, adds one free-standing atom,
> edits all nine supported authored atom properties in one Rust-owned operation,
> connects two existing atoms with a revision-bound Rust-owned single-bond drag, extends an
> atom into empty space with one carbon and bond, moves an atom by dragging its rendered
> position, deletes durable atoms or bonds with their correct Rust-owned topology semantics,
> changes one selected normal bond among single, double, and triple order, imports bounded
> SMILES, regenerates one existing molecule's 2D coordinates while retaining its current
> centroid and mean bond length, applies Rust undo/redo, and saves/reopens through Ferrum-Chem
> without importing OASA.
> It does not yet provide general bond-style tools or a complete native editing workflow. The
> retained full
> `ferrum-qt` editor is still an OASA-backed migration preview.

<!-- screenshots:begin (managed by screenshot-docs) -->
<!-- screenshots:end -->

## One drawing, durable document

Chemical drawings should remain useful scientific records, not fragile pictures tied to one
GUI session. Ferrum is for scientists, educators, and contributors who need a visible drawing
surface plus a testable CDML document engine with explicit preservation boundaries.

- Run Rust-only CDML inspection, validation, structural rewrite, CD-SVG extraction, and
  render-observation reports from the `ferrum` command.
- Open and make bounded atom edits in the standalone OASA-free Qt route with
  `ferrum-qt --native`.
- Keep unknown XML and persistent document identity through Ferrum's structural CDML path.
- Inspect SMILES through a deliberately named ABI-4 native adapter instead of implicit library
  discovery.

## Two current desktop routes

The project deliberately exposes its migration boundary rather than hiding it.

- `ferrum-qt --native drawing.cdml` is the standalone Ferrum-Chem route. On the verified macOS
  arm64 configuration, it opens, renders, changes an atom element, adds one free-standing
  atom, edits all nine supported authored atom properties as one Rust-owned revision, connects
  two existing atoms with a single-bond drag, extends an atom into empty space
  with one carbon and bond, moves an atom with exact Rust-owned coordinates, deletes a durable
  atom with its typed incident bonds or deletes one durable bond while preserving its endpoint
  atoms, changes one selected normal bond among single, double, and triple order, imports bounded
  SMILES, regenerates one durable molecule's coordinates off the Qt thread while preserving its
  current placement, applies Rust undo/redo, and saves/reopens CDML without OASA. Use it for the
  completed bounded native document slice.
- `ferrum-qt drawing.cdml` is the retained full PySide6 editor. It remains useful for its legacy
  interactive drawing workflow, but still depends on OASA and is not evidence of a completed
  Rust cutover.

The native route does not yet choose a non-carbon element or bond order during the bond gesture,
edit wedge, hashed, dashed, aromatic, or ring-side depiction styles, delete other object classes,
regenerate styled or non-ordinary molecule graphs, import arbitrary chemistry, or make a
cross-platform packaging
claim. This is a clean pre-production migration boundary, not a compatibility shim.

## Quick start: inspect a real document

Rust 1.97.1 or newer is required. From a source checkout, run the Rust CLI against the
checked-in authored CDML corpus:

```bash
cd packages/ferrum-rust
cargo run --locked --quiet --bin ferrum -- cdml inspect ../../tests/e2e/corpus/authored_document_forms.cdml
```

Success prints one newline-terminated `ferrum-cdml-inspection-v1` JSON report. In the current
corpus example, `diagnostic_count` is `0` and the report identifies molecule `m1`. For a
durable installation and the full platform requirements, see [docs/INSTALL.md](docs/INSTALL.md).

## Command-line proof

The Rust CLI is useful today without Python or `OTHER_REPOS/` for its CDML commands. For
example, the render path emits the exact observation contract consumed by the native projection:

```bash
ferrum cdml render-observation drawing.cdml
```

It writes one `ferrum-render-observation-v1` JSON line on success. The other current CDML
commands include `inspect`, `validate`, `rewrite`, and `extract-cdsvg`; all accept a file path or
standard input where documented. Existing-molecule coordinate regeneration is also available
through an explicitly named chemistry adapter:

```bash
ferrum cdml generate-coordinates --adapter /absolute/path/libferrum_chem.dylib \
  --molecule-id molecule-1 drawing.cdml --output regenerated.cdml
```

Ferrum targets the exact authored molecule `id`, regenerates its complete ordinary graph, and
publishes the accepted CDML atomically. It preserves the molecule's centroid, mean bond length,
and existing `z` values rather than reproducing particular serialized bytes or pixels.

SMILES inspection is intentionally explicit about its chemistry authority and native library:

```bash
ferrum smiles inspect --adapter /absolute/path/libferrum_chem.dylib CCO
```

The adapter must be an absolute, regular ABI-4 library rather than a symlink. On success, the
command emits `ferrum-smiles-inspection-v1` with canonical SMILES, atoms, bonds, and coordinates.
See [docs/USAGE.md](docs/USAGE.md) for command contracts, stream examples, and failure behavior.

## Native wheel evidence

Ferrum-Chem has a direct-extension, native-wheel proof on macOS arm64. That evidence covers a
minimal clean-environment install, native closure, and LGPL relinking route; it is not yet a
general consumer wheel or a cross-platform desktop release. The Qt package's normal preview
installation still declares its temporary OASA dependency.

## Documentation

- [docs/INSTALL.md](docs/INSTALL.md) explains the verified Rust CLI install, Qt preview setup,
  and current platform limits.
- [docs/USAGE.md](docs/USAGE.md) is the command reference with CDML stream, rewrite, extraction,
  render-observation, coordinate-regeneration, and explicit-adapter chemistry examples.
- [docs/CODE_ARCHITECTURE.md](docs/CODE_ARCHITECTURE.md) describes ownership boundaries and the
  Rust, Python binding, and Qt layers.
- [docs/FILE_STRUCTURE.md](docs/FILE_STRUCTURE.md) maps the workspace, applications, tests, and
  generated evidence.
- [docs/QT_CONTRACT.md](docs/QT_CONTRACT.md) records the Qt presentation and native-route
  contracts.
- [docs/active_plans/ferrum-plan-v3.md](docs/active_plans/ferrum-plan-v3.md) tracks migration
  milestones and acceptance gates.
- [docs/PROVENANCE.md](docs/PROVENANCE.md) records the frontend lineage and licensing boundary.

## License

Ferrum-Qt is [AGPL-3.0-only](LICENSE.AGPL-3.0.md). Ferrum-Chem is
[LGPL-3.0-only](LICENSE.LGPL-3.0.md). RDKit remains a BSD-3-Clause dependency.
