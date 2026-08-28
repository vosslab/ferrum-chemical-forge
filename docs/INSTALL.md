# Install

Ferrum is a pre-production local-build project. A successful build creates the
Rust CLI, PySide6 desktop launcher, and private native runtime in one immutable
program root below `build/programs/`, then publishes it through `build/current`.
It does not publish or globally install a Ferrum package.

## Requirements

- A checkout of this repository on the current macOS arm64 development route.
- Rust 1.97.1 or newer with Cargo, as declared in
  [`packages/ferrum-rust/Cargo.toml`](../packages/ferrum-rust/Cargo.toml).
- Homebrew's Python 3.12 and the product and developer dependencies in
  [`pip_requirements.txt`](../pip_requirements.txt) and
  [`pip_requirements-dev.txt`](../pip_requirements-dev.txt).
- The Homebrew tools declared in [`Brewfile`](../Brewfile), including CMake,
  LLVM, and Rustup.

## Install dependencies

From the repository root, install the declared macOS tools and Python
dependencies:

```bash
brew bundle
python3 -m pip install -r pip_requirements.txt -r pip_requirements-dev.txt
```

Do not source `source_me.sh` before the first build: it deliberately refuses
to load until the checkout has a matching staged native extension. After a
successful build, use `source source_me.sh && python3` for every repository
Python command. The script puts this checkout's Qt source and the native
extension from `build/runtime/python` ahead of any installed Ferrum package.

## Build the local program

```bash
./build.sh
```

The build promotes one complete local program through these canonical paths:

- `build/current/bin/ferrum` for the Rust CLI.
- `build/current/bin/ferrum-qt` for the PySide6 desktop application.
- `build/current/runtime/python/` for the checkout-private Python extension and its
  dependent libraries.

`build/bin/` and `build/runtime/` are stable links through `build/current` and
are the shorter commands used elsewhere in the repository. The launchers
resolve only their sibling runtime below the selected program root; neither
looks for a globally installed Ferrum package or chemistry engine.

## Verify install

```bash
./build.sh
build/bin/ferrum --version
build/bin/ferrum-qt --help
source source_me.sh && python3 -c 'import ferrum_chem; print(ferrum_chem.__file__)'
```

Run the repository acceptance lane after a local change; it verifies hygiene,
the staged runtime and launchers, registered CLI E2E checks, PyO3 bindings,
and offscreen Qt tests. Run the complete Rust-only gate when changing Rust
code:

```bash
./all_test.sh
./check_rust.sh
```

## Known gaps

- Verify a supported cross-platform consumer installation before documenting
  operating systems beyond the current macOS arm64 development route.
