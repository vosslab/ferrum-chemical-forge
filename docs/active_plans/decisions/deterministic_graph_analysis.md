# Deterministic graph analysis decision

Status: implemented for M3 on 2026-08-11.

## Boundary

`Molecule::graph()` creates an immutable `MoleculeGraph` analysis view. The private
implementation uses `petgraph::UnGraph`, but no `NodeIndex`, `EdgeIndex`, graph handle,
or dependency-specific error crosses the public API. Results contain owned Ferrum
identities, immutable vectors, optional distances, or the owned `UnknownGraphVertex`
error.

Vertices enter the graph in core order: atoms, groups, molecule-local text, then
queries. Bonds enter in molecule source order. Public components, bridge bonds,
articulation vertices, matching pairs, distance rows, paths, and cycles translate back
to those stable identities before they leave the boundary.

## Algorithms

Ferrum uses the locked `petgraph` implementation for connected-component count, path
connectivity, bridges, articulation points, maximum-cardinality matching, Dijkstra,
and Floyd-Warshall. Ferrum supplies ordering and result conversion around those
algorithms. An empty graph is connected by the public convenience predicate, its
diameter is zero, and unreachable all-pairs distances are `None`, never a sentinel.

Shortest paths are reconstructed from the complete Dijkstra distance map. When more
than one predecessor is equally short, the lower stable vertex order wins. Maximum
matching is canonicalized by endpoint order and then pair order; the locked graph and
dependency versions make the selected maximum-cardinality matching repeatable.

## Fundamental-cycle policy

Cycle selection is Ferrum code rather than dependency behavior. For every connected
component it performs these steps:

1. Build one breadth-first spanning tree from each possible root. Neighbor traversal
   uses bond source order, then edge index and stable vertex order.
2. Form one fundamental cycle for every non-tree edge from the unique tree path plus
   that closing edge.
3. Select the candidate with the lexicographically smallest score: total cycle length,
   largest cycle length, sorted cycle lengths, canonical cycle keys, root order, then
   sorted tree-edge indexes.
4. Canonicalize each selected cycle across both directions and every rotation, comparing
   stable vertex order before bond source order.

This is a deterministic shortest stable-BFS fundamental basis. It is not a smallest-set
of-smallest-rings claim, aromaticity perception, or chemical ring-family model. A pair
of parallel bonds therefore has a valid two-edge graph cycle. Disconnected components
receive independent bases, and the number of returned cycles is always
`edges - vertices + components`.

## Compatibility consequence

The fixed compatibility topologies match the historical backend for every discrete
graph result except the basis selected for one bridged graph. The historical backend
returns cycle lengths 5 and 6; Ferrum returns 5 and 5. This is an intended level-4
Ferrum behavior: the project rule minimizes the total length of a stable fundamental
basis and does not contain a bridged-molecule special case.

The current reference checkout returned the same cycle selection in 100 repeated calls.
Earlier planning recorded instability in an older path, but that was not reproduced by
the completion probe and is not presented as current evidence. Ferrum still owns the
policy so a transitive graph-library traversal change cannot redefine its document
behavior.
