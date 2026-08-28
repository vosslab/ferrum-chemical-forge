# Ferrum E2E evidence

This document records Ferrum-owned end-to-end evidence. It complements the
vendored generic [E2E_TESTS.md](E2E_TESTS.md); it does not replace it.

## CDXML simple-molecule import

The permanent CLI gate is `tests/e2e/e2e_cdxml_open_cli.py`. It runs a supplied
local `ferrum` build, uses inline CDXML in a temporary directory, and proves the
public `formats` input-only capability and a successful
`open --format cdxml --output` CDML receipt. It also proves that CDXML
`Display="Wavy"`, `Display="Bold"`, and `Display="Dash"` become the durable
`s1`, `b1`, and `d1` tokens. A presentation on a non-single source bond, and a
valid first fragment followed by an invalid later fragment, each produce one
typed refusal with no output artifact or source-detail leak. It has no network
or committed fixture dependency.

The permanent Qt gate is `tests/e2e/e2e_cdxml_open_qt.py`. It starts the real
PySide6 File/Open action through the native descriptor and worker route, waits
for its queued public completion, and verifies one issue-free rendered molecule
whose Rust-issued bond presentations are, in order, Wavy, Bold, and Dashed. It
also verifies the retained `s1`, `b1`, and `d1` CDML tokens. The current-tab
route refuses CDXML without replacing or mutating the active tab; new-document
File/Open owns CDXML publication. Its input is an inline temporary file; it
does not assert pixels, delays, or whole error strings.

`tests/e2e/run_all.sh` registers both gates after validating the staged local
runtime. `./all_test.sh` owns that registration lane after `./build.sh` has
sealed the local CLI and Python runtime. The focused Rust import tests also
prove the shared exact-revision clean-render publication gate for CML and SDF,
so clean renderer admission is a generic interchange contract rather than a
CDXML-only check.

The release evidence is separate. Capture a real macOS screenshot of File/Open
and the resulting document in a 16:10 outer application window, including the
ribbon and status bar. Pair it with a keyboard/accessibility walkthrough. This
human evidence demonstrates integrated usability; it is not a permanent
pixel-equivalence or timing test. Remote CI, release approval, and human visual
and accessibility acceptance remain unclaimed by these automated gates.
