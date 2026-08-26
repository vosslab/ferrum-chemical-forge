# M4b: Document molecule SMARTS query

Status: active implementation plan

Date: 2026-08-20

## Patch 3 API-owned live SMARTS receipt correction

### Decision and ownership

The renderer/document-render prepared-target design is rejected and must be
deleted, not adapted. `ferrum-document-render` already depends on
`ferrum-render`; moving `RenderInteractionSessionV1` into `ferrum-render`
would create a Cargo cycle and would pollute the pure projection crate with
mutable document lifecycle. Keep the generic interaction session where it is,
but remove every SMARTS-specific state, method, proof type, and export from
both crates.

`ferrum-api` owns the complete live SMARTS run, receipt, and redemption flow.
The feature-neutral private snapshot/core lives in
`ferrum_api::document_smarts_snapshot_v1`; its PyO3-only live attachment lives
beside the sole live `PyDocumentSession`. The attachment borrows that session's
one `RenderInteractionSessionV1` and the packaged chemistry runtime
synchronously; it constructs neither a second `DocumentSession` nor a second
runtime. No reverse dependency is added:

```text
ferrum-api -> ferrum-document-render -> ferrum-render -> ferrum-document
ferrum-api -> ferrum-chemistry
```

### Private snapshot, run, and redemption contract

Add feature-neutral, crate-private `document_smarts_snapshot_v1.rs` plus
`python_binding/live_document_smarts_query_v1.rs`, imported only by the private
Python-binding module. The neutral module is the only constructor for the
shared snapshot and is available to the default `ferrum-api` build, stateless
protocol/CLI, and the optional PyO3 attachment. The private values are
non-public, non-cloneable, non-debuggable, and non-serializable:

```text
OwnedDocumentSmartsSnapshotV1 -> one observation plus owned direct targets
OwnedSmartsTargetV1           -> RecordId, source order, MolGraph,
                                 graph_position_to_record_id: Vec<RecordId>
LiveDocumentSmartsRunV1       -> fence, origin, plan generation, native rows,
                                 private durable joins
LiveDocumentSmartsReceiptV1   -> opaque issuer/session key only; no liveness copy
LiveSmartsReceiptLedgerV1     -> PyDocumentSession-owned mutable receipt rows
LiveSmartsPlanGenerationV1    -> PyDocumentSession-owned monotonic generation
```

`OwnedDocumentSmartsSnapshotV1::from_accepted_observation_v1` consumes exactly
one authenticated `SessionDocumentObservationV1` and, in one lowering loop per
direct molecule, produces its complete `MolGraph` and equal-length
`graph_position_to_record_id`. The vector is graph-position aligned, not
source/projection ordered. It carries revision, digest, source order, direct
molecule identity, and complete graph facts; it is shared by stateless and
live execution. It is not raw CDML and is never a second session. The
constructor rejects a missing, duplicate, foreign, non-atom, or surplus atom
key and rejects a graph/key length mismatch before native dispatch. It must not
repair a failed join by iterating `root.atoms()`, by source-order zipping, or by
re-lowering/re-parsing. Selected SMARTS is derived privately from the selected
target graph through the trusted `molecule_to_smarts` path.

For live execution only, the API creates exactly one render observation using
`derive_render_observation_from_accepted_operation_v1(&observation)` from that
same accepted observation. It derives anchors only by looking up every
`graph_position_to_record_id` in that one result's composed `AtomLocal` anchor
map. The helper requires a one-to-one complete mapping for every target before
any matcher call: missing, duplicate, foreign, non-atom, excluded, surplus, or
mismatched keys fail the whole run as `unsupported_document`; no second
projection, source-order join, or fallback traversal is allowed.

`PyDocumentSession` is the authentic render-plan replacement owner. Its private
`publish_live_render_plan_v1` transaction first clears all active Rust SMARTS
receipts and its published plan, then increments `LiveSmartsPlanGenerationV1`, then
derives and publishes the new plan from the accepted observation. The increment
and clear occur before every successful plan publication or reprojection,
including initial publication, document mutation, viewport/layout or display
projection replacement, tab activation/deactivation, tab switch, rerun, close,
and disposal. The session ledger records that generation in each run; redemption
compares it before renderer validation or paint allocation. Revision/digest
alone are insufficient and are retained as independent fences. The Qt
integration inventory must route every plan-publishing/replacing lifecycle
entry point through this transaction; a direct publication is a test failure.

The native rows, IDs, indexes, anchors, fence, origin, plan generation, and
receipt-row state stay in private Rust state. The binding returns Qt only copied
query summaries and, after redemption, finite identity-free paint bounds. It
never returns a graph, record ID, native index, anchor, render plan, selector,
row vector, generated SMARTS, receipt internals, origin, or fence.

