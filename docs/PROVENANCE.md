# Provenance and licensing

## Project identity

Ferrum is a two-part CDML chemical drawing platform under active construction.
Ferrum is the PySide6 desktop application. Ferrum-Chem is the Rust document
and chemistry engine authoritative for the current bounded native routes.

Ferrum project copyright notices identify Neil R. Voss, 2026. The canonical license
files contain unmodified GNU license terms rather than project-specific notices.

The two components have deliberately different licenses:

- Ferrum is AGPL-3.0-only. Its repository notice is `LICENSE.AGPL-3.0`,
  and its distributable package notice is `packages/ferrum-chem-qt.app/LICENSE`.
- Ferrum-Chem is LGPL-3.0-only. Its repository notice is `LICENSE.LGPL-3.0`.
- RDKit is a BSD-3-Clause native dependency used only through Ferrum's private
  local-runtime adapter. Ferrum does not ship a desktop distribution.
- `petgraph` 0.8.3 is an MIT-or-Apache-2.0 Rust source dependency. Ferrum uses a
  private graph and its standard algorithms while owning public ordering, errors,
  identities, and fundamental-cycle selection.
- Ferrum's `ferrum-geometry::straighten_depiction` is an arithmetic-only derived
  implementation of RDKit's `straightenDepiction` algorithm, consulted from
  `Code/GraphMol/Depictor/RDDepictor.cpp` at revision
  `d1f7d6a59d712ddaf732b60173fd6223b3cd5003`. RDKit is BSD-3-Clause licensed.
  Ferrum does not copy or link the RDKit implementation: it independently expresses
  the algorithm over Ferrum's `Point2` and records differential evidence against a
  separately launched local RDKit process. This attribution records the source,
  purpose, and license context; it is not legal advice.
- Ferrum's molecule-label face is the byte-verified Telex Regular font copied from
  RDKit `Release_2026_03_4/Data/Fonts/Telex-Regular.ttf`: 38,940 bytes and SHA-256
  `eeaa2d17d105b6b46e5368ecd990f5b19c50131ff922dbf79bfb9bb45c249871`.
  Ferrum distributes it at `crates/render/assets/fonts/Telex-Regular.ttf` with the
  upstream SIL Open Font License 1.1 notice at
  `crates/render/assets/licenses/Telex-OFL-1.1.txt`. The observed FreeType metadata
  is family `Telex` and PostScript name `Telex-Regular`. This is an exact-face
  resource for Ferrum rendering, not a system-family lookup.
