# Use Ferrum

The Rust `ferrum` command reads CDML without Python. Ferrum-Qt starts its ordinary
window on a bounded Rust-native document path, but it is not yet a finished desktop
replacement.

## Verify Rust code

Run the repository-owned Cargo front door from anywhere inside the checkout:

```bash
./check_rust.sh
```

The script checks rustfmt, all Cargo targets, strict Clippy, workspace tests,
doctests, and API documentation for both the main eight-crate workspace and the
standalone PyO3 extension workspace. It retains the bounded Cargo target cache
for faster later builds. Native wheel construction, RDKit source builds, and
Python/Qt end-to-end tests remain separate platform-specific gates.

## Quick start

Inspect an authored CDML fixture from the repository root:

```bash
ferrum cdml inspect tests/e2e/corpus/authored_document_forms.cdml
```

Produce one complete render observation for a CDML file:

```bash
ferrum cdml render-observation drawing.cdml
```

Render one complete admitted CDML document through the native SVG backend:

```bash
ferrum cdml render svg drawing.cdml --output drawing.svg
```

Render through the direct vector PDF or raster PNG backend:

```bash
ferrum cdml render pdf drawing.cdml --output drawing.pdf
ferrum cdml render png drawing.cdml --output drawing.png --width 800 --height 1131
```

Start Ferrum-Qt after completing the setup in [INSTALL.md](INSTALL.md):

```bash
ferrum-qt
```

It creates a new empty Rust-owned document. External CDML Open, including a command-line
file argument, admits an uncompressed `.cdml` file through the same Rust-owned local-CDML
V1 profile used by the native render command. Parsing and render observation run outside
the Qt thread; Python does not read the file or select its numeric resource limits.

## Ordinary-window native document

The normal `MainWindow` is the native-first product root. File > New creates another
empty Rust-owned document; closing the last tab leaves a supported zero-page shell. It
does not create or register a legacy document session. The retained OASA-backed migration
editor is the explicit internal `LegacyCompatibilityMainWindow`, not the ordinary command
path. OASA remains a dependency until the M22 migration is complete.

File > Open accepts one uncompressed local `.cdml` document. Each accepted path opens in a
new Rust-native tab; multiple startup paths queue through the same asynchronous admission
boundary. A duplicate exact origin activates its existing tab. Cancel Open and window close
invalidate delivery while a native read already in progress finishes safely. CD-SVG,
compressed input, same-tab replacement, and recent-file routing remain separate work.

Select one or more durable atoms or bonds and choose Chemistry > Molecule Information...
to inspect every owning direct-root molecule. Ferrum keeps the accepted authored source
facts beside RDKit-perceived isotope/charge-aware formula, atom counts, net formal charge,
average molecular weight, monoisotopic mass, and average-mass element percentages. Multiple
roots are shown in document order with one checked combined selection. The dialog is
read-only and selectable; closing or changing the source tab discards stale worker delivery.

To edit an authored molecule label, select one or more durable atoms or bonds from exactly
one molecule and choose Chemistry > Set Molecule Name.... Ferrum stores the entered text
exactly, including surrounding whitespace; submit an empty value to remove the name. An
unchanged value is a no-op. The change participates in Undo/Redo and ordinary Save/reopen.

In a native document, select exactly one durable atom to use Edit > Change Element with
Ferrum or Edit > Edit Atom Properties with Ferrum. The property action sends only a closed
Rust patch for properties the dialog can represent exactly; fractional or other
unrepresentable authored facts produce a visible error before mutation, and cancelling the
dialog changes nothing. File > Save and Save As publish through Rust, and closing the tab
retires that native session.

The native document has Edit > Undo with Ferrum and Edit > Redo with Ferrum. These actions
use only the Rust session's revision-checked history and install a fresh authoritative
projection. If no history target exists, Ferrum reports that failure and does not mutate the
document.

Edit > Set Atom Number with Ferrum... is available only when exactly one durable atom is
selected. It submits a positive atom number and an explicit show-number choice through the
typed Rust session; cancelling changes nothing. Edit > Clear Atom Number with Ferrum
removes both the durable number and its show-number state in one revision-bound Rust
operation, retains the selection, and then disables itself. Neither action falls back to
OASA.

Edit > Delete Selected Atom with Ferrum atomically deletes one selected durable atom and
its directly typed incident bonds. Edit > Delete Selected Bond with Ferrum deletes one
selected durable bond while retaining both endpoint atoms. In both cases Rust replaces the
projection, clears selection, and supports native Undo; typed failures remain visible without
an OASA or local-scene fallback.

