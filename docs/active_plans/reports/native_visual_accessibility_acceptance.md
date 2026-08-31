# Native visual and accessibility acceptance

## Status and owner

- Status: awaiting current-build maintainer review.
- Decision owner: Ferrum maintainer.
- Implementation owners for findings: Qt interaction/accessibility for widget,
  focus, naming, role, state, and recovery defects; semantic palette ownership
  for contrast defects; Rust rendering for any reproducible plan or geometry
  defect.

This is the human acceptance receipt for Ferrum's complete native desktop
surface. It complements automated contracts and the documentation tour; it
does not replace either boundary.

## Preconditions

Record a decision only after all of these are current and green:

1. `./build.sh` publishes the local application.
2. `./check_rust.sh` and `./all_test.sh` exit zero.
3. Both strict glyph-bond measurement lanes report zero violations.
4. `./capture_gui_screenshots.sh` transactionally publishes all 13 scenes from
   an active desktop session.

## Required walkthrough

Use the ordinary `build/current/bin/ferrum-qt` application and the storyboard
in [GUI_TOUR.md](../../GUI_TOUR.md). Exercise the visible controls rather than
calling test-only hooks.

| Review area | Success condition |
| --- | --- |
| Typography | Atkinson molecule labels, charges, isotopes, and two-letter element symbols are legible at normal document scale. |
| Chemical drawing | Bonds appear attached without crossing visible label ink; styled and Haworth-front bonds retain their intended topology. |
| Keyboard | A keyboard-only user can enter and leave authoring modes, operate dialogs and docks, recover focus, and cancel safely. |
| Focus | Keyboard focus is visible, ordered, and restored to the expected control after completion, refusal, and cancellation. |
| Accessibility | Interactive controls expose accurate names, roles, states, values, and enabled/disabled status to native assistive technology. |
| Contrast | Text and meaningful controls meet the repository's 5.5:1 house target wherever that target applies. |
| Recovery | Refusal and error wording is readable, actionable, and leaves the document and interaction state understandable. |
| Window composition | The complete ribbon, tabs, canvas, docks, dialogs, and status bar remain readable and uncropped at the supported review size. |

## Acceptance receipt

Complete every field; an empty field is not an acceptance decision.

- Reviewer:
- Review date and time zone:
- Source revision or worktree identity:
- `build/current` runtime receipt identity:
- macOS version, display scale, and Qt version:
- Input method and assistive technology used:
- GUI tour result:
- Keyboard and focus result:
- Accessibility result:
- Contrast result:
- Typography and chemical-drawing result:
- Recovery result:
- Findings and assigned owners:
- Decision: `ACCEPT` or `REJECT`.

Acceptance succeeds only when every required area is accepted and each
non-blocking observation has a named owner. A rejection creates bounded repair
tasks against the owning Rust, Qt, or palette files and requires a fresh clean
build plus the affected automated and human validation steps before rereview.