`_run_live_document_smarts_query_v1`,
`_show_live_document_smarts_match_v1`, and
`_clear_live_document_smarts_query_v1`, and
`_clear_live_document_smarts_receipts_v1` are private, undocumented PyO3 bridge
operations on the existing `PyDocumentSession`. Show accepts an opaque receipt
and display-row ordinal only. The receipt key is non-forgeable and names no
borrowed renderer or copied liveness flag. A foreign receipt is refused before
touching the receiving ledger. For a local receipt, the ledger atomically
removes/reserves the requested row under its exclusive session mutation gate
before any renderer, anchor, fence, generation, or paint-allocation work. The
validation order is: issuer/session identity; atomic local reserve; immutable
receipt target/row bounds; current revision/digest and origin; current plan
generation; private graph-position/RecordId/anchor correspondence; renderer
issuance validation; then finite identity-free paint allocation. A failure after
reservation remains consumed and is never restored; neither success nor a
post-lookup refusal can replay. Concurrent/reentrant show attempts contend for
the same atomic reserve, so exactly one can proceed. The synchronous bridge
holds the `PyDocumentSession` lease through validation and issuance, while
clearing and plan replacement take the same gate and cannot interleave a
redemption. A paint instruction is not itself one-use: it is a finite
identity-free copied value whose *issuance* is one-use and fenced; Qt may retain
it only until the next mandatory source invalidation. Rerun, mutation,
reprojection, deactivation, tab close, and disposal invalidate Qt's source-bound
receipt state and clear the Rust-held receipt.
Snapshot checks remain mandatory if a frontend lifecycle call is missed.

### Removal of superseded proof surfaces

Delete rather than re-export the renderer and document-render SMARTS proof
surface: `smarts_target_proof_v1`, `RenderObservationV1::prepare_smarts_targets_v1`,
all `RenderPreparedSmarts*`, `RenderSmartsOverlayV1`, target descriptor,
match/projection/anchor/generation accessors, document-render prepared-target
modules, `prepare_smarts_targets_v1`, and
`issue_prepared_smarts_overlay_v1`. No compatibility alias, `#[doc(hidden)]`
re-export, or adapter is permitted. Preserve generic
`RenderInteractionSessionV1` behavior unchanged.

The stateless protocol creates one temporary `DocumentSession`, obtains one
observation, calls the same feature-neutral snapshot constructor, projects
bounded summary facts, and drops all target state. It creates no render
observation, plan, receipt, or reveal capability. The default non-Python
`ferrum-api`/CLI build must compile this path. A shared-observation fixture must
prove that protocol and live construction produce identical private target
record identities, source order, graph position keys, and `MolGraph` facts.
The Qt dock calls only the three private bridge operations; it never parses
SMARTS, lowers molecules, runs chemistry, derives coordinates, or holds a
redeemable selector.

### Proof obligations

Rustdoc-JSON public-surface oracles with hidden items must prove that the three
crates expose none of the removed proof/redeem names, including through macros,
aliases, globs, or re-exports. External fixtures must be unable to import,
construct, debug, clone, serialize, dereference, or redeem any SMARTS renderer
operation. Positive PyO3/API tests prove that only copied summary fields and
issued identity-free paint bounds cross the private bridge. A source-derived Qt
lifecycle oracle proves every plan publication/replacement reaches
`publish_live_render_plan_v1`; direct renderer publication is rejected.

Deterministic live tests cover foreign receipt, stale revision/digest, stale
plan generation, rerun, mutation, reprojection, every Qt plan-publication
route, deactivation, disposal, excluded/unrenderable targets, missing/duplicate
foreign/non-atom/surplus graph keys, duplicate native positions, out-of-range
rows, success replay, and post-lookup refusal replay. Both replay refusals emit
no geometry; post-lookup refusal proves the row was consumed before renderer
validation. Existing reaction, catalog, and vector interaction tests remain
unchanged, proving generic interaction ownership did not move.

## Purpose

M4b delivers a bounded current-document SMARTS query and a live-session Qt
match reveal. It is Rust-owned, not a port of OASA's mutable fragment-search
generator or BKChem's directory-opening add-on. It searches direct molecule
records in one accepted document snapshot; it never searches files,
directories, subdocuments, or a chemical database.

The public search operation is `document.molecule.smarts.query.v1`. It is
stateless public JSON for CLI and data clients. Interactive Qt reveal uses only
the private API/PyO3 live attachment held by its tab controller; it is
not an independent chemistry operation and is not a CLI workflow.

## Frozen V1 decisions

1. `query.v1` accepts either one SMARTS expression or exactly one Rust-issued
   durable direct-molecule root ID from the same document.
2. `query.v1` traverses every eligible direct molecule in authoritative
   document source order for both query forms. The selected query molecule
   supplies only the private query graph and remains an eligible target.
3. The public per-molecule cap is `1..128`, defaults to `50`, and must be no
   larger than the `1..256` total cap, which defaults to `200`. Zero-match
   molecules are absent from the result.
4. A result never invents an omitted-match count. Each returned molecule is
   `complete` or `truncated`; the document traversal is `complete` or
   `incomplete`.
