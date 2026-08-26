# Frontend Convergence Receipt

## Scope

This one-time T21 receipt compares the converged Ferrum interface seams with
the read-only historical BKChem Qt reference at
`OTHER_REPOS/bkchem-oasa/packages/bkchem-qt.app/bkchem_qt/`.
It records transferable interface patterns, rather than treating the reference
tree as a source of document ownership or a product identity.

## One-Time Inventory

On 2026-08-19, a `diff -rq` measurement between the two package directories
reported 149 differing entries.  The inventory was 362 Ferrum files (159
Python files) and 339 reference files (148 Python files).  This is descriptive
evidence only: no percentage, file-count, or textual similarity threshold is a
release gate.  Ferrum intentionally has Rust-specific files and excludes
historical Python document/session and chemistry implementations.

## Rename-Only Transfer Receipt

| Shared seam | Reference pattern or recent change | Ferrum mapping and proof |
| --- | --- | --- |
| Resources, menus, and ribbon | Historical `resources/menus.yaml` declares menu hierarchy separately from callbacks; `f3a6b2f` also adjusted historical `resources/modes.yaml` for responsive tools. These read-only non-shipping reference paths are intentionally plain text. | Ferrum's [`menus.yaml`](../../../packages/ferrum-chem-qt.app/ferrum_qt/resources/menus.yaml) is the sole menu-placement authority, and `ribbon_layout.yaml` declares the grouped, labelled task ribbon. Both resolve existing registry-owned QActions by stable semantic ID; no Ferrum mode schema remains. |
| Action IDs and menu registry | The reference `action_registry.py`, `menu_builder.py`, `context_menu.py`, and `platform_menu.py` separate stable IDs, menu placement, selection menus, and platform roles. Those non-shipping paths remain plain text. | Ferrum preserves those four module seams at [`actions/`](../../../packages/ferrum-chem-qt.app/ferrum_qt/actions/) but maps each ID to an already-owned live `QAction`; the test asserts the public callable signatures. This transfers the composition pattern without copying reference callbacks or models. |
| Mode IDs and lifecycle | The reference `modes/base_mode.py` and `mode_manager.py` provide stable mode identity and lifecycle dispatch. `f3a6b2f` refined arrow, vector, bracket, edit, and interaction paths. | Ferrum's [`modes/base_mode.py`](../../../packages/ferrum-chem-qt.app/ferrum_qt/modes/base_mode.py) and [`mode_manager.py`](../../../packages/ferrum-chem-qt.app/ferrum_qt/modes/mode_manager.py) map the lifecycle concept to `ModeId`, immutable `ModeContext`, and semantic intent dispatch. The test checks mode values, context fields, and lifecycle signatures. |
| Public widgets | The reference `widgets/mode_toolbar.py` gained its responsive compact chooser in `f3a6b2f`; its property, status, zoom, and periodic-table modules establish focused view-client surfaces. | Ferrum mirrors the five public module seams under [`widgets/`](../../../packages/ferrum-chem-qt.app/ferrum_qt/widgets/). The compact chooser reuses the same actions; property/status/zoom receive projection or display values; the periodic table emits an element symbol. The test locks their focused constructors, rather than source text. |
| Dialog field and result shapes | The reference `dialogs/preferences_dialog.py` and `theme_chooser_dialog.py` separate application preference collection from ordinary window controls. | Ferrum maps these names to [`preferences_dialog.py`](../../../packages/ferrum-chem-qt.app/ferrum_qt/dialogs/preferences_dialog.py) and [`theme_chooser_dialog.py`](../../../packages/ferrum-chem-qt.app/ferrum_qt/dialogs/theme_chooser_dialog.py). The test locks the frozen preference-result field order and chooser signatures; application settings and theme application remain outside the dialog. |
| Keybinding table | The reference keybinding manager pattern centralizes portable shortcuts rather than distributing accelerators through widgets. | [`config/keybindings.py`](../../../packages/ferrum-chem-qt.app/ferrum_qt/config/keybindings.py) supplies stable dotted action IDs, default table, persistence, and reset. The test locks `set_binding`, `reset_defaults`, and the supported tool bindings. |

## Rust Authority Boundary

Ferrum's rename/mapping transfer stops at UI composition.  The following are
deliberate Rust-specific boundaries, not parity gaps:

- Document and session authority belongs to the Rust document/session API.
  Python controls receive immutable observations and submit bounded operations.
- Canvas geometry and presentation projection are Rust-issued facts rendered by
  Ferrum clients.  There is no Python compatibility DOM or object-item model.
- Chemistry parsing, conversion, validation, coordinate generation, and
  rendering belong to the Rust backend and typed adapter boundary.
- Local Python undo stacks, OASA bridge calls, and mutable document models from
  the historical reference are not transferable seams.

## Recent Reference Delta Disposition

`f3a6b2f` applied the transferable responsive mode-toolbar, property-summary,
status, zoom, action-reuse, bracket/edit lifecycle, and dialog/controller
concepts above.  Its compatibility DOM, local document/session, historical
chemistry implementation, object-level canvas, and removed Python wavy helper
were discarded or replaced by the Rust boundaries.

`f8fd0e6` applied the useful outcome of routing arrow/geometric property
changes through a single backend-authoritative operation path and the smaller
projection-oriented controller shape.  Its reference-specific Python
document-object, CDML I/O, and historical backend patches were deliberately
discarded.  Ferrum does not promise the reference's unsupported creation
workflows merely because their controllers existed there.

## Executable Evidence

Run the focused portability contract with:

```bash
source source_me.sh && python3 -m pytest \
  packages/ferrum-chem-qt.app/tests/test_frontend_portability_contract.py -q
```

The test validates public IDs, result fields, and callable signatures.  It
does not compare source text and it does not create a standing tree-diff gate.