- M13's in-memory PNG and PDF sinks use locked pure-Rust source dependencies:
  [`tiny-skia` 0.12.0](https://crates.io/crates/tiny-skia/0.12.0) and its
  [`tiny-skia-path` 0.12.0](https://crates.io/crates/tiny-skia-path/0.12.0)
  dependency are BSD-3-Clause; [`png` 0.18.1](https://crates.io/crates/png/0.18.1)
  and [`pdf-writer` 0.15.0](https://crates.io/crates/pdf-writer/0.15.0) are MIT OR
  Apache-2.0. Their locked sources are respectively
  [linebender/tiny-skia](https://github.com/linebender/tiny-skia),
  [image-rs/image-png](https://github.com/image-rs/image-png), and
  [typst/pdf-writer](https://github.com/typst/pdf-writer). These packages have no
  build scripts or native-graphics linkage in Ferrum's locked M13 sink surface.
  They are an in-process Rust boundary, not a packaged native graphics closure.
  `tiny-skia` and `tiny-skia-path` use Rust `unsafe` internally; Ferrum does not
  claim that their use makes the renderer implementation entirely safe Rust.

This document records the project's intended licensing boundary and development
provenance. It is not legal advice. The complete applicable GNU license texts are in
this repository; distribution work must also include all required third-party notices.

## Ferrum lineage

The current `packages/ferrum-chem-qt.app/` tree is the user's own PySide6
frontend carried forward from the local BKChem-Qt reference tree. It is a
frontend continuation, not a claim that the Qt application was rewritten from
scratch. Package metadata and the Python namespace now identify Ferrum. M1b
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

## CDML reference contract

Ferrum adopts the upstream historical
[CDML_BACKEND_TO_FRONTEND_CONTRACT.md](CDML_BACKEND_TO_FRONTEND_CONTRACT.md) and
[CDML_FORMAT_SPEC.md](CDML_FORMAT_SPEC.md) as explicit reference documents. They
were copied from `vosslab/bkchem-oasa` commit
`f3a6b2ffb354c63a5d87d2f76c12b43a07bac36c` (repository HEAD
`f8fd0e6fbd67d40e48c4d6e38116524e85a6d8ed`). The original source SHA-256 values
are `7cd02af29bff5ce4f004e25fa0c9884efc636c23e46417a24525cf3ee75ca097` for the
contract and `defa534555fcfc20d223ef8341c66f8c1d6ff3fad4f6aa45f7f85212c071fbdb`
for the specification.

The copied text retains historical OASA/BKChem names. For Ferrum planning, they
map respectively to the intended Ferrum-Chem backend and Ferrum frontend.
These documents are compatibility and security reference boundaries, not a claim
that every described operation is implemented. Any local divergence requires a
deliberate reconciliation with the named upstream source rather than silent drift.

The boundary keeps Ferrum-Chem replaceable at the source and Rust-library
boundary. `build.sh` creates the adapter and extension only under the checkout's
`build/` directory; it has no publication or installation route.

## Evidence and limits

The architecture and milestone decisions are recorded in
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md). This provenance
record follows its stated scope: preserve the existing frontend's lineage, exclude
OASA and the Tk frontend from production carry-forward, and use RDKit as the chemistry
authority behind a project-owned adapter.

No statement here establishes copyright ownership for external contributors,
changes an upstream license, or replaces a file-by-file redistribution review.

Generated native libraries are build artifacts, not repository sources. Local
runtime staging stays below `build/`; host dylibs are never tracked.

The local native build uses upstream CMake, LLVM/Clang, and Rustup tooling with
the Apple SDK and system linker as macOS inputs. It builds the closed native
adapter under `build/`; `all_test.sh` validates the resulting local runtime.

## Haworth projection terminology

The first `ferrum-domain::haworth` slice was independently designed from the
following IUPAC nomenclature references, retrieved 2026-08-12:

- IUPAC, *Nomenclature of Carbohydrates: Recommendations 1996*, Pure and
  Applied Chemistry 68(10), 1919-2008, PDF at
  <https://publications.iupac.org/pac/1996/pdf/6810x1919.pdf>.
- IUPAC carbohydrate nomenclature web guidance, sections 2-Carb-5,
  <https://iupac.qmul.ac.uk/2carb/05.html>, and 2-Carb-6/7,
  <https://iupac.qmul.ac.uk/2carb/06n07.html>.

Those public nomenclature sources establish the terminology used by the module:
five-membered furanose and six-membered pyranose Haworth representations, and
the distinct, explicitly supplied alpha/beta and D/L semantic facts. Ferrum's
new Rust module does not reproduce source code, coordinate tables, tests, or
rendering decisions from OASA. It accepts a caller-selected C/O cycle and emits
an independently designed deterministic projection plan; it does not reconstruct
stereochemistry from a drawing or legacy display facts.

## Test infrastructure provenance

The retained package test configuration required a local `pytest_kill_after`
plugin. Ferrum's `tests/pytest_kill_after.py` was independently implemented for
the same generic, opt-in deadline behavior: timer setup, faulthandler output,
exit status 124, and cleanup. The ignored historical GPL-2.0-only helper was
consulted as behavioral evidence. This is an attribution and transparency note,
not a claim that GPL text or upstream source was transplanted, and not legal
advice.
