# Ferrum Chemical Forge

A pre-alpha CDML chemical drawing platform for scientists and educators that combines a retained PySide6 desktop editor with a planned Rust chemistry backend for durable document ownership.

> Status: pre-alpha. The Rust backend, the Ferrum-Qt namespace rename, and the
> frontend-to-backend integration are not complete. The present Qt frontend still
> depends on OASA, so this checkout has no supported newcomer install-and-run path.

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

This repository is at migration milestone M1. The checked-in Qt package is a retained
frontend scaffold; its package metadata identifies Ferrum-Qt, but its Python namespace
still uses the pre-rename name. The seven-crate Rust skeleton is populated and builds,
but it has no functional backend yet. These facts make any installation guide
premature, so the project does not present an unverified command as a quick start.

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
