# Ferrum GUI Tour

The Ferrum GUI tour is a manual documentation-capture workflow. It proves completed
user-visible states from the locally built Rust and PySide6 application without adding
visual timing, image bytes, or display access to `all_test.sh`.

## Capture command

Build the local application first, then run this command from the repository root with
an active desktop session:

```bash
./build.sh
./capture_gui_screenshots.sh
```

The command uses `source_me.sh`, the staged `build/runtime/python` native extension,
and the PySide6 source application. Each scene receives a fresh temporary workspace,
opens only ordinary authored CDML input, uses Ferrum's visible command and canvas
workflows, checks an observable completed result, retires the active editing selection, frames the
durable content on a high-contrast documentation canvas, hides the optional hex grid through its
visible action, and captures one fixed 1440 by 900 window. It stages every PNG first, then writes
the complete verified set to `docs/screenshots/` together so a failed scene cannot publish a
partial tour.

Use the following optional commands during local diagnosis:

```bash
./capture_gui_screenshots.sh --list
./capture_gui_screenshots.sh --backend qt
./capture_gui_screenshots.sh --backend easy-screenshot
./capture_gui_screenshots.sh --scene template_catalog
```

`--backend auto` is the default. It prefers the `easy-screenshot` `screenshot` command
when installed and able to capture the exact titled Ferrum window. The current machine
does not provide that command. Qt first uses `QScreen.grabWindow` for the same real
visible top-level Ferrum window, then uses Qt's top-level-widget snapshot only when the
platform declines that screen grab. Neither Qt path requires macOS Screen Recording
permission. The explicit `easy-screenshot` backend remains useful on a configured desktop
that grants the capture terminal Screen Recording permission.

## Storyboard

The capture command produces these stable paths after every scene has completed:

| File | Completed visible capability |
| --- | --- |
| `docs/screenshots/workspace.png` | Editable chemical-document workspace with an ordinary carbonyl fragment. |
| `docs/screenshots/atom_authoring.png` | Add Atom at Point command completed on the canvas. |
| `docs/screenshots/direct_bond.png` | Draw Bond committed between ordinary authored atoms. |
| `docs/screenshots/inserted_cyclohexane.png` | Detached cyclohexane insertion completed. |
| `docs/screenshots/attached_cyclohexane.png` | Cyclohexane attachment completed from an authored carbon. |
| `docs/screenshots/template_catalog.png` | Named oxygen-ring user template placed through the template chooser, with that chooser visibly selected beside the completed placement. |
| `docs/screenshots/selected_atom_edit.png` | Selected carbon changed to nitrogen through Change Element. |
| `docs/screenshots/smarts_result.png` | SMARTS query dock showing a completed carbon match. |
| `docs/screenshots/reaction_arrow.png` | Straight reaction arrow committed to the document. |
| `docs/screenshots/presentation_vector.png` | Renderer-preflighted presentation line committed to the document. |

These are documentation evidence, not permanent visual-regression tests. The harness
uses visible command actions and canvas events together with controlled application-level
readiness and framing hooks. It does not represent a user-input-only E2E test. It captures
completed artifacts rather than pointer previews or editing overlays. The template-catalog
view composites Ferrum's actual selected chooser beside a completed placement, making the named
catalog provenance observable. It does not create source files, application caches, or test
artifacts in the repository.

## Captured tour

![Ferrum workspace showing an authored carbonyl fragment](screenshots/workspace.png)

![Ferrum canvas after Add Atom at Point completed](screenshots/atom_authoring.png)

![Ferrum canvas after Draw Bond completed](screenshots/direct_bond.png)

![Ferrum canvas after cyclohexane insertion](screenshots/inserted_cyclohexane.png)

![Ferrum canvas after cyclohexane attachment](screenshots/attached_cyclohexane.png)

![Ferrum selected template chooser beside a completed reusable oxygen-ring placement](screenshots/template_catalog.png)

![Ferrum canvas after selected carbon changes to nitrogen](screenshots/selected_atom_edit.png)

![Ferrum SMARTS query dock showing a completed carbon match](screenshots/smarts_result.png)

![Ferrum canvas after a reaction arrow is committed](screenshots/reaction_arrow.png)

![Ferrum canvas after a presentation vector is committed](screenshots/presentation_vector.png)

## Publishing screenshots in documentation

The root README has a managed `screenshot-docs` block populated from the inspected current tour.
Re-run this command and inspect the resulting images before changing its embeds. Reuse the exact
paths above with descriptive Markdown alt text and never add placeholder images.
