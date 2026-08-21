# Native wheel build process

`build.sh` is the sole developer entry point for a local Ferrum native wheel. It owns
native source inputs, compiler staging, publication, cleanup, and disk admission. Do
not invoke `build_native_wheel.py` with an `output_native_wheel/native-*` destination:
that legacy rotating-output layout is rejected deliberately.

## Build commands

The default command builds the CLI, native wheel, matching engine bundle, and Qt wheel:

```bash
./build.sh
```

Use a single target when only that artifact is needed:

```bash
./build.sh cli
./build.sh native
./build.sh qt
```

Run `./build.sh --help` for all supported target and input combinations. Native input
selectors are mutually exclusive:

```bash
./build.sh native --native-sealed-input-root /absolute/path/to/sealed-native-input-root
./build.sh native --native-source-archive-root /absolute/path/to/native-source-archives
```

## Source input policy

Without a selector, the native builder creates a profile-scoped, hash-verified source
archive cache at `build/native-source-archives/`. It downloads only missing pinned
archives for that invocation, then removes the managed cache when the native invocation
finishes, fails, or receives `TERM`, `INT`, or `HUP`.

The managed cache is therefore not an offline reuse mechanism. For a reproducible
offline build, provide exactly one explicit input root:

- `--native-sealed-input-root` uses one builder-validated sealed input root.
- `--native-source-archive-root` uses a local directory of pinned source archives.

`build.sh` never removes either explicit input root. They are the durable source-input
contract; all managed caches are intentionally temporary.

## Storage contract

Each native build compiles only below `build/native-staging/`. The published artifact
is one immutable payload selected through this stable path:

```text
output_native_wheel/current/
```

`current` is an atomically replaced symbolic link, not a build worktree. It always
selects either the complete previous publication or the complete new publication. The
selected payload contains:

- `wheelhouse/ferrum_chem-*.whl`
- `native-wheel-build-receipt.json`
- `ferrum-engine-bundle/`

The CLI and Qt artifacts are published separately at `build/bin/ferrum` and
`build/wheelhouse/ferrum_qt-*.whl`. Use the exact paths printed by the successful build;
do not choose a timestamped wheel from an old output directory.

Before every native build, `build.sh` removes only build-owned obsolete state:

- legacy `output_native_wheel/native-*` worktrees
- unpublished or retired hidden native publications
- prior native staging roots
- the managed native source-archive cache

It preserves `output_native_wheel/current/` until a new publication has passed all
artifact, receipt, engine-bundle, and copied-payload source-closure validation. The
receipt records one canonical source-subset manifest from the completed
`maturin-project/` staging tree, including the deterministic Maturin include and rpath
transforms. It excludes only the builder-owned staged notice bundle and package
`.dylibs` closure; wheelhouse, Cargo output, and the engine bundle are siblings outside
that tree. Every other staged regular file is an admitted Ferrum source. The wrapper
recomputes this exact manifest and checks the copied wheel digest and filename against
the copied receipt immediately before the atomic `current` pointer replacement. It also
parses the copied engine-bundle manifest and requires its exact regular-file member set
and SHA-256 values.

## Disk budget

Every non-help `build.sh` command measures the complete checkout with `du -sk`. Native
targets first complete their owned cleanup, then the command refuses to start Cargo or
the native builder when the checkout exceeds 20 GiB. The diagnostic includes `du -sh`
output and remediation.

The repository also has a pytest budget guard for the same 20 GiB checkout limit. The
build gate prevents new compiler work from worsening a large checkout; the pytest guard
keeps accumulated generated data from being accepted unnoticed.

An over-budget failure means state outside the one current publication remains. Remove
only non-source, non-current generated data after identifying its owner, then rerun the
same `build.sh` command. Manual cleanup is exceptional: ordinary native builds reclaim
their own temporary state automatically.

## Concurrency and signals

Native builds acquire `build/native-build.lock` before preflight cleanup. The lock has
an acquisition-specific owner token, so a waiting build cannot remove another build's
staging tree, publication candidate, or lock. A stale lock is recovered only when its
recorded process is absent.

On normal completion, failure, `EXIT`, `TERM`, `INT`, or `HUP`, the active build cleans
only its own staging root, managed cache, unpublished candidate, and retired payload.
It leaves the valid `current` publication intact and releases its owned lock last.

## Install local artifacts

After a successful default build, install the published native and Qt wheels, then
install the matching bundle for the CLI:

```bash
source source_me.sh && python3 -m pip install --force-reinstall --no-deps \
  /absolute/path/output_native_wheel/current/wheelhouse/ferrum_chem-*.whl \
  /absolute/path/build/wheelhouse/ferrum_qt-*.whl
build/bin/ferrum engine install \
  /absolute/path/output_native_wheel/current/ferrum-engine-bundle
build/bin/ferrum engine status
ferrum-qt
```

This is a developer artifact flow. The broader release wheelhouse process is owned by
`packages/ferrum-rust/tools/build_release_wheelhouse.py`.

## Lifecycle verification

`tests/e2e/e2e_build_sh_native_wrapper.sh` verifies the build contract without a real
native compiler run. It covers one retained current publication, stale-output cleanup,
builder-failure preservation, copied wheel, receipt, and engine-bundle mutation refusal before the
pointer swap, disk-budget refusal before a builder starts, lock contention and ownership,
and cleanup across interruption before and after publication. The builder self-test
also proves that generated staged notices and `.dylibs` do not change the source subset,
while a staged authored-source mutation fails publication validation.
