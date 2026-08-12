# Graph analysis parity report

## Verdict

M3 is complete. Ferrum provides the required graph algorithms through an immutable,
dependency-private analysis view and owns a deterministic fundamental-cycle policy.
Its M2 prerequisite is green. Ten fixed topologies cover cyclic, fused, bridged,
acyclic, disconnected, and boundary graphs; every non-cycle discrete result matches
the current historical reference.

## Evidence boundary

The reference checkout was executed read-only as a one-time oracle on 2026-08-11. Its
topologies and discrete results were transcribed into Ferrum-owned Rust fixtures. The
permanent tests do not read, import, package, or locate that checkout, so deleting
`OTHER_REPOS/` does not change the build or the test result.

The comparison covers vertex and edge count, connected-component sizes, connectivity,
diameter, bridge count, articulation-point count, maximum-matching cardinality,
cycle-space rank, and sorted cycle lengths. Floating-point behavior is not involved.

| Topology | V | E | Components | Diameter | Bridges | Articulation | Matching | Rank | Reference cycles | Ferrum cycles |
| --- | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| Benzene | 6 | 6 | 6 | 3 | 0 | 0 | 3 | 1 | 6 | 6 |
| Cholesterol | 28 | 31 | 28 | 15 | 11 | 9 | 13 | 4 | 5, 6, 6, 6 | 5, 6, 6, 6 |
| Naphthalene | 10 | 11 | 10 | 5 | 0 | 0 | 5 | 2 | 6, 6 | 6, 6 |
| Steroid skeleton | 17 | 20 | 17 | 8 | 0 | 0 | 8 | 4 | 5, 6, 6, 6 | 5, 6, 6, 6 |
| Caffeine | 14 | 15 | 14 | 6 | 5 | 5 | 7 | 2 | 5, 6 | 5, 6 |
| Hexane | 6 | 5 | 6 | 5 | 5 | 4 | 3 | 0 | none | none |
| Single atom | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | none | none |
| Two disconnected bonds | 4 | 2 | 2, 2 | 1 | 2 | 0 | 2 | 0 | none | none |
| Cyclopentane | 5 | 5 | 5 | 2 | 0 | 0 | 2 | 1 | 5 | 5 |
| Bridged bicyclic | 7 | 8 | 7 | 3 | 0 | 0 | 3 | 2 | 5, 6 | 5, 5 |

## Classified divergence

The bridged-bicyclic cycle-size difference is an intended level-4 Ferrum change. Both
outputs contain the required two independent fundamental cycles. Ferrum selects the
5/5 basis because its stable-BFS candidate score has a lower total length than the 5/6
reference basis. This follows the general policy; no topology or molecule name is
recognized by the implementation.

The current reference basis was stable over 100 calls. This completion run therefore
does not claim present run-to-run instability. Ferrum's reason for owning the policy is
the smaller deterministic basis and independence from undocumented dependency traversal.

## Repeatability and boundaries

Every fixed topology repeats both its exact Ferrum cycle basis and exact matching pairs
100 times. Separate properties cover conventional source-to-target path order, stable
tie breaking, parallel-bond cycles, disconnected distance `None`, unknown-vertex errors,
all four vertex kinds, empty graphs, and single-vertex graphs.

Validation on macOS arm64 with the repository's locked dependencies:

```text
cargo test -p ferrum-core --target aarch64-apple-darwin
25 passed

cargo clippy -p ferrum-core --target aarch64-apple-darwin --all-targets -- -D warnings
passed
```