5. The feature-neutral private API snapshot core owns one-pass
   observation-derived target lowering (including graph-position-to-`RecordId`
   identity), FCG1 construction, native execution, and the public completeness
   DTO. It never issues a reveal capability and is usable by default CLI and
   optional PyO3 builds alike. The private `LiveDocumentSmartsQueryBridgeV1`
   beside `PyDocumentSession` owns live-tab runs, exact same-observation
   plan/anchor joins, an API-owned monotonic plan-publication generation,
   atomic one-use receipt reservation, and redemption. Qt consumes only
   Rust-issued summary facts and finite identity-free paint instructions.
6. Patch 2 first removes the `ferrum-chemistry-sys` package. Its generated
   ABI constants, raw dynamic loading, foreign-buffer ownership, and
   ABI/capability admission belong privately to
   `ferrum_chemistry::native_engine::adapter_boundary`. The existing public
   explicit-adapter `ferrum-chemistry` boundary, including
   `NativeChemEngine::load`, remains for typed general chemistry; M4b does not
   promise its removal. Only after the raw boundary is closed may it add
   private FCQ1/FQM1 transport and public typed
   `ChemEngine::smarts_match`. That operation accepts `&MolGraph` and owned
   typed options/results only. Every returned target position is a position in
   that caller-supplied `MolGraph`, never a native matcher index. It exposes no
   FCQ1/FQM1/FCG1, C pointer, adapter, path, native index, raw detail, or
   document ID.
7. One injected sealed runtime remains the sole runtime for public
   document-query protocol, CLI, PyO3, and live-Qt delivery. Whole-program
   runtime consolidation is a later architectural milestone, not an M4b
   prerequisite.
7. Public SMARTS is at most 8,192 UTF-8 bytes, has at most 64 parsed query
   atoms, and addresses at most 256 direct targets. Rust and C++ preflight raw
   input byte bounds; the pinned native `SmartsToMol` parser counts query atoms
   before any `SubstructMatch` work. It returns fixed
   `INVALID_REQUEST` / `query_atom_limit_exceeded` when the parsed count is
   above 64. Public JSON responses are at most 1 MiB. V1 supports data/result
   caps and cancellation before dispatch. It makes no hard-timeout or
   active-native-call cancellation guarantee.

Any change needs contradictory evidence recorded here and in the changelog.

## Reference parity and improvement

The historical BKChem add-on selects one molecule, searches immediate SVG/CDML
files in a chosen directory, opens matching files, and stops at the first
matching molecule. OASA's mutable generator needs caller cleanup when stopped
early. Ferrum preserves "does this structure contain this fragment?" while
making ownership, scope, bounds, and visibility explicit.

| Historical behavior | Ferrum M4b behavior |
| --- | --- |
| Selected fragment plus directory picker. | Explicit SMARTS or one durable direct-molecule root in the supplied document. |
| Cleanup-sensitive mutable search state. | Stack-local native objects and owned Rust DTOs. |
| Matching documents open as a result. | One dock lists source-ordered, bounded mappings. |
| First hit per document. | Bounded mappings per direct molecule with truthful completeness facts. |
| No safe match activation contract. | Live-tab receipt-fenced renderer overlay redemption. |

External file/directory search needs separate discovery, loading, worker, and
cross-document identity design.

## Ownership and data flow

```text
CLI / data client -- JSON --> public protocol facade
                         |
                         v
              private API molecule-query snapshot core
              - authenticate input fence and direct roots
              - lower selected root privately when requested
              - build FCG1 from the owned observation-derived snapshot
              - invoke the API sealed trusted runtime
                         |
                         v
public typed ChemEngine::smarts_match(&MolGraph, owned options)
                         |
                         v
private native_engine::adapter_boundary -- FCQ1 --> ABI-5 adapter / pinned RDKit
                               |
                               v
                       FQM1 bounded index matrix
                               |
                               v
                    public bounded match facts

DocumentTab live Rust document/render-interaction session
                         |
                         v
ferrum-api PyO3 attachment: LiveDocumentSmartsQueryBridgeV1
              - one PyDocumentSession / one RenderInteractionSessionV1
              - private exact plan/anchor join, receipt ledger, and redemption
              - session-owned plan-publication generation and clear gate
                         |
                         v
                    renderer-issued overlay primitives/bounds
```

No RDKit graph/query/exception/lifetime, raw adapter path, native index, CDML,
renderer object, geometry, renderer plan, raw wire bytes, or reveal capability
is public JSON or the M4b typed SMARTS surface. The existing public typed
general-chemistry adapter boundary is not a document-query authority.
Documents own durable `RecordId`s; the private API run owns the live SMARTS
receipt and derives identity-free paint bounds from the same live observation.

`LiveDocumentSmartsQueryBridgeV1` is implemented in the private
`ferrum_api::python_binding::live_document_smarts_query_v1` module as methods
on the existing `PyDocumentSession`. It calls neither renderer SMARTS methods
nor document-render SMARTS methods because those surfaces are removed. There is
no callback, borrowed session lease, unsafe pointer, second `PyDocumentSession`,
raw CDML reload, or duplicate runtime. `ferrum-api` owns sealed-runtime
admission, observation-derived snapshots, the private ledger, and the one-use
receipt. Qt receives only closed summaries and paint instructions; it
invalidates source-bound local state before requesting the Rust clear operation.
None is reexported through public Rust, protocol, or PyO3
facades.

