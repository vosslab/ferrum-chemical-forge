# Release history

## Unreleased pre-alpha work

- Ferrum remains pre-production. The repository builds and validates one local
  CLI and PySide6 application under `build/`; it has no release or installation
  workflow.

## v26.08 - 2026-08-12

### Highlights

- Added the standalone Rust `ferrum` command for typed CDML inspection and structural
  rewriting, with explicit standard-input and standard-output behavior.
- Added typed CDML storage that preserves assigned CDML records, unknown attributes,
  ordered opaque children, and document identity and ordering facts.
- Renamed the retained Qt frontend product and installed command to Ferrum and
  `ferrum-qt` while keeping the existing application contracts explicit.
- Added graph analysis for connected components, bridges, articulation points, matchings,
  shortest paths, distances, diameter, and a deterministic fundamental-cycle basis.

### Notable fixes

- Tightened the native build policy and provenance checks so declared RDKit inputs,
  CMake settings, compiler tools, downloaded archives, and packaged Mach-O closure are
  verified rather than inherited from the host environment.
- Corrected CDML bond-token interpretation, document identity collisions, and typed
  opaque-container handling so valid authored documents retain their structure.
- Made Qt launch-file processing and controlled shutdown fail visibly instead of
  publishing a successful smoke receipt after an incomplete open.

### Compatibility notes

- Ferrum is the user-facing product name and `ferrum-qt` is the installed command.
  Existing settings, templates, clipboard ownership value, and session identifiers
  remain compatibility identifiers during the migration.
- The native chemistry boundary remains intentionally narrow: macOS arm64 and
  GraphMol kekulization. It is not a claim of complete Qt, CDML, coordinate,
  cross-platform, or broad RDKit API replacement.

### Validation

- The Rust workspace passed formatting, target checking, warnings-denied Clippy, and
  unit and integration tests on `aarch64-apple-darwin`.
- The local runtime validation recorded extension and dynamic-library closure
  checks together with CLI and offscreen Qt smoke checks.
