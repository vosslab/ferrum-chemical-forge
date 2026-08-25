# Ferrum roadmap

Ferrum is a pre-alpha chemistry-drawing application with a Rust-owned chemistry and
CDML document engine. This page gives users and contributors the direction of travel;
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md) remains the
authoritative milestone tracker and acceptance contract.

## What works today

- Ordinary `ferrum-qt` starts one Rust-native desktop application. It opens supported
  local CDML, applies bounded native edits with Undo and Redo, saves and reopens CDML,
  and exports native SVG, PDF, and PNG artifacts.
- The Rust workspace owns the supported CDML document path, render observations, and
  bounded chemistry-adapter and artifact-publication routes.
- The explicit OASA compatibility host and its production dependency declarations are
  retired. Historical OASA and BKChem material is isolated provenance or oracle input,
  not a desktop runtime.
- M15 is complete: supported native utility workflows are bounded peptide sequence
  insertion through `prepare_ferrum_peptide_insertion_v1`, selected-path linear-form conversion, and clean/hex-grid/terminal-bond/
  bond-length/bond-angle/ring geometry repair. Compact sugar code and names, known-group
  expansion, biomolecule/system catalogs, substructure search, oxidation, generated
  names, and broader chemistry checks are pre-production drops.
- M16 is complete following independent close review. One ordinary Rust-native window
  owns supported document routes; CDXML, CML, `.cdsvg`, `.svgz`, and compressed CDML
  refuse before read, while retired host-only families remain explicit drops.

These are pre-alpha capabilities. Supported document classes and operations are the
ones stated in the active plan and capability matrix; unsupported historical workflows
receive a clear refusal or a recorded pre-production drop.

## Current priorities

1. M17/M18 are complete: they provide the small stateless operation protocol,
   generated schema, Python boundary, and the only shipping `ferrum` CLI family:
   `protocol schema` and `protocol run`.
2. M19 is independently accepted and complete. Ferrum's supported developer
   path remains the repository-local Rust CLI and Qt application built by
   `build.sh`.

M21 is an optional browser/WASM contract proof. It does not block the desktop path or
change the M17, M18, and M19 sequence above.

## Deliberate boundaries

- Ferrum provides the PySide6 interaction surface; Ferrum-Chem owns chemistry and
  document authority.
- RDKit remains behind one project-owned native adapter.
- Historical OASA/BKChem sources inform provenance and isolated comparisons only.
- Unsupported legacy features are not compatibility obligations unless a bounded
  Ferrum contract adopts them.
- The versioned protocol has four document operations only and excludes batch,
  network, session, Qt, path, and chemistry-adapter surfaces until a named
  workflow justifies a later contract.
- Browser-oriented WebAssembly work validates a shared contract; it is not a second
  frontend.

## Follow the work

Start with [README.md](../README.md) for the project overview and
[USAGE.md](USAGE.md) for current commands. Consult
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md) for milestone status,
scope, dependencies, evidence, and acceptance gates.
