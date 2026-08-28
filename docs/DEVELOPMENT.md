# Develop Ferrum

Use this guide to change and verify the repository. [INSTALL.md](INSTALL.md)
sets up a checkout; [USAGE.md](USAGE.md) explains the supported CLI and desktop
workflows. The active migration scope and accepted evidence live in
[active_plans/active/FULL_PARITY_RUST_FIRST.md](active_plans/active/FULL_PARITY_RUST_FIRST.md).

## Preserve shared work

Ferrum is developed in a shared, frequently dirty worktree. Establish the scope
you own before editing and preserve every unrelated change:

```bash
git status --short
git diff -- path/to/file
```

Do not stage, commit, reset, restore, clean, or reformat unrelated work. Agents
do not change the Git index; a human owns staging and commits. Keep each change
small, update [CHANGELOG.md](CHANGELOG.md) for human review, and record evidence
at the same scope as the behavior changed.

## Navigate the codebase

Use the current Graphify map to identify a narrow starting point before broad
repository exploration. Its graph is navigation evidence, not architectural
proof; confirm every conclusion in the current source and tests.

```bash
graphify query "How does the requested behavior cross Rust, PyO3, and Qt?" --budget 1500
graphify explain path/or/symbol
graphify affected path/or/symbol --depth 2
```

Use [CODE_ARCHITECTURE.md](CODE_ARCHITECTURE.md) and
[FILE_STRUCTURE.md](FILE_STRUCTURE.md) to choose the owning layer. Read the
nearest contract and its focused tests before moving a boundary. `OTHER_REPOS/`
is ignored, read-only reference material, never a production dependency.

## Build the local program

`build.sh` is the supported local product builder. It creates one sealed
program below `build/programs/`, then atomically exposes its CLI, Qt launcher,
and native runtime through `build/current`, `build/bin/`, and `build/runtime/`.
It neither publishes a wheel nor installs Ferrum globally.

```bash
./build.sh
build/bin/ferrum --version
build/bin/ferrum-qt
```

The local Python bootstrap requires that staged extension. Run Python only after
a successful build, always through `source_me.sh`:

```bash
source source_me.sh && python3 -B -m pytest tests/ -q -W error
```

The bootstrap selects Python 3.12, keeps repository Qt source ahead of the
sealed extension runtime, validates that exact extension import, and prevents
bytecode-cache writes. Do not substitute a globally installed Ferrum package.
See [LOCAL_BUILD.md](LOCAL_BUILD.md) for build ownership, leases, and cleanup.

## Choose the validation lane

Run the narrowest lane that proves the edited contract while iterating, then run
the applicable front door before handoff.

| Change or question | Command | What it proves |
| --- | --- | --- |
| Rust package behavior | `cd packages/ferrum-rust && cargo test -p PACKAGE --locked` | Affected native contract |
| Rust workspace boundary | `./check_rust.sh` | Formatting, checks, strict Clippy, tests, and docs for both Rust workspaces |
| Repository policy or Python behavior | `source source_me.sh && python3 -B -m pytest tests/ -q -W error` | Fast repository checks; E2Es stay excluded |
| Qt presentation behavior | `source source_me.sh && QT_QPA_PLATFORM=offscreen python3 -B -m pytest packages/ferrum-chem-qt.app/tests/ -q -W error` | Deterministic headless Qt behavior through the staged extension |
| Product workflow | `./build.sh && bash tests/e2e/run_all.sh` | Registered CLI and Qt E2Es against the staged local program |
| Local acceptance | `./build.sh && ./all_test.sh` | Hygiene, staged-runtime receipt, launcher smoke, registered E2Es, PyO3, and Qt suites |

Run `./check_rust.sh` after Rust workspace changes. It uses lockfiles and a
disposable `build/.cargo-check-target/` directory; it removes that work area
when complete. Use an explicit Cargo target only when qualifying a platform or
changing target-sensitive native code. Rust and boundary rules are in
[RUST_STYLE.md](RUST_STYLE.md) and [RUST_PYO3_STYLE.md](RUST_PYO3_STYLE.md).

Keep permanent pytest deterministic, offline, fast, and behavioral. A workflow
with real subprocesses, multiple boundaries, visual review, or timing belongs
in the explicit E2E or one-time evidence lane described by
[PYTEST_STYLE.md](PYTEST_STYLE.md) and [FERRUM_E2E_TESTS.md](FERRUM_E2E_TESTS.md).

## Verify packaged boundaries

Run the isolated wheel gate after changing PyO3, Cargo extension ownership, or
Maturin configuration:

```bash
bash devel/verify_python_wheel.sh
```

The gate builds the checked-in Maturin project with its lockfile, installs the
wheel in a disposable CPython 3.12 environment, clears the developer
`PYTHONPATH`, imports the installed native module, and creates a Rust-owned
empty document. It proves packaging independently of the staged local runtime.

The optional `tests/e2e/reference/` environment contains Python RDKit only for
one-time maintainer measurements. Keep it out of product and routine developer
dependencies.

## Review documentation

Keep durable Markdown ASCII-only, concise, and linked with paths relative to
the document that contains them. Check documentation from the repository root:

```bash
source source_me.sh && python3 -B -m pytest tests/test_ascii_compliance.py tests/test_markdown_links.py -q -W error
git diff --check
```

Run [MARKDOWN_STYLE.md](MARKDOWN_STYLE.md)'s link and encoding rules before
handoff. Screenshots are one-time documentation evidence, not permanent visual
regressions; capture and review them using [GUI_TOUR.md](GUI_TOUR.md).
