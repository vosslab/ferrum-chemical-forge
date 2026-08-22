# TODO

Ferrum has one Rust-owned document/runtime route and no live OASA Python
backend. The following gates remain before the migration and supported-product
work are complete.

## Desktop convergence

- Finish portable menu, mode, context-menu, platform-menu, widget, and dialog
  declarations that drive the running application.
- Record every asynchronous Qt worker's cancellation, result, GUI-thread
  commit, and revision/digest fencing contract; consolidate only where that
  evidence supports a shared helper.
- Complete the keyboard-only drawing E2E and the structural accessibility
  inventory for every interactive widget and dialog.
- Port the remaining reviewed frontend interaction changes without importing
  Python chemistry or document authority.

## Backend

- Finish the geometry migration inventory and move remaining authoritative
  computations to their assigned Rust crates; the disposable hex grid stays in
  Python.
- Build the chemistry extension with its packaged, replaceable adapter closure
  and run the complete Qt suite without a modal adapter failure.

Track milestone status and acceptance evidence in
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).
