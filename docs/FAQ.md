# Frequently asked questions

## Is Ferrum a replacement?

Ferrum-Chem is the Rust replacement in progress for the historical OASA backend. The
`ferrum` command-line interface is self-contained: it does not require Python, OASA,
or `OTHER_REPOS/` to inspect, validate, rewrite, or observe a CDML document. See
[USAGE.md](USAGE.md) for its current commands.

The ordinary `ferrum-qt` application remains a migration preview and still requires
OASA for its retained legacy workflow. This does not mean that the Rust CLI depends
on OASA.

## What is the native bounded editor?

`ferrum-qt --native drawing.cdml` starts the separate OASA-free Rust-native CDML
route. It can open, render, change a selected atom element, add one free-standing
atom, undo, redo, save, save as, reopen, and close `.cdml` documents through the
Rust session. It deliberately has no fallback to the legacy window.

The native editor is a narrow vertical slice, not the general editor: it currently
opens only `.cdml` files and has no bond drawing, atom movement/deletion, coordinate
generation, or retained import codecs. Use ordinary `ferrum-qt` when that legacy
contributor-preview behavior is required.

## Which formats work now?

The Rust CLI accepts UTF-8 CDML for its document commands. It can also extract one
canonical CDML payload from decoded UTF-8 CD-SVG XML with `ferrum cdml extract-cdsvg`.
Compressed `.svgz` input is not accepted. The native Qt route opens only `.cdml`.

Ferrum preserves parsed CDML structure rather than promising byte-for-byte output.
For the precise preservation boundary and command examples, read [USAGE.md](USAGE.md).

## Can I use every chemistry tool?

Not yet. The current Rust CLI supports CDML inspection, validation, structural rewrite,
CD-SVG extraction, render observation, explicit-adapter SMILES inspection, and bounded
molblock/SDF/SMARTS codec slices. It does not yet provide a general conversion suite or
a Haworth renderer. The implementation sequence and open milestones are in
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).

## Is Ferrum ready for production?

No. Ferrum is pre-alpha. The native-wheel proof is currently bounded to macOS arm64,
and the desktop cutover is incomplete. [INSTALL.md](INSTALL.md) identifies the current
source-install requirements and limits.