## SMARTS profile

The profile is `rdkit-smarts-to-mol-pinned-v1`: all SMARTS syntax accepted by
the pinned adapter's `SmartsToMol` is accepted. Matching is exactly pinned
RDKit substructure matching with `useChirality=true` and `uniquify=true`.
It includes aromaticity, isotopes, formal charges, stereochemistry, recursive
SMARTS, disconnected SMARTS, query bonds, atom lists, and implicit-hydrogen
semantics whenever that pinned parser accepts them.

Malformed or atomless parser output is `invalid_query`. There is no string
pre-parser, narrow grammar, `unsupported_syntax`, or profile-refusal state.
The native FCQ1 implementation calls this pinned parser once, counts its parsed
query atoms, and refuses more than 64 with the fixed
`INVALID_REQUEST` / `query_atom_limit_exceeded` result before any
`SubstructMatch`, row allocation, FQM1 success receipt, or Rust live-reveal
receipt issuance. Raw UTF-8 byte bounds remain independently enforced before
parsing by Rust and C++; no Rust string parser is introduced for atom counting.
`unsupported_document` applies only when a target document/direct molecule
cannot be converted to the supported FCG1/RDKit target graph. No hard timeout
is promised.

Fixtures cover aromatic, isotope/charge, stereochemical, recursive, and
disconnected accepted SMARTS; malformed and atomless syntax; zero matches; and
an unconvertible target graph. UI labels distinguish invalid query, unsupported
target, resource limit, and zero matches.

## Patch 2 ownership sequence and proof gates

Patch 2 is a single chemistry-boundary change before protocol, CLI, PyO3, or
Qt query delivery work. It deliberately takes advantage of Ferrum's
pre-production state: the sole production consumer of `ferrum-chemistry-sys`
is `ferrum-chemistry`, so the raw facade can be removed rather than preserved
as a compatibility package.

### Work package 2.1: close the adapter boundary

Move the generated ABI constants required by the loader, plus `Library`, ABI
function pointers, ABI/capability admission, foreign owned-buffer release, and
`!Send + !Sync` enforcement, into private
`ferrum_chemistry::native_engine::adapter_boundary` ownership. Remove the
`ferrum-chemistry-sys` workspace package, source tree, and dependency entirely.
The existing public
`NativeChemEngine::load` explicit-adapter path remains a typed
general-chemistry API and invokes this private boundary internally. It returns
no loader, buffer, raw request/response, capability, or wire facade.

### Work package 2.2: add typed SMARTS chemistry

Add private `native_engine::smarts_wire` to encode FCQ1 and decode FQM1,
reusing private FCG1 graph encoding. The decoder validates status/detail
legality, row width, index range against the target graph, duplicate target
positions, overflow, and full-byte consumption before constructing owned typed
values. Add public `ChemEngine::smarts_match(query, target, options)` and a
`NativeChemEngine` implementation returning typed query-ordered target
positions in the caller-supplied `target: &MolGraph` plus a `truncated` fact;
they are never native indexes. Add the dedicated closed public error branch
`ChemistryError::SmartsMatchUnavailable { reason: SmartsMatchUnavailableReason }`,
where the closed reason enum is exactly `runtime_unavailable`, `abi_incompatible`,
`capability_unavailable`, `native_call_failed`, `malformed_native_response`, or
`native_rejected`. Every new `smarts_match` loader, ABI, capability, native-call,
FQM1 decoder, and native-rejection failure maps to this branch with no supplied
string, path, adapter diagnostic, wire marker, native detail, or wrapped source
error. In particular, it must never surface through the existing detail-bearing
`NativeBoundary` or `NativeRejected` variants. Typed results must not contain
wire bytes, `u32` matrix rows, C pointers, a library handle, path, or foreign
allocation.

### Work package 2.3: prove the external surface before protocol delivery

Run external-consumer compile proofs for default and `python-binding`
`ferrum-api` builds. A protocol consumer uses only `execute_operation_v1` and
public protocol DTO/schema types. A direct `ferrum-chemistry` consumer may
construct the existing typed explicit adapter and call typed general-chemistry
operations, including `ChemEngine::smarts_match`, but cannot obtain a raw
loader, buffer, FCQ1/FQM1/FCG1 value, native index, or native detail. It also
cannot import private document-query core/prepared types, the API sealed
runtime, runtime-aware execution, live receipt/reveal authority, or
Python-binding internals. A direct dependency on the removed
`ferrum-chemistry-sys` package must fail because no workspace package or path
remains.

The chemistry test gate proves `NativeChemEngine::smarts_match` returns owned
typed mappings. Source/compile evidence proves no public FCQ1/FQM1 function
or raw sys facade remains. These gates are required in addition to ABI
hostile-detail, CLI, PyO3, and Qt delivery evidence; they prevent a passing
protocol path from concealing a leaked native ownership surface.

