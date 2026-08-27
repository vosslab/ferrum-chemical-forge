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
