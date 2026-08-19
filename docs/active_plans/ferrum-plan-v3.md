# Plan: Ferrum v3, a Rust chemistry backend for Ferrum

**Authority:** this is the active implementation plan and status tracker.
It supersedes `floofy-snacking-ripple.md` and `floofy-snacking-ripple-v2.md`.
Completed-milestone evidence lives in
`docs/active_plans/reports/completed_milestone_evidence.md`; the large
session-adoption record lives in
`docs/active_plans/reports/m16_session_adoption_evidence.md`. Those focused reports
preserve implementation history without obscuring current decisions.

## Purpose and boundaries

Ferrum is the retained PySide6 CDML editor in
`packages/ferrum-chem-qt.app/ferrum_qt/`, rebranded from its historical frontend.
Ferrum-Chem is the Rust engine that replaces OASA. The distinction is intentional:
the frontend is continued user code under a new product identity; the chemistry and
document backend is a new implementation. `OTHER_REPOS/` is read-only historical
reference material and may not participate in build, test, runtime, or release.

The product goals are to:

- make Rust the authoritative CDML, chemistry, geometry, and rendering backend;
- keep ordinary Ferrum usable as supported routes move to native sessions;
- preserve existing CDML structurally, including opaque content, rather than promise
  writer-byte equivalence;
- place RDKit behind one project-owned, replaceable native adapter;
- provide a coherent Python, CLI, and desktop boundary with no production OASA,
  Python RDKit, Tk, or BKChem product route; and
- establish release evidence for supported wheels before calling the product shipped.

Non-goals: carrying OASA/Tk as production code, rewriting the Qt architecture merely
because of lineage, changing the CDML on-disk format, browser-front-end development,
and a pure-Rust replacement for mature RDKit chemistry perception.

The first usable increment remains the **thin workflow**: open a simple CDML
molecule, generate coordinates through Ferrum, draw Rust-issued operations on the Qt
canvas, and save with every persistent object preserved. It is an ongoing regression
gate, not a claim of product release.

## Working decisions

1. **Chemistry authority:** RDKit performs molecule perception, valence, ring,
   aromaticity, canonical ranking, and format work. Rust ports self-contained
   arithmetic/depiction utilities and owns public DTOs. `ChemEngine` never leaks an
   RDKit representation.
2. **Packaging shape:** Ferrum-Chem is a separately replaceable shared library under
   LGPL v3; Ferrum is AGPL v3. Each release target needs an actual clean-install and
   relink receipt. A source build that happened to work locally is not release proof.
3. **CDML source of truth:** written CDML contracts win, followed by documents users
   created, RDKit chemistry behavior, recorded intended Ferrum changes, then OASA
   observations. Fixture divergences are classified as compatibility, known defect,
   implementation accident, or intended change before they influence code.
4. **Protocol:** public V1 is pathless, JSON, stateless, versioned, resource-bounded,
   and schema-backed. Filesystem selection belongs to CLI/desktop transports, never
   protocol payloads. Extension-adapter discovery belongs behind one Python boundary.
5. **Evidence discipline:** tests protect durable semantics. Wheel builds, GUI
   walkthroughs, oracle comparisons, visual inspection, benchmarks, and current
   platform observations remain E2E or report evidence unless they meet the repository
   permanent-test policy.

## Current state

The retained frontend is visibly Ferrum, uses `ferrum_qt`, and has one lazy native
extension adapter. Rust has eight workspace crates (`api`, `chemistry`,
`chemistry-sys`, `core`, `document`, `domain`, `geometry`, and `render`). T27 reduced
`ferrum-api` to delivery ownership: CLI, pathless protocol, trusted protocol runtime,
and transport. Chemistry codecs live in `chemistry`; document/session and CDML work in
`document`; scientific plans in `domain`; and render plans/artifacts in `render`.

M19 is **done** on current local evidence. T1-T27 delivered the CI front door,
hygiene/plan restructuring, safe Rust closure, six protocol-backed verbs, explicit
engine bundles, the Ferrum frontend seams, worker and geometry ownership, keyboard
authoring/accessibility, refusal wording, documentation, and crate decomposition.
The final local Python/Qt gate passed 5,916 repository tests, 213 installed binding
tests, and 393 Qt tests with 1 skipped. `check_rust.sh` passed before that final
Python/Qt run. The new GitHub Actions workflow has not yet run remotely, so no remote
CI claim is made. M20 and M22 remain open release gates; M21 has not started. The
capability matrix remains the closure ledger:
[Ferrum capability matrix](audits/ferrum_qt_capability_matrix.md).