The nonleakage promise in this work package is deliberately limited to the new
`smarts_match` operation. Legacy typed general-chemistry operations may retain
their existing `NativeBoundary` and `NativeRejected` contracts until the later
broad chemistry-redaction redesign; they must not be used as a shortcut for
M4b SMARTS failures.

## ABI-5 contract

### Capability and loading

ABI-5 adds `FERRUM_CHEM_CAPABILITY_SMARTS_MATCH` and:

```c
uint32_t ferrum_chem_smarts_match_v1(
    const uint8_t *request, uint64_t request_len,
    ferrum_chem_owned_buffer *response) FERRUM_CHEM_NOEXCEPT;
```

The capability is advertised only with the compiled symbol. The loader rejects
ABI/capability disagreement and loads the symbol only when advertised. CMake
and sealed-wheel tooling explicitly retain pinned `RDKitSubstructMatch`. No
Python RDKit, SWIG, or second runtime is permitted.

### FCQ1 request

FCQ1 is little-endian and exactly consumed:

```text
magic[4] = FCQ1
wire_version:u32 = 1
smarts_length:u32
fcg1_length:u32
max_matches:u32
flags:u32 = 0
smarts UTF-8 bytes
FCG1 bytes
```

Both sides reject empty, NUL-containing, invalid UTF-8, or oversized queries;
invalid caps; malformed/trailing content; and graph-wire overflow. Rust and C++
enforce raw byte bounds before parsing. The FCQ1 native path then calls pinned
`SmartsToMol`, counts parsed query atoms, and returns the closed
`INVALID_REQUEST` / `query_atom_limit_exceeded` FQM1 outcome for more than 64
atoms before `SubstructMatch` or match-row allocation. Rust maps only this
closed native outcome to public `invalid_request` /
`query_atom_limit_exceeded`; protocol and Qt preserve that closed reason and
never display a native diagnostic. Native ABI limits remain private and broader
than the public contract: 65,536 query bytes, 1,024 parser-admitted query
atoms, 10,000 matches, and 1,000,000 matrix cells. Multiplication and
allocation are checked first.

### FQM1 response ABI

```text
magic[4] = FQM1
wire_version:u32 = 1
result_status:u32
detail_length:u32
query_atom_count:u32
match_count:u32
flags:u32
detail ASCII bytes
match_count rows of query_atom_count target-atom u32 indexes
```

`result_status` has exactly these numeric values and fixed ASCII detail codes:

| Status | Value | Detail | Legal payload |
| --- | ---: | --- | --- |
| `OK` | 0 | empty | `query_atom_count > 0`; rows permitted; only `TRUNCATED` flag permitted. |
| `INVALID_REQUEST` | 1 | `invalid_request` or `query_atom_limit_exceeded` | zero counts, zero flags, no rows. |
| `INVALID_QUERY` | 2 | `invalid_query` | zero counts, zero flags, no rows. |
| `UNSUPPORTED_TARGET` | 3 | `unsupported_target` | zero counts, zero flags, no rows. |
| `RESOURCE_LIMITED` | 4 | `resource_limited` | zero counts, zero flags, no rows. |
| `NATIVE_FAILURE` | 5 | `native_failure` | zero counts, zero flags, no rows. |

The only flag is `FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED`. It is legal only
with `OK` and says enumeration observed at least one match beyond the FCQ1
cap. `OK` with zero rows is a complete no-match result. The Rust decoder rejects
unknown status/detail/flags, illegal combinations, overflow, truncation or
trailing data, out-of-range indexes, and duplicate target indexes within a row.

Rows are query-index ordered. Complete returned rows are lexicographically
sorted before serialization. Results are deterministic for one pinned
adapter/input; cross-RDKit-version global lexical minimality is not promised.
Native `what()`, parser text, paths, CDML, and arbitrary bytes never enter a
detail field. Hostile-detail tests inject each at C++, Rust decoder, protocol,
CLI, PyO3, and Qt-display boundaries and prove fixed-code redaction.

## Public query and cap contract

Request:

```json
{
  "kind": "document.molecule.smarts.query.v1",
  "document": {
    "cdml": "<bounded CDML document text>",
    "expected_revision": "...",
    "expected_digest_hex": "..."
  },
  "query": {"kind": "smarts", "value": "[O;H1]"},
  "limits": {
    "max_matches_per_molecule": 50,
    "max_total_matches": 200
  }
}
```

The alternate query is exactly:

```json
{"kind": "selected_molecule", "molecule_id": "durable-direct-root-id"}
```

Qt obtains `molecule_id` only from an existing Rust observation of the active
tab. CLI receives that durable root ID explicitly. The protocol accepts one
selected ID only and, before graph construction, refuses absent, duplicate,
multiple, ambiguous, excluded, nonmolecule, or non-direct-root IDs. It then
authenticates membership in the accepted fenced document. The selected query

