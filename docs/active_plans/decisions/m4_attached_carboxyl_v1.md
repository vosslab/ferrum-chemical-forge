# M4 attached Carboxyl V1 decision

## Context

The delivered attached compact-group catalog supports `Me`, `NO2`, `Et`,
`OMe`, `CH2OH`, and `Carboxyl`. This delivered decision records the sixth
bounded M4 recipe, `Carboxyl`; it does not close M4 or full parity.

The active milestone authority remains
[compact_group_authoring_v1.md](../active/compact_group_authoring_v1.md).

## Objectives

- Add one catalog-owned attached recipe without expanding the generic delivery
  architecture.
- Preserve Rust ownership of chemistry, session materialization, issued key and
  label facts, and the generic PyO3/Qt transport boundary.
- Keep permanent evidence semantic, offline, fast, and inline.

## Design philosophy

This applies **Fix the design, not the symptom** and **Design for
adaptability**. The closed Rust catalog owns the key, label, authoring
capability, and recipe chemistry. PyO3 and Qt transport Rust-issued facts; no
binding or Qt branch interprets `Carboxyl` chemistry.

## Scope

- Persisted catalog key: `carboxyl`; Rust enum variant: `Carboxyl`; visible
  label: `COOH`.
- Attached-only neutral topology: `R-C(=O)-OH`, with implicit hydrogens and no
  formal-charge fields.
- Deterministic local atoms: `attachment_carbon` at `(0, 0)`,
  `carbonyl_oxygen` at `(24, 18)`, and `hydroxyl_oxygen` at `(24, -18)`.
- One normal double C=O bond joins `attachment_carbon` to `carbonyl_oxygen`;
  one normal single C-O bond joins `attachment_carbon` to `hydroxyl_oxygen`.
- The existing exterior normal single bond rewires to `attachment_carbon`,
  which is the returned materialization focus, while retaining that bond's
  durable identity, style, and order.
- The immutable catalog adds `Carboxyl` to the existing attached-authoring
  keys and recipe support. Generic session capacity/materialization, Rust-issued
  key/label transport, PyO3 key parsing, and projection/render label paths
  remain generic.

## Non-goals

- Free Carboxyl placement.
- Carboxylate, pH-dependent, resonance, or alternative charge representations.
- Triple-bond or aromatic schema changes.
- `Cyano`, `Phenyl`, `AcylChloride`, or any other catalog selection.
- A PyO3 or Qt chemistry branch, chemistry label parser, alias, or new generic
  attached CLI/protocol operation.

## Delivered evidence

- Semantic inline catalog coverage proves the `Carboxyl` key/label, neutral
  atom facts, deterministic local topology, carbon focus, and normal
  single/double bond kinds.
- Session topology/materialization coverage proves exterior normal-single-bond
  rewiring retains durable identity, style, and order while the returned focus
  is the attachment carbon.
- The focused Rust catalog and attached-session validation reported passing:
  `cargo check`, targeted `cargo test`, `cargo fmt --check`, and `cargo clippy`
  for the affected document-model and document crates.

These delivered checks are deterministic, offline, and protect the
catalog/session contract. They do not add timing, pixel, byte, or per-recipe
history coverage when the existing generic lifecycle evidence already covers it.

## Completed validation evidence

- A fresh build produced `build/current/bin/ferrum`, `ferrum-qt`, and the
  installed Python runtime. The existing attached binding suite passed 8/8.
- The final `all_test.sh` gate passed: 7,637 hygiene checks, every named CLI and
  Qt E2E, 280 installed binding tests, and 214 Qt tests passed with one skip.
- One-time installed-Qt evidence selected the Rust-issued `carboxyl` / `COOH`
  chooser entry, selected the rendered group through public hit testing, and
  ran the production materialize action to terminal `succeeded` / `updated`.
- Exact `R-C(=O)-OH` topology and exterior-bond semantics remain permanent
  Rust/session evidence, their ownership layer. The Qt check proves the real
  integration workflow without duplicating private semantic inspection in GUI.

The raw one-time Qt receipt reports `FAIL` only because public Molecule Report
cannot independently expose the private bond topology. The acceptance
adjudication is `PASS`: a public topology report is future parity scope, not a
Carboxyl blocker, and no testing-only API or duplicate GUI test is added.

## Delivered implementation

The immutable Rust catalog now issues the `carboxyl` / `Carboxyl` / `COOH`
attached-authoring choice and its neutral `R-C(=O)-OH` recipe. The existing
generic materializer remains the sole owner of exterior-bond rewiring and
attachment-carbon focus. No candidate-specific PyO3, projection, renderer, or
Qt chemistry branch is introduced.

The two unselected attached catalog choices remain `AcylChloride` and `Phenyl`.
Cyano is now the delivered seventh attached recipe under its separate decision.
Free Carboxyl placement and every non-goal above remain outside this delivered
slice.

The historical handoff is complete. If a future selected recipe shows that the
generic materialization algorithm cannot represent its required topology, stop
catalog expansion and design a replacement algorithm before resuming.