## Status tracker

Allowed status vocabulary: `not started`, `in progress`, `blocked`, `done`.

| M | Milestone | Exit condition | Status | Owner |
| --- | --- | --- | --- | --- |
| M1a | Identity and licensing | Accurate identity, licenses, provenance, and base hygiene | done | `maintainer` |
| M1b | Ferrum rename and capability matrix | Ferrum starts; capabilities are enumerated | done | `coder` |
| M1c | Rust workspace skeleton | Final crate layout builds | done | `coder` |
| M1d | Oracle/preservation inventory | Historical comparator and coverage inventory established | done | `tester` |
| M1e | Exclusion checks | No activated production OASA/Tk route | done | `tester` |
| M2 | Core model | Corpus molecules project to owned model | done | `coder` |
| M3 | Graph/cycles | Deterministic graph analysis | done | `coder` |
| M4a | Packaging viability | Replaceable native library mechanism proven | done | `maintainer` |
| M4b | Adapter semantics | One narrow chemistry trait reaches RDKit | done | `expert_coder` |
| M4c | Coordinate parity | Measured tolerance and current-target parity | done | `tester` |
| M4d | Qt chemistry slice | Ferrum consumes adapter output | done | `coder` |
| M5 | Chemistry codecs | Supported codec semantic gates pass | done | `expert_coder` |
| M6 | XML/opaque retention | Structural preservation boundary proven | done | `coder` |
| M7 | Identity/order/references | IDs/order/reference semantics preserved | done | `coder` |
| M8 | Typed records | Assigned typed CDML overlay complete | done | `coder` |
| M8a | Early session adoption | Thin workflow reaches Ferrum | done | `expert_coder` |
| M9 | Document-core semantics | Atomic revisions/history/recovery are single-owned | done | `expert_coder` |
| M10 | Corpus preservation | Coverage-led structural gate passes | done | `tester` |
| M11 | Geometry/straighten | Atomic geometry result and parity receipt | done | `coder` |
| M12 | Render ops/glyph metrics | Qt consumes Rust-issued render facts | done | `coder` |
| M13 | Render backends | Checked SVG/PNG/PDF lowering | done | `coder` |
| M14 | Haworth infrastructure | Bounded source-backed topology/layout/depiction | done | `expert_coder` |
| M15 | Domain utilities | Adopted utilities bounded; other families disposed | done | `expert_coder` |
| M16 | Session adoption | One ordinary native window owns supported routes | done | `expert_coder` |
| M17 | Operation protocol | Versioned public contract frozen | done | `expert_coder` |
| M18 | Python module and CLI | Supported binding and shell route callable | done | `coder` |
| M19 | Integration closure | Every supported matrix row has an appropriate lane | done | `integrator` |
| M20 | Packaging/platform matrix | Each admitted target clean-installs and relinks | in progress | `maintainer` |
| M21 | WASM proof | Project WASM validates frozen contract | not started | `expert_coder` |
| M22 | Supported-product release | Release artifacts/workflows prove boundary | in progress | `maintainer` |

## Milestone definitions

Completed milestones retain their detailed evidence in the reports above. These
definitions preserve the contract needed to assess regressions or reopen a milestone.

