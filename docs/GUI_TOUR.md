# Ferrum GUI Tour

The Ferrum GUI tour is a manual documentation-capture workflow. It records candidate
user-visible states from the locally built PySide6 application without adding visual
timing, image bytes, or display access to `all_test.sh`.

## Capture command

Build the local application first, then run this command from the repository root with
an active desktop session:

```bash
./build.sh
./capture_gui_screenshots.sh
```

The command uses `source_me.sh` and the promoted `build/current` program through the
stable `build/runtime/python` and `build/bin` links, then starts the PySide6 application.
It applies the light document theme transiently
for deterministic documentation contrast without changing the user's saved theme.
Each scene receives a fresh temporary workspace. Ordinary chemistry begins as direct
SMILES insertion; capability-specific scenes use the real CDXML File/Open route and
Ferrum's native peptide importer. It then uses Ferrum's visible command and canvas
workflows, checks an observable
completed result, retires transient pointer previews, preserves meaningful selection
highlights, exposes the task-appropriate Home, Structure, Reactions, Annotate, or View ribbon
tab, frames the durable content on a high-contrast documentation canvas, hides the optional hex
grid through its visible action, and captures one fixed 1440 by 900 application window. The target is the full
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
./capture_gui_screenshots.sh --scene workspace --ribbon-tab reactions
```

`--backend auto` is the default. It prefers the `easy-screenshot` `screenshot`
command when installed and able to capture the exact titled Ferrum window. Qt first
uses `QScreen.grabWindow` for the same real visible top-level Ferrum window, then
uses Qt's top-level-widget snapshot when the platform declines that screen grab.
The explicit `easy-screenshot` backend requires the capture process to have the
platform permissions needed for whole-window screen capture.
Each scene owns its task-appropriate ribbon tab. `--ribbon-tab` is a focused diagnostic
override; it does not change the registered full-tour storyboard.

## Storyboard

The registered full-run storyboard targets these paths. A focused diagnostic run
may write one path, but it does not publish or validate the complete tour.

| File | Completed visible capability |
| --- | --- |
| `docs/screenshots/workspace.png` | Editable sucrose with two rings, a glycosidic oxygen, and stereochemical wedge/hash bonds. |
| `docs/screenshots/pentapeptide_import.png` | Native ANKLE pentapeptide import: one 40-atom, 39-bond molecule moved wholly onto the visible page through Ferrum's typed root operation. |
| `docs/screenshots/atom_authoring.png` | Sucrose with one additional authored atom; Add Atom at Point is visibly rearmed. |
| `docs/screenshots/direct_bond.png` | A committed C-O single bond remains selected. |
| `docs/screenshots/inserted_cyclohexane.png` | Detached six-carbon cyclohexane beside the original carbon. |
| `docs/screenshots/attached_cyclohexane.png` | Cyclohexane with all six ring bonds plus the retained host C-O bond; the visible 7-atom/7-bond document proves attachment did not suppress host ink. |
| `docs/screenshots/template_catalog.png` | Alpha-D-glucofuranose selected in the Template Catalog before placement. |
| `docs/screenshots/selected_atom_edit.png` | Carbon changed to nitrogen through the selected-atom editing workflow. |
| `docs/screenshots/smarts_result.png` | Distearoylphosphatidylcholine prepared through direct SMILES insertion with `[O]` returning all eight oxygen matches. |
| `docs/screenshots/reaction_arrow.png` | Straight reaction arrow committed below editable sucrose. |
| `docs/screenshots/presentation_vector.png` | Adenine-thymine pair with two dashed, noncovalent Watson-Crick hydrogen-bond guides. |
| `docs/screenshots/cdxml_open.png` | ChemDraw XML opened as an editable centered C-O-N-F chain with Wavy, Bold, and Dashed bonds. |
| `docs/screenshots/view_controls.png` | Visible status-bar zoom, Page, and Content controls beside complete distearoylphosphatidylcholine. |
| `docs/screenshots/command_palette_reaction.png` | Command Palette query showing registered reaction commands above sucrose and a reaction arrow. |

## Capture architecture

The registered 14-scene tour uses biological context for sucrose, the ANKLE
pentapeptide, distearoylphosphatidylcholine (DSPC), an adenine-thymine pair, and the
glucose template catalog. Each frame exposes the ribbon tab that owns its visible task;
Home remains visible for general workspace and query context. The harness requires the
fixed logical window, a 16:10 raster, and a Properties title bar wide enough for its
complete visible title. Re-run the transactional command after a source change, then
inspect the staged complete set before treating these embeds as current evidence.

The biochemical scenes prove different boundaries. Sucrose, adenine, thymine, and their
ordinary screenshots are direct SMILES insertions. The capture source reads sucrose and
the two bases from `data/pubchem_molecules_data.yml`; its isomeric sucrose SMILES uses
`@` and `@@`, so RDKit emits the corresponding wedge/hash stereobonds. An achiral SMILES
has no such stereochemical bond directions. The CID 65146 DSPC scene uses the exact
user-provided non-stereospecific SMILES: the YAML's CID 94190 entry is a distinct
stereospecific compound. DSPC uses an A2 landscape page and Content framing while the
molecular graph remains at the canonical 40-point UI bond scale.
Before capture, the source rejects each nonfinite, coincident, or individually
off-canonical bond using a 2% tolerance derived from `DEFAULT_BOND_LENGTH_PT`;
a correct average cannot conceal an invalid individual bond.

ANKLE runs through the actual native peptide worker and yields one 40-atom, 39-bond
molecule. The adenine and thymine rings are separate molecule roots with 19 atoms and
20 covalent bonds; their two dashed presentation lanes remain outside the covalent
molecule graph. Molecular identity is supplied by the repository PubChem data for
sucrose and the bases, and by the exact CID 65146 source SMILES for DSPC.

The remaining scenes preserve focused evidence for direct bonds, detached and attached
rings, selected-atom editing, bounded CDXML, reaction arrows, visible view controls, and
the live command registry. The Template Catalog still shows the selected Rust-backed
glucose entry before placement. Command Palette entries come from the ribbon's YAML
command definitions; that ownership is documented here rather than inferred from alt
text.

The CDXML scene makes the repaired atom-label contract visible: each structural element
core is centered on its connected bond axis, while visible glyph ink retains clearance
from the Wavy, Bold, and Dashed painted bond footprints. This is visual documentation
evidence for the Rust render plan, not a pixel-regression test.

The reviewed frames make the current task state and available actions legible: named
ribbon commands identify authoring modes, docks name query/catalog context, and the
status bar reports the active mode and view scale. Final human native visual,
accessibility, contrast, and focus acceptance remains distinct from this documentation
review. The aggregate E2E, PyO3, Qt, and post-fix audit receipts are recorded
separately because a screenshot run does not prove those gates.

These are documentation evidence, not permanent visual-regression tests. The harness
uses visible command actions and canvas events together with controlled application-level
readiness and framing hooks. It does not represent a user-input-only E2E test. It captures
completed artifacts rather than transient pointer previews. Selection highlights may remain when
they explain a completed editing state. The template-catalog
view uses Ferrum's actual selected catalog palette before any placement, making the named catalog
provenance observable without claiming an obscured canvas result. It does not create source files,
application caches, or test artifacts in the repository.

## Regenerated tour

These embeds are the registered documentation tour. Focused diagnostic captures remain
outside this section; refresh and inspect the full set after capture-source changes.

![Editable stereochemical sucrose drawing in the Ferrum workspace](screenshots/workspace.png)

![Native ANKLE pentapeptide import with one 40-atom and 39-bond molecule](screenshots/pentapeptide_import.png)

![Sucrose with an additional authored atom and Add Atom at Point visibly rearmed](screenshots/atom_authoring.png)

![Committed C-O single bond selected in the Ferrum canvas](screenshots/direct_bond.png)

![Detached six-carbon cyclohexane beside the original carbon](screenshots/inserted_cyclohexane.png)

![Cyclohexane with six ring bonds and the retained host C-O bond](screenshots/attached_cyclohexane.png)

![Alpha-D-glucofuranose selected in the Template Catalog before placement](screenshots/template_catalog.png)

![Carbon changed to nitrogen through the selected-atom editing workflow](screenshots/selected_atom_edit.png)

![Distearoylphosphatidylcholine with all eight oxygen SMARTS matches reported](screenshots/smarts_result.png)

![Committed reaction arrow below editable sucrose](screenshots/reaction_arrow.png)

![Adenine-thymine pair with two dashed noncovalent Watson-Crick hydrogen-bond guides](screenshots/presentation_vector.png)

![Centered C-O-N-F document opened from ChemDraw XML with wavy, bold, and dashed bonds](screenshots/cdxml_open.png)

![Visible status-bar zoom, Page, and Content controls beside complete distearoylphosphatidylcholine](screenshots/view_controls.png)

![Command Palette search results for registered reaction commands above sucrose and a reaction arrow](screenshots/command_palette_reaction.png)

## Publishing screenshots in documentation

The root [README.md](../README.md) has a managed `screenshot-docs` block populated
from this inspected current tour.
Re-run this command and inspect the resulting images before changing its embeds. Reuse the exact
paths above with descriptive Markdown alt text and never add placeholder images.