Selected-root lowering is private and fixed. The core resolves that one direct
same-snapshot molecule, projects its complete `MolGraph`, and calls the
existing trusted `ChemEngine::molecule_to_smarts` through the same pinned
adapter/profile used by the sealed runtime. The resulting SMARTS is private;
it is then sent in FCQ1 to `smarts_match` for every eligible target, including
the source molecule, in source order. A selected-source projection or lowering
refusal is `unsupported_document` / `selected_source_not_matchable`; a target
projection refusal is `unsupported_document` / `target_not_matchable`.

`DocumentSmartsQuerySummaryV1` exposes no query text. Remove
`query_display` entirely rather than redacting or repurposing it. Its only
query-origin fact is the closed, identity-free discriminator
`query_origin: "raw_smarts" | "selected_molecule"`. It conveys which request
form ran, but no durable root ID, raw SMARTS, generated SMARTS, normalized
query, atom count, or other query-derived identity. The raw expression remains
caller input only; the selected-root `molecule_to_smarts` output remains inside
the feature-neutral core from lowering through native matching and is dropped
before summary, error, diagnostic, receipt, overlay, log, `Display`, or
`Debug` construction.

Before resolution, allocation, native dispatch, receipt minting, or DTO growth,
the core uses checked arithmetic and preflights bounds knowable without parsing:
8,192 SMARTS bytes, 256 direct targets, requested caps, and the 1 MiB JSON
response maximum. The native FCQ1 parser is the sole authority for the full
pinned-profile 64 parsed-query-atom bound; its closed limit outcome prevents
matching and receipt minting. For each target it passes
`min(max_matches_per_molecule, remaining_global)` to FCQ1. When the global
budget reaches zero it stops traversal and reports tagged incomplete without a
numeric omitted count. A request cap reached is a successful tagged fact; an
inability to allocate, mint a typed receipt, or fit a valid result in the
response budget is `resource_limited`, respectively `reveal_receipt_unavailable`
or `response_size_exceeded`.

Direct target traversal follows the authoritative document source order. For
each eligible molecule, the remaining global capacity is
`max_total_matches - matches_returned`; its FCQ1 cap is the lesser of that
remaining capacity and `max_matches_per_molecule`. The next molecule is not
examined once the remaining capacity is zero. A molecule with zero returned
matches is omitted.

The success DTO has no numeric omitted total:

```json
{
  "kind": "document.molecule.smarts.query.v1",
  "document": {"revision": "...", "digest": "..."},
  "query": {"origin": "raw_smarts"},
  "result": {
    "traversal": {"kind": "complete"},
    "molecules": [{
      "molecule_id": "durable-direct-root-id",
      "matches": [{
        "ordinal": 1
      }],
      "completeness": {"kind": "complete"}
    }],
    "totals": {"molecules_matched": 1, "matches_returned": 1}
  }
}
```

A capped molecule is
`{"kind":"truncated","lower_bound":"at_least_one_omitted"}`. It makes
only that lower-bound claim. A global-cap stop is
`{"kind":"incomplete","reason":"total_match_budget_reached",
"unexamined_molecules_possible":true}`. It does not assert a numeric omitted
count or claim anything about unexamined molecules. The result is otherwise
`{"kind":"complete"}`. Exact spelling is schema-owned at implementation.

## Live Qt reveal bridge

There is no `document.molecule.smarts.reveal.v1` public JSON operation. The
stateless JSON result carries no capability, geometry, renderer bounds, native
atom index, or dereferenceable match identity; ordinal is display ordering
only. CLI and ordinary PyO3 data clients consume that result only.

For one `DocumentTab`, its private API binding borrows the existing
`PyDocumentSession.session: RenderInteractionSessionV1` and obtains one
same-revision document observation plus one render observation. The shared
`OwnedDocumentSmartsSnapshotV1` derives all direct target graphs from the
document observation. The live run additionally derives and retains the exact
graph-index -> durable atom -> composed `AtomLocal` join from the render
observation. It uses the sealed runtime only after every target is admitted.
The API-owned receipt validates origin, revision/digest, generation, liveness,
row bounds, and one-use state before it copies identity-free finite paint
bounds. Qt holds only that issued instruction and invalidates its source-bound
local activation before every mutation, observation replacement, reprojection,
rerun, tab switch, or disposal; Rust clears the matching receipt. No ordinary
`DocumentSession` facade method, public
receipt/paint class, Rust-to-Qt callback, JSON route, CLI route, or raw
position projection is permitted.

Live lowering is atomic: any incomplete correspondence for a direct target or
selected root returns closed `unsupported_document` before chemistry and
publishes no summary, run, receipt, traversal result, or overlay. Stateless
execution shares only the owned document snapshot, remains render-independent,
and drops all target state after projecting its truthful summary.

## Closed errors and recovery

