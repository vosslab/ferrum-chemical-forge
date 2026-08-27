# Use Ferrum

Ferrum provides a Rust `ferrum` command-line tool and a bounded `ferrum-qt`
drawing application. Build the local program first as described in
[INSTALL.md](INSTALL.md). The launchers below use the repo-owned runtime; no
Ferrum package installation is part of ordinary use.

## Use local build artifacts

Build the local CLI, Qt launcher, and native Python runtime:

```bash
./build.sh
```

Run either program directly from the build tree:

```bash
build/bin/ferrum --version
build/bin/ferrum-qt
```

The native extension is
`build/runtime/python/ferrum_chem<Python ABI suffix>`, with its local `.dylibs/`
closure. `build/bin/ferrum` resolves its sealed chemistry runtime only from the
sibling `build/runtime/engine-v1` directory. The local build workflow ends at
these runnable repository artifacts; it does not install or publish Ferrum
packages.

## Discover declared interchange formats

Use `formats` to inspect declared input and output formats, their operation eligibility,
and their resource limits before selecting a source or starting the local chemistry runtime:

```bash
build/bin/ferrum formats
build/bin/ferrum formats --json
```

The default prints a concise line-oriented projection for people. `--json`
writes the versioned `ferrum-interchange-capabilities-v1` catalog for tools.
Each catalog entry preserves independent input and output canonical/display
names, format/profile IDs, aliases, suffixes, limits, policy, and runtime
facts; the outer entry records only their resolver join. The command only
discovers declared contracts; it does not read a source, execute a conversion,
construct a document, or load the local chemistry runtime.

`inspect-graph` uses this same catalog to select a declared source decoder
before it reports a bounded read-only decoded-semantic graph.

### Import a simple CDXML molecule

`formats` reports CDXML as the canonical `.cdxml` input capability. Its output
is `none` in text mode and `null` in `formats --json`: Ferrum imports CDXML but
does not write or save it.

```bash
build/bin/ferrum formats
build/bin/ferrum open molecule.cdxml --format cdxml --output result.cdml
```

The supported profile imports unprefixed simple molecule fragments through the Rust
interchange registry. It records declared losses in canonical category order:
`lexical_syntax`, then `document_view_metadata`. It refuses unsupported chemistry before a
new document is published. See
[m2_cdxml_simple_molecule_import_v1.md](active_plans/decisions/m2_cdxml_simple_molecule_import_v1.md)
for the exact profile, limits, and exclusions.

In Ferrum Qt, use File > Open and select a `.cdxml` file offered by the descriptor-driven
filter. A successful import opens a conversion-only native tab with ChemDraw XML provenance;
its first Save or Save As writes CDML, not CDXML. A rejected file leaves the current document
unchanged. CDX binary files, namespaces, compressed input, and CDXML chemistry or presentation
outside the profile remain refused. `ferrum convert` refuses CDXML before reading it because
the descriptor makes it eligible only for `open`.

## Inspect a decoded graph

Inspect CML or SDF records without creating a document:

```bash
build/bin/ferrum inspect-graph molecule.cml --from cml
build/bin/ferrum inspect-graph records.sdf --from sdf --json
```

The shared summary reports zero-based `record_index`, exact graph counts, and
format-specific fact coverage and normalization. SDF titles are display facts,
not identifiers; SDF source IDs are unsupported, while retained title and
property-count facts are explicit. SDF coordinates, aromaticity, and stereo
are native-normalized decoded semantics, not raw-molfile fidelity claims.

## Convert a molecule

The local build launcher supplies its local engine runtime to `convert` and
`coords`:

```bash
build/bin/ferrum convert aspirin.smi --to sdf_v2000 --output aspirin.sdf
```

`convert` accepts one source file or `-` for standard input. Its exact syntax
names are `smiles`, `inchi_standard`, `inchi_fixed_h`, `molblock_v2000`,
`molblock_v3000`, `sdf_v2000`, `sdf_v3000`, and `cdml`. The bounded CML
interchange profile accepts `cml`, `cml1`, and `cml2` as input aliases; `cml`
and `cml2` are also canonical CML2 output aliases, while `cml1` is input-only.
Ferrum infers `.cml` alongside `.smi`/`.smiles`, `.inchi`, `.mol`/`.molblock`,
`.sdf`, and `.cdml`; use `--from` for standard input or another suffix.

