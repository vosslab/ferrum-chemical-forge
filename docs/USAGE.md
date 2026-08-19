# Use Ferrum

Ferrum provides a Rust `ferrum` command-line tool and a bounded `ferrum-qt` drawing application.
The CLI runs without Python; `convert` and `coords` additionally need a trusted engine bundle.
Install the CLI first as described in [INSTALL.md](INSTALL.md).

## Convert a molecule

Install a compatible, explicitly provisioned engine bundle before calling `convert` or `coords`:

```bash
ferrum engine install /path/to/ferrum-engine-bundle
ferrum engine status
ferrum convert aspirin.smi --to sdf_v2000 --output aspirin.sdf
```

`convert` accepts one source file or `-` for standard input. Its exact syntax names are `smiles`,
`inchi_standard`, `inchi_fixed_h`, `molblock_v2000`, `molblock_v3000`, `sdf_v2000`, `sdf_v3000`,
and `cdml`. Ferrum infers only `.smi`/`.smiles`, `.inchi`, `.mol`/`.molblock`, `.sdf`, and `.cdml`;
use `--from` for standard input or another suffix.

```bash
printf 'CCO\n' | ferrum convert - --from smiles --to molblock_v2000 > ethanol.mol
ferrum convert input.mol --from molblock_v2000 --to smiles --output output.smi
```

If `ferrum engine status` is not `ready`, engine verbs finish with the typed
`chemistry_unavailable` refusal. They do not discover an adapter from the current directory,
`PATH`, Python installations, or environment variables. An engine bundle is installed from one
explicit directory, validated for the current host and ABI, copied into Ferrum's application-data
root, and made active through an atomic record update. Reinstalling a valid bundle replaces the
active record; `status` reports `not-installed`, `ready`, or `invalid`.

## Render a drawing

Render one supported complete CDML document as SVG, PDF, or transparent PNG:

```bash
ferrum render drawing.cdml --output drawing.svg
ferrum render drawing.cdml --to pdf --output drawing.pdf
ferrum render drawing.cdml --to png --output drawing.png
```

Ferrum infers a named artifact format from `.svg`, `.pdf`, or `.png`; use `--to` for standard
output or an unfamiliar suffix. SVG and PDF are vector artifacts. PNG uses one output pixel per
Rust page point with transparency; that is a page-geometry rule, not a print-DPI promise.

## Draw with the keyboard

Start a new window or open an uncompressed CDML drawing:

```bash
ferrum-qt
ferrum-qt drawing.cdml
```

For a keyboard-only small drawing task, activate File > Open with the platform Open shortcut,
choose a CDML document, then use these commands while canvas focus is active:

1. Press `Ctrl+8` for Add Atom. Arrow keys move the crosshair by one grid step; `Shift+Arrow`
   makes a fine move. Press `Enter` to place an atom.
2. Press `Ctrl+2` for Draw Bond. Press `Enter` on the first atom, move to the second atom, and
   press `Enter` again to commit a bond. Press `Escape` to cancel without changing the document.
3. Use the platform Undo shortcut to reverse the last change, then use the platform Save shortcut
   to save or Save As to choose a new CDML path.

Pointer editing remains available. The bounded desktop route supports Rust-owned atom and normal
bond edits, selected molecule work, supported insertions, coordinate work, Undo/Redo, CDML save,
and SVG/PDF/PNG export. Unsupported document features or formats are refused with next-step
guidance and do not alter the active document. See [FILE_FORMATS.md](FILE_FORMATS.md) for admitted
files and publication rules.

## Six command verbs

All human verbs create one V1 operation request and use the same Rust executor:

- `ferrum inspect INPUT [--json]` prints a semantic CDML inspection report.
- `ferrum validate INPUT [--level structural|typed] [--json]` prints validation facts.
- `ferrum rewrite INPUT [-o OUTPUT] [--json]` writes structurally preserved CDML.
- `ferrum render INPUT [-o OUTPUT] [--to svg|pdf|png] [--json]` writes one complete artifact.
- `ferrum convert INPUT [--from FORMAT] --to FORMAT [-o OUTPUT] [--json]` converts one bounded
  molecular-interchange source through the installed engine.
- `ferrum coords DOCUMENT [-o OUTPUT] [--json]` regenerates all direct molecule coordinates through
  the installed engine.

`inspect` and `validate` report to standard output. `rewrite`, `render`, `convert`, and `coords`
write raw completed results to standard output when `--output` is omitted or `-`. `--json` instead
writes the complete operation envelope and cannot be combined with a named output destination.

Named outputs use safe publication. Ferrum refuses to replace its retained input source or an
observed hard-link alias. A successful rewrite may normalize serialization details: it preserves
structure, not bytes. A render result is complete for its requested profile; it does not claim pixel
equivalence to another renderer.

## Results and failures

Human diagnostics go to standard error. Exit statuses are:

- `0`: a completed success or typed protocol refusal.
- `1`: input, processing, or confirmed publication failure.
- `2`: command-line usage error.
- `3`: a named output may have been published but Ferrum cannot confirm it.

Use `--json` when another program needs a stable discriminator. Test `schema`, operation `kind`,
and error `category`, not diagnostic text. The complete request and response contract is in
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

## Machine protocol

The lower-level protocol command accepts one UTF-8 JSON request and emits one JSON success or typed
error envelope:

```bash
ferrum protocol schema
ferrum protocol run request.json
ferrum protocol run request.json --output response.json
```

Protocol payloads contain document or interchange text, never paths. It has no batch, network,
session, Qt, or adapter-discovery capability. The generated schema and precise operation envelopes
are specified in [FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md).

## Current boundaries

Ferrum is pre-production. The verified desktop route is a bounded macOS arm64 CPython 3.12 route;
it is not a cross-platform consumer release. The Rust CLI is a separate Cargo-installed command.
See [INSTALL.md](INSTALL.md) for release evidence and [PROVENANCE.md](PROVENANCE.md) for concise
lineage and licensing information.
