//! Explicit ABI-4 SMILES inspection.

use std::path::Path;

use serde::Serialize;
use thiserror::Error;

use crate::{
    AtomChirality, BondDirection, BondOrder, BondStereo, ChemistryError, ExplicitAdapterError,
    SmilesMolecule, load_explicit_adapter,
};

/// The machine-readable schema emitted by SMILES inspection.
pub const SMILES_INSPECTION_SCHEMA_V1: &str = "ferrum-smiles-inspection-v1";

/// Inspect SMILES using one caller-selected, regular ABI-4 adapter library.
pub fn inspect_smiles(
    adapter_path: &Path,
    smiles: &str,
) -> Result<SmilesInspectionV1, SmilesInspectionError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    let facts = molecule_inspection_facts(&molecule)?;
    Ok(SmilesInspectionV1 {
        schema: SMILES_INSPECTION_SCHEMA_V1,
        adapter_abi: crate::ADAPTER_ABI_VERSION,
        canonical_smiles: facts.canonical_smiles,
        atoms: facts.atoms,
        bonds: facts.bonds,
        coordinates: facts.coordinates,
    })
}

/// Extract owned atom-aligned inspection facts from one imported molecule.
pub fn molecule_inspection_facts(
    molecule: &SmilesMolecule,
) -> Result<MoleculeInspectionFactsV1, SmilesInspectionError> {
    let graph = molecule.molecule();
    let coordinates = graph
        .coordinates()
        .ok_or(SmilesInspectionError::MissingCoordinates)?;
    Ok(MoleculeInspectionFactsV1 {
        canonical_smiles: molecule.canonical_smiles().to_owned(),
        atoms: graph
            .atoms()
            .iter()
            .map(|atom| SmilesAtomInspectionV1 {
                atomic_number: atom.atomic_number().get(),
                formal_charge: atom.formal_charge(),
                isotope: atom.isotope(),
                explicit_hydrogens: atom.explicit_hydrogens(),
                aromatic: atom.is_aromatic(),
                chirality: atom_chirality_name(atom.chirality()),
                radical_electrons: atom.radical_electrons(),
                no_implicit: atom.no_implicit(),
                atom_map_number: atom.atom_map_number(),
            })
            .collect(),
        bonds: graph
            .bonds()
            .iter()
            .map(|bond| SmilesBondInspectionV1 {
                start: bond.start(),
                end: bond.end(),
                order: bond_order_name(bond.order()),
                aromatic: bond.is_aromatic(),
                stereo: bond_stereo_name(bond.stereo()),
                direction: bond_direction_name(bond.direction()),
                stereo_atoms: bond.stereo_atoms(),
            })
            .collect(),
        coordinates: coordinates
            .points()
            .iter()
            .map(|point| SmilesPointInspectionV1 {
                x: point.x(),
                y: point.y(),
            })
            .collect(),
    })
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoleculeInspectionFactsV1 {
    pub canonical_smiles: String,
    pub atoms: Vec<SmilesAtomInspectionV1>,
    pub bonds: Vec<SmilesBondInspectionV1>,
    pub coordinates: Vec<SmilesPointInspectionV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SmilesInspectionV1 {
    schema: &'static str,
    adapter_abi: u32,
    canonical_smiles: String,
    atoms: Vec<SmilesAtomInspectionV1>,
    bonds: Vec<SmilesBondInspectionV1>,
    coordinates: Vec<SmilesPointInspectionV1>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SmilesAtomInspectionV1 {
    atomic_number: u8,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
    aromatic: bool,
    chirality: &'static str,
    radical_electrons: u8,
    no_implicit: bool,
    atom_map_number: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SmilesBondInspectionV1 {
    start: usize,
    end: usize,
    order: &'static str,
    aromatic: bool,
    stereo: &'static str,
    direction: &'static str,
    stereo_atoms: Option<(usize, usize)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SmilesPointInspectionV1 {
    x: f64,
    y: f64,
}
#[derive(Debug, Error)]
pub enum SmilesInspectionError {
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    #[error("ABI-4 adapter returned a SMILES molecule without atom-aligned coordinates")]
    MissingCoordinates,
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
}
fn atom_chirality_name(value: AtomChirality) -> &'static str {
    match value {
        AtomChirality::Unspecified => "unspecified",
        AtomChirality::TetrahedralCw => "tetrahedral_cw",
        AtomChirality::TetrahedralCcw => "tetrahedral_ccw",
        AtomChirality::Other => "other",
    }
}
fn bond_order_name(value: BondOrder) -> &'static str {
    match value {
        BondOrder::Aromatic => "aromatic",
        BondOrder::Single => "single",
        BondOrder::Double => "double",
        BondOrder::Triple => "triple",
        BondOrder::Quadruple => "quadruple",
    }
}
fn bond_stereo_name(value: BondStereo) -> &'static str {
    match value {
        BondStereo::None => "none",
        BondStereo::Any => "any",
        BondStereo::Z => "z",
        BondStereo::E => "e",
        BondStereo::Cis => "cis",
        BondStereo::Trans => "trans",
        BondStereo::Other => "other",
    }
}
fn bond_direction_name(value: BondDirection) -> &'static str {
    match value {
        BondDirection::None => "none",
        BondDirection::BeginWedge => "begin_wedge",
        BondDirection::BeginDash => "begin_dash",
        BondDirection::EndUpRight => "end_up_right",
        BondDirection::EndDownRight => "end_down_right",
        BondDirection::Other => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    #[test]
    fn relative_adapter_path_is_rejected_before_native_loading() {
        assert!(matches!(
            crate::load_explicit_adapter(Path::new("libferrum_chem.dylib")),
            Err(ExplicitAdapterError::RelativePath { .. })
        ));
    }
    #[test]
    fn schema_and_closed_enum_names_are_exact() {
        assert_eq!(SMILES_INSPECTION_SCHEMA_V1, "ferrum-smiles-inspection-v1");
        assert_eq!(bond_order_name(BondOrder::Quadruple), "quadruple");
    }
}