```bash
build/bin/ferrum convert molecule.cml --to cml2 --output canonical.cml
```

Direct CML/CML2-to-CML conversion is pure Rust and retains validated molecule
and atom IDs and record order. Other source formats emit canonical CML2 only
when every admitted fact is losslessly representable; unsupported XML,
molecular facts, titles, properties, or coordinates return a typed refusal
without partial output. This is bounded external interchange, not a Ferrum
document format: CDML remains the sole document, session, history, and Qt-local
format. Qt File > Open immediately admits valid CML/CML2 as a clean native CDML
tab; the desktop application has no CML export route.

```bash
printf 'CCO\n' | build/bin/ferrum convert - --from smiles --to molblock_v2000 > ethanol.mol
build/bin/ferrum convert input.mol --from molblock_v2000 --to smiles --output output.smi
build/bin/ferrum convert molecule.cml --to smiles --output molecule.smi
```

If the local runtime cannot supply chemistry, engine verbs finish with the
typed `chemistry_unavailable` refusal. Rebuild with `./build.sh`; the
application does not search for an adapter in a per-user installation, the
current directory, `PATH`, Python installations, or environment variables.

## Export selected SDF

Export two or more authored direct-root molecules from one CDML document as an
atomic multi-record SDF file. Repeat `--molecule-id` for each selected root and
choose either V2000 or V3000 syntax:

```bash
build/bin/ferrum document export-sdf --input drawing.cdml \
  --molecule-id root-a --molecule-id root-b \
  --version v3000 --output selected.sdf
```

The command resolves the authored source IDs against one revision/digest-fenced
Rust document observation. Rust defines the record order; command-line argument
order does not. Ferrum publishes `--output` only after every selected record is
available, so a refusal leaves an existing destination unchanged.

## Render a drawing

Render one supported complete CDML document as SVG, PDF, or transparent PNG:

```bash
build/bin/ferrum render drawing.cdml --output drawing.svg
build/bin/ferrum render drawing.cdml --to pdf --output drawing.pdf
build/bin/ferrum render drawing.cdml --to png --output drawing.png
```

Ferrum infers a named artifact format from `.svg`, `.pdf`, or `.png`; use
`--to` for standard output or an unfamiliar suffix. SVG and PDF are vector
artifacts. PNG uses one output pixel per Rust page point with transparency;
that is a page-geometry rule, not a print-DPI promise.

## Draw with the keyboard

Start a new window or open an uncompressed CDML drawing:

```bash
build/bin/ferrum-qt
build/bin/ferrum-qt drawing.cdml
```

For a keyboard-only small drawing task, activate File > Open with the platform
Open shortcut, choose a CDML document, then use these commands while canvas
focus is active:

1. Press `Ctrl+8` for Add Atom. Arrow keys move the crosshair by one grid step;
   `Shift+Arrow` makes a fine move. Press `Enter` to place an atom.
2. Press `Ctrl+2` for Draw Bond. Press `Enter` on the first atom, move to the
   second atom, and press `Enter` again to commit a bond. Press `Escape` to
   cancel without changing the document.
3. Use the platform Undo shortcut to reverse the last change, then use the
   platform Save shortcut to save or Save As to choose a new CDML path.

Pointer editing remains available. The bounded desktop route supports Rust-owned
atom and normal bond edits, selected molecule work, supported insertions,
coordinate work, Undo/Redo, CDML save, and SVG/PDF/PNG export. Unsupported
document features or formats are refused with next-step guidance and do not
alter the active document. See [FILE_FORMATS.md](FILE_FORMATS.md) for admitted
files and publication rules.

## Insert a regular ring

Choose Edit > `Insert Regular Ring...`, select Cyclopropane through
Cyclooctane, then release once at an empty canvas location. The existing
`Insert Cyclohexane Ring` command remains a C6 shortcut to that same action.
Rust accepts only the closed C3-C8 family and owns the
`DocumentOperationV1` transaction, ring topology and geometry, durable CDML
facts, renderer admission, history, and Undo/Redo. Escape, an occupied
location, or any typed refusal leaves the document unchanged. Escape disarms
the chooser; an occupied location leaves it ready for a new empty location.

