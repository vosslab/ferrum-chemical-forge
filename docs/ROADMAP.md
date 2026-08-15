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
- M15 is complete: supported native utility workflows are bounded peptide-template
  insertion, selected-path linear-form conversion, and clean/hex-grid/terminal-bond/
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
2. M19 is independently accepted and complete. M20 source
   implementation is accepted for a proposed macOS arm64/CPython 3.12 target: it has a
   two-wheel, offline/scrubbed builder and retained clean-install/relink E2E route. M22 source
   work now supplies the dual-license source-release check, wheel-local notice mechanism, and
   predicate artifact inventory. Real target evidence remains pending until separately
   provisioned local Cargo and Qt wheelhouses are available. The Cargo-installed Rust CLI remains
   separate from both Python wheels.
3. M20 and M22 remain open. Release only after the actual target install/relink, source-archive
   CLI, artifact-inventory, and human legal/release review make the supported boundary coherent.

M21 is an optional browser/WASM contract proof. It does not block the desktop path or
change the M17, M18, M19, M20, and M22 sequence above.

## Release bar

Ferrum becomes a supported product when the native session owns every capability
classified as supported, the shipped distribution runs without an OASA, Tk, or Python
RDKit runtime dependency, and the supported platform matrix has real installation and
relinking evidence. CDML preservation is structural: persistent objects, identifiers,
order, and supported meaning must remain intact. Byte-for-byte files, pixel-equivalent
renders, arbitrary test counts, and unmeasured timing limits are not release criteria.

The proposed M20 target is macOS arm64 with CPython 3.12 only. Its two first-party Python
artifacts are Ferrum-Chem and Ferrum-Qt; Qt build and runtime dependencies stay in explicit local
wheelhouses, and the Rust CLI remains a Cargo-installed command. Source-accepted M20/M22 work is
not a release claim: both milestones remain open until a real no-index clean install,
installed-resource and LGPL relink observation, source-archive CLI run, classified artifact
inventory, and human legal/release review succeed on that target. Other platforms require their
own evidence.

## Deliberate boundaries

- Ferrum-Qt provides the PySide6 interaction surface; Ferrum-Chem owns chemistry and
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
