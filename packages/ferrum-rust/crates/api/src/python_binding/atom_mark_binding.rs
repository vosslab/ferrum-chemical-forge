//! Closed Python atom-mark values for Rust document operations.

use ferrum_document::{AtomMarkActionV1, AtomMarkKindV1};
use pyo3::prelude::*;

/// Exact add/remove intent for one authored atom mark.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "AtomMarkActionV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyAtomMarkActionV1 {
    Add,
    Remove,
}

impl From<PyAtomMarkActionV1> for AtomMarkActionV1 {
    fn from(value: PyAtomMarkActionV1) -> Self {
        match value {
            PyAtomMarkActionV1::Add => Self::Add,
            PyAtomMarkActionV1::Remove => Self::Remove,
        }
    }
}

/// Closed vocabulary of atom marks that Ferrum can persist and render.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "AtomMarkKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyAtomMarkKindV1 {
    Plus,
    Minus,
    Radical,
    Biradical,
    Electronpair,
    DottedElectronpair,
    PzOrbital,
}

impl From<PyAtomMarkKindV1> for AtomMarkKindV1 {
    fn from(value: PyAtomMarkKindV1) -> Self {
        match value {
            PyAtomMarkKindV1::Plus => Self::Plus,
            PyAtomMarkKindV1::Minus => Self::Minus,
            PyAtomMarkKindV1::Radical => Self::Radical,
            PyAtomMarkKindV1::Biradical => Self::Biradical,
            PyAtomMarkKindV1::Electronpair => Self::Electronpair,
            PyAtomMarkKindV1::DottedElectronpair => Self::DottedElectronpair,
            PyAtomMarkKindV1::PzOrbital => Self::PzOrbital,
        }
    }
}