Edit > Edit Bond Properties with Ferrum is available for one selected durable bond. It sends
one revision-bound Rust patch through the capability-limited `BondDialog`; the current profile
supports normal single, double, and triple bonds and only renderer-supported width and
centering combinations. Unrepresentable facts fail visibly before mutation, and cancellation
does nothing.

This is an M16 adoption slice, not a claim that every legacy capability has moved to Ferrum.
The complete retained session/action cutover remains open.

## CDML commands

- `ferrum cdml inspect INPUT [--format json|text]` prints a typed CDML summary.
- `ferrum cdml validate INPUT [--typed] [--format json|text]` validates retained
  structure; `--typed` also requires current core molecule facts.
- `ferrum cdml rewrite INPUT --output OUTPUT` structurally re-emits CDML.
- `ferrum cdml rewrite INPUT --check` verifies structural preservation without
  writing a file.
- `ferrum cdml extract-cdsvg INPUT --output OUTPUT` extracts one canonical CDML
  payload from decoded CD-SVG.
- `ferrum cdml render-observation INPUT` emits one complete V1 render-observation
  JSON object.
- `ferrum cdml render svg INPUT --output OUTPUT` renders a complete document
  through the native SVG backend.
- `ferrum cdml render pdf INPUT --output OUTPUT` renders a complete document
  through the native vector PDF backend.
- `ferrum cdml render png INPUT --output OUTPUT --width WIDTH --height HEIGHT`
  renders a complete document through the native raster PNG backend.
- `ferrum cdml generate-coordinates --adapter LIBRARY --molecule-id ID INPUT
  --output OUTPUT` regenerates one existing ordinary durable molecule.

Use `-` as an input path for standard input. For rewrite and extraction, use
`--output -` for standard output; all three native artifact commands support
both stream positions too. `--help` describes the installed CLI and
`--version` reports the installed Ferrum version.

## Native document artifacts

Every `cdml render` format admits uncompressed CDML with the versioned
`ferrum-local-cdml-ingress-v1` five-dimensional resource profile. It observes
revision zero, composes one authenticated page, refuses any excluded root, and
then lowers through exactly one native backend. SVG enforces its completed-text
cap while appending; PDF performs structural preflight plus a completed-output
check; PNG checks raw RGBA allocation before rasterization and encoded bytes
afterward.
For file output, Ferrum retains the exact opened source descriptor and refuses
the source path or an observed hard-link alias as the destination.
Confirmed publication is silent. If the platform cannot confirm directory-entry
durability after replacement, the command succeeds and writes a warning to
standard error; failures that may have published remain errors with that fact in
their diagnostic.

```bash
ferrum cdml render svg - --output result.svg < drawing.cdml
ferrum cdml render pdf - --output result.pdf < drawing.cdml
ferrum cdml render png - --output result.png --width 800 --height 1131 \
  < drawing.cdml
```

Use `--max-output-bytes` to select a completed-artifact cap. PDF also exposes
`--max-plan-items` and `--max-path-commands`. PNG requires exact nonzero
dimensions, defaults to a white background, accepts `--background transparent`
or six hexadecimal RGB digits without `#`, and exposes `--max-raw-bytes`.
Defaults are recorded in
`docs/active_plans/reports/local_cdml_render_profile_v1.md`.
PDF is produced directly from Ferrum's vector draw stream; it never contains an
SVG document merely wrapped in a PDF container.

## Render observation

`render-observation` writes exactly one newline-terminated
`ferrum-render-observation-v1` JSON object on success. It loads the supplied CDML at
revision 0 and reports the resulting Ferrum depiction plan and issues.

```bash
ferrum cdml render-observation - < drawing.cdml
```

Argument errors exit with status 2. Accepted commands that cannot read, process, or
write data exit with status 1 and write the error to standard error.

## Existing-molecule coordinate regeneration

The provisional coordinate command requires an explicit absolute, regular, non-symlink ABI-4
adapter and an exact authored molecule `id`. Ferrum does not guess a molecule or search for an
adapter.

```bash
ferrum cdml generate-coordinates --adapter /absolute/path/libferrum_chem.dylib \
  --molecule-id molecule-1 drawing.cdml --output regenerated.cdml
```

Use `-` for either input or output when piping CDML. Ferrum parses and selects the durable molecule
before loading native code, sends the adapter a coordinate-free complete graph, and accepts all
returned atom points as one document operation. File output uses the same atomic publication path
as `rewrite`.

