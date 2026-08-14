//! ABI-4 FCM1 decoder, isolated from the legacy graph-operation coordinator.

use super::*;
use crate::adapter_contract::{
    FERRUM_CHEM_BOND_DIRECTION_BEGINDASH, FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE,
    FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT, FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT,
    FERRUM_CHEM_BOND_DIRECTION_NONE, FERRUM_CHEM_BOND_DIRECTION_OTHER, FERRUM_CHEM_BOND_STEREO_ANY,
    FERRUM_CHEM_BOND_STEREO_CIS, FERRUM_CHEM_BOND_STEREO_E, FERRUM_CHEM_BOND_STEREO_NONE,
    FERRUM_CHEM_BOND_STEREO_OTHER, FERRUM_CHEM_BOND_STEREO_TRANS, FERRUM_CHEM_BOND_STEREO_Z,
    FERRUM_CHEM_CHIRAL_OTHER, FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW,
    FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW, FERRUM_CHEM_CHIRAL_UNSPECIFIED,
    FERRUM_CHEM_MOLECULE_FLAGS_NONE, FERRUM_CHEM_MOLECULE_RESERVED,
    FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE, FERRUM_CHEM_MOLECULE_WIRE_VERSION,
};

pub(super) fn validate_input(smiles: &str) -> Result<(), ChemistryError> {
    if smiles.is_empty() {
        return invalid_input("must not be empty");
    }
    if smiles.as_bytes().contains(&0) {
        return invalid_input("must not contain NUL bytes");
    }
    if smiles.len() > FERRUM_CHEM_SMILES_MAX_BYTES {
        return invalid_input(&format!(
            "has {} bytes, above the {FERRUM_CHEM_SMILES_MAX_BYTES}-byte ABI limit",
            smiles.len()
        ));
    }
    Ok(())
}

fn invalid_input(reason: &str) -> Result<(), ChemistryError> {
    Err(ChemistryError::InvalidSmilesInput {
        reason: reason.to_owned(),
    })
}