| Class | Stable reasons | Recovery |
| --- | --- | --- |
| `invalid_request` | `query_missing`, `query_variant_invalid`, `query_too_long`, `query_atom_limit_exceeded`, `target_limit_exceeded`, `match_cap_invalid`, `match_caps_inconsistent` | Correct the request. |
| `invalid_query` | `invalid_smarts_query`, `query_has_no_atoms` | Edit SMARTS or select a molecule. |
| `stale_document` | `revision_mismatch`, `digest_mismatch` | Refresh and rerun. |
| `unsupported_document` | `selected_source_not_molecule`, `selected_source_not_matchable`, `target_not_matchable` | Use a direct molecule or supported target. |
| `resource_limited` | `response_size_exceeded`, `reveal_receipt_unavailable`, `native_resource_limited` | Narrow the query or lower scope. |
| `cancelled` | `cancelled_before_dispatch` | Run again. |
| `chemistry_unavailable` | `native_runtime_unavailable` | Repair matching sealed runtime. |

Malformed native replies, ABI/load failures, unexpected native exceptions, and
hostile details map from the dedicated closed SMARTS chemistry error to
`chemistry_unavailable`. Only fixed codes and safe messages cross the public
boundary. API, CLI, and PyO3 retain their broader existing redaction gates in
addition to this direct-library guarantee.

## Qt design

`Chemistry -> SMARTS Query...` opens/focuses a nonmodal right dock bound to one
source tab and live typed bridge. It offers mutually exclusive SMARTS text and
one currently observed selected molecule, concise grammar guidance, and
explicit per-molecule/document cap status. Detached JSON workers are used only
for stateless CLI/data-client operation; the dock invokes the typed bridge for
its live query/reveal path and never receives JSON capabilities.

A row activates only through its current Rust-held typed receipt. A run is
disabled while pending. Cancellation is enabled only before dispatch. Tab
switch, source close, stale document, new query, or renderer-plan replacement
invalidates activation; the dock does not edit the document. Keyboard, empty,
zero-match, invalid, unavailable, stale, capped, and reveal-refused states are
all explicit.

## Work packages

1. **ABI and adapter boundary:** ABI-5 header/generated status matrix,
   capability, symbol, private raw loader/buffer boundary, explicit
   SubstructMatch linkage, and retained-library audit.
2. **Native matcher:** FCG1/FCQ1 parsing, pinned parser/matching profile,
   parser-owned 64 parsed-query-atom admission before `SubstructMatch`, RAII,
   exact FQM1 encoding, cap observation, and hostile-detail containment.
3. **Safe Rust chemistry:** owned options/results, caller-`MolGraph` target
   positions, the dedicated detail-free `SmartsMatchUnavailable` closed error
   mapping, private FCQ1/FQM1 codecs, and `ChemEngine::smarts_match` with no
   raw/native lifetime escape. The existing public typed explicit-adapter
   construction remains outside document-query authority; its legacy error
   contract is not widened by M4b.
4. **Document/protocol core:** fenced request schema, selected-root resolver and
   private `molecule_to_smarts` lowering, public-bound admission, source-order
   traversal/cap allocation, private atom projection, and truthful public DTO.
5. **Live bridge:** remove all renderer/document-render SMARTS proof exports;
   add feature-neutral API `document_smarts_snapshot_v1` and the private PyO3
   module beside `PyDocumentSession`. The sole snapshot constructor performs
   one-pass graph-position-to-`RecordId` lowering. Add exact same-observation
   plan/anchor joins, the session-owned `publish_live_render_plan_v1`
   clear-plus-monotonic-generation transaction, a ledger with atomic
   reserve-before-validation receipt consumption, and one-use-issued
   identity-free Qt paint instructions.
6. **CLI and PyO3:** one named stateless query executor through the existing
   trusted runtime; CLI never invokes reveal; redaction mapping and fixtures.
7. **Qt dock:** typed bridge acquisition from its source DocumentTab, observed
   root acquisition, source lifecycle, receipt-only reveal, and display states.
8. **Docs/evidence:** API, CLI, GUI, ABI, cap, cancellation, profile, and
   changelog documentation with isolated-artifact proof.

## Test and evidence gates

1. ABI-5 header/generated contract/capability/symbol/retained-link audits
   agree; ABI-4 rejects ABI-5; every FQM1 status/detail/field/flag legality is
   tested, including hostile C++ details.
2. Native tests cover accepted profile fixtures, malformed/atomless query,
   a parser-accepted 65-query-atom fixture that returns only
   `INVALID_REQUEST` / `query_atom_limit_exceeded` before target traversal,
   matching, row allocation, or receipt issuance, no-match, target conversion
   refusal, UTF-8/NUL/private ABI caps, deterministic rows, FQM1 truncation,
   and malformed/trailing FCQ1/FQM1. Rust, protocol, CLI, PyO3, and Qt tests
   prove the same closed public reason with no native diagnostic leakage.
3. Rust tests prove closed mapping, overflow, graph-wire integration,
   caller-`MolGraph` position projection, and no native lifetime escape. A
   direct-library hostile-fixture suite injects an absolute loader path and
   adapter, FCQ1, FQM1, and native-detail tokens into each new SMARTS failure
   category, then proves `Display`, `Debug`, and every public error field leak
   none of them. The API, CLI, and PyO3 suites retain their broader redaction
   gates for all public protocol behavior.
