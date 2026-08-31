# Related projects

Ferrum serves chemists and chemical-software developers who need to author,
inspect, convert, or preserve bounded 2D chemical documents. These links are
visitor destinations, not a dependency list or an implementation recipe.

## Confirmed related projects

### BKChem

- Relationship: prior art or inspiration
- Link: [BKChem project site](https://bkchem.zirael.org/)
- Why visitors may care: Chemists comparing Ferrum's desktop drawing direction can
  inspect the free chemical-drawing application that supplied Ferrum's carried-forward
  frontend lineage.
- Evidence: [PROVENANCE.md](PROVENANCE.md) records the lineage, and the
  [BKChem project site](https://bkchem.zirael.org/) describes BKChem as a chemical
  drawing program.

### OASA

- Relationship: prior art or inspiration
- Link: [OASA project page](https://bkchem.zirael.org/oasa_en.html)
- Why visitors may care: Developers investigating Ferrum's historical behavior
  comparisons can consult the format-manipulation library that Ferrum-Chem replaces.
- Evidence: [PROVENANCE.md](PROVENANCE.md) identifies OASA as external migration-oracle
  evidence, and the [OASA project page](https://bkchem.zirael.org/oasa_en.html)
  identifies it as the library that forms BKChem's base.

## Possible related projects

### Ketcher

- Relationship: direct alternative or competitor
- Link: [Ketcher project](https://github.com/epam/ketcher)
- Why visitors may care: Chemists can use this web-based editor to draw and edit
  molecular structures and reactions when browser embedding is more useful than
  Ferrum's local desktop workflow.
- Evidence: Ketcher's [official README](https://github.com/epam/ketcher/blob/master/README.md)
  describes a web-based editor for chemists that supports chemical drawing, editing,
  history, and molecular-file import and export.
- Confidence: likely

### MarvinSketch

- Relationship: direct alternative or competitor
- Link: [MarvinSketch getting started guide](https://docs.chemaxon.com/latest/marvin_marvinsketch_getting-started.html)
- Why visitors may care: Chemists can use this desktop drawing environment when they
  need its broader structure-editing, validation, template, and calculation workflow.
- Evidence: The official [MarvinSketch getting started guide](https://docs.chemaxon.com/latest/marvin_marvinsketch_getting-started.html)
  documents chemical drawing, atom and bond editing, templates, structural checks, and
  2D or 3D cleanup.
- Confidence: likely

### Open Babel

- Relationship: same-workflow project or independent implementation
- Link: [Open Babel command-line guide](https://openbabel.org/docs/Command-line_tools/babel.html)
- Why visitors may care: Developers can use `obabel` to convert, filter, and manipulate
  molecular files when their workflow needs broad chemical-format interchange rather
  than Ferrum's bounded conversion and CDML-document preservation path.
- Evidence: Open Babel's official guide describes `obabel` as a program for
  interconverting chemical formats, filtering molecules, and manipulating chemical
  data; [USAGE.md](USAGE.md) documents Ferrum's adjacent CLI interchange workflow.
- Confidence: likely

### RDKit

- Relationship: companion project, extension, or interoperability tool
- Link: [RDKit overview](https://www.rdkit.org/docs/Overview.html)
- Why visitors may care: Chemical-software developers can explore RDKit for
  programmatic cheminformatics and molecular-drawing work adjacent to Ferrum's
  document-authoring and local CLI workflows.
- Evidence: [PROVENANCE.md](PROVENANCE.md) identifies RDKit's narrow local-runtime
  role in Ferrum, while the [RDKit documentation](https://www.rdkit.org/docs/)
  provides its cheminformatics and drawing APIs.
- Confidence: possible

### Chemical Markup Language

- Relationship: domain standard, guide, dataset, or other visitor resource
- Link: [Chemical Markup Language specifications](https://www.xml-cml.org/spec/)
- Why visitors may care: Developers working with Ferrum's bounded CML/CML2 route can
  consult the chemical XML schemas, conventions, dictionaries, and validation context.
- Evidence: The [CML specifications](https://www.xml-cml.org/spec/) define those XML
  resources, and [FILE_FORMATS.md](FILE_FORMATS.md) defines Ferrum's bounded CML/CML2
  profile.
- Confidence: likely

## Evidence notes

This refresh used two bounded discovery rounds on 2026-08-31. The seed round began
with Ferrum's document, format, and provenance evidence; the widening round verified
the explicit BKChem/OASA lineage and drawing-editor leads against their official
project documentation. Entries appear only where the candidate and Ferrum support a
specific shared or adjacent visitor workflow.
