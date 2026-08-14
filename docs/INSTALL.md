# Install Ferrum

This repository currently provides the Rust `ferrum` command-line tool and a
contributor-only Ferrum-Qt bounded native CDML editor. The macOS arm64 Python wheel is
packaging evidence, not a generally supported desktop distribution.

## Requirements

- A checkout of this repository.
- Rust 1.97.1 or newer and Cargo. The workspace records this minimum in
  `packages/ferrum-rust/Cargo.toml`.
- Python 3.12 for Python tools, the native wheel proof, and Ferrum-Qt.
- A macOS arm64 host only when running the native-wheel proof.

## Install the CLI

From the repository root, install the Rust command into Cargo's normal binary
directory:

```bash
cargo install --path packages/ferrum-rust/crates/api --locked
```

Ensure Cargo's binary directory is on `PATH`, then verify the installed command:

```bash
ferrum --version
```

The current source reports `ferrum 26.8.0`.

## Cargo policy

- Run workspace commands with `--locked`; `Cargo.lock` selects the exact resolved
  dependency graph.
- The workspace minimum Rust version is 1.97.1.
- Ordinary direct Rust dependencies use `version = "*"`; package publication is
  disabled in the workspace manifest.

## Native bounded-editor setup

The `ferrum-qt --native` route requires Python 3.12, the declared Qt dependencies,
and a compatible installed `ferrum-chem` ABI-4 FCM1 extension. After those
prerequisites are present, install the contributor application from the repository root:

```bash
source source_me.sh
python3 -m pip install --editable packages/ferrum-chem-qt.app
```

The source environment script exports unbuffered Python output and disables bytecode
files. It must be sourced, not executed.

## Verify the native route

Verify the lightweight command boundary without opening a window:

```bash
ferrum-qt --version
```

## Native-wheel evidence

The native-wheel proof is limited to macOS arm64. From the repository root on that
platform, run:

```bash
source source_me.sh && PYTHONDONTWRITEBYTECODE=1 python3 \
  tests/e2e/e2e_native_wheel.py
```

It verifies a clean-environment wheel install and LGPL relinking route for the sealed
ABI-4 FCM1 profile. It does not establish cross-platform support, a consumer wheel
release, or a completed OASA replacement.

## Known gaps

- TODO: qualify each additional target platform before documenting it as supported.
- TODO: publish a consumer installation path for the Ferrum-Qt native route.
