# Local Build

`./build.sh` builds Ferrum for local development and testing. It does not
publish a wheel or install a package outside the checkout.

## Outputs

The command stages two local launchers below `build/`:

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
`build/bin/ferrum-qt`. Both launchers consume the staged local runtime directly;
they do not use a globally installed Ferrum package.

## Test workflow

Build before running the complete suite:

```bash
./build.sh
./all_test.sh
```

`all_test.sh` first runs repository hygiene checks. It then requires the local
extension created by `build.sh` and runs the PyO3 and Qt suites with that
extension ahead of any globally installed copy.

The runtime receipt seals the extension, native adapter, and both executable
launchers. Before importing Ferrum, `all_test.sh` validates that receipt,
checks the Qt launcher's shell syntax, runs the local CLI's bounded `--version`
smoke command and `C`-to-`C` conversion smoke, and executes the supported local CLI E2Es against
`build/bin/ferrum`: human CLI verbs and selected-molecule SDF V2000/V3000
export. These checks use the staged runtime only; they do not install Ferrum.
The runner also drives the offscreen Qt P0.2 root-selection workflow through the
staged extension: click and marquee selection, drag translation, undo, save,
and Rust reopen.

## Disk ownership

Ferrum separates retained local runtime products from disposable compiler work:

- `build/bin/` and `build/runtime/` are retained because they are the runnable
  local application.
- `build/.cargo-target/` is private compiler work for `build.sh`. The build
  cleanup removes it on success, failure, or interruption.
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

One root `build.sh` invocation owns `build/` at a time. A second invocation
fails before changing shared build state, avoiding concurrent compiler-cache or
runtime staging corruption.
