//! Deterministic graph analysis over immutable Ferrum molecules.

use std::collections::{HashMap, VecDeque};

use petgraph::algo::articulation_points::articulation_points;
use petgraph::algo::{
    bridges, connected_components, dijkstra, floyd_warshall, has_path_connecting, maximum_matching,
};
use petgraph::graph::NodeIndex;

use crate::{Molecule, RecordId, VertexRef};

use super::model::{AnalysisGraph, EdgeData, NodeData};
use super::{
    AllPairsDistances, ConnectedComponent, FundamentalCycle, MatchingPair, MoleculeGraph,
    UnknownGraphVertex, VertexDistance, cycles,
};

impl Molecule {
    /// Build one reusable graph-analysis view of this immutable molecule.
    #[must_use]
    pub fn graph(&self) -> MoleculeGraph {
        MoleculeGraph::new(self)
    }
}

impl MoleculeGraph {
    /// Build the private petgraph mirror from a validated molecule.
    #[must_use]
    pub fn new(molecule: &Molecule) -> Self {
        let mut graph = AnalysisGraph::new_undirected();
        let mut index_by_vertex = HashMap::new();
        let vertices = molecule
            .atoms()
            .iter()
            .map(|vertex| VertexRef::Atom(vertex.identity().clone()))
            .chain(
                molecule
                    .groups()
                    .iter()
                    .map(|vertex| VertexRef::Group(vertex.identity().clone())),
            )
            .chain(
                molecule
                    .texts()
                    .iter()
                    .map(|vertex| VertexRef::Text(vertex.identity().clone())),
            )
            .chain(
                molecule
                    .queries()
                    .iter()
                    .map(|vertex| VertexRef::Query(vertex.identity().clone())),
            );
        for vertex in vertices {
            let index = graph.add_node(NodeData {
                vertex: vertex.clone(),
            });
            let previous = index_by_vertex.insert(vertex, index);
            debug_assert!(previous.is_none(), "validated molecule vertices are unique");
        }
        for (source_order, bond) in molecule.bonds().iter().enumerate() {
            let start = index_by_vertex[bond.start()];
            let end = index_by_vertex[bond.end()];
            graph.add_edge(
                start,
                end,
                EdgeData {
                    bond: bond.identity().clone(),
                    source_order,
                },
            );
        }
        Self {
            graph,
            index_by_vertex,
        }
    }

    /// Return the number of graph vertices.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Return the number of graph bonds.
    #[must_use]
    pub fn bond_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Return connected components with deterministic member and component order.
    #[must_use]
    pub fn connected_components(&self) -> Vec<ConnectedComponent> {
        let mut seen = vec![false; self.graph.node_count()];
        let mut components = Vec::new();
        for start in self.graph.node_indices() {
            if seen[start.index()] {
                continue;
            }
            let mut queue = VecDeque::from([start]);
            seen[start.index()] = true;
            let mut members = Vec::new();
            while let Some(node) = queue.pop_front() {
                members.push(node);
                let mut neighbors = self.graph.neighbors(node).collect::<Vec<_>>();
                neighbors.sort_by_key(|neighbor| neighbor.index());
                for neighbor in neighbors {
                    if !seen[neighbor.index()] {
                        seen[neighbor.index()] = true;
                        queue.push_back(neighbor);
                    }
                }
            }
            members.sort_by_key(|node| node.index());
            components.push(ConnectedComponent {
                vertices: members
                    .into_iter()
                    .map(|node| self.graph[node].vertex.clone())
                    .collect(),
            });
        }
        components
    }

