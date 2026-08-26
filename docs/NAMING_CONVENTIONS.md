# NAMING_CONVENTIONS.md

Ferrum uses Rust for chemistry and durable document behavior, Python/PySide6
for application presentation, and PyO3 for narrow transport. This policy makes
those ownership boundaries readable. It supplements, rather than repeats,
[PYTHON_STYLE.md](PYTHON_STYLE.md), [REPO_STYLE.md](REPO_STYLE.md), and the
Rust style already enforced by the workspace formatter.

## Core rule

Name a thing for its owning boundary and convert it once at that boundary.

For example, `CompactGroupCatalogKeyV1::Methoxy` is Rust-owned chemistry
identity and `OMe` is its Rust-derived visible label. PyO3 transports the
issued key and label facts; Qt presents them. Qt does not reconstruct a label,
parse a catalog alias, or maintain a parallel compact-group list.

The same rule applies to document IDs, fences, renderer observations, action
availability, operation receipts, and wire fields. A conversion belongs in one
named adapter or binding function, with its source and destination explicit.

## Boundary matrix

| Boundary | Names and spelling | Ownership rule |
| --- | --- | --- |
| Rust | `snake_case` modules, functions, locals, and fields; `UpperCamelCase` types, traits, and enum variants; `SCREAMING_SNAKE_CASE` constants; Cargo crate names use `kebab-case`. | Domain, document, protocol, and rendering names describe the Rust contract. |
| Python | `snake_case` modules, functions, locals, and fields; `UpperCamelCase` classes; `SCREAMING_SNAKE_CASE` constants. | Python names describe Qt presentation or a typed Rust fact, never a second chemistry model. |
| PyO3 and wire DTOs | Rust owns conversion from domain types to frozen transport facts. Wire field spelling is explicit and stable, normally `snake_case`. | Convert once in the binding; consumers read issued fields and do not recreate aliases, labels, IDs, or chemistry facts. |
| Shell and environment | Lowercase `snake_case` script names; uppercase `SCREAMING_SNAKE_CASE` environment variables. | Scripts state one local workflow; environment names describe configuration, not an internal implementation detail. |
| CLI | Commands and flags are lowercase kebab words; public schemas and operation names identify their durable contract. | A CLI term names a user capability, then maps once to its Rust operation or descriptor. |
| CDML/XML | Element and attribute names preserve the specified document spelling; Rust model names use ordinary Rust spelling. | Serialization/deserialization owns the mapping. UI code does not interpret raw XML names. |
| Chemistry catalog | Persisted key: `methoxy`; Rust enum: `Methoxy`; visible label: `OMe`. | The catalog owns all three facts. Labels are display values, not chemistry input aliases. |
| Files, docs, and tests | Authored source filenames use lowercase `snake_case`; durable Markdown references use `SCREAMING_SNAKE_CASE.md`; tests state the behavior or public contract they prove. | The path names the primary owned subject, not a temporary migration or implementation chronology. |

## Language and acronym spelling

Treat acronyms as words in identifiers: `Cdml`, `Sdf`, and `Smarts` in Rust
and type names; `cdml`, `sdf`, and `smarts` in Python modules, variables, and
wire fields. Preserve conventional all-capital chemistry labels such as `OMe`
only when they are visible catalog labels or exact serialized values.

Maintained Rust and Python source filenames never begin with an underscore as a
temporary-file convention. Python members may begin with one underscore when
they are deliberately module- or instance-private. A private member still has
one clear owner and must not conceal a second public route.

## Durable versions and lifecycle

Use `V1` in Rust type names and `_v1` in Rust function/module names only for a
durable serialized, public protocol, schema, receipt, or cross-boundary
contract. Use an unversioned name for internal implementations, helpers, and
private in-process state. Tests carry a version only when they mirror a real
versioned contract. Versions never record migration chronology.

Ferrum is pre-production. Rename an obsolete name in place and update its
callers. Use one canonical present-tense name for each responsibility.

Lifecycle words describe state, not age:

| Word | Meaning |
| --- | --- |
| `pending` | Accepted preparation awaiting one terminal action. |
| `consumed` | A one-use capability was used and cannot be used again. |
| `committed` | A prepared transition passed admission and changed durable state. |
| `cancel` / `cancelled` | End a pending action without a durable mutation. |
| `closed` | A dialog, session, or resource reached its normal terminal boundary. |
| `dispose` / `disposed` | Release graphics objects after their owner removes them from the scene. |
| `invalidate` / `invalid` | Mark a source-bound fact unavailable because its source changed. |
| `clear` / `cleared` | Empty copied local state while preserving its owner. |
| `remove` / `removed` | Delete generated or build output owned by the current workflow. |

## Boundary vocabulary

Use these terms for the responsibility they identify:

| Term | Meaning |
| --- | --- |
| `adapter` | Converts between two concrete external or platform representations. |
| `binding` | PyO3-owned API that exposes Rust capability or immutable facts to Python. |
| `bridge` | Narrow connection between two owned subsystems while neither adopts the other's model. |
| `projection` | Read-only render or UI-facing view derived from authoritative state. |
| `observation` | Current read-only fact captured from a live document, renderer, or UI target. |
| `facts` | Immutable, named data issued by an owner without authority to mutate it. |
| `candidate` | Proposed mutation awaiting validation and renderer/session admission. |
| `pending` | One-use accepted preparation that has not reached a terminal result. |
| `receipt` | Durable outcome facts returned after a committed or completed operation. |
| `artifact` | Produced local output with its own provenance and lifecycle, not live document state. |
| `session` | Live, fence-aware owner of preparation, commit, history, and current document state. |

## Files, tests, and documentation

Name a module after its primary responsibility, such as
`attached_compact_group_binding.rs` or `compact_group_authoring.py`. Split a
module when it owns more than one responsibility instead of adding chronology
suffixes. Name a permanent test after the public behavior or durable contract:
`e2e_attached_methoxy_materialization.py` describes a visible workflow;
`attached_methoxy_materialization_survives_history_and_reopen` mirrors a
durable session contract.

Documentation uses the user capability or contract name. Explain internal
ownership once where it matters, then link to the owning source or decision.
Avoid duplicating a generated, vendored, or generic policy locally.

## Review checklist

- Does the name identify the owner and one boundary conversion?
- Does Rust issue chemistry, document, and transport facts exactly once?
- Does Python present issued facts without reconstructing labels, aliases, IDs,
  or chemistry meaning?
- Does a `V1` or `_v1` name identify a real durable contract?
- Does the lifecycle name identify the exact current state transition?
- Does the path and test name state the behavior or contract it owns?
- Has an obsolete pre-production name been renamed in place?
