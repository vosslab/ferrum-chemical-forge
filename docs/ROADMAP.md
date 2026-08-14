# Ferrum roadmap

Ferrum is a pre-alpha migration from the historical OASA-backed drawing stack to a
Rust-owned chemistry and CDML document engine. This page gives users and contributors
the direction of travel; [active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md)
is the authoritative milestone tracker and acceptance contract.

## What works today

- The Rust workspace reads, inspects, and structurally rewrites supported CDML without
  Python or reference-repository files at runtime.
- The `ferrum` CLI reports CDML render observations and can inspect SMILES through an
  explicitly supplied native adapter.
- The Ferrum-Qt native route opens, renders, changes an atom element, adds one
  free-standing atom, applies Rust undo/redo, and saves/reopens CDML through the
  Rust document session. Start it with `ferrum-qt --native drawing.cdml`.
- Structural preservation, stable object identity and ordering, typed document
  records, geometry, and render projection have executable evidence in the workspace.

These are contributor-preview capabilities, not a promise that every historical
Ferrum-Qt workflow is Rust-backed or release-ready.

## Current priorities

1. Replace the remaining OASA-backed desktop capabilities one bounded capability at a
   time, beginning with the paths users need to create, edit, and revise a drawing.
2. Expand chemistry and depiction through the single Ferrum-owned RDKit adapter, with
   measured parity rather than approximate reimplementations.
3. Broaden real-document preservation coverage before making stronger compatibility
   claims.
4. Close the application, packaging, platform, and WebAssembly proof gates required
   for a supported product.

## Release bar

Ferrum becomes a supported product only when the Rust session owns the complete
desktop document path, every supported capability in the Ferrum-Qt matrix has a
verified replacement, and the shipped distribution runs without OASA or a separately
installed chemistry runtime. Preservation is structural: existing CDML must retain
its persistent objects, identifiers, order, and supported meaning; byte-for-byte
output is not the compatibility contract.

## Deliberate boundaries

- Ferrum-Qt retains the PySide6 desktop interaction surface while Ferrum-Chem owns
  chemistry and document authority.
- RDKit remains the chemistry authority behind one project-owned native adapter.
- The historical OASA/BKChem source is reference material for isolated comparisons,
  never a production runtime dependency.
- The browser-oriented WebAssembly work is a contract proof, not a second frontend.

## Follow the work

Start with [README.md](../README.md) for the project overview and
[USAGE.md](USAGE.md) for current commands. Consult
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md) for milestone status,
scope, dependencies, evidence, and remaining acceptance gates.
