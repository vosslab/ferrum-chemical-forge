# Frequently asked questions

## What is Ferrum?

Ferrum is a pre-alpha chemical-document application. Rust is the sole owner of
chemistry, document state, history, rendering, admission, file-format
decisions, and typed refusals. PyO3 carries frozen Rust-issued facts across the
native boundary; it is not a second chemistry, document, renderer, or format
implementation. Ferrum Qt is the PySide6 desktop client: it owns interaction,
presentation, and accessibility while sending bounded user intent and never
maintaining a shadow document model. See
[CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md) for the ownership boundary.

## How does Ferrum use historical migration evidence?

Ferrum is being built as the replacement direction, but complete migration
parity is not yet claimed. Historical OASA and BKChem material is read-only
migration evidence: Ferrum reimplements needed behavior as Rust-owned
contracts instead of retaining a Python chemistry or document model. That
material is not a runtime dependency, compatibility target, or fallback. The
remaining scope and acceptance ledger is
[FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

## How do the command line and desktop relate?

`ferrum` is the Rust command-line tool and `ferrum-qt` is the PySide6 desktop
application. Both use the same Rust-owned chemistry and document contracts.
Build the checkout with `./build.sh`, then use the launchers in `build/bin/`.
[USAGE.md](USAGE.md) lists supported commands and desktop workflows.

## Which file formats can I use?

Use `build/bin/ferrum formats` as the current machine-readable and
human-readable capability source. CDML is Ferrum's editable/save format. The
desktop additionally has bounded File/Open profiles for CD-SVG, CML/CML2,
SDF, and input-only CDXML simple molecules. The conversion registry is a
separate, declared capability; Ferrum is not a general chemical file converter.

An accepted interchange import becomes a new clean CDML document only after
the complete Rust render observation has no suppression, plan issues, or member
issues. A refusal therefore publishes neither a new desktop tab nor a CLI
output artifact. CDX, compressed input, arbitrary SVG, and CDXML outside the
declared simple-molecule profile are refused. See
[FILE_FORMATS.md](FILE_FORMATS.md) for exact profiles, limits, losses, and
publication rules.

## Why does the repository include Atkinson Hyperlegible Mono?

Ferrum vendors the official Atkinson Hyperlegible Next and Mono families with
their provenance and license records. The current molecule-label role is not
Mono: Rust selects the vendored proportional Atkinson Hyperlegible Next Regular
resource by exact bytes, and Qt replays that issued choice without system-font
discovery or substitution. The Mono files are vendored assets, not a desktop
preference or an alternative molecule-label default. See
[PROVENANCE.md](PROVENANCE.md) for the selected resource and font catalog.

## How are Wavy, Bold, and Dashed CDXML bonds imported?

The bounded CDXML importer accepts `Display="Wavy"`, `Display="Bold"`, and
`Display="Dash"` only on an ordinary single bond: `Order` must be omitted or
`"1"`, and the bond cannot also carry stereochemical direction. Ferrum stores
the result as its own fixed-single presentation: `s1` (Wavy), `b1` (Bold), or
`d1` (Dashed). It does not retain raw CDXML display data or source layout.

The Rust renderer generates the corresponding geometry for every target,
including SVG, PDF, PNG, and the Qt projection. A non-single or otherwise
unsupported displayed bond receives a typed refusal before publication. The
exact grammar, render geometry, losses, and atomicity rules are in
[the CDXML import decision](active_plans/decisions/m2_cdxml_simple_molecule_import_v1.md).

## What can I edit in Ferrum Qt?

Ferrum Qt currently exposes a growing bounded set of Rust-owned workflows,
including atom and bond authoring, supported bond presentations, selection,
rings, reactions, templates, Undo/Redo, CDML publication, and SVG/PDF/PNG
artifact export. The current visible routes are documented in
[GUI_TOUR.md](GUI_TOUR.md). Unsupported requests receive typed refusals and
recovery guidance; the desktop does not delegate them to legacy code.

For step-by-step workflows, use [USAGE.md](USAGE.md). The parity ledger is the
source of truth for work that remains.

## Is Ferrum ready for production work?

No. Ferrum is a local-checkout, pre-alpha build, not a released desktop
distribution. Local automated Rust, PyO3, Qt, and end-to-end evidence covers
bounded behavior, but full migration parity, human desktop/accessibility
acceptance, remote CI, and release-artifact validation remain open. The
supported local route is macOS arm64 with Rust 1.97.1 or newer and Python 3.12;
[INSTALL.md](INSTALL.md) defines the exact build and runtime environment.
