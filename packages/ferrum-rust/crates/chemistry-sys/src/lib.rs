//! Safe ownership boundary for the dynamically loaded Ferrum chemistry adapter.
//!
//! The public API loads an explicitly selected ABI-4 adapter and returns owned
//! response bytes.  The crate does not interpret chemistry payloads or create
//! a link-time dependency on RDKit.

mod buffer;
mod contract;
mod loader;

pub use contract::{
    AdapterError, FERRUM_CHEM_ADAPTER_ABI_VERSION, FERRUM_CHEM_ALL_KNOWN_CAPABILITIES,
    FERRUM_CHEM_CAPABILITY_GENERATE_2D, FERRUM_CHEM_CAPABILITY_INCHI,
    FERRUM_CHEM_CAPABILITY_KEKULIZE, FERRUM_CHEM_CAPABILITY_MOLFILE,
    FERRUM_CHEM_CAPABILITY_MOLFILE_READ, FERRUM_CHEM_CAPABILITY_SDF_READ,
    FERRUM_CHEM_CAPABILITY_SDF_WRITE, FERRUM_CHEM_CAPABILITY_SMARTS,
    FERRUM_CHEM_CAPABILITY_SMILES_MOLECULE, FERRUM_CHEM_MAX_RESPONSE_BYTES,
};
pub use loader::ChemistryAdapter;

#[cfg(test)]
mod buffer_tests;