With two or more durable molecules selected, use `Export Selected Molecules as
SDF V2000...` or `Export Selected Molecules as SDF V3000...`. The dialog chooses
only the `.sdf` destination; Rust authenticates the selected membership and
document snapshot, establishes canonical record order, and provides the complete
file before Qt publishes it atomically. The established one-record SDF actions
remain available for their existing workflow.

## Reverse a selected wedge direction

Choose Edit > `Reverse Selected Wedge Direction` when exactly one selected
direct-molecule bond is a solid `w1` or hashed `h1` wedge. The command is
disabled for no selection, multiple selections, normal bonds, and every other
bond style. It preserves the selected bond's durable document-object ID,
unordered connectivity, and wedge type while reversing only the ordered CDML
endpoints; the same durable bond remains selected after the accepted change.

Selection and mutation use deliberately different identities. The renderer
issues the durable object ID used to select the bond. Qt passes the current
projected source ID only to construct the one Rust operation; it does not use a
source ID to rediscover or select a bond. Rust validates the fenced request,
swaps the eligible endpoints in its detached candidate, reparses and admits the
candidate, then owns history, Undo/Redo, CDML persistence, and typed atomic
refusals.

A visible wedge is selectable across its Rust-derived rendered envelope, not
only on an invisible bond centerline. The observer publishes one semantic Bond
target from the lowered wedge bounds, with the shared pointer tolerance; it
does not retain a structural child `DisplayOnly` target for the same bond. The
ordinary Qt selection refresh is coalesced by a single-shot timer owned by the
tab widget it queries, so a queued refresh cannot outlive that tab host.

## Create and inspect reactions

Select the complete roots that make up a reaction, then choose Chemistry >
`Create Reaction...`. In the modeless `Define Reaction` panel, assign the
Rust-projected roles for reactants, products, the arrow, optional pluses, and
optional condition text, then select `Create Reaction`. Ferrum refuses an
incomplete or stale definition without changing the document and directs the
user to refresh the selection when needed.

Choose Chemistry > `Reaction Inspector` to review the current Rust-issued
definitions. The inspector can highlight a definition's members, edit its
roles, delete only the definition while retaining its member roots, or nudge
all members together with an optional view-hex-grid snap. Each action refreshes
from current Rust observations; typed refusals leave the document unchanged,
and the platform Undo shortcut restores an accepted deletion or movement.

## Place, attach, delete, and materialize compact groups

Interactive compact-group authoring is a deliberately narrow Qt workflow.
Free placement and attachment are separate operations:

1. Choose Draw > Compact groups > `Place Compact Group...`.
2. Select `Me` and release once on the canvas.
3. Ferrum creates one direct-root compact group. It is initially represented as
   zero atoms and zero bonds.

For an attached group:

1. Select one eligible atom on the canvas.
2. Choose Draw > Compact groups > `Attach Compact Group...`.
3. Select the Rust-projected `Me`, `NO2`, `Et`, `OMe`, `CH2OH`, `Carboxyl`,
   `Cyano`, `AcylChloride`, or `Phenyl` choice, then choose
   `Attach to Selected Atom`.
4. Release once on the canvas to supply the attachment direction.
5. Explicitly select the resulting compact-group label.
6. Choose Chemistry > `Materialize Selected Compact Group`.
7. Choose Chemistry > `Molecule Report...` to inspect the materialized result.

Qt maps the canvas release through its current view-snapping policy and sends
the resulting finite coordinates through the private PyO3 seam. Rust validates
the typed `Point3V1` and candidate geometry, then owns the anchor, canonical
orientation, durable IDs, renderer admission, and history transition. Rust
cannot establish that a coordinate originated from Qt's view-snapping policy;
that policy remains a Qt responsibility. The delivered public surface admits
only `Me` for free placement and `Me`, `NO2`, `Et`, `OMe`, `CH2OH`, `Carboxyl`,
`Cyano`, `AcylChloride`, or `Phenyl` for attached authoring. `NO2`
materializes as `R-[N+](=O)[O-]`; Rust preserves its atom formal charges through
history and reopen, while Molecule Report exposes the supported net formal
charge. `Et` materializes as two neutral carbons joined by one normal single
bond. `OMe` materializes as neutral `R-O-CH3`; the exterior bond and returned
focus are oxygen.

