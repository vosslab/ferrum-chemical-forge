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

## Convert a molecule

The local build launcher supplies its local engine runtime to `convert` and
`coords`:

```bash
build/bin/ferrum convert aspirin.smi --to sdf_v2000 --output aspirin.sdf
```

`convert` accepts one source file or `-` for standard input. Its exact syntax
names are `smiles`, `inchi_standard`, `inchi_fixed_h`, `molblock_v2000`,
`molblock_v3000`, `sdf_v2000`, `sdf_v3000`, and `cdml`. CML is an input-only
profile: use its registry-owned `cml`, `cml1`, or `cml2` aliases. Ferrum
infers `.cml` alongside `.smi`/`.smiles`, `.inchi`, `.mol`/`.molblock`, `.sdf`,
and `.cdml`; use `--from` for standard input or another suffix. CML conversion
accepts only the documented simple-molecule CML/CML2 profile and returns a
typed refusal for unsupported XML or molecular features.

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

With two or more durable molecules selected, use `Export Selected Molecules as
SDF V2000...` or `Export Selected Molecules as SDF V3000...`. The dialog chooses
only the `.sdf` destination; Rust authenticates the selected membership and
document snapshot, establishes canonical record order, and provides the complete
file before Qt publishes it atomically. The established one-record SDF actions
remain available for their existing workflow.

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

Automation sends the existing `document.molecule.report.v1` request through
`build/bin/ferrum protocol run`; it is an operation-protocol route, not a
separate local CLI verb or named report command. Use
`snapshot { cdml, revision, digest_hex }` and one or more selected direct-root
molecule IDs. See
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md) for the exact request,
completed receipt, aggregate, and typed-refusal fields.

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

- `0`: a completed success or typed protocol refusal.
- `1`: input, processing, or confirmed publication failure.
- `2`: command-line usage error.
- `3`: a named output may have been published but Ferrum cannot confirm it.

Use `--json` when another program needs a stable discriminator. Test `schema`,
operation `kind`, and error `category`, not diagnostic text. The complete
request and response contract is in
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

## Fenced document commands

### Compact-group materialization

`document.compact-group.materialize.v1` materializes one attached direct-root
typed compact group from a caller-supplied fenced snapshot. It accepts exactly
`document { cdml, expected_revision, expected_digest_hex }`, opaque
`molecule_id`, and opaque `compact_group_id`; these identifiers are neither
labels nor recipes. Only typed `Me` and `NO2` groups materialize. The route
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

On success, `materialization` returns source revision and digest, source
molecule/group IDs, `replacement_focus_atom_id`, committed canonical
`document`, and its next `document_fence`. A typed refusal exposes only one
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

The generic protocol and named CLI route are delivered. PyO3 live-session
registration and the Qt `chemistry.expand_compact_group` action remain deferred.

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

## Current boundaries

Ferrum is pre-production. The verified desktop route is a bounded macOS arm64
CPython 3.12 route; it is not a cross-platform consumer release. The Rust CLI
and Qt application are both local build products. See [INSTALL.md](INSTALL.md)
for the local workflow and [PROVENANCE.md](PROVENANCE.md) for concise lineage
and licensing information.
