# Ferrum E2E evidence

This document records Ferrum-owned end-to-end evidence. It complements the
vendored generic [E2E_TESTS.md](E2E_TESTS.md); it does not replace it.

## CDXML simple-molecule import

The permanent CLI gate is `tests/e2e/e2e_cdxml_open_cli.py`. It runs a supplied
local `ferrum` build, uses inline CDXML in a temporary directory, and proves the
public `formats` input-only capability, a successful `open --format cdxml --output`
CDML receipt, and a typed refusal that publishes no output artifact. It has no
network or committed fixture dependency.

The permanent Qt gate is
`packages/ferrum-chem-qt.app/tests/test_ferrum_native_cdml_open.py`. It uses
the visible File/Open path through the native descriptor/worker route and
proves successful CDXML tab installation, typed CDXML provenance, and refusal
nonmutation with descriptor-neutral recovery. Shared converted-source coverage owns
the common CDML Save/Save As/reopen behavior.
Its inputs are inline or `tmp_path` files; it does not assert pixels, delays,
or whole error strings.

The release evidence is separate. Capture a real macOS screenshot of File/Open
and the resulting document in a 16:10 outer application window, including the
ribbon and status bar. Pair it with a keyboard/accessibility walkthrough. This
human evidence demonstrates integrated usability; it is not a permanent
pixel-equivalence or timing test.