### Attach a compact group by CLI

The public stateless operation is `document.compact-group.attach.v1`. It is
available through both existing transports:

```bash
build/bin/ferrum protocol run request.json
build/bin/ferrum document command document.compact-group.attach.v1 request.json
```

The request contains a fenced CDML snapshot, its molecule and anchor durable-ID
pair, one closed catalog key, and finite release coordinates. Rust creates a
short-lived document session, prepares the candidate, then commits it
immediately or drops it without mutation. The request shape is defined in
[ferrum-operation-v1.schema.json](../packages/ferrum-rust/crates/api/protocol/ferrum-operation-v1.schema.json).

On success, the receipt preserves the source fence, echoes the target and
catalog key, allocates a compact-group ID, returns committed CDML, and supplies
a reusable next document fence. It intentionally omits the release intent,
candidate pose, renderer overlay values, and pending/session capability facts.
Clients use the returned document and fence for a subsequent stateless request.
Typed refusals preserve the document and provide stable category and recovery
facts.

Both attachment transports write exactly one versioned envelope and exit `0`
after either an accepted outcome or a typed refusal. Nonzero status is reserved
for command usage, request input, transport, or output/publication failure.

Materialization is a separate delivered operation. For a sole direct-root
compact group with zero atoms and zero bonds, it replaces that group in the
same molecule with the immutable recipe atoms and bonds; it does not rewrite
another root or change attached-group topology. Methyl materializes to one
explicit carbon, and a loaded sole-root Ethyl group materializes to its two
explicit carbons. The replacement is one history transition and survives
Undo, Redo, and reopen.

If exactly one selected atom has an exact-current unavailable result for a
reviewed choice, the existing action can present a label-derived refusal. For `Me`,
the existing `Attach Compact Group...` action remains enabled. Activating it
opens Ferrum's standard accessible `Action Not Available` dialog with the
visible message `Me cannot attach to the selected atom. Select another atom and
try again.` Dismiss the dialog, select an eligible atom in the same document,
and use the same action to open the chooser. Stale, missing, or nonmatching
availability facts keep the action disabled with generic readiness guidance.
The guarded chooser and Rust's typed refusal remain the authority; this does
not create a fallback route.

To delete an attached compact group, use the existing Select Structure tool to
select exactly one visible compact-group label, then press `Delete` or
`Backspace`. Qt forwards only the renderer-issued parent molecule and
compact-group durable IDs. Rust verifies direct membership, removes exactly the
group and its unique exterior bond through one history transition. Its public
receipt reports removed atom, bond, and compact-group counts. Document-private
`PersistentId` values remain internal. Mixed or multi-group selection is refused
before preparation. Use the platform Undo shortcut to restore the group; Redo
reapplies the same committed deletion.

## Molecule diagnostics

Select one or more direct-root molecules and use `Molecule Report...` to inspect
their read-only Ferrum report. The report uses the selected IDs and the current
public snapshot, then returns a completed receipt or typed refusal.
The returned root records follow source order; each record's findings use the
deterministic report-category order for text, capacity, groups, zero-order, and
graph/composition facts. The aggregate composition is complete or omitted. The
command does not alter the document, selection, history, renderer state, or
authored molecule name.

Each returned molecule keeps chemical configuration and drawing evidence
separate: `stereo_semantics` carries durable tetrahedral and E/Z configuration,
while Rust-issued `stereo_depiction` carries editable directed-bond and E/Z
carrier-mark facts. Qt displays those facts and never derives configuration from
a mark or coordinates, or invents a mark from configuration.

Automation can send the same `document.molecule.report.v1` request through
either the generic protocol runner or its named positional adapter:

```bash
build/bin/ferrum protocol run report-request.json
build/bin/ferrum document command document.molecule.report.v1 report-request.json
```