pub(super) fn decode(response: &[u8]) -> Result<SmilesMolecule, ChemistryError> {
    if response.len() < MOLECULE_RESPONSE_HEADER_LENGTH {
        return Err(ChemistryError::TruncatedNativeResponse);
    }
    let mut reader = Reader::new(response);
    if reader.take(4).map_err(decode_error)? != MOLECULE_RESPONSE_MAGIC {
        return malformed("FCM1 response magic is invalid");
    }
    if reader.u32().map_err(decode_error)? != FERRUM_CHEM_MOLECULE_WIRE_VERSION {
        return malformed("FCM1 wire version is unsupported");
    }
    let status = reader.u32().map_err(decode_error)?;
    let detail_length =
        usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let smiles_length =
        usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let atom_count = usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let bond_count = usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    if reader.u32().map_err(decode_error)? != FERRUM_CHEM_MOLECULE_FLAGS_NONE {
        return malformed("FCM1 flags are nonzero");
    }
    if detail_length > FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES
        || smiles_length > FERRUM_CHEM_SMILES_MAX_BYTES
        || atom_count > FERRUM_CHEM_KEKULIZE_MAX_ATOMS as usize
        || bond_count > FERRUM_CHEM_KEKULIZE_MAX_BONDS as usize
    {
        return malformed("FCM1 declared size exceeds its ABI limit");
    }
    let detail = text(
        reader.take(detail_length).map_err(decode_error)?,
        "FCM1 detail",
    )?;
    let canonical_smiles = text(
        reader.take(smiles_length).map_err(decode_error)?,
        "FCM1 canonical SMILES",
    )?;
    if !matches!(
        status,
        FERRUM_CHEM_RESULT_OK
            | FERRUM_CHEM_RESULT_MALFORMED_REQUEST
            | FERRUM_CHEM_RESULT_INVALID_MOLECULE
            | FERRUM_CHEM_RESULT_DEPICTION_FAILURE
            | FERRUM_CHEM_RESULT_RESOURCE_LIMIT
            | FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE
            | FERRUM_CHEM_RESULT_INTERNAL_FAILURE
    ) {
        return malformed("FCM1 status is unsupported");
    }
    if status != FERRUM_CHEM_RESULT_OK {
        if detail.is_empty()
            || !canonical_smiles.is_empty()
            || atom_count != 0
            || bond_count != 0
            || !reader.is_empty()
        {
            return malformed("failed FCM1 response contains molecule data");
        }
        return Err(ChemistryError::NativeRejected {
            status,
            reason: detail.to_owned(),
        });
    }
    if !detail.is_empty() || canonical_smiles.is_empty() {
        return malformed("successful FCM1 response has invalid text");
    }
    let record_bytes = atom_count
        .checked_mul(FERRUM_CHEM_MOLECULE_ATOM_BYTES)
        .and_then(|n| {
            bond_count
                .checked_mul(FERRUM_CHEM_MOLECULE_BOND_BYTES)
                .and_then(|m| n.checked_add(m))
        })
        .and_then(|n| {
            atom_count
                .checked_mul(COORDINATE_BYTES)
                .and_then(|m| n.checked_add(m))
        })
        .ok_or_else(|| ChemistryError::MalformedNativeResponse {
            reason: "FCM1 record length overflows".to_owned(),
        })?;
    if response.len().saturating_sub(reader.cursor) != record_bytes {
        return malformed("FCM1 records are truncated or trailing");
    }
    let atoms = (0..atom_count)
        .map(|_| atom(&mut reader))
        .collect::<Result<Vec<_>, _>>()?;
    let bonds = (0..bond_count)
        .map(|_| bond(&mut reader))
        .collect::<Result<Vec<_>, _>>()?;
    let points = (0..atom_count)
        .map(|_| point(&mut reader))
        .collect::<Result<Vec<_>, _>>()?;
    let molecule =
        MolGraph::new(atoms, bonds, Some(Coordinates::new(points))).map_err(|error| {
            ChemistryError::MalformedNativeResponse {
                reason: error.to_string(),
            }
        })?;
    SmilesMolecule::new(canonical_smiles, molecule).map_err(|error| {
        ChemistryError::MalformedNativeResponse {
            reason: format!("FCM1 canonical molecule is invalid: {error}"),
        }
    })
}

