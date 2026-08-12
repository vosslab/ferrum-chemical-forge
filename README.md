# Ferrum Chemical Forge

A pre-alpha CDML chemical drawing platform for scientists and educators that combines
a retained PySide6 desktop editor with a Rust document and chemistry backend under
active migration.

> Status: pre-alpha. The Rust backend can inspect and structurally rewrite CDML from
> the shell, and the Qt package now uses the `ferrum_qt` namespace. The
> frontend-to-backend integration is not complete, and the present Qt frontend still
> has migration-only OASA dependencies.

<!-- screenshots:begin (managed by screenshot-docs) -->
<!-- screenshots:end -->

## One drawing, durable document

Ferrum aims to make chemical drawing a dependable scientific workflow: a visible Qt
editor for people and an authoritative, testable document engine for the chemistry.
The first end-to-end target is deliberately small: open one CDML molecule, generate
coordinates, render Ferrum-produced operations on the Qt canvas, and save without
losing persistent objects.

- Ferrum-Qt preserves the interactive PySide6 drawing workflow already familiar to
  BKChem-Qt users.
- Ferrum-Chem will own CDML documents in Rust instead of carrying OASA forward as
  production backend code.
- RDKit will remain the chemistry authority behind one project-owned adapter rather
  than being reimplemented approximately.
- Preservation, not byte-for-byte output matching, is the compatibility goal for
  documents users have already created.

## Current status

The Rust workspace now owns validated molecule records, deterministic graph analysis,
structural CDML storage, persistent identity/order, and a typed CDML-to-core projection.
Its corpus comparison has no unexpected differences, and permanent Rust builds and
tests do not use `OTHER_REPOS/`. The `ferrum` executable provides a self-contained
Rust path for CDML inspection and structural rewrite. Ferrum-Qt now uses the
`ferrum_qt` namespace, while its live document path still depends on migration-only
OASA behavior. The desktop application therefore remains a contributor preview, not
a completed Ferrum release.

## Command-line tools

The Rust backend installs the `ferrum` executable. It reads CDML without Python or
`OTHER_REPOS/`; `docs/INSTALL.md` and `docs/USAGE.md` contain the verified build and
command examples.

## First success target

The first supported success will be the thin workflow: open a simple CDML document,
obtain coordinates through Ferrum-Chem, see the result in Ferrum-Qt, and save a
structurally preserved document. Until that workflow lands, contributors can use the
milestone plan as the implementation contract rather than relying on the legacy
frontend as a completed Ferrum release.

## Project record

[docs/active_plans/ferrum-plan-v3.md](docs/active_plans/ferrum-plan-v3.md)
describes the architecture, ownership boundaries, milestones, and acceptance gates for
the migration. [docs/PROVENANCE.md](docs/PROVENANCE.md) records the frontend lineage
and the intended licensing boundary.

## License

Ferrum-Qt is AGPL-3.0-only, and Ferrum-Chem is LGPL-3.0-only. RDKit remains a
BSD-3-Clause dependency. The repository notices are `LICENSE.AGPL-3.0.md` and
`LICENSE.LGPL-3.0.md`; see the GNU project's official
[AGPL text](https://www.gnu.org/licenses/agpl-3.0.html) and
[LGPL text](https://www.gnu.org/licenses/lgpl-3.0.html) for the complete terms.
