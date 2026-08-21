# Install Ferrum

This repository provides the Rust `ferrum` command-line tool and a bounded Ferrum native
CDML editor for contributors. M20 and M22 source work is accepted for a proposed macOS arm64,
CPython 3.12 release route, but its real offline two-wheel install, relink, source-archive CLI,
artifact-inventory, and human review evidence is pending. It is not yet a generally supported
desktop distribution.

## Requirements

- A checkout of this repository.
- Rust 1.97.1 or newer and Cargo. The workspace records this minimum in
  `packages/ferrum-rust/Cargo.toml`.
- Python 3.12 for Python tools, the native wheel proof, and Ferrum.
- A macOS arm64 host and CPython 3.12 only when running the M20 package-release proof.

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

## Install an engine bundle

The document-only verbs work after the Cargo install. `ferrum convert` and `ferrum coords` also
need a compatible Ferrum engine bundle supplied by the release process for this executable and host.
Install that explicit directory once, then verify its active status:

```bash
ferrum engine install /path/to/ferrum-engine-bundle
ferrum engine status
```

The installer validates the bundle schema, host target, adapter ABI, manifest, and regular
non-symbolic closure before copying it into Ferrum's application-data root. Ferrum does not search
for adapters in a working directory, `PATH`, Python environment, or environment variable. See
[USAGE.md](USAGE.md#convert-a-molecule) for command examples and
[FERRUM_API_CONTRACT.md](FERRUM_API_CONTRACT.md) for the typed unavailable-engine result.

## Cargo policy

- Run workspace commands with `--locked`; `Cargo.lock` selects the exact resolved
  dependency graph.
- The workspace minimum Rust version is 1.97.1.
- Ordinary direct Rust dependencies use `version = "*"`; package publication is
  disabled in the workspace manifest.

## Native bounded-editor setup

The ordinary `ferrum-qt` application requires Python 3.12, the declared Qt dependencies,
and a compatible installed `ferrum-chem` ABI-4 FCM1 extension. After those prerequisites
are present, install the contributor application from the repository root:

```bash
source source_me.sh
python3 -m pip install --editable packages/ferrum-chem-qt.app
```

The source environment script exports unbuffered Python output and disables bytecode
files. It must be sourced, not executed.

## Build local native artifacts

`build.sh` is a source-verified developer build, not bare Maturin packaging. The canonical
storage, cleanup, concurrency, input, and disk-budget contract is
[NATIVE_WHEEL_BUILD.md](NATIVE_WHEEL_BUILD.md). The default developer build is:

```bash
./build.sh
```

Use the printed paths from that invocation in this order:

```bash
source source_me.sh && python3 -m pip install --force-reinstall --no-deps \
  /absolute/path/output_native_wheel/current/wheelhouse/ferrum_chem-*.whl \
  /absolute/path/build/wheelhouse/ferrum_qt-*.whl
build/bin/ferrum engine install \
  /absolute/path/output_native_wheel/current/ferrum-engine-bundle
build/bin/ferrum engine status
ferrum-qt
```

The native builder validates its declared input, compiles and relocates the adapter closure, writes
the wheel receipt, and emits the separately installable CLI engine bundle. The developer command
does not constitute the broader offline release proof below.

## Verify Ferrum

Verify the lightweight command boundary without opening a window:

```bash
ferrum-qt --version
```

## M20 package-release proof

This is a maintainer E2E procedure, not an ordinary contributor install and not a fast pytest.
It is limited to macOS arm64 with CPython 3.12. Before it can run, provision these local inputs:

- a Cargo home that passes the builder's offline locked preflight;
- a source archive or sealed native-input root for the existing Ferrum-Chem builder;
- a target-matching third-party runtime wheelhouse for the Qt manifest; and
- a separate local Qt build-backend wheelhouse containing the required setuptools and wheel
  artifacts.

The source tree does not currently contain the required external wheelhouses, so the following is
the accurate future proof command rather than a release-install instruction:

```bash
source source_me.sh && python3 packages/ferrum-rust/tools/build_release_wheelhouse.py build \
  --output-root output_release_m20 \
  --source-archive-root /absolute/path/native-source-archives \
  --cargo-home /absolute/path/cargo-home \
  --dependency-wheelhouse /absolute/path/qt-runtime-wheelhouse \
  --qt-build-dependency-wheelhouse /absolute/path/qt-build-backend-wheelhouse

source source_me.sh && python3 tests/e2e/e2e_release_wheelhouse.py \
  --release-root output_release_m20 \
  --dependency-wheelhouse /absolute/path/qt-runtime-wheelhouse
```

The builder and E2E use only their named local inputs, scrub ambient Python, pip, and macOS loader
paths, and keep intermediates under the ignored output root or temporary space. The proof builds
the `ferrum-chem` and `ferrum-qt` wheels, installs those selected wheels with `--no-index`, checks
the installed protocol/schema and Qt resource boundaries, and repeats the chemistry observation
after the existing target-specific LGPL relink replacement. A receipt is published only after that
complete observation. This is retained E2E/release evidence, not a timing, hash-equivalence,
member-count, pixel, network, or platform-matrix gate.

`ferrum` is intentionally outside both Python wheels. Install it separately with Cargo as shown
above. Do not treat a successful source check, a local source-tree launch, or a wheel tag as a
supported release.

## M22 release closure

After the M20 receipt exists, retain the release-artifact inventory against the final two wheels,
the committed-release source archive, and that receipt. This maintainer-only closeout phase
delegates all predicate checks to the inventory verifier, then atomically writes
`ferrum-release-artifact-inventory.json` beside the supplied M20 receipt.

```bash
source source_me.sh && python3 packages/ferrum-rust/tools/build_release_wheelhouse.py closeout \
  --chem-wheel /absolute/path/ferrum_chem-26.08-cp312-cp312-macosx_11_0_arm64.whl \
  --qt-wheel /absolute/path/ferrum_qt-26.08-py3-none-any.whl \
  --source-archive /absolute/path/ferrum-26.08.tar.gz \
  --receipt /absolute/path/ferrum-release-package-receipt.json
```

For the standard M20 output root, the receipt is
`output_release_m20/ferrum-release-package-receipt.json` and this command writes
`output_release_m20/ferrum-release-artifact-inventory.json`. The command creates no wheels,
source archives, or release claim. Its output still requires human legal and release review before
publication.

The source archive must retain both root license texts, and the native wheel route prepares the
Ferrum-Chem LGPL, RDKit BSD-3-Clause, InChI MIT, Telex OFL, and reviewed notice-index roles in
its standard distribution-metadata license directory. The final inventory and a human legal and
release review decide whether the actual artifacts meet those predicates. They are one-time
release evidence, not permanent pytest or packaging-count gates.

## Known gaps

- TODO: qualify each additional target platform before documenting it as supported.
- TODO: publish a consumer installation path for the Ferrum native route.
