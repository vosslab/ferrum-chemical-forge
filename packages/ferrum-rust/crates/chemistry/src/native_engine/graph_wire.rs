//! Complete ABI-4 molecular graph request encoding for chemistry codecs.

use super::*;
use crate::adapter_contract::{
    FERRUM_CHEM_BOND_DIRECTION_BEGINDASH, FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE,
    FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT, FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT,
    FERRUM_CHEM_BOND_DIRECTION_NONE, FERRUM_CHEM_BOND_STEREO_ANY, FERRUM_CHEM_BOND_STEREO_CIS,
    FERRUM_CHEM_BOND_STEREO_E, FERRUM_CHEM_BOND_STEREO_NONE, FERRUM_CHEM_BOND_STEREO_TRANS,
    FERRUM_CHEM_BOND_STEREO_Z, FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW,
    FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW, FERRUM_CHEM_CHIRAL_UNSPECIFIED,
    FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE,
};

const MAGIC: [u8; 4] = *b"FCG1";

pub(super) fn encode(molecule: &MolGraph) -> Result<Vec<u8>, ChemistryError> {
    let atom_count = checked_count(
        molecule.atoms().len(),
        FERRUM_CHEM_KEKULIZE_MAX_ATOMS,
        "atom count",
    )?;
    let bond_count = checked_count(
        molecule.bonds().len(),
        FERRUM_CHEM_KEKULIZE_MAX_BONDS,
        "bond count",
    )?;
    let capacity = FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES
        .checked_add(
            usize::try_from(atom_count).expect("u32 fits usize") * FERRUM_CHEM_GRAPH_ATOM_BYTES,
        )
        .and_then(|length| {
            length.checked_add(
                usize::try_from(bond_count).expect("u32 fits usize") * FERRUM_CHEM_GRAPH_BOND_BYTES,
            )
        })
        .ok_or_else(|| ChemistryError::UnsupportedNativeRequest {
            reason: "complete graph request length overflows this platform".to_owned(),
        })?;

    let mut output = Vec::with_capacity(capacity);
    output.extend_from_slice(&MAGIC);
    put_u32(&mut output, FERRUM_CHEM_GRAPH_WIRE_VERSION);
    put_u32(&mut output, atom_count);
    put_u32(&mut output, bond_count);
    put_u32(&mut output, FERRUM_CHEM_GRAPH_FLAGS_NONE);
    for atom in molecule.atoms() {
        encode_atom(&mut output, atom)?;
    }
    for bond in molecule.bonds() {
        encode_bond(&mut output, bond)?;
    }
    debug_assert_eq!(output.len(), capacity);
    Ok(output)
}

fn encode_atom(output: &mut Vec<u8>, atom: &MolAtom) -> Result<(), ChemistryError> {
    if atom
        .atom_map_number()
        .is_some_and(|number| number > i32::MAX as u32)
    {
        return unsupported("atom-map number above the native signed range");
    }
    output.push(atom.atomic_number().get());
    output.push(u8::from(atom.is_aromatic()));
    output.push(wire_u8(match atom.chirality() {
        AtomChirality::Unspecified => FERRUM_CHEM_CHIRAL_UNSPECIFIED,
        AtomChirality::TetrahedralCw => FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW,
        AtomChirality::TetrahedralCcw => FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW,
        AtomChirality::Other => return unsupported("atom chirality class other"),
    }));
    output.push(0);

    let mut presence = 0_u32;
    if atom.formal_charge().is_some() {
        presence |= FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE;
    }
    if atom.isotope().is_some() {
        presence |= FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE;
    }
    if atom.explicit_hydrogens().is_some() {
        presence |= FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS;
    }
    put_u32(output, presence);
    put_i32(output, atom.formal_charge().unwrap_or(0));
    put_u16(output, atom.isotope().unwrap_or(0));
    put_u16(output, atom.explicit_hydrogens().unwrap_or(0));
    output.push(atom.radical_electrons());
    output.push(u8::from(atom.no_implicit()));
    put_u16(output, 0);
    put_u32(output, atom.atom_map_number().unwrap_or(0));
    Ok(())
}

fn encode_bond(output: &mut Vec<u8>, bond: &MolBond) -> Result<(), ChemistryError> {
    put_u32(output, index(bond.start(), "bond start")?);
    put_u32(output, index(bond.end(), "bond end")?);
    output.push(wire_u8(match bond.order() {
        BondOrder::Single => FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE,
        BondOrder::Double => FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE,
        BondOrder::Triple => FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE,
        BondOrder::Aromatic => FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC,
        BondOrder::Quadruple => FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE,
    }));
    output.push(u8::from(bond.is_aromatic()));
    output.push(wire_u8(match bond.stereo() {
        BondStereo::None => FERRUM_CHEM_BOND_STEREO_NONE,
        BondStereo::Any => FERRUM_CHEM_BOND_STEREO_ANY,
        BondStereo::Z => FERRUM_CHEM_BOND_STEREO_Z,
        BondStereo::E => FERRUM_CHEM_BOND_STEREO_E,
        BondStereo::Cis => FERRUM_CHEM_BOND_STEREO_CIS,
        BondStereo::Trans => FERRUM_CHEM_BOND_STEREO_TRANS,
        BondStereo::Other => return unsupported("bond stereo class other"),
    }));
    output.push(wire_u8(match bond.direction() {
        BondDirection::None => FERRUM_CHEM_BOND_DIRECTION_NONE,
        BondDirection::BeginWedge => FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE,
        BondDirection::BeginDash => FERRUM_CHEM_BOND_DIRECTION_BEGINDASH,
        BondDirection::EndUpRight => FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT,
        BondDirection::EndDownRight => FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT,
        BondDirection::Other => return unsupported("bond direction class other"),
    }));
    let (first, second) = match bond.stereo_atoms() {
        Some((first, second)) => (
            index(first, "first stereo reference")?,
            index(second, "second stereo reference")?,
        ),
        None => (
            FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE,
            FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE,
        ),
    };
    put_u32(output, first);
    put_u32(output, second);
    put_u32(output, 0);
    Ok(())
}

fn index(value: usize, field: &str) -> Result<u32, ChemistryError> {
    u32::try_from(value).map_err(|_| ChemistryError::UnsupportedNativeRequest {
        reason: format!("{field} does not fit the complete graph wire"),
    })
}

fn wire_u8(value: u32) -> u8 {
    u8::try_from(value).expect("generated graph enum constant fits u8")
}

fn unsupported<T>(fact: &str) -> Result<T, ChemistryError> {
    Err(ChemistryError::UnsupportedNativeRequest {
        reason: format!("{fact} cannot be reconstructed exactly for codec export"),
    })
}
