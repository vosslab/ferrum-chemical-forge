//! Stable graph analysis over immutable Ferrum molecules.
//!
//! This routing facade publishes query values while implementation modules retain
//! the private `petgraph` representation and deterministic algorithms.

mod analysis;
mod cycles;
mod model;

/// Graph-query values that preserve Ferrum's stable record order.
pub use model::{
    AllPairsDistances, ConnectedComponent, FundamentalCycle, MatchingPair, MoleculeGraph,
    UnknownGraphVertex, VertexDistance,
};

#[cfg(test)]
mod tests;
