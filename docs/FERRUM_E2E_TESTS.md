# Ferrum E2E tests

This guide records Ferrum-specific end-to-end test practice. The repository-wide
test policy remains in [E2E_TESTS.md](E2E_TESTS.md),
[PYTEST_STYLE.md](PYTEST_STYLE.md), and [tests/TESTS_README.md](../tests/TESTS_README.md).

## Purpose and scope

Ferrum E2Es prove a supported local user workflow across the staged Rust
runtime, the private PyO3 bridge, and the Qt application. They are behavioral
evidence for public CLI and GUI contracts. They are not source-shape checks,
renderer pixel comparisons, artifact byte comparisons, or performance
benchmarks.

Use an E2E when the interaction needs more than one implementation boundary.
Keep Rust unit and integration tests responsible for document, chemistry,
identity, history, and renderer-admission rules. Keep PyO3 tests responsible
for the typed private bridge. Keep Qt tests responsible for local presentation
behavior. An E2E should exercise their supported public path without duplicating
those lower-level assertions.

## Running Ferrum E2Es

Build the local product before running the staged E2E runner:

```bash
./build.sh
tests/e2e/run_all.sh
```

[tests/e2e/run_all.sh](../tests/e2e/run_all.sh) is the registration point for
supported local E2Es. Add a new permanent Ferrum E2E there with a descriptive
phase label. The runner validates the staged local runtime, sets Qt to offscreen
operation, and invokes the registered CLI and Qt workflows.

Use the project Python environment for authoring, focused runs, and helper
commands:

```bash
source source_me.sh && python3 -B tests/e2e/NAME.py
```

Every direct Qt E2E selects the test-owned `offscreen` platform before it
imports PySide6. This keeps the same command safe on macOS and headless hosts;
the runner retains its process-level `QT_QPA_PLATFORM=offscreen` export as a
second launch boundary. Product launchers and [source_me.sh](../source_me.sh)
retain native desktop Qt behavior. Verify a real desktop GUI launch manually as
one-time product validation, not as a permanent E2E timing or image-equivalence
gate.

`./all_test.sh` is the broader repository validation command. Use the focused
runner while developing an E2E, then use `./all_test.sh` to detect integration,
hygiene, and registered-test drift. A build, a focused E2E, and the full suite
answer different questions; none substitutes for the others.

## Suite ownership

The registered entries in [tests/e2e/run_all.sh](../tests/e2e/run_all.sh) are
the permanent local E2E suite. `./all_test.sh` invokes that runner after it
validates the staged local runtime and launchers.

[tests/e2e/ferrum_qt_e2e.py](../tests/e2e/ferrum_qt_e2e.py) owns shared Qt E2E
launch and interaction support. Keep workflow-specific behavior in its
`e2e_*.py` script; keep the shared module focused on cross-workflow process and
UI boundaries. It is support code, not a fixture catalog or a separate test.

A script under `tests/e2e/` is not permanent merely because of its location.
Treat an unregistered script as focused implementation evidence until its
recurring product value satisfies the permanent-test checklist below.

## Public interaction rules

Qt E2Es run with `QT_QPA_PLATFORM=offscreen`. They must discover and operate
the application through supported visible UI surfaces: actions, menus, dialogs,
canvas input, visible text, accessible names, and public state shown by the
application. The test should use the same document for setup, refusal or
cancellation, recovery, and final success whenever that makes the contract
clear.

Do not assert private Rust identifiers, raw CDML, source layout, helper names,
or exact pixels and bytes. Do not use arbitrary sleeps or timing thresholds.
When a modal could prevent progress, use a narrowly scoped liveness guard to
record and dismiss an unexpected modal. It protects the test harness from a
deadlock; it is not a responsiveness requirement. Expected modals should be
found through their public title, accessible name, body, or controls and be
dismissed by the supported visible control.

Ordinary permanent E2Es remain offline. Network-dependent work belongs in a
separate, explicitly justified validation lane rather than the normal Ferrum
E2E runner.

## Setup and fixture policy

Prefer inline, minimal setup created through the user-facing workflow. A test
may establish a small same-document baseline before exercising its contract.
Use temporary files only where the workflow genuinely needs a file boundary.
Do not introduce shared fixture catalogs, raw document payloads, or synthetic
data layers merely to make an E2E shorter.

## Permanent-test checklist

A permanent Ferrum E2E has all of these properties:

- It protects a supported, user-visible CLI or GUI contract across a real
  boundary.
- It is deterministic, offline, self-contained, and suitable for the normal
  local runner.
- Its setup and assertions are minimal and behavioral.
- It has a clear failure mode that a lower-level Rust, PyO3, or Qt test would
  not cover alone.
- Its registration in [tests/e2e/run_all.sh](../tests/e2e/run_all.sh) is justified by recurring product
  value rather than implementation proof.

Use one-time checks for fresh-build investigation, manual visual inspection,
and documentation screenshots. They can establish confidence during a change,
but they do not become permanent E2Es unless they meet the checklist above.

