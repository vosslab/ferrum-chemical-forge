# M6 command palette V1 bounded delivery

This supporting record preserves the delivered slice moved out of the canonical
parity ledger on 2026-08-28. The active
[FULL_PARITY_RUST_FIRST.md](../active/FULL_PARITY_RUST_FIRST.md) remains the M6
scope and status authority.

## Delivered contract

Ferrum now provides a modeless command palette as a registry-derived command
client. `Ctrl+K` is the portable shortcut policy; Qt renders it as `Cmd+K` on
native macOS. The same action is also available from **View > Commands >
Command Palette...**. The palette searches each live registered action's label,
help text, and stable ID, keeps disabled actions visible with an unavailable
explanation, and invokes the exact selected live `QAction` only after a final
enabled-state check.

The keyboard contract is deliberately narrow: the search field retains focus;
bare Up and Down move result selection; Return activates the selected command;
Escape closes the palette and restores the invoking focus. Modified arrows
remain ordinary text-field input. The action registry remains the sole command
projection and handler owner. `resources/menus.yaml` remains authoritative for
the menu placement.

The keybinding layer validates the complete prospective live shortcut set before
startup setup, user reassignment, or default reset changes preferences, managed
bindings, or an action shortcut. This makes collisions with both managed and
otherwise registered actions atomic failures rather than partial state changes.

Permanent evidence is intentionally compact: focused Qt tests own live search,
disabled-command refusal, exact action activation, bare/modifier arrow behavior,
Escape focus restoration, and atomic live-shortcut collision handling. The
planned reaction-specific palette E2E was rejected as redundant: registered
reaction E2E and focused Qt tests already own its durable semantics. Native
shortcut dispatch and accessibility remain one-time real 16:10 desktop evidence,
not a pixel, timing, or reaction-fixture gate.

Current delivery checkpoint: the independently accepted `ActionRegistry`
token/identity-guarded destruction-retirement repair closes the stale-QAction
defect, and the nominal `DocumentDisplayRefreshableV1` ABC boundary is also
delivered and independently accepted at code level. Permanent lifecycle
regressions cover feature-owned `register_existing()` stable-ID reuse/successor
palette dispatch and portable `register()` plus `bind_qt_action()` destruction,
declaration retention, successor rebinding, and dispatch. The display-refresh
evidence covers nominal membership, structural-look-alike rejection, and direct
delegating-adapter forwarding.

Source review, focused diagnosis, the transactionally staged 13-scene recapture,
and image-by-image agent visual review are complete. The current candidate set
uses a non-persistent documentation theme, visible Rust catalog provenance,
page-contained examples, YAML-owned command breadcrumbs, Rust-measured molecule
bounds, and Rust-owned observed-page centering for interchange imports. Final
human release sign-off remains separate. The guidance-format, fresh build,
complete aggregate, registered E2E, installed PyO3, full Qt, affected Rust test,
strict lint, and isolated wheel gates passed; seven independent post-fix reviews
completed and their actionable findings were repaired. Resume with broader
parity-ledger reconciliation, human release sign-off when preparing a release,
and the later approved, in-progress M5.A decision. That earlier stabilization checkpoint did not
approve M5.A; the current decision does. Neither advances full parity.

This completes one bounded M6 discoverability slice. It does not prove a full
M6 usability program, real desktop visual acceptance, or complete Ferrum parity.

## Subsequent shared Command Reference delivery

The modeless Command Reference is a second client of the same unversioned
`CommandCatalogEntry` projection. F1 and **Help > Command Reference...** open
a read-only surface that searches live action label, help, stable ID, current
native shortcut, and validated YAML breadcrumb. It keeps unavailable commands
visible with an explanation, does not activate commands, starts at the filter,
and restores the invoker's focus after Close or Escape. Its filter, result list,
status, and Close control have explicit accessible names/descriptions and a
defined tab order.

The focused Qt evidence covers live metadata, search, no-match, unavailable
state, nonactivation, and focus restoration. That is permanent behavioral
evidence only. Native F1 dispatch and real assistive-technology/visual review
remain human desktop acceptance work; neither this record nor the earlier
palette evidence closes M6 or full parity.

