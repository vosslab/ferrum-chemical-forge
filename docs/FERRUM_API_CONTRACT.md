# Ferrum API contract

This reference defines the stateless Ferrum operation-protocol V1 contract. It is the machine
interface behind the human CLI verbs; task-oriented examples are in [USAGE.md](USAGE.md).

## Transport and envelope

`ferrum protocol run` reads one UTF-8 JSON request from a named file or standard input. Its request
schema is `ferrum-operation-request-v1`:

```json
{
  "schema": "ferrum-operation-request-v1",
  "request_id": "caller-chosen-opaque-id",
  "operation": {
    "kind": "document.inspect",
    "document": "<cdml xmlns=\"urn:ferrum:cdml\"/>"
  }
}
```

`request_id` is returned unchanged after admission but has no identity, ordering, persistence, or
authorization meaning. An admitted decodable request returns either
`ferrum-operation-response-v1` or `ferrum-operation-error-v1`. Clients must discriminate on schema,
operation `kind`, and error `category`, never on a diagnostic `message`.

The protocol has a derived UTF-8 transport limit before allocation and JSON parsing. Invalid JSON or
an over-budget transport produces no response envelope. Existing CDML admission and complete-artifact
bounds apply independently after transport admission.

## Operations

The V1 operation set is closed:

| Operation kind | Request fields | Successful outcome |
| --- | --- | --- |
| `document.inspect` | `document` CDML text | `report`, `document_fence` |
| `document.validate` | `document`, `level`: `structural` or `typed` | `level`, `report` |
| `document.rewrite` | `document` | structural `document`, `report` |
| `document.render_artifact` | `document`, `format` | `format`, `media_type`, complete base64 `artifact_base64` |
| `chemistry.convert` | `input.format`, `input.text`, `output_format` | `format`, converted `text`, `record_count` |
| `interchange.inspect_graph.v1` | `input.format`, `input.text` | decoded-semantic graph summary with profile, record facts, counts, coverage, and normalization |
| `document.generate_coordinates` | `document` | structural `document`, `regenerated_molecule_count` |
| `presentation.author.v1` | `document`, `expected_revision`, `expected_digest_hex`, typed `authoring` | schema-defined committed document and durable root outcome |
| `catalog.list.v1` | no document input | immutable catalog schema, version, and entry summaries |
| `catalog.insert.v1` | `document`, revision/digest fence, catalog ID, finite anchor | changed `document`, `identifier`, `committed_revision`, `document_fence` |
| `document.molecule.report.v1` | `snapshot { cdml, revision, digest_hex }`, one or more durable direct-root `molecule_ids` | source receipt, source-ordered root records, complete-or-omitted aggregate, deterministic structured findings |
| `document.molecule.diagnostics.v1` | `snapshot { cdml, revision, digest_hex }`, up to 128 durable direct-root `molecule_ids` whose selector bytes total at most 2 KiB | bounded deterministic structural findings or a typed refusal |
| `document.molecule.smarts.query.v1` | admitted document and bounded raw or selected SMARTS query | bounded, non-redeemable query summary |
| `document.atom.oxidation.observe.v1` | fenced `document`, durable direct-root `molecule_id`, durable `atom_id` | one fenced accepted oxidation number or closed unavailable reason |
| `document.compact-group.materialize.v1` | fenced `document`, opaque direct-root `molecule_id`, opaque `compact_group_id` | source receipt, committed document, next fence, replacement focus |

The closed interchange format names are `smiles`, `inchi_standard`, `inchi_fixed_h`,
`molblock_v2000`, `molblock_v3000`, `sdf_v2000`, `sdf_v3000`, and `cdml`. Render formats are `svg`,
`pdf`, and `png_one_pixel_per_point_transparent`.

`chemistry.convert` and `document.generate_coordinates` need an out-of-band trusted chemistry
runtime. Their JSON carries only owned text and closed names: never a filesystem path, library
handle, session, or adapter locator. The default executor returns `chemistry_unavailable` when that
capability is absent.

