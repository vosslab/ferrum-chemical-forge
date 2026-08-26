# M4 attached Cyano v1

## Decision

Cyano is delivered as the seventh bounded M4 attached compact-group recipe.
The authoritative architecture gate is the plain path
`ferrum-post-carboxyl-recipe-architecture-gate-20260826.md`.

## Approved contract

- Use the existing catalog key `cyano`, enum `Cyano`, and derived label `CN`.
- Support attached-only neutral nitrile `R-C#N`.
- Use neutral `attachment_carbon` at `(0, 0)` and neutral `terminal_nitrogen`
  at `(24, 0)`, with implicit hydrogens and no formal charges.
- Add one internal `Triple`, `Normal` bond from `attachment_carbon` to
  `terminal_nitrogen`.
- Encode the bond through the ordinary typed-CDML writer as normal triple `n3`.
- Return the materialized `attachment_carbon` as focus.
- Preserve the existing ordinary normal-single exterior bond's durable identity,
  order, and presentation, and rewire that bond to `attachment_carbon`.
- Use the existing anchor-relative recipe rotation and pose. No Cyano-specific
  geometry rule is approved.

## Ownership boundary

- Add `Triple` only to canonical `CompactGroupRecipeBondOrderV1`.
- Do not add a wire type or a Cyano branch in PyO3, Qt, session, or rendering.
- Reuse the shared attached lifecycle and typed refusal behavior.

## Delivered evidence

- Rust semantic tests and independent review prove neutral `R-C#N`, carbon
  focus, the ordinary normal-triple `n3` recipe bond, and retained exterior
  normal-single identity, order, and presentation after generic rewiring.
- Binding, session, and Qt transport are generic; no chemistry-specific Qt
  branch was added. Shared lifecycle evidence remains the reuse boundary for
  Undo, Redo, and reopen.
- A current build succeeded, and the installed public Qt Attach -> chooser ->
  materialize workflow passed. Molecule Report showed `C3H5N`; exact topology
  remains Rust semantic-test evidence rather than a Qt report claim.
- The final full validation gates remain repository-level evidence work. No
  key/count/coordinate snapshots, fixtures, or Cyano-specific GUI end-to-end
  test were added.

## Exclusions

- `AcylChloride` and `Phenyl` remain separate undelivered choices.
- Aromatic recipes, free placement, aliases, and batch admission are outside
  this decision.
