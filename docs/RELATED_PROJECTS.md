# Related projects

Ferrum serves chemists and chemical-software developers who need to draw, inspect,
convert, or preserve 2D molecular records. These links are visitor destinations, not a
dependency list or an implementation recipe.

## Confirmed related projects

### BKChem

- Relationship: prior art or inspiration
- Link: [BKChem project site](https://bkchem.zirael.org/)
- Why visitors may care: Chemists comparing Ferrum's desktop drawing direction can inspect the
  historical free chemical-drawing application from which Ferrum's PySide6 frontend lineage began.
- Evidence: [PROVENANCE.md](PROVENANCE.md) records the carried-forward local frontend lineage,
  and the [BKChem project site](https://bkchem.zirael.org/) describes a chemical drawing program.

### OASA

- Relationship: prior art or inspiration
- Link: [OASA project page](https://bkchem.zirael.org/oasa_en.html)
- Why visitors may care: Developers investigating a historical behavior comparison can consult
  the documented Python format-library reference that Ferrum-Chem replaces with a Rust backend.
- Evidence: [PROVENANCE.md](PROVENANCE.md) defines OASA as external migration-oracle evidence,
  while the [OASA project page](https://bkchem.zirael.org/oasa_en.html) describes it as the
  format-manipulation library that formed BKChem's base.

### RDKit

- Relationship: companion project, extension, or interoperability tool
- Link: [RDKit overview](https://www.rdkit.org/docs/Overview.html)
- Why visitors may care: Chemical-software developers can use its cheminformatics and 2D/3D
  molecular operations alongside Ferrum's focused document and local CLI workflows.
- Evidence: [PROVENANCE.md](PROVENANCE.md) identifies RDKit as Ferrum's private local-runtime
  chemistry dependency, and the [RDKit overview](https://www.rdkit.org/docs/Overview.html)
  documents its cheminformatics toolkit and molecular operations.

## Possible related projects

### Open Babel

- Relationship: direct alternative or competitor
- Link: [Open Babel command-line guide](https://openbabel.org/docs/Command-line_tools/babel.html)
- Why visitors may care: It provides a broader command-line route for converting, filtering,
  and manipulating chemical file data when a workflow does not require Ferrum's CDML document
  preservation and desktop editor.
- Evidence: Open Babel documents `obabel` as a molecular-file conversion and manipulation CLI;
  Ferrum documents its own bounded `convert` and `inspect-graph` workflows in [USAGE.md](USAGE.md).
- Confidence: likely

### Ketcher

- Relationship: direct alternative or competitor
- Link: [Ketcher project](https://github.com/epam/ketcher)
- Why visitors may care: It is a web-based alternative for drawing and editing molecular
  structures and reactions when browser embedding is more useful than Ferrum's local PySide6
  desktop application.
- Evidence: Ketcher's official project documentation describes a web-based chemical structure
  editor with drawing, editing, history, and molecular-file import/export capabilities.
- Confidence: likely

### MarvinSketch

- Relationship: direct alternative or competitor
- Link: [MarvinSketch getting started guide](https://docs.chemaxon.com/latest/marvin_marvinsketch_getting-started.html)
- Why visitors may care: It is a chemically aware desktop editor for structures, queries, and
  reactions when a visitor needs a broader commercial drawing and calculation workflow.
- Evidence: The official getting-started guide documents MarvinSketch drawing, editable atom and
  bond properties, templates, chemical checks, and 2D/3D cleanup.
- Confidence: likely

### Chemical Markup Language

- Relationship: domain standard, guide, dataset, or other visitor resource
- Link: [Chemical Markup Language specifications](https://www.xml-cml.org/spec/)
- Why visitors may care: It explains the XML molecular-interchange concepts behind Ferrum's
  bounded CML/CML2 import, inspection, and conversion routes.
- Evidence: The CML specifications describe schema and convention rules for chemical XML;
  [FILE_FORMATS.md](FILE_FORMATS.md) and [USAGE.md](USAGE.md) define Ferrum's CML/CML2 profile.
- Confidence: possible

## Evidence notes

This refresh used two bounded discovery rounds on 2026-08-28: a seed round from Ferrum's
document, format, and chemistry-toolkit evidence; then a widening round for historical lineage
and desktop drawing alternatives. Each entry passes the visitor-relevance gate with Ferrum's
current product or provenance documentation and the candidate's own authoritative documentation.
Toolchain-only dependencies are intentionally absent.
