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

## Run the operation protocol

### `protocol run` rejects a request

Use `ferrum protocol schema` to obtain the generated request shape, then submit one UTF-8
JSON request file with `ferrum protocol run request.json`. A decodable request refusal is a
JSON error envelope; malformed JSON, an over-budget request, unreadable input, or a confirmed
output-publication failure has no envelope and exits 1. `--output -` is a usage error. The
protocol intentionally does not accept direct SMILES, SDF, molblock, InChI, CD-SVG, adapter, or
path-bearing operation inputs.

## Start Ferrum-Qt

### Open or create a drawing

Launch Ferrum-Qt with an uncompressed CDML drawing:

```bash
ferrum-qt drawing.cdml
```

`ferrum-qt` is the sole application command. Run it without a path to start the
window, then use File > New or File > Open. Ferrum opens uncompressed `.cdml`
drawings and decoded `.svg` files that contain embedded CDML through its Rust-native
document flow.

### A chosen drawing is unsupported

Ferrum does not convert formats while opening them. For ChemDraw XML (`.cdxml`) or
Chemical Markup Language (`.cml`), use the source application or a converter to make
an uncompressed `.cdml` drawing, then open that result. For compressed CDML, SVG, or
`.cdsvg` files, choose the uncompressed source: a `.cdml` drawing or decoded `.svg`
file containing embedded CDML. The rejected file and the current document remain
unchanged.

### Make a recovery copy

Use File > Recovery Export CDML... to copy the current document's Rust snapshot to a
new `.cdml` file without changing its saved-file association or unsaved state. Use
Save or Save As when the selected path should become the document's normal save
location.

If Ferrum reports that recovery-copy durability is unconfirmed, inspect the chosen
destination before relying on the copy.
