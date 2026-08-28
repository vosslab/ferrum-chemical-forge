# Frequently asked questions

## What is Ferrum?

Ferrum is a pre-production chemical-document application. Ferrum-Chem is the
Rust owner of chemistry, document state, history, rendering, admission, and
typed refusals. Ferrum Qt is the PySide6 interaction client. See
[CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md) for the ownership boundary.

## Does Ferrum replace OASA and BKChem?

That is the intended direction, not a completion claim. Ferrum replaces product
OASA behavior with Rust contracts and a bounded native chemistry adapter; it does
not port the historical Python backend or restore a Python document model.
Historical OASA and BKChem material is read-only provenance and migration-oracle
evidence, never a runtime dependency or compatibility authority. The remaining
scope and acceptance evidence are in
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

## How do the CLI and desktop relate?

`ferrum` is the Rust command-line tool. `ferrum-qt` is the bounded PySide6
drawing application. Both use the same Rust-owned document and chemistry
contracts; the desktop client does not fall back to OASA or another editor.
Build the checkout with `./build.sh`, then run the two local launchers from
`build/bin/`. [USAGE.md](USAGE.md) gives the supported commands and keyboard
workflow.

## Which formats can I use?

Use `build/bin/ferrum formats` to inspect the current descriptor-declared
formats and operation eligibility. CDML is Ferrum's editable and save format.
The desktop can also admit bounded CD-SVG, CML/CML2 simple-molecule input, and
the input-only CDXML simple-molecule profile. Chemistry conversion accepts only
the declared conversion profiles; it is not a general file converter.

CDX, compressed input, `.cdsvg`, `.svgz`, arbitrary SVG, and CDXML outside the
bounded profile are refused without changing the current document. See
[FILE_FORMATS.md](FILE_FORMATS.md) for the exact input, output, loss, and
publication rules.

## What can I edit in Ferrum Qt?

The native desktop route supports a growing, bounded set of Rust-owned document
workflows, including atom and normal-bond edits, selected-root work, regular
rings, reactions, compact groups, coordinate work, Undo/Redo, CDML publication,
and SVG/PDF/PNG artifact export. Unsupported features produce typed refusals
with recovery guidance rather than silently switching to a legacy route.

For step-by-step desktop workflows, use [USAGE.md](USAGE.md). The feature work
that remains before full parity is tracked in
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

## Is Ferrum ready for a production workflow?

No. Ferrum is currently a local-checkout, pre-production build. The supported
native route is macOS arm64 with Rust 1.97.1 or newer and Python 3.12. Build,
runtime, and verification requirements are in [INSTALL.md](INSTALL.md).
