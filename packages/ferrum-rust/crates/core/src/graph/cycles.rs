//! Ferrum-owned deterministic fundamental-cycle selection.

use std::collections::{BTreeSet, VecDeque};

use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::visit::EdgeRef;

use super::model::AnalysisGraph;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct IndexedCycle {
    pub(super) vertices: Vec<NodeIndex>,
    pub(super) edges: Vec<EdgeIndex>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CycleKey {
    vertices: Vec<usize>,
    edges: Vec<(usize, usize)>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct BasisScore {
    total_length: usize,
    maximum_length: usize,
    sorted_lengths: Vec<usize>,
    canonical_cycles: Vec<CycleKey>,
    root: usize,
    tree_edges: Vec<(usize, usize)>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BasisCandidate {
    score: BasisScore,
    cycles: Vec<IndexedCycle>,
}

pub(super) fn fundamental_cycle_basis(graph: &AnalysisGraph) -> Vec<IndexedCycle> {
    let adjacency = stable_adjacency(graph);
    let components = stable_components(graph, &adjacency);
    let edge_order = stable_edge_order(graph);
    let mut result = Vec::new();

    for component in components {
        let members = component_membership(graph, &component);
        let component_edges = edge_order
            .iter()
            .copied()
            .filter(|edge| {
                let (start, _) = graph
                    .edge_endpoints(*edge)
                    .expect("graph edge indexes remain valid");
                members[start.index()]
            })
            .collect::<Vec<_>>();
        // Every stable BFS root yields a valid fundamental basis. Select globally by
        // total length, longest cycle, sorted lengths, canonical cycles, root, then
        // tree-edge order so neither dependency traversal nor hash order chooses it.
        let best = component
            .iter()
            .copied()
            .map(|root| candidate_for_root(graph, &adjacency, &component_edges, root))
            .min_by(|left, right| left.score.cmp(&right.score))
            .expect("a connected component contains at least one vertex");
        result.extend(best.cycles);
    }

    result
}

fn stable_adjacency(graph: &AnalysisGraph) -> Vec<Vec<(EdgeIndex, NodeIndex)>> {
    let mut adjacency = vec![Vec::new(); graph.node_count()];
    for edge in graph.edge_references() {
        adjacency[edge.source().index()].push((edge.id(), edge.target()));
        adjacency[edge.target().index()].push((edge.id(), edge.source()));
    }
    for neighbors in &mut adjacency {
        neighbors
            .sort_by_key(|(edge, node)| (graph[*edge].source_order, edge.index(), node.index()));
    }
    adjacency
}

fn stable_edge_order(graph: &AnalysisGraph) -> Vec<EdgeIndex> {
    let mut edges = graph.edge_indices().collect::<Vec<_>>();
    edges.sort_by_key(|edge| (graph[*edge].source_order, edge.index()));
    edges
}

fn stable_components(
    graph: &AnalysisGraph,
    adjacency: &[Vec<(EdgeIndex, NodeIndex)>],
) -> Vec<Vec<NodeIndex>> {
    let mut seen = vec![false; graph.node_count()];
    let mut components = Vec::new();
    for start in graph.node_indices() {
        if seen[start.index()] {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut component = Vec::new();
        seen[start.index()] = true;
        while let Some(node) = queue.pop_front() {
            component.push(node);
            for &(_, neighbor) in &adjacency[node.index()] {
                if !seen[neighbor.index()] {
                    seen[neighbor.index()] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        component.sort_by_key(|node| node.index());
        components.push(component);
    }
    components
}

fn component_membership(graph: &AnalysisGraph, component: &[NodeIndex]) -> Vec<bool> {
    let mut members = vec![false; graph.node_count()];
    for node in component {
        members[node.index()] = true;
    }
    members
}

fn candidate_for_root(
    graph: &AnalysisGraph,
    adjacency: &[Vec<(EdgeIndex, NodeIndex)>],
    component_edges: &[EdgeIndex],
    root: NodeIndex,
) -> BasisCandidate {
    let tree_edges = breadth_first_tree(adjacency, root);
    let tree_adjacency = tree_adjacency(graph, &tree_edges);
    let cycles = component_edges
        .iter()
        .copied()
        .filter(|edge| !tree_edges.contains(edge))
        .map(|closing_edge| {
            let (start, end) = graph
                .edge_endpoints(closing_edge)
                .expect("graph edge indexes remain valid");
            let (vertices, mut edges) = tree_path(&tree_adjacency, start, end);
            edges.push(closing_edge);
            canonical_cycle(graph, vertices, edges)
        })
        .collect::<Vec<_>>();
    let mut sorted_lengths = cycles
        .iter()
        .map(|cycle| cycle.edges.len())
        .collect::<Vec<_>>();
    sorted_lengths.sort_unstable();
    let mut canonical_cycles = cycles
        .iter()
        .map(|cycle| cycle_key(graph, cycle))
        .collect::<Vec<_>>();
    canonical_cycles.sort();
    let mut scored_tree_edges = tree_edges
        .iter()
        .map(|edge| (graph[*edge].source_order, edge.index()))
        .collect::<Vec<_>>();
    scored_tree_edges.sort_unstable();
    let score = BasisScore {
        total_length: sorted_lengths.iter().sum(),
        maximum_length: sorted_lengths.last().copied().unwrap_or(0),
        sorted_lengths,
        canonical_cycles,
        root: root.index(),
        tree_edges: scored_tree_edges,
    };
    BasisCandidate { score, cycles }
}

fn breadth_first_tree(
    adjacency: &[Vec<(EdgeIndex, NodeIndex)>],
    root: NodeIndex,
) -> BTreeSet<EdgeIndex> {
    let mut seen = vec![false; adjacency.len()];
    let mut queue = VecDeque::from([root]);
    let mut tree_edges = BTreeSet::new();
    seen[root.index()] = true;
    while let Some(node) = queue.pop_front() {
        for &(edge, neighbor) in &adjacency[node.index()] {
            if !seen[neighbor.index()] {
                seen[neighbor.index()] = true;
                tree_edges.insert(edge);
                queue.push_back(neighbor);
            }
        }
    }
    tree_edges
}

fn tree_adjacency(
    graph: &AnalysisGraph,
    tree_edges: &BTreeSet<EdgeIndex>,
) -> Vec<Vec<(EdgeIndex, NodeIndex)>> {
    let mut adjacency = vec![Vec::new(); graph.node_count()];
    for &edge in tree_edges {
        let (start, end) = graph
            .edge_endpoints(edge)
            .expect("graph edge indexes remain valid");
        adjacency[start.index()].push((edge, end));
        adjacency[end.index()].push((edge, start));
    }
    for neighbors in &mut adjacency {
        neighbors
            .sort_by_key(|(edge, node)| (graph[*edge].source_order, edge.index(), node.index()));
    }
    adjacency
}

fn tree_path(
    adjacency: &[Vec<(EdgeIndex, NodeIndex)>],
    start: NodeIndex,
    end: NodeIndex,
) -> (Vec<NodeIndex>, Vec<EdgeIndex>) {
    if start == end {
        return (vec![start], Vec::new());
    }
    let mut parent_nodes = vec![None; adjacency.len()];
    let mut parent_edges = vec![None; adjacency.len()];
    let mut queue = VecDeque::from([start]);
    parent_nodes[start.index()] = Some(start);
    while let Some(node) = queue.pop_front() {
        if node == end {
            break;
        }
        for &(edge, neighbor) in &adjacency[node.index()] {
            if parent_nodes[neighbor.index()].is_none() {
                parent_nodes[neighbor.index()] = Some(node);
                parent_edges[neighbor.index()] = Some(edge);
                queue.push_back(neighbor);
            }
        }
    }

    let mut vertices = vec![end];
    let mut edges = Vec::new();
    let mut current = end;
    while current != start {
        edges.push(parent_edges[current.index()].expect("tree endpoints have one path"));
        current = parent_nodes[current.index()].expect("tree endpoints have one path");
        vertices.push(current);
    }
    vertices.reverse();
    edges.reverse();
    (vertices, edges)
}

fn canonical_cycle(
    graph: &AnalysisGraph,
    vertices: Vec<NodeIndex>,
    edges: Vec<EdgeIndex>,
) -> IndexedCycle {
    debug_assert_eq!(vertices.len(), edges.len());
    debug_assert!(!vertices.is_empty());
    let mut orientations = vec![(vertices.clone(), edges.clone())];
    let mut reversed_vertices = Vec::with_capacity(vertices.len());
    reversed_vertices.push(vertices[0]);
    reversed_vertices.extend(vertices[1..].iter().rev().copied());
    let reversed_edges = edges.iter().rev().copied().collect::<Vec<_>>();
    orientations.push((reversed_vertices, reversed_edges));

    let mut best = None;
    for (oriented_vertices, oriented_edges) in orientations {
        for offset in 0..oriented_vertices.len() {
            let rotated_vertices = oriented_vertices[offset..]
                .iter()
                .chain(&oriented_vertices[..offset])
                .copied()
                .collect::<Vec<_>>();
            let rotated_edges = oriented_edges[offset..]
                .iter()
                .chain(&oriented_edges[..offset])
                .copied()
                .collect::<Vec<_>>();
            let key = CycleKey {
                vertices: rotated_vertices.iter().map(|node| node.index()).collect(),
                edges: rotated_edges
                    .iter()
                    .map(|edge| (graph[*edge].source_order, edge.index()))
                    .collect(),
            };
            let candidate = (key, rotated_vertices, rotated_edges);
            if best
                .as_ref()
                .is_none_or(|(best_key, _, _)| candidate.0 < *best_key)
            {
                best = Some(candidate);
            }
        }
    }
    let (_, vertices, edges) = best.expect("a nonempty cycle has an orientation");
    IndexedCycle { vertices, edges }
}

fn cycle_key(graph: &AnalysisGraph, cycle: &IndexedCycle) -> CycleKey {
    CycleKey {
        vertices: cycle.vertices.iter().map(|node| node.index()).collect(),
        edges: cycle
            .edges
            .iter()
            .map(|edge| (graph[*edge].source_order, edge.index()))
            .collect(),
    }
}
