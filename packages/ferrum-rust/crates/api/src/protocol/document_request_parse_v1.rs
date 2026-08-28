//! Shared parsing for protocol fields that identify a document and its revision fence.

use ferrum_document::DocumentObjectIdV1;

use super::execution::ExecutionFailureV1;

pub(crate) fn parse_document_object_id(
    value: &str,
    field: &str,
) -> Result<DocumentObjectIdV1, ExecutionFailureV1> {
    DocumentObjectIdV1::parse(value).map_err(|_| {
        ExecutionFailureV1::invalid_request(format!(
            "{field} is not a durable document object identifier"
        ))
    })
}

pub(crate) fn parse_sha256_hex(value: &str) -> Result<[u8; 32], ExecutionFailureV1> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(ExecutionFailureV1::invalid_request(
            "expected_digest_hex must be a lowercase SHA-256 digest",
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let high = hexadecimal_nibble(pair[0])
            .ok_or_else(|| ExecutionFailureV1::invalid_request("expected_digest_hex is invalid"))?;
        let low = hexadecimal_nibble(pair[1])
            .ok_or_else(|| ExecutionFailureV1::invalid_request("expected_digest_hex is invalid"))?;
        digest[index] = (high << 4) | low;
    }
    Ok(digest)
}

const fn hexadecimal_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use ferrum_document::DocumentObjectIdV1;

    use super::parse_document_object_id;

    #[test]
    fn compact_group_selector_requires_a_durable_document_object_id() {
        let expected = DocumentObjectIdV1::from_entropy_bytes([0x4a; 16]);

        assert_eq!(
            parse_document_object_id(expected.as_str(), "molecule_id")
                .expect("durable selector parses"),
            expected
        );
        assert!(parse_document_object_id("molecule-source-id", "molecule_id").is_err());
    }
}