## Screenshot automation

The GUI tour is automated documentation evidence, separate from the permanent
E2E suite. Build Ferrum, list the available scenes, and capture the tour with:

```bash
./build.sh
./capture_gui_screenshots.sh --list
./capture_gui_screenshots.sh
```

The capture script drives the locally built Qt application and publishes
verified PNG files under `docs/screenshots/`. Use `--scene NAME` for one scene
or `--backend qt` when a deterministic Qt capture is preferable. See
[GUI_TOUR.md](GUI_TOUR.md) for the scene catalog and review workflow.

Each screenshot targets a 16:10, 1440 by 900 complete `QMainWindow` surface.
The visible authoring ribbon and status bar are part of that surface; the canvas
is content within it, not the screenshot aspect boundary. The capture harness
measures each PNG and rejects a backend result that crops Ferrum or includes
extra window geometry.

Review the resulting images for visible correctness and documentation quality.
Do not turn those images into pixel-equivalence tests. Permanent E2Es assert
the user-visible behavior that creates the scene; the screenshot tour records
what that behavior looks like for readers.

## Example: free methyl placement

The free-Me E2E is an example of a permanent public GUI contract: it invokes
Draw > Compact groups > **Place Compact Group...**, selects the Me option, releases on the canvas, and
then verifies the placed durable group through supported UI behavior. Its
Molecule Report assertion reads `Authored graph: 1 atoms, 0 bonds`, followed by
`Formula: CH4`; together these distinguish the explicit authored representation
from the compact-group chemistry. It proves the public authoring path without
imposing a fixture catalog or asserting a private representation. See its registered entry in
[tests/e2e/run_all.sh](../tests/e2e/run_all.sh) for the current executable name.

## Example: reviewed NO2 attachment

The public attached-NO2 E2E draws ethane through visible authoring tools,
selects an eligible carbon, chooses the visible `NO2` option from Draw > Compact
groups > **Attach Compact Group...**, and materializes the result through the supported workflow.
It verifies editable atom selection and Molecule Report facts `Authored graph:
5 atoms, 4 bonds`, elements `C2/N1/O2`, `Formula: C2H5NO2`, and `Net formal
charge: +0`.

Those facts prove the generic chooser, durable attachment, materialization, and
user-facing composition without exposing private IDs or recipe internals. Rust
unit and session tests own the individual `+1` nitrogen and `-1` oxygen proof.
Manual visual orientation review is one-time release evidence; pixel
comparison, timing thresholds, fixture catalogs, and incidental selection or
record ordering are not permanent requirements.

## Example: reviewed Methoxy attachment

The public attached-OMe E2E draws its carbon anchor through visible bond
authoring, selects that anchor, chooses `OMe` from Draw > Compact groups > **Attach Compact Group...**,
and materializes the result through the existing visible action. Molecule
Report then observes `C3H8O`, `C: 3, O: 1`, and the corresponding editable
authored graph. These are user-facing chemistry facts, not a private recipe or
coordinate assertion.

Rust/session and binding tests own oxygen-first attachment, generic
renderer-issued pose normalization, history, reopen, and refused no-mutation
semantics. The E2E remains permanent because it covers the distinct public
chooser-to-materialization workflow without raw CDML, private IDs, mocks,
pixel equality, arbitrary waits, or fixture catalogs.

## Example: Check Structure compact-group recovery

The public diagnostics E2E authors an attached `Me` group through the visible
Qt workflow, leaves the group unexpanded, and uses Chemistry > `Check
Structure...`. It requires the accessible modeless dialog to expose the
Rust-owned unexpanded-group finding and materialization recovery while stating
that the check leaves the molecule unchanged. It then uses the existing visible
materialization workflow and verifies `Formula: C3H8` through Molecule Report.

This is permanent E2E evidence because it covers a user-visible, read-only
diagnosis and recovery handoff that lower layers cannot prove. It creates state
through the public UI and makes no raw-CDML, private-ID, fixture-catalog,
pixel, arbitrary-delay, or canvas-navigation assertion. Fresh staged-extension
execution, full-suite execution, and visual review remain one-time validation
evidence rather than additional permanent tests.

## Example: reaction workflow

The public reaction-workflow E2E creates a reaction through the visible
**Create Reaction** action, then uses **Reaction Inspector** to replace roles,
highlight and nudge a member, and delete only the reaction definition. It
locates the accessible `Reaction details` and `Validation: Strict` surfaces and
observes the durable semantic result: role replacement changes the reaction,
while definition-only deletion preserves its member structures.

Expected nested modals are registered before the workflow begins; any
unexpected modal remains a fail-closed harness failure. The E2E makes no raw
CDML, private-ID, coordinate, count, timing, pixel, or fixture-catalog
assertion. It remains permanent because it proves the visible reaction
authoring contract across the Rust runtime, PyO3 bridge, and Qt application.
