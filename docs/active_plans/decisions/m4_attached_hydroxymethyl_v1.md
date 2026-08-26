# M4 attached Hydroxymethyl V1 decision

## Context

The delivered attached compact-group catalog supports `Me`, `NO2`, `Et`, `OMe`,
and `CH2OH`. This delivered decision records `Hydroxymethyl` as the fifth
bounded M4 recipe.

The active milestone authority remains
[compact_group_authoring_v1.md](../active/compact_group_authoring_v1.md).

## Objectives

- Add one catalog-owned attached recipe without expanding the generic delivery
  architecture.
- Preserve the Rust-owned generic catalog, session, transport, and Qt chooser
  boundaries.
- Keep permanent evidence semantic, offline, fast, and inline.

## Design philosophy

This applies **Fix the design, not the symptom** and **Design for
adaptability**. The closed Rust catalog owns chemistry, label, authoring
capability, and attachment semantics. PyO3 and Qt transport Rust-issued choice
facts and retain no key-specific chemistry.

## Scope

- Persisted catalog key: `hydroxymethyl`.
- Rust-issued visible label: `CH2OH`.
- Neutral topology: `R-CH2-OH`.
- Recipe atom order: neutral `attachment_carbon` (`C`) at `(0.0, 0.0)`, then
  neutral `hydroxyl_oxygen` (`O`) at `(24.0, 0.0)`.
- One normal single internal bond joins `attachment_carbon` to
  `hydroxyl_oxygen`.
- The exterior bond rewires to `attachment_carbon`; that carbon's durable ID is
  the returned materialization focus.
- Materialization atomically replaces exactly one attached compact group and
  retains the exterior bond's durable identity, order, and presentation.
- The catalog adds this recipe to the existing attached-authoring keys. Generic
  session, capacity, materialization, protocol, PyO3, Qt, history, and reopen
  contracts remain unchanged.
- The accessible unavailable-choice wording derives the selected Rust-issued
  label or stays label-neutral through the typed refusal path. It must not
  retain a hard-coded `Me cannot attach ...` message.

## Non-goals

- Free `Hydroxymethyl` placement; free placement remains `Me`-only.
- A public attachment CLI or protocol operation.
- Aliases, formula input, a label parser, runtime chemistry parsing, or a
  frontend chemistry switch.
- Aromatic, triple-bond, carbonyl, or broader M4/M5 catalog work.
- Explicit-hydrogen or zero-charge attributes. Ordinary neutral typed-CDML
  `C` and `O` plus normal single-bond topology retain implicit-hydrogen
  semantics.

## Permanent evidence

- Extend existing inline catalog recipe coverage with one `Hydroxymethyl` row
  proving neutral C-O topology, carbon attachment role, and unsupported-key
  refusal. Do not assert recipe-array size or depiction coordinates.
- Extend existing attached-session/materialization parameterization only as
  needed to prove exterior-bond identity retention, carbon returned focus,
  atomic commit/refusal, and the already-covered history/reopen contract.
- No key-specific PyO3 or Qt test is added: existing generic transport remains
  the boundary, and the delivered Rust semantic tests prove the new row.

These are permanent only because they are deterministic, offline, fast, inline,
and protect durable semantic contracts. Do not add a test module, fixture,
network connection, subprocess pytest test, timing assertion, pixel/byte
comparison, or per-recipe lifecycle suite.

## One-time implementation checks

- Rebuilt the native Rust targets and ran the changed focused suites, full
  document-model and document suites, formatting, check, and clippy gates.
- Inspect representative rendered `CH2OH` orientation, label bounds, hit
  target, selection, and close-release pose in a disposable production-shaped
  Qt walkthrough.
- Probe capacity and geometry boundaries; retain a screenshot only as review
  evidence when useful.

No new public Qt E2E is added. Existing public authoring and materialization
E2Es cover the generic workflow. A later E2E requires a specific, recurring
public contract that lower-layer semantic evidence and the existing generic
workflow do not cover.

## Implementation handoff

The implementation owner adds the immutable recipe and attached-authoring key,
performs the row-level chooser review, and corrects generic refusal presentation
if required. The delivery close-out owner updates durable API documentation and
`docs/CHANGELOG.md`; neither is changed by this decision record.