`interchange.inspect_graph.v1` accepts owned CML or SDF text and returns one
bounded decoded-semantic summary without constructing a document. The CML
profile is runtime-free; the SDF profile uses the trusted native runtime and
discloses native normalization rather than raw-source fidelity. The response
reports exact checked counts, zero-based record order, typed record facts, and
the profile's complete fact coverage. JSON mode emits one versioned success or
typed error envelope. Human mode emits the line-oriented summary on success or
one standard-error diagnostic on refusal. Complete output is admitted before
publication, so a response-limit refusal cannot append to partial success
bytes.

`document.inspect` admits one snapshot and returns its canonical
`document_fence { expected_revision, expected_digest_hex }` with the report. A caller can carry
that fence into a later request-owned mutation; the human CLI report remains read-only.

`catalog.insert.v1` performs one complete request-owned catalog insertion. On success, it returns
only the changed canonical `document`, created `identifier`, observed `committed_revision`, and a
portable `document_fence` for that returned document. The fence has revision zero because the next
stateless request admits returned CDML into a fresh session; its digest is derived from that exact
returned text. A stale fence is a typed refusal with no success outcome, so a caller retries from
the returned document and fence rather than interpreting partial catalog result data.

`presentation.author.v1` performs one complete request-owned mutation. Its closed authoring
families are Vector, terminal Electron/Retro/Normal arrow, Curved Equilibrium arrow,
Polyline/Polygon path, and explicit-endpoint DirectBond. A request contains finite serializable
geometry and, for DirectBond, durable atom IDs or finite new-atom points. It contains no Qt
pointer, viewport, hit-test, preview, session, capability, or reservation object. The adapter
creates and redeems those short-lived authorities internally, returning only the accepted CDML
snapshot and durable result facts.

`document.molecule.smarts.query.v1` reports bounded source-order match facts for one admitted
document snapshot. Its summary contains no match membership, atom identity, query text, native
state, geometry, or live reveal capability. A response whose canonical public envelope exceeds
the fixed 1 MiB budget is replaced before delivery with the typed
`resource_limit` / `response_size_exceeded` refusal. It carries no partial rows or query result.
The generated [ferrum-operation-v1.schema.json](../packages/ferrum-rust/crates/api/protocol/ferrum-operation-v1.schema.json)
defines the current request and response fields for the closed operation set.

### Molecule report

`document.molecule.report.v1` reads one or more selected durable direct-root
molecules from a caller-supplied `snapshot { cdml, revision, digest_hex }` and
`molecule_ids`. The revision is delivery provenance. The report evaluates that
snapshot without changing the caller's CDML, document revision, history,
selection, renderer state, or authored molecule facts.

The completed receipt preserves the source revision and verified digest. Its
root records follow the source order of the selected direct roots, and its
aggregate is complete or omitted. Within each record, findings use the stable
report-category order: text, capacity, groups, zero-order, then existing graph
and composition categories. Findings are not a globally source-ordered stream.
Each finding has a severity, closed code, recovery category, typed semantic
location, and nullable detail. An unaddressable source location remains a typed
report outcome; clients use its code, recovery category, and location rather
than parsing diagnostic text. Unknown or non-direct roots and an invalid
snapshot use the ordinary typed error envelope; they do not promise structured
recovery guidance. The report opts into the shared final-envelope response
budget. An over-budget response is refused rather than delivered partially and
is the response-budget case that carries structured recovery for reducing the
request.

The named local adapter is `ferrum document command
document.molecule.report.v1 <input>`. It reads the same complete operation
request as `ferrum protocol run` and returns the same typed envelope; it adds no
report-specific request schema, executor, or result translation.

Each root record has a nullable `stereo_semantics` descriptor. When present, its
tetrahedral descriptors are ordered by ascending center position and its E/Z
descriptors by ascending bond position. Tetrahedral ligands are exactly four
entries: atom positions are strictly ascending, distinct neighbors of the center,
or the tagged explicit-hydrogen sentinel as the fourth entry for a center with
exactly one explicit hydrogen. E/Z ligands are distinct from each other and the
double-bond endpoints, and each neighbors its corresponding endpoint. All
positions are zero-based molecular source indices. These are durable molecular
facts, separate from directional wedge/hash drawing presentation. The checked-in
schema is generated directly from the Rust DTOs and defines the exact closed JSON
grammar, including the nullable field, fixed tetrahedral cardinality, and
descriptor vocabularies:
[ferrum-operation-v1.schema.json](../packages/ferrum-rust/crates/api/protocol/ferrum-operation-v1.schema.json).

