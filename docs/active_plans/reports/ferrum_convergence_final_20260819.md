# Ferrum convergence final verification

This report closes the local T1-T27 convergence implementation on 2026-08-19. It
records local evidence only; it does not claim that the GitHub Actions workflow has
run, that another platform is qualified, or that release/legal review is complete.

## Scope supersession

This report preserves the 2026-08-19 chronology. On 2026-08-22, the active plan
superseded wheel, installed-package, publication, and release work with the
local-development contract: `build.sh` stages the runnable program under `build/`,
`build/bin/ferrum` and `build/bin/ferrum-qt` use that staged runtime, and
`all_test.sh` is the repository acceptance suite. References below to wheels, engine
bundles, CI, or release gates are historical evidence, not current acceptance work.

## Delivered boundary

- The CI workflow calls `pytest tests/`, `all_test.sh`, and `check_rust.sh` on
  macOS arm64 after building and installing the controlled native dependency.
- The frozen pathless protocol now carries six operations through the human CLI:
  inspect, validate, rewrite, render, convert, and coordinate generation.
- The final local CPython 3.12 macOS arm64 `ferrum-chem` wheel produced a validated
  explicit engine bundle. Installing that bundle exercised both engine-dependent
  CLI verbs; absent or invalid bundles produce a completed typed refusal.
- The frontend is Ferrum with the restored declarative/action/mode/widget/dialog
  seams, one extension adapter, named worker ownership, Rust geometry ownership,
  keyboard-only authoring, accessibility structure, and user-facing refusals.
- T27 leaves `ferrum-api` as the delivery layer only. Chemistry, document, domain,
  geometry, and render implementations live in their owning crates with lower crates
  independent of `ferrum-api`.

## Final local results

| Gate | Result |
| --- | --- |
| `source source_me.sh && bash ./all_test.sh` | 5,916 repository tests passed; 213 installed binding tests passed; 393 Qt tests passed, 1 skipped |
| `bash ./check_rust.sh` | Formatting, checks, strict Clippy, tests, docs, and the separate PyO3 gates passed in the same convergence run |
| Six-verb CLI E2E | Passed with the final validated engine bundle and with the no-bundle typed-refusal path |
| Native wheel and engine bundle | CPython 3.12 macOS arm64 wheel and `ferrum-engine-bundle-v1` validated locally |
| `git diff --check` | Passed during the convergence verification |

The test counts are run receipts, not permanent count assertions. Semantic protocol, CLI,
binding, and Qt behavior tests remain the durable gates.

## Historical release limits

- GitHub Actions had not yet executed the new workflow remotely.
- The local macOS arm64 wheel/bundle evidence was not a cross-platform qualification.
- M20 then required recorded release-wheelhouse/install/relink evidence for every
  admitted target.
- M22 then required final release artifacts, source-archive and inventory checks, and
  human legal and release review.
