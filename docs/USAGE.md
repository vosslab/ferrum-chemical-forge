# Use Ferrum from shell

The `ferrum` executable provides headless access to Ferrum's Rust document backend.
Its current commands inspect CDML through the typed core model and structurally
rewrite CDML without depending on Python or `OTHER_REPOS/`.

## Rust CLI quick start

From the repository root, inspect the checked-in authored corpus document:

```bash
ferrum cdml inspect tests/e2e/corpus/authored_document_forms.cdml
```

The command writes one compact JSON object to standard output. The object uses the
preview `ferrum-cdml-inspection-v1` schema and reports document identity, typed record
counts, diagnostics, and per-molecule atom, non-atom vertex, and bond counts.

## Rust command-line interface

The available command groups and flags are:

- `ferrum cdml inspect INPUT [--format json|text]` requires Ferrum's current core
  projection and summarizes one CDML document.
- `ferrum cdml validate INPUT [--typed] [--format json|text]` validates retained CDML
  structure and identity. `--typed` additionally requires the current core projection.
- `ferrum cdml rewrite INPUT --output OUTPUT` parses and structurally re-emits CDML.
- `ferrum cdml rewrite INPUT --check` serializes, reparses, and verifies the documented
  structural-preservation contract without writing a document.
- `ferrum cdml extract-cdsvg INPUT --output OUTPUT` extracts the single canonical CDML
  payload from decoded CD-SVG, verifies its structural serialization transaction, and publishes it.
- `-` selects standard input where `INPUT` appears.
- `--output -` selects standard output for rewritten or extracted CDML.
- `--help` and `--version` describe the installed executable.

All JSON reports are newline-terminated preview schemas: inspection uses
`ferrum-cdml-inspection-v1`, validation uses `ferrum-cdml-validation-v1`, and rewrite
checking uses `ferrum-cdml-rewrite-check-v1`. JSON is the default; deterministic text
output is for people and is not a parsing contract. Argument errors exit with status 2.
Accepted commands that cannot read, process, validate, or write data exit with status 1
and send the error to standard error. Success exits with status 0 and keeps standard
error empty.

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

Extract a canonical CDML payload from CD-SVG without an intermediate file:

```bash
ferrum cdml extract-cdsvg - --output - < drawing.svg > extracted.cdml
```

Check that a rewrite would retain the documented structural facts, without
creating an output file:

```bash
ferrum cdml rewrite drawing.cdml --check
```

Validate only retained CDML structure, including opaque XML Ferrum can preserve:

```bash
ferrum cdml validate drawing.cdml
```

## Rust inputs and outputs

The CDML commands accept UTF-8 input. `rewrite` preserves parsed XML structure,
including opaque elements, namespaces, comments, processing instructions, and mixed
content. File rewrites serialize and validate before atomically replacing their target
for concurrent readers. They do not make a crash-durability claim for a successful rename.
Tree serialization may normalize lexical details such as prefix choice, attribute order,
CDATA boundaries, entity spelling, or XML declarations. `--check` verifies structure;
it does not promise byte-for-byte or lexical identity.

`extract-cdsvg` accepts decoded UTF-8 XML with an SVG root and exactly one canonical CDML
payload. It writes verified CDML through the same atomic file-output boundary as `rewrite`.
Compressed `.svgz` input is not accepted by this command.

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
  become available; the CLI does not yet provide a Haworth renderer.
- Add packaged Ferrum-Qt workflows after the desktop application uses this backend.