| Milestone | Depends on | Durable definition |
| --- | --- | --- |
| M1a-M1e | - | Maintain accurate product/licensing/provenance identity, a historical capability ledger, preservation inventory, and production exclusion boundary. Reference-only OASA material is permitted only in classified history/provenance locations. |
| M2-M3 | M1c | Keep an immutable owned molecule model and deterministic graph/cycle algorithms. Any oracle divergence requires source-of-truth classification. |
| M4a-M4d | M2, M4a as applicable | Keep the native adapter narrow, replaceable, and package-relative; record reference defaults; derive coordinate tolerances from measurement; frontends receive owned Rust facts. |
| M5 | M4b | Support only explicitly contracted SMILES/SMARTS/molblock/SDF/InChI profiles. Compare deterministic identifiers exactly and writer formats semantically. |
| M6-M10 | M1c/M2/M8 as applicable | Parse under explicit resource limits, reject DTD/entity/network paths, retain opaque typed-tree content, preserve IDs/order/references, and verify coverage-led structural CDML round trips. |
| M11-M13 | M2/M11 | Keep y-up geometry and checked render operations canonical. Qt, SVG, PNG, and PDF consume one validated plan; unsupported roots are named exclusions rather than silently lost. |
| M14 | M5, M13 | Limit Haworth work to explicit selected topology/layout/depiction contracts until a future authoring/session contract is approved. |
| M15 | M5 | Retain only documented peptide, linear-form, and geometry workflows. New domain utilities require a source-backed native contract; dropped families must not acquire hidden fallback routes. |
| M16 | M10 | Maintain one ordinary Rust-native session/window for each supported route, typed refusal before mutation for unsupported routes, revision/digest fencing, thread-confined sessions, and no OASA host fallback. |
| M17 | M16 | Preserve V1 schema/version/error/resource-limit semantics. New operations require an explicit versioned contract, including transport/admission ownership. |
| M18 | M17 | Public Python and CLI surfaces are generated/checked from the frozen contract. Human convenience verbs may only construct protocol requests. |

### M19 integration closure

**Depends on:** M18, M14, M15.

**Exit condition:** every M1b capability-matrix row classified **supported** has
the required semantic Rust test, fast offline Python test, or real E2E lane. Each
known defect, refusal, and pre-production drop has a documented disposition. A lane
may be replaced if it violates `PYTEST_STYLE.md`, but not silently removed.

**Completion record (2026-08-19):**

- T1-T4 installed the macOS arm64 CI workflow, completed hygiene and plan compression,
  and closed the Rust `unsafe` boundary; remote GitHub Actions execution remains pending.
- T5-T9 added `chemistry.convert` and `document.generate_coordinates`, the six
  protocol-only CLI verbs, semantic CLI E2E, and worked command help.
- T10-T21 renamed the frontend module surface, centralized the extension adapter,
  documented worker semantics, restored declaration/action/mode/widget/keybinding/dialog
  seams, moved owned geometry, and recorded frontend convergence.
- T22-T26 completed keyboard-only authoring and accessibility structure, user-facing
  refusals, the keyboard E2E, and the task-first documentation/API contract register.
- T27 moved owner logic into the lower crates and left `ferrum-api` as the small delivery
  boundary. Protocol and all six verbs retained their black-box behavior through this move.

The final local gate is recorded in
[ferrum_convergence_final_20260819.md](reports/ferrum_convergence_final_20260819.md).
Performance observations may guide future work but carry no invented threshold.
Third-party plugin support remains an intentional pre-production drop.

### M20 packaging and platform matrix

**Depends on:** M18 and M4a; closes after M19.

For each admitted target, build the selected Ferrum-Chem and Ferrum-Qt wheels from
recorded inputs, install them no-index into a scrubbed environment, execute an
installed chemistry request, load package-owned Qt resources, and repeat the chemistry
request after an LGPL relink. Library discovery must work through the packaged link
layout (for macOS, `@loader_path`), not `DYLD_LIBRARY_PATH`. A local final macOS arm64
Ferrum-Chem wheel and validated CLI engine bundle now exercise both chemistry verbs;
this is implementation evidence, not a final two-wheel release receipt or a remote-CI
result.

### M21 WASM proof

**Depends on:** M4b and M17.

Build the project's MinimalLib WASM target and run the frozen request set through it
and native Ferrum. Use exact comparison for discrete closed outputs and documented
semantic/geometric invariants across differing RDKit builds. If WASM needs a
browser-shaped concept in `ChemEngine`, record that divergence rather than weakening
the native trait.

### M22 establish as supported product

**Depends on:** M19 and M20.

After those milestones, produce a committed-release source archive, final wheels,
source-archive Cargo CLI run, classified artifact inventory, migration/install docs,
and human legal/release review. The artifact must contain no OASA/Tk runtime and no
Python RDKit dependency, while retaining intentional RDKit closure, notices,
provenance, CDML heritage, and a Ferrum About acknowledgement of BKChem lineage.
M21 is nonblocking. Source mechanisms alone do not satisfy this exit.

## Workstream ownership

