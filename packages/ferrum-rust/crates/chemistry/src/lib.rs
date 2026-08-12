//! Stable, owned chemistry values at Ferrum's engine boundary.
//!
//! This crate intentionally knows neither CDML nor a native toolkit.  An engine
//! receives a validated [`MolGraph`] and returns a new one, keeping callers free
//! of foreign handles, borrowed buffers, and toolkit-specific representations.

mod adapter_contract;
mod engine;
mod model;
mod native_engine;

pub use crate::engine::{
    ChemEngine, ChemistryError, KekulizeOptions, KekulizeOptionsError, UnavailableChemEngine,
};
pub use crate::model::{
    AtomicNumber, BondOrder, Coordinates, MolAtom, MolBond, MolGraph, MolGraphError, Point2,
    SmilesDepiction,
};
pub use crate::native_engine::NativeChemEngine;

pub use crate::adapter_contract::ADAPTER_ABI_VERSION;
pub(crate) use crate::adapter_contract::{
    FERRUM_CHEM_COORDINATE_BYTES, FERRUM_CHEM_KEKULIZE_ATOM_BYTES, FERRUM_CHEM_KEKULIZE_BOND_BYTES,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC, FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE, FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE, FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED,
    FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS, FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE,
    FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE, FERRUM_CHEM_KEKULIZE_MAX_ATOMS,
    FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS, FERRUM_CHEM_KEKULIZE_MAX_BONDS,
    FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES, FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL,
    FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS, FERRUM_CHEM_KEKULIZE_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_KEKULIZE_RESPONSE_HEADER_BYTES, FERRUM_CHEM_KEKULIZE_WIRE_VERSION,
    FERRUM_CHEM_RESULT_INTERNAL_FAILURE, FERRUM_CHEM_RESULT_INVALID_MOLECULE,
    FERRUM_CHEM_RESULT_KEKULIZE_FAILURE, FERRUM_CHEM_RESULT_MALFORMED_REQUEST,
    FERRUM_CHEM_RESULT_OK, FERRUM_CHEM_SMILES_MAX_BYTES, FERRUM_CHEM_SMILES_RESPONSE_HEADER_BYTES,
};
