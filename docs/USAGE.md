# Use Ferrum from shell

The `ferrum` executable provides headless access to Ferrum's Rust document backend.
Its current commands inspect CDML through the typed core model and structurally
rewrite CDML without depending on Python or `OTHER_REPOS/`.

## Rust CLI quick start

Inspect the checked-in authored corpus document:

```bash
ferrum cdml inspect tests/e2e/corpus/authored_document_forms.cdml
```

The command writes one compact JSON object to standard output. The object uses the
`ferrum-cdml-inspection-v1` schema and reports document identity, typed record counts,
diagnostics, and per-molecule atom, non-atom vertex, and bond counts.

## Rust command-line interface

The available command groups and flags are:

- `ferrum cdml inspect INPUT` validates and summarizes one CDML document.
- `ferrum cdml rewrite INPUT --output OUTPUT` parses and structurally re-emits CDML.
- `-` selects standard input where `INPUT` appears.
- `--output -` selects standard output for rewritten CDML.
- `--help` and `--version` describe the installed executable.

Argument errors exit with status 2. Accepted commands that cannot read, process, or
write data exit with status 1 and send the error to standard error. Successful data
stays on standard output.

## Rust CLI examples

Inspect a pipeline without a temporary input file:

```bash
ferrum cdml inspect - < drawing.cdml
```

Rewrite a document to another file under the structural-preservation contract:

```bash
ferrum cdml rewrite drawing.cdml --output rewritten.cdml
```

Rewrite through a pipeline:

```bash
ferrum cdml rewrite - --output - < drawing.cdml > rewritten.cdml
```

## Rust inputs and outputs

`inspect` and `rewrite` currently accept UTF-8 CDML. `rewrite` preserves parsed XML
structure, including opaque elements, namespaces, comments, processing instructions,
and mixed content. Tree serialization may normalize lexical details such as prefix
choice, attribute order, CDATA boundaries, entity spelling, or XML declarations.

## Launch the Qt preview

After the contributor-preview source install described in
[INSTALL.md](INSTALL.md), start the desktop application with its installed command:

```bash
ferrum-qt
```

Ferrum-Qt remains under backend migration. The command starts the retained PySide6
application; it does not imply that the self-contained Rust cutover is complete.

## Native-wheel packaging evidence

On macOS arm64, run the native-wheel packaging and relinking proof from the repository
root:

```bash
source source_me.sh && PYTHONDONTWRITEBYTECODE=1 python3 tests/e2e/e2e_native_wheel.py --target aarch64-apple-darwin
```

The test writes its result JSON to standard output and, on success, retains only the
ignored local evidence record at
`output_native_wheel/evidence/native-wheel-e2e-receipt.json`. The record captures
the hash-verified source inputs, actual toolchain versions, wheel digest, native
closure, and the before/after relinking probes. It is development evidence, not a
wheel or desktop release artifact; the temporary wheel and native libraries are
removed after the test.

## Current usage gaps

- Add chemistry conversion and rendering commands only as their Rust implementations
  become available; the CLI does not claim OASA's current Haworth renderer.
- Add packaged Ferrum-Qt workflows after the desktop application uses this backend.
