//! Bounded FSI1 decoding for ordered SDF import records.

use super::*;

const MAGIC: [u8; 4] = *b"FSI1";

pub(super) fn validate_input(input: &str) -> Result<(), ChemistryError> {
    if input.is_empty() {
        return invalid_input("must not be empty");
    }
    if input.as_bytes().contains(&0) {
        return invalid_input("must not contain NUL bytes");
    }
    if input.len() > FERRUM_CHEM_MAX_RESPONSE_BYTES {
        return invalid_input("exceeds the ABI input bound");
    }
    Ok(())
}

fn invalid_input<T>(reason: &str) -> Result<T, ChemistryError> {
    Err(ChemistryError::InvalidSdfInput {
        reason: reason.to_owned(),
    })
}

pub(super) fn decode(response: &[u8]) -> Result<Vec<ImportedSdfRecord>, ChemistryError> {
    if response.len() < FERRUM_CHEM_SDF_RESPONSE_HEADER_BYTES {
        return Err(ChemistryError::TruncatedNativeResponse);
    }
    let mut reader = Reader::new(response);
    if reader.take(4).map_err(decode_error)? != MAGIC {
        return malformed("FSI1 response magic is invalid");
    }
    if reader.u32().map_err(decode_error)? != FERRUM_CHEM_SDF_WIRE_VERSION {
        return malformed("FSI1 wire version is unsupported");
    }
    let status = reader.u32().map_err(decode_error)?;
    let detail_length =
        usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let record_count =
        usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    if reader.u32().map_err(decode_error)? != FERRUM_CHEM_SDF_FLAGS_NONE {
        return malformed("FSI1 flags are nonzero");
    }
    if detail_length > FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES
        || record_count > FERRUM_CHEM_SDF_MAX_RECORDS as usize
    {
        return malformed("FSI1 declared size exceeds its ABI limit");
    }
    let detail = text(
        reader.take(detail_length).map_err(decode_error)?,
        "FSI1 detail",
    )?;
    if !matches!(
        status,
        FERRUM_CHEM_RESULT_OK
            | FERRUM_CHEM_RESULT_MALFORMED_REQUEST
            | FERRUM_CHEM_RESULT_INVALID_MOLECULE
            | FERRUM_CHEM_RESULT_RESOURCE_LIMIT
            | FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE
            | FERRUM_CHEM_RESULT_INTERNAL_FAILURE
    ) {
        return malformed("FSI1 status is unsupported");
    }
    if status != FERRUM_CHEM_RESULT_OK {
        if detail.is_empty() || record_count != 0 || !reader.is_empty() {
            return malformed("failed FSI1 response contains record data");
        }
        return Err(ChemistryError::CodecFailed {
            codec: "SDF import",
            reason: detail.to_owned(),
        });
    }
    if !detail.is_empty() || record_count == 0 {
        return malformed("successful FSI1 response has invalid detail or record count");
    }
    let mut total_properties = 0_usize;
    let mut records = Vec::with_capacity(record_count);
    for _ in 0..record_count {
        let molecule_length =
            usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
        let title_length =
            usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
        let property_count =
            usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
        if reader.u32().map_err(decode_error)? != FERRUM_CHEM_SDF_FLAGS_NONE {
            return malformed("FSI1 record flags are nonzero");
        }
        total_properties = total_properties
            .checked_add(property_count)
            .ok_or_else(|| malformed_error("FSI1 property count overflows"))?;
        if total_properties > FERRUM_CHEM_SDF_MAX_PROPERTIES as usize {
            return malformed("FSI1 property count exceeds its ABI limit");
        }
        let molecule = fcm1::decode(reader.take(molecule_length).map_err(decode_error)?)?;
        let title = text(
            reader.take(title_length).map_err(decode_error)?,
            "FSI1 title",
        )?;
        if title.contains(['\r', '\n']) {
            return malformed("FSI1 title contains a line break");
        }
        let mut properties = Vec::with_capacity(property_count);
        for _ in 0..property_count {
            let name_length =
                usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
            let value_length =
                usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
            let name = text(
                reader.take(name_length).map_err(decode_error)?,
                "FSI1 property name",
            )?;
            let value = text(
                reader.take(value_length).map_err(decode_error)?,
                "FSI1 property value",
            )?;
            properties.push(SdfProperty::new(name, value).map_err(|error| {
                ChemistryError::MalformedNativeResponse {
                    reason: format!("FSI1 property is invalid: {error}"),
                }
            })?);
        }
        records.push(ImportedSdfRecord::from_native(
            molecule,
            title.to_owned(),
            properties,
        ));
    }
    if !reader.is_empty() {
        return Err(ChemistryError::TrailingNativeResponse);
    }
    Ok(records)
}

fn text<'a>(bytes: &'a [u8], field: &str) -> Result<&'a str, ChemistryError> {
    if bytes.contains(&0) {
        return malformed(&format!("{field} contains NUL"));
    }
    std::str::from_utf8(bytes).map_err(|_| malformed_error(&format!("{field} is not UTF-8")))
}

fn malformed<T>(reason: &str) -> Result<T, ChemistryError> {
    Err(malformed_error(reason))
}

fn malformed_error(reason: &str) -> ChemistryError {
    ChemistryError::MalformedNativeResponse {
        reason: reason.to_owned(),
    }
}
