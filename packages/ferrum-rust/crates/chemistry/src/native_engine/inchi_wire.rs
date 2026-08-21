//! Closed ABI-4 InChI request encoding and input validation.

use crate::{
    ChemistryError, InchiMode, MolGraph, FERRUM_CHEM_INCHI_FLAGS_NONE, FERRUM_CHEM_INCHI_KEY_BYTES,
    FERRUM_CHEM_INCHI_MAX_BYTES, FERRUM_CHEM_INCHI_MODE_FIXED_HYDROGEN,
    FERRUM_CHEM_INCHI_MODE_STANDARD, FERRUM_CHEM_INCHI_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_INCHI_WIRE_VERSION,
};

use super::{graph_wire, put_u32};

const MAGIC: [u8; 4] = *b"FCI1";

pub(crate) fn validate_input(inchi: &str) -> Result<(), ChemistryError> {
    if inchi.is_empty() {
        return invalid("input must not be empty");
    }
    if inchi.len() > FERRUM_CHEM_INCHI_MAX_BYTES {
        return invalid("input exceeds the ABI byte limit");
    }
    if !inchi.is_ascii() {
        return invalid("input must be ASCII");
    }
    if inchi
        .bytes()
        .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return invalid("input must be one line without control or whitespace bytes");
    }
    if !inchi.starts_with("InChI=1S/") && !inchi.starts_with("InChI=1/") {
        return invalid("input must begin with InChI=1S/ or InChI=1/");
    }
    Ok(())
}

pub(crate) fn validate_key(inchi: &str, key: &str) -> Result<(), ChemistryError> {
    let bytes = key.as_bytes();
    if bytes.len() != FERRUM_CHEM_INCHI_KEY_BYTES || bytes[14] != b'-' || bytes[25] != b'-' {
        return malformed_key("InChIKey has an invalid length or separator layout");
    }
    if bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| index != 14 && index != 25 && !byte.is_ascii_uppercase())
    {
        return malformed_key("InChIKey contains a non-uppercase hash character");
    }
    let expected_kind = if inchi.starts_with("InChI=1S/") {
        b'S'
    } else {
        b'N'
    };
    if bytes[23] != expected_kind || bytes[24] != b'A' {
        return malformed_key("InChIKey standardness or version marker is invalid");
    }
    Ok(())
}

pub(crate) fn encode(molecule: &MolGraph, mode: InchiMode) -> Result<Vec<u8>, ChemistryError> {
    let graph = graph_wire::encode(molecule)?;
    let graph_length =
        u32::try_from(graph.len()).map_err(|_| ChemistryError::UnsupportedNativeRequest {
            reason: "complete graph is too large for the InChI request envelope".to_owned(),
        })?;
    let mode = match mode {
        InchiMode::Standard => FERRUM_CHEM_INCHI_MODE_STANDARD,
        InchiMode::FixedHydrogen => FERRUM_CHEM_INCHI_MODE_FIXED_HYDROGEN,
    };
    let mut request = Vec::with_capacity(FERRUM_CHEM_INCHI_REQUEST_HEADER_BYTES + graph.len());
    request.extend_from_slice(&MAGIC);
    put_u32(&mut request, FERRUM_CHEM_INCHI_WIRE_VERSION);
    put_u32(&mut request, mode);
    put_u32(&mut request, graph_length);
    put_u32(&mut request, FERRUM_CHEM_INCHI_FLAGS_NONE);
    request.extend_from_slice(&graph);
    Ok(request)
}

fn invalid<T>(reason: &str) -> Result<T, ChemistryError> {
    Err(ChemistryError::InvalidInchiInput {
        reason: reason.to_owned(),
    })
}

fn malformed_key<T>(reason: &str) -> Result<T, ChemistryError> {
    Err(ChemistryError::MalformedNativeResponse {
        reason: reason.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_validation_accepts_only_one_bounded_inchi_line() {
        for accepted in ["InChI=1S/CH4/h1H4", "InChI=1/CH4/h1H4"] {
            validate_input(accepted).expect("valid InChI prefix is accepted");
        }
        for rejected in ["", "CH4", "InChI=1S/CH4\n", "InChI=1S/CH 4"] {
            assert!(matches!(
                validate_input(rejected),
                Err(ChemistryError::InvalidInchiInput { .. })
            ));
        }
    }

    #[test]
    fn key_validation_enforces_the_official_layout_and_input_kind() {
        validate_key(
            "InChI=1S/C9H12/c1-2-6-9-7-4-3-5-8-9/h3-5,7-8H,2,6H2,1H3",
            "ODLMAHJVESYWTB-UHFFFAOYSA-N",
        )
        .expect("official standard InChIKey layout is accepted");
        validate_key("InChI=1/CH4/h1H4/f/h1H4", "VNWKTOKETHGBQD-DYIVTJBTNA-N")
            .expect("official non-standard marker is accepted");

        for rejected in [
            "ODLMAHJVESYWTB-UHFFFAOYSA",
            "ODLMAHJVESYWTB_UHFFFAOYSA_N",
            "odlmahjvesywtb-UHFFFAOYSA-N",
            "ODLMAHJVESYWTB-UHFFFAOYNA-N",
        ] {
            assert!(matches!(
                validate_key("InChI=1S/CH4/h1H4", rejected),
                Err(ChemistryError::MalformedNativeResponse { .. })
            ));
        }
    }
}