| Workstream | Scope | Provides |
| --- | --- | --- |
| WS-A | M1c, M2, M3 | Core model and deterministic graph behavior |
| WS-B | M4a-M5 | Chemistry adapter, codecs, and package mechanism |
| WS-C | M11-M13 | Geometry and checked render plans/backends |
| WS-D | M6-M10 | Authoritative CDML document/session semantics |
| WS-E | M14-M15 | Bounded domain capability |
| WS-F | M1b, M16-M19 | Desktop adoption, protocol, bindings, CLI, matrix closure |
| WS-G | M1a, M1d, M1e, M4c, M10, M20-M22 | Evidence, packaging, release closure |

No workstream may change another's public boundary without an explicit contract
decision and an owner review. The API crate/bindings/desktop are WS-F review scope;
native adapter/build tooling are WS-B/WS-G joint review scope.

## Acceptance and verification gates

- **Patch gate:** run the affected Rust/Python checks, formatting, and clippy. New
  durable logic receives the smallest stable semantic test that meets
  [pytest policy](../PYTEST_STYLE.md).
- **Thin-workflow gate:** from M8a onward, the simple open/coordinate/render/save
  workflow passes on relevant milestone exits.
- **Preservation gate:** committed corpus documents round-trip structurally with the
  [coverage inventory](audits/cdml_preservation_coverage.md). Backend output is the
  verdict; frontend reconstruction is not allowed to mask loss.
- **Boundary gate:** every new parser, FFI, external input, or package route records
  trust boundary, validation owner, resource limits, typed malformed-input failure,
  and adversarial coverage before acceptance.
- **Independent review:** M4a, M5, M10, M13, M16, and M19 closure receive review by
  an agent who did not implement the reviewed scope.

### Comparison rules

| Output | Rule |
| --- | --- |
| Coordinates | Measured tolerance for one recorded RDKit build; invariants across builds |
| Atom/bond fields and orders | Exact for carried discrete facts |
| InChI/InChIKey | Exact deterministic strings |
| Canonical SMILES/SMARTS | Exact within one recorded build; semantic round trip across versions |
| Molblock/SDF | Semantic equivalence |
| CDML/SVG | Structural equivalence under stated normalization |
| Render DTOs | Ordered typed semantics, exact discrete facts, finite `f64`; no JSON-byte rule |
| Glyph/raster/straighten observations | Current-target report evidence; no invented pixel/timing threshold |

Tolerance procedure: measure repeated oracle variation, measure supported-platform
variation, set a threshold outside the noise floor, record the source, and state
exact equality only for discrete/zero-variation facts. A threshold without that record
does not protect an exit gate.

## Test placement

- `cargo test`: fast Rust unit/property/semantic checks.
- `pytest tests/`: deterministic, offline, inline/small tests only.
- `tests/e2e/e2e_*.py`: real RDKit, wheel, installed CLI, corpus, GUI, oracle,
  packaging, and WASM observations.
- `devel/` plus reports: one-time measurements and walkthrough receipts.

Never put a subprocess, build, network path, large shared fixture, or timing
expectation in the permanent pytest lane. A current test that violates this policy is
replaced by a suitable semantic or E2E lane before M19 calls the row verified.

## Migration and release policy

Ordinary `ferrum-qt` is Rust-native. There is no compatibility editor, OASA host, or
silent legacy fallback. Unsupported historical workflows are refused before mutation
or recorded as pre-production drops. Ferrum uses `Ferrum` QSettings and
`~/.ferrum/templates`; it makes no historical preference/template migration promise.
CDML on-disk compatibility is maintained structurally, not lexically.

The release checklist is intentionally short:

- [x] Thin workflow and preservation gate established.
- [x] Production manifests remove OASA and Python RDKit.
- [x] M19 capability matrix reconciled and all supported rows verified locally.
- [ ] M20 admitted-target wheel/install/relink receipts complete.
- [ ] Release inventory proves OASA/Tk/Python-RDKit absence and native notices.
- [ ] Install/migration/provenance documentation reflects the released artifact.
- [ ] Human legal and release review completed.
- [ ] GitHub Actions has run the new workflow against this convergence commit.

## Remaining active decisions

| Decision | Owner | Required before |
| --- | --- | --- |
| Supported release targets and their recorded build inputs | `maintainer` | M20 target admission |
| WASM-native divergence, if any | `expert_coder` | M21 exit |
| Release artifact / migration wording and legal sign-off | `maintainer` + human | M22 exit |

Update this file when status, an active decision, dependency, or exit criterion
changes. Put commands, corpus measurements, receipts, and completed implementation
detail in focused reports, then link them here.
