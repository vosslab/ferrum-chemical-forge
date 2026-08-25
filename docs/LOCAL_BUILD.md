# Local Build

`./build.sh` builds Ferrum for local development and testing. It does not
publish a wheel or install a package outside the checkout.

## Outputs

The command stages one immutable local program below `build/programs/` and
publishes it by atomically replacing `build/current`. The stable paths below
continue to be the developer interface:

Before staging, a locked build removes any obsolete direct `build/bin/` or
`build/runtime/` directory. A supported local runtime is created only by the
current leased-program build flow; direct-layout artifacts are disposable.

- `build/bin/ferrum` is the Rust CLI.
- `build/bin/ferrum-qt` starts the source Qt application with the locally built
  `ferrum_chem` extension.
- `build/runtime/python/ferrum_chem<Python ABI suffix>` is that extension for
  local Python and Qt tests, for example `ferrum_chem.cpython-312-darwin.so`.
- `build/runtime/python/.dylibs/` contains the extension's local dynamic-library
  closure.
- `build/runtime/engine-v1/` contains the sealed chemistry runtime consumed by
  `build/bin/ferrum`.

Run the CLI with `build/bin/ferrum` or the desktop application with
`build/bin/ferrum-qt`. Both launchers and the Python runtime resolve through
`build/current`, so one pointer always selects the complete sealed program.
They do not use a globally installed Ferrum package.

## Test workflow

Build before running the complete suite:

```bash
./build.sh
./all_test.sh
```

`all_test.sh` first runs repository hygiene checks. It then sources
`source_me.sh`, which owns the local GUI import order: repository Qt source
first, sealed local extension runtime second, then retained caller entries.
The PyO3 and Qt suites preserve that order, preventing globally installed
Ferrum modules from replacing the source Qt application or sealed extension.

The runtime receipt seals the extension, native adapter, both lease wrappers,
and the CLI and Qt payloads they execute. Before importing Ferrum, `all_test.sh`
validates that receipt, checks the Qt launcher's shell syntax, runs the local
CLI's bounded `--version` smoke command and `C`-to-`C` conversion smoke, and
executes the supported local CLI E2Es against `build/bin/ferrum`: human CLI
verbs and selected-molecule SDF V2000/V3000 export. These checks use the
staged runtime only; they do not install Ferrum.
The runner also drives the offscreen Qt P0.2 root-selection workflow through the
staged extension: click and marquee selection, drag translation, undo, save,
and Rust reopen.

## Disk ownership

Ferrum separates retained local runtime products from disposable compiler work:

- `build/programs/<opaque-id>/` is one immutable runnable local program.
  `build/current` selects it atomically, while `build/bin/` and
  `build/runtime/` are stable links through that pointer.
- `build/.cargo-target-<opaque-id>/` is private compiler work for one
  `build.sh` owner. Cleanup removes it on success, failure, interruption, or
  the next locked build startup.
- `build/.cargo-check-target/` is private compiler work for `check_rust.sh`.
  That checker removes it after its gate finishes.

All supported Rust front doors use disposable target work below `build/`.
Package-local `packages/ferrum-rust/target/` directories, including nested
PyO3 target directories, are not build outputs. `build.sh` retains no
per-invocation output roots, archive cache, wheelhouse, or global installation.
Before and after compilation, it refuses a checkout larger than 20 GiB.

The cleanup contract has focused synthetic temporary-root coverage. A one-time
post-build measurement recorded a 1.2 GiB checkout with a 591 MiB `build/`
directory and no source-package target directories; that observation informs
capacity planning but is not a permanent machine-dependent test.

One root `build.sh` invocation owns `build/` at a time through an operating
system advisory lock held by a small owner process on the stable
`build/.build.lock` inode. The build shell and Cargo/Python descendants do not
inherit that lock descriptor. A second invocation fails before changing shared
build state; when the owner exits, including interruption, its lock releases
even if an interrupted compiler descendant is still winding down in its own
per-owner target root. Lock metadata is never used to decide ownership or
reclaim a build.

Every generated `ferrum` and `ferrum-qt` launcher takes a shared advisory lock
on its immutable program root's `.ferrum-runtime.lease` inode before it execs
the CLI or Python Qt process. The inherited descriptor keeps that lease through
the complete runtime lifetime. Cleanup attempts a nonblocking exclusive lock on
the same inode: it preserves a shared-held root and fails safe on indeterminate
lock errors. A non-current root with a missing, non-regular, or unreadable
lease is malformed local build output and is removed; a valid exclusive lease
is also removed. Lease state is the ownership authority; process/path scans and
metadata are diagnostic only.

Every locked startup and post-promotion cleanup considers only inactive,
non-current `program-*` roots before the checkout-size guard and compilation.
It never traverses unrelated `build/programs/` content or mutates a published
root through `build/current`, `build/bin`, or `build/runtime`. Candidate-native
temporary work is removed only while it remains in staging; an inactive owned
root is reclaimed as a whole. This recovers a root stranded by a crash after
its staging rename but before `build/current` is replaced.
