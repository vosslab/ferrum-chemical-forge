# Provenance and licensing

## Project identity

Ferrum is a two-part CDML chemical drawing platform under active construction.
Ferrum-Qt is the PySide6 desktop application. Ferrum-Chem is the planned Rust
document and chemistry engine that will become the authoritative backend.

Ferrum project copyright notices identify Neil R. Voss, 2026. The canonical license
files contain unmodified GNU license terms rather than project-specific notices.

The two components have deliberately different licenses:

- Ferrum-Qt is AGPL-3.0-only. Its repository notice is `LICENSE.AGPL-3.0.md`,
  and its distributable package notice is `packages/ferrum-chem-qt.app/LICENSE`.
- Ferrum-Chem is LGPL-3.0-only. Its repository notice is `LICENSE.LGPL-3.0.md`.
- RDKit is designated as a BSD-3-Clause dependency. It is not yet bundled or
  linked by this pre-alpha repository.

This document records the project's intended licensing boundary and development
provenance. It is not legal advice. The complete applicable GNU license texts are in
this repository; distribution work must also include all required third-party notices.

## Ferrum-Qt lineage

The current `packages/ferrum-chem-qt.app/` tree is the user's own PySide6
frontend carried forward from the local BKChem-Qt reference tree. It is a
frontend continuation, not a claim that the Qt application was rewritten from
scratch. Package metadata now identifies Ferrum-Qt, while the Python namespace
rename remains M1b work.

The local reference document
`OTHER_REPOS/bkchem-oasa/docs/GPL_FILE_PURPOSES.md` is the historical licensing
evidence consulted for this migration. It records that, at its stated scan date,
the historical tree had no pure GPLv2 files remaining and identifies mixed
GPLv2/LGPLv3 areas. It is gitignored reference material, not distributed as part
of this repository and not a blanket legal conclusion about every historical file.

## Ferrum-Chem boundary

Ferrum-Chem is a new Rust backend. It replaces the OASA backend rather than
copying it into this repository. During migration, OASA can remain an external
oracle for behavior comparisons; it is not Ferrum-Chem production code.

The planned boundary keeps Ferrum-Chem separately replaceable so downstream
recipients can relink against a modified LGPL library. The build and packaging
mechanism is not implemented yet; M4a proves it and M20 verifies it on each
supported platform.

## Evidence and limits

The architecture and milestone decisions are recorded in
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md). This provenance
record follows its stated scope: preserve the existing frontend's lineage, exclude
OASA and the Tk frontend from production carry-forward, and use RDKit as the chemistry
authority behind a project-owned adapter.

No statement here establishes copyright ownership for external contributors,
changes an upstream license, or replaces a file-by-file redistribution review.

## Test infrastructure provenance

The retained package test configuration required a local `pytest_kill_after`
plugin. Ferrum's `tests/pytest_kill_after.py` was independently implemented for
the same generic, opt-in deadline behavior: timer setup, faulthandler output,
exit status 124, and cleanup. The ignored historical GPL-2.0-only helper was
consulted as behavioral evidence. This is an attribution and transparency note,
not a claim that GPL text or upstream source was transplanted, and not legal
advice.
