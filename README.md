# Ferrum Chemical Forge

A pre-alpha chemical drawing platform for scientists and educators, pairing a Rust CDML
engine with one Rust-native PySide6 interface.

> Status: pre-alpha. Ordinary `ferrum-qt` is an accepted macOS arm64 bounded editor: it
> opens, renders, changes an atom element, adds one free-standing atom,
> edits all nine supported authored atom properties in one Rust-owned operation,
> connects two existing atoms with a revision-bound Rust-owned single-bond drag, extends an
> atom into empty space with one carbon and bond, moves an atom by dragging its rendered
> position, deletes durable atoms or bonds with their correct Rust-owned topology semantics,
> changes one selected normal bond among single, double, and triple order, imports bounded
> SMILES, regenerates one existing molecule's 2D coordinates while retaining its current
> centroid and mean bond length, applies Rust undo/redo, and saves/reopens through Ferrum-Chem
> without a compatibility host. It opens uncompressed `.cdml` and decoded CD-SVG `.svg`
> payloads, and refuses unsupported or compressed formats without changing the active document.
> It does not yet provide general bond-style tools or a complete native editing workflow.

<!-- screenshots:begin (managed by screenshot-docs) -->
<!-- screenshots:end -->

## One drawing, durable document

Chemical drawings should remain useful scientific records, not fragile pictures tied to one
GUI session. Ferrum is for scientists, educators, and contributors who need a visible drawing
surface plus a testable CDML document engine with explicit preservation boundaries.

- Run one versioned, stateless JSON document operation through the `ferrum protocol`
  command family.
- Open and make bounded atom edits in the ordinary native-first Qt route with `ferrum-qt`.
- Keep unknown XML and persistent document identity through Ferrum's structural CDML path.
- Keep chemistry adapters behind Ferrum-owned desktop workflows rather than a public CLI
  adapter-discovery contract.

## Native-first desktop route

Ordinary `ferrum-qt drawing.cdml` starts the Rust-owned Ferrum-Chem route. On the verified
macOS arm64 configuration, it opens, renders, changes an atom element, adds one free-standing
atom, edits all nine supported authored atom properties as one Rust-owned revision, connects two
existing atoms with a single-bond drag, extends an atom into empty space with one carbon and
bond, moves an atom with exact Rust-owned coordinates, deletes a durable atom with its typed
incident bonds or deletes one durable bond while preserving its endpoint atoms, changes one
selected normal bond among single, double, and triple order, imports bounded SMILES, regenerates
one durable molecule's coordinates off the Qt thread while preserving its current placement,
applies Rust undo/redo, and saves/reopens CDML through Ferrum-Chem. Use it for the completed
bounded native document slice.

The ordinary route does not yet choose a non-carbon element or bond order during the bond gesture,
edit wedge, hashed, dashed, aromatic, or ring-side depiction styles, delete other object classes,
regenerate styled or non-ordinary molecule graphs, import arbitrary chemistry, or make a
cross-platform packaging claim. The ordinary command has one native document window and no
fallback editor; unsupported workflows need their own Rust-owned contract before they are added.

## Quick start: inspect a real document

Rust 1.97.1 or newer is required. From a source checkout, run the Rust CLI against the
checked-in authored CDML corpus:

```bash
cd packages/ferrum-rust
cargo run --locked --quiet --bin ferrum -- protocol schema
```

This prints the checked-in request/response schema. To run an operation, place one request JSON
object in a file and use `ferrum protocol run request.json`. For the request shape, response
envelopes, and a complete example, see [docs/USAGE.md](docs/USAGE.md). For a durable
installation and the full platform requirements, see [docs/INSTALL.md](docs/INSTALL.md).

## Command-line protocol

The shipping Rust CLI exposes only `ferrum protocol schema` and `ferrum protocol run INPUT
[--output OUTPUT]`. Its four closed operations inspect, validate, structurally rewrite, or render
one admitted CDML document. It has no batch, network, session, Qt, adapter-discovery, or
path-bearing protocol payload. A successful rewrite preserves document structure, not serialized
bytes; rendered artifacts are complete results, not pixel-equivalence claims. See
[docs/USAGE.md](docs/USAGE.md) for the request contract, safe output publication, and failures.

## Package-release status

M20 and M22 source work is accepted for a proposed initial target: macOS arm64 with CPython
3.12. Its maintainer route builds two first-party Python wheels, `ferrum-chem` and `ferrum-qt`,
with separate, explicit local Cargo, Qt build-backend, and Qt runtime wheelhouses. The Rust
`ferrum` command remains a separate Cargo-installed tool; neither Python wheel supplies it.
The real offline two-wheel install, installed-resource, LGPL relink, source-archive CLI, and
final artifact-inventory observations remain pending because target-matching external inputs are
not presently available. Human legal and release review also remains required. This is not yet a
supported consumer release or a cross-platform claim. Historical source material and oracle
inputs remain isolated provenance evidence, not product runtime dependencies or a desktop
fallback.

## Documentation

- [docs/INSTALL.md](docs/INSTALL.md) explains the verified Rust CLI install, Qt preview setup,
  and current platform limits.
- [docs/USAGE.md](docs/USAGE.md) is the command reference for the frozen JSON operation protocol
  and the separate bounded Ferrum-Qt workflow.
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
[LGPL-3.0-only](LICENSE.LGPL-3.0.md). The source-accepted native wheel route prepares its
Ferrum, RDKit, InChI, and Telex notices as a standard wheel-local bundle; final contents require
the pending artifact inventory and human legal review. See [docs/PROVENANCE.md](docs/PROVENANCE.md).
