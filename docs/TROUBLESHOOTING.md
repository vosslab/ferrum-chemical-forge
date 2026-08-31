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

The development shell requires the extension staged in this checkout and
refuses a globally installed `ferrum_chem` extension. If it reports either
`Ferrum local runtime is unavailable` message, rebuild the sealed local
program, then source the shell setup again:

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

### Atkinson molecule-label font verification fails

Messages containing `verified Atkinson Hyperlegible Next asset` or `Atkinson
Hyperlegible Next admission contract mismatch` come from the Rust-owned bundled
molecule-label resource. They do not mean that macOS is missing a system font:
the selected face is compiled into the local `ferrum_chem` extension and verified
before rendering uses it.

Run the Rust gate, then rebuild and test the sealed local program:

```bash
./check_rust.sh
./build.sh
./all_test.sh
```

Do not substitute a system-installed font, add a font directory to a launch
environment, or copy a font into `build/runtime`. If the Rust gate reports the
verification failure, retain its exact output and repair the repository-owned
font resource or its admission contract before attempting another GUI capture.

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

## Capture documentation screenshots

### Screenshot capture cannot start or cannot find the staged extension

The documentation tour captures the locally built application; it is not part
of `all_test.sh` and it needs an active macOS desktop session. Rebuild the
sealed local program, confirm the available scene names, then capture the full
tour:

```bash
./build.sh
./capture_gui_screenshots.sh --list
./capture_gui_screenshots.sh
```

The capture command uses the same staged runtime that `source_me.sh` validates.
Use that repository-owned runtime and an on-screen Qt platform so the tour
documents real visible windows. A complete run stages every
scene before it replaces the stable PNG files in `docs/screenshots/`.

### The screen-capture backend fails or crops Ferrum

The default backend first tries the optional macOS `easy-screenshot` command,
then falls back to a Qt capture of the visible Ferrum window. If the explicit
`--backend easy-screenshot` route fails, grant the required macOS screen-capture
permission or use the deterministic Qt route:

```bash
./capture_gui_screenshots.sh --backend qt
```

The capture harness requires a 1440 by 900 logical Ferrum window, confirms that
the named authoring ribbon and status bar are visible widgets, and rejects a PNG
whose dimensions are not a 16:10 full-window surface. It verifies those window
and widget conditions, not every rendered pixel. For a backend geometry refusal,
use `--backend qt` or configure the window-capture tool to capture only Ferrum's
application surface, then rerun the full tour. Review the regenerated images
manually; screenshot capture is documentation evidence, not a permanent
pixel-equivalence test.

### A screenshot scene or surface check is refused

The capture tour verifies more than PNG creation. A refusal such as `Ferrum
capture requires the visible authoring ribbon`, `Ferrum capture requires the
visible status bar`, or `capture output is not a usable window PNG` means that
the requested scene is not documentation-ready. A full run stages all scenes
before publishing, so a failed full tour leaves the existing published tour in
place.

Use the named-scene route to isolate the reported workflow without replacing the
other images:

```bash
./capture_gui_screenshots.sh --scene template_catalog --backend qt
```

If the focused capture succeeds after the underlying application repair, run
`./capture_gui_screenshots.sh --backend qt` to refresh the complete tour from
one local build. If the focused scene still reports a missing command, control,
or completed document state, preserve the last complete published tour and
record the scene as a failed GUI capability.

### Refresh one screenshot while diagnosing a scene

Use a named scene only for a focused diagnostic update:

```bash
./capture_gui_screenshots.sh --scene template_catalog
```

It publishes that one verified PNG and leaves the other tour images intact.
After a documentation refresh, run the complete tour so the full set represents
one current build and review the resulting images as described in
[GUI_TOUR.md](GUI_TOUR.md).

## Run the operation protocol

### `protocol run` rejects a request

Use `build/bin/ferrum protocol schema` to obtain the generated request shape,
then submit one UTF-8 JSON request from a named file or standard input with
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
that contain embedded CDML, bounded CML/CML2, bounded input-only ChemDraw XML
(`.cdxml`) simple-molecule input, and runtime-backed SDF inputs (`.sdf` and `.sd`).
CML, CDXML, and SDF each open as a clean new CDML document in a new tab; their
first Save writes CDML and does not retain the interchange source as the save
destination. SDF requires the installed trusted chemistry runtime. CDX, unsupported
CDXML chemistry or presentation, namespaces, compressed CDML, `.cdsvg`, and CML
outside the supported profile refuse without changing the current document. The
current CDXML profile accepts fixed-single Wavy, Bold, and Dash depictions; consult
[FILE_FORMATS.md](FILE_FORMATS.md) for the exact format boundary.

Interchange import publishes a new tab only after Rust produces a complete,
issue-free render for the committed candidate. A refusal at that final admission
step leaves no partial document or replacement tab to recover; retain the source
and investigate it through the typed File/Open or CLI refusal instead.

### Make a recovery copy

Use File > Recovery Export CDML... to copy the current document's Rust snapshot
to a new `.cdml` file without changing its saved-file association or unsaved
state. Use Save or Save As when the selected path should become the document's
normal save location.

If Ferrum reports that recovery-copy durability is unconfirmed, inspect the
chosen destination before relying on the copy.
