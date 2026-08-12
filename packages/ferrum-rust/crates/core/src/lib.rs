//! Shared, chemistry-independent Ferrum molecule types.
//!
//! This facade preserves the public core-model API while focused modules own
//! validated record identity, spatial values, vertices, bonds, and molecules.
//! Serde is internal persistence and testing support, not a public wire ABI.

mod atom;
mod bond;
mod error;
mod formatting;
pub mod graph;
mod identity;
mod molecule;
mod position;
mod vertex;

pub use atom::Atom;
pub use bond::{Bond, BondOrder, BondStyle};
pub use error::ModelError;
pub use identity::{
    Identifier, InvalidIdentifier, LegacyFingerprint, RecordId, RecordKind, RecordOrigin,
};
pub use molecule::Molecule;
pub use position::Position;
pub use vertex::{NonAtomVertex, VertexRef};

#[cfg(test)]
mod tests;
