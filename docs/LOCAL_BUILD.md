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

Compiler intermediates are created only in `build/.cargo-target` and removed on
success, failure, or interruption. The final local CLI and extension remain in
`build/`; no per-invocation output roots, archive cache, wheelhouse, or global
installation is created. Before and after compilation, `build.sh` refuses a
checkout larger than 20 GiB.

One root `build.sh` invocation owns `build/` at a time. A second invocation
fails before changing shared build state, avoiding concurrent compiler-cache or
runtime staging corruption.