fn malformed<T>(reason: &str) -> Result<T, ChemistryError> {
    Err(ChemistryError::MalformedNativeResponse {
        reason: reason.to_owned(),
    })
}
fn text<'a>(bytes: &'a [u8], field: &str) -> Result<&'a str, ChemistryError> {
    if bytes.contains(&0) {
        return malformed(&format!("{field} contains NUL"));
    }
    std::str::from_utf8(bytes).map_err(|_| ChemistryError::MalformedNativeResponse {
        reason: format!("{field} is not UTF-8"),
    })
}
fn atom(reader: &mut Reader<'_>) -> Result<MolAtom, ChemistryError> {
    let number = AtomicNumber::try_from(reader.u8().map_err(decode_error)?).map_err(|_| {
        ChemistryError::MalformedNativeResponse {
            reason: "FCM1 atomic number is invalid".to_owned(),
        }
    })?;
    let aromatic = bool_from_byte(reader.u8().map_err(decode_error)?, "FCM1 atom aromatic")
        .map_err(decode_error)?;
    let chirality = match u32::from(reader.u8().map_err(decode_error)?) {
        FERRUM_CHEM_CHIRAL_UNSPECIFIED => AtomChirality::Unspecified,
        FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW => AtomChirality::TetrahedralCw,
        FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW => AtomChirality::TetrahedralCcw,
        FERRUM_CHEM_CHIRAL_OTHER => AtomChirality::Other,
        _ => return malformed("FCM1 atom chirality is invalid"),
    };
    if reader.u8().map_err(decode_error)? != FERRUM_CHEM_MOLECULE_RESERVED as u8 {
        return malformed("FCM1 atom reserved bytes are nonzero");
    }
    let charge = reader.i32().map_err(decode_error)?;
    let isotope = reader.u16().map_err(decode_error)?;
    let hydrogens = reader.u16().map_err(decode_error)?;
    let radicals = reader.u8().map_err(decode_error)?;
    let no_implicit = bool_from_byte(reader.u8().map_err(decode_error)?, "FCM1 atom no_implicit")
        .map_err(decode_error)?;
    if reader.u16().map_err(decode_error)? != FERRUM_CHEM_MOLECULE_RESERVED as u16 {
        return malformed("FCM1 atom reserved bytes are nonzero");
    }
    let map = reader.u32().map_err(decode_error)?;
    MolAtom::from_native(
        number,
        charge,
        isotope,
        hydrogens,
        aromatic,
        chirality,
        radicals,
        no_implicit,
        map,
    )
    .map_err(|error| ChemistryError::MalformedNativeResponse {
        reason: error.to_string(),
    })
}
fn bond(reader: &mut Reader<'_>) -> Result<MolBond, ChemistryError> {
    let start = usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let end = usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let order = match u32::from(reader.u8().map_err(decode_error)?) {
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE => BondOrder::Single,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE => BondOrder::Double,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE => BondOrder::Triple,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC => BondOrder::Aromatic,
        FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE => BondOrder::Quadruple,
        _ => return malformed("FCM1 bond type is invalid"),
    };
    let aromatic = bool_from_byte(reader.u8().map_err(decode_error)?, "FCM1 bond aromatic")
        .map_err(decode_error)?;
    let stereo = match u32::from(reader.u8().map_err(decode_error)?) {
        FERRUM_CHEM_BOND_STEREO_NONE => BondStereo::None,
        FERRUM_CHEM_BOND_STEREO_ANY => BondStereo::Any,
        FERRUM_CHEM_BOND_STEREO_Z => BondStereo::Z,
        FERRUM_CHEM_BOND_STEREO_E => BondStereo::E,
        FERRUM_CHEM_BOND_STEREO_CIS => BondStereo::Cis,
        FERRUM_CHEM_BOND_STEREO_TRANS => BondStereo::Trans,
        FERRUM_CHEM_BOND_STEREO_OTHER => BondStereo::Other,
        _ => return malformed("FCM1 bond stereo is invalid"),
    };
    let direction = match u32::from(reader.u8().map_err(decode_error)?) {
        FERRUM_CHEM_BOND_DIRECTION_NONE => BondDirection::None,
        FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE => BondDirection::BeginWedge,
        FERRUM_CHEM_BOND_DIRECTION_BEGINDASH => BondDirection::BeginDash,
        FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT => BondDirection::EndUpRight,
        FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT => BondDirection::EndDownRight,
        FERRUM_CHEM_BOND_DIRECTION_OTHER => BondDirection::Other,
        _ => return malformed("FCM1 bond direction is invalid"),
    };
    let first = reader.u32().map_err(decode_error)?;
    let second = reader.u32().map_err(decode_error)?;
    if reader.u32().map_err(decode_error)? != FERRUM_CHEM_MOLECULE_RESERVED {
        return malformed("FCM1 bond reserved bytes are nonzero");
    }
    let refs = if first == FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE
        && second == FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE
    {
        None
    } else if first != FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE
        && second != FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE
    {
        Some((
            usize::try_from(first).expect("u32 fits usize"),
            usize::try_from(second).expect("u32 fits usize"),
        ))
    } else {
        return malformed("FCM1 stereo references are incomplete");
    };
    Ok(MolBond::from_native(
        start, end, order, aromatic, stereo, direction, refs,
    ))
}
fn point(reader: &mut Reader<'_>) -> Result<Point2, ChemistryError> {
    let x = f64::from_le_bytes(
        reader
            .take(8)
            .map_err(decode_error)?
            .try_into()
            .expect("fixed"),
    );
    let y = f64::from_le_bytes(
        reader
            .take(8)
            .map_err(decode_error)?
            .try_into()
            .expect("fixed"),
    );
    Point2::new(x, y).map_err(|_| ChemistryError::MalformedNativeResponse {
        reason: "FCM1 coordinate is non-finite".to_owned(),
    })
}