Both forms use one generic request and return the same typed envelope. Use
`snapshot { cdml, revision, digest_hex }` and one or more selected direct-root
molecule IDs. See
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md) for the exact request,
completed receipt, aggregate, and typed-refusal fields.

## Check structure diagnostics

`Check Structure...` is a distinct, runtime-free read-only operation, not a
Molecule Report variant. Select one or more direct-root molecules and choose
Chemistry > `Check Structure...`. Ferrum captures the current CDML, revision,
digest, and durable selected-root IDs, then returns deterministic structural
findings in an accessible modeless dialog. It never changes the document,
history, selection, canvas, or current navigation.

Automation can use the named local CLI operation:

```bash
build/bin/ferrum document command document.molecule.diagnostics.v1 diagnostics-request.json
```

The request contains `snapshot { cdml, revision, digest_hex }` and selected
durable direct-root `molecule_ids`. It accepts at most 128 IDs and 2 KiB of
selector bytes; exceeding either bound returns a typed resource refusal, with
no partial result. The findings are deterministic and bounded. Missing
authored `formal_charge` remains an unknown source fact, not a delivered
chemical defect; `IncompleteAuthoredCharge` is reserved for later work.

For example, an attached unexpanded `Me` group is reported with recovery
guidance to use the existing materialization action. Diagnostics neither
materialize the group nor auto-fix any finding. See
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md) for the closed request and
receipt boundary.

## Six V1 executor verbs

The six general-purpose commands below create one V1 operation request and use
the same Rust executor. `document export-sdf` is a separate document-export
route described above.

- `build/bin/ferrum inspect INPUT [--json]` prints a semantic CDML inspection report.
- `build/bin/ferrum validate INPUT [--level structural|typed] [--json]` prints validation facts.
- `build/bin/ferrum rewrite INPUT [-o OUTPUT] [--json]` writes structurally preserved CDML.
- `build/bin/ferrum render INPUT [-o OUTPUT] [--to svg|pdf|png] [--json]` writes one complete artifact.
- `build/bin/ferrum convert INPUT [--from FORMAT] --to FORMAT [-o OUTPUT] [--json]` converts one bounded molecular-interchange source through the local engine.
- `build/bin/ferrum coords DOCUMENT [-o OUTPUT] [--json]` regenerates all direct molecule coordinates through the local engine.

`inspect` and `validate` report to standard output. `rewrite`, `render`,
`convert`, and `coords` write raw completed results to standard output when
`--output` is omitted or `-`. `--json` instead writes the complete operation
envelope and cannot be combined with a named output destination.

Named outputs use safe publication. Ferrum refuses to replace its retained input
source or an observed hard-link alias. A successful rewrite may normalize
serialization details: it preserves structure, not bytes. A render result is
complete for its requested profile; it does not claim pixel equivalence to
another renderer.

## Results and failures

Human diagnostics go to standard error. Exit statuses are:

- `0`: a completed success.
- `1`: an input, processing, or confirmed publication failure.
- `2`: command-line usage error.
- `3`: a named output may have been published but Ferrum cannot confirm it.

Use `--json` when another program needs a stable discriminator. Test `schema`,
operation `kind`, and error `category`, not diagnostic text. The complete
request and response contract is in
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

Human-oriented commands emit one standard-error diagnostic for an unsuccessful
outcome, then exit `1`. A `--json` command emits one protocol envelope and an
unsuccessful envelope also exits `1`, without a second diagnostic. The
delivered `document.compact-group.attach.v1` route is the explicit typed-
envelope exception described above: its accepted and refused outcomes both
exit `0`.

## Fenced document commands

### Compact-group materialization

`document.compact-group.materialize.v1` materializes one typed compact group in
a direct-root molecule from a caller-supplied fenced snapshot. It accepts exactly
`document { cdml, expected_revision, expected_digest_hex }`, opaque
`molecule_id`, and opaque `compact_group_id`; Rust parses both as durable
`DocumentObjectIdV1` selectors, and they are neither labels nor recipes. Only
typed `Me`, `NO2`, `Et`, `OMe`, `CH2OH`, `Carboxyl`, `Cyano`, `AcylChloride`,
and `Phenyl` groups materialize. The route
accepts no free-form labels or recipes and has no legacy alias.

