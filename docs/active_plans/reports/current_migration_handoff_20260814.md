# Ferrum migration account-switch handoff

## Purpose

This is the restart point for the 2026-08-14 manager run. The worktree is heavily dirty and
user-owned. Do not reset, clean, stage, or rewrite unrelated changes. The historical collaboration
tree is operationally quarantined; the collaboration API has no delete primitive, so use only
fresh task names after the account switch.

The active goal remains the complete Rust replacement of OASA, Ferrum-Qt adoption, CLI and Python
contract freeze, packaging, and final capability closure. The goal is not complete.

## Decisions fixed in this run

- Keep provisional `ferrum cdml render {svg,pdf,png}`. Do not add `render-svg`, extension
  inference, or an SVG-in-PDF implementation. M17 approves the operation protocol and M18 freezes
  the eventual CLI spelling, streams, exits, and E2E contract.
- Do not start M17 while M16 still has two document authorities. First make every retained
  supported document path use Rust `DocumentSession`, or record an explicit unsupported or
  known-defect decision in the capability matrix.
- Do not remove OASA imports piecemeal. Ordinary `ferrum-qt` is already OASA-free; the explicit
  compatibility host remains the owner of the legacy session island. Retire that whole host and
  dependency only after every retained capability has a native owner or a recorded drop decision.
- Pre-release persistence intentionally uses `Ferrum` / `Ferrum-Qt` QSettings and
  `~/.ferrum/templates`. There is no BKChem preference or template migration promise because the
  product has no production users. Historical provenance and real internal compatibility IDs stay.
- The next authority-transfer feature is native `linear-form.convert`. Its named
  `linear-form-direction-v1` contract starts at the lower durable source-order endpoint. This is an
  intentional persistent-CDML divergence from OASA's `(x, y, id)` endpoint ordering.

## Accepted work

The following linear-form layers are implemented and independently accepted:

1. Pure `ferrum-domain` planner: source-order direction, simple-path validation, fixed 10-point
   geometry, exterior-component translation, selected-hydrogen facts, and typed resource errors.
2. Collision-safe fragment ID allocator: opaque-ID checks, typed exhaustion, and tentative copied
   sequence state which does not advance the session until commit.
3. Typed-document adapter: direct-root extraction, canonical metadata repair/new-ID classification,
   reverse-owned record repair, mark and atom movement, z/opaque preservation, and fallible
   retirement/writer paths.
4. `DocumentSession` transaction: one-use pending receipt, precomputed observation, fallible token
   issuance, allocation-free post-consumption commit tail, deferred ID installation, no-op behavior,
   history, undo/redo, and save/reopen semantics.
5. Public in-process Rust API: revision/digest/direct-root authentication and immediate prepare to
   commit with closed changed/no-change results. It adds no CLI, wire, serde, or stable Python API.

Primary receipts are under
`/private/tmp/ferrum-manager-20260814-next-migration.B5h9nb/`:

- `design_native_linear_form_v1.d7e2.report.md`
- `review_native_linear_form_v1_design.83af.report.md`
- `linear_form_direction_oracle.5aa1.report.md`
- `implement_linear_form_domain_v1.114c.report.md`
- `review_linear_form_domain_v1.0d8f.report.md`
- `finish_linear_form_document_v1.61f0.report.md`
- `review_fragment_id_allocator_v1.51a2.report.md`
- `implement_linear_form_document_adapter_v1.3c7b.report.md`
- `review_linear_form_document_adapter_v1.1e6d.report.md`
- `implement_linear_form_session_v1.885e.report.md`
- `review_linear_form_session_v1.d3f4.report.md`
- `implement_linear_form_api_v1.9d0a.report.md`
- `review_linear_form_api_v1.287b.report.md`

## Current unaccepted checkpoint

The private PyO3 layer is implemented but **not accepted**. Current files include
`crates/api/python/src/document_linear_form_binding.rs`, its binding and module registration,
`crates/api/python/tests/test_document_linear_form_binding.py`, and exhaustive mappings for the new
linear-form session errors in `document_error_binding.rs`.

The binding remains absent from `crates/api/wheel_metadata/ferrum_chem.pyi`, CLI, wire, serde, and
public docs. It accepts borrowed Python strings plus an exact tuple of atom IDs, maps surrogates and
resource failures to `DocumentLinearFormError`, and returns the authoritative session result.

Checkpoint evidence:

- Nested `cargo check --manifest-path crates/api/python/Cargo.toml --locked --offline` passes.
- A fresh debug extension was built.
- The installed binding test was 7/8 before the final assertion-only correction.
- No fresh rerun or independent review occurred after that final correction.
- Checkpoint receipt is `implement_linear_form_pyo3_v1.f3a8.report.md` in the private report
  directory named above.

## Exact restart sequence

1. Read `AGENTS.md`, Rust/PyO3/Python/pytest/repository style guides, this handoff, and the accepted
   design and review receipts.
2. Inspect the exact PyO3 diff. Run nested format, check, test, strict Clippy, and rustdoc gates.
3. Rebuild or reuse only a provenance-matched fresh debug extension, then run
   `test_document_linear_form_binding.py` with Python 3.12 through `source source_me.sh`.
4. Assign a fresh independent PyO3 reviewer. Remediate findings and rerun the same evidence.
5. Only after PyO3 acceptance, implement the thin ordinary-native Qt action. Qt expands selected
   bonds to endpoint atom IDs, authenticates one direct root and current provenance, calls the
   private binding synchronously, reinstalls only changed observations, and restores accepted atom
   selection. The compatibility-host OASA route remains until its separate removal cut.
6. Independently review Qt behavior, then update the changelog, M15 narrative, FQ-010/FQ-018, and
   the OASA ownership ledger. Record the native-route retirement without claiming M15/M16 closure.

## Other audit receipts

The same private report directory contains the current OASA ownership, M15, M16/M17, CLI,
frontend-drift, and packaging audits. Their important conclusions are reflected above. M20 and M22
remain blocked by M16-M19 closure and the live compatibility host; do not start a product packaging
envelope or remove the OASA dependency yet.
