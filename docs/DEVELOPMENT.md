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
all-target Clippy, unit/integration/doc tests, and API docs. Its disposable
Cargo work area is `build/.cargo-check-target/`, which it removes when the gate
finishes; it does not create package-local target directories. Use focused
package tests while iterating. Run a separate `--target` check only when
qualifying a platform or changing target-sensitive native code; it is not a
routine edit flag. Rust conventions and ownership rules are in
[RUST_STYLE.md](RUST_STYLE.md); PyO3 packaging rules are in
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

Ferrum has one ordinary local application entry point. After completing the
contributor setup in [INSTALL.md](INSTALL.md), build and run it directly:

```bash
./build.sh
build/bin/ferrum-qt
```

The local build and Qt suites are the permanent developer validation boundary.

## Verify the Python wheel

Run the isolated packaging gate after changing PyO3, Cargo extension ownership,
or Maturin configuration:

```bash
bash devel/verify_python_wheel.sh
```

The gate builds the checked-in Maturin project with its lockfile and a disposable
Cargo target, installs the wheel into a disposable CPython 3.12 environment,
clears the local developer `PYTHONPATH`, and imports the installed native module.
It also creates an empty Rust-owned document so a package wrapper alone cannot
satisfy the check.

Ferrum's Rust engine is the local runtime chemistry backend. Accepted migration
evidence remains in `docs/active_plans/reports/`. The optional
`tests/e2e/reference/` environment contains only Python RDKit for one-time
maintainer measurements. Keep that dependency outside the product and normal
developer environment.

## Check documentation

Keep Markdown ASCII-only and use repository-relative links. Before handing off a
documentation change, run the focused checks from the repository root:

```bash
source source_me.sh
python3 -B -m pytest tests/test_ascii_compliance.py tests/test_markdown_links.py -q -W error
git diff --check
```

See [MARKDOWN_STYLE.md](MARKDOWN_STYLE.md) for the complete documentation rules.
