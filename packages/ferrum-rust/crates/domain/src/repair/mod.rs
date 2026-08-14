//! Pure, immutable coordinate-repair planning.
//!
//! This module accepts a caller-selected depiction graph and returns a sparse
//! coordinate patch. It neither mutates a molecule nor infers chemistry from
//! a drawing. A caller owns atomically applying the returned patch at its
//! persistence boundary after validating its coordinate preconditions.

mod plan;
mod types;

pub use plan::{plan_repair, plan_repair_with_outcome};
pub use types::{
    CoordinatePatch, CoordinateReplacement, DepictionBond, DepictionGraph, DepictionVertex,
    PatchPreconditionError, RepairError, RepairKind, RepairOutcome, RepairRequest,
};

#[cfg(test)]
mod tests;