The result retains the molecule's existing centroid, existing mean bond length when bonded, and
each atom's `z` value. It rejects missing or non-durable selectors, pseudo-vertices, unsupported
atom facts, drawing-specific bond styles, unsupported bond orders, unusable source scale, stale
results, and unsafe adapter paths. The contract is semantic; it does not require byte-identical
CDML, pixel-identical rendering, or coordinates from a different toolkit version.

## SMILES inspection

The provisional SMILES route requires an explicitly supplied ABI-4 adapter library.
The path must be absolute, name a regular file, and not be a symbolic link. Ferrum
does not search for an adapter.

```bash
ferrum smiles inspect --adapter /absolute/path/libferrum_chem.dylib CCO
```

Success writes one newline-terminated `ferrum-smiles-inspection-v1` JSON object with
the canonical SMILES value, atom and bond facts, and atom-aligned coordinates.

To parse one SMILES value and re-emit its complete graph through the optional canonical
SMILES writer:

```bash
ferrum smiles canonicalize --adapter /absolute/path/libferrum_chem.dylib 'C(C)O'
```

Success writes exactly one printable canonical-isomeric SMILES line (`CCO` in this
example). The command never searches for an adapter and does not use OASA. This remains
a provisional pre-M18 CLI operation rather than a frozen shell contract.

## Bounded SDF inspection

The provisional SDF reader uses the same explicit ABI-4 adapter policy. It accepts a
UTF-8 SDF file or `-` for standard input and writes one
`ferrum-sdf-inspection-v1` JSON object.

```bash
ferrum sdf inspect --adapter /absolute/path/libferrum_chem.dylib molecules.sdf
```

The report retains record order, titles, ordered repeated properties, complete
molecule facts, and atom-aligned finite 2D coordinates. It is not a general SDF
compatibility claim: three-dimensional conformers, compressed suppliers, and
arbitrary field types remain open.

## Bounded molblock inspection

The provisional molblock reader accepts exactly one bounded UTF-8 V2000 or V3000
molblock from a file or standard input. It uses the same explicit adapter policy and
writes one `ferrum-molblock-inspection-v1` JSON object.

```bash
ferrum molblock inspect --adapter /absolute/path/libferrum_chem.dylib molecule.mol
```

The report contains the canonical SMILES value, complete atom and bond facts, and
finite atom-aligned 2D coordinates. It rejects 3D conformers, SDF record separators,
multiple CTAB terminators, and oversized input instead of inferring a broader format.

## CDML examples

Rewrite a document to a new file:

```bash
ferrum cdml rewrite drawing.cdml --output rewritten.cdml
```

Check the structural contract without creating output:

```bash
ferrum cdml rewrite drawing.cdml --check
```

Extract CDML from a decoded CD-SVG file:

```bash
ferrum cdml extract-cdsvg drawing.svg --output extracted.cdml
```

`rewrite` preserves parsed XML structure, including opaque elements, namespaces,
comments, processing instructions, and mixed content. It does not promise
byte-for-byte or lexical identity. Compressed `.svgz` input is not accepted by
`extract-cdsvg`.

## Native bounded editor

`ferrum-qt` starts the OASA-free Rust-native product root with a new empty document. It
opens uncompressed local `.cdml` files through the approved Rust-owned local-CDML V1
profile and saves through the Rust publication boundary. Its bounded Edit menu can:

- import one SMILES or representable InChI string in a native worker, or import
  one bounded local V2000/V3000 Molfile, then commit the complete handle-free
  molecule only if the source document revision and digest remain current.
  Unsupported stereochemistry and other document facts fail visibly rather than
  being removed;
- choose `Edit > Import Supported Peptide Sequence...` to insert one strict native
  peptide template. Enter uppercase one-letter text with no spaces, using only
  `ACDEFGIKLMNQRSTVY`; the exact accepted text is sent unchanged. H, P, and W are
  currently unsupported and fail visibly before native library loading. A successful
  import is one ordinary Rust-owned history entry and can be undone/redone, saved, and
  reopened. This is a bounded native profile, not generic peptide construction or a
  legacy OASA fallback;
- import every supported 2D record from one bounded UTF-8 `.sdf` or `.sd` file as a
  single undoable Rust transaction. Records are placed in a horizontal row at the
  current insertion anchor, with one requested bond length between adjacent bounds.
  Titles, blank titles, property order, repeated property names, and multiline values
  remain attached to their source molecule in preserved Ferrum extension metadata;
