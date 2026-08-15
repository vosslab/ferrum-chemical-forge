# Provenance and licensing

## Project identity

Ferrum is a two-part CDML chemical drawing platform under active construction.
Ferrum-Qt is the PySide6 desktop application. Ferrum-Chem is the Rust document
and chemistry engine authoritative for the current bounded native routes.

Ferrum project copyright notices identify Neil R. Voss, 2026. The canonical license
files contain unmodified GNU license terms rather than project-specific notices.

The two components have deliberately different licenses:

- Ferrum-Qt is AGPL-3.0-only. Its repository notice is `LICENSE.AGPL-3.0.md`,
  and its distributable package notice is `packages/ferrum-chem-qt.app/LICENSE`.
- Ferrum-Chem is LGPL-3.0-only. Its repository notice is `LICENSE.LGPL-3.0.md`.
- RDKit is a BSD-3-Clause native dependency. M20 and M22 have source-accepted mechanisms for a
  proposed macOS arm64/CPython 3.12 two-wheel route. The required target clean-install, relink,
  source-archive CLI, classified-artifact, and human legal/release evidence remains pending.
  Ferrum does not yet ship a supported desktop distribution.
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

## Native distribution notices

The source-accepted native-wheel packager prepares a wheel-local notice bundle in the standard
`ferrum_chem-*.dist-info/licenses/` directory and names those files with PEP 639 `License-File`
metadata. Its roles are the Ferrum-Chem LGPL-3.0 text, the RDKit BSD-3-Clause text, the InChI MIT
text, the Telex OFL 1.1 text, and the reviewed `THIRD_PARTY_NOTICES.md` index in the native wheel
metadata source. The final release inventory checks these semantic roles rather than a wheel member
count or byte identity.

The InChI MIT text is not guessed from a generic archive license. The packager extracts the
complete leading license and attribution comment from the hash-verified pinned InChI 1.07.3
archive member `INCHI-1-SRC/INCHI_API/libinchi/src/inchi_dll.c`, the source route used by the
declared native closure. If that source path, selected closure, or pinned source changes, the
notice index and human review must change with it.

This mechanism is source-accepted only. Before publication, M20 must produce its actual receipt,
M22 must classify the final wheels and committed source archive, and a human must review the final
notice inventory and publication decision.

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
map respectively to the intended Ferrum-Chem backend and Ferrum-Qt frontend.
These documents are compatibility and security reference boundaries, not a claim
that every described operation is implemented. Any local divergence requires a
deliberate reconciliation with the named upstream source rather than silent drift.

The boundary keeps Ferrum-Chem separately replaceable so downstream recipients can
relink against a modified LGPL library. The historical macOS arm64 M4a proof first
established that mechanism with a stub wheel. M4b then installed the ABI 2 chemistry
adapter, replaced `libferrum_chem.dylib`, and ran the same native kekulization result
in fresh Rust processes before and after a deliberate distinct-byte `RelWithDebInfo`
replacement for the wheel's `Release` adapter. The proof is recorded in
[active_plans/reports/native_kekulization.md](active_plans/reports/native_kekulization.md).
M20 source implementation now defines one proposed macOS arm64/CPython 3.12 route: it produces
Ferrum-Chem and Ferrum-Qt wheels from explicit local Cargo, Qt build-backend, and runtime
wheelhouses, then uses a scrubbed no-index clean install and post-relink observation. The external
wheelhouses are unavailable, so its runtime receipt is pending and no platform is yet supported.

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
actual version used. The current profile builds a narrow GraphMol/FileParsers closure
into a Ferrum-owned sealed stage and uses RDKit, configure-time Catch2, Better Enums,
and header-only Boost; InChI, CoordGen, and MAEParser are excluded. Each artifact
records an exact official RDKit tag and archive digest, while new builds advance to
the latest stable release and compare semantics with the previous stable release.
The historical direct-wheel E2E proves the native load/relink mechanism on macOS arm64. The M20
two-wheel target proof remains pending; neither result establishes cross-platform support or a
finished desktop release.

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
