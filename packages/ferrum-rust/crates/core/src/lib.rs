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

/// Atom records in a Ferrum molecule.
pub use atom::Atom;
/// Bond records and their carried chemical presentation facts.
pub use bond::{Bond, BondOrder, BondStyle};
/// Validation errors returned while constructing core records.
pub use error::ModelError;
/// Stable identifiers and their validated provenance.
pub use identity::{
    Identifier, InvalidIdentifier, LegacyFingerprint, RecordId, RecordKind, RecordOrigin,
};
/// Immutable molecule records containing validated vertices and bonds.
pub use molecule::Molecule;
/// Finite three-dimensional coordinates for an atom.
pub use position::Position;
/// Non-atom vertices and typed references accepted by bonds.
pub use vertex::{NonAtomVertex, VertexRef};

#[cfg(test)]
mod tests;
