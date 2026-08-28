# Install

Ferrum is a pre-production local-build project. A successful build creates the
Rust CLI, PySide6 desktop launcher, and private native runtime below `build/`;
it does not publish or globally install a Ferrum package.

## Requirements

- A checkout of this repository on the current macOS arm64 development route.
- Rust 1.97.1 or newer with Cargo, as declared in
  [`packages/ferrum-rust/Cargo.toml`](../packages/ferrum-rust/Cargo.toml).
- Homebrew's Python 3.12 and the Python dependencies in
  [`pip_requirements-dev.txt`](../pip_requirements-dev.txt).
- The Homebrew tools declared in [`Brewfile`](../Brewfile), including CMake,
  LLVM, and Rustup.

## Install dependencies

From the repository root, install the declared macOS tools and Python
dependencies:

```bash
brew bundle
python3 -m pip install -r pip_requirements-dev.txt
```

Do not source `source_me.sh` before the first build: it deliberately refuses
to load until the checkout has a matching staged native extension. After a
successful build, use `source source_me.sh && python3` for repository Python
commands.

## Build the local program

```bash
./build.sh
```

The build produces these local launchers:

- `build/bin/ferrum` for the Rust CLI.
- `build/bin/ferrum-qt` for the PySide6 desktop application.
- `build/runtime/python/` for the checkout-private Python extension and its
  dependent libraries.

The launchers resolve only their sibling runtime under `build/`; neither looks
for a globally installed Ferrum package or chemistry engine.

## Verify install

```bash
./build.sh
./all_test.sh
```

`all_test.sh` verifies repository hygiene, the staged runtime and launchers,
registered CLI E2E checks, PyO3 bindings, and offscreen Qt tests. Run the
complete Rust-only gate when changing Rust code:

```bash
./check_rust.sh
```

## Known gaps

- Verify a supported cross-platform consumer installation before documenting
  operating systems beyond the current macOS arm64 development route.
