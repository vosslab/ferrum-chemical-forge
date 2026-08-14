//! Ordered FSD1 request encoding for native RDKit SDWriter export.

use super::*;

const MAGIC: [u8; 4] = *b"FSD1";

pub(super) fn encode(
    records: &[SdfRecord],
    version: MolblockVersion,
) -> Result<Vec<u8>, ChemistryError> {
    if records.is_empty() {
        return Err(ChemistryError::CodecFailed {
            codec: "SDF",
            reason: "SDF export requires at least one record".to_owned(),
        });
    }
    let record_count =
        u32::try_from(records.len()).map_err(|_| ChemistryError::UnsupportedNativeRequest {
            reason: "SDF record count does not fit the native wire".to_owned(),
        })?;
    let mut output = Vec::with_capacity(FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES);
    output.extend_from_slice(&MAGIC);
    put_u32(&mut output, FERRUM_CHEM_SDF_WIRE_VERSION);
    put_u32(&mut output, record_count);
    put_u32(&mut output, FERRUM_CHEM_SDF_FLAGS_NONE);

    for record in records {
        encode_record(&mut output, record, version)?;
        if output.len() > FERRUM_CHEM_MAX_RESPONSE_BYTES {
            return Err(ChemistryError::UnsupportedNativeRequest {
                reason: "SDF request exceeds the ABI wire bound".to_owned(),
            });
        }
    }
    Ok(output)
}

fn encode_record(
    output: &mut Vec<u8>,
    record: &SdfRecord,
    version: MolblockVersion,
) -> Result<(), ChemistryError> {
    let molecule = molblock_wire::encode(record.molecule(), version)?;
    let molecule_length = wire_length(molecule.len(), "SDF molecule")?;
    let title_length = wire_length(record.title().len(), "SDF title")?;
    let property_count = u32::try_from(record.properties().len()).map_err(|_| {
        ChemistryError::UnsupportedNativeRequest {
            reason: "SDF property count does not fit the native wire".to_owned(),
        }
    })?;
    let property_bytes = record
        .properties()
        .iter()
        .try_fold(0_usize, |total, property| {
            total
                .checked_add(FERRUM_CHEM_SDF_PROPERTY_HEADER_BYTES)
                .and_then(|length| length.checked_add(property.name().len()))
                .and_then(|length| length.checked_add(property.value().len()))
                .ok_or_else(|| ChemistryError::UnsupportedNativeRequest {
                    reason: "SDF property bytes exceed the native wire bound".to_owned(),
                })
        })?;
    let record_bytes = FERRUM_CHEM_SDF_RECORD_HEADER_BYTES
        .checked_add(molecule.len())
        .and_then(|length| length.checked_add(record.title().len()))
        .and_then(|length| length.checked_add(property_bytes))
        .ok_or_else(|| ChemistryError::UnsupportedNativeRequest {
            reason: "SDF record bytes exceed the native wire bound".to_owned(),
        })?;
    let request_bytes = output.len().checked_add(record_bytes).ok_or_else(|| {
        ChemistryError::UnsupportedNativeRequest {
            reason: "SDF request bytes exceed the native wire bound".to_owned(),
        }
    })?;
    if request_bytes > FERRUM_CHEM_MAX_RESPONSE_BYTES {
        return Err(ChemistryError::UnsupportedNativeRequest {
            reason: "SDF request exceeds the ABI wire bound".to_owned(),
        });
    }
    output.reserve(record_bytes);
    put_u32(output, molecule_length);
    put_u32(output, title_length);
    put_u32(output, property_count);
    put_u32(output, FERRUM_CHEM_SDF_FLAGS_NONE);
    output.extend_from_slice(&molecule);
    output.extend_from_slice(record.title().as_bytes());
    for property in record.properties() {
        put_u32(
            output,
            wire_length(property.name().len(), "SDF property name")?,
        );
        put_u32(
            output,
            wire_length(property.value().len(), "SDF property value")?,
        );
        output.extend_from_slice(property.name().as_bytes());
        output.extend_from_slice(property.value().as_bytes());
    }
    Ok(())
}

fn wire_length(length: usize, field: &str) -> Result<u32, ChemistryError> {
    u32::try_from(length).map_err(|_| ChemistryError::UnsupportedNativeRequest {
        reason: format!("{field} length does not fit the native wire"),
    })
}