```json
{
  "schema": "ferrum-operation-request-v1",
  "request_id": "caller-chosen-opaque-id",
  "operation": {
    "kind": "document.compact-group.materialize.v1",
    "document": {
      "cdml": "canonical-cdml-text",
      "expected_revision": 0,
      "expected_digest_hex": "lowercase-sha256-digest"
    },
    "molecule_id": "opaque-rust-issued-id",
    "compact_group_id": "opaque-rust-issued-id"
  }
}
```

On success, `materialization` returns the fenced revision and digest, durable
molecule/group selectors, a durable `replacement_focus_atom_id`, committed
canonical `document`, and its next `document_fence`. A typed refusal exposes only one
closed `compact_group_materialization_refusal { category, recovery }` pair:
`stale_document_fence`/`refresh_and_retry`,
`unknown_or_foreign_target`/`correct_target`,
`ineligible_target`/`choose_eligible_target`,
`renderer_preparation_refusal`/`document_unchanged`, or
`session_conflict_or_replayed_preparation`/`refresh_and_retry`.

```bash
build/bin/ferrum protocol run compact-materialize.json
build/bin/ferrum document command document.compact-group.materialize.v1 compact-materialize.json
```

The generic protocol and named CLI route are delivered. They are stateless:
the request carries durable selectors and the admitted CDML snapshot fence.

In `ferrum-qt`, select one visible `Me`, `NO2`, `Et`, `OMe`, `CH2OH`, `Carboxyl`,
`Cyano`, `AcylChloride`, or `Phenyl` compact-group label and use Chemistry >
`Materialize Selected Compact Group`. The action becomes available
only when the Rust live session reports the selected durable molecule/group
pair as eligible for the installed revision/digest fence. On success Ferrum
installs the returned observation and selects Rust's replacement focus atom.
Undo, Redo, and reopen create a current observation and therefore require a
fresh availability result; the UI does not retain a chemistry decision, raw
CDML, or a source-ID substitute for the live durable address. A sole free `Me`
root becomes one explicit-carbon molecule; an attached group retains its
existing exterior topology while its group is replaced by the immutable recipe
graph.

`inspect --json` returns a `document_fence`. Use its revision and digest with the same input
document in a later request-owned document command. This local workflow authors one vector:

```bash
build/bin/ferrum inspect drawing.cdml --json > inspect.json
jq -n --rawfile document drawing.cdml --slurpfile inspected inspect.json \
  '{kind: "presentation.author.v1", document: $document,
    expected_revision: $inspected[0].outcome.document_fence.expected_revision,
    expected_digest_hex: $inspected[0].outcome.document_fence.expected_digest_hex,
    authoring: {kind: "vector", vector_kind: "line",
      start: {x: 0.0, y: 0.0}, end: {x: 40.0, y: 0.0},
      appearance_policy: "effective_drawing_standard"}}' > author.json
build/bin/ferrum document command presentation.author.v1 author.json > author-result.json
```

The generic author result is defined by the generated
[ferrum-operation-v1.schema.json](../packages/ferrum-rust/crates/api/protocol/ferrum-operation-v1.schema.json).
Use its returned document for a subsequent stateless request. Do not depend on internal
authoring capabilities, reservations, or live-session state.

This equivalent local workflow inserts a catalog entry at a finite document coordinate:

```bash
build/bin/ferrum inspect drawing.cdml --json > inspect.json
jq -n --rawfile document drawing.cdml --slurpfile inspected inspect.json \
  '{kind: "catalog.insert.v1", document: $document,
    expected_revision: $inspected[0].outcome.document_fence.expected_revision,
    expected_digest_hex: $inspected[0].outcome.document_fence.expected_digest_hex,
    catalog_id: "system/rings/benzene", anchor_x: 100.0, anchor_y: 50.0}' > catalog.json
build/bin/ferrum document command catalog.insert.v1 catalog.json > catalog-result.json
```

A stale catalog fence returns no success outcome. Its nested facts have this shape:

```json
{
  "error": {
    "catalog_placement_refusal": {
      "category": "stale_snapshot",
      "recovery": "refresh_and_restart"
    }
  }
}
```

