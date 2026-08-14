//! Bounded ABI-4 text-response decoding shared by chemistry codecs.

use super::*;

const MAGIC: [u8; 4] = *b"FCT1";

pub(super) fn decode(response: &[u8], codec: &'static str) -> Result<String, ChemistryError> {
    decode_with_line_policy(response, codec, false)
}

pub(super) fn decode_multiline(
    response: &[u8],
    codec: &'static str,
) -> Result<String, ChemistryError> {
    decode_with_line_policy(response, codec, true)
}

fn decode_with_line_policy(
    response: &[u8],
    codec: &'static str,
    multiline: bool,
) -> Result<String, ChemistryError> {
    if response.len() < FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES {
        return Err(ChemistryError::TruncatedNativeResponse);
    }
    let mut reader = Reader::new(response);
    if reader.take(4).map_err(decode_error)? != MAGIC {
        return malformed("FCT1 response magic is invalid");
    }
    if reader.u32().map_err(decode_error)? != FERRUM_CHEM_TEXT_WIRE_VERSION {
        return malformed("FCT1 wire version is unsupported");
    }
    let status = reader.u32().map_err(decode_error)?;
    let detail_length =
        usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    let text_length = usize::try_from(reader.u32().map_err(decode_error)?).expect("u32 fits usize");
    if reader.u32().map_err(decode_error)? != FERRUM_CHEM_TEXT_FLAGS_NONE {
        return malformed("FCT1 flags are nonzero");
    }
    if detail_length > FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES {
        return malformed("FCT1 detail exceeds its ABI limit");
    }
    let declared = detail_length.checked_add(text_length).ok_or_else(|| {
        ChemistryError::MalformedNativeResponse {
            reason: "FCT1 declared text length overflows".to_owned(),
        }
    })?;
    if response.len().saturating_sub(reader.cursor) != declared {
        return malformed("FCT1 text is truncated or trailing");
    }
    let detail = text(
        reader.take(detail_length).map_err(decode_error)?,
        "FCT1 detail",
    )?;
    let output = text(
        reader.take(text_length).map_err(decode_error)?,
        "FCT1 output",
    )?;
    if !reader.is_empty() {
        return Err(ChemistryError::TrailingNativeResponse);
    }
    if !matches!(
        status,
        FERRUM_CHEM_RESULT_OK
            | FERRUM_CHEM_RESULT_MALFORMED_REQUEST
            | FERRUM_CHEM_RESULT_INVALID_MOLECULE
            | FERRUM_CHEM_RESULT_RESOURCE_LIMIT
            | FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE
            | FERRUM_CHEM_RESULT_INTERNAL_FAILURE
    ) {
        return malformed("FCT1 status is unsupported");
    }
    if status != FERRUM_CHEM_RESULT_OK {
        if detail.is_empty() || !output.is_empty() {
            return malformed("failed FCT1 response has invalid text fields");
        }
        return Err(ChemistryError::CodecFailed {
            codec,
            reason: detail.to_owned(),
        });
    }
    if !detail.is_empty() || output.is_empty() {
        return malformed("successful FCT1 response has invalid text fields");
    }
    if !multiline && output.contains(['\r', '\n']) {
        return malformed("FCT1 output contains a line break");
    }
    Ok(output.to_owned())
}

fn text<'a>(bytes: &'a [u8], field: &str) -> Result<&'a str, ChemistryError> {
    if bytes.contains(&0) {
        return malformed(&format!("{field} contains NUL"));
    }
    std::str::from_utf8(bytes).map_err(|_| ChemistryError::MalformedNativeResponse {
        reason: format!("{field} is not UTF-8"),
    })
}

fn malformed<T>(reason: &str) -> Result<T, ChemistryError> {
    Err(ChemistryError::MalformedNativeResponse {
        reason: reason.to_owned(),
    })
}
