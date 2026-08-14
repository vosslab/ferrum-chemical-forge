# Troubleshooting

## Use the supported Python

### `python3` uses the wrong environment

Run Python commands from the repository root after loading the repository shell
environment:

```bash
source source_me.sh && python3 -B -m pytest
```

`source_me.sh` requires Bash, sources the local shell setup first, and then sets the
repository's unbuffered-output and no-bytecode defaults. Use the repository's Python
3.12 environment. Do not run the script as an executable: its environment changes
would remain in the child process.

### `__pycache__` directories appear

`source_me.sh` exports `PYTHONDONTWRITEBYTECODE=1`, but that applies only to Python
processes launched after it is sourced. For a one-off command that must not write
bytecode, also pass `-B`:

```bash
source source_me.sh && python3 -B path/to/script.py
```

An isolated Python process (`-I`) ignores environment variables, including
`PYTHONDONTWRITEBYTECODE`; it must also receive `-B`. The native-wheel proof follows
this rule for its isolated probes.

## Load the native wheel

### `import ferrum_chem` fails on macOS

The native-wheel proof is currently verified only for macOS arm64. Build and verify it
from the repository root so the test can inspect the installed extension and its
bundled `.dylibs` closure:

```bash
source source_me.sh && python3 -B tests/e2e/e2e_native_wheel.py
```

Do not treat a successful source-tree import as wheel evidence. The proof uses a fresh
environment with ambient dynamic-library variables scrubbed and writes a receipt to
`output_native_wheel/evidence/native-wheel-e2e-receipt.json` on success. Other
platforms are not yet qualified.

## Inspect SMILES

### `smiles inspect` rejects the adapter

`ferrum smiles inspect` deliberately does not discover a native library. Supply an
absolute path to the verified ABI-4 adapter library; the path must name a regular file
and cannot be a symbolic link:

```bash
ferrum smiles inspect --adapter /absolute/path/libferrum_chem.dylib CCO
```

Relative paths fail with `adapter path must be absolute`. A symlink, directory, or
other non-regular path fails before the library is loaded. This prevents an accidental
or ambient loader choice from changing the chemistry backend.

## Start Ferrum-Qt

### Native bounded editor behaves differently

Use the bounded Rust-native CDML editor explicitly:

```bash
ferrum-qt --native drawing.cdml
```

This public route loads the Rust wheel and does not import OASA. It currently covers
native CDML open, render, selected-atom element changes, one free-standing atom
insertion, Rust undo/redo, save, reopen, and close. Do not expect bond drawing or the
other retained legacy editing features in this route.

The ordinary command remains the retained PySide6 application during migration:

```bash
ferrum-qt drawing.cdml
```

That legacy route still has migration-only OASA dependencies. Use one route or the
other for a run; `--native` is the required choice when validating the OASA-free CDML
path.

### Native smoke receipt is rejected

`--smoke-receipt` belongs only to the retained legacy startup path. It is intentionally
unavailable with `--native`. Use a positive `--smoke-exit` value for a controlled native
startup check instead:

```bash
ferrum-qt --native --smoke-exit 0.05 drawing.cdml
```
