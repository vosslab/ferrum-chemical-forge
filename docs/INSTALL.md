# Install Ferrum from source

The current source tree provides the Rust `ferrum` command-line executable and a
contributor-preview Ferrum-Qt package. Ferrum-Qt is still migrating away from OASA,
so its source install is not yet the self-contained desktop distribution Ferrum will
eventually ship.

## Rust CLI requirements

- A checkout of this repository.
- Rust 1.85 or newer, matching `packages/ferrum-rust/Cargo.toml`.
- Cargo with the `aarch64-apple-darwin` target for the currently verified macOS arm64
  build.
- Network access for Cargo's first dependency download; later locked builds use the
  local Cargo cache.

## Install the Rust CLI

From the repository root:

```bash
cd packages/ferrum-rust
cargo install --path crates/api --locked --target aarch64-apple-darwin
```

Cargo installs the executable as `ferrum` in its normal binary directory. That
directory must be on `PATH` to invoke the command by name.

## Verify the Rust install

```bash
ferrum --version
```

The current source reports `ferrum 26.8.0`.

## Install the Qt preview

Ferrum-Qt currently requires the declared PySide6 and migration-only OASA Python
dependencies. From the repository root, install the source package into the active
Python 3.12 environment:

```bash
source source_me.sh
python3 -m pip install --editable packages/ferrum-chem-qt.app
```

The installed application command is `ferrum-qt`. Verify the command without opening
a window:

```bash
ferrum-qt --version
```

The current source reports `Ferrum-Qt 26.08`.

## Current installation gaps

- The native-wheel proof has passed only on macOS arm64. It proves a minimal
  clean-environment install and LGPL relink route, not a supported consumer package.
- Qualify and document platforms other than macOS arm64 before claiming support.
- Add the self-contained Ferrum-Qt wheel after its Rust backend cutover; the preview
  install above still resolves the temporary OASA dependency.
