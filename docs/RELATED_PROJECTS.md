# Related projects

Ferrum serves chemists and chemical-software developers who need to draw, inspect,
convert, or preserve 2D molecular records. These links are visitor destinations, not a
dependency list or an implementation recipe.

## Confirmed related projects

### BKChem

- Relationship: prior art or inspiration
- Link: [BKChem user documentation](https://bkchem.zirael.org/wiki/)
- Why visitors may care: It documents the established 2D chemical-drawing workflow that
  Ferrum's desktop application continues for people creating and editing molecular drawings.
- Evidence: Ferrum's [PROVENANCE.md](PROVENANCE.md) records the carried-forward BKChem-Qt
  frontend lineage; BKChem's official wiki describes it as a free chemical drawing program.

### OASA

- Relationship: upstream source, fork, or successor
- Link: [OASA project page](https://bkchem.zirael.org/oasa_en.html)
- Why visitors may care: Developers comparing historical format-manipulation behavior can use
  OASA as the documented predecessor that Ferrum-Chem replaces with a Rust backend.
- Evidence: Ferrum's [PROVENANCE.md](PROVENANCE.md) explicitly identifies OASA as the replaced
  backend and a migration oracle; OASA's project page identifies it as BKChem's Python format
  manipulation library.

### RDKit

- Relationship: companion project, extension, or interoperability tool
- Link: [RDKit overview](https://www.rdkit.org/docs/Overview.html)
- Why visitors may care: Chemical-software developers can use its documented cheminformatics
  algorithms and representations alongside Ferrum's focused document and CLI workflow.
- Evidence: Ferrum's [PROVENANCE.md](PROVENANCE.md) identifies RDKit as the native chemistry
  authority behind Ferrum's private adapter, while RDKit documents its C++ toolkit and 2D/3D
  molecular operations.

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

### Chemical Markup Language

- Relationship: domain standard, guide, dataset, or other visitor resource
- Link: [Chemical Markup Language documentation](https://www.xml-cml.org/documentation/FAQ.html)
- Why visitors may care: It explains the XML molecular-interchange concepts behind Ferrum's
  bounded CML/CML2 import, inspection, and conversion routes.
- Evidence: The CML documentation defines CML as an XML language for core chemical concepts;
  [FILE_FORMATS.md](FILE_FORMATS.md) and [USAGE.md](USAGE.md) define Ferrum's CML/CML2 profile.
- Confidence: possible

## Evidence notes

This guide uses two bounded discovery rounds on 2026-08-28: a seed round from Ferrum's CLI,
desktop, CDML, CML, and lineage evidence; then a widening round for drawing alternatives and
interchange resources. The published entries pass the visitor-relevance gate with official
project documentation and Ferrum's current interface or provenance evidence. Toolchain-only
dependencies are intentionally absent.
