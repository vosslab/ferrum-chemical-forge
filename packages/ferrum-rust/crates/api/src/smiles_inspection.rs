//! Explicit ABI-4 SMILES inspection for the provisional Ferrum CLI surface.

use std::path::Path;

use ferrum_chemistry::{AtomChirality, BondDirection, BondOrder, BondStereo, ChemistryError};
use serde::Serialize;
use thiserror::Error;

use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};

/// The single machine-readable schema emitted by `ferrum smiles inspect`.
pub const SMILES_INSPECTION_SCHEMA_V1: &str = "ferrum-smiles-inspection-v1";

/// Inspect SMILES using one caller-selected, regular ABI-4 adapter library.
///
/// This intentionally provisional, pre-M18 API accepts no adapter discovery:
/// callers supply an absolute, non-symbolic-link regular file. The operation owns
/// the loaded adapter for its duration and returns only owned Rust data.
pub fn inspect_smiles(
    adapter_path: &Path,
    smiles: &str,
) -> Result<SmilesInspectionV1, SmilesInspectionError> {
    let engine = load_explicit_adapter(adapter_path)?;
    let molecule = engine.smiles_to_molecule(smiles)?;
    let facts = molecule_inspection_facts(&molecule)?;

    Ok(SmilesInspectionV1 {
        schema: SMILES_INSPECTION_SCHEMA_V1,
        adapter_abi: ferrum_chemistry::ADAPTER_ABI_VERSION,
        canonical_smiles: facts.canonical_smiles,
        atoms: facts.atoms,
        bonds: facts.bonds,
        coordinates: facts.coordinates,
    })
}

pub(crate) fn molecule_inspection_facts(
    molecule: &ferrum_chemistry::SmilesMolecule,
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
pub(crate) struct MoleculeInspectionFactsV1 {
    pub(crate) canonical_smiles: String,
    pub(crate) atoms: Vec<SmilesAtomInspectionV1>,
    pub(crate) bonds: Vec<SmilesBondInspectionV1>,
    pub(crate) coordinates: Vec<SmilesPointInspectionV1>,
}

/// Immutable JSON payload for the provisional ABI-4 inspection operation.
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

/// One atom in native adapter order.
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

/// One bond in native adapter order.
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

/// One finite atom coordinate in native atom order.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SmilesPointInspectionV1 {
    x: f64,
    y: f64,
}

/// A rejected adapter path or chemistry boundary result for SMILES inspection.
#[derive(Debug, Error)]
pub enum SmilesInspectionError {
    /// The caller-selected adapter path cannot be loaded safely.
    #[error(transparent)]
    Adapter(#[from] ExplicitAdapterError),
    /// The ABI-4 adapter returned a molecule without complete coordinates.
    #[error("ABI-4 adapter returned a SMILES molecule without atom-aligned coordinates")]
    MissingCoordinates,
    /// The adapter could not load, parse SMILES, or satisfy its ABI boundary.
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
    use std::path::Path;

    use super::{
        SMILES_INSPECTION_SCHEMA_V1, atom_chirality_name, bond_direction_name, bond_order_name,
        bond_stereo_name,
    };
    use crate::explicit_adapter::{ExplicitAdapterError, load_explicit_adapter};
    use ferrum_chemistry::{AtomChirality, BondDirection, BondOrder, BondStereo};

    #[test]
    fn schema_and_closed_enum_names_are_exact() {
        assert_eq!(SMILES_INSPECTION_SCHEMA_V1, "ferrum-smiles-inspection-v1");
        assert_eq!(
            atom_chirality_name(AtomChirality::TetrahedralCw),
            "tetrahedral_cw"
        );
        assert_eq!(bond_order_name(BondOrder::Quadruple), "quadruple");
        assert_eq!(bond_stereo_name(BondStereo::Trans), "trans");
        assert_eq!(
            bond_direction_name(BondDirection::EndDownRight),
            "end_down_right"
        );
    }

    #[test]
    fn relative_adapter_path_is_rejected_before_native_loading() {
        let Err(error) = load_explicit_adapter(Path::new("libferrum_chem.dylib")) else {
            panic!("relative adapter paths are not an inspection route");
        };

        assert!(matches!(error, ExplicitAdapterError::RelativePath { .. }));
    }

    #[test]
    fn missing_absolute_adapter_path_is_rejected_before_native_loading() {
        let Err(error) =
            load_explicit_adapter(Path::new("/definitely/not/ferrum/libferrum_chem.dylib"))
        else {
            panic!("missing adapter is not a native loading route");
        };

        assert!(matches!(error, ExplicitAdapterError::Metadata { .. }));
    }

    #[cfg(unix)]
    #[test]
    fn symbolic_link_adapter_path_is_rejected_before_native_loading() {
        let directory =
            std::env::temp_dir().join(format!("ferrum-smiles-inspection-{}", std::process::id()));
        let target = directory.join("adapter.dylib");
        let link = directory.join("adapter-link.dylib");
        std::fs::create_dir_all(&directory).expect("create temporary directory");
        std::fs::write(&target, "not a native library").expect("create adapter target");
        std::os::unix::fs::symlink(&target, &link).expect("create adapter link");

        let Err(error) = load_explicit_adapter(&link) else {
            panic!("symbolic links are unsafe adapter paths");
        };

        assert!(matches!(error, ExplicitAdapterError::UnsafePath { .. }));
        std::fs::remove_file(&link).expect("remove adapter link");
        std::fs::remove_file(&target).expect("remove adapter target");
        std::fs::remove_dir(&directory).expect("remove temporary directory");
    }
}
