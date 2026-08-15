# Frequently asked questions

## Is Ferrum a replacement?

Ferrum-Chem is the Rust replacement for the historical OASA backend. The `ferrum`
command-line interface is self-contained: it does not require Python, OASA, or
`OTHER_REPOS/` to run one versioned JSON document operation. See [USAGE.md](USAGE.md)
for its current commands.

`ferrum-qt` starts Ferrum's one PySide6 Rust-native document window; it has no
alternate OASA or legacy editor. Its production dependencies are PySide6, shiboken6,
PyYAML, and `ferrum-chem`; OASA is not a production dependency. Historical OASA
sources and oracle/provenance material remain isolated from the product for reference
and end-to-end comparison only.

## What is the native bounded editor?

`ferrum-qt drawing.cdml` starts the Rust-native document window. It can open, render,
change a selected atom element, add one free-standing atom, undo, redo, save, save as,
reopen, and close `.cdml` documents through the Rust session. It deliberately has no
fallback to another editor.

The native editor is a narrow vertical slice, not the general editor: it opens
uncompressed `.cdml` files and decoded `.svg` files containing one canonical CDML
payload. Its documented bounded editing, chemistry import, geometry, template, and
artifact-export routes remain distinct native workflows rather than general conversion.

## Which formats work now?

The `ferrum` CLI accepts a JSON protocol request containing CDML text; it does not
offer direct file-format subcommands. The native Qt route opens uncompressed `.cdml`
or decoded `.svg` with exactly one canonical embedded CDML payload. It refuses CDXML,
CML, `.cdsvg`, `.svgz`, and compressed input before document mutation.

Ferrum preserves parsed CDML structure rather than promising byte-for-byte output.
For the precise protocol and desktop boundaries, read [USAGE.md](USAGE.md).

## Can I use every chemistry tool?

Not yet. The CLI's four protocol operations are document inspection, validation,
structural rewrite, and artifact rendering. Ferrum-Qt separately supports the bounded
native workflows recorded in the capability matrix; it is not a general conversion
suite. The implementation sequence and open milestones are in
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).

## Is Ferrum ready for production?

No. Ferrum is pre-alpha. The native-wheel proof is currently bounded to macOS arm64,
and the desktop cutover is incomplete. [INSTALL.md](INSTALL.md) identifies the current
source-install requirements and limits.
