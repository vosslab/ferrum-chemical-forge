# M4b SMARTS Capture Action Handoff Fix

Date: 2026-08-20

## Scope

Correct P1 from `ferrum-smarts-qt-selected-root-capture-review.md`: a
construction-time `findChildren(QAction)` scan could not cover later actions
and connected too late to promise cancellation before ownership transfer.

## Change

`FerrumInteractionActionHandoff` is now window-owned and exposes the only
registration path for actions that take canvas interaction ownership. Its
guard connection is made before the owning handler connection and synchronously
calls the selected-root capture canceller first. The capture controller no
longer scans QObject children. The SMARTS dock action intentionally remains a
normal non-owning QAction, so opening the dock does not cancel its own explicit
canvas-choice mode.

Current registrations cover Add Atom, all line/vector/text/bracket/move tool
actions, structure selection, catalog and user-template placement, and both
Haworth placement actions. Future interaction-owning actions must call
`_connect_interaction_action_v1`; ordinary commands must not use it.

## Changed Files

- `ferrum_qt/ferrum/interaction_action_handoff.py`
- `ferrum_qt/ferrum/main_window.py`
- `ferrum_qt/ferrum/smarts_selected_root_capture.py`
- `ferrum_qt/ferrum/smarts_query_dock.py`
- `ferrum_qt/ferrum/line_tools.py`
- `ferrum_qt/ferrum/structure_selection.py`
- `ferrum_qt/ferrum/catalog_palette.py`
- `ferrum_qt/ferrum/user_templates.py`
- `ferrum_qt/ferrum/haworth_tool.py`
- `ferrum_qt/ferrum/direct_glycosidic_haworth_tool.py`
- `tests/test_interaction_action_handoff.py`
- `docs/CHANGELOG.md`

## Regression Coverage Added

`tests/test_interaction_action_handoff.py` provides:

- A representative registered authoring action proof: an armed real capture
  has removed its viewport filter before the incoming handler installs tool
  state; a following canvas click cannot mint a token.
- The identical proof for a QAction created after capture setup and registered
  through the explicit handoff seam.
- A non-owning SMARTS dock QAction proof: it opens without cancelling its own
  armed molecule-choice capture.

## Verification Status

Tests were added but not executed in this handoff task. The next independent
review should run the focused module and verify the full Qt action construction
path, especially dynamic feature actions adopting the explicit registration
seam.

## Residual Risk

Qt cannot infer semantic ownership from an arbitrary QAction. A future canvas
tool that bypasses `_connect_interaction_action_v1` is a code-review defect and
will not receive the ordering guarantee. The explicit seam is deliberately
auditable and avoids pretending a global child scan can solve that problem.
