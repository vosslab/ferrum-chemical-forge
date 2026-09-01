# Authoring ribbon UX review

## Status

The redesigned ribbon meets the automated interaction, accessibility, responsive-layout, and
native-render acceptance defined here. Final assistive-technology narration and subjective
desktop feel remain human acceptance, as required by the Qt contract.

## Task model

The primary user is a desktop chemistry author moving repeatedly among four kinds of work:

1. preserve or reverse work with New, Open, Save, Undo, and Redo;
2. select, draw, and reshape molecular structure;
3. add reaction notation and explanatory annotation; and
4. adjust the current view without leaving the document context.

The critical sequence is choose a recognizable task, find a chemistry command by icon and label,
act directly on the canvas, inspect the active-tool status, and recover through Escape or Undo.
At narrower widths the same command must remain reachable without reordering the task model or
creating a second action identity.

## Baseline findings

The pre-change tracked `docs/screenshots/workspace.png` was inspected at its native 1440 by 900
resolution. It exposed one flat gray bar, generic platform tabs, almost-uniform groups, and mostly
text-only commands. Universal file/history commands competed with authoring tasks, while the
original BKChem visual vocabulary already packaged with Ferrum was absent from the main surface.

The first redesigned build still failed visual consistency. Its flat horizontal action row let
text determine every rectangle: primary controls ranged from 84 to 182 pixels, supporting
controls from 79 to 286 pixels, and **Brackets and lines** reached 881 pixels. Transparent default
edges also made ordinary controls look unrelated to checked, focused, and overflow controls.

| Heuristic | Baseline severity | Delivered response |
| --- | ---: | --- |
| Visibility and hierarchy | 3 | Persistent dark header, active task tab, white/dark cards, and semantic rails. |
| Match to chemistry work | 3 | Structure, reaction, annotation, and view tasks use recognizable BKChem artwork. |
| Recognition over recall | 3 | Primary commands combine a large icon and label; supporting commands keep both. |
| Consistency | 3 | Shared action identity plus one 72-pixel tile frame, 32-pixel width rhythm, and edge system. |
| Flexibility and efficiency | 2 | Quick access keeps file/history actions stable; task pages compact predictably. |
| Aesthetic and minimal design | 3 | Two-row support stacks, aligned edges, cards, and contrast establish hierarchy. |

Severity uses the Nielsen 0--4 scale, where 3 is a major usability problem. The redesign follows
LibreOffice's current task-tab, grouped-command, compact-overflow pattern without copying its
implementation. The original BKChem artwork supplies domain recognition rather than decorative
novelty. References: [LibreOffice NotebookBar help](https://help.libreoffice.org/latest/sq/text/shared/01/notebook_bar.html?DbPAR=CALC&System=UNIX)
and [The Document Foundation's current icon/text implementation note](https://dev.blog.documentfoundation.org/2026/08/31/notebookbar-part-2-icon-text-and-the-code/).

## Delivered interaction contract

- `ribbon_layout.yaml` is the sole owner of quick access, global actions, task placement, role,
  priority, and semantic accent.
- `CommandIconCatalog` is an exact mapping over all ribbon commands and resolves every icon before
  mutating a shared action.
- The header exposes task navigation and Command Palette persistently. All clients have tooltips,
  accessible names and descriptions, and strong keyboard focus.
- Responsive pages reduce later groups from expanded to compact to collapsed. Supporting commands
  move to **More**, then the complete group moves to one labelled popup; action identity and focus
  survive the transition.
- `ribbon_contract.py` owns the component geometry: 72-pixel tiles, paired 34-pixel support rows,
  4-pixel row gaps, 8-pixel component/group gaps, and bounded 32-pixel width increments. A lone
  supporting command fills the same 72-pixel frame instead of leaving an unbalanced half-column.
- Primary, supporting, compact **More**, and collapsed **More** controls share the same background,
  border, corner radius, disabled treatment, semantic-accent hover, checked boundary, and focus
  boundary. The group caption preserves context when a collapsed trigger uses the compact label.
- Light and dark themes own a closed ribbon palette. Text pairs meet 4.5:1 WCAG contrast; focus,
  default/checked state boundaries, and category rails meet 3:1.

## Visual and automated evidence

The complete 14-frame [GUI tour](../../GUI_TOUR.md) is the verified light-theme result at
1440 by 900. Each registered scene exposes the task tab that owns its visible operation;
the same real Qt capture lane can expose another tab or theme for focused diagnosis:

```bash
source source_me.sh && python3 devel/capture_gui_screenshots.py \
  --backend qt --scene workspace --theme dark --ribbon-tab reactions
```

The final full-tour contact sheet covers Home, Structure, Reactions, Annotate, and View at native
1440 by 900 resolution. Individual overlay and reaction frames were also inspected; the complete
reaction-arrow head, live dialog backgrounds, and task tabs remain visible. Eleven unreferenced
legacy flat-toolbar smoke captures and the disposable before-state review directory were removed,
leaving no old ribbon screenshot in the live working-tree corpus. A separate native 1000-pixel diagnostic verified compact/collapsed alignment and found
the initial clipped group-popup labels; the final shared **More** silhouette removes that defect.
The active tab, category rails, labels, icon variants, selected-tool context, document canvas,
properties dock, and status bar were visible without clipping. Focused pytest covers the geometry
rhythm, action identity, header accessibility, live-theme icon refresh, tab selection, Escape
cancellation, focus transfer, responsive
reachability, declarative-resource refusal, and measured palette contrast.
The final `./all_test.sh` front door passed 8,680 repository-hygiene checks, every registered
CLI/Qt E2E workflow, 283 installed PyO3 tests, and 449 Ferrum Qt tests.

## Human acceptance

A human desktop review should still confirm native Tab/Shift-Tab order, screen-reader naming,
tooltip timing, and whether the combined color and BKChem imagery feels appropriately energetic
during ordinary editing. Any failure belongs to the ribbon presentation owners; chemistry and
document owners must not absorb a visual workaround.
