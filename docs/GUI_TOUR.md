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
uses ordinary authored CDML input except for the bounded CDXML File/Open scene,
and uses Ferrum's visible command and canvas
workflows, checks an observable completed result, retires the active editing selection, frames the
durable content on a high-contrast documentation canvas, hides the optional hex grid through its
visible action, and captures one fixed 1440 by 900 application window. The target is the full
`QMainWindow` surface, including the visible ribbon and status bar; the canvas is framed within that
surface and is not the aspect-ratio boundary. The harness requires a 1440 by 900 logical window and
rejects a staged PNG whose raster aspect is not 16:10. This permits platform display scaling; human
visual review confirms that the captured surface is complete and uncropped.
It stages every requested PNG first. A default full run begins publication only
after every scene and dimension check succeeds, so a failed scene publishes none
of that run. Publication then replaces the stable files sequentially; human review
of the final set remains required. A diagnostic `--scene` run publishes only its
selected image.

Use the following optional commands during local diagnosis:

```bash
./capture_gui_screenshots.sh --list
./capture_gui_screenshots.sh --backend qt
./capture_gui_screenshots.sh --backend easy-screenshot
./capture_gui_screenshots.sh --scene template_catalog
```

`--backend auto` is the default. It prefers the `easy-screenshot` `screenshot`
command when installed and able to capture the exact titled Ferrum window. Qt first
uses `QScreen.grabWindow` for the same real visible top-level Ferrum window, then
uses Qt's top-level-widget snapshot when the platform declines that screen grab.
The explicit `easy-screenshot` backend requires the capture process to have the
platform permissions needed for whole-window screen capture.

## Storyboard

The registered full-run storyboard targets these paths. A focused diagnostic run
may write one path, but it does not publish or validate the complete tour.

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
| `docs/screenshots/cdxml_open.png` | Bounded ChemDraw XML opened through the Rust interchange registry as an editable document. |
| `docs/screenshots/view_controls.png` | Visible status-bar reset and content-fit controls applied to the active canvas. |
| `docs/screenshots/command_palette_reaction.png` | Command Palette finding the registered reaction-arrow action in a real window. |

## Paused capture status

The `ActionRegistry` destruction-retirement repair and nominal document-display
refresh contract are delivered and independently accepted at code level. The last
automated 13-scene run completed after the lifecycle repair, but human visual
review accepted five frames and rejected eight. It is therefore not an accepted
tour or publication source.

The GUI capture-driver/catalog repair is implemented with focused evidence: it
uses the public Rust-backed template palette, visible catalog provenance,
theme-owned colors, scene-specific final states, and neutral arrow/vector
captures. The command-palette hierarchy/relevance repair is also implemented with
focused evidence: YAML breadcrumbs and relevance ordering work with declared
dynamic-menu placement. Both patches require independent final review; neither
has been recaptured or independently visually reviewed.

Resume with independent review of those two repairs, focused recapture and then
the complete 13-scene run, followed by human visual review. Do not represent the
current embeds as a freshly regenerated or visually accepted set. The guidance
document-format, aggregate, E2E, PyO3, Qt, and post-fix audit gates remain
separate open work.

These are documentation evidence, not permanent visual-regression tests. The harness
uses visible command actions and canvas events together with controlled application-level
readiness and framing hooks. It does not represent a user-input-only E2E test. It captures
completed artifacts rather than pointer previews or editing overlays. The template-catalog
view uses Ferrum's actual selected catalog palette beside a completed placement, making the named
catalog provenance observable. It does not create source files, application caches, or test
artifacts in the repository.

## Previously published tour

These embeds are the prior documented tour. They do not claim that the current
13-scene run or its human visual review is complete. Focused diagnostic captures
remain outside this section until the full set is accepted.

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

The root [README.md](../README.md) has a managed `screenshot-docs` block populated
from the previously inspected published tour.
Re-run this command and inspect the resulting images before changing its embeds. Reuse the exact
paths above with descriptive Markdown alt text and never add placeholder images.