Refresh with `inspect --json`, rebuild the operation JSON from the returned fence, and retry.

### Atom oxidation observation

`document.atom.oxidation.observe.v1` reads one selected durable atom from one direct-root
molecule without changing the document. Build one complete fenced protocol request from the
current CDML, its `document_fence`, and durable molecule and atom identifiers from the current
document observation. Do not reuse a fence or identifier after changing the source document.

```json
{
  "schema": "ferrum-operation-request-v1",
  "request_id": "local-oxidation-observation",
  "operation": {
    "kind": "document.atom.oxidation.observe.v1",
    "document": {
      "cdml": "<current CDML text>",
      "expected_revision": 0,
      "expected_digest_hex": "current-lowercase-sha256"
    },
    "molecule_id": "current-direct-root-id",
    "atom_id": "current-atom-id"
  }
}
```

Use the named local CLI route with a request file or standard input:

```bash
build/bin/ferrum document-atom-oxidation-observe --request oxidation.json
cat oxidation.json | build/bin/ferrum document-atom-oxidation-observe --request -
```

The response is one canonical protocol envelope. An accepted observation contains a signed
oxidation number under the `formal-electron-assignment-hcno-v1` convention. A completed
`unavailable` observation contains one closed reason instead of a number when the whole root is
outside the materialized H/C/N/O profile. V1 requires each hydrogen to be an explicit H atom
vertex and every atom to record an authored explicit-hydrogen fact of zero; implicit, omitted, or
aggregate hydrogen representation completes as `hydrogen_topology_unsupported`. A stale source
or invalid durable address is a typed refusal, so refresh the document and create a fresh request
rather than interpreting it as an unavailable chemistry result. See
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md) for exact fields and refusal categories.

## Machine protocol

The lower-level protocol command accepts one UTF-8 JSON request and emits one
JSON success or typed error envelope:

```bash
build/bin/ferrum protocol schema
build/bin/ferrum protocol run request.json
build/bin/ferrum protocol run request.json --output response.json
```

Protocol payloads contain document or interchange text, never paths. It has no
batch, network, session, Qt, or adapter-discovery capability. The generated
schema and precise operation envelopes are specified in
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

## Inspect decoded CML and SDF graphs

Use `ferrum inspect-graph molecule.cml --from cml` or `ferrum inspect-graph records.sdf --from sdf` to report ordered decoded-semantic record, atom, and bond counts without creating a document. `--from` is required; `cml`, `cml1`, and `cml2` select the runtime-free CML profile, while SDF uses the declared native semantic profile. Add `--json` for the complete versioned protocol envelope. CML coordinates remain CML-decoded; SDF coordinates, aromaticity, stereo, and direction are native-normalized semantics rather than raw-molfile claims.

Ferrum is pre-production. The verified desktop route is a bounded macOS arm64
CPython 3.12 route; it is not a cross-platform consumer release. The Rust CLI
and Qt application are both local build products. See [INSTALL.md](INSTALL.md)
for the local workflow and [PROVENANCE.md](PROVENANCE.md) for concise lineage
and licensing information.
## Place a free compact group

Choose Draw > Compact groups > **Place Compact Group...**, select **Me**, then release once on the canvas. Ferrum creates a new molecule root containing the methyl compact group at the snapped canvas position. This is a direct-root group, so it has no atoms or bonds until a later supported materialization workflow.

The chooser currently offers only `Me`. Attached compact groups, templates, arbitrary orientation, dragging, batch placement, raw CDML authoring, and command-line placement are separate capabilities and are not implied by this action.
## Command palette

Use **View > Commands > Command Palette...** to search Ferrum's currently live
commands. The portable shortcut is `Ctrl+K`; Qt displays it as `Cmd+K` on
native macOS. Search matches each action's label, help text, and stable ID.

The palette keeps unavailable commands visible and explains why they cannot run.
It triggers the exact selected action only while that action remains enabled.
Keep typing in the search field while using bare Up and Down to change the
selection; Return runs the selected command and Escape closes the palette and
returns focus to the invoking control. Modified arrows retain ordinary text
field behavior.

Ferrum derives palette content from the live action registry, and
`resources/menus.yaml` remains the authority for its View-menu placement.