- export Standard or Fixed-H InChI for one chosen durable molecule. Rust validates
  the complete projected graph and exact document provenance before packaged native
  work; a still-current result is copied to the clipboard without changing the
  document;
- export one selected durable molecule through `Chemistry > Export Molfile V2000...`
  or `Export Molfile V3000...`. Rust preserves supported atom, bond, and coordinate
  facts, converts the document's downward-positive y axis to Molfile's
  upward-positive convention, and publishes the exact native result as `.mol`
  without changing the document or adopting the destination path. An authored
  molecule name is passed into the optional native title-aware writer and
  retained as the exact first Molfile line;
- export one selected durable molecule through
  `Chemistry > Export SDF Record V2000...` or
  `Export SDF Record V3000...`. Imported SDF metadata remains authoritative:
  blank or exact titles, property order, duplicate property names, and
  multiline values are recovered from the retained typed document. An
  ordinary molecule uses its authored name as the title or a blank title when
  unnamed. Rust writes the one-record SDF envelope and publishes the exact
  `.sdf` receipt without changing the document or adopting the destination
  path. This is not a multi-record selection, public Python, CLI, or wire
  interface;
- inspect one or more molecules through `Chemistry > Molecule Information...` by
  selecting durable atoms or bonds. The read-only result combines exact authored source
  facts with native formula, charge, mass, isotope-aware counts, and mass percentages;
- set or clear the exact authored name of one molecule through
  `Chemistry > Set Molecule Name...` after selecting one or more of its durable atoms or
  bonds. Empty input clears the attribute; unchanged input adds no history;

- change the element of one durably selected atom;
- edit one durably selected atom's authored element, formal charge, valence,
  isotope, multiplicity, label visibility, hydrogen visibility, label font size,
  and label colour as one Rust-owned history entry;
- add one free-standing atom to a durable molecule after an explicit element and molecule
  choice, using the exact clicked scene position with `z = 0`;
- connect exactly two selected durable atoms in one molecule with a normal single bond, or
  use Draw Single Bond to drag from one existing atom to another or into empty space, where
  Rust creates one carbon and its single bond as one history entry;
- move one durable atom with Move Atom while preserving the pointer-to-atom offset;
- delete one durable atom and every typed bond incident to that atom as one history entry;
- delete one durable typed bond while preserving both endpoint atoms;
- delete one selected durable direct-root presentation object while refusing to split a
  bracket pair;
- bring selected durable presentation roots to the front, send them to the back, or reverse
  their existing element slots;
- normalize eligible non-ring bond lengths to an explicitly entered scene-point value while
  retaining existing directions and ring geometry;
- normalize movable non-ring bond angles to distinct nearest 60-degree directions while
  preserving nondegenerate lengths, authored child order, ring geometry, and anchored edges;
- regularize one supported simple ring per selected molecule to an explicitly entered side
  length while retaining its centroid and attached acyclic substituent geometry;
- snap selected molecules to an explicitly entered hex-grid spacing, or snap every durable
  molecule when no object is selected;
- straighten degree-one terminal bonds in selected molecules, or in every durable molecule
  when no object is selected;
- clean selected bonded molecules, or every durable bonded molecule when nothing is selected,
  in a cancellable native-chemistry worker at an explicitly entered target bond length while
  retaining each molecule's source centroid;
- change one selected durable normal bond among single, double, and triple order;
- edit one selected durable direct-root Text's baseline, subscript, and superscript runs,
  integer size, and foreground colour as one Rust-owned history entry;
- regenerate one durable ordinary molecule's coordinates in a worker while retaining its current
  centroid and mean bond length;
- undo and redo those Rust-owned operations; and
- refresh the authoritative Rust observation if an accepted mutation could not be projected.

The File menu exports the current Rust snapshot as SVG, PDF, or PNG. Export obtains a fresh
observation at the displayed revision and paints a detached scene, so selection and hover feedback
are not included. SVG and PDF remain vector output. PNG uses 72 pixels per inch because native
document geometry is defined in 72 points per inch; it also honors Qt's configured image-allocation
limit. All three formats publish atomically and reject an existing symbolic-link destination.

