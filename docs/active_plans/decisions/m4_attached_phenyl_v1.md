# M4 attached Phenyl V1 decision

## Status

Delivered on 2026-08-26. This is the ninth and final attached compact-group
recipe. Its completed gates are final code/test review, fresh local build,
installed public Qt Attach -> chooser -> materialize evidence, installed binding
contract 8/8, and `all_test.sh` exit 0. This closes the compact-group recipe
milestone only; M4 and the Rust/OASA/BKChem parity objective remain incomplete.

## Binding contract

- The only identity is Rust catalog `Phenyl`, persisted key `phenyl`, display
  name `Phenyl`, and compact label `Ph`; attachment site `0` is the sole site.
- Materialization uses one neutral explicit six-carbon regular-hexagon recipe
  with `attachment_carbon` at `(0.0, 0.0)` as focus and attachment point.
- The directed internal order is `attachment_carbon -> ortho_upper_carbon`
  normal double, then alternating normal single/double around the closed cycle,
  ending `ortho_lower_carbon -> attachment_carbon` normal single.
- The accepted exterior bond retains its durable ID, normal-single `n1` order
  and style, and anchor-side direction; only its compact-group endpoint rewires
  to the materialized attachment carbon.
- The generic catalog, session, materialization, binding, projection, render,
  and Qt route remain the only delivery path.

## Exclusions

No aromatic enum/model/schema branch, aromatic CDML, alternate Kekule form,
key alias, compatibility route, or Phenyl-specific PyO3/Qt logic is added.
Current document-to-chemistry lowering rejects aromatic document bonds, so the
explicit normal single/double recipe is the sole current contract.

## Approved review evidence

Focused catalog/materialization/session semantic tests prove exact atom/bond
maps, neutral valence, focus, directed exterior identity, one-group removal,
Undo/Redo, and reopen. Final independent review passed. The approved
normal-order Kekule graph is proved by role-addressed native document lowering
and target-addressed draw-plan semantics for both exterior directions; it is
not an aromatic-input `kekulize` gate. A fresh build promoted the installed
runtime. The public installed Attach -> chooser -> materialize workflow
completed `succeeded` / `updated`, reported `C8H10`, and retained a usable
scene. The installed binding contract passed 8/8, and `all_test.sh` exited 0
with 7,633 hygiene, 280 binding, and 220 Qt tests passed with one skip.
