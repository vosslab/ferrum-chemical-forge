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
- RDKit is designated as a BSD-3-Clause dependency. A macOS arm64 packaging proof
  source-builds its declared profile and bundles it only in an ephemeral test wheel;
  Ferrum does not yet ship a desktop distribution.
- `petgraph` 0.8.3 is an MIT-or-Apache-2.0 Rust source dependency. Ferrum uses a
  private graph and its standard algorithms while owning public ordering, errors,
  identities, and fundamental-cycle selection.

This document records the project's intended licensing boundary and development
provenance. It is not legal advice. The complete applicable GNU license texts are in
this repository; distribution work must also include all required third-party notices.

## Ferrum-Qt lineage

The current `packages/ferrum-chem-qt.app/` tree is the user's own PySide6
frontend carried forward from the local BKChem-Qt reference tree. It is a
frontend continuation, not a claim that the Qt application was rewritten from
scratch. Package metadata and the Python namespace now identify Ferrum-Qt. M1b
is complete for the installed-command rename, application-start, and authored-CDML
open evidence: the offscreen `ferrum-qt` process writes its controlled receipt and
exits without a traceback. That proof does not cover worker-routed non-CDML imports,
Rust-backend adoption, or the remaining migration-preview formats.

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

The boundary keeps Ferrum-Chem separately replaceable so downstream recipients can
relink against a modified LGPL library. The historical macOS arm64 M4a proof first
established that mechanism with a stub wheel. M4b then installed the ABI 2 chemistry
adapter, replaced `libferrum_chem.dylib`, and ran the same native kekulization result
in fresh Rust processes before and after a deliberate distinct-byte `RelWithDebInfo`
replacement for the wheel's `Release` adapter. The proof is recorded in
[active_plans/reports/native_kekulization.md](active_plans/reports/native_kekulization.md).
M20 still verifies the full distribution route on every supported platform.

## Evidence and limits

The architecture and milestone decisions are recorded in
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md). This provenance
record follows its stated scope: preserve the existing frontend's lineage, exclude
OASA and the Tk frontend from production carry-forward, and use RDKit as the chemistry
authority behind a project-owned adapter.

No statement here establishes copyright ownership for external contributors,
changes an upstream license, or replaces a file-by-file redistribution review.

Generated native libraries are build artifacts, not repository sources. The native
staging and Python-package `.libs` directories are ignored; future wheel tooling must
assemble them under an ignored output tree or during packaging, never track host dylibs.

The native build proof uses upstream CMake, LLVM/Clang, and Rustup tooling, with the
Apple SDK and system linker recorded as macOS platform inputs. It builds from
hash-verified sources, uses Boost headers without compiled Boost libraries, and turns
off Python RDKit and SWIG wrappers. Maturin remains unpinned; the receipt records the
actual version used. The successful E2E publishes JSON evidence only, not a wheel or
native binary. Its current profile builds only GraphMol into a Ferrum-owned sealed
stage and uses RDKit, configure-time Catch2, Better Enums, and header-only Boost;
InChI, CoordGen, and MAEParser are excluded. Its scope includes ABI 2 kekulization
semantics and a distinct-byte LGPL relinking proof, but not CDML parity, Qt adoption,
broader chemistry APIs, coordinate parity, cross-platform support, or a desktop
release.

## Test infrastructure provenance

The retained package test configuration required a local `pytest_kill_after`
plugin. Ferrum's `tests/pytest_kill_after.py` was independently implemented for
the same generic, opt-in deadline behavior: timer setup, faulthandler output,
exit status 124, and cleanup. The ignored historical GPL-2.0-only helper was
consulted as behavioral evidence. This is an attribution and transparency note,
not a claim that GPL text or upstream source was transplanted, and not legal
advice.