Each root record also has a required nullable `stereo_depiction` descriptor.
`null` is the only absence state. When present, `directed_bonds` preserves the
source-order bond index, endpoint direction, and closed `solid_wedge` or
`hashed_wedge` drawing presentation for tetrahedral depiction. Its
`double_bond_carrier_marks` preserve `double_bond_index`, `carrier_bond_index`,
and closed `up` or `down` marks for E/Z drawing. These are Rust-issued drawing
facts: clients display them but never derive an E/Z configuration from a mark,
or manufacture a mark from configuration or coordinates. Chemical meaning
remains exclusively in `stereo_semantics`.

### Molecule diagnostics

`document.molecule.diagnostics.v1` is separate from
`document.molecule.report.v1`. It reads a fenced immutable
`snapshot { cdml, revision, digest_hex }` and durable selected direct-root
`molecule_ids`, then produces only deterministic, bounded structural findings.
At most 128 selectors and 2 KiB of selector bytes are admitted. An over-limit
request uses the typed resource-refusal path; it is neither truncated nor
partially executed.

The operation is runtime-free and read-only: it never requests chemistry
runtime work or changes the snapshot, document session, history, renderer,
selection, or navigation. Findings carry Rust-owned codes, severity, locations,
and recovery data. Missing authored `formal_charge` is intentional unknown
source state. `IncompleteAuthoredCharge` is reserved and is not a V1 result.

The named CLI form is `ferrum document command
document.molecule.diagnostics.v1 <input>`. Qt captures owned snapshot values
and durable selected-root IDs on its UI thread; a detached worker invokes the
module-level owned-snapshot PyO3 executor, never a session-bound method. On
delivery, Qt authenticates the current tab, fence, and selected roots before
showing its accessible modeless read-only dialog. It does not auto-fix,
materialize, navigate the canvas, or alter selection.

### Atom oxidation observation

`document.atom.oxidation.observe.v1` reads one selected durable atom in one durable direct-root
molecule from a caller-supplied, revision-and-digest-fenced CDML snapshot. Its operation payload
contains exactly `document { cdml, expected_revision, expected_digest_hex }`, `molecule_id`, and
`atom_id`. It never changes CDML, document revision, history, selection, renderer state, or atom
marks.

The completed `observation` always repeats its source revision and digest, molecule and atom IDs,
and document-root order. It uses the closed convention
`formal-electron-assignment-hcno-v1`: the whole direct root must be a fully materialized H/C/N/O
graph with explicit formal charges and explicit hydrogen topology. For this V1 profile, each
hydrogen is an explicit H atom vertex, and every atom records an authored explicit-hydrogen fact
of zero. Implicit, omitted, or aggregate hydrogen representation is unavailable as
`hydrogen_topology_unsupported`. An accepted result has
`status: "accepted"` and one signed `oxidation_number`. An unsupported whole-root fact instead
has `status: "unavailable"` and exactly one closed `unavailable_reason`, with no number. The
closed reasons are `element_outside_profile`, `formal_charge_unavailable`,
`hydrogen_topology_unsupported`, `aromaticity_unsupported`, `radical_unsupported`,
`bond_order_unavailable`, `bond_order_unsupported`, `non_atom_vertex_unsupported`,
`coordination_or_delocalization_unsupported`, `component_invariant_failed`, and
`arithmetic_overflow`.

An unavailable result is a completed observation, not a refusal. A stale fence, unknown atom,
non-direct molecule, atom/root mismatch, unsupported document, or bounded-resource refusal uses
the ordinary typed error envelope instead. Clients must use the error category and recovery facts,
not diagnostic text, to decide whether to refresh the source, select another atom, or reduce the
request.

### Compact-group materialization

