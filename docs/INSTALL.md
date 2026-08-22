# Run Ferrum from a checkout

Ferrum is pre-production and currently runs from this repository's local build.
The supported contributor path creates no global Ferrum installation and no
published wheel. [`build.sh`](../build.sh) stages the local CLI and Qt launcher
under `build/`; [`all_test.sh`](../all_test.sh) verifies that local runtime.

## Requirements

- A checkout of this repository.
- Rust 1.97.1 or newer and Cargo. The workspace records this minimum in
  [`packages/ferrum-rust/Cargo.toml`](../packages/ferrum-rust/Cargo.toml).
- Python 3.12 and the dependencies in [`pip_requirements-dev.txt`](../pip_requirements-dev.txt).
- A macOS arm64 host for the current native and Qt route.

Set up the Python test dependencies once:

```bash
source source_me.sh && python3 -m pip install -r pip_requirements-dev.txt
```

## Build and run locally

From the repository root, build the local application:

```bash
./build.sh
```

Run the resulting applications directly from `build/`:

```bash
build/bin/ferrum --version
build/bin/ferrum-qt
```

`build/bin/ferrum-qt drawing.cdml` opens one local CDML drawing. The local
launchers select the ABI-specific extension at
`build/runtime/python/ferrum_chem<Python ABI suffix>` and its adjacent
`.dylibs/` closure. The CLI derives its validated chemistry closure only from
its sibling `build/runtime/engine-v1` directory. These local programs do not
use a globally installed Ferrum package or a per-user engine installation.

## Verify the local build

```bash
./build.sh
./all_test.sh
./check_rust.sh
```

`all_test.sh` starts with repository hygiene checks and then runs the PyO3 and
Qt suites against the local extension. It is the normal developer and CI test
entry point.
