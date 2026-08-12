//! Stable, owned chemistry values at Ferrum's engine boundary.
//!
//! This crate intentionally knows neither CDML nor a native toolkit.  An engine
//! receives a validated [`MolGraph`] and returns a new one, keeping callers free
//! of foreign handles, borrowed buffers, and toolkit-specific representations.

mod engine;
mod model;
mod native_engine;

pub use crate::engine::{
    ChemEngine, ChemistryError, KekulizeOptions, KekulizeOptionsError, UnavailableChemEngine,
};
pub use crate::model::{
    AtomicNumber, BondOrder, Coordinates, MolAtom, MolBond, MolGraph, MolGraphError, Point2,
};
pub use crate::native_engine::NativeChemEngine;

include!(concat!(env!("OUT_DIR"), "/adapter_wire_constants.rs"));
