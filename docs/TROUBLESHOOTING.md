# Troubleshooting

## Use the supported Python

### `python3` uses the wrong environment

Run Python commands from the repository root after loading the repository shell
environment:

```bash
source source_me.sh && python3 -B -m pytest
```

`source_me.sh` requires Bash, sources the local shell setup first, and then sets
the repository's unbuffered-output and no-bytecode defaults. Use the
repository's Python 3.12 environment. It must be sourced, not executed.

### `source_me.sh` says that the local runtime is unavailable

The development shell deliberately refuses to import a global or stale
`ferrum_chem` extension. If it reports either `Ferrum local runtime is
unavailable` message, rebuild the sealed local program, then source the shell
setup again:

```bash
./build.sh
source source_me.sh && python3 -B -m pytest
```

Do not prepend a globally installed Ferrum package to `PYTHONPATH` to work
around the error. The supported order is source Qt code, then the extension
under `build/runtime/python`, then retained caller entries.

### `__pycache__` directories appear

`source_me.sh` exports `PYTHONDONTWRITEBYTECODE`, but that applies only to
Python processes launched after it is sourced. For a one-off command that must
not write bytecode, also pass `-B`:

```bash
source source_me.sh && python3 -B path/to/script.py
```

An isolated Python process (`-I`) ignores environment variables, including
`PYTHONDONTWRITEBYTECODE`; it must also receive `-B`. Do not use `py_compile` or
`compileall` as a validation shortcut: they explicitly write bytecode. Validate
behavior with pytest or inspect syntax with `ast.parse()` instead.

## Load the local native runtime

### The local `ferrum_chem` runtime fails on macOS

The local runtime is currently verified only for macOS arm64. Rebuild and test
it from the repository root:

```bash
./build.sh
./all_test.sh
```

`build/bin/ferrum-qt` selects the ABI-specific
`build/runtime/python/ferrum_chem<Python ABI suffix>` and its `.dylibs/`
closure. The test entry point confirms that the PyO3 and Qt suites use that
local runtime rather than a globally installed extension. Other platforms are
not yet qualified.

### `all_test.sh` reports a missing, stale, or modified local runtime

`all_test.sh` validates a receipt for the local extension, adapter, and
launchers before it runs the CLI, E2E, PyO3, and Qt suites. Its
`complete local Ferrum runtime is missing` or `local Ferrum runtime is stale or
has been modified` error means that the local build must be replaced as one
unit:

```bash
./build.sh
./all_test.sh
```

Do not copy a dylib, extension, or launcher into `build/runtime` by hand. The
stable `build/bin` and `build/runtime` paths resolve through one atomically
promoted program root.

### `./build.sh` says another build owns the local build

Only one `./build.sh` invocation may promote the shared local runtime. If the
command says `another ./build.sh invocation owns the local build; wait for it
to finish`, wait for that invocation and rerun your command. Do not delete or
edit `build/.build.lock`: the operating-system lock, not lock-file contents,
determines ownership.

### `./build.sh` exceeds the 20 GiB checkout budget

The build removes only its known compiler and staging paths. When it reports
that the checkout exceeds the 20 GiB build budget, inspect the categories it
lists and unrelated large checkout content; then rerun the build. Do not use a
recursive cleanup against the checkout as a build recovery step.

### Rust tooling is unavailable

`./check_rust.sh` checks for Cargo, `rustc`, `rustfmt`, and Clippy before it
uses the workspaces. Install the Rust toolchain required by
[INSTALL.md](INSTALL.md). For an absent formatting or lint component, use the
exact recovery commands reported by the checker:

```bash
rustup component add rustfmt
rustup component add clippy
```

Then rerun `./check_rust.sh`. This Rust gate does not build the local Python
extension or run Qt tests; use `./build.sh` and `./all_test.sh` for that route.

## Run Qt tests without a display

### Qt tests need a headless platform

Ferrum's permanent Qt tests and registered E2Es use Qt's offscreen platform.
For a focused Qt pytest run, use the same environment:

```bash
source source_me.sh
QT_QPA_PLATFORM=offscreen python3 -B -m pytest packages/ferrum-chem-qt.app/tests/ -q -W error
```

Use `QT_QPA_PLATFORM=offscreen` for tests, not as evidence that the desktop
application has been visually accepted. Start `build/bin/ferrum-qt` in a macOS
desktop session for a one-time visual and accessibility review.

## Run the operation protocol

### `protocol run` rejects a request

Use `build/bin/ferrum protocol schema` to obtain the generated request shape,
then submit one UTF-8 JSON request file with
`build/bin/ferrum protocol run request.json`. A decodable request refusal is a
JSON error envelope; malformed JSON, an over-budget request, unreadable input,
or a confirmed output-publication failure has no envelope and exits 1.
`--output -` is a usage error. The protocol intentionally does not accept direct
SMILES, SDF, molblock, InChI, CD-SVG, adapter, or path-bearing operation inputs.

## Start Ferrum

### Open or create a drawing

Launch Ferrum with an uncompressed CDML drawing:

```bash
build/bin/ferrum-qt drawing.cdml
```

`build/bin/ferrum-qt` is the local application launcher. Run it without a path
to start the window, then use File > New or File > Open. Ferrum opens
uncompressed `.cdml` drawings and decoded `.svg` files that contain embedded
CDML through its Rust-native document flow.

### A chosen drawing is unsupported

Ferrum's File > Open accepts uncompressed `.cdml` drawings, decoded `.svg` files
that contain embedded CDML, bounded CML/CML2, and bounded input-only ChemDraw XML
(`.cdxml`) simple-molecule input. CML and CDXML open as clean new documents and their
first Save writes CDML. CDX, unsupported CDXML chemistry or presentation, namespaces,
compressed CDML, `.cdsvg`, and CML outside the supported profile refuse without changing
the current document. See [FILE_FORMATS.md](FILE_FORMATS.md) for the exact boundary.

### Make a recovery copy

Use File > Recovery Export CDML... to copy the current document's Rust snapshot
to a new `.cdml` file without changing its saved-file association or unsaved
state. Use Save or Save As when the selected path should become the document's
normal save location.

If Ferrum reports that recovery-copy durability is unconfirmed, inspect the
chosen destination before relying on the copy.