    /// Return true for an empty graph or one connected component.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.graph.node_count() == 0 || connected_components(&self.graph) == 1
    }

    /// Test path connectivity between two known vertices.
    pub fn has_path(&self, from: &VertexRef, to: &VertexRef) -> Result<bool, UnknownGraphVertex> {
        let from = self.node_index(from)?;
        let to = self.node_index(to)?;
        Ok(has_path_connecting(&self.graph, from, to, None))
    }

    /// Return bridge bond identities in source order.
    #[must_use]
    pub fn bridges(&self) -> Vec<RecordId> {
        let mut found = bridges(&self.graph)
            .map(|edge| edge.weight().clone())
            .collect::<Vec<_>>();
        found.sort_by_key(|edge| edge.source_order);
        found.into_iter().map(|edge| edge.bond).collect()
    }

    /// Return articulation vertices in stable graph order.
    #[must_use]
    pub fn articulation_points(&self) -> Vec<VertexRef> {
        let mut found = articulation_points(&self.graph)
            .into_iter()
            .collect::<Vec<_>>();
        found.sort_by_key(|node| node.index());
        found
            .into_iter()
            .map(|node| self.graph[node].vertex.clone())
            .collect()
    }

    /// Return one deterministic maximum-cardinality matching.
    ///
    /// Petgraph's source-order node and edge traversal chooses among equally large
    /// matchings. Ferrum canonicalizes endpoint and pair order before returning it.
    #[must_use]
    pub fn maximum_matching(&self) -> Vec<MatchingPair> {
        let matching = maximum_matching(&self.graph);
        let mut pairs = matching
            .edges()
            .map(|(first, second)| {
                if first.index() <= second.index() {
                    (first, second)
                } else {
                    (second, first)
                }
            })
            .collect::<Vec<_>>();
        pairs.sort_by_key(|(first, second)| (first.index(), second.index()));
        pairs
            .into_iter()
            .map(|(first, second)| MatchingPair {
                first: self.graph[first].vertex.clone(),
                second: self.graph[second].vertex.clone(),
            })
            .collect()
    }

    /// Return Dijkstra unit-edge distances to every reachable vertex.
    pub fn distances_from(
        &self,
        source: &VertexRef,
    ) -> Result<Vec<VertexDistance>, UnknownGraphVertex> {
        let source = self.node_index(source)?;
        let distances = dijkstra(&self.graph, source, None, |_| 1_usize);
        Ok(self
            .graph
            .node_indices()
            .filter_map(|node| {
                distances.get(&node).map(|distance| VertexDistance {
                    vertex: self.graph[node].vertex.clone(),
                    distance: *distance,
                })
            })
            .collect())
    }

    /// Return a deterministic shortest path in conventional source-to-target order.
    pub fn shortest_path(
        &self,
        source: &VertexRef,
        target: &VertexRef,
    ) -> Result<Option<Vec<VertexRef>>, UnknownGraphVertex> {
        let source = self.node_index(source)?;
        let target = self.node_index(target)?;
        let distances = dijkstra(&self.graph, source, None, |_| 1_usize);
        let Some(mut distance) = distances.get(&target).copied() else {
            return Ok(None);
        };
        let mut reversed = vec![target];
        let mut current = target;
        while current != source {
            let mut predecessors = self
                .graph
                .neighbors(current)
                .filter(|neighbor| distances.get(neighbor) == Some(&(distance - 1)))
                .collect::<Vec<_>>();
            predecessors.sort_by_key(|node| node.index());
            current = *predecessors
                .first()
                .expect("a positive shortest-path distance has a predecessor");
            reversed.push(current);
            distance -= 1;
        }
        reversed.reverse();
        Ok(Some(
            reversed
                .into_iter()
                .map(|node| self.graph[node].vertex.clone())
                .collect(),
        ))
    }

    /// Return Floyd-Warshall unit-edge distances for every ordered vertex pair.
    #[must_use]
    pub fn all_pairs_distances(&self) -> AllPairsDistances {
        let distances = floyd_warshall(&self.graph, |_| 1_usize)
            .expect("positive unit edges cannot form a negative cycle");
        let vertices = self
            .graph
            .node_indices()
            .map(|node| self.graph[node].vertex.clone())
            .collect::<Vec<_>>();
        let index_by_vertex = vertices
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, vertex)| (vertex, index))
            .collect();
        let rows = self
            .graph
            .node_indices()
            .map(|from| {
                self.graph
                    .node_indices()
                    .map(|to| distances[&(from, to)])
                    .map(|distance| (distance != usize::MAX).then_some(distance))
                    .collect()
            })
            .collect();
        AllPairsDistances {
            vertices,
            rows,
            index_by_vertex,
        }
    }

    /// Return the largest finite shortest-path distance, or zero for no edges.
    #[must_use]
    pub fn diameter(&self) -> usize {
        self.all_pairs_distances()
            .rows
            .iter()
            .flatten()
            .flatten()
            .copied()
            .max()
            .unwrap_or(0)
    }

    /// Return the undirected cycle-space dimension `E - V + components`.
    #[must_use]
    pub fn cycle_rank(&self) -> usize {
        self.graph.edge_count() + connected_components(&self.graph) - self.graph.node_count()
    }

    /// Return Ferrum's canonical shortest stable-BFS fundamental cycle basis.
    ///
    /// Candidate roots are ordered by total cycle length, longest cycle, sorted
    /// lengths, canonical cycles, root order, and tree-edge order. This gives stable
    /// results independent of dependency traversal details.
    #[must_use]
    pub fn cycle_basis(&self) -> Vec<FundamentalCycle> {
        cycles::fundamental_cycle_basis(&self.graph)
            .into_iter()
            .map(|cycle| FundamentalCycle {
                vertices: cycle
                    .vertices
                    .into_iter()
                    .map(|node| self.graph[node].vertex.clone())
                    .collect(),
                bonds: cycle
                    .edges
                    .into_iter()
                    .map(|edge| self.graph[edge].bond.clone())
                    .collect(),
            })
            .collect()
    }

    fn node_index(&self, vertex: &VertexRef) -> Result<NodeIndex, UnknownGraphVertex> {
        self.index_by_vertex
            .get(vertex)
            .copied()
            .ok_or_else(|| UnknownGraphVertex {
                vertex: vertex.clone(),
            })
    }
}
