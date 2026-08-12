//! Higher-level chemistry-domain utilities.
//!
//! This crate owns pure domain plans.  It accepts explicit, already-selected
//! chemistry facts and never guesses a ring, carbohydrate numbering, or
//! stereochemistry from a drawing format.

pub mod catalog;
pub mod haworth;
pub mod peptide;
pub mod repair;
pub mod sugar;

pub use peptide::{
    PeptideSequence, PeptideSyntaxError, PeptideTerminus, ProtonationIntent, ResidueCode,
    TerminusIntent, parse_one_letter_sequence,
};
pub use repair::{
    CoordinatePatch, CoordinateReplacement, DepictionBond, DepictionGraph, DepictionVertex,
    PatchPreconditionError, RepairError, RepairKind, RepairRequest, plan_repair,
};
