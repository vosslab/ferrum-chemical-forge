# Code architecture

## Overview

Ferrum is a pre-production chemical-document editor with one authoritative
Rust model. The Rust workspace owns chemistry, CDML document state, validation,
history, rendering admission, and immutable render facts. The `ferrum` CLI and
the PySide6 desktop editor consume that same model through narrow boundaries;
the desktop application does not maintain a second chemistry or label-layout
implementation.

The repository keeps historical BKChem and OASA material only as ignored
reference evidence under `OTHER_REPOS/`. It is not a runtime, packaging, or
product dependency. The current parity ledger is
[active_plans/active/FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

## Major components

- [../packages/ferrum-rust/](../packages/ferrum-rust/) is the single
  Ferrum-Chem Cargo workspace. Its [Cargo.toml](../packages/ferrum-rust/Cargo.toml)
  defines the shared Rust version, edition, license, and lint policy.
- [../packages/ferrum-rust/crates/core/](../packages/ferrum-rust/crates/core/),
  [../packages/ferrum-rust/crates/chemistry/](../packages/ferrum-rust/crates/chemistry/),
  [../packages/ferrum-rust/crates/domain/](../packages/ferrum-rust/crates/domain/),
  and [../packages/ferrum-rust/crates/geometry/](../packages/ferrum-rust/crates/geometry/)
  own chemistry-domain types, chemistry integration, shared domain facts, and
  geometry. They do not own Qt presentation.
- [../packages/ferrum-rust/crates/document/](../packages/ferrum-rust/crates/document/)
  owns the CDML-facing document session, mutation admission, history, and the
  immutable `DocumentRenderObservationV2` entry point. Its document projection
  is the authority passed to rendering; a render consumer cannot reopen or
  mutate a session through an observation.
- [../packages/ferrum-rust/crates/document-model/](../packages/ferrum-rust/crates/document-model/),
  [../packages/ferrum-rust/crates/document-projection/](../packages/ferrum-rust/crates/document-projection/),
  and [../packages/ferrum-rust/crates/document-render/](../packages/ferrum-rust/crates/document-render/)
  contain shared durable document records, immutable projection facts, and
  document-to-render interaction preparation. The separate
  [../packages/ferrum-rust/crates/graph-lowering/](../packages/ferrum-rust/crates/graph-lowering/)
  crate lowers capability-free projection facts into chemistry graphs without
  taking document-session, renderer, API, or PyO3 dependencies.
- [../packages/ferrum-rust/crates/render/](../packages/ferrum-rust/crates/render/)
  owns render-plan construction and exact rendering geometry. Its V4 plan is a
  closed batch grammar: each target has either a complete `RenderBatchV4` or a
  typed `RenderIssue`, never both. Atom, compact-group, and bond batches have
  distinct typed content. Batch and issue paint orders are globally ordered
  renderer facts, so consumers never invent an ordering rule.
- The renderer's [verified_telex_glyph_metrics.rs](../packages/ferrum-rust/crates/render/src/verified_telex_glyph_metrics.rs)
  validates the bundled Telex face and issues exact glyph runs, visible-ink
  bounds, and the structural core-element run. Qt replays those issued glyph
  facts; it does not substitute a system font or remeasure labels.
- The renderer's private `packages/ferrum-rust/crates/render/src/atom_bond/final_ink_collision.rs`
  admits a bond only after its complete lowered ink is disjoint from every
  non-endpoint atom-label envelope. It works from closed bond operations,
  including stroke and path geometry, rather than reconstructing an axis or
  querying screen pixels. An intersection emits an `UnrenderableTarget` issue
  instead of a partial or rerouted bond.
- Generic document authoring is admitted against the complete resolved render
  plans, not just a root-class summary. The candidate may retain or repair an
  existing imported diagnostic, but may not introduce a new root exclusion,
  plan issue, or member depiction issue. The opaque accepted value retains the
  exact candidate realization used for preview paint and is rederived at the
  one-use commit boundary.
- Undo and redo are a distinct private history policy: a retained history target
  is rederived and authenticated exactly, even when that older target contains
  an imported diagnostic. This preserves honest history without weakening the
  no-new-omission rule for ordinary authoring.
- [../packages/ferrum-rust/crates/api/](../packages/ferrum-rust/crates/api/)
  composes CLI and public native routes. [../packages/ferrum-rust/crates/api-python/](../packages/ferrum-rust/crates/api-python/)
  is the sole workspace owner of the built `ferrum_chem` PyO3 extension; its
  Python package configuration and binding tests live in
  [../packages/ferrum-rust/crates/api/python/](../packages/ferrum-rust/crates/api/python/).
- `packages/ferrum-rust/crates/api/src/python_binding/render_plan_binding.rs`
  and `packages/ferrum-rust/crates/api/src/python_binding/render_plan_content_binding.rs`
  convert the Rust `RenderObservationV2` and V4 batches once into frozen PyO3
  DTOs. Derived generic operation replay is only a convenience over the closed
  typed payloads; it is not an alternative semantic protocol.
- [../packages/ferrum-chem-qt.app/](../packages/ferrum-chem-qt.app/) owns the
  AGPL-licensed PySide6 application. Its [ferrum_render_projection.py](../packages/ferrum-chem-qt.app/ferrum_qt/canvas/ferrum_render_projection.py)
  accepts only the exact V2 observation schema and V4 plan schema, validates
  provenance and coordinate-space receipts, builds a detached graphics scene,
  and manages its disposal. It does not open CDML, call a document session, or
  interpret chemistry labels.

## Render data flow

The normal molecule display route is:

```text
Rust document session
  -> immutable document projection
  -> verified-Telex renderer and final-ink bond admission
  -> RenderObservationV2 containing V4 molecule plans
  -> frozen ferrum_chem PyO3 DTOs
  -> Qt exact-schema validation
  -> disposable QGraphicsScene and FerrumPlanItem objects
```

Each molecule plan carries its durable document-root identity, source revision,
and digest. Qt retains durable selection keys only for Rust-issued targets and
disposes the previous scene only after a new observation passes validation and
is installed. Render issues remain typed diagnostics with no graphics item.

Atom labels are a renderer-to-Qt contract: the atom-local anchor, optional mask,
text runs, glyph IDs, run origins, exact full/core ink bounds, and core-element
run index originate in Rust. Bond clipping and final-ink refusal happen before
the plan crosses the binding. The Qt scene only paints the accepted operations.

Native linear-form spacing is a domain-to-document contract.
`LinearFormBondLength::NATIVE` owns the exact 40-point spacing used by the
planner and the exact `bond_length=40` `IntType` CDML metadata token. Imported
nonmatching forms remain preservation-only; there is no second writable 10-point
grammar or renderer-side spacing repair.

Direct-root presentation (text, plus signs, arrows, and vectors) follows the
same immutable-observation pattern but remains distinct from molecule batches.
The Qt projection may compose its detached presentation scene with the molecule
scene; it does not merge ownership of document or molecule facts.

## Desktop and command flow

The Qt package entry point is [../packages/ferrum-chem-qt.app/ferrum_qt/cli.py](../packages/ferrum-chem-qt.app/ferrum_qt/cli.py),
which creates the ordinary application and window through
[../packages/ferrum-chem-qt.app/ferrum_qt/app.py](../packages/ferrum-chem-qt.app/ferrum_qt/app.py).
Feature modules under [../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/)
collect user intent, call a typed PyO3 operation, and refresh from a replacement
observation or present a typed refusal. Canvas, dialog, action, theme, and
resource packages own Qt concerns only.

Local document opening is intentionally split across
[local_document_open_contract.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_contract.py),
[local_document_open_composition.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_composition.py),
[local_document_open_controller.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_controller.py),
[local_document_open_delivery.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_delivery.py),
and [local_document_open_host.py](../packages/ferrum-chem-qt.app/ferrum_qt/ferrum/local_document_open_host.py).
Those modules separate immutable facts, window composition, request/lease
orchestration, staged delivery, and the single publication/replacement
transaction. Rust remains the authority for file admission and document
semantics.

## Testing and verification

- The canonical alignment corpus is
  `packages/ferrum-rust/crates/document/tests/fixtures/atom_label_bond_alignment_cases_v1.json`.
  Its Rust consumer, `packages/ferrum-rust/crates/document/tests/atom_label_bond_alignment_corpus.rs`,
  proves the semantic document-to-V4 boundary.
- `packages/ferrum-chem-qt.app/tests/test_atom_label_bond_alignment_corpus.py`
  consumes the same rows through the installed PyO3 DTOs and production Qt
  projection. It checks exact Telex replay, ordered operations, accepted bond
  disjointness, and the target-specific third-label refusal without duplicating
  a fixture table in Python.
- `packages/ferrum-chem-qt.app/tests/test_ferrum_render_projection.py`
  exercises the projection boundary, including interleaved batch and issue
  paint order. Rust crate tests cover renderer and document invariants.
- [../check_rust.sh](../check_rust.sh), [../build.sh](../build.sh), and
  [../all_test.sh](../all_test.sh) are repository-provided aggregate routes.
  GUI screenshots and human visual/accessibility review remain separate
  evidence lanes; passing unit tests does not replace either review.

## Extension points

- Add chemistry or durable document behavior in its responsible Rust crate,
  then expose only a validated immutable projection or typed mutation result.
- Add rendering behavior in [../packages/ferrum-rust/crates/render/](../packages/ferrum-rust/crates/render/)
  before touching PyO3 or Qt. A new label or bond form must be lowered,
  measured, admitted, and represented in the closed plan grammar by Rust.
- Add a PyO3 DTO only at the binding boundary and keep its name versioned only
  when it is a durable serialized or cross-boundary contract. Private Rust
  helpers remain unversioned.
- Add Qt behavior in its feature or canvas owner, consuming issued DTO facts
  rather than rebuilding a chemistry, font, or geometry model.
- Add a durable alignment case to the one JSON corpus, then make both its Rust
  and installed Qt consumers prove the same row.

## Known gaps

- Full BKChem/OASA feature parity remains an active migration goal; consult the
  parity ledger before presenting a bounded implemented slice as complete.
- Refresh native-window screenshots and obtain human visual/accessibility
  acceptance after renderer or Qt visual changes. Those checks are intentionally
  not reduced to pixel snapshots or unit assertions.