New atom and bond identifiers are generated by the Rust document session. Add Atom does not
implicitly create a bond, invoke a hidden snap policy, or run coordinate generation. Draw Single
Bond holds only a disposable Qt preview; the exact endpoint or release point, revision check,
identities, history, and saved records remain Rust-owned. Releasing in empty space creates a
carbon at the exact scene point and its normal single bond atomically; it does not invoke a
hidden snap or coordinate-generation policy. The gesture still creates a normal single bond;
choosing another element or order during that gesture and editing wedge, hashed, dashed,
aromatic, or ring-side depiction styles are not exposed. Move Atom sends one final finite point
to the Rust session; Qt retains only its temporary drag line, and Rust owns the revision,
history, projection, and saved `<point>` record. Delete Selected Atom sends only one durable atom
identity. Rust removes that atom plus its direct typed incident bonds atomically; opaque
reference-looking XML remains preservation data. Delete Selected Bond likewise sends only one
durable bond identity, and Rust removes exactly that typed molecule bond without changing its
endpoint atoms. Change Selected Bond Order sends only that durable identity and a closed Rust
single/double/triple value. Generate Molecule Coordinates retains one molecule's current mean
bond length. Clean Geometry is a separate selected-or-all-molecule operation: it validates the
complete batch,
regenerates each supported graph behind the native chemistry boundary at the entered bond length,
and commits all resulting x/y changes together. Neither action asks Qt to interpret persistent
chemistry or coordinate topology. If projection replacement fails after Rust accepted a mutation,
the tab disables every mutating, save, and close action until Refresh Authoritative View succeeds.

Rotate Selected Atoms is a checkable pointer tool for one or more durable atoms. Drag around the
selection center to see a dashed local skeleton, release to submit one Rust rotation, or press Esc
to discard the preview. The preview is interaction guidance only: Qt does not move authoritative
render-plan items or write document coordinates, and Rust remains responsible for the committed
geometry, history entry, and replacement render observation.

Move Complete Roots is a checkable pointer tool for complete durable selections: every atom of a
molecule or a supported durable presentation root. Drag to move dashed local root bounds, release
to submit one Rust translation, or press Esc to discard the preview. Qt never moves the installed
render-plan items. The captured revision, exact root identities, complete-selection validation,
document coordinates, history, and replacement observation remain Rust-owned.

Delete Selected Presentations resolves every rendered durable identity back to an authored
direct-root selector, then sends the complete selection and closed record kinds to Rust. Rust
verifies the expected revision, exact direct-root kinds, and bracket-pair ownership before changing
the document atomically. It does not delete projection-local roots or a partial bracket pair; both
members of a selected bracket pair are deleted together.

Presentation stack actions send a closed ordering mode plus exact kind-and-source-ID selectors.
Rust retains selected source order for front/back moves, reverses only selected element slots for
the swap action, leaves non-element root content in its existing slots, and requires both members
of an authoritative bracket pair whenever either is selected.

In a native tab, Edit Atom Properties with Ferrum reuses the shared visual `AtomDialog`, but Qt
sends only a tuple of exact frozen property-change values for the selected durable atom. Rust
validates and applies the complete closed patch as one detached, revisioned operation. Unchanged
dialog defaults are not written to CDML; clearing an optional authored property removes it. Values
that the retained dialog cannot represent, including fractional values for an integer-only control,
are rejected with an error instead of being clamped or silently remapped. The ordinary legacy
property dock and legacy property actions remain outside this route.

Edit Text Properties likewise submits one closed tuple of frozen run and appearance changes.
Rust validates the complete rich-run grammar, durable target, and expected revision before it
changes the retained document. The current native form disables bold, italic, and font-family
controls because the verified renderer has only the regular Telex face. These options are shown
as unavailable instead of being approximated with a system font or accepted into an invisible
post-edit object.

Generate Molecule Coordinates sends one immutable revision- and digest-bound document observation
to the packaged Rust chemistry worker. RDKit receives a coordinate-free graph, so it cannot reuse
the old drawing. Ferrum then places the new atom-aligned coordinates at the molecule's existing
centroid and scales bonded molecules to their existing mean bond length. The UI-thread session
accepts all atom points as one history entry only if the source revision and digest are still
current. Bondless molecules retain their centroid without an invented scale. Molecules with
groups, query/text vertices, unsupported atom facts, or non-normal/unsupported bonds are rejected
instead of being partially converted.

The broader desktop application and remaining editing workflows are still under migration.

## Known gaps

- TODO: publish a supported consumer installer for the macOS arm64 ABI-4 FCM1 wheel.
- TODO: add bond-style and gesture-order/element choices, other object deletion,
  coordinate regeneration for non-ordinary graphs, and the remaining document classes before
  presenting the native route
  as a full editor.
