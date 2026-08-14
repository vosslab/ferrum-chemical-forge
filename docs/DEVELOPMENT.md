# Develop Ferrum

Use this guide for repository changes. [INSTALL.md](INSTALL.md) describes
contributor setup; [USAGE.md](USAGE.md) describes the public commands.

## Preserve shared work

This migration is developed in a shared, often dirty worktree. Start by checking
the files you own, and preserve unrelated changes:

```bash
git status --short
git diff -- path/to/file
```

Do not stage, commit, reset, restore, clean, or reformat unrelated files. Keep an
implementation slice narrow, and record verified user-visible behavior before
expanding it. The active migration contract is
[active_plans/ferrum-plan-v3.md](active_plans/ferrum-plan-v3.md).

## Run Rust gates

Run the repository-owned front door after a Rust change:

```bash
./check_rust.sh
```

The script checks both Rust workspaces with the repository lockfiles, strict
all-target Clippy, unit/integration/doc tests, and API docs while reusing the
bounded Cargo cache. Use focused package tests while iterating. Run a separate
`--target` check only when qualifying a platform or changing target-sensitive
native code; it is not a routine edit flag. Rust conventions and ownership rules
are in [RUST_STYLE.md](RUST_STYLE.md); PyO3 packaging rules are in
[RUST_PYO3_STYLE.md](RUST_PYO3_STYLE.md).

## Run Python and Qt tests

Use the repository bootstrap for every Python command. It selects Python 3.12 and
exports `PYTHONDONTWRITEBYTECODE=1`, so source checks do not create `__pycache__`
directories.

```bash
source source_me.sh
python3 -B -m pytest tests/ packages/ferrum-chem-qt.app/tests/ -q -W error
```

For headless Qt validation, set the platform explicitly:

```bash
source source_me.sh
QT_QPA_PLATFORM=offscreen python3 -B -m pytest packages/ferrum-chem-qt.app/tests/ -q -W error
```

The native CDML route is a separate installed-wheel proof. Supply the exact wheel
under review rather than relying on an ambient extension:

```bash
source source_me.sh
python3 -B packages/ferrum-chem-qt.app/tests/e2e_native_cdml_file_route.py \
  --wheel /absolute/path/ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl
```

It opens, renders, saves, reopens, and closes a CDML document through
`ferrum-qt --native`'s OASA-free host. The wider native-wheel closure and relinking
proof is documented in [USAGE.md](USAGE.md#native-wheel-packaging-evidence).

## Check documentation

Keep Markdown ASCII-only and use repository-relative links. Before handing off a
documentation change, run the focused checks from the repository root:

```bash
source source_me.sh
python3 -B -m pytest tests/test_ascii_compliance.py tests/test_markdown_links.py -q -W error
git diff --check
```

See [MARKDOWN_STYLE.md](MARKDOWN_STYLE.md) for the complete documentation rules.
