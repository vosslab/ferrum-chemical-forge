# M4 attached AcylChloride V1 decision

## Decision

AcylChloride is delivered as the eighth bounded M4 attached compact-group
recipe. The approval review is `PASS`; the prior architecture gate is
`ferrum-post-cyano-recipe-architecture-gate-20260826.md`.

## Delivered contract

- The existing catalog key `acyl_chloride`, enum `AcylChloride`, and derived
  display and short label `COCl` are the sole identity.
- The recipe is neutral attached `R-C(=O)-Cl`, with `attachment_carbon` at
  `(0.0, 0.0)`, `carbonyl_oxygen` at `(24.0, 18.0)`, and `chlorine` at
  `(24.0, -18.0)`.
- The existing immutable recipe model carries normal `Double` C=O and normal
  `Single` C-Cl bonds. Materialization returns `attachment_carbon` as focus.
- The shared attached materializer retains the accepted exterior normal-single
  bond's durable identity, order, and presentation while rewiring it to the
  attachment carbon.
- Existing anchor-relative rotation and attached pose remain unchanged.

## Delivery evidence and exclusions

- Rust catalog tests prove the exact `acyl_chloride` / `AcylChloride` / `COCl`
  identity, neutral C/O/Cl atoms, coordinates, attachment-carbon focus, and
  normal Double C=O plus Single C-Cl topology. Session semantics prove focused
  materialization and directed exterior identity preservation: the retained
  bond keeps its durable ID, order, style, and original anchor-side endpoint,
  while only its compact-group side is rewired to the returned carbon focus.
- A fresh `build.sh` promotion produced the current CLI, Qt launcher, and
  installed Python runtime. The one-time installed Qt workflow publicly used
  Attach, selected the accessible `acyl_chloride` / `COCl` chooser row,
  materialized it to `succeeded` / `updated`, and kept Molecule Report usable.
  The public report proves C/O/Cl composition (`C3H5ClO`) only; it does not
  prove exact topology, carbon focus, bond orders, or exterior identity.
- The approval review passed, and `all_test.sh` completed with 7,633 hygiene
  tests, 280 installed binding tests, and 220 Qt tests with one skip.
- No PyO3, Qt, renderer, projection, protocol, CLI, alias, compatibility, or
  chemistry-specific path is added. Free placement and alternate charge or
  hydrolysis behavior remain outside this decision.
