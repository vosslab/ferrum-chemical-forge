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
  `shiboken6` for the Ferrum-Qt desktop application.
- Notes: Ferrum-Qt uses the Qt Widgets desktop path through the official Python
  bindings.

## Evidence notes

This map is grounded in the Ferrum package manifests and
[PROVENANCE.md](PROVENANCE.md), then checked against the primary BKChem/OASA reference
remote, the RDKit project repository, and the official Qt for Python documentation.
