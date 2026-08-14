//! Coordinate-bearing complete-graph request for explicit molblock export.

use super::*;

const MAGIC: [u8; 4] = *b"FCB1";
const TITLED_MAGIC: [u8; 4] = *b"FBT1";

pub(crate) fn validate_title(title: &str) -> Result<(), ChemistryError> {
    if title.contains(['\0', '\r', '\n']) {
        return Err(ChemistryError::CodecFailed {
            codec: "molblock",
            reason: "molblock title cannot contain NUL or line-break characters".to_owned(),
        });
    }
    u32::try_from(title.len()).map_err(|_| ChemistryError::UnsupportedNativeRequest {
        reason: "molblock title length does not fit the native wire".to_owned(),
    })?;
    Ok(())
}

pub(super) fn validate_output_title(output: &str, expected: &str) -> Result<(), ChemistryError> {
    if output.split_once('\n').map(|(title, _)| title) != Some(expected) {
        return Err(ChemistryError::MalformedNativeResponse {
            reason: "molblock response title does not match the requested title".to_owned(),
        });
    }
    Ok(())
}

pub(super) fn encode_titled(
    molecule: &MolGraph,
    version: MolblockVersion,
    title: &str,
) -> Result<Vec<u8>, ChemistryError> {
    validate_title(title)?;
    let molecule_request = encode(molecule, version)?;
    let molecule_length = u32::try_from(molecule_request.len()).map_err(|_| {
        ChemistryError::UnsupportedNativeRequest {
            reason: "molblock request length does not fit the titled native wire".to_owned(),
        }
    })?;
    let title_length =
        u32::try_from(title.len()).map_err(|_| ChemistryError::UnsupportedNativeRequest {
            reason: "molblock title length does not fit the native wire".to_owned(),
        })?;
    let capacity = FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES
        .checked_add(molecule_request.len())
        .and_then(|length| length.checked_add(title.len()))
        .ok_or_else(|| ChemistryError::UnsupportedNativeRequest {
            reason: "titled molblock request length overflows this platform".to_owned(),
        })?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| ChemistryError::ResourceExhausted {
            operation: "molecule_to_molblock_with_title",
        })?;
    output.extend_from_slice(&TITLED_MAGIC);
    put_u32(&mut output, FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION);
    put_u32(&mut output, molecule_length);
    put_u32(&mut output, title_length);
    output.extend_from_slice(&molecule_request);
    output.extend_from_slice(title.as_bytes());
    debug_assert_eq!(output.len(), capacity);
    Ok(output)
}

pub(super) fn encode(
    molecule: &MolGraph,
    version: MolblockVersion,
) -> Result<Vec<u8>, ChemistryError> {
    let coordinates = molecule
        .coordinates()
        .ok_or_else(|| ChemistryError::CodecFailed {
            codec: "molblock",
            reason: "molblock export requires one coordinate for every atom".to_owned(),
        })?;
    let graph = graph_wire::encode(molecule)?;
    let records = graph
        .get(FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES..)
        .expect("FCG1 encoder always emits its complete header");
    let coordinate_bytes = coordinates
        .points()
        .len()
        .checked_mul(FERRUM_CHEM_COORDINATE_BYTES)
        .ok_or_else(|| ChemistryError::UnsupportedNativeRequest {
            reason: "molblock coordinate byte length overflows this platform".to_owned(),
        })?;
    let capacity = FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES
        .checked_add(records.len())
        .and_then(|length| length.checked_add(coordinate_bytes))
        .ok_or_else(|| ChemistryError::UnsupportedNativeRequest {
            reason: "molblock request length overflows this platform".to_owned(),
        })?;

    let mut output = Vec::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| ChemistryError::ResourceExhausted {
            operation: "molecule_to_molblock",
        })?;
    output.extend_from_slice(&MAGIC);
    put_u32(&mut output, FERRUM_CHEM_MOLBLOCK_WIRE_VERSION);
    put_u32(
        &mut output,
        match version {
            MolblockVersion::V2000 => FERRUM_CHEM_MOLBLOCK_FORMAT_V2000,
            MolblockVersion::V3000 => FERRUM_CHEM_MOLBLOCK_FORMAT_V3000,
        },
    );
    put_u32(
        &mut output,
        u32::try_from(molecule.atoms().len()).map_err(|_| {
            ChemistryError::UnsupportedNativeRequest {
                reason: "molblock atom count does not fit the native wire".to_owned(),
            }
        })?,
    );
    put_u32(
        &mut output,
        u32::try_from(molecule.bonds().len()).map_err(|_| {
            ChemistryError::UnsupportedNativeRequest {
                reason: "molblock bond count does not fit the native wire".to_owned(),
            }
        })?,
    );
    put_u32(&mut output, FERRUM_CHEM_MOLBLOCK_FLAGS_NONE);
    output.extend_from_slice(records);
    for point in coordinates.points() {
        output.extend_from_slice(&point.x().to_le_bytes());
        output.extend_from_slice(&point.y().to_le_bytes());
    }
    debug_assert_eq!(output.len(), capacity);
    Ok(output)
}