`document.compact-group.materialize.v1` is a stateless generic operation for
one eligible typed compact group in a direct-root molecule. It supports an
attached group and a self-contained free group. Its payload contains exactly
`document { cdml, expected_revision, expected_digest_hex }`, `molecule_id`, and
`compact_group_id`. The document is a revision/digest-fenced snapshot and both
target identifiers are caller-supplied serialized `DocumentObjectIdV1` values.
Rust parses and resolves them in the admitted snapshot. They are opaque durable
document-object selectors, not labels, catalog keys, formulae, paths, geometry,
or recipe input.

The successful `materialization` receipt has schema
`ferrum-document-compact-group-materialization-v1`. It repeats the fenced
revision, digest, molecule ID, and compact-group ID, then returns the committed
canonical `document`, its next request-owned `document_fence`, and the
authoritative durable `replacement_focus_atom_id` in that committed snapshot.
Preparation state and generated replacement IDs remain session-private.

The error envelope may carry exactly one
`compact_group_materialization_refusal { category, recovery }` pair. The closed
pairs are `stale_document_fence` / `refresh_and_retry`,
`unknown_or_foreign_target` / `correct_target`, `ineligible_target` /
`choose_eligible_target`, `renderer_preparation_refusal` /
`document_unchanged`, and `session_conflict_or_replayed_preparation` /
`refresh_and_retry`. They expose no source CDML, candidate, or recipe.

Only typed `Me`, `NO2`, `Et`, `OMe`, and `CH2OH` compact groups materialize in this public route. It
does not accept free-form labels, formulas, recipes, or a legacy alias. The
generic protocol, named CLI command, PyO3 live-session route, and Qt action
are delivered. Both stateless and live routes use durable
`DocumentObjectIdV1` molecule and compact-group selectors with their applicable
revision/digest fence; neither reconstructs a target from source data.

The live session also issues a closed, fenced compact-materialization
availability observation for a selected durable molecule/group pair. Qt uses
only `Eligible` from that observation to enable `Materialize Selected Compact
Group`; the operation revalidates the same session-owned eligibility during
preparation and commit. Availability is transient session interaction data,
not a compact-group projection or persisted CDML field, and it exposes no
candidate, recipe, or raw CDML to Qt.

Live render targets retain separate identities: a disposable
`render_identifier` for scene/render-plan ownership, a Rust-issued
`durable_object_id` for live selection and mutation, and, when an operation
needs a parent, a Rust-issued `durable_molecule_object_id`. `source_order` is
display/projection ordering only. Qt submits the durable target, durable owner
when required, and the installed revision/digest fence; it never converts a
render ID, source ID, source order, or projected CDML into a live target.
Returned Rust focus selects `(kind, durable_object_id)` after projection
installation. Structural targets missing a required durable identity are
rejected before Qt installs the observation.

## Success and error data

A successful envelope has this shape:

```json
{
  "schema": "ferrum-operation-response-v1",
  "request_id": "caller-chosen-opaque-id",
  "outcome": {"kind": "document.inspect", "report": {}}
}
```

A typed refusal has this shape; `request_id` is present only when the envelope admitted it:

```json
{
  "schema": "ferrum-operation-error-v1",
  "request_id": "caller-chosen-opaque-id",
  "error": {
    "category": "chemistry_unavailable",
    "operation": "chemistry.convert",
    "message": "human-readable detail"
  }
}
```

Stable error categories are `invalid_request`, `unsupported_protocol_version`,
`document_admission_failed`, `document_invalid`, `render_unsupported`, `render_failed`,
`chemistry_unavailable`, `conversion_failed`, `conversion_unsupported`,
`coordinate_generation_failed`, `resource_limit`, `internal_failure`, `stale_document`,
`atom_not_found`, `molecule_not_direct_root`, `atom_not_in_selected_molecule`,
`unsupported_document`, and `cancelled_before_dispatch`.

