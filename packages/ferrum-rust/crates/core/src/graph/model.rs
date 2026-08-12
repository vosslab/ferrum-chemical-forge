//! Public graph-query values and the private `petgraph` backing model.

use std::collections::HashMap;

use petgraph::graph::{NodeIndex, UnGraph};
use thiserror::Error;

use crate::{RecordId, VertexRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct NodeData {
    pub(super) vertex: VertexRef,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct EdgeData {
    pub(super) bond: RecordId,
    pub(super) source_order: usize,
}

pub(super) type AnalysisGraph = UnGraph<NodeData, EdgeData>;

/// A vertex supplied to a graph query is not part of the analyzed molecule.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
#[error("vertex is not present in this molecule graph: {vertex:?}")]
pub struct UnknownGraphVertex {
    /// The rejected typed vertex reference.
    pub vertex: VertexRef,
}

/// One connected component with members in Ferrum's stable graph order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConnectedComponent {
    pub(super) vertices: Vec<VertexRef>,
}

impl ConnectedComponent {
    /// Return component members in stable graph order.
    #[must_use]
    pub fn vertices(&self) -> &[VertexRef] {
        &self.vertices
    }
}

/// One reachable vertex and its unit-edge distance from a query source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VertexDistance {
    pub(super) vertex: VertexRef,
    pub(super) distance: usize,
}

impl VertexDistance {
    /// Return the reached vertex.
    #[must_use]
    pub fn vertex(&self) -> &VertexRef {
        &self.vertex
    }

    /// Return its shortest unit-edge distance.
    #[must_use]
    pub fn distance(&self) -> usize {
        self.distance
    }
}

/// A pair selected by maximum-cardinality matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchingPair {
    pub(super) first: VertexRef,
    pub(super) second: VertexRef,
}

impl MatchingPair {
    /// Return the lower stable-order endpoint.
    #[must_use]
    pub fn first(&self) -> &VertexRef {
        &self.first
    }

    /// Return the higher stable-order endpoint.
    #[must_use]
    pub fn second(&self) -> &VertexRef {
        &self.second
    }
}

/// Unit-edge all-pairs distances in stable vertex order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AllPairsDistances {
    pub(super) vertices: Vec<VertexRef>,
    pub(super) rows: Vec<Vec<Option<usize>>>,
    pub(super) index_by_vertex: HashMap<VertexRef, usize>,
}

impl AllPairsDistances {
    /// Return the row and column vertex order.
    #[must_use]
    pub fn vertices(&self) -> &[VertexRef] {
        &self.vertices
    }

    /// Return the immutable distance matrix. `None` means no connecting path.
    #[must_use]
    pub fn rows(&self) -> &[Vec<Option<usize>>] {
        &self.rows
    }

    /// Resolve one pair distance without exposing matrix indexes.
    pub fn distance(
        &self,
        from: &VertexRef,
        to: &VertexRef,
    ) -> Result<Option<usize>, UnknownGraphVertex> {
        let from_index = self.index(from)?;
        let to_index = self.index(to)?;
        Ok(self.rows[from_index][to_index])
    }

    fn index(&self, vertex: &VertexRef) -> Result<usize, UnknownGraphVertex> {
        self.index_by_vertex
            .get(vertex)
            .copied()
            .ok_or_else(|| UnknownGraphVertex {
                vertex: vertex.clone(),
            })
    }
}

/// One canonically ordered fundamental cycle.
///
/// `bonds()[i]` connects `vertices()[i]` to the next vertex, wrapping at the end.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundamentalCycle {
    pub(super) vertices: Vec<VertexRef>,
    pub(super) bonds: Vec<RecordId>,
}

impl FundamentalCycle {
    /// Return cyclically ordered vertices in canonical orientation.
    #[must_use]
    pub fn vertices(&self) -> &[VertexRef] {
        &self.vertices
    }

    /// Return bonds aligned with the canonical vertex cycle.
    #[must_use]
    pub fn bonds(&self) -> &[RecordId] {
        &self.bonds
    }
}

/// A private-`petgraph`, owned analysis view over one immutable molecule.
///
/// Construction follows the stable core order: atoms, groups, molecule-local text,
/// queries, then bonds in source order. Public results translate back to Ferrum types;
/// `NodeIndex` and petgraph edge handles never cross this boundary.
#[derive(Clone, Debug)]
pub struct MoleculeGraph {
    pub(super) graph: AnalysisGraph,
    pub(super) index_by_vertex: HashMap<VertexRef, NodeIndex>,
}
