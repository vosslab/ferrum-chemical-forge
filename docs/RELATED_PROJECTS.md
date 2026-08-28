# Related projects

## Confirmed related projects

### BKChem and OASA reference tree

- Relationship: upstream source or fork
- Link: https://github.com/vosslab/bkchem-oasa
- Evidence: [PROVENANCE.md](PROVENANCE.md) identifies the retained Qt frontend as a
  continuation of the local BKChem-Qt reference tree and names OASA as the external
  migration oracle.
- Notes: This read-only reference establishes frontend lineage and behavior evidence;
  Ferrum-Chem is a new Rust backend rather than an OASA code carry-forward.

### RDKit

- Relationship: direct dependency
- Link: https://github.com/rdkit/rdkit
- Evidence: [PROVENANCE.md](PROVENANCE.md) identifies RDKit as Ferrum's BSD-3-Clause
  chemistry authority behind the project-owned adapter.
- Notes: RDKit remains behind Ferrum-Chem's private adapter boundary.

### Qt for Python (PySide6)

- Relationship: direct dependency
- Link: https://doc.qt.io/qtforpython-6/
- Evidence: `packages/ferrum-chem-qt.app/pyproject.toml` declares `PySide6` and
  `shiboken6` for the Ferrum desktop application.
- Notes: Ferrum uses the Qt Widgets desktop path through the official Python
  bindings.

### PyO3 and Maturin

- Relationship: binding and package toolchain
- Link: https://github.com/PyO3/pyo3
- Evidence: `packages/ferrum-rust/crates/api-python/Cargo.toml` owns the PyO3
  `cdylib`, while `crates/api/python/pyproject.toml` selects that dedicated
  extension crate for the Maturin wheel.
- Notes: The extension is a narrow Python client of Rust-owned operations; Qt
  feature modules reach it through `ferrum_qt/ferrum/engine.py`.

### Kurbo and nalgebra

- Relationship: direct geometry dependencies
- Link: https://github.com/linebender/kurbo
- Evidence: `packages/ferrum-rust/crates/geometry/Cargo.toml` declares both
  crates for the Rust geometry layer.
- Notes: Ferrum keeps authoritative geometry in Rust while Qt projects the
  resulting document and render facts.

## Evidence notes

This map is grounded in the Ferrum package manifests and
[PROVENANCE.md](PROVENANCE.md), then checked against the primary BKChem/OASA reference
remote, the RDKit project repository, and the official Qt for Python documentation.