4. Document/protocol tests prove source order, selected-query target inclusion
   through the one private lowering, zero-match omission, two-molecule cap
   allocation, molecule truncation lower bound, global incomplete traversal,
   durable same-snapshot projection, selected aromatic/isotope-charge/stereo/
   disconnected fixtures, cap zero/max/per-greater-than-total refusal, maximum
   total spread, and 1 MiB response-budget refusal. JSON-schema and PyO3-
   binding regression cases exercise both raw-SMARTS and selected-molecule
   requests with distinct sentinel SMARTS text. They prove the public summary
   carries only `query_origin`, never `query_display`, and no raw or generated
   SMARTS reaches JSON, Python-visible summary fields, errors, logs, `Display`,
   or `Debug`; normal result summaries and identity-free issued paint bounds
   remain available for both forms.
5. Live-bridge tests prove the default non-Python CLI/API build uses the same
   feature-neutral snapshot constructor as the PyO3 bridge, including an
   identical-observation target identity/source-order/graph-key/graph fixture.
   They prove atomic observation-derived admission across every
   in-scope direct molecule: duplicate, missing, excluded, display-only,
   foreign, unrenderable, and mismatched plan/anchor facts fail closed as
   `unsupported_document` with no summary, run, receipt, traversal result, or
   overlay; selected-root failure is identical. They prove valid current
   redemption and foreign receipt, stale revision/digest, rerun, mutation,
   reprojection, every plan-publication lifecycle route, deactivation, disposal,
   out-of-range row, duplicate native position, success replay, post-lookup
   refusal replay, and two-tab identical-CDML cross-redemption refusal without
   geometry. A post-lookup refusal must prove its row was consumed before
   renderer validation, and a source-derived lifecycle oracle must reject any
   direct plan publication that bypasses the session transaction. Rustdoc-JSON
   and external fixture tests prove removed renderer and
   document-render SMARTS proof/redeem surfaces cannot be reached through a
   module, macro, glob, alias, or re-export; positive API/PyO3 tests expose
   only copied summaries and issued identity-free paint bounds. A direct
   chemistry consumer remains allowed only typed general-chemistry calls and
   is refused raw loader/buffer/wire access.
6. CLI/PyO3 tests submit both complete request variants through one packaged
   runtime; CLI only queries; runtime/path/graph bypasses are unavailable or
   redacted.
7. Qt tests prove no local chemistry/geometry or authorization, pending/
   pre-dispatch cancellation, all display states, typed-receipt-only activation, and lifecycle
   invalidation. A fresh isolated Python 3.12 proof pairs sealed ABI-5 native
   and Qt wheels and exercises CLI, PyO3, and real Qt interaction.

## Non-goals

- Filesystem, directory, database, or cross-document search.
- Hard timeout, kill-safe cancellation, or active-native-call interruption.
- A narrow custom SMARTS grammar or unsupported-syntax classification.
- Public raw engine/loader/buffer/wire/document/renderer handles, public JSON
  reveal capabilities, Python RDKit, SWIG, duplicate document-query runtimes,
  raw native rows, or Qt-local chemistry, geometry, or authorization. The
  existing public typed `NativeChemEngine::load` general-chemistry boundary is
  explicitly not removed by M4b.
- Match editing, document mutation, cross-tab reveal, or a CLI reveal workflow.
- Cross-version byte-identical unbounded match ordering.

## Integration risks

| Risk | Mitigation |
| --- | --- |
| Expensive subgraph matching. | Bounded inputs/results, worker responsiveness, no-hard-timeout boundary; later process isolation if needed. |
| Cap semantics misrepresented. | Tagged per-molecule and traversal completeness with multi-molecule boundary tests. |
| Receipt misuse. | API-private one-use receipt, full reauthentication, identical-CDML cross-tab tests, identity-free paint issuance, and invalidation tests. |
| ABI/wheel drift. | Exact status matrix and retained-link tests plus fresh artifacts. |
| SMARTS loader or native diagnostics leak through direct typed errors. | Every new `smarts_match` failure uses the closed detail-free `SmartsMatchUnavailable` branch; hostile direct-library fixtures assert that absolute paths and adapter/FCQ1/FQM1/native-detail tokens are absent from `Display`, `Debug`, and public fields. API, CLI, and PyO3 retain broader redaction gates. Legacy chemistry operations are explicitly deferred to the broader redaction redesign. |
| Typed explicit adapters remain public. | Keep their scope to owned typed general chemistry; prove raw loader/buffer/wire and private document-query/runtime/reveal authority are unreachable. Consolidate whole-program runtime ownership later. |

## Completion criteria

M4b completes only when sealed ABI-5, CLI query, PyO3 query, and the Qt dock
deliver both query variants with the stated profile, source order, truthful cap
facts, opaque reveal lifecycle, redaction, and fresh installed-artifact proof.
