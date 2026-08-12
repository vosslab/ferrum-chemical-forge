# Geometry and straighten parity evidence

## Scope

This M11 evidence covers the Ferrum-owned finite two-dimensional representation,
explicit `kurbo` and `nalgebra` boundary conversion, affine transforms, directional
wedge arithmetic, pointy-top hex-grid snapping, and a Rust-only port of RDKit
`straightenDepiction`. No Ferrum geometry code calls or links RDKit.

## Oracle and provenance handoff

The straightening implementation was derived from
`OTHER_REPOS/rdkit/Code/GraphMol/Depictor/RDDepictor.cpp`, function
`straightenDepiction`, at RDKit source revision
`d1f7d6a59d712ddaf732b60173fd6223b3cd5003`. The BSD-3-derived algorithm,
source revision, reason for the derivation, and non-copying boundary are recorded
in `docs/PROVENANCE.md`.

The separately launched oracle command was:

```bash
source source_me.sh && python3 devel/measure_straighten_depiction.py
```

It used RDKit 2026.03.5 and executes each input 25 times in the Python oracle
process. It then launches the Rust evidence process separately and compares the
same named cases, both `minimizeRotation` branches, every coordinate component,
and the reported applied rotation. The applied rotation is authoritatively derived
from the full input/output coordinate transform using the summed dot and cross
products, not from a displacement of one atom.

```bash
cd packages/ferrum-rust
cargo run --quiet -p ferrum-geometry --example straighten_probe
```

For the asymmetric three-bond input, Ferrum produced the same coordinates as
RDKit for both branches to the printed f64 precision. `minimize_rotation=false`
applied `-0.38615438234591110` radians, and `true` applied
`0.13744439325238780` radians. The ten-degree single-bond input exercised the
same selected rotation in both branches. Fifteen-degree and thirty-degree inputs
exercise the half-increment and exact-increment boundaries. The local cross-process
comparison's largest coordinate difference was `3.645723512257204e-18`, and its
largest applied-rotation difference was `0.0` radians.

## Repeatability and tolerance status

The maximum component variation across the 25 oracle repeats was `0.0` for both
inputs and for both `minimizeRotation` branches. This establishes a local oracle
repeatability observation, not a cross-platform tolerance. No cross-platform
measurement has been run, so M11's plan-level acceptance tolerance remains open.
The executable comparison records the current local result but deliberately does
not claim a CI threshold: the required platform sweep must set the threshold outside
its measured variation rather than add a guessed epsilon.

## Rust validation

```text
cargo test -p ferrum-geometry        12 passed
cargo check -p ferrum-geometry       passed
cargo clippy -p ferrum-geometry -- -D warnings  passed
cargo fmt --check                    passed
```

The unit suite covers conversion rejection, transform composition, wedge direction
and area, deterministic Euclidean hex-grid ties, bounded and invalid overlays,
unrepresentable finite lattice requests, invalid bond endpoints, both straighten
branches, increment boundaries, and the documented zero-length-bond policy. The
nearest-grid tie is lexicographic by `HexIndex`; the test asserts its selected
coordinate, not an incidental collection count.

## Design boundary

`ferrum_geometry::Point2` owns finite Ferrum coordinate values. They are unitless
drawing coordinates in an ordinary y-up Cartesian frame; positive rotation is
counter-clockwise. A y-down screen renderer converts its frame at its own boundary.
`kurbo::Point` and `nalgebra::Point2<f64>` are only explicit conversion targets at
renderer and linear-algebra boundaries. This keeps persistent CDML facts independent
of either library while retaining maintained infrastructure where it is justified.

The public hex-overlay API returns `GridIndexUnrepresentable` when finite rectangle
values cannot be safely represented in the signed lattice basis. It returns `None`
only for a representable request exceeding the explicit display budget. Inverted
bounds are an error. `straighten_depiction` intentionally follows RDKit's x-clamp
for a zero-length bond, treating it as a horizontal zero-angle contribution; callers
that need to reject that degenerate input do so before normalization.
