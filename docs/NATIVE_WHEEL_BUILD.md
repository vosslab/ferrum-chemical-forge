# Native wheel build process

`build.sh` is the developer entry point for local Ferrum artifacts. It owns source
admission, compiler staging, paired wheel publication, cleanup, and disk admission.
Do not invoke the native builder directly with an `output_native_wheel/native-*`
destination: the retired rotating-output layout is deliberately rejected.

## Build commands

The supported commands are:

```bash
./build.sh all
./build.sh wheels
./build.sh cli
```

`all` is the default. It builds the CLI and the paired native and Qt wheels. `wheels`
builds and publishes the pair without the CLI. `cli` builds only `build/bin/ferrum` and
does not publish a wheel. There are no `native` or `qt` targets.

Native source selectors apply to `all` and `wheels` and are mutually exclusive:

```bash
./build.sh wheels --native-sealed-input-root /absolute/path/to/sealed-native-input-root
./build.sh wheels --native-source-archive-root /absolute/path/to/native-source-archives
```

Without a selector, the builder creates a hash-verified, invocation-scoped archive cache
under `build/native-source-archives/`. It is removed when the build succeeds, fails, or
receives `TERM`, `INT`, or `HUP`. Explicit input roots are never removed; they are the
reproducible offline-input contract.

## Publication contract

Each wheel build stages below the build-owned `build/native-staging/` and
`build/qt-staging/` roots, then publishes exactly one immutable native-plus-Qt pair.
Select it only through:

```text
output_native_wheel/current/
```

`current` is atomically replaced after validation. It is not a mutable build directory
and it is not safe to select old timestamped or `native-*` outputs. The selected pair
contains:

- `wheelhouse/ferrum_chem-*.whl`
- `wheelhouse/ferrum_qt-*.whl`
- `native-wheel-build-receipt.json`
- `developer-wheel-publication-receipt.json`
- `ferrum-engine-bundle/`

The publication receipt uses schema `ferrum-developer-wheel-publication-v4`. It binds
the exact native and Qt wheel digests, native receipt, engine-bundle manifest, Rust
source closure, and the admitted, staged, and final Qt worktree source-closure evidence.
The builder rechecks Qt worktree closure immediately before the atomic `current` swap;
live-worktree drift refuses publication and leaves the prior selected pair intact.

The canonical Qt staging inventory excludes generated `build/`, egg-info, caches, and
bytecode. Every delivered `ferrum_qt/**` wheel member must map byte-for-byte to an
admitted staged source member. Admitted sources may intentionally be absent from the
wheel. The wheel's one generated dist-info tree is outside that payload boundary.

Use the pair receipt as the selector for any installation or acceptance work. Do not
combine the current native wheel with a Qt wheel from `build/wheelhouse/` or another
publication.

## Storage, budget, and cleanup

Before a wheel build, `build.sh` removes only build-owned obsolete state: retired
`output_native_wheel/native-*` worktrees, unpublished or retired publications, native
staging roots, and the managed archive cache. It preserves `current` until the new pair
is fully validated and atomically selected.

Every non-help command measures the checkout with `du -sk`. `all` and `wheels` perform
their owned cleanup first, then refuse to start native compilation when the checkout
exceeds 20 GiB. The repository pytest guard enforces the same limit. An over-budget
failure means some non-current generated data needs ownership-aware cleanup; do not
delete source or the selected publication to bypass the guard.

Wheel builds acquire `build/native-build.lock`. The lock has an acquisition-specific
owner token, so a waiting build cannot remove another build's state. On normal exit,
failure, `TERM`, `INT`, or `HUP`, the active build clears only its staging, managed
cache, unpublished candidate, retired payload, and lock. Signal cleanup retains the
valid `current` pair.

## Install local artifacts

After a successful paired build, install only the selected pair and its matching engine
bundle:

```bash
source source_me.sh && python3 -m pip install --force-reinstall --no-deps \
  /absolute/path/output_native_wheel/current/wheelhouse/ferrum_chem-*.whl \
  /absolute/path/output_native_wheel/current/wheelhouse/ferrum_qt-*.whl
build/bin/ferrum engine install \
  /absolute/path/output_native_wheel/current/ferrum-engine-bundle
build/bin/ferrum engine status
ferrum-qt
```

This is a developer artifact flow. The broader release wheelhouse process is owned by
`packages/ferrum-rust/tools/build_release_wheelhouse.py`.

## Verification boundaries

`tests/e2e/e2e_build_sh_native_wrapper.sh` exercises wrapper cleanup, lock ownership,
disk refusal, publication preservation, and signal handling without a native compile. It
runs from `./all_test.sh` after the Python and Qt suites. Focused builder fixtures exercise
atomic replacement failure and source-closure refusal.

Run the installed dual-wheel Qt E2E against the exact selected pair after a successful
wheel build:

```bash
source source_me.sh && python3 packages/ferrum-chem-qt.app/tests/e2e/e2e_blank_canvas_direct_bond.py \
  --native-wheel "$PWD"/output_native_wheel/current/wheelhouse/ferrum_chem-*.whl \
  --qt-wheel "$PWD"/output_native_wheel/current/wheelhouse/ferrum_qt-*.whl
```

The E2E uses a fresh temporary virtual environment with system site packages only for the
PySide6 runtime. It installs the selected Ferrum wheels with `--ignore-installed --no-deps`,
proves that `ferrum_qt` originates from that fresh pair, verifies the complete Qt package
member set and bytes, then exercises public UI behavior. These checks prove bounded
contracts; they do not make a successful build or E2E claim until those commands run.

Run the final live-SMARTS artifact-pair E2E against the same selected pair, matching
engine bundle, and CLI:

```bash
source source_me.sh && python3 tests/e2e/e2e_smarts_final_live_combined.py \
  --native-wheel "$PWD"/output_native_wheel/current/wheelhouse/ferrum_chem-*.whl \
  --qt-wheel "$PWD"/output_native_wheel/current/wheelhouse/ferrum_qt-*.whl \
  --bundle "$PWD"/output_native_wheel/current/ferrum-engine-bundle \
  --cli "$PWD"/build/bin/ferrum
```

This is an explicit release/acceptance E2E, not a permanent `./all_test.sh` gate. Its
single receipt preserves isolated CLI, PyO3, and real-Qt live-SMARTS evidence for the
exact current publication without mutating the published artifacts.