When `presentation.author.v1` refuses after envelope admission, its error includes typed
`presentation_author_refusal` facts: an authoring kind, a closed category, and a recovery action.
`refresh_and_restart` means the caller must obtain a current fence and create a new request;
`change_geometry` means finite geometry violates that family's contract;
`adjust_endpoint` means the DirectBond endpoint choice is invalid; and `document_unchanged`
confirms that the supplied document remains the correct retry basis. `change_presentation` and
`report_conflict` describe the remaining typed policy or result-conflict cases. Clients use these
facts rather than parsing diagnostic messages.

When `catalog.insert.v1` refuses after envelope admission, its error includes
`catalog_placement_refusal { category, recovery }`. A stale fence has
`category: "stale_snapshot"` and `recovery: "refresh_and_restart"`; it has no success outcome
or partial document result.

`document.rewrite` provides structural preservation rather than byte identity. A successful render
contains one complete standard-base64 artifact or no artifact; it never exposes a partial result.

## CLI presentation rules

The `inspect`, `validate`, `rewrite`, `render`, `convert`, `coords`, and `open` commands construct their
corresponding requests. The named document commands
`document command presentation.author.v1`, `document command catalog.insert.v1`,
`document command document.molecule.report.v1`, and
`document command document.compact-group.materialize.v1` accept one
complete operation JSON object, just as `protocol run` does, so a script can use the fence from
`document.inspect` without an in-process session. `--json` emits the complete envelope. `ferrum open --json` emits the same `document.molecule.interchange.import.v1` success or typed-refusal envelope as the named protocol operation, with the verb-owned opaque request ID `ferrum-cli`. An admitted CML refusal writes exactly one error envelope to standard output, leaves standard error empty, exits `1`, and publishes no CDML artifact. Other completed unsuccessful human-oriented verb outcomes likewise exit `1` after exactly one diagnostic or JSON envelope. Named protocol subcommands retain their separate protocol exit contract. Without
`--json`, inspection and validation print their reports while result-producing commands emit raw
text or artifact bytes. Named output uses safe publication and cannot replace a retained source or
observed hard-link alias.

`ferrum document-atom-oxidation-observe --request PATH` accepts one complete
`ferrum-operation-request-v1` envelope for `document.atom.oxidation.observe.v1`; `--request -`
reads that envelope from standard input. It writes one canonical JSON success or typed-refusal
envelope and a trailing newline to standard output. An accepted result and a typed domain refusal
both exit `0`; malformed, unreadable, over-budget, or invalid UTF-8 input is a transport failure.

For human-oriented CLI verbs, exit `0` means a completed success; completed unsuccessful outcomes,
including JSON output, exit `1` after exactly one diagnostic or envelope. Named protocol
subcommands retain their separate contract: exit `0` covers a completed success or typed refusal;
exit `1` is an input, processing, or confirmed publication failure; exit `2` is a usage failure;
exit `3` means publication may have occurred but cannot be confirmed. The protocol runner also uses
safe publication for a named JSON response and rejects `--output -`.

## Python boundary

The stateless public automation surface of `ferrum_chem` exposes only these
protocol functions:

```python
def execute_operation_v1(request_json: str) -> str: ...
def operation_protocol_schema_v1() -> str: ...
```

`OperationProtocolErrorV1(FerrumError)` exposes a stable `category` for invalid JSON,
resource-limit, or execution-unavailable failures that occur before an envelope can exist.
Non-string input follows normal Python type checking. This surface accepts no mapping, bytes,
path, session, receipt, or Qt object.

The in-tree Qt bridge is a separate private, session-affine extension surface.
Each native tab owns a Rust document session and accepts committed changes only
through durable receipts and replacement observations. It is not part of the
stateless automation API; see [CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md) for
the desktop ownership boundary.

## Exclusions

V1 excludes batch requests, protocol paths, network access, adapter discovery, CD-SVG or compressed
input, selection or root export, arbitrary template recipes, clipboard, recovery copies, arbitrary
document mutation, and render observation. Its admitted mutations are the closed, fenced
`presentation.author.v1` and `catalog.insert.v1` operations. Desktop editing has its own Rust-owned
live contracts; see [USAGE.md](USAGE.md) and [FILE_FORMATS.md](FILE_FORMATS.md).
